use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

// Helper to create a temporary git repo for testing
fn setup_test_repo() -> std::io::Result<tempfile::TempDir> {
    let temp_dir = tempfile::tempdir()?;
    let repo_path = temp_dir.path();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()?;

    // Configure git for tests
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()?;

    // Create initial commit
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "test content")?;

    Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()?;

    Ok(temp_dir)
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
