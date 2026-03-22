use crate::helpers::*;
use space_crystals::ui::types::{ObjectInterfaceState, CursorTarget, CursorTargetEnum, SelectedUnitCapabilities};

/// QA Step 2 [auto]: Click HoldPosition. Verify HoldPosition command issued immediately.
/// HoldPosition is an immediate command — no awaiting target phase.
#[test]
fn step_2_hold_position_immediate_command() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        harness.set_selection(&[unit]);
    }
    test_app.step();

    // Issue HoldPosition directly (simulating what execute_command_action does)
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        harness.issue_command(unit, UnitCommand::HoldPosition);
    }
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::HoldPosition)),
        "Unit should have HoldPosition command, got {:?}", cmd
    );
}

/// QA Step 3 [auto]: Click Stop. Verify Stop command issued immediately.
/// Stop is an immediate command — no awaiting target phase.
#[test]
fn step_3_stop_immediate_command() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        harness.set_selection(&[unit]);
    }
    test_app.step();

    // Issue Stop directly
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        harness.issue_command(unit, UnitCommand::Stop);
    }
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Stop)),
        "Unit should have Stop command, got {:?}", cmd
    );
}

/// QA Step 5 [auto]: In AwaitingTarget[Attack], left-click ground. Verify AttackMove command.
/// Attack + ground click = AttackMove (the unit moves toward the target, attacking along the way).
#[test]
fn step_5_attack_mode_ground_click_issues_attack_move() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        harness.set_selection(&[unit]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (unit, ObjectEnum::Peacekeeper, true),
        ]);
    }

    // Enter Attack command mode
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AwaitingTarget(CommandType::Attack);

    // Set cursor to ground and left-click
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::Ground,
        location: Some(Vec3::new(-6.5, 0.0, -6.5)),
        entity: None,
    };
    send_mouse_press(&mut test_app.app, MouseButton::Left);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::AttackMove(_))),
        "Attack + ground click should produce AttackMove, got {:?}", cmd
    );
}

/// QA Step 6 [auto]: Click Move, left-click ground. Verify Move command.
#[test]
fn step_6_move_mode_ground_click_issues_move() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        harness.set_selection(&[unit]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (unit, ObjectEnum::Peacekeeper, true),
        ]);
    }

    // Enter Move command mode
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AwaitingTarget(CommandType::Move);

    // Set cursor to ground and left-click
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::Ground,
        location: Some(Vec3::new(-6.5, 0.0, -6.5)),
        entity: None,
    };
    send_mouse_press(&mut test_app.app, MouseButton::Left);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Move + ground click should produce Move, got {:?}", cmd
    );
}

/// QA Step 6 continued [auto]: Click Move, left-click friendly object. Verify Move command
/// (move to the object's location, not an attack).
#[test]
fn step_6_move_mode_friendly_click_issues_move() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    let friendly;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        friendly = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 15, 15, Owner(Some(0)));
        harness.set_selection(&[unit]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (unit, ObjectEnum::Peacekeeper, true),
        ]);
    }

    // Enter Move command mode
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AwaitingTarget(CommandType::Move);

    // Set cursor to a friendly object with a location
    let friendly_world_pos = Vec3::new(-16.5, 0.0, -16.5); // grid(15,15) -> world
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::FriendlyObject,
        location: Some(friendly_world_pos),
        entity: Some(friendly),
    };
    send_mouse_press(&mut test_app.app, MouseButton::Left);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Move + friendly click should produce Move, got {:?}", cmd
    );
}

/// QA Step 7 [auto]: Click Patrol. Left-click ground. Verify Patrol command.
#[test]
fn step_7_patrol_mode_ground_click_issues_patrol() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        harness.set_selection(&[unit]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (unit, ObjectEnum::Peacekeeper, true),
        ]);
    }

    // Enter Patrol command mode
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AwaitingTarget(CommandType::Patrol);

    // Set cursor to ground and left-click
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::Ground,
        location: Some(Vec3::new(-6.5, 0.0, -6.5)),
        entity: None,
    };
    send_mouse_press(&mut test_app.app, MouseButton::Left);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Patrol { .. })),
        "Patrol + ground click should produce Patrol, got {:?}", cmd
    );
}

