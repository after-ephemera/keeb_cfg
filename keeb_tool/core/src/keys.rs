//! KMK key catalog: display labels and a curated palette for the editor.

use serde::{Deserialize, Serialize};

/// A selectable key offered in the editor's palette.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteKey {
    /// Python expression written to the file.
    pub code: String,
    /// Short glyph shown on the 3D keycap / palette button.
    pub label: String,
    /// Grouping for the palette UI.
    pub group: String,
}

/// Return true when `token` looks like a preamble-defined alias
/// (a bare identifier, not a `KC.*` expression or call).
pub fn is_alias_token(token: &str) -> bool {
    let t = token.trim();
    if t.is_empty() || t.starts_with("KC.") || t.contains('(') || t.contains('.') {
        return false;
    }
    // Bare identifier: leading alpha/underscore, rest alnum/underscore.
    let mut chars = t.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Derive a short display label for any key expression.
pub fn label_for(raw: &str) -> String {
    let t = raw.trim();

    if is_alias_token(t) {
        // Friendly names for the aliases used in this repo's config.
        return match t {
            "LOWER" => "▼ Lower".into(),
            "RAISE" => "▲ Raise".into(),
            "BASE" => "⌂ Base".into(),
            "SKGUI" => "⌘ (sticky)".into(),
            "DLFT" => "Desk◀".into(),
            "DRGT" => "Desk▶".into(),
            "LGUIPP" => "Tab/⌘".into(),
            "LCTLPP" => "Ctrl±".into(),
            "LBRC" => "[ {".into(),
            "RBRC" => "] }".into(),
            "LALT" => "+ / Alt".into(),
            "F" => "F/⌘".into(),
            "J" => "J/⌘".into(),
            other => other.into(),
        };
    }

    // Simple KC.* lookups.
    if let Some(rest) = t.strip_prefix("KC.") {
        // Complex expression like LCTL(...) / TD(...) / HT(...): summarize.
        if let Some(idx) = rest.find('(') {
            let head = &rest[..idx];
            return match head {
                "TD" => "TapDance".into(),
                "HT" => "Hold/Tap".into(),
                "MT" => "Mod/Tap".into(),
                "LT" => "Layer/Tap".into(),
                "TO" | "MO" | "TG" | "DF" => format!("L{}", digits_in(rest)),
                "SK" => "sticky".into(),
                "LCTL" => "Ctl+…".into(),
                "LSFT" => "Sft+…".into(),
                "LGUI" | "RGUI" => "⌘+…".into(),
                "LALT" => "Alt+…".into(),
                _ => head.to_string(),
            };
        }
        return simple_label(rest);
    }

    // Anything else: show verbatim (truncated).
    if t.len() > 8 {
        format!("{}…", &t[..7])
    } else {
        t.to_string()
    }
}

fn digits_in(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn simple_label(code: &str) -> String {
    match code {
        "TRNS" => "▽".into(),
        "NO" => "".into(),
        "N0" => "0".into(),
        "N1" => "1".into(),
        "N2" => "2".into(),
        "N3" => "3".into(),
        "N4" => "4".into(),
        "N5" => "5".into(),
        "N6" => "6".into(),
        "N7" => "7".into(),
        "N8" => "8".into(),
        "N9" => "9".into(),
        "GRAVE" => "`".into(),
        "MINUS" => "-".into(),
        "EQUAL" => "=".into(),
        "LBRC" => "[".into(),
        "RBRC" => "]".into(),
        "BSLS" => "\\".into(),
        "SCLN" => ";".into(),
        "QUOTE" => "'".into(),
        "COMM" => ",".into(),
        "DOT" => ".".into(),
        "SLSH" => "/".into(),
        "SPC" => "␣".into(),
        "ENT" => "⏎".into(),
        "BSPC" => "⌫".into(),
        "DEL" => "⌦".into(),
        "TAB" => "⇥".into(),
        "ESC" => "⎋".into(),
        "LSFT" => "⇧".into(),
        "RSFT" => "⇧".into(),
        "LCTL" => "⌃".into(),
        "RCTL" => "⌃".into(),
        "LALT" => "⌥".into(),
        "RALT" => "⌥".into(),
        "LGUI" => "⌘".into(),
        "RGUI" => "⌘".into(),
        "UP" => "↑".into(),
        "DOWN" => "↓".into(),
        "LEFT" => "←".into(),
        "RIGHT" => "→".into(),
        "TILD" => "~".into(),
        "EXLM" => "!".into(),
        "AT" => "@".into(),
        "HASH" => "#".into(),
        "DLR" => "$".into(),
        "PERC" => "%".into(),
        "CIRC" => "^".into(),
        "AMPR" => "&".into(),
        "ASTR" => "*".into(),
        "LPRN" => "(".into(),
        "RPRN" => ")".into(),
        "LCBR" => "{".into(),
        "RCBR" => "}".into(),
        "PLUS" => "+".into(),
        "UNDS" => "_".into(),
        "PIPE" => "|".into(),
        // Single letters and F-keys pass through unchanged.
        _ => code.to_string(),
    }
}

/// A curated palette of common keys for the editor, grouped for the UI.
pub fn palette() -> Vec<PaletteKey> {
    let mut v = Vec::new();
    let mut push = |code: &str, label: &str, group: &str| {
        v.push(PaletteKey {
            code: code.into(),
            label: label.into(),
            group: group.into(),
        })
    };

    for c in 'A'..='Z' {
        push(&format!("KC.{c}"), &c.to_string(), "Letters");
    }
    let nums = [
        ("N1", "1"), ("N2", "2"), ("N3", "3"), ("N4", "4"), ("N5", "5"),
        ("N6", "6"), ("N7", "7"), ("N8", "8"), ("N9", "9"), ("N0", "0"),
    ];
    for (c, l) in nums {
        push(&format!("KC.{c}"), l, "Numbers");
    }
    let punct = [
        ("GRAVE", "`"), ("MINUS", "-"), ("EQUAL", "="), ("LBRC", "["),
        ("RBRC", "]"), ("BSLS", "\\"), ("SCLN", ";"), ("QUOTE", "'"),
        ("COMM", ","), ("DOT", "."), ("SLSH", "/"),
    ];
    for (c, l) in punct {
        push(&format!("KC.{c}"), l, "Punctuation");
    }
    let mods = [
        ("LSFT", "⇧ LShift"), ("RSFT", "⇧ RShift"), ("LCTL", "⌃ LCtrl"),
        ("RCTL", "⌃ RCtrl"), ("LALT", "⌥ LAlt"), ("RALT", "⌥ RAlt"),
        ("LGUI", "⌘ LGui"), ("RGUI", "⌘ RGui"),
    ];
    for (c, l) in mods {
        push(&format!("KC.{c}"), l, "Modifiers");
    }
    let nav = [
        ("TAB", "⇥ Tab"), ("ESC", "⎋ Esc"), ("SPC", "␣ Space"),
        ("ENT", "⏎ Enter"), ("BSPC", "⌫ Bksp"), ("DEL", "⌦ Del"),
        ("UP", "↑"), ("DOWN", "↓"), ("LEFT", "←"), ("RIGHT", "→"),
        ("HOME", "Home"), ("END", "End"), ("PGUP", "PgUp"), ("PGDOWN", "PgDn"),
    ];
    for (c, l) in nav {
        push(&format!("KC.{c}"), l, "Nav / Edit");
    }
    for n in 1..=12 {
        push(&format!("KC.F{n}"), &format!("F{n}"), "Function");
    }
    push("KC.TRNS", "▽ Transparent", "Special");
    push("KC.NO", "∅ No-op", "Special");

    v
}
