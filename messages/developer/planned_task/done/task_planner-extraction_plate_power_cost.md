# extraction_plate_power_cost

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-dc_builds_extraction_facility.md

## Task

Add PowerValue(-3) component to the Extraction Plate entity so each plate draws 3 power from the GDO power grid.

### Changes needed:

1. **game/utils.rs** - `spawn_extraction_plate()` (around line 375): Add `PowerValue(-3)` to the entity's component bundle. This is a single line addition alongside the existing components (ObjectInstance, StructureInstance, Owner, etc.).

2. **Add a test**: Verify that a spawned ExtractionPlate entity has `PowerValue(-3)`. Also verify power grid integration: spawn a PowerPlant (PowerValue(20)) and multiple ExtractionPlates, then run `compute_power_grid` and assert the power totals reflect the plates' drain (e.g., 1 PP + 3 EPs = 20 - 9 = 11 net power).

### Context:
- `PowerValue` is already imported in utils.rs (used by PowerPlant spawn with `PowerValue(20)`).
- `compute_power_grid` in faction.rs queries all entities with `(Owner, PowerValue, ObjectInstance)` and sums generated/consumed power per player — no changes needed there; the system will automatically pick up the new component.
- Negative PowerValue means consumption; positive means generation. ExtractionPlate uses -3 per the design doc.

## Technical Context

### Files to modify

1. **`artifacts/developer/src/game/types/structures.rs`** (line ~423):
   - Add `pub const EP_POWER: i32 = -3;` to the `gdo_structure_stats` module, right after the existing EP constants (`EP_MAX_HP`, `EP_POINT_ARMOR`, `EP_FULL_ARMOR`, `EP_BUILD_RADIUS` at lines 420-423).
   - **Update existing test** at line 812: `extraction_plate_has_no_power_cost` currently asserts plates DON'T consume power. Rename to `extraction_plate_power_cost` and change to assert `EP_POWER == -3`.
   - Add a new test asserting `PowerValue(EP_POWER).0 < 0` (consumer pattern, matching `power_value_consumer_is_negative` test at line 781).

2. **`artifacts/developer/src/game/utils.rs`** (line ~375-392):
   - In `spawn_extraction_plate()`, add `PowerValue(EP_POWER),` to the component bundle at line 375. Insert it after the existing components (e.g., after `ExtractionPlateState` or `BuildRadiusExtension`).
   - `PowerValue` is already available via `use super::types::*;` (line 4) — it's defined in structures.rs line 43.
   - `EP_POWER` will be available via `use super::types::gdo_structure_stats::*;` (line 6).

3. **`artifacts/developer/src/game/world/faction.rs`** (test module at line 2072):
   - Add a power grid integration test. Pattern to follow:
     - Use `TestApp::new()` to create a headless app (sets up GDO faction with `FactionPlugin` registered)
     - Use `TestHarness::spawn_structure_at_grid(ObjectEnum::PowerPlant, ...)` for the power plant
     - Use `TestHarness::spawn_extraction_plate_at_grid(grid_x, grid_z, owner, patch_entity)` for plates — note this requires an `attached_patch` entity (spawn a dummy or use `harness.spawn_resource()`)
     - Run `compute_power_grid` system via `app.world_mut().run_system_once(compute_power_grid)`
     - Assert `GdoPlayerResources.power_generated == 20` and `power_consumed == 3 * num_plates`

### Key types and patterns

- **`PowerValue(pub i32)`** — Component at structures.rs:43. Positive = generation, negative = consumption.
- **`compute_power_grid`** — faction.rs:232. Queries `(&Owner, &PowerValue, &ObjectInstance)` and sums per player. No changes needed — it will automatically pick up `PowerValue` on ExtractionPlates.
- **Constant naming convention**: `{PREFIX}_POWER` — see `DC_POWER`, `PP_POWER`, `BK_POWER`, `EF_POWER`, `ST_POWER` in gdo_structure_stats module (structures.rs:390-438).
- **Test harness** (`shared/testing/harness.rs`): `spawn_extraction_plate_at_grid()` at line 201 delegates to `spawn_extraction_plate()` — will automatically include the new PowerValue.
- **TestApp** (`shared/testing/test_app.rs`): `TestApp::new()` creates a GDO faction app with `FactionPlugin` registered (line 95), which includes `compute_power_grid` in its systems.

### Integration notes

- `compute_power_grid` runs every tick and will automatically detect the new `PowerValue` on ExtractionPlate entities — no system registration changes needed.
- The HUD info panel query (`ui/hud.rs`:227) already includes `Option<&PowerValue>` — ExtractionPlates selected in the HUD will now show their power cost.

## Dependencies

- **extraction_plate_power_slowdown** (sibling task): That task applies `power_ratio` slowdown to plate mining. It depends on this task because plates must first draw power for power_ratio to be affected. However, both tasks modify independent code paths (spawn vs mining system), so they can be implemented in parallel.
