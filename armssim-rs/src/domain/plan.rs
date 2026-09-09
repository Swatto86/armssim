//! Builds the coordinate-ascent search space from the equipped set plus the
//! bag/bank candidate pool: one group per independently-optimized slot (or
//! ring/trinket/weapon pair), each with its mutually-exclusive options.

use std::collections::{HashMap, HashSet};

use super::gear::{GearSet, ItemSlot, ItemSpec};
use super::item::ItemCatalog;

/// One option assigns one or more slots; `None` clears the slot.
pub type Assignment = Vec<(usize, Option<ItemSpec>)>;

/// One independently-optimized decision. `Swap` options are absolute (they name
/// the items outright); the refinement variants are relative to whatever item
/// occupies the slot when the ascent reaches them, which is why they carry ids
/// rather than pre-built assignments.
pub enum Group {
    /// Mutually-exclusive item assignments for a slot or slot pair.
    Swap(Vec<Assignment>),
    /// Candidate gems for one socket of the item in `slot`.
    Gem {
        slot: usize,
        index: usize,
        ids: Vec<i32>,
    },
    /// Candidate enchants for the item in `slot`.
    Enchant { slot: usize, ids: Vec<i32> },
}

impl Group {
    /// Every gear set this group can produce from `current`. Options that would
    /// not change anything are dropped: they would cost a sim to learn nothing.
    pub fn expand(&self, current: &GearSet) -> Vec<GearSet> {
        match self {
            Group::Swap(options) => options.iter().map(|opt| apply(current, opt)).collect(),
            Group::Gem { slot, index, ids } => {
                let Some(spec) = current.get(*slot) else {
                    return Vec::new();
                };
                let socketed = spec.gems.get(*index).copied().unwrap_or(0);
                ids.iter()
                    .filter(|id| **id != socketed)
                    .map(|id| {
                        let mut next = spec.clone();
                        if next.gems.len() <= *index {
                            next.gems.resize(*index + 1, 0);
                        }
                        next.gems[*index] = *id;
                        let mut set = current.clone();
                        set.set(*slot, Some(next));
                        set
                    })
                    .collect()
            }
            Group::Enchant { slot, ids } => {
                let Some(spec) = current.get(*slot) else {
                    return Vec::new();
                };
                ids.iter()
                    .filter(|id| **id != spec.enchant)
                    .map(|id| {
                        let mut next = spec.clone();
                        next.enchant = *id;
                        let mut set = current.clone();
                        set.set(*slot, Some(next));
                        set
                    })
                    .collect()
            }
        }
    }
}

/// The groups to permute, plus diagnostics.
pub struct Plan {
    pub groups: Vec<Group>,
    /// Bag item ids that aren't equippable gear.
    pub skipped: Vec<i32>,
}

/// Single-occupancy slots optimized one at a time (rings/trinkets/weapons are
/// handled as pairs below).
const SINGLE_SLOTS: [ItemSlot; 11] = [
    ItemSlot::Head,
    ItemSlot::Neck,
    ItemSlot::Shoulder,
    ItemSlot::Back,
    ItemSlot::Chest,
    ItemSlot::Wrist,
    ItemSlot::Hands,
    ItemSlot::Waist,
    ItemSlot::Legs,
    ItemSlot::Feet,
    ItemSlot::Ranged,
];

fn sig(spec: Option<&ItemSpec>) -> String {
    match spec {
        None => "-".to_string(),
        Some(s) => s.sig(),
    }
}

/// Filter to real items (`Some`, id > 0), capping duplicate signatures at 2 —
/// a pair never needs more than two physical copies of the same item, and the
/// cap is what lets `add_pair_group` offer a same-item self-pair when the
/// player genuinely owns two.
fn dedupe<I: IntoIterator<Item = Option<ItemSpec>>>(specs: I) -> Vec<ItemSpec> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut out = Vec::new();
    for s in specs {
        let Some(spec) = s else { continue };
        if spec.id == 0 {
            continue;
        }
        let count = counts.entry(spec.sig()).or_insert(0);
        if *count < 2 {
            *count += 1;
            out.push(spec);
        }
    }
    out
}

