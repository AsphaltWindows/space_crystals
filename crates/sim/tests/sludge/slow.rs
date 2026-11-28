//! The slow (`factions/syndicate/units/shadebringer.md > Sludge (toggle)`: "**Infantry and non-hover vehicles** that
//! step into sludge are **slowed by Y%** for **Z seconds**"; "Not affected: hover units (HoverVehicle, HoverCraft) and
//! air units"; "**Affects everyone**"; "if it drives back over sludge (its own or another Shadebringer's), it is slowed
//! like any other vehicle"; `> Placeholder stats`: "units are slowed by **Y = 40%** for **Z = 2 s** after leaving it,
//! and the slow **refreshes while they stay in it**"; design change 0425: "a unit is in sludge [...] when its
//! **centre** is inside the trail").

use super::*;

/// [`field`] with player 2 playing `faction`, and player 1's Shadebringer (1) at (320, 800) laying its trail east from
/// tick 2, to (1700, 800): x 293 to 1544, laid from tick 2 to tick 161, each patch 10 s on the ground (`trail`).
fn trail(faction: Faction) -> ScenarioBuilder {
    field_with(faction).spawn(SHADEBRINGER, Some(P1), point(320.0, 800.0))
}

/// [`trail`] started: the Shadebringer's Sludge on and its Move in tick 2.
fn laying(scenario: ScenarioBuilder) -> TestSim {
    let mut t = started(scenario);
    t.at(2, P1, press(&[id(1)], sludge(true)));
    t.at(2, P1, order(&[id(1)], move_to(1700.0, 800.0)));
    t
}

/// A unit's ticks, from the next one until `until`: whether its centre was in sludge where the tick before left it
/// (what the tick's slow reads), when the slow it holds at the tick's end ends (`Buff::until`), and its move.
fn ticks(t: &mut TestSim, unit: ObjectId, until: Tick) -> Vec<(Tick, bool, Option<Tick>, f32)> {
    let mut ticks = Vec::new();
    while t.sim().tick() < until {
        let was_in = in_sludge(t, object(t, unit).pos);
        t.step();
        ticks.push((t.sim().tick(), was_in, slow_until(t, unit), speed(t, unit)));
    }
    ticks
}

/// The plan's acceptance, "a Peacekeeper crossing the trail moves at 0.6 × its speed while in it and for 2 s after":
/// player 2's Peacekeeper, walking south across the trail at its MaxSpeed of 4 su a tick, holds the slow from the tick
/// whose start finds its centre in the sludge, given again in every such tick, until 33 ticks after the last
/// (`Buff::until`), the last tick it works in the 32nd after it (a buff is dropped at the end of its last tick): in
/// those ticks it walks 2.4 su a tick, and 4 again from the tick after.
#[test]
fn a_peacekeeper_crossing_the_trail_moves_at_0_6_of_its_speed_in_it_and_for_2_s_after() {
    let mut t = laying(trail(Faction::Gdo).spawn(ObjectKind::Peacekeeper, Some(P2), point(1000.0, 500.0)));
    let pk = id(2);
    t.at(60, P2, order(&[pk], move_to(1000.0, 1100.0)));
    run_until(&mut t, 60);
    let mut last_in: Option<Tick> = None;
    let (mut inside, mut slowed) = (0, 0);
    for (tick, was_in, until, speed) in ticks(&mut t, pk, 200) {
        if was_in {
            last_in = Some(tick);
            inside += 1;
        }
        let holds = last_in.is_some_and(|last| tick <= last + AFTER);
        let kept = last_in.filter(|last| tick < last + AFTER).map(|last| last + AFTER + 1);
        assert_eq!(until, kept, "tick {tick}");
        let top = if holds { 4.0 * SLOWED } else { 4.0 };
        assert!((speed - top).abs() < 1e-4, "tick {tick}: {speed} su, not {top}");
        slowed += u32::from(holds);
    }
    assert!(inside >= 15, "{inside} ticks in it");
    assert_eq!(slowed, inside + AFTER, "and 2 s after");
    assert_eq!(slow_until(&t, pk), None);
}

