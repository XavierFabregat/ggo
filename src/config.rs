use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for ggo behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub frecency: FrecencyConfig,

    #[serde(default)]
    pub behavior: BehaviorConfig,
}

/// Frecency algorithm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrecencyConfig {
    /// Half-life in days for exponential decay (default: 7 days)
    /// After this duration, a branch's recency weight is halved
    #[serde(default = "default_half_life_days")]
    pub half_life_days: f64,
}

/// Behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// Threshold for auto-selecting a branch (score ratio)
    #[serde(default = "default_auto_select_threshold")]
    pub auto_select_threshold: f64,

    /// Enable fuzzy matching by default
    #[serde(default = "default_fuzzy")]
    pub default_fuzzy: bool,

    /// Case-insensitive matching by default
    #[serde(default)]
    pub default_ignore_case: bool,
}

// Default value functions
fn default_half_life_days() -> f64 {
    7.0 // 1 week
}
fn default_auto_select_threshold() -> f64 {
    2.0
}
fn default_fuzzy() -> bool {
    true
}

impl Default for FrecencyConfig {
    fn default() -> Self {
        Self {
            half_life_days: default_half_life_days(),
        }
    }
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            auto_select_threshold: default_auto_select_threshold(),
            default_fuzzy: default_fuzzy(),
            default_ignore_case: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frecency: FrecencyConfig::default(),
            behavior: BehaviorConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from file, or use defaults if file doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)
            .context("Failed to read configuration file")?;

        let config: Config = toml::from_str(&content)
            .context("Failed to parse configuration file")?;

        Ok(config)
    }

    /// Get the path to the config file
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("ggo");

        std::fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        Ok(config_dir.join("config.toml"))
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;

        std::fs::write(&config_path, content)
            .context("Failed to write configuration file")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let config = Config::default();

        assert_eq!(config.frecency.half_life_days, 7.0);
        assert_eq!(config.behavior.auto_select_threshold, 2.0);
        assert!(config.behavior.default_fuzzy);
        assert!(!config.behavior.default_ignore_case);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).expect("Failed to serialize");

        assert!(toml_str.contains("half_life_days"));
        assert!(toml_str.contains("7.0"));
        assert!(toml_str.contains("auto_select_threshold"));
        assert!(toml_str.contains("2.0"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            [frecency]
            half_life_days = 14.0

            [behavior]
            auto_select_threshold = 1.5
            default_fuzzy = false
        "#;

        let config: Config = toml::from_str(toml_str).expect("Failed to parse");

        assert_eq!(config.frecency.half_life_days, 14.0);
        assert_eq!(config.behavior.auto_select_threshold, 1.5);
        assert!(!config.behavior.default_fuzzy);
    }

    #[test]
    fn test_partial_config() {
        let toml_str = r#"
            [frecency]
            half_life_days = 3.5
        "#;

        let config: Config = toml::from_str(toml_str).expect("Failed to parse");

        assert_eq!(config.frecency.half_life_days, 3.5);
        // Other values should use defaults
        assert_eq!(config.behavior.auto_select_threshold, 2.0);
        assert!(config.behavior.default_fuzzy);
    }

    #[test]
    fn test_empty_config_uses_defaults() {
        let toml_str = "";

        let config: Config = toml::from_str(toml_str).expect("Failed to parse");

        assert_eq!(config.frecency.half_life_days, 7.0);
        assert_eq!(config.behavior.auto_select_threshold, 2.0);
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_dir = temp_dir.path().join(".config/ggo");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.toml");

        let mut config = Config::default();
        config.frecency.half_life_days = 14.0;
        config.behavior.auto_select_threshold = 3.0;

        // Save manually for testing
        let content = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, content).unwrap();

        // Load manually for testing
        let loaded_content = std::fs::read_to_string(&config_path).unwrap();
        let loaded: Config = toml::from_str(&loaded_content).unwrap();

        assert_eq!(loaded.frecency.half_life_days, 14.0);
        assert_eq!(loaded.behavior.auto_select_threshold, 3.0);
    }

    #[test]
    fn test_invalid_config_returns_error() {
        let toml_str = r#"
            [frecency]
            half_life_days = "not a number"
        "#;

        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }
}
