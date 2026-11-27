//! What the gate refuses (`factions/gdo/units/artillery_rig.md`; `Sim::check_order` with
//! `sc_sim::vehicle_deploy::can_take`): a deploy while one is under way, a deploy to a type that doesn't deploy into the
//! type it names, and an order that would move a unit whose type can't move.

use super::*;

/// Acceptance, "Deploy is unavailable until it completes" (`> Modes`: "**Undeploy is only available once deployment
/// is complete**"): while the Rig deploys, the gate refuses it a Deploy and an Undeploy alike (`Reject::Deploying`);
/// deployed, it takes an Undeploy and no Deploy, which its type doesn't do (`Reject::LacksAbility`); undeploying, the
/// same the other way.
#[test]
fn a_deploy_under_way_refuses_another() {
    let mut t = field(RIG, |b| b);
    assert_eq!(gate(&t, &[UNIT], undeploy(), false), Err(Reject::LacksAbility));
    let given = give(&mut t, &[UNIT], deploy(), false);
    for kind in [deploy(), undeploy()] {
        for queue in [false, true] {
            assert_eq!(
                gate(&t, &[UNIT], kind, queue),
                Err(Reject::Deploying),
                "{kind:?}, Shift {queue}"
            );
        }
    }
    t.run_until(given + FRAMES - 1);
    assert_eq!(
        gate(&t, &[UNIT], undeploy(), false),
        Err(Reject::Deploying),
        "its last frame"
    );
    t.step();
    assert_eq!(gate(&t, &[UNIT], undeploy(), false), Ok(()));
    assert_eq!(gate(&t, &[UNIT], deploy(), false), Err(Reject::LacksAbility));
    give(&mut t, &[UNIT], undeploy(), false);
    assert_eq!(gate(&t, &[UNIT], deploy(), false), Err(Reject::Deploying));
}

/// A deploy goes to the units whose type deploys into the type it names: a Peacekeeper takes none
/// (`Reject::LacksAbility`); in a selection with a Rig, the Rig deploys and the Peacekeeper's orders stay as they were.
#[test]
fn only_a_type_that_deploys_into_it_takes_a_deploy() {
    let pk = id(2);
    let mut t = field(RIG, |b| b.spawn(PK, Some(P1), SpawnAt::Point(from_home(0.0, gu(3.0)))));
    assert_eq!(gate(&t, &[pk], deploy(), false), Err(Reject::LacksAbility));
    let goal = move_to(from_home(gu(5.0), gu(3.0)));
    give(&mut t, &[pk], goal, false);
    give(&mut t, &[UNIT, pk], deploy(), false);
    assert_eq!(current(&t, UNIT), Some(deploy()));
    assert_eq!(current(&t, pk), Some(goal), "the Peacekeeper's Move goes on");
}

/// Plan decision T87, "immobile by its def", and the model's one rule (`vehicle_deploy::can_take`): the deployed Rig,
/// which can't move, takes no order that would take it somewhere (`Reject::CannotMove`): a Move, to a point or to an
/// object, an Attack Move, a Patrol; it takes a Hold, a Stop, an Attack and an Attack Ground. In a selection with a
/// Peacekeeper, the Peacekeeper alone takes the Move.
#[test]
fn a_unit_that_cant_move_takes_no_order_that_moves_it() {
    let (pk, enemy) = (id(2), id(3));
    let mut t = deployed(|b| {
        b.spawn(PK, Some(P1), SpawnAt::Point(from_home(0.0, gu(3.0)))).spawn(
            PK,
            Some(P2),
            SpawnAt::Point(east_of_rig(4.0)),
        )
    });
    let there = from_home(gu(5.0), gu(-3.0));
    for kind in [
        move_to(there),
        OrderKind::Move(Target::Object(pk)),
        OrderKind::AttackMove(there),
        OrderKind::Patrol(there),
    ] {
        assert_eq!(gate(&t, &[UNIT], kind, false), Err(Reject::CannotMove), "{kind:?}");
    }
    for kind in [
        OrderKind::Hold,
        OrderKind::Stop,
        OrderKind::Attack(enemy),
        OrderKind::AttackGround(there),
    ] {
        assert_eq!(gate(&t, &[UNIT], kind, false), Ok(()), "{kind:?}");
    }
    let at = place(&t, UNIT);
    give(&mut t, &[UNIT, pk], move_to(there), false);
    assert_eq!(current(&t, UNIT), None, "the Rig ignores it");
    assert_eq!(current(&t, pk), Some(move_to(there)));
    t.run(100);
    assert_eq!(place(&t, UNIT), at);
}
