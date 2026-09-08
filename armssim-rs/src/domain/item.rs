//! Item classification: maps an item's engine type/hand-type to the equipment
//! slots it can occupy, and defines the catalog contract the optimizer depends
//! on. Mirrors the engine's `slotsFor`/`weaponSlots` rules.

use super::gear::ItemSlot;

/// Where an item can be equipped, plus its display name.
#[derive(Clone, Debug)]
pub struct ItemInfo {
    pub name: String,
    /// Every slot this item can occupy (e.g. a ring → Finger1 + Finger2).
    pub slots: Vec<ItemSlot>,
}

/// Slots for an item given its engine `ItemType` and `HandType` integer codes.
/// Returns empty for non-gear (reagents, quest items) and for one-handers,
/// which this two-handed Arms optimizer ignores.
pub fn slots_for(item_type: i32, hand_type: i32) -> Vec<ItemSlot> {
    use ItemSlot::*;
    match item_type {
        1 => vec![Head],
        2 => vec![Neck],
        3 => vec![Shoulder],
        4 => vec![Back],
        5 => vec![Chest],
        6 => vec![Wrist],
        7 => vec![Hands],
        8 => vec![Waist],
        9 => vec![Legs],
        10 => vec![Feet],
        11 => vec![Finger1, Finger2],
        12 => vec![Trinket1, Trinket2],
        13 => weapon_slots(hand_type),
        14 => vec![Ranged],
        _ => vec![],
    }
}

/// Hand slots a weapon can occupy. Arms is a two-handed spec — Mortal Strike,
/// Slam and the 2H talents (Impale, Two-Handed Weapon Specialization, Poleaxe
/// Specialization) all assume a 2H main hand — so only `HandTypeTwoHand`
/// weapons are candidates and the off-hand is always empty. One-handers and
/// off-hands are deliberately excluded.
fn weapon_slots(hand_type: i32) -> Vec<ItemSlot> {
    use ItemSlot::*;
    match hand_type {
        4 => vec![MainHand], // HandTypeTwoHand
        _ => vec![],         // one-hand / main-hand / off-hand / unknown
    }
}

/// Read-only item database the planner and report query. Implemented by the
/// infrastructure layer over the engine's `db.json`.
pub trait ItemCatalog {
    /// Slot info for an equippable item; `None` if absent or not gear.
    fn lookup(&self, id: i32) -> Option<ItemInfo>;
}
