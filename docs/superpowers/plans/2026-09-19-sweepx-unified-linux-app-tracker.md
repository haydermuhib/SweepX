# SweepX: Unified Linux Application Tracker & Deep Uninstaller Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a standalone, high-performance, modular Linux desktop application and CLI in Rust (`SweepX`) that discovers, tracks, inspects, and deeply purges applications across all packaging tiers (APT, DNF, Pacman, Flatpak, Snap, AppImage, and unmanaged `/opt` / `.desktop` / manual installations) with hybrid residual file scanning, safety guardrails, and FreeDesktop trash support.

**Architecture:** A multi-layered architecture featuring an asynchronous discovery and scanning engine (`tokio`), a pluggable package manager abstraction (`PackageManager` trait), a hybrid heuristic-and-signature residual cleaner with strict safety blacklists, an embedded SQLite state/audit logger (`rusqlite`), a CLI interface (`clap`), and a modern immediate-mode desktop UI (`eframe` / `egui`).

**Tech Stack:** Rust (2021 edition), `eframe` (0.28+), `egui`, `tokio`, `rusqlite`, `serde`, `serde_json`, `clap`, `trash`, `walkdir`, `which`, `regex`, `chrono`, `freedesktop-desktop-entry`.

**Spec:** Unified Linux Application Tracker & Deep Uninstaller Specification.

## Global Constraints
- Target platform: All Linux distributions (Debian/Ubuntu, Fedora/RHEL, Arch, openSUSE, etc.).
- Modular, cleanly decoupled crates/modules with zero hard-coded distro assumptions.
- Safety: Under NO circumstances may root parent directories (`/`, `/home`, `/usr`, `~`, `~/.config`, `~/.cache`, `/etc`, `/var`) be deleted. Must pass strict path validation.
- All background disk and command operations must run asynchronously without blocking the UI thread.
- FreeDesktop Trash deletion by default with explicit optional hard purge.

---

