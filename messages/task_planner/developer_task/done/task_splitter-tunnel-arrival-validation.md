# tunnel-arrival-validation

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-worker-built-validation.md

## Task

Add arrival validation to `build_tunnel_behavior_system` in `artifacts/developer/src/game/units/systems/behaviors.rs`.

Currently, when a `BuildingTunnelBehavior` unit arrives at its target (the `BuildTunnelPhase::MovingToSite` branch, ~line 929), it only checks single-agent enforcement and cost before spawning the tunnel. It does NOT validate that the tiles are still buildable and unoccupied.

**What to implement:**

After the `BUILD_ARRIVAL_THRESHOLD` distance check passes (line 929) and before the single-agent enforcement check (line 932), add a call to `can_worker_place_structure()` to validate the placement:

1. Convert the target world position to grid coordinates using `world_to_grid(target, 1.0)`
2. Get the tunnel's footprint size (1x1 for a base Tunnel — use `ObjectEnum::Tunnel.object_type().size`)
3. Call `can_worker_place_structure(grid_x, grid_z, size_x, size_z, &tiles, &structures)`
4. If validation fails (`Err`): cancel the build — set `UnitCommand::Idle`, `BaseBehaviorState::None`, `LocomotionChannel::Stationary`, `OrientationChannel::Maintaining`, remove `BuildingTunnelBehavior`, log the rejection, and `continue`

**System parameter additions needed:**
The `build_tunnel_behavior_system` currently does not query tiles or structures. You need to add these query parameters:
- `tiles: Query<(&GridPosition, &TilePreset), With<Tile>>`
- `structures: Query<(&GridPosition, &StructureInstance, &ObjectInstance)>`

These are the same types used by `BuildingStructureBehavior`'s system (see ~line 475 for the pattern).

**Reference implementation:** The `building_structure_behavior_system` (~line 464) already does exactly this pattern — arrival check → `can_worker_place_structure()` → proceed or cancel. Follow the same pattern.

**Tests to add:**
- Test that arrival at a valid location proceeds to construction (existing tests may cover this)
- Test that arrival at a location with an unbuildable tile cancels the build (agent idles, behavior removed)
- Test that arrival at a location overlapping an existing structure cancels the build
