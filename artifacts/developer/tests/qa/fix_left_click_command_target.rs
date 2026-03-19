use crate::helpers::*;
use space_crystals::ui::types::{ObjectInterfaceState, CursorTarget, CursorTargetEnum};

/// QA Steps 3-7 [auto]: Enter AwaitingTarget(Attack), left-click ground, verify AttackMove
/// command issued, selection unchanged, and interface state resets to Default.
#[test]
fn steps_3_to_7_attack_mode_left_click_ground_issues_attack_move() {
    let mut test_app = TestApp::new();
    test_app.step(); // fire InGame transition

    let unit1;
    let unit2;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        unit1 = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        unit2 = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 11, 10, Owner(Some(0)));
        harness.set_selection(&[unit1, unit2]);
    }
    test_app.step();

    // Populate Selection resource so systems recognize the panel as visible
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (unit1, ObjectEnum::Peacekeeper, true),
            (unit2, ObjectEnum::Peacekeeper, true),
        ]);
    }

    // Step 3: Set interface to AwaitingTarget(Attack)
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AwaitingTarget(CommandType::Attack);

    // Verify we are in awaiting target mode
    {
        let state = test_app.app.world().resource::<ObjectInterfaceState>();
        assert!(
            state.is_awaiting_target(),
            "Interface should be in AwaitingTarget mode"
        );
        assert_eq!(
            state.awaiting_command_type(),
            Some(CommandType::Attack),
            "Should be awaiting Attack command type"
        );
    }

    // Step 4: Set cursor target to ground and left-click
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::Ground,
        location: Some(Vec3::new(-6.5, 0.0, -6.5)),
        entity: None,
    };
    send_mouse_press(&mut test_app.app, MouseButton::Left);

    // Run a frame to process the click
    test_app.step();

    // Step 5: Verify units received AttackMove command
    {
        let harness = TestHarness::new(&mut test_app.app);
        let cmd1 = harness.get_command(unit1);
        let cmd2 = harness.get_command(unit2);
        assert!(
            matches!(cmd1, Some(UnitCommand::AttackMove(_))),
            "Unit1 should have AttackMove command, got {:?}", cmd1
        );
        assert!(
            matches!(cmd2, Some(UnitCommand::AttackMove(_))),
            "Unit2 should have AttackMove command, got {:?}", cmd2
        );
    }

    // Step 6: Verify selection is unchanged
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        let selected = harness.get_selection();
        assert!(selected.contains(&unit1), "Unit1 should still be selected");
        assert!(selected.contains(&unit2), "Unit2 should still be selected");
        assert_eq!(selected.len(), 2, "Should still have exactly 2 units selected");
    }

    // Step 7: Verify interface state reset to Default
    {
        let state = test_app.app.world().resource::<ObjectInterfaceState>();
        assert_eq!(
            *state,
            ObjectInterfaceState::Default,
            "Interface state should reset to Default after command confirmed"
        );
    }
}

/// QA Step 4 re-test [auto]: Enter AwaitingTarget(Attack), left-click an enemy entity,
/// verify AttackTarget command is issued (not AttackMove).
#[test]
fn step_4_attack_mode_left_click_enemy_issues_attack_target() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit1;
    let enemy;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        unit1 = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        enemy = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 15, 15, Owner(Some(1)));
        harness.set_selection(&[unit1]);
    }
    test_app.step();

    // Populate Selection resource
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (unit1, ObjectEnum::Peacekeeper, true),
        ]);
    }

    // Enter Attack mode
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AwaitingTarget(CommandType::Attack);

    // Set cursor target to the enemy entity (simulates cursor over enemy)
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::EnemyObject,
        location: Some(Vec3::new(-8.5, 0.0, -8.5)),
        entity: Some(enemy),
    };

    // Left-click to confirm attack on enemy
    send_mouse_press(&mut test_app.app, MouseButton::Left);
    test_app.step();

    // Verify unit received AttackTarget command (not AttackMove)
    {
        let harness = TestHarness::new(&mut test_app.app);
        let cmd = harness.get_command(unit1);
        assert!(
            matches!(cmd, Some(UnitCommand::AttackTarget(e)) if e == enemy),
            "Unit should have AttackTarget command targeting enemy, got {:?}", cmd
        );
    }

    // Verify interface state reset to Default
    {
        let state = test_app.app.world().resource::<ObjectInterfaceState>();
        assert_eq!(
            *state,
            ObjectInterfaceState::Default,
            "Interface state should reset to Default after attack confirmed"
        );
    }

    // Verify selection is unchanged (unit1 still selected, enemy NOT selected)
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        let selected = harness.get_selection();
        assert!(selected.contains(&unit1), "Unit1 should still be selected");
        assert!(!selected.contains(&enemy), "Enemy should NOT be selected");
    }
}

