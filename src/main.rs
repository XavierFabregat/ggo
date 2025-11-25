mod cli;
mod frecency;
mod git;
mod interactive;
mod matcher;
mod storage;

use anyhow::{bail, Result};
use clap::Parser;

use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.stats {
        show_stats()?;
        return Ok(());
    }

    let pattern = cli.pattern.as_deref().unwrap_or("");

    // Handle the special '-' pattern to go back to previous branch
    if pattern == "-" {
        checkout_previous_branch()?;
        return Ok(());
    }

    if cli.list {
        list_matching_branches(pattern, cli.ignore_case)?;
    } else {
        let branch = find_and_checkout_branch(pattern, cli.ignore_case, cli.interactive)?;
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

fn list_matching_branches(pattern: &str, ignore_case: bool) -> Result<()> {
    let branches = git::get_branches()?;
    let matches = matcher::filter_branches(&branches, pattern, ignore_case);

    if matches.is_empty() {
        bail!("No branches found matching '{}'", pattern);
    }

    // Get frecency data for ranking
    let repo_path = git::get_repo_root().unwrap_or_default();
    let records = storage::get_branch_records(&repo_path).unwrap_or_default();
    
    // Convert matches to owned strings for frecency sorting
    let match_strings: Vec<String> = matches.iter().map(|s| s.to_string()).collect();
    let ranked = frecency::sort_branches_by_frecency(&match_strings, &records);

    println!("Branches matching '{}' (ranked by frecency):\n", pattern);
    for (i, (branch, score)) in ranked.iter().enumerate() {
        let marker = if i == 0 { "→" } else { " " };
        let score_display = if *score > 0.0 {
            format!(" ({:.1})", score)
        } else {
            String::new()
        };
        println!("  {} {}{}", marker, branch, score_display);
    }

    if ranked.len() > 1 {
        println!("\n({} matches, → indicates checkout target)", ranked.len());
    }

    Ok(())
}

fn checkout_previous_branch() -> Result<()> {
    let repo_path = git::get_repo_root()?;
    
    let previous_branch = storage::get_previous_branch(&repo_path)?
        .ok_or_else(|| anyhow::anyhow!("No previous branch found"))?;

    // Save current branch before switching
    if let Ok(current_branch) = git::get_current_branch() {
        if let Err(e) = storage::save_previous_branch(&repo_path, &current_branch) {
            eprintln!("Warning: Failed to save current branch: {}", e);
        }
    }

    // Checkout the previous branch
    git::checkout(&previous_branch)?;

    // Record the checkout for frecency tracking
    if let Err(e) = storage::record_checkout(&repo_path, &previous_branch) {
        eprintln!("Warning: Failed to record checkout: {}", e);
    }

    println!("Switched to branch '{}'", previous_branch);
    Ok(())
}

fn find_and_checkout_branch(pattern: &str, ignore_case: bool, interactive: bool) -> Result<String> {
    let branches = git::get_branches()?;
    let matches = matcher::filter_branches(&branches, pattern, ignore_case);

    if matches.is_empty() {
        bail!("No branch found matching '{}'", pattern);
    }

    // Get frecency data for ranking
    let repo_path = git::get_repo_root().unwrap_or_default();
    let records = storage::get_branch_records(&repo_path).unwrap_or_default();
    
    // Convert matches to owned strings for frecency sorting
    let match_strings: Vec<String> = matches.iter().map(|s| s.to_string()).collect();

    // Determine which branch to checkout
    let branch_to_checkout = if interactive || match_strings.len() > 1 {
        // Use interactive mode if explicitly requested OR if there are multiple matches
        interactive::select_branch(&match_strings, &records)?
    } else {
        // Single match or non-interactive: use best frecency match
        let ranked = frecency::sort_branches_by_frecency(&match_strings, &records);
        ranked[0].0.clone()
    };

    // Save current branch as previous before switching
    if let Ok(current_branch) = git::get_current_branch() {
        // Only save if we're switching to a different branch
        if current_branch != branch_to_checkout {
            if let Err(e) = storage::save_previous_branch(&repo_path, &current_branch) {
                eprintln!("Warning: Failed to save current branch: {}", e);
            }
        }
    }

    // Checkout the branch
    git::checkout(&branch_to_checkout)?;

    // Record the checkout for frecency tracking
    if let Err(e) = storage::record_checkout(&repo_path, &branch_to_checkout) {
        // Don't fail the checkout if recording fails, just warn
        eprintln!("Warning: Failed to record checkout: {}", e);
    }

    Ok(branch_to_checkout)
}
