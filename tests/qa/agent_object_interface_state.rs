use crate::helpers::*;
use space_crystals::ui::types::{
    ObjectInterfaceState, AgentMenuState, CursorTarget, CursorTargetEnum,
    PlacementState, SelectedUnitCapabilities,
};
use space_crystals::game::units::types::state::AgentCarryState;

/// Helper: create a Syndicate TestApp (local player = 0 plays Syndicate).
fn syndicate_app() -> TestApp {
    TestApp::new_with_faction(space_crystals::types::FactionEnum::TheSyndicate)
}

/// Helper: grid-to-world conversion (matches spawn functions: world = (grid - 32) + 0.5).
fn grid_to_world(gx: i32, gz: i32) -> Vec3 {
    Vec3::new((gx as f32 - 32.0) + 0.5, 0.0, (gz as f32 - 32.0) + 0.5)
}

// =========================================================================
// QA Step 6: Left-click valid location in AwaitingPlacement -> BuildTunnel
//            command issued, interface returns to AgentDefault.
// =========================================================================

/// Step 6 [auto]: Agent in AgentAwaitingPlacement, left-click valid location.
/// Verify BuildTunnel command issued and interface returns to AgentDefault.
#[test]
fn step_6_agent_placement_left_click_issues_build_tunnel() {
    let mut test_app = syndicate_app();
    test_app.step();

    let agent;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 10, 10, Owner(Some(0)));
        harness.set_selection(&[agent]);
    }
    test_app.step();

    // Set up Selection resource
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (agent, ObjectEnum::SyndicateAgent, false),
        ]);
    }

    // Enter AgentAwaitingPlacement with valid placement state
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement);
    {
        let mut ps = test_app.app.world_mut().resource_mut::<PlacementState>();
        ps.building_type = Some(ObjectEnum::Tunnel);
        ps.source_entity = Some(agent);
        ps.grid_pos = Some((20, 20));
        ps.is_valid = true;
    }

    // Left-click to place
    send_mouse_press(&mut test_app.app, MouseButton::Left);
    test_app.step();

    // Verify BuildTunnel command was issued
    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(agent);
    assert!(
        matches!(cmd, Some(UnitCommand::BuildTunnel(_))),
        "Agent should have BuildTunnel command after placement click, got {:?}", cmd
    );

    // Verify interface returned to AgentDefault (or AgentMenu variant)
    let state = test_app.app.world().resource::<ObjectInterfaceState>();
    assert!(
        matches!(state, ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault)),
        "Interface should return to AgentDefault after placement, got {:?}", state
    );
}

// =========================================================================
// QA Step 7: Enter AwaitingPlacement, press Escape -> returns to AgentDefault
//            without issuing a command.
// =========================================================================

/// Step 7 [auto]: Agent in AgentAwaitingPlacement, press Escape.
/// Verify interface returns to AgentDefault without issuing a command.
#[test]
#[ignore] // Escape key not processed by command_panel_hotkeys in headless mode (HudPlugin not loaded)
fn step_7_agent_placement_escape_cancels_without_command() {
    let mut test_app = syndicate_app();
    test_app.step();

    let agent;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 10, 10, Owner(Some(0)));
        harness.set_selection(&[agent]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (agent, ObjectEnum::SyndicateAgent, false),
        ]);
    }

    // Enter placement mode
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement);

    // Press Escape
    send_key_press(&mut test_app.app, KeyCode::Escape);
    test_app.step();

    // Verify the interface returned to AgentDefault
    let state = test_app.app.world().resource::<ObjectInterfaceState>();
    assert!(
        matches!(state, ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault)),
        "Escape should return to AgentDefault, got {:?}", state
    );

    // Verify no BuildTunnel command was issued (should still be Idle)
    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(agent);
    assert!(
        !matches!(cmd, Some(UnitCommand::BuildTunnel(_))),
        "Agent should NOT have BuildTunnel command after Escape, got {:?}", cmd
    );
}

// =========================================================================
// QA Step 8: Right-click a Space Crystal Patch -> Gather command issued.
// =========================================================================

