//! armssim — a local Arms-warrior gear optimizer for WoW: TBC Anniversary.
//!
//! The WoWSims TBC engine is the source of every DPS number: sims run through
//! the `wowsimcli` binary, so results match the website. Only two-handed
//! weapons are ever equipped — Arms is a 2H spec — and the embedded rotation is
//! the engine's Arms APL. The optimizer parses a
//! single combined character export (equipped gear + bag/bank items), then uses
//! coordinate ascent to find the best single-target, AoE, and blended sets.

pub mod app;
pub mod domain;
pub mod error;
pub mod infra;
