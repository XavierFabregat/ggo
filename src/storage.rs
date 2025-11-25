use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Branch usage record from the database
#[derive(Debug, Clone)]
pub struct BranchRecord {
    pub repo_path: String,
    pub branch_name: String,
    pub switch_count: i64,
    pub last_used: i64,
}

/// Get the path to the ggo data directory (~/.config/ggo on Unix)
fn get_data_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    let ggo_dir = config_dir.join("ggo");
    std::fs::create_dir_all(&ggo_dir)
        .context("Failed to create ggo config directory")?;

    Ok(ggo_dir)
}

/// Get the path to the SQLite database file
fn get_db_path() -> Result<PathBuf> {
    Ok(get_data_dir()?.join("data.db"))
}

/// Open a connection to the database, creating it if necessary
pub fn open_db() -> Result<Connection> {
    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path)
        .context("Failed to open database")?;

    // Create tables if they don't exist
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
    )
    .context("Failed to create branches table")?;

    Ok(conn)
}

/// Get current Unix timestamp in seconds
fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Record a branch checkout, updating or inserting the usage record
pub fn record_checkout(repo_path: &str, branch_name: &str) -> Result<()> {
    let conn = open_db()?;
    let now = now_timestamp();

    conn.execute(
        "INSERT INTO branches (repo_path, branch_name, switch_count, last_used)
         VALUES (?1, ?2, 1, ?3)
         ON CONFLICT(repo_path, branch_name) DO UPDATE SET
            switch_count = switch_count + 1,
            last_used = ?3",
        [repo_path, branch_name, &now.to_string()],
    )
    .context("Failed to record checkout")?;

    Ok(())
}

/// Get all branch records for a specific repository
pub fn get_branch_records(repo_path: &str) -> Result<Vec<BranchRecord>> {
    let conn = open_db()?;

    let mut stmt = conn
        .prepare(
            "SELECT repo_path, branch_name, switch_count, last_used
             FROM branches
             WHERE repo_path = ?1
             ORDER BY last_used DESC",
        )
        .context("Failed to prepare query")?;

    let records = stmt
        .query_map([repo_path], |row| {
            Ok(BranchRecord {
                repo_path: row.get(0)?,
                branch_name: row.get(1)?,
                switch_count: row.get(2)?,
                last_used: row.get(3)?,
            })
        })
        .context("Failed to query branches")?
        .filter_map(|r| r.ok())
        .collect();

    Ok(records)
}

/// Get all branch records across all repositories
pub fn get_all_records() -> Result<Vec<BranchRecord>> {
    let conn = open_db()?;

    let mut stmt = conn
        .prepare(
            "SELECT repo_path, branch_name, switch_count, last_used
             FROM branches
             ORDER BY last_used DESC",
        )
        .context("Failed to prepare query")?;

    let records = stmt
        .query_map([], |row| {
            Ok(BranchRecord {
                repo_path: row.get(0)?,
                branch_name: row.get(1)?,
                switch_count: row.get(2)?,
                last_used: row.get(3)?,
            })
        })
        .context("Failed to query branches")?
        .filter_map(|r| r.ok())
        .collect();

    Ok(records)
}

/// Get statistics summary
pub struct Stats {
    pub total_switches: i64,
    pub unique_branches: i64,
    pub unique_repos: i64,
}

pub fn get_stats() -> Result<Stats> {
    let conn = open_db()?;

    let total_switches: i64 = conn
        .query_row("SELECT COALESCE(SUM(switch_count), 0) FROM branches", [], |row| row.get(0))
        .unwrap_or(0);

    let unique_branches: i64 = conn
        .query_row("SELECT COUNT(*) FROM branches", [], |row| row.get(0))
        .unwrap_or(0);

    let unique_repos: i64 = conn
        .query_row("SELECT COUNT(DISTINCT repo_path) FROM branches", [], |row| row.get(0))
        .unwrap_or(0);

    Ok(Stats {
        total_switches,
        unique_branches,
        unique_repos,
    })
}

