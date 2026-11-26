//! Evacuation on destruction (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`:
//! "Everyone inside is evacuated alive"; "Units come out unchanged"; "evacuated units appear around the building's
//! footprint wherever there is open space, unharmed"), at the spots of plan decision T62 in Q0106's order. A debug
//! removal stands for the destruction: both are the one way objects leave the game (`Sim::remove_objects`).

use super::*;

/// Player 1's Recruits ordered into [`A1`] at tick 2, run until they all are in; they are in the order they went in,
/// longest inside first.
fn fill(t: &mut TestSim, recruits: &[ObjectId]) -> Vec<ObjectId> {
    t.at(2, P1, enter(recruits, A1));
    run_checked(t, 2);
    while inside(t, A1).len() < recruits.len() {
        assert!(t.sim().tick() < 200, "{} are in", inside(t, A1).len());
        step_checked(t);
    }
    inside(t, A1)
}

/// The events of tick `tick`, in the order the sim emitted them.
fn events_of(t: &TestSim, tick: Tick) -> Vec<SimEvent> {
    (t.events().iter())
        .filter(|&&(at, _)| at == tick)
        .map(|(_, e)| e.clone())
        .collect()
}

/// The units evacuated out of `building` so far, in order.
fn evacuated(t: &TestSim, building: ObjectId) -> Vec<ObjectId> {
    (t.events().iter())
        .filter_map(|(_, e)| match e {
            SimEvent::Evacuated { unit, building: from } if *from == building => Some(*unit),
            _ => None,
        })
        .collect()
}

/// No two of `units` overlap: their 24 x 24 boxes may touch, no more.
fn apart(t: &TestSim, units: &[ObjectId]) {
    for (i, &a) in units.iter().enumerate() {
        for &b in &units[i + 1..] {
            let d = (t.sim().object(a).unwrap().pos - t.sim().object(b).unwrap().pos).abs();
            assert!(d.x >= 2.0 * HALF.x || d.y >= 2.0 * HALF.y, "{a:?} and {b:?} overlap");
        }
    }
}

/// The plan's acceptance (slice M5-F1): "Destroying the Armory with ten inside puts all ten around it, alive, in the
/// fixed order, none overlapping anything." In the tick the Armory goes, after its `Removed`, each comes out unharmed
/// at the first spot around the footprint where it fits (Placeholder (Q0106 point 1): "Rings around the footprint,
/// nearest first, each ring clockwise from the middle of the exit side"), longest inside first (point 2), and idles
/// there (point 4). Ring 0's boxes stand against the footprint (x 768..960, y 640..768): from the middle of side C
/// (west) north along it, its corner, then east along side A, each a box (24 su) from the last.
#[test]
fn ten_recruits_inside_come_out_alive_around_the_armory_in_a_fixed_order() {
    // Player 1 keeps a Storage, and so isn't defeated with the Armory.
    let mut t = base().spawn(ObjectKind::Storage, Some(P1), tile(2, 2)).start(); // 2
    t.without_auto_targeting();
    let recruits = block(&mut t, 1, 10, 5, Vec2::new(1150.0, 704.0));
    let order = fill(&mut t, &recruits);
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, A1);
    run_checked(&mut t, gone);
    let events = events_of(&t, gone);
    let removed = events.iter().position(|e| *e == SimEvent::Removed(A1));
    let first = events.iter().position(|e| matches!(e, SimEvent::Evacuated { .. }));
    assert!(removed.is_some_and(|r| first.is_some_and(|f| r < f)), "{events:?}");
    assert_eq!(evacuated(&t, A1), order, "longest inside first");
    assert!(t.sim().held().next().is_none());
    let spots = [
        (756.0, 704.0),
        (756.0, 680.0),
        (756.0, 656.0),
        (756.0, 628.0),
        (792.0, 628.0),
        (816.0, 628.0),
        (840.0, 628.0),
        (864.0, 628.0),
        (888.0, 628.0),
        (912.0, 628.0),
    ];
    for (&unit, (x, y)) in order.iter().zip(spots) {
        let o = t.sim().object(unit).expect("alive, on the map");
        assert!(
            o.pos.distance(Vec2::new(x, y)) < 0.1,
            "{unit:?} at {}, not ({x}, {y})",
            o.pos
        );
        assert_eq!((o.owner, o.hp), (Some(P1), Some(40.0)), "unchanged");
        assert!(o.orders.as_ref().unwrap().is_idle());
    }
    apart(&t, &order);
    // They idle where they came out.
    let at: Vec<Vec2> = order.iter().map(|&unit| t.sim().object(unit).unwrap().pos).collect();
    run_checked(&mut t, gone + 32);
    let now: Vec<Vec2> = order.iter().map(|&unit| t.sim().object(unit).unwrap().pos).collect();
    assert_eq!(now, at);
    assert!(t.sim().player(P1).is_some_and(|p| !p.is_defeated()));
}

