#!/usr/bin/env bash
set -e

# ggo installer script
# Usage: curl -sSf https://raw.githubusercontent.com/XavierFabregat/ggo/master/install.sh | bash

INSTALL_DIR="${GGO_INSTALL_DIR:-$HOME/.local/bin}"
REPO_URL="https://github.com/XavierFabregat/ggo"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${BLUE}INFO:${NC} $1"
}

print_success() {
    echo -e "${GREEN}SUCCESS:${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}WARNING:${NC} $1"
}

print_error() {
    echo -e "${RED}ERROR:${NC} $1"
}

check_command() {
    command -v "$1" >/dev/null 2>&1
}

detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        MINGW*|MSYS*|CYGWIN*)    echo "windows";;
        *)          echo "unknown";;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   echo "x86_64";;
        aarch64|arm64)  echo "aarch64";;
        *)              echo "unknown";;
    esac
}

install_rust() {
    print_info "Rust is not installed. Installing Rust via rustup..."
    if ! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
        print_error "Failed to install Rust"
        exit 1
    fi

    # Source cargo env
    if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck source=/dev/null
        . "$HOME/.cargo/env"
    fi

    print_success "Rust installed successfully"
}

install_from_crates_io() {
    print_info "Installing ggo from crates.io..."

    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"

    # Install to a temp location first (cargo always uses <root>/bin/)
    TEMP_ROOT=$(mktemp -d)

    if cargo install ggo --root "$TEMP_ROOT"; then
        # Move the binary to the desired location
        mv "$TEMP_ROOT/bin/ggo" "$INSTALL_DIR/ggo"
        chmod +x "$INSTALL_DIR/ggo"
        rm -rf "$TEMP_ROOT"

        print_success "ggo installed successfully via cargo"
        return 0
    else
        rm -rf "$TEMP_ROOT"
        print_warning "Failed to install from crates.io, will try building from source"
        return 1
    fi
}

install_from_source() {
    print_info "Building ggo from source..."

    # Create temporary directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    print_info "Cloning repository..."
    if ! git clone --depth 1 "$REPO_URL" ggo; then
        print_error "Failed to clone repository"
        rm -rf "$TEMP_DIR"
        exit 1
    fi

    cd ggo

    print_info "Building ggo (this may take a few minutes)..."
    if ! cargo build --release; then
        print_error "Failed to build ggo"
        rm -rf "$TEMP_DIR"
        exit 1
    fi

    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"

    # Install binary
    print_info "Installing to $INSTALL_DIR..."
    if ! cp target/release/ggo "$INSTALL_DIR/ggo"; then
        print_error "Failed to install binary"
        rm -rf "$TEMP_DIR"
        exit 1
    fi

    # Make executable
    chmod +x "$INSTALL_DIR/ggo"

    # Cleanup
    cd -
    rm -rf "$TEMP_DIR"

    print_success "ggo installed successfully from source"
}

check_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        print_warning "$INSTALL_DIR is not in your PATH"
        echo ""
        echo "Add it to your PATH by adding this line to your shell config file:"
        echo ""

        case "$(basename "$SHELL")" in
            bash)
                echo "  echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.bashrc"
                echo "  source ~/.bashrc"
                ;;
            zsh)
                echo "  echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.zshrc"
                echo "  source ~/.zshrc"
                ;;
            fish)
                echo "  fish_add_path $INSTALL_DIR"
                ;;
            *)
                echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
                ;;
        esac
        echo ""
        return 1
    fi
    return 0
}

main() {
    echo "╔════════════════════════════════════════╗"
    echo "║   ggo - Smart Git Branch Navigation   ║"
    echo "╔════════════════════════════════════════╝"
    echo ""

    OS=$(detect_os)
    ARCH=$(detect_arch)

    print_info "Detected OS: $OS"
    print_info "Detected Architecture: $ARCH"
    echo ""

    # Check for unsupported systems
    if [ "$OS" = "unknown" ] || [ "$ARCH" = "unknown" ]; then
        print_error "Unsupported system: $OS/$ARCH"
        print_info "Please build from source manually:"
        echo "  git clone $REPO_URL"
        echo "  cd ggo"
        echo "  cargo install --path ."
        exit 1
    fi

    # Check for required tools
    if ! check_command git; then
        print_error "git is required but not installed"
        print_info "Please install git first: https://git-scm.com/downloads"
        exit 1
    fi

    # Check for cargo/rust
    if ! check_command cargo; then
        print_warning "Cargo not found"

        # Ask user if they want to install Rust
        if [ -t 0 ]; then
            read -p "Would you like to install Rust now? (y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                install_rust
            else
                print_error "Rust is required to build ggo"
                print_info "Install Rust manually: https://rustup.rs"
                exit 1
            fi
        else
            install_rust
        fi
    fi

    # Try installing from crates.io first (faster)
    if ! install_from_crates_io; then
        # Fallback to building from source
        install_from_source
    fi

    echo ""
    print_success "Installation complete!"
    echo ""

    # Check if install directory is in PATH
    IN_PATH=0
    check_path || IN_PATH=$?

    # Test installation
    if [ $IN_PATH -eq 0 ]; then
        if check_command ggo; then
            print_info "Verifying installation..."
            ggo --version
            echo ""
            print_success "ggo is ready to use!"
            echo ""
            echo "Get started:"
            echo "  ggo --help              # View all commands"
            echo "  ggo feat                # Smart checkout with fuzzy matching"
            echo "  ggo alias m master      # Create branch aliases"
            echo "  ggo -                   # Jump to previous branch"
        else
            print_warning "Installation completed but ggo is not immediately available"
            print_info "Try opening a new terminal or run: hash -r"
        fi
    else
        print_info "After adding to PATH, verify with: ggo --version"
    fi

    echo ""
    print_info "Documentation: $REPO_URL"
    print_info "Report issues: $REPO_URL/issues"
}

main "$@"
