//! The Sludge's toggle (`factions/syndicate/units/shadebringer.md > Charges and recharging`: "**Each charge is a
//! tank** that **drains while its toggle is on** ... When a tank runs **empty**, that ability switches off"; `>
//! Placeholder stats`: "a toggle can be switched on with **any charge** left in its tank", "Toggles follow the
//! mode-switch rule (keep orders, not queueable)"; design change 0425: "a Shadebringer with a toggle on that isn't
//! moving keeps laying its effect **where it stands** (and keeps draining)", "an empty tank switches its toggle off").

use super::*;

/// A Shadebringer of player 1 at (320, 800), facing east, as [`started`] leaves it in tick 1: id 1.
fn parked() -> TestSim {
    let mut t = started(field().spawn(SHADEBRINGER, Some(P1), point(320.0, 800.0)));
    run_until(&mut t, 1);
    t
}

/// Design change 0425's Verification, "Park a Shadebringer with Cover on: smoke keeps appearing on its spot until the
/// tank empties, then the toggle turns off by itself and the smoke clears 12 s later", with its Sludge (Cover is slice
/// M6-SC's): switched on in tick 2, the parked Shadebringer lays its patch at its rear edge, where it stands, and lays
/// it again every tick, its 10 s starting over, while its tank drains 10 a second, a 160th a tick; in the 160th tick,
/// 10 s on, the tank is empty and the toggle goes off by itself; its patch lasts 10 s more and is gone in the tick
/// after, and the tank, empty, drains no more. A press to switch it on again is refused ([`Reject::TankEmpty`]), and,
/// sent anyway, changes nothing.
#[test]
fn a_parked_shadebringer_lays_where_it_stands_until_its_tank_is_empty_and_its_patch_lasts_10_s_more() {
    let mut t = parked();
    let sb = id(1);
    assert_eq!(gate(&t, P1, press(&[sb], sludge(true))), Ok(()));
    t.at(2, P1, press(&[sb], sludge(true)));
    let spot = Vec2::new(320.0 - HALF_LENGTH, 800.0);
    for tick in 2..2 + FULL_USE {
        t.step();
        let tank = tank(&t, sb);
        let empty = tick == 1 + FULL_USE;
        assert_eq!((tank.used, tank.on), ((tick - 1) * DRAIN, !empty), "tick {tick}");
        let [patch] = laid_by(&t, sb)[..] else {
            panic!("tick {tick}: one patch, {:?}", laid_by(&t, sb));
        };
        assert_eq!(
            (patch.centre, patch.radius, patch.until),
            (spot, RADIUS, tick + LIFE),
            "tick {tick}"
        );
        assert_eq!(object(&t, sb).pos, Vec2::new(320.0, 800.0), "parked");
    }
    let off = 1 + FULL_USE;
    assert_eq!(
        (t.sim().tick(), tank(&t, sb).used, tank(&t, sb).laid),
        (off, CAPACITY, None)
    );
    while t.sim().tick() < off + LIFE - 1 {
        t.step();
        assert_eq!(laid_by(&t, sb).len(), 1, "tick {}: its patch", t.sim().tick());
        assert_eq!((tank(&t, sb).used, tank(&t, sb).on), (CAPACITY, false));
    }
    t.step();
    assert_eq!(laid_by(&t, sb), [], "gone 10 s after it was last laid");
    assert!(t.sim().patches().is_empty());
    assert_eq!(gate(&t, P1, press(&[sb], sludge(true))), Err(Reject::TankEmpty));
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, press(&[sb], sludge(true)));
    run_until(&mut t, tick + 1);
    assert_eq!(
        (tank(&t, sb).on, laid_by(&t, sb)),
        (false, vec![]),
        "refused, nothing laid"
    );
}

