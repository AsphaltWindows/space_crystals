use crate::helpers::*;
use bevy::ecs::system::RunSystemOnce;

/// Helper: run fog of war system directly (FixedUpdate doesn't fire in headless TestApp).
fn run_fog_system(app: &mut bevy::app::App) {
    app.world_mut().run_system_once(space_crystals::game::world::map::update_fog_of_war).unwrap();
}

/// QA Step 1 [auto]: Start a new game. Do not spawn or move any units.
/// QA Step 2 [auto]: Observe the fog of war reveal area around the deployment center.
#[test]
fn step_1_2_fog_revealed_around_deployment_center() {
    let mut test_app = TestApp::new();
    test_app.step(); // OnEnter(InGame) fires, spawns grid + DC
    test_app.step(); // Apply deferred commands
    run_fog_system(&mut test_app.app);

    let harness = TestHarness::new(&mut test_app.app);

    // GDO deployment center spawns at grid (30, 30), 4x4 structure.
    // Vision center = (30 + 4/2, 30 + 4/2) = (32, 32). Sight range = 6.
    let center_vis = harness.get_visibility(0, 32, 32);
    assert_eq!(
        center_vis,
        VisibilityStateEnum::Visible,
        "Deployment center's center tile (32,32) should be visible"
    );
}

/// QA Step 3 [auto]: Count visible tiles in each cardinal direction.
/// QA Step 4 [auto]: Verify equal counts in all four cardinal directions.
#[test]
fn step_3_4_cardinal_direction_symmetry() {
    let mut test_app = TestApp::new();
    test_app.step_n(2);
    run_fog_system(&mut test_app.app);

    let harness = TestHarness::new(&mut test_app.app);
    let cx = 32;
    let cz = 32;

    let mut north_count = 0;
    let mut south_count = 0;
    let mut east_count = 0;
    let mut west_count = 0;

    for d in 1..=10 {
        if harness.get_visibility(0, cx, cz - d) == VisibilityStateEnum::Visible { north_count += 1; }
        if harness.get_visibility(0, cx, cz + d) == VisibilityStateEnum::Visible { south_count += 1; }
        if harness.get_visibility(0, cx + d, cz) == VisibilityStateEnum::Visible { east_count += 1; }
        if harness.get_visibility(0, cx - d, cz) == VisibilityStateEnum::Visible { west_count += 1; }
    }

    assert!(north_count > 0, "Should have visible tiles north of center");
    assert_eq!(north_count, south_count,
        "North ({}) and South ({}) should match", north_count, south_count);
    assert_eq!(east_count, west_count,
        "East ({}) and West ({}) should match", east_count, west_count);
    assert_eq!(north_count, east_count,
        "All cardinal: N={}, S={}, E={}, W={}", north_count, south_count, east_count, west_count);
}

/// QA Step 5 [auto]: Verify symmetric reveal in diagonal directions.
#[test]
fn step_5_diagonal_symmetry() {
    let mut test_app = TestApp::new();
    test_app.step_n(2);
    run_fog_system(&mut test_app.app);

    let harness = TestHarness::new(&mut test_app.app);
    let cx = 32;
    let cz = 32;

    let mut ne = 0;
    let mut nw = 0;
    let mut se = 0;
    let mut sw = 0;

    for d in 1..=10 {
        if harness.get_visibility(0, cx + d, cz - d) == VisibilityStateEnum::Visible { ne += 1; }
        if harness.get_visibility(0, cx - d, cz - d) == VisibilityStateEnum::Visible { nw += 1; }
        if harness.get_visibility(0, cx + d, cz + d) == VisibilityStateEnum::Visible { se += 1; }
        if harness.get_visibility(0, cx - d, cz + d) == VisibilityStateEnum::Visible { sw += 1; }
    }

    assert!(ne > 0, "Should have visible tiles in NE diagonal");
    assert_eq!(ne, nw, "NE ({}) and NW ({}) should match", ne, nw);
    assert_eq!(se, sw, "SE ({}) and SW ({}) should match", se, sw);
    assert_eq!(ne, se, "All diagonals: NE={}, NW={}, SE={}, SW={}", ne, nw, se, sw);
}

/// QA Step 6 [auto]: Repeat with a second player's deployment center.
#[test]
fn step_6_second_player_symmetry() {
    let mut test_app = TestApp::new();
    test_app.step_n(2);

    // Spawn a second deployment center for player 1.
    // Use grid (10, 10) to avoid interference from enemy Peacekeepers
    // spawned around (50, 50) during GDO game start.
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        harness.spawn_structure_at_grid(ObjectEnum::DeploymentCenter, 10, 10, Owner(Some(1)));
    }
    test_app.step(); // Apply spawn
    run_fog_system(&mut test_app.app);

    let harness = TestHarness::new(&mut test_app.app);

    // 4x4 at grid (10,10) → vision center (12, 12)
    let cx = 12;
    let cz = 12;

    assert_eq!(
        harness.get_visibility(1, cx, cz),
        VisibilityStateEnum::Visible,
        "Player 1's DC center (12,12) should be visible"
    );

    // Cardinal symmetry for player 1
    let mut n = 0;
    let mut s = 0;
    let mut e = 0;
    let mut w = 0;

    for d in 1..=10 {
        if harness.get_visibility(1, cx, cz - d) == VisibilityStateEnum::Visible { n += 1; }
        if harness.get_visibility(1, cx, cz + d) == VisibilityStateEnum::Visible { s += 1; }
        if harness.get_visibility(1, cx + d, cz) == VisibilityStateEnum::Visible { e += 1; }
        if harness.get_visibility(1, cx - d, cz) == VisibilityStateEnum::Visible { w += 1; }
    }

    assert!(n > 0, "Player 1 should have visible tiles north");
    assert_eq!(n, s, "Player 1: N ({}) and S ({}) should match", n, s);
    assert_eq!(e, w, "Player 1: E ({}) and W ({}) should match", e, w);
}
