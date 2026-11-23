# agent_rightclick_gather_fix

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-agent_resource_gathering_r1.md

## Task

Fix right-click Gather command for Syndicate Agents on SpaceCrystalsPatch and SupplyDeliveryStation entities. QA reports that right-clicking a crystal field with an Agent selected does nothing — no Gather command is issued.

### What to investigate

The gather handler in `src/game/units/systems/core/command_dispatch.rs` at ~line 343 checks:
```rust
if is_right_click && command_type == CommandType::Default && has_selected_agents {
    if let Ok((..., crystal_opt, ...)) = target_info.get(target_entity) {
        if crystal_opt.is_some() { ... }
    }
}
```

This code looks structurally correct. Possible failure points to investigate:

1. **Cursor target detection**: `update_cursor_target` in `ui/command_panel/state.rs` uses ray-AABB intersection with `SelectionBounds`. Crystal patches are spawned with `SelectionBounds` and `ObjectInstance` in `game/world/resources.rs`. Verify the AABB is sized correctly and the ray intersection actually detects crystal patches (check debug logs).

2. **CursorTargetEnum classification**: Crystal patches have `Owner::neutral()`, so `cursor_target.kind` should be `NeutralObject`. Verify this doesn't cause an early return anywhere before the gather handler.

3. **Fall-through to ground move**: If `cursor_target.entity` is None (AABB miss), the entity click block at line 93 is skipped entirely, and the system falls through to the ground move handler — which would explain 'nothing happens' if the Move also fails or the Agent appears to do nothing visible.

4. **System ordering**: Verify `update_cursor_target` runs before `right_click_move_command` in the schedule.

5. **has_selected_agents check**: At line 89, this checks `obj.object_type == ObjectEnum::SyndicateAgent`. Verify the Agent is spawned with the correct ObjectEnum variant.

### What to fix

Once the root cause is identified, fix it so that:
- Right-clicking a SpaceCrystalsPatch issues `UnitCommand::Gather(target)` and inserts `GatheringResourceBehavior`
- Right-clicking a SupplyDeliveryStation issues `UnitCommand::Gather(target)` and inserts `GatheringResourceBehavior`
- The Agent walks to the target and begins the gathering behavior

### What to verify

- Run `cargo test` to ensure no regressions
- Add a test if the fix is non-trivial (e.g., if the root cause is a missing component or incorrect bounds)
