use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Branch usage record from the database
#[derive(Debug, Clone)]
pub struct BranchRecord {
    #[allow(dead_code)]
    pub repo_path: String,
    pub branch_name: String,
    pub switch_count: i64,
    pub last_used: i64,
}

/// Branch alias record from the database
#[derive(Debug, Clone)]
pub struct Alias {
    pub repo_path: String,
    pub alias: String,
    pub branch_name: String,
    pub created_at: i64,
}

/// Get the path to the ggo data directory (~/.config/ggo on Unix)
fn get_data_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    let ggo_dir = config_dir.join("ggo");
    std::fs::create_dir_all(&ggo_dir).context("Failed to create ggo config directory")?;

    Ok(ggo_dir)
}

/// Get the path to the SQLite database file
fn get_db_path() -> Result<PathBuf> {
    Ok(get_data_dir()?.join("data.db"))
}

/// Open a connection to the database, creating it if necessary
pub fn open_db() -> Result<Connection> {
    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path).context("Failed to open database")?;

    initialize_tables(&conn)?;
    Ok(conn)
}

/// Initialize database tables
fn initialize_tables(conn: &Connection) -> Result<()> {
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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS aliases (
            repo_path TEXT NOT NULL,
            alias TEXT NOT NULL,
            branch_name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            PRIMARY KEY (repo_path, alias)
        )",
        [],
    )
    .context("Failed to create aliases table")?;

    Ok(())
}

#[cfg(test)]
fn open_test_db() -> Result<Connection> {
    // Use in-memory database for tests to ensure isolation
    let conn = Connection::open_in_memory().context("Failed to open in-memory database")?;

    initialize_tables(&conn)?;
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
        .map_while(Result::ok)
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
        .map_while(Result::ok)
        .collect();

    Ok(records)
}

/// Get statistics summary
pub struct Stats {
    pub total_switches: i64,
    pub unique_branches: i64,
    pub unique_repos: i64,
    pub db_path: PathBuf,
}

pub fn get_stats() -> Result<Stats> {
    let conn = open_db()?;
    let db_path = get_db_path()?;

    let total_switches: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(switch_count), 0) FROM branches",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let unique_branches: i64 = conn
        .query_row("SELECT COUNT(*) FROM branches", [], |row| row.get(0))
        .unwrap_or(0);

    let unique_repos: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT repo_path) FROM branches",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(Stats {
        total_switches,
        unique_branches,
        unique_repos,
        db_path,
    })
}

/// Save the previous branch for quick access (like cd -)
pub fn save_previous_branch(repo_path: &str, branch_name: &str) -> Result<()> {
    let conn = open_db()?;

    // Create the previous_branch table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS previous_branch (
            repo_path TEXT PRIMARY KEY,
            branch_name TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .context("Failed to create previous_branch table")?;

    let now = now_timestamp();

    conn.execute(
        "INSERT OR REPLACE INTO previous_branch (repo_path, branch_name, updated_at)
         VALUES (?1, ?2, ?3)",
        [repo_path, branch_name, &now.to_string()],
    )
    .context("Failed to save previous branch")?;

    Ok(())
}

/// Get the previous branch for the given repository
pub fn get_previous_branch(repo_path: &str) -> Result<Option<String>> {
    let conn = open_db()?;

    // Make sure the table exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS previous_branch (
            repo_path TEXT PRIMARY KEY,
            branch_name TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .ok();

    let result = conn.query_row(
        "SELECT branch_name FROM previous_branch WHERE repo_path = ?1",
        [repo_path],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(branch) => Ok(Some(branch)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e).context("Failed to get previous branch"),
    }
}

/// Create or update an alias for a branch
pub fn create_alias(repo_path: &str, alias: &str, branch_name: &str) -> Result<()> {
    let conn = open_db()?;
    let now = now_timestamp();

    conn.execute(
        "INSERT OR REPLACE INTO aliases (repo_path, alias, branch_name, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        [repo_path, alias, branch_name, &now.to_string()],
    )
    .context("Failed to create alias")?;

    Ok(())
}

