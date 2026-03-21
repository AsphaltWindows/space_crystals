# supply-chopper-dropoff-command

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-supply_chopper_commands.md

## Task

Add the `UnitCommand::DropOffSupplies(Entity)` variant and update the Supply Chopper right-click resolution to be state-dependent with carrying-units gating.

### Changes needed:

**1. Add DropOffSupplies command variant** (`game/units/types/state/commands.rs`):
- Add `DropOffSupplies(Entity)` to `UnitCommand` enum
- Mark it as requiring target (`is_entity_targeted() => true`)
- Add tests matching existing PickUpSupplies/AttachToTower pattern

**2. Update right-click resolution** (`game/units/systems/core.rs`, ~line 350-391):
The current SupplyChopper right-click handler issues PickUpSupplies for SDS and AttachToTower for own SupplyTower unconditionally. Update:

- **Right-click SDS**: Only issue PickUpSupplies if chopper is NOT carrying units. Check `chopper_state.carried_units` or equivalent field. (Currently no carrying-units check.)
- **Right-click own SupplyTower**: Make state-dependent:
  - If chopper `carried_supplies > 0`: issue `DropOffSupplies(tower_entity)` instead of AttachToTower
  - If chopper `carried_supplies == 0` AND not carrying units: issue `AttachToTower(tower_entity)`
  - If carrying units: do nothing (fall through to default move)

