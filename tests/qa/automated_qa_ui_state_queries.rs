use crate::helpers::*;
use space_crystals::ui::types::{
    ObjectInterfaceState, StructureMenuState,
    CommandButtonAction, CommandButtonEnabled, CommandButtonCommon, GridSlot,
    UnitIcon, StructureIcon, ResourceIcon,
};
use space_crystals::testing::{CommandSlotInfo, InfoPanelSnapshot};

/// QA Step 1 [auto]: Spawn a Barracks, select it, advance 1 frame, call
/// `get_interface_state()` — verify it returns
/// `ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu)`.
///
/// Note: Without HudPlugin, the interface state transition system doesn't run.
/// We manually set the interface state as the system would, then verify the
/// harness query reads it correctly.
#[test]
fn step_1_get_interface_state_barracks() {
    let mut test_app = TestApp::new();
    test_app.step();

    let bk;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        bk = harness.spawn_structure_at_grid(ObjectEnum::Barracks, 20, 20, Owner(Some(0)));
        harness.set_selection(&[bk]);
    }

    // Set the interface state as the UI system would when a Barracks is selected
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);

    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let state = harness.get_interface_state();
    assert_eq!(
        state,
        ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu),
        "Interface state should be BarracksMenu after selecting a Barracks"
    );
}

/// QA Step 2 [auto]: With a Barracks selected, call `get_visible_commands()` —
/// verify the result contains expected build commands in correct slot positions
/// with correct enabled states.
///
/// Since HudPlugin is not loaded, we manually spawn button entities with the
/// required components (CommandButtonAction, GridSlot, CommandButtonEnabled,
/// CommandButtonCommon) and verify the harness query correctly reads them.
#[test]
fn step_2_get_visible_commands_barracks() {
    let mut test_app = TestApp::new();
    test_app.step();

    // Spawn Barracks button entities as the command panel system would
    test_app.app.world_mut().spawn((
        CommandButtonAction::BkTrain(ObjectEnum::Peacekeeper),
        GridSlot { row: 0, col: 0 },
        CommandButtonEnabled(true),
        CommandButtonCommon(false),
    ));
    test_app.app.world_mut().spawn((
        CommandButtonAction::BkCancel,
        GridSlot { row: 2, col: 1 },
        CommandButtonEnabled(true),
        CommandButtonCommon(false),
    ));
    test_app.app.world_mut().spawn((
        CommandButtonAction::SetRallyPoint,
        GridSlot { row: 2, col: 2 },
        CommandButtonEnabled(true),
        CommandButtonCommon(false),
    ));

    let mut harness = TestHarness::new(&mut test_app.app);
    let commands = harness.get_visible_commands();

    assert_eq!(commands.len(), 3, "Barracks should have 3 visible commands");

    // Verify slot positions (sorted by (row, col))
    assert_eq!(commands[0].slot, (0, 0), "BkTrain should be at (0,0)");
    assert!(commands[0].enabled, "BkTrain should be enabled");
    assert!(!commands[0].is_common, "BkTrain should not be common");

    assert_eq!(commands[1].slot, (2, 1), "BkCancel should be at (2,1)");
    assert!(commands[1].enabled, "BkCancel should be enabled");

    assert_eq!(commands[2].slot, (2, 2), "SetRallyPoint should be at (2,2)");
    assert!(commands[2].enabled, "SetRallyPoint should be enabled");
}

/// QA Step 3 [auto]: Select a Drone Controller, trigger the build submenu,
/// advance 1 frame, call `get_interface_state()` — verify it returns
/// `ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu)`.
#[test]
fn step_3_get_interface_state_dc_build_menu() {
    let mut test_app = TestApp::new();
    test_app.step();

    let dc;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        dc = harness.spawn_structure_at_grid(ObjectEnum::DeploymentCenter, 20, 20, Owner(Some(0)));
        harness.set_selection(&[dc]);
    }

    // Set the interface state to DcBuildMenu (as if user clicked "Build" button)
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);

    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let state = harness.get_interface_state();
    assert_eq!(
        state,
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu),
        "Interface state should be DcBuildMenu after opening DC build submenu"
    );
}

