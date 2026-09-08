//! Locates the WoWSims engine (`wowsimcli.exe` + `db.json`). Both files are
//! embedded directly into the compiled binary at build time (staged into
//! `engine-resources/` by `scripts/stage-engine.ps1` -- see the README's
//! "Getting the engine" section) so the shipped exe is genuinely standalone:
//! no installer, no separate resource folder that has to travel alongside
//! it. On first launch -- or after an update changes the embedded payload --
//! they're written once to the app's local data directory and reused from
//! there on subsequent launches.

use std::fs;
use std::path::Path;

use tauri::Manager;

use armssim::infra::engine::EnginePaths;

const WOWSIMCLI_EXE: &[u8] = include_bytes!("../engine-resources/wowsimcli.exe");
const DB_JSON: &[u8] = include_bytes!("../engine-resources/assets/database/db.json");

pub fn resolve(app: &tauri::AppHandle) -> Result<EnginePaths, String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join("engine");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let cli = dir.join("wowsimcli.exe");
    let db_json = dir.join("db.json");
    write_if_stale(&cli, WOWSIMCLI_EXE)?;
    write_if_stale(&db_json, DB_JSON)?;

    Ok(EnginePaths { cli, db_json })
}

/// Skips the write when a file of the same size is already there -- the
/// common case on every launch after the first.
fn write_if_stale(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let up_to_date = fs::metadata(path)
        .map(|m| m.len() as usize == bytes.len())
        .unwrap_or(false);
    if !up_to_date {
        fs::write(path, bytes).map_err(|e| format!("write {}: {e}", path.display()))?;
    }
    Ok(())
}