impl Plan {
    pub fn build(equipped: &GearSet, bag: &[ItemSpec], catalog: &dyn ItemCatalog) -> Plan {
        let mut skipped = Vec::new();

        // Classify the candidate pool into per-slot buckets.
        let mut by_slot: HashMap<usize, Vec<ItemSpec>> = HashMap::new();
        let mut ring_pool = Vec::new();
        let mut trinket_pool = Vec::new();
        // Two-handed main-hand candidates. Arms wields a 2H, so there is no
        // off-hand pool and no pairing to search: the off-hand is cleared by
        // every weapon option.
        let mut mh_pool = Vec::new();

        for it in bag {
            let Some(info) = catalog.lookup(it.id) else {
                skipped.push(it.id);
                continue;
            };
            let has = |s: ItemSlot| info.slots.contains(&s);
            if has(ItemSlot::Finger1) {
                ring_pool.push(it.clone());
            } else if has(ItemSlot::Trinket1) {
                trinket_pool.push(it.clone());
            } else if has(ItemSlot::MainHand) {
                mh_pool.push(it.clone());
            } else {
                by_slot
                    .entry(info.slots[0].index())
                    .or_default()
                    .push(it.clone());
            }
        }
        let mut groups = Vec::new();

        // Single-slot groups: current item first, then deduped bag candidates.
        for slot in SINGLE_SLOTS {
            let idx = slot.index();
            let mut seen = HashSet::new();
            let mut options: Vec<Assignment> = Vec::new();
            let mut add = |value: Option<ItemSpec>, seen: &mut HashSet<String>| {
                if seen.insert(sig(value.as_ref())) {
                    options.push(vec![(idx, value)]);
                }
            };
            add(equipped.get(idx).cloned(), &mut seen);
            for spec in by_slot.get(&idx).into_iter().flatten() {
                add(Some(spec.clone()), &mut seen);
            }
            if options.len() > 1 {
                groups.push(Group::Swap(options));
            }
        }

        // Paired groups: rings, trinkets, weapons.
        let ring_pool = dedupe(
            [equipped.get(10).cloned(), equipped.get(11).cloned()]
                .into_iter()
                .chain(ring_pool.into_iter().map(Some)),
        );
        add_pair_group(
            &mut groups,
            ItemSlot::Finger1,
            ItemSlot::Finger2,
            &ring_pool,
        );

        let trinket_pool = dedupe(
            [equipped.get(12).cloned(), equipped.get(13).cloned()]
                .into_iter()
                .chain(trinket_pool.into_iter().map(Some)),
        );
        add_pair_group(
            &mut groups,
            ItemSlot::Trinket1,
            ItemSlot::Trinket2,
            &trinket_pool,
        );

        // The equipped main hand only joins the pool if it is itself a 2H —
        // an export taken in dual-wield gear must not seed a one-hander into
        // an Arms search.
        let equipped_mh = equipped
            .get(14)
            .filter(|spec| is_two_hander(catalog, spec.id))
            .cloned();
        let equipped_seeded = equipped_mh.is_some();
        let mh = dedupe(
            [equipped_mh]
                .into_iter()
                .chain(mh_pool.into_iter().map(Some)),
        );
        add_weapon_group(&mut groups, &mh, equipped_seeded);

        Plan { groups, skipped }
    }
}

/// Unordered distinct pairs across two slots (rings/trinkets). `pool` may
/// hold the same signature twice (see `dedupe`) — that's what lets a
/// self-pair (both slots holding the same item) appear when two physical
/// copies exist. `seen` collapses the rest of the pool's duplicate copies
/// down to one option per distinct unordered pair (ring1/ring2 order doesn't
/// matter, so the sorted signature pair is the right dedup key).
fn add_pair_group(groups: &mut Vec<Group>, slot_a: ItemSlot, slot_b: ItemSlot, pool: &[ItemSpec]) {
    let mut options: Vec<Assignment> = Vec::new();
    let mut seen = HashSet::new();
    for i in 0..pool.len() {
        for j in (i + 1)..pool.len() {
            let mut key = [pool[i].sig(), pool[j].sig()];
            key.sort();
            if !seen.insert(key) {
                continue;
            }
            options.push(vec![
                (slot_a.index(), Some(pool[i].clone())),
                (slot_b.index(), Some(pool[j].clone())),
            ]);
        }
    }
    if options.len() > 1 {
        groups.push(Group::Swap(options));
    }
}

