use crate::helpers::*;
use space_crystals::game::types::{GdoPlayerResources, SyndicatePlayerResources, Player, BarracksState};
use space_crystals::game::types::factions::{GDO_UNIT_CONTROL_CAP, SYNDICATE_MAX_TUNNEL_SPACE};
use space_crystals::game::world::faction::barracks_production_tick_system;
use bevy::ecs::system::RunSystemOnce;

/// QA Step 1 [auto]: GDO: Start with 0 units. Build units until UnitControl reaches 200.
/// Attempt to build one more unit — verify it is blocked. Lose a unit, verify you can build again.
#[test]
fn step_1_gdo_unit_control_cap() {
    let mut test_app = TestApp::new();
    test_app.step(); // startup

    // Check initial state
    {
        let world = test_app.app.world_mut();
        let mut query = world.query::<(&Player, &GdoPlayerResources)>();
        let (_, res) = query.iter(world).next().expect("GDO player should exist");
        assert_eq!(res.unit_control_cap, GDO_UNIT_CONTROL_CAP, "GDO cap should be 200");
    }

    // Directly set unit_control_used to 199 and verify we can build 1 more
    {
        let world = test_app.app.world_mut();
        let mut query = world.query::<(&Player, &mut GdoPlayerResources)>();
        for (_, mut res) in query.iter_mut(world) {
            res.unit_control_used = 199;
            assert!(res.has_unit_control(1), "Should be able to build 1 unit at 199/200");
            assert!(!res.has_unit_control(2), "Should NOT be able to build 2 units at 199/200");
        }
    }

    // Set to 200 and verify blocked
    {
        let world = test_app.app.world_mut();
        let mut query = world.query::<(&Player, &mut GdoPlayerResources)>();
        for (_, mut res) in query.iter_mut(world) {
            res.unit_control_used = 200;
            assert!(!res.has_unit_control(1), "Should NOT be able to build at 200/200");
        }
    }

    // Decrement by 1 (simulating unit death) and verify we can build again
    {
        let world = test_app.app.world_mut();
        let mut query = world.query::<(&Player, &mut GdoPlayerResources)>();
        for (_, mut res) in query.iter_mut(world) {
            res.unit_control_used = res.unit_control_used.saturating_sub(1);
            assert!(res.has_unit_control(1), "Should be able to build after losing a unit (199/200)");
        }
    }
}

/// QA Step 2 [auto]: Syndicate: Verify tunnel space checking works.
#[test]
fn step_2_syndicate_tunnel_space_cap() {
    // Default SyndicatePlayerResources has tunnel_space_provided = 20
    let mut res = SyndicatePlayerResources::default();

    // Can build within cap
    assert!(res.has_tunnel_space(1), "Should have space for 1 unit");
    assert!(res.has_tunnel_space(20), "Should have space for 20 units");
    assert!(!res.has_tunnel_space(21), "Should NOT have space for 21 units");

    // After using some space
    res.tunnel_space_used = 18;
    assert!(res.has_tunnel_space(2), "Should have space for 2 more at 18/20");
    assert!(!res.has_tunnel_space(3), "Should NOT have space for 3 more at 18/20");

    // At zero provided
    res.tunnel_space_provided = 0;
    res.tunnel_space_used = 0;
    assert!(!res.has_tunnel_space(1), "Should NOT have space at 0 provided");
}

/// QA Step 5 [auto]: For all factions, verify the "used / available" values update correctly
/// as units are built, destroyed, or cap-providing structures change.
#[test]
fn step_5_gdo_death_decrements_cap() {
    let mut test_app = TestApp::new();
    test_app.step(); // startup — spawns grid, DC, player resources

    // Spawn a peacekeeper owned by player 0 (GDO)
    let pk;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        pk = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    // Manually increment unit_control_used (simulating production system)
    {
        let world = test_app.app.world_mut();
        let mut query = world.query::<(&Player, &mut GdoPlayerResources)>();
        for (_, mut res) in query.iter_mut(world) {
            res.unit_control_used = 10;
        }
    }

    // Kill the peacekeeper
    {
        let world = test_app.app.world_mut();
        let mut obj = world.get_mut::<ObjectInstance>(pk).unwrap();
        obj.apply_damage(1000.0);
    }

    // Step to run remove_dead_entities_system which should decrement cap
    test_app.step_n(3);

    // Verify unit_control_used decreased
    {
        let world = test_app.app.world_mut();
        let mut query = world.query::<(&Player, &GdoPlayerResources)>();
        for (player, res) in query.iter(world) {
            if player.player_number == 0 {
                assert!(res.unit_control_used < 10,
                    "Unit death should decrement unit_control_used, got {}", res.unit_control_used);
            }
        }
    }
}

