# supply-chopper-interface

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-supply-chopper.md

## Task

Add the SupplyChopper command panel interface and AwaitingTarget modes.

### What already exists
- ObjectEnum::SupplyChopper with all stats (60x60, HP=150, armor 1/1, sight=5, groupable, unarmed)
- SupplyChopperState component (carried_supplies, attached_tower) in game/types/structures.rs
- spawn_supply_chopper() in game/utils.rs with DragMovementParams
- UnitCommand::PickUpSupplies(Entity) and UnitCommand::AttachToTower(Entity) in units/types/state/commands.rs
- Right-click resolution in core.rs (~line 349): SDS→PickUpSupplies, own ST→AttachToTower
- command_to_state mapping maps both to CommandType::Default (commands.rs ~line 215)
- object_type_supports_action already excludes attack actions for SupplyChopper (command_panel.rs ~line 2176)

### What needs to be implemented

1. **CommandType variants**: Add `CommandType::PickUpSupplies` and `CommandType::AttachToTower` to the CommandType enum in units/types/state/commands.rs.

2. **Command-to-state mapping**: Update commands.rs line 215-216 so UnitCommand::PickUpSupplies maps to CommandType::PickUpSupplies (not Default), and UnitCommand::AttachToTower maps to CommandType::AttachToTower (not Default).

3. **SupplyChopper command panel grid**: When a SupplyChopper is the active selection group, show the following 3x3 grid in command_panel.rs:
   - (0,0) Q = Move → enters AwaitingTarget[Move]
   - (0,1) W = Pick Up Supplies → enters AwaitingTarget[PickUpSupplies]
   - (0,2) E = Attach to Tower → enters AwaitingTarget[AttachToTower]
   - (1,0) A = Stop → issues Stop immediately
   - (1,1) S = HoldPosition → issues HoldPosition immediately
   - No attack commands (unarmed unit)
   Add CommandButtonAction variants (e.g., UnitPickUpSupplies, UnitAttachToTower) and wire them into execute_command_action, button_label, button_availability, and object_type_supports_action.

4. **AwaitingTarget resolution**: In right_click_move_command (core.rs), add left-click entity handling for CommandType::PickUpSupplies (left-click SDS → issue PickUpSupplies command, reset state) and CommandType::AttachToTower (left-click own SupplyTower → issue AttachToTower command, reset state). Follow the same pattern as the existing Enter/Gather/DropOff AwaitingTarget handlers (~lines 274-346).

5. **Tests**: Verify SupplyChopper grid layout, button availability (no attack buttons), AwaitingTarget mode transitions, and object_type_supports_action for the new action variants.

## Technical Context

### Files to modify

**1. `artifacts/developer/src/game/units/types/state/commands.rs`** — CommandType enum & mapping
- **CommandType enum** (line 76): Add `PickUpSupplies` and `AttachToTower` variants after `ScheduleDeliveries` (line 93).
- **CommandType::name()** (line 97): Add match arms returning `"Pick Up Supplies"` and `"Attach to Tower"`.
- **CommandType::hotkey()** (line 118): Add match arms — use `"W"` for PickUpSupplies and `"E"` for AttachToTower (matching grid positions (0,1) and (0,2)).
- Existing tests enumerate all CommandType variants (lines 350-352, 510-514) — update these lists.

**2. `artifacts/developer/src/game/units/systems/commands.rs`** — command_to_state mapping
- **command_state_sync_system** (line ~215-216): Change `UnitCommand::PickUpSupplies(entity) => (CommandType::Default, ...)` to `(CommandType::PickUpSupplies, None, Some(*entity))`. Same for `AttachToTower` → `CommandType::AttachToTower`.

**3. `artifacts/developer/src/ui/types.rs`** — CommandButtonAction enum
- **CommandButtonAction enum** (line 240): Add two new variants after `UnitGather` (line 306):
  ```rust
  /// SupplyChopper: Pick Up Supplies (enters AwaitingTarget)
  ChopperPickUpSupplies,
  /// SupplyChopper: Attach to Tower (enters AwaitingTarget)
  ChopperAttachToTower,
  ```
