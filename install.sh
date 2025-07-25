#!/usr/bin/env bash

set -e

# Git Time Traveler Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/yourusername/git-timetraveler/main/install.sh | bash

REPO="yourusername/git-timetraveler"
BINARY_NAME="git-timetraveler"
INSTALL_DIR="$HOME/.local/bin"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Detect platform and architecture
detect_platform() {
    local platform
    case "$(uname -s)" in
        Linux*)     platform="x86_64-unknown-linux-gnu";;
        Darwin*)    
            case "$(uname -m)" in
                arm64)  platform="aarch64-apple-darwin";;
                *)      platform="x86_64-apple-darwin";;
            esac
            ;;
        *)          
            log_error "Unsupported platform: $(uname -s)"
            exit 1
            ;;
    esac
    echo "$platform"
}

# Get latest release version
get_latest_version() {
    curl -s "https://api.github.com/repos/$REPO/releases/latest" | 
        grep '"tag_name":' | 
        sed -E 's/.*"([^"]+)".*/\1/'
}

# Download and install
install_binary() {
    local platform="$1"
    local version="$2"
    
    log_info "Detecting platform: $platform"
    log_info "Latest version: $version"
    
    # Determine file extension
    local extension=".tar.xz"
    if [[ "$platform" == *"windows"* ]]; then
        extension=".zip"
    fi
    
    local filename="${BINARY_NAME}-${platform}${extension}"
    local download_url="https://github.com/$REPO/releases/download/$version/$filename"
    
    log_info "Downloading $filename..."
    
    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    trap "rm -rf $temp_dir" EXIT
    
    # Download the archive
    if ! curl -fsSL "$download_url" -o "$temp_dir/$filename"; then
        log_error "Failed to download $filename"
        log_error "URL: $download_url"
        exit 1
    fi
    
    log_info "Extracting archive..."
    
    # Extract based on file type
    cd "$temp_dir"
    if [[ "$extension" == ".zip" ]]; then
        unzip -q "$filename"
    else
        tar -xf "$filename"
    fi
    
    # Find the binary
    local binary_path
    binary_path=$(find . -name "$BINARY_NAME" -type f | head -1)
    
    if [[ -z "$binary_path" ]]; then
        log_error "Binary not found in archive"
        exit 1
    fi
    
    log_info "Installing to $INSTALL_DIR..."
    
    # Create install directory
    mkdir -p "$INSTALL_DIR"
    
    # Copy binary and make executable
    cp "$binary_path" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    
    log_success "Git Time Traveler installed successfully!"
    
    # Check if install directory is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        log_warning "Add $INSTALL_DIR to your PATH to use git-timetraveler from anywhere:"
        echo "  echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.bashrc"
        echo "  source ~/.bashrc"
    fi
    
    log_info "Usage:"
    echo "  $INSTALL_DIR/$BINARY_NAME --help"
    echo "  $INSTALL_DIR/$BINARY_NAME --year 1990"
}

# Main installation process
main() {
    echo "🚀 Git Time Traveler Installation"
    echo "=================================="
    echo
    
    # Check dependencies
    for cmd in curl tar; do
        if ! command -v "$cmd" &> /dev/null; then
            log_error "$cmd is required but not installed"
            exit 1
        fi
    done
    
    local platform
    platform=$(detect_platform)
    
    local version
    version=$(get_latest_version)
    
    if [[ -z "$version" ]]; then
        log_error "Failed to get latest version"
        exit 1
    fi
    
    install_binary "$platform" "$version"
    
    echo
    log_success "Installation complete! 🎉"
    echo
    echo "Try it out:"
    echo "  $INSTALL_DIR/$BINARY_NAME --year 1990"
}

# Run main function
main "$@" 