/// Verify GDO_UNIT_CONTROL_CAP is 200 and SYNDICATE_MAX_TUNNEL_SPACE is 200
#[test]
fn cap_constants_correct() {
    assert_eq!(GDO_UNIT_CONTROL_CAP, 200, "GDO cap should be 200");
    assert_eq!(SYNDICATE_MAX_TUNNEL_SPACE, 200, "Syndicate max tunnel space should be 200");
}

/// Verify that cost-0 units (like SupplyChopper) are always allowed when at cap.
/// has_unit_control(0) at exactly cap returns true (200 + 0 <= 200).
#[test]
fn cost_zero_units_allowed_at_cap() {
    let mut res = GdoPlayerResources::default();

    // At exactly cap — cost-0 should still be allowed
    res.unit_control_used = GDO_UNIT_CONTROL_CAP;
    assert!(res.has_unit_control(0), "Cost-0 units should be allowed at exactly cap (200/200)");

    // Below cap — cost-0 obviously allowed
    res.unit_control_used = 100;
    assert!(res.has_unit_control(0), "Cost-0 units should be allowed below cap (100/200)");

    // Verify SupplyChopper actually has cost 0
    assert_eq!(ObjectEnum::SupplyChopper.unit_control_cost(), 0,
        "SupplyChopper should have unit_control_cost of 0");

    // Verify all structures have cost 0
    assert_eq!(ObjectEnum::DeploymentCenter.unit_control_cost(), 0,
        "DeploymentCenter should have unit_control_cost of 0");
    assert_eq!(ObjectEnum::PowerPlant.unit_control_cost(), 0,
        "PowerPlant should have unit_control_cost of 0");
    assert_eq!(ObjectEnum::Barracks.unit_control_cost(), 0,
        "Barracks should have unit_control_cost of 0");
}

/// Verify barracks_production_tick_system increments unit_control_used when a unit finishes.
/// Sets up a Barracks with a Peacekeeper near completion, runs the system directly, and checks
/// that unit_control_used increased by the Peacekeeper's control cost.
#[test]
fn barracks_production_increments_unit_control() {
    let mut test_app = TestApp::new();
    test_app.step(); // startup — spawns grid, DC, player resources

    // Spawn a barracks owned by player 0 (GDO)
    let barracks_entity;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        barracks_entity = harness.spawn_structure_at_grid(ObjectEnum::Barracks, 15, 15, Owner(Some(0)));
    }
    test_app.step();

    // Set barracks to have a Peacekeeper almost complete (1 frame from done)
    // and set unit_control_used to a known value
    let initial_used = 50u32;
    {
        let world = test_app.app.world_mut();

        // Configure barracks production — near completion
        let mut bk_state = world.get_mut::<BarracksState>(barracks_entity).unwrap();
        let cost = BarracksState::production_cost(&ObjectEnum::Peacekeeper).unwrap();
        bk_state.current_build = Some(ObjectEnum::Peacekeeper);
        bk_state.current_build_progress = Some(cost.build_frames as f32 - 0.5);

        // Set known unit_control_used
        let mut query = world.query::<(&Player, &mut GdoPlayerResources)>();
        for (_, mut res) in query.iter_mut(world) {
            res.unit_control_used = initial_used;
        }
    }

    // Run barracks_production_tick_system directly (FixedUpdate doesn't fire in headless TestApp)
    test_app.app.world_mut().run_system_once(barracks_production_tick_system).unwrap();
    // Flush deferred commands (entity spawning)
    test_app.step();

    // Verify unit_control_used was incremented by Peacekeeper's control cost
    {
        let world = test_app.app.world_mut();
        let mut query = world.query::<(&Player, &GdoPlayerResources)>();
        let (_, res) = query.iter(world).next().expect("GDO player should exist");
        let expected = initial_used + ObjectEnum::Peacekeeper.unit_control_cost();
        assert_eq!(res.unit_control_used, expected,
            "Barracks production should increment unit_control_used from {} to {}, got {}",
            initial_used, expected, res.unit_control_used);
    }

    // Verify the barracks build state was cleared (production complete)
    {
        let world = test_app.app.world_mut();
        let bk_state = world.get::<BarracksState>(barracks_entity).unwrap();
        assert!(bk_state.current_build.is_none(),
            "Barracks should have no current build after completion");
        assert!(bk_state.current_build_progress.is_none(),
            "Barracks build progress should be cleared after completion");
    }
}

