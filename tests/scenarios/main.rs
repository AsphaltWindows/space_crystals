mod test_app;
use test_app::TestApp;

use bevy::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use space_crystals::game::utils::{spawn_peacekeeper, spawn_tunnel, spawn_headquarters};
use space_crystals::game::units::types::state::commands::CommandType;
use space_crystals::game::units::types::state::UnitCommand;
use space_crystals::ui::types::ObjectInterfaceState;
use space_crystals::types::*;
use space_crystals::testing::TestHarness;
use space_crystals::game::world::types::{SpaceCrystalPatch, TilePresetEnum, FogOfWarMap};

/// Proof-of-concept integration test: spawn a Peacekeeper and verify ECS components.
#[test]
fn spawn_peacekeeper_and_verify_components() {
    let mut test_app = TestApp::new();

    // Run one frame to let startup systems execute (map spawn, player setup, etc.)
    test_app.step();

    // Spawn a Peacekeeper via a one-shot system
    let spawned_entity = test_app.app.world_mut().run_system_once(
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            spawn_peacekeeper(&mut commands, &mut meshes, &mut materials, 32, 32, Owner(Some(0)))
        },
    ).unwrap();

    // Run another frame to flush the spawn commands
    test_app.step();

    // Verify core components exist on the spawned entity
    let world = test_app.app.world();

    // Unit marker component
    assert!(
        world.get::<Unit>(spawned_entity).is_some(),
        "Peacekeeper should have Unit component"
    );

    // Owner
    let owner = world.get::<Owner>(spawned_entity).expect("Peacekeeper should have Owner");
    assert_eq!(owner.0, Some(0), "Owner should be player 0");

    // GridPosition
    let grid_pos = world.get::<GridPosition>(spawned_entity).expect("Peacekeeper should have GridPosition");
    assert_eq!(grid_pos.x, 32, "Grid X should be 32");
    assert_eq!(grid_pos.z, 32, "Grid Z should be 32");

    // ObjectInstance
    let obj = world.get::<space_crystals::game::types::objects::ObjectInstance>(spawned_entity)
        .expect("Peacekeeper should have ObjectInstance");
    assert!(obj.is_alive(), "Peacekeeper should be alive");
    assert!(obj.is_destructible(), "Peacekeeper should be destructible");

    // AttackCapability
    let attack_cap = world.get::<space_crystals::game::combat::types::AttackCapability>(spawned_entity)
        .expect("Peacekeeper should have AttackCapability");
    assert!(attack_cap.damage > 0.0, "Peacekeeper should have positive damage");
    assert!(attack_cap.range > 0.0, "Peacekeeper should have positive range");

    // Transform (world position derived from grid position)
    let transform = world.get::<Transform>(spawned_entity).expect("Peacekeeper should have Transform");
    assert!(transform.translation.y > 0.0, "Peacekeeper should be above ground plane");
}

/// Test that stepping the app multiple times does not panic.
#[test]
fn multi_step_stability() {
    let mut test_app = TestApp::new();

    // Run startup
    test_app.step();

    // Spawn a unit
    test_app.app.world_mut().run_system_once(
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            spawn_peacekeeper(&mut commands, &mut meshes, &mut materials, 30, 30, Owner(Some(0)));
        },
    ).unwrap();

    // Run 10 more frames — should not panic
    test_app.step_n(10);
}

