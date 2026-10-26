# SweepX Architecture and Contributor Guide

This document describes the internal architecture, subsystems, data flow, and file responsibilities of SweepX.

---

## 1. System Architecture

```mermaid
graph TD
    subgraph Entrypoints
        CLI["CLI Entrypoint<br><code>src/cli/mod.rs</code>"]
        GUI["GUI Application<br><code>src/ui/app.rs</code> (Iced 0.14)"]
    end

    subgraph Core Engine
        Scanner["Multi-Tier Scanner<br><code>src/scanner/orchestrator.rs</code>"]
        Tracker["Installation Watcher<br><code>src/tracker/snapshot.rs</code>"]
        Cleaner["Clean and Purge Engine<br><code>src/cleaner/executor.rs</code>"]
        Safety["Safety Barrier<br><code>src/cleaner/safety.rs</code>"]
        DB["SQLite Database<br><code>src/db/repository.rs</code>"]
        Updater["GitHub Self-Updater<br><code>src/updater/mod.rs</code>"]
    end

    subgraph Data Sources and Systems
        NativePM["Native PMs<br>(dpkg / pacman / rpm)"]
        Containers["Containers<br>(Flatpak / Snap)"]
        Bottles["Bottles Wine Apps<br>(~/.var/app/.../bottles)"]
        Filesystem["Filesystem<br>(/opt, ~/.local/bin, ~/.cargo/bin, AppImages)"]
        XDG["XDG Tree and Services<br>(~/.config, ~/.cache, systemd)"]
    end

    CLI --> Scanner
    CLI --> Tracker
    CLI --> Cleaner
    CLI --> Updater

    GUI --> Scanner
    GUI --> Cleaner
    GUI --> DB
    GUI --> Updater

    Scanner --> NativePM
    Scanner --> Containers
    Scanner --> Bottles
    Scanner --> Filesystem

    Tracker --> DB
    Cleaner --> Safety
    Cleaner --> XDG
    Cleaner --> DB
```

---

## 2. Subsystem Breakdown

```
src/
├── main.rs                  # Application entry point, CLI routing, and GUI launch
├── lib.rs                   # Library exports and shared module definitions
├── cli/                     # CLI parser, command handlers, and terminal output
│   ├── commands.rs          # Subcommands definition using clap
│   └── mod.rs               # Terminal UI, progress reporting, tables, and execution
├── db/                      # SQLite database connection, schema, and queries
│   ├── schema.rs            # Table migrations (install_manifests, manifest_files, audit_logs)
│   └── repository.rs        # Database methods, audit logging, and recovered space calculations
├── models/                  # Core domain models
│   ├── app.rs               # Application sum type, InstallMethod, and SizeUnitSystem
│   ├── artifact.rs          # AppArtifact and ArtifactKind definitions
│   ├── residual.rs          # ResidualCandidate and file category types
│   └── mod.rs               # Pure formatting utilities (format_size_with_unit)
├── scanner/                 # Discovery engines across all packaging tiers
│   ├── orchestrator.rs      # Multi-tier async scanner, deduplicator, and container insulator
│   ├── desktop_entry.rs     # Freedesktop .desktop parser and icon resolver
│   ├── flatpak.rs           # Flatpak CLI client and runtime parser
│   ├── snap.rs              # Snap CLI client and revision tracker
│   ├── bottles.rs           # Bottles Wine prefix scanner and YAML parser
│   ├── appimage.rs          # AppImage detector, ELF header parser, and integrator
│   ├── manual.rs            # User-space $PATH, shell profile, and /opt scanner
│   ├── symlink_graph.rs     # Stow-style symlink graph resolver
│   └── native/              # Package manager clients (dpkg, rpm, pacman)
├── cleaner/                 # Residual detection, safety validator, and executor
│   ├── safety.rs            # Zero-tolerance safety validator and protected path blacklist
│   ├── heuristic.rs         # Isolated container, XDG, and dotdir residual crawler
│   ├── signatures.rs        # Known directory patterns dictionary for complex apps
│   ├── executor.rs          # Physical package purge, trash, and permanent deletion
│   ├── privilege.rs         # Root elevation manager via pkexec and sudo
│   └── system_sweep.rs      # System optimizer (old Snaps, unused Flatpaks, package caches)
├── tracker/                 # Filesystem snapshot diffing and install watcher
│   ├── snapshot.rs          # Directory tree capture and diff calculation
│   └── mod.rs               # Watcher engine and manifest persistence
├── ui/                      # Iced 0.14 desktop application (WGPU backend)
│   ├── app.rs               # GUI state loop, view layouts, modal dialogs, and window setup
│   ├── theme.rs             # Brand design tokens (Burgundy, Cream, Pistachio)
│   └── mod.rs               # UI exports, sort and filter helpers, window icon accessor
└── updater/                 # Self-updater querying GitHub Releases API
    └── mod.rs               # Release download, atomic binary replacement, asset installation
```

