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
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Show an interactive menu to select a branch
pub fn select_branch(
    branches: &[String],
    records: &[BranchRecord],
) -> Result<String> {
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
    println!("\n{:<40} │ {:>12} │ {:>12} │ Last used", 
             "Branch", "Frecency", "Usage");
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
        assert_eq!(truncate("this is a very long branch name", 15), "this is a ve...");
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
}

