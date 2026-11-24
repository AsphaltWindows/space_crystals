# 0147 — No damage modifier touches Shields (clarification)

- **Date**: 2026-11-24T18:40:00-05:00
- **Answers**: Q0042 (point 3) — already answered by the docs
- **Docs**: `systems/combat.md > DamageModifierOrder` (wording clarified)

## What changed and why
No design change. "Shields take damage at full value, and only the overflow enters step 1" already means: no pre-armor reduction, armor, DirectionalArmor or minimum applies to Shields; all apply to the overflow only (e.g. a Drone's HoverVehicle DirectionalArmor). For AoE the full value is the damage share. Wording made explicit.

## Implementation impact
As the Director proposed.

## Verification
Hit a shielded Drone from behind for less than its Shields: Shields lose exactly the damage. Hit it for more: only the overflow gets the rear × 1.25 (after armor).
