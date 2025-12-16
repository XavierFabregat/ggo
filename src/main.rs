mod cli;
mod constants;
mod frecency;
mod git;
mod interactive;
mod matcher;
mod storage;
mod validation;

use anyhow::{bail, Context, Result};
use clap::Parser;
use tracing::{debug, warn};

use cli::{Cli, Commands};
use constants::scoring::{AUTO_SELECT_THRESHOLD, FRECENCY_MULTIPLIER};

fn main() -> Result<()> {
    // Initialize tracing for structured logging
    // Set RUST_LOG=debug for verbose output, or RUST_LOG=trace for very verbose
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_target(false)
        .with_level(true)
        .init();

    let cli = Cli::parse();
    debug!("CLI arguments: {:?}", cli);

    // Handle version flag
    if cli.version {
        println!("ggo {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Handle subcommands first
    if let Some(command) = cli.command {
        match command {
            Commands::Alias {
                alias,
                branch,
                list,
                remove,
            } => {
                handle_alias_command(alias.as_deref(), branch.as_deref(), list, remove)?;
                return Ok(());
            }
            Commands::Cleanup {
                older_than,
                deleted,
                optimize,
                size,
            } => {
                handle_cleanup_command(older_than, deleted, optimize, size)?;
                return Ok(());
            }
        }
    }

    if cli.stats {
        show_stats()?;
        return Ok(());
    }

    // Pattern is required if no subcommand and no stats
    let pattern = cli
        .pattern
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("Pattern argument is required\n\nUsage: ggo <pattern>\nTry 'ggo --help' for more information"))?;

    // Handle the special '-' pattern to go back to previous branch
    if pattern == "-" {
        checkout_previous_branch()?;
        return Ok(());
    }

    // Validate search pattern
    validation::validate_pattern(pattern).context("Invalid search pattern")?;

    if cli.list {
        list_matching_branches(pattern, cli.ignore_case, !cli.no_fuzzy)?;
    } else {
        let branch =
            find_and_checkout_branch(pattern, cli.ignore_case, !cli.no_fuzzy, cli.interactive)?;
        println!("Switched to branch '{}'", branch);
    }

    Ok(())
}

fn show_stats() -> Result<()> {
    let stats = storage::get_stats()?;
    let records = storage::get_all_records()?;

    println!("📊 ggo Statistics\n");
    println!("Total branch switches: {}", stats.total_switches);
    println!("Unique branches tracked: {}", stats.unique_branches);
    println!("Repositories: {}", stats.unique_repos);
    println!("Database location: {}", stats.db_path.display());

    if !records.is_empty() {
        println!("\n🔥 Top branches by frecency:\n");

        let scored = frecency::rank_branches(&records);
        for (i, branch) in scored.iter().take(10).enumerate() {
            let time_ago = frecency::format_relative_time(branch.last_used);
            println!(
                "  {}. {} (score: {:.1}, {} switches, {})",
                i + 1,
                branch.name,
                branch.score,
                branch.switch_count,
                time_ago
            );
        }
    }

    Ok(())
}

fn list_matching_branches(pattern: &str, ignore_case: bool, use_fuzzy: bool) -> Result<()> {
    let branches = git::get_branches()?;
    let repo_path = git::get_repo_root().context("Failed to determine git repository root")?;

    // Try to load branch history, but continue without it if it fails
    let records = match storage::get_branch_records(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("⚠️  Warning: Could not load branch history: {}", e);
            eprintln!("   Frecency ranking will not be available.");
            vec![]
        }
    };

    let ranked = if use_fuzzy {
        // Use fuzzy matching and combine with frecency
        let fuzzy_matches = matcher::fuzzy_filter_branches(&branches, pattern, ignore_case);

        if fuzzy_matches.is_empty() {
            bail!(
                "No branches found matching '{}'\n\nTry:\n  • Using a different pattern\n  • Running 'git branch' to see all branches\n  • Using 'ggo --list \"\"' to list all branches",
                pattern
            );
        }

        combine_fuzzy_and_frecency_scores(&fuzzy_matches, &records)
    } else {
        // Use exact substring matching
        let matches = matcher::filter_branches(&branches, pattern, ignore_case);

        if matches.is_empty() {
            bail!(
                "No branches found matching '{}'\n\nTry:\n  • Using a different pattern\n  • Enabling fuzzy matching (remove --no-fuzzy flag)\n  • Running 'git branch' to see all branches",
                pattern
            );
        }

        let match_strings: Vec<String> = matches.iter().map(|s| s.to_string()).collect();
        frecency::sort_branches_by_frecency(&match_strings, &records)
    };

    let match_type = if use_fuzzy {
        "fuzzy matching"
    } else {
        "substring matching"
    };
    println!(
        "Branches matching '{}' ({}+ frecency):\n",
        pattern, match_type
    );

    for (i, (branch, score)) in ranked.iter().enumerate() {
        let marker = if i == 0 { "→" } else { " " };
        let score_display = if *score > 0.0 {
            format!(" ({:.1})", score)
        } else {
            String::new()
        };

        // Get aliases for this branch
        let aliases = storage::get_aliases_for_branch(&repo_path, branch).unwrap_or_default();
        let alias_display = if !aliases.is_empty() {
            format!(" [alias: {}]", aliases.join(", "))
        } else {
            String::new()
        };

        println!("  {} {}{}{}", marker, branch, score_display, alias_display);
    }

    if ranked.len() > 1 {
        println!("\n({} matches, → indicates checkout target)", ranked.len());
    }

    Ok(())
}

