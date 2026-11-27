//! The Bike's way out of the network and back in (`factions/syndicate/tunnels.md > Tunnel Network`: "Tier 2+: Vehicles
//! (Wheeled, Tracked, Drill, Hover Vehicle, Mech)"; `factions/syndicate/tunnel_menus.md > EjectMenu`, design changes
//! 0250 and 0251; plan decision T100): only a Tier 2+ Tunnel lets it out, at the exit path's spot just outside Side A,
//! facing out, its box turned with it, whichever way the Tunnel is turned; and it walks back in through the entry path.
//! The 72 × 48 box of slice M6-ST1's Transport is the sim's own test (`sim/network/tests.rs`), since only a test can
//! give a ground vehicle that box.

use sc_sim::{MoveStatus, Rotation, math};

use super::*;

/// [`assembly`]'s game with its Tunnel turned by `rotation`, about its centre: Side A faces north unturned, then, a
/// quarter turn clockwise at a time, east, south and west.
fn turned(rotation: Rotation) -> TestSim {
    let mut scenario = plant().spawn(ASSEMBLY, Some(P1), SpawnAt::Tile(VA_AT)).build();
    scenario.map.spawns[0].orientation = BuildingOrientation {
        rotation,
        flip_h: false,
        flip_v: false,
    };
    let mut t = TestSim::new(scenario);
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    t.debug_at(1, DebugCommand::SetTier { id: T1, tier: 2 });
    run_checked(&mut t, 1);
    t
}

/// Where a Bike stands once out of `tunnel`'s exit path: on its spot, facing out, its box against the wall.
fn bike_out(t: &TestSim, tunnel: ObjectId) -> Vec2 {
    let paths = paths(t, tunnel);
    paths.exit + paths.outward * (BIKE_LENGTH / 2.0 + 1.0 / 64.0)
}

/// Whether `unit`'s shape overlaps that of any other unit on the map.
fn overlaps_a_unit(t: &TestSim, unit: ObjectId) -> bool {
    let own = shape(t, unit);
    (t.sim().objects()).any(|o| o.id != unit && o.mover.is_some() && Shape::of(o).overlaps(&own))
}

/// Design change 0250 ("Each unit appears at the exit path's spot just outside Side A") for a vehicle, in the four
/// rotations of the Tunnel (plan decision T100: "out of a Tier 2+ Tunnel's Side A exit path"): a Bike ejected comes out
/// facing out of Side A, its 56 × 32 Silhouette turned with it (slice M5-V1), so 56 su deep out of the side and 32 su
/// along it, its box against the wall, a sixty-fourth of a su clear of the footprint, on the exit path's spot. It then
/// drives 1 grid unit straight out (`factions/syndicate/tunnel_menus.md > EjectMenu`: "with no rally point, a unit
/// walks about 1 grid unit straight out of Side A and idles"), never turning.
#[test]
fn a_bike_comes_out_of_side_a_facing_out_whichever_way_the_tunnel_faces() {
    let mut ways = Vec::new();
    for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
        let mut t = turned(rotation);
        let [bike] = bikes_inside(&mut t, 1)[..] else {
            panic!("one Bike");
        };
        let paths = paths(&t, T1);
        ways.push(paths.outward);
        let start = t.sim().tick() + 1;
        t.at(start, P1, eject(T1, BIKE));
        run_checked(&mut t, start);
        assert_eq!(exits(&t, T1), [(start, bike)], "{rotation:?}");
        let o = object(&t, bike);
        let out = bike_out(&t, T1);
        assert!(o.pos.distance(out) < 1e-3, "{rotation:?}: at {}, not {out}", o.pos);
        assert!(
            math::facing_vector(o.facing).distance(paths.outward) < 1e-6,
            "{rotation:?}: facing out"
        );
        let half = shape(&t, bike).bound_half();
        let along = paths.outward.abs();
        let (deep, across) = (half.dot(along), half.dot(Vec2::ONE - along));
        assert!(
            (deep - BIKE_LENGTH / 2.0).abs() < 1e-3 && (across - BIKE_WIDTH / 2.0).abs() < 1e-3,
            "{rotation:?}: 56 su out of the side, 32 along it: {half}"
        );
        let footprint = Shape::footprint(object(&t, T1).placement.as_ref().expect("placed"));
        assert!(
            !shape(&t, bike).overlaps(&footprint),
            "{rotation:?}: clear of the Tunnel"
        );
        assert!(shape(&t, bike).gap(&footprint) < 0.1, "{rotation:?}: against the wall");
        run_for(&mut t, 40);
        let o = object(&t, bike);
        let parked = out + paths.outward * sc_sim::SU_PER_GU;
        assert!(
            o.pos.distance(parked) < 1.0,
            "{rotation:?}: 1 grid unit out, at {}",
            o.pos
        );
        assert!(
            math::facing_vector(o.facing).distance(paths.outward) < 1e-4,
            "{rotation:?}: never turning"
        );
    }
    let mut sorted: Vec<(i32, i32)> = ways.iter().map(|v| (v.x.round() as i32, v.y.round() as i32)).collect();
    sorted.sort_unstable();
    assert_eq!(
        sorted,
        [(-1, 0), (0, -1), (0, 1), (1, 0)],
        "Side A faced each way: {ways:?}"
    );
}

