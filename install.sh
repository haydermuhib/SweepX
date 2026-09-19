#!/usr/bin/env bash
# SweepX - Unified Linux Application Tracker & Deep Purge
# Universal 1-line installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/haydermuhib/SweepX/main/install.sh | bash

set -e

REPO_OWNER="haydermuhib"
REPO_NAME="SweepX"
BINARY_NAME="sweepx"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}====================================================${NC}"
echo -e "${BLUE}        ⚡ SweepX Linux Installer                   ${NC}"
echo -e "${BLUE}====================================================${NC}"

# 1. Detect Operating System
OS="$(uname -s)"
if [ "$OS" != "Linux" ]; then
    echo -e "${RED}Error: SweepX is only supported on Linux.${NC}"
    exit 1
fi

# 2. Detect CPU Architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    aarch64|arm64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

echo -e "Detected platform: ${GREEN}Linux ($TARGET_ARCH)${NC}"

# 3. Determine Installation Directory
if [ "$EUID" -eq 0 ]; then
    INSTALL_DIR="/usr/local/bin"
    APPS_DIR="/usr/local/share/applications"
else
    INSTALL_DIR="$HOME/.local/bin"
    APPS_DIR="$HOME/.local/share/applications"
fi

mkdir -p "$INSTALL_DIR"
mkdir -p "$APPS_DIR"

# 4. Check Download Tools
DOWNLOADER=""
if command -v curl >/dev/null 2>&1; then
    DOWNLOADER="curl"
elif command -v wget >/dev/null 2>&1; then
    DOWNLOADER="wget"
else
    echo -e "${RED}Error: neither curl nor wget was found. Please install curl or wget.${NC}"
    exit 1
fi

# 5. Fetch and Download Release Binary
RELEASE_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest/download/sweepx-${TARGET_ARCH}-unknown-linux-gnu.tar.gz"
FALLBACK_BIN_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest/download/sweepx"

TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

INSTALLED_SUCCESS=0

echo -e "Fetching latest release for ${TARGET_ARCH}..."

if [ "$DOWNLOADER" = "curl" ]; then
    if curl -fsSL -o "$TEMP_DIR/sweepx.tar.gz" "$RELEASE_URL" 2>/dev/null; then
        tar -xzf "$TEMP_DIR/sweepx.tar.gz" -C "$TEMP_DIR" 2>/dev/null || true
        if [ -f "$TEMP_DIR/sweepx" ]; then
            mv "$TEMP_DIR/sweepx" "$INSTALL_DIR/$BINARY_NAME"
            chmod +x "$INSTALL_DIR/$BINARY_NAME"
            INSTALLED_SUCCESS=1
        fi
    elif curl -fsSL -o "$TEMP_DIR/sweepx" "$FALLBACK_BIN_URL" 2>/dev/null; then
        mv "$TEMP_DIR/sweepx" "$INSTALL_DIR/$BINARY_NAME"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
        INSTALLED_SUCCESS=1
    fi
elif [ "$DOWNLOADER" = "wget" ]; then
    if wget -q -O "$TEMP_DIR/sweepx.tar.gz" "$RELEASE_URL" 2>/dev/null; then
        tar -xzf "$TEMP_DIR/sweepx.tar.gz" -C "$TEMP_DIR" 2>/dev/null || true
        if [ -f "$TEMP_DIR/sweepx" ]; then
            mv "$TEMP_DIR/sweepx" "$INSTALL_DIR/$BINARY_NAME"
            chmod +x "$INSTALL_DIR/$BINARY_NAME"
            INSTALLED_SUCCESS=1
        fi
    elif wget -q -O "$TEMP_DIR/sweepx" "$FALLBACK_BIN_URL" 2>/dev/null; then
        mv "$TEMP_DIR/sweepx" "$INSTALL_DIR/$BINARY_NAME"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
        INSTALLED_SUCCESS=1
    fi
fi

# 6. Fallback: If precompiled release isn't ready, build with cargo if available
if [ "$INSTALLED_SUCCESS" -eq 0 ]; then
    echo -e "${YELLOW}Prebuilt binary release not found. Checking for Rust / cargo toolchain...${NC}"
    if command -v cargo >/dev/null 2>&1; then
        echo -e "Compiling SweepX from source via cargo..."
        cargo install --git "https://github.com/${REPO_OWNER}/${REPO_NAME}.git" --root "$HOME/.local"
        INSTALLED_SUCCESS=1
    else
        echo -e "${RED}Failed to download precompiled binary and cargo was not found.${NC}"
        echo -e "Please install Rust (https://rustup.rs) or download the release manually from:"
        echo -e "https://github.com/${REPO_OWNER}/${REPO_NAME}/releases"
        exit 1
    fi
fi

# 7. Register Desktop Launcher
cat <<EOF > "$APPS_DIR/sweepx.desktop"
[Desktop Entry]
Type=Application
Name=SweepX
Comment=Unified Linux Application Tracker & Deep Uninstaller
Exec=$INSTALL_DIR/$BINARY_NAME gui
Icon=system-software-install
Categories=System;Utility;Settings;
Terminal=false
StartupNotify=true
EOF
chmod +x "$APPS_DIR/sweepx.desktop"

# 8. Check PATH configuration
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}Notice: $INSTALL_DIR is not in your PATH.${NC}"
    
    # Try adding to ~/.bashrc or ~/.zshrc
    if [ -f "$HOME/.bashrc" ]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
        echo -e "Added ~/.local/bin to ~/.bashrc"
    fi
    if [ -f "$HOME/.zshrc" ]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
        echo -e "Added ~/.local/bin to ~/.zshrc"
    fi
fi

echo -e "\n${GREEN}====================================================${NC}"
echo -e "${GREEN}  🎉 SweepX was successfully installed!             ${NC}"
echo -e "${GREEN}====================================================${NC}"
echo -e "Binary installed at: ${BLUE}$INSTALL_DIR/$BINARY_NAME${NC}"
echo -e "Desktop launcher at: ${BLUE}$APPS_DIR/sweepx.desktop${NC}"
echo -e "\nTo run SweepX GUI:   ${GREEN}sweepx${NC}"
echo -e "To run CLI scan:     ${GREEN}sweepx list${NC}"
echo -e "To check updates:    ${GREEN}sweepx update${NC}\n"
