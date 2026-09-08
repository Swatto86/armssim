//! Assembles a `RaidSimRequest` (engine protojson) from a parsed character and
//! a candidate equipment set. The buff/debuff/consume/encounter values mirror
//! the engine's own full-raid defaults so DPS lines up with the WoWSims site;
//! the rotation is the engine's Arms APL, embedded verbatim.

use serde_json::{json, Value};

use crate::app::character::Character;
use crate::domain::gear::{GearSet, ItemSpec, Scenario, NUM_SLOTS};
use crate::error::ArmssimError;

/// The engine's Arms APL, embedded at build time.
const ARMS_APL: &str = include_str!("arms.apl.json");

/// Builds sim requests for one character; the static parts (buffs, rotation,
/// player options, encounters) are assembled once and reused per gear set.
pub struct RequestBuilder {
    player_base: Value,
    st_encounter: Value,
    aoe_encounter: Value,
}

impl RequestBuilder {
    pub fn new(character: &Character) -> anyhow::Result<RequestBuilder> {
        let race = race_proto(&character.race)?;
        let (p1, p2) = professions(&character.professions);
        let rotation: Value = serde_json::from_str(ARMS_APL).expect("embedded arms APL is valid");

        let player_base = json!({
            "name": character.name,
            "race": race,
            "class": "ClassWarrior",
            "consumables": consumes(),
            "buffs": individual_buffs(),
            "dpsWarrior": {
                "options": {
                    "classOptions": {
                        "defaultShout": "WarriorShoutBattle",
                        "defaultStance": "WarriorStanceBerserker",
                        "startingRage": 50.0,
                        "queueDelay": 250,
                        "stanceSnapshot": true,
                        "hasBsT2": true
                    }
                }
            },
            "talentsString": character.talents,
            "profession1": p1,
            "profession2": p2,
            "rotation": rotation,
            "reactionTimeMs": 200,
            "channelClipDelayMs": 100,
            "inFrontOfTarget": false,
            "distanceFromTarget": 5.0
        });

        Ok(RequestBuilder {
            player_base,
            st_encounter: single_target_encounter(),
            aoe_encounter: multi_target_encounter(),
        })
    }

    /// Assemble a full request for `set` in the given scenario.
    pub fn build(&self, set: &GearSet, scenario: Scenario, iterations: u32, seed: i64) -> Value {
        let mut player = self.player_base.clone();
        player["equipment"] = json!({ "items": equipment_items(set) });

        let encounter = match scenario {
            Scenario::SingleTarget => self.st_encounter.clone(),
            Scenario::Aoe => self.aoe_encounter.clone(),
        };

        json!({
            "raid": {
                "parties": [{ "players": [player], "buffs": party_buffs() }],
                "buffs": raid_buffs(),
                "debuffs": debuffs()
            },
            "encounter": encounter,
            "simOptions": { "iterations": iterations, "randomSeed": seed }
        })
    }
}

/// Render the 17 slots as engine `ItemSpec`s; an empty slot becomes `{}`.
fn equipment_items(set: &GearSet) -> Vec<Value> {
    (0..NUM_SLOTS)
        .map(|i| match set.get(i) {
            Some(spec) if spec.id != 0 => item_spec(spec),
            _ => json!({}),
        })
        .collect()
}

fn item_spec(spec: &ItemSpec) -> Value {
    let mut v = json!({ "id": spec.id });
    if spec.enchant != 0 {
        v["enchant"] = json!(spec.enchant);
    }
    if !spec.gems.is_empty() {
        v["gems"] = json!(spec.gems);
    }
    if spec.random_suffix != 0 {
        // Engine proto field `random_suffix` → protojson `randomSuffix`.
        v["randomSuffix"] = json!(spec.random_suffix);
    }
    v
}

fn race_proto(race: &str) -> anyhow::Result<&'static str> {
    let key: String = race
        .to_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let name = match key.as_str() {
        "human" => "RaceHuman",
        "orc" => "RaceOrc",
        "dwarf" => "RaceDwarf",
        "nightelf" => "RaceNightElf",
        "gnome" => "RaceGnome",
        "draenei" => "RaceDraenei",
        "tauren" => "RaceTauren",
        "troll" => "RaceTroll",
        "undead" | "scourge" => "RaceUndead",
        "bloodelf" => "RaceBloodElf",
        _ => return Err(ArmssimError::UnsupportedRace(race.to_string()).into()),
    };
    Ok(name)
}

fn profession_proto(name: &str) -> &'static str {
    match name.to_lowercase().as_str() {
        "alchemy" => "Alchemy",
        "blacksmithing" => "Blacksmithing",
        "enchanting" => "Enchanting",
        "engineering" => "Engineering",
        "herbalism" => "Herbalism",
        "jewelcrafting" => "Jewelcrafting",
        "leatherworking" => "Leatherworking",
        "mining" => "Mining",
        "skinning" => "Skinning",
        "tailoring" => "Tailoring",
        _ => "ProfessionUnknown",
    }
}

fn professions(profs: &[crate::app::character::Profession]) -> (&'static str, &'static str) {
    let p1 = profs
        .first()
        .map_or("ProfessionUnknown", |p| profession_proto(&p.name));
    let p2 = profs
        .get(1)
        .map_or("ProfessionUnknown", |p| profession_proto(&p.name));
    (p1, p2)
}

// --- Engine full-raid defaults (mirrors core.Full*Buffs / defaultConsumes) ---

fn raid_buffs() -> Value {
    json!({
        "bloodlust": true,
        "arcaneBrilliance": true,
        "powerWordFortitude": "TristateEffectImproved",
        "divineSpirit": "TristateEffectImproved",
        "giftOfTheWild": "TristateEffectImproved",
        "thorns": "TristateEffectImproved",
        "shadowProtection": true
    })
}

