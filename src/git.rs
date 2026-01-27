use git2::Repository;

use crate::error::{GgoError, Result};
use crate::validation;

/// Get all local git branches in the current repository
pub fn get_branches() -> Result<Vec<String>> {
    let repo = Repository::open_from_env().map_err(|_| GgoError::NotGitRepository)?;

    let mut branches = Vec::new();

    for branch in repo.branches(Some(git2::BranchType::Local))? {
        let (branch, _) = branch?;
        if let Some(name) = branch.name()? {
            branches.push(name.to_string());
        }
    }

    Ok(branches)
}

/// Checkout the specified branch
pub fn checkout(branch: &str) -> Result<()> {
    // Validate branch name before attempting checkout
    validation::validate_branch_name(branch)?;

    let repo = Repository::open_from_env().map_err(|_| GgoError::NotGitRepository)?;

    // Find the branch reference
    let refname = format!("refs/heads/{}", branch);
    let obj = repo
        .revparse_single(&refname)
        .map_err(|_| GgoError::BranchNotFound(branch.to_string()))?;

    // Checkout the branch
    repo.checkout_tree(&obj, None)
        .map_err(|e| GgoError::CheckoutFailed(branch.to_string(), e.to_string()))?;

    // Update HEAD to point to the branch
    repo.set_head(&refname)
        .map_err(|e| GgoError::CheckoutFailed(branch.to_string(), e.to_string()))?;

    Ok(())
}

/// Get the root path of the current git repository
pub fn get_repo_root() -> Result<String> {
    let repo = Repository::open_from_env().map_err(|_| GgoError::NotGitRepository)?;

    let workdir = repo
        .workdir()
        .ok_or_else(|| GgoError::Other("Repository has no working directory (bare repository?)".to_string()))?;

    let path = workdir
        .to_str()
        .ok_or_else(|| GgoError::Other("Repository path contains invalid UTF-8".to_string()))?
        .to_string();

    // Validate the returned repo path
    validation::validate_repo_path(&path)?;

    Ok(path)
}

