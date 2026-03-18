# Task: Agent ObjectInterfaceState

## Current State
The Agent unit has no ObjectInterfaceState defined. The command panel shows no Agent-specific commands when an Agent is selected. The Agent does not use BasicCombatUnitInterfaceState because its right-click resolution is more complex (resource-context-sensitive Tunnel interaction).

## Desired State
Implement the Agent's ObjectInterfaceState as a unique interface (not BasicCombatUnitInterfaceState). Since Agent is Ungroupable, the SelectionGroup always contains exactly one Agent instance, so the panel displays that Agent's interface state.

### DefaultState Commands
- **A: Build Tunnel** -- enters AwaitingPlacement for a Tunnel (StateOnlyTransition). Ghost preview follows cursor, snapped to grid. Tinted green when valid, red when invalid. R rotates 90 degrees clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically. Left-click valid location confirms placement and dispatches the Agent to that location (CommandIssuingTransition, returns to DefaultState). Escape/right-click cancels back to DefaultState.
- **B: Drop Off Resources** -- targeted command (CommandIssuingTransition). Requires clicking an own Tunnel. Agent walks to the appropriate side automatically (Side B for crystals, Side C for supplies). Always visible, **greyed out when Agent is not carrying resources**.

### Unit Commands
Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel

### Right-Click Resolution
| Right-Click Target | Command Issued |
|---|---|
| Crystal field | Gather crystals |
| Supply source | Gather supplies |
| Own Tunnel (carrying resources) | Drop off resources (auto-routes to correct side) |
| Own Tunnel (not carrying resources) | Enter |
| Enemy unit/building | Attack (melee) |
| Ground | Move |

### Multi-Select Note
Despite being Ungroupable, right-click commands are issued to **all selected Agents** simultaneously (not just the ActiveGroup Agent).

## Technical Context

### ObjectInterfaceState Architecture
- **State enum**: `ObjectInterfaceState` at `src/ui/types.rs:145` — currently has `Default`, `AwaitingTarget(CommandType)`, `StructureMenu(StructureMenuState)`. The Agent needs a new variant (e.g., `AgentMenu(AgentMenuState)`) since it's a unit with unique commands, not a structure.
- **AgentMenuState enum**: New enum needed in `src/ui/types.rs` with variants: `AgentDefault`, `AgentAwaitingPlacement` (for Build Tunnel ghost). Pattern follows `StructureMenuState`.
- **`is_placement_mode()`** at `src/ui/types.rs:156`: Must be updated to include `AgentAwaitingPlacement`.

### Command Panel Grid Slots
- **`get_grid_slot_action()`** at `src/ui/command_panel.rs:40`: Add a match arm for `ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault)` returning:
  - `(0, 0)` → new `CommandButtonAction::AgentBuildTunnel`
  - `(0, 1)` → new `CommandButtonAction::AgentDropOff` (needs greyed-out support)
- **`CommandButtonAction` enum** at `src/ui/types.rs:216`: Add `AgentBuildTunnel` and `AgentDropOff` variants.

### Button Grey-Out (Drop Off Resources)
- Currently no greyed-out button pattern exists in the command panel. Buttons are either shown (Some) or hidden (None) in `get_grid_slot_action()`.
- **New requirement**: A conditional visibility mechanism for `AgentDropOff` — always visible but greyed when Agent has no resources. Two options:
  1. Add an `is_carrying` field to `SelectedUnitCapabilities` at `src/ui/types.rs:318`, computed from Agent carry state.
  2. Return the button action always, but gate interactivity via a separate `is_enabled` check in the button click handler.
- Option 1 is cleaner: extend `SelectedUnitCapabilities` with `is_carrying: bool` (or `agent_carrying: bool` to be specific), compute it in `compute_selected_unit_capabilities()` at `src/ui/command_panel.rs:1144`.

