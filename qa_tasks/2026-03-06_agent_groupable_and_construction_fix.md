# Task: Agent Groupable and Construction Rule Fix

## Current State
The Syndicate Agent unit was implemented with `Groupable: true`. Per the corrected feature specification, Agent should be `Groupable: false` (Ungroupable) -- each Agent forms its own SelectionGroup. Additionally, the single-Agent construction rule (only one Agent may construct a given Tunnel; multiple Agents cannot speed up construction) is not explicitly enforced.

## Desired State
1. **Set Agent `Groupable: false`**: Each Agent must always be its own SelectionGroup, identical to how Tunnels work. Despite being ungroupable, right-click commands are still issued to **all selected Agents** simultaneously (this is a special case noted in the feature spec).
2. **Enforce single-Agent construction**: When an Agent is dispatched to build a Tunnel at a location where another Agent is already constructing, the command should be rejected or the second Agent should not begin construction. Only one Agent may construct a given Tunnel at a time.

## Justification
`features/syndicate_objects.md` (Agent section): "Groupable: false (Ungroupable -- each Agent is its own SelectionGroup, but right-click commands are issued to all selected Agents simultaneously)" and Building section: "Only one Agent may construct a given Tunnel -- multiple Agents cannot speed up construction." The original `syndicate_agent_unit` ticket (now in QA) had `Groupable: true`, which was incorrect per the design source. Corrected in `feature_updates/2026-03-06_syndicate_objects_agent_control_panel.md`.

## QA Steps
1. Select a single Agent. Verify it forms its own SelectionGroup of size 1.
2. Box-select multiple Agents. Verify each Agent is in its own separate SelectionGroup (Tab/Shift-Tab cycles through individual Agents, not a merged group).
3. With multiple Agents selected, right-click on ground. Verify all selected Agents receive the Move command, not just the ActiveGroup Agent.
4. With multiple Agents selected, right-click on an enemy. Verify all selected Agents receive the Attack command.
5. Order Agent A to build a Tunnel at a location. While Agent A is constructing, order Agent B to build at the same location.
6. Verify Agent B's build command is rejected -- Agent B should not begin constructing the same Tunnel.
7. Verify only one Agent is embedded in the partially-built Tunnel at any time.
8. Destroy the partially-built Tunnel while Agent A is constructing. Verify Agent A emerges. Now order Agent B to build a new Tunnel at that location. Verify Agent B can construct successfully (the restriction is per-Tunnel, not per-location after destruction).

## Expected Experience
When selecting multiple Agents, the selection panel should show them as separate entries (one per SelectionGroup), and Tab should cycle between them. Despite this ungroupable status, right-clicking issues commands to all selected Agents -- the player should see all their Agents respond to the right-click, not just the currently active one. If a player tries to send a second Agent to help build an already-under-construction Tunnel, the second Agent should not participate in construction.

---

## Technical Context

### Part 1: Agent Groupable (Already Done)

**Status: This part is already implemented correctly.** The developer can verify and skip.

- `src/game/types/objects.rs:225-231` — `ObjectEnum::SyndicateAgent` already has `groupable: false` with comment "Ungroupable — each Agent is its own SelectionGroup"
- `src/game/types/objects.rs:1091-1094` — Test `test_syndicate_agent_is_ungroupable()` already asserts `!obj.groupable`
- `src/shared/types.rs:170-199` — `Selection::build_from_entities()` already handles ungroupable entities correctly (each gets own group at line 190-194)
- Right-click command dispatch (`src/game/units/systems/core.rs:176-532`) queries `With<Selected>` and iterates ALL selected units, not just active group — Agent commands at lines 282-344 correctly filter by `ObjectEnum::SyndicateAgent` within the full selected set

### Part 2: Single-Agent Construction Enforcement (Needs Implementation)

**Status: Not yet implemented.** The BuildTunnel flow currently has no check preventing multiple Agents from building at the same location.

#### Current Build Flow
1. UI: Agent command panel button triggers `AgentMenuState::AgentAwaitingPlacement` — `src/ui/command_panel.rs:125,1101`
2. Placement click: `src/game/world/faction.rs:1205-1212` — issues `UnitCommand::BuildTunnel(world_pos)` to the source Agent entity
3. Command-to-behavior: Not yet wired — the `UnitCommand::BuildTunnel` variant exists (`src/game/units/types/state/commands.rs:33`) but the behavior pipeline conversion is part of the `agent_tunnel_building_command_and_behavior` developer task

#### Where to Add the Enforcement
The single-Agent construction check should be added in the **behavior dispatch** (when `UnitCommand::BuildTunnel` is converted to `BuildingStructureBehavior`/`BuildingTunnelBehavior`). At that point, query all existing `BuildingStructureBehavior` components to check if any other entity is already building at the same target location (within a small tolerance).

**Key files:**
- `src/game/units/systems/behaviors.rs:470-532` — `building_behavior_system()` — the existing build behavior system. The enforcement check could go here (reject if another builder already targets same location) or in the command dispatch that creates the behavior
- `src/game/units/types/state/commands.rs:33` — `UnitCommand::BuildTunnel(Vec3)` definition
- `src/game/world/faction.rs:1205-1212` — placement click issues BuildTunnel command

**Recommended approach:**
- In the system that converts `UnitCommand::BuildTunnel` to the building behavior marker, add a query for all existing `BuildingStructureBehavior` components. If any has a `target_location` matching the new build target (within ~1.0 world unit tolerance), reject the command and set the Agent to `UnitCommand::Idle` instead.
- Alternative: check at `building_behavior_system` entry — when a new `BuildingStructureBehavior` is detected (`!arrived`), verify no other entity with `BuildingStructureBehavior` targets the same location.

**Pattern reference:** `BuildingStructureBehavior` at `src/game/units/systems/behaviors.rs` has `target_location: Vec3` — compare this field across all builders.

### Dependencies
- **`2026-03-07_agent_tunnel_building_command_and_behavior`** — The BuildTunnel behavior pipeline must exist before the single-agent construction enforcement can be implemented. The enforcement check runs at behavior dispatch time, which this task creates. **Part 2 of this task depends on this task being completed first.** Part 1 (groupable fix) is already done and has no dependency.
