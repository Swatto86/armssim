//! Socket colour rules and meta-gem activation.
//!
//! The engine does **not** evaluate meta-gem requirements: `ItemSpec.meta_gem_disabled`
//! is documented as "set by the UI", and `sim/core/database.go` only reads that
//! flag. So whoever builds the sim request owns this — us. Getting it wrong
//! silently credits a meta gem that would be dark in game.
//!
//! The colour-matching rules and the condition table mirror the site's
//! `ui/core/proto_utils/gems.ts` and `gear.ts` (engine release v0.0.133).

/// Engine `GemColor` codes. Note Blue is 3 and Green is 5, not the other way
/// round — worth stating because the obvious guess is wrong.
pub const META: i32 = 1;
pub const RED: i32 = 2;
pub const BLUE: i32 = 3;
pub const YELLOW: i32 = 4;
pub const GREEN: i32 = 5;
pub const ORANGE: i32 = 6;
pub const PURPLE: i32 = 7;
pub const PRISMATIC: i32 = 8;

/// Can this gem physically go in this socket? Meta gems only fit meta sockets,
/// and nothing else fits a meta socket.
pub fn fits_socket(gem_color: i32, socket_color: i32) -> bool {
    if socket_color == META || gem_color == META {
        return socket_color == META && gem_color == META;
    }
    true
}

/// Does this gem match the socket well enough to earn the item's socket bonus?
/// The engine models this itself (`database.go` adds `SocketBonus` only on a
/// match), so this is only used for reporting, never for scoring.
pub fn earns_socket_bonus(gem_color: i32, socket_color: i32) -> bool {
    if gem_color == PRISMATIC {
        return true;
    }
    match socket_color {
        META => gem_color == META,
        RED => matches!(gem_color, RED | PURPLE | ORANGE),
        YELLOW => matches!(gem_color, YELLOW | ORANGE | GREEN),
        BLUE => matches!(gem_color, BLUE | PURPLE | GREEN),
        PRISMATIC => gem_color != META,
        _ => false,
    }
}

/// Whether a gem counts toward a primary colour for meta requirements. A hybrid
/// counts for both of its components — an Orange gem is both red and yellow.
pub fn counts_as(gem_color: i32, primary: i32) -> bool {
    earns_socket_bonus(gem_color, primary)
}

/// A meta gem's activation requirement.
pub struct MetaCondition {
    pub id: i32,
    pub min_red: u32,
    pub min_yellow: u32,
    pub min_blue: u32,
    /// "more X than Y" conditions; `(0, 0)` when unused.
    pub greater: i32,
    pub lesser: i32,
}

impl MetaCondition {
    fn is_met(&self, red: u32, yellow: u32, blue: u32) -> bool {
        if red < self.min_red || yellow < self.min_yellow || blue < self.min_blue {
            return false;
        }
        if self.greater == 0 {
            return true;
        }
        let count = |c: i32| match c {
            RED => red,
            YELLOW => yellow,
            BLUE => blue,
            _ => 0,
        };
        count(self.greater) > count(self.lesser)
    }
}

/// Every TBC meta gem, transcribed from the site's `gems.ts`.
const CONDITIONS: &[MetaCondition] = &[
    cond(25899, 2, 2, 2),         // Brutal Earthstorm Diamond
    cond(34220, 0, 0, 2),         // Chaotic Skyfire Diamond
    cond(25890, 2, 2, 2),         // Destructive Skyfire Diamond
    cond(35503, 3, 0, 0),         // Ember Skyfire Diamond
    cond(35501, 0, 1, 2),         // Eternal Earthstorm Diamond
    cond(32641, 0, 3, 0),         // Imbued Unstable Diamond
    cond(25901, 2, 2, 2),         // Insightful Earthstorm Diamond
    cond(25896, 0, 0, 3),         // Powerful Earthstorm Diamond
    cond(32409, 2, 2, 2),         // Relentless Earthstorm Diamond
    cond(25894, 1, 2, 0),         // Swift Skyfire Diamond
    cond(28557, 1, 2, 0),         // Swift Starfire Diamond
    cond(28556, 1, 2, 0),         // Swift Windfire Diamond
    cond(25898, 0, 0, 5),         // Tenacious Earthstorm Diamond
    cond(32410, 2, 2, 2),         // Thundering Skyfire Diamond
    compare(25897, RED, BLUE),    // Bracing Earthstorm Diamond
    compare(25895, RED, YELLOW),  // Enigmatic Skyfire Diamond
    compare(25893, BLUE, YELLOW), // Mystical Skyfire Diamond
    compare(32640, BLUE, YELLOW), // Potent Unstable Diamond
];

const fn cond(id: i32, min_red: u32, min_yellow: u32, min_blue: u32) -> MetaCondition {
    MetaCondition {
        id,
        min_red,
        min_yellow,
        min_blue,
        greater: 0,
        lesser: 0,
    }
}

const fn compare(id: i32, greater: i32, lesser: i32) -> MetaCondition {
    MetaCondition {
        id,
        min_red: 0,
        min_yellow: 0,
        min_blue: 0,
        greater,
        lesser,
    }
}

/// Is this meta gem lit, given the colour counts of every other gem equipped?
/// An unknown meta id is treated as active, matching the site's fallback.
pub fn meta_active(meta_id: i32, red: u32, yellow: u32, blue: u32) -> bool {
    match CONDITIONS.iter().find(|c| c.id == meta_id) {
        Some(c) => c.is_met(red, yellow, blue),
        None => true,
    }
}

