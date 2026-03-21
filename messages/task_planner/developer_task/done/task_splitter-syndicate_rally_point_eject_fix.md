# syndicate-rally-point-eject-fix

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-syndicate-rally-point.md

## Task

Fix the eject decision logic in `headquarters_production_tick_system` (faction.rs ~line 407) so that units stay in the Tunnel Network when no rally point is set.

**Current bug**: The match arm uses `_ => true` as the default, which catches `None` (no rally point) and causes the unit to eject. Per the design spec, `None` should mean the unit stays in the tunnel network.

**Current code** (~line 407-413 of faction.rs):
```rust
let should_eject = match &hq_state.rally_point {
    Some(RallyTarget::Object(entity)) if *entity == expansion_marker.parent_tunnel => {
        false
    }
    _ => true, // BUG: None should be false
};
```

**Correct logic**:
```rust
let should_eject = match &hq_state.rally_point {
    Some(RallyTarget::Object(entity)) if *entity == expansion_marker.parent_tunnel => false,
    Some(_) => true,  // Location or non-parent Object → eject
    None => false,     // No rally point → stay in tunnel network
};
```

The existing tests at ~line 2077-2105 (`headquarters_production_eject_decision_with_parent_tunnel_rally` and `headquarters_production_eject_decision_with_enemy_rally`) already verify the correct behavior with `None => false`. The system code just doesn't match. Fix the system and ensure all existing tests pass.