### New UnitCommand Variants
- `UnitCommand` at `src/game/units/types/state/commands.rs:7` needs new variants:
  - `Gather(Entity)` — target is a SpaceCrystalsPatch or SupplyDeliveryStation
  - `DropOffResources(Entity)` — target is an own Tunnel
  - `BuildTunnel(Vec3)` — target location for tunnel placement
- Update `UnitCommand::is_available()` at line 33 — these commands should require `is_syndicate` (or a new `is_agent` flag).

### New Agent Carry State Component
- No carry/resource state component exists yet. Need a new component (e.g., `AgentCarryState`) in `src/game/units/types/` or `src/game/units/types/state/`:
  ```rust
  #[derive(Component, Default, Clone, Debug)]
  pub struct AgentCarryState {
      pub crystals: u32,
      pub supplies: u32,
  }
  impl AgentCarryState {
      pub fn is_carrying(&self) -> bool { self.crystals > 0 || self.supplies > 0 }
      pub fn carrying_crystals(&self) -> bool { self.crystals > 0 }
  }
  ```
- This component should be added to Agent entities at spawn time (in `spawn_agent()` at `src/game/utils.rs`).

### Agent Detection in update_command_panel_state
- `update_command_panel_state()` at `src/ui/command_panel.rs:206` currently routes to `ObjectInterfaceState::Default` for all selected units (line 229). When the selected unit(s) include Agents, it should switch to `AgentMenu(AgentDefault)` instead.
- Detection: query `ObjectInstance` on selected units, check if `object_type == ObjectEnum::SyndicateAgent`. Since Agent is ungroupable, a single selected Agent means exactly 1 unit in selection.
- The system's unit query at line 211 doesn't include `ObjectInstance` — add `&ObjectInstance` to it, or add a separate Agent-specific query.

### Right-Click Resolution
- `right_click_move_command()` at `src/game/units/systems/core.rs:178` is the main right-click handler (330+ lines). It already has precedent for unit-type-specific branching (Supply Chopper at lines 244-277).
- **Agent-specific right-click branch**: After the attack check and before the generic move fallthrough, add an Agent branch that checks:
  1. Target is `SpaceCrystalsPatch` → `UnitCommand::Gather(target_entity)`
  2. Target is `SupplyDeliveryStation` → `UnitCommand::Gather(target_entity)` (supplies)
  3. Target is own Tunnel + Agent is carrying → `UnitCommand::DropOffResources(target_entity)`
  4. Target is own Tunnel + Agent not carrying → `UnitCommand::Enter(target_entity)` (existing)
  5. Fallthrough to existing enemy attack / ground move logic
- Detecting target types: query for `SpaceCrystalPatch` component (at `src/game/world/types.rs`), `TunnelState` component (at `src/game/types/structures.rs`), or check `ObjectInstance.object_type`.
- Detecting carry state: query `AgentCarryState` on the selected Agent entities.
- The `target_info` query at line 184 already queries `SupplyDeliveryStation` — extend it or add a parallel query for `SpaceCrystalPatch` and `TunnelState`.

### Build Tunnel Placement Flow
- Existing placement infrastructure: `PlacementState` resource at `src/ui/types.rs:278`, `PlacementGhost` component at `src/ui/types.rs:270`, `update_placement_ghost()` at `src/game/world/faction.rs:847`, `placement_click_system()` at `src/game/world/faction.rs:990`.
- Current placement systems are wired to DC/EF structures as the source. Agent tunnel placement needs to:
  1. Set `PlacementState.building_type = Some(ObjectEnum::Tunnel)` and `source_entity` to the Agent entity (not a structure).
  2. Reuse existing ghost/validation systems. `can_place_building()` at `src/game/world/utils.rs:179` may need adjustment — Agent uses `can_worker_place_structure()` from the `worker_built_structure_arrival_validation` task.
  3. On valid click: issue `UnitCommand::BuildTunnel(world_pos)` to the Agent, NOT spawn the structure immediately (Agent must walk there first).
