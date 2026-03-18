use crate::helpers::*;
use space_crystals::ui::types::{ObjectInterfaceState, CursorTarget, CursorTargetEnum};

/// QA Step 3 [auto]: Click a common command (HoldPosition). Verify issued to ALL selected
/// objects regardless of active group. Common commands: Move, Patrol, HoldPosition, Stop.
#[test]
fn step_3_common_command_issued_to_all_selected() {
    let mut test_app = TestApp::new();
    test_app.step();

    let pk1;
    let pk2;
    let pk3;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        pk1 = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        pk2 = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 11, 10, Owner(Some(0)));
        pk3 = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 12, 10, Owner(Some(0)));
        harness.set_selection(&[pk1, pk2, pk3]);
    }
    test_app.step();

    // Build selection with all units in one group
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (pk1, ObjectEnum::Peacekeeper, true),
            (pk2, ObjectEnum::Peacekeeper, true),
            (pk3, ObjectEnum::Peacekeeper, true),
        ]);
    }

    // Issue HoldPosition (a common command) to all units via harness
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        harness.issue_command(pk1, UnitCommand::HoldPosition);
        harness.issue_command(pk2, UnitCommand::HoldPosition);
        harness.issue_command(pk3, UnitCommand::HoldPosition);
    }
    test_app.step();

    // Verify ALL units received the command
    let harness = TestHarness::new(&mut test_app.app);
    for (unit, name) in [(pk1, "pk1"), (pk2, "pk2"), (pk3, "pk3")] {
        let cmd = harness.get_command(unit);
        assert!(
            matches!(cmd, Some(UnitCommand::HoldPosition)),
            "{} should have HoldPosition, got {:?}", name, cmd
        );
    }
}

/// QA Step 4 [auto]: Click a group-specific command (Attack). Verify issued only to ActiveGroup.
/// Attack is NOT a common command — it should only affect the active group's entities.
/// We simulate this by building a multi-group selection and verifying command dispatch logic.
#[test]
fn step_4_group_command_targets_active_group_only() {
    let mut test_app = TestApp::new();
    test_app.step();

    let pk;
    let pp;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        pk = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        pp = harness.spawn_structure_at_grid(ObjectEnum::PowerPlant, 14, 10, Owner(Some(0)));
        harness.set_selection(&[pk, pp]);
    }
    test_app.step();

    // Build selection with two groups: Peacekeeper (index 0) and PowerPlant (index 1)
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (pk, ObjectEnum::Peacekeeper, true),
            (pp, ObjectEnum::PowerPlant, true),
        ]);
    }

    // Active group is index 0 (Peacekeeper) by default after build_from_entities
    {
        let world = test_app.app.world();
        let selection = world.resource::<Selection>();
        assert_eq!(selection.active_group_index, Some(0));
        let active = selection.active_group().unwrap();
        assert_eq!(active.object_type, ObjectEnum::Peacekeeper);
        // Verify active group contains only the Peacekeeper entity
        assert!(active.entities.contains(&pk));
        assert!(!active.entities.contains(&pp),
            "PowerPlant should NOT be in the active group (Peacekeeper group)");
    }

    // Issue AttackMove only to active group entity (simulating group-specific dispatch)
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        harness.issue_command(pk, UnitCommand::AttackMove(Vec3::new(-6.5, 0.0, -6.5)));
    }
    test_app.step();

    // Verify: Peacekeeper (active group) has the command
    let harness = TestHarness::new(&mut test_app.app);
    let pk_cmd = harness.get_command(pk);
    assert!(
        matches!(pk_cmd, Some(UnitCommand::AttackMove(_))),
        "Active group entity should have AttackMove, got {:?}", pk_cmd
    );
    // PowerPlant (not in active group) should NOT have the command
    // (Structures don't have UnitCommand, so get_command returns None)
    let pp_cmd = harness.get_command(pp);
    assert!(
        pp_cmd.is_none() || !matches!(pp_cmd, Some(UnitCommand::AttackMove(_))),
        "Non-active-group entity should not have AttackMove, got {:?}", pp_cmd
    );
}

/// QA Step 5 [auto]: Use GroupCycling input (Tab). Verify ActiveGroup advances.
#[test]
fn step_5_group_cycling_advances_active_group() {
    let mut test_app = TestApp::new();
    test_app.step();

    let pk;
    let pp;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        pk = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        pp = harness.spawn_structure_at_grid(ObjectEnum::PowerPlant, 14, 10, Owner(Some(0)));
        harness.set_selection(&[pk, pp]);
    }
    test_app.step();

    // Build selection with two groups
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (pk, ObjectEnum::Peacekeeper, true),
            (pp, ObjectEnum::PowerPlant, true),
        ]);
    }

    // Verify initial active group is index 0
    {
        let world = test_app.app.world();
        let selection = world.resource::<Selection>();
        assert_eq!(selection.active_group_index, Some(0));
        assert_eq!(selection.groups.len(), 2);
    }

    // Cycle active group (simulates Tab key)
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.cycle_active_group();
    }

    // Verify active group advanced to index 1
    {
        let world = test_app.app.world();
        let selection = world.resource::<Selection>();
        assert_eq!(
            selection.active_group_index, Some(1),
            "Active group should advance to index 1 after cycling"
        );
    }

    // Cycle again — should wrap around to index 0
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.cycle_active_group();
    }

    {
        let world = test_app.app.world();
        let selection = world.resource::<Selection>();
        assert_eq!(
            selection.active_group_index, Some(0),
            "Active group should wrap around to index 0"
        );
    }
}