/// "If it drives back over sludge (its own or another Shadebringer's), it is slowed like any other vehicle": the
/// Shadebringer drives east to (1000, 800) laying its trail, its Sludge switched off once it has parked, and backs up
/// over its trail (`OrderKind::Reverse`) to (700, 800). From the tick its centre is in the trail it holds the slow, and
/// backs up at 1.8 su a tick at most, 0.6 of its ReverseMaxSpeed of 3; past the patches that have gone, and 2 s on, at
/// 3. In the same game with its Sludge never on it backs up at 3 all the way.
#[test]
fn the_shadebringer_is_slowed_backing_up_over_its_own_trail() {
    let run = |sludge_on: bool| {
        let mut t = started(field().spawn(SHADEBRINGER, Some(P1), point(320.0, 800.0)));
        let sb = id(1);
        if sludge_on {
            t.at(2, P1, press(&[sb], sludge(true)));
            t.at(100, P1, press(&[sb], sludge(false)));
        }
        t.at(2, P1, order(&[sb], move_to(1000.0, 800.0)));
        t.at(101, P1, order(&[sb], OrderKind::Reverse(Vec2::new(700.0, 800.0))));
        run_until(&mut t, 100);
        assert_eq!(object(&t, sb).pos, Vec2::new(1000.0, 800.0), "parked");
        ticks(&mut t, sb, 300)
    };
    let mut slowed = 0;
    let mut held = false;
    for (tick, _, until, speed) in run(true) {
        // Slowed in a tick that ends its slow too.
        let top = if until.is_some() || held {
            REVERSE_TOP * SLOWED
        } else {
            REVERSE_TOP
        };
        assert!(speed <= top + 1e-4, "tick {tick}: {speed} su");
        slowed += u32::from(until.is_some());
        held = until.is_some();
    }
    assert!(slowed >= 80, "slowed for {slowed} ticks");
    let unslowed = run(false);
    assert!(unslowed.iter().all(|&(_, _, until, _)| until.is_none()));
    let top = unslowed.iter().map(|&(_, _, _, speed)| speed).fold(0.0, f32::max);
    assert!((top - REVERSE_TOP).abs() < 1e-4, "{top}");
}

/// A Shadebringer crossing another's trail at its ForwardMaxSpeed of 8 su a tick (`movement/wheeled.rs`: "never above
/// ForwardMaxSpeed"; "towards the speed the unit wants ... by at most ... Deceleration down"): from the tick its centre
/// is in the trail it holds the slow, and brakes by its Deceleration of 1 su a tick, to 4.8 su, 0.6 of its top speed,
/// which it keeps until 2 s after it left the trail; then it speeds up to 8 again.
#[test]
fn a_shadebringer_crossing_another_s_trail_brakes_to_0_6_of_its_top_speed_and_keeps_it_2_s() {
    let mut t = laying(trail(Faction::Gdo).spawn(SHADEBRINGER, Some(P1), point(600.0, 300.0)));
    let crosser = id(2);
    t.at(60, P1, order(&[crosser], move_to(600.0, 1300.0)));
    run_until(&mut t, 60);
    let ticks = ticks(&mut t, crosser, 260);
    let slowed: Vec<&(Tick, bool, Option<Tick>, f32)> = ticks.iter().filter(|tick| tick.2.is_some()).collect();
    let (first, last) = (slowed[0].0, slowed.last().unwrap().0);
    assert_eq!(slowed.len() as Tick, last - first + 1, "one run of ticks");
    let entered = ticks.iter().position(|tick| tick.0 == first).unwrap();
    assert!(
        (ticks[entered - 1].3 - TOP).abs() < 1e-4,
        "at full speed before: {:?}",
        ticks[entered - 1]
    );
    for (i, &(tick, _, _, speed)) in ticks[entered..].iter().take_while(|tick| tick.2.is_some()).enumerate() {
        let braking = (TOP - 1.0 - i as f32).max(TOP * SLOWED);
        assert!((speed - braking).abs() < 1e-3, "tick {tick}: {speed} su, not {braking}");
    }
    let after = &ticks[entered + slowed.len()..];
    assert!(after.iter().any(|tick| (tick.3 - TOP).abs() < 1e-4), "full speed again");
    assert!(slowed.len() as Tick > AFTER + 4, "{} ticks", slowed.len());
}

