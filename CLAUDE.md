# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ggo** is a smart Git branch navigation tool inspired by zoxide. It combines fuzzy matching with a frecency algorithm (frequency + recency) to help users quickly switch between branches with minimal keystrokes.

**Core Value Proposition:** Learn from user habits to intelligently rank and auto-select git branches, reducing the cognitive load of branch navigation.

## Essential Commands

### Development
```bash
# Build the project
cargo build

# Run the binary
./target/debug/ggo <pattern>
cargo run -- <pattern>

# Run tests (184 total)
cargo test                    # All tests
cargo test storage            # Module-specific tests
cargo test --test '*'         # Integration tests only

# Linting and formatting
cargo clippy --all-targets --all-features -- -D warnings  # Lint (should have no warnings)
cargo fmt                                                 # Format code
```

### Testing the Binary
```bash
# Install locally for testing
cargo install --path .

# Test common workflows
ggo --version
ggo --list ""                 # List all branches
ggo feat                      # Fuzzy match + frecency
ggo -                         # Previous branch
ggo --stats                   # Usage statistics
ggo alias m master            # Create alias
```

### Database Management
```bash
# Database location
~/.config/ggo/data.db

# For testing, use isolated database
export GGO_DATA_DIR=/tmp/ggo_test_db

# Clean up test data
rm -rf /tmp/ggo_test_db
```

## Code Architecture

### Module Structure

```
src/
├── main.rs         - CLI entry point and main orchestration logic
├── cli.rs          - Command-line argument parsing (clap)
├── git.rs          - Git operations via libgit2 (get_branches, checkout, etc.)
├── storage.rs      - SQLite database layer (frecency records, aliases)
├── frecency.rs     - Frecency scoring algorithm and ranking
├── matcher.rs      - Fuzzy matching and exact substring matching
├── interactive.rs  - Terminal UI for branch selection (inquire)
├── validation.rs   - Input validation (branch names, repo paths)
└── constants.rs    - Shared constants and configuration values
```

### Key Design Decisions

**1. Git Operations (git.rs)**
- Uses `git2` crate (libgit2) for better performance and reliability
- Validates all branch names before operations to prevent command injection
- All functions return `Result<T>` for consistent error handling

**2. Database Layer (storage.rs)**
- SQLite with schema versioning (currently v2)
- Migrations are incremental and automatic on first connection
- Two main tables: `branches` (frecency data) and `aliases` (per-repo shortcuts)
- Database isolation for tests via `GGO_DATA_DIR` env var
- Per-repo scoping: all queries filter by `repo_path` to ensure data isolation

**3. Frecency Algorithm (frecency.rs)**
- Combines frequency (switch_count) with recency (last_used timestamp)
- Time-based decay: 4.0x (hour), 2.0x (day), 1.0x (week), 0.5x (month), 0.25x (older)
- Final score = `switch_count * recency_weight`
- When combined with fuzzy matching: `fuzzy_score + (frecency_score * 10)`

**4. Branch Selection Logic (main.rs)**
- Priority order: Exact alias → Exact match → Fuzzy match
- Auto-selection threshold: Top score must be ≥2x second score
- Otherwise shows interactive menu (inquire)
- Interactive mode can be forced with `--interactive` flag

**5. Error Handling Strategy**
- Core operations (git, validation) fail fast with detailed error messages
- Storage operations use graceful degradation with warnings
- If frecency tracking fails, continue without it (tool still functional)

### Important Patterns

**Per-Repository Isolation:**
Every database operation is scoped by `repo_path` to ensure:
- Aliases in repo A don't affect repo B
- Frecency scores are per-repo (same branch name in different repos tracked separately)
- No cross-repo data leakage

**Race Condition Prevention:**
Before every checkout, re-verify the branch exists:
```rust
let current_branches = git::get_branches()?;
if !current_branches.contains(&branch_to_checkout) {
    bail!("Branch '{}' no longer exists", branch_to_checkout);
}
```

**Previous Branch Tracking:**
The `ggo -` command mimics `cd -`:
- Saves current branch before every checkout
- Stored in `previous_branch` table (per-repo)
- Only saves if switching to a different branch

### Testing Architecture

**Database Isolation (CRITICAL):**
Tests use a separate database to avoid corrupting production data:
```rust
// In tests
scopeguard::defer! {
    let _ = std::env::remove_var("GGO_DATA_DIR");
}
let test_db_dir = tempfile::tempdir()?;
std::env::set_var("GGO_DATA_DIR", test_db_dir.path());
```

**Test Structure:**
- Unit tests: Inline `#[cfg(test)] mod tests` in each module
- Integration tests: `tests/integration_tests.rs`
- 184 total tests covering all critical paths

**Running Single Tests:**
```bash
cargo test test_name                    # Run specific test
cargo test storage::tests::             # Run all storage tests
cargo test -- --nocapture              # Show println! output
```