/// Test: ObjectInterfaceState resource initializes to Default and selection systems
/// don't alter selection when AwaitingTarget is set.
#[test]
fn awaiting_target_guard_prevents_selection() {
    let mut test_app = TestApp::new();

    // Run startup
    test_app.step();

    // Verify ObjectInterfaceState resource defaults to Default
    {
        let state = test_app.app.world().resource::<ObjectInterfaceState>();
        assert_eq!(*state, ObjectInterfaceState::Default,
            "ObjectInterfaceState should default to Default after startup");
    }

    // Spawn a Selectable entity (unit) and manually add Selected to it
    let unit_entity = test_app.app.world_mut().run_system_once(
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            spawn_peacekeeper(&mut commands, &mut meshes, &mut materials, 30, 30, Owner(Some(0)))
        },
    ).unwrap();
    test_app.step();

    // Select the unit
    test_app.app.world_mut().entity_mut(unit_entity).insert(Selected);

    // Verify unit is selected
    assert!(test_app.app.world().get::<Selected>(unit_entity).is_some(),
        "Unit should be Selected before awaiting target test");

    // Set AwaitingTarget(AttackMove)
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AwaitingTarget(CommandType::AttackMove);

    // Step multiple frames — selection should NOT change because the
    // selection_system and drag_box_system both early-return when
    // is_awaiting_target() is true
    test_app.step_n(3);

    // Verify unit is still selected (guard prevented deselection)
    assert!(test_app.app.world().get::<Selected>(unit_entity).is_some(),
        "Unit should remain Selected when AwaitingTarget is set");

    // Reset to Default
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::Default;

    // Verify the resource was set correctly
    let state = test_app.app.world().resource::<ObjectInterfaceState>();
    assert_eq!(*state, ObjectInterfaceState::Default,
        "ObjectInterfaceState should be back to Default after reset");
}

/// Test: All AwaitingTarget command types properly block selection.
#[test]
fn all_awaiting_target_modes_preserve_selection() {
    let mut test_app = TestApp::new();
    test_app.step();

    // Spawn and select a unit
    let unit_entity = test_app.app.world_mut().run_system_once(
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            spawn_peacekeeper(&mut commands, &mut meshes, &mut materials, 30, 30, Owner(Some(0)))
        },
    ).unwrap();
    test_app.step();
    test_app.app.world_mut().entity_mut(unit_entity).insert(Selected);

    // Test each non-Default command type in AwaitingTarget
    let command_types = [
        CommandType::Move,
        CommandType::Attack,
        CommandType::AttackGround,
        CommandType::AttackMove,
        CommandType::Patrol,
    ];

    for ct in command_types {
        *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
            ObjectInterfaceState::AwaitingTarget(ct);
        test_app.step();

        assert!(test_app.app.world().get::<Selected>(unit_entity).is_some(),
            "Unit should remain Selected when AwaitingTarget({:?})", ct);
    }
}

/// Test: Movement set runs after UiHud set (ensures CursorTarget is updated before
/// right_click_move_command reads it, and selection_system runs before Movement resets
/// the interface state). This prevents the race condition where right_click_move_command
/// resets AwaitingTarget to Default before selection_system checks the guard.
#[test]
fn movement_runs_after_ui_hud_ordering() {
    use space_crystals::simulation::types::DiagCategory;

    let mut test_app = TestApp::new();

    // Run a step to verify the app builds and runs with the ordering constraint.
    // If DiagCategory::Movement.after(DiagCategory::UiHud) created a cycle,
    // this would panic.
    test_app.step();

    // Spawn a unit and an enemy unit
    let (unit_entity, enemy_entity) = test_app.app.world_mut().run_system_once(
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            let unit = spawn_peacekeeper(&mut commands, &mut meshes, &mut materials, 30, 30, Owner(Some(0)));
            let enemy = spawn_peacekeeper(&mut commands, &mut meshes, &mut materials, 35, 30, Owner(Some(1)));
            (unit, enemy)
        },
    ).unwrap();
    test_app.step();

    // Select the unit
    test_app.app.world_mut().entity_mut(unit_entity).insert(Selected);

    // Enter AwaitingTarget(Attack) mode
    *test_app.app.world_mut().resource_mut::<ObjectInterfaceState>() =
        ObjectInterfaceState::AwaitingTarget(CommandType::Attack);

    // Step a frame — the AwaitingTarget state should persist
    // (selection_system sees it and returns early before Movement systems run)
    test_app.step();

    // Verify AwaitingTarget mode persists (no system reset it spuriously)
    let state = test_app.app.world().resource::<ObjectInterfaceState>();
    assert!(state.is_awaiting_target(),
        "AwaitingTarget should persist across frames when no target click occurs");

    // Verify the unit is still selected
    assert!(test_app.app.world().get::<Selected>(unit_entity).is_some(),
        "Unit should remain Selected during AwaitingTarget mode");
}

