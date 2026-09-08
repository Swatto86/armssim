# Architecture

armssim answers one question: **which of the items I already own should an Arms
warrior equip?** It never computes damage itself — the WoWSims TBC engine does
that — so the whole design is about turning a character export into a lot of
well-formed sim requests and searching the results.

## Layers (armssim-rs)

```
cli / main.rs        argument parsing, progress, the printed report
  app/               character.rs   parse the addon export
                     optimize.rs    coordinate ascent + baseline eval
                     simulator.rs   the "score these gear sets" contract
  domain/            gear.rs        17 slots, ItemSpec, ST/AoE scenarios
                     item.rs        engine item type/hand type -> equippable slots
                     plan.rs        the search space: groups of exclusive options
                     objective.rs   ST / AoE / blended score
                     report.rs      equipped -> best diff
  infra/             engine.rs      locate wowsimcli + db.json
                     itemdb.rs      db.json -> ItemCatalog
                     request.rs     gear set -> engine RaidSimRequest (protojson)
                     wowsims.rs     run wowsimcli, one process per sim, N in parallel
```

`domain` knows nothing about JSON or the engine; `app` depends on the
`Simulator` trait, not on `wowsimcli`. That is what makes the optimizer testable
without an engine present.

`armssim-gui` is a Tauri shell around the same library. It **embeds**
`wowsimcli.exe` and `db.json` at compile time (`engine_resources.rs`), staged
into `src-tauri/engine-resources/` by `scripts/stage-engine.ps1` from the repo's
untracked `engine/` directory — so the GUI crate does not compile until that
script has run.

## Decisions that are Arms-specific

- **Two-handers only.** `domain::item::weapon_slots` accepts `HandTypeTwoHand`
  and nothing else, and every option in the weapon group sets the off-hand to
  `None`. An export taken in dual-wield gear therefore converges on a legal Arms
  setup rather than a 1H-with-empty-off-hand hybrid, and the equipped main hand
  is only seeded into the search if it is itself a 2H. `main.rs` warns when it
  is not.
- **The rotation is the engine's Arms APL, embedded verbatim**
  (`infra/arms.apl.json`, copied from `wowsims/tbc-new`
  `ui/warrior/dps/apls/arms.apl.json`). It is data, not something armssim
  reimplements: re-copy the upstream file to update the rotation.
- **The request mirrors the site's defaults** so DPS lines up with wowsims.com:
  full raid buffs/debuffs, the warrior preset's class options (Berserker stance,
  Battle Shout, 50 starting rage, 250 ms queue delay, stance snapshot, Battle
  Shout T2), and the preset consumables.
- **The sharpening stone is declared as `ohImbueId`, not `mhImbueId`** — the
  same field the site's warrior preset sets. The engine ignores the main-hand
  imbue while Windfury Totem is active but applies the off-hand one to *both*
  weapons, so on a two-hander this is the field that actually delivers it.
  Measured at ~5 DPS on the sample export.

## Search

Coordinate ascent: hold every slot fixed, try each candidate for one group, keep
the best, repeat until a full pass yields no improvement. Groups are the single
slots, the ring pair, the trinket pair, and the (main-hand-only) weapon group.
Each candidate evaluation is one `wowsimcli` process; a bounded worker pool
keeps every core busy. Consequence: multi-piece set bonuses can be missed,
because each piece looks like a downgrade until the set completes.

## Engine coupling

`request.rs` hand-writes the engine's protojson. Unknown fields are silently
discarded by the engine, so a rename upstream fails quietly rather than loudly —
when the engine is updated, re-check that request against
`ui/warrior/dps/presets.ts` and re-run the sample export. Known-good engine
release: **v0.0.133** (source commit `dd25f578`).
