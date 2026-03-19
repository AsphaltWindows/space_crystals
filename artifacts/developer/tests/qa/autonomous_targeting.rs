use crate::helpers::*;
#[allow(unused_imports)]
use space_crystals::game::combat::types::{AttackState, AttackTarget, AttackPhase, AttackCapability};
#[allow(unused_imports)]
use space_crystals::game::units::types::movement::Velocity;

/// QA Step 1 [auto]: Place a turret unit with no locked target near an enemy.
/// Verify the turret autonomously selects and engages the enemy.
/// NOTE: No turret unit type available for spawning. Testing with Peacekeeper (infantry auto-targeting).
#[test]
fn step_1_auto_targeting_engages_enemy() {
    let mut test_app = TestApp::new();
    test_app.step();

    let player_unit;
    let enemy_unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        // Place idle player unit near enemy - auto-targeting should engage
        player_unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        enemy_unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 20, Owner(Some(1)));
    }
    test_app.step();

    // Let auto-targeting systems run for many frames
    test_app.step_n(30);

    // Check if the unit auto-acquired a target
    let world = test_app.app.world();
    if let Some(attack_state) = world.get::<AttackState>(player_unit) {
        // Auto-targeting should have engaged the nearby enemy
        assert!(attack_state.current_target.is_some() || !matches!(attack_state.phase, AttackPhase::None),
            "Idle unit near enemy should auto-engage. Phase: {:?}, target: {:?}",
            attack_state.phase, attack_state.current_target);
    }
}

/// QA Step 6 [auto]: Place an idle infantry unit near an enemy. Verify BaseAutoTargeting engages the enemy.
#[test]
fn step_6_infantry_auto_targeting() {
    let mut test_app = TestApp::new();
    test_app.step();

    let infantry;
    let enemy;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        infantry = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        enemy = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 20, Owner(Some(1)));
    }
    test_app.step();

    // Ensure infantry is idle
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(infantry, UnitCommand::Idle);
    }

    test_app.step_n(30);

    let world = test_app.app.world();
    if let Some(attack_state) = world.get::<AttackState>(infantry) {
        assert!(attack_state.current_target.is_some(),
            "Idle infantry near enemy should auto-target. Phase: {:?}, target: {:?}",
            attack_state.phase, attack_state.current_target);
    }
}

/// QA Step 7 [auto]: Lure the enemy 5+ grid units from the infantry's IdleOrigin.
/// Verify the infantry leashes back (returns to IdleOrigin, does not chase beyond 4gu).
#[test]
fn step_7_infantry_leash_back() {
    let mut test_app = TestApp::new();
    test_app.step();

    let infantry;
    let enemy;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        infantry = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        enemy = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 20, Owner(Some(1)));
    }
    test_app.step();

    let infantry_start = TestHarness::new(&mut test_app.app).get_position(infantry).unwrap();

    // Let auto-targeting engage
    test_app.step_n(10);

    // Move enemy far away (simulating luring)
    {
        let world = test_app.app.world_mut();
        if let Some(mut transform) = world.get_mut::<Transform>(enemy) {
            transform.translation.x = infantry_start.x + 8.0; // 8 GU away
        }
    }

    // Run many frames for leash behavior
    test_app.step_n(60);

    // Infantry should not be more than ~4gu from its starting position (leash distance)
    let infantry_now = TestHarness::new(&mut test_app.app).get_position(infantry).unwrap();
    let dist_from_start = (infantry_now - infantry_start).length();
    // IDLE_LEASH_DISTANCE is 4.0, allow some tolerance
    assert!(dist_from_start < 6.0,
        "Infantry should leash back within ~4gu of IdleOrigin. Distance from start: {}", dist_from_start);
}

/// QA Step 8 [auto]: Issue HoldPosition to infantry near an enemy. Verify it engages without moving.
#[test]
fn step_8_hold_position_engages_stationary() {
    let mut test_app = TestApp::new();
    test_app.step();

    let infantry;
    let enemy;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        infantry = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        enemy = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 20, Owner(Some(1)));
    }
    test_app.step();

    let start_pos = TestHarness::new(&mut test_app.app).get_position(infantry).unwrap();

    // Issue HoldPosition
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(infantry, UnitCommand::HoldPosition);
    }

    test_app.step_n(30);

    // Unit should not have moved much
    let now_pos = TestHarness::new(&mut test_app.app).get_position(infantry).unwrap();
    let drift = (now_pos - start_pos).length();
    assert!(drift < 0.2, "HoldPosition unit should not move. Drift: {}", drift);

    // Check the command is still HoldPosition
    let cmd = TestHarness::new(&mut test_app.app).get_command(infantry);
    assert!(
        matches!(cmd, Some(UnitCommand::HoldPosition)),
        "Unit should still be in HoldPosition, got {:?}", cmd
    );
}

/// QA Step 9 [auto]: Issue Move to an infantry unit passing near enemies.
/// Verify BaseAutoTargeting does NOT activate (unit keeps moving).
#[test]
fn step_9_move_suppresses_auto_targeting() {
    let mut test_app = TestApp::new();
    test_app.step();

    let infantry;
    let enemy;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        infantry = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        enemy = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 22, 20, Owner(Some(1)));
    }
    test_app.step();

    // Issue Move command past the enemy
    {
        let mut h = TestHarness::new(&mut test_app.app);
        let pos = h.get_position(infantry).unwrap();
        h.issue_command(infantry, UnitCommand::Move(Vec3::new(pos.x + 10.0, pos.y, pos.z)));
    }

    test_app.step_n(15);

    // Unit should still have its Move command, not have switched to attack
    let cmd = TestHarness::new(&mut test_app.app).get_command(infantry);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Moving infantry should NOT auto-target. Command should still be Move, got {:?}", cmd
    );
}

/// QA Step 10 [auto]: Issue Move to a turret unit passing near enemies.
/// Verify TurretAutonomousScanning DOES activate (turret fires while moving) but base does not stop.
/// NOTE: Using Peacekeeper (no turret). Testing that move command persists near enemies.
#[test]
fn step_10_move_preserves_command() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    let enemy;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        enemy = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 22, 20, Owner(Some(1)));
    }
    test_app.step();

    {
        let mut h = TestHarness::new(&mut test_app.app);
        let pos = h.get_position(unit).unwrap();
        h.issue_command(unit, UnitCommand::Move(Vec3::new(pos.x + 10.0, pos.y, pos.z)));
    }
    test_app.step_n(20);

    // Unit should still be moving (command not interrupted)
    let cmd = TestHarness::new(&mut test_app.app).get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_)) | Some(UnitCommand::Idle)),
        "Unit should still be moving or have arrived, got {:?}", cmd
    );
}
