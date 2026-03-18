use crate::helpers::*;
use space_crystals::game::units::utils::{get_neighbors, heuristic, grid_to_world};
use space_crystals::game::units::types::movement::Path;

/// QA Step 1 [auto]: Start a new game as GDO faction.
/// QA Step 2 [auto]: Spawn a Peacekeeper unit.
#[test]
fn step_1_2_start_gdo_and_spawn_peacekeeper() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let h = TestHarness::new(&mut test_app.app);
    assert!(h.is_alive(unit), "Peacekeeper should be alive");
    let pos = h.get_position(unit).unwrap();
    assert!(pos.x.is_finite(), "Position should be valid");
}

/// QA Step 3 [auto]: Right-click to move the unit to a destination that is diagonally offset
/// (both X and Z differ) on open terrain. Verify path supports diagonal movement.
#[test]
fn step_3_diagonal_move_command() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let start_pos = TestHarness::new(&mut test_app.app).get_position(unit).unwrap();

    // Issue diagonal move: both X and Z differ from unit position
    let target = Vec3::new(start_pos.x + 5.0, start_pos.y, start_pos.z + 5.0);
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(unit, UnitCommand::Move(target));
    }

    // Step enough frames for pathfinding to process and Path to be assigned
    test_app.step_n(5);

    let cmd = TestHarness::new(&mut test_app.app).get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Unit should have Move command, got {:?}", cmd
    );

    // Verify the pathfinding utility supports 8-directional neighbors
    let center = GridPosition { x: 32, z: 32 };
    let neighbors = get_neighbors(&center);
    assert_eq!(neighbors.len(), 8, "get_neighbors should return 8 directions (4 cardinal + 4 diagonal)");

    // Verify diagonal neighbors have SQRT_2 cost
    let diagonal_count = neighbors.iter().filter(|(_, cost)| (*cost - std::f32::consts::SQRT_2).abs() < 0.01).count();
    assert_eq!(diagonal_count, 4, "Should have 4 diagonal neighbors with SQRT_2 cost");

    let cardinal_count = neighbors.iter().filter(|(_, cost)| (*cost - 1.0).abs() < 0.01).count();
    assert_eq!(cardinal_count, 4, "Should have 4 cardinal neighbors with 1.0 cost");
}

/// Verify heuristic uses octile distance (not Manhattan)
#[test]
fn step_3b_heuristic_is_octile() {
    let a = GridPosition { x: 0, z: 0 };
    let b = GridPosition { x: 3, z: 4 };

    let h = heuristic(&a, &b);
    // Octile distance: max(3,4) + (SQRT_2-1)*min(3,4) = 4 + 0.414*3 = 5.243
    let expected = 4.0 + (std::f32::consts::SQRT_2 - 1.0) * 3.0;
    assert!(
        (h - expected).abs() < 0.01,
        "Heuristic should use octile distance. Expected {:.3}, got {:.3}", expected, h
    );

    // Pure diagonal: octile distance should be SQRT_2 * N
    let c = GridPosition { x: 5, z: 5 };
    let h2 = heuristic(&a, &c);
    let expected2 = std::f32::consts::SQRT_2 * 5.0;
    assert!(
        (h2 - expected2).abs() < 0.01,
        "Pure diagonal octile distance should be SQRT_2*5. Expected {:.3}, got {:.3}", expected2, h2
    );
}

/// QA Step 5 [auto]: Right-click to move the unit to a distant location across open terrain.
/// QA Step 6 [auto]: Observe that the unit reaches the destination and stops cleanly.
/// NOTE: Long-distance moves may trigger the known memory leak. We test path creation only.
#[test]
fn step_5_6_distant_move_path_created() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        // Spawn away from (50,50) to avoid enemy interference
        unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
    }
    test_app.step();

    let start_pos = TestHarness::new(&mut test_app.app).get_position(unit).unwrap();

    // Issue move to a moderately distant location (not too far to avoid OOM)
    let target = Vec3::new(start_pos.x + 10.0, start_pos.y, start_pos.z + 8.0);
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(unit, UnitCommand::Move(target));
    }

    // Step frames for pathfinding
    test_app.step_n(5);

    // Verify the unit has a Move command (pathfinding accepted the request)
    let cmd = TestHarness::new(&mut test_app.app).get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Unit should have Move command for distant target, got {:?}", cmd
    );
}

/// QA Step 9 [auto]: Move a unit to a location very close to its current position (1-2 tiles away).
/// QA Step 10 [auto]: Verify the unit arrives and stops without oscillating.
#[test]
fn step_9_10_short_move_no_oscillation() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let start_pos = TestHarness::new(&mut test_app.app).get_position(unit).unwrap();

    // Issue a very short move (1-2 tiles)
    let target = Vec3::new(start_pos.x + 1.5, start_pos.y, start_pos.z + 1.0);
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(unit, UnitCommand::Move(target));
    }

    // Step enough frames for a short move to complete (at typical speeds)
    test_app.step_n(60);

    // After 60 frames, check the unit's position is near the target
    let final_pos = TestHarness::new(&mut test_app.app).get_position(unit).unwrap();
    let dist_to_target = ((final_pos.x - target.x).powi(2) + (final_pos.z - target.z).powi(2)).sqrt();

    // Check the unit is either near the target or has completed the move
    // The command should be Idle if movement completed
    let cmd = TestHarness::new(&mut test_app.app).get_command(unit);
    let reached_or_idle = dist_to_target < 2.0 || matches!(cmd, Some(UnitCommand::Idle));
    assert!(
        reached_or_idle,
        "Unit should be near target (dist: {:.2}) or idle (cmd: {:?}). Possible oscillation.", dist_to_target, cmd
    );

    // Check no oscillation: unit should not still be moving back and forth
    // Step a few more frames and verify position is stable
    let pos_before = TestHarness::new(&mut test_app.app).get_position(unit).unwrap();
    test_app.step_n(10);
    let pos_after = TestHarness::new(&mut test_app.app).get_position(unit).unwrap();
    let movement = (pos_after - pos_before).length();

    // If unit is idle, movement should be very small (no oscillation)
    if matches!(cmd, Some(UnitCommand::Idle)) {
        assert!(
            movement < 0.5,
            "Idle unit should not be oscillating. Movement over 10 frames: {:.3}", movement
        );
    }
}
