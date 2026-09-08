//! Core gear value types: an item specification, the 17-slot equipment layout,
//! and the two encounter scenarios the optimizer evaluates.

use serde::Deserialize;

/// Number of equipment slots the engine models (0..=16).
pub const NUM_SLOTS: usize = 17;

/// An equippable item with its enchant, gems and random suffix, as exported by
/// WowSimsExporter. The addon uses snake_case keys (`random_suffix`).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ItemSpec {
    pub id: i32,
    #[serde(default)]
    pub enchant: i32,
    #[serde(default)]
    pub gems: Vec<i32>,
    /// Random suffix id ("...of the Bear"); 0 / absent for fixed items. The
    /// engine derives different stats from it, so it must round-trip.
    #[serde(default, rename = "random_suffix")]
    pub random_suffix: i32,
}

impl ItemSpec {
    /// Stable signature used to deduplicate candidate items. Includes the random
    /// suffix so two suffix variants of the same base id stay distinct.
    pub fn sig(&self) -> String {
        format!(
            "{}/{}/{}/{:?}",
            self.id, self.random_suffix, self.enchant, self.gems
        )
    }
}

/// A single equipment slot. Discriminants match the engine's `ItemSlot` enum so
/// the value doubles as the index into a [`GearSet`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemSlot {
    Head = 0,
    Neck = 1,
    Shoulder = 2,
    Back = 3,
    Chest = 4,
    Wrist = 5,
    Hands = 6,
    Waist = 7,
    Legs = 8,
    Feet = 9,
    Finger1 = 10,
    Finger2 = 11,
    Trinket1 = 12,
    Trinket2 = 13,
    MainHand = 14,
    OffHand = 15,
    Ranged = 16,
}

impl ItemSlot {
    pub fn index(self) -> usize {
        self as usize
    }

    /// Human-readable label used in the change report.
    pub fn label(self) -> &'static str {
        use ItemSlot::*;
        match self {
            Head => "Head",
            Neck => "Neck",
            Shoulder => "Shoulder",
            Back => "Back",
            Chest => "Chest",
            Wrist => "Wrist",
            Hands => "Hands",
            Waist => "Waist",
            Legs => "Legs",
            Feet => "Feet",
            Finger1 | Finger2 => "Ring",
            Trinket1 | Trinket2 => "Trinket",
            MainHand => "MainHand",
            OffHand => "OffHand",
            Ranged => "Ranged",
        }
    }
}

/// An equipment configuration indexed by [`ItemSlot`]. An empty slot is `None`.
#[derive(Clone, Debug, Default)]
pub struct GearSet {
    slots: [Option<ItemSpec>; NUM_SLOTS],
}

impl GearSet {
    pub fn new() -> Self {
        GearSet {
            slots: std::array::from_fn(|_| None),
        }
    }

    pub fn get(&self, slot: usize) -> Option<&ItemSpec> {
        self.slots[slot].as_ref()
    }

    pub fn set(&mut self, slot: usize, spec: Option<ItemSpec>) {
        self.slots[slot] = spec;
    }
}

/// Encounter shape: single boss vs. the engine's standard ~20-target pull.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenario {
    SingleTarget,
    Aoe,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_snake_case_random_suffix() {
        let it: ItemSpec =
            serde_json::from_str(r#"{"id":123,"enchant":5,"gems":[1,2],"random_suffix":-19}"#)
                .unwrap();
        assert_eq!(it.id, 123);
        assert_eq!(it.random_suffix, -19);
    }

    #[test]
    fn random_suffix_defaults_to_zero_when_absent() {
        let it: ItemSpec = serde_json::from_str(r#"{"id":1}"#).unwrap();
        assert_eq!(it.random_suffix, 0);
    }

    #[test]
    fn sig_distinguishes_suffix_variants_of_same_base() {
        let a: ItemSpec = serde_json::from_str(r#"{"id":1,"random_suffix":-19}"#).unwrap();
        let b: ItemSpec = serde_json::from_str(r#"{"id":1,"random_suffix":-20}"#).unwrap();
        assert_ne!(a.sig(), b.sig());
    }
}