/// QA Step 4 [auto]: Select multiple units of different types, advance 1 frame,
/// call `get_selection_groups()` — verify correct groups with correct types
/// and member counts.
#[test]
fn step_4_get_selection_groups_multi_type() {
    let mut test_app = TestApp::new();
    test_app.step();

    let pk1;
    let pk2;
    let chopper;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        pk1 = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        pk2 = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 11, 10, Owner(Some(0)));
        chopper = harness.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 12, 10, Owner(Some(0)));
        harness.set_selection(&[pk1, pk2, chopper]);
    }

    // Build selection groups properly
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (pk1, ObjectEnum::Peacekeeper, true),
            (pk2, ObjectEnum::Peacekeeper, true),
            (chopper, ObjectEnum::SupplyChopper, true),
        ]);
    }

    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let groups = harness.get_selection_groups();

    assert_eq!(groups.len(), 2, "Should have 2 groups: Peacekeeper and SupplyChopper");

    // Find the Peacekeeper group
    let pk_group = groups.iter().find(|g| g.object_type == ObjectEnum::Peacekeeper)
        .expect("Peacekeeper group should exist");
    assert_eq!(pk_group.entities.len(), 2, "Peacekeeper group should have 2 entities");
    assert!(pk_group.entities.contains(&pk1));
    assert!(pk_group.entities.contains(&pk2));

    // Find the SupplyChopper group
    let ch_group = groups.iter().find(|g| g.object_type == ObjectEnum::SupplyChopper)
        .expect("SupplyChopper group should exist");
    assert_eq!(ch_group.entities.len(), 1, "SupplyChopper group should have 1 entity");
    assert!(ch_group.entities.contains(&chopper));
}

/// QA Step 5 [auto]: With a multi-group selection, call `get_active_group()` —
/// verify it returns the expected active group. Cycle active group and re-query —
/// verify the return value changes.
#[test]
fn step_5_get_active_group_and_cycle() {
    let mut test_app = TestApp::new();
    test_app.step();

    let pk;
    let chopper;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        pk = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        chopper = harness.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 12, 10, Owner(Some(0)));
    }

    // Build selection with two groups
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (pk, ObjectEnum::Peacekeeper, true),
            (chopper, ObjectEnum::SupplyChopper, true),
        ]);
    }

    // Active group should be the first group (index 0)
    let harness = TestHarness::new(&mut test_app.app);
    let active = harness.get_active_group().expect("Should have an active group");
    let first_type = active.object_type;

    // Cycle to next group
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        let current_index = selection.active_group_index.unwrap_or(0);
        let next_index = (current_index + 1) % selection.groups.len();
        selection.active_group_index = Some(next_index);
    }

    let harness = TestHarness::new(&mut test_app.app);
    let active2 = harness.get_active_group().expect("Should still have an active group");

    assert_ne!(
        first_type, active2.object_type,
        "Active group type should change after cycling: first was {:?}, second was {:?}",
        first_type, active2.object_type
    );
}

/// QA Step 6 [auto]: Select a single unit, advance 1 frame, call `get_info_panel()` —
/// verify the returned snapshot matches the entity's name, health, and type data.
#[test]
fn step_6_get_info_panel_single_unit() {
    let mut test_app = TestApp::new();
    test_app.step();

    let pk;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        pk = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        harness.set_selection(&[pk]);
    }

    // Build selection
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (pk, ObjectEnum::Peacekeeper, true),
        ]);
    }

    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let snapshot = harness.get_info_panel().expect("Info panel should show selected unit");

    assert_eq!(snapshot.entity, pk, "Info panel entity should be the selected Peacekeeper");
    assert_eq!(snapshot.object_type, ObjectEnum::Peacekeeper, "Object type should be Peacekeeper");
    assert!(snapshot.hp.is_some(), "Peacekeeper should have HP");
    let (hp, max_hp) = snapshot.hp.unwrap();
    assert_eq!(hp, max_hp, "Newly spawned unit should be at full HP");
    assert!(max_hp > 0.0, "Max HP should be positive");
}

/// QA Step 7 [auto]: Select multiple units, advance 1 frame, call
/// `get_selection_panel_portraits()` — verify it returns the correct entities
/// with ActiveGroup highlighting on the expected subset.
#[test]
fn step_7_get_selection_panel_portraits() {
    let mut test_app = TestApp::new();
    test_app.step();

    let pk;
    let chopper;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        pk = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        chopper = harness.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 12, 10, Owner(Some(0)));
        // Mark both as Selected so selection sync doesn't clear them
        harness.set_selection(&[pk, chopper]);
    }

    // Build selection groups with Peacekeeper as active group (index 0)
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (pk, ObjectEnum::Peacekeeper, true),
            (chopper, ObjectEnum::SupplyChopper, true),
        ]);
    }

    // Manually spawn portrait UI entities (HudPlugin not loaded)
    test_app.app.world_mut().spawn(UnitIcon { unit_entity: pk });
    test_app.app.world_mut().spawn(UnitIcon { unit_entity: chopper });

    // Query portraits WITHOUT stepping (to avoid selection sync overwriting our groups)
    let mut harness = TestHarness::new(&mut test_app.app);
    let portraits = harness.get_selection_panel_portraits();

    assert_eq!(portraits.len(), 2, "Should have 2 portraits");

    // Peacekeeper should be in active group (first group)
    let pk_portrait = portraits.iter().find(|(e, _)| *e == pk)
        .expect("Peacekeeper portrait should exist");
    assert!(pk_portrait.1, "Peacekeeper should be highlighted as active group");

    // SupplyChopper should NOT be in active group
    let ch_portrait = portraits.iter().find(|(e, _)| *e == chopper)
        .expect("SupplyChopper portrait should exist");
    assert!(!ch_portrait.1, "SupplyChopper should NOT be highlighted (not in active group)");
}

