# SweepX — Brand Logo Guidelines

> Official visual identity and asset specification for SweepX: Unified Linux Application Tracker & Deep Purge Engine.

---

## 1. The Logo
- **Mark**: The Orbital Reticle (Tier Scanner)
- **Concept**: Two concentric segmented orbital rings representing multi-tiered Linux runtime ecosystems (Native DNF/APT, Flatpak, Snap, Container artifacts) sliced by an active 45° radar sweep beam in vibrant Pistachio, centered around an optical detection core.
- **Master Files**: Located in [`assets/logo/kit/`](file:///home/haider/Desktop/MyGithub/SweepX/assets/logo/kit/)
  - `sweepx-symbol.svg` (Color symbol on light surfaces)
  - `sweepx-symbol-dark.svg` (Color symbol on dark surfaces)
  - `sweepx-symbol-black.svg` (1-color solid black)
  - `sweepx-symbol-white.svg` (1-color solid white)
  - `sweepx-symbol-small.svg` (Simplified small-cut / favicon)
  - `sweepx-app-icon.svg` & `sweepx-app-icon.png` (Desktop & launcher icon)
  - `sweepx-lockup-horizontal.svg` & `sweepx-lockup-dark.svg` (Official lockups)
  - `sweepx-lockup-banner.svg` & `sweepx-lockup-banner.png` (README / Hero banner)
  - `favicon.ico`, `apple-touch-icon.png`, `site.webmanifest`

---

## 2. Clear Space
Keep a clear zone of **1 × [X]** around the logo on all sides, where **[X]** equals the width of the central core lens (radius = 20, diameter = 40 units on a 256 canvas, or ~16% of symbol width). No text, borders, or interface elements may encroach upon this exclusion zone.

---

## 3. Minimum Reproduction Sizes

| Version | Screen Dimension | Target Context |
|---|---|---|
| **Horizontal Lockup** | 120 px wide minimum | Header bars, README badges, dialog headers |
| **Full Symbol** | 32 px minimum | App window icon, navigation sidebars |
| **Small-Cut Symbol (`-small`)** | 16 px | Browser tab favicon, system tray, status bar |
| **Desktop App Icon** | 48 px – 512 px | Linux Application Menu (`.desktop`), GNOME Dash, AppStream |

---

## 4. Brand Color Palette

| Role | Color Name | HEX | RGB | Use Case |
|---|---|---|---|---|
| **Canvas / Background** | Deep Burgundy | `#38121c` | `rgb(56, 18, 28)` | Dark UI canvas, primary brand background, dark-mode ink |
| **Card / Surface** | Rich Wine | `#421622` | `rgb(66, 22, 34)` | Elevated cards, modal dialogs, squircle app-icon boundary |
| **Text / Ink** | Warm Cream | `#ede5d2` | `rgb(237, 229, 210)` | Wordmark "Sweep", primary headings, high-contrast dark-mode surfaces |
| **Kinetic Accent** | Pistachio Green | `#9ecb82` | `rgb(158, 203, 130)` | Radar sweep beam, wordmark "X", active purge buttons |
| **Hover / Apex** | Light Pistachio | `#b5dca0` | `rgb(181, 220, 160)` | Interactive focus states, hover glow, terminal highlights |

**Approved Combinations**:
- **Dark Mode (Default)**: Warm Cream + Pistachio mark on Deep Burgundy (`#38121c`).
- **Light / Print**: Deep Burgundy + Pistachio mark on Warm Cream or pure white.
- **Monochrome**: 100% Black on white, or 100% White on dark surfaces.

---

## 5. Typography
- **Wordmark**: Hand-crafted geometric sans with negative tracking (`-1px`), converted into pure vector bezier paths.
- **Application Interface Font**: Inter / -apple-system / system-ui (fallback to Cantarell on Linux desktops).

---

## 6. Usage Rules & Don'ts
- **DO NOT** rotate the radar sweep beam away from its 45-degree axis.
- **DO NOT** recolor the mark with unapproved rainbow or neon gradients.
- **DO NOT** squash, stretch, or alter the aspect ratio.
- **DO NOT** separate the concentric rings from the core lens in official lockups.
- **DO NOT** add drop shadows, outer glows, or bevel filters directly to the SVG masters.
