use anyhow::{bail, Context, Result};
use std::io::BufRead;
use std::process::Command;

/// Get all local git branches in the current repository
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
        .filter_map(|line| line.ok())
        .map(|line| line.trim().trim_start_matches('*').trim().to_string())
        .collect();

    Ok(branches)
}

/// Checkout the specified branch
pub fn checkout(branch: &str) -> Result<()> {
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

/// Get the root path of the current git repository
pub fn get_repo_root() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to execute git rev-parse")?;

    if !output.status.success() {
        bail!("Not a git repository");
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(path)
}

/// Get the name of the current branch
pub fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to execute git branch --show-current")?;

    if !output.status.success() {
        bail!("Failed to get current branch (detached HEAD?)");
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    if branch.is_empty() {
        bail!("Not on a branch (detached HEAD)");
    }
    
    Ok(branch)
}

