/// Check if a branch name matches the given pattern
pub fn matches(branch: &str, pattern: &str, ignore_case: bool) -> bool {
    if ignore_case {
        branch.to_lowercase().contains(&pattern.to_lowercase())
    } else {
        branch.contains(pattern)
    }
}

/// Filter branches by pattern and return matching ones
pub fn filter_branches<'a>(
    branches: &'a [String],
    pattern: &str,
    ignore_case: bool,
) -> Vec<&'a String> {
    branches
        .iter()
        .filter(|branch| matches(branch, pattern, ignore_case))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_case_sensitive() {
        assert!(matches("feature/auth", "feat", false));
        assert!(matches("feature/auth", "auth", false));
        assert!(!matches("feature/auth", "FEAT", false));
    }

    #[test]
    fn test_matches_case_insensitive() {
        assert!(matches("feature/auth", "FEAT", true));
        assert!(matches("FEATURE/AUTH", "feat", true));
        assert!(matches("Feature/Auth", "feature", true));
    }

    #[test]
    fn test_filter_branches() {
        let branches = vec![
            "main".to_string(),
            "feature/auth".to_string(),
            "feature/dashboard".to_string(),
            "bugfix/login".to_string(),
        ];

        let matches = filter_branches(&branches, "feature", false);
        assert_eq!(matches.len(), 2);
        assert_eq!(*matches[0], "feature/auth");
        assert_eq!(*matches[1], "feature/dashboard");
    }
}

