use anyhow::{bail, Result};
use std::path::Path;

use crate::constants::validation::*;

/// Validate that a branch name is safe and valid according to git rules
pub fn validate_branch_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Branch name cannot be empty");
    }

    if name.len() > MAX_BRANCH_NAME_LENGTH {
        bail!(
            "Branch name too long (max {} characters)",
            MAX_BRANCH_NAME_LENGTH
        );
    }

    // Check for dangerous characters that could cause issues
    let dangerous_chars = ['\0', '\n', '\r'];
    if name.chars().any(|c| dangerous_chars.contains(&c)) {
        bail!("Branch name contains invalid characters (null, newline, or carriage return)");
    }

    // Git branch name restrictions
    if name.starts_with('-') {
        bail!("Branch name cannot start with '-' (conflicts with git flags)");
    }

    if name.starts_with('.') {
        bail!("Branch name cannot start with '.'");
    }

    if name.contains("..") {
        bail!("Branch name cannot contain '..' (git path traversal restriction)");
    }

    if name.ends_with('/') {
        bail!("Branch name cannot end with '/'");
    }

    if name.ends_with('.') {
        bail!("Branch name cannot end with '.'");
    }

    if name.contains("//") {
        bail!("Branch name cannot contain '//' (double slashes)");
    }

    if name.contains(' ') {
        bail!("Branch name cannot contain spaces");
    }

    // Check for other problematic characters
    if name.contains('@') && name.contains('{') {
        bail!("Branch name cannot contain '@{{' (git revision syntax)");
    }

    if name.contains('~') {
        bail!("Branch name cannot contain '~' (git revision syntax)");
    }

    if name.contains('^') {
        bail!("Branch name cannot contain '^' (git revision syntax)");
    }

    if name.contains(':') {
        bail!("Branch name cannot contain ':' (git ref syntax)");
    }

    if name.contains('?') || name.contains('*') || name.contains('[') {
        bail!("Branch name cannot contain wildcards (?, *, [)");
    }

    Ok(())
}

/// Validate that a repo path is safe and valid
pub fn validate_repo_path(path: &str) -> Result<()> {
    if path.is_empty() {
        bail!("Repository path cannot be empty");
    }

    if path.len() > MAX_REPO_PATH_LENGTH {
        bail!(
            "Repository path too long (max {} characters)",
            MAX_REPO_PATH_LENGTH
        );
    }

    // Check for null bytes
    if path.contains('\0') {
        bail!("Repository path contains null bytes");
    }

    let path_obj = Path::new(path);

    // Must be absolute path for safety
    if !path_obj.is_absolute() {
        bail!("Repository path must be absolute (got relative path)");
    }

    // Verify it exists
    if !path_obj.exists() {
        bail!("Repository path does not exist: {}", path);
    }

    // Must be a directory
    if !path_obj.is_dir() {
        bail!("Repository path is not a directory: {}", path);
    }

    Ok(())
}

/// Validate search pattern
pub fn validate_pattern(pattern: &str) -> Result<()> {
    if pattern.len() > MAX_PATTERN_LENGTH {
        bail!(
            "Search pattern too long (max {} characters)",
            MAX_PATTERN_LENGTH
        );
    }

    // Check for null bytes
    if pattern.contains('\0') {
        bail!("Search pattern contains null bytes");
    }

    // Pattern can be empty (matches all branches)
    // Pattern can contain most characters (for fuzzy matching)
    // Just check for obviously dangerous things

    Ok(())
}