/// Whether `id` is a two-handed weapon in the engine's item database — the
/// only weapon an Arms search will equip.
pub fn is_two_hander(catalog: &dyn ItemCatalog, id: i32) -> bool {
    catalog
        .lookup(id)
        .is_some_and(|info| info.slots.contains(&ItemSlot::MainHand))
}

/// Two-handed main-hand options. Unlike the Fury sibling of this tool there is
/// no pairing to search: each option equips one 2H and clears the off-hand, so
/// an export taken in dual-wield gear converges on a legal Arms setup.
fn add_weapon_group(groups: &mut Vec<Group>, mh: &[ItemSpec], equipped_seeded: bool) {
    let mut options: Vec<Assignment> = Vec::new();
    let mut seen = HashSet::new();
    for m in mh {
        if !seen.insert(m.sig()) {
            continue;
        }
        options.push(vec![
            (ItemSlot::MainHand.index(), Some(m.clone())),
            (ItemSlot::OffHand.index(), None),
        ]);
    }
    // A group is only worth a sim if it can actually change the gear. When the
    // equipped two-hander leads the pool, one option is just the current
    // weapon; when the export is dual-wield nothing was seeded, so a single
    // bag two-hander is the only Arms-legal choice and must still be searched.
    let minimum = if equipped_seeded { 2 } else { 1 };
    if options.len() >= minimum {
        groups.push(Group::Swap(options));
    }
}

