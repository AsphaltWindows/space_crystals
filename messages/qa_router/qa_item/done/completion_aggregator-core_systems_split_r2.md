# core_systems_split_r2

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

## Content

The core_systems_split_r1 rework fixed 13 of the 19 originally failing QA integration tests. However, 6 tests still fail. cargo check passes and all 1,871 unit tests pass. The remaining 6 failures fall into 3 categories:

### Category 1: Interface state not updating after selection (2 failures)
- automated_qa_ui_state_queries::step_1_get_interface_state_barracks
- automated_qa_ui_state_queries::step_3_get_interface_state_dc_build_menu

Error: Interface state stays `Default` instead of changing to `StructureMenu(BarracksMenu)` or `StructureMenu(DcBuildMenu)` after selecting a structure in the test harness. The test harness select_entity() call does not trigger the system that updates InterfaceState based on selection.

### Category 2: Auto-targeting / combat engagement not triggering (3 failures)
- autonomous_targeting::step_1_auto_targeting_engages_enemy
- autonomous_targeting::step_6_infantry_auto_targeting
- combat_behaviors::step_10_hold_position_engages_nearby

Error: Idle units near enemies do not auto-engage. AttackPhase is None and AttackTarget is None after stepping. The autonomous targeting system does not appear to run or does not find valid targets in the test setup.

Example: `Idle unit near enemy should auto-engage. Phase: None, target: None`

### Category 3: Constant mismatch (1 failure)
- gdo_supply_tower_and_chopper::supply_tower_constants

Error: `ST_BUILD_FRAMES` is 160 but the test expects 240 (15 seconds at 16fps). Either the constant or the test expectation needs to be updated to match the design spec.

## QA Instructions

1. `cargo check` must pass with no errors
2. `cargo test --lib` must pass (all unit tests)
3. `cargo test --test qa` must pass with 0 failures
4. Specifically verify these 6 tests pass:
   - `cargo test --test qa automated_qa_ui_state_queries::step_1`
   - `cargo test --test qa automated_qa_ui_state_queries::step_3`
   - `cargo test --test qa autonomous_targeting::step_1`
   - `cargo test --test qa autonomous_targeting::step_6`
   - `cargo test --test qa combat_behaviors::step_10`
   - `cargo test --test qa gdo_supply_tower_and_chopper::supply_tower_constants`
