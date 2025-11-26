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
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

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
    fn test_calculate_score_within_hour() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let record = BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "main".to_string(),
            switch_count: 5,
            last_used: now - 3599, // Just under 1 hour ago
        };

        let score = calculate_score(&record);
        assert_eq!(score, 20.0); // 5 * 4.0
    }

    #[test]
    fn test_calculate_score_within_day() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let record = BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "develop".to_string(),
            switch_count: 8,
            last_used: now - 43200, // 12 hours ago
        };

        let score = calculate_score(&record);
        assert_eq!(score, 16.0); // 8 * 2.0
    }

    #[test]
    fn test_calculate_score_within_week() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let record = BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "feature".to_string(),
            switch_count: 6,
            last_used: now - 259200, // 3 days ago
        };

        let score = calculate_score(&record);
        assert_eq!(score, 6.0); // 6 * 1.0
    }

    #[test]
    fn test_calculate_score_within_month() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let record = BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "bugfix".to_string(),
            switch_count: 4,
            last_used: now - 1209600, // 14 days ago
        };

        let score = calculate_score(&record);
        assert_eq!(score, 2.0); // 4 * 0.5
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

    #[test]
    fn test_calculate_score_zero_switches() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let record = BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "unused".to_string(),
            switch_count: 0,
            last_used: now - 60,
        };

        let score = calculate_score(&record);
        assert_eq!(score, 0.0); // 0 * 4.0
    }

    #[test]
    fn test_rank_branches_empty() {
        let records: Vec<BranchRecord> = vec![];
        let ranked = rank_branches(&records);
        assert!(ranked.is_empty());
    }

    #[test]
    fn test_rank_branches_single() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let records = vec![BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "main".to_string(),
            switch_count: 5,
            last_used: now - 60,
        }];

        let ranked = rank_branches(&records);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].name, "main");
        assert_eq!(ranked[0].score, 20.0);
        assert_eq!(ranked[0].switch_count, 5);
    }

    #[test]
    fn test_rank_branches_sorted_by_score() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let records = vec![
            BranchRecord {
                repo_path: "/test".to_string(),
                branch_name: "old".to_string(),
                switch_count: 10,
                last_used: now - 3000000, // Old: score = 2.5
            },
            BranchRecord {
                repo_path: "/test".to_string(),
                branch_name: "recent".to_string(),
                switch_count: 5,
                last_used: now - 60, // Recent: score = 20.0
            },
            BranchRecord {
                repo_path: "/test".to_string(),
                branch_name: "medium".to_string(),
                switch_count: 3,
                last_used: now - 43200, // Day: score = 6.0
            },
        ];

        let ranked = rank_branches(&records);
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].name, "recent");
        assert_eq!(ranked[0].score, 20.0);
        assert_eq!(ranked[1].name, "medium");
        assert_eq!(ranked[1].score, 6.0);
        assert_eq!(ranked[2].name, "old");
        assert_eq!(ranked[2].score, 2.5);
    }

    #[test]
    fn test_sort_branches_by_frecency_empty_branches() {
        let branches: Vec<String> = vec![];
        let records: Vec<BranchRecord> = vec![];
        let sorted = sort_branches_by_frecency(&branches, &records);
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_sort_branches_by_frecency_no_records() {
        let branches = vec![
            "main".to_string(),
            "develop".to_string(),
            "feature".to_string(),
        ];
        let records: Vec<BranchRecord> = vec![];

        let sorted = sort_branches_by_frecency(&branches, &records);
        assert_eq!(sorted.len(), 3);

        // All should have score 0.0
        for (_, score) in &sorted {
            assert_eq!(*score, 0.0);
        }
    }

    #[test]
    fn test_sort_branches_by_frecency_with_records() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let branches = vec![
            "main".to_string(),
            "develop".to_string(),
            "feature".to_string(),
        ];

        let records = vec![
            BranchRecord {
                repo_path: "/test".to_string(),
                branch_name: "develop".to_string(),
                switch_count: 10,
                last_used: now - 60, // score = 40.0
            },
            BranchRecord {
                repo_path: "/test".to_string(),
                branch_name: "main".to_string(),
                switch_count: 5,
                last_used: now - 43200, // score = 10.0
            },
        ];

        let sorted = sort_branches_by_frecency(&branches, &records);
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].0, "develop");
        assert_eq!(sorted[0].1, 40.0);
        assert_eq!(sorted[1].0, "main");
        assert_eq!(sorted[1].1, 10.0);
        assert_eq!(sorted[2].0, "feature");
        assert_eq!(sorted[2].1, 0.0);
    }

    #[test]
    fn test_sort_branches_by_frecency_partial_records() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let branches = vec![
            "branch-a".to_string(),
            "branch-b".to_string(),
            "branch-c".to_string(),
        ];

        let records = vec![BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "branch-b".to_string(),
            switch_count: 3,
            last_used: now - 60,
        }];

        let sorted = sort_branches_by_frecency(&branches, &records);
        assert_eq!(sorted[0].0, "branch-b");
        assert!(sorted[0].1 > 0.0);
        assert_eq!(sorted[1].1, 0.0);
        assert_eq!(sorted[2].1, 0.0);
    }

    #[test]
    fn test_format_relative_time_just_now() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        assert_eq!(format_relative_time(now), "just now");
        assert_eq!(format_relative_time(now - 30), "just now");
        assert_eq!(format_relative_time(now - 59), "just now");
    }

    #[test]
    fn test_format_relative_time_minutes() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        assert_eq!(format_relative_time(now - 60), "1m ago");
        assert_eq!(format_relative_time(now - 120), "2m ago");
        assert_eq!(format_relative_time(now - 1800), "30m ago");
        assert_eq!(format_relative_time(now - 3599), "59m ago");
    }

    #[test]
    fn test_format_relative_time_hours() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        assert_eq!(format_relative_time(now - 3600), "1h ago");
        assert_eq!(format_relative_time(now - 7200), "2h ago");
        assert_eq!(format_relative_time(now - 43200), "12h ago");
        assert_eq!(format_relative_time(now - 86399), "23h ago");
    }

    #[test]
    fn test_format_relative_time_days() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        assert_eq!(format_relative_time(now - 86400), "1d ago");
        assert_eq!(format_relative_time(now - 172800), "2d ago");
        assert_eq!(format_relative_time(now - 432000), "5d ago");
        assert_eq!(format_relative_time(now - 604799), "6d ago");
    }

    #[test]
    fn test_format_relative_time_weeks() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        assert_eq!(format_relative_time(now - 604800), "1w ago");
        assert_eq!(format_relative_time(now - 1209600), "2w ago");
        assert_eq!(format_relative_time(now - 2591999), "4w ago");
    }

    #[test]
    fn test_format_relative_time_months() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        assert_eq!(format_relative_time(now - 2592000), "1mo ago");
        assert_eq!(format_relative_time(now - 5184000), "2mo ago");
        assert_eq!(format_relative_time(now - 7776000), "3mo ago");
        assert_eq!(format_relative_time(now - 31536000), "12mo ago");
    }

    #[test]
    fn test_format_relative_time_boundary_conditions() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Exact boundaries
        assert_eq!(format_relative_time(now - 0), "just now");
        assert_eq!(format_relative_time(now - 3600), "1h ago");
        assert_eq!(format_relative_time(now - 86400), "1d ago");
        assert_eq!(format_relative_time(now - 604800), "1w ago");
        assert_eq!(format_relative_time(now - 2592000), "1mo ago");
    }
}
