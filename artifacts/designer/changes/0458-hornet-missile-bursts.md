# 0458 — The Hornet's missile bursts at the spot (AoE, ground only)

- **Date**: 2026-11-27T14:15:00-05:00
- **Answers**: Q0182 (part 4)
- **Docs**: `factions/gdo/units/hornet.md` (TargetType; Placeholder stats > Attack)

## What changed and why
The user chose option (b). The Hornet's non-homing (DoublyDisjointed) missile now **bursts at the spot it flies to** (systems/combat.md > AoEDamage), like the Rocketman's (0445). As an air-to-ground weapon it hits **ground and underground objects only**. Friendly fire applies, and a burst over an empty circle does nothing. No unit in the game now fires a non-homing single-target projectile, so the question of "which object at the spot" no longer arises.

## Implementation impact
- **Changed:** the Hornet's TargetType is AoE, replacing GK's SingleTarget assumption and TailDisjointed's hit rule.
- **Damage:** **45 total over a 32 su AoERadius** (designer placeholder; was 30 single-target). Range, speed, timings and ammo are unchanged.

## Verification
1. Order a Hornet to attack a moving Raptor. If the Raptor has left the spot, the missile bursts there and does at most partial damage.
2. Order it to attack a stationary tank square-on. The tank takes about 30 before armor.
3. Fire into a clump of infantry. Several take a share, own units included.
4. Fire at the ground under an enemy aircraft. The aircraft takes nothing.
