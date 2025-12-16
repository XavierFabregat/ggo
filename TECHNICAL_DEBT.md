# Technical Debt & Issues

> **Status:** Generated from code review on 2025-12-16
> **Overall Assessment:** B+ (7.5/10) - Solid foundation, needs hardening before public release

---

## Table of Contents

1. [Critical Issues](#critical-issues)
2. [High Priority](#high-priority)
3. [Medium Priority](#medium-priority)
4. [Low Priority](#low-priority)
5. [Quick Wins](#quick-wins)
6. [Long-term Improvements](#long-term-improvements)

---

## Critical Issues

### 🚨 C1: Inconsistent Error Handling Strategy

**Location:** Throughout codebase, especially `main.rs:72-73, 191`

**Problem:**
```rust
// Multiple error handling strategies mixed together:
let repo_path = git::get_repo_root().unwrap_or_default();  // Silent failure
let records = storage::get_branch_records(&repo_path).unwrap_or_default();  // Silent failure

if let Err(e) = storage::record_checkout(&repo_path, &branch_to_checkout) {
    eprintln!("Warning: Failed to record checkout: {}", e);  // Warning but continue
}
```

**Impact:**
- Users don't know when core functionality (frecency tracking) fails
- Empty repo_path causes silent bugs downstream
- Debugging is extremely difficult
- Undermines the core value proposition

**Remedies:**

**Option 1: Fail Fast (Recommended)**
```rust
// main.rs
fn find_and_checkout_branch(...) -> Result<String> {
    let branches = git::get_branches()?;
    let repo_path = git::get_repo_root()
        .context("Failed to determine git repository root")?;
    let records = storage::get_branch_records(&repo_path)
        .context("Failed to load branch history")?;

    // ... rest of function

    // Record or fail
    storage::record_checkout(&repo_path, &branch_to_checkout)
        .context("Failed to update branch usage statistics")?;

    Ok(branch_to_checkout)
}
```

**Option 2: Graceful Degradation with User Notification**
```rust
fn find_and_checkout_branch(...) -> Result<String> {
    let branches = git::get_branches()?;
    let repo_path = git::get_repo_root()
        .context("Not in a git repository")?;

    // Try to load history, but continue without it
    let records = match storage::get_branch_records(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("⚠️  Warning: Could not load branch history: {}", e);
            eprintln!("   Frecency ranking will not be available.");
            vec![]
        }
    };

    // ... checkout logic ...

    // Try to record, warn if it fails
    if let Err(e) = storage::record_checkout(&repo_path, &branch_to_checkout) {
        eprintln!("⚠️  Warning: Could not save branch usage: {}", e);
        eprintln!("   This won't affect branch history.");
    }

    Ok(branch_to_checkout)
}
```

**Estimated Effort:** 3-4 hours

---

### 🚨 C2: Race Condition in Branch Operations

**Location:** `main.rs:183-243`

**Problem:**
```rust
// Line 189: Read branches
let branches = git::get_branches()?;

// ... many lines later ...

// Line 235: Checkout (branch might be deleted in between)
git::checkout(&branch_to_checkout)?;
```

**Impact:**
- Branch could be deleted by another process
- Branch could be renamed
- Uncommitted changes could prevent checkout
- No atomic check-and-checkout

**Remedies:**

**Option 1: Check Before Checkout**
```rust
fn find_and_checkout_branch(...) -> Result<String> {
    let branches = git::get_branches()?;
    // ... selection logic ...

    // Re-verify branch exists before checkout
    let current_branches = git::get_branches()
        .context("Failed to verify branch list")?;

    if !current_branches.contains(&branch_to_checkout) {
        bail!("Branch '{}' no longer exists", branch_to_checkout);
    }

    // Save previous branch
    if let Ok(current_branch) = git::get_current_branch() {
        if current_branch != branch_to_checkout {
            storage::save_previous_branch(&repo_path, &current_branch)?;
        }
    }

    // Attempt checkout
    git::checkout(&branch_to_checkout)
        .context(format!("Failed to checkout branch '{}'", branch_to_checkout))?;

    // Record success
    storage::record_checkout(&repo_path, &branch_to_checkout)?;

    Ok(branch_to_checkout)
}
```

**Option 2: Add Git Status Check**
```rust
// git.rs
pub fn can_checkout() -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;

    // If there are uncommitted changes, warn user
    Ok(output.stdout.is_empty())
}

// Use before checkout:
if !git::can_checkout()? {
    eprintln!("⚠️  Warning: You have uncommitted changes");
    // Optionally prompt user or add --force flag
}
```

**Estimated Effort:** 2 hours

---

### 🚨 C3: No Database Migrations or Versioning

**Location:** `storage.rs:42-58`

**Problem:**
- Schema changes will break existing installations
- No way to upgrade database structure
- No version tracking
- Users would need to delete database and lose history

**Remedies:**

**Option 1: Simple Version Check**
```rust
// storage.rs

const CURRENT_SCHEMA_VERSION: i32 = 1;

fn initialize_tables(conn: &Connection) -> Result<()> {
    // Create version table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        )",
        [],
    )?;

    // Get current version
    let version: i32 = conn
        .query_row(
            "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Run migrations
    if version < CURRENT_SCHEMA_VERSION {
        run_migrations(conn, version)?;
    }

    Ok(())
}

fn run_migrations(conn: &Connection, from_version: i32) -> Result<()> {
    let now = now_timestamp();

    match from_version {
        0 => {
            // Initial schema
            conn.execute(
                "CREATE TABLE IF NOT EXISTS branches (
                    id INTEGER PRIMARY KEY,
                    repo_path TEXT NOT NULL,
                    branch_name TEXT NOT NULL,
                    switch_count INTEGER DEFAULT 1,
                    last_used INTEGER NOT NULL,
                    UNIQUE(repo_path, branch_name)
                )",
                [],
            )?;

            // Add indices for performance
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_repo_last_used
                 ON branches(repo_path, last_used DESC)",
                [],
            )?;

            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_repo_branch
                 ON branches(repo_path, branch_name)",
                [],
            )?;

            conn.execute(
                "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
                [&CURRENT_SCHEMA_VERSION.to_string(), &now.to_string()],
            )?;
        }
        // Future migrations go here
        // 1 => { /* migration from v1 to v2 */ }
        _ => {}
    }

    Ok(())
}
```

**Option 2: Use Migration Crate**
```toml
[dependencies]
rusqlite_migration = "1.0"
```

```rust
use rusqlite_migration::{Migrations, M};

fn initialize_tables(conn: &Connection) -> Result<()> {
    let migrations = Migrations::new(vec![
        M::up("CREATE TABLE branches (...)"),
        M::up("CREATE INDEX idx_repo_last_used ..."),
        // Future migrations
    ]);

    migrations.to_latest(conn)
        .context("Failed to run database migrations")?;

    Ok(())
}
```

**Estimated Effort:** 3-4 hours

---

### 🚨 C4: Missing User Documentation

**Location:** No README.md

**Problem:**
- Users don't know how to install
- No usage examples
- No troubleshooting guide
- No contribution guidelines

**Remedies:**

Create `README.md`:
```markdown
# ggo - Smart Git Navigation

A zoxide-style tool for intelligent git branch navigation with frecency-based ranking.

## Installation

### From Source
```bash
git clone https://github.com/yourusername/ggo.git
cd ggo
cargo install --path .
```

### Binary Release
Download from [Releases](https://github.com/yourusername/ggo/releases)

## Quick Start

```bash
# Checkout branch with fuzzy matching + frecency
ggo feat

# List matching branches
ggo -l feature

# Go back to previous branch
ggo -

# Show statistics
ggo --stats
```

## How It Works

ggo learns from your branch usage patterns...

[See ROADMAP.md for full feature set]

## Configuration

Database location: `~/.config/ggo/data.db`

## Troubleshooting

**"Not a git repository"**
- Run ggo from within a git repository

**Branch history not tracking**
- Check database permissions...

## License

[Add license]

## Contributing

[See CONTRIBUTING.md]
```

**Estimated Effort:** 1-2 hours

---

## High Priority

### H1: Git Operations Performance & Reliability

**Location:** `git.rs` (entire file)

**Problem:**
1. Shelling out to git is slow and unreliable
2. No git version checking
3. No timeout handling
4. stderr is often discarded
5. Output encoding issues with `from_utf8_lossy`

**Current Code:**
```rust
pub fn get_branches() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["branch"])
        .output()
        .context("Failed to execute git branch")?;

    if !output.status.success() {
        bail!("Not a git repository or git command failed");
    }

    let branches: Vec<String> = output
        .stdout
        .lines()
        .map_while(Result::ok)
        .map(|line| line.trim().trim_start_matches('*').trim().to_string())
        .collect();

    Ok(branches)
}
```

**Remedies:**

**Option 1: Add Timeout and Better Error Messages**
```rust
use std::time::Duration;
use std::process::Stdio;

pub fn get_branches() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["branch", "--list"])  // More explicit
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to execute git command. Is git installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Git command failed: {}", stderr.trim());
    }

    // Validate UTF-8
    let stdout = String::from_utf8(output.stdout)
        .context("Git output contains invalid UTF-8. Branch names must be valid UTF-8.")?;

    let branches: Vec<String> = stdout
        .lines()
        .map(|line| {
            line.trim()
                .trim_start_matches('*')
                .trim()
                .to_string()
        })
        .filter(|b| !b.is_empty())
        .collect();

    Ok(branches)
}
```

**Option 2: Switch to git2 (libgit2)**
```toml
[dependencies]
git2 = "0.18"
```

```rust
use git2::Repository;

pub fn get_branches() -> Result<Vec<String>> {
    let repo = Repository::open_from_env()
        .context("Not a git repository")?;

    let branches = repo.branches(Some(git2::BranchType::Local))?
        .filter_map(|b| b.ok())
        .filter_map(|(branch, _)| branch.name().ok().flatten())
        .map(String::from)
        .collect();

    Ok(branches)
}

pub fn checkout(branch: &str) -> Result<()> {
    let repo = Repository::open_from_env()
        .context("Not a git repository")?;

    // Validate branch name
    let refname = format!("refs/heads/{}", branch);
    let obj = repo.revparse_single(&refname)
        .context(format!("Branch '{}' not found", branch))?;

    repo.checkout_tree(&obj, None)?;
    repo.set_head(&refname)?;

    Ok(())
}

pub fn get_current_branch() -> Result<String> {
    let repo = Repository::open_from_env()
        .context("Not a git repository")?;

    let head = repo.head()
        .context("Could not get HEAD reference")?;

    if !head.is_branch() {
        bail!("Not on a branch (detached HEAD)");
    }

    let branch_name = head
        .shorthand()
        .ok_or_else(|| anyhow::anyhow!("Invalid branch name"))?;

    Ok(branch_name.to_string())
}

pub fn get_repo_root() -> Result<String> {
    let repo = Repository::open_from_env()
        .context("Not a git repository")?;

    let workdir = repo.workdir()
        .ok_or_else(|| anyhow::anyhow!("Repository has no working directory"))?;

    Ok(workdir.to_string_lossy().to_string())
}
```

**Pros of git2:**
- Faster (no process spawning)
- More reliable
- Better error messages
- Type-safe API
- Cross-platform

**Cons:**
- Larger dependency
- More complex API
- Requires learning libgit2

**Estimated Effort:**
- Option 1: 2 hours
- Option 2: 4-6 hours

---

### H2: Database Performance Issues

**Location:** `storage.rs`

**Problem:**
1. Opens new connection on every operation
2. No connection pooling
3. Missing database indices
4. Inefficient previous_branch table creation on every call

**Current Code:**
```rust
pub fn record_checkout(repo_path: &str, branch_name: &str) -> Result<()> {
    let conn = open_db()?;  // ❌ New connection every time
    let now = now_timestamp();

    conn.execute(
        "INSERT INTO branches (repo_path, branch_name, switch_count, last_used)
         VALUES (?1, ?2, 1, ?3)
         ON CONFLICT(repo_path, branch_name) DO UPDATE SET
            switch_count = switch_count + 1,
            last_used = ?3",
        [repo_path, branch_name, &now.to_string()],
    )?;

    Ok(())
}
```

**Remedies:**

**Option 1: Add Indices (Quick Win)**
```rust
fn initialize_tables(conn: &Connection) -> Result<()> {
    // Create tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS branches (
            id INTEGER PRIMARY KEY,
            repo_path TEXT NOT NULL,
            branch_name TEXT NOT NULL,
            switch_count INTEGER DEFAULT 1,
            last_used INTEGER NOT NULL,
            UNIQUE(repo_path, branch_name)
        )",
        [],
    )?;

    // Add indices for common queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_repo_last_used
         ON branches(repo_path, last_used DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_last_used
         ON branches(last_used DESC)",
        [],
    )?;

    // Create previous_branch table once
    conn.execute(
        "CREATE TABLE IF NOT EXISTS previous_branch (
            repo_path TEXT PRIMARY KEY,
            branch_name TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;

    Ok(())
}
```

**Option 2: Connection Pooling**
```rust
use std::sync::Mutex;
use once_cell::sync::Lazy;

static DB_CONN: Lazy<Mutex<Connection>> = Lazy::new(|| {
    Mutex::new(open_db().expect("Failed to open database"))
});

pub fn record_checkout(repo_path: &str, branch_name: &str) -> Result<()> {
    let conn = DB_CONN.lock().unwrap();
    let now = now_timestamp();

    conn.execute(
        "INSERT INTO branches (repo_path, branch_name, switch_count, last_used)
         VALUES (?1, ?2, 1, ?3)
         ON CONFLICT(repo_path, branch_name) DO UPDATE SET
            switch_count = switch_count + 1,
            last_used = ?3",
        [repo_path, branch_name, &now.to_string()],
    )?;

    Ok(())
}
```

**Option 3: Batch Operations**
```rust
// For multiple updates, use transactions
pub fn record_checkouts(updates: &[(String, String)]) -> Result<()> {
    let conn = open_db()?;
    let tx = conn.transaction()?;
    let now = now_timestamp();

    for (repo_path, branch_name) in updates {
        tx.execute(
            "INSERT INTO branches (repo_path, branch_name, switch_count, last_used)
             VALUES (?1, ?2, 1, ?3)
             ON CONFLICT(repo_path, branch_name) DO UPDATE SET
                switch_count = switch_count + 1,
                last_used = ?3",
            [repo_path, branch_name, &now.to_string()],
        )?;
    }

    tx.commit()?;
    Ok(())
}
```

**Estimated Effort:**
- Option 1: 30 minutes (do this NOW)
- Option 2: 2 hours
- Option 3: 1 hour

---

### H3: Input Validation & Security

**Location:** Multiple files

**Problem:**
1. No validation of branch names
2. No validation of repo paths
3. Potential command injection (mitigated by using .args(), but still concerning)
4. No sanitization of user input

**Remedies:**

**Add Input Validation Module:**
```rust
// src/validation.rs

use anyhow::{bail, Result};
use std::path::Path;

/// Validate that a branch name is safe and valid
pub fn validate_branch_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Branch name cannot be empty");
    }

    if name.len() > 255 {
        bail!("Branch name too long (max 255 characters)");
    }

    // Check for dangerous characters
    let dangerous_chars = ['\0', '\n', '\r'];
    if name.chars().any(|c| dangerous_chars.contains(&c)) {
        bail!("Branch name contains invalid characters");
    }

    // Git branch name restrictions
    if name.starts_with('-') {
        bail!("Branch name cannot start with '-'");
    }

    if name.contains("..") {
        bail!("Branch name cannot contain '..'");
    }

    if name.ends_with('/') || name.ends_with('.') {
        bail!("Branch name cannot end with '/' or '.'");
    }

    Ok(())
}

/// Validate that a repo path is safe and valid
pub fn validate_repo_path(path: &str) -> Result<()> {
    if path.is_empty() {
        bail!("Repository path cannot be empty");
    }

    let path_obj = Path::new(path);

    // Must be absolute path
    if !path_obj.is_absolute() {
        bail!("Repository path must be absolute");
    }

    // Must exist
    if !path_obj.exists() {
        bail!("Repository path does not exist");
    }

    // Must be a directory
    if !path_obj.is_dir() {
        bail!("Repository path is not a directory");
    }

    Ok(())
}

/// Validate search pattern
pub fn validate_pattern(pattern: &str) -> Result<()> {
    if pattern.len() > 255 {
        bail!("Search pattern too long (max 255 characters)");
    }

    // Check for null bytes and other dangerous characters
    if pattern.contains('\0') {
        bail!("Search pattern contains null bytes");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_branch_name() {
        assert!(validate_branch_name("feature/auth").is_ok());
        assert!(validate_branch_name("main").is_ok());
        assert!(validate_branch_name("").is_err());
        assert!(validate_branch_name("-bad").is_err());
        assert!(validate_branch_name("has..dots").is_err());
        assert!(validate_branch_name("trailing/").is_err());
    }

    #[test]
    fn test_validate_pattern() {
        assert!(validate_pattern("feat").is_ok());
        assert!(validate_pattern("feature/").is_ok());
        assert!(validate_pattern(&"a".repeat(256)).is_err());
    }
}
```

**Use Validation:**
```rust
// git.rs
pub fn checkout(branch: &str) -> Result<()> {
    validation::validate_branch_name(branch)
        .context("Invalid branch name")?;

    let output = Command::new("git")
        .args(["checkout", branch])
        .output()
        .context("Failed to execute git checkout")?;

    // ... rest of function
}

// main.rs
fn find_and_checkout_branch(pattern: &str, ...) -> Result<String> {
    validation::validate_pattern(pattern)
        .context("Invalid search pattern")?;

    // ... rest of function
}
```

**Estimated Effort:** 3 hours

---

## Medium Priority

### M1: Frecency Algorithm Improvements

**Location:** `frecency.rs:14-40`

**Problem:**
- Stepped decay instead of exponential
- Hardcoded, non-configurable weights
- No consideration of branch diversity
- Arbitrary weight values

**Current Code:**
```rust
pub fn calculate_score(record: &BranchRecord) -> f64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let age_seconds = now - record.last_used;

    let recency_weight = if age_seconds < 3600 {
        4.0
    } else if age_seconds < 86400 {
        2.0
    } else if age_seconds < 604800 {
        1.0
    } else if age_seconds < 2592000 {
        0.5
    } else {
        0.25
    };

    record.switch_count as f64 * recency_weight
}
```

**Remedies:**

**Option 1: Exponential Decay (Like zoxide)**
```rust
// frecency.rs

/// Half-life in seconds (default: 1 week)
const HALF_LIFE_SECONDS: f64 = 604800.0;

/// Calculate frecency score using exponential decay
///
/// Formula: score = frequency × exp(-λ × age)
/// where λ = ln(2) / half_life
pub fn calculate_score(record: &BranchRecord) -> f64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as f64;

    let age_seconds = now - record.last_used as f64;

    // Decay constant (lambda)
    let lambda = (2.0_f64).ln() / HALF_LIFE_SECONDS;

    // Exponential decay: e^(-λt)
    let recency_weight = (-lambda * age_seconds).exp();

    // Combine frequency and recency
    record.switch_count as f64 * recency_weight
}
```

**Option 2: Configurable Weights**
```rust
// frecency.rs

pub struct FrecencyConfig {
    pub hour_weight: f64,
    pub day_weight: f64,
    pub week_weight: f64,
    pub month_weight: f64,
    pub old_weight: f64,
}

impl Default for FrecencyConfig {
    fn default() -> Self {
        Self {
            hour_weight: 4.0,
            day_weight: 2.0,
            week_weight: 1.0,
            month_weight: 0.5,
            old_weight: 0.25,
        }
    }
}

pub fn calculate_score_with_config(
    record: &BranchRecord,
    config: &FrecencyConfig,
) -> f64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let age_seconds = now - record.last_used;

    let recency_weight = if age_seconds < 3600 {
        config.hour_weight
    } else if age_seconds < 86400 {
        config.day_weight
    } else if age_seconds < 604800 {
        config.week_weight
    } else if age_seconds < 2592000 {
        config.month_weight
    } else {
        config.old_weight
    };

    record.switch_count as f64 * recency_weight
}

// Keep existing function for backward compatibility
pub fn calculate_score(record: &BranchRecord) -> f64 {
    calculate_score_with_config(record, &FrecencyConfig::default())
}
```

**Option 3: Hybrid Approach**
```rust
/// Calculate frecency with both frequency and diversity bonuses
pub fn calculate_score_advanced(
    record: &BranchRecord,
    total_branches: usize,
) -> f64 {
    let base_score = calculate_score(record);

    // Diversity bonus: reduce score for repos with many branches
    // (encourages focusing on frequently-used branches)
    let diversity_factor = if total_branches > 10 {
        1.0 / (1.0 + (total_branches as f64 - 10.0) * 0.05)
    } else {
        1.0
    };

    base_score * diversity_factor
}
```

**Estimated Effort:** 2-3 hours

---

### M2: Duplicate Code in Tests

**Location:** `storage.rs:260-378`, `git.rs:122-293`

**Problem:**
- Test helper functions duplicated across modules
- Reduces maintainability
- Inconsistent test setup

**Remedies:**

**Create Test Utilities Module:**
```rust
// tests/common/mod.rs

use std::process::Command;
use std::fs;
use std::path::Path;
use anyhow::Result;

/// Setup a temporary git repository for testing
pub fn setup_test_repo() -> Result<tempfile::TempDir> {
    let temp_dir = tempfile::tempdir()?;
    let repo_path = temp_dir.path();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()?;

    // Configure git for tests
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()?;

    // Create initial commit
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "test content")?;

    Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()?;

    Ok(temp_dir)
}

/// Create a branch in the test repository
pub fn create_branch(repo_path: &Path, branch_name: &str) -> Result<()> {
    Command::new("git")
        .args(["branch", branch_name])
        .current_dir(repo_path)
        .output()?;
    Ok(())
}

/// Generate a unique repo path for testing
pub fn unique_repo_path() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("/test/repo/{}", id)
}
```

**Use in Tests:**
```rust
// storage.rs tests
#[cfg(test)]
mod tests {
    use super::*;

    // Import common test utilities
    use crate::tests::common::{setup_test_repo, unique_repo_path};

    #[test]
    fn test_record_checkout() {
        let repo_path = unique_repo_path();
        // ... test implementation
    }
}
```

**Estimated Effort:** 1 hour

---

### M3: Hardcoded Constants

**Location:** Multiple files

**Problem:**
- Magic numbers throughout code
- Fuzzy + frecency multiplier hardcoded
- Time windows hardcoded
- Display widths hardcoded

**Remedies:**

**Create Constants Module:**
```rust
// src/constants.rs

/// Frecency scoring constants
pub mod frecency {
    pub const HOUR_SECONDS: i64 = 3600;
    pub const DAY_SECONDS: i64 = 86400;
    pub const WEEK_SECONDS: i64 = 604800;
    pub const MONTH_SECONDS: i64 = 2592000;

    pub const HOUR_WEIGHT: f64 = 4.0;
    pub const DAY_WEIGHT: f64 = 2.0;
    pub const WEEK_WEIGHT: f64 = 1.0;
    pub const MONTH_WEIGHT: f64 = 0.5;
    pub const OLD_WEIGHT: f64 = 0.25;
}

/// Scoring combination constants
pub mod scoring {
    /// Multiplier for frecency when combining with fuzzy match scores
    pub const FRECENCY_MULTIPLIER: f64 = 10.0;
}

/// Display constants
pub mod display {
    pub const MAX_BRANCH_NAME_WIDTH: usize = 40;
    pub const INTERACTIVE_PAGE_SIZE: usize = 15;
    pub const TABLE_SEPARATOR_WIDTH: usize = 85;
}

/// Database constants
pub mod database {
    pub const SCHEMA_VERSION: i32 = 1;
    pub const MAX_BRANCH_NAME_LENGTH: usize = 255;
    pub const MAX_REPO_PATH_LENGTH: usize = 4096;
}
```

**Update Code to Use Constants:**
```rust
// main.rs
use crate::constants::scoring::FRECENCY_MULTIPLIER;

fn combine_fuzzy_and_frecency_scores(...) -> Vec<(String, f64)> {
    let combined_score = fuzzy_score + (frecency_score * FRECENCY_MULTIPLIER);
    // ...
}

// frecency.rs
use crate::constants::frecency::*;

pub fn calculate_score(record: &BranchRecord) -> f64 {
    let age_seconds = now - record.last_used;

    let recency_weight = if age_seconds < HOUR_SECONDS {
        HOUR_WEIGHT
    } else if age_seconds < DAY_SECONDS {
        DAY_WEIGHT
    } // ...
}
```

**Estimated Effort:** 1-2 hours

---

### M4: Unbounded Database Growth

**Location:** `storage.rs`

**Problem:**
- No cleanup of old branches
- No limit on stored records
- Database will grow indefinitely
- No VACUUM or maintenance

**Remedies:**

**Add Cleanup Function:**
```rust
// storage.rs

/// Remove branch records older than the specified age
pub fn cleanup_old_records(max_age_days: i64) -> Result<()> {
    let conn = open_db()?;
    let now = now_timestamp();
    let cutoff = now - (max_age_days * 86400);

    conn.execute(
        "DELETE FROM branches WHERE last_used < ?1",
        [cutoff],
    )?;

    Ok(())
}

/// Remove branches that no longer exist in their repositories
pub fn cleanup_deleted_branches() -> Result<usize> {
    let conn = open_db()?;
    let records = get_all_records()?;

    let mut deleted = 0;

    for record in records {
        // Check if repo still exists
        if let Ok(repo_branches) = crate::git::get_branches_for_repo(&record.repo_path) {
            // Check if branch still exists
            if !repo_branches.contains(&record.branch_name) {
                conn.execute(
                    "DELETE FROM branches WHERE repo_path = ?1 AND branch_name = ?2",
                    [&record.repo_path, &record.branch_name],
                )?;
                deleted += 1;
            }
        }
    }

    Ok(deleted)
}

/// Optimize database (VACUUM and ANALYZE)
pub fn optimize_database() -> Result<()> {
    let conn = open_db()?;
    conn.execute("VACUUM", [])?;
    conn.execute("ANALYZE", [])?;
    Ok(())
}

/// Get database size in bytes
pub fn get_database_size() -> Result<u64> {
    let db_path = get_db_path()?;
    let metadata = std::fs::metadata(db_path)?;
    Ok(metadata.len())
}
```

**Add Automatic Cleanup:**
```rust
// main.rs or storage.rs

/// Run maintenance tasks if needed
fn maybe_run_maintenance() -> Result<()> {
    // Check if we've run maintenance recently
    let conn = open_db()?;

    let last_maintenance: Option<i64> = conn.query_row(
        "SELECT value FROM metadata WHERE key = 'last_maintenance'",
        [],
        |row| row.get(0),
    ).ok();

    let now = now_timestamp();
    let should_run = match last_maintenance {
        None => true,
        Some(last) => (now - last) > 86400 * 7, // Weekly
    };

    if should_run {
        // Cleanup old records (>1 year)
        cleanup_old_records(365)?;

        // Optimize database
        optimize_database()?;

        // Update last maintenance time
        conn.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES ('last_maintenance', ?1)",
            [now],
        )?;
    }

    Ok(())
}
```

**Estimated Effort:** 2-3 hours

---

### M5: No Logging Framework

**Location:** Throughout codebase

**Problem:**
- Uses println! and eprintln! everywhere
- No structured logging
- No log levels
- Difficult to debug in production

**Remedies:**

**Add Tracing:**
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

```rust
// main.rs

use tracing::{info, warn, error, debug};
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    let cli = Cli::parse();

    info!("Starting ggo");
    debug!("CLI args: {:?}", cli);

    // ... rest of main
}

// Throughout code, replace:
// println!("...") -> info!(...)
// eprintln!("Warning: ...") -> warn!(...)
// eprintln!("Error: ...") -> error!(...)
```

**Usage:**
```bash
# Normal operation (info only)
ggo feat

# Verbose logging
RUST_LOG=debug ggo feat

# Very verbose
RUST_LOG=trace ggo feat
```

**Estimated Effort:** 2 hours

---

## Low Priority

### L1: Missing License File

**Location:** Root directory

**Problem:**
- No LICENSE file
- Legal ambiguity
- Can't be used in production

**Remedies:**

Choose and add a license:

**MIT License (Permissive):**
```
Create LICENSE file with MIT text
```

**Apache 2.0 (Patent protection):**
```
Create LICENSE file with Apache 2.0 text
```

**GPL v3 (Copyleft):**
```
Create LICENSE file with GPL v3 text
```

**Add to Cargo.toml:**
```toml
[package]
license = "MIT"
# or
license = "Apache-2.0"
# or
license = "GPL-3.0"
```

**Estimated Effort:** 5 minutes

---

### L2: Shell Completions

**Location:** Missing

**Problem:**
- No shell completions for bash/zsh/fish
- Users need to type full flags

**Remedies:**

**Add Completions:**
```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
clap_complete = "4.5"
```

```rust
// src/completions.rs

use clap::CommandFactory;
use clap_complete::{generate_to, shells};
use std::env;
use std::io::Error;

pub fn generate_completions() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = crate::cli::Cli::command();
    let bin_name = env!("CARGO_PKG_NAME");

    generate_to(shells::Bash, &mut cmd, bin_name, &outdir)?;
    generate_to(shells::Zsh, &mut cmd, bin_name, &outdir)?;
    generate_to(shells::Fish, &mut cmd, bin_name, &outdir)?;

    Ok(())
}
```

**Installation instructions in README:**
```bash
# Bash
ggo --generate-completion bash > /usr/local/share/bash-completion/completions/ggo

# Zsh
ggo --generate-completion zsh > /usr/local/share/zsh/site-functions/_ggo

# Fish
ggo --generate-completion fish > ~/.config/fish/completions/ggo.fish
```

**Estimated Effort:** 1-2 hours

---

### L3: Better Error Types

**Location:** Throughout codebase

**Problem:**
- Using anyhow::Error everywhere
- No structured error types
- Can't match on specific errors

**Remedies:**

**Create Error Types:**
```toml
[dependencies]
thiserror = "1.0"
```

```rust
// src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GgoError {
    #[error("Not in a git repository")]
    NotGitRepository,

    #[error("Branch '{0}' not found")]
    BranchNotFound(String),

    #[error("No branches match pattern '{0}'")]
    NoMatchingBranches(String),

    #[error("Git command failed: {0}")]
    GitCommandFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Invalid branch name: {0}")]
    InvalidBranchName(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("User cancelled operation")]
    UserCancelled,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, GgoError>;
```

**Use in Code:**
```rust
// git.rs
use crate::error::{GgoError, Result};

pub fn get_branches() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["branch"])
        .output()
        .map_err(|_| GgoError::NotGitRepository)?;

    if !output.status.success() {
        return Err(GgoError::GitCommandFailed(
            "Failed to list branches".to_string()
        ));
    }

    // ...
}

pub fn checkout(branch: &str) -> Result<()> {
    // Validate first
    if branch.is_empty() {
        return Err(GgoError::InvalidBranchName(
            "Branch name cannot be empty".to_string()
        ));
    }

    let output = Command::new("git")
        .args(["checkout", branch])
        .output()
        .map_err(|e| GgoError::IoError(e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("did not match any") {
            return Err(GgoError::BranchNotFound(branch.to_string()));
        }
        return Err(GgoError::GitCommandFailed(stderr.to_string()));
    }

    Ok(())
}
```

**Better Error Messages:**
```rust
// main.rs
fn main() {
    if let Err(e) = run() {
        match e {
            GgoError::NotGitRepository => {
                eprintln!("❌ Error: Not in a git repository");
                eprintln!("   Run this command from within a git repository.");
                std::process::exit(1);
            }
            GgoError::BranchNotFound(branch) => {
                eprintln!("❌ Error: Branch '{}' not found", branch);
                eprintln!("   Run 'git branch' to see available branches.");
                std::process::exit(1);
            }
            _ => {
                eprintln!("❌ Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
```

**Estimated Effort:** 3-4 hours

---

## Quick Wins

These can be done in under 30 minutes each:

### QW1: Add LICENSE File
Create LICENSE file with chosen license (5 minutes)

### QW2: Add .gitattributes
```
# .gitattributes
* text=auto eol=lf
*.rs text
*.toml text
*.md text
Cargo.lock binary
```

### QW3: Add Database Indices
Already covered in H2, but worth emphasizing as quick win (30 minutes)

### QW4: Fix Version Display
```rust
// cli.rs
#[derive(Parser)]
#[command(name = "ggo")]
#[command(version = env!("CARGO_PKG_VERSION"))]  // Show actual version
#[command(about = "Smart Git Navigation Tool", long_about = None)]
pub struct Cli {
    // ...
}
```

### QW5: Add .editorconfig
```ini
# .editorconfig
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true

[*.rs]
indent_style = space
indent_size = 4

[*.toml]
indent_style = space
indent_size = 2

[*.md]
indent_style = space
indent_size = 2
trim_trailing_whitespace = false
```

### QW6: Add Rust Toolchain File
```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
```

### QW7: Improve Error Message Format
Add emoji and better formatting to error messages (15 minutes)

---

## Long-term Improvements

### LT1: Repository Tracking (Phase 4)
Implement multi-repository navigation from ROADMAP

### LT2: Branch Aliases (Phase 5)
Allow custom shortcuts for common branches

### LT3: Git Hooks Integration
Auto-track on any branch switch

### LT4: Configuration File
Support `~/.config/ggo/config.toml` for user preferences

### LT5: Statistics Visualization
Add charts/graphs for branch usage patterns

### LT6: Team Sync
Export/import frecency data for teams

### LT7: Remote Branch Support
Track and switch to remote branches

### LT8: Performance Benchmarks
Add criterion benchmarks for critical paths

### LT9: Integration Tests for Edge Cases
More comprehensive integration testing

### LT10: Cross-platform CI
Add Windows and macOS to CI pipeline

---

## Priority Order for Implementation

**Week 1 (Critical):**
1. Add README.md (C4) - 2h
2. Add LICENSE (L1) - 5m
3. Fix error handling (C1) - 4h
4. Add database indices (H2 Option 1) - 30m
5. Fix database migrations (C3) - 4h

**Week 2 (High Priority):**
6. Add input validation (H3) - 3h
7. Improve git operations (H1 Option 1) - 2h
8. Fix race condition (C2) - 2h
9. Add test utilities (M2) - 1h

**Week 3 (Polish):**
10. Add constants module (M3) - 2h
11. Add logging (M5) - 2h
12. Add database cleanup (M4) - 3h
13. Shell completions (L2) - 2h

**Week 4 (Nice to Have):**
14. Better error types (L3) - 4h
15. Improve frecency algorithm (M1) - 3h
16. Quick wins (QW1-QW7) - 2h

---

## Testing Checklist

After implementing fixes, verify:

- [ ] All tests pass
- [ ] Clippy shows no warnings
- [ ] Code is formatted (cargo fmt)
- [ ] Documentation builds (cargo doc)
- [ ] Binary builds in release mode
- [ ] README is accurate
- [ ] License is added
- [ ] CI passes on all branches
- [ ] Manual testing of core workflows:
  - [ ] Basic checkout works
  - [ ] Fuzzy matching works
  - [ ] Interactive mode works
  - [ ] Statistics display works
  - [ ] Previous branch (`-`) works
  - [ ] Error messages are helpful

---

## Questions for Discussion

1. **Git2 vs Shelling Out:** Should we switch to libgit2 or stick with git commands?
2. **Error Handling Philosophy:** Fail fast or graceful degradation?
3. **Frecency Algorithm:** Keep simple or implement exponential decay?
4. **Logging:** Add tracing or keep simple?
5. **License Choice:** MIT, Apache 2.0, or GPL?
6. **Public Release Timeline:** When do we want to release v1.0?

---

## Metrics to Track

Once fixes are implemented, track:
- Test coverage percentage
- Average response time for common operations
- Database size growth rate
- Error rate in production
- User feedback and bug reports

---

*Document generated: 2025-12-16*
*Next review: After Phase 4 completion*
