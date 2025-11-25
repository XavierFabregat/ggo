use clap::Parser;

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
///     ggo -i FEAT      Case-insensitive match for 'FEAT'
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
pub struct Cli {
    /// Search pattern to match branch names
    pub pattern: String,

    /// List matching branches without checking out
    #[arg(short, long)]
    pub list: bool,

    /// Case-insensitive pattern matching
    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,
}

