use anyhow::Result;
use inquire::Select;

use crate::frecency;
use crate::storage::BranchRecord;

/// Represents a branch with its display information
#[derive(Clone)]
pub struct BranchOption {
    pub name: String,
    pub score: f64,
    pub switch_count: i64,
    pub last_used: Option<i64>,
}

impl std::fmt::Display for BranchOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let score_str = if self.score > 0.0 {
            format!("score: {:.1}", self.score)
        } else {
            "new".to_string()
        };

        let usage_str = if self.switch_count > 0 {
            format!("{} switches", self.switch_count)
        } else {
            "never used".to_string()
        };

        let time_str = if let Some(last_used) = self.last_used {
            frecency::format_relative_time(last_used)
        } else {
            "never".to_string()
        };

        write!(
            f,
            "{:<40} │ {:>12} │ {:>12} │ {}",
            truncate(&self.name, 40),
            score_str,
            usage_str,
            time_str
        )
    }
}

/// Truncate a string to a maximum length, adding ellipsis if needed
/// Uses character count (not byte count) to safely handle multi-byte UTF-8 characters
fn truncate(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

/// Show an interactive menu to select a branch
pub fn select_branch(branches: &[String], records: &[BranchRecord]) -> Result<String> {
    // Rank branches by frecency
    let ranked = frecency::sort_branches_by_frecency(branches, records);

    // Create options with metadata
    let mut options: Vec<BranchOption> = Vec::new();
    for (branch, score) in ranked {
        let record = records.iter().find(|r| r.branch_name == branch);
        let option = BranchOption {
            name: branch.clone(),
            score,
            switch_count: record.map(|r| r.switch_count).unwrap_or(0),
            last_used: record.map(|r| r.last_used),
        };
        options.push(option);
    }

    if options.is_empty() {
        anyhow::bail!("No branches available for selection");
    }

    // Show header
    println!(
        "\n{:<40} │ {:>12} │ {:>12} │ Last used",
        "Branch", "Frecency", "Usage"
    );
    println!("{}", "─".repeat(85));

    // Create the select prompt
    let selection = Select::new("Select a branch to checkout:", options)
        .with_page_size(15)
        .prompt()?;

    Ok(selection.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(
            truncate("this is a very long branch name", 15),
            "this is a ve..."
        );

        // Test with multi-byte UTF-8 characters (emoji, etc.)
        assert_eq!(truncate("feature/🚀-rocket", 20), "feature/🚀-rocket");
        assert_eq!(
            truncate("feature/🚀-rocket-launch-system", 15),
            "feature/🚀-ro..."
        );

        // Test with other Unicode characters
        assert_eq!(truncate("café-branch", 20), "café-branch");
        assert_eq!(truncate("日本語ブランチ名前", 8), "日本語ブラ...");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate("exactly-ten", 11), "exactly-ten");
        assert_eq!(truncate("twelve-chars", 12), "twelve-chars");
    }

    #[test]
    fn test_truncate_empty_string() {
        assert_eq!(truncate("", 10), "");
    }

    #[test]
    fn test_truncate_single_char() {
        assert_eq!(truncate("a", 10), "a");
        // When max_len is 1, truncating "abcdef" results in "..." (0 chars + ...)
        assert_eq!(truncate("abcdef", 1), "...");
    }

    #[test]
    fn test_truncate_zero_length() {
        // When max_len is 0, string won't need truncation if it's empty, otherwise "..."
        assert_eq!(truncate("", 0), "");
        assert_eq!(truncate("test", 0), "...");
    }

    #[test]
    fn test_truncate_three_length() {
        // When max_len is 3 and string is 4 chars, it needs truncation
        assert_eq!(truncate("tes", 3), "tes"); // Exactly 3 chars, no truncation
        assert_eq!(truncate("test", 3), "..."); // 4 chars > 3, truncate to 0 + "..."
        assert_eq!(truncate("testing", 3), "...");
    }

    #[test]
    fn test_truncate_four_length() {
        assert_eq!(truncate("testing", 4), "t...");
    }

    #[test]
    fn test_branch_option_display() {
        let option = BranchOption {
            name: "feature/auth".to_string(),
            score: 42.5,
            switch_count: 10,
            last_used: Some(1700000000),
        };
        let display = format!("{}", option);
        assert!(display.contains("feature/auth"));
        assert!(display.contains("42.5"));
        assert!(display.contains("10 switches"));
    }

    #[test]
    fn test_branch_option_display_zero_score() {
        let option = BranchOption {
            name: "new-branch".to_string(),
            score: 0.0,
            switch_count: 0,
            last_used: None,
        };
        let display = format!("{}", option);
        assert!(display.contains("new-branch"));
        assert!(display.contains("new"));
        assert!(display.contains("never used"));
        assert!(display.contains("never"));
    }

    #[test]
    fn test_branch_option_display_no_usage() {
        let option = BranchOption {
            name: "unused-branch".to_string(),
            score: 0.0,
            switch_count: 0,
            last_used: Some(1700000000),
        };
        let display = format!("{}", option);
        assert!(display.contains("unused-branch"));
        assert!(display.contains("new"));
        assert!(display.contains("never used"));
    }

    #[test]
    fn test_branch_option_display_high_score() {
        let option = BranchOption {
            name: "popular-branch".to_string(),
            score: 999.9,
            switch_count: 100,
            last_used: Some(1700000000),
        };
        let display = format!("{}", option);
        assert!(display.contains("popular-branch"));
        assert!(display.contains("999.9"));
        assert!(display.contains("100 switches"));
    }

    #[test]
    fn test_branch_option_display_long_name() {
        let option = BranchOption {
            name: "feature/very-long-branch-name-that-should-be-truncated-in-display".to_string(),
            score: 10.0,
            switch_count: 5,
            last_used: Some(1700000000),
        };
        let display = format!("{}", option);
        assert!(display.contains("..."));
    }

    #[test]
    fn test_branch_option_display_with_special_chars() {
        let option = BranchOption {
            name: "feature/auth-🔐".to_string(),
            score: 15.5,
            switch_count: 3,
            last_used: Some(1700000000),
        };
        let display = format!("{}", option);
        assert!(display.contains("feature/auth-🔐"));
        assert!(display.contains("15.5"));
        assert!(display.contains("3 switches"));
    }

    #[test]
    fn test_branch_option_clone() {
        let option = BranchOption {
            name: "test".to_string(),
            score: 10.0,
            switch_count: 5,
            last_used: Some(1700000000),
        };
        let cloned = option.clone();
        assert_eq!(option.name, cloned.name);
        assert_eq!(option.score, cloned.score);
        assert_eq!(option.switch_count, cloned.switch_count);
        assert_eq!(option.last_used, cloned.last_used);
    }
}
