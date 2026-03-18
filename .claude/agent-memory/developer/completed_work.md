# Completed Work Log

Comprehensive list of all completed developer tasks, for reference.

## Core Types & Infrastructure
- `simulation_core_types_and_constants` — SimulationCorePlugin, GridUnit, SpaceUnit, constants
- `entity_hierarchy_and_invisible_entities` — VisibleEntity/InvisibleEntity markers, consolidated FactionEnum
- `faction_resource_definitions` — renamed fields, stockpile/capacity doc comments, 16 tests
- `resource_types` — is_destructible(), 36 tests
- `tile_placement` — MAX_ELEVATION, TilePlacement::new(), 8 tests
- `object_and_structure_types_and_instances` — InfoPanel trait, validate_size(), 17 tests
- `unit_type_and_instance` — CommandQueue, placeholder state components, 19 tests
- `unit_commands_and_command_state` — AttackMove/Reverse, VecDeque, is_available(), 16 tests
- `behavior_states_and_action_channels` — BaseBehaviorState enum, 5 action channels, 27 tests
- `attack_attributes_types_and_targeting` — AttackTypeEnum properties, AttackTarget, 38 tests

## Combat & Units
- `elevation_modifier` — ElevationMap, elevation_modifier(), combat integration, 13 tests
- `fullyconnected_melee_subtype` — FullyConnectedSubtype, MELEE_RANGE, 8 tests
- `damage_calculation_and_directional_armor` — Armor, Silhouette, DamageEvent enum, 21 tests
- `attack_phases` — PhaseActionConstraints, is_interruptible(), turret-aware movement, command gates, 19 tests
- `autonomous_targeting` — split auto_target into turret_autonomous_scanning + base_auto_target, can_threaten stub, 19 tests
- `combat_behaviors` — 5 behavior systems (AttackingObject/Location, AttackMove, HoldPosition, PatrolScanning), 22 tests
- `movement_behaviors` — 4 behavior systems (MovingToLocation/Object, Reversing, Stopping), MoveObjectTarget, 26 tests
- `fix_units_moving_while_attacking` — clear_movement_state helpers, Aiming phase guard, 13 tests

## GDO Faction
- `gdo_peacekeeper` — verification-only, 14 new tests
- `gdo_build_area_and_deployment_center` — verification-only, unblocks extraction_facility
- `gdo_power_plant_and_barracks` — rally point, unit control check, unit_control_cost(), 28 tests
- `gdo_extraction_facility_and_plate` — has_plate fix, plate destruction cleanup, HUD SC display, 8 tests
- `gdo_supply_tower_and_chopper` — spawn functions, DC build menu, production tick, right-click context, 23+15 tests

## Syndicate Faction
- `syndicate_agent_unit` — SyndicateAgent ObjectEnum, TunnelSpaceCost, spawn function, 22 tests
- `tunnel_structure_and_network` — ObjectEnum::Tunnel, TunnelTier, TransitTier, TunnelState, 30 tests
- `tunnel_area_and_construction_rules` — TunnelArea, cost functions, validate_tunnel_upgrade, 35 tests
- `tunnel_expansions_and_starting_condition` — Headquarters, TunnelExpansionMarker, HeadquartersState, 25 tests
- `enter_command_and_entering_tunnel_behavior` — Enter(Entity) UnitCommand, EnteringTunnelBehavior, 22 tests
- `syndicate_hq_production_interface` — HeadquartersMenu state, HqTrain/HqCancel, production tick, 17 tests
- `agent_tunnel_building_command_and_behavior` — BuildingTunnelBehavior, BuildTunnelPhase, spawn_tunnel_under_construction, 21 tests

## UI & Selection
- `structure_flipping` — flip fields, F/G hotkeys, oriented_labels(), 15 tests
- `selection_system_and_control_groups` — Selection resource, SelectionGroup, Tab cycling, recall-and-center, 27 tests
- `faction_display_hud` — ResourceBarField enum, faction-aware HUD, 19 tests
- `basic_combat_unit_interface_state` — SelectedUnitCapabilities, conditional grid slots, 22 tests
- `command_panel_and_interface_state_machine` — ObjectInterfaceState, CursorTarget, StructureMenuState, 35+ tests
- `tunnel_object_interface_state` — 4 states, 5 actions, EjectionQueue, expansion placement, 25 tests
- `agent_object_interface_state` — AgentMenuState, AgentCarryState, Gather/DropOff/BuildTunnel commands, 29 tests
- `command_indicators` — CommandIndicator component, sync system, color-coded, 19 tests
- `selection_panel` — portrait click system (left/shift/ctrl/alt), 11 tests

## World & Map
- `fog_of_war_visibility_states` — SightRange, FogOfWarMap, LastKnownStructures, vision systems, 25 tests
- `fog_of_war_centering_fix` — vision_center() helper, multi-tile structure offset, 8 tests
- `building_placement_visibility_check` — footprint_is_visible(), can_place_building fog check, 12 tests
- `construction_hp_rule` — ConstructionHP component, under_construction(), tick system, 18 tests
- `unit_cap_systems` — cap constants, death decrement, 14 tests
- `power_grid_system` — verification only, 13 new tests

## Bug Fixes
- `remove_wasd_camera_movement` — arrow-keys-only camera panning
- `fix_left_click_command_target` — CommandMode guard, 5 tests
- `viewport_black_line_glitch` — .ceil() fix for viewport rounding
- `fix_selection_click_offset` (v1+v2) — screen-space projection, viewport_offset(), 8+8 tests
- `box_selection_priority` — 5-tier drag-box priority, closest_to_center, 10 tests
- `phantom_command_panel_deployment_center` — Or<With> filter on selection_group_sync_system, 7 tests
- `fix_units_not_providing_vision` — grid_position_sync_system for moving units, 9 tests
- `pathfinding_diagonal_and_oscillation_fix` — 8-dir A*, octile heuristic, corner-cutting prevention, 17 tests
- `fix_memory_leak_oom_freeze` — RepathAttempts, mesh caching, diagnostics, 14 tests
- `ground_unit_collision` — OccupancyMap, find_path occupancy, AABB collision, NeedsRepath, 14 tests

## Infrastructure & Testing
- `headless_test_app_infrastructure` — lib.rs, shared/ reorg, TestApp, 2 integration tests
- `test_harness_commands` — TestHarness struct, spawn/select/command methods, StatesPlugin fix, 14 integration tests
- `per_system_performance_diagnostics` — DiagCategory SystemSet, PerformanceMetrics, F3 overlay, 19 tests
- `game_app_state_machine` — AppState enum (Menu/InGame), all systems gated, 4 tests
- `faction_selection_screen` — SelectedFaction resource, menu UI, faction-conditional spawning, 14 tests
- `worker_built_structure_arrival_validation` — Build UnitCommand, BuildingStructureBehavior, 25 tests

## Bevy 0.17 Migration
- `bevy_017_phase1_compilation_fix` — 316+ errors fixed, 60/62 tests pass
- `bevy_017_phase2_warning_cleanup` — zero warnings
- `bevy_017_phase3_ecs_architecture_refactor` — configure_sets(), DespawnOnExit, single() pattern, 3 tests
- `bevy_017_phase4_unit_system_refactor` — system ordering, emissive→LinearRgba, iter()→&query, 1 test
- `bevy_017_phase5_integration_testing` — 1431 tests pass, 0 failures
- Phase 4 verification passes (world, ui, game_types, combat, shared_testing, simulation, top_level) — all compliant