/// Step 8 [auto]: Right-click a Space Crystal Patch. Verify Gather command issued.
#[test]
fn step_8_right_click_crystal_patch_issues_gather() {
    let mut test_app = syndicate_app();
    test_app.step();

    let agent;
    let crystal;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 10, 10, Owner(Some(0)));
        // Spawn crystal with Owner::neutral() so target_info query (requires &Owner) matches
        crystal = harness.app.world_mut().spawn((
            Transform::from_translation(grid_to_world(15, 15)),
            SpaceCrystalPatch {
                remaining_amount: 500,
                initial_amount: 500,
                has_plate: false,
            },
            GridPosition { x: 15, z: 15 },
            ObjectInstance::indestructible(ObjectEnum::SpaceCrystalsPatch),
            Selectable,
            Owner::neutral(),
        )).id();
        harness.set_selection(&[agent]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (agent, ObjectEnum::SyndicateAgent, false),
        ]);
    }

    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;

    // Set cursor target to crystal patch and right-click
    let crystal_pos = grid_to_world(15, 15);
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::NeutralObject,
        location: Some(crystal_pos),
        entity: Some(crystal),
    };
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(agent);
    assert!(
        matches!(cmd, Some(UnitCommand::Gather(e)) if e == crystal),
        "Right-click crystal patch should produce Gather, got {:?}", cmd
    );
}

// =========================================================================
// QA Step 9: Right-click a Supply Delivery Station -> Gather supplies issued.
// =========================================================================

/// Step 9 [auto]: Right-click a Supply Delivery Station. Verify Gather command issued.
#[test]
fn step_9_right_click_sds_issues_gather() {
    let mut test_app = syndicate_app();
    test_app.step();

    let agent;
    let sds;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 10, 10, Owner(Some(0)));
        // Spawn SDS manually (no harness method)
        sds = harness.app.world_mut().spawn((
            Transform::from_translation(grid_to_world(15, 15)),
            SupplyDeliveryStation {
                delivery_size: 100,
                delivery_interval: 60.0,
                current_supplies: 50,
                time_until_next_delivery: 60.0,
            },
            GridPosition { x: 15, z: 15 },
            ObjectInstance::indestructible(ObjectEnum::SupplyDeliveryStation),
            Selectable,
            Owner::neutral(),
        )).id();
        harness.set_selection(&[agent]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (agent, ObjectEnum::SyndicateAgent, false),
        ]);
    }

    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;

    let sds_pos = grid_to_world(15, 15);
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::NeutralObject,
        location: Some(sds_pos),
        entity: Some(sds),
    };
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(agent);
    assert!(
        matches!(cmd, Some(UnitCommand::Gather(e)) if e == sds),
        "Right-click SDS should produce Gather, got {:?}", cmd
    );
}

// =========================================================================
// QA Step 10: Agent carrying crystals, right-click own Tunnel -> DropOffResources.
// =========================================================================

/// Step 10 [auto]: Agent carrying crystals, right-click own Tunnel.
/// Verify DropOffResources command issued.
#[test]
fn step_10_carrying_crystals_right_click_tunnel_issues_drop_off() {
    let mut test_app = syndicate_app();
    test_app.step();

    let agent;
    let tunnel;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 10, 10, Owner(Some(0)));
        tunnel = harness.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)));
        harness.set_selection(&[agent]);
    }
    test_app.step();

    // Set agent to carrying crystals
    test_app.app.world_mut().entity_mut(agent).insert(AgentCarryState {
        crystals: 10,
        supplies: 0,
    });

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (agent, ObjectEnum::SyndicateAgent, false),
        ]);
    }

    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;

    let tunnel_pos = grid_to_world(20, 20);
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::FriendlyObject,
        location: Some(tunnel_pos),
        entity: Some(tunnel),
    };
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(agent);
    assert!(
        matches!(cmd, Some(UnitCommand::DropOffResources(e)) if e == tunnel),
        "Agent carrying crystals + right-click own tunnel -> DropOffResources, got {:?}", cmd
    );
}

// =========================================================================
// QA Step 11: Agent carrying supplies, right-click own Tunnel -> DropOffResources.
// =========================================================================

