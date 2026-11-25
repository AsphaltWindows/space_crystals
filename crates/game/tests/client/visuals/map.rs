//! The map: objects drawn on its terrain and every pick on the ground drawn, the grid overlay toggled by its
//! action, and another scenario loaded, which rebuilds the map and the visuals.

use super::*;

/// Objects stand on the terrain (drawn lifted by its height), and every pick resolves to the ground drawn under
/// the cursor.
#[test]
fn objects_and_picking_follow_the_terrain_height() {
    let mut app = headless_app(plateau_scenario());
    updates(&mut app, 2);
    let patch = app.world().resource::<SimRes>().object(ObjectId(1)).unwrap().pos;
    let entity = app.world().resource::<ObjectVisuals>().by_id[&ObjectId(1)];
    let translation = app.world().get::<Transform>(entity).unwrap().translation;
    assert_eq!(translation.truncate(), Vec2::new(patch.x, -(patch.y - lift(3.0))));

    // Point the camera at the patch: the cursor at the viewport's center picks the patch's ground point.
    let map = app.world().resource::<SimRes>().map().clone();
    app.world_mut().resource_mut::<CameraView>().center = drawn_position(&map, patch);
    app.update();
    assert_eq!(camera_center(&app), drawn_position(&map, patch), "not clamped");
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    let (position, viewport) = viewport_rect(size, insets);
    let middle = (position.as_vec2() + viewport.as_vec2() / 2.0) / size.scale_factor;
    let view = *app.world().resource::<CameraView>();
    let ground = cursor_to_ground(middle, size, insets, view, 1.0, &map).unwrap();
    assert!((ground - patch).length() < 0.01, "{ground} should be {patch}");
}

#[test]
fn the_grid_toggles_with_its_action() {
    let mut app = headless_app(small_scenario());
    app.update();
    let grid_visibility = |app: &mut App| {
        let mut query = app.world_mut().query_filtered::<&Visibility, With<GridOverlay>>();
        *query.single(app.world()).unwrap()
    };
    assert_eq!(grid_visibility(&mut app), Visibility::Hidden);
    press(&mut app, InputAction::ToggleGrid);
    app.update();
    assert_eq!(grid_visibility(&mut app), Visibility::Inherited);
    // A press fires once.
    updates(&mut app, 2);
    assert_eq!(grid_visibility(&mut app), Visibility::Inherited);
    press(&mut app, InputAction::ToggleGrid);
    app.update();
    assert_eq!(grid_visibility(&mut app), Visibility::Hidden);
}

#[test]
fn loading_another_scenario_rebuilds_the_map_and_the_visuals() {
    let mut app = headless_app(small_scenario());
    updates(&mut app, 3);
    assert_eq!(app.world().resource::<MapView>().builds, 1);

    let mut other = scenarios::empty(20, 10);
    other.map.spawns.push(patch_at(1, 1));
    load_scenario(app.world_mut(), other);
    app.update();

    assert_eq!(app.world().resource::<MapView>().builds, 2);
    let handle = app.world().resource::<MapRenderAssets>().terrain_mesh.clone();
    let vertices = app
        .world()
        .resource::<Assets<Mesh>>()
        .get(&handle)
        .unwrap()
        .count_vertices();
    assert_eq!(vertices, 20 * 10 * 4);
    // Object 1 of the new sim is a different object than object 1 of the old one.
    assert_eq!(visual_ids(&mut app), BTreeSet::from([ObjectId(1)]));
    assert_eq!(
        app.world().resource::<RecordedInputs>().ticks.len(),
        tick(&app) as usize
    );
}