/// Verify that barracks_production_tick_system does NOT check unit control cap at spawn time.
/// A unit queued when under cap but completing when at cap should still spawn (current design).
/// This documents the current behavior — cap is only checked at queue time.
#[test]
fn barracks_production_spawns_even_at_cap() {
    let mut test_app = TestApp::new();
    test_app.step(); // startup

    // Spawn a barracks
    let barracks_entity;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        barracks_entity = harness.spawn_structure_at_grid(ObjectEnum::Barracks, 15, 15, Owner(Some(0)));
    }
    test_app.step();

    // Set barracks near completion AND set unit_control_used to cap
    {
        let world = test_app.app.world_mut();

        let mut bk_state = world.get_mut::<BarracksState>(barracks_entity).unwrap();
        let cost = BarracksState::production_cost(&ObjectEnum::Peacekeeper).unwrap();
        bk_state.current_build = Some(ObjectEnum::Peacekeeper);
        bk_state.current_build_progress = Some(cost.build_frames as f32 - 0.5);

        // Set to exactly at cap — no room for more units
        let mut query = world.query::<(&Player, &mut GdoPlayerResources)>();
        for (_, mut res) in query.iter_mut(world) {
            res.unit_control_used = GDO_UNIT_CONTROL_CAP;
        }
    }

    // Run barracks_production_tick_system directly (FixedUpdate doesn't fire in headless TestApp)
    test_app.app.world_mut().run_system_once(barracks_production_tick_system).unwrap();
    // Flush deferred commands (entity spawning)
    test_app.step();

    // Verify unit_control_used now exceeds cap (unit was spawned and incremented)
    {
        let world = test_app.app.world_mut();
        let mut query = world.query::<(&Player, &GdoPlayerResources)>();
        let (_, res) = query.iter(world).next().expect("GDO player should exist");
        assert_eq!(res.unit_control_used, GDO_UNIT_CONTROL_CAP + ObjectEnum::Peacekeeper.unit_control_cost(),
            "Barracks should spawn even at cap (no spawn-time check), pushing used above cap");
    }

    // Verify barracks finished the build
    {
        let world = test_app.app.world_mut();
        let bk_state = world.get::<BarracksState>(barracks_entity).unwrap();
        assert!(bk_state.current_build.is_none(),
            "Barracks should have completed the build even at cap");
    }
}

/// Verify that queue-time check blocks adding units when at cap.
/// BarracksState::try_queue succeeds mechanically, but the UI/command_panel checks
/// has_unit_control before calling try_queue. This test verifies has_unit_control
/// correctly blocks at the boundary.
#[test]
fn queue_time_cap_check_blocks_at_boundary() {
    let mut res = GdoPlayerResources::default();
    let pk_cost = ObjectEnum::Peacekeeper.unit_control_cost();

    // Just under cap — queueing should be allowed
    res.unit_control_used = GDO_UNIT_CONTROL_CAP - pk_cost;
    assert!(res.has_unit_control(pk_cost),
        "Should allow queue at {}/{}", res.unit_control_used, res.unit_control_cap);

    // Exactly at cap — queueing should be blocked
    res.unit_control_used = GDO_UNIT_CONTROL_CAP;
    assert!(!res.has_unit_control(pk_cost),
        "Should block queue at {}/{}", res.unit_control_used, res.unit_control_cap);

    // One over cap — should still block
    res.unit_control_used = GDO_UNIT_CONTROL_CAP + 1;
    assert!(!res.has_unit_control(pk_cost),
        "Should block queue at {}/{}", res.unit_control_used, res.unit_control_cap);
}
