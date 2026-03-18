# Feature: Automated QA System

## Overview
Infrastructure enabling the QA agent to execute the majority of QA task steps programmatically, without human interaction. Consists of three layers: a Command Interface for game control/query, a QA Step Tagging convention for ticket authoring, and an Automated QA Runner that executes tagged steps and reports results.

## Design Sources
- Forum discussions: `automated_game_testing_facility`, `agent_testing_next_phase`, `fully_automated_qa_pipeline`
- Phase 1 (headless TestApp) is already implemented

## Specifications

### Layer 1: Command Interface

The Command Interface is an ECS-direct API that allows test code to issue game commands and query game state without going through the UI. It runs in the same process as the game (or headless TestApp) and operates on Bevy's `World` directly.

#### Command Categories

**Spawn Commands**
- `spawn_unit(faction, unit_type, position) -> Entity` — spawn a unit at a world position
- `spawn_structure(faction, structure_type, position, rotation, flip_h, flip_v) -> Entity` — spawn a placed structure
- `spawn_resource(resource_type, position, amount) -> Entity` — spawn a resource node

**Unit Commands**
- `issue_command(entity, command_type, target)` — issue a unit command (Move, AttackMove, AttackTarget, Stop, HoldPosition, Patrol, Enter, Gather, BuildTunnel)
- `set_selection(entities)` — set the current selection to a list of entities
- `clear_selection()` — clear selection

**Game State Commands**
- `set_resources(faction, resource_type, amount)` — set a faction's resource count
- `advance_frames(n)` — advance the simulation by N frames
- `set_tile(position, tile_preset)` — set a tile's type at a grid position
- `reveal_map(faction)` — reveal all tiles for a faction (disable fog of war)
- `set_camera(position, zoom)` — move the camera

#### Query Categories

**Entity Queries**
- `get_position(entity) -> Vec3` — get entity world position
- `get_health(entity) -> (current, max)` — get entity HP
- `get_attack_state(entity) -> AttackPhase` — get current attack phase
- `get_behavior(entity) -> BehaviorState` — get current behavior
- `get_command(entity) -> CommandState` — get current command
- `get_movement(entity) -> (velocity, path_target)` — get movement state
- `is_alive(entity) -> bool` — check if entity still exists and has HP > 0

**World Queries**
- `get_visibility(faction, tile_position) -> VisibilityState` — get fog of war state for a tile
- `get_resources(faction, resource_type) -> u32` — get current resource amount
- `get_entities_in_area(position, radius) -> Vec<Entity>` — find entities near a point
- `get_selection() -> Vec<Entity>` — get currently selected entities
- `count_entities(filter) -> usize` — count entities matching a filter (e.g., faction + unit_type)

**Structural Queries**
- `get_tunnel_network(faction) -> TunnelNetworkInfo` — tunnel count, total space, units inside
- `get_structure_state(entity) -> StructureState` — construction progress, operational status

**UI State Queries**

Queries for client-side ECS components (ControlState, CommandPanel, InfoPanel, SelectionPanel). These query UI state derived from game state each tick — callers should ensure UI sync systems have run (e.g., via `advance_frames(1)`) before querying to avoid stale results.

- `get_interface_state() -> ObjectInterfaceState` — current command panel mode (DefaultState, ConstructionSubmenu, AwaitingTarget, etc.)
- `get_visible_commands() -> Vec<(SlotPosition, CommandButtonAction, IsEnabled, IsCommon)>` — all commands currently displayed in the command panel, with grid slot position, enabled/disabled state, and whether the command is common (shared across all selection groups) or group-specific
- `get_active_group() -> Option<SelectionGroup>` — which SelectionGroup's commands are currently displayed in the command panel
- `get_selection_groups() -> Vec<SelectionGroup>` — all SelectionGroups in the current selection, with type and member count
- `get_info_panel() -> InfoPanelContent` — current info panel display data (entity name, health, portrait, stats)
- `get_selection_panel_portraits() -> Vec<(Entity, IsHighlighted)>` — portrait grid contents with ActiveGroup highlighting state

#### Assertion Helpers

Convenience wrappers that combine queries with assertions for cleaner test code:

- `assert_position_near(entity, expected_pos, tolerance)` — entity is within tolerance of expected position
- `assert_health_equals(entity, expected_hp)`
- `assert_phase_equals(entity, expected_phase)`
- `assert_behavior_equals(entity, expected_behavior)`
- `assert_dead(entity)` — entity is dead or despawned
- `assert_visible(faction, tile_position)` — tile is in Visible state
- `assert_resource_at_least(faction, resource_type, min_amount)`
- `assert_selection_count(expected_count)`
- `assert_interface_state(expected_state)` — command panel is in expected ObjectInterfaceState
- `assert_command_visible(slot, command_action)` — specific command is visible in expected grid slot
- `assert_command_not_visible(command_action)` — command is not present in the panel
- `assert_command_enabled(slot)` — command in slot is enabled (not grayed out)
- `assert_active_group_type(expected_object_type)` — ActiveGroup matches expected object type
- `assert_info_panel_shows(entity)` — info panel is displaying data for the specified entity

