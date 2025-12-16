use anyhow::{bail, Context, Result};
use std::io::BufRead;
use std::process::Command;

use crate::validation;

/// Get all local git branches in the current repository
pub fn get_branches() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["branch"])
        .output()
        .context("Failed to execute git branch")?;

    if !output.status.success() {
        bail!("Not a git repository or git command failed");
    }

    let branches: Vec<String> = output
        .stdout
        .lines()
        .map_while(Result::ok)
        .map(|line| line.trim().trim_start_matches('*').trim().to_string())
        .collect();

    Ok(branches)
}

/// Checkout the specified branch
pub fn checkout(branch: &str) -> Result<()> {
    // Validate branch name before attempting checkout
    validation::validate_branch_name(branch)
        .context("Cannot checkout invalid branch name")?;

    let output = Command::new("git")
        .args(["checkout", branch])
        .output()
        .context("Failed to execute git checkout")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        bail!("Git checkout failed: {}", error.trim());
    }

    Ok(())
}

/// Get the root path of the current git repository
pub fn get_repo_root() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to execute git rev-parse")?;

    if !output.status.success() {
        bail!("Not a git repository");
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Validate the returned repo path
    validation::validate_repo_path(&path)
        .context("Git returned invalid repository path")?;

    Ok(path)
}

/// Get the name of the current branch
pub fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to execute git branch --show-current")?;

    if !output.status.success() {
        bail!("Failed to get current branch (detached HEAD?)");
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if branch.is_empty() {
        bail!("Not on a branch (detached HEAD)");
    }

    Ok(branch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

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

    // Helper to run get_branches in a specific directory
    fn get_branches_in_dir(dir: &Path) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(["branch"])
            .current_dir(dir)
            .output()
            .context("Failed to execute git branch")?;

        if !output.status.success() {
            bail!("Not a git repository or git command failed");
        }

        let branches: Vec<String> = output
            .stdout
            .lines()
            .map_while(Result::ok)
            .map(|line| line.trim().trim_start_matches('*').trim().to_string())
            .collect();

        Ok(branches)
    }

    #[test]
    fn test_get_branches_empty_repo() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");

        let result = get_branches_in_dir(temp_dir.path());

        assert!(result.is_ok());
        let branches = result.unwrap();
        // Should have at least the default branch (main or master)
        assert!(!branches.is_empty());
        assert!(branches.iter().any(|b| b == "main" || b == "master"));
    }

    #[test]
    fn test_get_branches_multiple() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let repo_path = temp_dir.path();

        // Create additional branches
        Command::new("git")
            .args(["branch", "feature-a"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["branch", "feature-b"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let result = get_branches_in_dir(repo_path);

        assert!(result.is_ok());
        let branches = result.unwrap();
        assert!(branches.len() >= 3);
        assert!(branches.contains(&"feature-a".to_string()));
        assert!(branches.contains(&"feature-b".to_string()));
    }

    #[test]
    fn test_get_branches_strips_asterisk() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let repo_path = temp_dir.path();

        // Create and checkout a new branch
        Command::new("git")
            .args(["checkout", "-b", "test-branch"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let result = get_branches_in_dir(repo_path);

        assert!(result.is_ok());
        let branches = result.unwrap();
        // Ensure no branch has asterisk
        for branch in &branches {
            assert!(!branch.starts_with('*'));
            assert!(!branch.contains('*'));
        }
        assert!(branches.contains(&"test-branch".to_string()));
    }

    #[test]
    fn test_get_branches_not_git_repo() {
        let temp_dir = tempfile::tempdir().unwrap();

        let result = get_branches_in_dir(temp_dir.path());

        assert!(result.is_err());
    }

    // Helper to checkout in a specific directory
    fn checkout_in_dir(dir: &Path, branch: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["checkout", branch])
            .current_dir(dir)
            .output()
            .context("Failed to execute git checkout")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            bail!("Git checkout failed: {}", error.trim());
        }

        Ok(())
    }

    #[test]
    fn test_checkout_existing_branch() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let repo_path = temp_dir.path();

        // Create a new branch
        Command::new("git")
            .args(["branch", "test-checkout"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let result = checkout_in_dir(repo_path, "test-checkout");

        assert!(result.is_ok());
    }

    #[test]
    fn test_checkout_nonexistent_branch() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");

        let result = checkout_in_dir(temp_dir.path(), "nonexistent-branch");

        assert!(result.is_err());
    }

    // Helper to get repo root from a specific directory
    fn get_repo_root_in_dir(dir: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(dir)
            .output()
            .context("Failed to execute git rev-parse")?;

        if !output.status.success() {
            bail!("Not a git repository");
        }

        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(path)
    }

    // Helper to get current branch in a specific directory
    fn get_current_branch_in_dir(dir: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(dir)
            .output()
            .context("Failed to execute git branch --show-current")?;

        if !output.status.success() {
            bail!("Failed to get current branch (detached HEAD?)");
        }

        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if branch.is_empty() {
            bail!("Not on a branch (detached HEAD)");
        }

        Ok(branch)
    }

    #[test]
    fn test_get_repo_root() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let repo_path = temp_dir.path();

        // Create a subdirectory
        let subdir = repo_path.join("subdir");
        fs::create_dir(&subdir).unwrap();

        let result = get_repo_root_in_dir(&subdir);

        assert!(result.is_ok());
        let root = result.unwrap();
        // Should return the repo root, not the subdirectory
        assert_eq!(Path::new(&root).file_name(), repo_path.file_name());
    }

    #[test]
    fn test_get_repo_root_not_git_repo() {
        let temp_dir = tempfile::tempdir().unwrap();

        let result = get_repo_root_in_dir(temp_dir.path());

        assert!(result.is_err());
    }

    #[test]
    fn test_get_current_branch() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let repo_path = temp_dir.path();

        // Create and checkout a new branch
        Command::new("git")
            .args(["checkout", "-b", "current-test"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let result = get_current_branch_in_dir(repo_path);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "current-test");
    }

    #[test]
    fn test_get_current_branch_not_git_repo() {
        let temp_dir = tempfile::tempdir().unwrap();

        let result = get_current_branch_in_dir(temp_dir.path());

        assert!(result.is_err());
    }

    #[test]
    fn test_get_current_branch_detached_head() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let repo_path = temp_dir.path();

        // Get the commit hash
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let commit_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Checkout the commit directly (detached HEAD)
        Command::new("git")
            .args(["checkout", &commit_hash])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let result = get_current_branch_in_dir(repo_path);

        // Should fail because we're in detached HEAD state
        assert!(result.is_err());
    }
}
