# QA Agent Memory

## Key Bugs
- **Memory leak / OOM crash**: FIXED as of 2026-03-09. User confirms game runs smoother, no crash.
- **Black line glitch — wrong root cause**: Not a viewport boundary seam. Line is 5 tiles long, centered horizontally, 10 grid squares above bottom HUD. Still present. Failed QA again 2026-03-09.
- **Patrol broken**: Units stop short of waypoint, no return leg. Affects all unit types.
- **Health display stale**: HUD HP only updates on re-selection, not real-time during combat.
- **Syndicate Agent spawn broken**: Agent spawns under Tunnel with no Move command. Blocks ~6 Syndicate tasks.
- **Attack mode left-click broken**: In AwaitingTarget[Attack], clicking enemy doesn't register attack. Repeat failure.
- **Info panel stale on control group switch**: Command panel updates but info panel/portrait don't. Forum topic opened.

## Fixed Bugs (2026-03-08 session)
- Vision centering on units — FIXED, units reveal fog correctly
- Phantom command panels on empty tiles — FIXED
- Faction selection screen Syndicate HQ missing — FIXED

## Automated QA Test Generation
- **Test infrastructure**: `tests/qa/main.rs` + `tests/qa/helpers.rs`. Modules registered between `// --- QA MODULES START/END ---` markers.
- **FixedUpdate doesn't fire** in headless TestApp (MinimalPlugins). Use `app.world_mut().run_system_once(system_fn)` to trigger FixedUpdate systems directly.
- **pub(crate) systems** can't be called from integration tests. Replicate logic in inline closures for `run_system_once`.
- **Float precision**: N additions of (1.0/N) doesn't reach exactly 1.0 for most N. Use build_frames=10 or add extra ticks.
- **Enemy spawn interference**: GDO game start spawns enemy Peacekeepers at grid (50,50) for player 1. Avoid that area in player 1 tests.
- **Successfully generated tests for**: fog_of_war_centering_fix (4), construction_hp_rule (6), syndicate_agent_unit (8), agent_groupable_and_construction_fix (4), tunnel_structure_and_network (11), elevation_modifier (7), attack_phases (8), unit_cap_systems (4), grid_coverage_full_map (2), resource_entity_selectability_fix (5), box_selection_priority (10), fix_units_moving_while_attacking (10), ground_unit_collision (10), autonomous_targeting (6), combat_behaviors (10), movement_behaviors (5), air_unit_soft_separation (5), selection_system_and_control_groups (11), selection_panel (2), tunnel_area_and_construction_rules (18), faction_display_hud (2), pathfinding_diagonal_and_oscillation_fix (5), enter_command_and_entering_tunnel_behavior (5), worker_built_structure_arrival_validation (6), fix_left_click_command_target (4), basic_combat_unit_interface_state (14), command_panel_and_interface_state_machine (9), back_button_hotkey_consistency (9), tunnel_object_interface_state (22), agent_object_interface_state (16), gdo_supply_tower_and_chopper (17), agent_tunnel_building_command_and_behavior (9), agent_resource_gathering_commands_and_behaviors (10), headquarters_stats_correction (4), guard_unit (2). Total: ~238 QA tests across 35 task files.
- **ObjectInstance fields**: `object_type: ObjectEnum` (not `object_enum`), `max_hp: Option<f32>`, `hp: Option<f32>`.
- **Escape key doesn't work in headless tests**: `send_key_press(KeyCode::Escape)` + `test_app.step()` does NOT trigger `command_panel_hotkeys`. The system likely has preconditions (CursorOverUi state, selection context) not met in headless mode. Right-click navigation via CursorTarget + MouseButtonInput DOES work. Workaround: test Escape transitions via direct state manipulation; defer keyboard verification to human review.
- **Input simulation helpers**: `send_mouse_press(app, button)` and `send_key_press(app, key_code)` in helpers.rs. Use event-based input (MouseButtonInput/KeyboardInput) — direct `ButtonInput::press()` gets cleared by Bevy's input system in PreUpdate before game systems run in Update.
- **DC blocks pathfinding**: GDO Deployment Center at grid (30,30) blocks tiles (31-34, 30-33). Grid (32,32) = world (0.5,0,0.5) is BLOCKED. Never use as pathfinding target.
- **Grid-to-world offset**: Don't assume grid (x,z) = world (x,z). Map applies center offset. Unit at grid (21,21) → world (-10.5,-10.5).
- **GdoPlayerResources is Component, not Resource**: Stored on Player entities. Use `Query<(&Player, &GdoPlayerResources)>`.
- **TunnelArea testable standalone**: `TunnelArea::new()`, `recalculate()`, `fits_expansion()`, `overlaps()` work without ECS context.
- **AttackState field name**: `current_target: Option<AttackTarget>`, NOT `target: Option<Entity>`. Use `AttackTarget::UnitTarget(entity)`.
- **Selection API**: `Selection::build_from_entities(&mut self, &[(Entity, ObjectEnum, bool)])` — 3rd element is groupable flag.
- **Private module workaround**: `resources` module is private (`mod resources`). Use inline closures with `run_system_once` to replicate system logic. Import `bevy::ecs::system::RunSystemOnce` trait.
- **Naming**: `ObjectEnum::Peacekeeper` (not GDOPeacekeeper). `Owner(Some(0))` for player 0. `FactionEnum::TheSyndicate` (not `Syndicate`).
- **Bevy 0.17 Entity API**: `Entity::from_raw_u32(n).unwrap()` (not `from_raw(n)`).
- **Non-buildable tile type**: `TilePresetEnum::Mountain` (not `Rock` — no Rock variant).
- **QA-automation tasks can self-QA**: User said automation-related tasks can be QA'd by attempting to use the new facilities to QA other tasks. Track new automation capabilities as they land.

