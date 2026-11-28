//! The landing slots (`factions/gdo/structures/airfield.md > Landing slots`: "An Airfield has **4 landing slots**, one
//! per possible assignment, so all of its aircraft can be landed at the same time"; "Landing and take-off: **16
//! frames (1 s)** each (placeholder)") and the flight home that ends there (`> Returning and rearming`: "**Return to
//! Base** command: sends the aircraft to its assigned Airfield to land and rearm (hotkey **F**, placeholder).
//! Right-clicking its own Airfield does the same").

use super::*;

/// The plan's acceptance, "Four Hornets out of ammo land on the 4 pads": four Hornets assigned to [`A`], landed there,
/// fire their 5 missiles at the ground (each lifting off as its Attack Ground starts), fly home by themselves
/// (`> Returning and rearming`: "When an aircraft runs out of ammo, it automatically flies back to its assigned
/// Airfield, lands and rearms"), and land on the 4 slots, one each, where they rearm and stay.
#[test]
fn four_hornets_out_of_ammo_land_on_the_four_slots() {
    let mut t = field(|builder| builder);
    let points: Vec<Vec2> = (0..4).map(|i| Vec2::new(700.0 + 110.0 * i as f32, 760.0)).collect();
    let hornets = landed_hornets(&mut t, 7, &points, A);
    let mut slots: Vec<Option<usize>> = hornets
        .iter()
        .map(|&n| slot_of(t.sim().object(id(n)).unwrap().pos))
        .collect();
    slots.sort();
    assert_eq!(slots, [Some(0), Some(1), Some(2), Some(3)], "a slot each");

    let spot = Vec2::new(864.0, 1100.0);
    next(&mut t, P1, order(&hornets, OrderKind::AttackGround(spot)));
    for &n in &hornets {
        assert_eq!(landing_of(t.sim(), n), None, "#{n} lifted off");
    }
    step_until(&mut t, 600, "out of ammo", |sim| {
        hornets.iter().all(|&n| ammo(sim, n) == 0)
    });
    for &n in &hornets {
        assert_eq!(
            doing(t.sim(), n),
            Some(OrderKind::ReturnToBase),
            "#{n} flies home by itself"
        );
    }
    step_until(&mut t, 800, "home, rearmed", |sim| {
        hornets.iter().all(|&n| home(sim, n))
    });
    let mut slots: Vec<Option<usize>> = hornets.iter().map(|&n| slot_of(t.pos(n))).collect();
    slots.sort();
    assert_eq!(slots, [Some(0), Some(1), Some(2), Some(3)], "on the 4 slots, one each");
    t.run(64);
    assert!(hornets.iter().all(|&n| home(t.sim(), n)), "they stay landed");
}

/// "Landing and take-off: **16 frames (1 s)** each": a Hornet assigned to [`A`] flies to its first slot, comes to rest
/// over it (plan decision T88: "a HoverCraft brakes to a stop over its pad"), and lands there in 16 ticks, holding
/// still; its flight home ends once it is down (it is full).
#[test]
fn a_hornet_lands_on_its_slot_in_16_ticks_holding_still() {
    let mut t = field(|builder| builder);
    t.spawn_unit_at(2, HORNET, P1, Vec2::new(864.0, 760.0));
    t.step();
    next(&mut t, P1, order(&[7], OrderKind::Rebase(id(A))));
    let mut prev = None;
    let until = t.sim().tick() + 200;
    let began = loop {
        if landing_of(t.sim(), 7).is_some() {
            break t.sim().tick();
        }
        prev = Some(t.pos(7));
        assert!(t.sim().tick() < until, "landing");
        t.step();
    };
    let at = t.pos(7);
    assert_eq!(slot_of(at), Some(0), "over its slot: {at}");
    assert_eq!(prev, Some(at), "at rest over it as its landing begins");
    for frame in 1..=LANDING {
        assert!(!down(t.sim(), 7), "frame {frame}: landing");
        assert_eq!(doing(t.sim(), 7), Some(OrderKind::Rebase(id(A))));
        t.step();
        assert_eq!(t.pos(7), at, "it holds still");
    }
    assert_eq!(t.sim().tick(), began + LANDING);
    assert!(down(t.sim(), 7), "down, 16 ticks on");
    t.step();
    assert!(home(t.sim(), 7), "its flight home over");
    assert_eq!(t.pos(7), at);
}

