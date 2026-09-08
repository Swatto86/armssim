//! The parsed WowSimsExporter payload.
//!
//! The addon's "Export Everything" (`/wse exportall`) writes a single document
//! where `gear.items` holds the 17 equipped slots followed by the appended
//! bag/bank candidate items — there is no separate `bagItems` key. Empty
//! equipped slots serialize as `null` holes. Older/sample exports instead put
//! candidates in a top-level `bagItems`; both shapes are accepted here.

use anyhow::Context;
use serde::Deserialize;

use crate::domain::gear::{GearSet, ItemSpec, NUM_SLOTS};
use crate::error::ArmssimError;

/// A profession the character has trained.
#[derive(Clone, Debug)]
pub struct Profession {
    pub name: String,
}

/// The fields of the export we use to drive a sim.
#[derive(Clone, Debug)]
pub struct Character {
    pub name: String,
    pub race: String,
    pub talents: String,
    pub professions: Vec<Profession>,
    /// Equipped items in engine slot order; `None` is an empty slot.
    pub gear: Vec<Option<ItemSpec>>,
    /// Bag + bank candidate items.
    pub bag_items: Vec<ItemSpec>,
}

#[derive(Deserialize)]
struct RawProfession {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize, Default)]
struct RawGear {
    /// Tolerates `null` entries for empty equipped slots.
    #[serde(default)]
    items: Vec<Option<ItemSpec>>,
}

#[derive(Deserialize)]
struct RawCharacter {
    #[serde(default)]
    name: String,
    #[serde(default)]
    race: String,
    #[serde(default)]
    class: String,
    #[serde(default)]
    spec: String,
    #[serde(default)]
    talents: String,
    #[serde(default)]
    professions: Vec<RawProfession>,
    #[serde(default)]
    gear: RawGear,
    #[serde(default, rename = "bagItems")]
    bag_items: Vec<ItemSpec>,
}

impl Character {
    /// Decode a WowSimsExporter JSON document (either export shape).
    pub fn parse(data: &[u8]) -> anyhow::Result<Character> {
        let raw: RawCharacter = serde_json::from_slice(data).context("parse exporter json")?;

        // The sim is Arms-warrior-only; reject other classes/specs up front
        // rather than producing a silently-bogus warrior sim. Absent fields
        // (older exports) are treated as a match.
        if !raw.class.is_empty() && !raw.class.eq_ignore_ascii_case("warrior") {
            return Err(ArmssimError::WrongClass(raw.class).into());
        }
        if !raw.spec.is_empty() && !raw.spec.eq_ignore_ascii_case("arms") {
            return Err(ArmssimError::WrongSpec(raw.spec).into());
        }

        // "Export Everything" appends the candidate pool onto gear.items after
        // the 17 equipped slots. If there's no explicit bagItems and gear runs
        // past slot 17, the tail is the candidate pool.
        let mut gear = raw.gear.items;
        let mut bag_items = raw.bag_items;
        if bag_items.is_empty() && gear.len() > NUM_SLOTS {
            bag_items = gear.split_off(NUM_SLOTS).into_iter().flatten().collect();
        }

        if gear.iter().all(Option::is_none) {
            return Err(ArmssimError::NoGear.into());
        }

        Ok(Character {
            name: raw.name,
            race: raw.race,
            talents: raw.talents,
            professions: raw
                .professions
                .into_iter()
                .map(|p| Profession { name: p.name })
                .collect(),
            gear,
            bag_items,
        })
    }

    /// Baseline gear set built from the currently-equipped items.
    pub fn equipped_set(&self) -> GearSet {
        let mut g = GearSet::new();
        for (i, it) in self.gear.iter().enumerate() {
            if i >= NUM_SLOTS {
                break;
            }
            if let Some(spec) = it {
                g.set(i, Some(spec.clone()));
            }
        }
        g
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn items(ids: &[i32]) -> serde_json::Value {
        json!(ids.iter().map(|id| json!({ "id": id })).collect::<Vec<_>>())
    }

    #[test]
    fn splits_combined_gear_items_export_everything() {
        // 17 equipped + 3 appended candidates, no top-level bagItems.
        let ids: Vec<i32> = (1..=20).collect();
        let doc = json!({
            "class": "warrior", "spec": "arms",
            "gear": { "items": items(&ids) }
        });
        let c = Character::parse(doc.to_string().as_bytes()).unwrap();
        assert_eq!(c.gear.len(), NUM_SLOTS);
        assert_eq!(c.gear[0].as_ref().unwrap().id, 1);
        assert_eq!(c.bag_items.len(), 3);
        assert_eq!(c.bag_items[0].id, 18);
    }

    #[test]
    fn keeps_separate_bag_items_when_present() {
        let doc = json!({
            "class": "warrior",
            "gear": { "items": items(&(1..=17).collect::<Vec<_>>()) },
            "bagItems": items(&[101, 102])
        });
        let c = Character::parse(doc.to_string().as_bytes()).unwrap();
        assert_eq!(c.gear.len(), NUM_SLOTS);
        assert_eq!(c.bag_items.len(), 2);
        assert_eq!(c.bag_items[1].id, 102);
    }

    #[test]
    fn tolerates_interior_null_holes() {
        let mut arr: Vec<serde_json::Value> = (1..=17).map(|id| json!({ "id": id })).collect();
        arr[13] = serde_json::Value::Null; // empty Trinket2
        let doc = json!({ "gear": { "items": arr } });
        let c = Character::parse(doc.to_string().as_bytes()).unwrap();
        assert!(c.gear[13].is_none());
        assert!(c.gear[12].is_some());
    }

    #[test]
    fn rejects_non_warrior_class() {
        let doc = json!({ "class": "mage", "gear": { "items": items(&[1, 2]) } });
        assert!(Character::parse(doc.to_string().as_bytes()).is_err());
    }

    #[test]
    fn rejects_non_arms_spec() {
        let doc = json!({ "spec": "fury", "gear": { "items": items(&[1, 2]) } });
        assert!(Character::parse(doc.to_string().as_bytes()).is_err());
    }

    #[test]
    fn empty_gear_is_no_gear() {
        let doc = json!({ "gear": { "items": [] } });
        assert!(Character::parse(doc.to_string().as_bytes()).is_err());
    }
}