- Follow naming pattern: `Chopper` prefix for chopper-specific actions, like `Agent` prefix for agent-specific.

**4. `artifacts/developer/src/ui/command_panel.rs`** — Grid layout, labels, availability, execute

- **`get_grid_slot_action()`** (line 49): The SupplyChopper currently uses the `ObjectInterfaceState::Default` grid (line 141-151) which shows combat commands. Two options:
  - **Option A (recommended)**: Add a new check for SupplyChopper before the Default grid. Detect via `caps` or a new flag. Since the Default grid uses `caps.has_attack` etc., and choppers have `has_attack=false`, most attack buttons are already gated. But the chopper needs its own W and E slots (PickUpSupplies, AttachToTower) which don't exist in the default grid. Therefore, the cleanest approach is to **check `active_type` or add a `is_chopper` flag** and insert a dedicated match block before the Default fallthrough.
  - The function signature currently takes `caps: &SelectedUnitCapabilities` — it does NOT receive `active_type`. You'll need to either: (a) pass `active_type: Option<ObjectEnum>` as a new parameter, or (b) add an `is_chopper: bool` field to `SelectedUnitCapabilities`.
  - Pattern to follow: look at how `ObjectInterfaceState::AgentMenu` (line 152-162) adds agent-specific commands in a dedicated block. For choppers, add similar handling within or alongside the `Default` state.

- **Chopper grid layout** in `get_grid_slot_action()`:
  ```
  (0,0) Q = UnitMove         (0,1) W = ChopperPickUpSupplies  (0,2) E = ChopperAttachToTower
  (1,0) A = UnitStop          (1,1) S = UnitHoldPosition        (1,2) _
  (2,0) _                     (2,1) _                           (2,2) _
  ```
  Note: Stop at (1,0)/A and HoldPosition at (1,1)/S differ from the Default grid which has Stop at (2,1)/X and Hold at (0,2)/E. Follow the task spec's layout.

- **`grid_button_label()`** (line 2262): Add match arms:
  ```rust
  CommandButtonAction::ChopperPickUpSupplies => format!("[{}] Pick Up\nSupplies", hotkey),
  CommandButtonAction::ChopperAttachToTower => format!("[{}] Attach\nTower", hotkey),
  ```

- **`execute_command_action()`** (line 1108): Add match arms for the two new actions. Both are target commands (enter AwaitingTarget mode):
  ```rust
  CommandButtonAction::ChopperPickUpSupplies => {
      **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::PickUpSupplies);
      info!("Command mode: Pick Up Supplies");
  }
  CommandButtonAction::ChopperAttachToTower => {
      **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::AttachToTower);
      info!("Command mode: Attach to Tower");
  }
  ```
  Follow pattern of `UnitEnter` (line 1461-1463) and `UnitGather` (line 1465-1467).

- **`is_unit_action()`** (line 2159): Add `ChopperPickUpSupplies` and `ChopperAttachToTower` to the match list.

- **`object_type_supports_action()`** (line 2178): Add match arms:
  ```rust
  CommandButtonAction::ChopperPickUpSupplies |
  CommandButtonAction::ChopperAttachToTower => matches!(obj, ObjectEnum::SupplyChopper),
  ```

- **`is_action_active()`** (line 2358): Add:
  ```rust
  (CommandButtonAction::ChopperPickUpSupplies, CommandType::PickUpSupplies) => true,
  (CommandButtonAction::ChopperAttachToTower, CommandType::AttachToTower) => true,
  ```

- **`grid_button_enabled()`** (line 2326): Both new actions default to `true` via the `_ => true` fallthrough — no changes needed unless availability gating is required now (the sibling task `supply-chopper-command-panel` mentions availability based on carrying units, but this task's spec doesn't gate them).

- **`update_command_panel_state()`** (line 280): Currently, when active group is SupplyChopper, it falls through to the `else` branch (line 410-416) which sets `ObjectInterfaceState::Default`. This is correct — the chopper-specific grid should be rendered within the `Default` state by detecting the active type in `get_grid_slot_action()`.

