//! The simulation contract the optimizer depends on. The infrastructure layer
//! implements it by driving the WoWSims engine.

use crate::domain::gear::{GearSet, Scenario};

/// One unit of work: simulate a gear set in a given scenario.
pub type Job<'a> = (&'a GearSet, Scenario);

/// Scores gear sets by DPS. Implementations decide how the work is parallelized;
/// taking all jobs at once lets the backend keep every core busy even when an
/// individual ascent step has only a few candidates.
pub trait Simulator: Sync {
    /// Run each job once, returning DPS index-aligned with `jobs`.
    fn run(&self, jobs: &[Job<'_>], iterations: u32, seed: i64) -> anyhow::Result<Vec<f64>>;
}