/// Get the branch name for an alias
pub fn get_alias(repo_path: &str, alias: &str) -> Result<Option<String>> {
    let conn = open_db()?;

    let result = conn.query_row(
        "SELECT branch_name FROM aliases WHERE repo_path = ?1 AND alias = ?2",
        [repo_path, alias],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(branch) => Ok(Some(branch)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e).context("Failed to get alias"),
    }
}

/// Delete an alias
pub fn delete_alias(repo_path: &str, alias: &str) -> Result<()> {
    let conn = open_db()?;

    conn.execute(
        "DELETE FROM aliases WHERE repo_path = ?1 AND alias = ?2",
        [repo_path, alias],
    )
    .context("Failed to delete alias")?;

    Ok(())
}

/// List all aliases for a repository
pub fn list_aliases(repo_path: &str) -> Result<Vec<Alias>> {
    let conn = open_db()?;

    let mut stmt = conn
        .prepare(
            "SELECT repo_path, alias, branch_name, created_at
             FROM aliases
             WHERE repo_path = ?1
             ORDER BY alias",
        )
        .context("Failed to prepare query")?;

    let aliases = stmt
        .query_map([repo_path], |row| {
            Ok(Alias {
                repo_path: row.get(0)?,
                alias: row.get(1)?,
                branch_name: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .context("Failed to query aliases")?
        .map_while(Result::ok)
        .collect();

    Ok(aliases)
}

/// Get all aliases pointing to a specific branch
pub fn get_aliases_for_branch(repo_path: &str, branch_name: &str) -> Result<Vec<String>> {
    let conn = open_db()?;

    let mut stmt = conn
        .prepare(
            "SELECT alias
             FROM aliases
             WHERE repo_path = ?1 AND branch_name = ?2
             ORDER BY alias",
        )
        .context("Failed to prepare query")?;

    let aliases = stmt
        .query_map([repo_path, branch_name], |row| row.get::<_, String>(0))
        .context("Failed to query aliases")?
        .map_while(Result::ok)
        .collect();

    Ok(aliases)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Generate a unique repo path for testing to avoid conflicts
    fn unique_repo_path() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("/test/repo/{}", id)
    }

    // Test-specific versions that use a provided connection
    fn do_record_checkout(conn: &Connection, repo_path: &str, branch_name: &str) -> Result<()> {
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

    fn do_get_branch_records(conn: &Connection, repo_path: &str) -> Result<Vec<BranchRecord>> {
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
            .map_while(Result::ok)
            .collect();

        Ok(records)
    }

    fn do_get_all_records(conn: &Connection) -> Result<Vec<BranchRecord>> {
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
            .map_while(Result::ok)
            .collect();

        Ok(records)
    }

    fn do_save_previous_branch(
        conn: &Connection,
        repo_path: &str,
        branch_name: &str,
    ) -> Result<()> {
        // Create the previous_branch table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS previous_branch (
                repo_path TEXT PRIMARY KEY,
                branch_name TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .context("Failed to create previous_branch table")?;

        let now = now_timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO previous_branch (repo_path, branch_name, updated_at)
             VALUES (?1, ?2, ?3)",
            [repo_path, branch_name, &now.to_string()],
        )
        .context("Failed to save previous branch")?;

        Ok(())
    }

    fn do_get_previous_branch(conn: &Connection, repo_path: &str) -> Result<Option<String>> {
        // Make sure the table exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS previous_branch (
                repo_path TEXT PRIMARY KEY,
                branch_name TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .ok();

        let result = conn.query_row(
            "SELECT branch_name FROM previous_branch WHERE repo_path = ?1",
            [repo_path],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(branch) => Ok(Some(branch)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to get previous branch"),
        }
    }

    #[test]
    fn test_open_db_creates_table() {
        let result = open_test_db();
        assert!(result.is_ok());

        let conn = result.unwrap();

        // Verify table exists
        let table_check: Result<i64, _> = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='branches'",
            [],
            |row| row.get(0),
        );

        assert!(table_check.is_ok());
        assert_eq!(table_check.unwrap(), 1);
    }

    #[test]
    fn do_record_checkout_new_branch() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        let result = do_record_checkout(&conn, &repo_path, "main");
        assert!(result.is_ok());

        // Verify the record was created
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM branches WHERE repo_path = ?1 AND branch_name = ?2",
                [&repo_path, "main"],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 1);

        // Verify switch_count is 1
        let switch_count: i64 = conn
            .query_row(
                "SELECT switch_count FROM branches WHERE repo_path = ?1 AND branch_name = ?2",
                [&repo_path, "main"],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(switch_count, 1);
    }

    #[test]
    fn do_record_checkout_existing_branch() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        // Record first checkout
        do_record_checkout(&conn, &repo_path, "main").unwrap();

        // Record second checkout
        let result = do_record_checkout(&conn, &repo_path, "main");
        assert!(result.is_ok());

        // Verify switch_count was incremented
        let switch_count: i64 = conn
            .query_row(
                "SELECT switch_count FROM branches WHERE repo_path = ?1 AND branch_name = ?2",
                [&repo_path, "main"],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(switch_count, 2);
    }

    #[test]
    fn do_record_checkout_multiple_repos() {
        let conn = open_test_db().unwrap();
        let repo_path1 = unique_repo_path();
        let repo_path2 = unique_repo_path();

        do_record_checkout(&conn, &repo_path1, "main").unwrap();
        do_record_checkout(&conn, &repo_path2, "main").unwrap();

        let records1 = do_get_branch_records(&conn, &repo_path1).unwrap();
        let records2 = do_get_branch_records(&conn, &repo_path2).unwrap();

        assert_eq!(records1.len(), 1);
        assert_eq!(records2.len(), 1);
        assert_eq!(records1[0].repo_path, repo_path1);
        assert_eq!(records2[0].repo_path, repo_path2);
    }

    #[test]
    fn do_record_checkout_updates_timestamp() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_record_checkout(&conn, &repo_path, "main").unwrap();

        let first_timestamp: i64 = conn
            .query_row(
                "SELECT last_used FROM branches WHERE repo_path = ?1 AND branch_name = ?2",
                [&repo_path, "main"],
                |row| row.get(0),
            )
            .unwrap();

        // Wait a bit and record again
        std::thread::sleep(std::time::Duration::from_millis(100));
        do_record_checkout(&conn, &repo_path, "main").unwrap();

        let second_timestamp: i64 = conn
            .query_row(
                "SELECT last_used FROM branches WHERE repo_path = ?1 AND branch_name = ?2",
                [&repo_path, "main"],
                |row| row.get(0),
            )
            .unwrap();

        assert!(second_timestamp >= first_timestamp);
    }

    #[test]
    fn do_get_branch_records_empty() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        let result = do_get_branch_records(&conn, &repo_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn do_get_branch_records_single() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_record_checkout(&conn, &repo_path, "main").unwrap();

        let records = do_get_branch_records(&conn, &repo_path).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].repo_path, repo_path);
        assert_eq!(records[0].branch_name, "main");
        assert_eq!(records[0].switch_count, 1);
    }

    #[test]
    fn do_get_branch_records_multiple() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_record_checkout(&conn, &repo_path, "main").unwrap();
        do_record_checkout(&conn, &repo_path, "develop").unwrap();
        do_record_checkout(&conn, &repo_path, "feature").unwrap();

        let records = do_get_branch_records(&conn, &repo_path).unwrap();
        assert_eq!(records.len(), 3);

        let branch_names: Vec<&str> = records.iter().map(|r| r.branch_name.as_str()).collect();
        assert!(branch_names.contains(&"main"));
        assert!(branch_names.contains(&"develop"));
        assert!(branch_names.contains(&"feature"));
    }

    #[test]
    fn do_get_branch_records_ordered_by_last_used() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_record_checkout(&conn, &repo_path, "first").unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        do_record_checkout(&conn, &repo_path, "second").unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        do_record_checkout(&conn, &repo_path, "third").unwrap();

        let records = do_get_branch_records(&conn, &repo_path).unwrap();
        assert_eq!(records.len(), 3);

        // Should be ordered by last_used DESC
        assert_eq!(records[0].branch_name, "third");
        assert_eq!(records[1].branch_name, "second");
        assert_eq!(records[2].branch_name, "first");
    }

    #[test]
    fn do_get_branch_records_filters_by_repo() {
        let conn = open_test_db().unwrap();
        let repo_path1 = unique_repo_path();
        let repo_path2 = unique_repo_path();

        do_record_checkout(&conn, &repo_path1, "main").unwrap();
        do_record_checkout(&conn, &repo_path2, "main").unwrap();
        do_record_checkout(&conn, &repo_path2, "develop").unwrap();

        let records1 = do_get_branch_records(&conn, &repo_path1).unwrap();
        let records2 = do_get_branch_records(&conn, &repo_path2).unwrap();

        assert_eq!(records1.len(), 1);
        assert_eq!(records2.len(), 2);
    }

    #[test]
    fn do_get_all_records_empty() {
        let conn = open_test_db().unwrap();
        let result = do_get_all_records(&conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn do_get_all_records_multiple_repos() {
        let conn = open_test_db().unwrap();
        let repo_path1 = unique_repo_path();
        let repo_path2 = unique_repo_path();

        do_record_checkout(&conn, &repo_path1, "main").unwrap();
        do_record_checkout(&conn, &repo_path1, "develop").unwrap();
        do_record_checkout(&conn, &repo_path2, "main").unwrap();

        let records = do_get_all_records(&conn).unwrap();
        assert_eq!(records.len(), 3);
    }

    #[test]
    fn do_get_all_records_ordered() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_record_checkout(&conn, &repo_path, "first").unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        do_record_checkout(&conn, &repo_path, "second").unwrap();

        let records = do_get_all_records(&conn).unwrap();

        assert_eq!(records.len(), 2);
        // Should be ordered by last_used DESC
        assert_eq!(records[0].branch_name, "second");
        assert_eq!(records[1].branch_name, "first");
    }

    #[test]
    fn test_get_stats_empty() {
        let result = get_stats();
        assert!(result.is_ok());

        let stats = result.unwrap();
        // Stats come from the actual database, so check it exists
        assert!(stats.total_switches >= 0);
        assert!(stats.unique_branches >= 0);
        assert!(stats.unique_repos >= 0);
    }

    #[test]
    fn test_get_stats_single_branch() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_record_checkout(&conn, &repo_path, "main").unwrap();

        // Count stats from our test connection
        let total: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(switch_count), 0) FROM branches",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let unique_branches: i64 = conn
            .query_row("SELECT COUNT(*) FROM branches", [], |row| row.get(0))
            .unwrap();
        let unique_repos: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT repo_path) FROM branches",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(total, 1);
        assert_eq!(unique_branches, 1);
        assert_eq!(unique_repos, 1);
    }

    #[test]
    fn test_get_stats_multiple_switches() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_record_checkout(&conn, &repo_path, "main").unwrap();
        do_record_checkout(&conn, &repo_path, "main").unwrap();
        do_record_checkout(&conn, &repo_path, "main").unwrap();

        let total: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(switch_count), 0) FROM branches",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(total, 3);
    }

    #[test]
    fn test_get_stats_multiple_branches_and_repos() {
        let conn = open_test_db().unwrap();
        let repo_path1 = unique_repo_path();
        let repo_path2 = unique_repo_path();

        do_record_checkout(&conn, &repo_path1, "main").unwrap();
        do_record_checkout(&conn, &repo_path1, "develop").unwrap();
        do_record_checkout(&conn, &repo_path2, "main").unwrap();
        do_record_checkout(&conn, &repo_path2, "main").unwrap();

        let total: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(switch_count), 0) FROM branches",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let unique_branches: i64 = conn
            .query_row("SELECT COUNT(*) FROM branches", [], |row| row.get(0))
            .unwrap();
        let unique_repos: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT repo_path) FROM branches",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(total, 4);
        assert_eq!(unique_branches, 3);
        assert_eq!(unique_repos, 2);
    }

    #[test]
    fn test_save_previous_branch() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        let result = do_save_previous_branch(&conn, &repo_path, "main");
        assert!(result.is_ok());

        let branch: String = conn
            .query_row(
                "SELECT branch_name FROM previous_branch WHERE repo_path = ?1",
                [&repo_path],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(branch, "main");
    }

    #[test]
    fn test_save_previous_branch_updates() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_save_previous_branch(&conn, &repo_path, "main").unwrap();
        do_save_previous_branch(&conn, &repo_path, "develop").unwrap();

        let branch: String = conn
            .query_row(
                "SELECT branch_name FROM previous_branch WHERE repo_path = ?1",
                [&repo_path],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(branch, "develop");

        // Verify only one record exists
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM previous_branch WHERE repo_path = ?1",
                [&repo_path],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn test_save_previous_branch_multiple_repos() {
        let conn = open_test_db().unwrap();
        let repo_path1 = unique_repo_path();
        let repo_path2 = unique_repo_path();

        do_save_previous_branch(&conn, &repo_path1, "main").unwrap();
        do_save_previous_branch(&conn, &repo_path2, "develop").unwrap();

        let branch1 = do_get_previous_branch(&conn, &repo_path1).unwrap();
        let branch2 = do_get_previous_branch(&conn, &repo_path2).unwrap();

        assert_eq!(branch1, Some("main".to_string()));
        assert_eq!(branch2, Some("develop".to_string()));
    }

    #[test]
    fn do_get_previous_branch_not_found() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        let result = do_get_previous_branch(&conn, &repo_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn do_get_previous_branch_exists() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_save_previous_branch(&conn, &repo_path, "main").unwrap();

        let result = do_get_previous_branch(&conn, &repo_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("main".to_string()));
    }

    #[test]
    fn do_get_previous_branch_different_repos() {
        let conn = open_test_db().unwrap();
        let repo_path1 = unique_repo_path();
        let repo_path2 = unique_repo_path();

        do_save_previous_branch(&conn, &repo_path1, "main").unwrap();

        let result1 = do_get_previous_branch(&conn, &repo_path1);
        let result2 = do_get_previous_branch(&conn, &repo_path2);

        assert_eq!(result1.unwrap(), Some("main".to_string()));
        assert_eq!(result2.unwrap(), None);
    }

    #[test]
    fn test_branch_record_clone() {
        let record = BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "main".to_string(),
            switch_count: 5,
            last_used: 1234567890,
        };

        let cloned = record.clone();
        assert_eq!(record.repo_path, cloned.repo_path);
        assert_eq!(record.branch_name, cloned.branch_name);
        assert_eq!(record.switch_count, cloned.switch_count);
        assert_eq!(record.last_used, cloned.last_used);
    }

    #[test]
    fn test_branch_record_debug() {
        let record = BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "main".to_string(),
            switch_count: 5,
            last_used: 1234567890,
        };

        let debug_str = format!("{:?}", record);
        assert!(debug_str.contains("/test"));
        assert!(debug_str.contains("main"));
        assert!(debug_str.contains("5"));
        assert!(debug_str.contains("1234567890"));
    }

    #[test]
    fn test_now_timestamp() {
        let timestamp = now_timestamp();
        assert!(timestamp > 0);

        // Should be reasonable (after year 2000, before year 3000)
        assert!(timestamp > 946684800); // 2000-01-01
        assert!(timestamp < 32503680000); // 3000-01-01
    }

    #[test]
    fn test_get_data_dir_creates_directory() {
        let result = get_data_dir();
        assert!(result.is_ok());

        let dir = result.unwrap();
        assert!(dir.exists());
        assert!(dir.is_dir());
    }

    #[test]
    fn test_get_db_path() {
        let result = get_db_path();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().ends_with("data.db"));
    }

    #[test]
    fn test_special_characters_in_branch_names() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        let special_branch = "feature/issue-#123_v2.0";
        do_record_checkout(&conn, &repo_path, special_branch).unwrap();

        let records = do_get_branch_records(&conn, &repo_path).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].branch_name, special_branch);
    }

    #[test]
    fn test_unicode_in_branch_names() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        let unicode_branch = "feature/日本語-🚀";
        do_record_checkout(&conn, &repo_path, unicode_branch).unwrap();

        let records = do_get_branch_records(&conn, &repo_path).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].branch_name, unicode_branch);
    }

    #[test]
    fn test_long_branch_names() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        let long_branch = "a".repeat(200);
        do_record_checkout(&conn, &repo_path, &long_branch).unwrap();

        let records = do_get_branch_records(&conn, &repo_path).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].branch_name, long_branch);
    }

    #[test]
    fn test_long_repo_paths() {
        let conn = open_test_db().unwrap();
        let long_path = format!("{}/long/{}", unique_repo_path(), "repo/".repeat(50));
        do_record_checkout(&conn, &long_path, "main").unwrap();

        let records = do_get_branch_records(&conn, &long_path).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].repo_path, long_path);
    }

    // Alias test helper functions
    fn do_create_alias(
        conn: &Connection,
        repo_path: &str,
        alias: &str,
        branch_name: &str,
    ) -> Result<()> {
        let now = now_timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO aliases (repo_path, alias, branch_name, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            [repo_path, alias, branch_name, &now.to_string()],
        )
        .context("Failed to create alias")?;

        Ok(())
    }

    fn do_get_alias(conn: &Connection, repo_path: &str, alias: &str) -> Result<Option<String>> {
        let result = conn.query_row(
            "SELECT branch_name FROM aliases WHERE repo_path = ?1 AND alias = ?2",
            [repo_path, alias],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(branch) => Ok(Some(branch)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to get alias"),
        }
    }

    fn do_delete_alias(conn: &Connection, repo_path: &str, alias: &str) -> Result<()> {
        conn.execute(
            "DELETE FROM aliases WHERE repo_path = ?1 AND alias = ?2",
            [repo_path, alias],
        )
        .context("Failed to delete alias")?;

        Ok(())
    }

    fn do_list_aliases(conn: &Connection, repo_path: &str) -> Result<Vec<Alias>> {
        let mut stmt = conn
            .prepare(
                "SELECT repo_path, alias, branch_name, created_at
                 FROM aliases
                 WHERE repo_path = ?1
                 ORDER BY alias",
            )
            .context("Failed to prepare query")?;

        let aliases = stmt
            .query_map([repo_path], |row| {
                Ok(Alias {
                    repo_path: row.get(0)?,
                    alias: row.get(1)?,
                    branch_name: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .context("Failed to query aliases")?
            .map_while(Result::ok)
            .collect();

        Ok(aliases)
    }

    fn do_get_aliases_for_branch(
        conn: &Connection,
        repo_path: &str,
        branch_name: &str,
    ) -> Result<Vec<String>> {
        let mut stmt = conn
            .prepare(
                "SELECT alias
                 FROM aliases
                 WHERE repo_path = ?1 AND branch_name = ?2
                 ORDER BY alias",
            )
            .context("Failed to prepare query")?;

        let aliases = stmt
            .query_map([repo_path, branch_name], |row| row.get::<_, String>(0))
            .context("Failed to query aliases")?
            .map_while(Result::ok)
            .collect();

        Ok(aliases)
    }

    #[test]
    fn test_create_alias() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        let result = do_create_alias(&conn, &repo_path, "m", "master");
        assert!(result.is_ok());

        let branch = do_get_alias(&conn, &repo_path, "m").unwrap();
        assert_eq!(branch, Some("master".to_string()));
    }

    #[test]
    fn test_create_alias_updates_existing() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_create_alias(&conn, &repo_path, "m", "master").unwrap();
        do_create_alias(&conn, &repo_path, "m", "main").unwrap();

        let branch = do_get_alias(&conn, &repo_path, "m").unwrap();
        assert_eq!(branch, Some("main".to_string()));

        // Verify only one alias exists
        let aliases = do_list_aliases(&conn, &repo_path).unwrap();
        assert_eq!(aliases.len(), 1);
    }

    #[test]
    fn test_get_alias_not_found() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        let result = do_get_alias(&conn, &repo_path, "nonexistent");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_delete_alias() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_create_alias(&conn, &repo_path, "m", "master").unwrap();
        let result = do_delete_alias(&conn, &repo_path, "m");
        assert!(result.is_ok());

        let branch = do_get_alias(&conn, &repo_path, "m").unwrap();
        assert_eq!(branch, None);
    }

    #[test]
    fn test_list_aliases_empty() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        let aliases = do_list_aliases(&conn, &repo_path).unwrap();
        assert_eq!(aliases.len(), 0);
    }

    #[test]
    fn test_list_aliases_multiple() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_create_alias(&conn, &repo_path, "m", "master").unwrap();
        do_create_alias(&conn, &repo_path, "d", "develop").unwrap();
        do_create_alias(&conn, &repo_path, "f", "feature/test").unwrap();

        let aliases = do_list_aliases(&conn, &repo_path).unwrap();
        assert_eq!(aliases.len(), 3);

        let alias_names: Vec<&str> = aliases.iter().map(|a| a.alias.as_str()).collect();
        assert!(alias_names.contains(&"m"));
        assert!(alias_names.contains(&"d"));
        assert!(alias_names.contains(&"f"));
    }

    #[test]
    fn test_list_aliases_sorted() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_create_alias(&conn, &repo_path, "z", "zzz").unwrap();
        do_create_alias(&conn, &repo_path, "a", "aaa").unwrap();
        do_create_alias(&conn, &repo_path, "m", "mmm").unwrap();

        let aliases = do_list_aliases(&conn, &repo_path).unwrap();
        assert_eq!(aliases.len(), 3);

        // Should be sorted alphabetically by alias
        assert_eq!(aliases[0].alias, "a");
        assert_eq!(aliases[1].alias, "m");
        assert_eq!(aliases[2].alias, "z");
    }

    #[test]
    fn test_list_aliases_filters_by_repo() {
        let conn = open_test_db().unwrap();
        let repo_path1 = unique_repo_path();
        let repo_path2 = unique_repo_path();

        do_create_alias(&conn, &repo_path1, "m", "master").unwrap();
        do_create_alias(&conn, &repo_path2, "m", "main").unwrap();
        do_create_alias(&conn, &repo_path2, "d", "develop").unwrap();

        let aliases1 = do_list_aliases(&conn, &repo_path1).unwrap();
        let aliases2 = do_list_aliases(&conn, &repo_path2).unwrap();

        assert_eq!(aliases1.len(), 1);
        assert_eq!(aliases2.len(), 2);
    }

    #[test]
    fn test_get_aliases_for_branch_empty() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        let aliases = do_get_aliases_for_branch(&conn, &repo_path, "master").unwrap();
        assert_eq!(aliases.len(), 0);
    }

    #[test]
    fn test_get_aliases_for_branch_single() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_create_alias(&conn, &repo_path, "m", "master").unwrap();

        let aliases = do_get_aliases_for_branch(&conn, &repo_path, "master").unwrap();
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0], "m");
    }

    #[test]
    fn test_get_aliases_for_branch_multiple() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_create_alias(&conn, &repo_path, "m", "master").unwrap();
        do_create_alias(&conn, &repo_path, "main", "master").unwrap();
        do_create_alias(&conn, &repo_path, "prod", "master").unwrap();
        do_create_alias(&conn, &repo_path, "d", "develop").unwrap();

        let aliases = do_get_aliases_for_branch(&conn, &repo_path, "master").unwrap();
        assert_eq!(aliases.len(), 3);
        assert!(aliases.contains(&"m".to_string()));
        assert!(aliases.contains(&"main".to_string()));
        assert!(aliases.contains(&"prod".to_string()));
    }

    #[test]
    fn test_alias_with_special_characters() {
        let conn = open_test_db().unwrap();
        let repo_path = unique_repo_path();

        do_create_alias(&conn, &repo_path, "my-alias", "feature/test-123").unwrap();

        let branch = do_get_alias(&conn, &repo_path, "my-alias").unwrap();
        assert_eq!(branch, Some("feature/test-123".to_string()));
    }

    #[test]
    fn test_alias_struct_clone() {
        let alias = Alias {
            repo_path: "/test".to_string(),
            alias: "m".to_string(),
            branch_name: "master".to_string(),
            created_at: 1234567890,
        };

        let cloned = alias.clone();
        assert_eq!(alias.repo_path, cloned.repo_path);
        assert_eq!(alias.alias, cloned.alias);
        assert_eq!(alias.branch_name, cloned.branch_name);
        assert_eq!(alias.created_at, cloned.created_at);
    }

    #[test]
    fn test_alias_struct_debug() {
        let alias = Alias {
            repo_path: "/test".to_string(),
            alias: "m".to_string(),
            branch_name: "master".to_string(),
            created_at: 1234567890,
        };

        let debug_str = format!("{:?}", alias);
        assert!(debug_str.contains("/test"));
        assert!(debug_str.contains("m"));
        assert!(debug_str.contains("master"));
        assert!(debug_str.contains("1234567890"));
    }
}