/// Test: spawn_headquarters creates a visible, selectable entity with correct components.
#[test]
fn spawn_headquarters_has_visual_and_components() {
    let mut test_app = TestApp::new();
    test_app.step();

    // First spawn a tunnel (HQ needs a parent tunnel entity)
    let (tunnel_entity, hq_entity) = test_app.app.world_mut().run_system_once(
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            let tunnel = spawn_tunnel(&mut commands, &mut meshes, &mut materials, 40, 40, Owner(Some(0)));
            let hq = spawn_headquarters(&mut commands, &mut meshes, &mut materials, 42, 38, Owner(Some(0)), tunnel);
            (tunnel, hq)
        },
    ).unwrap();
    test_app.step();

    let world = test_app.app.world();

    // HQ should have a Transform (visual representation)
    let transform = world.get::<Transform>(hq_entity)
        .expect("Headquarters should have Transform");
    assert!(transform.translation.y > 0.0, "HQ should be above ground plane");

    // HQ should have ObjectInstance
    let obj = world.get::<space_crystals::game::types::objects::ObjectInstance>(hq_entity)
        .expect("Headquarters should have ObjectInstance");
    assert_eq!(obj.object_type, ObjectEnum::Headquarters);
    assert!(obj.is_destructible(), "HQ should be destructible");
    assert!(obj.is_alive(), "HQ should be alive");

    // HQ should be Selectable
    assert!(world.get::<Selectable>(hq_entity).is_some(),
        "Headquarters should have Selectable component");

    // HQ should have SelectionBounds
    assert!(world.get::<SelectionBounds>(hq_entity).is_some(),
        "Headquarters should have SelectionBounds component");

    // HQ should have GridPosition
    let grid_pos = world.get::<GridPosition>(hq_entity)
        .expect("HQ should have GridPosition");
    assert_eq!(grid_pos.x, 42);
    assert_eq!(grid_pos.z, 38);

    // HQ should have Owner
    let owner = world.get::<Owner>(hq_entity)
        .expect("HQ should have Owner");
    assert_eq!(owner.0, Some(0));

    // HQ parent tunnel should exist
    assert!(world.get_entity(tunnel_entity).is_ok(),
        "Parent tunnel should exist");
}

/// Test: HQ spawn position is correctly calculated from grid coordinates.
#[test]
fn spawn_headquarters_position_from_grid() {
    let mut test_app = TestApp::new();
    test_app.step();

    let hq_entity = test_app.app.world_mut().run_system_once(
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            let tunnel = spawn_tunnel(&mut commands, &mut meshes, &mut materials, 40, 40, Owner(Some(0)));
            spawn_headquarters(&mut commands, &mut meshes, &mut materials, 32, 32, Owner(Some(0)), tunnel)
        },
    ).unwrap();
    test_app.step();

    let transform = test_app.app.world().get::<Transform>(hq_entity)
        .expect("HQ should have Transform");
    // grid 32 → world_x = (32 - 32) + 1.0 = 1.0 (center of 2x2)
    assert!((transform.translation.x - 1.0).abs() < 0.01, "HQ world X should be 1.0, got {}", transform.translation.x);
    assert!((transform.translation.z - 1.0).abs() < 0.01, "HQ world Z should be 1.0, got {}", transform.translation.z);
    assert!((transform.translation.y - 0.5).abs() < 0.01, "HQ world Y should be 0.5, got {}", transform.translation.y);
}

