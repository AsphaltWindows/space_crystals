//! Eject All (`factions/cults/structures/garrison.md > Placeholder stats > Leaving`: "units leave **through the
//! door**; on **Eject All** they all come out **at once**, ignoring collision with each other until they are clear of
//! the footprint"; its `> Panel`'s "**A: Eject All** (everyone inside leaves)"), through the door on side B
//! (`ExitRule::AtOnce`).

use super::*;

/// The units standing in `building`'s doorway (`Mover::exit`).
fn in_doorway(t: &TestSim, building: ObjectId) -> Vec<ObjectId> {
    (t.sim().objects())
        .filter(|o| o.mover.as_ref().is_some_and(|mover| mover.exit == Some(building)))
        .map(|o| o.id)
        .collect()
}

/// Whether two of `units`' boxes overlap.
fn any_overlap(t: &TestSim, units: &[ObjectId]) -> bool {
    let pos: Vec<Vec2> = units
        .iter()
        .map(|&unit| t.sim().object(unit).expect("on the map").pos)
        .collect();
    (0..pos.len()).any(|i| {
        (i + 1..pos.len()).any(|j| {
            let d = (pos[i] - pos[j]).abs();
            d.x < 2.0 * HALF.x && d.y < 2.0 * HALF.y
        })
    })
}

/// Design change 0335's verification: "Four Militants in a Garrison: Eject All releases all four together from side
/// B." All four come out in the tick of the command, each at the door's one spot, "half inside the building, at the
/// middle of the exit side" (`core/entities.md > Structure Type > Exit Doorway` 1), facing out, in the doorway; they
/// pass through each other as they walk straight out of side B together (2: the Garrison has no rally point), and
/// none of them is ever deeper in. Once they are clear of the footprint they are idle, and they part, as idle units
/// that overlap do (`systems/units.md > UnitCollision > Units that already overlap > Idle units push apart`), until
/// none overlaps another. The Garrison is empty, and the gate takes no Eject All on it any more.
#[test]
fn eject_all_lets_all_four_out_through_the_door_at_once() {
    let mut t = base().start();
    t.without_auto_targeting();
    let pool = fill(&mut t, &[MILITANT; 4]);
    let now = t.sim().tick() + 1;
    t.at(now, P1, eject_all(&[G1]));
    run_checked(&mut t, now);
    let out: Vec<ObjectId> = exits(&t)
        .iter()
        .filter(|&&(at, _)| at == now)
        .map(|&(_, unit)| unit)
        .collect();
    assert_eq!(out, pool, "all four at once, in the pool's order");
    assert_eq!(inside(&t, G1), []);
    let mut sorted = pool.clone();
    sorted.sort();
    assert_eq!(in_doorway(&t, G1), sorted, "all in the doorway");
    for &unit in &pool {
        let o = t.sim().object(unit).expect("on the map");
        assert_eq!(o.pos, DOOR, "{unit:?} at the door's spot");
        assert!(
            (o.facing.abs() - std::f32::consts::PI).abs() < 1e-4,
            "facing west: {}",
            o.facing
        );
        assert!(o.orders.as_ref().unwrap().is_idle());
    }
    assert_eq!(validate(&t, eject_all(&[G1])), Err(Reject::NothingInside));
    // Out of the doorway together, through each other.
    let mut last: Vec<Vec2> = pool.iter().map(|_| DOOR).collect();
    while !in_doorway(&t, G1).is_empty() {
        assert!(t.sim().tick() < now + 20, "out by now\n{}", t.ascii(Some(DOOR)));
        step_checked(&mut t);
        for (i, &unit) in pool.iter().enumerate() {
            let pos = t.sim().object(unit).unwrap().pos;
            assert!(
                pos.x <= last[i].x && pos.y == DOOR.y,
                "{unit:?} straight out: {} to {pos}",
                last[i]
            );
            last[i] = pos;
        }
    }
    let clear = t.sim().tick();
    for &unit in &pool {
        let o = t.sim().object(unit).unwrap();
        assert!(!on_footprint(o.pos), "{unit:?} clear of the footprint: {}", o.pos);
        assert_eq!(o.pos.x, DOOR.x - HALF.x, "{unit:?} just clear of side B");
    }
    assert!(
        clear - now <= 4,
        "a step of 4 su a tick for 12 su: out in {} ticks",
        clear - now
    );
    assert!(any_overlap(&t, &pool), "still together as they come clear");
    while any_overlap(&t, &pool) {
        assert!(t.sim().tick() < clear + 120, "apart by now\n{}", t.ascii(Some(DOOR)));
        step_checked(&mut t);
        for &unit in &pool {
            let pos = t.sim().object(unit).unwrap().pos;
            assert!(!on_footprint(pos), "{unit:?} back on the footprint at {pos}");
        }
    }
    println!(
        "clear of the footprint {} ticks after the command, apart {} later",
        clear - now,
        t.sim().tick() - clear
    );
    for &unit in &pool {
        let o = t.sim().object(unit).unwrap();
        assert!(o.orders.as_ref().unwrap().is_idle(), "{unit:?} idles");
    }
}