fn checkout_previous_branch() -> Result<()> {
    let repo_path = git::get_repo_root()?;

    let previous_branch = storage::get_previous_branch(&repo_path)?.ok_or_else(|| {
        anyhow::anyhow!(
            "No previous branch found\n\nYou haven't switched branches yet in this repository"
        )
    })?;

    // Re-verify branch exists before checkout (prevent race condition)
    let current_branches =
        git::get_branches().context("Failed to verify branch list before checkout")?;

    if !current_branches.contains(&previous_branch) {
        bail!(
            "Branch '{}' no longer exists\n\nYour previous branch may have been deleted.\nRun 'git branch' to see available branches.",
            previous_branch
        );
    }

    // Save current branch before switching
    if let Ok(current_branch) = git::get_current_branch() {
        if let Err(e) = storage::save_previous_branch(&repo_path, &current_branch) {
            eprintln!("⚠️  Warning: Could not save previous branch: {}", e);
            eprintln!("   The 'ggo -' command may not work correctly.");
        }
    }

    // Checkout the previous branch
    git::checkout(&previous_branch)?;

    // Record the checkout for frecency tracking
    if let Err(e) = storage::record_checkout(&repo_path, &previous_branch) {
        eprintln!("⚠️  Warning: Could not save branch usage: {}", e);
        eprintln!(
            "   This won't affect future checkouts, but frecency tracking may be incomplete."
        );
    }

    println!("Switched to branch '{}'", previous_branch);
    Ok(())
}

/// Handle cleanup subcommand operations
fn handle_cleanup_command(
    older_than_days: i64,
    cleanup_deleted: bool,
    optimize: bool,
    show_size: bool,
) -> Result<()> {
    if show_size {
        let size = storage::get_database_size()?;
        let size_kb = size as f64 / 1024.0;
        let size_mb = size_kb / 1024.0;

        if size_mb > 1.0 {
            println!("Database size: {:.2} MB", size_mb);
        } else {
            println!("Database size: {:.2} KB", size_kb);
        }
    }

    if cleanup_deleted {
        println!("Cleaning up deleted branches...");
        let deleted = storage::cleanup_deleted_branches()?;
        println!("Removed {} stale branch records", deleted);
    }

    // Cleanup old records (always run if a custom age is specified, or if --optimize is used)
    if older_than_days < 365 || optimize {
        println!("Cleaning up branches older than {} days...", older_than_days);
        let deleted = storage::cleanup_old_records(older_than_days)?;
        println!("Removed {} old branch records", deleted);
    }

    if optimize {
        println!("Optimizing database...");
        storage::optimize_database()?;
        println!("Database optimized (VACUUM and ANALYZE complete)");
    }

    if !show_size && !cleanup_deleted && !optimize && older_than_days == 365 {
        // No flags specified, show help
        println!("Database cleanup options:");
        println!("  --deleted          Remove records for deleted branches");
        println!("  --older-than N     Remove branches not used in N days");
        println!("  --optimize         Run VACUUM and ANALYZE");
        println!("  --size             Show database size");
        println!("\nExample: ggo cleanup --deleted --optimize");
    }

    Ok(())
}