---

### 1. Multi-Tier Scanner Engine (`src/scanner/`)

The scanner queries package registries, container runtimes, and local directories concurrently.

| File | Scanner Type | Responsibility |
| :--- | :--- | :--- |
| [`src/scanner/orchestrator.rs`](src/scanner/orchestrator.rs) | `ScannerOrchestrator` | Runs all tier scanners concurrently, applies multi-tier deduplication rules, protects Snap and Flatpak container packages from native package overwrites, and merges desktop launchers with native packages. |
| [`src/scanner/native/apt.rs`](src/scanner/native/apt.rs) | `AptScanner` | Queries Debian and Ubuntu `dpkg-query` and parses status records. |
| [`src/scanner/native/dnf.rs`](src/scanner/native/dnf.rs) | `DnfScanner` | Queries Fedora, RHEL, and openSUSE `rpm` database for installed software. |
| [`src/scanner/native/pacman.rs`](src/scanner/native/pacman.rs) | `PacmanScanner` | Queries Arch Linux `pacman -Qi` for local packages. |
| [`src/scanner/flatpak.rs`](src/scanner/flatpak.rs) | `FlatpakScanner` | Parses output from `flatpak list --app` and extracts versions, sizes, and IDs. |
| [`src/scanner/snap.rs`](src/scanner/snap.rs) | `SnapScanner` | Inspects Snap revisions, publisher channels, and package status without blocking on missing daemon sockets. |
| [`src/scanner/bottles.rs`](src/scanner/bottles.rs) | `BottlesScanner` | Scans Bottles Wine prefixes (`~/.var/app/com.usebottles.bottles/data/bottles/bottles`), parses YAML configuration, and detects Windows application payloads and launchers. |
| [`src/scanner/appimage.rs`](src/scanner/appimage.rs) | `AppImageScanner` | Discovers standalone AppImages, verifies magic bytes, and provides desktop integration. |
| [`src/scanner/manual.rs`](src/scanner/manual.rs) | `ManualScanner` | Discovers standalone software across `/opt`, standard user binary locations (`~/.local/bin`, `~/.cargo/bin`, `~/Applications`, `~/bin`, `/usr/local/bin`), and dynamic user-space `$PATH` directories. Queries the user shell (`$SHELL -il`) and parses shell profile files (`~/.zshrc`, `~/.bashrc`, `~/.cargo/env`, `~/.profile`) to retrieve authentic development tools in GUI sessions without hardcoding specific application names. |
| [`src/scanner/desktop_entry.rs`](src/scanner/desktop_entry.rs) | `DesktopEntryScanner` | Scans `~/.local/share/applications` and `/usr/share/applications`, extracting Exec commands, Categories, and icon paths. |
| [`src/scanner/symlink_graph.rs`](src/scanner/symlink_graph.rs) | `SymlinkGraph` | Traces symlink graphs in `/usr/local/bin` and `~/.local/bin` to locate target binaries. |

---

### 2. Safety Barrier and Residual Cleaner (`src/cleaner/`)

Every candidate path must pass validation before removal.

```mermaid
graph TD
    Target["Residual Target Paths"] --> Safety{"safety.rs<br>(Blacklist Check)"}
    Safety -- "Matches System Root, Base XDG, or Shared Container" --> Denied["Block Deletion (SafetyViolation)"]
    Safety -- "Passes Path Depth and Blacklist Gates" --> Executor["executor.rs<br>(Trash or Permanent Unlink)"]
    Executor --> Audit["db/repository.rs<br>(Record Freed Space)"]
```

