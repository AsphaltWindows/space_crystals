//! The Supplies hatch (`factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD`, design change 0248: Side D;
//! `factions/syndicate/units/agent.md > Gathering > Supplies`: "Drop-off side: Tunnel Side D"): a delivery there
//! and one at the crystal hatch on Side B in the same ticks, each within reach of its own hatch's square (Placeholder:
//! Q0082 point 1, 8 su); the nearest Tunnel for Supplies, by its Supplies hatch (Placeholder: Q0082 point 2); and a
//! Drop Off order with a Supply.

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

/// Q0082 point 2: the nearest Tunnel for a Supply is the one whose Supplies hatch is nearest, as the crystal hatch
/// decides for crystals (`factions/syndicate/units/agent.md > Gathering rules`: "the nearest own operational Tunnel
/// (straight line to its crystal hatch)"). The west Tunnel (1) stands 1 grid unit from the station (2), its crystal
/// hatch facing it and its Supplies hatch on its far side; the east one (3) stands 4 grid units off, its Supplies hatch
/// facing the station. The Agent delivers at the east one, though the west one is nearer by its centre and by its
/// crystal hatch.
#[test]
fn the_nearest_tunnel_for_supplies_is_the_one_with_the_nearest_supplies_hatch() {
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
    let centre_distance = |n: u32| object(&t, n).pos.distance(at);
    assert!(centre_distance(west) < centre_distance(east));
    assert!(distance(west, Resource::Crystals) < distance(east, Resource::Crystals));
    assert!(distance(east, Resource::Supplies) < distance(west, Resource::Supplies));
    let delivered = run_to(&mut t, &[4], &[STATION], 400, "the delivery", |t| {
        !deliveries(t).is_empty()
    });
    assert_eq!(deliveries(&t), [(delivered, 4, east, CARRY, Resource::Supplies)]);
    assert_eq!(node(&t, 4), Some(east), "kept for the Gather");
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
