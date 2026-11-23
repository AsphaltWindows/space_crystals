# agent_rightclick_gather_fix

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Key Files

1. **`artifacts/developer/src/game/units/systems/core/command_dispatch.rs`** — The `right_click_move_command` system. Contains the gather handler at lines 343-392 and existing passing tests at lines 1398-1459 (`gather_crystal_inserts_behavior_component`, `gather_sds_inserts_behavior_component`). These tests mock `CursorTarget` directly, bypassing cursor detection, so they prove the dispatch logic works but don't test actual AABB hit detection.

2. **`artifacts/developer/src/ui/command_panel/state.rs`** (lines 17-102) — The `update_cursor_target` system. Queries entities with `(Entity, &Transform, &Owner, &SelectionBounds), With<ObjectInstance>`. Does ray-AABB intersection with `click_pad = 0.3` world-space padding. Sets `CursorTarget.entity` and `CursorTarget.kind`. Crystal patches (neutral) get `CursorTargetEnum::NeutralObject`. Both systems already have debug logging that fires on click frames.

3. **`artifacts/developer/src/game/world/resources.rs`** (lines 11-80) — `spawn_space_crystal_patches()`. Spawns crystal patches with: `Transform::from_xyz(world_x, 0.4, world_z)`, `ObjectInstance::indestructible(...)`, `Owner::neutral()`, `SpaceCrystalPatch{...}`, `Selectable`, `SelectionBounds::from_dimensions(0.6, 0.8, 0.6)`, `GridPosition`. The SelectionBounds give half-extents of (0.3, 0.4, 0.3) — total AABB with padding is 1.2×1.4×1.2 centered at Y=0.4.

4. **`artifacts/developer/src/game/entities/neutral/space_crystals_patch.rs`** — The `SpaceCrystalsPatchDef` entity definition. Has the same components as resources.rs. The actual spawning uses resources.rs, not this definition.

5. **`artifacts/developer/src/game/entities/neutral/supply_delivery_station.rs`** — The `SupplyDeliveryStationDef`. SDS spawns with `SelectionBounds::from_dimensions(1.6, 0.2, 1.6)` at Y=0.1. Its AABB is much wider (2.2×0.8×2.2 with padding) so may be easier to hit.

6. **`artifacts/developer/src/ui/utils.rs`** (lines 34-55) — `ray_aabb_intersect()` function. Standard slab-based ray-AABB test. Used by `update_cursor_target`.

7. **`artifacts/developer/src/game/units/systems/behaviors/resource_gathering.rs`** — `gathering_resource_behavior_system`. Processes `GatheringResourceBehavior` through phases: MovingToResource → Extracting → MovingToTunnel → DroppingOff → repeat. Uses constants from `agent.rs`: `AGENT_MINING_DURATION=48`, `AGENT_PICKUP_DURATION=48`, `AGENT_CRYSTAL_CARRY=50`, `AGENT_SUPPLY_CARRY=1`.

### System Ordering

- `DiagCategory::UiHud` runs BEFORE `DiagCategory::Movement` (configured in `simulation/mod.rs` line 34)
- `update_cursor_target` is in UiHud set (`ui/mod.rs` line 39)
- `right_click_move_command` is in Movement set (`game/units/mod.rs` line 87)
- So cursor target is always fresh when command dispatch runs ✅

### Code Flow (Right-Click Default Mode on Crystal Patch)

1. `update_cursor_target` → ray-AABB detects crystal → sets `CursorTarget { entity: Some(crystal), kind: NeutralObject, location: Some(ground_pos) }`
2. `right_click_move_command` reads CursorTarget:
   - Line 30: `is_right_click = true`
   - Line 68: `command_type = Default` (AgentMenu doesn't produce an awaiting_command_type)
   - Line 89: `has_selected_agents = true` (ObjectEnum::SyndicateAgent)
   - Line 93: enters entity-click block (entity is Some)
   - Line 96-100: should_attack = false (NeutralObject, not EnemyObject)
   - Lines 123-341: DropOff/Enter/PickUpSupplies/AttachToTower/DropOffSupplies/Chopper handlers — all skipped
   - Line 343: `is_right_click && Default && has_selected_agents` → TRUE
   - Line 344-368: `target_info.get(crystal)` → `crystal_opt.is_some()` → issues `UnitCommand::Gather(crystal)` + inserts `GatheringResourceBehavior`

**If cursor target detection fails** (entity=None): the entity-click block is skipped entirely, falls through to ground-click handler at line 505, which issues a Move command to the ground position. The Agent walks to the crystal's location but doesn't gather — looks like "nothing happens."

### Existing Patterns to Follow

- **DropOff handler** (lines 122-151): Good template for entity-targeting commands. Issues command + inserts behavior + resets to AgentMenu(AgentDefault).
- **Chopper smart right-click** (lines 270-341): Shows pattern for right-click auto-detection of entity types (SDS → PickUp, SupplyTower → DropOff/Attach).
- **Debug logging**: Both `update_cursor_target` (line 82-89) and `right_click_move_command` (line 39-47) already log click details. Use these to diagnose the issue in-game.

### Diagnosis Strategy

1. Run `cargo run`, select an Agent, right-click on a crystal patch
2. Check console for `CursorTarget click debug:` line — does it show `entity=Some(...)` or `entity=None`?
3. If `entity=None` and `targets_checked > 0`: AABB miss — increase `SelectionBounds` for crystal patches in `resources.rs` (e.g., `from_dimensions(1.0, 1.0, 1.0)` or larger)
4. If `entity=None` and `targets_checked=0`: crystal patches aren't matching the query — check component bundles
5. If `entity=Some(...)`: check `right_click_move_command` debug line — is `selected_count > 0`? Is `cursor_kind=NeutralObject`? Trace through early returns.

### Also Note: Missing Left-Click Gather Handler

The `right_click_move_command` system has NO handler for `CommandType::Gather` left-click on entities. When the user presses G (enters AwaitingTarget(Gather) mode) and left-clicks a crystal/SDS, the entity-click section falls through without matching any handler, then the ground handler at line 717 resets the mode. This is a secondary bug — the primary task is fixing right-click, but if the fix is straightforward, consider adding a left-click Gather entity handler in the entity-click section (between lines 341-343) following the same pattern as the right-click handler but checking `command_type == CommandType::Gather` instead.

## Dependencies

None — this is a standalone bugfix. The command dispatch system, cursor target system, and resource gathering behavior system are all already implemented and tested. The fix should be isolated to either the cursor target detection (SelectionBounds sizing) or the command dispatch entity-click handling.
