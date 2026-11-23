# agent_tunnel_placement_ghost_fix

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-agent_groupable_construction_r1.md

## Task

Fix the missing placement ghost for Agent Build Tunnel command. The manage_placement_ghost system in src/game/world/faction/placement.rs (lines 36-56) has a match on *panel_state to determine building_type, but it lacks a case for AgentAwaitingPlacement. It falls through to _ => None, so the ghost entity is never spawned.

**The fix**: Add a match arm for ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement) that returns placement_state.building_type (same pattern as the TunnelAwaitingPlacement and RecruitAwaitingPlacement arms on lines 47-53).

**Exact location**: src/game/world/faction/placement.rs, line 54 (just before the _ => None catchall).

**Verification**: The rest of the Agent placement flow already works. Run cargo test to verify no regressions. Existing test agent_awaiting_placement_is_placement_mode confirms the state is recognized as placement mode.

## Technical Context

**File to change**: artifacts/developer/src/game/world/faction/placement.rs

**Exact edit** (line 54, between the CultsRecruitMenu(RecruitAwaitingPlacement) arm and the _ => None catchall):

Add this match arm:
```rust
ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement) => {
    // The building_type is already set in PlacementState by execute_command_action
    placement_state.building_type
}
```

**Pattern to follow**: Lines 51-54 (RecruitAwaitingPlacement arm) — identical structure. The placement_state.building_type is pre-populated by the action execution path before the interface state transitions to AgentAwaitingPlacement.

**Import check**: AgentMenuState is already imported on line 8 of placement.rs. No new imports needed.

**Why it works**: The placement pipeline for Agent Build Tunnel is:
1. execute_command_action (actions.rs:595-605) sets placement_state.building_type = Some(ObjectEnum::Tunnel) and transitions to AgentAwaitingPlacement
2. manage_placement_ghost (placement.rs:17) detects the state change, matches the interface state to get building_type, and spawns a ghost entity — THIS IS THE BROKEN STEP
3. update_placement_ghost_position (placement.rs:~130) moves the ghost to cursor
4. validate_placement_position (placement.rs:209) checks tile validity
5. confirm_placement (placement.rs:302) handles left-click to finalize

Steps 1, 3, 4, 5 already handle AgentAwaitingPlacement. Only step 2 is missing the match arm.

**No system ordering or plugin changes needed** — the system is already registered and running.

## Dependencies

None — this is a standalone one-line fix to an existing match expression.
