# ExportAll

Standalone WoW addon that exports a character's gear as a **single JSON string**
in one click. No WSE (WowSimsExporter) dependency.

Built specifically for **armssim**'s bulk/optimizer input format.

## Install

Copy this folder into your WoW client:

```
C:\Program Files (x86)\World of Warcraft\_anniversary_\Interface\AddOns\ExportAll\
```

(`/reload` to pick it up.)

## Use

1. Type `/exportall` — opens a frame with the exported JSON.
2. Click **Generate**, `Ctrl+C`, paste into `me.json`.
3. Run armssim.

The status label tells you exactly what was captured:
- `equipped only` — if you have no bag/bank candidates
- `equipped + bags` — away from a banker
- `equipped + bags + bank` — while the bank window is open

Shortcut: `/exportallchat` — dumps the same JSON directly into your chat frame.

## What it does

Reads 17 equipped slots + every item in carried bags 0–4 + every item in the bank
(Bank container + bank bags 5–11, only while the bank window is open) and writes
them as a single `gear.items` array. That is the same single-file format armssim
consumes — the first 17 entries are your current gear, the rest are the candidate
pool.

The 16 supported classic interfaces are listed in the TOC (50504 / 50503 / 40402 /
38001 / 20505 / 11508). It loads on any classic client that Blizzard still serves.

## Notes

- Bank containers return items only while the bank window is open (Blizzard API
  limit). The scan is a safe no-op when the bank is closed, so `/exportall` works
  anywhere.
- This addon replaces the previous `WowSimsExporter` + bank-patch workflow.
- You can keep WSE installed for its normal features, but it is no longer
  required for armssim exports.