| File | Component | Responsibility |
| :--- | :--- | :--- |
| [`src/cleaner/safety.rs`](src/cleaner/safety.rs) | `SafetyValidator` | Validates target paths against protected system roots (`/`, `/usr`, `/etc`), base XDG folders (`~/.config`, `~/.cache`), and shared container repositories (`~/.local/share/flatpak`, `/var/lib/flatpak`, `/var/lib/snapd`, `/usr/bin/flatpak`). |
| [`src/cleaner/heuristic.rs`](src/cleaner/heuristic.rs) | `ResidualCleaner` | Crawls app-specific folders. Enforces container isolation (`~/.var/app/<id>` for Flatpak, `~/snap/<name>` for Snap), evaluates direct dotdirs (`~/.<app>`), and filters out forbidden keywords (`flatpak`, `snap`, `systemd`, `usr`, `bin`). |
| [`src/cleaner/signatures.rs`](src/cleaner/signatures.rs) | `ResidualSignatures` | Dictionary of known directory patterns for applications that place data across multiple non-standard paths (e.g. VS Code, Chrome, Firefox). |
| [`src/cleaner/executor.rs`](src/cleaner/executor.rs) | `DeletionExecutor` | Removes files by moving them to desktop Trash or unlinking permanently. For package purges (`ManualOpt`, `CustomDesktop`, `AppImage`), unlinks executables, launchers, and tracked artifacts, recording audit entries only upon successful deletion. |
| [`src/cleaner/system_sweep.rs`](src/cleaner/system_sweep.rs) | `SystemSweepEngine` | Cleans shared system bloat: removes disabled Snap revisions, runs `flatpak uninstall --unused`, and clears package manager caches. Records cleaned paths and freed bytes directly into the audit history. |
| [`src/cleaner/privilege.rs`](src/cleaner/privilege.rs) | `PrivilegeManager` | Manages root elevation requests via `pkexec` or `sudo` when working with system-level paths. |

---

### 3. Installation Watcher (`src/tracker/`)

Tracks files created during manual builds (`make install` or custom install scripts) to allow clean uninstallation later.

| File | Component | Responsibility |
| :--- | :--- | :--- |
| [`src/tracker/snapshot.rs`](src/tracker/snapshot.rs) | `FilesystemSnapshot` | Captures recursive directory trees in `/usr/local`, `~/.local`, and `/opt` before and after an installation command. |
| [`src/tracker/mod.rs`](src/tracker/mod.rs) | `InstallWatcher` | Calculates directory diffs (new and modified files) and writes manifests to the SQLite database. |

---

### 4. Database Storage (`src/db/`)

Embedded SQLite storage for audit history and installation manifests.

| File | Purpose |
| :--- | :--- |
| [`src/db/schema.rs`](src/db/schema.rs) | Manages tables: `install_manifests`, `manifest_files`, and `audit_logs` (storing timestamp, package name, install method, freed bytes, and deleted file paths JSON). |
| [`src/db/repository.rs`](src/db/repository.rs) | Provides methods to store manifests, record uninstallation actions with detailed file path lists, and compute cumulative disk space recovered. |

---

### 5. User Interface (`src/ui/`)

Desktop interface built on Rust GUI framework `Iced` (0.14) with hardware-accelerated WGPU backend, responsive layout primitives, custom capsule scrollbars, and a bespoke design system.

| File | Component / Area | Responsibility |
| :--- | :--- | :--- |
| [`src/ui/app.rs`](src/ui/app.rs) | `SweepXApp` | Main application state loop, asynchronous Tokio channel receiver, window lifecycle, and modal stack. |
| [`src/ui/theme.rs`](src/ui/theme.rs) | `Theme` | Brand design tokens (Deep Burgundy `#38121C` canvas and modals, Warm Cream `#EDE5D2` cards and text, Pistachio `#9ECB82` action elements), elevations, typography, and button styling. |
| [`src/ui/mod.rs`](src/ui/mod.rs) | Exports & Helpers | Re-exports UI types, sorting and filtering dispatchers, and public in-memory window icon decoder `get_window_icon()`. |

#### View Architecture (inside `src/ui/app.rs`):
* `view_applications_dashboard`: Application inventory table, packaging tier filter pills, search input, sort column headers, and deterministic storage footprint summary cards derived strictly from package payloads.
* `view_inspector_overlay`: Detailed inspection modal displaying installed application payload, discovered user residual files, and total combined footprint.
* `view_clean_overlay`: Uninstallation checklist with a selectable primary package payload row, individual residual item checkboxes, safety validation summary, and permanent versus trash deletion modes.
* `view_system_sweep`: System optimizer for disabled Snap revisions, unused Flatpak runtimes, and host package manager caches.
* `view_audit_history`: Audit log timeline displaying uninstalled applications, timestamps, and lifetime storage recovery statistics.
* `view_audit_inspector_overlay`: Searchable modal for inspecting the exact file paths deleted during past package purges or system sweeps, with one-click path copying.

