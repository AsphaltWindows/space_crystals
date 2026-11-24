# 0031 — Attack range is measured edge to edge

- **Date**: 2026-11-23T19:04:35-05:00
- **Answers**: Q0009 (point 1, range; "proximity" and silhouette rotation still open)
- **Docs**: `systems/combat.md > AttackAttributes > Range Measurement` (new)

## What changed and why
User chose **C (edge to edge)**, not the Director's default A (attacker centre → target's nearest point). Range and MinRange are the shortest gap between the attacker's shape (Silhouette or footprint) and the target's shape. Big targets are reachable at the same range as small ones, a big attacker's or turret's own body doesn't reduce its range, and melee is a gap of about 0. Ground-location targets measure from the attacker's shape to the point.

## Implementation impact
- **Replaces** default A. Distance checks for Range/MinRange use shape-to-shape distance on both sides.
- Range values in unit stats keep their meaning as "gap". Placeholder values don't need rescaling.

## Verification
- A range-5 Peacekeeper and a range-5 large turret both start firing when the gap between their edges and the target's edge reaches 5 grid units.
- A unit attacking a 4x4 Deployment Center stops 5 tiles from its wall, not from its centre.
- A melee unit attacks when touching the target, including large buildings.
