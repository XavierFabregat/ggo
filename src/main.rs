use std::env;
use std::process::{Command, exit};
use std::io::BufRead;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: ggo <pattern>");
        eprintln!("Example: ggo expo");
        exit(1);
    }

    let pattern = &args[1];

    match find_and_checkout_branch(pattern) {
        Ok(branch) => println!("Switched to branch '{}'", branch),
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}

fn find_and_checkout_branch(pattern: &str) -> Result<String, String> {
    let branches = get_git_branches()?;

    let matching_branch = branches
        .iter()
        .find(|branch| branch.contains(pattern))
        .ok_or_else(|| format!("No branch found matching '{}'", pattern))?;

    checkout_branch(matching_branch)?;

    Ok(matching_branch.clone())
}

fn get_git_branches() -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(&["branch"])
        .output()
        .map_err(|e| format!("Failed to execute git branch: {}", e))?;

    if !output.status.success() {
        return Err("Not a git repository or git command failed".to_string());
    }

    let branches: Vec<String> = output
        .stdout
        .lines()
        .filter_map(|line| line.ok())
        .map(|line| {
            line.trim()
                .trim_start_matches('*')
                .trim()
                .to_string()
        })
        .collect();

    Ok(branches)
}

fn checkout_branch(branch: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(&["checkout", branch])
        .output()
        .map_err(|e| format!("Failed to execute git checkout: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git checkout failed: {}", error));
    }

    Ok(())
}
