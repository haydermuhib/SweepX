# Feature: System Sweep & Storage Reclaimer

## Sub-features
- Disabled snap revision scanner
- Unused flatpak runtime detector
- Package manager cache cleaner (APT, DNF, Pacman)
- Orphaned package dependencies detection
- Broken and dangling symlinks detector

## How to get to it (user POV)
```bash
sweepx system-sweep
```

## Driving it with harness
Run non-destructive test:
```bash
cargo test --test system_sweep_test
./target/debug/sweepx system-sweep --dry-run
```

## Observable End State
- All reclaimable caches are inventoried with accurate byte sizes.
- Dry-run reports total reclaimable disk space without deleting any files.
