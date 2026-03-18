# Ticket: Fix Spawn Point to Respect Building Rotation

## Current State
`barracks_production_tick_system` at `src/game/world/faction.rs:221` hardcodes the spawn offset to `(grid_pos.x + 1, grid_pos.z + 3)`, which corresponds to the B side position for R0 rotation only. The system query (line 225) does not include `&StructureInstance`, so it has no access to the building's rotation. Produced Peacekeepers always spawn at the south side regardless of how the Barracks was rotated during placement.

The same bug exists in `supply_tower_production_tick_system` at `faction.rs:1504`, which hardcodes spawn offset at lines 1530-1531 with the same `(x + 1, z + 3)` pattern for the free Supply Chopper spawn on placement and produced choppers.

## Desired State
Both production tick systems compute spawn position relative to the building's actual rotation:
1. Add `&StructureInstance` to the barracks query at line 225 and the supply tower query.
2. Compute the B-side exit tile based on `structure_instance.rotation`:
   - R0: B side at z+ direction -> offset (1, 3) for 3x2 Barracks
   - R90: B side at x+ direction -> offset (3, 0) (size becomes 2x3)
   - R180: B side at z- direction -> offset (1, -1)
   - R270: B side at x- direction -> offset (-1, 0)
3. Consider using `StructureInstance::oriented_labels()` (at `objects.rs:150`) to generically determine which physical side is "B" for the current rotation, making the solution reusable for any future unit-producing structure.
4. Apply the equivalent rotation-aware offsets for Supply Tower (size 3x3, Symmetry AAAA — since all sides are A, the spawn side convention may differ; verify which side is the intended exit).

## Justification
`features/gdo_objects.md` specifies Barracks as Size 3x2, Symmetry ABAC, with "Units exit from B side." The symmetry labels rotate with the building — when a Barracks is placed at R90, the physical B side faces a different direction. The current hardcoded offset only works for R0, making rotated Barracks spawn units from the wrong side. Originated from forum topic `barracks_spawn_point_ignores_rotation.md`.

## QA Steps
1. Launch the game as GDO.
2. Build a Barracks at default rotation (R0). Produce a Peacekeeper. Note which side the unit spawns from (should be the B side — the long edge).
3. Build a second Barracks rotated 90 degrees (press R once before placing). Produce a Peacekeeper. Verify: unit spawns from the rotated B side (90 degrees from the first Barracks).
4. Build a third Barracks rotated 180 degrees. Produce a Peacekeeper. Verify: unit spawns from the opposite side compared to the R0 Barracks.
5. Build a fourth Barracks rotated 270 degrees. Produce a Peacekeeper. Verify: unit spawns from the 270-degree rotated B side.
6. Build a Supply Tower at default rotation. Verify the free Supply Chopper spawns from the correct side.
7. Build a Supply Tower rotated 90 degrees. Verify the free Supply Chopper spawns from the rotated correct side.
8. Produce additional Supply Choppers from both towers and verify spawn positions match their rotations.

## Expected Experience
- Peacekeepers emerge from the B side of the Barracks regardless of rotation — the exit side visually rotates with the building.
- Supply Choppers spawn from the correct side of the Supply Tower regardless of rotation.
- The spawn position is always adjacent to and outside the building footprint (units don't spawn inside the structure).
