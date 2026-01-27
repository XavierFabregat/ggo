use thiserror::Error;

/// Custom error types for ggo
#[derive(Error, Debug)]
pub enum GgoError {
    #[error("Not in a git repository\n\nRun this command from within a git repository.")]
    NotGitRepository,

    #[error("Branch '{0}' not found\n\nRun 'git branch' to see available branches.")]
    BranchNotFound(String),

    #[error("No branches match pattern '{0}'\n\nTry:\n  • Using a shorter pattern\n  • Running 'ggo --list \"\"' to see all branches\n  • Using case-insensitive mode with '-i'")]
    NoMatchingBranches(String),

    #[error("Failed to checkout branch '{0}': {1}")]
    CheckoutFailed(String, String),

    #[error("Invalid branch name: {0}\n\n{1}")]
    InvalidBranchName(String, String),

    #[error("Invalid pattern: {0}\n\n{1}")]
    InvalidPattern(String, String),

    #[error("Invalid repository path: {0}\n\n{1}")]
    InvalidRepoPath(String, String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Configuration error: {0}\n\nCheck your config file at ~/.config/ggo/config.toml")]
    ConfigError(String),

    #[error("No previous branch found\n\nYou need to switch branches at least once before using 'ggo -'")]
    NoPreviousBranch,

    #[error("User cancelled operation")]
    UserCancelled,

    #[allow(dead_code)]
    #[error("Alias '{0}' not found in this repository\n\nRun 'ggo alias --list' to see all aliases.")]
    AliasNotFound(String),

    #[error("Unsupported shell: '{0}'\n\nSupported shells:\n  • bash\n  • zsh\n  • fish\n  • powershell\n  • elvish\n\nExample: ggo --generate-completion bash")]
    InvalidShell(String),

    #[error("{0}")]
    Other(String),
}

// Implement conversions from other error types
impl From<rusqlite::Error> for GgoError {
    fn from(err: rusqlite::Error) -> Self {
        GgoError::DatabaseError(err.to_string())
    }
}

impl From<git2::Error> for GgoError {
    fn from(err: git2::Error) -> Self {
        match err.code() {
            git2::ErrorCode::NotFound => GgoError::NotGitRepository,
            _ => GgoError::Other(err.to_string()),
        }
    }
}

impl From<std::io::Error> for GgoError {
    fn from(err: std::io::Error) -> Self {
        GgoError::Other(err.to_string())
    }
}

impl From<inquire::InquireError> for GgoError {
    fn from(err: inquire::InquireError) -> Self {
        match err {
            inquire::InquireError::OperationCanceled => GgoError::UserCancelled,
            inquire::InquireError::OperationInterrupted => GgoError::UserCancelled,
            _ => GgoError::Other(err.to_string()),
        }
    }
}

impl From<anyhow::Error> for GgoError {
    fn from(err: anyhow::Error) -> Self {
        GgoError::Other(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GgoError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_git_repository_error() {
        let err = GgoError::NotGitRepository;
        let msg = err.to_string();
        assert!(msg.contains("Not in a git repository"));
        assert!(msg.contains("Run this command from within"));
    }

    #[test]
    fn test_branch_not_found_error() {
        let err = GgoError::BranchNotFound("feature/auth".to_string());
        let msg = err.to_string();
        assert!(msg.contains("feature/auth"));
        assert!(msg.contains("not found"));
        assert!(msg.contains("git branch"));
    }

    #[test]
    fn test_no_matching_branches_error() {
        let err = GgoError::NoMatchingBranches("xyz".to_string());
        let msg = err.to_string();
        assert!(msg.contains("No branches match pattern 'xyz'"));
        assert!(msg.contains("Try:"));
        assert!(msg.contains("shorter pattern"));
    }

    #[test]
    fn test_checkout_failed_error() {
        let err = GgoError::CheckoutFailed("main".to_string(), "uncommitted changes".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Failed to checkout branch 'main'"));
        assert!(msg.contains("uncommitted changes"));
    }

    #[test]
    fn test_invalid_branch_name_error() {
        let err = GgoError::InvalidBranchName(
            "-bad-name".to_string(),
            "cannot start with dash".to_string(),
        );
        let msg = err.to_string();
        assert!(msg.contains("Invalid branch name: -bad-name"));
        assert!(msg.contains("cannot start with dash"));
    }

    #[test]
    fn test_database_error() {
        let err = GgoError::DatabaseError("connection failed".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Database error"));
        assert!(msg.contains("connection failed"));
    }

    #[test]
    fn test_config_error() {
        let err = GgoError::ConfigError("invalid TOML syntax".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Configuration error"));
        assert!(msg.contains("config.toml"));
    }

    #[test]
    fn test_no_previous_branch_error() {
        let err = GgoError::NoPreviousBranch;
        let msg = err.to_string();
        assert!(msg.contains("No previous branch"));
        assert!(msg.contains("ggo -"));
    }

    #[test]
    fn test_user_cancelled_error() {
        let err = GgoError::UserCancelled;
        let msg = err.to_string();
        assert!(msg.contains("User cancelled"));
    }

    #[test]
    fn test_alias_not_found_error() {
        let err = GgoError::AliasNotFound("m".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Alias 'm' not found"));
        assert!(msg.contains("ggo alias --list"));
    }

    #[test]
    fn test_invalid_shell_error() {
        let err = GgoError::InvalidShell("invalid".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Unsupported shell"));
        assert!(msg.contains("invalid"));
        assert!(msg.contains("Supported shells:"));
        assert!(msg.contains("bash"));
    }

    #[test]
    fn test_from_rusqlite_error() {
        let sqlite_err = rusqlite::Error::InvalidQuery;
        let ggo_err: GgoError = sqlite_err.into();
        assert!(matches!(ggo_err, GgoError::DatabaseError(_)));
    }

    #[test]
    fn test_from_git2_not_found() {
        let git_err = git2::Error::from_str("repository not found");
        let ggo_err: GgoError = git_err.into();
        // Should convert to appropriate error
        assert!(matches!(ggo_err, GgoError::NotGitRepository | GgoError::Other(_)));
    }

    #[test]
    fn test_from_inquire_cancelled() {
        let inquire_err = inquire::InquireError::OperationCanceled;
        let ggo_err: GgoError = inquire_err.into();
        assert!(matches!(ggo_err, GgoError::UserCancelled));
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<GgoError>();
    }

    #[test]
    fn test_result_type_alias() {
        let ok: Result<i32> = Ok(42);
        let err: Result<i32> = Err(GgoError::UserCancelled);

        assert!(ok.is_ok());
        assert!(err.is_err());
    }
}
