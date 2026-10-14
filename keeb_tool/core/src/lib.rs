//! keeb_core: parse / edit / generate / analyze KMK keymaps.
//!
//! The crate is deliberately UI-agnostic so it can be unit-tested on its own
//! and reused from the Tauri backend (see `../../src-tauri`).
//!
//! Pipeline:
//!   raw main.py  --parse-->  Keymap  --edit-->  Keymap  --generate-->  main.py
//!
//! The parser is *surgical*: it splits `main.py` into `preamble`,
//! the `keyboard.keymap = [...]` block, and `postamble`. Only the keymap block
//! is ever regenerated; everything else (modules, OLED, RGB, tap-dance helper
//! definitions, ...) is preserved byte-for-byte.

pub mod model;
pub mod keys;
pub mod parser;
pub mod generator;
pub mod layout;
pub mod ferris;
pub mod flash;

pub use ferris::{suggest_compact_layout, CompactSuggestion, TargetBoard};
pub use generator::generate_main_py;
pub use model::{Key, Keymap, Layer};
pub use parser::parse_main_py;
