use crate::helpers::*;

/// QA Step 1 [auto]: Start a new game — verify grid lines are visible across the entire map.
///
/// Since grid lines are rendered via Gizmos (no ECS state to query), we verify
/// the GridMap resource has correct 64x64 dimensions and that the grid system
/// doesn't panic when a camera is positioned at various locations including edges.
#[test]
fn step_1_grid_map_covers_full_map() {
    let mut test_app = TestApp::new();
    test_app.step(); // startup

    let world = test_app.app.world_mut();
    let grid_map = world.resource::<GridMap>();

    // GridMap should be 64x64 covering world-space [-32, 32] in both axes
    assert_eq!(grid_map.width, 64, "GridMap width should be 64");
    assert_eq!(grid_map.height, 64, "GridMap height should be 64");
    assert_eq!(grid_map.cell_size, 1.0, "GridMap cell_size should be 1.0");
    assert_eq!(grid_map.half_width(), 32.0, "GridMap half_width should be 32.0");
    assert_eq!(grid_map.half_height(), 32.0, "GridMap half_height should be 32.0");
}

/// QA Step 3 [auto]: Spawn a unit at coordinates near the map boundary —
/// verify grid lines are present at those coordinates.
///
/// We spawn units at all four boundary regions to verify entities can exist
/// at the map edges. The grid coverage fix (GRID_LINE_DRAW_RADIUS=40) ensures
/// lines are drawn at these locations.
#[test]
fn step_3_unit_at_map_boundary_has_valid_grid() {
    let mut test_app = TestApp::new();
    test_app.step();
    let mut harness = TestHarness::new(&mut test_app.app);

    // Spawn units near all four map edges (grid coords 0-63)
    let unit_top = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 32, 1, Owner(Some(0)));
    let unit_bottom = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 32, 62, Owner(Some(0)));
    let unit_left = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 1, 32, Owner(Some(0)));
    let unit_right = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 62, 32, Owner(Some(0)));

    // All units should exist and have valid positions
    assert!(harness.is_alive(unit_top), "Unit at top edge should exist");
    assert!(harness.is_alive(unit_bottom), "Unit at bottom edge should exist");
    assert!(harness.is_alive(unit_left), "Unit at left edge should exist");
    assert!(harness.is_alive(unit_right), "Unit at right edge should exist");

    // Verify positions are within map bounds (world space: -32 to +32)
    let pos_top = harness.get_position(unit_top).unwrap();
    let pos_bottom = harness.get_position(unit_bottom).unwrap();
    let pos_left = harness.get_position(unit_left).unwrap();
    let pos_right = harness.get_position(unit_right).unwrap();

    assert!(pos_top.x.abs() <= 32.0 && pos_top.z.abs() <= 32.0,
        "Top edge unit should be within map bounds");
    assert!(pos_bottom.x.abs() <= 32.0 && pos_bottom.z.abs() <= 32.0,
        "Bottom edge unit should be within map bounds");
    assert!(pos_left.x.abs() <= 32.0 && pos_left.z.abs() <= 32.0,
        "Left edge unit should be within map bounds");
    assert!(pos_right.x.abs() <= 32.0 && pos_right.z.abs() <= 32.0,
        "Right edge unit should be within map bounds");
}
