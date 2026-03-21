# agent_dropoff_target_click

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-agent-interface.md

## Task

Implement left-click entity handling in AwaitingTarget(DropOff) mode so that clicking an own Tunnel issues a DropOffResources command to selected Agents.

### Current State
In `core.rs`, the `right_click_move_command` system handles the Agent's DropOff flow partially:
- The AgentDropOff button correctly enters `AwaitingTarget(CommandType::DropOff)` (in command_panel.rs ~line 1419)
- Right-click on own Tunnel in Default mode correctly issues DropOffResources/Enter (lines 364-386)
- But left-click on a Tunnel entity while in `AwaitingTarget(DropOff)` mode falls through to the ground handler, which simply resets the interface state without issuing any command (lines 568-571)

### Required Change
In the entity-click section of `right_click_move_command` (around line 256), add handling for `CommandType::DropOff`:
- When `command_type == CommandType::DropOff` and the user left-clicks on a target entity
- Check if the target is an own Tunnel (same pattern as lines 365: `tunnel_opt.is_some() && target_owner.player_number() == Some(local_player.0)`)
- If yes: iterate selected agents, clear movement state, insert `UnitCommand::DropOffResources(target_entity)` and `AttackState::default()`
- Return interface state to `ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault)` (not Default, since agents are selected)
- If the target is not an own Tunnel, reset back to AgentDefault without issuing a command

### Files to Modify
- `src/game/units/systems/core.rs`: Add DropOff target-click handling in entity-click section of `right_click_move_command`

### Tests
Add tests verifying:
- Left-click on own Tunnel in DropOff mode issues DropOffResources command
- Left-click on non-Tunnel entity in DropOff mode resets to AgentDefault without issuing command
- Left-click on enemy Tunnel in DropOff mode resets without issuing command (must be own Tunnel)

## Technical Context

### File to Modify
- **`artifacts/developer/src/game/units/systems/core.rs`**: The `right_click_move_command` system (starts at line 179)

### Where to Insert the New Code
Insert a new block **after the attack handling block** (ends at line 284) and **before the right-click chopper-specific checks** (line 287). The ideal insertion point is around line 285-286, as a new conditional block:

```rust
// Left-click entity in DropOff mode: target must be own Tunnel
if command_type == CommandType::DropOff {
    if let Ok((_sds_opt, _st_opt, target_owner, _crystal_opt, tunnel_opt)) = target_info.get(target_entity) {
        if tunnel_opt.is_some() && target_owner.player_number() == Some(local_player.0) {
            for (entity, _, _, _, attack_state_opt, _, obj, _) in &selected_units {
                if obj.object_type == ObjectEnum::SyndicateAgent {
                    if let Some(attack_state) = attack_state_opt {
                        if !attack_state.phase.is_interruptible() {
                            continue;
                        }
                    }
                    let mut entity_cmds = commands_ecs.entity(entity);
                    clear_movement_state_full(&mut entity_cmds);
                    entity_cmds.insert((UnitCommand::DropOffResources(target_entity), AttackState::default()));
                }
            }
            info!("Agent: DropOff command via target click");
            *interface_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
            return;
        }
    }
    // Target is not an own Tunnel — reset without issuing command
    *interface_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
    return;
}
```

### Pattern to Follow
The code pattern mirrors the existing right-click Agent tunnel interaction at lines 364-386:
- Same `target_info.get(target_entity)` query with destructured `tunnel_opt` and `target_owner`
- Same ownership check: `target_owner.player_number() == Some(local_player.0)`
- Same unit iteration filtering by `ObjectEnum::SyndicateAgent`
- Same interruptibility check on `AttackState`
- Same `clear_movement_state_full` + `insert` pattern

**Key difference from the right-click handler**: The DropOff left-click does NOT need to check `AgentCarryState::is_carrying()` — the DropOff button is only shown when agents are carrying (see command_panel.rs ~line 1543), so by the time the user clicks a target, we can assume the intent is to drop off.

### Import Required
**`AgentMenuState`** is NOT currently imported in `core.rs`. Add it to the existing import on line 10:
```rust
// Line 10 currently:
use crate::ui::types::{CursorTarget, CursorTargetEnum, CursorOverUi, ObjectInterfaceState, CommandPanelTarget, StructureMenuState};
// Change to:
use crate::ui::types::{CursorTarget, CursorTargetEnum, CursorOverUi, ObjectInterfaceState, CommandPanelTarget, StructureMenuState, AgentMenuState};
```

### Ground-Click Section Update
Also update the ground-click fallthrough at line 568-571 to return to `AgentMenu(AgentDefault)` instead of `Default` for `DropOff`:
```rust
CommandType::Gather | CommandType::DropOff => {
    // Gather and DropOff require clicking a target entity, not ground.
    // Ground click resets mode.
    *interface_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
}
```
Note: `Gather` may also benefit from returning to AgentDefault, but the task scope only requires DropOff. If you split the match arm, only change DropOff. Alternatively, since both Gather and DropOff are agent-only commands, changing both is safe.

### Key Types
- **`CommandType::DropOff`** — defined in `src/game/units/types/state/commands.rs` (line ~107)
- **`UnitCommand::DropOffResources(Entity)`** — the command to insert on units
- **`ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault)`** — defined in `src/ui/types.rs` (line ~231)
- **`AttackState`** — from `src/game/combat/types.rs`, `phase.is_interruptible()` checks if the unit can be interrupted
- **`clear_movement_state_full`** — local helper in core.rs that removes movement components
- **`target_info` query** — `Query<(Option<&SupplyDeliveryStation>, Option<&SupplyTowerState>, &Owner, Option<&SpaceCrystalPatch>, Option<&TunnelState>), With<ObjectInstance>>` (line 186)

### Existing Test Module
Tests live in `mod tests` at line 1375 of `core.rs`. Current tests use `World::new()` + `RunSystemOnce` pattern. However, testing `right_click_move_command` is complex due to its many query parameters. Consider writing focused integration-style tests that:
1. Set up a `World` with required resources (`ObjectInterfaceState`, `CursorTarget`, `CursorOverUi`, `ButtonInput<MouseButton>`, `LocalPlayer`, `GridMap`, `OccupancyMap`)
2. Spawn a selected agent entity and a tunnel entity with `TunnelState` + `Owner`
3. Configure `CursorTarget` to point at the tunnel entity
4. Set `ObjectInterfaceState::AwaitingTarget(CommandType::DropOff)`
5. Simulate a left mouse click via `ButtonInput<MouseButton>`
6. Run `right_click_move_command` via `RunSystemOnce`
7. Assert `UnitCommand::DropOffResources` was inserted and interface state is `AgentMenu(AgentDefault)`

### Escape Handler (Already Correct)
The escape handler in `command_panel.rs` (line 897-905) already correctly returns `AwaitingTarget(_)` to `AgentMenu(AgentDefault)` when agents are selected, so no changes needed there.

## Dependencies

None — this is a standalone fix within the existing `right_click_move_command` system. All required types and patterns already exist in the codebase.
