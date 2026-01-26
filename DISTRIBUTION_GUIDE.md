# 📦 Complete Guide: Package Manager Distribution for ggo

## Why This Matters

**Current state:** Users need Rust + cargo to install
- Blocks 95% of potential users
- High friction for trying the tool

**With package managers:**
```bash
brew install ggo        # macOS/Linux
winget install ggo      # Windows
```
- Zero friction
- Professional appearance
- Automatic updates
- Massive discoverability boost

## ✅ **Completed: curl Install Script**

**Status:** ✅ Implemented (install.sh)

A curl-based install script is now available:
```bash
curl -sSf https://raw.githubusercontent.com/XavierFabregat/ggo/master/install.sh | bash
```

**Features:**
- Detects OS and architecture
- Checks for Rust/cargo, offers to install if missing
- Tries crates.io first (fast), falls back to source build
- Installs to `~/.local/bin` (customizable via `GGO_INSTALL_DIR`)
- Provides clear error messages and PATH instructions
- Works on macOS, Linux, and Windows (Git Bash/WSL)

**Benefits:**
- Lowers barrier for users without Rust
- Professional appearance
- Common pattern (like rustup, homebrew, etc.)
- Bridges gap until official package managers

**Complements package managers** - Users can try ggo immediately while we work on Homebrew/Winget.

---

## 🍺 **1. Homebrew (macOS & Linux)** - HIGHEST PRIORITY

**Impact:** 10M+ active users, de-facto standard for macOS developers

### Step 1: Create a Homebrew Tap

```bash
# Create a new repo: homebrew-tap
mkdir homebrew-tap
cd homebrew-tap

# Create formula directory
mkdir Formula
```

### Step 2: Create Formula File

**Formula/ggo.rb:**
```ruby
class Ggo < Formula
  desc "Smart git branch navigation with frecency-based ranking"
  homepage "https://github.com/XavierFabregat/ggo"
  version "0.2.0"
  license "MIT"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/XavierFabregat/ggo/releases/download/v0.2.0/ggo-macos-arm64"
    sha256 "FILL_IN_SHA256_HERE"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/XavierFabregat/ggo/releases/download/v0.2.0/ggo-macos-amd64"
    sha256 "FILL_IN_SHA256_HERE"
  elsif OS.linux? && Hardware::CPU.intel?
    url "https://github.com/XavierFabregat/ggo/releases/download/v0.2.0/ggo-linux-amd64"
    sha256 "FILL_IN_SHA256_HERE"
  end

  def install
    bin.install "ggo-macos-arm64" => "ggo" if OS.mac? && Hardware::CPU.arm?
    bin.install "ggo-macos-amd64" => "ggo" if OS.mac? && Hardware::CPU.intel?
    bin.install "ggo-linux-amd64" => "ggo" if OS.linux?
  end

  test do
    system "#{bin}/ggo", "--version"
  end
end
```

### Step 3: Generate SHA256 Checksums

```bash
# After each release, generate checksums
shasum -a 256 ggo-macos-arm64
shasum -a 256 ggo-macos-amd64
shasum -a 256 ggo-linux-amd64
```

### Step 4: Push to GitHub

```bash
cd homebrew-tap
git init
git add Formula/ggo.rb
git commit -m "Add ggo formula v0.2.0"
git remote add origin git@github.com:XavierFabregat/homebrew-tap.git
git push -u origin main
```

### Step 5: Users Install

```bash
brew tap XavierFabregat/tap
brew install ggo

# Or in one command:
brew install XavierFabregat/tap/ggo
```

### Automation: Auto-Update Formula on Release

