//! The gem/enchant refinement pass: given a chosen gear set, what should go in
//! its sockets and on its enchantable slots.
//!
//! This runs *after* the item search, because a socket only exists once you've
//! decided which item is in the slot. Candidates are shortlisted here and the
//! engine picks the winner — the shortlist never decides anything on its own.

use super::gear::{GearSet, ItemSlot};
use super::gems;
use super::item::ItemCatalog;
use super::plan::Group;

/// A gem as the engine's database describes it.
#[derive(Clone, Debug)]
pub struct GemInfo {
    pub id: i32,
    pub name: String,
    pub color: i32,
    /// Engine stat array (42 entries, indexed by the `Stat` enum).
    pub stats: Vec<f64>,
    pub unique: bool,
    pub profession: String,
}

/// An enchant. Keyed by `(effect_id, item_type)` — the database reuses an
/// effect id across slots (2564 is both "Enchant Gloves - Superior Agility" and
/// "Enchant Weapon - Agility"), so the id alone does not identify one.
#[derive(Clone, Debug)]
pub struct EnchantInfo {
    pub effect_id: i32,
    pub item_type: i32,
    pub name: String,
    pub two_hand_only: bool,
}

/// Gem and enchant lookups, implemented over the engine's `db.json`.
pub trait RefineCatalog {
    fn gem(&self, id: i32) -> Option<&GemInfo>;
    fn all_gems(&self) -> &[GemInfo];
    fn enchant(&self, effect_id: i32, item_type: i32) -> Option<&EnchantInfo>;
}

/// How many gems to put in front of the engine per socket. Four is enough to
/// cover "pure strength / strength+crit / hit / expertise" without turning the
/// refinement pass into the expensive part of the run.
const GEMS_PER_SOCKET: usize = 4;

/// Rough stat values used *only* to shortlist gems, from the engine's own P3-P5
/// Arms EP preset (`ui/warrior/dps/presets.ts`). Deliberately not used to pick a
/// winner: several gems and every proc enchant would rank wrong.
const EP: [(usize, f64); 8] = [
    (0, 1.00),  // Strength
    (1, 0.80),  // Agility
    (17, 0.45), // Attack power
    (20, 1.01), // Melee hit
    (21, 1.05), // Melee crit
    (22, 0.85), // Melee haste
    (23, 0.23), // Armour penetration
    (24, 1.78), // Expertise
];

/// Meta gems worth a melee warrior's time. Curated rather than derived: the
/// interesting part of a meta gem (Relentless's +3% crit damage) is a scripted
/// effect with no entry in the database's stat array, so a stat-based shortlist
/// would rank them all at zero.
const META_CANDIDATES: [i32; 3] = [
    32409, // Relentless Earthstorm Diamond
    32410, // Thundering Skyfire Diamond
    25899, // Brutal Earthstorm Diamond
];

/// Enchants worth simming per slot, keyed by engine `ItemType`. Curated for the
/// same reason: Mongoose has an all-zero stat line in the database because it is
/// implemented as a proc in `sim/common/tbc/enchants.go`, so deriving this list
/// from stats would throw away the best weapon enchant in the game.
const ENCHANTS: [(i32, &[i32]); 9] = [
    (1, &[3003, 3096]),                          // Head
    (3, &[2997, 2986, 2717, 2983]),              // Shoulder
    (4, &[368]),                                 // Back
    (5, &[2661]),                                // Chest
    (6, &[2647, 1593, 1891]),                    // Wrist
    (7, &[684, 1594, 2564]),                     // Hands
    (9, &[3012, 3010]),                          // Legs
    (10, &[2939, 2657, 2658]),                   // Feet
    (13, &[3225, 2673, 2667, 2670, 2668, 3222]), // Weapon
];

fn ep(stats: &[f64]) -> f64 {
    EP.iter()
        .map(|(i, w)| stats.get(*i).copied().unwrap_or(0.0) * w)
        .sum()
}

/// Is this gem one the player could actually socket? Unique gems are excluded
/// outright: the optimizer would happily suggest the same one twice, which the
/// game would not allow.
fn usable(gem: &GemInfo, professions: &[String]) -> bool {
    if gem.unique {
        return false;
    }
    if !gem.profession.is_empty() && gem.profession != "ProfessionUnknown" {
        let has = professions
            .iter()
            .any(|p| p.eq_ignore_ascii_case(&gem.profession));
        if !has {
            return false;
        }
    }
    true
}

