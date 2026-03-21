# command-panel-rightclick-cancel

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-command-panel-framework.md

## Task

Add right-click cancel (back to previous state) for all multi-stage ObjectInterfaceState states that currently only support Escape cancel. The command panel framework is already fully implemented (3x3 grid, hotkeys, Z=Back, X=Cancel, C=Rally, CommonCommands vs GroupCommands visual distinction, Tab group cycling, Escape cancel). The only gap is QA step 6: right-click from a sub-menu should behave the same as Z/Escape.

**What needs to be added:**

A system (or addition to an existing system) that handles right-click when the ObjectInterfaceState is in a non-Default, non-placement multi-stage state, resetting it to the previous state — same transitions as Escape in `command_panel_hotkeys` (lines ~867-907 of command_panel.rs).

Specifically, right-click should cancel from these states (matching Escape behavior):
- `StructureMenu(DcBuildMenu)` → `StructureMenu(DcIdle)`
- `StructureMenu(DcConstructing)` → `StructureMenu(DcIdle)`
- `StructureMenu(DcReadyToPlace)` → `StructureMenu(DcIdle)`
- `StructureMenu(TunnelExpandMenu)` → `StructureMenu(TunnelIdle)`
- `StructureMenu(TunnelEjectMenu)` → `StructureMenu(TunnelIdle)`
- `StructureMenu(EfConstructing)` → `StructureMenu(EfIdle)`
- `StructureMenu(EfReadyToPlace)` → `StructureMenu(EfIdle)`
- `AwaitingTarget(SetRallyPoint)` → appropriate previous state (BarracksMenu/HeadquartersMenu/SupplyTowerMenu)
- `AwaitingTarget(ScheduleDeliveries)` → `StructureMenu(SupplyTowerMenu)`

**Already handled (do NOT duplicate):**
- Placement modes (`DcAwaitingPlacement`, `EfAwaitingPlacement`, `TunnelAwaitingPlacement`, `AgentAwaitingPlacement`) — right-click cancel is already in `placement_click_system` in faction.rs line 1280
- Regular unit AwaitingTarget modes (Move, Attack, Patrol, etc.) — right-click in `right_click_move_command` issues a command which resets state

## Technical Context

### Files to modify

1. **`artifacts/developer/src/ui/command_panel.rs`** — Best location for the new system. Add a new public function `right_click_cancel_submenu` next to `command_panel_hotkeys` (after line ~925). This keeps all command panel input handling co-located.

2. **`artifacts/developer/src/ui/mod.rs`** (line 41) — Register the new system in the `DiagCategory::UiHud` system set, alongside `command_panel_hotkeys`.

### Implementation pattern

Follow the Escape handler pattern in `command_panel_hotkeys` (lines 867-909). The new system should:

```rust
pub fn right_click_cancel_submenu(
    buttons: Res<ButtonInput<MouseButton>>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    selection: Res<Selection>,
    panel_target: Res<CommandPanelTarget>,
    // Need to determine which production structure is selected for SetRallyPoint return
    bk_query: Query<(), With<BarracksState>>,
    hq_query: Query<(), With<HeadquartersState>>,
    st_query: Query<(), With<SupplyTowerState>>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }
    
    match &*interface_state {
        // Structure sub-menus → parent idle state (copy from Escape handler lines 869-888)
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu) |
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing) |
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace) => {
            *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu) |
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu) => {
            *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::EfConstructing) |
        ObjectInterfaceState::StructureMenu(StructureMenuState::EfReadyToPlace) => {
            *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        }
        // AwaitingTarget(SetRallyPoint) → determine which structure menu to return to
        ObjectInterfaceState::AwaitingTarget(CommandType::SetRallyPoint) => {
            // SetRallyPoint can be entered from BarracksMenu, HeadquartersMenu, or SupplyTowerMenu
            // Use panel_target entity to determine which one
            if let Some(entity) = panel_target.entity {
                if bk_query.get(entity).is_ok() {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);
                } else if hq_query.get(entity).is_ok() {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
                } else if st_query.get(entity).is_ok() {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu);
                } else {
                    *interface_state = ObjectInterfaceState::Default;
                }
            } else {
                *interface_state = ObjectInterfaceState::Default;
            }
        }
        // AwaitingTarget(ScheduleDeliveries) → back to SupplyTowerMenu
        ObjectInterfaceState::AwaitingTarget(CommandType::ScheduleDeliveries) => {
            *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu);
        }
        _ => {} // Don't handle Default, placement modes, or unit AwaitingTarget
    }
}
```

### Key types and imports needed
- `ObjectInterfaceState` and `StructureMenuState` from `crate::ui::types`
- `CommandType` from `crate::game::units::types::state::commands`
- `CommandPanelTarget` from `crate::ui::command_panel`
- `Selection` from `crate::shared::types`
- `BarracksState`, `HeadquartersState`, `SupplyTowerState` from `crate::game::world::faction`

### Conflict avoidance
- **`placement_click_system`** (faction.rs:1280): Guards with `panel_state.is_placement_mode()` — only runs for `*AwaitingPlacement` states. No conflict since the new system explicitly does NOT match placement states.
- **`production_rally_point_system`** (faction.rs:550): Only fires when state is `BarracksMenu | HeadquartersMenu | SupplyTowerMenu` (the idle production states). The new system does NOT match these states — it matches `AwaitingTarget(SetRallyPoint)` instead. No conflict.
- **`right_click_move_command`** (core.rs:179): Returns early when `is_placement_mode()` is true (line 236), and returns early when command_type is `SetRallyPoint` (line 243) or `ScheduleDeliveries` (line 248). For the structure sub-menus (DcBuildMenu, TunnelExpandMenu, etc.), `right_click_move_command` checks `is_awaiting_target()` which returns false, so it falls through to regular right-click which only processes if there are selected units and a valid cursor target — structure menus don't have selected units. No conflict.

### System ordering
No strict ordering requirements. The new system reads `MouseButton::Right` and writes `ObjectInterfaceState`. It should NOT run after `placement_click_system` or `production_rally_point_system` since those guard on different states. Register it in the same tuple as `command_panel_hotkeys` in `ui/mod.rs` line 41.

### Tests
Add tests in the same file verifying:
1. Right-click from each sub-menu state transitions to the correct parent state
2. Right-click from AwaitingTarget(SetRallyPoint) returns to correct structure menu based on panel_target entity type
3. Right-click from AwaitingTarget(ScheduleDeliveries) returns to SupplyTowerMenu
4. Right-click from Default/placement states does NOT change state

Use `World::new()` + `init_resource` pattern similar to existing command_panel tests (see line ~3400+ of command_panel.rs for examples).

## Dependencies

None — this task is standalone. The command panel framework (grid layout, hotkeys, Escape cancel) is already fully implemented. This task only adds right-click as an additional cancel trigger using the same state transitions.
