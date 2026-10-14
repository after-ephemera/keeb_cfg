//! Suggest how to fold a 58-key Lily58 base layer onto a smaller board
//! (Ferris/Sweep 34 or Corne 42): what to keep in the 3×N grid, which thumbs
//! survive, and where the orphaned keys should go (home-row mods, layers,
//! combos).

use crate::model::{Key, Keymap};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TargetBoard {
    /// Ferris / Sweep: 3×5 per half + 2 thumbs = 34 keys.
    FerrisSweep,
    /// Corne (crkbd): 3×6 per half + 3 thumbs = 42 keys.
    Corne,
}

impl TargetBoard {
    pub fn from_str(s: &str) -> TargetBoard {
        match s.to_ascii_lowercase().as_str() {
            "corne" | "crkbd" | "42" => TargetBoard::Corne,
            _ => TargetBoard::FerrisSweep,
        }
    }
    fn name(self) -> &'static str {
        match self {
            TargetBoard::FerrisSweep => "Ferris / Sweep (34 keys)",
            TargetBoard::Corne => "Corne (42 keys)",
        }
    }
    fn cols_per_half(self) -> usize {
        match self {
            TargetBoard::FerrisSweep => 5,
            TargetBoard::Corne => 6,
        }
    }
    fn thumbs_per_half(self) -> usize {
        match self {
            TargetBoard::FerrisSweep => 2,
            TargetBoard::Corne => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRef {
    pub source_index: usize,
    pub raw: String,
    pub label: String,
}

impl KeyRef {
    fn of(km_layer: &[Key], idx: usize) -> KeyRef {
        let k = &km_layer[idx];
        KeyRef {
            source_index: idx,
            raw: k.raw.clone(),
            label: k.label.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relocation {
    pub raw: String,
    pub label: String,
    pub reason: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactSuggestion {
    pub target: String,
    pub total_keys: usize,
    pub cols_per_half: usize,
    pub rows: usize,
    pub thumbs_per_half: usize,
    /// 3 rows × cols_per_half of kept left-hand keys.
    pub left_rows: Vec<Vec<KeyRef>>,
    pub right_rows: Vec<Vec<KeyRef>>,
    pub left_thumbs: Vec<KeyRef>,
    pub right_thumbs: Vec<KeyRef>,
    pub relocations: Vec<Relocation>,
    pub notes: Vec<String>,
}

// Lily58 base-layer index groups (see parser/layout docs).
const L_ROW1: [usize; 6] = [12, 13, 14, 15, 16, 17]; // LALT Q W E R T
const L_ROW2: [usize; 6] = [24, 25, 26, 27, 28, 29]; // ESC  A S D F G
const L_ROW3: [usize; 6] = [36, 37, 38, 39, 40, 41]; // LSFT Z X C V B
const R_ROW1: [usize; 6] = [18, 19, 20, 21, 22, 23]; // Y U I O P MINUS (inner→outer)
const R_ROW2: [usize; 6] = [30, 31, 32, 33, 34, 35]; // H J K L SCLN QUOTE
const R_ROW3: [usize; 6] = [44, 45, 46, 47, 48, 49]; // N M COMM DOT SLSH BSLS
const L_THUMBS: [usize; 4] = [50, 51, 52, 53]; // outer→inner
const R_THUMBS: [usize; 4] = [54, 55, 56, 57]; // inner→outer
const INNER_EXTRAS: [usize; 2] = [42, 43]; // [{ ]}

/// Build a compact-layout suggestion from a Lily58 keymap's base layer.
pub fn suggest_compact_layout(km: &Keymap, target: TargetBoard) -> CompactSuggestion {
    let cols = target.cols_per_half();
    let thumbs = target.thumbs_per_half();
    let mut notes = Vec::new();
    let mut relocations = Vec::new();

    let base = match km.layers.first() {
        Some(l) => &l.keys,
        None => {
            return CompactSuggestion {
                target: target.name().into(),
                total_keys: 0,
                cols_per_half: cols,
                rows: 3,
                thumbs_per_half: thumbs,
                left_rows: vec![],
                right_rows: vec![],
                left_thumbs: vec![],
                right_thumbs: vec![],
                relocations,
                notes: vec!["No base layer to analyze.".into()],
            };
        }
    };

    if base.len() != 58 {
        notes.push(format!(
            "Heuristics are tuned for a 58-key Lily58; this base layer has {} keys, so mapping may be approximate.",
            base.len()
        ));
    }

    // Keep 5 columns per half for Ferris (drop the outer mod column),
    // all 6 for Corne.
    let (l_keep, l_drop): (std::ops::Range<usize>, Vec<usize>) = if cols == 5 {
        // Left: drop col0 (outer mods), keep cols 1..=5.
        (1..6, vec![L_ROW1[0], L_ROW2[0], L_ROW3[0]])
    } else {
        (0..6, vec![])
    };
    let (r_keep, r_drop): (std::ops::Range<usize>, Vec<usize>) = if cols == 5 {
        // Right: keep inner 5 cols (0..=4), drop outer col5.
        (0..5, vec![R_ROW1[5], R_ROW2[5], R_ROW3[5]])
    } else {
        (0..6, vec![])
    };

    let get = |idx: usize| KeyRef::of(base, idx);
    let build = |src_row: &[usize; 6], keep: &std::ops::Range<usize>| -> Vec<KeyRef> {
        src_row[keep.clone()].iter().map(|&i| get(i)).collect()
    };

    let left_rows = vec![
        build(&L_ROW1, &l_keep),
        build(&L_ROW2, &l_keep),
        build(&L_ROW3, &l_keep),
    ];
    let right_rows = vec![
        build(&R_ROW1, &r_keep),
        build(&R_ROW2, &r_keep),
        build(&R_ROW3, &r_keep),
    ];

    // Thumbs: keep the innermost `thumbs` per half (closest to the split).
    let left_thumbs: Vec<KeyRef> = L_THUMBS[L_THUMBS.len() - thumbs..]
        .iter()
        .map(|&i| get(i))
        .collect();
    let right_thumbs: Vec<KeyRef> = R_THUMBS[..thumbs].iter().map(|&i| get(i)).collect();

    // ---- Relocations for everything that got dropped ----

    // Number row.
    relocations.push(Relocation {
        raw: "KC.N1 … KC.N0 / KC.TAB / KC.GRAVE".into(),
        label: "Number row".into(),
        reason: "No number row on a 3×5/3×6 board.".into(),
        suggestion: "Put numbers on a layer. You already have a numpad on your Raise layer — \
                     reuse it, or add `KC.LT(RAISE, KC.SPC)` so holding a thumb exposes numbers."
            .into(),
    });

    // Dropped outer mod column (Ferris only).
    for &i in l_drop.iter().chain(r_drop.iter()) {
        let k = &base[i];
        relocations.push(reloc_for_dropped(k, i));
    }

    // Inner bracket extras.
    for &i in &INNER_EXTRAS {
        if i < base.len() {
            let k = &base[i];
            relocations.push(Relocation {
                raw: k.raw.clone(),
                label: k.label.clone(),
                reason: "Inner-column extras don't exist on the compact board.".into(),
                suggestion: "Brackets work well as combos (e.g. `E+R` → `(`, `U+I` → `)`) \
                             or on the symbol layer."
                    .into(),
            });
        }
    }

    // Dropped thumbs.
    let dropped_left_thumbs = &L_THUMBS[..L_THUMBS.len() - thumbs];
    let dropped_right_thumbs = &R_THUMBS[thumbs..];
    for &i in dropped_left_thumbs.iter().chain(dropped_right_thumbs.iter()) {
        let k = &base[i];
        relocations.push(Relocation {
            raw: k.raw.clone(),
            label: k.label.clone(),
            reason: format!(
                "Only {} thumb key(s) per half survive; this one is dropped.",
                thumbs
            ),
            suggestion: "Fold layer switches into layer-taps on the surviving thumbs, e.g. \
                         `KC.LT(1, KC.SPC)` (tap = Space, hold = Lower)."
                .into(),
        });
    }

    // Home-row-mod recommendation (the big win on small boards).
    notes.push(
        "Recommended: adopt home-row mods to recover the modifiers you lose with the outer \
         column — e.g. A=GUI, S=Alt, D=Ctrl, F=Shift (and mirror on J K L ;) via \
         `KC.MT(KC.A, KC.LGUI)` etc. Your existing HoldTap tuning (tap_time=200) transfers directly."
            .into(),
    );
    notes.push(
        "You already use tap-dance and hold-tap heavily, so the compact board mostly means \
         moving those tricks onto the home row and thumbs instead of a dedicated outer column."
            .into(),
    );

    let total = cols * 2 * 3 + thumbs * 2;
    CompactSuggestion {
        target: target.name().into(),
        total_keys: total,
        cols_per_half: cols,
        rows: 3,
        thumbs_per_half: thumbs,
        left_rows,
        right_rows,
        left_thumbs,
        right_thumbs,
        relocations,
        notes,
    }
}

fn reloc_for_dropped(k: &Key, idx: usize) -> Relocation {
    // Tailored advice for the specific outer-column keys of this config.
    let (reason, suggestion) = match idx {
        12 => (
            "Outer column removed; this is your `+`/Alt hold-tap.",
            "Alt → home-row mod on `A` (`KC.MT(KC.A, KC.LALT)`); `+` lives on the symbol layer.",
        ),
        24 => (
            "Outer column removed; this is Esc.",
            "Esc → a combo (e.g. `Q+W`) or a tap on a layer thumb; many use a hold-tap Esc on Tab.",
        ),
        36 => (
            "Outer column removed; this is Shift.",
            "Shift → home-row mod on `F` (`KC.MT(KC.F, KC.LSFT)`) or a one-shot/sticky shift on a thumb.",
        ),
        23 | 35 | 49 => (
            "Outer right column removed.",
            "`-`, `'`, `\\` → symbol layer or combos.",
        ),
        _ => (
            "Dropped with the outer column.",
            "Relocate to the symbol/nav layer or a combo.",
        ),
    };
    Relocation {
        raw: k.raw.clone(),
        label: k.label.clone(),
        reason: reason.into(),
        suggestion: suggestion.into(),
    }
}
