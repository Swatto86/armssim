//! Infrastructure layer: adapters over the bundled WoWSims engine. Implements
//! the contracts the inner layers define — the item catalog and the simulator —
//! and translates gear sets into engine sim requests.

pub mod engine;
pub mod itemdb;
pub mod request;
pub mod wowsims;
