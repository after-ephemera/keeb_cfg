//! Data model shared across parsing, editing and generation.

use serde::{Deserialize, Serialize};

/// A single key entry in a layer.
///
/// `raw` is the exact Python expression as it appears (or should appear) in
/// `main.py`, e.g. `"KC.TAB"`, `"LALT"`, `"KC.LCTL(KC.LEFT)"`,
/// `"KC.TD(KC.HT(KC.TAB, KC.LGUI), KC.LGUI(KC.TAB))"`.
///
/// `label` is a short human-friendly glyph derived for display; it is never
/// written back to the file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    /// Exact Python expression, round-tripped verbatim.
    pub raw: String,
    /// Short display glyph (derived, not persisted).
    pub label: String,
    /// True when `raw` is a bare identifier defined in the preamble
    /// (e.g. `LOWER`, `F`, `SKGUI`) rather than a `KC.*` expression.
    pub is_alias: bool,
    /// True when this is a "transparent" key (`KC.TRNS`) or "no-op" (`KC.NO`).
    pub is_transparent: bool,
}

impl Key {
    pub fn new(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let trimmed = raw.trim().to_string();
        let is_transparent = trimmed == "KC.TRNS" || trimmed == "KC.NO";
        let is_alias = crate::keys::is_alias_token(&trimmed);
        let label = crate::keys::label_for(&trimmed);
        Key {
            raw: trimmed,
            label,
            is_alias,
            is_transparent,
        }
    }
}

/// A single layer: a flat, row-major list of keys in keymap order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layer {
    /// Optional human name pulled from the `# name` comment on the layer.
    pub name: Option<String>,
    pub keys: Vec<Key>,
}

/// A parsed keymap plus the surrounding file text needed to regenerate it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keymap {
    /// Everything in `main.py` before `keyboard.keymap = [`.
    pub preamble: String,
    /// Everything after the keymap's closing `]`.
    pub postamble: String,
    /// Names defined in the preamble (`LOWER = KC.TO(1)` -> "LOWER").
    pub aliases: Vec<String>,
    pub layers: Vec<Layer>,
}

impl Keymap {
    /// Total physical key count of the first (base) layer.
    pub fn key_count(&self) -> usize {
        self.layers.first().map(|l| l.keys.len()).unwrap_or(0)
    }
}