/// `factions/syndicate/units/assault_bike.md`: "UnitBase - WheeledVehicle (exits only through Tier 2+ Tunnels)"
/// (`factions/syndicate/tunnels.md > Tunnel Network > Transit Tier Requirements`: "Tier 2+: Vehicles";
/// `factions/syndicate/tunnel_menus.md > EjectMenu`: "Unit types whose base category exceeds this Tunnel's tier are
/// [...] disabled"). With a Bike inside, player 1's second Tunnel (5), at Tier 1, refuses its Eject
/// ([`Reject::NoTransit`]) and lets nothing out when sent one anyway, and refuses a Bike's Enter, where the Tier 2
/// Tunnel takes both: the Bike comes out of it.
#[test]
fn only_a_tier_2_tunnel_lets_a_bike_out() {
    let mut t = started(
        plant()
            .spawn(ASSEMBLY, Some(P1), SpawnAt::Tile(VA_AT))
            .spawn(TUNNEL, Some(P1), tile(30, 4)),
        2,
    );
    let tier_1 = id(4);
    assert_eq!(object(&t, tier_1).tunnel.as_ref().map(|state| state.tier), Some(1));
    let [bike] = bikes_inside(&mut t, 1)[..] else {
        panic!("one Bike");
    };
    let refused = Reject::NoTransit {
        tunnel: tier_1,
        kind: Some(BIKE),
    };
    assert_eq!(gate(&t, P1, eject(tier_1, BIKE)), Err(refused));
    assert_eq!(gate(&t, P1, eject(T1, BIKE)), Ok(()), "the Tier 2 Tunnel lets it out");
    let start = t.sim().tick() + 1;
    t.at(start, P1, eject(tier_1, BIKE));
    run_for(&mut t, 20);
    assert_eq!(
        (exits(&t, tier_1), inside(&t)),
        (vec![], vec![bike]),
        "nothing comes out"
    );
    t.at(t.sim().tick() + 1, P1, eject(T1, BIKE));
    run_for(&mut t, 1);
    assert_eq!(exits(&t, T1).len(), 1);
    assert_eq!(object(&t, bike).pos, bike_out(&t, T1));
    let refused = Reject::NoTransit {
        tunnel: tier_1,
        kind: None,
    };
    assert_eq!(gate(&t, P1, order(&[bike], OrderKind::Enter(tier_1))), Err(refused));
    assert_eq!(gate(&t, P1, order(&[bike], OrderKind::Enter(T1))), Ok(()));
}

