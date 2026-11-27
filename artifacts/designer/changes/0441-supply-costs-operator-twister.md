# 0441 — Supply costs: Drone Operator 1 (was 2), Twister 1 (new)

- **Date**: 2026-11-26T20:35:00-05:00
- **Answers**: —
- **Docs**: `factions/gdo/overview.md > GDOResources > Supplies` (table); `factions/gdo/units/drone_operator.md > Placeholder stats`; `factions/gdo/units/twister.md > Placeholder stats`; `factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`

## What changed and why
User adjustment to 0435: the **Drone Operator costs 1 Supply** (was 2) and the **Twister now costs 1 Supply** (it had none).

Current set: Field Engineer 1, Drone Operator 1, Twister 1, Hornet 2, Dragonfly 4, Artillery Rig 5, Prospector 8.

## Implementation impact
Updates 0435's numbers; the Twister's Factory button now also needs a Supply.

## Verification
With 1 Supply: a Drone Operator or a Twister can be queued (not both); with 0, both are greyed.