## Common Development Patterns

### Adding a New Git Operation
1. Add function to `git.rs` using `git2` crate
2. Validate inputs with `validation::validate_*`
3. Return `Result<T>` with descriptive error context
4. Add unit tests covering success and error cases

### Adding a New Database Table
1. Increment `CURRENT_SCHEMA_VERSION` in `storage.rs`
2. Add migration case in `run_migrations()`
3. Create indices for common queries
4. Add CRUD functions with per-repo filtering
5. Test with `GGO_DATA_DIR` isolation

### Modifying Frecency Algorithm
1. Update `frecency.rs::calculate_score()`
2. Consider impact on existing users (smooth migration)
3. Update constants in `constants.rs` if adding config
4. Run full test suite to verify ranking behavior
5. Test with real-world branch usage patterns

## Important Constraints

### Security
- All user input (patterns, branch names, aliases) is validated before use
- Branch names are validated against git's naming rules
- No shell command concatenation - use `git2` library directly
- SQL queries use parameterized statements (rusqlite handles this)

### Performance
- Database operations should be <5ms (uses indices)
- Branch listing via libgit2 is fast (~10-20ms)
- Total command execution target: <50ms for common operations

### Compatibility
- Rust 1.70+ (specified in rust-toolchain.toml)
- Git 2.0+ (via libgit2)
- Cross-platform: Linux, macOS, Windows (not all CI tested yet)

## Known Technical Debt

See `TECHNICAL_DEBT.md` for comprehensive tracking. Key completed items:
- ✅ Database migrations and versioning
- ✅ Input validation and security
- ✅ Error handling consistency
- ✅ Git operations via libgit2
- ✅ Branch alias system (v0.2.0)

Remaining items include:
- Database cleanup for old/deleted branches
- Logging framework (currently using println!/eprintln!)
- Shell completions (bash/zsh/fish)

## Roadmap Context

Current version: **0.2.0** (Aliases + Fuzzy Matching)

Completed phases:
- Phase 1: Basic pattern matching
- Phase 2: Frecency & smart ranking
- Phase 3: Fuzzy matching & interactive mode
- Phase 5 (partial): Branch aliases

See `ROADMAP.md` for upcoming features (multi-repo tracking, git hooks integration, team sync).

## Git Workflow

**Branch Structure:**
- `master` - Main branch (use for PRs)
- Feature branches follow pattern: `feature/description` or just descriptive names

**Commit Message Style:**
Based on recent commits:
```
<action> <description>

Examples:
- Fix version flag to use lowercase -v instead of -V
- Add database indices for query optimization
- Switch to git2 (libgit2) for better performance (H1)
- Document H2 resolution - indices sufficient (H2)
```

Reference TECHNICAL_DEBT.md issue codes (H1, C3, etc.) when addressing tracked items.

**Pre-Commit Checklist (REQUIRED):**
Before committing any changes, ensure all of the following pass:
```bash
cargo test                                            # All tests must pass
cargo fmt -- --check                                  # Code must be formatted
cargo clippy --all-targets --all-features -- -D warnings  # No clippy warnings allowed
```

If any of these fail, fix the issues before committing.

**Automated Pre-Commit Hooks:**
To automatically run these checks before every commit, you can use `cargo-husky`:
```bash
# Add to Cargo.toml [dev-dependencies]
cargo-husky = "1"

# Or set up manually with git hooks
# Create .git/hooks/pre-commit and make it executable
```

See https://github.com/rhysd/cargo-husky for setup instructions.

## Notes for AI Assistants

**Pre-Commit Requirements (MANDATORY):**
Before committing, these checks MUST all pass:
- `cargo test` - All tests must pass
- `cargo fmt -- --check` - Code must be properly formatted
- `cargo clippy --all-targets --all-features -- -D warnings` - No clippy warnings allowed

**When Making Changes:**
1. Run all pre-commit checks before committing (see above)
2. Consider impact on existing users' databases
3. Update TECHNICAL_DEBT.md if fixing tracked issues
4. Test with actual git repository operations

**When Adding Features:**
1. Consider per-repo isolation requirements
2. Add validation for user inputs
3. Use Result<T> return types consistently
4. Add comprehensive unit tests
5. Test error paths, not just happy paths
6. Consider edge cases (detached HEAD, bare repos, deleted branches)

**Testing Gotchas:**
- Always use `GGO_DATA_DIR` env var in tests
- Clean up with `scopeguard::defer!` to ensure test isolation
- Integration tests need actual git repos (use tempfile::tempdir)
- Some git operations require initial commit to work

**Error Message Style:**
Use bullet points and actionable suggestions:
```rust
bail!(
    "No branches found matching '{}'\n\nTry:\n  • Using a different pattern\n  • Running 'git branch' to see all branches",
    pattern
);
```