#### Implementation Approach

- All commands and queries are Rust functions operating on `&mut World` or `&World`
- Exposed as a `TestHarness` struct that wraps a `World` reference
- Tests using the headless `TestApp` (Phase 1) already have `World` access — `TestHarness` is a convenience layer on top
- No network protocol, no IPC — same-process, direct ECS access
- Commands that modify state (spawn, issue_command, advance_frames) take `&mut World`
- Queries that read state take `&World`

#### File Organization

- `src/testing/harness.rs` — `TestHarness` struct with all command/query methods
- `src/testing/assertions.rs` — assertion helper functions
- `src/testing/mod.rs` — module exports (behind `#[cfg(test)]` or a `testing` feature flag)

---

### Layer 2: QA Step Tagging

QA steps in ticket files (`/qa_tasks/*.md`) are tagged to indicate whether they can be executed automatically or require human verification.

#### Tag Format

Each QA step is prefixed with a tag:

- `[auto]` — Fully automatable. The QA agent can execute and verify this step using the Command Interface without human involvement.
- `[human]` — Requires human verification. Visual checks, UX feel, audio, animations, or anything that cannot be verified through ECS state queries.
- `[semi]` — The setup/execution is automatable, but the verification requires human judgment. The automated runner sets up the scenario; the human evaluates the result.

#### Tagging Guidelines (for Project Manager)

A step is `[auto]` when ALL of the following hold:
1. The precondition can be set up via Command Interface (spawn entities, set resources, position units)
2. The action can be triggered via Command Interface (issue commands, advance frames)
3. The expected result can be verified via Query/Assertion (check position, health, phase, behavior, visibility)

A step is `[human]` when ANY of the following apply:
1. Verification requires visual inspection (sprite rendering, animation correctness, UI layout)
2. Verification requires evaluating subjective quality (movement "feels smooth", "no flickering")
3. The step involves real-time interaction patterns (drag-select feel, camera scroll responsiveness)

A step is `[semi]` when:
1. The scenario can be automatically constructed, but the pass/fail judgment is visual or subjective

#### Example (adapted from attack_phases QA task)

```
1. [auto] Verify attack phases execute in order: Aiming -> Firing -> Cooldown -> Reloading.
2. [auto] Verify Aiming phase is interruptible: issuing a move command during Aiming cancels the attack.
3. [auto] Verify Firing phase is not interruptible: issuing a move command during Firing does not cancel the attack.
7. [human] For a TurretSource unit: verify unit base visibly continues moving while turret attacks.
```

#### Retroactive Tagging

Existing QA tasks (35 currently in `/qa_tasks`) should be tagged when the Command Interface is available. This is a one-time pass by the Project Manager, updating each file's QA Steps section with the appropriate tags.

---

### Layer 3: Automated QA Runner

The QA agent's execution mode for processing QA tasks without human interaction.

#### Execution Flow

1. **Pick up a QA task** from `/qa_tasks` (same as current flow)
2. **Scan for step tags**: Parse the QA Steps section for `[auto]`, `[human]`, and `[semi]` tags
3. **Execute `[auto]` steps first**:
   - For each `[auto]` step, the QA agent writes and runs a test function using the Command Interface
   - Each step produces a result: PASS or FAIL (with error details)
4. **Evaluate automated results**:
   - If any `[auto]` step FAILs → the task FAILs. Annotate the task with failure details and return to `/developer_tasks`. Do NOT proceed to human steps.
   - If all `[auto]` steps PASS → proceed to human steps (if any)
5. **Handle `[human]` and `[semi]` steps**:
   - If the task has ONLY `[auto]` steps and all passed → task PASSES. Move to `/completed_tasks`.
   - If the task has `[human]` or `[semi]` steps remaining → move the task to a `/qa_human_review` directory with automated results annotated. These accumulate for batch human QA sessions.

#### Test Function Generation

The QA agent translates each `[auto]` step into a Rust test function:

