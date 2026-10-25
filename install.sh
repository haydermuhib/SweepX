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
CYAN='\033[1;36m'
PURPLE='\033[1;35m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}====================================================${NC}"
echo -e "${CYAN}        ⚡ SweepX Linux Installer                   ${NC}"
echo -e "${CYAN}====================================================${NC}"

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
    ICONS_DIR="/usr/local/share/icons/hicolor/scalable/apps"
    PIXMAPS_DIR="/usr/local/share/pixmaps"
else
    INSTALL_DIR="$HOME/.local/bin"
    APPS_DIR="$HOME/.local/share/applications"
    ICONS_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
    PIXMAPS_DIR="$HOME/.local/share/pixmaps"
fi

mkdir -p "$INSTALL_DIR"
mkdir -p "$APPS_DIR"
mkdir -p "$ICONS_DIR"
mkdir -p "$PIXMAPS_DIR"

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

# 6. Fallback: If precompiled release isn't ready, build from source
if [ "$INSTALLED_SUCCESS" -eq 0 ]; then
    echo -e "${YELLOW}Prebuilt release binary not found on GitHub. Checking for Rust compiler...${NC}"
    if ! command -v cargo >/dev/null 2>&1; then
        echo -e "${CYAN}Rust is required to build from source. Installing lightweight Rust toolchain via rustup...${NC}"
        if [ "$DOWNLOADER" = "curl" ]; then
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
        else
            wget -qO- https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
        fi
        if [ -f "$HOME/.cargo/env" ]; then
            source "$HOME/.cargo/env"
        fi
    fi

    if command -v cargo >/dev/null 2>&1; then
        echo -e "${GREEN}Building and installing SweepX from source via cargo...${NC}"
        cargo install --git "https://github.com/${REPO_OWNER}/${REPO_NAME}.git" --root "$HOME/.local"
        INSTALLED_SUCCESS=1
    else
        echo -e "${RED}Error: Could not install or locate Rust toolchain.${NC}"
        echo -e "Please install Rust from https://rustup.rs or check the releases page:"
        echo -e "https://github.com/${REPO_OWNER}/${REPO_NAME}/releases"
        exit 1
    fi
fi

# 7. Install Application Icon
cat <<'EOF' > "$ICONS_DIR/sweepx.svg"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512" role="img" aria-labelledby="title-app-icon">
  <title id="title-app-icon">SweepX Application Icon</title>
  <defs>
    <radialGradient id="tile-grad" cx="50%" cy="30%" r="75%">
      <stop offset="0%" stop-color="#4a1827"/>
      <stop offset="100%" stop-color="#2a0d15"/>
    </radialGradient>
  </defs>

  <!-- Squircle Base Tile (512x512 with 114px radius = Apple/GNOME modern HIG squircle) -->
  <rect x="24" y="24" width="464" height="464" rx="114" fill="url(#tile-grad)" stroke="#ede5d2" stroke-opacity="0.15" stroke-width="3"/>

  <!-- Symbol Graphic: Centered in 512x512 space (scaled 1.4x from 256x256) -->
  <g transform="translate(256, 256) scale(1.4) translate(-128, -128)">
    <!-- Core Lens -->
    <circle cx="128" cy="128" r="20" fill="#ede5d2"/>

    <!-- Outer Orbital Tier -->
    <path fill="none" stroke="#ede5d2" stroke-width="18" stroke-linecap="round" d="
      M 86 58 A 96 96 0 0 1 170 58
    "/>
    <path fill="none" stroke="#ede5d2" stroke-width="18" stroke-linecap="round" d="
      M 86 198 A 96 96 0 0 0 170 198
    "/>
    <path fill="none" stroke="#ede5d2" stroke-width="18" stroke-linecap="round" d="
      M 58 86 A 96 96 0 0 0 58 170
    "/>
    <path fill="none" stroke="#ede5d2" stroke-width="18" stroke-linecap="round" d="
      M 198 86 A 96 96 0 0 1 198 170
    "/>

    <!-- Inner Orbital Tier -->
    <path fill="none" stroke="#ede5d2" stroke-width="14" stroke-linecap="round" d="
      M 98 84 A 56 56 0 0 1 158 84
    "/>
    <path fill="none" stroke="#ede5d2" stroke-width="14" stroke-linecap="round" d="
      M 98 172 A 56 56 0 0 0 158 172
    "/>
    <path fill="none" stroke="#ede5d2" stroke-width="14" stroke-linecap="round" d="
      M 84 98 A 56 56 0 0 0 84 158
    "/>
    <path fill="none" stroke="#ede5d2" stroke-width="14" stroke-linecap="round" d="
      M 172 98 A 56 56 0 0 1 172 158
    "/>

    <!-- Kinetic Purge Sweep Blade in Pistachio Accent -->
    <rect x="118" y="24" width="20" height="208" rx="10" transform="rotate(-45 128 128)" fill="#9ecb82"/>
  </g>
</svg>
EOF

# Copy icon to pixmaps and top-level icon directories for universal DE compatibility
cp "$ICONS_DIR/sweepx.svg" "$PIXMAPS_DIR/sweepx.svg" 2>/dev/null || true
if [ "$EUID" -ne 0 ]; then
    mkdir -p "$HOME/.local/share/icons"
    cp "$ICONS_DIR/sweepx.svg" "$HOME/.local/share/icons/sweepx.svg" 2>/dev/null || true
fi

# 8. Register Desktop Launcher (Application Menu only)
cat <<EOF > "$APPS_DIR/sweepx.desktop"
[Desktop Entry]
Type=Application
Name=SweepX
Comment=Unified Linux Application Tracker & Deep Uninstaller
Exec=$INSTALL_DIR/$BINARY_NAME gui
Icon=$ICONS_DIR/sweepx.svg
Categories=System;Utility;Settings;
Terminal=false
StartupNotify=true
EOF
chmod +x "$APPS_DIR/sweepx.desktop"

# Clean up any leftover shortcut on the desktop directory if present
if [ -f "$HOME/Desktop/sweepx.desktop" ]; then
    rm -f "$HOME/Desktop/sweepx.desktop"
fi

# Update desktop and icon databases
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APPS_DIR" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

# 9. Check PATH configuration
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}Notice: $INSTALL_DIR is not in your PATH.${NC}"
    
    if [ -f "$HOME/.bashrc" ]; then
        if ! grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.bashrc"; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
            echo -e "Added ~/.local/bin to ~/.bashrc"
        fi
    fi
    if [ -f "$HOME/.zshrc" ]; then
        if ! grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.zshrc"; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
            echo -e "Added ~/.local/bin to ~/.zshrc"
        fi
    fi
    if [ -f "$HOME/.profile" ]; then
        if ! grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.profile"; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.profile"
            echo -e "Added ~/.local/bin to ~/.profile"
        fi
    fi
fi

echo -e "\n${GREEN}====================================================${NC}"
echo -e "${GREEN}  🎉 SweepX was successfully installed!             ${NC}"
echo -e "${GREEN}====================================================${NC}"
echo -e "Binary installed at: ${CYAN}$INSTALL_DIR/$BINARY_NAME${NC}"
echo -e "Icon installed at:   ${CYAN}$ICONS_DIR/sweepx.svg${NC}"
echo -e "App launcher at:     ${CYAN}$APPS_DIR/sweepx.desktop${NC}"

echo -e "\n🚀 Run from terminal: ${GREEN}sweepx${NC}"
echo -e "📦 CLI app listing:   ${GREEN}sweepx list${NC}"
echo -e "🔄 Check updates:     ${GREEN}sweepx update${NC}\n"
