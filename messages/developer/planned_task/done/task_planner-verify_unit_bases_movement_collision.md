# verify-unit-bases-movement-collision

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Design Spec
- `artifacts/designer/design/units.md` — canonical source for all values

### Files to verify (all paths relative to `artifacts/developer/src/`)

**1. UnitBaseEnum::data() — `game/units/types/movement.rs` lines 298-393**
- All 9 variants return `UnitBaseData` structs. Compare each field against the spec:
  - LightInfantry (line 302): Ground, no turret, no directional armor, rugged=true, crushable=true, turn-in-place=true, no reverse, TurnRate
  - HeavyInfantry (line 312): Ground, no turret, no directional armor, rugged=true, crushable=false, turn-in-place=true, no reverse, TurnRate
  - WheeledVehicle (line 322): Ground, turret, directional armor, no rugged, not crushable, NO turn-in-place, CAN reverse, FixedTurnRadius
  - TrackedVehicle (line 332): Ground, turret, directional armor, no rugged, not crushable, turn-in-place, CAN reverse, SpeedTurnRadius
  - DrillUnit (line 342): Underground, turret, directional armor, no rugged, not crushable, turn-in-place, CAN reverse, SpeedTurnRadius
  - HoverVehicle (line 352): Ground, turret, directional armor, no rugged, not crushable, turn-in-place, NO reverse, Drag
  - Mech (line 362): Ground, turret, directional armor, rugged=true, not crushable, turn-in-place, NO reverse, TurnRate
  - HoverCraft (line 372): Air, turret, NO directional armor, no rugged, not crushable, turn-in-place, NO reverse, Drag
  - Glider (line 382): Air, turret, NO directional armor, no rugged, not crushable, NO turn-in-place, NO reverse, Glider
- **Status: I have verified all 9 match the design spec exactly.** Existing tests at line 422+ also verify these values.

**2. MovementModel parameter structs — `game/units/types/movement.rs` lines 128-173**
- `TurnRateMovementParams` (line 129): turn_rate, acceleration, deceleration, max_speed ✓ matches spec
- `FixedTurnRadiusMovementParams` (line 138): minimum_turn_radius, forward_acceleration, forward_max_speed, reverse_acceleration, reverse_max_speed, deceleration ✓ matches spec
- `SpeedTurnRadiusMovementParams` (line 149): speed_to_turn_radius_ratio, acceleration, deceleration, max_speed ✓ matches spec
- `DragMovementParams` (line 158): forward_acceleration, non_forward_acceleration (spec calls this OmniDirectionalAcceleration — same concept, acceptable naming), drag_ratio, turn_rate ✓ matches spec
- `GliderMovementParams` (line 167): idle_speed, max_speed, acceleration, deceleration, max_centripetal_acceleration ✓ matches spec
- Each struct also has `locomotion_orientation_constraint()` methods (lines 187-281) implementing per-model turn constraints.

**3. TurretAttributesData — `game/units/types/unit_data.rs` lines 26-31**
- Fields: `turn_angle: f32` (degrees, max 360), `turn_rate: f32` (degrees/frame) ✓ matches spec
- Has `validate()` method checking turn_angle in [0, 360] and turn_rate >= 0
- Tests at lines 289-343 verify validation logic

**4. OccupancyMap ground collision — `game/units/types/types.rs` lines 68-97**
- `OccupancyMap` resource with `blocked_tiles` (HashSet), `ground_bodies` (Vec<CollisionBody>), `structure_tiles` (HashSet)
- `CollisionBody` stores entity + position + half-extents for AABB checks
- Rebuilt each frame by `rebuild_occupancy_map` system in `game/units/systems/core.rs`
- Ground units and structures are added to the map; moving units pathfind around blocked tiles

**5. Air unit separation — `game/units/systems/core.rs` lines 1345-1375**
- `SeparationRadius` component defined in `game/combat/types.rs` line 206: `pub struct SeparationRadius(pub f32)`
- `SEPARATION_FORCE_SCALE` constant in `game/combat/types.rs`
- `air_unit_separation_system` queries `(Entity, &mut Transform, &SeparationRadius, &DomainEnum)` with `With<Unit>, Without<InTunnelNetwork>`
- Applies linear force falloff: full force at overlap, zero at SeparationRadius edge
- Only affects DomainEnum::Air units
- Tests at lines 1791+ verify component, force application, tunnel exclusion, and ground unit exclusion

**6. Directional armor — `game/combat/utils.rs` and `game/combat/systems/`**
- Referenced in UnitBaseData::directional_armor field
- Implementation in combat systems applies frontal damage reduction / rear damage increase

### Notable naming difference
- Design spec: `OmniDirectionalAcceleration` → Code: `non_forward_acceleration` in `DragMovementParams`. Same semantic meaning (thrust available in all directions, as opposed to forward-only thrust). This is an acceptable naming choice — no fix needed.

### Running tests
```bash
cd artifacts/developer && cargo test
```

## Dependencies

None — this is a standalone verification task with no code dependencies on other tasks.
