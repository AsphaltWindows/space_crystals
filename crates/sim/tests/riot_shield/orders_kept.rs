//! The Riot Shield keeps the unit's orders (design changes 0150 and 0151, `factions/gdo/units/peacekeeper.md >
//! Riot Shield > Orders are kept`): an Attack follows its target behind the shield and resumes once it is down, an
//! AttackMove only moves, and the queue stays as it was.

use super::*;

/// The current order of unit `id` and its queue.
fn orders_of(t: &TestSim, id: u32) -> (Option<OrderKind>, Vec<OrderKind>) {
    let orders = t.sim().object(ObjectId(id)).unwrap().orders.as_ref().unwrap();
    let current = orders.current.as_ref().map(|active| active.order.kind);
    (current, orders.queue.iter().map(|order| order.kind).collect())
}

/// Design change 0150 (`> Riot Shield > Orders are kept`): raising the shield is a mode switch, not an order. An
/// explicit Attack is on hold behind it: "while the shield is up it follows the target [...]; when the shield is
/// lowered it **resumes attacking** that target" (0150's Verification: "it stops firing and follows the target;
/// lower it: it shoots that target again"). Raised while still aiming, nothing is fired (`systems/combat.md >
/// Aiming`: interruptible); raised while reloading, the shot fired before stays the only one. The Guard walks past
/// it, and the Peacekeeper follows it with the Attack still its order and nothing engaged; lowered in tick 300, it
/// engages the Guard again and hits it.
#[test]
fn an_attack_behind_a_raised_shield_follows_the_target_and_resumes_once_lowered() {
    for (raised, early) in [(3, vec![]), (8, vec![(4, 9.0)])] {
        let mut t = spawned(
            &[
                (PK, P1, Vec2::new(300.0, 300.0)),    // 1
                (GUARD, P2, Vec2::new(530.0, 300.0)), // 2, beyond its own Range of 1
            ],
            false,
        );
        let order = attack_order(&[1], 2, false);
        let away = OrderKind::Move(Target::Point(Vec2::new(420.0, 420.0)));
        t.at(2, P1, order.command)
            .ability_at(raised, P1, &ids(&[1]), RAISE)
            .order_at(20, P2, &ids(&[2]), away, false)
            .run_until(raised);
        let attack = OrderKind::Attack(ObjectId(2));
        assert_eq!(orders_of(&t, 1), (Some(attack), vec![]), "raised in tick {raised}");
        for tick in raised..300 {
            t.run_until(tick);
            let unit = t.sim().object(ObjectId(1)).unwrap();
            assert_eq!(orders_of(&t, 1).0, Some(attack), "tick {tick}");
            if tick > raised {
                assert_eq!(
                    unit.attack.as_ref().unwrap().target,
                    None,
                    "nothing engaged in tick {tick}"
                );
            }
        }
        assert_eq!(hits_on(&t, 2), early, "raised in tick {raised}");
        let (unit, guard) = (t.pos(1), t.pos(2));
        assert_eq!(guard, Vec2::new(420.0, 420.0));
        assert!(
            unit.distance(Vec2::new(300.0, 300.0)) > 80.0 && unit.distance(guard) < 60.0,
            "it followed the Guard: {unit}"
        );

        t.ability_at(300, P1, &ids(&[1]), LOWER).run_until(370);
        let later: Vec<(Tick, f32)> = hits_on(&t, 2).into_iter().filter(|&(tick, _)| tick > 300).collect();
        assert!(
            !later.is_empty(),
            "it shoots that target again (raised in tick {raised})"
        );
        assert!(later[0].0 >= 300 + T - 1, "only once the shield is down: {later:?}");
        assert_eq!(orders_of(&t, 1).0, Some(attack));
    }
}

/// Design change 0150 for AttackMove: "while the shield is up it only moves; once lowered it engages along the way
/// again". A Peacekeeper attack-moves past a Guard on Hold Position (which can't reach it) with its shield raised: it
/// walks on without engaging; lowered in tick 120 with the Guard in sight and on its leash, it engages the Guard and
/// hits it, still on its AttackMove.
#[test]
fn an_attack_move_behind_a_raised_shield_only_moves_and_engages_once_lowered() {
    let mut t = spawned(
        &[
            (PK, P1, Vec2::new(100.0, 100.0)),    // 1
            (GUARD, P2, Vec2::new(500.0, 300.0)), // 2
        ],
        true,
    );
    let destination = Vec2::new(900.0, 100.0);
    t.ability_at(1, P1, &ids(&[1]), RAISE)
        .order_at(1, P2, &ids(&[2]), OrderKind::Hold, false)
        .order_at(2, P1, &ids(&[1]), OrderKind::AttackMove(destination), false)
        .run_until(119);
    assert_eq!(attacks_by(&t, 1), Vec::<Tick>::new());
    assert!(t.pos(1).x > 300.0, "it walked on: {}", t.pos(1));
    assert_eq!(orders_of(&t, 1).0, Some(OrderKind::AttackMove(destination)));
    t.ability_at(120, P1, &ids(&[1]), LOWER).run_until(200);
    let fired = attacks_by(&t, 1);
    assert!(!fired.is_empty(), "it engages along the way again");
    assert!(fired[0] >= 120 + T - 1, "{fired:?}");
    assert!(!hits_on(&t, 2).is_empty());
    assert_eq!(orders_of(&t, 1).0, Some(OrderKind::AttackMove(destination)));
}

/// Design changes 0150 and 0151 (`> Riot Shield > Orders are kept`, `> Not queueable`): toggling the shield never
/// touches the command queue, and nothing of it enters the queue. A Peacekeeper walking with a Move, a queued Attack
/// and a queued Move has the same orders after each of a raise, a lowering and a raise again, and its walk goes on.
#[test]
fn toggling_the_shield_keeps_the_queue() {
    let mut t = spawned(
        &[
            (PK, P1, Vec2::new(100.0, 300.0)),    // 1
            (GUARD, P2, Vec2::new(300.0, 150.0)), // 2, in sight
        ],
        false,
    );
    let (first, last) = (Vec2::new(500.0, 300.0), Vec2::new(200.0, 400.0));
    t.order_at(2, P1, &ids(&[1]), OrderKind::Move(Target::Point(first)), false)
        .at(2, P1, attack_order(&[1], 2, true).command)
        .order_at(2, P1, &ids(&[1]), OrderKind::Move(Target::Point(last)), true)
        .run_until(3);
    let orders = orders_of(&t, 1);
    assert_eq!(
        orders,
        (
            Some(OrderKind::Move(Target::Point(first))),
            vec![OrderKind::Attack(ObjectId(2)), OrderKind::Move(Target::Point(last))]
        )
    );
    for (tick, ability) in [(4, RAISE), (5 + T, LOWER), (6 + 2 * T, RAISE)] {
        let before = t.pos(1);
        t.ability_at(tick, P1, &ids(&[1]), ability).run_until(tick);
        assert_eq!(orders_of(&t, 1), orders, "tick {tick}");
        t.run(1);
        assert_ne!(t.pos(1), before, "it walks on (tick {tick})");
    }
}