/// QA Step 8 [auto]: Use `assert_command_visible(slot, action)` for a known
/// visible command — verify it passes. Use `assert_command_not_visible(action)`
/// for a command not in the panel — verify it passes. Use `assert_command_visible`
/// for a command not present — verify it panics/fails.
#[test]
fn step_8_assert_command_visible_and_not_visible() {
    let mut test_app = TestApp::new();
    test_app.step();

    // Spawn a visible Move button at (0,0)
    test_app.app.world_mut().spawn((
        CommandButtonAction::UnitMove,
        GridSlot { row: 0, col: 0 },
        CommandButtonEnabled(true),
        CommandButtonCommon(true),
    ));

    // assert_command_visible should pass for UnitMove at (0,0)
    assert_command_visible(
        test_app.app.world_mut(),
        (0, 0),
        &CommandButtonAction::UnitMove,
    );

    // assert_command_not_visible should pass for UnitPatrol (not spawned)
    assert_command_not_visible(
        test_app.app.world_mut(),
        &CommandButtonAction::UnitPatrol,
    );
}

/// QA Step 8 supplement: Verify assert_command_visible panics for non-present command.
#[test]
#[should_panic(expected = "Expected command")]
fn step_8_assert_command_visible_panics_for_missing() {
    let mut test_app = TestApp::new();
    test_app.step();

    // No buttons spawned — assert_command_visible should panic
    assert_command_visible(
        test_app.app.world_mut(),
        (0, 0),
        &CommandButtonAction::UnitAttack,
    );
}

/// QA Step 9 [auto]: Use `assert_interface_state` with the correct state —
/// verify pass. Use it with the wrong state — verify failure.
#[test]
fn step_9_assert_interface_state_correct() {
    let mut test_app = TestApp::new();
    test_app.step();

    // Default state
    assert_interface_state(
        test_app.app.world(),
        ObjectInterfaceState::Default,
    );

    // Change to BarracksMenu and verify
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);

    assert_interface_state(
        test_app.app.world(),
        ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu),
    );
}

/// QA Step 9 supplement: Verify assert_interface_state panics with wrong state.
#[test]
#[should_panic(expected = "Interface state mismatch")]
fn step_9_assert_interface_state_panics_wrong() {
    let mut test_app = TestApp::new();
    test_app.step();

    // State is Default, but we assert BarracksMenu — should panic
    assert_interface_state(
        test_app.app.world(),
        ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu),
    );
}

/// QA Step 10 [auto]: Use `assert_active_group_type` and `assert_info_panel_shows`
/// with correct and incorrect values — verify pass and failure respectively.
#[test]
fn step_10_assert_active_group_type_correct() {
    let mut test_app = TestApp::new();
    test_app.step();

    let pk;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        pk = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
    }

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (pk, ObjectEnum::Peacekeeper, true),
        ]);
    }

    // assert_active_group_type should pass for Peacekeeper
    assert_active_group_type(test_app.app.world(), ObjectEnum::Peacekeeper);

    // assert_info_panel_shows should pass for the selected entity
    assert_info_panel_shows(test_app.app.world(), pk);
}

/// QA Step 10 supplement: Verify assert_active_group_type panics with wrong type.
#[test]
#[should_panic(expected = "Active group type mismatch")]
fn step_10_assert_active_group_type_panics_wrong() {
    let mut test_app = TestApp::new();
    test_app.step();

    let pk;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        pk = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
    }

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (pk, ObjectEnum::Peacekeeper, true),
        ]);
    }

    // Active group is Peacekeeper, but we assert SupplyChopper — should panic
    assert_active_group_type(test_app.app.world(), ObjectEnum::SupplyChopper);
}

/// QA Step 10 supplement: Verify assert_info_panel_shows panics for wrong entity.
#[test]
#[should_panic(expected = "not in the active selection group")]
fn step_10_assert_info_panel_shows_panics_wrong_entity() {
    let mut test_app = TestApp::new();
    test_app.step();

    let pk;
    let other;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        pk = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        other = harness.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 12, 10, Owner(Some(0)));
    }

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (pk, ObjectEnum::Peacekeeper, true),
        ]);
    }

    // Only pk is selected, but we assert 'other' — should panic
    assert_info_panel_shows(test_app.app.world(), other);
}
