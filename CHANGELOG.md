# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-01-27

### Added
- **Structured error handling** with thiserror for clear, actionable error messages
- **Shell completion generation** for bash, zsh, fish, powershell, and elvish (`--generate-completion` flag)
- **Database cleanup commands** (`ggo cleanup --deleted`, `--optimize`, `--size`, `--older-than`)
- **Configuration file support** at `~/.config/ggo/config.toml` for customizing behavior
- **Enhanced statistics visualization** with ASCII bar charts and repository breakdown tables
- **Configurable auto-select threshold** via config file
- **Homebrew distribution** via XavierFabregat/tap for fast macOS/Linux installation

### Changed
- Error messages now provide helpful context and actionable suggestions
- Statistics command (`--stats`) now displays beautiful ASCII visualizations
- Frecency algorithm uses exponential decay (configurable via `half_life_days`)
- Improved error types with specific variants for common failure cases

### Fixed
- Test count in README badge (now accurately shows 271 tests)
- Error display now uses user-friendly messages instead of debug output

### Security
- All error paths validated and tested
- Input validation covers edge cases

## [0.3.0] - 2026-01-26

### Added
- Database cleanup CLI with multiple maintenance options
- Shell completions for 5 different shells
- Enhanced statistics with ASCII bar charts and tables
- Configuration file support for frecency and behavior customization
- Repository breakdown in statistics view

### Changed
- Statistics visualization significantly improved with visual elements
- Database operations now have user-accessible cleanup tools

## [0.2.2] - 2026-01-26

### Added
- Homebrew tap distribution with automated SHA256 updates
- Automated GitHub workflows for Homebrew formula updates
- Pre-built binaries via tar.gz archives for faster installation

### Changed
- Release workflow now creates both tar.gz and raw binaries
- Installation documentation updated with Homebrew as primary method

## [0.2.1] - 2025-12-17

### Added
- Published to crates.io for the first time
- Crates.io metadata (description, keywords, categories)
- Installation via `cargo install ggo`

## [0.2.0] - 2025-12-16

### Added
- **Branch alias system** - Create per-repository shortcuts (`ggo alias m master`)
- Alias management commands (create, list, remove)
- Per-repository alias isolation
- Database schema versioning with migrations
- Comprehensive input validation and security measures
- Structured logging with tracing framework
- Database indices for improved query performance
- Exponential decay frecency algorithm (replaced stepped weights)
- Database cleanup and maintenance functions
- Common test utilities module

### Changed
- Switched from git commands to libgit2 for better performance and reliability
- Error handling now uses graceful degradation with warnings
- Frecency algorithm improved with 1-week half-life exponential decay
- All magic numbers extracted to constants module

### Fixed
- Race conditions in branch checkout operations
- Inconsistent error handling throughout codebase
- Database performance with proper indexing
- Test database isolation to prevent production data corruption

### Security
- Added comprehensive input validation (branch names, patterns, paths)
- Protection against command injection
- SQL injection prevention with parameterized queries
- Branch name validation against git naming rules

## [0.1.4] - 2025-11-25

### Added
- Intelligent auto-selection with 2x score threshold
- Interactive branch selection improvements

### Changed
- Branch switching now auto-selects when there's a clear winner
- Reduced unnecessary prompts for obvious choices

## [0.1.1] - 2025-11-25

### Added
- CI workflow for automated testing and linting
- Initial test suite

### Changed
- Improved code formatting and organization

## [0.1.0] - 2025-11-25

### Added
- Initial release of ggo
- Basic branch pattern matching with substring search
- Frecency-based branch ranking (frequency + recency)
- Fuzzy matching for forgiving pattern matching
- Interactive mode for branch selection
- Previous branch tracking (`ggo -`)
- Statistics command (`--stats`)
- SQLite database for usage tracking
- Support for case-insensitive matching (`-i` flag)
- List mode to preview matches (`-l` flag)

### Technical
- Built with Rust using libgit2
- Cross-platform support (Linux, macOS, Windows)
- Database stored at `~/.config/ggo/data.db`
- 184 tests ensuring reliability

[1.0.0]: https://github.com/XavierFabregat/ggo/compare/v0.3.0...v1.0.0
[0.3.0]: https://github.com/XavierFabregat/ggo/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/XavierFabregat/ggo/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/XavierFabregat/ggo/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/XavierFabregat/ggo/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/XavierFabregat/ggo/compare/v0.1.1...v0.1.4
[0.1.1]: https://github.com/XavierFabregat/ggo/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/XavierFabregat/ggo/releases/tag/v0.1.0
