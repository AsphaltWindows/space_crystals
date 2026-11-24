# 0150 — Mode switches keep orders (Convert, Riot Shield)

- **Date**: 2026-11-24T19:00:00-05:00
- **Answers**: Q0042 (point 6)
- **Docs**: `factions/colonists/units.md > Weapon Conversion > Orders are kept` (new); `factions/gdo/units.md > Peacekeeper > Riot Shield` (Orders are kept)

## What changed and why
Option A, extended by the user to the Riot Shield. Convert and raising/lowering the Riot Shield are mode switches, not orders: current orders stay.
- Legionnaire: holds still 4 frames, then continues its orders with the other weapon.
- Peacekeeper: orders it can't carry out while shielded are on hold. Explicit Attack → follows the target while shielded, resumes attacking when lowered. AttackMove/Patrol → moves only while shielded (0076), engages again once lowered.

## Implementation impact
Toggling never clears the command queue. Peacekeeper Attack order persists through shield up (executed as follow) — differs from "Attack orders are not possible" for *new* orders while shielded (still true: new explicit Attack leaves shielded units out, 0075).

## Verification
- Sabres attack-moving, press R: they pause 0.25 s and continue the attack-move as Rifles.
- Peacekeeper attacking a target, raise shield: it stops firing and follows the target; lower it: it shoots that target again.