/// Get the name of the current branch
pub fn get_current_branch() -> Result<String> {
    let repo = Repository::open_from_env().map_err(|_| GgoError::NotGitRepository)?;

    let head = repo.head().map_err(|_| GgoError::NotGitRepository)?;

    if !head.is_branch() {
        return Err(GgoError::Other("Not on a branch (detached HEAD)".to_string()));
    }

    let branch_name = head
        .shorthand()
        .ok_or_else(|| GgoError::Other("Invalid branch name".to_string()))?;

    Ok(branch_name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use std::fs;
    use std::path::Path;

    // Helper to create a temporary git repo for testing
    fn setup_test_repo() -> std::io::Result<tempfile::TempDir> {
        let temp_dir = tempfile::tempdir()?;
        let repo_path = temp_dir.path();

        // Initialize git repo using git2
        Repository::init(repo_path).unwrap();
        let repo = Repository::open(repo_path).unwrap();

        // Configure git for tests
        repo.config()
            .unwrap()
            .set_str("user.email", "test@example.com")
            .unwrap();
        repo.config()
            .unwrap()
            .set_str("user.name", "Test User")
            .unwrap();

        // Create initial commit
        let test_file = repo_path.join("test.txt");
        fs::write(&test_file, "test content")?;

        let mut index = repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = repo.signature().unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();

        Ok(temp_dir)
    }

    // Helper to get branches from a specific repo path
    fn get_branches_from_path(path: &Path) -> anyhow::Result<Vec<String>> {
        let repo = Repository::open(path).context("Not a git repository")?;

        let mut branches = Vec::new();

        for branch in repo.branches(Some(git2::BranchType::Local))? {
            let (branch, _) = branch.context("Failed to read branch")?;
            if let Some(name) = branch.name()? {
                branches.push(name.to_string());
            }
        }

        Ok(branches)
    }

    #[test]
    fn test_get_branches_empty_repo() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let result = get_branches_from_path(temp_dir.path());

        assert!(result.is_ok());
        let branches = result.unwrap();
        // Should have at least the default branch (usually 'master' or 'main')
        assert!(!branches.is_empty());
    }

    #[test]
    fn test_get_branches_multiple() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let repo = Repository::open(temp_dir.path()).unwrap();

        // Create additional branches
        let head = repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();

        repo.branch("feature-a", &commit, false).unwrap();
        repo.branch("feature-b", &commit, false).unwrap();

        let result = get_branches_from_path(temp_dir.path());

        assert!(result.is_ok());
        let branches = result.unwrap();
        assert!(branches.len() >= 3);
        assert!(branches.contains(&"feature-a".to_string()));
        assert!(branches.contains(&"feature-b".to_string()));
    }

    #[test]
    fn test_get_branches_strips_asterisk() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let result = get_branches_from_path(temp_dir.path());

        assert!(result.is_ok());
        let branches = result.unwrap();
        // Ensure no branch has asterisk (git2 doesn't add them)
        for branch in &branches {
            assert!(!branch.starts_with('*'));
            assert!(!branch.contains('*'));
        }
    }

    #[test]
    fn test_get_branches_not_git_repo() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = get_branches_from_path(temp_dir.path());

        assert!(result.is_err());
    }

    // Helper to checkout in a specific repo
    fn checkout_in_repo(path: &Path, branch: &str) -> anyhow::Result<()> {
        validation::validate_branch_name(branch).context("Cannot checkout invalid branch name")?;

        let repo = Repository::open(path).context("Not a git repository")?;

        let refname = format!("refs/heads/{}", branch);
        let obj = repo
            .revparse_single(&refname)
            .context(format!("Branch '{}' not found", branch))?;

        repo.checkout_tree(&obj, None)
            .context(format!("Failed to checkout branch '{}'", branch))?;

        repo.set_head(&refname)
            .context(format!("Failed to set HEAD to branch '{}'", branch))?;

        Ok(())
    }

    #[test]
    fn test_checkout_existing_branch() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let repo = Repository::open(temp_dir.path()).unwrap();

        // Create a new branch
        let head = repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();
        repo.branch("test-checkout", &commit, false).unwrap();

        let result = checkout_in_repo(temp_dir.path(), "test-checkout");

        assert!(result.is_ok());

        // Verify we're on the new branch
        let current_head = repo.head().unwrap();
        assert!(current_head.is_branch());
        assert_eq!(current_head.shorthand().unwrap(), "test-checkout");
    }

    #[test]
    fn test_checkout_nonexistent_branch() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let result = checkout_in_repo(temp_dir.path(), "nonexistent-branch");

        assert!(result.is_err());
    }

    // Helper to discover repo root from a subdirectory
    fn get_repo_root_from_path(path: &Path) -> anyhow::Result<String> {
        let repo = Repository::discover(path).context("Not a git repository")?;

        let workdir = repo.workdir().ok_or_else(|| {
            anyhow::anyhow!("Repository has no working directory (bare repository?)")
        })?;

        let root_path = workdir
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Repository path contains invalid UTF-8"))?
            .to_string();

        validation::validate_repo_path(&root_path)
            .context("Git returned invalid repository path")?;

        Ok(root_path)
    }

    #[test]
    fn test_get_repo_root() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let repo_path = temp_dir.path();

        // Create a subdirectory
        let subdir = repo_path.join("subdir");
        fs::create_dir(&subdir).unwrap();

        let result = get_repo_root_from_path(&subdir);

        assert!(result.is_ok());
        let root = result.unwrap();

        // Should return the repo root, not the subdirectory
        // Normalize paths for comparison
        let expected = repo_path.canonicalize().unwrap();
        let actual = Path::new(&root).canonicalize().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_repo_root_not_git_repo() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = get_repo_root_from_path(temp_dir.path());

        assert!(result.is_err());
    }

    // Helper to get current branch from a specific repo
    fn get_current_branch_from_repo(path: &Path) -> anyhow::Result<String> {
        use anyhow::bail;

        let repo = Repository::open(path).context("Not a git repository")?;

        let head = repo.head().context("Could not get HEAD reference")?;

        if !head.is_branch() {
            bail!("Not on a branch (detached HEAD)");
        }

        let branch_name = head
            .shorthand()
            .ok_or_else(|| anyhow::anyhow!("Invalid branch name"))?;

        Ok(branch_name.to_string())
    }

    #[test]
    fn test_get_current_branch() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let repo = Repository::open(temp_dir.path()).unwrap();

        // Create and checkout a new branch
        let head = repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();
        let branch = repo.branch("current-test", &commit, false).unwrap();
        repo.set_head(branch.get().name().unwrap()).unwrap();

        let result = get_current_branch_from_repo(temp_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "current-test");
    }

    #[test]
    fn test_get_current_branch_not_git_repo() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = get_current_branch_from_repo(temp_dir.path());

        assert!(result.is_err());
    }

    #[test]
    fn test_get_current_branch_detached_head() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let repo = Repository::open(temp_dir.path()).unwrap();

        // Checkout the commit directly (detached HEAD)
        let head = repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();
        repo.set_head_detached(commit.id()).unwrap();

        let result = get_current_branch_from_repo(temp_dir.path());

        // Should fail because we're in detached HEAD state
        assert!(result.is_err());
    }
}