/// The shortlist for one socket, best-EP first, always including whatever is
/// already in the socket so the ascent can keep it.
fn gem_candidates(
    socket_color: i32,
    current: i32,
    cat: &dyn RefineCatalog,
    professions: &[String],
) -> Vec<i32> {
    let mut ids: Vec<i32> = if socket_color == gems::META {
        META_CANDIDATES
            .iter()
            .copied()
            .filter(|id| cat.gem(*id).is_some())
            .collect()
    } else {
        let mut scored: Vec<(&GemInfo, f64)> = cat
            .all_gems()
            .iter()
            .filter(|g| gems::fits_socket(g.color, socket_color))
            .filter(|g| usable(g, professions))
            .map(|g| (g, ep(&g.stats)))
            .filter(|(_, score)| *score > 0.0)
            .collect();
        scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.id.cmp(&b.0.id)));
        scored
            .into_iter()
            .take(GEMS_PER_SOCKET)
            .map(|(g, _)| g.id)
            .collect()
    };

    if current != 0 && !ids.contains(&current) {
        ids.push(current);
    }
    ids
}

fn enchant_candidates(
    item_type: i32,
    current: i32,
    two_handed: bool,
    cat: &dyn RefineCatalog,
) -> Vec<i32> {
    let Some((_, ids)) = ENCHANTS.iter().find(|(t, _)| *t == item_type) else {
        return Vec::new();
    };
    let mut out: Vec<i32> = ids
        .iter()
        .copied()
        .filter(|id| match cat.enchant(*id, item_type) {
            Some(e) => two_handed || !e.two_hand_only,
            None => false,
        })
        .collect();
    if current != 0 && !out.contains(&current) {
        out.push(current);
    }
    out
}

/// Build the refinement groups for `set`: one per socket, one per enchantable
/// slot. A group with nothing to choose between is dropped.
pub fn plan(
    set: &GearSet,
    items: &dyn ItemCatalog,
    cat: &dyn RefineCatalog,
    professions: &[String],
) -> Vec<Group> {
    let mut groups = Vec::new();

    for slot in 0..super::gear::NUM_SLOTS {
        let Some(spec) = set.get(slot) else { continue };
        let Some(info) = items.lookup(spec.id) else {
            continue;
        };

        for (index, socket_color) in info.sockets.iter().enumerate() {
            let current = spec.gems.get(index).copied().unwrap_or(0);
            let ids = gem_candidates(*socket_color, current, cat, professions);
            if ids.len() > 1 || (ids.len() == 1 && ids[0] != current) {
                groups.push(Group::Gem { slot, index, ids });
            }
        }

        // Only the main hand is a weapon here, and armssim only ever equips a
        // two-hander, so every weapon enchant is eligible.
        let two_handed = slot == ItemSlot::MainHand.index();
        let ids = enchant_candidates(info.item_type, spec.enchant, two_handed, cat);
        if ids.len() > 1 || (ids.len() == 1 && ids[0] != spec.enchant) {
            groups.push(Group::Enchant { slot, ids });
        }
    }

    groups
}

/// Coordinate ascent optimises one socket at a time, so it can never discover a
/// lit meta gem: each individual blue gem is a loss until the one that flips the
/// requirement, which is the same blindness that costs it tier-set bonuses.
///
/// This proposes the cheapest repair — convert the fewest, least valuable gems
/// to the colours the meta is short of — as a single alternative set. The engine
/// still decides whether the repaired set actually beats the greedy one.
pub fn light_meta(
    set: &GearSet,
    items: &dyn ItemCatalog,
    cat: &dyn RefineCatalog,
    professions: &[String],
) -> Option<GearSet> {
    let all_gems = (0..super::gear::NUM_SLOTS)
        .filter_map(|i| set.get(i))
        .flat_map(|spec| spec.gems.iter().copied());
    let status = gems::meta_status(all_gems, |id| cat.gem(id).map(|g| g.color));
    let meta_id = status.meta_id?;
    if status.active {
        return None;
    }

    let mut repaired = set.clone();
    // Every non-meta socket, with what it currently holds.
    let mut sockets: Vec<(usize, usize, i32)> = Vec::new();
    for slot in 0..super::gear::NUM_SLOTS {
        let (Some(spec), Some(info)) = (
            set.get(slot),
            set.get(slot).and_then(|s| items.lookup(s.id)),
        ) else {
            continue;
        };
        for (index, color) in info.sockets.iter().enumerate() {
            if *color != gems::META {
                sockets.push((slot, index, spec.gems.get(index).copied().unwrap_or(0)));
            }
        }
    }

    // Greedy: repeatedly make the single cheapest conversion that reduces the
    // shortfall, until the meta lights or nothing is left to convert.
    let mut converted: Vec<(usize, usize)> = Vec::new();
    for _ in 0..sockets.len() {
        if meta_lit(&repaired, items, cat) {
            return Some(repaired);
        }
        let deficit = shortfall(&repaired, items, cat, meta_id);
        if deficit.iter().all(|(_, n)| *n == 0) {
            break;
        }

        let mut best: Option<(f64, usize, usize, i32)> = None;
        for (slot, index, current) in &sockets {
            if converted.contains(&(*slot, *index)) {
                continue;
            }
            let current_ep = cat.gem(*current).map_or(0.0, |g| ep(&g.stats));
            for candidate in cat.all_gems() {
                if !usable(candidate, professions) || candidate.color == gems::META {
                    continue;
                }
                // Only worth considering if it helps a colour we are short of.
                let helps = deficit
                    .iter()
                    .any(|(color, need)| *need > 0 && gems::counts_as(candidate.color, *color));
                if !helps {
                    continue;
                }
                let loss = current_ep - ep(&candidate.stats);
                if best.is_none_or(|(b, ..)| loss < b) {
                    best = Some((loss, *slot, *index, candidate.id));
                }
            }
        }

        let Some((_, slot, index, gem_id)) = best else {
            break;
        };
        let mut spec = repaired.get(slot)?.clone();
        if spec.gems.len() <= index {
            spec.gems.resize(index + 1, 0);
        }
        spec.gems[index] = gem_id;
        repaired.set(slot, Some(spec));
        converted.push((slot, index));
    }

    if meta_lit(&repaired, items, cat) {
        Some(repaired)
    } else {
        None
    }
}

