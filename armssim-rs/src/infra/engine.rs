//! Locating the bundled WoWSims engine: the `wowsimcli` binary that runs sims
//! and the `db.json` item database that drives classification.

use std::env;
use std::path::{Path, PathBuf};

use crate::error::ArmssimError;

/// Resolved paths into the engine directory.
pub struct EnginePaths {
    pub cli: PathBuf,
    pub db_json: PathBuf,
}

fn cli_name() -> &'static str {
    if cfg!(windows) {
        "wowsimcli.exe"
    } else {
        "wowsimcli"
    }
}

/// Resolve the engine directory, in priority order: an explicit `--engine`
/// path, the `ARMSSIM_ENGINE` env var, then a search upward from the executable
/// and the working directory for an `engine/` folder.
pub fn resolve(explicit: Option<PathBuf>) -> anyhow::Result<EnginePaths> {
    let mut roots: Vec<PathBuf> = Vec::new();

    if let Some(e) = explicit {
        roots.push(e);
    }
    if let Ok(e) = env::var("ARMSSIM_ENGINE") {
        if !e.is_empty() {
            roots.push(PathBuf::from(e));
        }
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.extend(dir.ancestors().map(Path::to_path_buf));
        }
    }
    if let Ok(cwd) = env::current_dir() {
        roots.extend(cwd.ancestors().map(Path::to_path_buf));
    }

    for root in roots {
        // The root may be the engine dir itself or a parent containing `engine/`.
        for candidate in [root.clone(), root.join("engine")] {
            let cli = candidate.join(cli_name());
            let db_json = candidate.join("assets").join("database").join("db.json");
            if cli.is_file() && db_json.is_file() {
                return Ok(EnginePaths { cli, db_json });
            }
        }
    }

    Err(ArmssimError::EngineNotFound.into())
}
