# supply-chopper-behaviors

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-supply-chopper.md

## Task

Implement behavior systems for SupplyChopper PickUpSupplies and AttachToTower commands, including supply transfer, attachment management, and repair.

### What already exists
- UnitCommand::PickUpSupplies(Entity) and UnitCommand::AttachToTower(Entity) are issued by right-click and AwaitingTarget resolution (core.rs)
- SupplyChopperState { carried_supplies, attached_tower } in game/types/structures.rs
- SupplyTowerState { attached_chopper, ... } in game/types/structures.rs
- SupplyDeliveryStation { current_supplies, ... } in game/world/types.rs
- MovingToObjectBehavior pattern exists for reference (units/systems/behaviors.rs)
- Free chopper spawns on ST placement (faction.rs ~line 1420) but attachment is NOT linked (supply-tower-placement-attach-chopper is a separate task)

### What needs to be implemented

1. **PickUpSupplies behavior system** (`supply_chopper_pickup_system`)
2. **AttachToTower behavior system** (`supply_chopper_attach_system`)
3. **Supply dropoff on tower landing**
4. **Detachment on command**
5. **Repair while landed/attached**
6. **Register systems**
7. **Tests**

## Technical Context

### Key Files to Modify

1. **`artifacts/developer/src/game/units/types/state/behavior.rs`** — Add new behavior marker components:
   - `PickingUpSuppliesBehavior { target_sds: Entity, phase: PickUpPhase }` with phases `MovingToSDS` and `Transferring { frames_remaining }`.
   - `AttachingToTowerBehavior { target_tower: Entity }` — simpler, just move-to-tower then attach on arrival.
   - Follow the pattern of `GatheringResourceBehavior` (line 225) and `DroppingOffResourcesBehavior` (line 259): `#[derive(Component)]` struct with phase enum and `::new()` constructor.

2. **`artifacts/developer/src/game/units/systems/behaviors.rs`** — Add new behavior system functions:
   - `supply_chopper_pickup_system`: Query choppers with `PickingUpSuppliesBehavior`. On `MovingToSDS` phase, compute distance to target SDS entity (get its `Transform`). When within `OBJECT_PROXIMITY_DISTANCE` (1.5 gu), transition to `Transferring` phase. On `Transferring` completion, transfer `SupplyDeliveryStation.current_supplies` → `SupplyChopperState.carried_supplies`, zero SDS. If chopper has `attached_tower`, auto-issue `UnitCommand::Move` back to tower position. Else set `UnitCommand::Idle`. Remove the behavior component on completion. Write `LocomotionChannel`/`OrientationChannel` like `gathering_resource_behavior_system` (line 609).
   - `supply_chopper_attach_system`: Query choppers with `AttachingToTowerBehavior`. Move toward target tower. On arrival (within `OBJECT_PROXIMITY_DISTANCE`), set `chopper_state.attached_tower = Some(tower_entity)` and `tower_state.attached_chopper = Some(chopper_entity)`. If chopper was previously attached to a different tower, clear old tower's `attached_chopper` first. Set `UnitCommand::Idle`, remove behavior component.
   - `supply_chopper_dropoff_system`: Query choppers that have `SupplyChopperState` with `attached_tower.is_some()` AND `carried_supplies > 0` AND `UnitCommand::Idle` AND are near their tower (within `OBJECT_PROXIMITY_DISTANCE`). Transfer `carried_supplies` to `GdoPlayerResources.supplies` (query `(&Player, &mut GdoPlayerResources)`, match `Owner`). Set `carried_supplies = 0`.
   - `supply_chopper_detach_system`: Query choppers with `SupplyChopperState` where `attached_tower.is_some()` AND `UnitCommand` is NOT `Idle`. When a chopper has a non-idle command AND has an attached tower, clear `chopper_state.attached_tower` and the tower's `tower_state.attached_chopper`. This should run early so other systems see the detached state.
   - `supply_chopper_repair_system`: Query choppers with `SupplyChopperState` where `attached_tower.is_some()` AND `UnitCommand::Idle` AND `ObjectInstance.hp < ObjectInstance.max_hp`. When at tower position (within proximity), increment `hp` by a small amount per tick (e.g., 1.0 HP/frame). Clamp to max_hp.

3. **`artifacts/developer/src/game/units/mod.rs`** — Register all new systems in the `UnitsPlugin::build()` method. Place them in the Phase 2 behavior systems tuple (after `rebuild_occupancy_map`, line 29):
   ```
   systems::behaviors::supply_chopper_pickup_system,
   systems::behaviors::supply_chopper_attach_system,
   systems::behaviors::supply_chopper_dropoff_system,
   systems::behaviors::supply_chopper_detach_system,
   systems::behaviors::supply_chopper_repair_system,
   ```

4. **`artifacts/developer/src/game/units/systems/core.rs`** — The right-click handler (line 349-392) already issues `UnitCommand::PickUpSupplies` and `UnitCommand::AttachToTower`, but does NOT insert the behavior marker components. Add behavior marker insertion alongside the command. Follow the pattern used for other commands in this file — when `UnitCommand::PickUpSupplies(target)` is issued, also insert `PickingUpSuppliesBehavior::new(target)`. Same for `AttachToTower` → `AttachingToTowerBehavior::new(target)`. The insertion points are:
   - Line 364: `issue_or_queue_command(... UnitCommand::PickUpSupplies(target_entity) ...)` — add `entity_cmds.insert(PickingUpSuppliesBehavior::new(target_entity));`
   - Line 384: `issue_or_queue_command(... UnitCommand::AttachToTower(target_entity) ...)` — add `entity_cmds.insert(AttachingToTowerBehavior::new(target_entity));`

