# harden_assertions_and_cleanup

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-test_suite_cleanup.md

## Task

Harden 33 soft assertions across 11 integration test files, remove dead test code, fix deprecated API usage, and add proper ignore annotations.

### Soft Assertion Hardening (33 sites across 11 files)

Replace all `if let Some(component) = world.get::<T>(entity)` assertion guards with `let component = world.get::<T>(entity).expect("Entity should have T component");` or `.unwrap()`. This ensures tests fail loudly when a component is missing instead of silently passing.

File-by-file count:
- fix_units_moving_while_attacking.rs: 7 instances
- tunnel_object_interface_state.rs: 6 instances
- gdo_supply_tower_and_chopper.rs: 4 instances
- combat_behaviors.rs: 4 instances
- autonomous_targeting.rs: 3 instances
- construction_hp_rule.rs: 2 instances
- tunnel_area_and_construction_rules.rs: 2 instances
- movement_behaviors.rs: 2 instances
- attack_phases.rs: 1 instance
- agent_resource_gathering_commands_and_behaviors.rs: 1 instance
- tunnel_expansions_and_starting_condition.rs: 1 instance

After hardening, run the full integration suite. If any newly-hardened assertions fail, that reveals a real bug -- fix the test setup or report the issue (do not revert to soft assertions).

### Dead Code Removal

1. Remove `create_dropoff_test_world` function at src/game/units/systems/core.rs:3806 (unused test helper)
2. Remove `create_right_click_entity_world` function at src/game/units/systems/core.rs:4080 (unused test helper)
3. Remove unused imports across integration test files (check for compiler warnings and fix)
4. Replace deprecated `world.iter_entities()` with appropriate query in tests/qa/fix_memory_leak_oom_freeze.rs:14

### Ignore Annotations

Add proper reason strings to all ignored tests:
- The 6 escape-key tests: change `#[ignore]` to `#[ignore = "Escape key not processed in headless mode"]` if not already annotated
- step_8_attack_ground_command: add `#[ignore = "reason"]` with appropriate reason
- step_9_reverse_command: add `#[ignore = "reason"]` with appropriate reason

### Files
- tests/qa/ -- 11 files for soft assertion hardening (listed above)
- tests/qa/fix_memory_leak_oom_freeze.rs -- deprecated API
- src/game/units/systems/core.rs -- remove 2 dead test helpers
- tests/qa/basic_combat_unit_interface_state.rs -- ignore annotations (if unannotated ignores exist here)
- tests/qa/tunnel_object_interface_state.rs -- ignore annotations (if escape-key ignores exist here)
- tests/qa/agent_object_interface_state.rs -- ignore annotations (if escape-key ignores exist here)

### Verification
- cargo test --test qa -- full suite passes with no new failures
- cargo test --lib -- no regressions, no dead code warnings from removed helpers
- cargo test --test qa 2>&1 | grep warning -- warning count should decrease

## Technical Context

### Soft Assertion Sites -- Exact Locations and Patterns

All files are under `artifacts/developer/tests/qa/`. There are three distinct patterns to harden:

**Pattern A -- `world.get::<T>()` read-only assertion guards (11 sites):**
These wrap assertions inside `if let Some`. Convert to `.expect("...")` and inline the assertion.

| File | Line | Component | Context |
|------|------|-----------|---------|
| fix_units_moving_while_attacking.rs | 89 | `Velocity` | Assertion: speed < 0.01 while attacking |
| fix_units_moving_while_attacking.rs | 318 | `AttackState` | Assertion: has target or in attack phase |
| fix_units_moving_while_attacking.rs | 348 | `AttackState` | Assertion: auto-targeting engaged |
| fix_units_moving_while_attacking.rs | 351 | `Velocity` | Nested inside line 348 guard |
| combat_behaviors.rs | 30 | `AttackState` | Assertion: has target after attack command |
| combat_behaviors.rs | 296 | `AttackState` | Assertion: target cleared after enemy dies |
| autonomous_targeting.rs | 30 | `AttackState` | Assertion: auto-acquired target |
| autonomous_targeting.rs | 62 | `AttackState` | Assertion: leash range target |
| attack_phases.rs | 151 | `AttackState` | Assertion: attack phase progression |
| movement_behaviors.rs | 93 | `Velocity` | Assertion: speed after move command |
| movement_behaviors.rs | 164 | `Velocity` | Assertion: speed after patrol |

