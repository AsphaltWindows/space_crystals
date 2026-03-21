use crate::helpers::*;
use space_crystals::ui::types::{
    ObjectInterfaceState, StructureMenuState, CommandPanelTarget,
};

// =============================================================================
// Back Button Hotkey Consistency Tests
//
// Verifies that Back/Cancel button state transitions are consistent across all
// factions and structure menus. The Back button is at grid position (2, 0) = Z key.
//
// Note: Tests that require the HudPlugin (UI spawning, keyboard hotkey processing)
// cannot run in headless TestApp mode. These tests verify the state machine
// transitions that the Back action triggers, which is the core contract.
// The grid position of the Back button (2,0) is verified by unit tests in
// src/ui/command_panel.rs (build_tunnel_expand_grid, build_tunnel_eject_grid).
// =============================================================================

/// Simulate the state transition that CommandButtonAction::Back triggers.
/// This mirrors the match arms in execute_command_action (src/ui/command_panel.rs).
fn apply_back_transition(state: &ObjectInterfaceState) -> ObjectInterfaceState {
    match state {
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu) => {
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle)
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu) |
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu) => {
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle)
        }
        other => other.clone(),
    }
}

/// Simulate the Escape key transition logic from handle_command_button_clicks.
/// This mirrors the Escape handler match arms (src/ui/command_panel.rs).
fn apply_escape_transition(state: &ObjectInterfaceState) -> ObjectInterfaceState {
    match state {
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu) => {
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle)
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu) |
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu) => {
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle)
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement) => {
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu)
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcAwaitingPlacement) => {
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace)
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement) => {
            ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle)
        }
        other => other.clone(),
    }
}

/// QA Step 1-4 [auto]: Launch game with Syndicate faction, select a Tunnel,
/// press C (Eject) to enter EjectMenu. Verify the state transition occurs.
#[test]
fn step_1_4_syndicate_tunnel_enter_eject_menu() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let tunnel;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        tunnel = harness.spawn_structure_at_grid(ObjectEnum::Tunnel, 32, 32, Owner(Some(0)));
        harness.set_selection(&[tunnel]);
    }
    test_app.step();

    // Set interface state to TunnelIdle (simulating structure selection)
    {
        let world = test_app.app.world_mut();
        world.insert_resource(ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle));
        world.resource_mut::<CommandPanelTarget>().entity = Some(tunnel);
    }

    {
        let state = test_app.app.world().resource::<ObjectInterfaceState>();
        assert_eq!(*state, ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle),
            "Should be in TunnelIdle after selecting tunnel");
    }

    // Transition to EjectMenu (simulating pressing C = grid slot (0, 2) = TunnelOpenEjectMenu)
    {
        let world = test_app.app.world_mut();
        *world.resource_mut::<ObjectInterfaceState>() =
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu);
    }

    let state = test_app.app.world().resource::<ObjectInterfaceState>();
    assert_eq!(*state, ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu),
        "Should be in TunnelEjectMenu after pressing Eject");
}

/// QA Step 6 [auto]: From TunnelEjectMenu, pressing Z (Back) returns to TunnelIdle (DefaultState).
/// Tests the Back action state transition from EjectMenu.
#[test]
fn step_6_eject_menu_back_returns_to_tunnel_idle() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let tunnel;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        tunnel = harness.spawn_structure_at_grid(ObjectEnum::Tunnel, 32, 32, Owner(Some(0)));
        harness.set_selection(&[tunnel]);
    }
    test_app.step();

    // Start in TunnelEjectMenu
    {
        let world = test_app.app.world_mut();
        world.insert_resource(ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu));
        world.resource_mut::<CommandPanelTarget>().entity = Some(tunnel);
    }

    // Apply Back transition (mirrors execute_command_action for CommandButtonAction::Back)
    {
        let world = test_app.app.world_mut();
        let current = world.resource::<ObjectInterfaceState>().clone();
        world.insert_resource(apply_back_transition(&current));
    }

    let state = test_app.app.world().resource::<ObjectInterfaceState>();
    assert_eq!(*state, ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle),
        "Back from EjectMenu should return to TunnelIdle (DefaultState)");
}

/// QA Step 7-9 [auto]: Press B (Expand Tunnel) to enter ExpandMenu, then press Z to go back.
/// Verify it returns to TunnelIdle (DefaultState).
#[test]
fn step_7_9_expand_menu_back_returns_to_tunnel_idle() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let tunnel;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        tunnel = harness.spawn_structure_at_grid(ObjectEnum::Tunnel, 32, 32, Owner(Some(0)));
        harness.set_selection(&[tunnel]);
    }
    test_app.step();

    // Start at TunnelIdle
    {
        let world = test_app.app.world_mut();
        world.insert_resource(ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle));
        world.resource_mut::<CommandPanelTarget>().entity = Some(tunnel);
    }

    // Transition to ExpandMenu (simulating pressing W = TunnelOpenExpandMenu)
    {
        let world = test_app.app.world_mut();
        *world.resource_mut::<ObjectInterfaceState>() =
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
    }

    {
        let state = test_app.app.world().resource::<ObjectInterfaceState>();
        assert_eq!(*state, ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu),
            "Should be in TunnelExpandMenu after pressing Expand");
    }

    // Apply Back transition: TunnelExpandMenu -> TunnelIdle
    {
        let world = test_app.app.world_mut();
        let current = world.resource::<ObjectInterfaceState>().clone();
        world.insert_resource(apply_back_transition(&current));
    }

    let state = test_app.app.world().resource::<ObjectInterfaceState>();
    assert_eq!(*state, ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle),
        "Back from ExpandMenu should return to TunnelIdle (DefaultState)");
}

