# Contributing to ggo

Thank you for your interest in contributing to ggo!

## Before You Start

1. Check [TECHNICAL_DEBT.md](TECHNICAL_DEBT.md) for known issues
2. Search existing [issues](https://github.com/XavierFabregat/ggo/issues) to avoid duplicates
3. For major changes, open an issue first to discuss the approach

## Development Setup

```bash
# Clone the repository
git clone https://github.com/XavierFabregat/ggo.git
cd ggo

# Install dependencies (Rust 1.70+ required)
# The project uses standard Cargo workflow

# Run tests
cargo test

# Build the project
cargo build

# Install locally for testing
cargo install --path .

# Test the installed binary
ggo --version
```

## Code Quality Standards

Before submitting a pull request, ensure all checks pass:

```bash
# All tests must pass
cargo test

# Code must be properly formatted
cargo fmt

# No clippy warnings allowed
cargo clippy --all-targets --all-features -- -D warnings
```

**Pre-push hooks are automatically installed** via cargo-husky when you run `cargo test` for the first time.

## Development Workflow

### Branch Structure
- `master` - Stable, production releases only
- `dev` - Integration branch for testing
- `release-X.Y.Z` - Release preparation branches
- `feat/feature-name` - Feature development
- `fix/issue-description` - Bug fixes

### Making Changes

1. **Create a feature branch** from `master`:
   ```bash
   git checkout master
   git pull
   git checkout -b feat/your-feature-name
   ```

2. **Make your changes** with tests:
   - Write tests first (TDD approach preferred)
   - Implement the feature
   - Ensure tests pass

3. **Follow code standards**:
   - Use Result<T> for error handling
   - Add validation for user inputs
   - Follow existing code patterns
   - Add comprehensive tests

4. **Commit your changes**:
   ```bash
   git add .
   git commit -m "Add feature description"
   ```

5. **Push and create PR**:
   ```bash
   git push origin feat/your-feature-name
   gh pr create --base dev
   ```

## Commit Message Format

Follow the existing style:

```
<action> <description>

Examples:
- Add shell completion generation feature
- Fix race condition in branch checkout
- Update README with installation instructions
```

Reference technical debt items when addressing them:
```
Fix database performance issues (H2)
```

## Testing Guidelines

### Writing Tests

- **Unit tests**: Test individual functions in isolation
- **Integration tests**: Test end-to-end workflows
- **Test edge cases**: Empty inputs, special characters, race conditions

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_works() {
        // Arrange
        let input = "test";

        // Act
        let result = function_under_test(input);

        // Assert
        assert!(result.is_ok());
    }
}
```

### Database Test Isolation

Always use isolated test databases:

```rust
scopeguard::defer! {
    let _ = std::env::remove_var("GGO_DATA_DIR");
}
let test_db_dir = tempfile::tempdir()?;
std::env::set_var("GGO_DATA_DIR", test_db_dir.path());
```

## Pull Request Process

1. **Ensure your PR**:
   - Has a clear description
   - References related issues (if any)
   - Includes tests for new functionality
   - Updates documentation if needed
   - Passes all CI checks

2. **PR Review**:
   - Address review feedback promptly
   - Keep commits focused and logical
   - Rebase on latest `dev` if needed

3. **After Approval**:
   - PR will be merged to `dev` first
   - After testing, merged to `master`
   - Included in next release

## Code Style

### Rust Idioms
- Use `?` operator for error propagation
- Prefer `if let` over `match` for Option/Result when appropriate
- Use iterators and functional style where clear
- Keep functions small and focused

### Error Handling
- Use the custom `GgoError` types from `src/error.rs`
- Provide helpful error messages with actionable suggestions
- Never panic in production code (use `Result` instead)

### Performance
- Target <50ms for common operations
- Use database indices for queries
- Avoid unnecessary allocations

## Adding New Features

### Per-Repository Isolation
If your feature involves database operations, ensure per-repo scoping:

```rust
// Always filter by repo_path
conn.execute(
    "SELECT * FROM table WHERE repo_path = ?1",
    [repo_path],
)?;
```

### Database Migrations
When adding database tables:

1. Increment `CURRENT_SCHEMA_VERSION` in `storage.rs`
2. Add migration in `run_migrations()`
3. Add indices for common queries
4. Test with fresh database and migration from previous version

## Documentation

- Update README.md for user-facing changes
- Update CLAUDE.md for development guidance
- Add inline comments for complex logic
- Update TECHNICAL_DEBT.md when fixing tracked items

## Getting Help

- **Questions?** Open an [issue](https://github.com/XavierFabregat/ggo/issues)
- **Bug reports:** Include `ggo --version`, OS, and steps to reproduce
- **Feature requests:** Explain the use case and expected behavior

## Code of Conduct

- Be respectful and constructive
- Focus on the code, not the person
- Help create a welcoming environment

---

Thank you for contributing to ggo! 🎉
