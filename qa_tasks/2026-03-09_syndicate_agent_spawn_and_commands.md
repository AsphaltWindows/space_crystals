# Task: Fix Syndicate Agent spawn position and missing commands (critical blocker)

## Source Ticket
`tickets/2026-03-09_syndicate_agent_spawn_and_commands.md`

## Summary
Two critical bugs prevent Syndicate Agents from functioning: (1) the Agent command panel is missing all standard unit commands (Move, Stop, Attack, etc.), and (2) the spawn position from HQ production may place Agents incorrectly. Bug #1 is the primary blocker for ~8 Syndicate QA tasks.

## Dependencies
None — this is a critical blocker and should be prioritized.

## Technical Context

### Bug 1: Missing Unit Commands in AgentMenuState (PRIMARY FIX)

**File**: `src/ui/command_panel.rs` lines 149-158

**Current code** — `AgentMenuState::AgentDefault` grid only defines two slots:
```rust
ObjectInterfaceState::AgentMenu(am) => match am {
    AgentMenuState::AgentDefault => match (row, col) {
        (0, 0) => Some(CommandButtonAction::AgentBuildTunnel),
        (0, 1) => Some(CommandButtonAction::AgentDropOff),
        _ => None,  // ← ALL standard commands missing
    },
    ...
}
```

**Problem**: When an Agent is selected, `update_command_panel_state()` at lines 410-425 routes to `ObjectInterfaceState::AgentMenu(AgentDefault)`, completely bypassing the standard unit commands defined in `ObjectInterfaceState::Default` (lines 137-147). The Agent gets Build Tunnel and Drop Off but no Move, Stop, Attack, etc.

**Fix**: Add the standard unit commands to the `AgentDefault` grid. Per the feature spec (`features/syndicate_objects.md`), Agent's DefaultState should have:
- Move, Stop, Attack (standard unit commands)
- Enter (tunnel entry)
- Gather, Drop Off Resources (resource commands)
- Build Tunnel (agent-specific)

Reference the `ObjectInterfaceState::Default` grid at lines 137-147 for the standard slot positions:
```rust
(0, 0) => Some(CommandButtonAction::Move),
(0, 1) => Some(CommandButtonAction::Attack),
(0, 2) => Some(CommandButtonAction::AttackGround),
(1, 0) => Some(CommandButtonAction::AttackMove),
(1, 1) => Some(CommandButtonAction::Patrol),
(1, 2) => Some(CommandButtonAction::HoldPosition),
(2, 0) => Some(CommandButtonAction::Stop),
(2, 2) => Some(CommandButtonAction::Reverse),
```

The Agent grid should include the relevant subset of these alongside its agent-specific commands. Check the feature spec for exact slot assignments — the Agent has a custom layout that combines standard and specialized commands.

**Key consideration**: The `CommandButtonAction` enum must already have all the standard actions (Move, Attack, Stop, etc.) defined. The Agent-specific actions `AgentBuildTunnel` and `AgentDropOff` are also already defined. You're just adding more match arms to the grid, no new enum variants needed.

### Bug 2: Spawn Position Verification

**File**: `src/game/world/faction.rs` lines 371-477 — `headquarters_production_tick_system()`

**Eject flow** (lines 406-446):
1. Checks `hq_state.rally_point` to decide `should_eject` (line 406-413)
2. If ejecting: calls `tunnel_side_world_position(tunnel_tf, tunnel_si, 'A')` at line 418 to get Side A position
3. Converts to grid via `world_to_grid(side_a_world)` at line 419
4. **Fallback**: If `tunnel_data.get(expansion_marker.parent_tunnel)` fails, falls back to `(32, 32)` = map center (line 422)
5. Spawns at computed position via `spawn_syndicate_agent()` / `spawn_syndicate_guard()`

**`tunnel_side_world_position()`** at `src/game/units/systems/behaviors.rs` lines 543-563:
- Gets oriented label order from `StructureInstance::oriented_labels(ABCD)`
- Computes offset 2.5 GU from tunnel center (tunnel is 4×4, half = 2.0 + 0.5 buffer)
- Returns Vec3 at the cardinal direction of Side A

