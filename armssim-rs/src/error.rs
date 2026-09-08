//! Typed errors for the layers that have meaningful failure modes. Application
//! glue (main, infrastructure orchestration) propagates these with `anyhow`.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArmssimError {
    #[error("character export contained no gear items")]
    NoGear,

    #[error(
        "engine not found: expected a directory containing wowsimcli.exe and \
         assets/database/db.json (pass --engine <dir> or set ARMSSIM_ENGINE)"
    )]
    EngineNotFound,

    #[error("unsupported race {0:?}")]
    UnsupportedRace(String),

    #[error("export is for class {0:?}; armssim only sims Arms warriors")]
    WrongClass(String),

    #[error(
        "export is for spec {0:?}; armssim only sims Arms warriors (the rotation is Arms-specific)"
    )]
    WrongSpec(String),

    #[error("sim backend failed: {0}")]
    Sim(String),
}
