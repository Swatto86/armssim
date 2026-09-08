//! Diffing a candidate set against the baseline into a human-readable list of
//! slot changes. Rings and trinkets are compared as unordered pairs, so a mere
//! ring1<->ring2 reorder is not reported.

use super::gear::{GearSet, ItemSlot, ItemSpec};
use super::item::ItemCatalog;

/// A single slot (or ring/trinket pair member) differing from the baseline.
pub struct SlotChange {
    pub label: String,
    pub from: String,
    pub to: String,
}

/// Slots compared individually (rings/trinkets handled as pairs).
const SINGLE_SLOTS: [ItemSlot; 13] = [
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
    ItemSlot::MainHand,
    ItemSlot::OffHand,
    ItemSlot::Ranged,
];

fn item_name(spec: Option<&ItemSpec>, catalog: &dyn ItemCatalog) -> String {
    match spec {
        None => "(empty)".to_string(),
        Some(s) if s.id == 0 => "(empty)".to_string(),
        Some(s) => catalog
            .lookup(s.id)
            .map(|info| info.name)
            .unwrap_or_else(|| format!("item {}", s.id)),
    }
}

fn id_of(spec: Option<&ItemSpec>) -> i32 {
    spec.map_or(0, |s| s.id)
}

pub fn diff(base: &GearSet, candidate: &GearSet, catalog: &dyn ItemCatalog) -> Vec<SlotChange> {
    let mut changes = Vec::new();

    for slot in SINGLE_SLOTS {
        let i = slot.index();
        if id_of(base.get(i)) != id_of(candidate.get(i)) {
            changes.push(SlotChange {
                label: slot.label().to_string(),
                from: item_name(base.get(i), catalog),
                to: item_name(candidate.get(i), catalog),
            });
        }
    }

    changes.extend(pair_diff("Ring", base, candidate, 10, 11, catalog));
    changes.extend(pair_diff("Trinket", base, candidate, 12, 13, catalog));
    changes
}

/// Treat two slots as an unordered multiset: report only the items that
/// genuinely left or entered the pair, pairing each removal with an
/// addition. Matches by id (not by pool position), so going from two
/// distinct items to two copies of one of them is reported as a single real
/// swap rather than one item vanishing into "(empty)".
fn pair_diff(
    label: &str,
    base: &GearSet,
    candidate: &GearSet,
    slot_a: usize,
    slot_b: usize,
    catalog: &dyn ItemCatalog,
) -> Vec<SlotChange> {
    let base_items = [base.get(slot_a), base.get(slot_b)];
    let cand_items = [candidate.get(slot_a), candidate.get(slot_b)];

    let mut cand_remaining: Vec<usize> = (0..cand_items.len()).collect();
    let mut removed = Vec::new();
    for &b in &base_items {
        let bid = id_of(b);
        match cand_remaining
            .iter()
            .position(|&j| id_of(cand_items[j]) == bid)
        {
            Some(pos) => {
                cand_remaining.remove(pos);
            }
            None => removed.push(b),
        }
    }
    let added: Vec<Option<&ItemSpec>> = cand_remaining.into_iter().map(|j| cand_items[j]).collect();

    let mut changes = Vec::new();
    for i in 0..removed.len().max(added.len()) {
        let from = removed.get(i).copied().flatten();
        let to = added.get(i).copied().flatten();
        changes.push(SlotChange {
            label: label.to_string(),
            from: item_name(from, catalog),
            to: item_name(to, catalog),
        });
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::ItemInfo;
    use std::collections::HashMap;

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

    fn named(name: &str) -> ItemInfo {
        ItemInfo {
            name: name.to_string(),
            slots: vec![],
        }
    }

    #[test]
    fn pair_diff_reports_real_swap_not_phantom_removal_when_duplicating() {
        let catalog = FakeCatalog(HashMap::from([
            (100, named("Ring A")),
            (200, named("Ring B")),
        ]));
        let mut base = GearSet::new();
        base.set(10, Some(spec(100)));
        base.set(11, Some(spec(200)));
        let mut candidate = GearSet::new();
        candidate.set(10, Some(spec(100)));
        candidate.set(11, Some(spec(100)));

        let changes = pair_diff("Ring", &base, &candidate, 10, 11, &catalog);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].from, "Ring B");
        assert_eq!(changes[0].to, "Ring A");
    }

    #[test]
    fn pair_diff_no_change_when_duplicate_pair_is_unchanged() {
        let catalog = FakeCatalog(HashMap::from([(100, named("Ring A"))]));
        let mut base = GearSet::new();
        base.set(10, Some(spec(100)));
        base.set(11, Some(spec(100)));
        let candidate = base.clone();

        let changes = pair_diff("Ring", &base, &candidate, 10, 11, &catalog);
        assert!(changes.is_empty());
    }
}
