# Developer Agent Memory

## Key References
- **Completed work list**: `completed_work.md` (separate file — ~80 tasks completed)
- **Insights**: `agent_logs/developer_insights.md` (always loaded)

## Project Structure
- `src/` at 7 entries (max) — game/, lib.rs, main.rs, README.md, shared/, simulation/, ui/
- `src/shared/` — crate-level types.rs, utils.rs, testing/ — re-exported via lib.rs
- `src/shared/testing/` — TestApp + TestHarness, feature-gated, at 7 entries (max)
- `tests/qa/` — QA integration tests, `helpers.rs` re-exports TestApp/TestHarness/assertions
- `src/game/` and `src/game/units/` use `types/` directories instead of `types.rs`
- `src/game/units/systems/` has core.rs, commands.rs, behaviors.rs
- `src/game/combat/systems/` has core.rs, behaviors.rs
- FactionEnum canonical in `src/shared/types.rs`, re-exported from `src/game/types/factions.rs`
- `ObjectInterfaceState` in `src/ui/types.rs` — replaces old CommandPanelState + CommandMode
- `CursorTarget` resource in `src/ui/types.rs` — per-frame cursor classification, always populated (not gated by cursor_over_ui)
- `ray_aabb_intersect` in `src/ui/utils.rs` — use for entity click detection (NOT screen-space projection, which fails with camera viewports)

## At-Capacity Directories (7 entries max)
- `src/`, `src/ui/`, `src/game/world/`, `src/game/combat/`, `src/game/units/systems/`, `src/game/units/types/`, `src/shared/testing/`

## Build & Engine
- Bevy 0.17, Rust Edition 2021, dynamic linking enabled
- Bundle tuple limit: 15 — use `.insert()` for overflow
- SystemParam tuple limit: 16 — `command_panel_hotkeys` and `rebuild_command_panel_ui` at limit
- `EntityCountDiagnosticsPlugin::default()` (has fields in 0.17)

## Bevy 0.17 API Quick Reference
- `for x in &query` / `&mut query` (not `.iter()`) — except chained calls need `.iter()`
- `emissive` on StandardMaterial is `LinearRgba`, not `Color`
- PbrBundle → (Mesh3d, MeshMaterial3d, Transform); NodeBundle → Node
- TextBundle → (Text, TextFont, TextColor); ButtonBundle → (Button, Node)
- Style merged INTO Node; Camera2dBundle → Camera2d; Camera3dBundle → (Camera3d, Transform)
- ChildBuilder → ChildSpawnerCommands; Parent → ChildOf
- despawn() is recursive by default; delta_secs(); single() returns Result
- viewport_to_world/world_to_viewport return Result not Option
- Handle<StandardMaterial> query → MeshMaterial3d<StandardMaterial>; Handle<Mesh> → Mesh3d
- TargetCamera → UiTargetCamera; ZIndex::Global(n) → ZIndex(n); BorderColor(c) → BorderColor::all(c)
- Entity::from_raw_u32(n).unwrap(); run_system_once() returns Result
- world.get_entity() returns Result — .is_none()→.is_err()
- Text::new(); **text for mutation; TextStyle → TextFont+TextColor
- JustifyText → Justify (doc alias exists but type is `Justify`)
- bevy::camera::Viewport; bevy::mesh::Indices
- WindowResolution::new(w, h) takes u32; Children iteration: `for child` (by value)

## Architecture Patterns
- **DiagCategory** SystemSet in `src/simulation/types.rs` — configured in SimulationCorePlugin with set-level ordering + `run_if(in_state(AppState::InGame))`. Individual systems don't need `.run_if()`.
- **Ordering chain**: Selection → Faction → UiHud → Movement → Combat → Turrets/Projectiles
- **DespawnOnExit(state)** — auto-despawns entities on state exit, enabled by `init_state`
- **AppState** enum (Menu/InGame) in `src/shared/types.rs`; all game systems gated via DiagCategory
- **IsDefaultUiCamera** required on each state's camera for UI rendering
- **OccupancyMap** rebuilt each frame; `find_path` takes `&OccupancyMap` + `self_pos`
- **CombatAssetCache** resource caches meshes/materials for transient combat entities
- **Asset caching rule**: systems spawning rapidly MUST cache mesh/material handles

## Testing
- `TestHarness<'a>` wraps `&mut App` — spawn_unit/structure/resource, selection, commands, etc.
- GdoPlayerResources/SyndicatePlayerResources are Components on Player entities, NOT Resources
- `testing` feature in default features; `MinimalPlugins` needs `StatesPlugin` added manually
- TestApp inits AppState + StatesPlugin + transitions to InGame
- `run_system_once` returns `Result<T, RunSystemError>` — unwrap outer Result; requires `use bevy::ecs::system::RunSystemOnce;` in test files
- **FixedUpdate doesn't fire in headless TestApp** — set state to near-complete, call `world.run_system_once(system_fn)`, then `step()` for deferred commands

## Common Gotchas
- Query conflicts are RUNTIME panics — use `Without<T>` or `ParamSet`
- Bevy forward is -Z — use `Transform::look_at()`, not manual Quat rotation
- `ObjectInstance.hp` and `max_hp` are `Option<f32>`
- FactionEnum variant is `TheSyndicate`, not `Syndicate`
- Structure overlap must check full footprints via `ObjectType::size`
- Air units: `find_path_for_domain()`, Y=1.5; Ground: Y=0.5
- Tech prereqs: check in both `execute_command_action` and `grid_button_enabled_ext`
- Negative scale (flip) + backface culling = visual artifacts → set `cull_mode: None` on flipped materials