### Existing Patterns to Follow

- **Behavior component lifecycle**: Insert on command issue → system processes each tick → remove on completion. See `GatheringResourceBehavior` (behaviors.rs:609-776) as the closest analog.
- **Phase-based state machine**: Use an enum for phases (e.g., `MovingToSDS`, `Transferring`). Pattern from `GatherPhase` (behavior.rs:204).
- **Action channels**: Write `LocomotionChannel::Moving(vec![target_pos])` for movement, `LocomotionChannel::Stationary` when arrived. Write `OrientationChannel::Turning(target_pos)` during movement, `OrientationChannel::Maintaining` when stationary. These are the newer movement pipeline — the chopper already spawns with both channel components (utils.rs:926-927).
- **Target entity lookup**: Use a separate `Query<&Transform, Without<BehaviorComponent>>` for targets, matching `moving_to_object_system` (behaviors.rs:158).
- **Cancellation on missing target**: If `targets.get(entity)` returns `Err`, set `UnitCommand::Idle`, `BaseBehaviorState::None`, stationary channels, and `remove::<BehaviorComponent>()`. See behaviors.rs:650-657.
- **Resource transfer**: For GDO resources, query `(&Player, &mut GdoPlayerResources)` and match the chopper's `Owner` to the player. Pattern from `faction.rs:163`.

### Key Types and Components

- `SupplyChopperState` (structures.rs:378): `{ carried_supplies: u32, attached_tower: Option<Entity> }`
- `SupplyTowerState` (structures.rs:314): `{ attached_chopper: Option<Entity>, ... }`
- `SupplyDeliveryStation` (world/types.rs:172): `{ current_supplies: u32, ... }`
- `GdoPlayerResources` (factions.rs:85): `{ space_crystals: i32, supplies: i32, ... }`
- `ObjectInstance` (objects.rs:55): `{ hp: Option<f32>, max_hp: Option<f32>, ... }`
- `SC_MAX_HP` (structures.rs:431): 150.0
- `Owner` component: use `owner.player_number()` to match with `Player`
- `BaseBehaviorState` (behavior.rs): path tracking state, set to `None` when behavior completes
- `LocomotionChannel` / `OrientationChannel` (behavior.rs): action channel components for movement pipeline

### Constants to Add

In `behaviors.rs` or `unit_data.rs`:
- `CHOPPER_PICKUP_DURATION: u32` — frames to transfer supplies from SDS (e.g., 30 frames = ~0.5 seconds at 60fps)
- `CHOPPER_REPAIR_RATE: f32` — HP restored per frame while landed (e.g., 1.0)
- `CHOPPER_ARRIVAL_THRESHOLD: f32` — can reuse `OBJECT_PROXIMITY_DISTANCE` (1.5 gu)

### System Ordering Considerations

- `supply_chopper_detach_system` should run before other chopper behavior systems so they see clean attachment state. Either add `.before()` ordering or place it first in the tuple.
- All chopper systems go in Phase 2 (after `rebuild_occupancy_map`) alongside other behavior systems (mod.rs:19-29).
- `supply_chopper_dropoff_system` and `supply_chopper_repair_system` don't need behavior marker components — they trigger on state conditions (idle + near tower + has supplies/damaged).

### Testing Approach

- Use `World::new()` for unit tests — spawn chopper entity with `SupplyChopperState`, `UnitCommand`, `Transform`, `LocomotionChannel`, `OrientationChannel`, `BaseBehaviorState`, `Velocity`, `ObjectInstance`, plus behavior marker. Spawn SDS/Tower entities with their components.
- Test supply pickup: verify `current_supplies` transfers from SDS to `carried_supplies`.
- Test attachment: verify both `chopper_state.attached_tower` and `tower_state.attached_chopper` are set.
- Test detachment: verify issuing any non-Idle command clears both sides of the attachment.
- Test repair: verify HP increments each tick while idle at tower, capped at `max_hp`.
- Test dropoff: verify `carried_supplies` → `GdoPlayerResources.supplies` and `carried_supplies = 0`.
- Follow test patterns from `behaviors.rs` (line 1340+) for setup structure.

## Dependencies

- **`supply-chopper-command-panel`** (sibling task): Implements AwaitingTarget command panel integration for PickUpSupplies/AttachToTower. The behaviors here fire when those commands are issued. Both can be developed in parallel since the right-click path in core.rs already issues the commands.
- **Existing movement channel systems**: The chopper spawns with `LocomotionChannel`/`OrientationChannel` (utils.rs:926-927) and `DragMovementParams` (utils.rs:892). The channel consumer systems must exist for actual movement to happen. If `action-channel-locomotion-orientation` task hasn't landed yet, the behavior systems will correctly write channels but movement won't visually occur until that consumer lands.
- **`command_state_sync_system`** (commands.rs:195): Already maps `PickUpSupplies`/`AttachToTower` to `CommandType::Default`. No changes needed.
