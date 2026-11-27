//! The orders given while a unit deploys (`factions/gdo/units/artillery_rig.md > Modes > Orders given while deploying`,
//! design change 0214): "the **last order given** (without Shift; each new one replaces the previous) is **held**, and
//! it is the first thing the unit tries once deployment finishes. Shift-queued orders line up behind it as usual. At
//! that point each order is checked against the deployed unit: one it can't carry out (e.g. **Move**, since a deployed
//! unit can't move) is **dropped** with the usual unavailable-command feedback, and the next one is tried."

use super::*;

/// Acceptance, "a Move given meanwhile is held, then dropped at completion with the feedback event": the gate takes the
/// Move of the deploying Rig, which goes on deploying where it stands, the Move behind its deploy; as the deploy
/// completes, the Move goes with a `SimEvent::OrderDropped`, after the `SimEvent::KindChanged`, and the deployed Rig is
/// idle where it deployed.
#[test]
fn a_move_given_while_it_deploys_is_held_then_dropped() {
    let mut t = field(RIG, |b| b);
    let at = place(&t, UNIT);
    let given = give(&mut t, &[UNIT], deploy(), false);
    let goal = move_to(from_home(gu(6.0), 0.0));
    t.run(10);
    give(&mut t, &[UNIT], goal, false);
    assert_eq!((current(&t, UNIT), queued(&t, UNIT)), (Some(deploy()), vec![goal]));
    t.run_until(given + FRAMES - 1);
    assert_eq!((kind_of(&t, UNIT), place(&t, UNIT)), (RIG, at), "it stood, deploying");
    assert!(dropped(&t).is_empty());
    t.step();
    assert_eq!(dropped(&t), [(given + FRAMES, UNIT, goal)]);
    let tick = given + FRAMES;
    let events: Vec<&SimEvent> = (t.events().iter())
        .filter(|(at, e)| *at == tick && matches!(e, SimEvent::KindChanged { .. } | SimEvent::OrderDropped { .. }))
        .map(|(_, e)| e)
        .collect();
    assert!(
        matches!(
            events[..],
            [SimEvent::KindChanged { .. }, SimEvent::OrderDropped { .. }]
        ),
        "{events:?}"
    );
    assert_eq!((current(&t, UNIT), queued(&t, UNIT)), (None, vec![]));
    t.run(100);
    assert_eq!((kind_of(&t, UNIT), place(&t, UNIT)), (DEPLOYED, at));
}

/// Acceptance, "an Attack given meanwhile runs once it is deployed" (0214's Verification: "right-click an enemy
/// mid-deploy → it attacks once deployed"): the mobile Rig, which takes no Attack by itself (`Reject::CannotAttack`:
/// "has no attack"), holds one while it deploys; nothing is fired until it has deployed, and then it shells the
/// Peacekeeper.
#[test]
fn an_attack_given_while_it_deploys_runs_once_deployed() {
    let target = id(2);
    let mut t = field(RIG, |b| b.spawn(PK, Some(P2), SpawnAt::Point(east_of_rig(4.0))));
    let attack = OrderKind::Attack(target);
    assert_eq!(gate(&t, &[UNIT], attack, false), Err(Reject::CannotAttack));
    let given = give(&mut t, &[UNIT], deploy(), false);
    t.run(20);
    give(&mut t, &[UNIT], attack, false);
    assert_eq!(queued(&t, UNIT), [attack]);
    t.run_until(given + FRAMES);
    assert!(blasts_by(&t, UNIT).is_empty() && dropped(&t).is_empty());
    assert_eq!(
        (kind_of(&t, UNIT), current(&t, UNIT)),
        (DEPLOYED, Some(attack)),
        "its held order first"
    );
    t.run(80);
    let blasts = blasts_by(&t, UNIT);
    assert_eq!(blasts.len(), 1, "{blasts:?}");
    assert!(
        hits_on(&t, target)
            .iter()
            .any(|&(tick, damage)| tick == blasts[0] && damage > 0.0),
        "the shell hit the Peacekeeper: {:?}",
        hits_on(&t, target)
    );
}

/// "Each new one replaces the previous": an Attack, then a Move, both without Shift, leave the Move alone held, which
/// is dropped; the Attack never runs.
#[test]
fn the_last_order_given_is_the_one_held() {
    let target = id(2);
    let mut t = field(RIG, |b| b.spawn(PK, Some(P2), SpawnAt::Point(east_of_rig(4.0))));
    let given = give(&mut t, &[UNIT], deploy(), false);
    give(&mut t, &[UNIT], OrderKind::Attack(target), false);
    let goal = move_to(from_home(0.0, gu(5.0)));
    give(&mut t, &[UNIT], goal, false);
    assert_eq!(queued(&t, UNIT), [goal]);
    t.run_until(given + FRAMES);
    assert_eq!(dropped(&t), [(given + FRAMES, UNIT, goal)]);
    t.run(150);
    assert!(blasts_by(&t, UNIT).is_empty(), "the Attack was replaced");
}

/// Acceptance, "Shift-queued orders line up after it": a held Attack, then a Shift-Move and a Shift-Attack Ground,
/// line up behind the deploy in that order; as it completes, the Move is dropped and the other two stay, the Attack
/// first.
#[test]
fn shift_queued_orders_line_up_behind_the_held_one() {
    let target = id(2);
    let mut t = field(RIG, |b| b.spawn(PK, Some(P2), SpawnAt::Point(east_of_rig(4.0))));
    let given = give(&mut t, &[UNIT], deploy(), false);
    let (attack, goal, ground) = (
        OrderKind::Attack(target),
        move_to(from_home(0.0, gu(5.0))),
        OrderKind::AttackGround(from_home(0.0, gu(6.0))),
    );
    give(&mut t, &[UNIT], attack, false);
    give(&mut t, &[UNIT], goal, true);
    give(&mut t, &[UNIT], ground, true);
    assert_eq!(current(&t, UNIT), Some(deploy()));
    assert_eq!(queued(&t, UNIT), [attack, goal, ground]);
    t.run_until(given + FRAMES);
    assert_eq!(dropped(&t), [(given + FRAMES, UNIT, goal)]);
    assert_eq!((current(&t, UNIT), queued(&t, UNIT)), (Some(attack), vec![ground]));
}

/// "When deploying completes, they are validated against the deployed type" (design change 0214): an Attack on a
/// Sparrow, held by the deploying Rig, is dropped as it deploys, since its shell hits ground alone ("TargetDomain:
/// Ground"; `Reject::CannotHit`, which the deployed Rig gets for it at the gate).
#[test]
fn a_held_attack_on_an_aircraft_is_dropped() {
    let sparrow = id(2);
    let mut t = field(RIG, |b| {
        b.spawn(ObjectKind::Sparrow, Some(P2), SpawnAt::Point(from_home(gu(4.0), 0.0)))
    });
    let given = give(&mut t, &[UNIT], deploy(), false);
    let attack = OrderKind::Attack(sparrow);
    give(&mut t, &[UNIT], attack, false);
    t.run_until(given + FRAMES);
    assert_eq!(dropped(&t), [(given + FRAMES, UNIT, attack)]);
    assert_eq!(gate(&t, &[UNIT], attack, false), Err(Reject::CannotHit(sparrow)));
}
