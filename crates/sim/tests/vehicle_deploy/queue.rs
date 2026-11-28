//! A queued deploy (`factions/gdo/units/artillery_rig.md > Modes`: "**Queueable**: Shift-Deploy waits until the
//! earlier orders are done (e.g. move here, then deploy). Without Shift it replaces the current orders and starts at
//! once."), and the orders Shift-queued behind one, judged by the gate against the type the unit will be when they run
//! (`systems/mechanics/modes.md > Archetype: Deploy (an order)`, `sc_sim::vehicle_deploy::can_take_queued`).

use super::*;

/// The brief's "move, then Shift-Deploy: it deploys where the move ends": the Rig drives 6 grid units east, an
/// Artillery Rig all the way; its deploy starts as the Move completes, where the Rig stopped, and 48 frames later it
/// is deployed there.
#[test]
fn a_shift_deploy_after_a_move_deploys_where_the_move_ends() {
    let mut t = field(RIG, |b| b);
    let goal = from_home(gu(6.0), 0.0);
    give(&mut t, &[UNIT], move_to(goal), false);
    give(&mut t, &[UNIT], deploy(), true);
    assert_eq!(queued(&t, UNIT), [deploy()]);
    let started = loop {
        assert!(t.sim().tick() < 400, "it never started its deploy");
        assert_eq!(kind_of(&t, UNIT), RIG);
        t.step();
        if current(&t, UNIT) == Some(deploy()) {
            break t.sim().tick();
        }
    };
    let stood = place(&t, UNIT);
    assert!(stood.0.distance(goal) < 8.0, "at the Move's end: {stood:?}");
    t.run_until(started + FRAMES - 1);
    assert_eq!((kind_of(&t, UNIT), place(&t, UNIT)), (RIG, stood));
    t.step();
    assert_eq!((kind_of(&t, UNIT), place(&t, UNIT)), (DEPLOYED, stood));
    assert_eq!(kind_changes(&t), [(started + FRAMES, UNIT, DEPLOYED)]);
}

/// "Without Shift it replaces the current orders and starts at once": a Deploy given to a driving Rig, unshifted,
/// ends its Move.
#[test]
fn a_deploy_without_shift_replaces_the_move() {
    let mut t = field(RIG, |b| b);
    give(&mut t, &[UNIT], move_to(from_home(gu(10.0), 0.0)), false);
    t.run(20);
    let given = give(&mut t, &[UNIT], deploy(), false);
    assert_eq!((current(&t, UNIT), queued(&t, UNIT)), (Some(deploy()), vec![]));
    t.run_until(given + FRAMES);
    assert_eq!(kind_of(&t, UNIT), DEPLOYED);
    assert!(t.pos(UNIT.0).x < from_home(gu(4.0), 0.0).x, "it stopped well short");
}

/// `systems/mechanics/modes.md > Archetype: Deploy (an order)`: an order is "checked against the new type", the type
/// the unit is when it runs. The deployed Rig given a Shift-Undeploy and a Shift-Move in one tick takes both, the Move
/// judged against the mobile Rig it will be; once undeployed, it drives there, nothing dropped.
#[test]
fn a_move_shift_queued_behind_an_undeploy_runs_once_it_is_mobile() {
    let mut t = deployed(|b| b);
    let goal = from_home(gu(6.0), 0.0);
    let tick = t.sim().tick() + 1;
    t.order_at(tick, P1, &[UNIT], undeploy(), true);
    t.order_at(tick, P1, &[UNIT], move_to(goal), true);
    t.step();
    assert_eq!(
        (current(&t, UNIT), queued(&t, UNIT)),
        (Some(undeploy()), vec![move_to(goal)])
    );
    t.run_until(tick + FRAMES);
    assert_eq!((kind_of(&t, UNIT), current(&t, UNIT)), (RIG, Some(move_to(goal))));
    t.run(600);
    assert!(
        place(&t, UNIT).0.distance(goal) < 8.0,
        "it drove there: {:?}",
        place(&t, UNIT)
    );
    assert!(dropped(&t).is_empty(), "{:?}", dropped(&t));
}