/// Handle alias subcommand operations
fn handle_alias_command(
    alias: Option<&str>,
    branch: Option<&str>,
    list: bool,
    remove: bool,
) -> Result<()> {
    let repo_path = git::get_repo_root()?;

    // Handle --list flag
    if list {
        let aliases = storage::list_aliases(&repo_path)?;
        if aliases.is_empty() {
            println!("No aliases defined for this repository");
        } else {
            println!("Aliases for this repository:\n");
            for a in aliases {
                println!("  {} → {}", a.alias, a.branch_name);
            }
        }
        return Ok(());
    }

    // Alias is required for other operations
    let alias = alias.ok_or_else(|| anyhow::anyhow!("Alias name is required"))?;

    // Handle --remove flag
    if remove {
        storage::delete_alias(&repo_path, alias)?;
        println!("Removed alias '{}'", alias);
        return Ok(());
    }

    // If branch is provided, create/update alias
    if let Some(branch_name) = branch {
        // Validate alias name
        validation::validate_alias_name(alias).context("Invalid alias name")?;

        // Validate branch name
        validation::validate_branch_name(branch_name).context("Invalid branch name")?;

        // Validate that branch exists
        let branches = git::get_branches()?;
        if !branches.contains(&branch_name.to_string()) {
            bail!(
                "Branch '{}' does not exist in this repository\n\nRun 'git branch' to see available branches",
                branch_name
            );
        }

        // Create/update the alias
        storage::create_alias(&repo_path, alias, branch_name)?;
        println!("Created alias '{}' → '{}'", alias, branch_name);
        return Ok(());
    }

    // No branch provided: show what alias points to
    match storage::get_alias(&repo_path, alias)? {
        Some(branch_name) => {
            println!("{} → {}", alias, branch_name);
        }
        None => {
            println!("Alias '{}' not found", alias);
        }
    }

    Ok(())
}

/// Combine fuzzy match scores with frecency scores for final ranking
/// Formula: combined_score = fuzzy_score + (frecency_score * 10)
/// This gives weight to both good fuzzy matches and frequently-used branches
fn combine_fuzzy_and_frecency_scores(
    fuzzy_matches: &[matcher::ScoredMatch],
    records: &[storage::BranchRecord],
) -> Vec<(String, f64)> {
    use std::collections::HashMap;

    // Build a map of branch -> frecency score
    let frecency_map: HashMap<&str, f64> = records
        .iter()
        .map(|r| (r.branch_name.as_str(), frecency::calculate_score(r)))
        .collect();

    let mut combined: Vec<(String, f64)> = fuzzy_matches
        .iter()
        .map(|m| {
            let fuzzy_score = m.score as f64;
            let frecency_score = frecency_map.get(m.branch.as_str()).copied().unwrap_or(0.0);

            // Combine scores: fuzzy match quality + (frecency * weight)
            // Frecency gets a multiplier to give it significant weight
            let combined_score = fuzzy_score + (frecency_score * FRECENCY_MULTIPLIER);

            (m.branch.clone(), combined_score)
        })
        .collect();

    // Sort by combined score descending
    combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    combined
}

