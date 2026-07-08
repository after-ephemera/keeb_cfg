//! Regenerate the `keyboard.keymap = [...]` block from a [`Keymap`].
//!
//! Only the keymap block is rebuilt; `preamble` and `postamble` are emitted
//! verbatim, so every module/helper/OLED/RGB definition survives untouched.

use crate::layout;
use crate::model::Keymap;

/// Produce a complete `main.py` from a (possibly edited) [`Keymap`].
pub fn generate_main_py(km: &Keymap) -> String {
    let mut out = String::with_capacity(km.preamble.len() + km.postamble.len() + 4096);
    out.push_str(&km.preamble);
    out.push_str(&render_keymap(km));
    out.push_str(&km.postamble);
    out
}

fn render_keymap(km: &Keymap) -> String {
    let mut s = String::from("[\n");
    for (li, layer) in km.layers.iter().enumerate() {
        if let Some(name) = &layer.name {
            s.push_str(&format!("    [  # {name}\n"));
        } else {
            s.push_str(&format!("    [  # layer {li}\n"));
        }
        s.push_str(&render_layer(layer));
        s.push_str("    ],\n");
    }
    s.push(']');
    s
}

fn render_layer(layer: &crate::model::Layer) -> String {
    let codes: Vec<&str> = layer.keys.iter().map(|k| k.raw.as_str()).collect();
    let rows = layout::row_lengths_for(codes.len());

    // Column width so rows line up as a readable grid.
    let width = codes.iter().map(|c| c.len()).max().unwrap_or(1).min(28);

    let mut s = String::new();
    let mut idx = 0;
    for &len in &rows {
        s.push_str("        ");
        for j in 0..len {
            if idx >= codes.len() {
                break;
            }
            let cell = format!("{:<width$}", codes[idx], width = width);
            s.push_str(cell.trim_end_matches(' '));
            s.push_str(&" ".repeat(width.saturating_sub(codes[idx].len())));
            s.push_str(", ");
            let _ = j;
            idx += 1;
        }
        s.push('\n');
    }
    // Any leftover keys (unexpected count) dumped flat.
    while idx < codes.len() {
        s.push_str(&format!("        {}, \n", codes[idx]));
        idx += 1;
    }
    s
}
