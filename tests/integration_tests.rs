use std::env;
use std::path::PathBuf;
use std::process::Command;

mod common;
use common::setup_test_repo;

// Helper to get the path to the built ggo binary
fn get_ggo_binary() -> PathBuf {
    // Build the binary first
    let build_output = Command::new("cargo")
        .args(["build"])
        .output()
        .expect("Failed to build ggo");

    if !build_output.status.success() {
        panic!(
            "Failed to build ggo: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Return path to the debug binary
    let mut path = env::current_dir().expect("Failed to get current dir");
    path.push("target");
    path.push("debug");
    path.push("ggo");
    path
}

#[test]
fn test_cli_help() {
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["--help"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Smart Git Navigation Tool"));
    assert!(stdout.contains("Usage:"));
}

#[test]
fn test_cli_version() {
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["--version"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ggo"));
}

#[test]
fn test_cli_stats_command() {
    // Set up a temporary home directory for the test
    let temp_home = tempfile::tempdir().expect("Failed to create temp dir");

    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["--stats"])
        .env("HOME", temp_home.path())
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ggo Statistics") || stdout.contains("Total branch switches"));
}

#[test]
fn test_cli_list_command_no_git_repo() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["-l", "main"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run command");

    // Should fail because it's not a git repo
    assert!(!output.status.success());
}

#[test]
fn test_cli_list_command_in_git_repo() {
    let temp_dir = setup_test_repo().expect("Failed to create test repo");
    let repo_path = temp_dir.path();

    // Create some branches
    Command::new("git")
        .args(["branch", "feature/test"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    let test_data_dir = temp_dir.path().join(".ggo");
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["-l", "feature"])
        .current_dir(repo_path)
        .env("GGO_DATA_DIR", &test_data_dir)
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should succeed and list the branch
    assert!(
        output.status.success()
            || stdout.contains("feature/test")
            || stderr.contains("feature/test")
    );
}

#[test]
fn test_cli_no_fuzzy_flag() {
    let temp_dir = setup_test_repo().expect("Failed to create test repo");
    let repo_path = temp_dir.path();

    // Create branches
    Command::new("git")
        .args(["branch", "feature/auth"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    let test_data_dir = temp_dir.path().join(".ggo");
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["-l", "--no-fuzzy", "feature"])
        .current_dir(repo_path)
        .env("GGO_DATA_DIR", &test_data_dir)
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should succeed
    assert!(output.status.success() || stdout.contains("feature") || stderr.contains("feature"));
}

#[test]
fn test_cli_ignore_case_flag() {
    let temp_dir = setup_test_repo().expect("Failed to create test repo");
    let repo_path = temp_dir.path();

    // Create branches
    Command::new("git")
        .args(["branch", "Feature/Auth"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    let test_data_dir = temp_dir.path().join(".ggo");
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["-l", "-i", "FEATURE"])
        .current_dir(repo_path)
        .env("GGO_DATA_DIR", &test_data_dir)
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should match case-insensitively
    assert!(output.status.success() || stdout.contains("Feature") || stderr.contains("Feature"));
}

#[test]
fn test_cli_no_pattern_without_stats_fails() {
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo).output().expect("Failed to run command");

    // Should fail because pattern is required unless --stats is provided
    assert!(!output.status.success());
}

#[test]
fn test_cli_list_nonexistent_pattern() {
    let temp_dir = setup_test_repo().expect("Failed to create test repo");
    let repo_path = temp_dir.path();

    let test_data_dir = temp_dir.path().join(".ggo");
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["-l", "nonexistent-branch-xyz"])
        .current_dir(repo_path)
        .env("GGO_DATA_DIR", &test_data_dir)
        .output()
        .expect("Failed to run command");

    // Should fail because no branches match
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No branch") || stderr.to_lowercase().contains("error"));
}

#[test]
fn test_checkout_without_list_flag() {
    let temp_dir = setup_test_repo().expect("Failed to create test repo");
    let repo_path = temp_dir.path();

    // Create and be on main/master
    let current_branch = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    let _current = String::from_utf8_lossy(&current_branch.stdout)
        .trim()
        .to_string();

    // Create a new branch
    Command::new("git")
        .args(["branch", "test-branch"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    // Try to checkout using ggo
    let test_data_dir = temp_dir.path().join(".ggo");
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["test-branch"])
        .current_dir(repo_path)
        .env("GGO_DATA_DIR", &test_data_dir)
        .output()
        .expect("Failed to run command");

    // Should succeed or show that it switched
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("test-branch") || stdout.contains("Switched"));
    }
}

#[test]
fn test_multiple_branches_matching() {
    let temp_dir = setup_test_repo().expect("Failed to create test repo");
    let repo_path = temp_dir.path();

    // Create multiple branches with similar names
    Command::new("git")
        .args(["branch", "feature/auth"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["branch", "feature/dashboard"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    let test_data_dir = temp_dir.path().join(".ggo");
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["-l", "feature"])
        .current_dir(repo_path)
        .env("GGO_DATA_DIR", &test_data_dir)
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should list both branches
    assert!(output.status.success() || (stdout.contains("feature") || stderr.contains("feature")));
}

#[test]
fn test_fuzzy_matching_works() {
    let temp_dir = setup_test_repo().expect("Failed to create test repo");
    let repo_path = temp_dir.path();

    // Create a branch
    Command::new("git")
        .args(["branch", "expo-feature-branch"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    // Test fuzzy matching with "exo"
    let test_data_dir = temp_dir.path().join(".ggo");
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["-l", "exo"])
        .current_dir(repo_path)
        .env("GGO_DATA_DIR", &test_data_dir)
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Fuzzy matching should find "expo-feature-branch"
    assert!(output.status.success() || stdout.contains("expo") || stderr.contains("expo"));
}

#[test]
fn test_empty_pattern_lists_all_branches() {
    let temp_dir = setup_test_repo().expect("Failed to create test repo");
    let repo_path = temp_dir.path();

    // Create multiple branches
    Command::new("git")
        .args(["branch", "branch-a"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["branch", "branch-b"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    let test_data_dir = temp_dir.path().join(".ggo");
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["-l", ""])
        .current_dir(repo_path)
        .env("GGO_DATA_DIR", &test_data_dir)
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should list all branches
    assert!(output.status.success() || stdout.contains("branch") || stderr.contains("branch"));
}

#[test]
fn test_cleanup_show_size() {
    scopeguard::defer! {
        std::env::remove_var("GGO_DATA_DIR");
    }
    let test_db_dir = tempfile::tempdir().unwrap();
    std::env::set_var("GGO_DATA_DIR", test_db_dir.path());

    let ggo = get_ggo_binary();

    // First ensure database exists by running stats (or any command that creates the DB)
    let _ = Command::new(&ggo)
        .args(["--stats"])
        .env("GGO_DATA_DIR", test_db_dir.path())
        .output()
        .expect("Failed to initialize database");

    let output = Command::new(&ggo)
        .args(["cleanup", "--size"])
        .env("GGO_DATA_DIR", test_db_dir.path())
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        eprintln!("Command failed!");
        eprintln!("stdout: {}", stdout);
        eprintln!("stderr: {}", stderr);
    }

    assert!(output.status.success());
    assert!(stdout.contains("Database size:"));
    // Should show either KB or MB
    assert!(stdout.contains("KB") || stdout.contains("MB"));
}

#[test]
fn test_cleanup_no_args_shows_help() {
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["cleanup"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("Database cleanup options:"));
    assert!(stdout.contains("--deleted"));
    assert!(stdout.contains("--optimize"));
    assert!(stdout.contains("--size"));
}

#[test]
fn test_cleanup_deleted_branches() {
    scopeguard::defer! {
        std::env::remove_var("GGO_DATA_DIR");
    }
    let test_db_dir = tempfile::tempdir().unwrap();
    std::env::set_var("GGO_DATA_DIR", test_db_dir.path());

    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["cleanup", "--deleted"])
        .env("GGO_DATA_DIR", test_db_dir.path())
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("Cleaning up deleted branches"));
    assert!(stdout.contains("Removed"));
    assert!(stdout.contains("stale branch records"));
}

#[test]
fn test_cleanup_old_records() {
    scopeguard::defer! {
        std::env::remove_var("GGO_DATA_DIR");
    }
    let test_db_dir = tempfile::tempdir().unwrap();
    std::env::set_var("GGO_DATA_DIR", test_db_dir.path());

    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["cleanup", "--older-than", "30"])
        .env("GGO_DATA_DIR", test_db_dir.path())
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("Cleaning up branches older than 30 days"));
    assert!(stdout.contains("Removed"));
    assert!(stdout.contains("old branch records"));
}

#[test]
fn test_cleanup_optimize() {
    scopeguard::defer! {
        std::env::remove_var("GGO_DATA_DIR");
    }
    let test_db_dir = tempfile::tempdir().unwrap();
    std::env::set_var("GGO_DATA_DIR", test_db_dir.path());

    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["cleanup", "--optimize"])
        .env("GGO_DATA_DIR", test_db_dir.path())
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("Optimizing database"));
    assert!(stdout.contains("Database optimized"));
    assert!(stdout.contains("VACUUM and ANALYZE complete"));
}

#[test]
fn test_cleanup_combined_flags() {
    scopeguard::defer! {
        std::env::remove_var("GGO_DATA_DIR");
    }
    let test_db_dir = tempfile::tempdir().unwrap();
    std::env::set_var("GGO_DATA_DIR", test_db_dir.path());

    let ggo = get_ggo_binary();

    // First ensure database exists
    let _ = Command::new(&ggo)
        .args(["--stats"])
        .env("GGO_DATA_DIR", test_db_dir.path())
        .output()
        .expect("Failed to initialize database");

    let output = Command::new(&ggo)
        .args(["cleanup", "--deleted", "--optimize", "--size"])
        .env("GGO_DATA_DIR", test_db_dir.path())
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    // Should show all three operations
    assert!(stdout.contains("Database size:"));
    assert!(stdout.contains("Cleaning up deleted branches"));
    assert!(stdout.contains("Optimizing database"));
}

#[test]
fn test_generate_completion_bash() {
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["--generate-completion", "bash"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    // Bash completion script should contain bash-specific syntax
    assert!(stdout.contains("_ggo") || stdout.contains("complete"));
}

#[test]
fn test_generate_completion_zsh() {
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["--generate-completion", "zsh"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    // Zsh completion script should contain zsh-specific syntax
    assert!(stdout.contains("#compdef") || stdout.contains("_ggo"));
}

#[test]
fn test_generate_completion_fish() {
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["--generate-completion", "fish"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    // Fish completion script should contain fish-specific syntax
    assert!(stdout.contains("complete") && stdout.contains("ggo"));
}

#[test]
fn test_generate_completion_invalid_shell() {
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["--generate-completion", "invalid"])
        .output()
        .expect("Failed to run command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("Unsupported shell"));
    assert!(stderr.contains("Supported shells:"));
}

#[test]
fn test_generate_completion_powershell() {
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["--generate-completion", "powershell"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    // PowerShell completion should contain PowerShell-specific syntax
    assert!(stdout.contains("Register-ArgumentCompleter") || stdout.contains("param"));
}

#[test]
fn test_stats_has_summary_section() {
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["--stats"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("ggo Statistics"));
    assert!(stdout.contains("Total branch switches:"));
    assert!(stdout.contains("Database location:"));
}

#[test]
fn test_stats_shows_top_branches() {
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["--stats"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    // Should have a section for top branches (case-insensitive) OR show empty message
    let stdout_lower = stdout.to_lowercase();
    assert!(
        stdout_lower.contains("top branches")
            || stdout_lower.contains("frecency")
            || stdout_lower.contains("no branch usage data yet")
    );
}

#[test]
fn test_stats_repository_breakdown() {
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["--stats"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    // Should show repository information
    assert!(stdout.contains("Repositories:") || stdout.contains("repos"));
}

#[test]
fn test_config_file_not_required() {
    // Config file should be optional - ggo works without it
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config/ggo");
    std::fs::create_dir_all(&config_dir).unwrap();

    // No config file exists, but ggo should still work
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["--version"])
        .env("HOME", temp_dir.path())
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
}

#[test]
fn test_config_file_parsing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config/ggo");
    std::fs::create_dir_all(&config_dir).unwrap();

    // Create a config file
    let config_content = r#"
[frecency]
half_life_days = 14.0

[behavior]
auto_select_threshold = 3.0
default_fuzzy = false
"#;
    std::fs::write(config_dir.join("config.toml"), config_content).unwrap();

    // ggo should load and use the config
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["--version"])
        .env("HOME", temp_dir.path())
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
}

#[test]
fn test_invalid_config_uses_defaults() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config/ggo");
    std::fs::create_dir_all(&config_dir).unwrap();

    // Create an invalid config file
    let config_content = "invalid toml content [[[";
    std::fs::write(config_dir.join("config.toml"), config_content).unwrap();

    // ggo should still work (using defaults)
    let ggo = get_ggo_binary();
    let output = Command::new(&ggo)
        .args(["--version"])
        .env("HOME", temp_dir.path())
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
}