/// The meta gem in a set of socketed gems, and whether it is lit.
pub struct MetaStatus {
    pub meta_id: Option<i32>,
    pub active: bool,
}

/// Work out the meta gem's state from every gem in a set. `color_of` resolves a
/// gem id to its engine `GemColor`; unknown ids are ignored. A set with no meta
/// gem is reported active — there is nothing to disable.
pub fn meta_status(
    gem_ids: impl Iterator<Item = i32>,
    color_of: impl Fn(i32) -> Option<i32>,
) -> MetaStatus {
    let mut meta = None;
    let (mut red, mut yellow, mut blue) = (0u32, 0u32, 0u32);

    for id in gem_ids {
        let Some(color) = color_of(id) else { continue };
        if color == META {
            meta = Some(id);
            continue;
        }
        // A hybrid counts toward both of its colours.
        if counts_as(color, RED) {
            red += 1;
        }
        if counts_as(color, YELLOW) {
            yellow += 1;
        }
        if counts_as(color, BLUE) {
            blue += 1;
        }
    }

    MetaStatus {
        active: meta.is_none_or(|id| meta_active(id, red, yellow, blue)),
        meta_id: meta,
    }
}

/// The minimum red/yellow/blue counts a meta gem asks for. A "more X than Y"
/// meta reports the count that would satisfy it against the current board, which
/// the caller refines by re-checking `meta_active` as it converts gems.
pub fn meta_minimums(meta_id: i32) -> (u32, u32, u32) {
    match CONDITIONS.iter().find(|c| c.id == meta_id) {
        Some(c) if c.greater == 0 => (c.min_red, c.min_yellow, c.min_blue),
        // Comparison metas have no fixed minimum; ask for one of the greater
        // colour at a time and let the caller re-test.
        Some(c) => {
            let one = |color: i32| u32::from(c.greater == color);
            (one(RED), one(YELLOW), one(BLUE))
        }
        None => (0, 0, 0),
    }
}

/// Requirement text for the report, e.g. "2 red, 2 yellow, 2 blue".
pub fn meta_requirement(meta_id: i32) -> Option<String> {
    let c = CONDITIONS.iter().find(|c| c.id == meta_id)?;
    if c.greater != 0 {
        let name = |v: i32| match v {
            RED => "red",
            YELLOW => "yellow",
            BLUE => "blue",
            _ => "?",
        };
        return Some(format!("more {} than {}", name(c.greater), name(c.lesser)));
    }
    let mut parts = Vec::new();
    for (n, label) in [
        (c.min_red, "red"),
        (c.min_yellow, "yellow"),
        (c.min_blue, "blue"),
    ] {
        if n > 0 {
            parts.push(format!("{n} {label}"));
        }
    }
    Some(parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hybrids_count_for_both_of_their_colours() {
        assert!(counts_as(ORANGE, RED));
        assert!(counts_as(ORANGE, YELLOW));
        assert!(!counts_as(ORANGE, BLUE));
        assert!(counts_as(PURPLE, RED));
        assert!(counts_as(PURPLE, BLUE));
        assert!(counts_as(GREEN, YELLOW));
        assert!(counts_as(GREEN, BLUE));
    }

    #[test]
    fn relentless_earthstorm_needs_two_of_each() {
        assert!(meta_active(32409, 2, 2, 2));
        assert!(!meta_active(32409, 2, 2, 1));
        assert!(meta_active(32409, 5, 3, 2));
    }

    #[test]
    fn compare_conditions_are_strict() {
        // Bracing Earthstorm: more red than blue.
        assert!(meta_active(25897, 3, 0, 2));
        assert!(!meta_active(25897, 2, 0, 2));
    }

    #[test]
    fn meta_socket_only_takes_meta_gems() {
        assert!(fits_socket(META, META));
        assert!(!fits_socket(RED, META));
        assert!(!fits_socket(META, RED));
        assert!(fits_socket(ORANGE, RED));
    }

    #[test]
    fn unknown_meta_is_assumed_active() {
        assert!(meta_active(1, 0, 0, 0));
    }

    #[test]
    fn meta_status_counts_hybrids_and_ignores_the_meta_itself() {
        let colors = |id: i32| match id {
            32409 => Some(META),
            30546 => Some(PURPLE), // red + blue
            32217 => Some(ORANGE), // red + yellow
            _ => None,
        };
        // meta + 2 purple + 2 orange = 4 red, 2 yellow, 2 blue.
        let ids = [32409, 30546, 30546, 32217, 32217];
        let status = meta_status(ids.into_iter(), colors);
        assert_eq!(status.meta_id, Some(32409));
        assert!(status.active);

        let short = [32409, 30546, 32217];
        assert!(!meta_status(short.into_iter(), colors).active);
    }

    #[test]
    fn a_set_with_no_meta_gem_is_active() {
        let status = meta_status([24027].into_iter(), |_| Some(RED));
        assert_eq!(status.meta_id, None);
        assert!(status.active);
    }

    #[test]
    fn requirement_text_reads_naturally() {
        assert_eq!(meta_requirement(32409).unwrap(), "2 red, 2 yellow, 2 blue");
        assert_eq!(meta_requirement(25897).unwrap(), "more red than blue");
    }
}
