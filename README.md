# SweepX

**Unified Linux application tracker, live installation watcher, and deep uninstaller.**

[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux-green.svg)](#installation)

SweepX is a standalone Linux desktop utility written in pure Rust. It tracks applications across native package managers, container formats, and manual directory installations, with tools to inspect file footprints, record live installs, optimize container storage, and completely remove applications and leftover caches.

---

## Quick installation

### One-line installer (All Linux distributions)

```bash
curl -fsSL https://raw.githubusercontent.com/haydermuhib/SweepX/main/install.sh | bash
```

The script detects your CPU architecture (`x86_64` or `aarch64`), installs the binary to `~/.local/bin/sweepx`, and registers a desktop launcher in your applications menu.

---

## Core capabilities

### 1. Unified cross-tier application tracker
SweepX scans packaging layers concurrently and streams results to the interface in real time:
- **Native package managers:** APT (Debian/Ubuntu/Mint), DNF/RPM (Fedora/RHEL), and Pacman (Arch/Manjaro).
- **Container runtimes:** Flatpak (`~/.var/app/`) and Snap (`~/snap/`).
- **Manual and portable apps:** Standalone AppImages (in `~/Applications`, `~/Downloads`, and `~/Desktop`), `/opt` suites, and custom user binaries in `~/.local/bin`.

### 2. Live install watch (Snapshot engine)
When compiling software from source or running manual install scripts (`./install.sh`, `make install`, `cargo install`):
```bash
sweepx watch --name "my-tool" sudo make install
```
SweepX records a point-in-time filesystem diff before and after installation, saves an exact file manifest to its database, and allows you to cleanly remove every created file later.

### 3. Symlink graph resolver
Scans `~/.local/bin`, `/usr/local/bin`, and `~/.local/share/applications` to resolve symlink targets in `/opt` and detect broken symlinks. When uninstalling an application, SweepX unlinks pointing symlinks across your PATH.

### 4. Container and system optimizer
- **Snap revisions:** Prunes old, disabled revisions from `/var/lib/snapd/snaps/`.
- **Flatpak runtimes:** Removes unused runtime platforms via `flatpak uninstall --unused`.
- **Package caches:** Cleans APT, DNF, and Pacman download caches.
- **Installer archives:** Flags bulky `.iso`, `.zip`, and `.tar.gz` installer downloads in `~/Downloads`.

### 5. AppImage desktop integration
Inspects AppImages, extracts embedded `.DirIcon` assets into `~/.local/share/icons`, and generates valid XDG `.desktop` launchers with a single click.

### 6. Strict safety rules
- **Protected paths:** Deletion is blocked for root and system directories (`/`, `/usr`, `/etc`, `/home`, `~/.config`).
- **FreeDesktop Trash:** Moves residual files to the desktop Trash by default, with an optional permanent deletion mode.
- **PolicyKit integration:** Uses `pkexec` graphical prompts for system operations with a terminal `sudo` fallback.

---

## Command-line interface

Launch the GUI by running `sweepx` or `sweepx gui`. For terminal workflows, the following subcommands are available:

```bash
# List all installed applications
sweepx list

# Filter applications by packaging category
sweepx list --category flatpak
sweepx list --category appimage
sweepx list --category native

# Output application records in JSON format
sweepx list --json

# Inspect an application's executable, launcher, and leftover files
sweepx inspect obsidian

# Deep-clean an application (moves leftover caches and configs to Trash)
sweepx purge slack

# Perform a dry run without deleting files
sweepx purge slack --dry-run

# Purge permanently without moving to Trash
sweepx purge slack --permanent

# Watch and record a manual installation command
sweepx watch --name "neovim-nightly" make install

# Scan and clean container bloat, package caches, and broken symlinks
sweepx sweep
sweepx sweep --dry-run

# View uninstallation history
sweepx history

# Check for updates from GitHub and self-update
sweepx update
sweepx update --check
```

<details>
<summary><b>📋 CLI command reference</b></summary>

| Subcommand | Flag / Option | Description |
| :--- | :--- | :--- |
| `list` | `--category <type>` | Filter by `all`, `native`, `flatpak`, `snap`, `appimage`, or `manual` |
| `list` | `-s, --all-system` | Include system libraries and core runtimes |
| `list` | `--json` | Output results as JSON |
| `inspect` | `<app_id>` | Display binaries, launchers, dependencies, and discovered configs |
| `purge` | `<app_id>` | Remove package and clean associated user files |
| `purge` | `--dry-run` | Preview files to be removed without deleting |
| `purge` | `--permanent` | Delete files directly instead of moving to Trash |
| `watch` | `-n, --name <name>` | Application identifier for the recorded manifest |
| `watch` | `<command...>` | Installation command to execute |
| `sweep` | `--dry-run` | Scan reclaimable container revisions and caches without deleting |
| `sweep` | `--json` | Output reclaimable items in JSON |
| `history` | None | Print chronological audit log of past uninstalls |
| `update` | `--check` | Check GitHub releases without updating |

</details>

---

## Building from source

### Prerequisites
- Linux (`x86_64` or `aarch64`)
- Rust toolchain (2024 edition / 1.85+)

```bash
# Clone the repository
git clone https://github.com/haydermuhib/SweepX.git
cd SweepX

# Run test suite
cargo test

# Build optimized release binary
cargo build --release

# Run SweepX
./target/release/sweepx
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    SweepX Entrypoint                        │
│                 (Dual Mode: GUI / CLI)                      │
└──────────────────────────────┬──────────────────────────────┘
                               │
               ┌───────────────┴───────────────┐
               ▼                               ▼
   ┌───────────────────────┐       ┌───────────────────────┐
   │    eframe / egui      │       │     clap CLI Runner   │
   │   Desktop GUI App     │       │     (Subcommands)     │
   └───────────┬───────────┘       └───────────┬───────────┘
               │                               │
               └───────────────┬───────────────┘
                               ▼
 ┌───────────────────────────────────────────────────────────┐
 │               Async Tokio Multi-Task Engine               │
 └─────────────────────────────┬─────────────────────────────┘
                               │
 ┌───────────────┬─────────────┴─┬─────────────┬─────────────┐
 ▼               ▼               ▼             ▼             ▼
┌──────────────┐┌──────────────┐┌────────────┐┌────────────┐┌─────────────┐
│ Orchestrator ││ Deep Cleaner ││ Symlink    ││ Snapshot   ││ SQLite DB   │
│ & Scanners   ││ & Safety Gate││ Resolver   ││ Watcher    ││ Audit Store │
├──────────────┤├──────────────┤├────────────┤├────────────┤├─────────────┤
│ • Desktop XDG││ • Signatures ││ • GNU Stow ││ • Live Diff││ • Apps Cache│
│ • Flatpak CLI││ • Heuristics ││ • Broken   ││ • Manifest ││ • Audit Logs│
│ • Snap CLI   ││ • Blacklist  ││   Cleaner  ││ • Zero-Loss││ • Manifests │
│ • APT/DNF/Pac││ • Trash-rs   ││ • Unlink   ││   Purge    ││ • History   │
│ • AppImages  ││ • PolicyKit  ││   PATH     ││            ││             │
└──────────────┘└──────────────┘└────────────┘└────────────┘└─────────────┘
```

<details>
<summary><b>📋 Packaging tier comparison</b></summary>

| Packaging tier | Discovery method | Residual locations | Uninstallation mechanism |
| :--- | :--- | :--- | :--- |
| **Native (APT/DNF/Pacman)** | Package manager database query + `.desktop` entries | `~/.config`, `~/.cache`, `~/.local/share` | Package manager command (`apt purge`, `dnf remove`, `pacman -Rns`) |
| **Flatpak** | `flatpak list --app` | `~/.var/app/<id>/` | `flatpak uninstall <id>` |
| **Snap** | `snap list` | `~/snap/<id>/`, `/var/snap/<id>/` | `snap remove <id>` |
| **AppImage** | Directory scans in `~/Applications`, `~/Downloads`, `~/Desktop` | `~/.config/<name>`, `~/.local/share/applications/` | Direct binary deletion + unlinking `.desktop` launcher |
| **Manual /opt** | Directory inspection in `/opt`, symlink resolution | `/opt/<app>`, `~/.local/bin/<symlink>` | Coordinated directory purge + PATH unlinking |

</details>

---

## License

SweepX is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