**.github/workflows/update-homebrew.yaml:**
```yaml
name: Update Homebrew Formula

on:
  release:
    types: [published]

jobs:
  update-formula:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout homebrew-tap
        uses: actions/checkout@v4
        with:
          repository: XavierFabregat/homebrew-tap
          token: ${{ secrets.HOMEBREW_TAP_TOKEN }}

      - name: Download release assets
        run: |
          VERSION=${GITHUB_REF#refs/tags/v}
          curl -L -o ggo-macos-arm64 \
            "https://github.com/XavierFabregat/ggo/releases/download/v${VERSION}/ggo-macos-arm64"
          curl -L -o ggo-macos-amd64 \
            "https://github.com/XavierFabregat/ggo/releases/download/v${VERSION}/ggo-macos-amd64"
          curl -L -o ggo-linux-amd64 \
            "https://github.com/XavierFabregat/ggo/releases/download/v${VERSION}/ggo-linux-amd64"

      - name: Calculate checksums
        id: checksums
        run: |
          echo "arm64=$(shasum -a 256 ggo-macos-arm64 | awk '{print $1}')" >> $GITHUB_OUTPUT
          echo "amd64=$(shasum -a 256 ggo-macos-amd64 | awk '{print $1}')" >> $GITHUB_OUTPUT
          echo "linux=$(shasum -a 256 ggo-linux-amd64 | awk '{print $1}')" >> $GITHUB_OUTPUT

      - name: Update formula
        run: |
          VERSION=${GITHUB_REF#refs/tags/v}
          sed -i "s/version \".*\"/version \"${VERSION}\"/" Formula/ggo.rb
          sed -i "s|download/v[0-9.]\+/|download/v${VERSION}/|g" Formula/ggo.rb
          # Update SHA256 hashes (you'll need a more sophisticated approach here)

      - name: Commit and push
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add Formula/ggo.rb
          git commit -m "Update ggo to ${GITHUB_REF#refs/tags/v}"
          git push
```

---

## 🪟 **2. Winget (Windows Package Manager)** - HIGH PRIORITY

**Impact:** Built into Windows 11, 100M+ potential users

### Step 1: Create Manifest

Winget uses manifest files in the **winget-pkgs** repository.

**manifests/x/XavierFabregat/ggo/0.2.0/XavierFabregat.ggo.yaml:**
```yaml
PackageIdentifier: XavierFabregat.ggo
PackageVersion: 0.2.0
PackageLocale: en-US
Publisher: Xavier Fabregat
PublisherUrl: https://github.com/XavierFabregat
PublisherSupportUrl: https://github.com/XavierFabregat/ggo/issues
Author: Xavier Fabregat
PackageName: ggo
PackageUrl: https://github.com/XavierFabregat/ggo
License: MIT
LicenseUrl: https://github.com/XavierFabregat/ggo/blob/master/LICENSE
ShortDescription: Smart git branch navigation with frecency-based ranking
Description: |
  ggo makes git branch navigation as intuitive as zoxide makes directory
  navigation. Uses fuzzy matching and frecency algorithm to intelligently
  rank branches based on your usage patterns.
Tags:
  - git
  - cli
  - productivity
  - developer-tools
ReleaseNotesUrl: https://github.com/XavierFabregat/ggo/releases/tag/v0.2.0
ManifestType: defaultLocale
ManifestVersion: 1.6.0
```

**manifests/x/XavierFabregat/ggo/0.2.0/XavierFabregat.ggo.installer.yaml:**
```yaml
PackageIdentifier: XavierFabregat.ggo
PackageVersion: 0.2.0
Platform:
  - Windows.Desktop
MinimumOSVersion: 10.0.0.0
Scope: user
InstallModes:
  - silent
UpgradeBehavior: install
Installers:
  - Architecture: x64
    InstallerType: portable
    InstallerUrl: https://github.com/XavierFabregat/ggo/releases/download/v0.2.0/ggo-windows-amd64.exe
    InstallerSha256: FILL_IN_SHA256_HERE
    Commands:
      - ggo
ManifestType: installer
ManifestVersion: 1.6.0
```

### Step 2: Submit to winget-pkgs

```bash
# Fork https://github.com/microsoft/winget-pkgs
git clone https://github.com/YOUR_USERNAME/winget-pkgs.git
cd winget-pkgs

# Create branch
git checkout -b add-ggo-0.2.0

# Add your manifests
mkdir -p manifests/x/XavierFabregat/ggo/0.2.0
cp XavierFabregat.ggo.yaml manifests/x/XavierFabregat/ggo/0.2.0/
cp XavierFabregat.ggo.installer.yaml manifests/x/XavierFabregat/ggo/0.2.0/

# Commit and create PR
git add .
git commit -m "Add XavierFabregat.ggo version 0.2.0"
git push origin add-ggo-0.2.0

# Open PR to microsoft/winget-pkgs
```

### Step 3: Automation with winget-releaser

Use the **winget-releaser** GitHub Action:

