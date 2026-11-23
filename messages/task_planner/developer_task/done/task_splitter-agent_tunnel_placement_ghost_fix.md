# agent_tunnel_placement_ghost_fix

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-agent_groupable_construction_r1.md

## Task

Fix the missing placement ghost for Agent Build Tunnel command. The `manage_placement_ghost` system in `src/game/world/faction/placement.rs` (lines 36-56) has a match on `*panel_state` to determine `building_type`, but it lacks a case for `AgentAwaitingPlacement`. It falls through to `_ => None`, so the ghost entity is never spawned.

**The fix**: Add a match arm for `ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement)` that returns `placement_state.building_type` (same pattern as the `TunnelAwaitingPlacement` and `RecruitAwaitingPlacement` arms on lines 47-53).

**Exact location**: `src/game/world/faction/placement.rs`, line 54 (just before the `_ => None` catchall). Insert:

```rust
ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement) => {
    placement_state.building_type
}
```

**Verification**: The rest of the Agent placement flow already works — action execution (actions.rs:595-605) correctly sets `placement_state.building_type = Some(ObjectEnum::Tunnel)`, validation (placement.rs:209), confirmation (placement.rs:302), and left-click placement (placement.rs:492) all handle `AgentAwaitingPlacement`. Only the ghost spawning is missing.

Run `cargo test` to verify no regressions. Existing test `agent_awaiting_placement_is_placement_mode` confirms the state is recognized as placement mode.
