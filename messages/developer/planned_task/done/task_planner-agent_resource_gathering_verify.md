# agent-resource-gathering-verify

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-agent-resource-gathering.md

## Task

Verify that Agent resource gathering and drop-off behaviors are fully implemented and working correctly. The implementation already exists — this is a verification-only task.

**What should already exist (verify all are present and correct):**

1. **GatheringResourceBehavior** component (behavior.rs) with GatherPhase enum: MovingToResource, Extracting, MovingToTunnel, DroppingOff
2. **DroppingOffResourcesBehavior** component (behavior.rs) with DropOffPhase enum: MovingToTunnel, DroppingOff
3. **AgentCarryState** component (types.rs) tracking crystals and supplies carried
4. **Constants** in unit_data.rs: AGENT_MINING_DURATION=48, AGENT_PICKUP_DURATION=48, AGENT_DROPOFF_DURATION=48, AGENT_CRYSTAL_CARRY=50, AGENT_SUPPLY_CARRY=1
5. **gathering_resource_behavior_system** (behaviors.rs) — full gather-deliver cycle with side occupancy checking
6. **dropping_off_resources_behavior_system** (behaviors.rs) — standalone drop-off with side occupancy checking
7. **Right-click integration** (core.rs) — Crystal patch and SDS right-click resolves to Gather command for Agents
8. **Side logic** — drop_off_side_for_carry: crystals→Side B, supplies→Side C
9. **Occupancy enforcement** — only one Agent per side at a time, crystal and supply deliveries can happen simultaneously
10. **Resource transfer** — resources added to SyndicatePlayerResources on drop-off completion

Run `cargo test` and verify all existing tests pass. If any piece is missing or broken, implement the fix. If everything passes, the task is complete.

## Technical Context

All 10 items listed above are confirmed present in the codebase. Here is a file-by-file map:

### Types & Components
- **`artifacts/developer/src/game/units/types/state/behavior.rs`**:
  - `GatherPhase` enum (line ~210): `MovingToResource`, `Extracting { frames_remaining }`, `MovingToTunnel { tunnel_entity, side_position }`, `DroppingOff { tunnel_entity, frames_remaining }`
  - `GatheringResourceBehavior` component (line ~229): `target_resource`, `resource_type`, `phase`, `path`, `path_index`
  - `DropOffPhase` enum (line ~248): `MovingToTunnel`, `DroppingOff { frames_remaining }`
  - `DroppingOffResourcesBehavior` component (line ~263): `target_tunnel`, `phase`, `path`, `path_index`
  - Tests for these types exist in the same file (lines ~710-782)

- **`artifacts/developer/src/game/units/types/state/types.rs`**: `AgentCarryState` component with `crystals: u32` and `supplies: u32`

- **`artifacts/developer/src/game/units/types/unit_data.rs`** (line ~185-197): All 5 constants present with correct values

### Behavior Systems
- **`artifacts/developer/src/game/units/systems/behaviors.rs`**:
  - `drop_off_side_for_carry()` (line ~594): Returns 'B' for crystals, 'C' for supplies
  - `gathering_resource_behavior_system()` (line ~609): Full gather cycle — movement, extraction, path to tunnel, drop-off, resource transfer to `SyndicatePlayerResources`, occupancy enforcement via occupied_sides vec
  - `dropping_off_resources_behavior_system()` (line ~778): Standalone drop-off — movement to tunnel side, occupancy wait, drop-off timer, resource transfer
  - Both systems registered in `artifacts/developer/src/game/units/mod.rs` (line ~27), running after `rebuild_occupancy_map`
  - **Existing tests** (lines ~2124-2600+): ~20 tests covering arrival threshold, side selection, phase transitions, resource transfer, cancellation on despawned tunnel/resource

### Right-Click Integration
- **`artifacts/developer/src/game/units/systems/core.rs`**:
  - Crystal patch right-click → `UnitCommand::Gather(target_entity)` (line ~365)
  - Supply Delivery Station right-click → `UnitCommand::Gather(target_entity)` (line ~383)
  - Own Tunnel right-click when carrying → `UnitCommand::DropOffResources(target_entity)` (line ~403)
  - DropOff target click mode (line ~287-302)
  - Test at line ~2294 for DropOff target click

### Verification Steps
1. Run `cargo test` — all tests should pass. Focus on test names containing `gather`, `drop_off`, `dropoff`
2. If any test fails, the fix will be in the file/line ranges above
3. If all pass, produce the task_completion — no code changes needed

## Dependencies

None — this is a verification-only task on existing, self-contained implementation.
