#!/usr/bin/env bash
# SweepX - Unified Linux Application Tracker & Deep Purge
# Universal 1-line installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/haydermuhib/SweepX/main/install.sh | bash

set -e

REPO_OWNER="haydermuhib"
REPO_NAME="SweepX"
BINARY_NAME="sweepx"

# Terminal Capabilities & Colors
if [ -t 1 ]; then
    IS_TTY=1
    PISTACHIO='\033[38;2;158;203;130m'
    PISTACHIO_BOLD='\033[1;38;2;158;203;130m'
    CREAM_BOLD='\033[1;38;2;237;229;210m'
    CYAN='\033[1;36m'
    GREEN='\033[1;32m'
    YELLOW='\033[1;33m'
    RED='\033[1;31m'
    DIM='\033[2m'
    NC='\033[0m'
else
    IS_TTY=0
    PISTACHIO=''
    PISTACHIO_BOLD=''
    CREAM_BOLD=''
    CYAN=''
    GREEN=''
    YELLOW=''
    RED=''
    DIM=''
    NC=''
fi

TEMP_DIR="$(mktemp -d)"

cleanup() {
    if [ "$IS_TTY" -eq 1 ]; then
        printf "\033[?25h" # Restore cursor
    fi
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT INT TERM

# Helper: Query terminal columns safely
get_term_cols() {
    local cols
    cols=$(tput cols 2>/dev/null || echo 80)
    case "$cols" in
        ''|*[!0-9]*) cols=80 ;;
    esac
    [ "$cols" -lt 40 ] && cols=80
    echo "$cols"
}

# Helper: Format byte counts to human-readable string without external forks
format_size() {
    local b=$1
    if [ "$b" -ge 1073741824 ]; then
        local gb=$(( b / 1073741824 ))
        local dec=$(( (b % 1073741824) * 10 / 1073741824 ))
        echo "${gb}.${dec} GB"
    elif [ "$b" -ge 1048576 ]; then
        local mb=$(( b / 1048576 ))
        local dec=$(( (b % 1048576) * 10 / 1048576 ))
        echo "${mb}.${dec} MB"
    elif [ "$b" -ge 1024 ]; then
        local kb=$(( b / 1024 ))
        local dec=$(( (b % 1024) * 10 / 1024 ))
        echo "${kb}.${dec} KB"
    else
        echo "${b} B"
    fi
}

