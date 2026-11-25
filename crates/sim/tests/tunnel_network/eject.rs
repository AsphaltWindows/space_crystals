//! The Eject command and the exit path (`factions/syndicate/tunnels.md > EjectMenu`, `> Tunnel > SymmetryType -
//! ABCD`; design changes 0248, 0250 and 0251).

use sc_sim::network::{PATH_OFFSET_SU, TRANSIT_DELAY_TICKS};
use sc_sim::scenario::BuildingOrientation;
use sc_sim::{PlayerCommand, Reject, Rotation, SideLetter, Target, sides};

use super::*;

fn validate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// `count` Guards of player 1's (ids from `first`) in a column north of [`base`]'s entry path, ordered into the
/// Tunnel at tick 2; runs until all are inside and past the transit delay.
fn guards_inside(t: &mut TestSim, count: u32) -> Vec<ObjectId> {
    let first = next_id(t);
    let guards: Vec<ObjectId> = (0..count).map(|i| id(first + i)).collect();
    for i in 0..count {
        t.spawn_unit_at(1, GUARD, P1, Vec2::new(866.0, 580.0 - 40.0 * i as f32));
    }
    t.at(2, P1, enter(&guards, T1));
    while inside(t).len() < count as usize {
        assert!(t.sim().tick() < 100 + 20 * count, "they file in: {:?}", inside(t));
        step_checked(t);
    }
    let last = entries(t).last().unwrap().0;
    run_checked(t, last + TRANSIT_DELAY_TICKS - 1);
    guards
}

/// Design change 0248's Verification: "Ejected units come out of the half of Side A nearer the crystal hatch." In
/// each of the Tunnel's 8 orientations ("Both turn and flip with the Tunnel") a Guard goes in by the half nearer Side
/// D and comes out on the half nearer Side B, its box against the wall, then walks 1 grid unit straight out.
#[test]
fn units_go_in_nearer_d_and_come_out_nearer_b_in_every_orientation() {
    for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
        for flip_h in [false, true] {
            let orientation = BuildingOrientation {
                rotation,
                flip_h,
                flip_v: false,
            };
            let mut t = syndicate(&flat(30, 24)).start();
            t.without_auto_targeting().debug_at(
                1,
                DebugCommand::Spawn {
                    kind: TUNNEL,
                    owner: Some(P1),
                    at: tile(13, 10),
                    orientation,
                },
            );
            run_checked(&mut t, 1);
            // The footprint covers tiles (13, 10) to (16, 13): its centre is (960, 768).
            let centre = Vec2::new(960.0, 768.0);
            let letters = sides(TUNNEL, orientation).unwrap();
            let facing = |letter: SideLetter| {
                let (x, y) = sc_sim::sides::DIRECTIONS[letters.iter().position(|&l| l == letter).unwrap()];
                Vec2::new(x as f32, y as f32)
            };
            let (out, to_b, to_d) = (facing(SideLetter::A), facing(SideLetter::B), facing(SideLetter::D));
            // A Guard 3 grid units out from the middle of Side A.
            t.spawn_unit_at(2, GUARD, P1, centre + out * (128.0 + 192.0))
                .at(3, P1, enter(&[id(2)], T1));
            let mut last = Vec2::ZERO;
            while inside(&t).is_empty() {
                assert!(t.sim().tick() < 120, "{orientation:?}");
                last = t.sim().object(id(2)).map_or(last, |o| o.pos);
                step_checked(&mut t);
            }
            assert!(
                (last - centre).dot(to_d) > 0.0,
                "in by the half nearer D: {orientation:?}"
            );
            let entered = t.sim().tick();
            t.at(entered + TRANSIT_DELAY_TICKS, P1, eject(T1, GUARD));
            run_checked(&mut t, entered + TRANSIT_DELAY_TICKS);
            let stand = centre + out * (128.0 + 18.0 + 1.0 / 64.0) + to_b * PATH_OFFSET_SU;
            assert_eq!(t.pos(2), stand, "out on the half nearer B: {orientation:?}");
            assert_eq!(paths(&t, T1).exit_stand(Vec2::splat(18.0)), stand);
            run_checked(&mut t, entered + TRANSIT_DELAY_TICKS + 30);
            assert_eq!(
                t.pos(2),
                stand + out * 64.0,
                "1 grid unit straight out: {orientation:?}"
            );
            assert!(t.sim().object(id(2)).unwrap().orders.as_ref().unwrap().is_idle());
        }
    }
}

