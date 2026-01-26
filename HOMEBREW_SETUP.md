# Homebrew Distribution Setup Guide

This guide explains the automated Homebrew distribution system for ggo.

## What We've Built

### 1. Release Workflow Enhancement
**File:** `.github/workflows/release.yaml`

**Changes:**
- Now creates both `.tar.gz` archives AND raw binaries
- Tar.gz files are used by Homebrew
- Raw binaries kept for backward compatibility

**Outputs per release:**
- `ggo-macos-arm64.tar.gz` + `ggo-macos-arm64`
- `ggo-macos-amd64.tar.gz` + `ggo-macos-amd64`
- `ggo-linux-amd64.tar.gz` + `ggo-linux-amd64`
- `ggo-windows-amd64.exe` (no tar.gz for Windows)

### 2. Homebrew Formula
**File:** `Formula/ggo.rb`

**Features:**
- Platform-specific downloads (macOS ARM/Intel, Linux)
- SHA256 verification for security
- Simple `bin.install` - just copies the binary
- Version test to verify installation

### 3. Automated SHA256 Updates
**File:** `.github/workflows/update-homebrew.yaml`

**How it works:**
1. Triggered when a new release is published
2. Downloads all tar.gz files from the release
3. Calculates SHA256 hashes
4. Updates the formula in the homebrew-tap repo
5. Commits and pushes automatically

**Requires:** `HOMEBREW_TAP_TOKEN` secret (see setup below)

## Setup Steps

### Step 1: Create homebrew-tap Repository

```bash
# On GitHub, create a new PUBLIC repository named: homebrew-tap
# Or use CLI:
gh repo create XavierFabregat/homebrew-tap --public --description "Homebrew formulae for ggo"
```

### Step 2: Initialize homebrew-tap

```bash
# Clone the new repo
git clone https://github.com/XavierFabregat/homebrew-tap.git
cd homebrew-tap

# Create Formula directory
mkdir Formula

# Copy the formula from this repo
cp /path/to/ggo/Formula/ggo.rb Formula/

# Commit and push
git add Formula/ggo.rb
git commit -m "Add ggo formula v0.2.1"
git push origin main
```

### Step 3: Create GitHub Token for Automation

```bash
# Create a Personal Access Token with 'repo' scope
# Go to: https://github.com/settings/tokens/new

# Token needs:
# - repo (all permissions)

# Add as repository secret:
gh secret set HOMEBREW_TAP_TOKEN --repo XavierFabregat/ggo
# Paste your token when prompted
```

### Step 4: Commit Changes to Feature Branch

```bash
# From ggo repository
git add .github/workflows/release.yaml
git add .github/workflows/update-homebrew.yaml
git add Formula/ggo.rb
git add HOMEBREW_SETUP.md
git commit -m "Add Homebrew distribution with automated SHA256 updates

- Update release workflow to create tar.gz archives
- Add Homebrew formula with platform-specific binaries
- Add automation to update formula SHA256s on release
- Keep raw binaries for backward compatibility"
```

### Step 5: Test the Setup

After merging to master and creating v0.2.1 tag:

```bash
# The release workflow will:
1. Build binaries for all platforms
2. Create tar.gz archives
3. Upload both to GitHub releases

# The update-homebrew workflow will:
1. Download the tar.gz files
2. Calculate SHA256 hashes
3. Update homebrew-tap/Formula/ggo.rb
4. Commit and push to homebrew-tap
```

## User Installation

Once setup is complete, users can install with:

```bash
# Add the tap
brew tap XavierFabregat/tap

# Install ggo
brew install ggo

# Or in one command
brew install XavierFabregat/tap/ggo
```

## Maintenance

### Releasing a New Version

1. Update version in `Cargo.toml`
2. Commit to master
3. Create a git tag: `git tag -a v0.2.2 -m "Release v0.2.2"`
4. Push tag: `git push origin v0.2.2`
5. Everything else is automatic!

The workflows will:
- Build and upload binaries
- Calculate new SHA256s
- Update Homebrew formula
- Users get the update via `brew upgrade ggo`

### Manual Formula Update (if needed)

If automation fails, manually update SHA256s:

```bash
# Download the release
VERSION=0.2.1
curl -sL "https://github.com/XavierFabregat/ggo/releases/download/v${VERSION}/ggo-macos-arm64.tar.gz" -o macos-arm64.tar.gz
curl -sL "https://github.com/XavierFabregat/ggo/releases/download/v${VERSION}/ggo-macos-amd64.tar.gz" -o macos-amd64.tar.gz
curl -sL "https://github.com/XavierFabregat/ggo/releases/download/v${VERSION}/ggo-linux-amd64.tar.gz" -o linux-amd64.tar.gz

# Calculate SHA256s
shasum -a 256 *.tar.gz

# Update Formula/ggo.rb in homebrew-tap repo
# Replace PLACEHOLDER_* with actual hashes
# Commit and push
```

## Troubleshooting

### Workflow fails with "404 Not Found"
- Check that the release exists and has tar.gz files
- Verify the tag format is `vX.Y.Z` (e.g., `v0.2.1`)

### SHA256 mismatch on user install
- Homebrew caches formulas, have users run: `brew update`
- Or clear cache: `brew cleanup ggo`

### Token permission issues
- Verify `HOMEBREW_TAP_TOKEN` has `repo` scope
- Check token hasn't expired
- Ensure homebrew-tap repo is public

## Benefits of This Setup

✅ **Zero manual work** - SHA256s update automatically
✅ **Fast installs** - Pre-built binaries, no Rust/cargo needed
✅ **Secure** - SHA256 verification on every install
✅ **Professional** - Users expect `brew install` for CLI tools
✅ **Discoverable** - Shows up in `brew search`

## Next Steps

After merging this:
1. Submit to official Homebrew (optional, requires 30+ GitHub stars)
2. Add shell completions
3. Consider Winget for Windows users
