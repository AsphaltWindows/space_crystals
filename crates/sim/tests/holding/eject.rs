//! Eject All (`factions/cults/structures/armory.md > Eject All`: "Immediate command that ejects all stored Recruits
//! from the exit side in a rapid stream, one after another"), through the Armory's exit doorway, M2's
//! (`core/entities.md > Structure Type > Exit Doorway`), on side C (`factions/cults/structures/armory.md > ExitSide -
//! opposite short side (C)`).

use super::*;

/// [`base`] with ten of player 1's Recruits inside its Armory, ordered in at tick 2, at the first tick they all are;
/// they are in the order they went in, longest inside first.
fn filled() -> (TestSim, Vec<ObjectId>) {
    let mut t = base().start();
    t.without_auto_targeting();
    let recruits = block(&mut t, 1, 10, 5, Vec2::new(1150.0, 704.0));
    t.at(2, P1, enter(&recruits, A1));
    run_checked(&mut t, 2);
    while inside(&t, A1).len() < 10 {
        assert!(t.sim().tick() < 200, "{} are in", inside(&t, A1).len());
        step_checked(&mut t);
    }
    let order = inside(&t, A1);
    (t, order)
}

/// The units standing in `building`'s exit doorway (`Mover::exit`).
fn in_doorway(t: &TestSim, building: ObjectId) -> Vec<ObjectId> {
    (t.sim().objects())
        .filter(|o| o.mover.as_ref().is_some_and(|mover| mover.exit == Some(building)))
        .map(|o| o.id)
        .collect()
}

/// The plan's acceptance (slice M5-F1): "Eject All streams the pool out of side C, one at a time." The first comes out
/// in the tick of the command, the others longest inside first, each "half inside the building, at the middle of the
/// exit side" (`core/entities.md > Structure Type > Exit Doorway` 1) once the one before it has walked clear of the
/// doorway (3: "the next unit can only appear once the doorway is free"): never two in the doorway, and a few ticks
/// apart. It comes out as far as there is room, at most half (M2's reading, `production/exit.rs`): further in while
/// the one before it still stands right in front of the doorway. With no order and no rally point (slice M5-F2's),
/// each walks straight out of side C (2), the ones in front stepping aside for it (4), and idles; the Armory is empty
/// and has room for ten again.
#[test]
fn eject_all_streams_the_pool_out_of_side_c_one_at_a_time() {
    let (mut t, order) = filled();
    let start = t.sim().tick() + 1;
    t.at(start, P1, eject_all(&[A1]));
    let mut out: Vec<(Tick, ObjectId, Vec2)> = Vec::new();
    while out.len() < 10 {
        assert!(
            t.sim().tick() < start + 200,
            "{} are out\n{}",
            out.len(),
            t.ascii(Some(EXIT))
        );
        step_checked(&mut t);
        let tick = t.sim().tick();
        let doorway = in_doorway(&t, A1);
        assert!(doorway.len() <= 1, "tick {tick}: {doorway:?} in the doorway");
        for &(_, unit) in exits(&t).iter().filter(|&&(at, _)| at == tick) {
            assert_eq!(doorway, [unit], "tick {tick}: it comes out into the doorway");
            out.push((tick, unit, t.sim().object(unit).expect("on the map").pos));
        }
    }
    let units: Vec<ObjectId> = out.iter().map(|&(_, unit, _)| unit).collect();
    assert_eq!(units, order, "longest inside first");
    assert_eq!(out[0].0, start, "the first at once");
    for &(tick, unit, pos) in &out {
        // In the doorway of side C (x = 768): from half inside to just inside, within the side's span.
        assert!(
            pos.x > EXIT.x - 0.5 && pos.x < EXIT.x + HALF.x + 0.5 && (pos.y - EXIT.y).abs() <= 64.0 - HALF.y,
            "{unit:?} came out at {pos} in tick {tick}"
        );
    }
    assert!(
        out[0].2.distance(EXIT) < 0.5,
        "the first half inside, at the middle: {}",
        out[0].2
    );
    let ticks: Vec<Tick> = out.iter().map(|&(tick, _, _)| tick).collect();
    let gaps: Vec<Tick> = ticks.windows(2).map(|w| w[1] - w[0]).collect();
    println!("out in ticks {ticks:?} (gaps {gaps:?})");
    // Each walks 24 su out at 4 su a tick, the one in front shouldered aside on the way (about 12 ticks a unit,
    // measured): never a hold of more than 1.25 s, and the ten out within 8 s.
    assert!(gaps.iter().all(|&gap| gap <= 20), "a stream, no jam: {gaps:?}");
    assert!(ticks[9] - ticks[0] <= 9 * 14, "one after another: {ticks:?}");
    run_for(&mut t, 64);
    assert!(inside(&t, A1).is_empty());
    assert_eq!(t.sim().room(A1), 10);
    for &unit in &order {
        let o = t.sim().object(unit).expect("on the map");
        assert!(o.orders.as_ref().unwrap().is_idle(), "{unit:?} idles");
        assert!(o.pos.x < EXIT.x, "{unit:?} out of side C, at {}", o.pos);
    }
    assert!(in_doorway(&t, A1).is_empty());
}