**.github/workflows/winget-release.yaml:**
```yaml
name: Publish to WinGet

on:
  release:
    types: [published]

jobs:
  publish:
    runs-on: windows-latest
    steps:
      - name: Publish to WinGet
        uses: vedantmgoyal2009/winget-releaser@v2
        with:
          identifier: XavierFabregat.ggo
          installers-regex: 'ggo-windows-amd64\.exe$'
          token: ${{ secrets.WINGET_TOKEN }}
```

**Users install:**
```powershell
winget install XavierFabregat.ggo
```

---

## 📦 **3. Cargo (Rust Users)** - MEDIUM PRIORITY

**Impact:** Rust developers (smaller audience but easy to set up)

### Publish to crates.io

```bash
# One-time setup
cargo login

# Before each release
cargo publish --dry-run
cargo publish
```

**Update Cargo.toml with metadata:**
```toml
[package]
name = "ggo"
version = "0.2.0"
edition = "2021"
authors = ["Xavier Fabregat <your.email@example.com>"]
description = "Smart git branch navigation with frecency-based ranking"
repository = "https://github.com/XavierFabregat/ggo"
homepage = "https://github.com/XavierFabregat/ggo"
documentation = "https://github.com/XavierFabregat/ggo"
readme = "README.md"
keywords = ["git", "cli", "productivity", "frecency"]
categories = ["command-line-utilities", "development-tools"]
license = "MIT"
```

**Users install:**
```bash
cargo install ggo
```

---

## 🐧 **4. Linux Package Managers**

### Option A: cargo-deb (Debian/Ubuntu .deb)

**Install:**
```bash
cargo install cargo-deb
```

**Add to Cargo.toml:**
```toml
[package.metadata.deb]
maintainer = "Xavier Fabregat <your.email@example.com>"
copyright = "2025, Xavier Fabregat <your.email@example.com>"
license-file = ["LICENSE", "4"]
extended-description = """\
ggo makes git branch navigation as intuitive as zoxide makes directory \
navigation. Uses fuzzy matching and frecency algorithm to intelligently \
rank branches based on your usage patterns."""
depends = "$auto"
section = "utility"
priority = "optional"
assets = [
    ["target/release/ggo", "usr/bin/", "755"],
    ["README.md", "usr/share/doc/ggo/", "644"],
]
```

**Build .deb package:**
```bash
cargo deb
# Creates target/debian/ggo_0.2.0_amd64.deb
```

**Add to GitHub releases:**
```yaml
# In release.yaml
- name: Build .deb package
  run: |
    cargo install cargo-deb
    cargo deb

- name: Upload .deb to release
  uses: softprops/action-gh-release@v2
  with:
    files: target/debian/*.deb
```

### Option B: cargo-rpm (Fedora/RHEL .rpm)

```bash
cargo install cargo-rpm
cargo rpm init
cargo rpm build

# Add to release workflow similar to .deb
```

### Option C: AUR (Arch Linux)

Create **PKGBUILD** in a separate `ggo-bin` AUR repo:

```bash
# Maintainer: Xavier Fabregat <your.email@example.com>
pkgname=ggo-bin
pkgver=0.2.0
pkgrel=1
pkgdesc="Smart git branch navigation with frecency-based ranking"
arch=('x86_64')
url="https://github.com/XavierFabregat/ggo"
license=('MIT')
depends=('gcc-libs')
provides=('ggo')
conflicts=('ggo')
source_x86_64=("$url/releases/download/v$pkgver/ggo-linux-amd64")
sha256sums_x86_64=('FILL_IN_SHA256')

package() {
    install -Dm755 "$srcdir/ggo-linux-amd64" "$pkgdir/usr/bin/ggo"
}
```

**Users install:**
```bash
yay -S ggo-bin
# or
paru -S ggo-bin
```

---

## 🪣 **5. Scoop (Windows Alternative)**

**Impact:** Popular alternative to Winget for developers

### Create Scoop Bucket

**ggo.json:**
```json
{
    "version": "0.2.0",
    "description": "Smart git branch navigation with frecency-based ranking",
    "homepage": "https://github.com/XavierFabregat/ggo",
    "license": "MIT",
    "architecture": {
        "64bit": {
            "url": "https://github.com/XavierFabregat/ggo/releases/download/v0.2.0/ggo-windows-amd64.exe",
            "hash": "FILL_IN_SHA256",
            "bin": [
                ["ggo-windows-amd64.exe", "ggo"]
            ]
        }
    },
    "checkver": {
        "github": "https://github.com/XavierFabregat/ggo"
    },
    "autoupdate": {
        "architecture": {
            "64bit": {
                "url": "https://github.com/XavierFabregat/ggo/releases/download/v$version/ggo-windows-amd64.exe"
            }
        }
    }
}
```

