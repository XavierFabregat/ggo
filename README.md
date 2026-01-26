# ggo - Smart Git Navigation

> A `zoxide`-style tool for intelligent git branch navigation with frecency-based ranking and aliases

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-184%20passing-brightgreen.svg)]()

## What is ggo?

`ggo` makes git branch navigation as intuitive as `zoxide` makes directory navigation. Instead of typing full branch names or using tab completion, `ggo` learns from your habits and gets you where you need to go with minimal keystrokes.

**Key Features:**
- 🚀 **Smart matching**: Fuzzy search finds branches even with typos
- 🧠 **Learns from you**: Frecency algorithm ranks frequently-used branches higher
- ⚡ **Aliases**: Create shortcuts like `ggo m` → `master`
- 🎯 **Intelligent selection**: Auto-selects when there's a clear winner (2x score threshold)
- 💾 **Previous branch**: Jump back with `ggo -` (like `cd -`)
- 📊 **Statistics**: Track your branch usage patterns

## Installation

### Quick Install (Recommended)

```bash
curl -sSf https://raw.githubusercontent.com/XavierFabregat/ggo/master/install.sh | bash
```

This script will:
- Detect your OS and architecture
- Install Rust if needed (via rustup)
- Try installing from crates.io first (faster)
- Fall back to building from source if needed
- Install to `~/.local/bin` (customizable with `GGO_INSTALL_DIR`)

### From Crates.io

```bash
cargo install ggo
```

### From Source

```bash
git clone https://github.com/XavierFabregat/ggo.git
cd ggo
cargo install --path .
```

### Requirements

- Rust 1.70+ (stable)
- Git 2.0+

## Quick Start

```bash
# Checkout branch with fuzzy matching
ggo feat              # Matches 'feature/auth', 'feat/dashboard', etc.

# List matching branches with scores
ggo --list feature    # See all matches and their frecency scores

# Create an alias
ggo alias m master    # Now 'ggo m' instantly checks out master

# Go back to previous branch
ggo -                 # Like 'cd -' for git

# View your usage statistics
ggo --stats
```

## Usage

### Basic Branch Checkout

```bash
ggo <pattern>         # Smart checkout with fuzzy matching + frecency
ggo expo              # Matches 'expo-feature-branch'
ggo auth              # Your most-used 'auth' branch ranks highest
```

### Listing Branches

```bash
ggo --list feat       # List all branches matching 'feat'
ggo -l feature        # Short form
ggo --list ""         # List all branches with frecency scores
```

### Branch Aliases

```bash
# Create aliases for frequently-used branches
ggo alias m master
ggo alias d develop
ggo alias prod production/main

# Use aliases
ggo m                 # Instant checkout to master

# Manage aliases
ggo alias m           # Show what 'm' points to
ggo alias --list      # List all aliases
ggo alias --remove m  # Remove an alias
```

### Flags & Options

```bash
-l, --list              # List matches without checking out
-i, --ignore-case       # Case-insensitive matching
--no-fuzzy              # Use exact substring matching
--interactive           # Always show selection menu
--stats                 # Show usage statistics
```

## How It Works

### Frecency Algorithm

`ggo` combines **frequency** (how often you use a branch) with **recency** (how recently you used it):

- Used in last hour: 4.0x weight
- Used in last day: 2.0x weight
- Used in last week: 1.0x weight
- Used in last month: 0.5x weight
- Older: 0.25x weight

Your most frequently AND recently used branches automatically rank higher.

### Intelligent Auto-Selection

When multiple branches match your pattern:
- **Clear winner** (≥2x score difference) → Auto-selects
- **Close scores** (<2x difference) → Shows interactive menu

This means fewer prompts when the answer is obvious, but still gives you choice when it matters.

### Per-Repository Aliases

Aliases are scoped per-repository, so `ggo m` can mean:
- `master` in your backend repo
- `main` in your frontend repo
- `develop` in your experimental repo

## Configuration

### Database Location

`ggo` stores branch history and aliases in:
```
~/.config/ggo/data.db  (Linux/macOS)
```

The database uses SQLite with automatic migrations, so upgrading `ggo` won't lose your history.

### Data Tracked

For each branch checkout, `ggo` records:
- Repository path
- Branch name
- Switch count (frequency)
- Last used timestamp (recency)

No sensitive data is collected. Everything stays local.

## Examples

### Typical Workflow

```bash
# First time using ggo - just type what you remember
ggo feature
# Shows interactive menu if multiple 'feature' branches exist

# After a few uses, ggo learns your preference
ggo feat
# Auto-selects your most-used feature branch

# Create aliases for your main branches
ggo alias m master
ggo alias d develop

# Now super fast navigation
ggo m     # → master
ggo d     # → develop
ggo -     # → back to previous
```

### With Multiple Projects

```bash
# In project A
ggo alias auth feature/authentication
ggo auth  # → feature/authentication

# In project B (different repo)
ggo alias auth auth-service
ggo auth  # → auth-service (different branch, same alias!)
```

## Troubleshooting

### "Not a git repository"

Make sure you're running `ggo` from within a git repository:
```bash
git status  # Verify you're in a git repo
```

### "No branches found matching..."

Try:
- Using a shorter pattern: `ggo fea` instead of `ggo feature-auth-v2`
- Listing all branches: `ggo --list ""`
- Using case-insensitive mode: `ggo -i FEATURE`

### Branch history not tracking

Check database permissions:
```bash
ls -la ~/.config/ggo/
# Should be readable and writable
```

### Frecency scores seem wrong

`ggo` ranks by usage patterns. If you just created a branch, it won't rank high yet. Use it a few times and it will climb the rankings.

## Development

### Running Tests

```bash
cargo test              # All tests (184 total)
cargo test storage      # Storage tests only
cargo test --test '*'   # Integration tests
```

### Linting

```bash
cargo fmt               # Format code
cargo clippy            # Run linter
```

### Building

```bash
cargo build             # Debug build
cargo build --release   # Optimized build
```

## Project Structure

```
ggo/
├── src/
│   ├── main.rs           # CLI entry and main logic
│   ├── cli.rs            # Command-line argument parsing
│   ├── git.rs            # Git operations wrapper
│   ├── matcher.rs        # Fuzzy and exact matching
│   ├── storage.rs        # SQLite database layer
│   ├── frecency.rs       # Frecency scoring algorithm
│   └── interactive.rs    # Terminal UI for selection
├── tests/
│   └── integration_tests.rs
├── ROADMAP.md            # Feature roadmap
├── TECHNICAL_DEBT.md     # Known issues and improvements
└── README.md             # This file
```

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the full feature roadmap.

**Current Status:** Phase 3 (Fuzzy Matching + Aliases)

**Completed:**
- ✅ Phase 1: Basic pattern matching
- ✅ Phase 2: Frecency & smart ranking
- ✅ Phase 3: Fuzzy matching & interactive mode
- ✅ Phase 5 (partial): Branch aliases

**Upcoming:**
- Phase 4: Repository tracking
- Phase 5: Advanced features (statistics, team sync)

## Contributing

Contributions are welcome! Please:

1. Check [TECHNICAL_DEBT.md](TECHNICAL_DEBT.md) for known issues
2. Write tests for new features
3. Run `cargo test` and `cargo clippy` before submitting
4. Follow the existing code style

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Inspired by [zoxide](https://github.com/ajeetdsouza/zoxide) - the smarter `cd` command.

---

**Made with ❤️ by [Xavier Fabregat](https://github.com/XavierFabregat)**
