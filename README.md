# armssim

A local **Arms-warrior** gear optimizer for **WoW: The Burning Crusade Anniversary**.
It searches your equipped + bag + bank items for the highest-DPS two-handed
setup, for both **single-target** and **AoE**, using all your CPU cores.

It does **not** reimplement combat maths — every DPS number comes straight from
the [WoWSims TBC](https://github.com/wowsims/tbc-new) engine, so results match
the website. The optimizer itself is written in **Rust**; it drives the engine
through the `wowsimcli` binary (one sim per gear candidate), so no Go toolchain
is needed to build or run it.

This is the Arms sibling of [furysim](https://github.com/Swatto86/furysim). The
difference is not cosmetic:

- the embedded rotation is the engine's **Arms APL** (Mortal Strike, slam
  weaving, Overpower stance-dancing, Sweeping Strikes), not the Fury one;
- only **two-handed** weapons are candidates, and every weapon swap **clears the
  off-hand** — so an export taken in dual-wield gear converges on a legal Arms
  setup instead of an impossible one.

If you want the dual-wield Mortal Strike ("Arms — Kebab") build, that is a Fury
gear model: use furysim.

---

## Layout

```
C:\Users\Swatto\armssim\
├─ engine\                     # WoWSims TBC engine: wowsimcli.exe + assets\database\db.json
├─ armssim-rs\                 # ← the optimizer (Rust)
│  ├─ Cargo.toml
│  └─ src\                     # domain / app / infra / cli — exporter parse, item->slot,
│                              #   permute, coordinate ascent, request builder, sim pool
├─ armssim-gui\                # Tauri desktop front-end around the same library
├─ addon\ExportAll\            # standalone "export everything" addon (no WSE dep)
├─ addon\WowSimsExporter-bank-patch\  # legacy: bank-scan patch for stock WSE
├─ docs\arms-reference.md      # stat/EP/talent/rotation/BiS reference, read out of the sim
├─ testdata\sample-arms.json   # sample export (P3 Arms preset + P4/P5 pieces in the "bags")
├─ run.ps1                     # convenience launcher
└─ README.md                   # this file
```

---

## Getting the engine

`engine\` is **not** tracked by this repo (see `.gitignore`). Two ways to get it:

**Prebuilt (recommended).** Grab `wowsimcli-windows.exe.zip` from the latest
[tbc-new release](https://github.com/wowsims/tbc-new/releases), and `db.json`
from the same tag:

```powershell
mkdir C:\Users\Swatto\armssim\engine\assets\database
# unzip wowsimcli-windows.exe.zip -> engine\wowsimcli.exe
# copy assets\database\db.json from the tbc-new source tree -> engine\assets\database\db.json
```

**From source** (needs Go *and* `protoc` + `protoc-gen-go`, since the generated
proto package is not checked in):

```powershell
git clone https://github.com/wowsims/tbc-new C:\path\to\tbc-new
cd C:\path\to\tbc-new
make proto          # generates sim\core\proto
go build -tags with_db -o wowsimcli.exe .\cmd\wowsimcli
```

Then point at it with `--engine C:\path\to\tbc-new` or `$env:ARMSSIM_ENGINE`.

> armssim hand-builds the engine's sim-request JSON (`armssim-rs\src\infra\request.rs`),
> so an upstream schema change could in principle break it. Known-good engine
> release: **v0.0.133** (source commit `dd25f578`). If a fresh clone stops
> working, try that tag before filing it as an armssim bug.

---

## How to run it

### Quick (sample data)
```powershell
cd C:\Users\Swatto\armssim
.\run.ps1
```

### Your own character
```powershell
.\run.ps1 -Character .\me.json
```

### Launcher options
```powershell
.\run.ps1 -Only blend                 # only the blended set (~3x faster)
.\run.ps1 -AoeFraction 0.5            # even 50/50 ST/AoE blend
.\run.ps1 -Only blend -AoeFraction 0.15   # mostly-ST raid, blend only
```
`-Only` = `all` | `st` | `blend` | `aoe`. `-AoeFraction` = 0..1.

### Directly via the binary
```powershell
C:\Users\Swatto\armssim\armssim-rs\target\release\armssim.exe `
    C:\path\to\character.json
```

Flags: `--iterations` (search precision, default 1500), `--final-iterations`
(precise re-sim of the winner, default 20000), `--aoe-fraction`, `--only`
(`all`|`st`|`blend`|`aoe`), `--seed`, `--jobs` (parallel sims, default = CPU
count), `--engine` (path to the engine directory; auto-detected if omitted),
`--no-refine` (skip the gem/enchant pass).

### Output: three sets
It prints the best gear for **three** objectives so you don't have to pick:
- **SINGLE TARGET** — max ST DPS
- **BLEND** — the "happy median": maximises effective DPS over a fight that is
  part ST, part AoE. Controlled by `--aoe-fraction` (default `0.3` = 30% of the
  fight in AoE).
- **AOE** — max multi-target DPS

Each set shows its ST *and* AoE DPS so you can see the trade-off. The BLEND set
is usually the one to actually equip if your content is mixed.

### Gems and enchants

Once the items are settled, a second pass chooses **what to socket and what to
enchant** on them, and prints the changes:

```
  Gems & enchants:
    Head      socket 1  (empty)  ->  Inscribed Pyrestone
    Head      enchant   (none)   ->  Glyph of Ferocity
    MainHand  enchant   Mongoose ->  Enchant Weapon - Executioner
  Still missing:
    Meta      Relentless Earthstorm Diamond is not active - needs 2 red, 2 yellow, 2 blue
```

Every candidate is simmed, exactly like an item swap - the shortlist only
decides what gets *offered*, never what wins. Gems come from the engine's own
database, filtered to your professions (Jewelcrafter-only gems appear only if
you are one) and excluding unique gems, which you could not socket twice anyway.
Enchant candidates are a curated per-slot list, because the best weapon enchants
are procs with no stat line to rank them by.

It also repairs a **dark meta gem**: filling every socket greedily leaves the
meta unlit, since no single gem swap can satisfy a 2-red/2-yellow/2-blue rule,
so the cheapest conversion back to a lit meta is simmed against the greedy set
and kept if it wins. On bare gear that repair alone was worth **+61 DPS** in
testing.

Turn the whole pass off with `--no-refine` (or the checkbox in the GUI) if you
only want to know what to equip.

---

## Getting your data out of the game

Use the **ExportAll** addon (in `addon\ExportAll\`):

1. Type `/exportall`. If you want bank items included, stand at a banker with the
   bank window open first.
2. Click **Generate**, copy the JSON, and save as `me.json`.
3. Run armssim on it.

> The status line tells you exactly what was exported: `equipped only`,
> `equipped + bags`, or `equipped + bags + bank`. Bank items are only captured
> while the bank window is open — that's a Blizzard API limit.
>
> **Format:** "Export All" writes a single `gear.items` array — the 17 equipped
> slots first, then every bag/bank candidate appended after. armssim splits it:
> the first 17 are your current gear, the rest are the candidate pool. (It also
> still accepts an older shape with a separate top-level `bagItems` array, and
> per-item `random_suffix` for "...of the X" items.)
>
> The addon writes `"spec": "arms"` when Arms is your deepest tree. armssim
> rejects an export that says `fury` — the rotation it sims is Arms-specific.
> **Export while specced and geared as Arms**, or the baseline number it prints
> for "current gear" is not an Arms number (it warns you if your main hand is
> not a two-hander).

---

## What it does (pipeline)

```
ExportAll (in-game)
   └─ character.json  (equipped + talents + bag/bank items, one file)
              │
              ▼
   armssim (Rust):  classify each candidate -> equipment slot (engine db.json),
             keeping only two-handers for the weapon slot
             coordinate ascent: hold all slots fixed, find the best item per
             slot, repeat until nothing improves — separately for ST and AoE
             (each candidate is one wowsimcli sim; the pool runs them across
              every CPU core)
              ▼
             second pass over the winning set: one group per socket and per
             enchantable slot, then a meta-gem repair if the greedy pass left
             the meta dark
              ▼
   "Equip exactly this for ST / for AoE, socket and enchant it like this",
   with the precise DPS delta
```

---

## Caveats (read these)

- **Coordinate ascent optimizes one slot at a time.** It can miss multi-piece
  tier-set bonuses, where each piece looks like a downgrade until the set
  completes. Verify big set-related swaps on the website. (This bites Arms less
  than Fury — the sim's own Arms presets lean on weapon DPS over set bonuses.)
- **Buffs are a full raid with Faerie Fire ON** (matching the engine's standard
  raid). If your raid differs, the absolute numbers shift; the relative ranking
  is usually stable. Buff configurability is a planned addition.
- **Gains are usually small (~1–2%)** once your gear is good — the bigger levers
  are the two-hander itself, then the hit and expertise caps (see
  `docs\arms-reference.md`). The gem/enchant pass is worth little on an already
  optimised set (+8 DPS on the sample) and a great deal on a bare one (+452 DPS,
  +18.6%, on the same items stripped of gems and enchants).
- **It suggests gems and enchants, it cannot check you own them.** The shortlist
  is what the engine's database says exists, minus the professions you lack.
- **Each candidate is a separate `wowsimcli` process.** Startup cost per sim is
  fixed, so a full `--only all` run takes a few minutes. Use `--only blend`
  and/or a lower `--iterations` to go faster.

---

## Rebuilding

```powershell
cd C:\Users\Swatto\armssim\armssim-rs
cargo build --release
```

The binary auto-detects the `engine\` directory by searching upward from the
executable and the working directory; override with `--engine <dir>` or the
`ARMSSIM_ENGINE` environment variable. It reads `engine\assets\database\db.json`
for item classification and runs `engine\wowsimcli.exe` for every sim.
