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
- `cargo test --lib` works. `cargo test` (all targets) fails on `tests/qa/automated_qa_ui_state_queries.rs` with unresolved imports — pre-existing issue, likely needs `testing` feature or module restructure.
- Cargo.toml was previously pinned at bevy "0.14" but code was written for 0.17 APIs (e.g. `Entity::from_raw_u32`). Now corrected to "0.17".

## Build Rules
- **Never run `cargo clean`** — always use incremental builds (operator directive)

## Combat Targeting Patterns
- `can_threaten(cap, domain)` in core.rs — checks if target's AttackCapability.target_domain can hit the defender's DomainEnum. Returns false for None (unarmed).
- `select_best_target(candidates)` in combat/utils.rs — 3-tier priority selection: threatening > least rotation > closest. Used by base_auto_target_system and hold_position_behavior_system.
- `is_valid_target(obj, vis, domain, attack_domain)` in combat/utils.rs — filters: destructible + visible + domain-compatible.
- `base_auto_target_system` only activates for Idle and HoldPosition (AttackMove removed — has own scanning).
- Idle units scan within SightRange; HoldPosition uses weapon range.

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
- QA integration tests (`tests/qa/`) have broken imports — `assert_command_visible`, `assert_interface_state`, etc. not found
- `PointerIndicator` component in `ui/types.rs`; pointer spawn/update systems and `PointerDisplayType::indicator_color()` impl in `ui/utils.rs`
- `ui/` directory is at 7 items (the limit) — any new files require reorganizing into subdirectories
