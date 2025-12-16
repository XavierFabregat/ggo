# ggo - Smart Git Navigation Tool

> A `zoxide`-style tool for intelligent git branch and repository navigation

## Vision

`ggo` (git go / grep go) aims to make git navigation as intuitive and efficient as `zoxide` makes directory navigation. Instead of typing full branch names or using tab completion, `ggo` learns from your habits and gets you where you need to go with minimal keystrokes.

## Phases

### Phase 1: MVP - Basic Pattern Matching ✓ (Current)
**Goal:** Replace the bash function with a working Rust CLI

**Features:**
- Accept a search pattern as argument
- List git branches
- Filter by pattern (substring match)
- Checkout first match
- Basic error handling

**Usage:**
```bash
ggo expo      # checkout first branch matching "expo"
ggo feature   # checkout first branch matching "feature"
```

**Technical:**
- Single `main.rs` implementation
- Use `std::process::Command` for git operations
- Simple string matching with `contains()`
- No persistence needed yet

---

### Phase 2: Frecency & Smart Ranking
**Goal:** Learn from usage patterns to prioritize branches

**Features:**
- Track branch usage (frequency + recency)
- Store usage data persistently
- Rank matches by frecency score (like zoxide does)
- `ggo feat` picks your most-used feature branch, not alphabetical first

**New Commands:**
```bash
ggo <pattern>           # smart checkout by frecency
ggo --list <pattern>    # show ranked matches
ggo --stats             # show usage statistics
```

**Technical:**
- Implement frecency algorithm (frequency × recency decay)
- Add SQLite or JSON storage (`~/.config/ggo/data.db`)
- Track: branch name, switch count, last used timestamp
- Update stats on every successful checkout

---

### Phase 3: Fuzzy Matching & Interactive Mode
**Goal:** More forgiving pattern matching and user choice

**Features:**
- Fuzzy matching algorithm (`ggo exo` matches "expo-feature-branch")
- Interactive mode when multiple good matches exist
- Keyboard navigation for selection
- Preview branch info before switching

**New Commands:**
```bash
ggo <pattern>              # fuzzy match with frecency
ggo -i <pattern>           # interactive mode (always show menu)
ggo --no-fuzzy <pattern>   # fallback to exact matching
```

**Technical:**
- Fuzzy matching library (e.g., `fuzzy-matcher` crate)
- Interactive TUI (e.g., `dialoguer` or `crossterm` + `ratatui`)
- Score matches by: fuzzy distance + frecency score
- Display: branch name, last used, switch count

---

### Phase 4: Repository Tracking
**Goal:** Navigate between repositories, not just branches

**Features:**
- Track multiple git repositories
- Quick jump between repos
- Remember favorite repos
- Show repo-specific branch stats

**New Commands:**
```bash
ggo repo <pattern>         # switch to a tracked repository
ggo repo add <path>        # manually add a repo to tracking
ggo repo list              # show all tracked repos
ggo --in <repo> <branch>   # checkout branch in specific repo
```

**Technical:**
- Store repo paths and metadata
- Scan common directories for git repos (e.g., `~/Code/**`)
- Track per-repo branch usage
- Handle repo context switching

---

### Phase 5: Advanced Features
**Goal:** Power user features and optimizations

**Features:**
- **Recent branch history:** Quick access to last N branches
  ```bash
  ggo --recent          # show recent branches
  ggo --back            # go to previous branch (like cd -)
  ```

- **Branch aliases:** Custom shortcuts
  ```bash
  ggo alias main=m      # ggo m → checkout main
  ggo alias feature/auth=auth
  ```

- **Statistics & insights:**
  ```bash
  ggo stats --top 10    # most used branches
  ggo stats --stale     # branches not used in 30 days
  ggo stats --graph     # visual usage patterns
  ```

- **Team sync:** Share frecency data
  ```bash
  ggo sync --export     # export usage data for team
  ggo sync --import     # import team patterns
  ```

- **Git integration:** Hook into git events
  - Auto-track on any branch switch
  - Clean up deleted branches from database

**Technical:**
- Implement git hooks for automatic tracking
- Add visualization (e.g., `termgraph`)
- Export/import formats (JSON)
- Background cleanup jobs

---

## Technical Architecture

### Project Structure
```
ggo/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── git.rs               # Git operations wrapper
│   ├── matcher.rs           # Pattern matching (exact, fuzzy)
│   ├── storage.rs           # Data persistence layer
│   ├── frecency.rs          # Frecency algorithm
│   ├── interactive.rs       # TUI for selection
│   └── config.rs            # User configuration
├── Cargo.toml
├── ROADMAP.md
└── README.md
```

### Key Dependencies (Future)
- `clap` - CLI argument parsing
- `rusqlite` or `serde_json` - Data persistence
- `fuzzy-matcher` - Fuzzy string matching
- `dialoguer` or `ratatui` - Interactive UI
- `anyhow` - Error handling
- `dirs` - Cross-platform config directories

### Data Storage
```
~/.config/ggo/
├── config.toml          # User settings
├── data.db              # SQLite database
└── aliases.json         # Custom aliases
```

### Database Schema (Phase 2+)
```sql
-- Branches table
CREATE TABLE branches (
    id INTEGER PRIMARY KEY,
    repo_path TEXT NOT NULL,
    branch_name TEXT NOT NULL,
    switch_count INTEGER DEFAULT 1,
    last_used TIMESTAMP,
    created_at TIMESTAMP,
    UNIQUE(repo_path, branch_name)
);

-- Repositories table (Phase 4)
CREATE TABLE repositories (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    name TEXT,
    last_used TIMESTAMP,
    access_count INTEGER DEFAULT 0
);
```

---

## Design Principles

1. **Speed First** - Every operation should feel instant
2. **Learn, Don't Configure** - Minimize configuration, maximize learning from usage
3. **Fail Gracefully** - Never leave the user in a broken git state
4. **Progressive Enhancement** - Each phase adds value without breaking previous functionality
5. **Cross-Platform** - Works on macOS, Linux, and Windows
6. **Single Binary** - Easy to install and distribute

---

## Success Metrics

- **Phase 1:** Successfully replaces bash function
- **Phase 2:** 80%+ of checkouts are "correct" on first try
- **Phase 3:** Users prefer `ggo` over tab completion
- **Phase 4:** Manages 10+ repositories seamlessly
- **Phase 5:** Becomes the primary git navigation tool

---

## Open Questions

- Should we support `git worktree` navigation?
- How to handle branches with the same name in different repos?
- Should we track remote branches separately?
- Integration with GitHub/GitLab for PR branches?
- Should we support other VCS (Mercurial, SVN)?

---

## Known Issues / Improvements

### CLI Flag Conflict
**Issue:** The `-i` flag is currently used for `--ignore-case`, but the roadmap originally planned it for interactive mode.

**Current state:**
- `-i` = `--ignore-case` (case-insensitive matching)
- `--interactive` = interactive mode (long form only)

**Proposed solution:**
- Keep `-i` for `--ignore-case` (common convention in Unix tools)
- Add `-I` (capital I) as short flag for `--interactive`
- This maintains compatibility and follows common CLI patterns

**Reference:** Phase 3 line 71 shows the original plan for `-i` as interactive mode

---

## Getting Started

Current status: **Phase 1 (MVP)**

Next steps:
1. Implement basic pattern matching in Rust
2. Add comprehensive error handling
3. Write tests for git operations
4. Create installation instructions
5. Gather user feedback before Phase 2
