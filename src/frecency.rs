use crate::constants::frecency::*;
use crate::storage::BranchRecord;
use std::time::{SystemTime, UNIX_EPOCH};

/// Half-life for exponential decay (1 week in seconds)
/// After this duration, a branch's recency weight is halved
const HALF_LIFE_SECONDS: f64 = 604800.0; // 1 week

/// Calculate the frecency score for a branch record using exponential decay.
///
/// Frecency = frequency × exp(-λ × age)
/// where λ = ln(2) / half_life
///
/// This provides smooth decay instead of stepped tiers, more similar to zoxide's algorithm.
/// The half-life is 1 week, meaning a branch's recency weight halves each week.
pub fn calculate_score(record: &BranchRecord) -> f64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as f64;

    let age_seconds = now - record.last_used as f64;

    // Decay constant (lambda) = ln(2) / half_life
    let lambda = 2.0_f64.ln() / HALF_LIFE_SECONDS;

    // Exponential decay: e^(-λt)
    // This gives smooth decay: 1.0 at t=0, 0.5 at t=half_life, 0.25 at t=2*half_life, etc.
    let recency_weight = (-lambda * age_seconds).exp();

    // Multiply frequency by decayed recency weight
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
    } else if age_seconds < HOUR_SECONDS {
        let mins = age_seconds / 60;
        format!("{}m ago", mins)
    } else if age_seconds < DAY_SECONDS {
        let hours = age_seconds / HOUR_SECONDS;
        format!("{}h ago", hours)
    } else if age_seconds < WEEK_SECONDS {
        let days = age_seconds / DAY_SECONDS;
        format!("{}d ago", days)
    } else if age_seconds < MONTH_SECONDS {
        let weeks = age_seconds / WEEK_SECONDS;
        format!("{}w ago", weeks)
    } else {
        let months = age_seconds / MONTH_SECONDS;
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
        // With exponential decay, very recent items have weight ~1.0
        // 10 * ~1.0 ≈ 10.0
        assert!(score > 9.9 && score < 10.1);
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
        // With exponential decay, 1 hour old has weight ~0.999
        // 5 * ~0.999 ≈ 5.0
        assert!(score > 4.9 && score < 5.1);
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
        // With exponential decay, 12 hours (~7% of half-life) has weight ~0.95
        // 8 * ~0.95 ≈ 7.6
        assert!(score > 7.5 && score < 7.7);
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
        // 3 days is ~0.43 of half-life, weight ≈ 0.75
        // 6 * ~0.75 ≈ 4.5
        assert!(score > 4.4 && score < 4.7);
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
            last_used: now - 1209600, // 14 days ago (2 weeks = 2 half-lives)
        };

        let score = calculate_score(&record);
        // 2 half-lives means weight = 0.25
        // 4 * 0.25 = 1.0
        assert!(score > 0.9 && score < 1.1);
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
            last_used: now - 3000000, // ~35 days ago (~5 half-lives)
        };

        let score = calculate_score(&record);
        // 5 half-lives: weight ≈ 0.03125 (1/32)
        // 10 * 0.03125 ≈ 0.31
        assert!(score > 0.3 && score < 0.35);
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
        assert_eq!(score, 0.0); // 0 * any_weight = 0
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
        // Score should be ~5.0 (5 switches * ~1.0 weight for very recent)
        assert!(ranked[0].score > 4.9 && ranked[0].score < 5.1);
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
                last_used: now - 3000000, // ~35 days: weight ≈ 0.03
            },
            BranchRecord {
                repo_path: "/test".to_string(),
                branch_name: "recent".to_string(),
                switch_count: 5,
                last_used: now - 60, // Recent: weight ≈ 1.0
            },
            BranchRecord {
                repo_path: "/test".to_string(),
                branch_name: "medium".to_string(),
                switch_count: 3,
                last_used: now - 43200, // 12 hours: weight ≈ 0.99
            },
        ];

        let ranked = rank_branches(&records);
        assert_eq!(ranked.len(), 3);
        // Should be sorted by score (highest first)
        assert_eq!(ranked[0].name, "recent");
        assert!(ranked[0].score > 4.9); // ~5.0
        assert_eq!(ranked[1].name, "medium");
        assert!(ranked[1].score > 2.8 && ranked[1].score < 2.9); // ~2.86
        assert_eq!(ranked[2].name, "old");
        assert!(ranked[2].score > 0.3 && ranked[2].score < 0.35); // ~0.31
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
                last_used: now - 60, // weight ≈ 1.0, score ≈ 10.0
            },
            BranchRecord {
                repo_path: "/test".to_string(),
                branch_name: "main".to_string(),
                switch_count: 5,
                last_used: now - 43200, // 12h: weight ≈ 0.99, score ≈ 5.0
            },
        ];

        let sorted = sort_branches_by_frecency(&branches, &records);
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].0, "develop");
        assert!(sorted[0].1 > 9.9 && sorted[0].1 < 10.1);
        assert_eq!(sorted[1].0, "main");
        assert!(sorted[1].1 > 4.7 && sorted[1].1 < 4.8);
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
        assert_eq!(format_relative_time(now), "just now");
        assert_eq!(format_relative_time(now - 3600), "1h ago");
        assert_eq!(format_relative_time(now - 86400), "1d ago");
        assert_eq!(format_relative_time(now - 604800), "1w ago");
        assert_eq!(format_relative_time(now - 2592000), "1mo ago");
    }
}
