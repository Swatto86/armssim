# WowSimsExporter — bank-scan patch

The stock WowSimsExporter only exports **equipped + carried bags**. This patch makes
it also include **bank** items in the bag-items payload used for bulk/optimizer sims.

> **Prefer `../ExportAll/`?** The standalone `ExportAll` addon does the same
> job (bank scan + single combined JSON) without patching WSE at all. Use that
> one and you can skip this patch entirely.

## What changed
`ExportStructures/EquipmentSpec.lua` → `FillFromBagItems()` now scans, in addition to
bags 0–4:
- `BANK_CONTAINER` (-1): the main bank window slots
- bank bags `NUM_BAG_SLOTS+1 .. NUM_BAG_SLOTS+NUM_BANKBAGSLOTS` (5–11)

Bank containers only return items **while the bank frame is open** (Blizzard API limit),
so the bank portion is captured only when you run the export **standing at a banker**.
Away from the bank it's a safe no-op (those containers report 0 slots).

## Install / re-apply
The patch is applied directly to the live addon at:
`World of Warcraft\_anniversary_\Interface\AddOns\WowSimsExporter\ExportStructures\EquipmentSpec.lua`
(original backed up alongside as `EquipmentSpec.lua.bak`).

**If CurseForge / an addon updater overwrites WowSimsExporter, the patch reverts.**
Re-apply by copying this folder's `ExportStructures/EquipmentSpec.lua` over the installed one.

## Test in-game
1. `/reload` (or relog) so the new Lua loads.
2. Walk to a banker and **open the bank**.
3. `/wse export` → the string should now include items sitting in your bank.
4. Move away from the bank and `/wse export` again → bank items drop out (expected).