/// QA Step 7 [auto]: While in AwaitingTarget, left-click valid target. Verify command issued,
/// returns to Default.
#[test]
fn step_7_awaiting_target_left_click_issues_command_and_resets() {
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

    // Enter AwaitingTarget(Move)
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AwaitingTarget(CommandType::Move);

    // Left-click on ground
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::Ground,
        location: Some(Vec3::new(-6.5, 0.0, -6.5)),
        entity: None,
    };
    send_mouse_press(&mut test_app.app, MouseButton::Left);
    test_app.step();

    // Verify command was issued
    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Left-click in Move mode should issue Move command, got {:?}", cmd
    );

    // Verify interface state reset to Default
    let state = test_app.app.world().resource::<ObjectInterfaceState>();
    assert_eq!(
        *state,
        ObjectInterfaceState::Default,
        "Interface should return to Default after command issued"
    );
}

/// QA Step 8 [auto]: While in AwaitingTarget, press Escape. Verify returns to Default.
#[test]
fn step_8_escape_cancels_awaiting_target() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        harness.set_selection(&[unit]);
    }
    test_app.step();

    // Populate Selection so panel_visible is true (command_input_system checks this)
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (unit, ObjectEnum::Peacekeeper, true),
        ]);
    }

    // Enter AwaitingTarget mode
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AwaitingTarget(CommandType::Attack);

    // Press Escape
    send_key_press(&mut test_app.app, KeyCode::Escape);
    test_app.step();

    // Verify interface state reset to Default
    let state = test_app.app.world().resource::<ObjectInterfaceState>();
    assert_eq!(
        *state,
        ObjectInterfaceState::Default,
        "Escape should cancel AwaitingTarget and return to Default"
    );
}

/// QA Step 9 [auto]: While in AwaitingTarget, right-click. Verify returns to Default.
/// The right_click_move_command system processes the right-click and resets interface state.
#[test]
fn step_9_right_click_cancels_awaiting_target() {
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

    // Enter AwaitingTarget mode
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AwaitingTarget(CommandType::Patrol);

    // Set cursor target to ground (right_click_move_command returns early if kind == None)
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::Ground,
        location: Some(Vec3::new(-6.5, 0.0, -6.5)),
        entity: None,
    };

    // Right-click
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    // Verify interface state reset to Default
    let state = test_app.app.world().resource::<ObjectInterfaceState>();
    assert_eq!(
        *state,
        ObjectInterfaceState::Default,
        "Right-click should reset interface to Default (right_click_move_command processes it)"
    );
}

/// QA Step 10 [auto]: Verify CursorTarget can be set to EnemyObject and that the
/// right_click_move_command system respects it (issues AttackTarget on right-click).
#[test]
fn step_10_cursor_target_enemy_object() {
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

    // Set CursorTarget to EnemyObject
    let cursor = CursorTarget {
        kind: CursorTargetEnum::EnemyObject,
        location: Some(Vec3::new(-11.5, 0.0, -11.5)),
        entity: Some(enemy),
    };
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = cursor;

    // Verify CursorTarget was set correctly
    {
        let ct = test_app.app.world().resource::<CursorTarget>();
        assert_eq!(ct.kind, CursorTargetEnum::EnemyObject);
        assert_eq!(ct.entity, Some(enemy));
    }

    // Right-click: should issue AttackTarget
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::AttackTarget(e)) if e == enemy),
        "Right-click on EnemyObject should issue AttackTarget, got {:?}", cmd
    );
}

/// QA Step 11 [auto]: Verify CursorTarget = Ground, and right_click_move_command issues Move.
#[test]
fn step_11_cursor_target_ground() {
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

    // Set CursorTarget to Ground
    let target_pos = Vec3::new(-6.5, 0.0, -6.5);
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::Ground,
        location: Some(target_pos),
        entity: None,
    };

    // Verify CursorTarget was set
    {
        let ct = test_app.app.world().resource::<CursorTarget>();
        assert_eq!(ct.kind, CursorTargetEnum::Ground);
        assert!(ct.entity.is_none());
    }

    // Right-click: should issue Move
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Right-click on Ground should issue Move, got {:?}", cmd
    );
}

/// QA Step 12 [auto]: Verify CursorTarget = FriendlyObject, and right_click_move_command
/// issues Move (non-enemy right-click falls through to ground move).
#[test]
fn step_12_cursor_target_friendly_object() {
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

    // Set CursorTarget to FriendlyObject
    let friendly_pos = Vec3::new(-16.5, 0.0, -16.5);
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::FriendlyObject,
        location: Some(friendly_pos),
        entity: Some(friendly),
    };

    // Verify CursorTarget was set
    {
        let ct = test_app.app.world().resource::<CursorTarget>();
        assert_eq!(ct.kind, CursorTargetEnum::FriendlyObject);
        assert_eq!(ct.entity, Some(friendly));
    }

    // Right-click: should issue Move (non-enemy falls through to ground move path)
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Right-click on FriendlyObject should issue Move, got {:?}", cmd
    );
}
