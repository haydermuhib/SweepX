---
name: verify-sweepx
description: Drive, verify, and capture evidence for SweepX CLI, discovery scanners, purge engine, and Iced GUI. Use for testing SweepX features, proving changes, or running verification checks.
---

# Verify SweepX

This skill provides a deterministic, repeatable harness to drive SweepX, exercise its CLI and GUI capabilities, and capture undeniable evidence of functionality.

## 1. Launch & Doctor

Before running any feature check, verify the environment:

```bash
# Doctor check: ensure binary compiles cleanly and responds
cargo check --bin sweepx
cargo build --bin sweepx
./target/debug/sweepx --version
```

If the exit code is non-zero, fix compilation before running feature drives.

## 2. Drive Recipes

### A. Application Discovery & Listing (CLI)
Verify that SweepX can discover installed system, flatpak, snap, or desktop applications:
```bash
./target/debug/sweepx list
```

### B. Application Inspection
Verify discovery of binary paths, launchers, and residual candidates:
```bash
# Scan a specific installed application
./target/debug/sweepx scan firefox || ./target/debug/sweepx list
```

### C. System Sweep & Space Reclaimer (Dry Run)
Verify scanner for unused packages, orphaned dependencies, and caches without deleting files:
```bash
./target/debug/sweepx system-sweep --dry-run
```

### D. Audit History Verification
Verify SQLite tamper-evident logging:
```bash
./target/debug/sweepx history
```

### E. GUI Smoke Test
Launch the Iced GUI and verify successful startup:
```bash
cargo run --bin sweepx
```

## 3. Evidence Collection

All proof artifacts should be collected with exit codes, outputs, and timestamps:
* **Terminal Transcripts**: Capture command exit codes and formatted outputs.
* **Audit State**: Assert records written to SQLite database (`~/.local/share/sweepx/sweepx.db` or test DB).
* **Test Suite**: Run `cargo test` to execute all 30 unit, integration, and safety tests.

## 4. Cleanup

Ensure no background instances or temporary test files linger:
```bash
# Kill any lingering sweepx processes started by the test harness
pkill -f "target/debug/sweepx" || true
```

## 5. Feature Map

Detailed driving instructions per feature are located in `features/`:
- [`features/cli-discovery.md`](features/cli-discovery.md)
- [`features/cli-purge.md`](features/cli-purge.md)
- [`features/system-sweep.md`](features/system-sweep.md)
- [`features/audit-history.md`](features/audit-history.md)
- [`features/gui-dashboard.md`](features/gui-dashboard.md)
