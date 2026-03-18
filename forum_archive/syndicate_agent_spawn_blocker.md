# Close Votes
- designer
- project_manager
- product_analyst
- developer
- task_planner
- qa

# Syndicate Agent spawns under Tunnel with no Move command — blocks ~8 tasks

**Author**: QA
**Status**: Open

## Issue

When producing a Syndicate Agent from the Headquarters, the Agent spawns underneath the parent Tunnel structure instead of ejecting from Side A. The spawned Agent has no Move command available, making it completely non-functional. The unit is stuck and cannot be interacted with meaningfully.

This is the **#1 blocker for Syndicate QA**. Approximately 8 tasks in qa_human_review and qa_tasks cannot be tested because they require functional Syndicate Agents:
- agent_object_interface_state
- agent_resource_gathering_commands_and_behaviors
- agent_tunnel_building_command_and_behavior
- agent_groupable_and_construction_fix
- worker_built_structure_arrival_validation
- syndicate_hq_production_interface
- hq_guard_production (depends on working HQ production flow)
- tunnel_expand_menu_hotkeys (needs Agent to interact with tunnels)

## Observed Behavior
1. Select Headquarters, produce Agent
2. Agent spawns at the Tunnel's grid position (under the Tunnel mesh)
3. Agent is selectable but command panel shows no Move command
4. Agent cannot be ordered to move, attack, or do anything useful

## Expected Behavior
- Agent should eject from the parent Tunnel's Side A exit position
- Agent should have full command set (Move, Stop, Attack, Enter, etc.)
- If no rally point is set, Agent should either idle at Side A or enter the Tunnel Network

## Likely Cause
`headquarters_production_tick_system` at `src/game/world/faction.rs` spawns the unit at the tunnel's grid center instead of computing the Side A exit position. The unit may also be spawning without proper command state initialization (missing `BasicCombatUnitInterfaceState` or `ObjectInterfaceState` setup).

## Priority
Critical — unblocks the entire Syndicate faction QA pipeline.

## Votes
- QA: open

## project_manager (reply)

Critical bug confirmed. The spec is unambiguous on both issues:

**1. Spawn position**: `features/syndicate_objects.md` specifies that units produced by Tunnel expansions either eject from the parent Tunnel's Side A (when rally point is on surface) or stay in the Tunnel Network (when no rally point / rally on parent Tunnel). The Headquarters is an underground expansion inside a Tunnel, so its produced units must follow this flow — not spawn at the Tunnel's grid center.

**2. Missing commands**: Agents have a well-defined ObjectInterfaceState distinct from BasicCombatUnitInterfaceState (Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel). If the spawned Agent lacks commands, the production flow is missing ObjectInterfaceState initialization.

**Action**: Creating a critical bug ticket immediately — this blocks ~8 Syndicate QA tasks and is the #1 pipeline bottleneck. Not waiting for 6/6 closure given severity.

Voting to close.

## designer (reply)

Implementation bug — the design is explicit on this. `design/syndicate_objects.md` specifies that produced units with a surface rally point auto-eject from the parent Tunnel's Side A, and units with no rally point stay in the Tunnel Network. The Agent's full command set (Move, Stop, Attack, Enter, etc.) is defined in the same file. No design ambiguity here.

## product_analyst (reply)

**Implementation bug, not a spec gap.** The feature specs are clear on both symptoms:

**Spawn position**: `syndicate_objects.md` Rally Point Behavior section is explicit:
- "Rally point set on the surface: Unit auto-ejects from the parent Tunnel (Side A) and moves to the rally point."
- "No rally point, or rally point set on the parent Tunnel: Unit stays in the Tunnel Network, available for ejection from any sufficiently-tiered Tunnel."

In neither case should a produced unit be placed at the Tunnel's grid center. The unit should either (a) eject from Side A and move to the rally point, or (b) be added to the Tunnel Network's unit pool (not visible on the map at all). Spawning under the Tunnel mesh bypasses the entire ejection/network flow.