/// "**Return to Base** command [...] (hotkey **F**, placeholder)" from anywhere: a Hornet of [`A`] with 3 missiles
/// left, far off on the map, flies home on its F, lands on its slot and rearms. A right-click on its own Airfield
/// gives it the same order. Given to it there, landed, the order lifts it off no more: it stays, the order over.
#[test]
fn f_from_anywhere_flies_it_home_to_land_and_rearm_and_so_does_a_right_click() {
    let mut t = field(|builder| builder);
    landed_hornets(&mut t, 7, &[Vec2::new(864.0, 760.0)], A);
    next(
        &mut t,
        P1,
        order(&[7], OrderKind::AttackGround(Vec2::new(864.0, 900.0))),
    );
    until_launched(&mut t, 7, 2, 300);
    let far = Vec2::new(3200.0, 2200.0);
    next(&mut t, P1, order(&[7], OrderKind::Move(Target::Point(far))));
    step_until(&mut t, 600, "far off", |sim| {
        let o = sim.object(id(7)).unwrap();
        o.pos.distance(far) < 2.0 && doing(sim, 7).is_none()
    });
    assert_eq!(ammo(t.sim(), 7), AMMO - 2);
    next(&mut t, P1, order(&[7], OrderKind::ReturnToBase));
    until_down(&mut t, 7, 600);
    step_until(&mut t, 4 * REARM, "rearmed", |sim| home(sim, 7));
    assert_eq!(slot_of(t.pos(7)), Some(0), "on its slot");

    // A right-click on its own Airfield: its Return to Base (`Sim::default_order`).
    next(&mut t, P1, order(&[7], OrderKind::Move(Target::Point(far))));
    step_until(&mut t, 600, "far off", |sim| {
        sim.object(id(7)).unwrap().pos.distance(far) < 2.0
    });
    let click = right_click(t.sim(), &[7], Target::Object(id(A)));
    assert_eq!(click, order(&[7], OrderKind::ReturnToBase));
    next(&mut t, P1, click);
    until_down(&mut t, 7, 600);
    step_until(&mut t, 2, "home", |sim| home(sim, 7));

    // Landed there already: it stays, and the order is over.
    let at = t.pos(7);
    next(&mut t, P1, order(&[7], OrderKind::ReturnToBase));
    assert!(down(t.sim(), 7), "it doesn't lift off");
    t.step();
    assert!(home(t.sim(), 7));
    assert_eq!(t.pos(7), at);
}

/// (M6-GH2) Out of the air units' separation (`movement::air`: **Placeholder (not in the design: the separation of an
/// aircraft on its slot)**): a Hornet down on [`A`]'s first slot stays on it, still, while another flies straight over
/// it, 10 su off its centre, well within their SeparationRadius of 56 su; and the one flying over isn't pushed off its
/// line by it either.
#[test]
fn a_hornet_down_on_its_slot_stays_there_while_another_flies_over_it() {
    let mut t = field(|builder| builder);
    landed_hornets(&mut t, 7, &[Vec2::new(864.0, 760.0)], A);
    let at = t.pos(7);
    assert_eq!(slot_of(at), Some(0));
    let line = at.y + 10.0;
    t.spawn_unit_at(t.sim().tick() + 1, HORNET, P1, Vec2::new(at.x - 400.0, line));
    t.step();
    let to = Vec2::new(at.x + 400.0, line);
    next(&mut t, P1, order(&[8], OrderKind::Move(Target::Point(to))));
    let mut nearest = f32::MAX;
    for _ in 0..150 {
        t.step();
        let tick = t.sim().tick();
        assert_eq!(t.pos(7), at, "tick {tick}: on its slot");
        assert!(down(t.sim(), 7), "tick {tick}: down");
        assert!(
            (t.pos(8).y - line).abs() < 1e-3,
            "tick {tick}: on its line, {}",
            t.pos(8)
        );
        nearest = nearest.min(t.pos(8).distance(at));
    }
    assert!(nearest < 12.0, "it flew over: {nearest} su off at the nearest");
    assert!(t.pos(8).x > at.x + 100.0, "and on: {}", t.pos(8));
}
