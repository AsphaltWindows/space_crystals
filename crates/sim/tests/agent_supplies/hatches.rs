//! The Supplies hatch (`factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD`, design change 0248: Side D;
//! `factions/syndicate/units/agent.md > Gathering > Supplies`: "Drop-off side: Tunnel Side D"): a delivery there
//! and one at the crystal hatch on Side B in the same ticks, each within reach of its own hatch's square (design
//! change 0292 point 1: "8 su reaches for pick-up at a station and for drop-off at the Supplies hatch"); the nearest
//! Tunnel for Supplies, by its crystal hatch, and the delivery at its Supplies hatch (0292 point 2); and a Drop Off
//! order with a Supply.

use super::*;

/// The plan's "Both sides at once": one Agent mines a patch 2 grid units east of the crystal hatch, another gathers
/// Supplies at a station 3 grid units west of the Supplies hatch, and their unloading overlaps: each side has a line
/// of its own. Each unloads within its DropOffReach of its own hatch's square, and each bank gets its own deliveries.
#[test]
fn a_crystal_delivery_at_b_and_a_supply_delivery_at_d_in_the_same_ticks() {
    // Tunnel 1 and station 2 as in `field`; a patch (3) on tile (18, 8); the miner (4) and the carrier (5).
    let mut t = flat(30, 20)
        .spawn(TUNNEL, Some(P1), tile(12, 6))
        .spawn_with(station_of(100), tile(7, 7))
        .spawn_with(patch_of(100_000), tile(18, 8))
        .spawn(AGENT, Some(P1), centre(17, 11))
        .spawn(AGENT, Some(P1), centre(10, 11))
        .start();
    let (miner, carrier) = (4, 5);
    gather(&mut t, 1, &[miner], 3);
    gather(&mut t, 1, &[carrier], STATION);
    let before = banks(&t, P1);
    let crystal_hatch = hatch(&t, T1, Resource::Crystals);
    let supplies_hatch = hatch(&t, T1, Resource::Supplies);
    let reach = |t: &TestSim, n: u32, square: &Shape| Shape::of(object(t, n)).gap(square) <= REACH;
    let mut together = 0;
    for _ in 0..1500 {
        step(&mut t, &[miner, carrier], &[STATION]);
        let (mining, carrying) = (is_unloading(&t, miner), is_unloading(&t, carrier));
        assert!(!mining || reach(&t, miner, &crystal_hatch), "tick {}", t.sim().tick());
        assert!(
            !carrying || reach(&t, carrier, &supplies_hatch),
            "tick {}",
            t.sim().tick()
        );
        if mining && carrying {
            together += 1;
        }
    }
    assert!(together > 0, "never in the same ticks");
    let delivered = deliveries(&t);
    let of = |n: u32| -> Vec<(u32, u32, Resource)> {
        (delivered.iter())
            .filter(|d| d.1 == n)
            .map(|d| (d.2, d.3, d.4))
            .collect()
    };
    let (mined, carried) = (of(miner), of(carrier));
    assert!(
        mined.len() >= 3 && mined.iter().all(|&d| d == (T1, 50, Resource::Crystals)),
        "{mined:?}"
    );
    assert!(
        carried.len() >= 5 && carried.iter().all(|&d| d == (T1, CARRY, Resource::Supplies)),
        "{carried:?}"
    );
    assert_eq!(
        banks(&t, P1),
        (before.0 + 50 * mined.len() as u32, before.1 + carried.len() as u32)
    );
}

/// Design change 0292 point 2 (`factions/syndicate/units/agent.md > Gathering > Supplies`): "'nearest Tunnel' for
/// Supplies is measured to the crystal hatch, the same measure as for crystals", "even though Supplies are delivered at
/// Side D". The two measures pick different Tunnels here: the west Tunnel (1) stands 1 grid unit from the station (2),
/// its crystal hatch facing it and its Supplies hatch on its far side; the east one (3) stands 4 grid units off, its
/// Supplies hatch facing the station. The Agent takes the west one, nearer by its crystal hatch though the east one's
/// Supplies hatch is nearer, keeps it for the Gather, and walks round it to unload at its Supplies hatch.
#[test]
fn the_nearest_tunnel_for_supplies_is_the_one_with_the_nearest_crystal_hatch() {
    let (west, east) = (1, 3);
    let mut t = flat(30, 20)
        .spawn(TUNNEL, Some(P1), tile(2, 6))
        .spawn_with(station_of(100), tile(7, 7))
        .spawn(TUNNEL, Some(P1), tile(13, 6))
        .spawn(AGENT, Some(P1), centre(8, 12))
        .start();
    gather(&mut t, 1, &[4], STATION);
    run_to(&mut t, &[4], &[STATION], 200, "a Supply", |t| load(t, 4).0 > 0);
    let at = t.pos(4);
    let distance = |n: u32, resource: Resource| {
        let point = drop_point_of(object(&t, n), resource).expect("a drop-off");
        point.centre.distance(at)
    };
    assert!(distance(west, Resource::Crystals) < distance(east, Resource::Crystals));
    assert!(distance(east, Resource::Supplies) < distance(west, Resource::Supplies));
    step(&mut t, &[4], &[STATION]);
    assert_eq!(node(&t, 4), Some(west), "chosen as its first return starts");
    let square = hatch(&t, west, Resource::Supplies);
    let delivered = run_to(&mut t, &[4], &[STATION], 600, "the delivery", |t| {
        if is_unloading(t, 4) {
            assert!(Shape::of(object(t, 4)).gap(&square) <= REACH, "at its Supplies hatch");
        }
        !deliveries(t).is_empty()
    });
    assert_eq!(deliveries(&t), [(delivered, 4, west, CARRY, Resource::Supplies)]);
    assert_eq!(
        object(&t, 4).worker.unwrap().drop_off,
        Some(id(west)),
        "kept for the Gather"
    );
}

