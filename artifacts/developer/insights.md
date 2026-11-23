# Developer Insights

## Code Organization
- `src/game/` — core game logic (combat, units, world, types)
- `src/ui/` — HUD, command panel, menu (7 files currently at limit)
- `src/shared/` — shared types, utils, testing infrastructure
- `src/simulation/` — diagnostics, instrumentation, overlay

## Key Files
- `src/ui/command_panel.rs` — `update_command_panel_state()`, `is_common_command()`, DcConstructing/EfConstructing state logic, DomainEnum references
- `src/game/world/map.rs` — likely contains `rebuild_occupancy_map`
- `src/game/units/systems/commands.rs` — unit command handling, command_state_sync_system, command_dequeue_system
- `src/game/units/types/state/` — behavior and command state types

## Patterns
- Unit types defined via EntityDefinition/UnitDefinition traits in entity def files + EntityRegistry for runtime dispatch
- Supply chopper behavior pattern: insert behavior marker component alongside UnitCommand at command-issue time (core.rs). Behavior system processes each tick, removes marker on completion. Auto-return: pickup → dropoff when attached_tower exists.
- FactionEnum uses `GlobalDefenseOrdinance` (not `GDO`). GdoPlayerResources requires `unit_control_cap` and `has_power_plant` fields.
- Interface states are enum-based state machines (DcIdle, DcConstructing, etc.)
- Production systems follow Barracks pattern (tick system, queue, spawn)
- Ungroupable entities (Tunnels, Agents) each form their own SelectionGroup
- ObjectEnum uses `Tunnel` (not `SyndicateTunnel`) for syndicate tunnel variant
- CommandType now includes HoldPosition and Stop variants (added for command-to-state mapping)
- command_state_sync_system maps UnitCommand → BaseCommandState; command_dequeue_system pops CommandQueue when Idle
- PickUpSupplies/AttachToTower chopper commands now have dedicated CommandType::PickUpSupplies/AttachToTower variants
- SupplyChopper has its own grid layout in `get_grid_slot_action` via `caps.is_chopper` guard on `ObjectInterfaceState::Default`
- Adding unit-type-specific grids: add flag to `SelectedUnitCapabilities`, set in `compute_selected_unit_capabilities`, use `if` guard on `Default` match arm in `get_grid_slot_action`
- When adding fields to `SelectedUnitCapabilities`, update all struct literal test helpers (or use `..Default::default()`)
- `PreviousSelectionSnapshot` resource tracks selection state for `interface_state_selection_reset_system` (uses Resource not Local for testability with `run_system_once`)
- `interface_state_validation_system` validates ObjectInterfaceState against active SelectionGroup, resets to Default if invalid
- Adding new CommandType variants requires updating the match in core.rs:right_click_move_command ground-click handler
- Glider UnitBaseEnum has `has_turret: true` — gliders use turrets for their weapon systems
- TurretCommandState relay pattern: add `Option<&mut TurretCommandState>` to query, use `if let Some(ref mut turret) = turret_state` to set/clear `locked_target`
- Shift-click queuing: `issue_or_queue_command()` in utils.rs; all command-issuing paths check shift and use queue. Test entities for `right_click_move_command` must include `CommandQueue::new()` and `ButtonInput::<KeyCode>::default()` resource.
- Bevy 0.17 system parameter limit is 16. `command_panel.rs` systems are at the limit — fold new query needs into existing queries (e.g., `Option<&mut CommandQueue>` added to `selected_units` query) instead of adding separate queries.
- `EntityWorldMut` works in `entity_cmds.queue(|mut entity: EntityWorldMut| { ... })` for deferred world access in `execute_command_action` (avoids borrow conflicts with immutable query borrows from `command_target_entities`)
- Movement system filter exclusions: `unit_movement_system`, `unit_rotation_system`, and `channel_fallback_locomotion_system` now exclude all 5 specialized types: TurnRate, FixedTurnRadius, SpeedTurnRadius, Drag, Glider. When adding new movement systems, ensure these filters stay in sync.
- AttackState has fields: `phase`, `time_in_phase`, `current_target` (not `target`). Use `Entity::from_raw_u32(n).unwrap()` for test entity creation.

## Bevy 0.17 Camera API
- `OrthographicProjection` is NOT a standalone component — must wrap in `Projection::Orthographic(...)`
- `Camera3d` has `#[require(Camera, Projection)]` — spawning with `Projection::Orthographic(...)` overrides the default perspective
- `ScalingMode` is in `bevy::camera::ScalingMode` (NOT `bevy::render::camera`)
- `ScalingMode::FixedHorizontal { viewport_width: f32 }` uses named field (not positional)
- `OrthographicProjection::default_3d()` exists — sets near=0.0, far=1000.0, WindowSize scaling
- `OrthographicProjection::default()` does NOT exist — use `default_3d()` or `default_2d()`

