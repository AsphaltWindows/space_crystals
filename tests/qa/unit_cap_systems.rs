use crate::helpers::*;
use space_crystals::game::types::{GdoPlayerResources, SyndicatePlayerResources, Player};
use space_crystals::game::types::factions::{GDO_UNIT_CONTROL_CAP, SYNDICATE_MAX_TUNNEL_SPACE};

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