/// Design change 0292 point 2 on the removal path: "a lost Tunnel means picking the nearest again"
/// (`factions/syndicate/units/agent.md > Gathering rules`), by the crystal hatch too. The layout of
/// [`the_nearest_tunnel_for_supplies_is_the_one_with_the_nearest_crystal_hatch`], with a third Tunnel (4) south of the
/// station, turned a quarter back so that its crystal hatch faces the station and its Supplies hatch looks away. The
/// Agent (5) carries a Supply back to the west Tunnel, which is removed: it takes the south one at once, nearer than
/// the east one by its crystal hatch though the east one's Supplies hatch is nearer, and unloads at its Supplies hatch.
#[test]
fn a_supply_carrier_that_loses_its_tunnel_takes_the_nearest_crystal_hatch() {
    use sc_sim::testing::TestSim;
    use sc_sim::{BuildingOrientation, DebugCommand, Rotation};
    let (west, east, south, agent) = (1, 3, 4, 5);
    let mut scenario = flat(30, 20)
        .spawn(TUNNEL, Some(P1), tile(2, 6))
        .spawn_with(station_of(100), tile(7, 7))
        .spawn(TUNNEL, Some(P1), tile(13, 6))
        .spawn(TUNNEL, Some(P1), tile(6, 12))
        .spawn(AGENT, Some(P1), centre(10, 3))
        .build();
    // Turned 270°: Side B to the north, on tile (8, 12); Side D to the south, on tile (8, 15).
    scenario.map.spawns[3].orientation = BuildingOrientation {
        rotation: Rotation::R270,
        ..BuildingOrientation::default()
    };
    let mut t = TestSim::new(scenario);
    gather(&mut t, 1, &[agent], STATION);
    run_to(&mut t, &[agent], &[STATION], 200, "a Supply", |t| load(t, agent).0 > 0);
    step(&mut t, &[agent], &[STATION]);
    assert_eq!(node(&t, agent), Some(west), "the nearest crystal hatch of the three");
    let at = t.pos(agent);
    let distance = |n: u32, resource: Resource| {
        let point = drop_point_of(object(&t, n), resource).expect("a drop-off");
        point.centre.distance(at)
    };
    assert!(distance(south, Resource::Crystals) < distance(east, Resource::Crystals));
    assert!(distance(east, Resource::Supplies) < distance(south, Resource::Supplies));
    let now = t.sim().tick();
    t.debug_at(now + 1, DebugCommand::Remove { id: id(west) });
    step(&mut t, &[agent], &[STATION]);
    assert_eq!(object(&t, agent).worker.unwrap().drop_off, Some(id(south)), "at once");
    let square = hatch(&t, south, Resource::Supplies);
    let delivered = run_to(&mut t, &[agent], &[STATION], 600, "the delivery", |t| {
        if is_unloading(t, agent) {
            assert!(
                Shape::of(object(t, agent)).gap(&square) <= REACH,
                "at its Supplies hatch"
            );
        }
        !deliveries(t).is_empty()
    });
    assert_eq!(deliveries(&t), [(delivered, agent, south, CARRY, Resource::Supplies)]);
}

/// `factions/syndicate/units/agent.md > Gathering rules` (design change 0255, one of the Drone's job rules): "Other
/// orders keep the load", and a Drop Off order "delivers to the Tunnel clicked, once": an Agent stopped with a Supply
/// keeps it, and a Drop Off on its Tunnel delivers it at the Supplies hatch.
#[test]
fn a_drop_off_order_delivers_a_supply_at_the_supplies_hatch() {
    let mut t = field(1, 3, station_of(10)).start();
    gather(&mut t, 1, &[3], STATION);
    run_to(&mut t, &[3], &[STATION], 200, "a Supply", |t| load(t, 3).0 > 0);
    let next = t.sim().tick() + 1;
    t.order_at(next, P1, &[id(3)], OrderKind::Stop, false);
    t.run(40);
    assert_eq!(load(&t, 3), (CARRY, Resource::Supplies), "a Stop keeps the load");
    assert!(deliveries(&t).is_empty());
    let before = banks(&t, P1);
    let next = t.sim().tick() + 1;
    t.order_at(next, P1, &[id(3)], OrderKind::DropOff(id(T1)), false);
    let square = hatch(&t, T1, Resource::Supplies);
    let delivered = run_to(&mut t, &[3], &[STATION], 300, "the delivery", |t| {
        if is_unloading(t, 3) {
            assert!(Shape::of(object(t, 3)).gap(&square) <= REACH, "at the Supplies hatch");
        }
        !deliveries(t).is_empty()
    });
    assert_eq!(deliveries(&t), [(delivered, 3, T1, CARRY, Resource::Supplies)]);
    assert_eq!(banks(&t, P1), (before.0, before.1 + 1));
    assert_eq!(load(&t, 3).0, 0);
    t.run(5);
    assert_eq!(order(&t, 3), None, "once");
}