**Potential issues to verify**:
1. Does `tunnel_data.get(expansion_marker.parent_tunnel)` succeed? If parent tunnel entity doesn't have `TunnelState` in the query, it falls back to (32, 32). Add a `warn!()` log on the `Err` branch to verify.
2. The Side A position is only 0.5 GU outside the tunnel footprint. The Agent mesh (capsule 0.28 radius) could visually overlap. Consider adding 0.5 GU more offset if visual overlap is confirmed during QA.
3. `world_to_grid()` at `src/game/units/utils.rs:13` may snap the grid position onto a tunnel tile — check that the spawn tile is actually outside the tunnel's footprint.

**No-rally branch** (lines 451-467): Spawns at (0,0) with `InTunnelNetwork` + `Visibility::Hidden`. This path should be correct — unit is invisible and in the network.

### Spawn Components Reference

`spawn_syndicate_agent()` at `src/game/utils.rs` lines 487-581 attaches:
- Movement: `MovementSpeed`, `RotationSpeed`, `Velocity`, `TurnRateMovementParams`
- Combat: `AttackCapability`, `AttackState`, `AttackType::FullyConnected(Melee)`
- Resource: `AgentCarryState::default()` (line 567)
- Command: `UnitCommand::Idle`, `CommandQueue`, `BaseCommandState`, `BaseBehaviorState`
- Tunnel: `TunnelSpaceCost(2)` (line 560)

All necessary components appear to be attached. The issue is purely the command panel grid, not missing components.

`spawn_syndicate_guard()` — verify it attaches similar components. Guard uses `BasicCombatUnitInterfaceState`, which routes through `ObjectInterfaceState::Default` and should already have all standard commands.

### Existing Tests

- `tests/qa/agent_object_interface_state.rs` — tests for AgentMenuState transitions. Update these to verify the new grid slots.
- `get_grid_slot_action()` test call sites (~80 in tests) won't need changes unless the function signature changes. If you add a parameter (like `tunnel_is_upgrading`), all test call sites need the new arg.

### Pattern Notes
- The `ObjectInterfaceState::Default` grid is the reference for standard unit commands — Agent should include the same core commands
- Guard units use `BasicCombatUnitInterfaceState` which maps to `ObjectInterfaceState::Default` — Guards should already have all commands. Verify this works correctly.
- The rally point + eject flow was implemented in the HQ production system and should work mechanically — the command panel is the critical fix

## QA Steps
1. [human] Start a game as Syndicate. Select the starting Tunnel, open Eject menu (C), and eject any pre-placed Agent. Verify the Agent appears at Side A with full commands.
2. [human] Select the Headquarters (underground). Set a rally point on the surface by right-clicking open ground. Produce an Agent (Q). Verify the Agent ejects from the parent Tunnel's Side A and moves toward the rally point.
3. [human] Verify the ejected Agent's command panel shows: Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel.
4. [human] Clear the rally point (or set it on the parent Tunnel). Produce another Agent. Verify the Agent does NOT appear on the surface — it should enter the Tunnel Network instead.
5. [human] Open the Tunnel's Eject menu (C) and verify the newly produced Agent appears in the network unit list. Eject it and confirm it appears at Side A with full commands.
6. [human] Produce a Guard (W) with a surface rally point. Verify the Guard ejects from Side A and moves to the rally. Verify the Guard's command panel shows Move, Stop, Attack, Enter.

## Expected Experience
- Step 1: An Agent unit appears at the Tunnel's Side A edge. Selecting it shows a command panel with Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel.
- Step 2: After production completes, an Agent walks out from Side A of the Tunnel and pathfinds toward the rally point on the ground.
- Step 3: The command panel displays all seven commands. Each is clickable/hotkey-accessible.
- Step 4: No unit appears on the map surface after production completes. The Tunnel Network unit count increases.
- Step 5: The Eject menu shows the Agent type with an incremented count. Clicking it causes an Agent to emerge from Side A with full commands.
- Step 6: A Guard emerges from Side A and moves to the rally point. Its command panel shows the four BasicCombatUnit commands.
