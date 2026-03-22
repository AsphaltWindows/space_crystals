# unit_crushing_mechanic

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-unit_bases_movement_collision_r1.md

## Task

Implement the unit crushing mechanic: TrackedVehicle and Mech units crush enemy LightInfantry on contact.

### Behavior

Per design doc, crushing occurs when a unit with the `can_crush` property (TrackedVehicle, Mech — see `UnitBaseData`) moves over an enemy unit with the `crushable` property (LightInfantry only).

### Implementation

1. Create a `crushing_system` that runs in Phase 3 (after movement systems) or Phase 4:
   - Query all units with `UnitBaseEnum`, `Transform`, `Owner`, `Silhouette`
   - For each unit whose `UnitBaseData` has a can-crush property (TrackedVehicle data has `crushable: false` but that's the crushable flag — need to check which bases CAN crush), check overlap with enemy LightInfantry units
   - Overlap check: AABB overlap between crusher silhouette and crushee silhouette at their current positions
   - On overlap: instantly kill the crushable unit (set HP to 0 or despawn via existing `remove_dead_entities_system`)
   - Only crush ENEMY units (different `Owner`)

2. The crushing property mapping from the design doc:
   - TrackedVehicle: crushes LightInfantry ✓
   - Mech: crushes LightInfantry ✓
   - LightInfantry: crushable ✓ (already `crushable: true` in `UnitBaseData`)
   - HeavyInfantry: NOT crushable (`crushable: false`)
   - All others: neither crush nor are crushable

3. Register the system in `UnitsPlugin` after movement systems complete.

### Notes
- The `UnitBaseData` struct already has a `crushable` field. You may need to add a `can_crush` field or derive it from the unit base enum directly (TrackedVehicle and Mech are the only crushers per design).
- Check `UnitBaseEnum::data()` in `movement.rs` for the existing `crushable` field.
