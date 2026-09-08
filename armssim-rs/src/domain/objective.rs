//! The search objective: what the coordinate ascent maximizes, and which
//! scenarios each objective needs simulated.

/// What a search run optimizes for.
#[derive(Clone, Copy, Debug)]
pub enum Objective {
    /// Maximize single-target DPS.
    SingleTarget,
    /// Maximize a weighted blend of ST and AoE (`aoe_fraction` in 0..=1).
    Blend { aoe_fraction: f64 },
    /// Maximize multi-target DPS.
    Aoe,
}

impl Objective {
    pub fn name(&self) -> String {
        match self {
            Objective::SingleTarget => "SINGLE TARGET".to_string(),
            Objective::Aoe => "AOE".to_string(),
            Objective::Blend { aoe_fraction } => format!(
                "BLEND  ({:.0}% ST / {:.0}% AoE)",
                (1.0 - aoe_fraction) * 100.0,
                aoe_fraction * 100.0
            ),
        }
    }

    pub fn need_st(&self) -> bool {
        matches!(self, Objective::SingleTarget | Objective::Blend { .. })
    }

    pub fn need_aoe(&self) -> bool {
        matches!(self, Objective::Aoe | Objective::Blend { .. })
    }

    /// Combine the two scenario DPS values into a single score to maximize.
    pub fn score(&self, st: f64, aoe: f64) -> f64 {
        match self {
            Objective::SingleTarget => st,
            Objective::Aoe => aoe,
            Objective::Blend { aoe_fraction } => (1.0 - aoe_fraction) * st + aoe_fraction * aoe,
        }
    }
}
