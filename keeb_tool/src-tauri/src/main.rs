// Prevent an extra console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use keeb_core::ferris::{suggest_compact_layout, CompactSuggestion, TargetBoard};
use keeb_core::flash::{find_circuitpy_drives, flash_main_py, CircuitPyDrive};
use keeb_core::keys::{palette, PaletteKey};
use keeb_core::layout::{lily58_layout, Slot};
use keeb_core::{generate_main_py, parse_main_py, Keymap};
use serde::Serialize;

/// Everything the UI needs to render a freshly loaded config.
#[derive(Serialize)]
struct ConfigView {
    keymap: Keymap,
    palette: Vec<PaletteKey>,
    layout: Vec<Slot>,
    source_path: String,
}

#[tauri::command]
fn load_config(path: String) -> Result<ConfigView, String> {
    let src = std::fs::read_to_string(&path).map_err(|e| format!("reading {path}: {e}"))?;
    let keymap = parse_main_py(&src).map_err(|e| e.to_string())?;
    Ok(ConfigView {
        keymap,
        palette: palette(),
        layout: lily58_layout(),
        source_path: path,
    })
}

/// Regenerate `main.py` text from an edited keymap (for preview / diff).
#[tauri::command]
fn preview(keymap: Keymap) -> String {
    generate_main_py(&keymap)
}

/// Write the edited keymap back to a file on disk.
#[tauri::command]
fn save_config(path: String, keymap: Keymap) -> Result<(), String> {
    let text = generate_main_py(&keymap);
    std::fs::write(&path, text).map_err(|e| format!("writing {path}: {e}"))
}

#[tauri::command]
fn list_drives() -> Vec<CircuitPyDrive> {
    find_circuitpy_drives()
}

/// Flash the keymap to a connected CircuitPython keyboard by copying `main.py`.
#[tauri::command]
fn flash(drive: String, keymap: Keymap) -> Result<String, String> {
    let text = generate_main_py(&keymap);
    flash_main_py(&drive, &text).map_err(|e| format!("flashing {drive}: {e}"))
}

#[tauri::command]
fn suggest(keymap: Keymap, target: String) -> CompactSuggestion {
    suggest_compact_layout(&keymap, TargetBoard::from_str(&target))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_config,
            preview,
            save_config,
            list_drives,
            flash,
            suggest
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lily58 Config Tool");
}
