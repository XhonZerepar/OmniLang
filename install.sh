#!/bin/bash
#
# OmniLang Installation Script
# 
# This script installs OmniLang on Unix-like systems (Linux, macOS, BSD)
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/XhonZerepar/OmniLang/main/install.sh | bash
#   curl -sSL https://raw.githubusercontent.com/XhonZerepar/OmniLang/main/install.sh | bash -s -- --version 0.2.0
#

set -e

# Configuration
REPO="XhonZerepar/OmniLang"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"
FORCE="${FORCE:-false}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        FreeBSD*)   echo "freebsd";;
        *)          echo "unknown";;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64)     echo "x86_64";;
        aarch64)    echo "arm64";;
        armv7l)     echo "armv7";;
        *)          echo "x86_64";;
    esac
}

# Print colored message
print_msg() {
    local color=$1
    shift
    echo -e "${color}$@${NC}"
}

# Download file
download_file() {
    local url=$1
    local output=$2
    
    if command -v curl &> /dev/null; then
        curl -sSL "$url" -o "$output"
    elif command -v wget &> /dev/null; then
        wget -q "$url" -O "$output"
    else
        print_msg "$RED" "Error: curl or wget is required"
        exit 1
    fi
}

# Get latest version
get_latest_version() {
    local api_url="https://api.github.com/repos/$REPO/releases/latest"
    
    if command -v curl &> /dev/null; then
        VERSION=$(curl -sSL "$api_url" | grep -o '"tag_name": "[^"]*' | cut -d'"' -f4 | sed 's/v//')
    elif command -v wget &> /dev/null; then
        VERSION=$(wget -q -O- "$api_url" | grep -o '"tag_name": "[^"]*' | cut -d'"' -f4 | sed 's/v//')
    else
        print_msg "$RED" "Error: curl or wget is required"
        exit 1
    fi
    
    echo "$VERSION"
}

# Check if already installed
check_installed() {
    if [ -f "$INSTALL_DIR/omc" ]; then
        if [ "$FORCE" = "false" ]; then
            print_msg "$YELLOW" "OmniLang is already installed at $INSTALL_DIR/omc"
            print_msg "$YELLOW" "Use --force to reinstall"
            exit 0
        fi
    fi
}

# Create installation directory
create_install_dir() {
    if [ ! -d "$INSTALL_DIR" ]; then
        mkdir -p "$INSTALL_DIR"
        print_msg "$GREEN" "Created installation directory: $INSTALL_DIR"
    fi
}

# Download and install OmniLang
install_omnilang() {
    local os=$(detect_os)
    local arch=$(detect_arch)
    local version=$VERSION
    
    if [ "$version" = "latest" ]; then
        version=$(get_latest_version)
    fi
    
    print_msg "$BLUE" "Installing OmniLang v$version for $os-$arch..."
    
    # Determine file extension
    local ext="tar.gz"
    if [ "$os" = "windows" ]; then
        ext="zip"
    fi
    
    # Build download URL
    local filename="omnilang-${version}-${os}-${arch}.${ext}"
    local url="https://github.com/$REPO/releases/download/v${version}/$filename"
    
    print_msg "$BLUE" "Downloading from $url..."
    
    # Download to temp directory
    local temp_dir=$(mktemp -d)
    local archive="${temp_dir}/omnilang.${ext}"
    
    download_file "$url" "$archive"
    
    # Extract
    print_msg "$BLUE" "Extracting..."
    cd "$temp_dir"
    
    if [ "$ext" = "tar.gz" ]; then
        tar -xzf "$archive"
    else
        unzip -q "$archive"
    fi
    
    # Copy binary
    print_msg "$BLUE" "Installing..."
    local binary="omnilang"
    if [ "$os" = "windows" ]; then
        binary="omnilang.exe"
    fi
    
    cp "$binary" "$INSTALL_DIR/omc"
    chmod +x "$INSTALL_DIR/omc"
    
    # Also install other tools if they exist
    if [ -f "omnilang" ]; then
        for tool in omnilang omf omp omdb omdoc; do
            if [ -f "$tool" ]; then
                cp "$tool" "$INSTALL_DIR/$tool"
                chmod +x "$INSTALL_DIR/$tool"
            fi
        done
    fi
    
    # Cleanup
    rm -rf "$temp_dir"
    
    print_msg "$GREEN" "OmniLang v$version installed successfully!"
    print_msg "$GREEN" "Binary installed to: $INSTALL_DIR/omc"
}

# Add to PATH if not already there
update_path() {
    local shell_rc=""
    
    case "$(detect_os)" in
        macos)  shell_rc="$HOME/.zshrc";;
        linux)  
            if [ -n "$ZSH_VERSION" ]; then
                shell_rc="$HOME/.zshrc"
            else
                shell_rc="$HOME/.bashrc"
            fi
            ;;
    esac
    
    if [ -n "$shell_rc" ]; then
        if ! grep -q "$INSTALL_DIR" "$shell_rc" 2>/dev/null; then
            echo "" >> "$shell_rc"
            echo "# OmniLang" >> "$shell_rc"
            echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_rc"
            print_msg "$GREEN" "Added $INSTALL_DIR to PATH in $shell_rc"
            print_msg "$YELLOW" "Please restart your shell or run: source $shell_rc"
        fi
    fi
}

# Verify installation
verify_install() {
    print_msg "$BLUE" "Verifying installation..."
    
    if "$INSTALL_DIR/omc" --version &> /dev/null; then
        print_msg "$GREEN" "Installation verified!"
        "$INSTALL_DIR/omc" --version
    else
        print_msg "$RED" "Installation verification failed"
        exit 1
    fi
}

# Print usage
usage() {
    echo "OmniLang Installation Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --version VERSION   Install specific version (default: latest)"
    echo "  --install-dir DIR   Installation directory (default: ~/.local/bin)"
    echo "  --force            Force reinstall"
    echo "  --help             Show this help message"
    echo ""
}

# Parse arguments
parse_args() {
    while [ $# -gt 0 ]; do
        case $1 in
            --version)
                VERSION="$2"
                shift 2
                ;;
            --install-dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --force)
                FORCE="true"
                shift
                ;;
            --help)
                usage
                exit 0
                ;;
            *)
                print_msg "$RED" "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
}

# Main function
main() {
    print_msg "$BLUE" "╔════════════════════════════════════════╗"
    print_msg "$BLUE" "║     OmniLang Installer v0.2.0          ║"
    print_msg "$BLUE" "╚════════════════════════════════════════╝"
    echo ""
    
    parse_args "$@"
    check_installed
    create_install_dir
    install_omnilang
    update_path
    verify_install
    
    echo ""
    print_msg "$GREEN" "All done! 🎉"
    print_msg "$GREEN" "Run 'omc --help' to get started"
}

main "$@"