# Helper: Animated transient spinner for background execution
run_step() {
    local msg="$1"
    local cmd="$2"

    if [ "$IS_TTY" -eq 0 ]; then
        echo "  • $msg..."
        eval "$cmd"
        echo "  ✓ $msg"
        return 0
    fi

    local spin_chars="⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"
    local spin_len=${#spin_chars}
    local spin_idx=0

    printf "\033[?25l" # Hide cursor
    eval "$cmd" >/dev/null 2>&1 &
    local cmd_pid=$!

    while kill -0 "$cmd_pid" 2>/dev/null; do
        local char="${spin_chars:spin_idx:1}"
        spin_idx=$(( (spin_idx + 1) % spin_len ))
        printf "\r  ${CYAN}%s${NC} %s...  " "$char" "$msg"
        sleep 0.08
    done
    wait "$cmd_pid"
    local code=$?
    printf "\033[?25h" # Restore cursor

    if [ "$code" -eq 0 ]; then
        printf "\r  ${GREEN}✓${NC} %s\033[K\n" "$msg"
        return 0
    else
        printf "\r  ${RED}✗${NC} %s failed\033[K\n" "$msg"
        return "$code"
    fi
}

# Helper: Responsive download progress bar adapting dynamically to terminal width
download_with_progress() {
    local url="$1"
    local output="$2"
    local label="$3"

    if [ "$IS_TTY" -eq 0 ]; then
        echo "  • Downloading ${label}..."
        curl -fsSL -o "$output" "$url"
        echo "  ✓ Download complete."
        return $?
    fi

    # Query content length via HTTP headers
    local total_bytes=0
    local headers
    headers=$(curl -sIL "$url" 2>/dev/null || true)
    total_bytes=$(echo "$headers" | tr -d '\r' | awk '/[Cc]ontent-[Ll]ength/ {print $2}' | tail -n1)
    case "$total_bytes" in
        ''|*[!0-9]*) total_bytes=0 ;;
    esac

    # Start curl transfer in background
    curl -fsSL -o "$output" "$url" &
    local curl_pid=$!
    local spin_chars="⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"
    local spin_len=${#spin_chars}
    local spin_idx=0
    local start_time
    start_time=$(date +%s)
    local last_bytes=0
    local speed=0
    local last_time=$start_time

    printf "\033[?25l" # Hide cursor

    while kill -0 "$curl_pid" 2>/dev/null; do
        local cur_bytes=0
        if [ -f "$output" ]; then
            cur_bytes=$(stat -c%s "$output" 2>/dev/null || wc -c < "$output" 2>/dev/null || echo 0)
        fi

        local now
        now=$(date +%s)
        local elapsed=$(( now - start_time ))
        local dt=$(( now - last_time ))
        if [ "$dt" -ge 1 ]; then
            speed=$(( (cur_bytes - last_bytes) / (dt > 0 ? dt : 1) ))
            last_bytes=$cur_bytes
            last_time=$now
        fi

        local char="${spin_chars:spin_idx:1}"
        spin_idx=$(( (spin_idx + 1) % spin_len ))

        local el_min=$(( elapsed / 60 ))
        local el_sec=$(( elapsed % 60 ))
        local el_fmt
        printf -v el_fmt "%02d:%02d" "$el_min" "$el_sec"

        local cur_fmt
        cur_fmt=$(format_size "$cur_bytes")
        local speed_fmt
        speed_fmt=$(format_size "$speed")

        local term_cols
        term_cols=$(get_term_cols)

        if [ "$total_bytes" -gt 0 ]; then
            local pct=$(( cur_bytes * 100 / total_bytes ))
            [ "$pct" -gt 100 ] && pct=100

            local tot_fmt
            tot_fmt=$(format_size "$total_bytes")

            local eta="--s"
            if [ "$speed" -gt 0 ]; then
                local remaining_bytes=$(( total_bytes - cur_bytes ))
                local eta_secs=$(( remaining_bytes / speed ))
                eta="${eta_secs}s"
            fi

            # Dynamically calculate bar width so the line never wraps
            local overhead=52
            if [ "$term_cols" -gt "$(( overhead + 12 ))" ]; then
                local bar_width=$(( term_cols - overhead ))
                [ "$bar_width" -gt 28 ] && bar_width=28
                local filled=$(( pct * bar_width / 100 ))
                local empty=$(( bar_width - filled ))
                local bar_fill=""
                local bar_empty=""
                [ "$filled" -gt 0 ] && printf -v bar_fill "%*s" "$filled" "" && bar_fill="${bar_fill// /=}"
                [ "$empty" -gt 0 ] && printf -v bar_empty "%*s" "$empty" "" && bar_empty="${bar_empty// /-}"

                printf "\r  ${CYAN}%s${NC} [%s] [${PISTACHIO}%s${NC}${DIM}%s${NC}] %s / %s (${PISTACHIO}%s/s${NC}, eta %s)\033[K" \
                    "$char" "$el_fmt" "$bar_fill" "$bar_empty" "$cur_fmt" "$tot_fmt" "$speed_fmt" "$eta"
            else
                # Compact mode for narrow terminal displays
                printf "\r  ${CYAN}%s${NC} [%s] %s / %s (%d%%, ${PISTACHIO}%s/s${NC})\033[K" \
                    "$char" "$el_fmt" "$cur_fmt" "$tot_fmt" "$pct" "$speed_fmt"
            fi
        else
            printf "\r  ${CYAN}%s${NC} [%s] %s (${PISTACHIO}%s/s${NC})\033[K" \
                "$char" "$el_fmt" "$cur_fmt" "$speed_fmt"
        fi

        sleep 0.08
    done

    wait "$curl_pid"
    local exit_code=$?
    printf "\033[?25h" # Restore cursor

    if [ "$exit_code" -eq 0 ]; then
        local final_bytes=0
        if [ -f "$output" ]; then
            final_bytes=$(stat -c%s "$output" 2>/dev/null || echo 0)
        fi
        local final_fmt
        final_fmt=$(format_size "$final_bytes")
        printf "\r  ${GREEN}✓${NC} Downloaded %s (${PISTACHIO}%s${NC})\033[K\n" "$label" "$final_fmt"
        return 0
    else
        printf "\r  ${RED}✗${NC} Download failed for %s\033[K\n" "$label"
        return "$exit_code"
    fi
}

# ----------------------------------------------------
# Responsive Header
# ----------------------------------------------------
echo ""
echo -e "${PISTACHIO_BOLD}⚡ SweepX Installer${NC}"
echo -e "${DIM}Unified Linux Application Tracker & Deep Purge${NC}"
echo -e "${DIM}──────────────────────────────────────────────────${NC}"
echo ""

# 1. Platform Detection
OS="$(uname -s)"
if [ "$OS" != "Linux" ]; then
    echo -e "  ${RED}✗ Error: SweepX is only supported on Linux.${NC}"
    exit 1
fi

ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    aarch64|arm64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        echo -e "  ${RED}✗ Error: Unsupported CPU architecture: $ARCH${NC}"
        exit 1
        ;;
esac

echo -e "  ${GREEN}✓${NC} Platform:  Linux (${TARGET_ARCH})"

# 2. Determine Installation Targets
if [ "$EUID" -eq 0 ]; then
    INSTALL_DIR="/usr/local/bin"
    APPS_DIR="/usr/local/share/applications"
    ICONS_DIR="/usr/local/share/icons/hicolor/scalable/apps"
    PIXMAPS_DIR="/usr/local/share/pixmaps"
    HICOLOR_DIR="/usr/local/share/icons/hicolor"
else
    INSTALL_DIR="$HOME/.local/bin"
    APPS_DIR="$HOME/.local/share/applications"
    ICONS_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
    PIXMAPS_DIR="$HOME/.local/share/pixmaps"
    HICOLOR_DIR="$HOME/.local/share/icons/hicolor"
fi

mkdir -p "$INSTALL_DIR"
mkdir -p "$APPS_DIR"
mkdir -p "$ICONS_DIR"
mkdir -p "$PIXMAPS_DIR"

echo -e "  ${GREEN}✓${NC} Target:    $INSTALL_DIR"

# 3. Check Download Tools
if ! command -v curl >/dev/null 2>&1; then
    echo -e "  ${RED}✗ Error: curl is required to install SweepX.${NC}"
    exit 1
fi

# 4. Fetch Release Binary with Interactive Progress Bar
RELEASE_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest/download/sweepx-${TARGET_ARCH}-unknown-linux-gnu.tar.gz"
FALLBACK_BIN_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest/download/sweepx"

INSTALLED_SUCCESS=0

if download_with_progress "$RELEASE_URL" "$TEMP_DIR/sweepx.tar.gz" "sweepx-${TARGET_ARCH}.tar.gz"; then
    tar -xzf "$TEMP_DIR/sweepx.tar.gz" -C "$TEMP_DIR" 2>/dev/null || true
    if [ -f "$TEMP_DIR/sweepx" ]; then
        mv "$TEMP_DIR/sweepx" "$INSTALL_DIR/$BINARY_NAME"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
        INSTALLED_SUCCESS=1
    fi
fi

if [ "$INSTALLED_SUCCESS" -eq 0 ]; then
    echo -e "  ${YELLOW}⚠ Precompiled archive not found. Trying standalone binary...${NC}"
    if download_with_progress "$FALLBACK_BIN_URL" "$TEMP_DIR/sweepx" "sweepx"; then
        mv "$TEMP_DIR/sweepx" "$INSTALL_DIR/$BINARY_NAME"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
        INSTALLED_SUCCESS=1
    fi
fi

# 5. Fallback: Build from source if precompiled binaries are unavailable
if [ "$INSTALLED_SUCCESS" -eq 0 ]; then
    echo -e "\n  ${YELLOW}⚠ Prebuilt release not available yet. Falling back to Cargo build...${NC}"
    if ! command -v cargo >/dev/null 2>&1; then
        echo -e "  ${CYAN}• Installing lightweight Rust toolchain via rustup...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
        if [ -f "$HOME/.cargo/env" ]; then
            # shellcheck disable=SC1091
            source "$HOME/.cargo/env"
        fi
    fi

    if command -v cargo >/dev/null 2>&1; then
        run_step "Compiling SweepX from source via Cargo" \
            "cargo install --git https://github.com/${REPO_OWNER}/${REPO_NAME}.git --root '$HOME/.local'"
        INSTALLED_SUCCESS=1
    else
        echo -e "  ${RED}✗ Error: Could not locate Rust compiler to build from source.${NC}"
        exit 1
    fi
fi

# 6. Configure Application Icon Assets & Clean Corrupted Caches
run_step "Configuring vector icons and repairing icon caches" "
    # Repair: Remove any truncated index.theme or broken icon-theme.cache created by previous SweepX versions
    if [ -f '$HICOLOR_DIR/index.theme' ]; then
        if grep -q 'Directories=scalable/apps' '$HICOLOR_DIR/index.theme' && [ \"\$(wc -l < '$HICOLOR_DIR/index.theme')\" -lt 25 ]; then
            rm -f '$HICOLOR_DIR/index.theme' '$HICOLOR_DIR/icon-theme.cache'
        fi
    fi
    if [ ! -f '$HICOLOR_DIR/index.theme' ] && [ -f '$HICOLOR_DIR/icon-theme.cache' ]; then
        rm -f '$HICOLOR_DIR/icon-theme.cache'
    fi

    cat <<'SVG_EOF' > '$ICONS_DIR/sweepx.svg'
<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 512 512\" width=\"512\" height=\"512\" role=\"img\" aria-labelledby=\"title-app-icon\">
  <title id=\"title-app-icon\">SweepX Application Icon</title>
  <defs>
    <radialGradient id=\"tile-grad\" cx=\"50%\" cy=\"30%\" r=\"75%\">
      <stop offset=\"0%\" stop-color=\"#4a1827\"/>
      <stop offset=\"100%\" stop-color=\"#2a0d15\"/>
    </radialGradient>
  </defs>
  <rect x=\"24\" y=\"24\" width=\"464\" height=\"464\" rx=\"114\" fill=\"url(#tile-grad)\" stroke=\"#ede5d2\" stroke-opacity=\"0.15\" stroke-width=\"3\"/>
  <g transform=\"translate(256, 256) scale(1.4) translate(-128, -128)\">
    <circle cx=\"128\" cy=\"128\" r=\"20\" fill=\"#ede5d2\"/>
    <path fill=\"none\" stroke=\"#ede5d2\" stroke-width=\"18\" stroke-linecap=\"round\" d=\"M 86 58 A 96 96 0 0 1 170 58\"/>
    <path fill=\"none\" stroke=\"#ede5d2\" stroke-width=\"18\" stroke-linecap=\"round\" d=\"M 86 198 A 96 96 0 0 0 170 198\"/>
    <path fill=\"none\" stroke=\"#ede5d2\" stroke-width=\"18\" stroke-linecap=\"round\" d=\"M 58 86 A 96 96 0 0 0 58 170\"/>
    <path fill=\"none\" stroke=\"#ede5d2\" stroke-width=\"18\" stroke-linecap=\"round\" d=\"M 198 86 A 96 96 0 0 1 198 170\"/>
    <path fill=\"none\" stroke=\"#ede5d2\" stroke-width=\"14\" stroke-linecap=\"round\" d=\"M 98 84 A 56 56 0 0 1 158 84\"/>
    <path fill=\"none\" stroke=\"#ede5d2\" stroke-width=\"14\" stroke-linecap=\"round\" d=\"M 98 172 A 56 56 0 0 0 158 172\"/>
    <path fill=\"none\" stroke=\"#ede5d2\" stroke-width=\"14\" stroke-linecap=\"round\" d=\"M 84 98 A 56 56 0 0 0 84 158\"/>
    <path fill=\"none\" stroke=\"#ede5d2\" stroke-width=\"14\" stroke-linecap=\"round\" d=\"M 172 98 A 56 56 0 0 1 172 158\"/>
    <rect x=\"118\" y=\"24\" width=\"20\" height=\"208\" rx=\"10\" transform=\"rotate(-45 128 128)\" fill=\"#9ecb82\"/>
  </g>
</svg>
SVG_EOF

    cp '$ICONS_DIR/sweepx.svg' '$PIXMAPS_DIR/sweepx.svg' 2>/dev/null || true
    if [ '$EUID' -ne 0 ]; then
        mkdir -p '$HOME/.local/share/icons'
        cp '$ICONS_DIR/sweepx.svg' '$HOME/.local/share/icons/sweepx.svg' 2>/dev/null || true
    fi

    rm -f '$HOME/.local/share/icons/sweepx.png' \
          '$HOME/.local/share/pixmaps/sweepx.png' \
          '$HICOLOR_DIR/48x48/apps/sweepx.png' \
          '$HICOLOR_DIR/128x128/apps/sweepx.png' \
          '$HICOLOR_DIR/256x256/apps/sweepx.png' 2>/dev/null || true
"

# 7. Register Desktop Launcher
run_step "Registering desktop launcher" "
    cat <<EOF > '$APPS_DIR/sweepx.desktop'
[Desktop Entry]
Type=Application
Name=SweepX
Comment=Unified Linux Application Tracker & Deep Uninstaller
Exec=$INSTALL_DIR/$BINARY_NAME gui
Icon=$ICONS_DIR/sweepx.svg
Categories=System;Utility;Settings;
Terminal=false
StartupNotify=true
StartupWMClass=sweepx
EOF
    chmod +x '$APPS_DIR/sweepx.desktop'
    rm -f '$HOME/Desktop/sweepx.desktop' 2>/dev/null || true
"

# 8. Refresh Desktop & Icon Databases
run_step "Updating desktop and icon caches" "
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database '$APPS_DIR' 2>/dev/null || true
    fi
    if [ -f '$HICOLOR_DIR/index.theme' ] && command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t '$HICOLOR_DIR' 2>/dev/null || true
    fi
    touch '$HICOLOR_DIR' '$APPS_DIR' 2>/dev/null || true
"

# 9. Verify PATH configuration
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo -e "  ${YELLOW}Notice: $INSTALL_DIR is not currently in your \$PATH.${NC}"
    for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
        if [ -f "$rc" ] && ! grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$rc"; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$rc"
            echo -e "  ${GREEN}✓${NC} Appended ~/.local/bin to $(basename "$rc")"
        fi
    done
fi

# ----------------------------------------------------
# Responsive Completion Summary
# ----------------------------------------------------
echo ""
echo -e "${GREEN}✓ SweepX installed successfully!${NC}"
echo ""
echo -e "${CREAM_BOLD}Installed components:${NC}"
echo -e "  • Binary:    ${CYAN}$INSTALL_DIR/$BINARY_NAME${NC}"
echo -e "  • Launcher:  ${CYAN}$APPS_DIR/sweepx.desktop${NC}"
echo -e "  • Icon:      ${CYAN}$ICONS_DIR/sweepx.svg${NC}"
echo ""
echo -e "${CREAM_BOLD}Quick start:${NC}"
echo -e "  ${PISTACHIO}sweepx${NC}         ${DIM}Launch graphical desktop interface${NC}"
echo -e "  ${PISTACHIO}sweepx list${NC}    ${DIM}List all discovered applications${NC}"
echo -e "  ${PISTACHIO}sweepx update${NC}  ${DIM}Check GitHub releases and update${NC}"
echo ""
