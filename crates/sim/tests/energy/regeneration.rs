//! Regeneration (`sc_sim::energy`): 0.5 a second, 2048 of `Object::energy`'s units a tick, from the start up to
//! MaxEnergy, every tick after the one the unit came into the game in.

use sc_sim::{Command, OrderKind};

use super::*;

/// The plan's acceptance, "Energy: start, cap and regeneration, exact in integers": a Disciple of the map's starts at
/// 50, and each tick adds 2048 exactly; one spawned in tick 1 has its start at that tick's end, as an object a command
/// brings in regenerates from the next tick (`docs/architecture/tick-pipeline.md > Tick pipeline`). 150 points to go take each 4800 ticks,
/// 300 s, to the tick, and then the energy stays at 200.
#[test]
fn a_disciple_regenerates_from_50_to_200_in_300_seconds_exactly() {
    let mut t = cults().spawn(DISCIPLE, Some(P1), point(300.0, 300.0)).start();
    let (first, spawned) = (ObjectId(1), ObjectId(2));
    t.spawn_unit_at(1, DISCIPLE, P1, Vec2::new(600.0, 300.0));
    assert_eq!(raw(&t, first), Some(50 * ONE), "its start");
    t.step();
    assert_eq!(raw(&t, first), Some(50 * ONE + A_TICK));
    assert_eq!(
        raw(&t, spawned),
        Some(50 * ONE),
        "its start at the end of the tick it came in"
    );
    for tick in 2..=40 {
        t.step();
        assert_eq!(raw(&t, first), Some(50 * ONE + tick * A_TICK), "tick {tick}");
    }
    t.run_until(4800);
    assert_eq!(raw(&t, first), Some(200 * ONE), "300 s after the start");
    assert_eq!(raw(&t, spawned), Some(200 * ONE - A_TICK), "a tick behind");
    t.step();
    assert_eq!((raw(&t, first), raw(&t, spawned)), (Some(200 * ONE), Some(200 * ONE)));
    t.run(32);
    assert_eq!(raw(&t, first), Some(200 * ONE), "never above MaxEnergy");
    let shown = sc_sim::energy::of(t.sim().object(first).unwrap());
    assert_eq!(shown, Some((200.0, 200.0)), "in points");
}

/// Placeholder (not in the design: "It regenerates naturally over time" names no exception): a Disciple inside a
/// building, a Garrison (`factions/cults/structures/garrison.md`: it takes the Cults' infantry), regenerates as on the
/// map, and comes out with what it gained inside.
#[test]
fn a_disciple_inside_a_garrison_regenerates_too() {
    let mut t = cults()
        .spawn(ObjectKind::Garrison, Some(P1), tile(10, 10))
        .spawn(DISCIPLE, Some(P1), point(900.0, 700.0))
        .start();
    t.without_auto_targeting();
    let (garrison, disciple) = (ObjectId(1), ObjectId(2));
    t.at(
        1,
        P1,
        Command::Order {
            units: vec![disciple],
            kind: OrderKind::Enter(garrison),
            queue: false,
        },
    );
    while t.sim().held_unit(disciple).is_none() {
        assert!(t.sim().tick() < 200, "it goes in");
        t.step();
    }
    let entered = raw(&t, disciple).unwrap();
    t.run(160);
    assert!(t.sim().held_unit(disciple).is_some(), "inside");
    assert_eq!(raw(&t, disciple), Some(entered + 160 * A_TICK), "10 s inside: 5 points");
}
