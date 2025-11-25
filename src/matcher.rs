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
            
            matcher.fuzzy_match(&search_text, &search_pattern).map(|score| {
                ScoredMatch {
                    branch: branch.clone(),
                    score,
                }
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
        
        // Exact matches should score higher
        assert_eq!(matches[0].branch, "feat");
        assert!(matches[0].score > matches[1].score);
    }

    #[test]
    fn test_fuzzy_filter_empty_pattern() {
        let branches = vec![
            "main".to_string(),
            "feature".to_string(),
        ];

        let matches = fuzzy_filter_branches(&branches, "", false);
        assert_eq!(matches.len(), 2);
    }
}