/// "A toggle can be switched on with **any charge** left in its tank" (design change 0369), and switched off at any
/// press: switched on in tick 2 and off in tick 159, the tank has 3 ticks of its charge left, with which the press
/// switches it on again, where it stands: its patch there is laid again, not a second; 3 ticks on it is empty, and off.
/// A press to switch it on goes to the units of a group with a charge left, refused for the group if none has
/// ([`Reject::TankEmpty`]); one to switch it off goes to every unit with the tank, empty or not. Neither goes to a unit
/// without the tank ([`Reject::LacksAbility`]: a Guard), and nor does the Cover's (F), which no unit has until slice
/// M6-SC.
#[test]
fn the_toggle_switches_on_with_any_charge_left_and_off_at_any_press() {
    let mut t = started(
        field()
            .spawn(SHADEBRINGER, Some(P1), point(320.0, 800.0))
            .spawn(SHADEBRINGER, Some(P1), point(320.0, 1100.0))
            .spawn(ObjectKind::Guard, Some(P1), point(600.0, 800.0)),
    );
    let (sb, full, guard) = (id(1), id(2), id(3));
    t.at(2, P1, press(&[sb], sludge(true)));
    t.at(159, P1, press(&[sb], sludge(false)));
    run_until(&mut t, 159);
    let left = CAPACITY - 157 * DRAIN;
    assert_eq!(
        (tank(&t, sb).on, tank(&t, sb).laid, CAPACITY - tank(&t, sb).used),
        (false, None, left)
    );
    assert_eq!(left, 3 * DRAIN, "3 ticks of charge");
    assert_eq!(gate(&t, P1, press(&[sb], sludge(true))), Ok(()), "any charge");
    t.at(170, P1, press(&[sb], sludge(true)));
    run_until(&mut t, 170);
    assert!(tank(&t, sb).on, "on again");
    let [patch] = laid_by(&t, sb)[..] else {
        panic!("one patch: {:?}", laid_by(&t, sb));
    };
    assert_eq!(patch.until, 170 + LIFE, "laid again where it stands");
    run_until(&mut t, 172);
    assert_eq!((tank(&t, sb).on, tank(&t, sb).used), (false, CAPACITY), "empty, off");
    assert_eq!(laid_by(&t, sb).len(), 1);

    assert_eq!(gate(&t, P1, press(&[sb], sludge(true))), Err(Reject::TankEmpty));
    assert_eq!(
        gate(&t, P1, press(&[sb, full], sludge(true))),
        Ok(()),
        "the charged one"
    );
    assert_eq!(gate(&t, P1, press(&[sb], sludge(false))), Ok(()), "off, empty or not");
    t.at(173, P1, press(&[sb, full], sludge(true)));
    run_until(&mut t, 173);
    assert_eq!((tank(&t, sb).on, tank(&t, full).on), (false, true));
    t.at(174, P1, press(&[sb, full], sludge(false)));
    run_until(&mut t, 174);
    assert_eq!(
        (tank(&t, full).on, tank(&t, full).used),
        (false, DRAIN),
        "off in its second tick"
    );
    for ability in [sludge(true), sludge(false)] {
        assert_eq!(
            gate(&t, P1, press(&[guard], ability)),
            Err(Reject::LacksAbility),
            "{ability:?}"
        );
    }
    for on in [true, false] {
        let cover = Ability::Cover { on };
        assert_eq!(
            gate(&t, P1, press(&[full], cover)),
            Err(Reject::LacksAbility),
            "slice M6-SC's"
        );
    }
}

/// "Toggles follow the mode-switch rule (keep orders, not queueable)": a Shadebringer driving east on its Move,
/// switched on and later off, keeps its order and its way all the while, laying its trail between the two presses and
/// none after; it arrives at its goal.
#[test]
fn the_toggle_keeps_the_shadebringer_s_orders() {
    let mut t = started(field().spawn(SHADEBRINGER, Some(P1), point(320.0, 800.0)));
    let sb = id(1);
    let goal = Vec2::new(1300.0, 800.0);
    t.at(2, P1, order(&[sb], move_to(goal.x, goal.y)));
    t.at(20, P1, press(&[sb], sludge(true)));
    t.at(60, P1, press(&[sb], sludge(false)));
    let current = |t: &TestSim| {
        let orders = object(t, sb).orders.as_ref().expect("a unit's orders");
        (orders.current.as_ref()).map(|active| (active.order.kind, active.since))
    };
    run_until(&mut t, 19);
    let before = current(&t);
    assert_eq!(before, Some((move_to(goal.x, goal.y), 2)));
    let mut x = object(&t, sb).pos.x;
    while t.sim().tick() < 70 {
        t.step();
        assert_eq!(current(&t), before, "tick {}: the same Move", t.sim().tick());
        assert!(object(&t, sb).pos.x > x, "tick {}: on its way", t.sim().tick());
        x = object(&t, sb).pos.x;
    }
    let laid = laid_by(&t, sb);
    assert!(laid.len() >= 10, "{laid:?}");
    assert!(
        laid.iter().all(|p| (20 + LIFE..60 + LIFE).contains(&p.until)),
        "{laid:?}"
    );
    step_until(&mut t, 200, "it arrives", |t| object(t, sb).pos == goal);
    assert_eq!(laid_by(&t, sb), laid, "nothing laid once off");
}