/// "ignoring collision with each other": only with each other. A Militant of player 1's holding position right in
/// front of the door (Hold: it never steps aside, `systems/units.md > Ground Collision > Who yields`) leaves the four
/// room to come out only fully inside, against side B (M2's reading of Exit Doorway 1, `production/exit.rs`: as far
/// out as there is room); they come out all the same, at once, and stand in the doorway, up to the Militant and no
/// farther, until it goes. Then they walk out.
#[test]
fn units_let_out_at_once_still_collide_with_the_others() {
    let mut t = base().start();
    t.without_auto_targeting();
    let pool = fill(&mut t, &[RECRUIT, MILITANT, GUNNER, DISCIPLE]);
    let guard = id(next_id(&t));
    let now = t.sim().tick() + 1;
    t.spawn_unit_at(now, MILITANT, P1, DOOR - Vec2::new(HALF.x, 0.0));
    t.order_at(now + 1, P1, &[guard], OrderKind::Hold, false);
    t.at(now + 2, P1, eject_all(&[G1]));
    run_checked(&mut t, now + 2);
    assert_eq!(inside(&t, G1), [], "all out at once");
    for &unit in &pool {
        let pos = t.sim().object(unit).expect("on the map").pos;
        let inner = DOOR + Vec2::new(HALF.x, 0.0);
        assert!(
            pos.distance(inner) < 0.1,
            "{unit:?} fully inside, against side B: {pos}"
        );
    }
    run_for(&mut t, 30);
    let blocker = t.sim().object(guard).unwrap().pos;
    for &unit in &pool {
        let pos = t.sim().object(unit).unwrap().pos;
        assert!(on_footprint(pos), "{unit:?} still in the doorway: {pos}");
        assert!(
            pos.x - HALF.x >= blocker.x + HALF.x - 0.01,
            "{unit:?} up to the Militant: {pos}"
        );
    }
    let open = t.sim().tick() + 1;
    t.debug_at(open, DebugCommand::Remove { id: guard });
    run_for(&mut t, 30);
    for &unit in &pool {
        let pos = t.sim().object(unit).unwrap().pos;
        assert!(!on_footprint(pos), "{unit:?} out: {pos}");
    }
}

/// The gate refuses Eject All on an empty Garrison (F1's reading of the Armory's "Only available if StoredRecruits is
/// not empty", for every building that holds units, `Sim::check_holding`); with one Gunner inside, it lets the Gunner
/// out at the door.
#[test]
fn an_empty_garrison_refuses_eject_all() {
    let mut t = base().start();
    t.without_auto_targeting();
    assert_eq!(validate(&t, eject_all(&[G1])), Err(Reject::NothingInside));
    let [gunner] = fill(&mut t, &[GUNNER])[..] else {
        unreachable!()
    };
    let now = t.sim().tick() + 1;
    t.at(now, P1, eject_all(&[G1]));
    run_checked(&mut t, now);
    assert_eq!(exits(&t), [(now, gunner)]);
    assert_eq!(t.sim().object(gunner).expect("out").pos, DOOR);
}
