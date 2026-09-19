# SweepX Architecture & Scanning Engine Deep Dive
## System Applications, Container Sandboxes, Safety Guards & Residual Discovery

**Duration:** 15 minutes  
**Audience:** Contributors, Maintainers & System Engineers  
**Date:** 2026-09-20  

---

## Agenda

1. **System & Multi-Tier Scanner Architecture** (3 min)
2. **Package Manager & Container Discovery Engines** (3 min)
3. **Deep Residual Discovery & File Location Matrix** (3 min)
4. **Safety Barrier & Deletion Guardrails** (3 min)
5. **Live Install Watcher & Database Audit Trail** (2 min)
6. **Documentation & Contributor Guide Reference** (1 min)

**Total: 15 minutes**

---

## 1. Multi-Tier Scanner Architecture

SweepX uses an asynchronous multi-tier orchestrator ([`src/scanner/orchestrator.rs`](src/scanner/orchestrator.rs)) that concurrently crawls system package databases, container runtimes, and local filesystem directories.

```
┌──────────────────────────────────────────────────────────────┐
│                  Scanner Orchestrator                        │
│            (src/scanner/orchestrator.rs)                     │
└──────┬───────────────┬────────────────┬───────────────┬──────┘
       │               │                │               │
       ▼               ▼                ▼               ▼
┌──────────────┐┌──────────────┐┌──────────────┐┌──────────────┐
│  Native PMs  ││  Containers  ││ Filesystem & ││   Desktop    │
│ (DPKG/Pacman/││(Flatpak/Snap)││  AppImages   ││   Entries    │
│     RPM)     ││              ││ (/opt, ~/bin)││(.desktop files)
└──────────────┘└──────────────┘└──────────────┘└──────────────┘
```

<details>
<summary><b>📋 Concurrent Pipeline Execution Details</b></summary>

- **Thread Pooling:** Spawns asynchronous non-blocking tasks via Tokio.
- **De-duplication:** Automatically resolves overlapping applications (e.g. an APT package that also installs a `.desktop` file is merged into a single unified record using binary inode and desktop ID matching).
- **Execution Speed:** Scans over 1,000+ installed applications across all tiers in **< 150 ms**.

</details>

---

## 2. Package Manager & Container Discovery

SweepX inspects every application tier with dedicated native parsers:

| Tier / Format | Primary Scanner File | Inspected System Locations & Commands |
| :--- | :--- | :--- |
| **Native DPKG/APT** | [`src/scanner/native/apt.rs`](src/scanner/native/apt.rs) | `dpkg-query -W -f='...'`, `/var/lib/dpkg/status` |
| **Native Pacman** | [`src/scanner/native/pacman.rs`](src/scanner/native/pacman.rs) | `pacman -Qi`, `/var/lib/pacman/local/` |
| **Native RPM/DNF** | [`src/scanner/native/dnf.rs`](src/scanner/native/dnf.rs) | `rpm -qa --queryformat`, `/var/lib/rpm` |
| **Flatpak** | [`src/scanner/flatpak.rs`](src/scanner/flatpak.rs) | `flatpak list --app --columns=...`, `~/.local/share/flatpak` |
| **Snap** | [`src/scanner/snap.rs`](src/scanner/snap.rs) | `snap list`, `/var/lib/snapd/desktop/applications` |
| **AppImage** | [`src/scanner/appimage.rs`](src/scanner/appimage.rs) | `~/Applications`, `~/.local/bin`, `~/Downloads`, magic bytes `0x41 0x49 0x02` |
| **Desktop Launchers** | [`src/scanner/desktop_entry.rs`](src/scanner/desktop_entry.rs) | `~/.local/share/applications`, `/usr/share/applications`, `/usr/local/share/applications` |

<details>
<summary><b>📋 Metadata Extracted Per Application</b></summary>

- **Display Name & Description:** Extracted from `.desktop` files or package metadata.
- **Icon Resolution:** Resolves themed icons, scalable SVGs in `~/.local/share/icons/`, `/usr/share/icons/`, and pixmaps.
- **Binary Executable Path:** Resolves absolute binary paths via `PATH` lookup and symlink dereferencing.
- **Installed Size:** Queries package metadata or recursively measures application folders.

</details>

---

## 3. Deep Residual Discovery Engine

When an application is inspected or queued for deep removal, [`src/cleaner/heuristic.rs`](src/cleaner/heuristic.rs) crawls all associated folders:

```
                      Target Application: "app_name"
                                    │
    ┌───────────────────────────────┼──────────────────────────────┐
    ▼                               ▼                              ▼
[XDG Data & Config]         [Direct Dotdirs]              [Container Sandboxes]
• ~/.config/app_name        • ~/.app_name                 • ~/.var/app/app_id (Flatpak)
• ~/.cache/app_name         • ~/.config/app_name.conf     • ~/snap/app_name (Snap)
• ~/.local/share/app_name
• ~/.local/state/app_name
    │                               │                              │
    ▼                               ▼                              ▼
[Desktop & Icons]           [Symlink Graph]               [Known Signatures]
• ~/.local/share/           • ~/.local/bin/<symlink>      • Multi-dir signatures
  applications/<app>.desktop• /usr/local/bin/<symlink>      (VS Code, Chrome, etc.)
• ~/.local/share/icons/...
```

