use clap::Parser;

/// ggo - Smart Git Navigation Tool
///
/// Searches through your git branches and checks out the best
/// matching branch based on frecency (frequency + recency).
///
/// EXAMPLES:
///     ggo expo         Checkout best branch matching 'expo'
///     ggo feature      Checkout best branch matching 'feature'
///     ggo -l feat      List all branches matching 'feat' with scores
///     ggo -i FEAT      Case-insensitive match for 'FEAT'
///     ggo --interactive feat   Show interactive menu to select branch
///     ggo --stats      Show usage statistics
///
/// NOTE:
///     ggo learns from your usage patterns. The more you use a branch,
///     the higher it ranks in search results.
#[derive(Parser)]
#[command(name = "ggo")]
#[command(version)]
#[command(about = "Smart Git Navigation Tool", long_about = None)]
pub struct Cli {
    /// Search pattern to match branch names
    #[arg(required_unless_present = "stats")]
    pub pattern: Option<String>,

    /// List matching branches without checking out
    #[arg(short, long)]
    pub list: bool,

    /// Case-insensitive pattern matching
    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,

    /// Show interactive menu to select from matches
    #[arg(long)]
    pub interactive: bool,

    /// Show usage statistics
    #[arg(long)]
    pub stats: bool,
}