fn meta_lit(set: &GearSet, _items: &dyn ItemCatalog, cat: &dyn RefineCatalog) -> bool {
    let all_gems = (0..super::gear::NUM_SLOTS)
        .filter_map(|i| set.get(i))
        .flat_map(|spec| spec.gems.iter().copied());
    gems::meta_status(all_gems, |id| cat.gem(id).map(|g| g.color)).active
}

/// How many more gems of each primary colour the meta still needs.
fn shortfall(
    set: &GearSet,
    _items: &dyn ItemCatalog,
    cat: &dyn RefineCatalog,
    meta_id: i32,
) -> [(i32, u32); 3] {
    let (mut red, mut yellow, mut blue) = (0u32, 0u32, 0u32);
    for slot in 0..super::gear::NUM_SLOTS {
        let Some(spec) = set.get(slot) else { continue };
        for id in &spec.gems {
            let Some(color) = cat.gem(*id).map(|g| g.color) else {
                continue;
            };
            if color == gems::META {
                continue;
            }
            if gems::counts_as(color, gems::RED) {
                red += 1;
            }
            if gems::counts_as(color, gems::YELLOW) {
                yellow += 1;
            }
            if gems::counts_as(color, gems::BLUE) {
                blue += 1;
            }
        }
    }
    let need = gems::meta_minimums(meta_id);
    [
        (gems::RED, need.0.saturating_sub(red)),
        (gems::YELLOW, need.1.saturating_sub(yellow)),
        (gems::BLUE, need.2.saturating_sub(blue)),
    ]
}

/// A socket or enchant slot left empty in the final set — reported even when
/// the refinement pass is switched off, because it is free DPS being ignored.
pub struct Gap {
    pub label: String,
    pub what: String,
}