**Users install:**
```powershell
scoop bucket add xavierfabregat https://github.com/XavierFabregat/scoop-bucket
scoop install ggo
```

---

## 📊 **Priority & ROI Matrix**

| Package Manager | Users | Setup Effort | Maintenance | ROI | Priority |
|----------------|-------|--------------|-------------|-----|----------|
| **Homebrew** | 10M+ | 2 hours | Low | ⭐⭐⭐⭐⭐ | **HIGH** |
| **Winget** | 100M+ | 3 hours | Low | ⭐⭐⭐⭐⭐ | **HIGH** |
| **Cargo** | 500K+ | 30 min | Very Low | ⭐⭐⭐⭐ | MEDIUM |
| **AUR** | 1M+ | 1 hour | Low | ⭐⭐⭐ | MEDIUM |
| **Scoop** | 500K+ | 1 hour | Low | ⭐⭐⭐ | LOW |
| **.deb/.rpm** | 10M+ | 2 hours | Medium | ⭐⭐⭐ | LOW |

---

## 🚀 **Recommended Implementation Order**

### Week 1: Core Package Managers (5 hours)
1. **Homebrew** (2 hours)
   - Create tap repo
   - Add formula
   - Test installation
   - Update README

2. **Winget** (3 hours)
   - Create manifests
   - Submit first PR to winget-pkgs
   - Set up winget-releaser action

### Week 2: Automation (2 hours)
3. **Automate updates** (2 hours)
   - Homebrew formula auto-update
   - Winget releaser GitHub Action
   - Add checksums to release workflow

### Week 3: Additional Channels (3 hours)
4. **Cargo** (30 min)
   - Add metadata to Cargo.toml
   - Publish to crates.io

5. **AUR** (1 hour)
   - Create PKGBUILD
   - Submit to AUR

6. **Scoop** (1 hour)
   - Create bucket
   - Add manifest

### Week 4: Documentation (1 hour)
7. **Update README** with all installation methods
8. **Create INSTALLATION.md** with troubleshooting

---

## 📝 **Update README.md**

```markdown
## Installation

### macOS & Linux (Homebrew)
```bash
brew install XavierFabregat/tap/ggo
```

### Windows (Winget)
```powershell
winget install XavierFabregat.ggo
```

### Windows (Scoop)
```powershell
scoop bucket add xavierfabregat https://github.com/XavierFabregat/scoop-bucket
scoop install ggo
```

### Rust Users (Cargo)
```bash
cargo install ggo
```

### Arch Linux (AUR)
```bash
yay -S ggo-bin
```

### From Source
```bash
git clone https://github.com/XavierFabregat/ggo.git
cd ggo
cargo install --path .
```
```

---

## 🎯 **Success Metrics**

Track these after distribution setup:

- **Downloads per package manager** (GitHub releases analytics)
- **Stars/watchers** (increases with discoverability)
- **Issues about installation** (should decrease dramatically)
- **Homebrew analytics** (if you opt in)

---

## 💡 **Final Thoughts**

**Distribution > Performance optimization.**

**Before package managers:**
- 100 GitHub stars
- 50 actual users (Rust developers only)
- High barrier to trying

**After package managers:**
- 1000+ stars (discoverable via `brew search git`)
- 500+ users (low friction to try)
- Professional appearance

**Total effort:** ~10 hours spread over a month
**Impact:** 10x increase in potential users

This is **high-leverage work** that directly impacts adoption.

---

## 📚 **Additional Resources**

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Winget Package Manifest Guide](https://learn.microsoft.com/en-us/windows/package-manager/package/)
- [crates.io Publishing Guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [AUR Submission Guidelines](https://wiki.archlinux.org/title/AUR_submission_guidelines)
- [Scoop App Manifest Guide](https://github.com/ScoopInstaller/Scoop/wiki/App-Manifests)

---

**Created:** 2025-12-16
**Last Updated:** 2025-12-16
**Status:** Ready for implementation
