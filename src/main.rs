mod cli;
mod git;
mod matcher;

use anyhow::{bail, Result};
use clap::Parser;

use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list {
        list_matching_branches(&cli.pattern, cli.ignore_case)?;
    } else {
        let branch = find_and_checkout_branch(&cli.pattern, cli.ignore_case)?;
        println!("Switched to branch '{}'", branch);
    }

    Ok(())
}

fn list_matching_branches(pattern: &str, ignore_case: bool) -> Result<()> {
    let branches = git::get_branches()?;
    let matches = matcher::filter_branches(&branches, pattern, ignore_case);

    if matches.is_empty() {
        bail!("No branches found matching '{}'", pattern);
    }

    println!("Branches matching '{}':", pattern);
    for (i, branch) in matches.iter().enumerate() {
        let marker = if i == 0 { "→" } else { " " };
        println!("  {} {}", marker, branch);
    }

    if matches.len() > 1 {
        println!("\n({} matches, → indicates checkout target)", matches.len());
    }

    Ok(())
}

fn find_and_checkout_branch(pattern: &str, ignore_case: bool) -> Result<String> {
    let branches = git::get_branches()?;
    let matches = matcher::filter_branches(&branches, pattern, ignore_case);

    let matching_branch = matches
        .first()
        .ok_or_else(|| anyhow::anyhow!("No branch found matching '{}'", pattern))?;

    git::checkout(matching_branch)?;

    Ok(matching_branch.to_string())
}