Example transformation for Pattern A:
```rust
// BEFORE (line 89):
if let Some(vel) = world.get::<Velocity>(player_unit) {
    let speed = vel.0.length();
    assert!(speed < 0.01, "...");
}
// AFTER:
let vel = world.get::<Velocity>(player_unit)
    .expect("Player unit should have Velocity component");
let speed = vel.0.length();
assert!(speed < 0.01, "...");
```

**Pattern B -- `world.get_mut::<T>()` test-setup guards (10 sites):**
These silently skip test setup if the component is missing. Convert to `.expect("...")`.

| File | Line | Component | Context |
|------|------|-----------|---------|
| fix_units_moving_while_attacking.rs | 79 | `AttackState` (mut) | Setup: set attack target+phase |
| fix_units_moving_while_attacking.rs | 117 | `AttackState` (mut) | Setup: set attack target+phase |
| fix_units_moving_while_attacking.rs | 236 | `AttackState` (mut) | Setup: set attack target+phase |
| combat_behaviors.rs | 63 | `Transform` (mut) | Setup: move enemy far away |
| combat_behaviors.rs | 101 | `ObjectInstance` (mut) | Setup: kill enemy |
| autonomous_targeting.rs | 93 | `Transform` (mut) | Setup: move enemy for leash test |
| gdo_supply_tower_and_chopper.rs | 281 | `SupplyChopperState` (mut) | Setup: reset chopper state |
| gdo_supply_tower_and_chopper.rs | 284 | `SupplyTowerState` (mut) | Setup: reset tower state |
| gdo_supply_tower_and_chopper.rs | 365 | `SupplyChopperState` (mut) | Setup: reset chopper state |
| gdo_supply_tower_and_chopper.rs | 368 | `SupplyTowerState` (mut) | Setup: reset tower state |

Example transformation for Pattern B:
```rust
// BEFORE (line 79):
if let Some(mut attack_state) = world.get_mut::<AttackState>(player_unit) {
    attack_state.current_target = Some(AttackTarget::UnitTarget(enemy_unit));
}
// AFTER:
let mut attack_state = world.get_mut::<AttackState>(player_unit)
    .expect("Player unit should have AttackState component");
attack_state.current_target = Some(AttackTarget::UnitTarget(enemy_unit));
```

**Pattern C -- Enum variant matching on local variables (12 sites):**
These use `if let Some(EnumVariant { .. })` on fields of already-owned structs (not `world.get`). The approach differs:

| File | Line | Pattern | Context |
|------|------|---------|---------|
| tunnel_object_interface_state.rs | 206 | `TunnelOperation::BuildingExpansion` mut | Setup: set progress to 1.0 |
| tunnel_object_interface_state.rs | 209 | `TunnelOperation::BuildingExpansion` ref | Assert: progress < required |
| tunnel_object_interface_state.rs | 214 | same, mut | Setup: set progress to 399.0 |
| tunnel_object_interface_state.rs | 217 | same, ref | Assert: progress < required |
| tunnel_object_interface_state.rs | 222 | same, mut | Setup: set progress to 400.0 |
| tunnel_object_interface_state.rs | 225 | same, ref | Assert: progress >= required |
| tunnel_area_and_construction_rules.rs | 256 | same, mut | Setup: set progress to 100.0 |
| tunnel_area_and_construction_rules.rs | 264 | same, ref | Assert: progress == 100.0 (already has else panic!) |
| construction_hp_rule.rs | 21 | `obj.max_hp` (Option field) | Logic: uses max_hp and hp for HP fraction calculation |
| construction_hp_rule.rs | 23 | `obj.hp` (Option field) | Nested inside max_hp guard |
| agent_resource_gathering_commands_and_behaviors.rs | 49 | `GatheringResourceBehavior` | Assert: no longer in Extracting phase |
| tunnel_expansions_and_starting_condition.rs | 413 | `Visibility` | Assert: agent should be visible |

For Pattern C enum variants, use `match` with explicit `panic!` on the else arm, or use `assert!(matches!(...))` + destructure. For `tunnel_area_and_construction_rules.rs:264`, note it ALREADY has an `else { panic!(...) }` arm -- that one just needs the setup counterpart at line 256 hardened.

For `construction_hp_rule.rs:21-23`, these are accessing Option fields (`obj.max_hp`, `obj.hp`) within a system closure iterating query results -- these are game logic, not pure assertions. Hardening means calling `.expect("ObjectInstance should have max_hp set")` instead of `if let Some`.

For `agent_resource_gathering_commands_and_behaviors.rs:49`, the `if let Some(g) = gathering` guard allows the test to pass even if the behavior component was removed. Add `.expect("Agent should still have GatheringResourceBehavior")` or decide if removal is a valid outcome (the comment says it is -- investigate and choose).