/// Validate alias name (more strict than branch names)
pub fn validate_alias_name(alias: &str) -> Result<()> {
    if alias.is_empty() {
        bail!("Alias name cannot be empty");
    }

    if alias.len() > MAX_ALIAS_LENGTH {
        bail!("Alias name too long (max {} characters)", MAX_ALIAS_LENGTH);
    }

    if alias.starts_with('-') {
        bail!("Alias name cannot start with '-' (conflicts with command flags)");
    }

    // Check if alias is a reserved word
    if matches!(alias, "stats" | "alias" | "list" | "remove") {
        bail!("Alias name '{}' is reserved and cannot be used", alias);
    }

    // Only allow alphanumeric, dash, and underscore
    if !alias
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        bail!("Alias name must contain only alphanumeric characters, dash (-), or underscore (_)");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Branch name validation tests
    #[test]
    fn test_validate_branch_name_valid() {
        assert!(validate_branch_name("feature/auth").is_ok());
        assert!(validate_branch_name("main").is_ok());
        assert!(validate_branch_name("bugfix-123").is_ok());
        assert!(validate_branch_name("feature/issue-#123_v2.0").is_ok());
    }

    #[test]
    fn test_validate_branch_name_empty() {
        assert!(validate_branch_name("").is_err());
    }

    #[test]
    fn test_validate_branch_name_starts_with_dash() {
        assert!(validate_branch_name("-bad").is_err());
    }

    #[test]
    fn test_validate_branch_name_starts_with_dot() {
        assert!(validate_branch_name(".hidden").is_err());
    }

    #[test]
    fn test_validate_branch_name_double_dots() {
        assert!(validate_branch_name("has..dots").is_err());
    }

    #[test]
    fn test_validate_branch_name_trailing_slash() {
        assert!(validate_branch_name("trailing/").is_err());
    }

    #[test]
    fn test_validate_branch_name_trailing_dot() {
        assert!(validate_branch_name("trailing.").is_err());
    }

    #[test]
    fn test_validate_branch_name_double_slash() {
        assert!(validate_branch_name("feature//bug").is_err());
    }

    #[test]
    fn test_validate_branch_name_spaces() {
        assert!(validate_branch_name("has spaces").is_err());
    }

    #[test]
    fn test_validate_branch_name_null_byte() {
        assert!(validate_branch_name("null\0byte").is_err());
    }

    #[test]
    fn test_validate_branch_name_newline() {
        assert!(validate_branch_name("new\nline").is_err());
    }

    #[test]
    fn test_validate_branch_name_git_revision_chars() {
        assert!(validate_branch_name("branch~1").is_err());
        assert!(validate_branch_name("branch^2").is_err());
        assert!(validate_branch_name("branch:ref").is_err());
    }

    #[test]
    fn test_validate_branch_name_wildcards() {
        assert!(validate_branch_name("branch*").is_err());
        assert!(validate_branch_name("branch?").is_err());
        assert!(validate_branch_name("branch[0]").is_err());
    }

    #[test]
    fn test_validate_branch_name_too_long() {
        let long_name = "a".repeat(256);
        assert!(validate_branch_name(&long_name).is_err());
    }

    // Pattern validation tests
    #[test]
    fn test_validate_pattern_valid() {
        assert!(validate_pattern("feat").is_ok());
        assert!(validate_pattern("feature/").is_ok());
        assert!(validate_pattern("").is_ok()); // Empty is ok (matches all)
        assert!(validate_pattern("123").is_ok());
    }

    #[test]
    fn test_validate_pattern_null_byte() {
        assert!(validate_pattern("null\0byte").is_err());
    }

    #[test]
    fn test_validate_pattern_too_long() {
        let long_pattern = "a".repeat(256);
        assert!(validate_pattern(&long_pattern).is_err());
    }

    // Alias name validation tests
    #[test]
    fn test_validate_alias_name_valid() {
        assert!(validate_alias_name("m").is_ok());
        assert!(validate_alias_name("main").is_ok());
        assert!(validate_alias_name("my-alias").is_ok());
        assert!(validate_alias_name("my_alias").is_ok());
        assert!(validate_alias_name("alias123").is_ok());
    }

    #[test]
    fn test_validate_alias_name_empty() {
        assert!(validate_alias_name("").is_err());
    }

    #[test]
    fn test_validate_alias_name_starts_with_dash() {
        assert!(validate_alias_name("-bad").is_err());
    }

    #[test]
    fn test_validate_alias_name_reserved() {
        assert!(validate_alias_name("stats").is_err());
        assert!(validate_alias_name("alias").is_err());
        assert!(validate_alias_name("list").is_err());
        assert!(validate_alias_name("remove").is_err());
    }

    #[test]
    fn test_validate_alias_name_special_chars() {
        assert!(validate_alias_name("has spaces").is_err());
        assert!(validate_alias_name("has/slash").is_err());
        assert!(validate_alias_name("has.dot").is_err());
        assert!(validate_alias_name("has@at").is_err());
    }

    #[test]
    fn test_validate_alias_name_too_long() {
        let long_alias = "a".repeat(51);
        assert!(validate_alias_name(&long_alias).is_err());
    }

    // Repo path validation tests
    #[test]
    fn test_validate_repo_path_empty() {
        assert!(validate_repo_path("").is_err());
    }

    #[test]
    fn test_validate_repo_path_null_byte() {
        assert!(validate_repo_path("/path/with\0null").is_err());
    }

    #[test]
    fn test_validate_repo_path_relative() {
        assert!(validate_repo_path("relative/path").is_err());
        assert!(validate_repo_path("./relative").is_err());
    }

    #[test]
    fn test_validate_repo_path_nonexistent() {
        assert!(validate_repo_path("/this/path/definitely/does/not/exist/12345").is_err());
    }

    #[test]
    fn test_validate_repo_path_too_long() {
        let long_path = format!("/{}", "a".repeat(4097));
        assert!(validate_repo_path(&long_path).is_err());
    }

    #[test]
    fn test_validate_repo_path_current_dir() {
        // This should work if we're in a real directory
        let current = std::env::current_dir().unwrap();
        let current_str = current.to_str().unwrap();
        assert!(validate_repo_path(current_str).is_ok());
    }
}
