use crate::helpers::*;
use bevy::ecs::system::RunSystemOnce;
use space_crystals::game::units::types::unit_data::{
    guard_type_data, guard_attack_data, GUARD_CONTROL_COST, GUARD_TUNNEL_SPACE_COST, GUARD_RUGGED_BONUS,
};
use space_crystals::game::combat::types::{AttackCapability, AttackType};
use space_crystals::game::units::types::movement::MovementSpeed;
use space_crystals::simulation::{FRAMES_PER_SECOND, SPACE_UNITS_PER_GRID_UNIT};

/// QA Step 1 [auto]: Verify `ObjectEnum` has a `SyndicateGuard` variant
#[test]
fn step_1_object_enum_has_syndicate_guard() {
    // Verify the variant exists and has correct object type properties
    let obj_type = ObjectEnum::SyndicateGuard.object_type();
    assert_eq!(obj_type.name, "Guard");
    assert_eq!(obj_type.size, (36, 36), "Guard silhouette should be 36x36");
    assert!(obj_type.destructible, "Guard should be destructible");
    assert_eq!(obj_type.sight_range, 5, "Guard sight range should be 5");
    assert!(obj_type.groupable, "Guard must be groupable per spec");

    // Verify it's classified as a unit, not a structure
    assert!(ObjectEnum::SyndicateGuard.is_unit(), "SyndicateGuard should be a unit");
    assert!(!ObjectEnum::SyndicateGuard.is_structure(), "SyndicateGuard should not be a structure");
}

/// QA Step 2 [auto]: Verify Guard spawn function sets correct HP (80), armor (1/1),
/// speed (1.25 GU/s), and attack stats (damage 6, range 3)
#[test]
fn step_2_guard_spawn_stats() {
    let mut test_app = TestApp::new();
    test_app.step(); // startup

    // Spawn a Guard using the spawn function via run_system_once
    let guard_entity = test_app.app.world_mut().run_system_once(|
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    | -> Entity {
        space_crystals::game::utils::spawn_syndicate_guard(
            &mut commands, &mut meshes, &mut materials,
            20, 20, Owner(Some(0)),
        )
    }).unwrap();

    // Flush commands
    test_app.step();

    let world = test_app.app.world();

    // Verify HP (80)
    let obj_instance = world.get::<ObjectInstance>(guard_entity).expect("Guard should have ObjectInstance");
    assert_eq!(obj_instance.object_type, ObjectEnum::SyndicateGuard);
    assert_eq!(obj_instance.max_hp, Some(80.0), "Guard max HP should be 80");
    assert_eq!(obj_instance.hp, Some(80.0), "Guard HP should be 80");

    // Verify type data: armor
    let type_data = guard_type_data();
    assert_eq!(type_data.point_armor, 1, "Guard point armor should be 1");
    assert_eq!(type_data.full_armor, 1, "Guard full armor should be 1");
    assert_eq!(type_data.max_hp, 80, "Guard max HP from type data should be 80");

    // Verify movement speed (5 SU/frame * 16 FPS / 64 SU/GU = 1.25 GU/s)
    let expected_speed = 5.0 * (FRAMES_PER_SECOND as f32) / (SPACE_UNITS_PER_GRID_UNIT as f32);
    assert!((expected_speed - 1.25).abs() < 0.001, "Expected speed should be 1.25 GU/s, got {}", expected_speed);
    let move_speed = world.get::<MovementSpeed>(guard_entity).expect("Guard should have MovementSpeed");
    assert!((move_speed.0 - expected_speed).abs() < 0.001,
        "Guard move speed should be {} (1.25 GU/s), got {}", expected_speed, move_speed.0);

    // Verify attack stats
    let attack = world.get::<AttackCapability>(guard_entity).expect("Guard should have AttackCapability");
    assert_eq!(attack.damage, 6.0, "Guard damage should be 6");
    assert_eq!(attack.range, 3.0, "Guard attack range should be 3 GU");
    assert_eq!(attack.min_range, 0.0, "Guard min range should be 0");
    assert!(matches!(attack.attack_type, AttackType::FullyConnected { .. }), "Guard should have FullyConnected attack");

    // Verify attack data details
    let attack_data = guard_attack_data();
    assert_eq!(attack_data.damage, 6);
    assert_eq!(attack_data.range, 3);
    assert_eq!(attack_data.aim_duration, 2);
    assert_eq!(attack_data.firing_duration, 1);
    assert_eq!(attack_data.cooldown_duration, 1);
    assert_eq!(attack_data.reload_duration, 4);

    // Verify constants
    assert_eq!(GUARD_CONTROL_COST, 1);
    assert_eq!(GUARD_TUNNEL_SPACE_COST, 2);
    assert!((GUARD_RUGGED_BONUS - 0.5).abs() < f32::EPSILON);
}