- Rotation/flip inputs: already handled by existing `placement_input_system` that reads R/Shift+R/F/Shift+F and updates `PlacementState.rotation`/`flip_*`.
- Cancel: Escape/right-click returns `ObjectInterfaceState` to `AgentMenu(AgentDefault)` and clears `PlacementState`.

### Groupable Fix (prerequisite from agent_groupable_and_construction_fix)
- `SyndicateAgent` at `src/game/types/objects.rs:230` currently has `groupable: true` — must be `false` per design. The `agent_groupable_and_construction_fix` task handles this.

### Key Entity/Component References
- `SpaceCrystalPatch` component: `src/game/world/types.rs` (marker for crystal patches)
- `SupplyDeliveryStation` component: `src/game/world/types.rs` (marker for SDS entities)
- `TunnelState` component: `src/game/types/structures.rs` (tunnel instance state)
- `ObjectInstance.object_type`: `src/game/types/objects.rs` — use to identify `ObjectEnum::SyndicateAgent`, `ObjectEnum::Tunnel`, etc.
- `Owner` component: `src/shared/types.rs` — use `owner.player_number()` to check "own" tunnel

### Files to Modify
1. **`src/ui/types.rs`**: Add `AgentMenuState` enum, `AgentMenu` variant to `ObjectInterfaceState`, `AgentBuildTunnel`/`AgentDropOff` to `CommandButtonAction`, extend `SelectedUnitCapabilities` with `agent_carrying`, update `is_placement_mode()`.
2. **`src/ui/command_panel.rs`**: Add `AgentMenu` match arms in `get_grid_slot_action()` and `update_command_panel_state()`. Add `AgentCarryState` to queries. Extend `compute_selected_unit_capabilities()`.
3. **`src/game/units/types/state/commands.rs`**: Add `Gather(Entity)`, `DropOffResources(Entity)`, `BuildTunnel(Vec3)` to `UnitCommand`. Update `is_available()`.
4. **`src/game/units/systems/core.rs`**: Add Agent-specific right-click branch in `right_click_move_command()`. Add `AgentCarryState` + target component queries.
5. **New file or extend existing**: `AgentCarryState` component definition (suggest `src/game/units/types/state/` or alongside other unit state types).
6. **`src/game/utils.rs`**: Add `AgentCarryState::default()` to Agent spawn function.
7. **`src/game/world/faction.rs`**: Extend `placement_click_system()` to handle Agent-sourced placements (issue `BuildTunnel` command instead of spawning structure directly).

## Dependencies
- **`command_panel_and_interface_state_machine`** (in qa_tasks): Provides the `ObjectInterfaceState` framework, `get_grid_slot_action()`, and `CommandButtonAction` that this task extends. Must be completed first.
- **`syndicate_agent_unit`** (in qa_tasks): Defines the Agent unit type data, spawn function, and `ObjectEnum::SyndicateAgent`. Must exist before interface state can reference it.
- **`agent_groupable_and_construction_fix`** (in developer_tasks): Fixes Agent's `groupable: true → false`. Must be done before or alongside this task — the single-Agent selection assumption depends on ungroupable.
- **`tunnel_structure_and_network`** (in qa_tasks): Tunnel structure must exist for Build Tunnel placement and Drop Off targets.
- **`enter_command_and_entering_tunnel_behavior`** (in qa_tasks): The Enter command variant and behavior for Tunnels. Shared fallback when Agent right-clicks own Tunnel without carrying resources.
- **`worker_built_structure_arrival_validation`** (in developer_tasks): Defines `can_worker_place_structure()` used for Agent tunnel placement validation. Needed for the Build Tunnel ghost validation.