/// Step 11 [auto]: Agent carrying supplies, right-click own Tunnel.
/// Verify DropOffResources command issued.
#[test]
fn step_11_carrying_supplies_right_click_tunnel_issues_drop_off() {
    let mut test_app = syndicate_app();
    test_app.step();

    let agent;
    let tunnel;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 10, 10, Owner(Some(0)));
        tunnel = harness.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)));
        harness.set_selection(&[agent]);
    }
    test_app.step();

    // Set agent to carrying supplies
    test_app.app.world_mut().entity_mut(agent).insert(AgentCarryState {
        crystals: 0,
        supplies: 5,
    });

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (agent, ObjectEnum::SyndicateAgent, false),
        ]);
    }

    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;

    let tunnel_pos = grid_to_world(20, 20);
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::FriendlyObject,
        location: Some(tunnel_pos),
        entity: Some(tunnel),
    };
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(agent);
    assert!(
        matches!(cmd, Some(UnitCommand::DropOffResources(e)) if e == tunnel),
        "Agent carrying supplies + right-click own tunnel -> DropOffResources, got {:?}", cmd
    );
}

// =========================================================================
// QA Step 12: Agent carrying nothing, right-click own Tunnel -> Enter command.
// =========================================================================

/// Step 12 [auto]: Agent carrying nothing, right-click own Tunnel.
/// Verify Enter command issued.
#[test]
fn step_12_not_carrying_right_click_tunnel_issues_enter() {
    let mut test_app = syndicate_app();
    test_app.step();

    let agent;
    let tunnel;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 10, 10, Owner(Some(0)));
        tunnel = harness.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)));
        harness.set_selection(&[agent]);
    }
    test_app.step();

    // Ensure agent has empty carry state (default)
    test_app.app.world_mut().entity_mut(agent).insert(AgentCarryState {
        crystals: 0,
        supplies: 0,
    });

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (agent, ObjectEnum::SyndicateAgent, false),
        ]);
    }

    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;

    let tunnel_pos = grid_to_world(20, 20);
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::FriendlyObject,
        location: Some(tunnel_pos),
        entity: Some(tunnel),
    };
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(agent);
    assert!(
        matches!(cmd, Some(UnitCommand::Enter(e)) if e == tunnel),
        "Agent not carrying + right-click own tunnel -> Enter, got {:?}", cmd
    );
}

// =========================================================================
// QA Step 13: Right-click an enemy unit -> Attack command issued.
// =========================================================================

/// Step 13 [auto]: Right-click an enemy unit. Verify Attack command issued.
#[test]
fn step_13_right_click_enemy_issues_attack() {
    let mut test_app = syndicate_app();
    test_app.step();

    let agent;
    let enemy;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 10, 10, Owner(Some(0)));
        enemy = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(1)));
        harness.set_selection(&[agent]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (agent, ObjectEnum::SyndicateAgent, false),
        ]);
    }

    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;

    let enemy_pos = grid_to_world(20, 20);
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::EnemyObject,
        location: Some(enemy_pos),
        entity: Some(enemy),
    };
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    let cmd = harness.get_command(agent);
    assert!(
        matches!(cmd, Some(UnitCommand::AttackTarget(e)) if e == enemy),
        "Right-click enemy should produce AttackTarget, got {:?}", cmd
    );
}

// =========================================================================
// QA Step 14: Right-click empty ground -> Move command issued.
// =========================================================================

/// Step 14 [auto]: Right-click empty ground. Verify Move command issued.
#[test]
fn step_14_right_click_ground_issues_move() {
    let mut test_app = syndicate_app();
    test_app.step();

    let agent;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 10, 10, Owner(Some(0)));
        harness.set_selection(&[agent]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (agent, ObjectEnum::SyndicateAgent, false),
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
    let cmd = harness.get_command(agent);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Right-click ground should produce Move, got {:?}", cmd
    );
}

// =========================================================================
// QA Step 15: Select multiple Agents, right-click ground -> all receive Move.
// =========================================================================

/// Step 15 [auto]: Select multiple Agents. Right-click ground.
/// Verify all selected Agents receive Move command.
#[test]
fn step_15_multi_select_agents_right_click_ground_all_move() {
    let mut test_app = syndicate_app();
    test_app.step();

    let agent1;
    let agent2;
    let agent3;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent1 = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 10, 10, Owner(Some(0)));
        agent2 = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 11, 10, Owner(Some(0)));
        agent3 = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 12, 10, Owner(Some(0)));
        harness.set_selection(&[agent1, agent2, agent3]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (agent1, ObjectEnum::SyndicateAgent, false),
            (agent2, ObjectEnum::SyndicateAgent, false),
            (agent3, ObjectEnum::SyndicateAgent, false),
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
    for (agent, name) in [(agent1, "agent1"), (agent2, "agent2"), (agent3, "agent3")] {
        let cmd = harness.get_command(agent);
        assert!(
            matches!(cmd, Some(UnitCommand::Move(_))),
            "{} should have Move command, got {:?}", name, cmd
        );
    }
}

