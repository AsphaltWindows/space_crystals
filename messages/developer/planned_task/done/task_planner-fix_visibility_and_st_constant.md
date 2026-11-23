# fix_visibility_and_st_constant

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-core_systems_split_r2.md

## Task

Fix 4 failing QA integration tests: 3 auto-targeting tests that fail because spawned enemies lack VisibilityStateEnum::Visible, and 1 constant mismatch test.

### Category A: Auto-targeting not engaging (3 tests)

**Root cause**: base_auto_target_system (combat/systems/core.rs:474) queries potential_targets with VisibilityStateEnum. The is_valid_target() function (combat/utils.rs:278) requires target_visibility == VisibilityStateEnum::Visible. But spawn_peacekeeper (game/utils.rs:399) does NOT insert VisibilityStateEnum on spawned units. So enemy units in tests have no VisibilityStateEnum component and fail the filter.

**Fix**: Add VisibilityStateEnum::Visible to the spawn bundle in ALL unit spawn functions that are used in tests:
- spawn_peacekeeper (game/utils.rs ~line 451)
- spawn_syndicate_agent (game/utils.rs)
- spawn_syndicate_guard (game/utils.rs)
- spawn_supply_chopper (game/utils.rs)

**Tests to fix**:
- autonomous_targeting::step_1_auto_targeting_engages_enemy
- autonomous_targeting::step_6_infantry_auto_targeting
- combat_behaviors::step_10_hold_position_engages_nearby

### Category B: ST_BUILD_FRAMES constant (1 test)

**Root cause**: ST_BUILD_FRAMES in src/game/types/structures.rs:433 is 160 but should be 240 (15 seconds at 16 FPS). The test expects 240. DeploymentCenterState::construction_cost already returns 240 for SupplyTower.

**Fix**:
1. Change ST_BUILD_FRAMES from 160 to 240 in structures.rs:433
2. Update the unit test at structures.rs:2055 that asserts ST_BUILD_FRAMES == 160 to assert 240
3. Search for any other references to ST_BUILD_FRAMES and verify consistency

**Test to fix**: gdo_supply_tower_and_chopper::supply_tower_constants

### Verification

All 4 tests must pass:
- cargo test --test qa autonomous_targeting::step_1
- cargo test --test qa autonomous_targeting::step_6
- cargo test --test qa combat_behaviors::step_10
- cargo test --test qa gdo_supply_tower_and_chopper::supply_tower_constants

Also verify no regressions:
- cargo test --lib
- cargo test --test qa

## Technical Context

### Files to modify

1. **artifacts/developer/src/game/utils.rs** — 4 spawn functions need VisibilityStateEnum::Visible added:
   - `spawn_peacekeeper` (line 451-492): Add `VisibilityStateEnum::Visible` to the third `.insert()` block (line 481) alongside Armor, Silhouette, SightRange. The final insert block currently ends at line 492 with `.id()`.
   - `spawn_syndicate_agent` (line 546-589): Same pattern — add to the third `.insert()` block (line 578) alongside Armor, Silhouette, SightRange.
   - `spawn_syndicate_guard` (line 643-686): Same pattern — add to the third `.insert()` block (line 675).
   - `spawn_supply_chopper` (line 906-944): Add to the third `.insert()` block (line 932) alongside Armor, Silhouette, SeparationRadius, SightRange.

2. **artifacts/developer/src/game/types/structures.rs** — 2 changes:
   - Line 433: Change `pub const ST_BUILD_FRAMES: u32 = 160;` to `= 240;`
   - Line 2055: Change `assert_eq!(ST_BUILD_FRAMES, 160);` to `assert_eq!(ST_BUILD_FRAMES, 240);`

### No new imports needed

`VisibilityStateEnum` is defined in `artifacts/developer/src/shared/types.rs:435` and is already available in `game/utils.rs` via the glob import `use crate::types::*;` on line 2.

### Pattern to follow

Each spawn function uses chained `.insert()` calls. The third insert block contains armor/silhouette/sight components. Add `VisibilityStateEnum::Visible` as an additional component in that block. Example for spawn_peacekeeper — the block at line 481 currently reads:

```rust
    )).insert((
        Armor { ... },
        Silhouette { ... },
        SightRange(ObjectEnum::Peacekeeper.object_type().sight_range),
    )).id()
```

Should become:

```rust
    )).insert((
        Armor { ... },
        Silhouette { ... },
        SightRange(ObjectEnum::Peacekeeper.object_type().sight_range),
        VisibilityStateEnum::Visible,
    )).id()
```

### Why this is the correct fix

The `base_auto_target_system` (combat/systems/core.rs:480) queries potential targets with `&VisibilityStateEnum` as a **required** query component — entities without it are excluded entirely from the query results. The `is_valid_target` function (combat/utils.rs:278-296) then checks `*target_visibility != VisibilityStateEnum::Visible`. Both conditions must be met: the component must exist AND be set to Visible. Setting it to Visible at spawn time is correct gameplay behavior — the fog-of-war system updates visibility each frame based on FogOfWarMap.

### Other spawn functions

Structure spawn functions (spawn_deployment_center, spawn_tunnel, etc.) may also lack VisibilityStateEnum. However, the failing tests only involve units, so fix the 4 unit spawn functions listed above. If structure targeting tests fail later, the same pattern applies.

## Dependencies

None. This is a standalone bug fix with no dependencies on other planned tasks.
