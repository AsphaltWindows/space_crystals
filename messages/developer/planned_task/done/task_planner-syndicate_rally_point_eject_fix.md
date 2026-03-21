# syndicate-rally-point-eject-fix

## Metadata
- **From**: task_planner
- **To**: developer

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
    Some(_) => true,  // Location or non-parent Object -> eject
    None => false,     // No rally point -> stay in tunnel network
};
```

The existing tests at ~line 2077-2105 already verify the correct behavior with `None => false`. The system code just doesn't match. Fix the system and ensure all existing tests pass.

## Technical Context

**File to change**: `artifacts/developer/src/game/world/faction.rs`

**Exact location**: Line 407-413 — the `let should_eject = match` block inside `headquarters_production_tick_system`.

**What to change**: Replace the `_ => true` catch-all arm with two explicit arms:
- `Some(_) => true` — any rally point that isn't the parent tunnel means eject
- `None => false` — no rally point means stay in tunnel network

**Also clean up**: The comment on the old `_ => true` line (line 412) has a malformed comment — it reads `/ Default:` instead of `// Default:`. This gets replaced entirely.

**Pattern to follow**: The existing unit tests at lines 2019-2025 (`headquarters_no_rally_means_tunnel_network`) and 2082-2088 (`headquarters_production_eject_decision_with_parent_tunnel_rally`) already use the correct 3-arm match pattern. Mirror that exact structure in the production system.

**Verification**: Run `cargo test` — all existing tests (especially the 3 eject-decision tests at lines 2005-2105) should pass without modification. No new tests needed.

## Dependencies

None — this is a self-contained bug fix in a single match expression.
