# unit_bases_movement_collision

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# unit-bases-movement-collision

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement all UnitBase types, MovementModel definitions, TurretAttributes, and UnitCollision rules as defined in `artifacts/designer/design/units.md`.

**UnitBase Types** (each defines Domain, HasTurret, DirectionalArmor, RuggedTerrain, Crushable, CanTurnInPlace, CanReverse, MovementModel):

1. **LightInfantry** — Ground, no turret, no directional armor, rugged=true, crushable=true, turn in place=true, no reverse. TurnRateMovement. Has RuggedTerrainDefenseBonus.
2. **HeavyInfantry** — Ground, no turret, no directional armor, rugged=true, crushable=false, turn in place=true, no reverse. TurnRateMovement.
3. **WheeledVehicle** — Ground, turret, directional armor, rugged=false, crushable=false, no turn in place, can reverse. FixedTurnRadiusMovement.
4. **TrackedVehicle** — Ground, turret, directional armor, rugged=false, crushable=false, turn in place=true, can reverse. SpeedTurnRadiusMovement. Crushes enemy LightInfantry.
5. **DrillUnit** — Underground, turret, directional armor, rugged=false, crushable=false, turn in place=true, can reverse. SpeedTurnRadiusMovement. Cannot fire underground; has above-ground mode.
6. **HoverVehicle** — Ground, turret, directional armor, rugged=false, crushable=false, turn in place=true, no reverse. DragMovement.
7. **Mech** — Ground, turret, directional armor, rugged=true, crushable=false, turn in place=true, no reverse. TurnRateMovement. Crushes enemy LightInfantry.
8. **HoverCraft** — Air, turret, no directional armor, rugged=false, crushable=false, turn in place=true, no reverse. DragMovement.
9. **Glider** — Air, turret, no directional armor, rugged=false, crushable=false, no turn in place, no reverse. GliderMovement. Must always maintain movement.

**MovementModel Definitions:**

1. **TurnRateMovement**: TurnRate, Acceleration, Deceleration, MaxSpeed. Fixed turn rate, responsive movement.
2. **FixedTurnRadiusMovement**: MinimumTurnRadius, ForwardAcceleration, ForwardMaxSpeed, ReverseAcceleration, ReverseMaxSpeed, Deceleration. Cannot turn in place, fixed turn radius.
3. **SpeedTurnRadiusMovement**: SpeedToTurnRadiusRatio, Acceleration, Deceleration, MaxSpeed. Can spin in place, turn radius widens with speed.
4. **DragMovement**: ForwardAcceleration, OmniDirectionalAcceleration, DragRatio, TurnRate. Thrust-based, drag opposes movement. MaxSpeed = (OmniAccel + ForwardAccel) / DragRatio.
5. **GliderMovement**: IdleSpeed, MaxSpeed, Acceleration, Deceleration, MaxCentripetalAcceleration. Must always move, circles when idle, turn radius = v^2/a.

**TurretAttributes:**
Only on units with HasTurret=true. Turret centered on unit facing.
- TurnAngle: degrees (full arc, max 360) — split equally CW/CCW from center
- TurnRate: degrees per frame

**UnitCollision:**
- **Ground**: Hard collision via Silhouette rectangle. No overlap. Idle units don't move aside. Moving units pathfind around.
- **Air**: No collision with ground/structures. Soft separation with other air units via SeparationRadius (per-type circular repulsion, must be > Silhouette).

## QA Instructions

1. Spawn a LightInfantry unit — verify it can traverse rugged terrain, can be crushed by TrackedVehicle/Mech, turns in place instantly.
2. Spawn a WheeledVehicle — verify it cannot turn in place, has a fixed turn radius when moving, and can reverse.
3. Spawn a TrackedVehicle — verify it can spin in place, turn radius widens at speed, and crushes enemy LightInfantry on contact.
4. Spawn a HoverVehicle — verify momentum-based movement with drag (slides when changing direction).
5. Spawn a HoverCraft (air) — verify it hovers, doesn't collide with ground units, has soft separation from other air units.
6. Spawn a Glider — verify it never stops moving, circles at idle speed when no orders, wider turns at higher speed.
7. Verify ground units cannot overlap each other (hard collision).
8. Verify air units push away from each other gently (soft separation) but can overlap slightly.
9. Select a turret unit — verify turret rotates independently from the base within its TurnAngle arc.
10. Test directional armor units (vehicles/mechs) — verify frontal hits deal less damage than rear hits.
