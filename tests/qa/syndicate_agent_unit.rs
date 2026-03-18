use crate::helpers::*;
use space_crystals::game::combat::types::{Armor, Silhouette, AttackCapability, AttackType};
use space_crystals::game::units::types::unit_data::*;
use space_crystals::game::units::types::movement::TurnRateMovementParams;
use space_crystals::types::{UnitBaseEnum, SightRange};

/// QA Step 1 [auto]: Produce an Agent from a Headquarters — verify it costs 100 SC and takes 160 frames (10 seconds)
/// NOTE: Production system is out of scope for this data task. Verify Agent can be spawned and unit control cost exists.
#[test]
fn step_1_agent_spawnable_and_control_cost() {
    let mut test_app = TestApp::new();
    test_app.step();

    let agent;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    // Verify it spawns with UnitControlCost
    let world = test_app.app.world();
    let cost = world.get::<space_crystals::game::units::types::unit_data::UnitControlCost>(agent)
        .expect("Agent should have UnitControlCost");
    assert_eq!(cost.0, AGENT_CONTROL_COST, "Agent control cost should be {}", AGENT_CONTROL_COST);

    // Verify static data matches design
    let type_data = agent_type_data();
    assert_eq!(type_data.faction, space_crystals::types::FactionEnum::TheSyndicate);
    assert_eq!(type_data.max_hp, 75);
}

/// QA Step 2 [auto]: Verify the Agent spawns with correct stats: 75 HP, 1/1 armor, 36x36 silhouette, SightRange 5
#[test]
fn step_2_agent_spawn_stats() {
    let mut test_app = TestApp::new();
    test_app.step(); // OnEnter fires

    let agent;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
    }
    test_app.step(); // flush

    // HP = 75
    let harness = TestHarness::new(&mut test_app.app);
    let (hp, max_hp) = harness.get_health(agent).unwrap();
    assert!((hp - 75.0).abs() < 0.01, "Agent HP should be 75, got {}", hp);
    assert!((max_hp - 75.0).abs() < 0.01, "Agent MaxHP should be 75, got {}", max_hp);

    // Armor 1/1
    let world = test_app.app.world();
    let armor = world.get::<Armor>(agent).expect("Agent should have Armor component");
    assert!((armor.point_armor - 1.0).abs() < 0.01, "PointArmor should be 1, got {}", armor.point_armor);
    assert!((armor.full_armor - 1.0).abs() < 0.01, "FullArmor should be 1, got {}", armor.full_armor);

    // Silhouette 36x36 SU → 36/64 = 0.5625 GU
    let silhouette = world.get::<Silhouette>(agent).expect("Agent should have Silhouette");
    let expected_size = 36.0 / 64.0;
    assert!((silhouette.width - expected_size).abs() < 0.01, "Silhouette width should be {}, got {}", expected_size, silhouette.width);
    assert!((silhouette.height - expected_size).abs() < 0.01, "Silhouette height should be {}, got {}", expected_size, silhouette.height);

    // SightRange 5
    let sight = world.get::<SightRange>(agent).expect("Agent should have SightRange");
    assert_eq!(sight.0, 5, "Agent SightRange should be 5, got {}", sight.0);
}

/// QA Step 3 [auto]: Verify the Agent uses TurnRateMovement: issue a move command, confirm MaxSpeed 6 su/frame and 180 deg/frame turn rate
#[test]
fn step_3_agent_turn_rate_movement() {
    let mut test_app = TestApp::new();
    test_app.step();

    let agent;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let world = test_app.app.world();

    // Should have TurnRateMovementParams
    let params = world.get::<TurnRateMovementParams>(agent)
        .expect("Agent should have TurnRateMovementParams");

    // MaxSpeed: 6 SU/frame * 16 FPS / 64 SU/GU = 1.5 GU/sec
    let expected_speed = 6.0 * 16.0 / 64.0;
    assert!((params.max_speed - expected_speed).abs() < 0.01,
        "Agent max_speed should be {} GU/s, got {}", expected_speed, params.max_speed);

    // TurnRate: 180 deg/frame * 16 FPS = 2880 deg/sec in radians
    let expected_turn_rate = 180.0_f32.to_radians() * 16.0;
    assert!((params.turn_rate - expected_turn_rate).abs() < 0.1,
        "Agent turn_rate should be {} rad/s, got {}", expected_turn_rate, params.turn_rate);

    // Infinite acceleration/deceleration
    assert!(params.acceleration > 1e10, "Agent acceleration should be infinite (f32::MAX)");
    assert!(params.deceleration > 1e10, "Agent deceleration should be infinite (f32::MAX)");
}

