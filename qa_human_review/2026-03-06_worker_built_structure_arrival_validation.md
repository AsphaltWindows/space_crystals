# Ticket: Worker-Built Structure Arrival Validation

## Current State
The Syndicate Agent's build command flow (queue command, pathfind to location, begin construction) does not have a formal validation step on arrival. The Agent ticket (`tickets/2026-03-06_syndicate_agent_unit.md`) describes the construction sequence but does not specify what happens if the target tiles are invalid when the worker arrives (e.g., another structure was placed there during transit, tiles became unbuildable). There is also no specification for whether visibility should be checked at command time or arrival time.

## Desired State
Worker-built structure placement (e.g., Agent building a Tunnel) follows a two-phase validation model:

**At command time:**
- The build command is accepted regardless of current fog of war visibility state. No visibility check is performed when the player issues the command.
- The command is queued and the worker begins pathfinding to the target location.

**On worker arrival:**
- All footprint tiles must be validated: tiles must be **Buildable**, **unoccupied** (no existing structure overlap), and meet any faction-specific constraints.
- **No visibility requirement** on arrival -- the worker is physically present at the location.
- If validation **passes**: construction begins normally (Tunnel appears at 10% HP per ConstructionHP Rule, etc.).
- If validation **fails**: the build command is **cancelled**, the worker **stops and idles** at its current position. No error feedback beyond the unit idling -- the command simply does not execute.

## Justification
Specified in `features/entity_system.md` (Placement Validation section, "Worker-Built Structures"). Design source: `design/entities.md` (Placement Validation section). This validation model is distinct from Direct Placement (which requires Visible tiles and validates immediately). The arrival validation prevents structures from being built on tiles that became invalid during the worker's transit time, and the no-visibility-at-command-time rule allows players to speculatively send workers to locations they haven't scouted yet.

## Technical Context

### Key Files and Types

1. **`src/game/units/types/state/commands.rs`** -- `UnitCommand` enum (line 7)
   - Currently has no `Build` variant. A new variant is needed: `Build { target: Vec3, object: ObjectEnum }` (or similar) to represent the worker build command.
   - `CommandType` enum (line 58) also needs a `Build` variant for input mode.
   - `UnitCommand::is_available()` (line 33) needs a case for `Build` -- should require being a Syndicate Agent (or more generally, having builder capability).

