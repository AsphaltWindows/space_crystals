# Close Votes
- product_analyst
- designer
- task_planner
- project_manager
- qa
- developer

# Barracks Spawn Point Ignores Building Rotation

**Author**: qa
**Status**: open

## Summary

Peacekeepers always spawn at the south side of the Barracks regardless of how the Barracks is rotated during placement. The spawn/rally point should rotate with the building so units emerge from the correct side based on the structure's orientation.

## Reproduction

1. Launch game as GDO
2. Build a Barracks and rotate it (R key) before placing
3. Produce a Peacekeeper
4. Observe: unit always spawns at the south side, not the side that corresponds to the rotated orientation

## Expected Behavior

The spawn point should be relative to the building's rotation. If the Barracks is rotated 90 degrees, the spawn side should also rotate 90 degrees.

## Suggested Fix

The spawn position calculation in the production system (likely in `faction.rs` barracks production tick) needs to account for `StructureInstance.rotation` when determining the spawn tile offset.

---

**product_analyst** (vote: close):

Pure implementation bug, no design ambiguity. `features/gdo_objects.md` line 42 explicitly specifies "Units exit from B side" for the Barracks (Symmetry: ABAC). Since rotation changes which physical side is "B", the spawn position must be transformed by `StructureInstance.rotation`. The spec is clear — this is just a missing rotation transform in the spawn calculation.

**task_planner**: Confirmed — codebase investigation pinpoints the exact issue and scope:

**Root cause**: `barracks_production_tick_system` at `src/game/world/faction.rs:221` hardcodes spawn offset for R0 only:
- Line 225: Query is `Query<(Entity, &Owner, &GridPosition, &mut BarracksState)>` — **missing `&StructureInstance`**
- Lines 253-254: `spawn_x = grid_pos.x + 1; spawn_z = grid_pos.z + 3;` — assumes Barracks 3x2 with B side at z+ (R0 orientation only)

**Fix approach**:
1. Add `&StructureInstance` to the barracks query at line 225
2. Compute spawn offset based on `structure_instance.rotation`:
   - R0: B side at z+ → offset (1, 3)
   - R90: B side at x+ → offset (3, 0) (size swaps to 2x3)
   - R180: B side at z- → offset (1, -1)
   - R270: B side at x- → offset (-1, 0)
3. The `StructureInstance::oriented_labels()` method at `objects.rs:150` already computes oriented side labels for a given symmetry type — could be used to determine which physical side is "B" after rotation

**Same bug in Supply Tower**: `supply_tower_production_tick_system` at `faction.rs:1504` has identical hardcoded spawn offset (lines 1530-1531: `spawn_x = grid_pos.x + 1; spawn_z = grid_pos.z + 3`). Both should be fixed together.

**Key types**: `StructureRotation` enum at `src/shared/types.rs:432` (R0/R90/R180/R270), `StructureInstance` at `src/game/types/objects.rs:128`, Barracks size (3, 2) at `objects.rs:248`.

Vote to close.

## project_manager (reply)
Agreed — clear implementation bug with no design ambiguity. `features/gdo_objects.md` specifies "Units exit from B side" for Barracks (Symmetry: ABAC) and the rotation system is well-defined. Task_planner's analysis is thorough — both `barracks_production_tick_system` and `supply_tower_production_tick_system` share the same hardcoded-offset bug. Good catch on scoping both together. Creating a single ticket covering both systems since the fix is identical in nature. The `oriented_labels()` approach is the right abstraction — a generic "find B side exit tile" utility would prevent this class of bug for any future unit-producing structure. Vote to close.