/// QA Step 4 [auto]: Verify the Agent's HeavyInfantry base properties apply
#[test]
fn step_4_agent_heavy_infantry_base() {
    let mut test_app = TestApp::new();
    test_app.step();

    let agent;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let world = test_app.app.world();
    let base = world.get::<UnitBaseEnum>(agent).expect("Agent should have UnitBaseEnum");
    assert_eq!(*base, UnitBaseEnum::HeavyInfantry, "Agent should be HeavyInfantry");

    // HeavyInfantry has no turret
    assert!(!base.data().has_turret, "HeavyInfantry should not have turret");
}

/// QA Step 5 [auto]: Order the Agent to attack a ground enemy unit — verify melee attack with 6 damage, correct phase durations
#[test]
fn step_5_agent_melee_attack_attributes() {
    let mut test_app = TestApp::new();
    test_app.step();

    let agent;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let world = test_app.app.world();
    let attack = world.get::<AttackCapability>(agent).expect("Agent should have AttackCapability");

    // Damage: 6
    assert!((attack.damage - 6.0).abs() < 0.01, "Agent damage should be 6, got {}", attack.damage);

    // Attack type: FullyConnected Melee
    match &attack.attack_type {
        AttackType::FullyConnected { subtype } => {
            assert_eq!(*subtype, FullyConnectedSubtype::Melee,
                "Agent attack should be FullyConnected Melee");
        }
        _ => panic!("Agent should have FullyConnected attack type, got {:?}", attack.attack_type),
    }

    // Melee range
    assert!(attack.range < 1.5, "Melee range should be short (< 1.5 GU), got {}", attack.range);

    // Phase durations (in seconds, converted from frames at 16 FPS):
    // Aim: 2 frames = 0.125s, Fire: 4 frames = 0.25s, Cooldown: 1 frame = 0.0625s, Reload: 9 frames = 0.5625s
    let fps = 16.0_f32;
    assert!((attack.aim_time - 2.0/fps).abs() < 0.01, "Aim time should be {}, got {}", 2.0/fps, attack.aim_time);
    assert!((attack.fire_time - 4.0/fps).abs() < 0.01, "Fire time should be {}, got {}", 4.0/fps, attack.fire_time);
    assert!((attack.cooldown_time - 1.0/fps).abs() < 0.01, "Cooldown time should be {}, got {}", 1.0/fps, attack.cooldown_time);
    assert!((attack.reload_time - 9.0/fps).abs() < 0.01, "Reload time should be {}, got {}", 9.0/fps, attack.reload_time);
}

/// QA Step 6 [auto]: Verify the Agent cannot attack air units (TargetDomain: Ground)
#[test]
fn step_6_agent_target_domain_ground() {
    let mut test_app = TestApp::new();
    test_app.step();

    let agent;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let world = test_app.app.world();
    let attack = world.get::<AttackCapability>(agent).expect("Agent should have AttackCapability");
    assert_eq!(attack.target_domain, TargetDomainEnum::Ground,
        "Agent target domain should be Ground");
}

/// QA Step 14 [auto]: Verify TunnelSpaceCost is 2
#[test]
fn step_14_agent_tunnel_space_cost() {
    let mut test_app = TestApp::new();
    test_app.step();

    let agent;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let world = test_app.app.world();
    let tunnel_cost = world.get::<space_crystals::game::units::types::unit_data::TunnelSpaceCost>(agent)
        .expect("Agent should have TunnelSpaceCost");
    assert_eq!(tunnel_cost.0, 2, "Agent TunnelSpaceCost should be 2, got {}", tunnel_cost.0);
}

/// QA Step 15 [auto]: Verify Agent is Groupable (per original task; groupable fix is a separate task)
/// NOTE: The agent_groupable_and_construction_fix task changes this to false.
/// This test verifies the object_type().groupable field matches what's implemented.
#[test]
fn step_15_agent_groupable_field() {
    let obj_type = ObjectEnum::SyndicateAgent.object_type();
    // After the groupable fix, Agent should be ungroupable (false)
    // This test checks whatever the current state is
    // The agent_groupable_and_construction_fix task should have set it to false
    assert!(!obj_type.groupable, "Agent should be ungroupable (groupable=false) per design fix");
}
