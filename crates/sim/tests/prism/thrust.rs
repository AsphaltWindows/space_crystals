use super::*;

/// "R, click a point 10 GU away: it thrusts 3 GU toward it" (design change 0434's Verification), and as far for a point
/// 1 grid unit away ("always the full 3 grid units whatever the distance"): fired in its tick's orders phase, the burst
/// moves the Prism 12 su a tick for 16 ticks, the order ends in the tick after, and the Prism stays where the burst
/// left it, at rest, nothing of the burst in its velocity or momentum.
#[test]
fn a_thrust_moves_it_three_grid_units_toward_its_point_whatever_its_distance() {
    for far in [10.0, 1.0] {
        let mut t = started(field().spawn(PRISM, Some(P1), point(640.0, 640.0)));
        let prism = id(1);
        assert_eq!(charges(&t, prism), CHARGES);
        t.at(2, P1, thrust(&[prism], Vec2::new(640.0 + 64.0 * far, 640.0), false));
        t.run_until(2);
        assert_eq!(t.pos(1), Vec2::new(652.0, 640.0), "{far} GU: its first tick");
        assert_eq!(charges(&t, prism), CHARGES - 1, "spent as it starts");
        t.run_until(1 + THRUST_TICKS);
        assert_eq!(t.pos(1), Vec2::new(640.0 + THRUST_SU, 640.0), "{far} GU away");
        assert!(
            object(&t, prism).orders.as_ref().unwrap().current.is_some(),
            "its burst over"
        );
        t.run_until(2 + THRUST_TICKS);
        assert!(
            object(&t, prism).orders.as_ref().unwrap().is_idle(),
            "the order ends after its burst"
        );
        t.run(32);
        assert_eq!(t.pos(1), Vec2::new(640.0 + THRUST_SU, 640.0), "where the burst left it");
        let mover = t.mover(1);
        assert_eq!((mover.velocity, mover.momentum), (Vec2::ZERO, Vec2::ZERO));
    }
}

/// The drift after a thrust is the Prism's own: a Prism flying east whose Move a thrust north replaces drifts exactly
/// as one whose Move a Stop replaces, its velocity and momentum the same in every tick, and ends 3 grid units north of
/// it.
#[test]
fn the_drift_during_and_after_a_thrust_is_its_movement_s_own() {
    let run = |thrusts: bool| {
        let mut t = started(field().spawn(PRISM, Some(P1), point(640.0, 640.0)));
        t.order_at(
            2,
            P1,
            &[id(1)],
            OrderKind::Move(sc_sim::Target::Point(Vec2::new(1600.0, 640.0))),
            false,
        );
        t.run_until(40);
        assert!(t.mover(1).momentum.x > 0.5, "under way: {:?}", t.mover(1).momentum);
        let at = t.pos(1);
        let command = match thrusts {
            true => thrust(&[id(1)], Vec2::new(at.x, at.y - 640.0), false),
            false => Command::Order {
                units: vec![id(1)],
                kind: OrderKind::Stop,
                queue: false,
            },
        };
        t.at(41, P1, command);
        let mut seen = Vec::new();
        for _ in 0..(THRUST_TICKS + 30) {
            t.step();
            let mover = t.mover(1);
            seen.push((t.pos(1), mover.velocity, mover.momentum));
        }
        seen
    };
    let (thrusted, stopped) = (run(true), run(false));
    for (tick, (with, without)) in thrusted.iter().zip(&stopped).enumerate() {
        assert_eq!(
            (with.1, with.2),
            (without.1, without.2),
            "tick {}: its drift",
            41 + tick
        );
        let north = (tick as f32 + 1.0).min(THRUST_TICKS as f32) * 12.0;
        assert_eq!(without.0 - with.0, Vec2::new(0.0, north), "tick {}", 41 + tick);
    }
}

