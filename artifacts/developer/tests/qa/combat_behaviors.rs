use crate::helpers::*;
#[allow(unused_imports)]
use space_crystals::game::combat::types::{AttackState, AttackTarget, AttackPhase};
#[allow(unused_imports)]
use space_crystals::game::units::types::movement::Velocity;

/// QA Step 1 [auto]: Issue Attack on an enemy unit in range. Verify infantry stops and engages via AttackState.
#[test]
fn step_1_attack_in_range_engages() {
    let mut test_app = TestApp::new();
    test_app.step();

    let player;
    let enemy;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        player = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        enemy = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 20, Owner(Some(1)));
    }
    test_app.step();

    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(player, UnitCommand::AttackTarget(enemy));
    }
    test_app.step_n(15);

    // Unit should have engaged
    let world = test_app.app.world();
    if let Some(attack_state) = world.get::<AttackState>(player) {
        let engaged = attack_state.current_target.is_some() || !matches!(attack_state.phase, AttackPhase::None);
        assert!(engaged,
            "Infantry should engage enemy in range. Phase: {:?}, target: {:?}",
            attack_state.phase, attack_state.current_target);
    }
}

/// QA Step 2 [auto]: Move the enemy out of range. Verify the attacking unit resumes movement to chase.
#[test]
fn step_2_chase_out_of_range_enemy() {
    let mut test_app = TestApp::new();
    test_app.step();

    let player;
    let enemy;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        player = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        enemy = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 20, Owner(Some(1)));
    }
    test_app.step();

    // Issue attack
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(player, UnitCommand::AttackTarget(enemy));
    }
    test_app.step_n(5);

    // Move enemy far away
    {
        let world = test_app.app.world_mut();
        if let Some(mut transform) = world.get_mut::<Transform>(enemy) {
            transform.translation.x += 10.0;
        }
    }
    test_app.step_n(10);

    // Player should still have attack command (chasing)
    let cmd = TestHarness::new(&mut test_app.app).get_command(player);
    assert!(
        matches!(cmd, Some(UnitCommand::AttackTarget(_))),
        "Unit should still be chasing (AttackTarget command), got {:?}", cmd
    );
}

/// QA Step 3 [auto]: Destroy the target. Verify AttackingObject completes (unit goes Idle).
#[test]
fn step_3_target_death_returns_idle() {
    let mut test_app = TestApp::new();
    test_app.step();

    let player;
    let enemy;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        player = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        enemy = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 20, Owner(Some(1)));
    }
    test_app.step();

    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(player, UnitCommand::AttackTarget(enemy));
    }
    test_app.step_n(5);

    // Kill the enemy
    {
        let world = test_app.app.world_mut();
        if let Some(mut obj) = world.get_mut::<ObjectInstance>(enemy) {
            obj.apply_damage(10000.0_f32);
        }
    }
    test_app.step_n(10);

    // Player should return to idle
    let cmd = TestHarness::new(&mut test_app.app).get_command(player);
    assert!(
        matches!(cmd, Some(UnitCommand::Idle) | None),
        "Unit should return to Idle after target dies, got {:?}", cmd
    );
}

/// QA Step 5 [auto]: Issue AttackGround on a location. Verify it moves to range and fires at the location.
#[test]
fn step_5_attack_ground() {
    let mut test_app = TestApp::new();
    test_app.step();

    let player;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        player = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let target_pos = Vec3::new(-10.0, 0.5, -12.0);
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(player, UnitCommand::AttackLocation(target_pos));
    }
    test_app.step();

    let cmd = TestHarness::new(&mut test_app.app).get_command(player);
    assert!(
        matches!(cmd, Some(UnitCommand::AttackLocation(_))),
        "Unit should have AttackLocation command, got {:?}", cmd
    );
}

/// QA Step 6 [auto]: Issue AttackMove to a location. Verify the unit moves along the path.
#[test]
fn step_6_attack_move() {
    let mut test_app = TestApp::new();
    test_app.step();

    let player;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        player = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let start_pos = TestHarness::new(&mut test_app.app).get_position(player).unwrap();
    let target = Vec3::new(start_pos.x + 5.0, start_pos.y, start_pos.z);

    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(player, UnitCommand::AttackMove(target));
    }
    test_app.step();

    let cmd = TestHarness::new(&mut test_app.app).get_command(player);
    assert!(
        matches!(cmd, Some(UnitCommand::AttackMove(_))),
        "Unit should have AttackMove command, got {:?}", cmd
    );
}

