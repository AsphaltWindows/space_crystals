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
- Unit types defined via ObjectEnum variants + type_data()/attack_data() functions + spawn_*() functions
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
- 1779 lib tests passing as of this session.
- `rebuild_command_panel_ui` and `command_panel_hotkeys` are at 16 system params (Bevy limit). To add cross-cutting guards (like ownership), use a resource field (e.g. `SelectedUnitCapabilities.owned_by_local_player`) set by an earlier system rather than adding query params.
- `handle_command_button_clicks` is also at 16 params. Ownership guard is handled indirectly — if panel buttons aren't rendered (by `rebuild_command_panel_ui` guard), there's nothing to click.
- TestApp::new_with_faction() (not TestApp::new(faction)) for faction-specific test apps. Owner::player(id) (not Owner::new(id)). GDO selected = player 0; DC is auto-spawned by init_faction_resources (adds DC_POWER=20 to power_generated).
- `OriginatingCenters` component in `game/units/types/unit_data.rs` — tracks which RC(s) produced a Cults unit. `cults_unit_death_tracking_system` in combat/systems/core.rs decrements RC `local_used` on unit death. Runs `.before(remove_dead_entities_system)` in CombatPlugin.
- RC auto-production: `recruitment_center_production_system` and `cults_unit_control_aggregation_system` in game/world/faction.rs, registered in FixedUpdate DiagCategory::Construction. Production: 12s base (192 frames), scales inversely with effectiveness.
- `CultsRecruit` variant in ObjectEnum; `spawn_cults_recruit()` in game/utils.rs — minimal placeholder (no attack, no movement params). `RECRUIT_MAX_HP = 50.0`.
- In faction.rs tests: `Unit` is `crate::types::Unit` (not `crate::game::units::types::Unit`). `GridMap` is `crate::game::world::types::GridMap`.
- `StructureMenuState::RecruitmentCenterMenu` for RC interface. `RcCancel` action resets `production_progress` to 0. RC `rally_point` is `Option<Vec3>` (not `Option<RallyTarget>` like BK/HQ/ST). `bk_hq_query` now includes `Option<&mut RecruitmentCenterState>` as 4th tuple element with `With<RecruitmentCenterState>` in Or filter.
- `resources.rs` has `interface_state_validation_system` that requires match exhaustion for all `StructureMenuState` variants — must add any new variant there.
- `selected_owners` query in `rebuild_command_panel_ui` now has 3 tuple elements: `(&Owner, Option<&HeadquartersState>, Option<&RecruitmentCenterState>)`. When destructuring, use `(owner, hq, rc)` or `(owner, ..)`.
- 1830 lib tests passing as of this session.
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
- 1866 lib tests passing as of this session.
