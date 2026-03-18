# Ticket: TestHarness Queries and Assertion Helpers

## Current State
The `TestHarness` (from the command interface ticket) provides methods to set up game state, but there is no convenience API for querying game state or asserting expected conditions. Tests would need raw ECS queries to verify outcomes.

## Desired State
Extend the `TestHarness` with query methods and add assertion helpers in `src/testing/assertions.rs`.

**Entity Queries:**
- `get_position(entity) -> Vec3`
- `get_health(entity) -> (current, max)`
- `get_attack_state(entity) -> AttackPhase`
- `get_behavior(entity) -> BehaviorState`
- `get_command(entity) -> CommandState`
- `get_movement(entity) -> (velocity, path_target)`
- `is_alive(entity) -> bool`

**World Queries:**
- `get_visibility(faction, tile_position) -> VisibilityState`
- `get_resources(faction, resource_type) -> u32`
- `get_entities_in_area(position, radius) -> Vec<Entity>`
- `get_selection() -> Vec<Entity>`
- `count_entities(filter) -> usize` (filter by faction + unit_type or similar)

**Structural Queries:**
- `get_tunnel_network(faction) -> TunnelNetworkInfo` (tunnel count, total space, units inside)
- `get_structure_state(entity) -> StructureState` (construction progress, operational status)

**Assertion Helpers** (in `src/testing/assertions.rs`):
- `assert_position_near(entity, expected_pos, tolerance)`
- `assert_health_equals(entity, expected_hp)`
- `assert_phase_equals(entity, expected_phase)`
- `assert_behavior_equals(entity, expected_behavior)`
- `assert_dead(entity)`
- `assert_visible(faction, tile_position)`
- `assert_resource_at_least(faction, resource_type, min_amount)`
- `assert_selection_count(expected_count)`

Query methods take `&World` (read-only). Assertion helpers panic with descriptive messages on failure, including actual vs expected values.

## Justification
Required by `features/automated_qa_system.md` Layer 1. Queries and assertions are essential for the QA agent to verify test outcomes programmatically. Without these, automated QA steps cannot determine pass/fail. The assertion helpers provide clean, readable test code and consistent error messages.

## QA Steps
1. [auto] Spawn a unit at (100, 0, 0). Call `get_position(entity)` and verify it returns approximately (100, 0, 0).
2. [auto] Spawn a unit and verify `get_health(entity)` returns `(max_hp, max_hp)` for that unit type.
3. [auto] Spawn two opposing units in range, issue attack command, advance frames. Verify `get_attack_state(attacker)` returns an expected `AttackPhase` value.
4. [auto] Verify `get_behavior(entity)` returns `Idle` for a freshly spawned unit with no commands.
5. [auto] Issue a move command to a unit, verify `get_command(entity)` reflects the move command state.
6. [auto] Verify `is_alive(entity)` returns `true` for a healthy unit and `false` for a unit reduced to 0 HP.
7. [auto] Set resources to 500, call `get_resources(faction, resource_type)`, verify it returns 500.
8. [auto] Spawn 3 GDO Peacekeepers. Call `count_entities` with faction=GDO, unit_type=Peacekeeper filter. Verify result is 3.
9. [auto] Call `get_entities_in_area(center, radius)` with known entity positions and verify correct entities are returned.
10. [auto] Call `assert_position_near(entity, expected, 1.0)` with a correct position — verify no panic. Call with an incorrect position — verify it panics with a descriptive message.
11. [auto] Call `assert_dead(entity)` on a dead entity — verify no panic. Call on a living entity — verify it panics.
12. [auto] Reveal map, call `assert_visible(faction, position)` — verify no panic. On unrevealed map, verify it panics.

## Expected Experience
All query methods return accurate game state data matching the ECS components. Assertion helpers pass silently on correct state and panic with clear, diagnostic error messages (showing actual vs expected values) on incorrect state. All tests run via `cargo test`.
