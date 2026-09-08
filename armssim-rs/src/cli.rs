//! Command-line interface.

use std::path::PathBuf;

use clap::{Parser, ValueEnum};

/// Which objective set(s) to compute.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Only {
    All,
    St,
    Blend,
    Aoe,
}

#[derive(Parser, Debug)]
#[command(
    name = "armssim",
    about = "Arms-warrior gear optimizer — drives the WoWSims TBC engine in a subprocess",
    long_about = "Searches your equipped + bag/bank items for the highest-DPS Arms setup, \
                  for single-target, AoE, and a blend, using the WoWSims TBC engine."
)]
pub struct Args {
    /// Character export JSON (equipped gear + bagItems in one file).
    pub character: PathBuf,

    /// Iterations per sim during the gear search (lower = faster).
    #[arg(long, default_value_t = 1500)]
    pub iterations: u32,

    /// Iterations for the precise re-sim of the winning set.
    #[arg(long = "final-iterations", default_value_t = 20000)]
    pub final_iterations: u32,

    /// Fraction of fight time spent in AoE, for the blended 'median' set (0..=1).
    #[arg(long = "aoe-fraction", default_value_t = 0.3)]
    pub aoe_fraction: f64,

    /// Which sets to compute (blend-only is ~3x faster).
    #[arg(long, value_enum, default_value_t = Only::All)]
    pub only: Only,

    /// Random seed (fixed for reproducibility).
    #[arg(long, default_value_t = 1)]
    pub seed: i64,

    /// Engine directory containing wowsimcli + assets/database/db.json
    /// (auto-detected if omitted).
    #[arg(long)]
    pub engine: Option<PathBuf>,

    /// Number of parallel sims (default: CPU count).
    #[arg(long)]
    pub jobs: Option<usize>,
}
