# verify-unit-bases-movement-collision

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-unit-bases-movement-collision.md

## Task

**Verification task — all code already exists.** Verify that the UnitBase types, MovementModel definitions, TurretAttributes, and UnitCollision implementations match the design spec in `artifacts/designer/design/units.md`.

All 9 UnitBaseEnum variants are fully implemented in `game/units/types/movement.rs` with correct data (Domain, HasTurret, DirectionalArmor, RuggedTerrain, Crushable, CanTurnInPlace, CanReverse, MovementModel). All 5 MovementModel parameter structs (TurnRateMovementParams, FixedTurnRadiusMovementParams, SpeedTurnRadiusMovementParams, DragMovementParams, GliderMovementParams) are defined with locomotion/orientation constraint methods. TurretAttributesData is in `game/units/types/unit_data.rs`. Ground collision uses OccupancyMap AABB in `game/units/types/types.rs`. Air soft separation uses SeparationRadius + `air_unit_separation_system` in `game/units/systems/core.rs`. Directional armor is in `game/combat/utils.rs` and `game/combat/systems/`.

Verify:
1. All 9 UnitBaseEnum::data() values match the design doc exactly
2. All 5 MovementModel parameter structs have the correct fields per spec
3. TurretAttributesData has TurnAngle (degrees, max 360) and TurnRate (degrees/frame)
4. OccupancyMap ground collision prevents unit overlap via AABB
5. Air unit separation system applies soft repulsion via SeparationRadius
6. Existing tests pass (`cargo test`)

If any discrepancy is found, fix it. If everything matches, this task is complete with no code changes needed.
