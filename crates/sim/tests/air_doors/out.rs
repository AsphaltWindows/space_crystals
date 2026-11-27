//! Out of the air exit door (`factions/syndicate/tunnel_menus.md > EjectMenu`: "**Air units** leave through the Tier 3
//! **air exit door**, in parallel with ground units; since aircraft don't collide, the door has its own **exit rate:
//! one aircraft per 8 frames (0.5 s), placeholder**"; plan decision T96: "Eject entries for air kinds release from the
//! air exit door on their own lane, one per 8 frames, beside the ground lane. The queue keeps its order within each
//! lane"), and the rise out of it (design change 0431: "it **rises** out of the exit door over **16 frames (1 s)**
//! [...], and it can be hit during both **as an air unit**. At the doors' rate (one aircraft per 8 frames) up to two
//! Phantoms rise at once").

use super::*;

/// The plan's acceptance, "Five Phantoms come out one every 8 ticks, while Guards walk out of Side A at the same time":
/// five Phantoms and three Guards trained into the network, all eight ejected in one tick, the Phantoms first in the
/// queue. The air lane lets the first Phantom out in that tick, over the air exit door, and each of the others 8 ticks
/// after the one before, in the queue's order; Side A's ground lane lets the first Guard out in that tick too, on the
/// exit path's spot (`Paths::exit_stand`), never waiting for the Phantoms ahead of it in the queue, and the others as
/// the spot clears, in the queue's order, the last before the last Phantom.
#[test]
fn five_phantoms_come_out_one_every_8_ticks_while_guards_walk_out_of_side_a() {
    let mut t = hangar();
    for _ in 0..3 {
        t.at(2, P1, train(HIDEOUT, GUARD));
    }
    let phantoms = trained_inside(&mut t, AA, PHANTOM, 5);
    let guards: Vec<ObjectId> = produced(&t, HIDEOUT).iter().map(|&(_, unit)| unit).collect();
    assert_eq!((guards.len(), inside(&t).len()), (3, 8));
    let out = t.sim().tick() + 1;
    for _ in 0..5 {
        t.at(out, P1, eject(T1, PHANTOM));
    }
    for _ in 0..3 {
        t.at(out, P1, eject(T1, GUARD));
    }
    run_checked(&mut t, out);
    assert_eq!(
        exits(&t, T1),
        [(out, guards[0]), (out, phantoms[0])],
        "a unit of each lane at once"
    );
    let half = object(&t, guards[0]).def().half_extents().unwrap();
    assert_eq!(
        object(&t, guards[0]).pos,
        paths(&t, T1).exit_stand(half),
        "on Side A's exit path"
    );
    assert_eq!(object(&t, phantoms[0]).pos, AIR_EXIT, "over the air exit door");
    step_until(&mut t, 200, "every unit comes out", |t| exits(t, T1).len() == 8);
    let left = exits(&t, T1);
    let lane = |units: &[ObjectId]| -> Vec<(Tick, ObjectId)> {
        left.iter().copied().filter(|(_, unit)| units.contains(unit)).collect()
    };
    let (air, ground) = (lane(&phantoms), lane(&guards));
    let expected: Vec<(Tick, ObjectId)> = (phantoms.iter().enumerate())
        .map(|(i, &unit)| (out + DOOR_RATE * i as Tick, unit))
        .collect();
    assert_eq!(air, expected, "one every 8 ticks, in the queue's order");
    let order: Vec<ObjectId> = ground.iter().map(|&(_, unit)| unit).collect();
    assert_eq!(order, guards, "in the queue's order");
    assert!(
        ground[2].0 < air[4].0,
        "the Guards never wait for the Phantoms: {ground:?}, {air:?}"
    );
    assert!(queued(&t, T1).is_empty() && inside(&t).is_empty());
}

