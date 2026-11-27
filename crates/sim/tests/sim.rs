//! The sim's integration tests, in one test program (slice M5-T): every test file is a module of it, so that an edit
//! to `sc_sim` compiles and links one program rather than one for each file. The timings are a program of their own,
//! `perf.rs`, and so are (M5-X1b) the games on other rules than the built-in ones, `rules.rs`, which change the rules
//! of the whole process. `Cargo.toml` lists all three, with `autotests = false`.
//!
//! - **A test file** `tests/foo.rs` is `mod foo;` here, and a folder `tests/foo/` (its root `main.rs`, its modules
//!   beside it) is `#[path = "foo/main.rs"] mod foo;`, in name order. A test file that no program includes fails
//!   [`test_programs`], which names it.
//! - **One file's tests**: filter by its module, `cargo test -p sc_sim --test sim holding::`. A filter matches any part
//!   of a test's path, so `gathering::` runs `agent_gathering::` and `recruit_gathering::` too.
//! - **One process**: the tests of every file run on the threads of one process. None may change the current folder,
//!   set an environment variable, exit the process or keep global state, and a test that writes files writes them in
//!   a folder of its own.

#[path = "agent_gathering/main.rs"]
mod agent_gathering;
#[path = "agent_supplies/main.rs"]
mod agent_supplies;
#[path = "air/main.rs"]
mod air;
#[path = "air_doors/main.rs"]
mod air_doors;
#[path = "airstrip/main.rs"]
mod airstrip;
#[path = "altar/main.rs"]
mod altar;
#[path = "armory/main.rs"]
mod armory;
#[path = "attack_behind_friends/main.rs"]
mod attack_behind_friends;
mod attack_move_leash;
mod attack_move_own_movement;
mod attack_move_reach;
mod attack_reveal;
#[path = "augmentation/main.rs"]
mod augmentation;
mod behaviours_attack;
mod behaviours_gate;
mod behaviours_idle;
mod behaviours_patrol;
mod behaviours_soak;
mod behaviours_travel;
mod building_armor;
#[path = "builds/main.rs"]
mod builds;
#[path = "cathedral/main.rs"]
mod cathedral;
#[path = "chop_shop/main.rs"]
mod chop_shop;
mod colonists;
mod colonists_start;
mod combat_damage;
mod combat_duel;
mod combat_phases;
mod combat_range;
mod command_gate;
#[path = "construction/main.rs"]
mod construction;
#[path = "conversion/main.rs"]
mod conversion;
#[path = "crushing/main.rs"]
mod crushing;
mod cults_combat;
#[path = "cults_construction/main.rs"]
mod cults_construction;
mod cults_start;
mod debug_commands;
mod default_order;
#[path = "defense_system/main.rs"]
mod defense_system;
#[path = "deploy/main.rs"]
mod deploy;
#[path = "deploy_gate/main.rs"]
mod deploy_gate;
#[path = "design_citations/main.rs"]
mod design_citations;
mod determinism;
#[path = "drone_operator/main.rs"]
mod drone_operator;
mod economy_mining;
#[path = "economy_oracle/main.rs"]
mod economy_oracle;
mod economy_resources;
#[path = "energy/main.rs"]
mod energy;
#[path = "expansions/main.rs"]
mod expansions;
#[path = "factory/main.rs"]
mod factory;
#[path = "field_engineer/main.rs"]
mod field_engineer;
mod fog_delay;
mod follow_itself;
#[path = "game_end/main.rs"]
mod game_end;
#[path = "game_start/main.rs"]
mod game_start;
#[path = "garrison/main.rs"]
mod garrison;
#[path = "gathering/main.rs"]
mod gathering;
mod gathering_pass_through;
#[path = "gathering_rules/main.rs"]
mod gathering_rules;
mod gathering_soak;
mod gdo_kinds;
#[path = "golden/main.rs"]
mod golden;
mod grenadier;
mod group_destinations;
mod gunner;
#[path = "hand_of_heaven/main.rs"]
mod hand_of_heaven;
#[path = "holding/main.rs"]
mod holding;
mod invariants_overlap;
mod invariants_state;
mod known_from_the_start;
#[path = "losing_tunnels/main.rs"]
mod losing_tunnels;
#[path = "movement_collision/main.rs"]
mod movement_collision;
mod movement_crowds;
#[path = "movement_drag/main.rs"]
mod movement_drag;
mod movement_separate;
mod movement_soak;
mod movement_stuck;
#[path = "movement_yield/main.rs"]
mod movement_yield;
mod nav;
mod nav_grid;
mod nav_serve;
#[path = "oriented/main.rs"]
mod oriented;
#[path = "pairings/main.rs"]
mod pairings;
#[path = "path_knowledge/main.rs"]
mod path_knowledge;
mod placement;
mod playtest_map;
#[path = "popup/main.rs"]
mod popup;
#[path = "prayer_mobile/main.rs"]
mod prayer_mobile;
#[path = "prerequisites/main.rs"]
mod prerequisites;
#[path = "production/main.rs"]
mod production;
mod queued_group_destinations;
#[path = "radar/main.rs"]
mod radar;
#[path = "recruit_gathering/main.rs"]
mod recruit_gathering;
#[path = "recruitment/main.rs"]
mod recruitment;
mod remembered_targets;
#[path = "repair/main.rs"]
mod repair;
#[path = "research/main.rs"]
mod research;
mod resources;
mod resources_block;
#[path = "riot_shield/main.rs"]
mod riot_shield;
mod robustness;
#[path = "rocketman/main.rs"]
mod rocketman;
mod scenario_files;
mod scenario_problems;
mod scenarios;
mod shields;
#[path = "shrine/main.rs"]
mod shrine;
#[path = "shroud/main.rs"]
mod shroud;
mod skirmish_soak;
mod starting_exploration;
mod structures;
mod structures_vision;
mod symmetry;
mod syndicate;
mod syndicate_kinds;
mod syndicate_line;
mod syndicate_soak;
mod syndicate_start;
mod target_lost;
mod target_priority;
mod test_programs;
#[path = "tracked/main.rs"]
mod tracked;
#[path = "tunnel_network/main.rs"]
mod tunnel_network;
mod unit_moves;
mod unit_orders;
#[path = "upgrades/main.rs"]
mod upgrades;
#[path = "vehicle_assembly/main.rs"]
mod vehicle_assembly;
#[path = "vehicles/main.rs"]
mod vehicles;
mod vision;
mod vision_memory;
#[path = "warp_in/main.rs"]
mod warp_in;