For `tunnel_expansions_and_starting_condition.rs:413`, `world.get::<Visibility>` is followed by `if let Some(vis) = visibility`. Harden to `.expect(...)`.

### Dead Code Removal -- Exact Spans

1. **`create_dropoff_test_world`** -- `artifacts/developer/src/game/units/systems/core.rs` lines 3806-3823 (18 lines). Defined but never called (confirmed: only 1 match in file = the definition). Remove entirely.

2. **`create_right_click_entity_world`** -- same file, lines 4080-4097 (18 lines). Defined but never called (confirmed: only 1 match = the definition). Remove entirely. Note: line numbers will shift after removing the first function; remove both in one pass or adjust.

3. **Unused imports** -- run `cargo test --test qa 2>&1 | grep "unused import"` to find all. Fix each.

4. **Deprecated `iter_entities()`** -- `artifacts/developer/tests/qa/fix_memory_leak_oom_freeze.rs` line 14:
```rust
// BEFORE:
for obj in world.iter_entities().filter_map(|e: EntityRef| world.get::<ObjectInstance>(e.id())) {
// AFTER (use world.query):
let mut query = world.query::<&ObjectInstance>();
for obj in query.iter(world) {
```
The existing pattern `world.query::<&T>()` is used extensively in `tests/qa/unit_cap_systems.rs` (10+ instances). Follow that convention.

### Ignore Annotations -- Exact Locations

**Already annotated with reason (no change needed):**
- `agent_object_interface_state.rs:86` -- `#[ignore] // Escape key not processed by command_panel_hotkeys in headless mode (HudPlugin not loaded)`
- `tunnel_object_interface_state.rs:256,281,422,485,524` -- all have `// Escape key...` comments

**Problem:** These use `#[ignore]` with a trailing comment, not `#[ignore = "reason"]`. The Rust convention for machine-readable ignore reasons is the `= "reason"` syntax. All 6 of these (plus the one in agent_object_interface_state.rs) should be converted to `#[ignore = "Escape key not processed in headless mode"]`.

**Need annotation added:**
- `basic_combat_unit_interface_state.rs:245` -- `#[ignore]` with NO reason. This is `step_8_attack_ground_command`. Add: `#[ignore = "No unit type with CanTargetGround capability currently exists"]`
- `basic_combat_unit_interface_state.rs:255` -- `#[ignore]` with NO reason. This is `step_9_reverse_command`. Add: `#[ignore = "No unit type with CanReverse capability currently exists"]`

### Bevy 0.17 Notes

- `World::get::<T>(entity)` returns `Option<&T>` -- safe to call `.expect()` on
- `World::get_mut::<T>(entity)` returns `Option<Mut<T>>` -- safe to call `.expect()` on
- `world.query::<&T>()` returns `QueryState` -- call `.iter(world)` to iterate; this is the replacement for `iter_entities().filter_map(...)`
- `EntityRef` type is used with `iter_entities()` -- will no longer be needed after the replacement

### File Paths (all relative to `artifacts/developer/`)

- `tests/qa/fix_units_moving_while_attacking.rs` -- 7 hardening sites
- `tests/qa/tunnel_object_interface_state.rs` -- 6 hardening sites + 5 ignore annotation fixes
- `tests/qa/gdo_supply_tower_and_chopper.rs` -- 4 hardening sites
- `tests/qa/combat_behaviors.rs` -- 4 hardening sites
- `tests/qa/autonomous_targeting.rs` -- 3 hardening sites
- `tests/qa/construction_hp_rule.rs` -- 2 hardening sites
- `tests/qa/tunnel_area_and_construction_rules.rs` -- 2 hardening sites
- `tests/qa/movement_behaviors.rs` -- 2 hardening sites
- `tests/qa/attack_phases.rs` -- 1 hardening site
- `tests/qa/agent_resource_gathering_commands_and_behaviors.rs` -- 1 hardening site
- `tests/qa/tunnel_expansions_and_starting_condition.rs` -- 1 hardening site
- `tests/qa/fix_memory_leak_oom_freeze.rs` -- deprecated API fix
- `tests/qa/basic_combat_unit_interface_state.rs` -- 2 ignore annotations
- `tests/qa/agent_object_interface_state.rs` -- 1 ignore annotation fix
- `src/game/units/systems/core.rs` -- 2 dead helper removals (lines 3806-3823, 4080-4097)

## Dependencies

None. This is a standalone cleanup task that touches only test code and dead helpers. No runtime behavior changes.
