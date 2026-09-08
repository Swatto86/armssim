# Arms Warrior Reference — WoW: TBC Anniversary

> **Where these numbers come from.** Everything in this file that is marked
> *(sim)* is read straight out of the [WoWSims TBC](https://github.com/wowsims/tbc-new)
> engine armssim drives — its Arms talent preset, Arms EP weights, Arms APL and
> per-phase Arms gear presets, as of engine release **v0.0.133** (upstream commit
> `dd25f578`, fetched 2026-09-08). Everything else is community consensus and a
> starting point only. Your own run of armssim is the final arbiter for *your*
> bags; where a number matters, trust the in-game tooltip and the sim over this
> doc.
>
> **Scope:** Arms is the **two-handed** DPS spec. armssim only ever equips a 2H
> main hand and an empty off-hand (see `armssim-rs/src/domain/item.rs`). The
> dual-wield Mortal Strike build the sim calls *"Arms — Kebab"* is a Fury gear
> model; use [furysim](https://github.com/Swatto86/furysim) for that.

---

## 1. Stat priority & caps

Priority order (cap the first two, then maximise the rest):

1. **Hit → 9%** (142 hit rating) for special attacks vs a level-73 boss. Only
   **6% from gear** is needed if a Balance druid keeps **Improved Faerie Fire**
   (+3% melee hit) up.
2. **Expertise → 6.5%** dodge reduction (26 expertise skill / 103 rating).
   **Weapon Mastery** (+5 expertise ≈ 1.25%) lowers what you need from gear.
   Orcs with an axe, and humans with a sword/mace, get +5 more from their racial
   weapon skill.
3. **Weapon damage / top-end DPS on the 2H** — this is the Arms-specific one.
   Mortal Strike and Slam both scale off weapon damage, so main-hand DPS is by
   far the heaviest single item stat *(sim: 5.85–6.0 EP per point of MH DPS)*.
4. **Strength**, then **crit**, **expertise** and **hit** past the caps.
5. **Haste**, **agility**, **armour penetration** last.

### EP weights *(sim)*

Straight from the engine's Arms presets — relative value per point, Strength = 1.

| Stat | P1–P2 | P3–P5 |
|---|---|---|
| Strength | 1.00 | 1.00 |
| Agility | 0.70 | 0.80 |
| Attack power | 0.46 | 0.45 |
| **Hit rating** | **1.81** | 1.01 |
| Crit rating | 0.95 | **1.05** |
| Haste rating | 0.80 | 0.85 |
| Armour penetration | 0.19 | 0.23 |
| **Expertise rating** | **1.81** | **1.78** |
| **Main-hand weapon DPS** | **5.85** | **6.00** |

Read the shift: in early phases hit and expertise dominate because you are
nowhere near the caps; by T6/Sunwell gear you are capped, hit collapses to ~1.0
and **crit becomes the best secondary**. Weapon DPS stays king throughout — a
bigger 2H is almost always the biggest single upgrade you can make.

### Conversion rates (level 70)

| Stat | Rate |
|---|---|
| Hit | 15.77 rating = 1% |
| Expertise | 3.9423 rating = 1 skill; 4 skill = 1% dodge/parry reduction |
| Crit | 22.08 rating = 1% |
| Agility | 33 Agi = 1% crit |
| Special (yellow) hit cap | 9% vs a level-73 boss |

---

## 2. Talents — 33/28/0 *(sim)*

Paste into the sim or the [Wowhead TBC calculator](https://www.wowhead.com/tbc/talent-calc):

```
32005011352010500221-0550000500521203
```

33 points in **Arms** (through **Mortal Strike**, the two-handed damage talents
and **Death Wish**), 28 in **Fury** (through **Flurry**, plus **Sweeping
Strikes** for cleave). The APL armssim runs uses **Mortal Strike**, **Slam**,
**Whirlwind**, **Execute**, **Overpower**, **Death Wish**, **Sweeping Strikes**,
**Bloodrage** and **Berserker Rage**, so a build missing any of those will sim
worse than it should.

The sim also ships an **"Arms — Kebab"** build (`34005021302010510321-0550000520501203`)
— Mortal Strike with *dual-wield* gear. armssim deliberately does not model it;
it is a Fury gear problem.

---

## 3. Rotation — what the APL actually does *(sim)*

armssim embeds the engine's Arms APL verbatim
(`armssim-rs/src/infra/arms.apl.json`). In priority order:

**Pre-pull:** Berserker Rage → Bloodrage + Battle Shout at -3s → Flame Cap at
-1s → Charge/Intercept to close.

**Cooldowns** (aligned to Bloodlust at ~5s): Bloodlust/drums, engineering
gadgets, on-use trinkets, Haste Potion, Flame Cap, racial (Blood Fury /
Berserking), **Death Wish**, then **Recklessness** if you have it enabled.

**Damage priority:**
1. **Sweeping Strikes** — 2+ targets only
2. **Whirlwind** — 2+ targets
3. **Slam**, but only inside the window just after a main-hand swing lands
   (~0.3 s), so the cast does not clip white damage — this is *slam weaving*,
   and it is the bit that makes 2H Arms fiddly to play by hand
4. **Mortal Strike** on cooldown
5. **Whirlwind** when Mortal Strike is more than 1.5 s away
6. **Execute** below 20% boss HP
7. **Cleave** — 2+ targets, 40+ rage
8. **Overpower weaving** — when a dodge has armed Overpower, step to Battle
   Stance, Overpower, step back to Berserker
9. **Heroic Strike** — single target, outside execute, 40+ rage
10. **Hamstring** as a further rage dump above 60 rage
11. **Bloodrage** below 90 rage, **Berserker Rage**, and a **Battle Shout**
    refresh when it is about to fall off

**Sunder Armor** is cast on single-target fights to help the tank get the debuff
stacked, then maintained if nobody else is applying it.

---

## 4. Gems *(sim, P5 preset)*

| Socket | Gem | Stat |
|---|---|---|
| Meta | **Relentless Earthstorm Diamond** | +12 Agi, +3% crit damage |
| Red | **Crimson Sun** (Str/crit) or **Bold Living Ruby** (+8 Str) | strength |
| Yellow | **Inscribed Pyrestone** (Str/crit) | strength + crit |
| Blue | **Jagged Seaspray Emerald** / **Sovereign Tanzanite** | to switch the meta on |
| Weapon sockets | **Smooth Lionseye** / **Inscribed Pyrestone** | crit / Str+crit |

The meta needs **2 red + 2 yellow + 2 blue** equipped, so you cannot go pure
Strength. Ignore socket bonuses below ~3 Strength or ~4 crit. Close a hit or
expertise gap with gems before gemming pure offence.

---

## 5. Enchants *(sim, P3–P5 presets)*

| Slot | Enchant |
|---|---|
| Head | **Glyph of Ferocity** |
| Shoulders | **Greater Inscription of Vengeance** (Aldor) / **of the Blade** (Scryer) |
| Back | Enchant Cloak — Greater Agility |
| Chest | Enchant Chest — **Exceptional Stats** |
| Wrist | Enchant Bracer — **Brawn** |
| Hands | Enchant Gloves — **Major Strength** |
| Legs | **Nethercobra Leg Armor** |
| Feet | Enchant Boots — **Cat's Swiftness** |
| **Two-hander** | **Mongoose** through P4; **Executioner** once you are hit/expertise capped (the sim's P5 preset uses Executioner) |

---

## 6. Gear presets by phase *(sim)*

These are the engine's own Arms presets — the sets whose DPS the website
publishes. armssim's job is to find the best set **in your bags**, which is
usually a partial version of one of these.

| Slot | Pre-raid | P2 (SSC/TK) | P5 (Sunwell) |
|---|---|---|---|
| Head | Mask of the Deceiver | Furious Gizmatic Goggles | Coif of Alleria |
| Neck | Adamantine Chain of the Unbroken | Pendant of the Perilous | Hard Khorium Choker |
| Shoulder | Ragesteel Shoulders | Shoulderpads of the Stranger | Demontooth Shoulderpads |
| Back | Vengeance Wrap | Vengeance Wrap | Cloak of Unforgivable Sin |
| Chest | Ragesteel Breastplate | Bloodsea Brigand's Vest | Bladed Chaos Tunic |
| Wrist | Black Felsteel Bracers | Bracers of Eradication | Onslaught Bracers |
| Hands | Fel Leather Gloves | Gloves of the Searing Grip | Thalassian Ranger Gauntlets |
| Waist | Deathforge Girdle | Belt of One-Hundred Deaths | Onslaught Belt |
| Legs | Midnight Legguards | Leggings of Murderous Intent | Leggings of the Immortal Night |
| Feet | Obsidian Clodstompers | Warboots of Obliteration | Onslaught Treads |
| Ring 1 | Shapeshifter's Signet | Band of the Ranger-General | Band of Ruinous Delight |
| Ring 2 | Ring of Arathi Warlords | Shapeshifter's Signet | Hard Khorium Band |
| Trinket 1 | Badge of the Swarmguard | Dragonspine Trophy | Dragonspine Trophy |
| Trinket 2 | Bloodlust Brooch | Badge of the Swarmguard | Blackened Naaru Sliver |
| **Two-hander** | Lionheart Champion | Twinblade of the Phoenix | Apolyon, the Soul-Render |
| Ranged | Mama's Insurance | Serpent Spine Longbow | Golden Bow of Quel'Thalas |

P1 runs Warbringer (T4) head/chest with Lionheart Champion; P3/P4 run Onslaught
(T6) head/shoulder/chest/hands with **Cataclysm's Edge**. Note how little tier
the Arms presets keep — the two-hander and raw weapon DPS outrank set bonuses,
which is the opposite instinct to Fury.

---

## 7. Consumables *(sim defaults, exactly what armssim sims with)*

| Type | Choice |
|---|---|
| Flask | **Flask of Relentless Assault** |
| Food | **Roasted Clefthoof** (Str) — Spicy Hot Talbuk if you need hit |
| Combat potion | **Haste Potion**, saved for the Bloodlust window |
| Conjured | **Flame Cap** |
| Weapon stone | **Adamantite Sharpening Stone** (Weightstone for maces) |
| Engineering | Super Sapper Charge, Goblin Sapper Charge, Adamantite Grenade |
| Scrolls | Scroll of Strength, Scroll of Agility |

A flask does not stack with battle/guardian elixirs — it replaces both. The sim
assumes the flask.

---

## 8. Raid buffs and debuffs

armssim sims a **full raid with every standard buff and debuff**, mirroring the
engine's own defaults (see `armssim-rs/src/infra/request.rs`): Bloodlust, drums,
Blessing of Kings/Might, Battle + Commanding Shout, Leader of the Pack,
Unleashed Rage, Strength of Earth / Grace of Air / Windfury totems, Improved
Faerie Fire, Sunder + Expose Armor, Curse of Recklessness, Blood Frenzy, Mangle,
Gift of Arthas, Improved Seal of the Crusader.

If your raid is thinner than that, the absolute DPS numbers move, but the
*ranking* between two gear sets is usually stable — which is all the optimizer
is using them for.

---

## Sources

- WoWSims TBC engine — [Arms APL, talents, EP weights and gear presets](https://github.com/wowsims/tbc-new/tree/master/ui/warrior/dps)
- [Icy Veins — Arms Warrior DPS (TBC Classic)](https://www.icy-veins.com/tbc-classic/arms-warrior-dps-pve-stat-priority)
- [Wowhead — Warrior DPS Stat Priority (TBC)](https://www.wowhead.com/tbc/guide/classes/warrior/dps-stat-priority-attributes-pve)
- [Wowhead — Warrior DPS Gems & Enchants (TBC)](https://www.wowhead.com/tbc/guide/classes/warrior/dps-enchants-gems-pve)
