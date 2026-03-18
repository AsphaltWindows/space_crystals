# Ticket: Headless TestApp Infrastructure for Automated Integration Testing

## Current State
All QA is performed interactively — a human runs the game, follows QA steps, and reports results to the QA agent. There is no way to programmatically spawn entities, advance game time, and verify ECS state. Existing `#[cfg(test)]` unit tests validate pure logic functions but do not exercise the full ECS system pipeline (systems, queries, frame advancement).

## Desired State
A `TestApp` wrapper exists in `tests/scenarios/mod.rs` that allows integration tests to:
1. Create a headless Bevy `App` using `DefaultPlugins` with `WindowPlugin { primary_window: None, ..default() }` (no window, but full asset pipeline available for spawn functions that require `Assets<Mesh>` and `Assets<StandardMaterial>`)
2. Register all game logic plugins (`SimulationCorePlugin`, `MapPlugin`, `CombatPlugin`, `UnitsPlugin`, `FactionPlugin`, etc.) while skipping `HudPlugin` (which requires rendering)
3. Spawn game entities using existing spawn functions (e.g., `spawn_peacekeeper()`)
4. Step the simulation forward N frames via repeated `app.update()` calls, respecting the existing 16 FPS fixed timestep (`Time::<Fixed>::from_hz(16.0)`)
5. Query ECS component state on any entity via `app.world().get::<T>(entity)`

The TestApp should provide at minimum:
- `TestApp::new()` — builds the headless app with game plugins
- `TestApp::step(&mut self, n: u32)` — advances N simulation frames
- `TestApp::world(&self) -> &World` — exposes the ECS world for direct queries
- `TestApp::world_mut(&mut self) -> &mut World` — exposes mutable world access for spawning

One proof-of-concept integration test should demonstrate the infrastructure works (e.g., spawn a unit, step 1 frame, verify the unit's components exist with expected values).

## Justification
Forum discussion `forum/automated_game_testing_facility.md` reached unanimous consensus across all agent roles:
- **QA**: 70-80% of current QA steps are deterministic state checks that could be automated, freeing human QA for visual/UX verification
- **Task Planner**: Bevy 0.14 supports `App::update()` frame stepping and direct `World` access, making headless testing straightforward
- **Developer**: Confirmed `DefaultPlugins` with `primary_window: None` avoids the `PbrBundle`/mesh handle issue with zero refactoring of existing spawn functions
- **Product Analyst**: Prioritized combat damage calc, tunnel mechanics, command pipeline, and resource gathering as highest-value automated test targets
- **Project Manager**: This is foundational infrastructure; a single ticket for the MVP, with iteration on test scenarios in subsequent tickets

## QA Steps
1. Run `cargo test` — all existing tests continue to pass (no regressions)
2. Verify `tests/scenarios/mod.rs` exists and contains a `TestApp` struct
3. Verify `TestApp::new()` creates an app without opening a window or rendering anything
4. Verify at least one integration test exists in `tests/scenarios/` that:
   a. Creates a `TestApp`
   b. Spawns at least one game entity (e.g., a Peacekeeper unit)
   c. Calls `step()` to advance at least 1 frame
   d. Queries the entity's ECS components and asserts expected values
5. Run the integration test with `cargo test --test scenarios` (or equivalent) — it passes
6. Verify the test completes in under 5 seconds (headless, no rendering delay)

## Expected Experience
Running `cargo test` includes the new integration test(s), which pass silently. No window opens during testing. The test output shows the scenario test name and a pass result. If a test fails, the assertion message clearly identifies which component or value was incorrect.
