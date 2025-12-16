use clap::{Parser, Subcommand};

/// ggo - Smart Git Navigation Tool
///
/// Searches through your git branches and checks out the best
/// matching branch based on frecency (frequency + recency).
///
/// EXAMPLES:
///     ggo expo         Checkout best branch matching 'expo' (fuzzy)
///     ggo exo          Matches 'expo-feature-branch' with fuzzy matching
///     ggo feature      Checkout best branch matching 'feature'
///     ggo -            Go back to previous branch (like cd -)
///     ggo -l feat      List all branches matching 'feat' with scores
///     ggo -i FEAT      Case-insensitive match for 'FEAT'
///     ggo --no-fuzzy feat   Use exact substring matching instead of fuzzy
///     ggo --interactive feat   Show interactive menu to select branch
///     ggo --stats      Show usage statistics
///
///     ggo alias m master        Create alias 'm' for branch 'master'
///     ggo alias m               Show what alias 'm' points to
///     ggo alias --list          List all aliases
///     ggo alias --remove m      Remove alias 'm'
///
/// NOTE:
///     ggo learns from your usage patterns. The more you use a branch,
///     the higher it ranks in search results. Fuzzy matching is enabled
///     by default for more forgiving pattern matching.
#[derive(Parser)]
#[command(name = "ggo")]
#[command(disable_version_flag = true)]
#[command(about = "Smart Git Navigation Tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Search pattern to match branch names (use '-' to go back to previous branch)
    pub pattern: Option<String>,

    /// List matching branches without checking out
    #[arg(short, long)]
    pub list: bool,

    /// Case-insensitive pattern matching
    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,

    /// Disable fuzzy matching (use exact substring matching instead)
    #[arg(long = "no-fuzzy")]
    pub no_fuzzy: bool,

    /// Show interactive menu to select from matches
    #[arg(long)]
    pub interactive: bool,

    /// Show usage statistics
    #[arg(long)]
    pub stats: bool,

    /// Print version
    #[arg(short = 'v', short_alias = 'V', long)]
    pub version: bool,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    /// Manage branch aliases
    Alias {
        /// Alias name (not required when using --list)
        #[arg(required_unless_present = "list")]
        alias: Option<String>,

        /// Branch name (if provided, creates/updates alias; if omitted, shows what alias points to)
        branch: Option<String>,

        /// List all aliases for the current repository
        #[arg(short, long)]
        list: bool,

        /// Remove the alias
        #[arg(short, long)]
        remove: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_verify_cli() {
        // This test verifies that the CLI definition is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn test_parse_simple_pattern() {
        let args = vec!["ggo", "feature"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("feature".to_string()));
        assert!(!cli.list);
        assert!(!cli.ignore_case);
        assert!(!cli.no_fuzzy);
        assert!(!cli.interactive);
        assert!(!cli.stats);
    }

    #[test]
    fn test_parse_with_list_flag() {
        let args = vec!["ggo", "-l", "main"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("main".to_string()));
        assert!(cli.list);
    }

    #[test]
    fn test_parse_with_long_list_flag() {
        let args = vec!["ggo", "--list", "develop"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("develop".to_string()));
        assert!(cli.list);
    }

    #[test]
    fn test_parse_with_ignore_case() {
        let args = vec!["ggo", "-i", "FEATURE"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("FEATURE".to_string()));
        assert!(cli.ignore_case);
    }

    #[test]
    fn test_parse_with_long_ignore_case() {
        let args = vec!["ggo", "--ignore-case", "TEST"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("TEST".to_string()));
        assert!(cli.ignore_case);
    }

    #[test]
    fn test_parse_with_no_fuzzy() {
        let args = vec!["ggo", "--no-fuzzy", "main"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("main".to_string()));
        assert!(cli.no_fuzzy);
    }

    #[test]
    fn test_parse_with_interactive() {
        let args = vec!["ggo", "--interactive", "feature"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("feature".to_string()));
        assert!(cli.interactive);
    }

    #[test]
    fn test_parse_stats_only() {
        let args = vec!["ggo", "--stats"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, None);
        assert!(cli.stats);
    }

    #[test]
    fn test_parse_multiple_flags() {
        let args = vec!["ggo", "-l", "-i", "--no-fuzzy", "test"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("test".to_string()));
        assert!(cli.list);
        assert!(cli.ignore_case);
        assert!(cli.no_fuzzy);
    }

    #[test]
    fn test_parse_all_flags() {
        let args = vec!["ggo", "-l", "-i", "--no-fuzzy", "--interactive", "branch"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("branch".to_string()));
        assert!(cli.list);
        assert!(cli.ignore_case);
        assert!(cli.no_fuzzy);
        assert!(cli.interactive);
        assert!(!cli.stats);
    }

    #[test]
    fn test_parse_dash_pattern() {
        let args = vec!["ggo", "-"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("-".to_string()));
    }

    #[test]
    fn test_parse_empty_pattern() {
        let args = vec!["ggo", ""];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("".to_string()));
    }

    #[test]
    fn test_parse_pattern_with_special_chars() {
        let args = vec!["ggo", "feature/auth-v2"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("feature/auth-v2".to_string()));
    }

    #[test]
    fn test_parse_pattern_with_spaces() {
        let args = vec!["ggo", "feature branch"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("feature branch".to_string()));
    }

    #[test]
    fn test_parse_unicode_pattern() {
        let args = vec!["ggo", "日本語"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("日本語".to_string()));
    }

    #[test]
    fn test_parse_list_before_pattern() {
        let args = vec!["ggo", "-l", "test"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("test".to_string()));
        assert!(cli.list);
    }

    #[test]
    fn test_parse_list_after_pattern() {
        let args = vec!["ggo", "test", "-l"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("test".to_string()));
        assert!(cli.list);
    }

    #[test]
    fn test_parse_flags_order_independent() {
        let args1 = vec!["ggo", "-l", "-i", "test"];
        let cli1 = Cli::parse_from(args1);

        let args2 = vec!["ggo", "-i", "-l", "test"];
        let cli2 = Cli::parse_from(args2);

        assert_eq!(cli1.pattern, cli2.pattern);
        assert_eq!(cli1.list, cli2.list);
        assert_eq!(cli1.ignore_case, cli2.ignore_case);
    }

    #[test]
    fn test_parse_no_args_without_stats() {
        // Pattern is optional now (for subcommands), so parse succeeds
        // but main() will handle the error if no command/stats/pattern provided
        let args = vec!["ggo"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_ok());
        let cli = result.unwrap();
        assert_eq!(cli.pattern, None);
        assert_eq!(cli.command, None);
        assert!(!cli.stats);
    }

    #[test]
    fn test_parse_stats_with_pattern() {
        let args = vec!["ggo", "--stats", "test"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some("test".to_string()));
        assert!(cli.stats);
    }

    #[test]
    fn test_parse_combined_short_flags() {
        // Note: clap doesn't support combining -l and -i as -li
        // They need to be separate
        let args = vec!["ggo", "-l", "-i", "test"];
        let cli = Cli::parse_from(args);

        assert!(cli.list);
        assert!(cli.ignore_case);
    }

    #[test]
    fn test_parse_long_pattern() {
        let long_pattern = "a".repeat(1000);
        let args = vec!["ggo", &long_pattern];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.pattern, Some(long_pattern));
    }

    #[test]
    fn test_default_values() {
        let args = vec!["ggo", "test"];
        let cli = Cli::parse_from(args);

        // Check default values
        assert!(!cli.list);
        assert!(!cli.ignore_case);
        assert!(!cli.no_fuzzy);
        assert!(!cli.interactive);
        assert!(!cli.stats);
    }

    #[test]
    fn test_parse_help_contains_description() {
        let mut cmd = Cli::command();
        let help = cmd.render_help().to_string();

        assert!(help.contains("Smart Git Navigation Tool"));
    }

    #[test]
    fn test_parse_help_contains_all_options() {
        let mut cmd = Cli::command();
        let help = cmd.render_help().to_string();

        assert!(help.contains("--list") || help.contains("-l"));
        assert!(help.contains("--ignore-case") || help.contains("-i"));
        assert!(help.contains("--no-fuzzy"));
        assert!(help.contains("--interactive"));
        assert!(help.contains("--stats"));
    }
}
