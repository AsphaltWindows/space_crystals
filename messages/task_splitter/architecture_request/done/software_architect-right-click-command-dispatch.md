# right-click-command-dispatch

## Metadata
- **From**: software_architect
- **To**: task_splitter

## Content

## Architecture Document

artifacts/software_architect/architecture/right-click-command-dispatch-refactor.md

## Summary

Refactor the monolithic `right_click_move_command` function in `src/game/units/systems/core.rs` (lines 243-830+, ~600 lines of logic) into a dispatch pattern with extracted per-command-type handler functions, a shared CommandContext struct, and a deduplicated command issuance helper. This is the highest-priority refactor identified in the codebase review (forum topic 2026-03-23T12-10-00Z, closed with unanimous agreement from developer, task_planner, task_splitter, designer, and automatic_qa).

**Why**: This single function handles ALL right-click/command-mode interactions for every unit type. The task_planner confirms nearly every combat/command task touches it, creating cross-cutting dependencies. Its 12-parameter signature, 6-Option target_info query, and 10+ duplicated command issuance patterns make each new feature riskier and harder to implement.

## Scope

**Primary file**: `src/game/units/systems/core.rs` — extract handler functions from `right_click_move_command`
**Secondary file**: `src/game/units/systems/commands.rs` (new or extend existing) — destination for extracted handlers

**NOT affected**: UI, combat systems, world systems, simulation, or any other module. The external behavior is unchanged — same inputs produce same outputs. No new Bevy systems are added; extracted functions are plain Rust fns called from within the existing system.

### Phases (all in scope for this request):

1. **Extract command type handlers** — move each command-type branch (Attack on entity, DropOff, Enter, PickUpSupplies, AttachToTower, DropOffSupplies, chopper right-click, agent right-click, recruit right-click, guard right-click, ground Move, Patrol, Attack-ground, HoldPosition, Stop) into its own function. Main function becomes a dispatcher under 100 lines.

2. **CommandContext struct** — bundle shared read-only parameters (shift_held, local_player_number, command_type, cursor info) into a struct to reduce parameter passing.

3. **Deduplicate command issuance** — extract the repeated check-interruptible/clear-state/issue-command pattern (appears 10+ times) into a `try_issue_command` helper function.

4. **TargetKind enum** (optional, lower priority) — replace the 6-Option target_info query tuple with a computed TargetKind enum. May be deferred to a follow-up if phases 1-3 sufficiently reduce complexity.

## QA Instructions

1. Run `cargo check` from `artifacts/developer/` — must compile with zero errors
2. Run `cargo test --lib` from `artifacts/developer/` — all 1886+ unit tests must pass
3. Run `cargo test --test qa --no-run` — integration tests must compile
4. Verify `right_click_move_command` function body is under 100 lines (dispatch logic only)
5. Verify the duplicated check-interruptible/clear-state/issue-command pattern appears at most once (in a helper)
6. Verify no extracted handler function exceeds 60 lines
7. Verify behavioral equivalence: the refactored code handles the same command types and unit types as before — no command paths were lost or changed
8. Verify all right-click commands work: Move, Attack (entity and ground), Patrol, HoldPosition, Stop, chopper PickUp/DropOff/Attach, agent Gather/Enter/DropOff, recruit EnterArmory, guard Enter tunnel

## Testing Requirements

### Unit tests for extracted handlers (new tests):
- `test_handle_attack_entity_command`: Selected unit with Attack command type, cursor on enemy entity -> AttackTarget command issued
- `test_handle_move_ground_command`: Selected unit, right-click ground -> Move command issued with pathfinding
- `test_handle_chopper_pickup_command`: Supply chopper with PickUpSupplies command on SDS -> PickUpSupplies command issued
- `test_handle_chopper_dropoff_supplies`: Supply chopper with carried supplies, right-click own SupplyTower -> DropOffSupplies issued
- `test_handle_chopper_attach_to_tower`: Supply chopper empty, right-click own SupplyTower -> AttachToTower issued
- `test_handle_agent_gather_crystal`: SyndicateAgent, right-click crystal patch -> Gather command issued
- `test_handle_agent_enter_tunnel`: SyndicateAgent not carrying, right-click own tunnel -> Enter command issued
- `test_handle_agent_dropoff_tunnel`: SyndicateAgent carrying resources, right-click own tunnel -> DropOffResources issued
- `test_handle_recruit_enter_armory`: CultsRecruit, right-click own armory with capacity -> EnterArmory issued
- `test_handle_guard_enter_tunnel`: SyndicateGuard, right-click own tunnel -> Enter command issued

### Unit tests for try_issue_command helper (new tests):
- `test_try_issue_command_interruptible`: Unit with interruptible attack state -> command issued, returns true
- `test_try_issue_command_not_interruptible`: Unit with non-interruptible attack state -> command NOT issued, returns false
- `test_try_issue_command_shift_queue`: shift_held=true -> command queued, movement state NOT cleared
- `test_try_issue_command_no_shift`: shift_held=false -> movement state cleared, attack state reset

### Regression tests:
- All existing unit tests in `core.rs` module (`cargo test --lib`) must pass unchanged
- All existing integration tests (`cargo test --test qa`) must compile and pass