## Build Environment
- **RESOLVED**: Updated Cargo.toml to Bevy 0.17 with explicit features, excluding `bevy_audio` (needs alsa-lib-devel) and `bevy_gilrs` (needs libudev-sys). Both are unavailable on this system (no sudo access).
- Rust 1.94.0 on Fedora 43, cargo installed via rustup at `~/.cargo/bin/cargo` (not on default PATH, use `export PATH="$HOME/.cargo/bin:$PATH"`)
- `cargo test` compiles fully. `cargo test --lib` passes all 1683 tests. Integration tests (`--test qa`, `--test scenarios`) compile and mostly pass — 21 QA + 1 scenario test have pre-existing runtime failures (game logic issues, not compilation).
- Testing module (`src/shared/testing/`) is always compiled (no cfg gate). `testing` feature exists in Cargo.toml but is not required.
- Cargo.toml was previously pinned at bevy "0.14" but code was written for 0.17 APIs (e.g. `Entity::from_raw_u32`). Now corrected to "0.17".

## Build Rules
- **Never run `cargo clean`** — always use incremental builds (operator directive)

## Combat Targeting Patterns
- `can_threaten(cap, domain)` in core.rs — checks if target's AttackCapability.target_domain can hit the defender's DomainEnum. Returns false for None (unarmed).
- `select_best_target(candidates)` in combat/utils.rs — 3-tier priority selection: threatening > least rotation > closest. Used by base_auto_target_system and hold_position_behavior_system.
- `is_valid_target(obj, vis, domain, attack_domain)` in combat/utils.rs — filters: destructible + visible + domain-compatible.
- `base_auto_target_system` only activates for Idle and HoldPosition (AttackMove removed — has own scanning).
- Idle units scan within SightRange; HoldPosition uses weapon range.

## Rust/Bevy Type Inference Gotchas
- In integration tests, `(val - literal).abs()` can fail type inference (E0282). Fix: use `f32::abs(val - literal)` or `(val - literal_f32).abs()`.
- Matching `&UnitCommand::Move(target)` gives `target` by value (Copy types) — do NOT deref with `*target`. Use `target` directly or compare with `&expected`.
- `World::iter_entities()` returns `EntityRef` — closures using it may need explicit type annotation: `|e: EntityRef|`

## Bevy 0.17 API Notes
- `World::get_entity()` returns `Result`, not `Option` — use `.is_ok()` not `.is_some()`

## Behavior Completion
- `behavior_completion_system` in behaviors.rs: transitions Move/Reverse/Stop → Idle when LocomotionChannel::Stationary + velocity < threshold. Registered in UnitsPlugin Phase 2. Glider circling is excluded.
- Known spec deviations (acceptable simplifications): AttackMove leash uses radial distance from origin (spec says perpendicular from PathReference); Patrol engagement uses direct AttackTarget (spec says wrap AttackMovingToLocation sub-behavior with leash).

## Channel Consumer Systems
- Dual movement pipeline: old (MoveTarget/Path) and new (LocomotionChannel/OrientationChannel) coexist via query filters (`Without<MoveTarget>` on channel systems, `With<MoveTarget>` implicitly on old systems)
- Channel consumers in core.rs: `channel_turnrate_locomotion_system`, `channel_fallback_locomotion_system`, `channel_orientation_system`
- All behavior-driven units (Peacekeepers, Agents, Guards) use TurnRateMovementParams — TurnRate consumer covers all current channel users
- Bevy default rotation `Quat::from_rotation_y(0.0)` faces -Z — tests must account for this
- In tests: use `Time::<()>::default()` and `resource_mut::<Time<()>>().advance_by(...)` for Bevy 0.17 Time
- Registered in UnitsPlugin Phase 3 alongside old movement systems, with grid_position_sync_system `.after()` the new systems