/// The plan's acceptance, "From a Tier 2 Tunnel, five queued Bikes come out of the exit path without overlapping", by
/// the Vehicle Assembly's rally point on the surface (`factions/syndicate/structures/README.md > Tunnel Expansions >
/// Rally Point Behavior`: "Unit auto-ejects from the parent Tunnel (Side A) and moves to the rally point"): five Bikes
/// queued at once come out one by one, each in the tick it is finished, 240 frames apart, on the exit path's spot,
/// overlapping no unit, and drive to the rally point north of the Tunnel, where they stop one behind another, none
/// left in the network or on the spot.
#[test]
fn five_bikes_trained_with_a_surface_rally_come_out_one_by_one() {
    let mut t = assembly();
    let to = Vec2::new(926.0, 160.0);
    let start = t.sim().tick() + 1;
    t.at(start, P1, rally(VA, Target::Point(to)));
    for _ in 0..5 {
        t.at(start, P1, train_bike(VA));
    }
    let mut out = Vec::new();
    while out.len() < 5 {
        assert!(t.sim().tick() < start + 5 * BIKE_FRAMES + 2, "{out:?}");
        step_checked(&mut t);
        let now = exits(&t, T1);
        if now.len() > out.len() {
            let (tick, bike) = now[out.len()];
            assert_eq!(tick, t.sim().tick());
            assert_eq!(object(&t, bike).pos, BIKE_OUT, "on the exit path's spot");
            assert!(!overlaps_a_unit(&t, bike), "clear of every unit");
            out.push((tick, bike));
        }
    }
    let finished: Vec<(Tick, ObjectId)> = produced(&t, VA);
    assert_eq!(out, finished, "each out in the tick it is finished");
    let ticks: Vec<Tick> = out.iter().map(|&(tick, _)| tick).collect();
    assert_eq!(ticks, [1, 2, 3, 4, 5].map(|n| start + n * BIKE_FRAMES - 1));
    run_for(&mut t, 300);
    assert!(inside(&t).is_empty() && queued(&t, T1).is_empty());
    for &(_, bike) in &out {
        let o = object(&t, bike);
        assert!(o.pos.distance(to) < 6.0 * sc_sim::SU_PER_GU, "{bike:?} at {}", o.pos);
        assert!(o.pos.y < 400.0, "{bike:?} off the exit path: {}", o.pos);
    }
}

/// Five Bikes ejected at once (`factions/syndicate/tunnel_menus.md > EjectMenu`: "Ejected units are queued"; design
/// change 0250: "If that spot is blocked [...], the next unit waits until it clears"), each sent on by its player as it
/// comes out: they come out one by one, in the order they went in ("the one that has been inside the network longest",
/// design change 0251), each on the exit path's spot once the one before has driven off it, overlapping no unit.
#[test]
fn five_ejected_bikes_come_out_one_by_one_as_each_is_sent_on() {
    let mut t = assembly();
    let bikes = bikes_inside(&mut t, 5);
    let start = t.sim().tick() + 1;
    for _ in 0..5 {
        t.at(start, P1, eject(T1, BIKE));
    }
    let mut out: Vec<(Tick, ObjectId)> = Vec::new();
    while out.len() < 5 {
        assert!(t.sim().tick() < start + 200, "{out:?}\n{}", t.ascii(None));
        step_checked(&mut t);
        let now = exits(&t, T1);
        if now.len() > out.len() {
            let (tick, bike) = now[out.len()];
            assert_eq!(object(&t, bike).pos, BIKE_OUT, "on the exit path's spot");
            assert!(!overlaps_a_unit(&t, bike), "clear of every unit");
            let away = Vec2::new(320.0 + 320.0 * out.len() as f32, 160.0);
            t.order_at(tick + 1, P1, &[bike], OrderKind::Move(Target::Point(away)), false);
            out.push((tick, bike));
        }
    }
    let order: Vec<ObjectId> = out.iter().map(|&(_, bike)| bike).collect();
    assert_eq!(order, bikes, "longest inside first");
    assert!(
        out.windows(2).all(|pair| pair[0].0 < pair[1].0),
        "one at a time: {out:?}"
    );
}