/// "Wherever there is open space": the spots taken already are skipped, whatever takes them: a structure (a Storage
/// against side A, x 768..896), a unit of the enemy's (on the middle of side B), and the Recruit Eject All had just
/// let into the doorway (at the middle of side C), which stays where it is, out of a doorway that is gone. The nine
/// still inside come out at the first free spots clockwise from the middle of side C, longest inside first, none
/// overlapping anything (and `TestSim`'s invariants hold).
#[test]
fn evacuees_take_the_free_spots_around_what_stands_there() {
    let mut t = base()
        .spawn(ObjectKind::Storage, Some(P1), tile(12, 8)) // 2: x 768..896, y 512..640
        .start();
    t.without_auto_targeting();
    let recruits = block(&mut t, 1, 10, 5, Vec2::new(1150.0, 704.0)); // 3..=12
    let order = fill(&mut t, &recruits);
    let eject = t.sim().tick() + 1;
    let enemy = id(next_id(&t));
    t.spawn_unit_at(eject, RECRUIT, P2, Vec2::new(984.0, 704.0));
    t.at(eject, P1, eject_all(&[A1]));
    run_checked(&mut t, eject);
    let leaving = order[0];
    assert_eq!(exits(&t), [(eject, leaving)]);
    let at = t.sim().object(leaving).unwrap().pos;
    assert!(at.distance(EXIT) < 0.5, "in the doorway: {at}");
    let gone = eject + 1;
    remove_at(&mut t, gone, A1);
    run_checked(&mut t, gone);
    assert_eq!(evacuated(&t, A1), order[1..], "the nine still inside, in order");
    // Ring 0 clockwise from the middle of side C: its middle taken by the Recruit in the doorway, side A's spots
    // but its last two by the Storage, and side B's middle by the enemy.
    let spots = [
        (756.0, 680.0),
        (756.0, 656.0),
        (756.0, 628.0),
        (912.0, 628.0),
        (936.0, 628.0),
        (972.0, 628.0),
        (972.0, 656.0),
        (972.0, 680.0),
        (972.0, 728.0),
    ];
    for (&unit, (x, y)) in order[1..].iter().zip(spots) {
        let pos = t.sim().object(unit).expect("alive, on the map").pos;
        assert!(pos.distance(Vec2::new(x, y)) < 0.1, "{unit:?} at {pos}, not ({x}, {y})");
    }
    let mut all: Vec<ObjectId> = order.clone();
    all.push(enemy);
    apart(&t, &all);
    let o = t.sim().object(leaving).unwrap();
    assert_eq!((o.pos, o.mover.as_ref().unwrap().exit), (at, None));
    run_checked(&mut t, gone + 32);
}

/// Before the defeat check (`economy::defeat`, at the end of the economy phase): with the Armory, player 1's last
/// structure, goes the player, but its Recruits come out first, alive, and turn neutral with its other units
/// (`core/game_rules.md > On Defeat`) instead of being lost inside.
#[test]
fn evacuees_come_out_before_their_owner_is_defeated() {
    let mut t = base().spawn(ARMORY, Some(P2), tile(20, 4)).start(); // 2
    t.without_auto_targeting();
    let recruits = block(&mut t, 1, 3, 3, Vec2::new(1150.0, 704.0)); // 3..=5
    let order = fill(&mut t, &recruits);
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, A1);
    run_checked(&mut t, gone);
    assert!(t.sim().player(P1).is_some_and(|p| p.is_defeated()));
    assert_eq!(evacuated(&t, A1), order);
    for &unit in &order {
        let o = t.sim().object(unit).expect("alive, on the map");
        assert_eq!(o.owner, None, "neutral");
    }
    let lost = (t.events().iter()).any(|(_, e)| matches!(e, SimEvent::Removed(unit) if order.contains(unit)));
    assert!(!lost);
}