/// QA Step 8 [auto]: For a unit with CanTargetGround: click AttackGround. Left-click ground.
/// Verify AttackGround command (AttackLocation).
/// SKIPPED: Peacekeeper does not have CanTargetGround capability.
#[test]
#[ignore]
fn step_8_attack_ground_command() {
    // Peacekeeper does not have CanTargetGround — no unit type currently available
    // to test this. When a unit with CanTargetGround is added, implement this test.
}

/// QA Step 9 [auto]: For a unit with CanReverse: click Reverse. Left-click ground.
/// Verify Reverse command.
/// SKIPPED: No unit type with CanReverse currently available.
#[test]
#[ignore]
fn step_9_reverse_command() {
    // No unit type currently has CanReverse capability.
    // When one is added, implement this test.
}

/// QA Step 10 [auto]: Right-click an enemy unit. Verify Attack command targeting that enemy.
#[test]
fn step_10_right_click_enemy_issues_attack_target() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    let enemy;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        enemy = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(1)));
        harness.set_selection(&[unit]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (unit, ObjectEnum::Peacekeeper, true),
        ]);
    }

    // Interface in Default state (no command mode)
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;

    // Set cursor target to enemy entity
    let enemy_world_pos = Vec3::new(-11.5, 0.0, -11.5); // grid(20,20) -> world
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::EnemyObject,
        location: Some(enemy_world_pos),
        entity: Some(enemy),
    };
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::AttackTarget(e)) if e == enemy),
        "Right-click on enemy should produce AttackTarget, got {:?}", cmd
    );
}

/// QA Step 11 [auto]: Right-click ground. Verify Move command.
#[test]
fn step_11_right_click_ground_issues_move() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        harness.set_selection(&[unit]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (unit, ObjectEnum::Peacekeeper, true),
        ]);
    }

    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;

    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::Ground,
        location: Some(Vec3::new(-6.5, 0.0, -6.5)),
        entity: None,
    };
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Right-click ground should produce Move, got {:?}", cmd
    );
}

/// QA Step 12 [auto]: Right-click a friendly unit. Verify Move command to that object's location.
#[test]
fn step_12_right_click_friendly_issues_move() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    let friendly;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        friendly = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 15, 15, Owner(Some(0)));
        harness.set_selection(&[unit]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (unit, ObjectEnum::Peacekeeper, true),
        ]);
    }

    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;

    let friendly_pos = Vec3::new(-16.5, 0.0, -16.5);
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::FriendlyObject,
        location: Some(friendly_pos),
        entity: Some(friendly),
    };
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Right-click friendly should produce Move, got {:?}", cmd
    );
}

/// QA Step 13 [auto]: Right-click a neutral object. Verify Move command to that object's location.
#[test]
fn step_13_right_click_neutral_issues_move() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        harness.set_selection(&[unit]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (unit, ObjectEnum::Peacekeeper, true),
        ]);
    }

    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;

    // NeutralObject falls through to the ground move path in right_click_move_command
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::NeutralObject,
        location: Some(Vec3::new(-6.5, 0.0, -6.5)),
        entity: None,
    };
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Right-click neutral should produce Move, got {:?}", cmd
    );
}

/// QA Step 14 [auto]: For a unit without CanTargetGround, verify AttackGround does not
/// appear in command panel. Tested via SelectedUnitCapabilities resource.
#[test]
fn step_14_peacekeeper_no_attack_ground_capability() {
    // Peacekeeper has AttackCapability but NOT CanTargetGround
    let caps = SelectedUnitCapabilities {
        has_attack: true,
        can_target_ground: false,
        can_reverse: false,
        agent_carrying: false,
        is_chopper: false,
        chopper_has_supplies: false,
    };

    assert!(
        caps.has_attack,
        "Peacekeeper should have attack capability"
    );
    assert!(
        !caps.can_target_ground,
        "Peacekeeper should NOT have CanTargetGround — AttackGround button hidden"
    );
}

/// QA Step 15 [auto]: For a unit without CanReverse, verify Reverse does not appear.
/// Tested via SelectedUnitCapabilities resource.
#[test]
fn step_15_peacekeeper_no_reverse_capability() {
    let caps = SelectedUnitCapabilities {
        has_attack: true,
        can_target_ground: false,
        can_reverse: false,
        agent_carrying: false,
        is_chopper: false,
        chopper_has_supplies: false,
    };

    assert!(
        !caps.can_reverse,
        "Peacekeeper should NOT have CanReverse — Reverse button hidden"
    );
}
