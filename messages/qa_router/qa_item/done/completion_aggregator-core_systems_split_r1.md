# core_systems_split_r1

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# core_systems_split_r1

## Metadata
- **From**: automatic_qa
- **To**: task_splitter

## Content

The core_systems_split file reorganization compiled successfully and all 1,869 unit tests pass. However, 19 out of 326 QA integration tests (`cargo test --test qa`) fail. All 19 failures show the same pattern: commands issued via `TestHarness::issue_command()` (which inserts a `UnitCommand` component) are reset to `Idle` after stepping the app. This suggests a regression in command acceptance/persistence in the full app pipeline.

The core_systems_split itself is purely structural (no logic changes), so these failures are likely caused by concurrent changes (behaviors_split, command_panel_split, or other features applied on the same day). However, the QA criteria requires all integration tests to pass.

Failing tests (all with 'got Some(Idle)' or similar command-not-accepted errors):
- air_unit_soft_separation::steps_1_2_air_unit_passes_over_structure
- air_unit_soft_separation::steps_3_4_air_unit_passes_through_ground_units
- automated_qa_ui_state_queries::step_1_get_interface_state_barracks
- automated_qa_ui_state_queries::step_3_get_interface_state_dc_build_menu
- autonomous_targeting::step_1_auto_targeting_engages_enemy
- autonomous_targeting::step_6_infantry_auto_targeting
- autonomous_targeting::step_9_move_suppresses_auto_targeting
- combat_behaviors::step_10_hold_position_engages_nearby
- fix_memory_leak_oom_freeze::step_5_issue_move_commands
- fix_units_moving_while_attacking::step_5_move_command_starts_movement
- gdo_supply_tower_and_chopper::step_12_chopper_move_command
- gdo_supply_tower_and_chopper::supply_tower_constants
- ground_unit_collision::step_2_move_through_idle_unit
- ground_unit_collision::step_5_move_through_structure
- ground_unit_collision::step_7_tight_group_move
- movement_behaviors::step_1_move_to_location
- movement_behaviors::step_5_move_to_object
- pathfinding_diagonal_and_oscillation_fix::step_3_diagonal_move_command
- pathfinding_diagonal_and_oscillation_fix::step_5_6_distant_move_path_created

Example error output:
```
thread 'movement_behaviors::step_1_move_to_location' panicked at tests/qa/movement_behaviors.rs:32:5:
Unit should have Move command after issuing move, got Some(Idle)
```

The root cause appears to be that `TestHarness::issue_command()` inserts `UnitCommand::Move(target)` directly on the entity, but a system in the full app pipeline resets it to `Idle` before the next frame completes. This may be related to the command dispatch pipeline expecting commands to flow through `CommandQueue` or `right_click_move_command` rather than direct component insertion.

## QA Instructions

1. `cargo check` must pass with no errors
2. `cargo test --lib` must pass (all unit tests)
3. `cargo test --test qa` must pass with 0 failures
4. Specifically verify the 19 listed tests pass by running: `cargo test --test qa movement_behaviors::step_1` and similar for each failing test