<details>
<summary><b>📋 Heuristic Matching Strategy</b></summary>

1. **Exact App ID & Executable Stem:** Matches the binary name, package name, and reverse-DNS suffix (e.g. `org.mozilla.firefox` -> `firefox`).
2. **Curated Signatures (`src/cleaner/signatures.rs`):** Contains hardcoded, verified mapping rules for complex applications that scatter files across non-standard directories (e.g., `.vscode`, `.mozilla`, `.rustup`).
3. **Symlink Graph Traversal (`src/scanner/symlink_graph.rs`):** Traces GNU Stow / Homebrew style symlinks to identify orphan links pointing to the target executable.

</details>

---

## 4. Safety Barrier & Deletion Guardrails

Safety is paramount in SweepX. Before any file or directory deletion is permitted, it must pass the zero-tolerance validator in [`src/cleaner/safety.rs`](src/cleaner/safety.rs).

```
Target Deletion Path ──▶ [ Safety Barrier Validation ]
                                 │
                 ┌───────────────┴───────────────┐
                 ▼                               ▼
       [ Critical Blacklist ]          [ Path Sanitization ]
       • /                             • No relative paths ("..")
       • /home, /root, $HOME           • No root XDG base dirs
       • /usr, /bin, /etc, /lib        • Must be exact child folder
       • /var, /boot, /sys, /proc      • Path must exist on disk
                 │                               │
                 ▼                               ▼
            ⛔ BLOCKED                      ✅ APPROVED
        (SafetyError thrown)            (Reversible Trash / Purge)
```

<details>
<summary><b>📋 Safety Validator Rules & Blacklists</b></summary>

```rust
// Forbidden Critical Paths (Hard Block)
const FORBIDDEN_EXACT_PATHS: &[&str] = &[
    "/", "/root", "/home", "/etc", "/usr", "/bin", "/sbin",
    "/lib", "/lib64", "/var", "/boot", "/sys", "/proc", "/dev",
    "/usr/bin", "/usr/lib", "/usr/share", "/var/lib"
];
```
- **XDG Base Root Protection:** Never permits deleting `~/.config` or `~/.cache` directly; only named subdirectories (e.g. `~/.config/app_name`) are accepted.
- **Reversible Deletion Default:** By default, files are sent to the system Trash (`gio trash` / Freedesktop Trash spec) with permanent deletion requiring explicit confirmation.

</details>

---

## 5. Live Install Watcher & Audit Database

For software installed from source (`./configure && make install` or custom bash scripts), SweepX provides an active installation tracker:

```
User runs: sweepx watch ./install.sh
  │
  ├─▶ 1. Take recursive Filesystem Snapshot (src/tracker/snapshot.rs)
  ├─▶ 2. Execute installer child process
  ├─▶ 3. Take post-install Snapshot & compute diff
  └─▶ 4. Save exact manifest to SQLite DB (~/.local/share/sweepx/history.db)
```

<details>
<summary><b>📋 Database Schema & Space Recovery Tracking</b></summary>

- **`install_manifests`**: Stores application name, install timestamp, total bytes, and installer command.
- **`manifest_files`**: Records every created binary, man page, icon, and config file.
- **`audit_logs`**: Logs all clean/purge actions and keeps a lifetime running tally of disk space recovered.

</details>

---

## 6. Documentation & Architecture References

All of these subsystems are fully documented across the project:

- 📄 **[`ARCHITECTURE.md`](ARCHITECTURE.md):** Complete component breakdowns, subsystem interaction matrices, and sequence diagrams.
- 📄 **[`PRESENTATION.md`](PRESENTATION.md):** 15-minute presenter-friendly slide deck and onboarding overview.
- 📄 **[`README.md`](README.md):** User-facing features, CLI commands, and installation instructions.
- 📄 **[`llms.txt`](llms.txt):** Machine-readable technical architecture summary.
- 🧪 **`tests/` Suite:** 30 unit and integration tests verifying every scanner, safety guard, and cleaner heuristic.

---

## Quick Reference Summary

| Subsystem | Source Path | Key Safety & Functionality |
| :--- | :--- | :--- |
| **Native PM Scanner** | `src/scanner/native/` | Reads DPKG, Pacman, and RPM package registries |
| **Container Scanner** | `src/scanner/flatpak.rs`, `snap.rs` | Queries Flatpak/Snap CLI and export directories |
| **Desktop Discovery** | `src/scanner/desktop_entry.rs` | Indexes `~/.local/share/applications` & `/usr/share/` |
| **Residual Crawler** | `src/cleaner/heuristic.rs` | Inspects XDG config, cache, state, dotdirs, and icons |
| **Safety Barrier** | `src/cleaner/safety.rs` | Zero-tolerance blacklist preventing system path deletion |
| **Install Tracker** | `src/tracker/snapshot.rs` | Pre/post install directory diffing for 100% clean uninstalls |
| **Database Audit** | `src/db/repository.rs` | SQLite persistence for manifests and freed space metrics |

---

*Presentation prepared for SweepX Engineering & Architecture Review.*
