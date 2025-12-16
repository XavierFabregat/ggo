/// Frecency scoring time windows (in seconds)
/// Note: Weights are no longer used as we switched to exponential decay
/// These constants remain for format_relative_time() function
pub mod frecency {
    /// One hour in seconds
    pub const HOUR_SECONDS: i64 = 3600;

    /// One day in seconds
    pub const DAY_SECONDS: i64 = 86400;

    /// One week in seconds
    pub const WEEK_SECONDS: i64 = 604800;

    /// One month in seconds (30 days)
    pub const MONTH_SECONDS: i64 = 2592000;
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
