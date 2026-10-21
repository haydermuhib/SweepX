# Feature: CLI Application Discovery

## Sub-features
- Native RPM / DPKG / Pacman installed application discovery
- Flatpak application discovery via `flatpak list --app`
- Snap application discovery via `snap list`
- Bottles Wine prefix discovery
- Desktop Entry `.desktop` parser across standard XDG paths

## How to get to it (user POV)
Run from any terminal:
```bash
sweepx list
```

## Driving it with harness
```bash
./target/debug/sweepx list
```

## Observable End State
- Table output displays application Name, Package ID, Packaging Type, and Total Payload Size.
- Exit code must be `0`.
- System libraries are hidden by default unless explicitly requested.

## Gotchas
- When testing in sandboxed CI environments where flatpak/snapd are not daemonized, external commands fail gracefully without breaking native discovery.
