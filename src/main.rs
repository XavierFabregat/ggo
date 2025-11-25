use anyhow::{Context, Result, bail};
use clap::Parser;
use std::io::BufRead;
use std::process::Command;

/// ggo - Smart Git Navigation Tool
///
/// Searches through your git branches and checks out the first
/// branch that matches the given pattern. Pattern matching is done
/// using simple substring matching.
///
/// EXAMPLES:
///     ggo expo         Checkout first branch containing 'expo'
///     ggo feature      Checkout first branch containing 'feature'
///     ggo main         Checkout first branch containing 'main'
///     ggo -l feat      List all branches matching 'feat'
///
/// NOTE:
///     This is the MVP version. Future versions will include:
///     - Frecency-based branch ranking
///     - Fuzzy matching
///     - Interactive selection mode
///     - Repository tracking
#[derive(Parser)]
#[command(name = "ggo")]
#[command(version)]
#[command(about = "Smart Git Navigation Tool", long_about = None)]
struct Cli {
    /// Search pattern to match branch names
    pattern: String,

    /// List matching branches without checking out
    #[arg(short, long)]
    list: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list {
        list_matching_branches(&cli.pattern)?;
    } else {
        let branch = find_and_checkout_branch(&cli.pattern)?;
        println!("Switched to branch '{}'", branch);
    }

    Ok(())
}

fn list_matching_branches(pattern: &str) -> Result<()> {
    let branches = get_git_branches()?;
    let matches: Vec<_> = branches
        .iter()
        .filter(|branch| branch.contains(pattern))
        .collect();

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

fn find_and_checkout_branch(pattern: &str) -> Result<String> {
    let branches = get_git_branches()?;

    let matching_branch = branches
        .iter()
        .find(|branch| branch.contains(pattern))
        .ok_or_else(|| anyhow::anyhow!("No branch found matching '{}'", pattern))?;

    checkout_branch(matching_branch)?;

    Ok(matching_branch.clone())
}

fn get_git_branches() -> Result<Vec<String>> {
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
        .filter_map(|line| line.ok())
        .map(|line| line.trim().trim_start_matches('*').trim().to_string())
        .collect();

    Ok(branches)
}

fn checkout_branch(branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["checkout", branch])
        .output()
        .context("Failed to execute git checkout")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        bail!("Git checkout failed: {}", error.trim());
    }

    Ok(())
}
