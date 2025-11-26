use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// A branch with its fuzzy match score
#[derive(Debug, Clone)]
pub struct ScoredMatch {
    pub branch: String,
    pub score: i64,
}

/// Check if a branch name matches the given pattern (substring match)
pub fn matches(branch: &str, pattern: &str, ignore_case: bool) -> bool {
    if ignore_case {
        branch.to_lowercase().contains(&pattern.to_lowercase())
    } else {
        branch.contains(pattern)
    }
}

/// Filter branches by pattern using substring matching
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

/// Filter and score branches using fuzzy matching
/// Returns branches with their fuzzy match scores, sorted by score (highest first)
pub fn fuzzy_filter_branches(
    branches: &[String],
    pattern: &str,
    ignore_case: bool,
) -> Vec<ScoredMatch> {
    if pattern.is_empty() {
        // If no pattern, return all branches with zero score
        return branches
            .iter()
            .map(|b| ScoredMatch {
                branch: b.clone(),
                score: 0,
            })
            .collect();
    }

    let matcher = SkimMatcherV2::default();

    let mut scored: Vec<ScoredMatch> = branches
        .iter()
        .filter_map(|branch| {
            let search_text = if ignore_case {
                branch.to_lowercase()
            } else {
                branch.clone()
            };

            let search_pattern = if ignore_case {
                pattern.to_lowercase()
            } else {
                pattern.to_string()
            };

            matcher
                .fuzzy_match(&search_text, &search_pattern)
                .map(|score| ScoredMatch {
                    branch: branch.clone(),
                    score,
                })
        })
        .collect();

    // Sort by score descending (higher scores = better matches)
    scored.sort_by(|a, b| b.score.cmp(&a.score));

    scored
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
    fn test_matches_empty_pattern() {
        assert!(matches("feature/auth", "", false));
        assert!(matches("main", "", true));
        assert!(matches("", "", false));
    }

    #[test]
    fn test_matches_empty_branch() {
        assert!(!matches("", "test", false));
        assert!(!matches("", "TEST", true));
    }

    #[test]
    fn test_matches_exact_match() {
        assert!(matches("main", "main", false));
        assert!(matches("MAIN", "MAIN", false));
        assert!(matches("main", "MAIN", true));
    }

    #[test]
    fn test_matches_partial() {
        assert!(matches("feature/auth-module", "auth", false));
        assert!(matches("prefix-middle-suffix", "middle", false));
        assert!(matches("start-of-branch", "start", false));
        assert!(matches("end-of-branch", "branch", false));
    }

    #[test]
    fn test_matches_special_characters() {
        assert!(matches("feature/auth-v2", "auth-v2", false));
        assert!(matches("bugfix/issue-#123", "#123", false));
        assert!(matches("release/v1.0.0", "v1.0", false));
    }

    #[test]
    fn test_matches_unicode() {
        assert!(matches("feature/日本語", "日本", false));
        assert!(matches("branch-🚀-rocket", "🚀", false));
        assert!(matches("café-feature", "café", false));
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

    #[test]
    fn test_filter_branches_no_matches() {
        let branches = vec!["main".to_string(), "develop".to_string()];

        let matches = filter_branches(&branches, "feature", false);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_filter_branches_empty_pattern() {
        let branches = vec![
            "main".to_string(),
            "develop".to_string(),
            "feature".to_string(),
        ];

        let matches = filter_branches(&branches, "", false);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_filter_branches_empty_list() {
        let branches: Vec<String> = vec![];
        let matches = filter_branches(&branches, "feature", false);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_filter_branches_case_sensitive() {
        let branches = vec![
            "Feature/Auth".to_string(),
            "feature/auth".to_string(),
            "FEATURE/AUTH".to_string(),
        ];

        let matches = filter_branches(&branches, "feature", false);
        assert_eq!(matches.len(), 1);
        assert_eq!(*matches[0], "feature/auth");
    }

    #[test]
    fn test_filter_branches_case_insensitive() {
        let branches = vec![
            "Feature/Auth".to_string(),
            "feature/auth".to_string(),
            "FEATURE/AUTH".to_string(),
        ];

        let matches = filter_branches(&branches, "feature", true);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_fuzzy_filter_branches() {
        let branches = vec![
            "main".to_string(),
            "expo-feature-branch".to_string(),
            "feature/dashboard".to_string(),
            "bugfix/login".to_string(),
        ];

        let matches = fuzzy_filter_branches(&branches, "exo", false);

        // Should match "expo-feature-branch" with fuzzy matching
        assert!(!matches.is_empty());
        assert_eq!(matches[0].branch, "expo-feature-branch");
    }

    #[test]
    fn test_fuzzy_filter_scores_ordering() {
        let branches = vec![
            "feat".to_string(),
            "feature".to_string(),
            "features/something".to_string(),
            "test/feat".to_string(),
        ];

        let matches = fuzzy_filter_branches(&branches, "feat", false);

        // Should have matches and they should be ordered
        assert!(!matches.is_empty());
        // Best matches should contain "feat"
        assert!(matches[0].branch.contains("feat"));
    }

    #[test]
    fn test_fuzzy_filter_empty_pattern() {
        let branches = vec!["main".to_string(), "feature".to_string()];

        let matches = fuzzy_filter_branches(&branches, "", false);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].score, 0);
        assert_eq!(matches[1].score, 0);
    }

    #[test]
    fn test_fuzzy_filter_empty_branches() {
        let branches: Vec<String> = vec![];
        let matches = fuzzy_filter_branches(&branches, "test", false);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_fuzzy_filter_no_matches() {
        let branches = vec!["main".to_string(), "develop".to_string()];

        let matches = fuzzy_filter_branches(&branches, "xyz", false);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_fuzzy_filter_case_insensitive() {
        let branches = vec![
            "Feature/Auth".to_string(),
            "MAIN".to_string(),
            "develop".to_string(),
        ];

        let matches = fuzzy_filter_branches(&branches, "AUTH", true);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].branch, "Feature/Auth");
    }

    #[test]
    fn test_fuzzy_filter_case_sensitive() {
        let branches = vec!["Feature/Auth".to_string(), "feature/auth".to_string()];

        let matches_lower = fuzzy_filter_branches(&branches, "auth", false);
        assert!(!matches_lower.is_empty());

        // Both branches should match regardless of case in pattern
        let matches_upper = fuzzy_filter_branches(&branches, "auth", true);
        assert!(!matches_upper.is_empty());

        // Should find branches with auth
        assert!(matches_lower
            .iter()
            .any(|m| m.branch.contains("auth") || m.branch.contains("Auth")));
    }

    #[test]
    fn test_fuzzy_filter_complex_pattern() {
        let branches = vec![
            "feature/authentication".to_string(),
            "feat/auth".to_string(),
            "fix/authorization".to_string(),
        ];

        let matches = fuzzy_filter_branches(&branches, "fauth", false);
        assert!(!matches.is_empty());
        // Should match branches containing f and auth
        assert!(matches.iter().any(|m| m.branch.contains("auth")));
    }

    #[test]
    fn test_fuzzy_filter_ordering_by_score() {
        let branches = vec![
            "test-feature-branch".to_string(),
            "test".to_string(),
            "testing".to_string(),
        ];

        let matches = fuzzy_filter_branches(&branches, "test", false);
        assert!(!matches.is_empty());

        // Should have matches ordered by score
        assert!(matches[0].score >= matches[1].score);
        assert!(matches[1].score >= matches[2].score);
    }

    #[test]
    fn test_scored_match_clone() {
        let original = ScoredMatch {
            branch: "test".to_string(),
            score: 100,
        };
        let cloned = original.clone();
        assert_eq!(original.branch, cloned.branch);
        assert_eq!(original.score, cloned.score);
    }

    #[test]
    fn test_scored_match_debug() {
        let scored = ScoredMatch {
            branch: "test".to_string(),
            score: 100,
        };
        let debug_str = format!("{:?}", scored);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("100"));
    }
}
