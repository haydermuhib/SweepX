# SweepX Architecture and Scanning Engine Deep Dive
## Multi-Tier Packaging, Container Sandboxes, Safety Barriers, and Footprint Accounting

**Duration:** 15 minutes  
**Audience:** Contributors, Maintainers, and Linux Systems Engineers  
**Date:** 2026-09-23  

---

## Agenda

1. Multi-Tier Scanner Architecture (3 min)
2. Package Manager and Container Discovery (3 min)
3. Residual Discovery Engine and Isolation Rules (3 min)
4. Safety Barrier and Deletion Guardrails (3 min)
5. Installation Watcher and SQLite Audit History (2 min)
6. Contributing and Subsystem Reference (1 min)

**Total: 15 minutes**

---

## 1. Multi-Tier Scanner Architecture

SweepX uses an asynchronous orchestrator ([`src/scanner/orchestrator.rs`](src/scanner/orchestrator.rs)) that concurrently queries system package databases, container runtimes, and local filesystem directories.

```
┌──────────────────────────────────────────────────────────────┐
│                  Scanner Orchestrator                        │
│            (src/scanner/orchestrator.rs)                     │
└──────┬───────────────┬────────────────┬───────────────┬──────┘
       │               │                │               │
       ▼               ▼                ▼               ▼
┌──────────────┐┌──────────────┐┌──────────────┐┌──────────────┐
│  Native PMs  ││  Containers  ││ Filesystem   ││   Desktop    │
│ (DPKG/Pacman/││(Flatpak/Snap)││  AppImages   ││   Entries    │
│     RPM)     ││              ││ (/opt, ~/bin)││(.desktop files)
└──────────────┘└──────────────┘└──────────────┘└──────────────┘
```

<details>
<summary><b>Pipeline Execution Details</b></summary>

- Asynchronous execution: Spawns non-blocking tasks via Tokio.
- Native deduplication: Merges `.desktop` launchers with underlying native packages using binary paths, package IDs, and name matching.
- Scan performance: Indexes 1,000+ installed applications across all tiers in under 150 ms.

</details>

---

## 2. Package Manager and Container Discovery

SweepX discovers installed software across multiple packaging systems:

| Packaging Format | Primary Scanner File | Inspected Locations and Commands |
| :--- | :--- | :--- |
| **Native DPKG/APT** | [`src/scanner/native/apt.rs`](src/scanner/native/apt.rs) | `dpkg-query -W -f='...'`, `/var/lib/dpkg/status` |
| **Native Pacman** | [`src/scanner/native/pacman.rs`](src/scanner/native/pacman.rs) | `pacman -Qi`, `/var/lib/pacman/local/` |
| **Native RPM/DNF** | [`src/scanner/native/dnf.rs`](src/scanner/native/dnf.rs) | `rpm -qa --queryformat`, `/var/lib/rpm` |
| **Flatpak** | [`src/scanner/flatpak.rs`](src/scanner/flatpak.rs) | `flatpak list --app --columns=...`, `~/.local/share/flatpak` |
| **Snap** | [`src/scanner/snap.rs`](src/scanner/snap.rs) | `snap list`, `/var/lib/snapd/desktop/applications` |
| **AppImage** | [`src/scanner/appimage.rs`](src/scanner/appimage.rs) | `~/Applications`, `~/.local/bin`, `~/bin`, magic bytes `0x41 0x49 0x02` |
| **Manual /opt** | [`src/scanner/manual.rs`](src/scanner/manual.rs) | `/opt/*`, `~/.local/bin/*` with `rpm -qf`, `dpkg -S`, or `pacman -Qo` ownership checks |
| **Desktop Launchers** | [`src/scanner/desktop_entry.rs`](src/scanner/desktop_entry.rs) | `~/.local/share/applications`, `/usr/share/applications` |

<details>
<summary><b>Metadata Extracted Per Application</b></summary>

- Application Name and Description: Read from package headers or `.desktop` files.
- Icon Paths: Resolves system icons, scalable SVGs in `~/.local/share/icons/`, and pixmaps.
- Binary Location: Resolves executable paths through `PATH` lookups and symlink dereferencing.
- Storage Footprint: Measures package manager reported size and physical payload directory size.

</details>

---

## 3. Residual Discovery Engine and Container Isolation

When an application is inspected or queued for cleaning, [`src/cleaner/heuristic.rs`](src/cleaner/heuristic.rs) crawls associated folders while enforcing container isolation:

```
                      Target Application: "app_name"
                                    │
    ┌───────────────────────────────┼──────────────────────────────┐
    ▼                               ▼                              ▼
[XDG Data and Config]       [Direct Dotdirs]              [Container Sandboxes]
• ~/.config/app_name        • ~/.app_name                 • ~/.var/app/app_id (Flatpak)
• ~/.cache/app_name         • ~/.config/app_name.conf     • ~/snap/app_name (Snap)
• ~/.local/share/app_name
• ~/.local/state/app_name
    │                               │                              │
    ▼                               ▼                              ▼
[Desktop Launchers]         [Symlink Graph]               [Known Signatures]
• ~/.local/share/           • ~/.local/bin/<symlink>      • Multi-dir signatures
  applications/<app>.desktop• /usr/local/bin/<symlink>      (VS Code, Chrome, Firefox)
• ~/.local/share/icons/...
```

