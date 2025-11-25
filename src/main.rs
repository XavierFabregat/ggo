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
        list_matching_branches(pattern, cli.ignore_case, !cli.no_fuzzy)?;
    } else {
        let branch = find_and_checkout_branch(pattern, cli.ignore_case, !cli.no_fuzzy, cli.interactive)?;
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
    let repo_path = git::get_repo_root().unwrap_or_default();
    let records = storage::get_branch_records(&repo_path).unwrap_or_default();

    let ranked = if use_fuzzy {
        // Use fuzzy matching and combine with frecency
        let fuzzy_matches = matcher::fuzzy_filter_branches(&branches, pattern, ignore_case);
        
        if fuzzy_matches.is_empty() {
            bail!("No branches found matching '{}'", pattern);
        }

        combine_fuzzy_and_frecency_scores(&fuzzy_matches, &records)
    } else {
        // Use exact substring matching
        let matches = matcher::filter_branches(&branches, pattern, ignore_case);
        
        if matches.is_empty() {
            bail!("No branches found matching '{}'", pattern);
        }

        let match_strings: Vec<String> = matches.iter().map(|s| s.to_string()).collect();
        frecency::sort_branches_by_frecency(&match_strings, &records)
    };

    let match_type = if use_fuzzy { "fuzzy matching" } else { "substring matching" };
    println!("Branches matching '{}' ({}+ frecency):\n", pattern, match_type);
    
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
            // Frecency gets a 10x multiplier to give it significant weight
            let combined_score = fuzzy_score + (frecency_score * 10.0);
            
            (m.branch.clone(), combined_score)
        })
        .collect();
    
    // Sort by combined score descending
    combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    combined
}

fn find_and_checkout_branch(pattern: &str, ignore_case: bool, use_fuzzy: bool, interactive: bool) -> Result<String> {
    let branches = git::get_branches()?;
    let repo_path = git::get_repo_root().unwrap_or_default();
    let records = storage::get_branch_records(&repo_path).unwrap_or_default();

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
    let branch_to_checkout = if interactive || ranked.len() > 1 {
        // Use interactive mode if explicitly requested OR if there are multiple matches
        let branch_list: Vec<String> = ranked.iter().map(|(b, _)| b.clone()).collect();
        interactive::select_branch(&branch_list, &records)?
    } else {
        // Single match or non-interactive: use best match
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