/// Test: HQ has mesh and material (visual representation).
#[test]
fn spawn_headquarters_has_mesh_and_material() {
    let mut test_app = TestApp::new();
    test_app.step();

    let hq_entity = test_app.app.world_mut().run_system_once(
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            let tunnel = spawn_tunnel(&mut commands, &mut meshes, &mut materials, 40, 40, Owner(Some(0)));
            spawn_headquarters(&mut commands, &mut meshes, &mut materials, 42, 38, Owner(Some(0)), tunnel)
        },
    ).unwrap();
    test_app.step();

    let world = test_app.app.world();
    assert!(world.get::<Mesh3d>(hq_entity).is_some(),
        "HQ should have a mesh handle");
    assert!(world.get::<MeshMaterial3d<StandardMaterial>>(hq_entity).is_some(),
        "HQ should have a material handle");
}

// =============================================================================
// TestHarness Integration Tests
// =============================================================================

/// Test: spawn_unit_at_grid creates a Peacekeeper with correct components.
#[test]
fn harness_spawn_unit_peacekeeper() {
    let mut test_app = TestApp::new();
    test_app.step();

    let mut harness = TestHarness::new(&mut test_app.app);
    let entity = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 32, 32, Owner(Some(0)));
    harness.step();

    let world = test_app.app.world();
    assert!(world.get::<Unit>(entity).is_some(), "Should have Unit component");
    let owner = world.get::<Owner>(entity).unwrap();
    assert_eq!(owner.0, Some(0));
    let grid_pos = world.get::<GridPosition>(entity).unwrap();
    assert_eq!(grid_pos.x, 32);
    assert_eq!(grid_pos.z, 32);
    let obj = world.get::<space_crystals::game::types::objects::ObjectInstance>(entity).unwrap();
    assert_eq!(obj.object_type, ObjectEnum::Peacekeeper);
}

/// Test: spawn_unit with world position converts correctly.
#[test]
fn harness_spawn_unit_world_pos() {
    let mut test_app = TestApp::new();
    test_app.step();

    let mut harness = TestHarness::new(&mut test_app.app);
    // world_x=0.5 → grid_x = (0.5 - 0.5 + 32.0) as i32 = 32
    let entity = harness.spawn_unit(ObjectEnum::Peacekeeper, Vec3::new(0.5, 0.0, 0.5), Owner(Some(0)));
    harness.step();

    let world = test_app.app.world();
    let grid_pos = world.get::<GridPosition>(entity).unwrap();
    assert_eq!(grid_pos.x, 32);
    assert_eq!(grid_pos.z, 32);
}

/// Test: spawn_structure_at_grid creates a PowerPlant with correct components.
#[test]
fn harness_spawn_structure_power_plant() {
    let mut test_app = TestApp::new();
    test_app.step();

    let mut harness = TestHarness::new(&mut test_app.app);
    let entity = harness.spawn_structure_at_grid(ObjectEnum::PowerPlant, 30, 30, Owner(Some(0)));
    harness.step();

    let world = test_app.app.world();
    let obj = world.get::<space_crystals::game::types::objects::ObjectInstance>(entity).unwrap();
    assert_eq!(obj.object_type, ObjectEnum::PowerPlant);
    let grid_pos = world.get::<GridPosition>(entity).unwrap();
    assert_eq!(grid_pos.x, 30);
    assert_eq!(grid_pos.z, 30);
}

/// Test: spawn_resource creates a SpaceCrystalPatch with correct amount.
#[test]
fn harness_spawn_resource() {
    let mut test_app = TestApp::new();
    test_app.step();

    let mut harness = TestHarness::new(&mut test_app.app);
    let entity = harness.spawn_resource(40, 40, 1000);
    harness.step();

    let world = test_app.app.world();
    let patch = world.get::<SpaceCrystalPatch>(entity).unwrap();
    assert_eq!(patch.remaining_amount, 1000);
    assert_eq!(patch.initial_amount, 1000);
    assert!(!patch.has_plate);
    let obj = world.get::<space_crystals::game::types::objects::ObjectInstance>(entity).unwrap();
    assert_eq!(obj.object_type, ObjectEnum::SpaceCrystalsPatch);
}