#### Window and Desktop Integration:
* **Wayland & Overview Icon Tracking**: Window settings configure `platform_specific.application_id = "sweepx"` and supply an in-memory decoded 512x512 RGBA `Icon` buffer. Desktop entries define `StartupWMClass=sweepx` and `StartupNotify=true`, ensuring Wayland compositors (GNOME Mutter, KDE KWin) associate the window with the official brand icon.
* **Unit System Discipline**: Implements `SizeUnitSystem` (IEC binary `GiB` default, SI decimal `GB`) with an interactive header toggle and pure total formatting function `format_size_with_unit`.

For complete visual identity specifications, vector marks, and typography guidelines, see **[Brand Guidelines](assets/logo/BRAND_GUIDELINES.md)**.

---

### 6. CLI Commands (`src/cli/`)

Headless command-line interface for terminal workflows and scripts.

| File | Purpose |
| :--- | :--- |
| [`src/cli/commands.rs`](src/cli/commands.rs) | Subcommands defined with `clap`: `list`, `inspect`, `clean`, `purge`, `sweep`, `watch`, `update`, `history`, and `install-assets`. |
| [`src/cli/mod.rs`](src/cli/mod.rs) | Command execution, terminal tables, JSON output mode, and progress indicators. |

---

### 7. In-Place Self-Updater (`src/updater/`)

| File | Purpose |
| :--- | :--- |
| [`src/updater/mod.rs`](src/updater/mod.rs) | Checks GitHub Releases for new versions, downloads assets, extracts tarballs, and atomically replaces the running binary. Manages desktop launcher and icon assets (`install_assets_to`), writes scalable SVG icons, removes legacy raster PNGs, repairs broken user hicolor themes, and triggers safe desktop cache invalidations. |

---

## 3. End-to-End Data Flow

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant UI as GUI / CLI
    participant Orch as Scanner Orchestrator
    participant PM as Package Managers and Filesystem
    participant Clean as Cleaner and Safety Barrier
    participant DB as SQLite DB

    User->>UI: Launch App or Run Command
    UI->>Orch: trigger_scan()
    par Concurrently query all tiers
        Orch->>PM: Scan native packages (dpkg / pacman / rpm)
        Orch->>PM: Scan Flatpak and Snap runtimes
        Orch->>PM: Scan Bottles Wine prefixes
        Orch->>PM: Scan /opt, user binaries, AppImages, and .desktop files
    end
    PM-->>Orch: Return raw application records
    Orch-->>UI: Return unified, deduplicated application inventory

    User->>UI: Select app to inspect or clean
    UI->>Clean: discover_residuals_for_app(app)
    Clean-->>UI: Return discovered residuals (~/.config, ~/.cache, ~/.var/app)

    User->>UI: Confirm Deletion
    UI->>Clean: execute_clean(selected_residuals)
    Clean->>Clean: validate_safety_blacklist(path)
    alt Path violates safety gate
        Clean-->>UI: Abort and return SafetyViolation error
    else Path is safe
        Clean->>PM: Move to Trash or unlink permanently
        Clean->>DB: Record freed space and deleted paths in audit_logs
        Clean-->>UI: Return success and update space metrics
    end
```

---

## 4. Contributing Guide

### Adding a new package manager scanner:
1. Create `src/scanner/native/<name>.rs` implementing package discovery.
2. Register the scanner in [`src/scanner/orchestrator.rs`](src/scanner/orchestrator.rs).
3. Add a corresponding test file in `tests/`.

### Adding a new system optimizer rule:
1. Add detection logic in [`src/cleaner/system_sweep.rs`](src/cleaner/system_sweep.rs).
2. Wire the rule into `scan_all_sweep_items()` so both the GUI Optimizer tab and `sweepx sweep` CLI pick it up automatically.

---

*Dual-licensed under MIT and Apache-2.0. Copyright (c) Haider Ali Tariq and SweepX Contributors.*
