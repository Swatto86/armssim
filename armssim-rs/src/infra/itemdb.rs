//! Item, gem and enchant data loaded from the engine's `db.json`, implementing
//! the domain's [`ItemCatalog`] and [`RefineCatalog`] contracts.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;

use crate::domain::item::{slots_for, ItemCatalog, ItemInfo};
use crate::domain::refine::{EnchantInfo, GemInfo, RefineCatalog};

struct Entry {
    name: String,
    item_type: i32,
    hand_type: i32,
    sockets: Vec<i32>,
}

/// In-memory lookups keyed by item id, gem id, and (enchant effect, slot type).
pub struct DbCatalog {
    by_id: HashMap<i32, Entry>,
    gems: Vec<GemInfo>,
    gems_by_id: HashMap<i32, usize>,
    enchants: Vec<EnchantInfo>,
    enchants_by_key: HashMap<(i32, i32), usize>,
}

#[derive(Deserialize)]
struct DbItem {
    id: i32,
    #[serde(default)]
    name: String,
    #[serde(default, rename = "type")]
    item_type: i32,
    #[serde(default, rename = "handType")]
    hand_type: i32,
    #[serde(default, rename = "gemSockets")]
    gem_sockets: Vec<i32>,
}

#[derive(Deserialize)]
struct DbGem {
    id: i32,
    #[serde(default)]
    name: String,
    #[serde(default)]
    color: i32,
    #[serde(default)]
    stats: Vec<f64>,
    #[serde(default)]
    unique: bool,
    /// Engine `Profession` enum value; 0 (or absent) means anyone can use it.
    #[serde(default, rename = "requiredProfession")]
    required_profession: i32,
}

/// Engine `Profession` enum → the name the character export uses.
fn profession_name(value: i32) -> String {
    match value {
        1 => "Alchemy",
        2 => "Blacksmithing",
        3 => "Enchanting",
        4 => "Engineering",
        5 => "Herbalism",
        6 => "Inscription",
        7 => "Jewelcrafting",
        8 => "Leatherworking",
        9 => "Mining",
        10 => "Skinning",
        11 => "Tailoring",
        _ => "",
    }
    .to_string()
}

#[derive(Deserialize)]
struct DbEnchant {
    #[serde(default, rename = "effectId")]
    effect_id: i32,
    #[serde(default)]
    name: String,
    #[serde(default, rename = "type")]
    item_type: i32,
    /// `1` marks a two-hand-only weapon enchant; absent otherwise.
    #[serde(default, rename = "enchantType")]
    enchant_type: i32,
}

#[derive(Deserialize)]
struct Db {
    #[serde(default)]
    items: Vec<DbItem>,
    #[serde(default)]
    gems: Vec<DbGem>,
    #[serde(default)]
    enchants: Vec<DbEnchant>,
}

impl DbCatalog {
    pub fn load(path: &Path) -> anyhow::Result<DbCatalog> {
        let data = std::fs::read(path)
            .with_context(|| format!("read item database {}", path.display()))?;
        let db: Db = serde_json::from_slice(&data).context("parse db.json")?;

        let by_id = db
            .items
            .into_iter()
            .map(|it| {
                (
                    it.id,
                    Entry {
                        name: it.name,
                        item_type: it.item_type,
                        hand_type: it.hand_type,
                        sockets: it.gem_sockets,
                    },
                )
            })
            .collect();

        let gems: Vec<GemInfo> = db
            .gems
            .into_iter()
            .map(|g| GemInfo {
                id: g.id,
                name: g.name,
                color: g.color,
                stats: g.stats,
                unique: g.unique,
                profession: profession_name(g.required_profession),
            })
            .collect();
        let gems_by_id = gems.iter().enumerate().map(|(i, g)| (g.id, i)).collect();

        // An effect id is reused across slots (2564 is both a glove and a weapon
        // enchant), so the key is (effect, item type).
        let enchants: Vec<EnchantInfo> = db
            .enchants
            .into_iter()
            .map(|e| EnchantInfo {
                effect_id: e.effect_id,
                item_type: e.item_type,
                name: e.name,
                two_hand_only: e.enchant_type == 1,
            })
            .collect();
        let enchants_by_key = enchants
            .iter()
            .enumerate()
            .map(|(i, e)| ((e.effect_id, e.item_type), i))
            .collect();

        Ok(DbCatalog {
            by_id,
            gems,
            gems_by_id,
            enchants,
            enchants_by_key,
        })
    }

    /// Gem colours for every gem in the database, for meta-gem activation.
    pub fn gem_colors(&self) -> HashMap<i32, i32> {
        self.gems.iter().map(|g| (g.id, g.color)).collect()
    }
}

impl ItemCatalog for DbCatalog {
    fn lookup(&self, id: i32) -> Option<ItemInfo> {
        let entry = self.by_id.get(&id)?;
        let slots = slots_for(entry.item_type, entry.hand_type);
        if slots.is_empty() {
            return None;
        }
        Some(ItemInfo {
            name: entry.name.clone(),
            slots,
            sockets: entry.sockets.clone(),
            item_type: entry.item_type,
        })
    }
}

impl RefineCatalog for DbCatalog {
    fn gem(&self, id: i32) -> Option<&GemInfo> {
        self.gems_by_id.get(&id).map(|i| &self.gems[*i])
    }

    fn all_gems(&self) -> &[GemInfo] {
        &self.gems
    }

    fn enchant(&self, effect_id: i32, item_type: i32) -> Option<&EnchantInfo> {
        self.enchants_by_key
            .get(&(effect_id, item_type))
            .map(|i| &self.enchants[*i])
    }
}
