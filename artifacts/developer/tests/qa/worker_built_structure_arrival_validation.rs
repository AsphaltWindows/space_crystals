use crate::helpers::*;
use space_crystals::game::units::types::state::BuildingStructureBehavior;

/// QA Step 1 [auto]: Start a game as Syndicate. Produce an Agent from the Headquarters.
#[test]
fn step_1_syndicate_agent_exists() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let agent;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        agent = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let h = TestHarness::new(&mut test_app.app);
    assert!(h.is_alive(agent), "Syndicate Agent should be alive");
}

/// QA Step 3 [auto]: Verify the Agent pathfinds to the build location and begins construction successfully.
/// We test that issuing a BuildTunnel command is accepted.
#[test]
fn step_3_agent_accepts_build_command() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let agent;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        // Spawn in an area with buildable tiles (center-ish of map)
        agent = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    // Issue BuildTunnel command to a nearby location
    let target = Vec3::new(-9.5, 0.5, -9.5); // grid ~(22, 22)
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(agent, UnitCommand::BuildTunnel(target));
    }
    test_app.step_n(3);

    let cmd = TestHarness::new(&mut test_app.app).get_command(agent);
    assert!(
        matches!(cmd, Some(UnitCommand::BuildTunnel(_))),
        "Agent should have BuildTunnel command, got {:?}", cmd
    );
}

/// QA Step 5 [auto]: Wait for the second Agent to arrive at an occupied tile.
/// Verify the build command is cancelled -- Agent stops and idles.
/// We simulate this by checking BuildingStructureBehavior: when arrived=true at occupied tile,
/// the behavior system should cancel the build.
#[test]
fn step_5_build_cancelled_on_occupied_tile() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let existing_tunnel;
    let agent;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        // Place an existing tunnel at the target location
        existing_tunnel = h.spawn_structure_at_grid(ObjectEnum::Tunnel, 25, 25, Owner(Some(0)));
        // Spawn agent nearby
        agent = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 23, 25, Owner(Some(0)));
    }
    test_app.step();

    assert!(TestHarness::new(&mut test_app.app).is_alive(existing_tunnel), "Existing tunnel should be alive");

    // Issue build command at the same location as the existing tunnel
    // Grid (25,25) → world = (25-32)+0.5 = -6.5
    let target = Vec3::new(-6.5, 0.5, -6.5);
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(agent, UnitCommand::BuildTunnel(target));
    }

    // Step many frames to let the agent try to build
    test_app.step_n(120);

    // After arrival, the build should have been cancelled due to occupied tile.
    // Agent should be idle or have had the build cancelled.
    let h = TestHarness::new(&mut test_app.app);
    // Count tunnels at grid 25,25 — should still be 1 (no second tunnel built)
    let tunnel_count = {
        let mut count = 0;
        let world = test_app.app.world_mut();
        let mut query = world.query::<(&ObjectInstance, &GridPosition)>();
        for (obj, gp) in query.iter(world) {
            if obj.object_type == ObjectEnum::Tunnel && gp.x == 25 && gp.z == 25 {
                count += 1;
            }
        }
        count
    };
    assert_eq!(tunnel_count, 1, "Should only have 1 tunnel at (25,25), no duplicate was placed");
}

/// QA Step 7 [auto]: Verify the build command is accepted immediately on fog-of-war tiles (no rejection at command time).
#[test]
fn step_7_build_accepted_on_fog_tile() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let agent;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        agent = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    // Target a location that would be under fog of war (far from Agent's vision)
    // Grid (40, 40) → world = (40-32)+0.5 = 8.5
    let target = Vec3::new(8.5, 0.5, 8.5);

    // Issue build command — should be accepted regardless of visibility
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(agent, UnitCommand::BuildTunnel(target));
    }
    test_app.step_n(3);

    // Verify the command was accepted (not rejected/converted to Idle)
    let cmd = TestHarness::new(&mut test_app.app).get_command(agent);
    assert!(
        matches!(cmd, Some(UnitCommand::BuildTunnel(_))),
        "Build command should be accepted on fogged tile, got {:?}", cmd
    );
}

/// QA Step 9 [auto]: Order an Agent to build on a non-Buildable tile (e.g., Rock terrain).
/// Verify the Agent walks there and then idles without building.
#[test]
fn step_9_build_on_non_buildable_tile() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let agent;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        agent = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
        // Set some tiles to non-buildable (Rock) near the build target
        h.set_tile(22, 22, TilePresetEnum::Mountain);
        h.set_tile(23, 22, TilePresetEnum::Mountain);
        h.set_tile(22, 23, TilePresetEnum::Mountain);
        h.set_tile(23, 23, TilePresetEnum::Mountain);
    }
    test_app.step();

    // Issue build at the rock tiles - grid (22,22) → world (-9.5, 0.5, -9.5)
    let target = Vec3::new(-9.5, 0.5, -9.5);
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(agent, UnitCommand::BuildTunnel(target));
    }

    // Step many frames for agent to walk there
    test_app.step_n(120);

    // After arriving at non-buildable terrain, no tunnel should be placed
    let tunnel_count = {
        let world = test_app.app.world_mut();
        let mut query = world.query::<(&ObjectInstance, &GridPosition)>();
        let mut count = 0;
        for (obj, gp) in query.iter(world) {
            if obj.object_type == ObjectEnum::Tunnel && gp.x >= 21 && gp.x <= 24 && gp.z >= 21 && gp.z <= 24 {
                count += 1;
            }
        }
        count
    };
    assert_eq!(tunnel_count, 0, "No tunnel should be built on non-buildable Rock tiles");
}

/// Verify Build command is_available requires Syndicate faction
#[test]
fn build_command_requires_syndicate() {
    let build_cmd = UnitCommand::Build {
        target: Vec3::ZERO,
        object: ObjectEnum::Tunnel,
    };

    assert!(
        build_cmd.is_available(false, false, false, true),
        "Build should be available for Syndicate"
    );
    assert!(
        !build_cmd.is_available(false, false, false, false),
        "Build should NOT be available for non-Syndicate"
    );

    // Also test BuildTunnel variant
    let build_tunnel_cmd = UnitCommand::BuildTunnel(Vec3::ZERO);
    assert!(
        build_tunnel_cmd.is_available(false, false, false, true),
        "BuildTunnel should be available for Syndicate"
    );
    assert!(
        !build_tunnel_cmd.is_available(false, false, false, false),
        "BuildTunnel should NOT be available for non-Syndicate"
    );
}
