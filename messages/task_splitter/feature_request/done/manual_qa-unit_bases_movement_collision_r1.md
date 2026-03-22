# unit_bases_movement_collision_r1

## Metadata
- **From**: manual_qa
- **To**: task_splitter

## Content

Rework for unit_bases_movement_collision: LightInfantry works correctly (rugged terrain traversal, turn in place, hard collision). However, only LightInfantry is available in-game. The remaining 8 unit base types and their associated mechanics could not be tested.

**What needs to be implemented/made available**:
- WheeledVehicle (fixed turn radius, no turn in place, reverse)
- TrackedVehicle (speed-based turn radius, spin in place, reverse, crushes enemy LightInfantry)
- HoverVehicle (drag movement, momentum-based)
- HoverCraft (air unit, no ground collision, soft separation)
- Glider (always moving, circles at idle, speed-based turn radius)
- DrillUnit (underground/above-ground modes)
- HeavyInfantry (rugged terrain, not crushable)
- Mech (turret, rugged terrain, crushes enemy LightInfantry)

**Associated mechanics that also need testing once units exist**:
- Turret rotation independent of base within TurnAngle arc
- Directional armor (frontal vs rear damage)
- Air unit soft separation
- TrackedVehicle/Mech crushing enemy LightInfantry

## QA Instructions

1. Spawn a WheeledVehicle — verify it cannot turn in place, has a fixed turn radius when moving, and can reverse.
2. Spawn a TrackedVehicle — verify it can spin in place, turn radius widens at speed, and crushes enemy LightInfantry on contact.
3. Spawn a HoverVehicle — verify momentum-based movement with drag (slides when changing direction).
4. Spawn a HoverCraft (air) — verify it hovers, doesn't collide with ground units, has soft separation from other air units.
5. Spawn a Glider — verify it never stops moving, circles at idle speed when no orders, wider turns at higher speed.
6. Spawn multiple air units — verify they push away from each other gently (soft separation) but can overlap slightly.
7. Select a turret unit — verify turret rotates independently from the base within its TurnAngle arc.
8. Test directional armor units (vehicles/mechs) — verify frontal hits deal less damage than rear hits.