## QA Steps
1. [human] Select an Agent. Verify the command panel shows two DefaultState commands: A (Build Tunnel) and B (Drop Off Resources).
2. [human] Verify button B (Drop Off Resources) is greyed out when the Agent is not carrying resources.
3. [human] Have the Agent pick up crystals from a Space Crystal Patch. Verify button B is now active (not greyed out).
4. [human] Press A (or hotkey). Verify the interface enters AwaitingPlacement: a ghost Tunnel preview follows the cursor, snapped to grid, tinted green on valid placement and red on invalid.
5. [human] While in AwaitingPlacement, press R. Verify the ghost rotates 90 degrees clockwise. Press Shift+R. Verify counter-clockwise rotation. Press F. Verify horizontal flip. Press Shift+F. Verify vertical flip.
6. [auto] Left-click a valid location. Verify the Agent is dispatched to build at that location (CommandIssuingTransition) and the interface returns to DefaultState.
7. [auto] Enter AwaitingPlacement again, then press Escape. Verify the interface returns to DefaultState without issuing a command.
8. [auto] Right-click a Space Crystal Patch. Verify the Gather crystals command is issued (Agent walks to the patch and begins mining).
9. [auto] Right-click a Supply Delivery Station. Verify the Gather supplies command is issued.
10. [auto] Have the Agent carry crystals. Right-click an own Tunnel. Verify the Drop Off Resources command is issued (Agent walks to Tunnel Side B).
11. [auto] Have the Agent carry supplies. Right-click an own Tunnel. Verify the Drop Off Resources command is issued (Agent walks to Tunnel Side C).
12. [auto] Have the Agent carry nothing. Right-click an own Tunnel. Verify the Enter command is issued (Agent walks to Side A and enters the Tunnel Network).
13. [auto] Right-click an enemy unit. Verify the Attack command is issued (Agent moves to melee range and attacks).
14. [auto] Right-click empty ground. Verify the Move command is issued.
15. [auto] Select multiple Agents. Right-click on ground. Verify all selected Agents receive the Move command, not just the ActiveGroup Agent.
16. [auto] Select multiple Agents. Right-click a Crystal field. Verify all selected Agents receive the Gather command.

## QA Failure (Resolved)
- Step 16 [auto]: Previously FAIL — Multi-select Agents right-clicking a Crystal field: agent1 received Move command instead of Gather. **Resolved**: The right-click handler at `src/game/units/systems/core.rs:284-302` correctly iterates all selected units and issues Gather to every SyndicateAgent. The fix was applied in a subsequent task that touched this code path. All 15 automated tests now pass (1 ignored: escape key headless limitation).

## Automated QA Results
- Step 6 [auto]: PASS — Placement left-click dispatches BuildTunnel, returns to DefaultState
- Step 7 [auto]: DEFERRED — Escape key not testable headlessly
- Step 8 [auto]: PASS — Right-click crystal patch issues Gather
- Step 9 [auto]: PASS — Right-click Supply Delivery Station issues Gather
- Step 10 [auto]: PASS — Carrying crystals + right-click tunnel → DropOff
- Step 11 [auto]: PASS — Carrying supplies + right-click tunnel → DropOff
- Step 12 [auto]: PASS — Not carrying + right-click tunnel → Enter
- Step 13 [auto]: PASS — Right-click enemy → Attack
- Step 14 [auto]: PASS — Right-click ground → Move
- Step 15 [auto]: PASS — Multi-select agents right-click ground → all Move
- Step 16 [auto]: PASS — Multi-select agents right-click crystal → all Gather (previously FAIL, now resolved)
- Steps 1-5 [human]: Pending human review

## Expected Experience
The Agent's command panel should feel distinct from standard combat units. The two buttons (Build Tunnel and Drop Off Resources) should clearly indicate the Agent's worker role. The Drop Off button should visually grey out when the Agent has no resources, providing clear feedback on carry state. The AwaitingPlacement flow for Tunnel building should match the existing building placement UX (ghost preview, rotation, flipping, green/red tinting). Right-clicking should feel context-aware: the Agent automatically does the "right thing" based on what was clicked and whether it's carrying resources. Clicking an own Tunnel should intelligently choose Enter vs. Drop Off based on carry state, without the player needing to explicitly pick the command.
