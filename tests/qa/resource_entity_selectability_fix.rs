use crate::helpers::*;
use bevy::ecs::system::RunSystemOnce;

/// Helper: Run the selection_group_sync logic inline.
/// The actual system is in a private module, so we replicate it here.
fn sync_selection_groups(app: &mut bevy::app::App) {
    app.world_mut().run_system_once(
        |selected_query: Query<(Entity, &ObjectInstance), With<Selected>>,
         mut selection: ResMut<Selection>| {
            let entities: Vec<(Entity, ObjectEnum, bool)> = selected_query
                .iter()
                .map(|(entity, obj_instance)| {
                    let object_type = obj_instance.object_type;
                    let groupable = object_type.object_type().groupable;
                    (entity, object_type, groupable)
                })
                .collect();
            selection.build_from_entities(&entities);
        },
    ).unwrap();
}

/// QA Step 1 [auto]: Spawn a SpaceCrystalsPatch. Click on it.
/// Verify Selection.groups is non-empty and contains ObjectEnum::SpaceCrystalsPatch.
#[test]
fn step_1_crystal_patch_selectable() {
    let mut test_app = TestApp::new();
    test_app.step();

    let patch = {
        let mut harness = TestHarness::new(&mut test_app.app);
        let patch = harness.spawn_resource(20, 20, 500);
        harness.set_selection(&[patch]);
        patch
    };

    // Run the selection group sync
    sync_selection_groups(&mut test_app.app);

    let selection = test_app.app.world().resource::<Selection>();
    assert!(!selection.groups.is_empty(),
        "Selection.groups should be non-empty after selecting a crystal patch");
    assert!(selection.contains_entity(patch),
        "Selection should contain the crystal patch entity");

    let has_crystal = selection.groups.iter()
        .any(|g| g.object_type == ObjectEnum::SpaceCrystalsPatch);
    assert!(has_crystal,
        "Selection.groups should contain a SpaceCrystalsPatch group");
}

/// QA Step 2 [auto]: Spawn a SupplyDeliveryStation. Click on it.
/// Verify Selection.groups is non-empty and contains ObjectEnum::SupplyDeliveryStation.
#[test]
fn step_2_supply_delivery_station_selectable() {
    let mut test_app = TestApp::new();
    test_app.step();

    // Spawn SDS manually since harness doesn't have a spawn_sds method
    let sds = test_app.app.world_mut().run_system_once(
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            let mesh = meshes.add(Cuboid::new(1.5, 0.8, 1.5));
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.6, 0.2),
                ..default()
            });
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(0.0, 0.4, 0.0),
                SupplyDeliveryStation {
                    delivery_size: 100,
                    delivery_interval: 60.0,
                    current_supplies: 50,
                    time_until_next_delivery: 30.0,
                },
                GridPosition { x: 30, z: 30 },
                ObjectInstance::indestructible(ObjectEnum::SupplyDeliveryStation),
                Selectable,
            )).id()
        },
    ).unwrap();

    // Select the SDS
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        harness.set_selection(&[sds]);
    }

    sync_selection_groups(&mut test_app.app);

    let selection = test_app.app.world().resource::<Selection>();
    assert!(!selection.groups.is_empty(),
        "Selection.groups should be non-empty after selecting an SDS");
    assert!(selection.contains_entity(sds),
        "Selection should contain the SDS entity");

    let has_sds = selection.groups.iter()
        .any(|g| g.object_type == ObjectEnum::SupplyDeliveryStation);
    assert!(has_sds,
        "Selection.groups should contain a SupplyDeliveryStation group");
}

/// QA Step 5 [auto]: Click on an empty tile (no entity).
/// Verify Selection.groups is empty and no phantom command panel appears.
#[test]
fn step_5_empty_tile_selection_empty() {
    let mut test_app = TestApp::new();
    test_app.step();

    {
        let mut harness = TestHarness::new(&mut test_app.app);
        // Clear selection (simulates clicking empty tile)
        harness.clear_selection();
    }

    sync_selection_groups(&mut test_app.app);

    let selection = test_app.app.world().resource::<Selection>();
    assert!(selection.groups.is_empty(),
        "Selection.groups should be empty after clicking empty tile");
    assert_eq!(selection.total_entity_count(), 0,
        "No entities should be selected");
}

/// QA Step 6 [auto]: Select a unit, then select a Crystal Patch.
/// Verify the selection switches cleanly — previous unit deselected, patch selected.
#[test]
fn step_6_selection_switches_unit_to_patch() {
    let mut test_app = TestApp::new();
    test_app.step();

    let (unit, patch) = {
        let mut harness = TestHarness::new(&mut test_app.app);
        let unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
        let patch = harness.spawn_resource(15, 15, 300);

        // First select the unit
        harness.set_selection(&[unit]);
        (unit, patch)
    };

    sync_selection_groups(&mut test_app.app);

    // Verify unit is selected
    {
        let selection = test_app.app.world().resource::<Selection>();
        assert!(selection.contains_entity(unit), "Unit should be selected initially");
    }

    // Now switch to the crystal patch
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        harness.set_selection(&[patch]);
    }

    sync_selection_groups(&mut test_app.app);

    let selection = test_app.app.world().resource::<Selection>();
    assert!(!selection.contains_entity(unit),
        "Unit should be deselected after switching to patch");
    assert!(selection.contains_entity(patch),
        "Crystal patch should be selected");
    assert_eq!(selection.groups.len(), 1,
        "Should have exactly one selection group");

    let has_crystal = selection.groups.iter()
        .any(|g| g.object_type == ObjectEnum::SpaceCrystalsPatch);
    assert!(has_crystal,
        "Selection group should be SpaceCrystalsPatch");
}

/// QA Step 8 [auto]: Verify phantom command panel regression does not recur:
/// click empty ground, assert Selection.groups is empty.
#[test]
fn step_8_no_phantom_command_panel_regression() {
    let mut test_app = TestApp::new();
    test_app.step();

    // First have something selected
    let unit = {
        let mut harness = TestHarness::new(&mut test_app.app);
        let unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        harness.set_selection(&[unit]);
        unit
    };

    sync_selection_groups(&mut test_app.app);

    // Verify unit is selected
    {
        let selection = test_app.app.world().resource::<Selection>();
        assert!(selection.contains_entity(unit));
    }

    // Now click empty ground (clear selection)
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        harness.clear_selection();
    }

    sync_selection_groups(&mut test_app.app);

    let selection = test_app.app.world().resource::<Selection>();
    assert!(selection.groups.is_empty(),
        "Selection.groups must be empty after clicking empty ground — no phantom panel");
    assert_eq!(selection.total_entity_count(), 0,
        "No entities should remain selected");
}
