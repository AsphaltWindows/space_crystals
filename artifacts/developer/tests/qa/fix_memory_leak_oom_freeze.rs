use crate::helpers::*;

/// QA Step 2 [auto]: Launch the game, select GDO faction.
/// TestApp::new() already starts a GDO game. Verify the app initializes without panic.
#[test]
fn step_2_launch_game_gdo() {
    let mut test_app = TestApp::new();
    test_app.step();

    // Verify the game world is populated (GDO faction start)
    let world = test_app.app.world();
    // Should have at least the Deployment Center spawned
    let mut has_dc = false;
    for obj in world.iter_entities().filter_map(|e| world.get::<ObjectInstance>(e.id())) {
        if obj.object_type == ObjectEnum::DeploymentCenter {
            has_dc = true;
            break;
        }
    }
    assert!(has_dc, "GDO game should start with a Deployment Center");
}

/// QA Step 4 [auto]: Produce 5 Peacekeeper units.
/// Spawn 5 Peacekeepers and verify they exist with correct components.
#[test]
fn step_4_produce_5_peacekeepers() {
    let mut test_app = TestApp::new();
    test_app.step();

    let mut units = Vec::new();
    {
        let mut h = TestHarness::new(&mut test_app.app);
        for i in 0..5 {
            let pk = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20 + i, 20, Owner(Some(0)));
            units.push(pk);
        }
    }
    test_app.step();

    let world = test_app.app.world();
    for (i, &unit) in units.iter().enumerate() {
        let obj = world.get::<ObjectInstance>(unit)
            .unwrap_or_else(|| panic!("Peacekeeper {} should exist", i));
        assert_eq!(obj.object_type, ObjectEnum::Peacekeeper, "Unit {} should be a Peacekeeper", i);
        assert!(world.get::<UnitCommand>(unit).is_some(), "Unit {} should have UnitCommand", i);
    }
}

/// QA Step 5 [auto]: Issue move commands to all 5 units.
/// Verify move commands are accepted and stored on each unit.
#[test]
fn step_5_issue_move_commands() {
    let mut test_app = TestApp::new();
    test_app.step();

    let mut units = Vec::new();
    {
        let mut h = TestHarness::new(&mut test_app.app);
        for i in 0..5 {
            let pk = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20 + i, 20, Owner(Some(0)));
            units.push(pk);
        }
    }
    test_app.step();

    // Issue move commands to a distant location
    let target = Vec3::new(-15.0, 0.0, -15.0);
    {
        let mut h = TestHarness::new(&mut test_app.app);
        for &unit in &units {
            h.issue_command(unit, UnitCommand::Move(target));
        }
    }
    test_app.step();

    // Verify all units have the Move command
    let world = test_app.app.world();
    for (i, &unit) in units.iter().enumerate() {
        let cmd = world.get::<UnitCommand>(unit)
            .unwrap_or_else(|| panic!("Unit {} should have UnitCommand after move", i));
        match cmd {
            UnitCommand::Move(t) => {
                assert_eq!(*t, target, "Unit {} move target should match", i);
            }
            _ => panic!("Unit {} expected Move command, got {:?}", i, cmd),
        }
    }
}
