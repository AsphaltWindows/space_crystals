# 0122 — Beacons are Ungroupable

- **Date**: 2026-11-24T14:40:00-05:00
- **Answers**: Q0040 (point 6)
- **Docs**: `factions/colonists/economy.md > Beacon > Groupable`; `systems/controls/selection_and_hud.md > SelectionGroup` (Groupable guidance)

## What changed and why
Option B. Beacon Groupable true → **false**. A Beacon is used up by its Warp In, like a worker used up by a build, so it follows the existing guidance: each Beacon is its own SelectionGroup; Warp In applies to the Tabbed-to Beacon. The guidance now names "structures used up by a build" alongside workers.

## Implementation impact
Changes the Beacon's Groupable flag. No special "only one Beacon" rule needed.

## Verification
Select three Beacons: three separate SelectionGroups; Tab cycles them; Warp In places a ghost on the active Beacon only.
