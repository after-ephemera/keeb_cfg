//! Detect a mounted CIRCUITPY drive and "flash" a keymap by copying `main.py`.
//!
//! KMK runs on CircuitPython, which mounts the keyboard as a USB mass-storage
//! volume named `CIRCUITPY` containing a `boot_out.txt`. Writing `main.py` to
//! that volume *is* flashing — CircuitPython reloads automatically on save.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitPyDrive {
    pub path: String,
    /// Contents of `boot_out.txt` (board + CircuitPython version), if readable.
    pub info: Option<String>,
    /// True when a `main.py` (or `code.py`) already exists on the volume.
    pub has_keymap: bool,
}

/// Scan common mount locations for CircuitPython volumes.
pub fn find_circuitpy_drives() -> Vec<CircuitPyDrive> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // macOS
    collect_children(Path::new("/Volumes"), &mut candidates);

    // Linux (udisks auto-mount locations)
    if let Ok(user) = std::env::var("USER") {
        collect_children(&PathBuf::from(format!("/media/{user}")), &mut candidates);
        collect_children(&PathBuf::from(format!("/run/media/{user}")), &mut candidates);
    }
    collect_children(Path::new("/media"), &mut candidates);
    collect_children(Path::new("/mnt"), &mut candidates);

    // Windows drive letters D:..Z:
    for letter in b'D'..=b'Z' {
        candidates.push(PathBuf::from(format!("{}:\\", letter as char)));
    }

    let mut drives = Vec::new();
    for path in candidates {
        if is_circuitpy(&path) {
            let info = std::fs::read_to_string(path.join("boot_out.txt"))
                .ok()
                .map(|s| s.trim().to_string());
            let has_keymap = path.join("main.py").exists() || path.join("code.py").exists();
            drives.push(CircuitPyDrive {
                path: path.to_string_lossy().to_string(),
                info,
                has_keymap,
            });
        }
    }
    drives
}

fn collect_children(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            out.push(e.path());
        }
    }
}

/// A volume is CircuitPython if it has `boot_out.txt`, or is literally named
/// CIRCUITPY (fallback before `boot_out.txt` is written).
fn is_circuitpy(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    if path.join("boot_out.txt").exists() {
        return true;
    }
    path.file_name()
        .map(|n| n.eq_ignore_ascii_case("CIRCUITPY"))
        .unwrap_or(false)
}

/// Write `contents` to `<drive>/main.py`, backing up any existing file first.
pub fn flash_main_py(drive: &str, contents: &str) -> std::io::Result<String> {
    let target = Path::new(drive).join("main.py");
    if target.exists() {
        let backup = Path::new(drive).join("main.py.bak");
        let _ = std::fs::copy(&target, &backup); // best-effort backup
    }
    std::fs::write(&target, contents)?;
    Ok(target.to_string_lossy().to_string())
}
