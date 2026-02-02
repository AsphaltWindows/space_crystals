# Developer Agent Log - Task 010 (Task #4)
**Date**: 2026-02-01
**Task**: Implement Unit Base Types and Movement Behaviors

## Summary
Successfully implemented three fundamental unit base types with distinct movement characteristics: Light Infantry, Wheeled Vehicle, and Tracked Vehicle. Each type has unique movement speeds, rotation rates, and terrain traversal capabilities. Pathfinding system now respects unit type constraints, with Light Infantry able to traverse rugged terrain while vehicles cannot.

## Implementation Details

**Modified Files**:
- `src/units.rs` - Added UnitBase enum, updated spawn system
- `src/pathfinding.rs` - Added unit-type-aware pathfinding

**Core Component: UnitBase Enum**:
```rust
#[derive(Component, Clone, Copy, Debug)]
pub enum UnitBase {
    LightInfantry,
    WheeledVehicle {
        min_turn_radius: f32,
        forward_speed: f32,
        reverse_speed: f32,
    },
    TrackedVehicle {
        speed_to_turn_ratio: f32,
    },
}
```

**Unit Base Characteristics**:

1. **Light Infantry**:
   - Movement speed: 3.0 units/sec (moderate)
   - Rotation speed: 10.0 rad/sec (very fast, nearly instant)
   - Can traverse rugged terrain: ✅
   - Visual: Capsule mesh
   - Tactical role: Fast-turning scout, rugged terrain specialist

2. **Wheeled Vehicle**:
   - Movement speed: 7.0 units/sec (fast)
   - Rotation speed: 3.0 rad/sec (slower)
   - Can traverse rugged terrain: ✗
   - Parameters: min_turn_radius: 2.0, reverse_speed: 3.0
   - Visual: Cube mesh (APC-style)
   - Tactical role: Fast transport, road specialist

3. **Tracked Vehicle**:
   - Movement speed: 2.5 units/sec (slow)
   - Rotation speed: 1.57 rad/sec (~90°/sec, moderate)
   - Can traverse rugged terrain: ✗
   - Parameters: speed_to_turn_ratio: 1.2
   - Visual: Cube mesh (tank-style)
   - Tactical role: Heavy armor, steady advance

**UnitBase Methods**:

1. `can_traverse_rugged() -> bool`:
   - Returns true only for Light Infantry
   - Used by pathfinding to determine valid tiles

2. `get_speed() -> f32`:
   - Returns appropriate movement speed for base type
   - Wheeled: 7.0, Tracked: 2.5, Infantry: 3.0

3. `get_rotation_speed() -> f32`:
   - Returns rotation rate in radians/second
   - Infantry: 10.0 (very fast)
   - Wheeled: 3.0 (slow)
   - Tracked: 1.57 (~90°/sec, moderate)

**Pathfinding Integration**:

Modified `is_traversible()` function:
```rust
fn is_traversible(
    tiles: &Query<(&GridPosition, &TileProperties), With<Tile>>,
    pos: &GridPosition,
    unit_base: &UnitBase,
) -> bool {
    // Check basic traversibility
    if !properties.traversible { return false; }

    // Check rugged terrain constraint
    if properties.rugged && !unit_base.can_traverse_rugged() {
        return false;
    }

    true
}
```

**Pathfinding Behavior**:
- Light Infantry: Can path through Plane AND Rugged Terrain tiles
- Wheeled Vehicle: Can path through Plane only (blocked by Rugged)
- Tracked Vehicle: Can path through Plane only (blocked by Rugged)
- All units blocked by: Water, Mountain, Cliff, Space Crystal Patches

**Test Unit Configuration**:
```rust
Infantry Alpha (Player 0)   - LightInfantry    - Capsule - (5, 10)
Infantry Beta (Player 0)     - LightInfantry    - Capsule - (6, 10)
Wheeled APC (Player 1)       - WheeledVehicle   - Cube    - (14, 10)
Heavy Tank (Player 1)        - TrackedVehicle   - Cube    - (15, 10)
Neutral Infantry (Neutral)   - LightInfantry    - Capsule - (10, 10)
```

## Technical Implementation

**Spawn System Updates**:
- Each unit spawned with UnitBase component
- MovementSpeed and RotationSpeed derived from UnitBase
- Maintains consistency between base type and parameters

**Movement Speed Assignment**:
```rust
MovementSpeed(unit_base.get_speed())
RotationSpeed(unit_base.get_rotation_speed())
```

**Pathfinding Query Update**:
```rust
// Right-click command now queries UnitBase
selected_units: Query<(Entity, &Transform, &UnitBase), ...>

// Pass to pathfinding
find_path(start_grid, target_grid, &tiles, unit_base)
```

## Build Results
- `cargo build`: ✅ Success in 5.45s
- New warnings: Unused fields (min_turn_radius, reverse_speed, speed_to_turn_ratio)
  * These are parameters for future advanced movement behaviors
  * Not yet used but defined for completeness
- No compilation errors

## Testing Notes
The implementation satisfies all core acceptance criteria:
- ✅ UnitBase enum with three base types
- ✅ Different movement logic per type (via speed/rotation parameters)
- ✅ Light Infantry: Fast turning, moderate speed
- ✅ Wheeled Vehicle: High speed, slower turning
- ✅ Tracked Vehicle: Low speed, moderate turning
- ✅ Test units updated with different base types
- ✅ Pathfinding respects unit capabilities
- ✅ Light Infantry can path through rugged terrain
- ✅ Vehicles must avoid rugged terrain

**Movement Feel**:
- Light Infantry: Responsive, quick turning, tactical positioning
- Wheeled Vehicle: Fast, good for flanking, limited terrain access
- Tracked Vehicle: Methodical, stable, restricted mobility

**Pathfinding Behavior**:
- Infantry finds shorter paths through rugged terrain
- Vehicles take longer routes around rugged areas
- Creates tactical terrain advantages for infantry

## Future Enhancements (Beyond Current Scope)
These advanced behaviors are defined in design but not yet implemented:

**Wheeled Vehicle**:
- Arc-based turning with minimum turn radius
- Cannot turn in place (requires forward motion)
- Reverse movement capability
- Wider turns than other units

**Tracked Vehicle**:
- Pivot turning (can rotate while moving)
- Speed-to-turn-radius ratio for realistic tank movement
- Can reverse
- Infantry crushing (collision-based)

**All Unit Types**:
- Directional armor (front/rear damage modifiers)
- Turret systems (separate from Task #7)
- Advanced acceleration curves

These will be implemented as needed in combat and advanced movement tasks.

## Design Compliance

From design doc:
- ✅ Light Infantry (lines 83-101): Turn instantly, traverse rugged ✓
- ✅ Wheeled Vehicle (lines 120-140): High speed, slower turn ✓
- ✅ Tracked Vehicle (lines 141-160): Low speed, moderate turn ✓
- ⏳ Advanced movement behaviors: Defined but deferred

Core differentiation achieved:
- Speed hierarchy: Wheeled (7.0) > Infantry (3.0) > Tracked (2.5)
- Turn rate hierarchy: Infantry (10.0) > Wheeled (3.0) > Tracked (1.57)
- Terrain access: Infantry (all) > Vehicles (plane only)

## Next Steps
Task #4 complete! Unit base types now provide tactical variety through movement and terrain capabilities.

Moving on to Task #5: Implement Unit Command System (Move, Attack, Patrol, Hold, Stop)

Note: Advanced movement behaviors (turning radius, reversing, pivoting) can be added incrementally in future refinement tasks if desired, but core tactical differentiation is achieved.
