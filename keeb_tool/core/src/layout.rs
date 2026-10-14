//! Physical geometry of the boards, used by the 3D view and the pretty-printer.
//!
//! Coordinates are in "key units" (1u = one keycap pitch). `x` grows to the
//! right, `y` grows *downward* (screen coordinates); the frontend flips `y`
//! for the 3D scene. Column stagger mimics the real Lily58 / Ferris ergo layout.

use serde::{Deserialize, Serialize};

/// A physical key slot: where it sits and which keymap index it maps to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slot {
    /// Index into a layer's `keys` vector.
    pub index: usize,
    pub x: f32,
    pub y: f32,
    /// "L" or "R" half.
    pub side: char,
    /// Logical row (0 = number row for Lily58).
    pub row: i32,
    /// True for thumb-cluster keys.
    pub thumb: bool,
}

/// Row lengths for the pretty-printer, keyed by total key count.
pub fn row_lengths_for(total: usize) -> Vec<usize> {
    match total {
        58 => vec![12, 12, 12, 14, 8], // Lily58: 4 alpha rows (inner extras) + 8 thumb
        _ => {
            // Fallback: 12 per line.
            let mut v = Vec::new();
            let mut remaining = total;
            while remaining > 12 {
                v.push(12);
                remaining -= 12;
            }
            if remaining > 0 {
                v.push(remaining);
            }
            v
        }
    }
}

/// Per-column downward stagger (in u) for a Lily58 half, outer→inner.
/// Middle-finger column (index 3) is highest; pinky and index drop lower.
const STAGGER: [f32; 6] = [0.35, 0.35, 0.15, 0.0, 0.1, 0.25];

/// Build the full physical layout of the Lily58 (58 slots) in keymap order.
///
/// Keymap order (from this repo's `main.py`):
///   row0: L0..L5  R0..R5            (12)
///   row1: L0..L5  R0..R5            (12)
///   row2: L0..L5  R0..R5            (12)
///   row3: L0..L5 [inL inR] R0..R5   (14)
///   thumb: L0 L1 L2 L3  R0 R1 R2 R3 (8)
pub fn lily58_layout() -> Vec<Slot> {
    let mut slots = Vec::with_capacity(58);
    // Gap between the two halves, in u.
    let split_gap = 3.0;
    let right_x0 = 6.0 + split_gap; // right half's leftmost (inner) column origin

    let mut idx = 0usize;
    // --- three number/alpha rows + one alpha row, interleaved L then R ---
    for row in 0..4 {
        let y = row as f32;
        // Left half: columns 0..5, left→right.
        for col in 0..6 {
            slots.push(Slot {
                index: idx,
                x: col as f32,
                y: y + STAGGER[col],
                side: 'L',
                row,
                thumb: false,
            });
            idx += 1;
        }
        // Row 3 has two extra inner keys ([{ and ]}) between the halves.
        if row == 3 {
            slots.push(Slot { index: idx, x: 5.6, y: 4.4, side: 'L', row: 3, thumb: true });
            idx += 1;
            slots.push(Slot { index: idx, x: right_x0 + 0.4, y: 4.4, side: 'R', row: 3, thumb: true });
            idx += 1;
        }
        // Right half: keymap order is inner→outer, so column 0 is the inner index.
        for col in 0..6 {
            let phys_col = 5 - col; // mirror: inner (0) sits at inner-most position
            slots.push(Slot {
                index: idx,
                x: right_x0 + phys_col as f32,
                y: y + STAGGER[5 - phys_col],
                side: 'R',
                row,
                thumb: false,
            });
            idx += 1;
        }
    }

    // --- thumb clusters: 4 left, then 4 right ---
    let thumb_y = 5.0;
    let left_thumb_x = [2.5, 3.5, 4.5, 5.5];
    for (i, &x) in left_thumb_x.iter().enumerate() {
        slots.push(Slot {
            index: idx,
            x,
            y: thumb_y + (i as f32) * 0.12,
            side: 'L',
            row: 4,
            thumb: true,
        });
        idx += 1;
    }
    let right_thumb_x = [right_x0 + 0.5, right_x0 + 1.5, right_x0 + 2.5, right_x0 + 3.5];
    for (i, &x) in right_thumb_x.iter().enumerate() {
        slots.push(Slot {
            index: idx,
            x,
            y: thumb_y + ((3 - i) as f32) * 0.12,
            side: 'R',
            row: 4,
            thumb: true,
        });
        idx += 1;
    }

    slots
}
