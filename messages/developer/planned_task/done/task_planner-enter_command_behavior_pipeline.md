# enter-command-behavior-pipeline

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-enter-command-tunnel.md

## Task

Wire UnitCommand::Enter to EnteringTunnelBehavior and fix the behavior system's tunnel network integration.

### What exists already
- `EnteringTunnelBehavior` component in `src/game/units/types/state/behavior.rs` (line ~135)
- `entering_tunnel_behavior_system` in `src/game/units/systems/behaviors.rs` (line ~420) — moves unit toward tunnel, despawns on arrival
- `UnitCommand::Enter(Entity)` variant in `src/game/units/types/state/commands.rs`
- `can_enter_tunnel()` validation in `src/game/units/utils.rs` (line ~157)
- `InTunnelNetwork` component used by production system (`src/game/world/faction.rs` line ~472)
- `tunnel_side_world_position()` utility re-exported from `src/game/units/utils.rs`

### What needs to be implemented

1. **Command dispatch system**: Create a system (or add to an existing one) that detects units with `UnitCommand::Enter(tunnel_entity)` and:
   - Validates the Enter command using `can_enter_tunnel()` (check faction, owner match, tier sufficiency)
   - If valid: inserts `EnteringTunnelBehavior::new(tunnel_entity)` on the unit, pathfinds to Side A position using `tunnel_side_world_position()`
   - If invalid (e.g., tier too low): rejects the command (remove UnitCommand, log warning)