/// Eject All empties the pool as it was when the command came: a Recruit that goes in while the others stream out
/// stays inside (it wasn't stored then), and the next Eject All lets it out.
#[test]
fn a_recruit_that_goes_in_during_the_stream_stays() {
    let (mut t, order) = filled();
    let start = t.sim().tick() + 1;
    let late = id(next_id(&t));
    t.spawn_unit_at(start, RECRUIT, P1, Vec2::new(1060.0, 704.0));
    t.at(start, P1, eject_all(&[A1]));
    t.at(start + 1, P1, enter(&[late], A1));
    while exits(&t).len() < 10 {
        assert!(t.sim().tick() < start + 200, "{} are out", exits(&t).len());
        step_checked(&mut t);
    }
    let went_in = entries(&t)
        .iter()
        .find(|&&(_, unit)| unit == late)
        .map(|&(tick, _)| tick);
    let last_out = exits(&t)[9].0;
    assert!(
        went_in.is_some_and(|tick| tick < last_out),
        "in during the stream: {went_in:?}, {last_out}"
    );
    let ejected: Vec<ObjectId> = exits(&t).iter().map(|&(_, unit)| unit).collect();
    assert_eq!(ejected, order);
    run_for(&mut t, 16);
    assert_eq!(inside(&t, A1), [late]);
    let again = t.sim().tick() + 1;
    t.at(again, P1, eject_all(&[A1]));
    run_checked(&mut t, again);
    assert_eq!(exits(&t).last(), Some(&(again, late)));
}

/// The gate (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`: Eject All is "Only available if
/// StoredRecruits is not empty"): refused for an empty Armory, for objects none of which holds units, and for another
/// player's Armory; taken for the player's Armory with a Recruit inside, the other objects of the selection ignored.
#[test]
fn the_gate_takes_eject_all_only_with_someone_inside() {
    let mut t = base().spawn(ARMORY, Some(P2), tile(20, 4)).start(); // 2
    t.without_auto_targeting()
        .spawn_unit_at(1, RECRUIT, P1, Vec2::new(1100.0, 704.0)) // 3
        .spawn_unit_at(1, RECRUIT, P1, Vec2::new(1100.0, 800.0)); // 4
    run_checked(&mut t, 1);
    assert_eq!(validate(&t, eject_all(&[A1])), Err(Reject::NothingInside));
    assert_eq!(validate(&t, eject_all(&[id(3)])), Err(Reject::NoHolders));
    assert_eq!(validate(&t, eject_all(&[A1, id(2)])), Err(Reject::NotOwned(id(2))));
    t.at(2, P1, enter(&[id(3)], A1));
    run_checked(&mut t, 60);
    assert_eq!(inside(&t, A1), [id(3)]);
    assert_eq!(validate(&t, eject_all(&[A1])), Ok(()));
    assert_eq!(validate(&t, eject_all(&[id(4), A1])), Ok(()));
    assert_eq!(validate(&t, eject_all(&[id(4)])), Err(Reject::NoHolders));
}
