# Lily58 Config Tool

A desktop app (Rust + Tauri + Three.js) that reads your KMK `lily58/main.py`,
shows the keymap on a **3D model of the keyboard**, lets you **rebind keys**,
**flashes the config to the connected keyboard**, and **suggests how to fold the
layout onto a smaller 42/34-key board** like a Ferris/Sweep or Corne.

See [`EVALUATION.md`](./EVALUATION.md) for a review of the current config.

![panels: 3D board · key editor · flash · compact-board suggestion]

## What it does

| Feature | How |
| --- | --- |
| **3D keyboard** | Procedurally built in Three.js from the real Lily58 column-stagger geometry (no external model to download). Orbit/zoom, per-key labels, layer tabs, click-to-select. |
| **Edit bindings** | Click a key → pick from a grouped palette, or type any raw KMK expression (`KC.MT(KC.A, KC.LGUI)`, tap-dances, etc.). |
| **Preview** | "Preview .py" shows the exact regenerated `main.py` before you save. |
| **Flash** | Scans for a mounted `CIRCUITPY` volume and copies `main.py` to it — that's all KMK/CircuitPython flashing is; the board auto-reloads. A `.bak` of the previous file is kept. |
| **Compact-board suggestion** | Maps your base layer onto Ferris/Sweep (34) or Corne (42): shows the kept 3×N grid + thumbs, and lists every dropped key with a concrete relocation (home-row mods, layers, combos). |

## Architecture

```
keeb_tool/
├── core/         Rust library — the tested heart (no UI deps)
│   ├── parser.rs      surgical KMK main.py -> Keymap
│   ├── generator.rs   Keymap -> main.py (preamble/postamble preserved verbatim)
│   ├── model.rs       Key / Layer / Keymap
│   ├── keys.rs        label lookup + editor palette
│   ├── layout.rs      physical geometry (Lily58 slots) for the 3D view
│   ├── ferris.rs      compact-board suggestion engine
│   ├── flash.rs       CIRCUITPY detection + main.py copy
│   └── tests/         round-trip + suggestion tests against the REAL config
├── src-tauri/    Tauri shell — thin command wrappers over `core`
└── ui/           Vite + Three.js frontend
```

**Design choices worth knowing:**

- **The parser is surgical.** It splits `main.py` into `preamble`, the
  `keyboard.keymap = [...]` block, and `postamble`. Only the keymap block is ever
  regenerated, so every module, OLED/RGB setup, and tap-dance helper definition
  (`LGUIPP`, `F`, `LBRC`, …) survives byte-for-byte. Complex expressions like
  `KC.TD(KC.HT(KC.TAB, KC.LGUI), KC.LGUI(KC.TAB))` are kept as single opaque
  tokens (a paren-aware tokenizer never splits on their inner commas).
- **Aliases are first-class.** Bare identifiers defined in your preamble show up
  with friendly glyphs and round-trip untouched.
- **3D is generated, not downloaded.** A GLTF of a Lily58 can't tell the app
  which mesh is the `F` key; generating caps from the layout means every keycap
  is individually labelled, highlightable, and themeable, and it works offline.

## Build & run

Prerequisites: [Rust](https://rustup.rs), Node 18+, and the
[Tauri v2 system deps](https://v2.tauri.app/start/prerequisites/) for your OS
(on Linux: `webkit2gtk-4.1`, `libgtk-3-dev`, etc.).

```bash
cd keeb_tool/ui && npm install         # frontend deps (three, tauri api, vite)
cd .. && cargo install tauri-cli --version '^2.0.0'   # once
cargo tauri dev                        # launches the app
```

Then paste the path to your keymap (e.g. `.../keeb_cfg/lily58/main.py`) and
click **Load**.

To build a distributable bundle: `cargo tauri build`.

## Testing

The core logic is fully unit-tested against your actual config:

```bash
cd keeb_tool/core && cargo test
```

Covers: parsing all 8 layers / 58 keys, keeping nested expressions intact,
lossless round-trip (parse → generate → re-parse), single-key edits not
disturbing neighbors, and both the Ferris (34) and Corne (42) suggestions.

> Note: the Tauri GUI requires system WebKit/GTK libraries to compile, so it
> can't build in a headless CI container — build it on your desktop. The `core`
> crate has no such dependency and tests anywhere.