pub fn gaps(set: &GearSet, items: &dyn ItemCatalog, cat: &dyn RefineCatalog) -> Vec<Gap> {
    let mut out = Vec::new();

    // A meta gem whose colour requirement is unmet sits dark: it keeps the
    // item's socket bonus but grants nothing itself. Easy to miss in game, and
    // the engine only knows because armssim tells it (see infra::request).
    let all_gems = (0..super::gear::NUM_SLOTS)
        .filter_map(|i| set.get(i))
        .flat_map(|spec| spec.gems.iter().copied());
    let meta = gems::meta_status(all_gems, |id| cat.gem(id).map(|g| g.color));
    if let (Some(id), false) = (meta.meta_id, meta.active) {
        let name = cat
            .gem(id)
            .map(|g| g.name.clone())
            .unwrap_or_else(|| format!("gem {id}"));
        let need = gems::meta_requirement(id).unwrap_or_else(|| "its colours".to_string());
        out.push(Gap {
            label: "Meta".to_string(),
            what: format!("{name} is not active — needs {need}"),
        });
    }

    for slot in 0..super::gear::NUM_SLOTS {
        let Some(spec) = set.get(slot) else { continue };
        let Some(info) = items.lookup(spec.id) else {
            continue;
        };
        let label = ItemSlot::from_index(slot)
            .map(|s| s.label().to_string())
            .unwrap_or_default();

        let empty = info
            .sockets
            .iter()
            .enumerate()
            .filter(|(i, _)| spec.gems.get(*i).copied().unwrap_or(0) == 0)
            .count();
        if empty > 0 {
            out.push(Gap {
                label: label.clone(),
                what: format!(
                    "{} empty socket{} on {}",
                    empty,
                    if empty == 1 { "" } else { "s" },
                    info.name
                ),
            });
        }
        if spec.enchant == 0 && ENCHANTS.iter().any(|(t, _)| *t == info.item_type) {
            out.push(Gap {
                label,
                what: format!("no enchant on {}", info.name),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::gear::ItemSpec;
    use crate::domain::item::ItemInfo;
    use std::collections::HashMap;

    struct Cat {
        gems: Vec<GemInfo>,
        enchants: Vec<EnchantInfo>,
    }

    impl RefineCatalog for Cat {
        fn gem(&self, id: i32) -> Option<&GemInfo> {
            self.gems.iter().find(|g| g.id == id)
        }
        fn all_gems(&self) -> &[GemInfo] {
            &self.gems
        }
        fn enchant(&self, effect_id: i32, item_type: i32) -> Option<&EnchantInfo> {
            self.enchants
                .iter()
                .find(|e| e.effect_id == effect_id && e.item_type == item_type)
        }
    }

    fn gem(id: i32, color: i32, strength: f64, unique: bool, prof: &str) -> GemInfo {
        let mut stats = vec![0.0; 42];
        stats[0] = strength;
        GemInfo {
            id,
            name: format!("gem{id}"),
            color,
            stats,
            unique,
            profession: prof.to_string(),
        }
    }

    fn catalog() -> Cat {
        Cat {
            gems: vec![
                gem(1, gems::RED, 8.0, false, ""),
                gem(2, gems::RED, 12.0, true, ""), // unique: never offered
                gem(3, gems::RED, 16.0, false, "Jewelcrafting"),
                gem(4, gems::ORANGE, 5.0, false, ""),
                gem(5, gems::YELLOW, 4.0, false, ""),
                gem(32409, gems::META, 0.0, false, ""),
            ],
            enchants: vec![
                EnchantInfo {
                    effect_id: 3012,
                    item_type: 9,
                    name: "Nethercobra".into(),
                    two_hand_only: false,
                },
                EnchantInfo {
                    effect_id: 3010,
                    item_type: 9,
                    name: "Cobrahide".into(),
                    two_hand_only: false,
                },
                EnchantInfo {
                    effect_id: 2667,
                    item_type: 13,
                    name: "Savagery".into(),
                    two_hand_only: true,
                },
                EnchantInfo {
                    effect_id: 3225,
                    item_type: 13,
                    name: "Executioner".into(),
                    two_hand_only: false,
                },
            ],
        }
    }

    struct Items(HashMap<i32, ItemInfo>);
    impl ItemCatalog for Items {
        fn lookup(&self, id: i32) -> Option<ItemInfo> {
            self.0.get(&id).cloned()
        }
    }

    fn legs(sockets: Vec<i32>) -> ItemInfo {
        ItemInfo {
            name: "Legs".into(),
            slots: vec![ItemSlot::Legs],
            sockets,
            item_type: 9,
        }
    }

    fn spec(id: i32, gems_in: Vec<i32>, enchant: i32) -> ItemSpec {
        ItemSpec {
            id,
            enchant,
            gems: gems_in,
            random_suffix: 0,
        }
    }

    #[test]
    fn unique_and_missing_profession_gems_are_not_offered() {
        let cat = catalog();
        let ids = gem_candidates(gems::RED, 0, &cat, &[]);
        assert!(!ids.contains(&2), "unique gem must not be offered");
        assert!(
            !ids.contains(&3),
            "jewelcrafting gem without the profession"
        );
        assert!(ids.contains(&1));
    }

    #[test]
    fn jewelcrafting_gems_appear_for_a_jewelcrafter() {
        let cat = catalog();
        let ids = gem_candidates(gems::RED, 0, &cat, &["Jewelcrafting".to_string()]);
        assert!(ids.contains(&3));
    }

    #[test]
    fn meta_socket_offers_only_meta_gems() {
        let cat = catalog();
        let ids = gem_candidates(gems::META, 0, &cat, &[]);
        assert_eq!(ids, vec![32409]);
    }

    #[test]
    fn the_socketed_gem_is_always_a_candidate() {
        let cat = catalog();
        let ids = gem_candidates(gems::RED, 999, &cat, &[]);
        assert!(
            ids.contains(&999),
            "must be able to keep what is already in"
        );
    }

    #[test]
    fn plan_covers_every_socket_and_the_enchant() {
        let cat = catalog();
        let items = Items(HashMap::from([(100, legs(vec![gems::RED, gems::YELLOW]))]));
        let mut set = GearSet::new();
        set.set(ItemSlot::Legs.index(), Some(spec(100, vec![0, 0], 0)));

        let groups = plan(&set, &items, &cat, &[]);
        let gem_groups = groups
            .iter()
            .filter(|g| matches!(g, Group::Gem { .. }))
            .count();
        let ench_groups = groups
            .iter()
            .filter(|g| matches!(g, Group::Enchant { .. }))
            .count();
        assert_eq!(gem_groups, 2, "one group per socket");
        assert_eq!(ench_groups, 1);
    }

    #[test]
    fn gaps_reports_empty_sockets_and_missing_enchants() {
        let items = Items(HashMap::from([(100, legs(vec![gems::RED, gems::YELLOW]))]));
        let mut set = GearSet::new();
        set.set(ItemSlot::Legs.index(), Some(spec(100, vec![1, 0], 0)));

        let found = gaps(&set, &items, &catalog());
        assert_eq!(found.len(), 2);
        assert!(found[0].what.contains("1 empty socket"));
        assert!(found[1].what.contains("no enchant"));
    }

    #[test]
    fn a_fully_gemmed_enchanted_item_has_no_gaps() {
        let items = Items(HashMap::from([(100, legs(vec![gems::RED]))]));
        let mut set = GearSet::new();
        set.set(ItemSlot::Legs.index(), Some(spec(100, vec![1], 3012)));
        assert!(gaps(&set, &items, &catalog()).is_empty());
    }

    #[test]
    fn light_meta_converts_the_cheapest_gems_until_the_meta_lights() {
        // Relentless Earthstorm needs 2 red, 2 yellow, 2 blue. The set starts
        // all-red, which is exactly what the greedy socket pass produces.
        let mut cat = catalog();
        cat.gems.push(gem(10, gems::YELLOW, 3.0, false, ""));
        cat.gems.push(gem(11, gems::BLUE, 2.0, false, ""));

        let items = Items(HashMap::from([(
            100,
            ItemInfo {
                name: "Helm".into(),
                slots: vec![ItemSlot::Head],
                sockets: vec![
                    gems::META,
                    gems::RED,
                    gems::RED,
                    gems::RED,
                    gems::RED,
                    gems::RED,
                    gems::RED,
                ],
                item_type: 1,
            },
        )]));
        let mut set = GearSet::new();
        set.set(
            ItemSlot::Head.index(),
            Some(spec(100, vec![32409, 1, 1, 1, 1, 1, 1], 0)),
        );

        assert!(!meta_lit(&set, &items, &cat), "starts dark");
        let repaired = light_meta(&set, &items, &cat, &[]).expect("a repair exists");
        assert!(
            meta_lit(&repaired, &items, &cat),
            "meta is lit after repair"
        );

        // Red was already satisfied, so only the yellow and blue shortfalls are
        // paid for: four gems change, no more.
        let before = set.get(ItemSlot::Head.index()).unwrap().gems.clone();
        let after = repaired.get(ItemSlot::Head.index()).unwrap().gems.clone();
        let changed = before.iter().zip(&after).filter(|(a, b)| a != b).count();
        assert_eq!(changed, 4, "two yellow + two blue, nothing gratuitous");
    }

    #[test]
    fn light_meta_leaves_an_already_lit_set_alone() {
        let cat = catalog();
        let items = Items(HashMap::from([(100, legs(vec![gems::RED]))]));
        let mut set = GearSet::new();
        set.set(ItemSlot::Legs.index(), Some(spec(100, vec![1], 0)));
        assert!(light_meta(&set, &items, &cat, &[]).is_none(), "no meta gem");
    }

    #[test]
    fn gaps_calls_out_a_dark_meta_gem() {
        let items = Items(HashMap::from([(100, legs(vec![gems::META, gems::RED]))]));
        let mut set = GearSet::new();
        // Relentless Earthstorm plus one red gem: nowhere near 2/2/2.
        set.set(
            ItemSlot::Legs.index(),
            Some(spec(100, vec![32409, 1], 3012)),
        );

        let found = gaps(&set, &items, &catalog());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].label, "Meta");
        assert!(found[0].what.contains("not active"));
        assert!(found[0].what.contains("2 red, 2 yellow, 2 blue"));
    }
}