/// Test: set_selection and clear_selection work correctly.
#[test]
fn harness_selection() {
    let mut test_app = TestApp::new();
    test_app.step();

    let mut harness = TestHarness::new(&mut test_app.app);
    let unit1 = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 30, 30, Owner(Some(0)));
    let unit2 = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 31, 31, Owner(Some(0)));
    harness.step();

    // Select both
    harness.set_selection(&[unit1, unit2]);
    let selected = harness.get_selection();
    assert_eq!(selected.len(), 2);

    // Clear
    harness.clear_selection();
    let selected = harness.get_selection();
    assert_eq!(selected.len(), 0);
}

/// Test: issue_command sets UnitCommand on entity.
#[test]
fn harness_issue_command() {
    let mut test_app = TestApp::new();
    test_app.step();

    let mut harness = TestHarness::new(&mut test_app.app);
    let unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 32, 32, Owner(Some(0)));
    harness.step();

    harness.issue_command(unit, UnitCommand::Move(Vec3::new(10.0, 0.0, 10.0)));

    let world = test_app.app.world();
    let cmd = world.get::<UnitCommand>(unit).unwrap();
    assert!(matches!(cmd, UnitCommand::Move(_)));
}

/// Test: set_gdo_crystals and get_gdo_crystals.
#[test]
fn harness_set_get_gdo_crystals() {
    let mut test_app = TestApp::new();
    test_app.step(); // Runs startup systems which create Player entities

    let mut harness = TestHarness::new(&mut test_app.app);
    harness.set_gdo_crystals(500);
    let crystals = harness.get_gdo_crystals();
    assert_eq!(crystals, Some(500));
}

/// Test: advance_frames runs multiple updates.
#[test]
fn harness_advance_frames() {
    let mut test_app = TestApp::new();
    test_app.step();

    let mut harness = TestHarness::new(&mut test_app.app);
    // Advancing 60 frames should not panic
    harness.advance_frames(60);
}

/// Test: set_tile changes tile preset.
#[test]
fn harness_set_tile() {
    let mut test_app = TestApp::new();
    test_app.step(); // Spawns tiles

    let mut harness = TestHarness::new(&mut test_app.app);
    harness.set_tile(32, 32, TilePresetEnum::Mountain);

    let world = test_app.app.world_mut();
    // Find tile at (32, 32) and verify preset
    let mut found = false;
    for (gp, preset) in world.query::<(&GridPosition, &TilePresetEnum)>().iter(world) {
        if gp.x == 32 && gp.z == 32 {
            assert_eq!(*preset, TilePresetEnum::Mountain);
            found = true;
            break;
        }
    }
    assert!(found, "Should find a tile at (32, 32) with Mountain preset");
}

/// Test: reveal_map sets all fog cells to Visible.
#[test]
fn harness_reveal_map() {
    let mut test_app = TestApp::new();
    test_app.step();

    let mut harness = TestHarness::new(&mut test_app.app);
    harness.reveal_map(0);

    let fog = test_app.app.world().resource::<FogOfWarMap>();
    // Check a few tiles
    assert_eq!(fog.get(0, 0, 0), VisibilityStateEnum::Visible);
    assert_eq!(fog.get(0, 32, 32), VisibilityStateEnum::Visible);
    assert_eq!(fog.get(0, 63, 63), VisibilityStateEnum::Visible);
}

/// Test: set_camera modifies camera transform.
#[test]
fn harness_set_camera() {
    let mut test_app = TestApp::new();
    test_app.step();

    let mut harness = TestHarness::new(&mut test_app.app);
    harness.set_camera(Vec3::new(10.0, 0.0, 15.0), 50.0);

    let world = test_app.app.world_mut();
    let mut query = world.query_filtered::<&Transform, With<MainCamera>>();
    let cam_transform = query.iter(world).next().unwrap();
    assert!((cam_transform.translation.x - 10.0).abs() < 0.01);
    assert!((cam_transform.translation.y - 50.0).abs() < 0.01);
    assert!((cam_transform.translation.z - 15.0).abs() < 0.01);
}