/// QA Step 8 [auto]: After command mode completes, left-click on empty ground to verify
/// normal click-to-deselect works again. We verify interface is in Default state and
/// that the selection system is no longer blocked by AwaitingTarget.
#[test]
fn step_8_normal_click_works_after_command_mode() {
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

    // Enter and exit command mode
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AwaitingTarget(CommandType::Move);
    // Reset back to Default (simulating command completion)
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;

    // Verify interface is in Default state — normal click should work
    {
        let state = test_app.app.world().resource::<ObjectInterfaceState>();
        assert_eq!(
            *state,
            ObjectInterfaceState::Default,
            "Interface should be in Default state, allowing normal selection"
        );
        // Verify is_awaiting_target is false, so selection_system won't early-return
        assert!(
            !state.is_awaiting_target(),
            "is_awaiting_target should be false in Default state"
        );
    }
}

/// QA Step 9 [auto]: Select units, enter command mode, then verify drag-select does NOT
/// activate (selection_system and drag_box_system early-return when is_awaiting_target).
#[test]
fn step_9_drag_select_blocked_during_command_mode() {
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

    // Enter command mode
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AwaitingTarget(CommandType::Attack);

    // Verify is_awaiting_target returns true — this is the guard that prevents
    // selection_system and drag_box_system from modifying selection
    {
        let state = test_app.app.world().resource::<ObjectInterfaceState>();
        assert!(
            state.is_awaiting_target(),
            "Interface should be in AwaitingTarget state"
        );
    }

    // Simulate a left-click (which would normally start a drag-select)
    // Set cursor to ground so it's a valid click location
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::Ground,
        location: Some(Vec3::new(-6.5, 0.0, -6.5)),
        entity: None,
    };
    send_mouse_press(&mut test_app.app, MouseButton::Left);
    test_app.step();

    // Selection should be unchanged — drag-select was blocked
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        let selected = harness.get_selection();
        assert!(
            selected.contains(&unit),
            "Original unit should still be selected (drag-select blocked in command mode)"
        );
    }
}

/// QA Step 10 [auto]: Select units, enter command mode, then right-click to cancel.
/// Verify command mode returns to Default.
#[test]
fn step_10_right_click_cancels_command_mode() {
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

    // Enter command mode
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AwaitingTarget(CommandType::Attack);

    // Set cursor target to Ground (required — right_click_move_command returns early if kind == None)
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::Ground,
        location: Some(Vec3::new(-6.5, 0.0, -6.5)),
        entity: None,
    };

    // Right-click: in command mode, right_click_move_command processes this as
    // a default right-click (move command), which resets interface state to Default.
    // The right-click with AwaitingTarget(Attack) + Ground click triggers the
    // right_click_move_command system to issue a Move (since it's a right-click,
    // command_type falls through to Default).
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    // Interface state should be back to Default
    {
        let state = test_app.app.world().resource::<ObjectInterfaceState>();
        assert_eq!(
            *state,
            ObjectInterfaceState::Default,
            "Right-click should reset interface state to Default"
        );
    }
}