<details>
<summary><b>Container Isolation and Anchor Firewall</b></summary>

- Container isolation: Flatpaks only discover `~/.var/app/<app_id>`. Snaps only discover `~/snap/<app_name>`.
- Shared repository protection: Shared directories like `~/.cache/flatpak` and `~/.local/share/flatpak` are never scanned as per-app residuals.
- Anchor firewall: Forbidden keywords (`flatpak`, `snap`, `systemd`, `usr`, `bin`, `lib`, `etc`) are blocked from being used as search anchors.
- System binary protection: Shared binaries (`/usr/bin/flatpak`, `/usr/bin/snap`, `/usr/bin/bash`) are never listed as deletion targets.

</details>

---

## 4. Safety Barrier and Deletion Guardrails

Before any path is deleted, it must pass the zero-tolerance validator in [`src/cleaner/safety.rs`](src/cleaner/safety.rs).

```
Target Deletion Path ──▶ [ Safety Barrier Validation ]
                                 │
                 ┌───────────────┴───────────────┐
                 ▼                               ▼
       [ Critical Blacklist ]          [ Path Sanitization ]
       • /                             • No relative paths ("..")
       • /home, /root, $HOME           • No base XDG roots
       • /usr, /bin, /etc, /lib        • Must be exact child folder
       • /var/lib/flatpak, /snap       • Path must exist on disk
                 │                               │
                 ▼                               ▼
            BLOCKED                         APPROVED
    (SafetyViolation returned)         (Trash or Permanent Unlink)
```

<details>
<summary><b>Protected Directory Rules</b></summary>

```rust
// Examples of protected paths blocked by SafetyValidator
const PROTECTED_SYSTEM_DIRS: &[&str] = &[
    "/", "/root", "/home", "/etc", "/usr", "/bin", "/sbin",
    "/lib", "/lib64", "/var", "/boot", "/sys", "/proc",
    "/usr/bin/flatpak", "/usr/bin/snap", "/var/lib/flatpak", "/var/lib/snapd"
];
```
- User Base Root Protection: Deleting `~/.config`, `~/.cache`, `~/.local/share/flatpak`, or `~/.local/share/icons` directly is blocked.
- Reversible Default: Files are sent to the desktop Trash by default (`gio trash`).

</details>

---

## 5. Transparent Footprint Accounting

SweepX computes storage footprints using exact addition in [`src/ui/views/inspector.rs`](src/ui/views/inspector.rs):

```
┌─────────────────────────────────────────────────────────────┐
│  📦 Application Payload (DNF / Flatpak / Snap / /opt)        │
│     Example: 433.7 MB (google-chrome-stable)                │
├─────────────────────────────────────────────────────────────┤
│  📂 Discovered User Artifacts & Sandboxes                   │
│     • Configuration (~/.config): 5.10 GB                    │
│     • Cache Data (~/.cache): 4.09 GB                        │
├─────────────────────────────────────────────────────────────┤
│  📊 Total Footprint = 433.7 MB + 5.10 GB + 4.09 GB = 9.62 GB│
└─────────────────────────────────────────────────────────────┘
```

---

## 6. Installation Watcher and SQLite Audit Trail

For source builds (`./configure && make install` or custom scripts), SweepX tracks file changes:

```
User runs: sweepx watch ./install.sh
  │
  ├─▶ 1. Take recursive filesystem snapshot (src/tracker/snapshot.rs)
  ├─▶ 2. Run installer child process
  ├─▶ 3. Take post-install snapshot and compute diff
  └─▶ 4. Save manifest to SQLite database (~/.local/share/sweepx/history.db)
```

---

## Subsystem Reference Summary

| Subsystem | Source Path | Responsibility |
| :--- | :--- | :--- |
| **Native PM Scanner** | `src/scanner/native/` | Queries DPKG, Pacman, and RPM package registries |
| **Container Scanner** | `src/scanner/flatpak.rs`, `snap.rs` | Queries Flatpak and Snap CLI and exports |
| **Manual Scanner** | `src/scanner/manual.rs` | Crawls `/opt` with package ownership verification |
| **Desktop Discovery** | `src/scanner/desktop_entry.rs` | Indexes `~/.local/share/applications` and `/usr/share/` |
| **Residual Crawler** | `src/cleaner/heuristic.rs` | Discovers XDG configs, caches, state, and sandboxes |
| **Safety Barrier** | `src/cleaner/safety.rs` | Blocks deletion of critical system and container paths |
| **Install Tracker** | `src/tracker/snapshot.rs` | Pre/post install directory diffing for clean removals |
| **Database Audit** | `src/db/repository.rs` | SQLite store for manifests and recovered space metrics |

---

*Prepared for SweepX Engineering and Architecture Review.*