/// Design change 0250's Verification ("Ejecting 10 Guards: they stream out continuously") and the plan's "Five
/// ejects at one Tunnel", as 0250 changed it: no minimum gap, "the exit rate is set by unit speed and size". Each
/// Guard appears on the exit spot as soon as the one before has walked off it: 36 su at 5 su a tick, 8 ticks, while
/// the way out is free. With no rally point each "walks about 1 grid unit straight out of Side A and idles", so the
/// later ones find the earlier ones standing there, which step aside for them (friendly yielding): the stream goes
/// on, a little slower.
#[test]
fn ten_guards_stream_out_of_the_exit_path() {
    let mut t = base().start();
    t.without_auto_targeting();
    let guards = guards_inside(&mut t, 10);
    let tick = t.sim().tick() + 1;
    for _ in 0..10 {
        t.at(tick, P1, eject(T1, GUARD));
    }
    run_checked(&mut t, tick + 120);
    let out = exits(&t, T1);
    // In the order they went in (the one inside longest first, design change 0251).
    let order: Vec<ObjectId> = out.iter().map(|&(_, unit)| unit).collect();
    let went_in: Vec<ObjectId> = entries(&t).iter().map(|&(_, unit)| unit).collect();
    assert_eq!(order, went_in);
    assert_eq!(order.len(), guards.len());
    let ticks: Vec<Tick> = out.iter().map(|&(at, _)| at).collect();
    assert_eq!(ticks[0], tick, "the first at once");
    let gaps: Vec<Tick> = ticks.windows(2).map(|w| w[1] - w[0]).collect();
    assert_eq!(gaps[0], 8, "as soon as the first is off the spot: {gaps:?}");
    assert!(
        gaps.iter().all(|&gap| (8..=16).contains(&gap)),
        "a stream, none held up: {gaps:?}"
    );
    // All of them on the exit spot, none anywhere else.
    for (i, &(at, unit)) in out.iter().enumerate() {
        let appeared = t
            .events()
            .iter()
            .any(|(tick, e)| *tick == at && *e == SimEvent::LeftNetwork { unit, tunnel: T1 });
        assert!(appeared, "#{i}");
    }
    assert!(inside(&t).is_empty());
}

/// The plan's acceptance for Eject and design change 0250: "If that spot is blocked (by any unit or structure), the
/// next unit waits until it clears; no other spot is tried." A unit parked on the exit spot holds the queue until it
/// moves; one parked elsewhere along Side A doesn't.
#[test]
fn a_unit_on_the_exit_spot_holds_the_queue_until_it_moves() {
    let mut t = base().start();
    t.without_auto_targeting();
    guards_inside(&mut t, 2);
    let tick = t.sim().tick() + 1;
    // Player 2's Guard on the exit spot, and player 1's Agent against Side A on its other end.
    let (blocker, parked) = (id(4), id(5));
    t.spawn_unit_at(tick, GUARD, P2, EXIT + Vec2::new(10.0, -8.0))
        .spawn_unit_at(tick, AGENT, P1, Vec2::new(800.0, 621.0))
        .at(tick + 1, P1, eject(T1, GUARD));
    run_checked(&mut t, tick + 80);
    assert_eq!(exits(&t, T1), [], "the queue waits");
    assert_eq!(t.sim().object(T1).unwrap().tunnel.as_ref().unwrap().eject.len(), 1);
    // The blocker walks off; the parked Agent stays.
    let away = OrderKind::Move(Target::Point(Vec2::new(1300.0, 500.0)));
    t.order_at(tick + 81, P2, &[blocker], away, false);
    run_checked(&mut t, tick + 120);
    let out = exits(&t, T1);
    assert_eq!(out.len(), 1);
    assert!(out[0].0 > tick + 81 && out[0].0 < tick + 100, "{out:?}");
    assert_eq!(t.pos(parked.0), Vec2::new(800.0, 621.0));
    assert!(t.sim().object(T1).unwrap().tunnel.as_ref().unwrap().eject.is_empty());
}

/// Design change 0251's Verification: "With two Guards inside (one entered 30 s ago, one 5 s ago), ejecting a Guard
/// brings out the one that entered first."
#[test]
fn the_unit_inside_longest_comes_out_first() {
    let mut t = base().start();
    t.without_auto_targeting()
        .spawn_unit_at(1, GUARD, P1, Vec2::new(866.0, 560.0)) // 2: the later one, though the lower id
        .spawn_unit_at(1, GUARD, P1, Vec2::new(700.0, 560.0)) // 3
        .at(2, P1, enter(&[id(3)], T1))
        .at(2 + 400, P1, enter(&[id(2)], T1));
    run_checked(&mut t, 2 + 480);
    let went_in = entries(&t);
    assert_eq!(
        went_in.iter().map(|&(_, unit)| unit).collect::<Vec<_>>(),
        [id(3), id(2)]
    );
    assert!(went_in[1].0 - went_in[0].0 >= 370, "{went_in:?}");
    t.at(2 + 481, P1, eject(T1, GUARD));
    run_checked(&mut t, 2 + 481);
    assert_eq!(exits(&t, T1), [(2 + 481, id(3))]);
    assert_eq!(inside(&t), [id(2)]);
}

