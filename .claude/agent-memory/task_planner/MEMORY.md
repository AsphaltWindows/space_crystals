# Task Planner Memory

## Key File Locations
- `src/shared/types.rs` - ObjectEnum, UnitBaseEnum, FactionEnum, SymmetryTypeEnum, Owner, etc.
- `src/game/types/objects.rs` - ObjectType, StructureType, ObjectInstance, StructureInstance, ObjectEnum::object_type()/structure_type()
- `src/game/types/structures.rs` - Structure instance states (DeploymentCenterState, BarracksState, etc.), stat constants (gdo_structure_stats)
- `src/game/types/factions.rs` - Faction resources (GdoPlayerResources, SyndicatePlayerResources, etc.), cap constants
- `src/game/types/mod.rs` - Re-exports factions, objects, structures
- `src/game/world/faction.rs` - Game start setup, player resource initialization

## Patterns
- Structure data: ObjectEnum has object_type() -> ObjectType and structure_type() -> Option<StructureType>
- Structure stats: constants in named modules (e.g., `gdo_structure_stats`)
- Structure instance state: Component structs with Default impl (e.g., BarracksState)
- Tests: inline #[cfg(test)] mod tests in each file

## Key Patterns for Unit Definitions
- Static data functions (e.g., `peacekeeper_type_data()`) at `src/game/units/types/unit_data.rs`
- Spawn functions (e.g., `spawn_peacekeeper()`) at `src/game/utils.rs`
- Speed conversion: SU/frame * FRAMES_PER_SECOND / SPACE_UNITS_PER_GRID_UNIT
- Frame conversion: `frames_to_seconds()` at unit_data.rs
- Melee range: `MELEE_RANGE` constant at `src/game/combat/types.rs:14`

## Selection & Command Panel Architecture
- Selection system: `src/game/world/resources.rs` — `selection_system()`, `drag_box_system()`, `selection_group_sync_system()`
- Command panel state: `src/ui/command_panel.rs` — `update_command_panel_state()`, `is_panel_visible()`
- `selection_group_sync_system` queries `(Entity, &ObjectInstance), (With<Selected>, Or<(With<Unit>, With<StructureInstance>)>)` — excludes neutral non-combat entities
- Neutral entities (SDS, SCP) have `Selectable` but differ: SDS has `ObjectInstance`, SCP does not
- HUD center section: `UnitsGridSection` in `src/ui/hud.rs` — `update_selected_units_grid_system` handles 0/1/multi-select display
- Multi-select cards: `spawn_multi_select_card()` at hud.rs:754 — reusable pattern for portrait rendering
- Camera centering pattern: query `MainCamera` transform, set translation.x/.z — see `control_group_system` at resources.rs:675

## GridPosition Sync Pattern
- `GridPosition` is set at spawn but NOT synced during movement for units
- Movement systems update `Transform.translation` only
- `world_to_grid()` at `src/game/units/utils.rs:13` converts Transform -> GridPosition
- Two `world_to_grid` functions exist: `src/game/units/utils.rs` (returns GridPosition) and `src/game/world/utils.rs` (returns tuple) — potential consolidation target

## Building Placement Architecture
- Validation: `can_place_building()` at `src/game/world/utils.rs:179` — checks build area, tile buildability, structure overlap, extraction plate on patch
- Ghost system: `update_placement_ghost()` at `src/game/world/faction.rs:847` — calls `can_place_building()`, sets ghost color
- Click system: `placement_click_system()` at `src/game/world/faction.rs:990` — gates on `placement_state.is_valid`
- Fog of war: `FogOfWarMap` resource at `src/game/world/types.rs:305` with `get(player_id, x, z)` method
- Tunnel placement has separate validation branch (exempt from surface rules)

## Pathfinding Architecture
- A* pathfinding: `src/game/units/pathfinding.rs:find_path()`
- Pathfinding helpers: `src/game/units/utils.rs` — `heuristic()`, `get_neighbors()`, `is_traversible()`, `smooth_path()`
- Two movement systems in `src/game/units/systems/core.rs`: `unit_movement_system` (non-TurnRate) and `turn_rate_movement_system` (TurnRate units)
- Behavior layer: `src/game/units/systems/behaviors.rs` — `moving_to_location_system()` manages `BaseBehaviorState.path_index` independently from `Path.current_waypoint`
- Waypoint threshold 0.3 hardcoded in both core.rs and behaviors.rs

## Worker-Built Placement Validation
- `can_place_building()` at `src/game/world/utils.rs:199` is GDO-specific (requires GdoBuildArea, checks fog visibility)
- Worker placement needs a separate `can_worker_place_structure()` — subset of checks (buildable + no overlap, no visibility/build area)
- Both share tile-buildability and structure-overlap logic — potential extraction into shared helpers
- `BuildingStructureBehavior` marker component pattern (like `EnteringTunnelBehavior`) recommended for build behavior

## Bevy 0.17 Viewport Coordinate Change (Critical)
- `viewport_to_world()` and `world_to_viewport()` now operate in **window-space** coordinates
- They use `logical_viewport_rect()` (includes viewport position) instead of 0.14's `logical_viewport_size()` (size only)
- Custom viewport offset subtraction (like `cursor_pos_in_viewport()`) causes **double-subtraction** in 0.17
- Source: `bevy_camera-0.17.3/src/camera.rs` lines 614-644, 508-534

## Ticket Processing Status
103 tickets processed total. Bug fix tickets now flowing from forum reports.

## Bevy 0.17 Migration (Active Priority)
- Project upgraded from Bevy 0.14 → 0.17.3. 316 compilation errors.
- Bevy skill: `~/.claude/skills/bevy/` with `references/bevy_specific_tips.md` for migration patterns
- Bevy 0.17.3 examples: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy-0.17.3/examples/`
- Key API changes: bundles removed (PbrBundle, NodeBundle, TextBundle), Style→Node, TextStyle→TextFont+TextColor, ChildBuilder→ChildSpawner, delta_seconds→delta_secs, get_single→single (returns Result), despawn_recursive→despawn, TargetCamera→UiTargetCamera, Handle<StandardMaterial>→MeshMaterial3d<StandardMaterial>
- `run_system_once()` now returns `Result<Out, RunSystemError>` — needs `.unwrap()` (impacts ~80 call sites across harness.rs and test files)
- `World::get_entity()` returns `Result<EntityRef, EntityDoesNotExistError>` instead of `Option` — `.is_some()`→`.is_ok()`, `None`→`Err(_)`, `Some(x)`→`Ok(x)`
- Bevy 0.17 ECS crate path: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.17.3/`
- Heaviest file: `src/ui/hud.rs` (132/316 errors = 42%)

## Collision Architecture (New - Session 75)
- `Silhouette` component at `src/game/combat/types.rs:194` — width/height in grid units
- `DomainEnum` at `src/shared/types.rs:377` — Ground/Air/Underground
- `has_structure_overlap()` at `src/game/world/faction.rs:1257` — pattern for spatial structure queries
- No collision/occupancy system exists yet — movement systems apply Transform directly
- `find_path()` has 5 callers across the codebase

## Command Indicator Architecture
- Existing ad-hoc indicators: `MoveTargetMarker` (cylinder at move target), `TargetHighlight` (torus on attack target)
- `UnitCommand` enum stores all target data needed for indicators (Vec3 positions, Entity targets)
- `right_click_move_command` at `src/game/units/systems/core.rs:177` is 330+ lines — bloated with visual feedback that should be in a sync system
