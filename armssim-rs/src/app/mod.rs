//! Application layer: parses the character export and orchestrates the gear
//! search through the [`Simulator`](simulator::Simulator) contract. Contains no
//! engine specifics and no domain combat logic.

pub mod character;
pub mod optimize;
pub mod simulator;
