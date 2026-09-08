//! Wire types shared between `commands::run_optimizer` and the frontend.
//! Mirrors `cli::Args`/`main.rs`'s console output, just structured instead of
//! printed.

use serde::{Deserialize, Serialize};

/// Which objective set(s) to compute — mirrors `cli::Only`, kept separate
/// since that type is clap-derived and CLI-only.
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum OnlySetting {
    All,
    St,
    Blend,
    Aoe,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunSettings {
    pub iterations: u32,
    pub final_iterations: u32,
    pub aoe_fraction: f64,
    pub only: OnlySetting,
    pub seed: i64,
    pub jobs: Option<usize>,
    /// Skip the gem/enchant pass. Defaults to false for older saved settings
    /// that predate it.
    #[serde(default)]
    pub no_refine: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub objective: String,
    pub sims: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotChangeDto {
    pub label: String,
    pub from: String,
    pub to: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveResult {
    pub name: String,
    pub st_dps: f64,
    pub aoe_dps: f64,
    pub st_delta: f64,
    pub st_delta_pct: f64,
    pub aoe_delta: f64,
    pub aoe_delta_pct: f64,
    pub search_sims: usize,
    pub refine_sims: usize,
    pub changes: Vec<SlotChangeDto>,
    /// Gem and enchant moves on items the search kept.
    pub refinements: Vec<RefineChangeDto>,
    /// Sockets and enchant slots still empty in the recommended set.
    pub missing: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefineChangeDto {
    pub label: String,
    /// "socket 1", "enchant", ...
    pub what: String,
    pub from: String,
    pub to: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunResult {
    pub character_name: String,
    pub character_race: String,
    pub candidate_pool: usize,
    pub skipped: usize,
    pub base_st: f64,
    pub base_aoe: f64,
    pub objectives: Vec<ObjectiveResult>,
}