## Known Issues (from forum review)
- `rebuild_occupancy_map` doesn't filter by DomainEnum — underground structures block surface movement
- `is_common_command()` hardcodes command classifications without checking selection composition
- `update_command_panel_state()` forces construction sub-menu state every frame
- Right-click only handles ground clicks, no entity detection
- Persistent horizontal black line in viewport (likely UI element, not camera artifact)
- QA integration tests: 21 runtime failures remain (Move commands becoming Idle after step, supply tower build frames mismatch, etc.) — these are game logic issues, not compilation problems
- `PointerIndicator` component in `ui/types.rs`; pointer spawn/update systems and `PointerDisplayType::indicator_color()` impl in `ui/utils.rs`
- `ui/` directory is at 7 items (the limit) — any new files require reorganizing into subdirectories
- FixedUpdate systems (barracks_production_tick, dc_construction_tick, etc.) don't fire reliably in headless TestApp. Use `world.run_system_once(system_fn)` to invoke them directly, then `test_app.step()` to flush deferred commands. Requires `use bevy::ecs::system::RunSystemOnce;` trait import.
- `drag_movement_system` and `glider_movement_system` in core.rs (after speed_turn_radius_movement_system). Drag uses `Option<&MoveTarget>` to handle idle (drag-only decel) vs active (thrust+drag). Glider uses `Option<&MoveTarget>` — idle circles at idle_speed with constant right turn; active accelerates to max_speed with speed-dependent turn radius. Both registered in Phase 3 of UnitsPlugin with `.after()` on crushing_system and grid_position_sync_system.
- Borrow checker gotcha: `velocity.0 -= velocity.0 * factor` fails — save RHS to local first: `let drag = velocity.0 * factor; velocity.0 -= drag;`
- 2076 lib tests passing as of this session.
- Structure stat constants canonical location: `game/entities/factions/{faction}/buildings/types.rs`. Re-exported via `game/types/structures::{gdo,syndicate,cults}_structure_stats` for backward compatibility.
- `ObjectEnum::object_type()` now delegates to entity definition `::object_type()` methods — single source of truth in entity def files.
- Unit-specific constants (PEACEKEEPER_CONTROL_COST, AGENT_MINING_DURATION, etc.) canonical location: entity def files (e.g., `game/entities/factions/gdo/units/peacekeeper.rs`). Old `unit_data.rs` re-exports removed.
- `game/utils.rs` now only contains billboard_label_system and spawn_ghost_side_labels. All spawn functions removed — use EntityRegistry::spawn() instead.
- `rebuild_command_panel_ui` and `command_panel_hotkeys` are at 16 system params (Bevy limit). To add cross-cutting guards (like ownership), use a resource field (e.g. `SelectedUnitCapabilities.owned_by_local_player`) set by an earlier system rather than adding query params.
- `handle_command_button_clicks` is also at 16 params. Ownership guard is handled indirectly — if panel buttons aren't rendered (by `rebuild_command_panel_ui` guard), there's nothing to click.
- TestApp::new_with_faction() (not TestApp::new(faction)) for faction-specific test apps. Owner::player(id) (not Owner::new(id)). GDO selected = player 0; DC is auto-spawned by init_faction_resources (adds DC_POWER=20 to power_generated).
- `OriginatingCenters` component in `game/units/types/unit_data.rs` — tracks which RC(s) produced a Cults unit. `cults_unit_death_tracking_system` in combat/systems/core.rs decrements RC `local_used` on unit death. Runs `.before(remove_dead_entities_system)` in CombatPlugin.
- RC auto-production: `recruitment_center_production_system` and `cults_unit_control_aggregation_system` in game/world/faction.rs, registered in FixedUpdate DiagCategory::Construction. Production: 12s base (192 frames), scales inversely with effectiveness.
- `CultsRecruit` variant in ObjectEnum; entity def in `game/entities/factions/cults/units/recruit.rs` — minimal placeholder (no attack, no movement params). `RECRUIT_MAX_HP = 50.0`.
- In faction.rs tests: `Unit` is `crate::types::Unit` (not `crate::game::units::types::Unit`). `GridMap` is `crate::game::world::types::GridMap`.
- `StructureMenuState::RecruitmentCenterMenu` for RC interface. `RcCancel` action resets `production_progress` to 0. RC `rally_point` is `Option<Vec3>` (not `Option<RallyTarget>` like BK/HQ/ST). `bk_hq_query` now includes `Option<&mut RecruitmentCenterState>` as 4th tuple element with `With<RecruitmentCenterState>` in Or filter.
- `resources.rs` has `interface_state_validation_system` that requires match exhaustion for all `StructureMenuState` variants — must add any new variant there.
- `selected_owners` query in `rebuild_command_panel_ui` now has 3 tuple elements: `(&Owner, Option<&HeadquartersState>, Option<&RecruitmentCenterState>)`. When destructuring, use `(owner, hq, rc)` or `(owner, ..)`.
- 1830 lib tests passing as of prior session.
- `move_command_dispatch_system` in commands.rs — dispatches dequeued movement commands (Move, AttackMove, Patrol, Reverse, AttackLocation) by computing paths via `find_path_for_domain`. Uses `Without<MoveTarget>` filter to skip already-dispatched commands. Falls back to Idle on no path found. Runs `.after(command_dequeue_system)`, and `command_state_sync_system` runs `.after()` both.
- 1879 lib tests passing as of this session.
- Attack right-click now computes initial approach path inline (like Move), then `attacking_object_behavior_system` handles ongoing pursuit/re-pathing. Queued attack commands (shift-held) skip inline path — `attacking_object_behavior_system` handles them after dequeue.
- `CultsConstructionState` component in `game/types/structures.rs` — tracks assigned recruits, construction progress, total frames. `CultsConstructionState::new(total_frames)`.
- `UnitCommand::ConstructBuilding(Entity)` — Recruit walks to & enters a Cults building under construction. `CommandType::ConstructBuilding` and `CommandType::AssistConstruction` added.
- `cults_construction_tick_system` in faction.rs — progress scales with recruit count. `construction_hp_tick_system` has `Without<CultsConstructionState>` filter to skip Cults buildings.
- `cults_construction_cancel_system` in faction.rs — ejects recruits when building dies. Registered in CombatPlugin `.before(remove_dead_entities_system)`.
- `cults_recruit_enter_construction_system` in behaviors.rs — Recruits with ConstructBuilding command enter building when within 2.0 distance threshold, become Hidden.
- `STORAGE_BUILD_FRAMES = 300` constant in structures.rs.
- `SymmetryTypeEnum::ABCB` added — allows non-square sizes (entrance A, matching long sides B, exit C). Must update `side_labels()` in utils.rs and `oriented_labels()` in objects.rs when adding new symmetry variants.
- `ArmoryState` component in `game/types/structures.rs` — stored_recruits (Vec<Entity>, max 10), training_queue (Option<ObjectEnum>), training_progress, rally_point (Option<RallyTarget>). Derives Default.
- `spawn_cults_armory()` in game/utils.rs — 3x2 Cults structure with ABCB symmetry, Armor component, ArmoryState. Purple-ish color (0.55, 0.2, 0.55). Armory constants in `cults_structure_stats` module.
- `UnitCommand::EnterArmory(Entity)` — Recruit enters Armory. `EnteringArmoryBehavior` marker in behavior.rs. `enter_armory_dispatch_system` + `entering_armory_behavior_system` in behaviors.rs. Right-click resolution in core.rs. `target_info` query in `right_click_move_command` now has 6 fields (added `Option<&ArmoryState>`).
- `spawn_cults_recruit()` now includes `LocomotionChannel`, `OrientationChannel`, `Velocity` (added for movement behaviors).
- `CultsSoldier`/`CultsGunner` ObjectEnum variants added as stubs — placeholder spawn functions in game/utils.rs, ObjectType entries in objects.rs, `is_unit()` returns true.
- `StructureMenuState::ArmoryMenu` — Armory command panel state. Grid: Q=TrainSoldier, W=TrainGunner, E=EjectAll, C=SetRallyPoint.
- `CommandButtonAction::ArmoryTrainSoldier/ArmoryTrainGunner/ArmoryEjectAll` — structure (not unit) actions. Crystal deduction uses deferred `commands.queue(|world: &mut World| {...})` to access `CultsPlayerResources` without adding a system param.
- `bk_hq_query` in command_panel.rs now includes `Option<&mut ArmoryState>` + `With<ArmoryState>` in Or filter — 5-element tuple.
- `selected_owners` query in `rebuild_command_panel_ui` now has 4 tuple elements: `(&Owner, Option<&HeadquartersState>, Option<&RecruitmentCenterState>, Option<&ArmoryState>)`.
- `ArmoryEjectionQueue` component in ui/types.rs — VecDeque<Entity> + cooldown, attached to Armory during eject.
- `armory_training_tick_system` + `armory_eject_tick_system` in faction.rs, registered in FixedUpdate/DiagCategory::Construction.
- `armory_exit_side_position()` in faction.rs — computes Side C world position for ABCB symmetry structures.
- `RallyTargetKind::Armory` variant added — rally point system supports Armory in `production_rally_point_system`.
- `CultsRecruitMenuState` enum in ui/types.rs — RecruitDefault, RecruitConstructMenu, RecruitAwaitingPlacement. `ObjectInterfaceState::CultsRecruitMenu(...)` variant. Pattern mirrors AgentMenu.
- `spawn_cults_storage_under_construction()` in game/utils.rs — uses ObjectInstance::under_construction + ConstructionHP + CultsConstructionState. Follows spawn_tunnel_under_construction pattern.
- `CommandButtonAction::RecruitConstruct`, `RecruitSelectBuilding(ObjectEnum)`, `RecruitAssistConstruction` — unit (not structure) actions, mapped in is_unit_action, object_type_supports_action, grid_button_enabled_ext.
- CultsRecruit placement uses `can_worker_place_structure` (same as Agent tunnel) — no build area, no fog check. No build area overlay for Cults placement.
- `placement_click_system` now has `selected_recruits` query for issuing ConstructBuilding to all selected recruits on left-click placement.
- 1918 lib tests passing as of this session.
- `faction.rs` split into `faction/` directory: mod.rs, setup.rs, power.rs, production.rs, construction.rs, resources.rs, rally.rs, placement.rs, ejection.rs. `get_power_ratio_for_owner` is `pub(super)` in power.rs. `armory_exit_side_position` is `pub(super)` in ejection.rs. All public items re-exported from mod.rs.
- System ordering in ui/mod.rs: `command_panel_hotkeys` and `right_click_cancel_submenu` must run `.after(update_command_panel_state)` to ensure `SelectedUnitCapabilities.owned_by_local_player` is set before guards read it.
- `DiagCategory::Commands` now runs BEFORE `DiagCategory::Movement` (added `.after(DiagCategory::Commands)` to Movement). This ensures command dequeue+dispatch complete and deferred inserts (MoveTarget/Path) are flushed before behavior systems run.
- `behavior_completion_system` has `Without<MoveTarget>` filter — prevents premature completion of old-pipeline Move commands that have MoveTarget but LocomotionChannel::Stationary (default for old pipeline). Old pipeline handles its own completion by removing MoveTarget and setting Idle directly.
- Dual pipeline gotcha: old-pipeline units have default LocomotionChannel::Stationary + BaseBehaviorState::None permanently. Any new-pipeline system that checks these defaults must exclude old-pipeline entities (via `Without<MoveTarget>` or similar) to avoid false triggers.
- `update_selected_units_grid_system` in hud.rs now has `fog_map: Res<FogOfWarMap>` and `local_player: Res<LocalPlayer>` params (15 total params). SCP/SDS/sc_patches queries include `&GridPosition`. Fog visibility gates: SCP remaining amount, SDS current supplies + next delivery, EP remaining SC from attached patch.
- `FogOfWarMap::new(width, height)` takes 2 args (not 3). `FogOfWarMap::set(player_id, x, z, state)` takes 4 args.
- 1897 lib tests passing as of this session.
- `issue_rally_command` in faction.rs now computes approach path (MoveTarget + Path) for enemy AttackTarget rally, matching right-click attack pattern. Shared by Barracks, HQ, and Armory.
- Test tiles for pathfinding: use `spawn_test_tile(world, x, z, TilePresetEnum::Plane)` helper. Grid coords must match `world_to_grid` output -- world (0,0,0) maps to grid (32,32). Use `grid_to_world(&GridPosition{x,z})` for consistent world positions.
- `CommandContext` struct in core.rs bundles shared read-only params for `right_click_move_command` (shift_held, local_player_number, is_right_click, is_left_click, in_command_mode, command_type, cursor_target). Constructed after early returns at line ~391.
- `try_issue_command` / `try_issue_command_no_attack_reset` in core.rs — extracted helpers for the interruptibility-check + clear-state + issue pattern. `try_issue_command` inserts `AttackState::default()` on clear; `_no_attack_reset` does not. Both return `bool` (true = issued/queued, false = not interruptible). For behavior inserts after command, get new `EntityCommands` via `commands_ecs.entity(entity)` after the helper returns true.
- Bevy 0.17: `Query::single_mut()` returns `Result`, must `.unwrap()` in tests.
- `right_click_move_command` in core.rs is now a ~63-line dispatch function. All command handling extracted into private handler functions: `handle_attack_entity`, `handle_dropoff_target`, `handle_enter_target`, `handle_pickup_supplies_target`, `handle_attach_to_tower_target`, `handle_dropoff_supplies_target`, `handle_chopper_right_click`, `handle_agent_right_click`, `handle_recruit_right_click`, `handle_guard_right_click`, plus ground-click handlers (`handle_move_ground`, `handle_patrol_ground`, `handle_attack_ground`, `handle_attack_move_ground`, `handle_attack_ground_location`, `handle_reverse_ground`). Shared ground pathfinding via `ground_pathfind_and_issue` helper.
- `SpaceCrystalsPatch` (with 's') is the ObjectEnum variant, not `SpaceCrystalPatch`. `SpaceCrystalPatch` is the component struct.
- `SupplyChopperState` has `Default` derive; `SupplyTowerState` does NOT (must construct all fields). Supply Chopper uses `UnitBaseEnum::HoverCraft`.
- 1917 lib tests passing as of this session.
- `TestHarness::set_selection()` now updates both `Selected` ECS markers AND `Selection` resource groups AND `PreviousSelectionSnapshot`. This prevents `interface_state_selection_reset_system` from spuriously resetting `ObjectInterfaceState` to Default when selection changes during tests.
- `game::world::resources` module is now `pub(crate)` (was private) -- needed for `PreviousSelectionSnapshot` access from testing harness.
- Concurrent developer agents may conflict when modifying the same source files. The scheduler can launch multiple developer instances simultaneously. If seeing files appear/disappear during builds, check `ps aux | grep claude` for concurrent agents.
- Fog selection filter: `selection_system` and `drag_box_system` in resources.rs now take `Res<SelectedFaction>` param. Non-owned entities must be Visible to be selectable, except SDS which is selectable in Explored areas for GDO players only. `fog_filter_allows` test helper takes `(is_owned, is_sds, is_gdo_player, visibility)`.
- `drag_box_system` selectables query now has 6 elements (added `Option<&SupplyDeliveryStation>`). `current_selection_tier` signature updated to match.
- `spawn_unit_at_grid` and `spawn_structure_at_grid` in harness.rs now insert `VisibilityStateEnum::Visible` on all spawned entities. This is required for auto-targeting tests (base_auto_target_system queries `&VisibilityStateEnum` as required component).
- `turn_rate_movement_system` now checks for `UnitCommand::Stop | UnitCommand::Idle` and removes `MoveTarget`/`Path` + zeroes velocity. Without this, TurnRate units with MoveTarget would keep moving on Stop because: (1) channel_turnrate_locomotion_system has `Without<MoveTarget>` filter, (2) turn_rate_movement_system didn't previously check command state.
- command_panel split files (state.rs, hotkeys.rs, ui_rebuild.rs, helpers.rs, tunnel_actions.rs, pointer.rs, actions.rs) need careful cross-module import management. Functions used across files in the `command_panel/` module must be `pub(super)`. `ui::utils::ray_aabb_intersect` is accessed as `crate::ui::utils::ray_aabb_intersect` from within `command_panel/state.rs`.
- 1920 lib tests passing, 318 QA tests passing (0 failures, 8 ignored) as of this session.
- `world.iter_entities()` is deprecated — use `world.query::<&T>()` + `query.iter(world)` instead.
- command_panel.rs (6357 lines) successfully split into command_panel/ directory module with 9 files: mod.rs, grid_layout.rs, state.rs, ui_rebuild.rs, hotkeys.rs, actions.rs, helpers.rs, pointer.rs, tunnel_actions.rs. Total 6559 lines. 1920 lib tests passing, 318 QA tests passing.
- Module structure: mod.rs re-exports 5 public systems. Cross-module functions use pub(super) visibility. All tests consolidated in helpers.rs test module (accesses sibling modules via super::super::module_name::function).
- Key patterns: super::super::types::* imports UI types. super::helpers:: imports shared helper functions. Tests that need private items from a specific module (pointer, hotkeys) access them via pub(super) visibility.
- The sandbox environment restores deleted/modified files between separate Bash tool calls. To modify files atomically, all changes must be in a single Bash command.
- `command_panel.rs` (6357 lines) split into `command_panel/` directory with 9 submodules: mod.rs, grid_layout.rs, state.rs, ui_rebuild.rs, hotkeys.rs, actions.rs, tunnel_actions.rs, pointer.rs, helpers.rs. All cross-module functions use `pub(super)` visibility. Tests distributed to their respective modules with explicit `use crate::types::*;` imports (since `use super::*;` only brings in the submodule's own scope).
- Split gotchas: (1) Python `\nfn name(` pattern misses functions with lifetimes `fn name<'a>(` or functions at very start of code section (no preceding newline). Use `sed -i 's/^fn name/pub(super) fn name/'` as backup. (2) Test modules in split files need `use crate::types::*;` explicitly because `use super::*;` only brings in the submodule's file-level imports. (3) Original `super::super::utils` paths become `crate::ui::utils` in deeper submodules.
- 1869 lib tests passing as of this session (zero warnings).
- behaviors.rs split into behaviors/ directory with 6 submodules: mod.rs, movement.rs, tunnel_interaction.rs, construction.rs, resource_gathering.rs, supply_chopper.rs.
- faction.rs split into faction/ directory with 9 submodules: mod.rs, setup.rs, power.rs, production.rs, construction.rs, resources.rs, rally.rs, placement.rs, ejection.rs.
- OBJECT_PROXIMITY_DISTANCE is `pub(super)` in movement.rs — used by supply_chopper.rs. tunnel_side_world_position is `pub` (not `pub(super)`) in resource_gathering.rs — re-exported outside behaviors module to faction.rs.
- Tunnel Arrival Validation tests live in construction.rs (they test building_tunnel_behavior_system). tunnel_interaction.rs tests that need construction helpers duplicate them locally.
- When splitting monolith files into directory modules, test helper functions in `#[cfg(test)] mod tests` are private to their module — they cannot be shared across sibling test modules. Either duplicate helpers or use `pub(super)` test utility modules.
- E0761 (duplicate module) occurs when both `foo.rs` and `foo/mod.rs` exist. After splitting a monolith into a directory, always remove the original `.rs` file.
- `world.iter_entities()` deprecated — replace with `world_mut().query::<(Entity, &T)>()` + `query.iter(world)`.
- A background linter automatically fixes some import issues between Edit tool calls — watch for it merging duplicate imports or adding missing ones.
- spawn_syndicate_player test helper is duplicated in construction.rs and resource_gathering.rs test modules (both need it).
- When splitting files with the sandbox: must create new directory + files AND delete old file AND compile ALL in a single Bash command. Sandbox restores deleted git-tracked files between separate Bash tool calls.
- 1871 lib tests passing as of this session. QA: 312 passed, 6 failed (pre-existing), 8 ignored.
- `behavior_completion_system` race fix: guard `BaseBehaviorState::None` prevents premature completion of Move/Reverse commands whose behavior system hasn't run yet. Stop is excluded from guard (completes immediately). This fixed 13+ QA test failures ("got Some(Idle)" errors).
- Entity definition system: `src/game/entities/` module with `EntityDefinition`, `UnitDefinition`, `StructureDefinition` traits + `EntityRegistry` resource. Types in `entities/types.rs`, plugin in `entities/registry.rs`. Faction scaffold under `entities/factions/{gdo,syndicate,cults}/` with `units/` and `buildings/` subdirs. `entities/neutral/` at entities level (not in factions/) to stay within 7-item dir limit.
- `game/` directory is at 8 items (over 7-item limit) due to adding `entities/`. This was required by the task — a future refactor could group `types/` + `utils.rs` or consolidate otherwise.
- 1883 lib tests passing as of prior session.
- Peacekeeper pilot entity definition at `entities/factions/gdo/units/peacekeeper.rs`. Pattern: impl `EntityDefinition` + `UnitDefinition`, define constants, static `COMMAND_SLOTS` array, `spawn()` inserts all gameplay components (no visual mesh — callers add mesh separately). Old `peacekeeper_type_data()`/`peacekeeper_attack_data()` in unit_data.rs are now thin wrappers delegating to the definition. Constants re-exported via `pub use`.
- `ObjectInstance` is in `crate::game::types` (not `crate::shared::types`). `UnitCommand` is in `crate::game::units::types` (not `crate::shared::types`).
- Task planner specified path as `src/game/entities/gdo/units/` but actual path is `src/game/entities/factions/gdo/units/`. Always verify paths against actual directory structure.
- Syndicate Agent/Guard entity definitions at `entities/factions/syndicate/units/{agent,guard}.rs`. Agent has 7 command slots (Default + AgentDefault states), uses MELEE_RANGE for attack capability, includes AgentCarryState + TunnelSpaceCost. Guard has 6 slots (Default only, includes AttackGround), uses `attack.range as f32`, no AgentCarryState.
- Entity definition thin wrapper pattern in unit_data.rs: functions delegate to `Def::base_stats()/unit_stats()/attack_data()`, constants become `pub use` re-exports from the entity definition module.
- Registry now has 10 entries: 4 units (Peacekeeper, SupplyChopper, SyndicateAgentDef, SyndicateGuardDef) + 6 structures (DeploymentCenter, PowerPlant, Barracks, ExtractionFacility, ExtractionPlate, SupplyTowerDef).
- GDO building definitions grouped into 3 files (buildings/ dir was at 7-item limit): base.rs (DC+PP), military.rs (BK+ST), extraction.rs (EF+EP). `SupplyTowerDef` (not `SupplyTower`) to avoid name conflict with existing `SupplyTowerState`.
- `StructureInstance` is in `crate::game::types` (re-exported from objects.rs), NOT `crate::shared::types`.
- ExtractionPlate spawn uses `Entity::PLACEHOLDER` for `attached_patch` since `SpawnParams` doesn't carry it — callers needing a real patch should insert `ExtractionPlateState` separately.
- Cults unit entity definitions at `entities/factions/cults/units/{recruit,soldier,gunner}.rs`. All 3 are placeholders (LightInfantry base, no attack data for Soldier/Gunner). Recruit has CultsRecruitMenu command slots (4 slots across 2 states). Soldier/Gunner have standard Default combat slots (5 each). All include OriginatingCenters component.
- Registry now has 15 entries: 7 units (PK, SC, Agent, Guard, Recruit, Soldier, Gunner) + 6 structures (DC, PP, BK, EF, EP, ST) + 2 resources (SpaceCrystalsPatch, SupplyDeliveryStation).
- `register_resource<T: EntityDefinition>()` method added to EntityRegistry — uses `EntityKind::Resource`, takes plain `EntityDefinition` (not Unit/StructureDefinition).
- Neutral entity definitions at `entities/neutral/{space_crystals_patch,supply_delivery_station}.rs`. Both indestructible, use `FactionEnum::Colonists` as placeholder faction, `Owner::neutral()` for actual neutrality. SpawnParams lacks resource-specific fields (amounts, intervals) — spawn uses defaults; callers override components after spawn.
- Syndicate building definitions at `entities/factions/syndicate/buildings/{tunnel,headquarters}.rs`. `TunnelDef` (not `Tunnel`) to follow naming convention. Tunnel cost() returns None (dynamic costs via re-exported `tunnel_construction_cost`/`tunnel_t2_upgrade_cost`/`tunnel_t3_upgrade_cost`). HQ spawn does NOT insert `TunnelExpansionMarker` (callers must add separately). Faction is `FactionEnum::TheSyndicate` (not `Syndicate`).
- Registry now has 20 entries: 7 units + 11 structures (added RC, CultsStorage, CultsArmory) + 2 resources.
- Cults building definitions at `entities/factions/cults/buildings/{recruitment_center,storage,armory}.rs`. `RecruitmentCenterDef`, `CultsStorageDef`, `CultsArmoryDef`. RC cost=None (primary structure). Storage cost=(0 SC, 300 frames). Armory cost=(150 SC, 0 frames). Armory is the only Cults building with Armor component. Storage spawn inserts ConstructionHP+CultsConstructionState when under_construction=true.
- 2112 lib tests passing as of this session.
- `EntityRegistry::cost(&ObjectEnum) -> Option<StructureCost>` convenience method on the registry. All production/construction cost lookups now go through registry instead of hardcoded match arms on state structs.
- `DeploymentCenterState::construction_cost()` and `cancellation_refund()` removed/changed: `cancellation_refund(&self, cost: &StructureCost)` takes pre-looked-up cost instead of doing internal lookup. Same for `ExtractionFacilityState::cancellation_refund()`.
- `BarracksState::production_cost()`, `HeadquartersState::production_cost()`, `SupplyTowerState::production_cost()` removed — use `registry.cost()` instead.
- `attack_states: Query<&AttackState>` folded into `selected_units` query in hotkeys.rs and actions.rs as `Option<&AttackState>` 4th tuple element. This freed a system param slot for `Res<EntityRegistry>`.
- `rebuild_command_panel_ui` folded 4 separate building queries (dc_query, bk_query, ef_query, st_query) into `building_states: Query<(Option<&DC>, Option<&BK>, Option<&EF>, Option<&ST>)>`. Extract via `building_states.get(entity).ok().and_then(|(dc, _, _, _)| dc)`. Saved 3 system params.
- `grid_button_enabled_ext` and `grid_button_enabled` now take `building_states` combined query instead of separate bk_query/st_query params.
- Production tick systems (`barracks_production_tick_system`, `headquarters_production_tick_system`, `supply_tower_production_tick_system`, `dc_construction_tick_system`, `ef_construction_tick_system`) all now take `Res<EntityRegistry>` param. Test apps using `run_system_once` for these need `app.insert_resource(build_registry())`.
- HUD bar now has 3 per-section update systems: `update_portrait_section_system`, `update_info_panel_section_system`, `update_selection_panel_section_system`. Old `update_selected_units_grid_system` and `UnitsGridSection` removed. Portrait queries `PortraitSection`, InfoPanel queries `InfoPanelSection`, SelectionPanel queries `SelectionPanelSection`. All run `.after(command_panel_hotkeys)`.
- 2122 lib tests passing as of this session.
- `get_grid_slot_action()` in grid_layout.rs now uses `EntityRegistry` lookups instead of hardcoded match arms. New signature: `(state, row, col, active_type: Option<ObjectEnum>, registry: &EntityRegistry, ctx: &SlotConditionContext)`. Callers (hotkeys.rs, ui_rebuild.rs) construct `SlotConditionContext` from the same runtime booleans and pass `selection.active_type()`.
- `SlotCondition` enum and `SlotConditionContext` struct in `game/entities/types.rs`. Conditions: `HasQueue`, `HasActiveConstruction`, `TunnelIsUpgrading`, `HasReadyPlate`. Entity defs use `CommandSlotDef::with_condition()` for conditional slots.
- `CommandSlotDef::new(state, row, col, action)` — const fn constructor that defaults condition to None. `CommandSlotDef::with_condition(...)` for conditional slots. All entity def files use these constructors.
- EF command_slots now has 3 entries: `EnterPlacement` with `HasReadyPlate` condition (priority), `EfBuildPlate` unconditional (fallback), `EfCancel` with `HasActiveConstruction`. Slots at same (row,col) — first matching condition wins.
- For Default state grid layout, each unit type defines its own command_slots in the registry. No caps-based filtering needed — the entity type's definition determines available commands. Tests use `test_grid_action_for()` with specific `ObjectEnum` active_type instead of caps.
- `EntityRegistry::spawn()` method dispatches to registered spawn_fn. Returns Entity id. Panics if object not registered.
- `SpawnParams` now has `flip_horizontal: bool, flip_vertical: bool` fields. Entity def spawns apply flip scale to Transform.
- `insert_entity_visuals()` in `game/entities/utils.rs` — centralized visual component insertion (Mesh3d, MeshMaterial3d, child labels) based on ObjectEnum. Called after `registry.spawn()` at all production/construction/placement/setup sites.
- `grid_to_spawn_position(grid_x, grid_z, size_x, size_z)` in `game/entities/utils.rs` — converts grid coords to world Vec3 center position for entity spawning.
- All spawn call sites (production.rs, construction.rs, placement.rs, setup.rs, behaviors/construction.rs, utility.rs, harness.rs) now use registry-based spawning. Old spawn_* functions in utils.rs remain as thin wrappers for scenario/integration tests.
- `TestApp` now includes `EntitiesPlugin` (EntityRegistry resource) for registry-based spawning in tests.
- `placement_click_system` lost `_panel_target: Res<CommandPanelTarget>` param (was unused) to stay within 16-param Bevy limit after adding `Res<EntityRegistry>`.
- 2079 lib tests passing as of this session.
- `compute_selected_unit_capabilities()` does NOT set `owned_by_local_player` — callers must re-apply ownership after overwriting `unit_caps`.
- Info panel vs selection panel icon scoping: `InfoPanelIcon` marker component distinguishes info panel icons from selection panel icons. Info panel queries filter `With<InfoPanelIcon>`.
- When adding DcBuild variants for new structures: add match arm in BOTH `grid_button_label()` and `grid_button_enabled()` in helpers.rs.