**5. `artifacts/developer/src/game/units/systems/core.rs`** — AwaitingTarget resolution
- **`right_click_move_command()`** (line 186): Add two new AwaitingTarget handlers in the entity-click section (after the Enter handler at line 346, before the chopper right-click block at line 348):

  ```rust
  // Left-click entity in PickUpSupplies mode: target must be SDS
  if command_type == CommandType::PickUpSupplies {
      if let Ok((sds_opt, _, _, _, _)) = target_info.get(target_entity) {
          if sds_opt.is_some() {
              for (entity, _, _, _, attack_state_opt, chopper_opt, _, _, mut command_queue) in &mut selected_units {
                  if chopper_opt.is_some() {
                      // ... issue PickUpSupplies(target_entity)
                  }
              }
              *interface_state = ObjectInterfaceState::Default;
              return;
          }
      }
      *interface_state = ObjectInterfaceState::Default;
      return;
  }

  // Left-click entity in AttachToTower mode: target must be own SupplyTower
  if command_type == CommandType::AttachToTower {
      if let Ok((_, st_opt, target_owner, _, _)) = target_info.get(target_entity) {
          if st_opt.is_some() && target_owner.player_number() == Some(local_player.0) {
              for (entity, _, _, _, attack_state_opt, chopper_opt, _, _, mut command_queue) in &mut selected_units {
                  if chopper_opt.is_some() {
                      // ... issue AttachToTower(target_entity)
                  }
              }
              *interface_state = ObjectInterfaceState::Default;
              return;
          }
      }
      *interface_state = ObjectInterfaceState::Default;
      return;
  }
  ```
  Follow the exact pattern of the DropOff handler (lines 291-318) and Enter handler (lines 320-346):
  - Check attack_state interruptibility
  - Use `clear_movement_state_full` when not shift_held
  - Use `issue_or_queue_command` for the actual command dispatch
  - Reset `interface_state` to `Default` after issuing (not `AgentMenu` — choppers use `Default`)

- **Important**: `target_info` query tuple (line ~192) already includes `Option<&SupplyDeliveryStation>` and `Option<&SupplyTowerState>`, so no query changes needed.

### Key patterns to follow

1. **Target command flow**: Button click → `execute_command_action` sets `AwaitingTarget(CommandType::X)` → next left-click entity in `right_click_move_command` resolves target → issues `UnitCommand::X(entity)` → resets `interface_state` to `Default`.

2. **Immediate command flow**: Button click → `execute_command_action` directly inserts `UnitCommand::X` on matching entities using `command_target_entities()`.

3. **Chopper filtering**: In core.rs, the chopper right-click handler (line 349) filters selected units via `chopper_opt.is_some()` — use the same pattern for AwaitingTarget resolution.

4. **AwaitingTarget cancel**: The existing `Back` button in AwaitingTarget state (line 163-164) returns to `Default`. The Escape handler in `command_panel_hotkeys` also handles this. No additional cancel logic needed.

### Relevant types and imports
- `CommandType` — `crate::game::units::types::state::commands::CommandType`
- `CommandButtonAction` — `crate::ui::types::CommandButtonAction`
- `ObjectInterfaceState` — `crate::ui::types::ObjectInterfaceState`
- `SupplyChopperState` — `crate::game::types::SupplyChopperState` (already imported in core.rs line 13)
- `SupplyDeliveryStation` — `crate::game::world::types::SupplyDeliveryStation`
- `SupplyTowerState` — `crate::game::types::SupplyTowerState`

### System ordering
- No new systems needed — all changes are within existing systems.
- `update_command_panel_state` runs before `rebuild_command_panel_ui` (both in `DiagCategory::UiHud`).
- `right_click_move_command` runs in `DiagCategory::Commands`.

## Dependencies

- **None** — This task is standalone. It only adds interface wiring (CommandType variants, grid layout, AwaitingTarget handlers) on top of already-existing UnitCommand variants, SupplyChopperState, and right-click resolution code. The behavior systems (supply-chopper-behaviors) depend on this task's CommandType variants but not vice versa.