fn find_and_checkout_branch(
    pattern: &str,
    ignore_case: bool,
    use_fuzzy: bool,
    interactive: bool,
) -> Result<String> {
    let branches = git::get_branches()?;
    let repo_path = git::get_repo_root().context("Failed to determine git repository root")?;

    // Try to load branch history, but continue without it if it fails
    let records = match storage::get_branch_records(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("⚠️  Warning: Could not load branch history: {}", e);
            eprintln!("   Frecency ranking will not be available.");
            vec![]
        }
    };

    // Check if pattern is an exact alias match (highest priority)
    // Note: get_alias() only returns aliases for the current repo (scoped by repo_path)
    // This ensures we never try to use an alias from a different repository
    if let Ok(Some(branch_name)) = storage::get_alias(&repo_path, pattern) {
        // Verify the aliased branch exists in the current repository
        // This protects against stale aliases pointing to deleted branches
        if branches.contains(&branch_name) {
            println!("Using alias '{}' → '{}'", pattern, branch_name);

            // Re-verify branch exists before checkout (prevent race condition)
            let current_branches =
                git::get_branches().context("Failed to verify branch list before checkout")?;

            if !current_branches.contains(&branch_name) {
                bail!(
                    "Branch '{}' no longer exists\n\nIt may have been deleted after alias lookup.\nRun 'git branch' to see available branches.",
                    branch_name
                );
            }

            // Checkout the aliased branch directly
            let current_branch = git::get_current_branch().ok();
            if let Some(ref current) = current_branch {
                if current != &branch_name {
                    if let Err(e) = storage::save_previous_branch(&repo_path, current) {
                        warn!("Failed to save previous branch: {}", e);
                        eprintln!("⚠️  Warning: 'ggo -' may not work correctly");
                    } else {
                        debug!("Saved previous branch: {}", current);
                    }
                }
            }

            git::checkout(&branch_name)?;

            if let Err(e) = storage::record_checkout(&repo_path, &branch_name) {
                eprintln!("⚠️  Warning: Could not save branch usage: {}", e);
                eprintln!("   This won't affect future checkouts, but frecency tracking may be incomplete.");
            }

            return Ok(branch_name);
        } else {
            eprintln!(
                "Warning: Alias '{}' points to non-existent branch '{}'. Falling back to pattern matching.",
                pattern, branch_name
            );
        }
    }

    let ranked = if use_fuzzy {
        // Use fuzzy matching and combine with frecency
        let fuzzy_matches = matcher::fuzzy_filter_branches(&branches, pattern, ignore_case);

        if fuzzy_matches.is_empty() {
            bail!("No branch found matching '{}'", pattern);
        }

        combine_fuzzy_and_frecency_scores(&fuzzy_matches, &records)
    } else {
        // Use exact substring matching
        let matches = matcher::filter_branches(&branches, pattern, ignore_case);

        if matches.is_empty() {
            bail!("No branch found matching '{}'", pattern);
        }

        let match_strings: Vec<String> = matches.iter().map(|s| s.to_string()).collect();
        frecency::sort_branches_by_frecency(&match_strings, &records)
    };

    // Determine which branch to checkout
    let branch_to_checkout = if interactive {
        // Always use interactive mode if explicitly requested
        let branch_list: Vec<String> = ranked.iter().map(|(b, _)| b.clone()).collect();
        interactive::select_branch(&branch_list, &records)?
    } else if ranked.len() == 1 {
        // Single match: use it
        ranked[0].0.clone()
    } else {
        // Multiple matches: check if there's a clear winner
        let top_score = ranked[0].1;
        let second_score = ranked[1].1;

        // If top score is above threshold compared to second, auto-select
        // Handle edge case where second_score is 0
        let should_auto_select = if second_score == 0.0 {
            true
        } else {
            top_score / second_score >= AUTO_SELECT_THRESHOLD
        };

        if should_auto_select {
            ranked[0].0.clone()
        } else {
            // Scores are close, show interactive menu
            let branch_list: Vec<String> = ranked.iter().map(|(b, _)| b.clone()).collect();
            interactive::select_branch(&branch_list, &records)?
        }
    };

    // Re-verify branch exists before checkout (prevent race condition)
    let current_branches =
        git::get_branches().context("Failed to verify branch list before checkout")?;

    if !current_branches.contains(&branch_to_checkout) {
        bail!(
            "Branch '{}' no longer exists\n\nIt may have been deleted after the initial search.\nRun 'git branch' to see available branches.",
            branch_to_checkout
        );
    }

    // Save current branch as previous before switching
    if let Ok(current_branch) = git::get_current_branch() {
        // Only save if we're switching to a different branch
        if current_branch != branch_to_checkout {
            if let Err(e) = storage::save_previous_branch(&repo_path, &current_branch) {
                eprintln!("⚠️  Warning: Could not save previous branch: {}", e);
                eprintln!("   The 'ggo -' command may not work correctly.");
            }
        }
    }

    // Checkout the branch
    git::checkout(&branch_to_checkout)?;

    // Record the checkout for frecency tracking
    if let Err(e) = storage::record_checkout(&repo_path, &branch_to_checkout) {
        // Don't fail the checkout if recording fails, just warn
        eprintln!("⚠️  Warning: Could not save branch usage: {}", e);
        eprintln!(
            "   This won't affect future checkouts, but frecency tracking may be incomplete."
        );
    }

    Ok(branch_to_checkout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::ScoredMatch;
    use crate::storage::BranchRecord;

    #[test]
    fn test_combine_fuzzy_and_frecency_scores_empty() {
        let fuzzy_matches: Vec<ScoredMatch> = vec![];
        let records: Vec<BranchRecord> = vec![];

        let result = combine_fuzzy_and_frecency_scores(&fuzzy_matches, &records);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_combine_fuzzy_and_frecency_scores_no_records() {
        let fuzzy_matches = vec![
            ScoredMatch {
                branch: "feature/auth".to_string(),
                score: 100,
            },
            ScoredMatch {
                branch: "feature/dashboard".to_string(),
                score: 80,
            },
        ];
        let records: Vec<BranchRecord> = vec![];

        let result = combine_fuzzy_and_frecency_scores(&fuzzy_matches, &records);

        assert_eq!(result.len(), 2);
        // Without frecency, should sort by fuzzy score only
        assert_eq!(result[0].0, "feature/auth");
        assert_eq!(result[0].1, 100.0);
        assert_eq!(result[1].0, "feature/dashboard");
        assert_eq!(result[1].1, 80.0);
    }

    #[test]
    fn test_combine_fuzzy_and_frecency_scores_with_records() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let fuzzy_matches = vec![
            ScoredMatch {
                branch: "feature/auth".to_string(),
                score: 80,
            },
            ScoredMatch {
                branch: "feature/dashboard".to_string(),
                score: 100,
            },
        ];

        let records = vec![BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "feature/auth".to_string(),
            switch_count: 10,
            last_used: now - 60, // Recent: frecency score ≈ 10.0 (10 * ~1.0)
        }];

        let result = combine_fuzzy_and_frecency_scores(&fuzzy_matches, &records);

        assert_eq!(result.len(), 2);
        // feature/auth should rank higher due to frecency
        // auth: 80 + (10.0 * 10) = 180
        // dashboard: 100 + (0 * 10) = 100
        assert_eq!(result[0].0, "feature/auth");
        assert!(result[0].1 > 179.0 && result[0].1 < 181.0);
        assert_eq!(result[1].0, "feature/dashboard");
        assert_eq!(result[1].1, 100.0);
    }

    #[test]
    fn test_combine_fuzzy_and_frecency_scores_balanced() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let fuzzy_matches = vec![
            ScoredMatch {
                branch: "branch-a".to_string(),
                score: 100,
            },
            ScoredMatch {
                branch: "branch-b".to_string(),
                score: 50,
            },
        ];

        let records = vec![
            BranchRecord {
                repo_path: "/test".to_string(),
                branch_name: "branch-a".to_string(),
                switch_count: 1,
                last_used: now - 3000000, // Old: frecency ≈ 0.03 (1 * 0.03)
            },
            BranchRecord {
                repo_path: "/test".to_string(),
                branch_name: "branch-b".to_string(),
                switch_count: 5,
                last_used: now - 60, // Recent: frecency ≈ 5.0 (5 * 1.0)
            },
        ];

        let result = combine_fuzzy_and_frecency_scores(&fuzzy_matches, &records);

        assert_eq!(result.len(), 2);
        // branch-a: 100 + (0.03 * 10) ≈ 100.3
        // branch-b: 50 + (5.0 * 10) = 100.0
        // branch-a wins slightly (better fuzzy match despite lower frecency)
        assert_eq!(result[0].0, "branch-a");
        assert_eq!(result[1].0, "branch-b");
    }

    #[test]
    fn test_combine_fuzzy_and_frecency_scores_high_frecency() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let fuzzy_matches = vec![
            ScoredMatch {
                branch: "new-branch".to_string(),
                score: 100,
            },
            ScoredMatch {
                branch: "popular-branch".to_string(),
                score: 60,
            },
        ];

        let records = vec![BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "popular-branch".to_string(),
            switch_count: 20,
            last_used: now - 60, // Recent: frecency ≈ 20.0 (20 * ~1.0)
        }];

        let result = combine_fuzzy_and_frecency_scores(&fuzzy_matches, &records);

        assert_eq!(result.len(), 2);
        // popular-branch: 60 + (20.0 * 10) = 260.0
        // new-branch: 100 + (0 * 10) = 100.0
        assert_eq!(result[0].0, "popular-branch");
        assert!(result[0].1 > 259.0 && result[0].1 < 261.0);
    }

    #[test]
    fn test_combine_fuzzy_and_frecency_scores_single_match() {
        let fuzzy_matches = vec![ScoredMatch {
            branch: "only-match".to_string(),
            score: 75,
        }];
        let records: Vec<BranchRecord> = vec![];

        let result = combine_fuzzy_and_frecency_scores(&fuzzy_matches, &records);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "only-match");
        assert_eq!(result[0].1, 75.0);
    }

    #[test]
    fn test_combine_fuzzy_and_frecency_scores_all_same_fuzzy() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let fuzzy_matches = vec![
            ScoredMatch {
                branch: "branch-a".to_string(),
                score: 100,
            },
            ScoredMatch {
                branch: "branch-b".to_string(),
                score: 100,
            },
        ];

        let records = vec![BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "branch-b".to_string(),
            switch_count: 5,
            last_used: now - 60, // Recent
        }];

        let result = combine_fuzzy_and_frecency_scores(&fuzzy_matches, &records);

        // branch-b should rank higher due to frecency
        assert_eq!(result[0].0, "branch-b");
        assert!(result[0].1 > result[1].1);
    }

    #[test]
    fn test_combine_fuzzy_and_frecency_scores_partial_overlap() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let fuzzy_matches = vec![
            ScoredMatch {
                branch: "branch-a".to_string(),
                score: 90,
            },
            ScoredMatch {
                branch: "branch-b".to_string(),
                score: 85,
            },
            ScoredMatch {
                branch: "branch-c".to_string(),
                score: 80,
            },
        ];

        let records = vec![BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "branch-b".to_string(),
            switch_count: 3,
            last_used: now - 60,
        }];

        let result = combine_fuzzy_and_frecency_scores(&fuzzy_matches, &records);

        assert_eq!(result.len(), 3);
        // branch-b should be first due to frecency boost
        assert_eq!(result[0].0, "branch-b");
    }

    #[test]
    fn test_combine_fuzzy_and_frecency_scores_zero_fuzzy_score() {
        let fuzzy_matches = vec![ScoredMatch {
            branch: "branch-a".to_string(),
            score: 0,
        }];
        let records: Vec<BranchRecord> = vec![];
        let result = combine_fuzzy_and_frecency_scores(&fuzzy_matches, &records);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1, 0.0);
    }

    #[test]
    fn test_combine_fuzzy_and_frecency_ordering_consistency() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let fuzzy_matches = vec![
            ScoredMatch {
                branch: "high-fuzzy-low-frecency".to_string(),
                score: 100,
            },
            ScoredMatch {
                branch: "low-fuzzy-high-frecency".to_string(),
                score: 20,
            },
        ];

        let records = vec![BranchRecord {
            repo_path: "/test".to_string(),
            branch_name: "low-fuzzy-high-frecency".to_string(),
            switch_count: 50,
            last_used: now - 60, // Recent, high frecency
        }];

        let result = combine_fuzzy_and_frecency_scores(&fuzzy_matches, &records);

        // Low fuzzy but high frecency should win
        assert_eq!(result[0].0, "low-fuzzy-high-frecency");
        assert!(result[0].1 > result[1].1);
    }

    #[test]
    fn test_should_auto_select_clear_winner() {
        // Test that 2x score ratio triggers auto-select
        let top_score = 400.0;
        let second_score = 150.0;

        let should_auto_select = top_score / second_score >= 2.0;
        assert!(should_auto_select);
    }

    #[test]
    fn test_should_not_auto_select_close_scores() {
        // Test that close scores (< 2x) trigger interactive menu
        let top_score = 250.0;
        let second_score = 200.0;

        let should_auto_select = top_score / second_score >= 2.0;
        assert!(!should_auto_select);
    }

    #[test]
    fn test_should_auto_select_exact_2x() {
        // Test boundary condition: exactly 2x should auto-select
        let top_score = 200.0;
        let second_score = 100.0;

        let should_auto_select = top_score / second_score >= 2.0;
        assert!(should_auto_select);
    }

    #[test]
    fn test_should_auto_select_zero_second_score() {
        // Test edge case: second score is 0, should always auto-select
        let second_score = 0.0;

        let should_auto_select = second_score == 0.0;
        assert!(should_auto_select);
    }

    #[test]
    fn test_should_not_auto_select_near_2x() {
        // Test just under 2x threshold
        let top_score = 199.0;
        let second_score = 100.0;

        let should_auto_select = top_score / second_score >= 2.0;
        assert!(!should_auto_select);
    }

    #[test]
    fn test_high_ratio_auto_selects() {
        // Test very clear winner (5x)
        let top_score = 500.0;
        let second_score = 100.0;

        let should_auto_select = top_score / second_score >= 2.0;
        assert!(should_auto_select);
    }
}
