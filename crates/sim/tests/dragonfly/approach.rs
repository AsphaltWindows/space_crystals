//! The Dragonfly's landing (design change 0428 point 5; `systems/mechanics/aircraft_landing.md > Vertical`): "**The
//! Dragonfly** (a Glider) lands and takes off **vertically**, as the Specter at an air door: it approaches its slot at
//! IdleSpeed, stops over it while it lands (16 frames), and takes off straight up, leaving at IdleSpeed. Landed, it
//! doesn't move" (`movement::approach`); and the plan's acceptance, "It never stops in the air. It lands from any
//! heading". **Placeholder (Q0192 point 2)**, the designer's (`systems/mechanics/aircraft_landing.md > Vertical`: "it
//! slows from MaxSpeed to IdleSpeed 4 GU before the spot, lines up by flying straight on [...] and lands facing the way
//! it came"): it flies the last 4 grid units at IdleSpeed (`Air::approach_gu`), and its arrival faces the way it came
//! in.

use super::*;

/// The sector of the compass that `facing` points into, of eight.
fn octant(facing: f32) -> i32 {
    ((math::wrap_angle(facing) / (math::PI / 4.0)).round() as i32).rem_euclid(8)
}

/// One landing of a Dragonfly (7) on [`B`]'s first slot: spawned at `start` (facing east, as a spawn does), flown on a
/// Move to `first` for `ticks` ticks if it has one, then rebased onto [`B`] by a right-click: it lands on the slot, the
/// last 8 ticks of its approach at least at IdleSpeed ([`lands_on`]), never stopping in the air ([`fly_down`]). Its
/// arrival's frame.
fn lands_from(start: Vec2, first: Option<(Vec2, Tick)>) -> Frame {
    let mut t = field(|builder| builder);
    t.spawn_unit_at(2, DRAGONFLY, P1, start);
    t.step();
    let n = 7;
    if let Some((to, ticks)) = first {
        next(&mut t, P1, move_to(&[n], to));
        for _ in 0..ticks {
            t.step();
            flies_or_is_held(frame(t.sim(), n));
        }
    }
    let click = right_click(t.sim(), &[n], Target::Object(id(B)));
    next(&mut t, P1, click);
    assert_eq!(doing(t.sim(), n), Some(OrderKind::Rebase(id(B))));
    let frames = fly_down(&mut t, n, 600);
    lands_on(&frames, B_SLOTS[0], 8)
}

/// "It lands from any heading": a Dragonfly starting in each of eight directions from [`B`]'s first slot, near (inside
/// its turn at MaxSpeed) and far, facing east from its spawn; and four already at MaxSpeed on a Move, away from the
/// slot, across it and past it. Each lands on its slot, never stopping in the air, the last of its approach at
/// IdleSpeed, and comes in on whatever heading its flight gave it: from every quarter of the compass among them.
#[test]
fn it_lands_from_any_heading_never_stopping_in_the_air() {
    let slot = B_SLOTS[0];
    let mut octants = std::collections::BTreeSet::new();
    for distance in [150.0, 600.0] {
        for i in 0..8 {
            let bearing = i as f32 * math::PI / 4.0;
            let arrival = lands_from(slot + math::facing_vector(bearing) * distance, None);
            octants.insert(octant(arrival.facing));
        }
    }
    let moving = [
        (Vec2::new(816.0, 1300.0), Vec2::new(816.0, 2400.0)),
        (Vec2::new(300.0, 900.0), Vec2::new(2400.0, 900.0)),
        (Vec2::new(1400.0, 700.0), Vec2::new(200.0, 700.0)),
        (Vec2::new(816.0, 400.0), Vec2::new(816.0, 2400.0)),
    ];
    for (start, to) in moving {
        let arrival = lands_from(start, Some((to, 40)));
        octants.insert(octant(arrival.facing));
    }
    assert!(octants.len() >= 6, "from many headings: {octants:?}");
    for quarter in [0, 2, 4, 6] {
        let near = [quarter, (quarter + 1) % 8, (quarter + 7) % 8];
        assert!(near.iter().any(|o| octants.contains(o)), "{quarter}: {octants:?}");
    }
}

/// Three Dragonflies rebased onto [`A`] at once land side by side, each on a slot of its own in id order, pushed by
/// none of the others' separation once there, none stopping in the air on the way ([`landed_dragonflies`]).
#[test]
fn three_at_once_land_side_by_side() {
    let mut t = field(|builder| builder);
    let points = [
        Vec2::new(760.0, 760.0),
        Vec2::new(864.0, 760.0),
        Vec2::new(968.0, 760.0),
    ];
    let ids = landed_dragonflies(&mut t, 7, &points, A);
    for (&n, slot) in ids.iter().zip(A_SLOTS) {
        assert!(t.pos(n).distance(slot) <= 8.0, "#{n} at {}, its slot {slot}", t.pos(n));
    }
    let held: Vec<Vec2> = ids.iter().map(|&n| t.pos(n)).collect();
    t.run(64);
    let after: Vec<Vec2> = ids.iter().map(|&n| t.pos(n)).collect();
    assert_eq!(after, held, "landed, they don't move");
}

/// **Placeholder (Q0192 point 2)**, the designer's (`systems/mechanics/aircraft_landing.md > Vertical`: "**No free
/// spot** at its base: a Glider circles over the structure's centre until one frees"): a Dragonfly flying home to an
/// Airfield whose slots are all taken flies on to its centre, as a Glider flies to any goal (`movement::glider`), and
/// takes the first slot freed, flying its approach to it from there. Four are landed on [`A`]; one is rebased onto
/// [`B`], which lifts it off, and in the same tick a fifth is rebased onto [`A`]: until the first has risen off its
/// slot, which it keeps meanwhile (`sc_sim::landing`), every slot is taken, and the fifth flies at [`A`]'s centre; then
/// it takes that slot, and lands there.
#[test]
fn with_no_slot_free_it_flies_on_to_the_centre_until_one_is() {
    let mut t = field(|builder| builder);
    let points = [
        Vec2::new(700.0, 760.0),
        Vec2::new(800.0, 760.0),
        Vec2::new(900.0, 760.0),
        Vec2::new(1000.0, 760.0),
    ];
    let ids = landed_dragonflies(&mut t, 7, &points, A);
    let leaving = ids[3];
    let freed = t.pos(leaving);
    t.spawn_unit_at(t.sim().tick() + 1, DRAGONFLY, P1, Vec2::new(864.0, 640.0));
    t.step();
    let fifth = 11;
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, order(&[leaving], OrderKind::Rebase(id(B))));
    t.at(tick, P1, order(&[fifth], OrderKind::Rebase(id(A))));
    t.step();
    assert_eq!((base(t.sim(), leaving), base(t.sim(), fifth)), (Some(B), Some(A)));
    let mut waited = 0;
    while mover(t.sim(), leaving).rise.is_some() {
        let m = mover(t.sim(), fifth);
        assert_eq!(m.goal, sc_sim::MoveGoal::Point(A_AT), "no slot free: on to the centre");
        assert!(!m.is_over());
        waited += 1;
        t.step();
        flies_or_is_held(frame(t.sim(), fifth));
    }
    assert_eq!(waited, LANDING - 1, "while the other rises");
    let frames = fly_down(&mut t, fifth, 600);
    let arrival = frames.iter().find(|f| f.over).expect("its approach");
    assert!(arrival.pos.distance(freed) <= 8.0, "on the slot freed: {arrival:?}");
}
