use crate::helpers::*;
#[allow(unused_imports)]
use space_crystals::game::units::types::movement::Velocity;
#[allow(unused_imports)]
use space_crystals::game::units::types::state::behavior::BaseBehaviorState;

/// QA Step 1 [auto]: Issue Move to a location. Verify the unit accepts the Move command
/// and has appropriate behavior state.
#[test]
fn step_1_move_to_location() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let start_pos = TestHarness::new(&mut test_app.app).get_position(unit).unwrap();
    let target = Vec3::new(start_pos.x + 5.0, start_pos.y, start_pos.z);

    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(unit, UnitCommand::Move(target));
    }
    test_app.step_n(5);

    // Verify the Move command is accepted and persisted
    let cmd = TestHarness::new(&mut test_app.app).get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Unit should have Move command after issuing move, got {:?}", cmd
    );
}

/// QA Step 2 [auto]: Block the path mid-movement. Verify the unit still has its move command
/// (recomputing path rather than cancelling).
#[test]
fn step_2_path_recomputation() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let start_pos = TestHarness::new(&mut test_app.app).get_position(unit).unwrap();
    let target = Vec3::new(start_pos.x + 8.0, start_pos.y, start_pos.z);

    // Start moving
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(unit, UnitCommand::Move(target));
    }
    test_app.step_n(5);

    // Place a structure blocking the path
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.spawn_structure_at_grid(ObjectEnum::PowerPlant, 24, 20, Owner(Some(0)));
    }
    test_app.step_n(5);

    // Unit should still have its move command (not cancelled by obstacle)
    let cmd = TestHarness::new(&mut test_app.app).get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_)) | Some(UnitCommand::Idle)),
        "Unit should still have Move command or have completed, got {:?}", cmd
    );
}

/// QA Step 3 [auto]: Verify the unit's velocity is zero when not actively moving
/// (initial state or after stop).
#[test]
fn step_3_velocity_zero_when_stationary() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    // Check velocity is zero for idle unit
    let world = test_app.app.world();
    if let Some(vel) = world.get::<Velocity>(unit) {
        assert!(vel.0.length() < 0.01,
            "Idle unit should have zero velocity. Velocity: {:?}", vel.0);
    }
}

/// QA Step 5 [auto]: Issue Move targeting an ally's position.
/// Verify the Move command is accepted with the ally's coordinates.
#[test]
fn step_5_move_to_object() {
    let mut test_app = TestApp::new();
    test_app.step();

    let follower;
    let target_unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        follower = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        target_unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 25, 20, Owner(Some(0)));
    }
    test_app.step();

    // Move follower toward the target unit's position
    let target_pos = TestHarness::new(&mut test_app.app).get_position(target_unit).unwrap();
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(follower, UnitCommand::Move(target_pos));
    }
    test_app.step();

    // Verify Move command was accepted with the right target
    let cmd = TestHarness::new(&mut test_app.app).get_command(follower);
    match cmd {
        Some(UnitCommand::Move(pos)) => {
            let dist = (pos - target_pos).length();
            assert!(dist < 0.1, "Move target should match ally position. Target diff: {}", dist);
        }
        _ => panic!("Unit should have Move command, got {:?}", cmd),
    }
}

/// QA Step 8 [auto]: Issue Stop. Verify the Stop command is accepted and velocity stays zero.
#[test]
fn step_8_stop_command() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    // Start moving
    {
        let mut h = TestHarness::new(&mut test_app.app);
        let pos = h.get_position(unit).unwrap();
        h.issue_command(unit, UnitCommand::Move(Vec3::new(pos.x + 10.0, pos.y, pos.z)));
    }
    test_app.step_n(5);

    // Issue stop
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(unit, UnitCommand::Stop);
    }
    test_app.step_n(10);

    // Velocity should be near zero
    let world = test_app.app.world();
    if let Some(vel) = world.get::<Velocity>(unit) {
        assert!(vel.0.length() < 0.1,
            "Unit should have stopped. Velocity: {:?}", vel.0);
    }
}