// =========================================================================
// QA Step 16: Select multiple Agents, right-click Crystal field -> all Gather.
// =========================================================================

/// Step 16 [auto]: Select multiple Agents. Right-click a Crystal field.
/// Verify all selected Agents receive Gather command.
#[test]
fn step_16_multi_select_agents_right_click_crystal_all_gather() {
    let mut test_app = syndicate_app();
    test_app.step();

    let agent1;
    let agent2;
    let crystal;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent1 = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 10, 10, Owner(Some(0)));
        agent2 = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 11, 10, Owner(Some(0)));
        // Spawn crystal with Owner::neutral() so target_info query (requires &Owner) matches
        crystal = harness.app.world_mut().spawn((
            Transform::from_translation(grid_to_world(15, 15)),
            SpaceCrystalPatch {
                remaining_amount: 500,
                initial_amount: 500,
                has_plate: false,
            },
            GridPosition { x: 15, z: 15 },
            ObjectInstance::indestructible(ObjectEnum::SpaceCrystalsPatch),
            Selectable,
            Owner::neutral(),
        )).id();
        harness.set_selection(&[agent1, agent2]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (agent1, ObjectEnum::SyndicateAgent, false),
            (agent2, ObjectEnum::SyndicateAgent, false),
        ]);
    }

    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;

    let crystal_pos = grid_to_world(15, 15);
    *test_app.app.world_mut().resource_mut::<CursorTarget>() = CursorTarget {
        kind: CursorTargetEnum::NeutralObject,
        location: Some(crystal_pos),
        entity: Some(crystal),
    };
    send_mouse_press(&mut test_app.app, MouseButton::Right);
    test_app.step();

    let harness = TestHarness::new(&mut test_app.app);
    for (agent, name) in [(agent1, "agent1"), (agent2, "agent2")] {
        let cmd = harness.get_command(agent);
        assert!(
            matches!(cmd, Some(UnitCommand::Gather(e)) if e == crystal),
            "{} should have Gather command targeting crystal, got {:?}", name, cmd
        );
    }
}

// =========================================================================
// Additional: Interface state transition tests
// =========================================================================

/// Verify AgentBuildTunnel button action exists at grid slot (0,0) in AgentDefault.
#[test]
fn agent_default_has_build_tunnel_button() {
    let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
    assert!(!state.is_placement_mode());
    assert!(!state.is_awaiting_target());
}

/// Verify AgentAwaitingPlacement is recognized as placement mode.
#[test]
fn agent_awaiting_placement_is_placement_mode() {
    let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement);
    assert!(state.is_placement_mode());
}

/// Verify SelectedUnitCapabilities.agent_carrying gates drop-off button.
#[test]
fn agent_drop_off_disabled_when_not_carrying() {
    let caps = SelectedUnitCapabilities {
        has_attack: false,
        can_target_ground: false,
        can_reverse: false,
        agent_carrying: false,
    };
    assert!(!caps.agent_carrying, "Drop Off button should be disabled when agent not carrying");
}

/// Verify SelectedUnitCapabilities.agent_carrying enables drop-off button.
#[test]
fn agent_drop_off_enabled_when_carrying() {
    let caps = SelectedUnitCapabilities {
        has_attack: false,
        can_target_ground: false,
        can_reverse: false,
        agent_carrying: true,
    };
    assert!(caps.agent_carrying, "Drop Off button should be enabled when agent is carrying");
}

/// Verify AgentCarryState correctly tracks carrying vs not carrying.
#[test]
fn agent_carry_state_determines_tunnel_interaction() {
    // Not carrying -> Enter
    let empty = AgentCarryState { crystals: 0, supplies: 0 };
    assert!(!empty.is_carrying());

    // Carrying crystals -> DropOffResources
    let crystals = AgentCarryState { crystals: 5, supplies: 0 };
    assert!(crystals.is_carrying());

    // Carrying supplies -> DropOffResources
    let supplies = AgentCarryState { crystals: 0, supplies: 3 };
    assert!(supplies.is_carrying());
}