/// "Three thrusts in a row, then R greys out": each spends a charge, and with all three spent the gate refuses a
/// fourth ([`Reject::NoCharges`]), the charges never coming back. A unit without thrusts is refused one
/// ([`Reject::LacksAbility`]), and ignores one given with a Prism that has a charge.
#[test]
fn three_thrusts_spend_the_three_charges_and_the_gate_then_refuses_one() {
    let mut t = started(field().spawn(PRISM, Some(P1), point(640.0, 640.0)).spawn(
        ObjectKind::Guard,
        Some(P1),
        point(640.0, 1100.0),
    ));
    let (prism, guard) = (id(1), id(2));
    let east = Vec2::new(4000.0, 640.0);
    assert_eq!(gate(&t, thrust(&[guard], east, false)), Err(Reject::LacksAbility));
    assert_eq!(gate(&t, thrust(&[guard, prism], east, false)), Ok(()));
    for n in 0..CHARGES {
        let at = t.sim().tick() + 1;
        t.at(at, P1, thrust(&[guard, prism], east, false));
        t.run_until(at + THRUST_TICKS);
        assert_eq!(charges(&t, prism), CHARGES - 1 - n);
        assert_eq!(t.pos(1).x, 640.0 + THRUST_SU * (n + 1) as f32, "thrust {}", n + 1);
    }
    assert_eq!(t.pos(2), Vec2::new(640.0, 1100.0), "the Guard ignored them");
    assert_eq!(gate(&t, thrust(&[prism], east, false)), Err(Reject::NoCharges));
    assert_eq!(
        gate(&t, thrust(&[prism], east, true)),
        Err(Reject::NoCharges),
        "queued too"
    );
    assert_eq!(gate(&t, thrust(&[guard, prism], east, false)), Err(Reject::NoCharges));
    t.run(200);
    assert_eq!(charges(&t, prism), 0, "never coming back");
}

/// "R + Shift-clicks on three points: three thrusts in a row": each starts in the tick after the burst before it ended,
/// spends its charge then, and goes toward its own point from where the one before left the Prism. Placeholder (Q0198):
/// a fourth, queued before any of them started and so taken by the gate, starts with no charge left and
/// ends at once, without moving, and the Move queued behind it starts in that same tick.
#[test]
fn shift_queued_thrusts_run_one_after_another_and_one_without_a_charge_ends_without_moving() {
    let mut t = started(field().spawn(PRISM, Some(P1), point(640.0, 640.0)));
    let prism = id(1);
    t.at(2, P1, thrust(&[prism], Vec2::new(1280.0, 640.0), false));
    t.at(2, P1, thrust(&[prism], Vec2::new(832.0, 1280.0), true));
    t.at(2, P1, thrust(&[prism], Vec2::new(0.0, 832.0), true));
    t.at(2, P1, thrust(&[prism], Vec2::new(640.0, 0.0), true));
    let away = Vec2::new(900.0, 832.0);
    t.order_at(2, P1, &[prism], OrderKind::Move(sc_sim::Target::Point(away)), true);
    t.run_until(2);
    assert_eq!(object(&t, prism).orders.as_ref().unwrap().queue.len(), 4, "all taken");
    let legs = [
        Vec2::new(832.0, 640.0),
        Vec2::new(832.0, 832.0),
        Vec2::new(640.0, 832.0),
    ];
    for (n, leg) in legs.into_iter().enumerate() {
        let start = 2 + THRUST_TICKS * n as Tick;
        t.run_until(start);
        assert_eq!(
            charges(&t, prism),
            CHARGES - 1 - n as u32,
            "thrust {} spent as it starts",
            n + 1
        );
        t.run_until(start + THRUST_TICKS - 1);
        assert_eq!(t.pos(1), leg, "thrust {}", n + 1);
    }
    t.run_until(2 + 3 * THRUST_TICKS);
    let orders = object(&t, prism).orders.as_ref().unwrap();
    let current = orders.current.as_ref().expect("the Move");
    assert_eq!(
        (current.order.kind, current.since),
        (OrderKind::Move(sc_sim::Target::Point(away)), t.sim().tick())
    );
    assert!(orders.queue.is_empty());
    assert!(
        t.pos(1).distance(Vec2::new(640.0, 832.0)) <= 1.0,
        "no fourth burst: {}",
        t.pos(1)
    );
}