/// The gate judges a Shift-queued order against the type the unit will be when it runs, after the deploys queued
/// before it (`vehicle_deploy::kind_then`). The deployed Rig attacking, an Undeploy queued behind its Attack: it takes
/// a
/// Shift-Move and a Shift-Deploy, the mobile Rig's, and refuses a Shift-Attack, the mobile Rig having no attack
/// (`Reject::CannotAttack`), and a second Undeploy (`Reject::LacksAbility`). Without Shift, its type now judges them.
#[test]
fn behind_a_queued_undeploy_the_gate_judges_the_mobile_type() {
    let enemy = id(2);
    let mut t = deployed(|b| b.spawn(PK, Some(P2), SpawnAt::Point(east_of_rig(4.0))));
    give(&mut t, &[UNIT], OrderKind::Attack(enemy), false);
    give(&mut t, &[UNIT], undeploy(), true);
    assert_eq!(queued(&t, UNIT), [undeploy()]);
    let there = from_home(gu(5.0), gu(-3.0));
    assert_eq!(gate(&t, &[UNIT], move_to(there), true), Ok(()));
    assert_eq!(gate(&t, &[UNIT], move_to(there), false), Err(Reject::CannotMove));
    assert_eq!(gate(&t, &[UNIT], deploy(), true), Ok(()));
    assert_eq!(gate(&t, &[UNIT], undeploy(), true), Err(Reject::LacksAbility));
    assert_eq!(gate(&t, &[UNIT], undeploy(), false), Ok(()));
    assert_eq!(
        gate(&t, &[UNIT], OrderKind::Attack(enemy), true),
        Err(Reject::CannotAttack)
    );
    assert_eq!(gate(&t, &[UNIT], OrderKind::Attack(enemy), false), Ok(()));
}

/// Behind a queued Deploy, the deployed type judges a Shift-queued order: a second Deploy is refused, the deployed Rig
/// only undeploying (`Reject::LacksAbility`), and an Undeploy taken; an Attack on a ground unit is taken, which the
/// mobile Rig refuses, and one on an aircraft refused, the deployed Rig's shell targeting the ground alone
/// (`Reject::CannotHit`); a Move too (`Reject::CannotMove`).
#[test]
fn behind_a_queued_deploy_the_gate_judges_the_deployed_type() {
    let (enemy, sparrow) = (id(2), id(3));
    let mut t = field(RIG, |b| {
        b.spawn(PK, Some(P2), SpawnAt::Point(east_of_rig(4.0))).spawn(
            ObjectKind::Sparrow,
            Some(P2),
            SpawnAt::Point(from_home(gu(4.0), 0.0)),
        )
    });
    give(&mut t, &[UNIT], move_to(from_home(0.0, gu(6.0))), false);
    give(&mut t, &[UNIT], deploy(), true);
    assert_eq!(queued(&t, UNIT), [deploy()]);
    assert_eq!(gate(&t, &[UNIT], deploy(), true), Err(Reject::LacksAbility));
    assert_eq!(gate(&t, &[UNIT], deploy(), false), Ok(()));
    assert_eq!(gate(&t, &[UNIT], undeploy(), true), Ok(()));
    assert_eq!(gate(&t, &[UNIT], OrderKind::Attack(enemy), true), Ok(()));
    assert_eq!(
        gate(&t, &[UNIT], OrderKind::Attack(enemy), false),
        Err(Reject::CannotAttack)
    );
    let air = OrderKind::Attack(sparrow);
    assert_eq!(gate(&t, &[UNIT], air, true), Err(Reject::CannotHit(sparrow)));
    assert_eq!(gate(&t, &[UNIT], move_to(HOME), true), Err(Reject::CannotMove));
}

/// Two deploys queued, each order judged against the type the deploy before it makes: a Move, a Shift-Deploy, a
/// Shift-Undeploy and a Shift-Move are all taken; the Rig deploys where the first Move ends, undeploys, and drives on.
/// The last Move, behind the Undeploy, is checked as that one completes, not as the Deploy does: nothing is dropped.
#[test]
fn a_move_behind_a_deploy_and_an_undeploy_runs() {
    let mut t = field(RIG, |b| b);
    let (first, second) = (from_home(gu(3.0), 0.0), from_home(gu(6.0), 0.0));
    give(&mut t, &[UNIT], move_to(first), false);
    give(&mut t, &[UNIT], deploy(), true);
    give(&mut t, &[UNIT], undeploy(), true);
    give(&mut t, &[UNIT], move_to(second), true);
    assert_eq!(queued(&t, UNIT), [deploy(), undeploy(), move_to(second)]);
    t.run(800);
    let changes: Vec<ObjectKind> = kind_changes(&t).iter().map(|&(_, _, to)| to).collect();
    assert_eq!((kind_of(&t, UNIT), changes), (RIG, vec![DEPLOYED, RIG]));
    assert!(dropped(&t).is_empty(), "{:?}", dropped(&t));
    assert!(
        place(&t, UNIT).0.distance(second) < 8.0,
        "it drove on: {:?}",
        place(&t, UNIT)
    );
}
