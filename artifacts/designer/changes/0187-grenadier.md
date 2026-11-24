# 0187 — New Syndicate unit: Grenadier

- **Date**: 2026-11-25T00:45:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Grenadier` (new); `factions/syndicate/structures.md > Hideout` (Produces, E: Build Grenadier), `> Chemical Plant` (description)

## What changed and why
User: Syndicate explosives infantry from the Hideout, requires a Chemical Plant. Explosive grenade: slow DoublyDisjointed projectile, ground only, moderate-high AoE damage; dodgeable; punishes static/clumped units. Attack Ground available (existing rule: every attack that can miss can target ground).

## Implementation impact
New unit type, concept only: **all numbers TBD** (roster pass is concept-only; numbers later). HeavyInfantry base and Groupable true assumed. Hideout panel E (tentative).

## Verification
Once numbers exist: grenade visibly slow; a moving target escapes the blast; Attack Ground available on the panel.