### Task 1: Project Initialization and Core Data Models

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/models/mod.rs`
- Create: `src/models/app.rs`
- Create: `src/models/artifact.rs`
- Create: `src/models/residual.rs`
- Test: `tests/models_test.rs`

**Interfaces:**
- Produces:
  - `enum InstallMethod { NativeApt, NativeDnf, NativePacman, Flatpak, Snap, AppImage, ManualOpt, CustomDesktop }`
  - `enum ArtifactKind { Binary, DesktopEntry, ConfigDir, CacheDir, DataDir, StateDir, LogDir, SystemdService, AppDir }`
  - `struct AppArtifact { kind: ArtifactKind, path: PathBuf, size_bytes: u64, exists: bool, is_protected: bool }`
  - `struct Application { id: String, name: String, display_name: String, version: Option<String>, install_method: InstallMethod, exec_path: Option<PathBuf>, icon: Option<String>, desktop_file: Option<PathBuf>, install_date: Option<DateTime<Utc>>, total_size_bytes: u64, artifacts: Vec<AppArtifact>, dependencies: Vec<String>, is_system: bool }`
  - `struct ResidualCandidate { app_id: String, app_name: String, path: PathBuf, kind: ArtifactKind, size_bytes: u64, confidence: f32, is_orphaned: bool }`

- [ ] **Step 1: Create Cargo.toml with dependencies**
- [ ] **Step 2: Write failing unit tests for models serialization and methods**
- [ ] **Step 3: Implement data models in `src/models/`**
- [ ] **Step 4: Run `cargo test --test models_test` and verify all pass**
- [ ] **Step 5: Commit `feat: initialize Cargo workspace and core data models`**

---

### Task 2: Safety Engine & Path Sanity Guardrails

**Files:**
- Create: `src/cleaner/mod.rs`
- Create: `src/cleaner/safety.rs`
- Test: `tests/safety_test.rs`

**Interfaces:**
- Produces:
  - `pub fn is_path_safe_to_delete(path: &Path) -> Result<bool, SafetyViolation>`
  - `pub fn sanitize_deletion_targets(paths: &[PathBuf]) -> (Vec<PathBuf>, Vec<(PathBuf, SafetyViolation)>)`
  - `enum SafetyViolation { SystemRoot, UserHomeRoot, SystemDirectory, ProtectedXdgBase, EmptyPath, RelativePath, SymlinkEscape }`

- [ ] **Step 1: Write comprehensive test cases covering root `/`, `/home/user`, `/etc`, `~/.config`, symlink loops, and valid paths like `~/.config/slack`**
- [ ] **Step 2: Run tests and verify failure**
- [ ] **Step 3: Implement `SafetyValidator` with canonicalization, blacklist rules, and boundary checks**
- [ ] **Step 4: Verify all safety tests pass**
- [ ] **Step 5: Commit `feat: implement strict safety guardrails and deletion validator`**

---

### Task 3: Desktop Entry and Manual `/opt` / `AppImage` Scanners

**Files:**
- Create: `src/scanner/mod.rs`
- Create: `src/scanner/desktop_entry.rs`
- Create: `src/scanner/manual.rs`
- Test: `tests/desktop_scanner_test.rs`

**Interfaces:**
- Produces:
  - `pub async fn scan_desktop_entries() -> Vec<Application>`
  - `pub async fn scan_manual_installations() -> Vec<Application>`
  - `pub fn parse_desktop_entry(content: &str, file_path: &Path) -> Option<Application>`

- [ ] **Step 1: Write tests for parsing real-world `.desktop` files (XDG spec compliant, Exec splitting, Icon resolving)**
- [ ] **Step 2: Run tests to verify failure**
- [ ] **Step 3: Implement desktop entry parser and `/opt`, `/usr/local/bin`, `~/.local/bin`, `~/Applications` scanners**
- [ ] **Step 4: Run tests and verify passing**
- [ ] **Step 5: Commit `feat: implement desktop entry and manual installation scanner`**

---

### Task 4: Universal Container Inspectors (Flatpak & Snap)

**Files:**
- Create: `src/scanner/flatpak.rs`
- Create: `src/scanner/snap.rs`
- Test: `tests/containers_scanner_test.rs`

**Interfaces:**
- Produces:
  - `pub async fn scan_flatpaks() -> Vec<Application>`
  - `pub async fn scan_snaps() -> Vec<Application>`
  - `pub fn parse_flatpak_list_output(stdout: &str) -> Vec<Application>`
  - `pub fn parse_snap_list_output(stdout: &str) -> Vec<Application>`

- [ ] **Step 1: Write unit tests with sample outputs from `flatpak list --columns=...` and `snap list`**
- [ ] **Step 2: Run tests to verify failure**
- [ ] **Step 3: Implement Flatpak and Snap CLI output parsers, metadata queries, and sandbox path locators (`~/.var/app/` and `~/snap/`)**
- [ ] **Step 4: Run tests and verify parsing accuracy**
- [ ] **Step 5: Commit `feat: implement Flatpak and Snap container scanners`**

---

### Task 5: Native Package Manager Engine (APT, DNF, Pacman)

**Files:**
- Create: `src/scanner/native/mod.rs`
- Create: `src/scanner/native/apt.rs`
- Create: `src/scanner/native/dnf.rs`
- Create: `src/scanner/native/pacman.rs`
- Test: `tests/native_pm_test.rs`

**Interfaces:**
- Produces:
  - `pub trait PackageManager: Send + Sync { fn name(&self) -> &'static str; async fn is_available(&self) -> bool; async fn list_installed(&self) -> Vec<Application>; async fn get_package_files(&self, pkg: &str) -> Vec<PathBuf>; async fn get_dependencies(&self, pkg: &str) -> Vec<String>; }`
  - `pub fn detect_native_package_managers() -> Vec<Box<dyn PackageManager>>`

- [ ] **Step 1: Write unit tests for parsing `dpkg-query`, `rpm -qa`, and `pacman -Qi` outputs**
- [ ] **Step 2: Run tests to verify failure**
- [ ] **Step 3: Implement `AptManager`, `DnfManager`, and `PacmanManager` with async query execution**
- [ ] **Step 4: Run tests and verify passing**
- [ ] **Step 5: Commit `feat: implement multi-distro native package manager abstractions`**

---

### Task 6: Hybrid Heuristic & Pattern Residual Discovery Engine

**Files:**
- Create: `src/cleaner/signatures.rs`
- Create: `src/cleaner/heuristic.rs`
- Test: `tests/residual_cleaner_test.rs`

**Interfaces:**
- Produces:
  - `pub struct AppSignature { pub id: &'static str, pub names: &'static [&'static str], pub config_paths: &'static [&'static str], pub cache_paths: &'static [&'static str], pub data_paths: &'static [&'static str] }`
  - `pub async fn discover_residuals_for_app(app: &Application) -> Vec<ResidualCandidate>`
  - `pub async fn scan_all_orphaned_residuals(installed_apps: &[Application]) -> Vec<ResidualCandidate>`

- [ ] **Step 1: Write tests for signature and heuristic matching (testing VS Code, Firefox, Spotify, and generic unknown apps against mock XDG directory structures)**
- [ ] **Step 2: Run tests to verify failure**
- [ ] **Step 3: Implement signature patterns catalog and heuristic XDG directory crawler**
- [ ] **Step 4: Run tests and verify passing**
- [ ] **Step 5: Commit `feat: implement hybrid heuristic and signature residual discovery`**

---

### Task 7: Deep Cleaning Execution & Privilege Escalation Engine

**Files:**
- Create: `src/cleaner/executor.rs`
- Create: `src/cleaner/privilege.rs`
- Test: `tests/executor_test.rs`

**Interfaces:**
- Produces:
  - `pub enum DeletionMode { Trash, Permanent }`
  - `pub struct PurgeReport { pub success: bool, pub deleted_paths: Vec<PathBuf>, pub errors: Vec<(PathBuf, String)>, pub freed_bytes: u64 }`
  - `pub async fn execute_purge(app: &Application, residuals: &[ResidualCandidate], mode: DeletionMode) -> Result<PurgeReport, String>`
  - `pub async fn run_elevated_command(cmd: &str, args: &[&str]) -> Result<String, String>`

- [ ] **Step 1: Write unit tests for safe deletion (verifying temporary test directories are trashed/deleted and safety boundaries are strictly respected)**
- [ ] **Step 2: Run tests to verify failure**
- [ ] **Step 3: Implement `execute_purge`, FreeDesktop trash integration via `trash-rs`, and `pkexec` privilege runner with `sudo` fallback**
- [ ] **Step 4: Run tests and verify passing**
- [ ] **Step 5: Commit `feat: implement purge executor with trash support and pkexec escalation`**

---

### Task 8: Local State, History & Audit Logging (SQLite)

**Files:**
- Create: `src/db/mod.rs`
- Create: `src/db/schema.rs`
- Create: `src/db/repository.rs`
- Test: `tests/db_test.rs`

**Interfaces:**
- Produces:
  - `pub struct Database { ... }`
  - `impl Database { pub fn open_default() -> Result<Self>; pub fn save_apps(&self, apps: &[Application]) -> Result<()>; pub fn record_purge_audit(&self, app_id: &str, app_name: &str, method: &str, freed_bytes: u64, paths: &[PathBuf]) -> Result<()>; pub fn get_installed_history(&self) -> Result<Vec<AppHistoryRecord>>; }`

- [ ] **Step 1: Write tests for database initialization, schema migration, application caching, and audit logging**
- [ ] **Step 2: Run tests to verify failure**
- [ ] **Step 3: Implement SQLite database schema and repository operations**
- [ ] **Step 4: Run tests and verify passing**
- [ ] **Step 5: Commit `feat: implement SQLite local state cache and uninstallation audit history`**

---

### Task 9: Unified Scanner Orchestration & CLI Interface

**Files:**
- Create: `src/scanner/orchestrator.rs`
- Create: `src/cli/mod.rs`
- Create: `src/cli/commands.rs`
- Modify: `src/main.rs`
- Test: `tests/cli_test.rs`

**Interfaces:**
- Produces:
  - `pub async fn scan_all_applications() -> Vec<Application>`
  - `pub struct Cli { ... }` supporting: `list`, `inspect <app>`, `purge <app> [--dry-run] [--permanent]`, `residuals [--clean]`, `gui` (default)

- [ ] **Step 1: Write integration tests for CLI argument parsing and scanner orchestration**
- [ ] **Step 2: Implement unified concurrent scanner orchestrator with deduplication logic**
- [ ] **Step 3: Implement CLI commands with formatted tabular output (colored badges, human-readable byte sizes)**
- [ ] **Step 4: Run tests and verify CLI command execution**
- [ ] **Step 5: Commit `feat: implement unified scanner orchestrator and full CLI interface`**

---

### Task 10: Rich Modern Desktop GUI (`eframe` / `egui`)

**Files:**
- Create: `src/ui/mod.rs`
- Create: `src/ui/theme.rs`
- Create: `src/ui/app.rs`
- Create: `src/ui/views/dashboard.rs`
- Create: `src/ui/views/inspector.rs`
- Create: `src/ui/views/clean_modal.rs`
- Create: `src/ui/views/history.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces:
  - `pub struct SweepXApp { ... }` implementing `eframe::App`
  - Rich UI features:
    - Real-time search bar & package tier filter chips (All, Native, Flatpak, Snap, AppImage, Manual)
    - Storage and application metrics header
    - Application table with icon/origin badge, size, and actions
    - Application Inspector side drawer
    - Deep Clean modal with interactive file checklist, safety tags, and Trash toggle
    - Asynchronous scanning updates via `tokio::sync::mpsc`

- [ ] **Step 1: Implement custom dark theme, colors, and layout tokens in `src/ui/theme.rs`**
- [ ] **Step 2: Implement UI state bridge and async task channels in `src/ui/app.rs`**
- [ ] **Step 3: Implement Dashboard table, search, and category filters**
- [ ] **Step 4: Implement Application Inspector drawer and Deep Clean interactive modal**
- [ ] **Step 5: Connect main application entrypoint to launch GUI window**
- [ ] **Step 6: Run `cargo check` and `cargo test` to verify complete workspace compilation and test suite**
- [ ] **Step 7: Commit `feat: implement complete eframe/egui desktop GUI for SweepX`**

---

### Task 11: End-to-End Verification and Polish

**Files:**
- Modify: `README.md`
- Test: `tests/e2e_workflow_test.rs`

- [ ] **Step 1: Write end-to-end workflow test verifying discovery -> residual finding -> dry-run purge -> audit log**
- [ ] **Step 2: Run all tests in the workspace (`cargo test`)**
- [ ] **Step 3: Build release binary (`cargo build --release`)**
- [ ] **Step 4: Create comprehensive README with screenshots/feature breakdown and CLI/GUI usage**
- [ ] **Step 5: Commit `docs: finalize documentation and end-to-end verification`**
