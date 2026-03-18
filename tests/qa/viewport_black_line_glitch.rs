use crate::helpers::*;

/// QA Step 1 [auto]: Launch the game at the default window size.
/// Verify the app starts up without panic and the game world is populated.
#[test]
fn step_1_launch_game_default_window() {
    let mut test_app = TestApp::new();
    test_app.step();

    // Verify the game world initialized
    let world = test_app.app.world();

    // Should have a populated grid map
    let grid_map = world.get_resource::<GridMap>().expect("GridMap resource should exist");
    assert!(grid_map.width > 0, "GridMap should have positive width");
    assert!(grid_map.height > 0, "GridMap should have positive height");
}