## Active Directives
- **Bevy migration — skip QA**: All Bevy-related tasks skip QA. Developer sends them to `/completed_tasks` directly. If any land in `/qa_tasks`, move to `/completed_tasks` with a note. Full end-to-end QA pass planned post-migration (user-initiated).

## Patterns
- Memory leak fixed — sessions no longer time-limited
- GDO core gameplay loop works: build area, structures, extraction, power grid, unit production, combat
- Syndicate side blocked: HQ shows unit commands instead of Agent production, tunnel hotkeys broken
- User prefers efficient QA — update tickets in real-time, don't wait until end of session
- Document bugs immediately when user reports them, don't defer
- Test count: ~1386 total (1150 unit + 209 QA + 27 integration), 0 known failures
- **Bevy 0.17 broke some test files**: `testing` feature removed. Some test files use pre-migration APIs. Affected: tunnel_structure_and_network, resource_entity_selectability_fix, combat_behaviors. Compile from cache but will fail on recompile.
- **Spawnable test units**: Peacekeeper (infantry), SyndicateAgent (worker), SupplyChopper (HoverCraft/Air), SyndicateGuard (HeavyInfantry/Ranged). No turret, Glider, or CanReverse unit available.
- Remaining partially-auto tasks: autonomous_targeting (4 steps need turret), combat_behaviors (1 needs Glider), movement_behaviors (3 need Glider/CanReverse)

## Current State (2026-03-09, post-session)
- 9 tasks passed, 7 failed back to developer this session
- Remaining in qa_human_review: tunnel_object_interface_state, agent_object_interface_state, worker_built_structure_arrival, agent_resource_gathering, agent_tunnel_building (all Syndicate-blocked except tunnel menus)
- Remaining in qa_tasks: agent_groupable_and_construction_fix, syndicate_hq_production_interface, tunnel_expand_menu_hotkeys (all Syndicate-blocked)
- Also in qa_tasks/developer_tasks: viewport_black_line, resource_selectability, tunnel_area (bounced back)
- Syndicate Agent spawn issue is the #1 unblock needed
- Test count: ~1414 total (1150 unit + 237 QA + 27 integration), 0 known failures
