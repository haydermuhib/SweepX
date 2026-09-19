# ⚡ SweepX

**Unified Linux Application Tracker & Deep Uninstaller**

[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux-green.svg)](#supported-distributions)

SweepX is a high-performance, standalone Linux desktop application and command-line utility written in Rust. It provides a single unified dashboard to discover, inspect, track, and surgically purge software across all Linux packaging tiers.

---

## 🌟 Key Features

### 1. Cross-Tier Unified Discovery Engine
SweepX discovers applications across every packaging layer without making distro-specific assumptions:
- **Native System Packages:** Auto-detects and inspects APT/dpkg (Debian/Ubuntu), DNF/RPM (Fedora/RHEL), and Pacman/ALPM (Arch/Manjaro).
- **Universal Containers:** Inspects Flatpak apps, sandbox storage (`~/.var/app/`), Snap packages (`~/snap/`), and standalone AppImages.
- **Manual & Unmanaged Software:** Discovers applications in `/opt/`, user binaries in `~/.local/bin/` and `/usr/local/bin/`, and parses standard XDG `.desktop` launchers.

### 2. Hybrid Heuristic & Pattern Residual Cleaner
When software is removed, leftover files often clutter your drive. SweepX combines:
- **Pattern Signatures:** Curated mappings for complex applications (VS Code, JetBrains IDEs, Chrome/Brave, Firefox, Spotify, Discord, Steam, Docker, etc.).
- **XDG Heuristics:** Crawls `~/.config`, `~/.cache`, `~/.local/share`, `~/.local/state`, `/var/log`, and `/etc` with fuzzy slug and reverse-DNS matching.
- **Orphan Scanner:** Detects leftover configuration and cache folders for apps that were uninstalled in the past.

### 3. Strict Safety Guardrails & FreeDesktop Trash
- **System Blacklist Protection:** Hard blocks preventing deletion of root and critical parent paths (`/`, `/home`, `/usr`, `/etc`, `~/.config`, etc.).
- **Trash by Default:** Moves leftover files to your desktop's FreeDesktop Trash (`trash-rs`), with an optional toggle for permanent deletion.
- **PolicyKit Escalation:** Uses native PolicyKit (`pkexec`) graphical auth prompts with seamless fallback to `sudo`.

### 4. Local SQLite State & Audit Trail
- Embedded SQLite database (`~/.local/share/sweepx/sweepx.db`).
- Tracks historical install/update dates, storage growth, and uninstallation audit logs.

### 5. Dual Interface: Modern GUI + Powerful CLI
- **GUI:** Sleek, responsive dark-themed immediate-mode interface powered by `eframe` & `egui`.
- **CLI:** Fast headless subcommands for terminal power users and scripts.

---

## 🚀 CLI Usage

SweepX includes a complete CLI interface:

```bash
# List all discovered applications
sweepx list

# Filter by packaging tier (native, flatpak, snap, appimage, manual)
sweepx list --category flatpak

# Output application list as formatted JSON
sweepx list --json

# Inspect detailed file paths, launcher entries, and residual caches
sweepx inspect com.spotify.Client

# Deep clean an application with a dry run
sweepx purge slack --dry-run

# Deep clean an application and move residuals to Desktop Trash
sweepx purge slack

# Permanently purge without moving to Trash
sweepx purge slack --permanent

# Scan and list all orphaned residual folders on disk
sweepx residuals

# Clean all detected orphaned caches and configs
sweepx residuals --clean

# View audit history of past uninstalled applications
sweepx history

# Launch GUI interface
sweepx gui
# (or simply run `sweepx` without arguments)
```

---

## 🛠️ Building & Running from Source

### Prerequisites
- Linux (x86_64, aarch64, or any standard Linux architecture)
- Rust toolchain (2024 edition / 1.85+)

### Compilation
```bash
# Clone the repository
git clone https://github.com/username/Gwen.git sweepx
cd sweepx

# Run all test suites (Models, Scanners, Cleaners, DB, E2E)
cargo test

# Build optimized release binary
cargo build --release

# Run SweepX
./target/release/sweepx
```

---

## 🏛️ Architecture Overview

```
                          ┌───────────────────────────┐
                          │   SweepX Entrypoint       │
                          │   (Dual Mode: GUI / CLI)  │
                          └─────────────┬─────────────┘
                                        │
                 ┌──────────────────────┴──────────────────────┐
                 ▼                                             ▼
     ┌───────────────────────┐                     ┌───────────────────────┐
     │    egui / eframe      │                     │     clap CLI Runner   │
     │   Desktop GUI App     │                     │     (Subcommands)     │
     └───────────┬───────────┘                     └───────────┬───────────┘
                 │                                             │
                 └──────────────────────┬──────────────────────┘
                                        ▼
                  ┌───────────────────────────────────────────┐
                  │          Async Tokio Core Engine          │
                  └─────────────────────┬─────────────────────┘
                                        │
        ┌───────────────────────────────┼───────────────────────────────┐
        ▼                               ▼                               ▼
┌───────────────┐               ┌───────────────┐               ┌───────────────┐
│ Discovery &   │               │ Deep Cleaner  │               │ Local SQLite  │
│ Scanner Engine│               │ & Purge Exec  │               │ Audit Store   │
├───────────────┤               ├───────────────┤               ├───────────────┤
│ • Desktop XDG │               │ • Signatures  │               │ • App Cache   │
│ • Flatpak CLI │               │ • Heuristics  │               │ • Audit Logs  │
│ • Snap CLI    │               │ • Safety Gate │               │ • History     │
│ • DNF / RPM   │               │ • Trash / Rm  │               │               │
│ • APT / Dpkg  │               │ • pkexec/sudo │               │               │
│ • Pacman/ALPM │               └───────────────┘               └───────────────┘
│ • /opt Manual │
└───────────────┘
```

---

## 🛡️ Safety Policies
SweepX enforces strict path verification before attempting any deletion or move-to-trash action. Attempting to delete protected system directories or root parent paths results in immediate rejection and is logged in the audit trail.

---

## 📄 License
Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