```rust
#[test]
fn qa_attack_phases_step_1_phase_order() {
    let mut app = TestApp::new();
    let harness = TestHarness::new(&mut app.world);

    // Setup: spawn two opposing units
    let attacker = harness.spawn_unit(Faction::GDO, UnitType::Peacekeeper, Vec3::new(100.0, 0.0, 0.0));
    let target = harness.spawn_unit(Faction::Syndicate, UnitType::Agent, Vec3::new(200.0, 0.0, 0.0));
    harness.reveal_map(Faction::GDO);

    // Action: order attack
    harness.issue_command(attacker, Command::AttackTarget, Some(target));

    // Verify phase sequence
    harness.advance_frames(1);
    harness.assert_phase_equals(attacker, AttackPhase::Aiming);

    // Advance through aim duration
    harness.advance_frames(aim_duration_frames);
    harness.assert_phase_equals(attacker, AttackPhase::Firing);

    harness.advance_frames(fire_duration_frames);
    harness.assert_phase_equals(attacker, AttackPhase::Cooldown);

    harness.advance_frames(cooldown_duration_frames);
    harness.assert_phase_equals(attacker, AttackPhase::Reloading);
}
```

#### Test Execution

- Tests are compiled and run via `cargo test --features testing` (or a dedicated test binary)
- The QA agent invokes this via Bash tool and parses stdout for PASS/FAIL results
- Each test function maps to exactly one QA step
- Test file location: `tests/qa/[task_name].rs`

#### Result Reporting

For each QA task, the QA agent produces a result summary:

```
## Automated QA Results
- Step 1 [auto]: PASS
- Step 2 [auto]: PASS
- Step 3 [auto]: FAIL — expected AttackPhase::Firing, got AttackPhase::Aiming after 16 frames
- Steps 4-6 [auto]: SKIPPED (prior failure)
- Steps 7-10 [human]: DEFERRED to human review
```

#### Failure Handling

When an `[auto]` step fails:
- The task is annotated with the failure details (which step, expected vs actual, relevant entity states)
- The task is returned to `/developer_tasks` with `## QA Failure` annotation (same as current manual failure flow)
- The generated test file is preserved in `tests/qa/` so the developer can reproduce the failure

---

### Pipeline Integration

#### Modified QA Agent Modes

The QA agent operates in two modes:

1. **Automated mode** (non-interactive, scheduled by supervisor): Picks up QA tasks, runs `[auto]` steps, passes/fails/defers as described above. This is the primary mode — runs every supervisor cycle.

2. **Human review mode** (interactive, user-initiated): Walks the user through accumulated `[human]` and `[semi]` steps from `/qa_human_review`. Same as current QA flow but only for the subset that genuinely needs human eyes.

#### Modified Project Manager Behavior

When creating QA tasks, the Project Manager tags each QA step with `[auto]`, `[human]`, or `[semi]` based on the tagging guidelines. This is part of the ticket creation process, not a separate pass.

#### New Directory

- `/qa_human_review` — QA tasks that passed all automated steps but have remaining human-verification steps. Accumulates between human QA sessions.

---

## Automation Coverage Estimate

Based on prior analysis of 35 QA tasks (~230 total steps), updated to reflect UI State Queries:

| Category | Tasks | Estimated Steps | Automation |
|----------|-------|----------------|------------|
| Fully automatable (game state) | ~18 | ~95 | 100% `[auto]` |
| Fully automatable (UI state) | ~6 | ~40 | 100% `[auto]` (requires UI State Queries) |
| Partially automatable | ~9 | ~80 | ~60% `[auto]`, ~40% `[human]`/`[semi]` |
| Visual/UX only | ~2 | ~15 | 100% `[human]` |
| **Total** | **35** | **~230** | **~82% automatable** |

Note: With UI State Queries, ~8 previously partially-automatable UI-focused tasks (command panel states, button inventories, interface state transitions) shift to fully or mostly automatable, raising overall automation from ~72% to ~82%.

## Implementation Priority

1. **Command Interface** (Layer 1) — enables everything else. No dependencies on other features. Estimated 2-3 tickets.
2. **QA Step Tagging** (Layer 2) — Project Manager retroactively tags existing 35 tasks + adopts convention for new tasks. 1 ticket for convention + 1 for retroactive pass.
3. **Automated QA Runner** (Layer 3) — QA agent mode changes + test generation. 2-3 tickets.

## Open Questions

1. **Test persistence**: Should generated test files in `tests/qa/` be kept permanently (growing regression suite) or cleaned up after tasks pass? Keeping them creates a regression suite at the cost of maintenance.
2. **Parallelism**: Can multiple QA tasks' tests run in parallel via `cargo test`, or must they be sequential (shared World state)? Each test creates its own TestApp, so parallelism should be safe.
3. **Flaky test handling**: How many retries before declaring a step as FAIL? Suggest 1 retry for non-deterministic failures, with flakiness flagged in results.

## Dependencies

- **Phase 1 headless TestApp**: Already implemented. The Command Interface builds on top of it.
- **No game feature dependencies**: This is pure infrastructure. Can proceed independently of any game feature work.