fn party_buffs() -> Value {
    json!({
        "ferociousInspiration": 1,
        "bloodPact": "TristateEffectImproved",
        "moonkinAura": "TristateEffectImproved",
        "leaderOfThePack": "TristateEffectImproved",
        "sanctityAura": "TristateEffectImproved",
        "devotionAura": "TristateEffectImproved",
        "retributionAura": "TristateEffectImproved",
        "concentrationAura": "TristateEffectImproved",
        "trueshotAura": true,
        "draeneiRacialMelee": true,
        "draeneiRacialCaster": true,
        "atieshDruid": 1,
        "atieshMage": 1,
        "atieshPriest": 1,
        "atieshWarlock": 1,
        "braidedEterniumChain": true,
        "eyeOfTheNight": true,
        "chainOfTheTwilightOwl": true,
        "jadePendantOfBlasting": true,
        "totemTwisting": true,
        "manaSpringTotem": "TristateEffectImproved",
        "manaTideTotems": 1,
        "totemOfWrath": 1,
        "wrathOfAirTotem": "TristateEffectImproved",
        "graceOfAirTotem": "TristateEffectImproved",
        "strengthOfEarthTotem": "TristateEffectImproved",
        "windfuryTotem": "TristateEffectImproved",
        "battleShout": "TristateEffectImproved",
        "commandingShout": "TristateEffectImproved",
        "drums": "LesserDrumsOfBattle"
    })
}

fn individual_buffs() -> Value {
    json!({
        "blessingOfKings": true,
        "blessingOfSanctuary": true,
        "blessingOfWisdom": "TristateEffectImproved",
        "blessingOfMight": "TristateEffectImproved",
        "unleashedRage": true
    })
}

fn debuffs() -> Value {
    json!({
        "judgementOfWisdom": true,
        "judgementOfLight": true,
        "improvedSealOfTheCrusader": "TristateEffectImproved",
        "misery": true,
        "curseOfElements": "TristateEffectImproved",
        "isbUptime": 1.0,
        "shadowWeaving": true,
        "improvedScorch": true,
        "wintersChill": true,
        "bloodFrenzy": true,
        "giftOfArthas": true,
        "mangle": true,
        "exposeArmor": "TristateEffectImproved",
        "faerieFire": "TristateEffectImproved",
        "sunderArmor": true,
        "curseOfRecklessness": true,
        "huntersMark": "TristateEffectImproved",
        "demoralizingRoar": "TristateEffectImproved",
        "thunderClap": "TristateEffectImproved",
        "insectSwarm": true,
        "scorpidSting": true,
        "shadowEmbrace": true,
        "screech": true,
        "hemorrhageUptime": 1.0
    })
}

fn consumes() -> Value {
    json!({
        "potId": 22838,        // Haste Potion
        "flaskId": 22854,      // Flask of Relentless Assault
        "foodId": 27658,       // Roasted Clefthoof
        "conjuredId": 22788,
        "explosiveId": 30217,  // Adamantite Grenade
        "superSapper": true,
        "goblinSapper": true,
        // Adamantite Sharpening Stone, declared exactly as the site's warrior
        // preset does — as the OFF-hand imbue. The engine ignores the main-hand
        // imbue whenever Windfury Totem is up (it is, below) but applies the
        // off-hand one to both weapons, so on a two-hander this is what
        // actually delivers the stone's +14 crit / +12 weapon damage. Verified:
        // removing it costs ~5 DPS on the sample export.
        "ohImbueId": 29453,
        "scrollAgi": true,
        "scrollStr": true
    })
}

/// The engine's default boss: level 73 mechanical, Armor 7685, AttackPower 320.
fn target() -> Value {
    let mut stats = vec![0.0_f64; 42];
    stats[17] = 320.0; // StatAttackPower
    stats[31] = 7685.0; // StatArmor
    json!({
        "level": 73,
        "mobType": "MobTypeMechanical",
        "stats": stats,
        "minBaseDamage": 15113.0,
        "damageSpread": 0.5,
        "swingSpeed": 2.0,
        "parryHaste": true
    })
}

fn execute_proportions(mut e: Value) -> Value {
    e["executeProportion20"] = json!(0.2);
    e["executeProportion25"] = json!(0.25);
    e["executeProportion35"] = json!(0.35);
    e["executeProportion45"] = json!(0.45);
    e["executeProportion90"] = json!(0.90);
    e
}

fn single_target_encounter() -> Value {
    execute_proportions(json!({
        "duration": 180.0,
        "targets": [target()]
    }))
}

/// The engine's LongMultiTarget combo: 21 targets, one disabled at start (~20
/// active).
fn multi_target_encounter() -> Value {
    let targets: Vec<Value> = (0..21)
        .map(|i| {
            let mut t = target();
            if i == 10 {
                t["disabledAtStart"] = json!(true);
            }
            t
        })
        .collect();
    execute_proportions(json!({
        "duration": 180.0,
        "targets": targets
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forwards_random_suffix_as_camel_case() {
        let spec = ItemSpec {
            id: 100,
            enchant: 0,
            gems: vec![],
            random_suffix: -19,
        };
        let v = item_spec(&spec);
        assert_eq!(v["randomSuffix"], json!(-19));
        assert!(v.get("enchant").is_none());
        assert!(v.get("gems").is_none());
    }

    #[test]
    fn omits_zero_random_suffix() {
        let spec = ItemSpec {
            id: 100,
            enchant: 3,
            gems: vec![10, 20],
            random_suffix: 0,
        };
        let v = item_spec(&spec);
        assert!(v.get("randomSuffix").is_none());
        assert_eq!(v["enchant"], json!(3));
        assert_eq!(v["gems"], json!([10, 20]));
    }
}