/// "Not affected: hover units (HoverVehicle, HoverCraft) and air units": player 2's Drone (the Colonists', a
/// HoverVehicle) and player 1's Phantom (a HoverCraft, an air unit), crossing the trail, never hold the slow, and move
/// tick by tick as in the same game with the Shadebringer's Sludge never on, their centres in the sludge all the same.
#[test]
fn hover_units_and_aircraft_cross_the_trail_unslowed() {
    let run = |sludge_on: bool| {
        let scenario = trail(Faction::Colonists)
            .spawn(ObjectKind::Drone, Some(P2), point(900.0, 500.0))
            .spawn(ObjectKind::Phantom, Some(P1), point(500.0, 500.0));
        let mut t = started(scenario);
        if sludge_on {
            t.at(2, P1, press(&[id(1)], sludge(true)));
        }
        t.at(2, P1, order(&[id(1)], move_to(1700.0, 800.0)));
        t.at(60, P2, order(&[id(2)], move_to(900.0, 1100.0)));
        t.at(60, P1, order(&[id(3)], move_to(500.0, 1100.0)));
        let mut seen = Vec::new();
        let mut in_it = [false; 2];
        while t.sim().tick() < 240 {
            t.step();
            for (k, unit) in [id(2), id(3)].into_iter().enumerate() {
                assert_eq!(slow_until(&t, unit), None, "tick {}: {unit:?}", t.sim().tick());
                in_it[k] |= in_sludge(&t, object(&t, unit).pos);
            }
            seen.push((object(&t, id(2)).pos, object(&t, id(3)).pos));
        }
        (seen, in_it)
    };
    let (with, in_it) = run(true);
    assert_eq!(in_it, [true, true], "both crossed the trail");
    let (without, _) = run(false);
    assert_eq!(with, without);
}

/// Design change 0425's Verification, "A 36x36 unit overlapping the sludge's edge but with its centre outside isn't
/// slowed": a Guard (36 × 36) walking east along the trail 30 su from its line, its box reaching 12 su into the discs
/// but its centre outside them, walks at its MaxSpeed of 5 su a tick, never slowed; another, 18 su from the line, its
/// centre in the trail all along (the discs 24 to 32 su apart, 48 across), holds the slow and walks at 3.
#[test]
fn a_guard_whose_box_reaches_into_the_trail_but_whose_centre_stays_out_is_not_slowed() {
    let mut t = laying(trail(Faction::Gdo));
    let (out, inside) = (id(2), id(3));
    t.debug_at(100, spawn(ObjectKind::Guard, P1, point(500.0, 830.0)));
    t.at(101, P1, order(&[out], move_to(800.0, 830.0)));
    t.debug_at(160, spawn(ObjectKind::Guard, P1, point(1100.0, 818.0)));
    t.at(161, P1, order(&[inside], move_to(1400.0, 818.0)));
    run_until(&mut t, 101);
    let walk = ticks(&mut t, out, 159);
    for &(tick, was_in, until, speed) in &walk {
        assert!(!was_in && until.is_none(), "tick {tick}: out of it");
        assert!((speed - 5.0).abs() < 1e-4, "tick {tick}: {speed} su");
    }
    let box_top = object(&t, out).pos.y - 18.0;
    assert!(box_top < 800.0 + RADIUS, "its box in the discs");
    run_until(&mut t, 162);
    for (tick, was_in, until, speed) in ticks(&mut t, inside, 230) {
        assert!(was_in && until == Some(tick + AFTER + 1), "tick {tick}: in it");
        assert!((speed - 5.0 * SLOWED).abs() < 1e-4, "tick {tick}: {speed} su");
    }
}