**3. Break attachment on player commands** (`game/units/systems/commands.rs` or core.rs):
- When any player-issued command is given to a SupplyChopper that has `attached_tower.is_some()`, clear the attachment (set `attached_tower = None`, also clear the tower's `attached_chopper` if it references this chopper)
- Automated scheduled delivery departures must NOT break attachment (only player-issued commands do)

### Reference:
- Design: `artifacts/designer/design/gdo_objects.md` lines 329-368
- SupplyChopperState fields: `carried_supplies`, `attached_tower`
- Existing right-click handler: `core.rs` ~line 350-391

## Technical Context

### Files to modify

1. **`artifacts/developer/src/game/units/types/state/commands.rs`** (UnitCommand enum, CommandType enum, is_available, tests)
   - Add `DropOffSupplies(Entity)` variant to `UnitCommand` enum (line ~31, after `DropOffResources`)
   - Add `is_available` match arm — should return `true` unconditionally (same as PickUpSupplies/AttachToTower, lines 57-59)
   - Add `CommandType::PickUpSupplies`, `CommandType::AttachToTower`, `CommandType::DropOffSupplies` variants to the `CommandType` enum (line ~93, after `ScheduleDeliveries`)
   - Add `name()` and `hotkey()` implementations for all 3 new CommandType variants
   - Add unit tests following the PickUpSupplies/AttachToTower pattern (lines 434-460)

2. **`artifacts/developer/src/game/units/systems/commands.rs`** (command_state_sync_system)
   - Line 214-216: Update mapping for PickUpSupplies and AttachToTower from `CommandType::Default` to their new dedicated `CommandType` variants
   - Add new mapping: `UnitCommand::DropOffSupplies(entity) => (CommandType::DropOffSupplies, None, Some(*entity))`

3. **`artifacts/developer/src/game/units/systems/core.rs`** (right_click_move_command)
   - **Right-click SDS handler (lines 351-370)**: Currently issues `PickUpSupplies` unconditionally for all choppers. Add a guard: only issue if the chopper does NOT have a `carried_units` field set (note: `carried_units` does NOT exist on `SupplyChopperState` yet — this will need to be added to `SupplyChopperState` OR checked via a separate marker component). For now, if no `carried_units` field is added yet, skip this gating and leave a TODO comment.
   - **Right-click own SupplyTower handler (lines 371-390)**: Currently issues `AttachToTower` unconditionally. Make state-dependent by reading `SupplyChopperState` from the chopper entity. The `chopper_opt` in the query is `Option<&SupplyChopperState>` — when Some, access `.carried_supplies`:
     - If `chopper_state.carried_supplies > 0`: issue `UnitCommand::DropOffSupplies(target_entity)`
     - Else: issue `UnitCommand::AttachToTower(target_entity)` (existing behavior)
   - Import note: `SupplyChopperState` is already imported on line 13

4. **`artifacts/developer/src/game/types/structures.rs`** (SupplyChopperState)
   - Consider whether to add a `carried_units: Vec<Entity>` or similar field. The task mentions "carrying units" gating but the field doesn't exist yet. If the feature isn't ready, skip this and use a TODO comment.

### Existing patterns to follow

- **UnitCommand variant pattern**: See `PickUpSupplies(Entity)` and `AttachToTower(Entity)` at lines 21-23 of commands.rs — `DropOffSupplies(Entity)` follows the same pattern
- **CommandType variant pattern**: See existing variants like `Enter`, `Gather`, `DropOff` at lines 87-90 — add new variants in the same style with `name()` and `hotkey()` implementations
- **Right-click handler pattern**: The chopper handler block (core.rs:349-392) iterates selected units, checks `chopper_opt.is_some()`, checks attack interruptibility, clears movement state, and issues command via `issue_or_queue_command`
- **issue_or_queue_command**: Imported from `game/units/utils` (core.rs:12). Takes `(EntityCommands, CommandQueue, UnitCommand, shift_held)`. When shift_held, appends to queue; otherwise replaces current command.
- **Test pattern**: See lines 437-460 for PickUpSupplies/AttachToTower tests — variant creation, matches!, is_available checks

### Key types and queries

- **`SupplyChopperState`** (`game/types/structures.rs:378`): `{ carried_supplies: u32, attached_tower: Option<Entity> }`
- **`SupplyTowerState`** (`game/types/structures.rs:314`): `{ attached_chopper: Option<Entity>, landed_chopper: Option<Entity>, scheduled_sds: Option<Entity>, build_queue: Vec<ObjectEnum>, current_build: Option<ObjectEnum> }`
- **selected_units query** (core.rs:186): `Query<(Entity, &Transform, &UnitBaseEnum, &Owner, Option<&AttackState>, Option<&SupplyChopperState>, &ObjectInstance, Option<&AgentCarryState>, &mut CommandQueue), (With<Unit>, With<Selected>)>`
- **target_info query** (core.rs:189): includes `Option<&SupplyDeliveryStation>`, `Option<&SupplyTowerState>`, `&Owner`, `Option<&SpaceCrystalPatch>`, `Option<&TunnelState>`

### Attachment breaking system

For breaking attachment on player commands, two approaches:
1. **In right_click_move_command** (core.rs): Before issuing any command to a chopper with `attached_tower.is_some()`, clear both sides. This handles all right-click commands but NOT hotkey commands.
2. **Dedicated system** (recommended): Add a small system in `commands.rs` or a new file that runs each frame. Query choppers with `SupplyChopperState` where `attached_tower.is_some()` and `UnitCommand` is not Idle. When a non-idle command is detected AND it wasn't set by the auto-delivery system, break the link. This needs a way to distinguish player commands from automated ones — e.g., a `PlayerIssuedCommand` marker component that right-click and hotkey handlers insert alongside `UnitCommand`.

The simpler v1 approach: break attachment inline in the right-click handler (core.rs) when issuing any non-AttachToTower command to choppers. This covers the primary case. Hotkey commands (Stop, HoldPosition) can be handled in `hold_position_system` (commands.rs:65) and `stop_command_system` (commands.rs:99) which already have chopper-accessible queries.

### System registration

- No new systems needed for the DropOffSupplies command variant itself — it's just data
- The command_state_sync_system already exists and just needs updated match arms
- If adding an attachment-breaking system, register it in `CommandsPlugin` (`game/units/systems/mod.rs:57`) under `DiagCategory::Commands`

## Dependencies

- **supply-chopper-command-panel** (sibling task): That task implements the command panel grid, button actions, and AwaitingTarget resolution for PickUpSupplies/AttachToTower/DropOffSupplies. This task adds the `DropOffSupplies` UnitCommand variant and CommandType variants that the command-panel task will reference. Either task can be done first — the command-panel task will consume the CommandType variants added here.
- **supply-chopper-behaviors** (sibling task from different parent feature): Implements the actual behavior systems that execute PickUpSupplies/AttachToTower commands. This task only adds the command variant and right-click resolution — behaviors are separate.
- **No blocking dependencies**: This task can be implemented standalone. The `carried_units` gating mentioned in the spec requires a field that doesn't exist yet — use TODO comments for that check.
