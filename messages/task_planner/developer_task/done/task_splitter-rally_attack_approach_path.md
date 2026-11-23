# rally_attack_approach_path

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-gdo_barracks_r1.md

## Task

Fix the `issue_rally_command` function in `artifacts/developer/src/game/world/faction.rs` (line ~425) so that when a produced unit is issued an AttackTarget command against an enemy rally target, it also receives a MoveTarget and Path for initial approach — matching the pattern used in the right-click attack fix in `right_click_move_command` (core.rs, lines ~355-375).

Currently (line 470), the enemy branch only inserts `UnitCommand::AttackTarget(*target_entity)` without MoveTarget/Path, causing the unit to stop instead of approaching. The friendly branch (line 472-481) correctly computes and inserts a path.

The fix: in the `is_enemy` branch of `issue_rally_command`, after inserting `UnitCommand::AttackTarget`, also compute a path from the spawn position to the target and insert `MoveTarget(target_pos)` and `Path { waypoints: smoothed, current_waypoint: 0 }`.

This function is shared by Barracks (line 404), Headquarters (line 571), and Armory (line 2531), so fixing it once fixes rally-attack for all production structures.

Also add a test verifying that `issue_rally_command` with a RallyTarget::Object pointing to an enemy entity results in both `UnitCommand::AttackTarget` and `MoveTarget` being inserted on the unit.