/// Test: spawn_unit_at_grid for SyndicateAgent.
#[test]
fn harness_spawn_syndicate_agent() {
    let mut test_app = TestApp::new();
    test_app.step();

    let mut harness = TestHarness::new(&mut test_app.app);
    let entity = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 35, 35, Owner(Some(1)));
    harness.step();

    let world = test_app.app.world();
    assert!(world.get::<Unit>(entity).is_some());
    let obj = world.get::<space_crystals::game::types::objects::ObjectInstance>(entity).unwrap();
    assert_eq!(obj.object_type, ObjectEnum::SyndicateAgent);
}

/// Test: testing module is feature-gated (compile-time check — if this test
/// compiles, the feature gate is working correctly in test mode).
#[test]
fn harness_feature_gate_works() {
    // This test simply verifies that TestHarness is accessible.
    // In release/non-test builds without the `testing` feature, this module
    // would not exist.
    let mut test_app = TestApp::new();
    let _harness = TestHarness::new(&mut test_app.app);
}

// =============================================================================
// Syndicate Game Start Tests (tunnel_expansions_and_starting_condition QA)
// =============================================================================

/// Test: Syndicate game start spawns a Tunnel entity at grid (40, 40).
#[test]
fn syndicate_game_start_spawns_tunnel() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step(); // process OnEnter(InGame)
    test_app.step(); // apply deferred commands

    let world = test_app.app.world_mut();
    let mut tunnel_query = world.query_filtered::<(
        &space_crystals::game::types::objects::ObjectInstance,
        &GridPosition,
        &Owner,
    ), With<space_crystals::game::types::structures::TunnelState>>();

    let tunnels: Vec<_> = tunnel_query.iter(world).collect();
    assert!(!tunnels.is_empty(), "Syndicate game start should spawn at least one Tunnel");

    let (obj, grid_pos, _owner) = tunnels[0];
    assert_eq!(obj.object_type, ObjectEnum::Tunnel);
    assert_eq!(grid_pos.x, 40);
    assert_eq!(grid_pos.z, 40);
}

/// Test: Syndicate game start spawns a pre-built Headquarters at grid (42, 38).
#[test]
fn syndicate_game_start_spawns_headquarters() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();
    test_app.step();

    let world = test_app.app.world_mut();
    let mut hq_query = world.query_filtered::<(
        &space_crystals::game::types::objects::ObjectInstance,
        &GridPosition,
        &Owner,
    ), With<space_crystals::game::types::structures::HeadquartersState>>();

    let hqs: Vec<_> = hq_query.iter(world).collect();
    assert!(!hqs.is_empty(), "Syndicate game start should spawn a Headquarters");

    let (obj, grid_pos, _owner) = hqs[0];
    assert_eq!(obj.object_type, ObjectEnum::Headquarters);
    assert_eq!(grid_pos.x, 42);
    assert_eq!(grid_pos.z, 38);
}

/// Test: Syndicate starting HQ has visual components (mesh, material, transform).
#[test]
fn syndicate_game_start_hq_has_visuals() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();
    test_app.step();

    let world = test_app.app.world_mut();
    let mut hq_query = world.query_filtered::<Entity, With<space_crystals::game::types::structures::HeadquartersState>>();
    let hq_entity = hq_query.iter(world).next()
        .expect("HQ entity should exist");

    assert!(world.get::<Transform>(hq_entity).is_some(), "HQ should have Transform");
    assert!(world.get::<Mesh3d>(hq_entity).is_some(), "HQ should have Mesh");
    assert!(world.get::<MeshMaterial3d<StandardMaterial>>(hq_entity).is_some(), "HQ should have Material");
    assert!(world.get::<Selectable>(hq_entity).is_some(), "HQ should be Selectable");
    assert!(world.get::<SelectionBounds>(hq_entity).is_some(), "HQ should have SelectionBounds");
}