/// Apply an assignment on top of a base set, producing a candidate.
pub fn apply(base: &GearSet, assignment: &Assignment) -> GearSet {
    let mut g = base.clone();
    for (slot, spec) in assignment {
        g.set(*slot, spec.clone());
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::ItemInfo;

    struct FakeCatalog(HashMap<i32, ItemInfo>);
    impl ItemCatalog for FakeCatalog {
        fn lookup(&self, id: i32) -> Option<ItemInfo> {
            self.0.get(&id).cloned()
        }
    }

    fn spec(id: i32) -> ItemSpec {
        ItemSpec {
            id,
            enchant: 0,
            gems: vec![],
            random_suffix: 0,
        }
    }

    fn ring_info() -> ItemInfo {
        ItemInfo {
            name: "ring".to_string(),
            slots: vec![ItemSlot::Finger1, ItemSlot::Finger2],
            sockets: vec![],
            item_type: 11,
        }
    }

    fn two_hand_info() -> ItemInfo {
        ItemInfo {
            name: "two-hander".to_string(),
            slots: vec![ItemSlot::MainHand],
            sockets: vec![],
            item_type: 13,
        }
    }

    fn ring_group_options(plan: &Plan) -> &[Assignment] {
        plan.groups
            .iter()
            .find(|g| group_touches(g, ItemSlot::Finger1))
            .map(options_of)
            .expect("ring group exists")
    }

    /// Item-swap groups are the only kind `Plan::build` produces; refinement
    /// groups are added later, from a chosen set.
    fn options_of(group: &Group) -> &[Assignment] {
        match group {
            Group::Swap(options) => options,
            _ => panic!("expected an item-swap group"),
        }
    }

    fn group_touches(group: &Group, slot: ItemSlot) -> bool {
        matches!(group, Group::Swap(options)
            if options.iter().any(|opt| opt.iter().any(|(s, _)| *s == slot.index())))
    }

    fn ids_of(opt: &Assignment) -> Vec<i32> {
        opt.iter()
            .filter_map(|(_, s)| s.as_ref().map(|s| s.id))
            .collect()
    }

    #[test]
    fn ring_pair_offers_self_pair_with_two_bag_copies() {
        let catalog = FakeCatalog(HashMap::from([(100, ring_info()), (200, ring_info())]));
        let equipped = GearSet::new();
        let bag = vec![spec(100), spec(100), spec(200)];

        let plan = Plan::build(&equipped, &bag, &catalog);
        let options = ring_group_options(&plan);
        assert!(
            options.iter().any(|opt| ids_of(opt) == [100, 100]),
            "expected a 100/100 self-pair when two physical copies exist"
        );
    }

    #[test]
    fn ring_pair_rejects_self_pair_with_one_copy() {
        // Three distinct rings so the group has more than one option (a
        // group with exactly one possible pairing isn't worth searching and
        // is dropped) while still holding only one copy of ring 100.
        let catalog = FakeCatalog(HashMap::from([
            (100, ring_info()),
            (200, ring_info()),
            (300, ring_info()),
        ]));
        let equipped = GearSet::new();
        let bag = vec![spec(100), spec(200), spec(300)];

        let plan = Plan::build(&equipped, &bag, &catalog);
        let options = ring_group_options(&plan);
        assert!(
            !options.iter().any(|opt| ids_of(opt) == [100, 100]),
            "should not offer a self-pair without two physical copies"
        );
    }

    fn weapon_group(plan: &Plan) -> &[Assignment] {
        plan.groups
            .iter()
            .find(|g| group_touches(g, ItemSlot::MainHand))
            .map(options_of)
            .expect("weapon group exists")
    }

    #[test]
    fn weapon_group_offers_each_two_hander_and_clears_the_off_hand() {
        let catalog = FakeCatalog(HashMap::from([
            (100, two_hand_info()),
            (200, two_hand_info()),
        ]));
        let equipped = GearSet::new();
        let bag = vec![spec(100), spec(200)];

        let plan = Plan::build(&equipped, &bag, &catalog);
        let options = weapon_group(&plan);
        assert_eq!(options.len(), 2);
        for opt in options {
            assert_eq!(ids_of(opt).len(), 1, "exactly one weapon per option");
            let off_hand = opt
                .iter()
                .find(|(slot, _)| *slot == ItemSlot::OffHand.index())
                .expect("every weapon option assigns the off-hand");
            assert!(off_hand.1.is_none(), "off-hand must be cleared for Arms");
        }
    }

    #[test]
    fn weapon_group_ignores_one_handers() {
        // A one-hander is not in the catalog's main-hand slot list at all
        // (see domain::item::weapon_slots), so it is skipped as non-gear and
        // never reaches the weapon group.
        let catalog = FakeCatalog(HashMap::from([(100, two_hand_info())]));
        let equipped = GearSet::new();
        let bag = vec![spec(100), spec(999)];

        let plan = Plan::build(&equipped, &bag, &catalog);
        assert!(plan.skipped.contains(&999));
        assert!(
            plan.groups.iter().all(|g| {
                options_of(g).iter().all(|opt| {
                    opt.iter()
                        .all(|(_, s)| s.as_ref().is_none_or(|s| s.id != 999))
                })
            }),
            "a one-hander must never be offered"
        );
    }

    #[test]
    fn lone_bag_two_hander_is_offered_when_the_equipped_main_hand_is_a_one_hander() {
        // Dual-wield export plus exactly one 2H in the bags: the equipped
        // one-hander is deliberately not seeded, so that single bag weapon is
        // the only Arms-legal option and must still be searched.
        let catalog = FakeCatalog(HashMap::from([
            (100, two_hand_info()),
            (999, ring_info()), // stands in for a non-2H equipped main hand
        ]));
        let mut equipped = GearSet::new();
        equipped.set(ItemSlot::MainHand.index(), Some(spec(999)));
        let bag = vec![spec(100)];

        let plan = Plan::build(&equipped, &bag, &catalog);
        let options = weapon_group(&plan);
        assert_eq!(options.len(), 1, "the lone two-hander must be an option");
        assert_eq!(ids_of(&options[0]), [100]);
    }

    #[test]
    fn equipped_one_hander_is_not_seeded_into_the_weapon_pool() {
        let catalog = FakeCatalog(HashMap::from([
            (100, two_hand_info()),
            (200, two_hand_info()),
            (999, ring_info()), // stands in for a non-2H equipped main hand
        ]));
        let mut equipped = GearSet::new();
        equipped.set(ItemSlot::MainHand.index(), Some(spec(999)));
        let bag = vec![spec(100), spec(200)];

        let plan = Plan::build(&equipped, &bag, &catalog);
        let options = weapon_group(&plan);
        assert_eq!(options.len(), 2);
        assert!(
            !options.iter().any(|opt| ids_of(opt).contains(&999)),
            "the equipped one-hander must not become an Arms candidate"
        );
    }
}
