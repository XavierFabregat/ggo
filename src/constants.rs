/// Frecency scoring time windows (in seconds)
pub mod frecency {
    /// One hour in seconds
    pub const HOUR_SECONDS: i64 = 3600;

    /// One day in seconds
    pub const DAY_SECONDS: i64 = 86400;

    /// One week in seconds
    pub const WEEK_SECONDS: i64 = 604800;

    /// One month in seconds (30 days)
    pub const MONTH_SECONDS: i64 = 2592000;

    /// Weight for branches used within the last hour
    pub const HOUR_WEIGHT: f64 = 4.0;

    /// Weight for branches used within the last day
    pub const DAY_WEIGHT: f64 = 2.0;

    /// Weight for branches used within the last week
    pub const WEEK_WEIGHT: f64 = 1.0;

    /// Weight for branches used within the last month
    pub const MONTH_WEIGHT: f64 = 0.5;

    /// Weight for branches older than a month
    pub const OLD_WEIGHT: f64 = 0.25;
}

/// Scoring combination constants
pub mod scoring {
    /// Multiplier for frecency when combining with fuzzy match scores
    /// Higher value gives more weight to frecency over fuzzy match quality
    pub const FRECENCY_MULTIPLIER: f64 = 10.0;

    /// Threshold ratio for auto-selecting a branch without showing menu
    /// If top score is this many times higher than second, auto-select
    pub const AUTO_SELECT_THRESHOLD: f64 = 2.0;
}

/// Validation limits
pub mod validation {
    /// Maximum length for branch names (git limit)
    pub const MAX_BRANCH_NAME_LENGTH: usize = 255;

    /// Maximum length for search patterns
    pub const MAX_PATTERN_LENGTH: usize = 255;

    /// Maximum length for alias names
    pub const MAX_ALIAS_LENGTH: usize = 50;

    /// Maximum length for repository paths
    pub const MAX_REPO_PATH_LENGTH: usize = 4096;
}

/// Database schema version
pub mod database {
    /// Current database schema version
    /// Increment this when making schema changes
    pub const SCHEMA_VERSION: i32 = 2;
}