/// The brief's question about design change 0440 (`systems/units.md`: an idle wheeled or tracked vehicle doesn't make
/// way), for the Director: five Bikes ejected at once with no order, as the EjectMenu leaves them
/// (`factions/syndicate/tunnel_menus.md > EjectMenu`: "with no rally point, a unit walks about 1 grid unit straight out
/// of Side A and idles"). The first comes out and parks 1 grid unit out, idle; the second comes out behind it once its
/// box is off the spot, and heads for the same 1 grid unit out, which the first holds and never gives up: it stops
/// short behind it ("arrived", the occupied-target rule), its box still over the exit path's spot. The spot never
/// clears: the other three wait in the queue for good, in the network. Only the player can free the path, by moving the
/// two on: then the third comes out. (Design change 0446 gave the Chop Shop and the Factory a default rally point for
/// this, and left the Syndicate's Tunnels out.)
#[test]
fn five_bikes_ejected_with_no_order_stall_the_exit_path_after_two() {
    let mut t = assembly();
    let bikes = bikes_inside(&mut t, 5);
    let start = t.sim().tick() + 1;
    for _ in 0..5 {
        t.at(start, P1, eject(T1, BIKE));
    }
    run_for(&mut t, 600);
    let out: Vec<ObjectId> = exits(&t, T1).iter().map(|&(_, bike)| bike).collect();
    assert_eq!(out, bikes[..2], "two come out");
    assert_eq!((queued(&t, T1), inside(&t)), (bikes[2..].to_vec(), bikes[2..].to_vec()));
    let (first, second) = (bikes[0], bikes[1]);
    assert!(object(&t, first).pos.distance(BIKE_OUT - Vec2::new(0.0, 64.0)) < 1.0);
    assert_eq!(t.status(first.0), MoveStatus::Arrived);
    assert_eq!(
        t.status(second.0),
        MoveStatus::Arrived,
        "stopped short behind the first"
    );
    let next_out = Shape::silhouette(BIKE, BIKE_OUT, -std::f32::consts::FRAC_PI_2);
    assert!(shape(&t, second).overlaps(&next_out), "over the exit path's spot");
    assert!(
        shape(&t, second).gap(&shape(&t, first)) < 1.0,
        "against the first's back"
    );
    // The player moves both on: the third comes out.
    let tick = t.sim().tick() + 1;
    t.order_at(
        tick,
        P1,
        &[first, second],
        OrderKind::Move(Target::Point(Vec2::new(926.0, 160.0))),
        false,
    );
    step_until(&mut t, 120, "the third Bike comes out", |t| exits(t, T1).len() == 3);
    assert_eq!(exits(&t, T1)[2].1, bikes[2]);
}

/// "can only enter or exit the network through a Tunnel whose tier is sufficient for their unit base category"
/// (`factions/syndicate/tunnels.md > Tunnel Network`), the way in (`systems/controls/commands.md > Enter`: "The unit
/// walks to the structure's entrance side and goes inside"): a Bike north of the Tier 2 Tunnel, ordered in, drives
/// into the entry path, its box turned with it (slice M5-V1: 56 su deep against the north-facing side), and goes into
/// the network; and once the transit delay has passed it comes out again.
#[test]
fn a_bike_drives_into_a_tier_2_tunnel_through_the_entry_path() {
    let mut t = assembly();
    let paths = paths(&t, T1);
    assert_eq!(
        (paths.entry, paths.exit, paths.outward),
        (ENTRY_SPOT, EXIT_SPOT, Vec2::NEG_Y)
    );
    assert_eq!((paths.entry + paths.exit) / 2.0, SIDE_A);
    let bike = id(next_id(&t));
    t.spawn_unit_at(2, BIKE, P1, Vec2::new(700.0, 400.0));
    run_checked(&mut t, 2);
    assert_eq!(gate(&t, P1, order(&[bike], OrderKind::Enter(T1))), Ok(()));
    t.at(3, P1, order(&[bike], OrderKind::Enter(T1)));
    step_until(&mut t, 200, "the Bike drives in", |t| t.sim().held_unit(bike).is_some());
    let entered = (t.events().iter()).any(|(_, e)| *e == SimEvent::EnteredNetwork { unit: bike, tunnel: T1 });
    assert!(entered);
    assert!(!(t.events().iter()).any(|(_, e)| matches!(e, SimEvent::EnterFailed { .. })));
    let delay = sc_sim::network::rules().transit_delay;
    t.at(t.sim().tick() + delay + 1, P1, eject(T1, BIKE));
    run_for(&mut t, delay + 1);
    assert_eq!(exits(&t, T1).last().map(|&(_, unit)| unit), Some(bike));
    assert_eq!(object(&t, bike).pos, BIKE_OUT);
}