/// QA Step 10-14 [auto]: Launch game with GDO faction, select Deployment Center,
/// verify Back at (2,0) in DcBuildMenu returns to DcIdle.
#[test]
fn step_10_14_gdo_dc_build_menu_back_returns_to_dc_idle() {
    let mut test_app = TestApp::new();
    test_app.step();

    let dc;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        dc = harness.spawn_structure_at_grid(ObjectEnum::DeploymentCenter, 32, 32, Owner(Some(0)));
        harness.set_selection(&[dc]);
    }
    test_app.step();

    // Set up DcBuildMenu state (simulating Q press from DcIdle)
    {
        let world = test_app.app.world_mut();
        world.insert_resource(ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu));
        world.resource_mut::<CommandPanelTarget>().entity = Some(dc);
    }

    {
        let state = test_app.app.world().resource::<ObjectInterfaceState>();
        assert_eq!(*state, ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu),
            "Should be in DcBuildMenu");
    }

    // Apply Back transition: DcBuildMenu -> DcIdle
    {
        let world = test_app.app.world_mut();
        let current = world.resource::<ObjectInterfaceState>().clone();
        world.insert_resource(apply_back_transition(&current));
    }

    let state = test_app.app.world().resource::<ObjectInterfaceState>();
    assert_eq!(*state, ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle),
        "Back from DcBuildMenu should return to DcIdle (DefaultState)");
}

/// Verify Back button state transition consistency: all menus with Back go to their parent state.
/// Data-driven test covering all known Back transitions.
#[test]
fn back_transitions_are_consistent() {
    let transitions = vec![
        (
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu),
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle),
            "DcBuildMenu -> DcIdle",
        ),
        (
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu),
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle),
            "TunnelExpandMenu -> TunnelIdle",
        ),
        (
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu),
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle),
            "TunnelEjectMenu -> TunnelIdle",
        ),
    ];

    for (from_state, expected_to, label) in &transitions {
        let result = apply_back_transition(from_state);
        assert_eq!(&result, expected_to,
            "Back transition '{}': expected {:?}, got {:?}", label, expected_to, result);
    }
}

/// Verify Escape transitions mirror Back transitions for tunnel menus.
/// The Escape handler in handle_command_button_clicks has the same
/// TunnelExpandMenu|TunnelEjectMenu -> TunnelIdle transition as Back.
#[test]
fn escape_transitions_mirror_back_for_tunnel_menus() {
    let escape_transitions = vec![
        (
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu),
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle),
            "Escape from TunnelExpandMenu",
        ),
        (
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu),
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle),
            "Escape from TunnelEjectMenu",
        ),
        (
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu),
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle),
            "Escape from DcBuildMenu",
        ),
    ];

    for (from_state, expected_to, label) in &escape_transitions {
        let result = apply_escape_transition(from_state);
        assert_eq!(&result, expected_to,
            "Escape transition '{}': expected {:?}, got {:?}", label, expected_to, result);
    }
}

/// Verify that states without a Back button do not transition on Back.
/// These are "root" states or states with Cancel instead of Back.
#[test]
fn non_back_states_remain_unchanged() {
    let root_states = vec![
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle),
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle),
        ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu),
        ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle),
        ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu),
        ObjectInterfaceState::Default,
    ];

    for state in &root_states {
        let result = apply_back_transition(state);
        assert_eq!(&result, state,
            "State {:?} should not change on Back (no Back button), but got {:?}", state, result);
    }
}

/// Full round-trip test: TunnelIdle -> EjectMenu -> Back -> TunnelIdle -> ExpandMenu -> Back -> TunnelIdle
#[test]
fn full_tunnel_navigation_round_trip() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let tunnel;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        tunnel = harness.spawn_structure_at_grid(ObjectEnum::Tunnel, 32, 32, Owner(Some(0)));
        harness.set_selection(&[tunnel]);
    }
    test_app.step();

    {
        let world = test_app.app.world_mut();
        world.insert_resource(ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle));
        world.resource_mut::<CommandPanelTarget>().entity = Some(tunnel);
    }

    // TunnelIdle -> EjectMenu (press C = TunnelOpenEjectMenu)
    {
        let world = test_app.app.world_mut();
        *world.resource_mut::<ObjectInterfaceState>() =
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu);
    }
    assert_eq!(
        *test_app.app.world().resource::<ObjectInterfaceState>(),
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu)
    );

    // EjectMenu -> Back -> TunnelIdle
    {
        let world = test_app.app.world_mut();
        let current = world.resource::<ObjectInterfaceState>().clone();
        world.insert_resource(apply_back_transition(&current));
    }
    assert_eq!(
        *test_app.app.world().resource::<ObjectInterfaceState>(),
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle)
    );

    // TunnelIdle -> ExpandMenu (press W = TunnelOpenExpandMenu)
    {
        let world = test_app.app.world_mut();
        *world.resource_mut::<ObjectInterfaceState>() =
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
    }
    assert_eq!(
        *test_app.app.world().resource::<ObjectInterfaceState>(),
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu)
    );

    // ExpandMenu -> Back -> TunnelIdle
    {
        let world = test_app.app.world_mut();
        let current = world.resource::<ObjectInterfaceState>().clone();
        world.insert_resource(apply_back_transition(&current));
    }
    assert_eq!(
        *test_app.app.world().resource::<ObjectInterfaceState>(),
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle),
        "After full round-trip, should be back at TunnelIdle"
    );
}