2. **Fix entering_tunnel_behavior_system**: Currently the system calls `commands.entity(entity).despawn()` on arrival at Side A. This is wrong — the production system uses `InTunnelNetwork` + `Visibility::Hidden` to keep entities queryable for ejection. Fix to:
   - Remove `EnteringTunnelBehavior` marker
   - Insert `InTunnelNetwork { owner_player }` (get owner from the unit's Owner component)
   - Set `Visibility::Hidden`
   - Remove movement-related components (MoveTarget, Path, Velocity, etc.)
   - Set locomotion/orientation channels to stopped/maintaining
   - Do NOT despawn the entity

3. **Tests**:
   - Test that UnitCommand::Enter triggers EnteringTunnelBehavior insertion
   - Test that tier validation rejects invalid entries
   - Test that on arrival, unit gets InTunnelNetwork + Hidden (not despawned)
   - Test that the unit entity still exists after entering the network

## Technical Context

### Files to modify

1. **`artifacts/developer/src/game/units/systems/behaviors.rs`** — Primary file
   - **Fix `entering_tunnel_behavior_system` (line 420-458)**: Currently despawns entity on arrival (line 451). Must instead:
     - The system query needs to be expanded to include `&Owner` and `&mut Visibility`
     - On arrival (distance < TUNNEL_ARRIVAL_THRESHOLD at line 448):
       - `commands.entity(entity).remove::<EnteringTunnelBehavior>()` (already done for cancel case at line 439)
       - `commands.entity(entity).insert(InTunnelNetwork { owner_player: owner.0.unwrap_or(0) })`
       - Set `*visibility = Visibility::Hidden`
       - Remove movement components: `.remove::<MoveTarget>().remove::<Path>()` and `.insert(Velocity(Vec3::ZERO))`
       - Set `*locomotion = LocomotionChannel::Stopping` and `*orientation = OrientationChannel::Maintaining`
       - Do NOT call `.despawn()`
   - **Add `enter_command_dispatch_system` (new system)**: Pattern after existing systems in this file. Queries:
     - `Query<(Entity, &UnitCommand, &ObjectInstance, &Owner), (With<Unit>, Without<EnteringTunnelBehavior>, Without<InTunnelNetwork>)>` — units with Enter command but no behavior yet
     - `Query<(&TunnelState, &Owner, &Transform, &StructureInstance), With<TunnelState>>` — tunnel targets for validation
   - Dispatch logic:
     - Match `UnitCommand::Enter(tunnel_entity)` 
     - Look up tunnel with `tunnels.get(tunnel_entity)` — cancel if tunnel doesn't exist
     - Determine `is_syndicate` from `ObjectInstance.object_type` (match `ObjectEnum::SyndicateAgent | ObjectEnum::SyndicateGuard => true`)
     - Determine `unit_base` from ObjectEnum: `SyndicateAgent => agent_type_data().unit_base`, `SyndicateGuard => guard_type_data().unit_base` (pattern at `command_panel.rs:1870-1873`)
     - Call `can_enter_tunnel(is_syndicate, unit_owner.0, tunnel_owner.0, &unit_base, &tunnel_state.tier)`
     - If Ok: insert `EnteringTunnelBehavior::new(tunnel_entity)` on the unit
     - If Err: remove `UnitCommand` (set to Idle), log warning
   - Add tests in the existing `#[cfg(test)] mod tests` section

2. **`artifacts/developer/src/game/units/mod.rs`** — Register the new system
   - Add `systems::behaviors::enter_command_dispatch_system` to the Phase 2 behavior systems tuple (line 19-28)
   - It should run `.after(systems::core::rebuild_occupancy_map)` like other behavior systems

3. **`artifacts/developer/src/game/units/types/state/behavior.rs`** — No changes needed (InTunnelNetwork already defined at line 324)

### Key types and imports

- `EnteringTunnelBehavior` — Component (line 135 of behavior.rs), has `::new(entity)` constructor
- `InTunnelNetwork { owner_player: u8 }` — Component (line 324 of behavior.rs)
- `LocomotionChannel` / `OrientationChannel` — mutable action channels (behavior.rs)
- `Owner(pub Option<u8>)` — Component (`shared/types.rs:32`)
- `ObjectInstance { object_type: ObjectEnum, .. }` — Component (`game/types/objects.rs:55`)
- `TunnelState { tier: TunnelTier, .. }` — Component (`game/types/structures.rs:572`)
- `StructureInstance` — Component needed for `tunnel_side_world_position()` (`game/types/objects.rs:128`)
- `can_enter_tunnel(is_syndicate, unit_owner, tunnel_owner, unit_base, tunnel_tier) -> Result<(), &str>` — (`game/units/utils.rs:157`)
- `tunnel_side_world_position(transform, structure_instance, side_char)` — returns `Vec3` (`game/units/systems/behaviors.rs:543`, re-exported from `utils.rs`)
- `MoveTarget`, `Path`, `Velocity`, `HoldingPosition`, `IdleOrigin` — movement components
- `Unit` marker component (`shared/types.rs:28`)
- `agent_type_data()`, `guard_type_data()` — for getting `unit_base` from type data (`game/units/types/unit_data.rs`)
- `UnitCommand::Idle` — the idle state to set when rejecting commands

### Patterns to follow

- **Behavior system pattern**: See `building_behavior_system` (line 473) and `building_tunnel_behavior_system` (line 892) for query structure. They query mutable channels + command + behavior marker.
- **Test setup pattern**: See `spawn_agent_with_build_tunnel()` helper (line 1558-1569) — spawns entity with Transform, Owner, behavior marker, channels, command, behavior state, Visibility. Tests use `App::new()` + `MinimalPlugins` + `Assets<Mesh>` + `Assets<StandardMaterial>` then `world.run_system_once(system_fn)`.
- **Command dispatch at right-click**: The right-click handler (`core.rs:390-411`) already inserts `UnitCommand::Enter(target_entity)` on agents clicking own tunnels. The new dispatch system picks up entities with this command and inserts the behavior marker.
- **ObjectEnum to is_syndicate**: Match `ObjectEnum::SyndicateAgent | ObjectEnum::SyndicateGuard` (pattern used throughout `core.rs:254,291,357,375,393`)
- **ObjectEnum to UnitBaseEnum**: Match like `command_panel.rs:1870-1873` using `agent_type_data().unit_base` / `guard_type_data().unit_base`

### System ordering (Bevy ECS)

- The dispatch system should run in Phase 2 (behavior systems), after `rebuild_occupancy_map`
- It must run BEFORE `entering_tunnel_behavior_system` so the behavior marker is present when the behavior system processes it (add `.before(entering_tunnel_behavior_system)` or use system ordering)
- The behavior system already runs in Phase 2 at line 24 of `mod.rs`

### Important details

- The `entering_tunnel_behavior_system` currently approximates Side A as tunnel center (`tunnel_transform.translation` at line 444). This is acceptable for now — the task description doesn't require changing this, but `tunnel_side_world_position()` is available if precision is desired.
- `InTunnelNetwork` is already filtered out by `air_unit_separation_system` (core.rs:1353 uses `Without<InTunnelNetwork>`)
- The ejection system in `faction.rs:1819-1862` already handles removing `InTunnelNetwork` and restoring visibility when ejecting units
- `UnitCommand::Idle` is the standard idle state — check if it exists as a variant or if removing UnitCommand entirely is the pattern (look at how `building_behavior_system` handles completion at line 487+)

## Dependencies

- **enter-right-click-integration** (sibling task): The right-click handler in `core.rs:390-411` already inserts `UnitCommand::Enter(target_entity)`. This dispatch system picks up that command. If both tasks are done in parallel, no conflict — the right-click code already exists.
- **Ejection system** (existing, `faction.rs:1819-1862`): The ejection pipeline reads `InTunnelNetwork` to find units to eject. This task's fix ensures units entering tunnels get `InTunnelNetwork` instead of being despawned, making them visible to the ejection system.
- **Production system** (existing, `faction.rs:472`): Already inserts `InTunnelNetwork` on newly produced units. This task follows the same pattern for manually entered units.
