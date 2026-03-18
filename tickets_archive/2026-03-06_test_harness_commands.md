# Ticket: TestHarness Command Interface

## Current State
Phase 1 headless `TestApp` infrastructure is implemented. Tests can create a headless app and access the Bevy `World` directly via `app.world`. However, there is no convenience API for common game operations — each test must manually insert components, resources, and entities using raw ECS calls. This makes test authoring verbose and error-prone.

## Desired State
A `TestHarness` struct in `src/testing/harness.rs` that wraps a `&mut World` reference and exposes high-level command methods:

**Spawn Commands:**
- `spawn_unit(faction, unit_type, position) -> Entity`
- `spawn_structure(faction, structure_type, position, rotation, flip_h, flip_v) -> Entity`
- `spawn_resource(resource_type, position, amount) -> Entity`

**Unit Commands:**
- `issue_command(entity, command_type, target)` — issue any unit command (Move, AttackMove, AttackTarget, Stop, HoldPosition, Patrol, Enter, Gather, BuildTunnel)
- `set_selection(entities)` — set the current selection to a list of entities
- `clear_selection()` — clear selection

**Game State Commands:**
- `set_resources(faction, resource_type, amount)` — set a faction's resource count
- `advance_frames(n)` — advance the simulation by N frames
- `set_tile(position, tile_preset)` — set a tile type at a grid position
- `reveal_map(faction)` — reveal all tiles for a faction (disable fog of war)
- `set_camera(position, zoom)` — move the camera

All commands operate via direct ECS manipulation — same process, no IPC. The `TestHarness` takes `&mut World` for mutation methods. The module is gated behind `#[cfg(test)]` or a `testing` feature flag.

File organization:
- `src/testing/harness.rs` — `TestHarness` struct with command methods
- `src/testing/mod.rs` — module exports

## Justification
Required by `features/automated_qa_system.md` Layer 1. The command interface is the foundation enabling automated QA — without it, the QA agent cannot programmatically set up test scenarios or issue game commands. This is the critical path for unblocking the QA pipeline bottleneck (35 tasks backlogged in `/qa_tasks`).

## QA Steps
1. [auto] Create a `TestApp`, construct a `TestHarness` from its world, and call `spawn_unit(Faction::GDO, UnitType::Peacekeeper, Vec3::new(100.0, 0.0, 0.0))`. Verify the returned entity exists and has the correct faction, unit type, and position components.
2. [auto] Call `spawn_structure(Faction::GDO, StructureType::PowerPlant, ...)` and verify the entity has structure components with correct type and position.
3. [auto] Call `spawn_resource(ResourceType::Crystal, Vec3::new(50.0, 0.0, 0.0), 1000)` and verify the resource node entity exists with correct amount.
4. [auto] Spawn two units, call `set_selection(vec![unit1, unit2])`, then verify selection state contains both entities. Call `clear_selection()` and verify selection is empty.
5. [auto] Spawn a unit, call `issue_command(unit, Command::Move, Some(target_pos))`, advance frames, and verify the unit's command state reflects a move command.
6. [auto] Call `set_resources(Faction::GDO, ResourceType::Crystal, 500)` and verify `get_resources` returns 500.
7. [auto] Call `advance_frames(60)` and verify the simulation has advanced by 60 frames (check tick count or elapsed time).
8. [auto] Call `set_tile(position, TilePreset::Mountain)` and verify the tile at that position has the mountain tile type.
9. [auto] Call `reveal_map(Faction::GDO)` and verify all tiles are in Visible state for GDO.
10. [auto] Verify the `testing` module is not compiled in release builds (only behind `#[cfg(test)]` or `testing` feature flag).

## Expected Experience
All tests pass via `cargo test`. Each spawn command returns a valid entity with the expected components. Each game state command modifies the world state as expected, verifiable through subsequent ECS queries. The testing module does not appear in release builds.