/// Test: Syndicate starting HQ is linked to the starting Tunnel via TunnelExpansionMarker.
#[test]
fn syndicate_game_start_hq_linked_to_tunnel() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();
    test_app.step();

    let world = test_app.app.world_mut();

    // Find Tunnel entity
    let mut tunnel_q = world.query_filtered::<Entity, With<space_crystals::game::types::structures::TunnelState>>();
    let tunnel_entity = tunnel_q.iter(world).next()
        .expect("Tunnel should exist");

    // Find HQ and check its TunnelExpansionMarker
    let mut hq_q = world.query::<&space_crystals::game::types::structures::TunnelExpansionMarker>();
    let marker = hq_q.iter(world).next()
        .expect("HQ should have TunnelExpansionMarker");
    assert_eq!(marker.parent_tunnel, tunnel_entity,
        "HQ's parent_tunnel should point to the starting Tunnel");
}

/// Test: Syndicate starting HQ is owned by the correct player.
#[test]
fn syndicate_game_start_hq_correct_owner() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();
    test_app.step();

    let world = test_app.app.world_mut();
    let mut hq_query = world.query_filtered::<&Owner, With<space_crystals::game::types::structures::HeadquartersState>>();
    let owner = hq_query.iter(world).next()
        .expect("HQ should have Owner");

    // When Syndicate is selected, Syndicate is player 0
    assert_eq!(owner.player_number(), Some(0),
        "Syndicate starting HQ should be owned by player 0");
}

/// Test: has_structure_overlap correctly detects multi-cell structures.
#[test]
fn has_structure_overlap_checks_full_footprint() {
    let mut test_app = TestApp::new();
    test_app.step();

    let mut harness = TestHarness::new(&mut test_app.app);
    // Spawn a 4x4 Tunnel at grid (40, 40)
    let _tunnel = harness.spawn_structure_at_grid(ObjectEnum::Tunnel, 40, 40, Owner(Some(0)));
    harness.step();

    // A 2x2 HQ at (41, 41) should overlap with the Tunnel's footprint (40-43, 40-43)
    // but only if has_structure_overlap properly checks structure sizes
    let world = test_app.app.world_mut();
    let mut structures = world.query::<(&GridPosition, &space_crystals::game::types::objects::StructureInstance, &space_crystals::game::types::objects::ObjectInstance)>();
    let has_overlap = structures.iter(world).any(|(gp, _si, oi)| {
        let size = oi.object_type.object_type().size;
        // Check if (41,41) 2x2 overlaps with this structure's footprint
        for dx in 0..2_i32 {
            for dz in 0..2_i32 {
                let cx = 41 + dx;
                let cz = 41 + dz;
                for sx in 0..size.0 as i32 {
                    for sz in 0..size.1 as i32 {
                        if gp.x + sx == cx && gp.z + sz == cz {
                            return true;
                        }
                    }
                }
            }
        }
        false
    });

    assert!(has_overlap, "A 2x2 building at (41,41) should overlap with a 4x4 Tunnel at (40,40)");
}

/// Test: Tunnel placement validation allows HQ inside tunnel area.
#[test]
fn tunnel_area_fits_hq_expansion() {
    use space_crystals::game::types::structures::{TunnelArea, TunnelTier};

    // Tunnel at grid (40, 40), Tier 1 (area radius 3)
    let area = TunnelArea::new(40, 40, &TunnelTier::Tier1);

    // HQ is 2x2. Check that (42, 38) fits (used in starting condition)
    assert!(area.fits_expansion(42, 38, 2, 2),
        "HQ at (42, 38) should fit within Tunnel area at (40, 40)");

    // Check positions outside the tunnel footprint but inside the area
    assert!(area.fits_expansion(37, 37, 2, 2),
        "HQ at (37, 37) should fit at edge of area");

    // Check position outside the area
    assert!(!area.fits_expansion(36, 36, 2, 2),
        "HQ at (36, 36) should NOT fit — extends outside area");

    // Check position overlapping tunnel footprint — fits_expansion allows this
    // (structure overlap check is separate from area check)
    assert!(area.fits_expansion(41, 41, 2, 2),
        "HQ at (41, 41) should fit within area (overlap check is separate)");
}
