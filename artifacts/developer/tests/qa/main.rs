// QA-generated integration tests.
// The QA agent appends `mod [task_name];` lines between the marker comments
// as it generates test files for each QA task.
#[allow(unused_imports)]
mod helpers;

// --- QA MODULES START ---
mod fog_of_war_centering_fix;
mod construction_hp_rule;
mod syndicate_agent_unit;
mod agent_groupable_and_construction_fix;
mod tunnel_structure_and_network;
mod elevation_modifier;
#[allow(unused)]
mod attack_phases;
mod unit_cap_systems;
mod damage_calculation_and_directional_armor;
mod grid_coverage_full_map;
mod resource_entity_selectability_fix;
#[allow(unused)]
mod box_selection_priority;
#[allow(unused)]
mod fix_units_moving_while_attacking;
#[allow(unused)]
mod ground_unit_collision;
#[allow(unused)]
mod autonomous_targeting;
#[allow(unused)]
mod combat_behaviors;
#[allow(unused)]
mod movement_behaviors;
mod air_unit_soft_separation;
mod selection_system_and_control_groups;
#[allow(unused)]
mod tunnel_expansions_and_starting_condition;
mod selection_panel;
mod tunnel_area_and_construction_rules;
mod faction_display_hud;
mod pathfinding_diagonal_and_oscillation_fix;
mod enter_command_and_entering_tunnel_behavior;
mod worker_built_structure_arrival_validation;
mod fix_left_click_command_target;
mod basic_combat_unit_interface_state;
mod command_panel_and_interface_state_machine;
mod back_button_hotkey_consistency;
#[allow(unused)]
mod agent_object_interface_state;
mod tunnel_object_interface_state;
mod gdo_supply_tower_and_chopper;
mod agent_tunnel_building_command_and_behavior;
mod fix_memory_leak_oom_freeze;
mod viewport_black_line_glitch;
mod agent_resource_gathering_commands_and_behaviors;
mod headquarters_stats_correction;
mod guard_unit;
mod automated_qa_ui_state_queries;
// --- QA MODULES END ---
