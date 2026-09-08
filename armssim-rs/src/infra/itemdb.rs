//! Item database loaded from the engine's `db.json`, implementing the domain's
//! [`ItemCatalog`] contract.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;

use crate::domain::item::{slots_for, ItemCatalog, ItemInfo};

struct Entry {
    name: String,
    item_type: i32,
    hand_type: i32,
}

/// In-memory item lookup keyed by item id.
pub struct DbCatalog {
    by_id: HashMap<i32, Entry>,
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
}

#[derive(Deserialize)]
struct Db {
    #[serde(default)]
    items: Vec<DbItem>,
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
                    },
                )
            })
            .collect();
        Ok(DbCatalog { by_id })
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
        })
    }
}
