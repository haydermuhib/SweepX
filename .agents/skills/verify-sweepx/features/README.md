# SweepX Feature Verification Map

This directory maps every major user-facing capability of SweepX with exact recipes to drive, inspect, and prove correctness.

| Feature Map | Target Surface | Driving Recipe |
| :--- | :--- | :--- |
| [CLI Discovery](cli-discovery.md) | Terminal / CLI | `sweepx list` |
| [Deep Purge & Safety](cli-purge.md) | Terminal / Safety Engine | `sweepx purge <id>` |
| [System Sweep](system-sweep.md) | Optimizer Engine | `sweepx system-sweep --dry-run` |
| [GUI Dashboard](gui-dashboard.md) | Iced 0.14 Desktop GUI | `sweepx` / `sweepx gui` |