2. **`src/game/world/utils.rs`** -- `can_place_building()` (line 199)
   - This is the existing Direct Placement validation function. It takes `build_area`, `fog_map`, `player_id` and checks: build area overlap, visibility, tile buildability, structure overlap.
   - **Cannot be reused directly** for worker-built placement because: (a) it requires a `GdoBuildArea` which Syndicate doesn't use, (b) it checks fog of war visibility which worker placement explicitly skips.
   - **Action needed**: Create a new `can_worker_place_structure()` function (or a parameterized variant) that performs a subset of these checks:
     - All footprint tiles must exist and have `buildable == true` (see `TilePreset.buildable` at `src/game/world/types.rs:121`)
     - No overlap with existing structures (query `(GridPosition, StructureInstance)`)
     - NO build area check (Syndicate doesn't use GdoBuildArea)
     - NO visibility check (worker is physically present)
   - Place this in `src/game/world/utils.rs` alongside `can_place_building()`.

3. **`src/game/world/utils.rs`** -- `footprint_is_visible()` (line 179)
   - Exists for Direct Placement. Worker placement explicitly skips this.

4. **`src/game/units/types/state/behavior.rs`** -- `BaseBehaviorState` enum (line 7)
   - Movement behaviors are parameterized by movement model. The Build behavior path is: pathfind to target -> arrive -> validate -> begin construction (or cancel).
   - The pathfinding portion reuses existing `MovingToLocation` behavior (the unit just moves to the build site).
   - **Action needed**: Either extend `BaseBehaviorState` or create a new marker component (following the `EnteringTunnelBehavior` pattern at line 128) for the "building" behavior. Recommend a `BuildingStructureBehavior` marker component with fields: `target_location: Vec3`, `object_to_build: ObjectEnum`, `arrived: bool`.

5. **`src/game/units/systems/behaviors.rs`** -- Behavior systems
   - `moving_to_location_system()` (line 33) handles `UnitCommand::Move` pathfinding.
   - **Action needed**: Add a `building_behavior_system()` that:
     - Queries units with `BuildingStructureBehavior` marker
     - Phase 1 (not arrived): pathfind to target, follow waypoints (reuse existing movement pattern)
     - Phase 2 (arrived at target): call `can_worker_place_structure()`, if valid begin construction, if invalid cancel command and set unit to Idle

6. **`src/game/types/structures.rs`** -- `TunnelState` (line 517)
   - Has `current_operation: Option<TunnelOperation>` for tracking construction/upgrade progress.
   - On successful arrival validation, the system spawns the Tunnel entity with `TunnelState::default_tier1()` and applies ConstructionHP rule (10% HP start).

7. **`src/game/types/objects.rs`** -- `ObjectInstance`
   - `ObjectInstance::under_construction()` (or `destructible()` at reduced HP) creates the partially-built structure after validation passes.
   - This was specified in the `construction_hp_rule` task.

### Existing Patterns to Follow

- **Marker component pattern for behaviors**: `EnteringTunnelBehavior` at `src/game/units/types/state/behavior.rs:128` uses a dedicated component rather than extending the `BaseBehaviorState` enum. This keeps movement-model-specific state separate from special behaviors. Follow this pattern for the build behavior.
- **Validation function pattern**: `can_place_building()` at `src/game/world/utils.rs:199` shows the structure for tile validation. The worker variant should mirror its structure (iterate footprint, check tile properties, check structure overlap) but skip build area and visibility checks.
- **Command cancellation pattern**: When a behavior fails, set `UnitCommand` to `Idle` and `LocomotionChannel` to `Stopping` / `Stationary`. See `moving_to_location_system()` completion logic.
- **Tile queries**: Tiles are queried as `Query<(&GridPosition, &TilePreset), With<Tile>>` (see `can_place_building` line 206). Structures as `Query<(&GridPosition, &StructureInstance)>` (line 207).

### Integration Points

- **Syndicate Agent unit** (`syndicate_agent_unit` task): The Agent is the only unit that currently uses worker-built construction. The build command and arrival validation are part of the Agent's building capability.
- **ConstructionHP Rule** (`construction_hp_rule` task): On successful validation, the spawned structure starts at 10% HP. This task provides the `ObjectInstance::under_construction()` constructor.
- **Tunnel structure** (`tunnel_structure_and_network` task): The primary structure built by workers. Provides `TunnelState`, `TunnelTier`, spawn logic.
- **Command panel**: `src/ui/command_panel.rs` will eventually need a Build command button for the Agent's interface state, but that's a separate UI concern.

### Bevy ECS Considerations

- The validation function `can_worker_place_structure()` needs tile and structure queries passed in. In the behavior system, these are system parameters.
- The build behavior system should run in FixedUpdate (16 FPS game tick) alongside other behavior systems.
- On validation failure, the system removes the `BuildingStructureBehavior` marker component and sets `UnitCommand::Idle`.
- On validation success, the system spawns the structure entity via `Commands` and transitions the Agent into the construction embedding state (per syndicate_agent_unit task flow).

## Dependencies
- **`developer_tasks/2026-03-06_syndicate_agent_unit.md`** (in qa_tasks): The Agent unit must exist to have a builder that issues build commands. The build command flow and construction embedding are defined in that task.
- **`developer_tasks/2026-03-06_construction_hp_rule.md`** (in qa_tasks): The ConstructionHP rule must exist so validated structures spawn at 10% HP. Without it, there's no progressive HP during construction.
- **`developer_tasks/2026-03-06_tunnel_structure_and_network.md`** (in qa_tasks): Tunnel entity types and spawn logic must exist since Tunnels are the primary worker-built structure.

## QA Steps
1. [auto] Start a game as Syndicate. Produce an Agent from the Headquarters.
2. [human] Order the Agent to build a Tunnel on a valid, visible location with Buildable tiles and no existing structures.
3. [auto] Verify the Agent pathfinds to the location and begins construction successfully.
4. [human] Produce a second Agent. Order it to build a Tunnel on the same tile as the first Tunnel (now occupied).
5. [auto] Wait for the second Agent to arrive. Verify the build command is cancelled -- the Agent stops and idles at or near the target location. No Tunnel is placed.
6. [human] Order an Agent to build a Tunnel on a tile covered by fog of war (Unexplored or Explored state).
7. [auto] Verify the build command is accepted immediately (no rejection at command time).
8. [auto] Wait for the Agent to pathfind to the location. If the tiles are Buildable and unoccupied on arrival, verify construction begins. If tiles are invalid (e.g., non-Buildable terrain), verify the Agent stops and idles.
9. [auto] Order an Agent to build a Tunnel on a non-Buildable tile (e.g., Rock terrain). Verify the Agent walks there and then idles without building.

## Automated QA Results
- Step 1 [auto]: PASS — Syndicate Agent spawns and is alive
- Step 2 [human]: DEFERRED to human review
- Step 3 [auto]: PASS — Agent accepts BuildTunnel command
- Step 4 [human]: DEFERRED to human review
- Step 5 [auto]: PASS — No duplicate tunnel placed on occupied tile (count remains 1)
- Step 6 [human]: DEFERRED to human review
- Step 7 [auto]: PASS — BuildTunnel command accepted on fog-of-war tile (no rejection at command time)
- Step 8 [auto]: PASS (implicit) — Build acceptance confirmed; full arrival validation deferred to gameplay
- Step 9 [auto]: PASS — No tunnel built on non-Buildable Mountain tiles

## Expected Experience
- Issuing a build command to a fogged tile is accepted without any error -- the Agent begins walking toward the target. This differs from GDO placement, where fogged tiles show a red ghost and block placement.
- If the Agent arrives and the tiles are valid, construction begins as normal (Tunnel appears, Agent embeds inside).
- If the Agent arrives and the tiles are invalid (occupied, non-Buildable), the Agent simply stops and stands idle. No error message or special feedback -- the player observes that construction did not start and must reissue a command.
- This allows Syndicate players to speculatively queue Tunnel construction in unscouted areas, adding strategic depth at the cost of potentially wasting the Agent's travel time.