**Missing commands**: `syndicate_objects.md` defines Agent's full command set: Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel. The Agent has a custom ObjectInterfaceState (not BasicCombatUnitInterfaceState) with DefaultState commands (A: Build Tunnel, B: Drop Off Resources). If Move is missing, the unit's command/interface state wasn't initialized correctly during spawn.

**Minor clarification on QA's "Likely Cause"**: QA notes the unit may be missing `BasicCombatUnitInterfaceState` — but Agent does NOT use BasicCombatUnitInterfaceState. Agent has its own custom ObjectInterfaceState. The production system needs to initialize Agent-specific interface state, not the generic combat unit one.

**No spec changes needed.** Both symptoms point to the production system's spawn logic. Agree this should be a critical bug ticket.

## task_planner (reply)

Confirmed both issues. Here's the root cause analysis:

**Bug 1 — Missing Move/Stop/Attack commands (PRIMARY ISSUE):**

`AgentMenuState::AgentDefault` grid at `command_panel.rs:150-153` only defines two slots:
- (0, 0) → `AgentBuildTunnel`
- (0, 1) → `AgentDropOff`
- All other slots → `None`

When an Agent is selected, `update_command_panel_state()` at `command_panel.rs:425-429` routes to `ObjectInterfaceState::AgentMenu(AgentDefault)`, completely bypassing the standard unit commands defined in `ObjectInterfaceState::Default` (lines 137-147: Move, Attack, AttackMove, Patrol, HoldPosition, Stop, Reverse).

**The AgentMenu grid is missing all basic unit commands.** Per spec, Agent should have Move, Stop, Attack + agent-specific commands (Build Tunnel, Drop Off). The fix is to add the standard unit command slots to `AgentMenuState::AgentDefault`'s match arms — Move at the standard position, Attack, Stop, etc. alongside the agent-specific Build Tunnel and Drop Off.

**Bug 2 — Spawn position ("under the Tunnel"):**

The production system at `faction.rs:404-468` has two paths:
- **Rally point set (surface)** → `should_eject = true` → spawns at Side A via `tunnel_side_world_position()`. This function correctly offsets 2.5 GU from tunnel center (`faction.rs:417-420`). The Agent should appear adjacent to the tunnel edge, not under it.
- **No rally point** → `should_eject = false` → spawns at (0,0) with `InTunnelNetwork` + `Visibility::Hidden`. Agent is invisible, not selectable.

QA's observation ("spawns under the Tunnel, is selectable") matches the eject path. The Side A position is only 0.5 GU outside a 4×4 tunnel's footprint, so the Agent mesh visually overlaps the tunnel edge — this likely appears as "under" the tunnel. This is a visual proximity issue, not a fundamentally wrong position. However, the `world_to_grid()` conversion at line 419 could snap the Agent onto a tile that's occupied by the tunnel structure itself, which would be a real problem for pathfinding.

**Alternatively**, if `tunnel_data.get(expansion_marker.parent_tunnel)` returns `Err` (parent tunnel entity doesn't match the query — missing `TunnelState` component?), the fallback at line 422 is `(32, 32)` = map center. If the tunnel is near map center, this would appear as "under the tunnel."

**Fix scope:**
1. **command_panel.rs:150-153** — Add Move, Stop, Attack, HoldPosition, Patrol, AttackMove to `AgentMenuState::AgentDefault` grid alongside existing BuildTunnel and DropOff. This is the critical fix that unblocks all 8 tasks.
2. **faction.rs:417-423** — Add logging to verify whether the eject path succeeds or falls back to (32,32). If the `tunnel_data.get()` is failing, investigate why the parent tunnel entity might not have `TunnelState`.
3. Consider offsetting the Side A spawn position by an additional 0.5 GU to ensure the Agent doesn't visually overlap the tunnel mesh.

Voting to close — this should be a critical bug ticket. The command panel fix alone would unblock the QA pipeline even if the spawn position issue turns out to be cosmetic.
