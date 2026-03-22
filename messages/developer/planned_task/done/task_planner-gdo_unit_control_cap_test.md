# gdo_unit_control_cap_test

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-factions_resources_r1.md

## Task

Add an integration test verifying that GDO unit production is blocked when the Unit Control cap (200) would be exceeded. In-game QA of this behavior is currently blocked because the Extraction Facility is not buildable (tracked in a separate forum topic), so this test provides verification of the cap enforcement logic.

### Changes needed:

1. **Add test in tests/ or as a unit test in faction.rs**: Create a test that:
   - Sets up a GDO player with GdoPlayerResources (unit_control_used near cap)
   - Verifies that has_unit_control() returns false when cost would exceed cap
   - Verifies production systems (barracks_production_tick_system or HQ equivalent) respect the cap check
   - Specifically: set unit_control_used=199, verify has_unit_control(1) is true, set unit_control_used=200, verify has_unit_control(1) is false

2. **Verify production gating**: The production gating code already exists:
   - command_panel.rs ~line 1205: checks has_unit_control(control_cost) before allowing train action
   - faction.rs ~line 927: barracks_production_tick_system checks has_unit_control before spawning
   - The test should confirm these paths correctly block when at cap

3. **Also verify unit_control_used increments on spawn**: faction.rs ~line 273 increments unit_control_used when HQ produces a unit. Verify this accounting is correct.

### Verification:
- cargo test should pass with the new test(s)
- Tests should cover: cap boundary (199->200 transition), at-cap blocking, over-cap blocking, cost-0 units (like SupplyChopper) always allowed

## Technical Context

### Existing Test Coverage (IMPORTANT — Read Before Writing)

Significant test coverage already exists for unit control cap logic:

**Unit tests in `artifacts/developer/src/game/types/factions.rs`** (lines 264-496):
- `gdo_unit_control_cap_enforcement` — tests has_unit_control(1), has_unit_control(200), !has_unit_control(201) on default resources
- `gdo_unit_control_with_existing_usage` — tests 199/200 boundary: has_unit_control(1) true, has_unit_control(2) false
- `gdo_over_cap_blocks_new_units` — tests cap < used scenario
- `gdo_unit_death_frees_cap_space` — tests saturating_sub + re-checking cap
- `gdo_saturating_sub_prevents_underflow` — underflow safety

**Unit tests in `artifacts/developer/src/game/types/objects.rs`** (lines 1124-1154):
- `test_peacekeeper_unit_control_cost` — asserts cost == 1
- `test_syndicate_agent_unit_control_cost` — asserts cost == 1
- `test_syndicate_guard_unit_control_cost` — asserts cost == 1
- `test_structure_unit_control_cost_is_zero` — DC, PP, Barracks == 0
- `test_resource_unit_control_cost_is_zero` — SCP, SDS == 0
- `supply_chopper_unit_control_cost_zero` (structures.rs:2052) — asserts SupplyChopper == 0

**Integration tests in `artifacts/developer/tests/qa/unit_cap_systems.rs`**:
- `step_1_gdo_unit_control_cap` — TestApp-based: verifies 199->can build 1, 200->blocked, death->unblocked
- `step_5_gdo_death_decrements_cap` — TestApp-based: spawns peacekeeper, kills it, verifies unit_control_used decremented
- `cap_constants_correct` — GDO_UNIT_CONTROL_CAP==200, SYNDICATE_MAX_TUNNEL_SPACE==200

**Since many tests already exist, the developer should focus on what's NOT yet tested:**
1. **Production system gating** — verify `barracks_production_tick_system` does NOT spawn a unit when at cap. Note: the cap check currently happens at **queue time** (command_panel.rs:1206, faction.rs:965), not at spawn time in `barracks_production_tick_system` (faction.rs:286-334). The tick system spawns unconditionally once progress completes and increments `unit_control_used` (line 308-314). Consider whether a test should verify queueing is blocked, or whether a spawn-time check is needed.
2. **SupplyChopper cost-0 always allowed** — `ObjectEnum::SupplyChopper.unit_control_cost() == 0` (objects.rs:381). Test that has_unit_control(0) returns true even when at cap.
3. **unit_control_used increment on production** — verify that after `barracks_production_tick_system` completes a build, `unit_control_used` is properly incremented.

### Key Types and Functions

- **`GdoPlayerResources`** (factions.rs:85): Component with `unit_control_used: u32`, `unit_control_cap: u32`, `space_crystals: i32`
- **`GdoPlayerResources::has_unit_control(cost: u32) -> bool`** (factions.rs:133): `self.unit_control_used + cost <= self.unit_control_cap`
- **`GDO_UNIT_CONTROL_CAP`** (factions.rs:76): constant = 200
- **`PEACEKEEPER_CONTROL_COST`** (unit_data.rs:145): constant = 1
- **`ObjectEnum::unit_control_cost()`** (objects.rs:375): maps Peacekeeper->1, SyndicateAgent->1, SyndicateGuard->1, SupplyChopper->0, all others->0
- **`BarracksState::production_cost()`**: returns cost struct with `space_crystals` and `build_frames`
- **`BarracksState::try_queue(unit_type)`**: adds to build_queue if not full

### Production System Architecture

Two cap-check sites exist for GDO barracks production:
1. **Queue-time check** (command_panel.rs:1205-1210 and faction.rs:965): checks `has_unit_control()` before allowing a unit into the build queue. If at cap, the unit is never queued.
2. **Spawn-time**: `barracks_production_tick_system` (faction.rs:266-335) does NOT check `has_unit_control()` — it spawns unconditionally when progress completes and increments `unit_control_used` (line 308-314). This means if a unit was queued when there was space but cap was reached before completion, the unit still spawns and can push `unit_control_used` above `unit_control_cap`.

### Files to Modify

- **`artifacts/developer/tests/qa/unit_cap_systems.rs`** — add new test functions here (file already exists and is registered in main.rs line 16)
- Alternatively, add unit tests in `artifacts/developer/src/game/types/factions.rs` test module (line ~240+) for pure has_unit_control logic
- No need to modify `tests/qa/main.rs` — `unit_cap_systems` module is already registered

### Test Patterns to Follow

Follow existing patterns in `tests/qa/unit_cap_systems.rs`:
- Use `TestApp::new()` + `test_app.step()` for integration tests
- Use `TestHarness::new(&mut test_app.app)` + `harness.spawn_unit_at_grid()` for spawning units
- Query `(&Player, &mut GdoPlayerResources)` to manipulate resources directly
- Use `crate::helpers::*` for shared imports
- For pure logic tests, create `GdoPlayerResources::default()` directly without TestApp

### Barracks Production Integration Test Setup

To test barracks production gating, you need:
1. A TestApp with a Barracks entity that has `BarracksState` with an item in the build queue
2. Set `GdoPlayerResources.unit_control_used` to 200
3. Run `test_app.step()` to tick `barracks_production_tick_system`
4. Note: the barracks tick system requires: `Commands`, `Assets<Mesh>`, `Assets<StandardMaterial>`, `Query<(Entity, &Owner, &GridPosition, &mut BarracksState, &StructureInstance)>`, `Query<(&Player, &mut GdoPlayerResources)>`, tiles/grid/rally/occupancy queries
5. TestApp should already provide all of these via the game plugin

## Dependencies

- **Existing test infrastructure**: `tests/qa/unit_cap_systems.rs` already exists with 3 tests — new tests should be added to this file.
- **TestApp/TestHarness**: Available via `space_crystals::testing::TestApp` and `space_crystals::testing::TestHarness` (helpers.rs:4-5).
- No dependencies on other planned_tasks — this is a standalone testing task.