/// `> EjectMenu` (design change 0251): "each click reserves a specific unit (the tile's count drops at the click);
/// the queue can't be cancelled; if the Tunnel is destroyed with a queue, the queued units stay in the network".
#[test]
fn a_click_reserves_a_unit_and_a_destroyed_tunnel_frees_it() {
    let mut t = base().spawn(TUNNEL, Some(P1), tile(22, 2)).start(); // 2
    t.without_auto_targeting();
    let guards = guards_inside(&mut t, 2); // 3 and 4
    let count = |t: &TestSim| t.sim().network_counts(P1).get(&GUARD).copied().unwrap_or(0);
    assert_eq!(count(&t), 2);
    let tick = t.sim().tick() + 1;
    // The exit spot is blocked, so the queue stands.
    t.spawn_unit_at(tick, GUARD, P2, EXIT)
        .at(tick + 1, P1, eject(T1, GUARD))
        .at(tick + 2, P1, eject(T1, GUARD));
    run_checked(&mut t, tick + 1);
    assert_eq!((count(&t), t.sim().reserved()), (1, vec![guards[0]]));
    run_checked(&mut t, tick + 2);
    assert_eq!((count(&t), t.sim().reserved()), (0, guards.clone()));
    assert_eq!(inside(&t), guards, "reserved, and still inside");
    // No unit of the kind is free: a third click is refused, at either Tunnel.
    for tunnel in [T1, id(2)] {
        assert_eq!(validate(&t, P1, eject(tunnel, GUARD)), Err(Reject::NotInNetwork(GUARD)));
    }
    assert_eq!(validate(&t, P1, eject(T1, AGENT)), Err(Reject::NotInNetwork(AGENT)));
    // The Tunnel is destroyed with its queue: both stay in the network, free to leave by the other Tunnel.
    t.debug_at(tick + 10, DebugCommand::Remove { id: T1 })
        .at(tick + 11, P1, eject(id(2), GUARD));
    run_checked(&mut t, tick + 11);
    assert_eq!(exits(&t, id(2)), [(tick + 11, guards[0])]);
    assert_eq!((count(&t), inside(&t)), (1, vec![guards[1]]));
}

/// The gate for an Eject: the player's own Tunnel in operation, whose tier lets the kind out (the plan's tiers: a
/// vehicle "is refused [...] Eject at a Tier 1 Tunnel, and allowed at Tier 2").
#[test]
fn the_gate_takes_an_eject_by_the_tunnels_owner_and_tier() {
    let mut t = base()
        .spawn(HIDEOUT, Some(P1), tile(16, 11)) // 2
        .spawn(TUNNEL, Some(P1), tile(22, 2)) // 3: Tier 2 below
        .start();
    t.without_auto_targeting()
        .debug_at(1, DebugCommand::SetTier { id: id(3), tier: 2 })
        .debug_at(1, spawn(TUNNEL, P2, tile(2, 14))) // 4
        .spawn_unit_at(1, ObjectKind::Drone, P1, Vec2::new(1500.0, 110.0)) // 5: a vehicle
        .at(2, P1, enter(&[id(5)], id(3)));
    while inside(&t).is_empty() {
        assert!(t.sim().tick() < 200);
        step_checked(&mut t);
    }
    let entered = t.sim().tick();
    run_checked(&mut t, entered + TRANSIT_DELAY_TICKS);
    let drone = ObjectKind::Drone;
    let no_transit = Reject::NoTransit {
        tunnel: T1,
        kind: Some(drone),
    };
    assert_eq!(validate(&t, P1, eject(T1, drone)), Err(no_transit), "Tier 1");
    assert_eq!(validate(&t, P1, eject(id(3), drone)), Ok(()), "Tier 2");
    assert_eq!(validate(&t, P1, eject(id(4), drone)), Err(Reject::NotOwned(id(4))));
    assert_eq!(validate(&t, P2, eject(id(3), drone)), Err(Reject::NotOwned(id(3))));
    assert_eq!(validate(&t, P1, eject(id(2), drone)), Err(Reject::NotATunnel(id(2))));
    assert_eq!(
        validate(&t, P1, eject(id(99), drone)),
        Err(Reject::UnknownObject(id(99)))
    );
    assert_eq!(validate(&t, P1, eject(id(3), GUARD)), Err(Reject::NotInNetwork(GUARD)));
    assert_eq!(
        validate(&t, P1, eject(id(3), TUNNEL)),
        Err(Reject::NoTransit {
            tunnel: id(3),
            kind: Some(TUNNEL)
        })
    );
    // Refused at Tier 1, it stays inside; then it comes out of the Tier 2 Tunnel.
    let tick = t.sim().tick();
    t.at(tick + 1, P1, eject(T1, drone))
        .at(tick + 2, P1, eject(id(3), drone));
    run_checked(&mut t, tick + 1);
    assert_eq!(inside(&t), [id(5)]);
    run_checked(&mut t, tick + 2);
    assert_eq!(exits(&t, id(3)), [(tick + 2, id(5))]);
}
