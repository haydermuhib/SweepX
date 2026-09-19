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

# 6. Fallback: If precompiled release isn't ready, build from source
if [ "$INSTALLED_SUCCESS" -eq 0 ]; then
    echo -e "${YELLOW}Prebuilt release binary not found on GitHub. Checking for Rust compiler...${NC}"
    if ! command -v cargo >/dev/null 2>&1; then
        echo -e "${BLUE}Rust is required to build from source. Installing lightweight Rust toolchain via rustup...${NC}"
        if [ "$DOWNLOADER" = "curl" ]; then
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
        else
            wget -qO- https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
        fi
        if [ -f "$HOME/.cargo/env" ]; then
            # Source cargo environment for this script
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
if [ "$EUID" -eq 0 ]; then
    ICONS_DIR="/usr/local/share/icons/hicolor/scalable/apps"
else
    ICONS_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
fi
mkdir -p "$ICONS_DIR"

cat <<'EOF' > "$ICONS_DIR/sweepx.svg"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="100%" height="100%">
  <defs>
    <linearGradient id="bg-grad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#1e1b4b"/>
      <stop offset="50%" stop-color="#0f172a"/>
      <stop offset="100%" stop-color="#020617"/>
    </linearGradient>
    <linearGradient id="bolt-grad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#38bdf8"/>
      <stop offset="50%" stop-color="#6366f1"/>
      <stop offset="100%" stop-color="#a855f7"/>
    </linearGradient>
    <linearGradient id="glow-grad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#00f2fe"/>
      <stop offset="100%" stop-color="#4facfe"/>
    </linearGradient>
    <filter id="glow" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur stdDeviation="12" result="blur" />
      <feComposite in="SourceGraphic" in2="blur" operator="over" />
    </filter>
  </defs>
  <rect x="32" y="32" width="448" height="448" rx="100" fill="url(#bg-grad)" stroke="#38bdf8" stroke-opacity="0.3" stroke-width="6"/>
  <circle cx="256" cy="256" r="160" fill="none" stroke="#6366f1" stroke-opacity="0.15" stroke-width="4" stroke-dasharray="12 8"/>
  <path d="M280 80 L160 270 L250 270 L230 432 L350 242 L260 242 Z" fill="url(#glow-grad)" opacity="0.4" filter="url(#glow)"/>
  <path d="M280 80 L160 270 L250 270 L230 432 L350 242 L260 242 Z" fill="url(#bolt-grad)" stroke="#ffffff" stroke-width="3" stroke-linejoin="round"/>
  <circle cx="360" cy="140" r="8" fill="#38bdf8" opacity="0.8"/>
  <circle cx="390" cy="180" r="5" fill="#818cf8" opacity="0.6"/>
  <circle cx="130" cy="360" r="7" fill="#a855f7" opacity="0.7"/>
  <circle cx="160" cy="400" r="4" fill="#38bdf8" opacity="0.5"/>
</svg>
EOF

# 8. Register Desktop Launchers (Application Menu & Desktop)
cat <<EOF > "$APPS_DIR/sweepx.desktop"
[Desktop Entry]
Type=Application
Name=SweepX
Comment=Unified Linux Application Tracker & Deep Uninstaller
Exec=$INSTALL_DIR/$BINARY_NAME gui
Icon=sweepx
Categories=System;Utility;Settings;
Terminal=false
StartupNotify=true
EOF
chmod +x "$APPS_DIR/sweepx.desktop"

# Optional Desktop shortcut if ~/Desktop exists
if [ -d "$HOME/Desktop" ]; then
    cp "$APPS_DIR/sweepx.desktop" "$HOME/Desktop/sweepx.desktop"
    chmod +x "$HOME/Desktop/sweepx.desktop"
    if command -v gio >/dev/null 2>&1; then
        gio set "$HOME/Desktop/sweepx.desktop" metadata::trusted true 2>/dev/null || true
    fi
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
echo -e "Binary installed at: ${BLUE}$INSTALL_DIR/$BINARY_NAME${NC}"
echo -e "Icon installed at:   ${BLUE}$ICONS_DIR/sweepx.svg${NC}"
echo -e "App launcher at:     ${BLUE}$APPS_DIR/sweepx.desktop${NC}"
if [ -d "$HOME/Desktop" ]; then
    echo -e "Desktop shortcut at: ${BLUE}$HOME/Desktop/sweepx.desktop${NC}"
fi
echo -e "\n🚀 Run from terminal: ${GREEN}sweepx${NC}"
echo -e "📦 CLI app listing:   ${GREEN}sweepx list${NC}"
echo -e "🔄 Check updates:     ${GREEN}sweepx update${NC}\n"
