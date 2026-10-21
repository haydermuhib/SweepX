# Feature: Desktop GUI Dashboard

## Sub-features
- Pure Rust Iced 0.14 graphical desktop interface
- Responsive search bar and packaging category filters
- Application Inspector modal with size accounting and selectable text
- Deep Purge modal with residual path checklist
- System Sweep optimizer tab with selective item check
- Purge & sweep audit history viewer

## How to get to it (user POV)
Launch from desktop application launcher or terminal:
```bash
sweepx
# or
sweepx gui
```

## Driving it with harness
```bash
# Doctor check
cargo check --bin sweepx

# Test launch and window creation
cargo run --bin sweepx
```

## Observable End State
- Application boots without cosmic-text panics.
- Brand triad palette (Wine `#4a1a22`, Sage `#827e59`, Cream `#ede5d2`) is applied across all cards.
- Text selection and copying functions seamlessly on all displayed paths and package IDs.
