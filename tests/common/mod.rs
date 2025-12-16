use git2::Repository;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Generate a unique repo path for testing
/// Used for in-memory database tests that need unique repo identifiers
pub fn unique_repo_path() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("/test/repo/{}", id)
}

/// Setup a temporary git repository for testing
/// Returns a TempDir that will be cleaned up when dropped
pub fn setup_test_repo() -> std::io::Result<tempfile::TempDir> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_repo_path_is_unique() {
        let path1 = unique_repo_path();
        let path2 = unique_repo_path();
        let path3 = unique_repo_path();

        assert_ne!(path1, path2);
        assert_ne!(path2, path3);
        assert_ne!(path1, path3);

        // Should follow expected format
        assert!(path1.starts_with("/test/repo/"));
        assert!(path2.starts_with("/test/repo/"));
        assert!(path3.starts_with("/test/repo/"));
    }

    #[test]
    fn test_setup_test_repo_creates_repo() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let repo_path = temp_dir.path();

        // Verify it's a git repository
        assert!(Repository::open(repo_path).is_ok());

        // Verify has initial commit
        let repo = Repository::open(repo_path).unwrap();
        assert!(repo.head().is_ok());

        // Verify test file exists
        let test_file = repo_path.join("test.txt");
        assert!(test_file.exists());
    }

    #[test]
    fn test_setup_test_repo_has_config() {
        let temp_dir = setup_test_repo().expect("Failed to create test repo");
        let repo = Repository::open(temp_dir.path()).unwrap();

        let config = repo.config().unwrap();
        let email = config.get_string("user.email").unwrap();
        let name = config.get_string("user.name").unwrap();

        assert_eq!(email, "test@example.com");
        assert_eq!(name, "Test User");
    }
}
