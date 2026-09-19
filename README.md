# SweepX

> Unified Linux application tracker, live installation watcher, and deep residual uninstaller in pure Rust.

[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Build Status](https://github.com/haydermuhib/SweepX/actions/workflows/release.yml/badge.svg)](https://github.com/haydermuhib/SweepX/actions)
[![Language: Rust](https://img.shields.io/badge/language-Rust%202024-orange.svg)](https://www.rust-lang.org/)
[![Showcase](https://img.shields.io/badge/Showcase-GitHub%20Pages-success.svg)](https://haydermuhib.github.io/SweepX/)

SweepX brings native packages (dpkg, pacman, rpm), Flatpaks, Snaps, standalone AppImages, and manual installations in `/opt` into one interface. It tracks manual builds in real time and strips leftover caches, configs, and orphaned background services when software is removed.

---

## Quick navigation

- [Interactive Showcase](https://haydermuhib.github.io/SweepX/)
- [Quick Installation](#quick-installation)
- [Key Features](#key-features)
- [Command-Line Interface](#command-line-interface)
- [Building from Source](#building-from-source)
- [Architecture & Contributor Guide](ARCHITECTURE.md)
- [LLM Specification](llms.txt)
- [License](#license)

---

## Quick installation

### Universal 1-line installer (all Linux distributions)

```bash
curl -fsSL https://raw.githubusercontent.com/haydermuhib/SweepX/main/install.sh | bash
```

The script detects your architecture (`x86_64` or `aarch64`), fetches the latest prebuilt release binary, installs vector desktop icons, creates application menu launchers, and sets up your terminal `PATH`. If prebuilt binaries are unavailable, it offers an automatic source compilation fallback.

---

## Key features

### 1. Unified package tracker
Discovers and tracks software across multiple packaging systems simultaneously:
- **Native packages:** Debian/Ubuntu (APT/dpkg), Fedora/RHEL (DNF/RPM), and Arch Linux (Pacman).
- **Container runtimes:** Flatpak user and system installations, plus active Snap packages.
- **Standalone binaries:** Portable AppImages and unmanaged software installed in `/opt` or `~/.local/bin`.

### 2. Live installation watcher
Tracks manual source builds (`make install`, custom installer scripts) in real time. SweepX records filesystem snapshot diffs and stores an exact manifest, allowing complete, zero-loss uninstallation later:
```bash
sweepx watch --name "my-tool" -- sudo make install
```

### 3. Deep residual hunter
Traditional package managers often leave user configuration directories, caches, and systemd units behind on disk. SweepX scans and purges:
- XDG configuration trees (`~/.config/<app>`)
- Application caches (`~/.cache/<app>`)
- Data directories (`~/.local/share/<app>`)
- User-level background services (`~/.config/systemd/user/<app>.service`)
- Dangling and broken symlinks in `/usr/local/bin` and `~/.local/bin`

### 4. System optimization sweep
Finds and reclaims disk space from obsolete runtime files:
- **Snap revisions:** Prunes disabled and old Snap revisions.
- **Flatpak runtimes:** Removes unreferenced runtime dependencies and temporary run data.
- **Package caches:** Cleans APT, DNF, and Pacman download caches.
- **Bulky installer archives:** Flags lingering `.iso`, `.zip`, and `.tar.gz` downloads in `~/Downloads`.

### 5. AppImage desktop integration
Inspects AppImages, extracts embedded `.DirIcon` assets into `~/.local/share/icons`, and generates valid XDG `.desktop` application menu launchers with one click.

### 6. Multi-tier safety barrier
- **Protected paths:** Deletions targeting root, user home roots (`/home/$USER`), `/usr`, `/etc`, or `/var` are strictly blocked.
- **FreeDesktop Trash:** Moves residual files to the desktop Trash by default, with an optional permanent purge mode.
- **PolicyKit integration:** Uses `pkexec` graphical prompts for system operations with terminal `sudo` fallback.

---

## Command-line interface

Launch the graphical dashboard with `sweepx`. For terminal automation and scripts, the following subcommands are available:

```bash
# List all installed software
sweepx list

# Filter applications by packaging tier
sweepx list --filter flatpak
sweepx list --filter appimage
sweepx list --filter native

# Output records as machine-readable JSON
sweepx list --json

# Inspect an application, its launchers, dependencies, and caches
sweepx inspect obsidian

# Clean leftover residual caches and configs
sweepx clean slack

# Dry-run residual cleanup without touching disk
sweepx clean slack --dry-run

# Purge an unmanaged /opt or AppImage installation
sweepx purge custom-tool

# Watch and record an installation command
sweepx watch --name "neovim-nightly" -- sudo make install

# Scan and clean system bloat (Snap revisions, Flatpak runtimes, caches)
sweepx sweep
sweepx sweep --dry-run

# View uninstallation audit history and lifetime reclaimed space
sweepx history

# Check for updates on GitHub and self-update
sweepx update --check
sweepx update
```

<details>
<summary><b>📋 CLI command reference</b></summary>

| Subcommand | Flag / Option | Description |
| :--- | :--- | :--- |
| `list` | `--filter <type>` | Filter by `native`, `flatpak`, `snap`, `appimage`, or `manual` |
| `list` | `-s, --all-system` | Include core system libraries and dependencies |
| `list` | `--json` | Output results in JSON format |
| `inspect` | `<app_id>` | Display binary paths, launchers, and discovered residual files |
| `clean` | `<app_id>` | Remove leftover user caches, configs, and systemd units |
| `clean` | `--dry-run` | Preview files to be removed without deleting |
| `clean` | `--permanent` | Delete files directly instead of moving to Trash |
| `clean` | `--yes` | Skip confirmation prompt |
| `purge` | `<app_id>` | Remove unmanaged software binary and all related artifacts |
| `watch` | `-n, --name <name>` | Application identifier for the recorded manifest |
| `watch` | `<command...>` | Installation command to execute and monitor |
| `sweep` | `--dry-run` | Scan reclaimable container revisions and caches without deleting |
| `sweep` | `--json` | Output reclaimable items in JSON |
| `history` | None | Print chronological audit log of past uninstalls and space saved |
| `update` | `--check` | Check GitHub releases without downloading |

</details>

---

## Building from source

### Prerequisites
- Linux (`x86_64` or `aarch64`)
- Rust toolchain (1.85+ / 2024 edition)
- Graphics libraries: `libx11-dev`, `libxcursor-dev`, `libxrandr-dev`, `libxi-dev`, `libgl1-mesa-dev`, `libxkbcommon-dev`, `libwayland-dev`

```bash
# Clone the repository
git clone https://github.com/haydermuhib/SweepX.git
cd SweepX

# Run test suite (30 unit & integration tests)
cargo test

# Build optimized release binary
cargo build --release

# Run SweepX
./target/release/sweepx
```

---

## Architecture overview

```
┌─────────────────────────────────────────────────────────────┐
│                      SweepX Entrypoint                      │
│                   (Dual Mode: GUI / CLI)                    │
└──────────────────────────────┬──────────────────────────────┘
                               │
               ┌───────────────┴───────────────┐
               ▼                               ▼
   ┌───────────────────────┐       ┌───────────────────────┐
   │     eframe / egui     │       │    clap CLI Runner    │
   │    Desktop GUI App    │       │     (Subcommands)     │
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

For complete subsystem diagrams, sequence flows, and contributor guides, see **[ARCHITECTURE.md](ARCHITECTURE.md)**.

---

## Documentation & Contributing

For a comprehensive understanding of the project internals, multi-tier scanning engine, zero-tolerance safety guardrails, and how to contribute:

- 📖 **[ARCHITECTURE.md](ARCHITECTURE.md)** — Complete subsystem breakdown, data flow sequence diagrams, and module responsibilities.
- 📊 **[PRESENTATION.md](PRESENTATION.md)** — 15-minute architecture slide deck & technical deep dive for contributors and maintainers.
- 🌐 **[Showcase Website](https://haydermuhib.github.io/SweepX/)** — Interactive landing page, feature showcase, and visual tour.
- 🤖 **[Machine Documentation](llms.txt)** — Context for AI agents and LLM tooling.

---

## License

Dual-licensed under either:
- **MIT License** ([LICENSE-MIT](LICENSE-MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

Copyright © Haider Ali Tariq and SweepX Contributors.
