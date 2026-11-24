# 0152 — After Convert the new weapon starts ready

- **Date**: 2026-11-24T19:10:00-05:00
- **Answers**: Q0042 (point 8)
- **Docs**: `factions/colonists/units.md > Weapon Conversion` (New weapon starts ready)

## What changed and why
Option A. No reload carry-over. Checked for a reload-skip exploit: round-trip conversion (8 frames) exceeds ReloadDuration (Sabre 5, Rifle 4), so none. Revisit if reloads grow beyond ~8 frames.

## Implementation impact
On conversion completion, attack phase state resets to ready.

## Verification
Rifle fires, convert during Reloading: the Sabre begins Aiming as soon as conversion ends and a target is in reach.