/// Design change 0431's "up to two Phantoms rise at once": two Phantoms ejected together, the second 8 ticks after
/// the first by the door's rate. Each appears over the air exit door with its rise at 0 of 16 ticks, the second while
/// the first has done 8; from then until the first's rise ends both are rising, 8 ticks, the first ahead of the second
/// by 8; the first's ends 16 ticks after it came out, the second's 16 after it did. Neither has an order: each stays
/// near the door once risen, pushed apart by the air units' separation (Placeholder (not in the design: an aircraft
/// out of the air doors with no rally point)).
#[test]
fn two_phantoms_ejected_8_frames_apart_are_both_rising_at_once() {
    let mut t = hangar();
    let [first, second] = trained_inside(&mut t, AA, PHANTOM, 2)[..] else {
        panic!("two Phantoms");
    };
    let out = t.sim().tick() + 1;
    t.at(out, P1, eject(T1, PHANTOM));
    t.at(out, P1, eject(T1, PHANTOM));
    run_checked(&mut t, out);
    assert_eq!(rising(&t, first), Some(0));
    run_checked(&mut t, out + DOOR_RATE - 1);
    assert!(
        t.sim().object(second).is_none(),
        "the door's rate holds the second back"
    );
    let mut both = 0;
    while t.sim().tick() < out + DOOR_RATE + DOOR_FRAMES {
        step_checked(&mut t);
        let tick = t.sim().tick();
        let (a, b) = (rising(&t, first), rising(&t, second));
        match tick - out {
            n if n < DOOR_FRAMES => assert_eq!(a, Some(n), "tick {tick}"),
            _ => assert_eq!(a, None, "tick {tick}"),
        }
        assert_eq!(
            b,
            (tick - out - DOOR_RATE < DOOR_FRAMES).then_some(tick - out - DOOR_RATE),
            "tick {tick}"
        );
        if tick == out + DOOR_RATE {
            assert_eq!(object(&t, second).pos, AIR_EXIT, "the second appears over the door");
        }
        if let (Some(a), Some(b)) = (a, b) {
            assert_eq!(a, b + DOOR_RATE, "tick {tick}");
            both += 1;
        }
    }
    assert_eq!(both, DOOR_RATE, "both rising for 8 ticks");
    assert_eq!(
        rising(&t, second),
        None,
        "the second's rise over 16 ticks after it came out"
    );
    assert_eq!(exits(&t, T1), [(out, first), (out + DOOR_RATE, second)]);
}

/// Design change 0431's "it can be hit during both **as an air unit**": a Phantom rallied north comes out over the air
/// exit door, and in the next tick player 2's Guard, 96 su east of the door off the Tunnel's roof (its laser's
/// TargetDomain is Universal), and player 2's Peacekeeper beside it (its TargetDomain is Ground: "a Peacekeeper hits
/// it" in the brief can't be, a reading for the Director) are ordered to attack it. The Guard hits it while it rises,
/// still over the door; the Peacekeeper never does, and its right-click on the Phantom is a Move to it, not an Attack
/// (`orders::resolution`), as on any air unit.
#[test]
fn a_rising_phantom_is_an_air_target() {
    let mut t = hangar();
    let guard = id(next_id(&t));
    let peacekeeper = id(guard.0 + 1);
    t.debug_at(2, spawn(GUARD, P2, SpawnAt::Point(AIR_EXIT + Vec2::new(96.0, 0.0))));
    t.debug_at(
        2,
        spawn(
            ObjectKind::Peacekeeper,
            P2,
            SpawnAt::Point(AIR_EXIT + Vec2::new(96.0, 72.0)),
        ),
    );
    t.at(2, P1, rally(AA, Target::Point(Vec2::new(948.0, 256.0))));
    t.at(2, P1, train(AA, PHANTOM));
    let done = 2 + PHANTOM_FRAMES - 1;
    run_checked(&mut t, done);
    let phantom = produced(&t, AA)[0].1;
    assert_eq!((exits(&t, T1), rising(&t, phantom)), (vec![(done, phantom)], Some(0)));
    let right_click = |unit| t.sim().default_order(unit, Target::Object(phantom));
    assert_eq!(right_click(guard), Some(OrderKind::Attack(phantom)));
    assert_eq!(right_click(peacekeeper), Some(OrderKind::Move(Target::Object(phantom))));
    let attack = order(&[guard, peacekeeper], OrderKind::Attack(phantom));
    assert_eq!(gate(&t, P2, attack.clone()), Ok(()), "player 2 sees it");
    t.at(done + 1, P2, attack);
    let mut hit_rising = 0;
    while t.sim().tick() < done + DOOR_FRAMES - 1 {
        step_checked(&mut t);
        let now = t.sim().tick();
        let hits: Vec<_> = (hits_on(&t, phantom).into_iter())
            .filter(|&(tick, ..)| tick == now)
            .collect();
        if !hits.is_empty() {
            assert!(rising(&t, phantom).is_some(), "tick {now}: rising");
            assert_eq!(object(&t, phantom).pos, AIR_EXIT, "over the door");
            assert!(hits.iter().all(|&(_, by, _)| by == guard), "{hits:?}");
            hit_rising += hits.len();
        }
    }
    assert!(hit_rising >= 1, "the Guard hits it during its rise");
    let hp = object(&t, phantom).hp.expect("destructible");
    assert!(hp < 70.0, "{hp}");
    run_for(&mut t, 64);
    assert!(
        hits_on(&t, phantom).iter().all(|&(_, by, _)| by != peacekeeper),
        "never the Peacekeeper"
    );
}
