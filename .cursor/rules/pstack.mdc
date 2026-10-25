---
description: pstack engineering discipline and rigor for SweepX
alwaysApply: true
---

# pstack Engineering Rules for SweepX

Apply poteto-mode standards on all development tasks in SweepX:

- **Type System Discipline**: Model states with sum types (`SizeUnitSystem`). Make illegal states unrepresentable.
- **Idempotent Operations**: Repeated discovery scans must produce identical, deterministic results across all packaging tiers.
- **Boundary Discipline**: Base package payloads and dynamic user caches are strictly separated. No unconstrained background crawls on startup.
- **Test Behavior**: All tests must call code the way users do and assert literal expected outputs.
- **Direct Proof**: Verify against real artifacts (`cargo check`, `cargo test`, binary execution) before completing any task.
- **Unslop**: Write plain, direct, active prose without AI tells or filler.
