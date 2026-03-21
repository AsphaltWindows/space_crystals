# agent_panel_layout

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-agent-interface.md

## Task

Update the Agent DefaultState command panel layout to match the design spec: show only 2 buttons instead of the current 7. Also fix the AwaitingTarget Escape handler to return to AgentDefault when agents are selected.

## Technical Context

### File to Modify
- `artifacts/developer/src/ui/command_panel.rs`

### Change 1: Update AgentDefault grid layout (lines 154-163)

Current code at line 153-163 maps 7 buttons for `AgentMenuState::AgentDefault`:
```rust
AgentMenuState::AgentDefault => match (row, col) {
    (0, 0) => Some(CommandButtonAction::UnitMove),
    (0, 1) if caps.has_attack => Some(CommandButtonAction::UnitAttack),
    (0, 2) => Some(CommandButtonAction::UnitEnter),
    (1, 0) => Some(CommandButtonAction::UnitGather),
    (1, 1) => Some(CommandButtonAction::AgentDropOff),
    (1, 2) => Some(CommandButtonAction::AgentBuildTunnel),
    (2, 0) => Some(CommandButtonAction::UnitStop),
    _ => None,
},
```

Replace with only 2 buttons per the design spec (`artifacts/designer/design/syndicate_objects.md` lines 266-268):
```rust
AgentMenuState::AgentDefault => match (row, col) {
    (0, 0) => Some(CommandButtonAction::AgentBuildTunnel),
    (0, 1) => Some(CommandButtonAction::AgentDropOff),
    _ => None,
},
```

Design spec confirms:
- **A (slot 0,0): Build Tunnel** — enters AwaitingPlacement
- **B (slot 0,1): Drop Off Resources** — targeted command

All other commands (Move, Stop, Attack, Enter, Gather) are right-click-only per the design spec's "Unit Commands" section and should NOT appear in the panel.

### Change 2: Fix AwaitingTarget Escape handler (lines 902-907)

Current code at lines 902-907 always returns to `ObjectInterfaceState::Default`:
```rust
ObjectInterfaceState::AwaitingTarget(_) => {
    let all_agents = selected_units.iter().count() > 0;
    let _ = all_agents;
    *interface_state = ObjectInterfaceState::Default;
    info!("Cancelled awaiting target, back to Default");
}
```

Fix by checking whether the active selection group is agents. The `selection` resource (`ResMut<Selection>` at line 849) is already available in the `command_panel_hotkeys` system. Use the same pattern from `update_command_panel_state` (line 408):
```rust
ObjectInterfaceState::AwaitingTarget(_) => {
    let active_is_agent = selection.active_type() == Some(ObjectEnum::SyndicateAgent);
    if active_is_agent {
        *interface_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
        info!("Cancelled awaiting target, back to AgentDefault");
    } else {
        *interface_state = ObjectInterfaceState::Default;
        info!("Cancelled awaiting target, back to Default");
    }
}
```

Key: `selection.active_type()` returns `Option<ObjectEnum>` — the type of the active selection group. This is exactly the same check used at line 408 in `update_command_panel_state`.

### Change 3: Update tests (lines 3041-3116)

Tests to **delete** (they test buttons that are being removed):
- `agent_default_shows_move_at_q` (line 3044) — Move removed from panel
- `agent_default_shows_attack_at_w_when_has_attack` (line 3052) — Attack removed
- `agent_default_hides_attack_at_w_when_no_attack` (line 3060) — Attack removed
- `agent_default_shows_enter_at_e` (line 3068) — Enter removed
- `agent_default_shows_gather_at_a` (line 3076) — Gather removed
- `agent_default_shows_stop_at_z` (line 3100) — Stop removed

Tests to **update**:
- `agent_default_shows_drop_off_at_s` (line 3084) — rename to `agent_default_shows_drop_off_at_w` and change slot from (1,1) to (0,1)
- `agent_default_shows_build_tunnel_at_d` (line 3092) — rename to `agent_default_shows_build_tunnel_at_q` and change slot from (1,2) to (0,0)
- `agent_default_no_extra_slots` (line 3108) — update to check ALL slots except (0,0) and (0,1) return None

Tests to **add**:
- New test for AwaitingTarget escape returning to AgentDefault when active group is SyndicateAgent (if testable — this requires ECS world setup similar to other system tests in the file)

### Key Types and Imports
- `ObjectInterfaceState` — defined in `src/ui/types.rs`, the state machine resource
- `AgentMenuState` — enum with `AgentDefault` and `AgentAwaitingPlacement` variants (in `src/ui/types.rs`)
- `CommandButtonAction` — enum for all button actions (in `src/ui/command_panel.rs` or `types.rs`)
- `ObjectEnum::SyndicateAgent` — used for agent detection
- `Selection::active_type()` — method on Selection resource returning `Option<ObjectEnum>`
- `SelectedUnitCapabilities` — no longer needs `has_attack` check in agent grid since Attack button is removed

## Dependencies

None — this is a self-contained UI layout change in a single file. The `AgentBuildTunnel` and `AgentDropOff` command button actions already exist and are already handled by the command execution flow.
