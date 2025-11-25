use crate::storage::BranchRecord;
use std::time::{SystemTime, UNIX_EPOCH};

/// Calculate the frecency score for a branch record.
/// 
/// Frecency = frequency × recency_weight
/// 
/// The recency weight decays over time:
/// - Used in last hour: weight = 4.0
/// - Used in last day: weight = 2.0
/// - Used in last week: weight = 1.0
/// - Used in last month: weight = 0.5
/// - Older: weight = 0.25
pub fn calculate_score(record: &BranchRecord) -> f64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let age_seconds = now - record.last_used;
    
    let recency_weight = if age_seconds < 3600 {
        // Last hour
        4.0
    } else if age_seconds < 86400 {
        // Last day
        2.0
    } else if age_seconds < 604800 {
        // Last week
        1.0
    } else if age_seconds < 2592000 {
        // Last month
        0.5
    } else {
        // Older
        0.25
    };

    record.switch_count as f64 * recency_weight
}

/// A branch with its calculated frecency score
#[derive(Debug, Clone)]
pub struct ScoredBranch {
    pub name: String,
    pub score: f64,
    pub switch_count: i64,
    pub last_used: i64,
}

/// Score and sort branches by frecency
pub fn rank_branches(records: &[BranchRecord]) -> Vec<ScoredBranch> {
    let mut scored: Vec<ScoredBranch> = records
        .iter()
        .map(|r| ScoredBranch {
            name: r.branch_name.clone(),
            score: calculate_score(r),
            switch_count: r.switch_count,
            last_used: r.last_used,
        })
        .collect();

    // Sort by score descending
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    scored
}

/// Given a list of branch names and their usage records, return them sorted by frecency.
/// Branches without usage data are placed at the end (score = 0).
pub fn sort_branches_by_frecency(
    branches: &[String],
    records: &[BranchRecord],
) -> Vec<(String, f64)> {
    let scored = rank_branches(records);

    let mut result: Vec<(String, f64)> = branches
        .iter()
        .map(|branch| {
            let score = scored
                .iter()
                .find(|s| s.name == *branch)
                .map(|s| s.score)
                .unwrap_or(0.0);
            (branch.clone(), score)
        })
        .collect();

    // Sort by score descending (branches with higher frecency first)
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    result
}

/// Format a timestamp as a human-readable relative time
pub fn format_relative_time(timestamp: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let age_seconds = now - timestamp;

    if age_seconds < 60 {
        "just now".to_string()
    } else if age_seconds < 3600 {
        let mins = age_seconds / 60;
        format!("{}m ago", mins)
    } else if age_seconds < 86400 {
        let hours = age_seconds / 3600;
        format!("{}h ago", hours)
    } else if age_seconds < 604800 {
        let days = age_seconds / 86400;
        format!("{}d ago", days)
    } else if age_seconds < 2592000 {
        let weeks = age_seconds / 604800;
        format!("{}w ago", weeks)
    } else {
        let months = age_seconds / 2592000;
        format!("{}mo ago", months)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_score_recent() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let record = BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "main".to_string(),
            switch_count: 10,
            last_used: now - 60, // 1 minute ago
        };

        let score = calculate_score(&record);
        assert_eq!(score, 40.0); // 10 * 4.0
    }

    #[test]
    fn test_calculate_score_old() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let record = BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "main".to_string(),
            switch_count: 10,
            last_used: now - 3000000, // ~35 days ago
        };

        let score = calculate_score(&record);
        assert_eq!(score, 2.5); // 10 * 0.25
    }
}

