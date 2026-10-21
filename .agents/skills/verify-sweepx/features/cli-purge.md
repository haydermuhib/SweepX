# Feature: Deep Purge & Safety Boundaries

## Sub-features
- Safety validation boundary (`SafetyValidator::validate_deletion_target`)
- Refuses relative paths, empty paths, system roots, and home root
- Uninstalls package via package manager (Flatpak / Snap / native)
- Safely moves discovered user caches/configs to trash or permanent delete
- Logs audit trail entry to SQLite

## How to get to it (user POV)
```bash
sweepx purge <application_id>
```

## Driving it with harness
Verify safety invariants using unit and integration test harnesses:
```bash
cargo test --test safety_test
cargo test --test executor_test
cargo test --test e2e_workflow_test
```

## Observable End State
- Dangerous paths (`/`, `/usr`, `/home/user`, `~/.config`) are unconditionally rejected.
- Audit history table records uninstalled application name, timestamp, and freed bytes.
