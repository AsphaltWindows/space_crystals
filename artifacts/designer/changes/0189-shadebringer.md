# 0189 — New Syndicate unit: Shadebringer (Sludge, Cover)

- **Date**: 2026-11-25T01:10:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Shadebringer` (new); `factions/syndicate/structures.md > Vehicle Assembly` (Produces), `> Vault` (prerequisite)

## What changed and why
User: Syndicate support vehicle from the Vehicle Assembly, requires a Vault. WheeledVehicle, no attack, no turret. Two toggleable abilities:
- **Sludge**: leaves a sludge trail lasting X s; infantry and non-hover vehicles entering it are slowed Y% for Z s.
- **Cover**: leaves a smoke wall that blocks line of sight into and behind it for all non-satellite units. This is the first sight-blocking mechanic in the game.

## Implementation impact
Concept only; numbers and several rules are TBD and being settled with the user (see the TBD lines in the section). Line-of-sight blocking is new to the vision model; don't build it until the rules are settled.

## Verification
—