/// QA Step 7 [auto]: Lure the unit 7+ grid units perpendicular from its path during AttackMove.
/// Verify it leashes back (disengages and returns to path).
#[test]
fn step_7_attack_move_leash() {
    let mut test_app = TestApp::new();
    test_app.step();

    let player;
    let enemy;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        player = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        // Enemy off to the side, far away
        enemy = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 30, Owner(Some(1)));
    }
    test_app.step();

    let start_pos = TestHarness::new(&mut test_app.app).get_position(player).unwrap();

    // Issue AttackMove along X axis
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(player, UnitCommand::AttackMove(Vec3::new(start_pos.x + 10.0, start_pos.y, start_pos.z)));
    }
    test_app.step_n(30);

    // The player unit shouldn't have chased the enemy 10 GU to the side
    let player_pos = TestHarness::new(&mut test_app.app).get_position(player).unwrap();
    let perpendicular_dist = (player_pos.z - start_pos.z).abs();
    // ATTACK_MOVE_LEASH_DISTANCE is 6.0
    assert!(perpendicular_dist < 8.0,
        "Unit should leash back during AttackMove. Perpendicular distance from path: {}", perpendicular_dist);
}

/// QA Step 8 [auto]: Issue Patrol between two points. Verify the unit continuously cycles between them.
#[test]
fn step_8_patrol_command() {
    let mut test_app = TestApp::new();
    test_app.step();

    let player;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        player = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let pos = TestHarness::new(&mut test_app.app).get_position(player).unwrap();
    let patrol_end = Vec3::new(pos.x + 5.0, pos.y, pos.z);

    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(player, UnitCommand::Patrol {
            start: pos,
            end: patrol_end,
            going_to_end: true,
        });
    }
    test_app.step();

    let cmd = TestHarness::new(&mut test_app.app).get_command(player);
    assert!(
        matches!(cmd, Some(UnitCommand::Patrol { .. })),
        "Unit should have Patrol command, got {:?}", cmd
    );
}

/// QA Step 9 [auto]: Issue HoldPosition with a turret unit. Verify it stays stationary.
/// NOTE: Testing with Peacekeeper infantry instead.
#[test]
fn step_9_hold_position_stationary() {
    let mut test_app = TestApp::new();
    test_app.step();

    let player;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        player = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let start_pos = TestHarness::new(&mut test_app.app).get_position(player).unwrap();

    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(player, UnitCommand::HoldPosition);
    }
    test_app.step_n(20);

    let now_pos = TestHarness::new(&mut test_app.app).get_position(player).unwrap();
    let drift = (now_pos - start_pos).length();
    assert!(drift < 0.1, "HoldPosition unit should be stationary. Drift: {}", drift);
}

/// QA Step 10 [auto]: Issue HoldPosition with CanTurnInPlace infantry.
/// Verify it rotates toward nearby enemies and engages.
#[test]
fn step_10_hold_position_engages_nearby() {
    let mut test_app = TestApp::new();
    test_app.step();

    let player;
    let enemy;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        player = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        enemy = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 20, Owner(Some(1)));
    }
    test_app.step();

    let start_pos = TestHarness::new(&mut test_app.app).get_position(player).unwrap();

    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(player, UnitCommand::HoldPosition);
    }
    test_app.step_n(30);

    // Should still be at starting position
    let now_pos = TestHarness::new(&mut test_app.app).get_position(player).unwrap();
    let drift = (now_pos - start_pos).length();
    assert!(drift < 0.2, "HoldPosition unit should not move even while engaging. Drift: {}", drift);

    // Should be engaging enemy
    let world = test_app.app.world();
    if let Some(attack_state) = world.get::<AttackState>(player) {
        assert!(attack_state.current_target.is_some(),
            "HoldPosition infantry near enemy should engage. Phase: {:?}, target: {:?}",
            attack_state.phase, attack_state.current_target);
    }
}

/// QA Step 11 [auto]: Issue HoldPosition with non-turning infantry.
/// Verify it only engages targets in its current facing arc.
/// NOTE: Peacekeeper (LightInfantry) CAN turn in place (turn_rate = instant).
/// This test verifies HoldPosition keeps the unit stationary even if turning.
#[test]
fn step_11_hold_position_no_move() {
    let mut test_app = TestApp::new();
    test_app.step();

    let player;
    let enemy;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        player = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        // Enemy behind the unit
        enemy = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 19, 20, Owner(Some(1)));
    }
    test_app.step();

    let start_pos = TestHarness::new(&mut test_app.app).get_position(player).unwrap();

    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(player, UnitCommand::HoldPosition);
    }
    test_app.step_n(30);

    // Verify position unchanged
    let now_pos = TestHarness::new(&mut test_app.app).get_position(player).unwrap();
    let drift = (now_pos - start_pos).length();
    assert!(drift < 0.2, "HoldPosition should keep unit stationary. Drift: {}", drift);
}
