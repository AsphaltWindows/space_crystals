# Ticket: Agent Groupable and Construction Rule Fix

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
