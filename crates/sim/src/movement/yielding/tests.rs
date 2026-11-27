use super::*;
use crate::defs::ObjectKind;
use crate::orders::{Order, OrderGroup, OrderKind};
use crate::scenario::SpawnAt;
use crate::sim::Sim;
use crate::testing::ScenarioBuilder;

fn peacekeeper(owner: Option<PlayerId>) -> Object {
    let builder = ScenarioBuilder::new("....\n....").spawn(
        ObjectKind::Peacekeeper,
        owner,
        SpawnAt::Point(Vec2::new(100.0, 64.0)),
    );
    Sim::new(&builder.build()).objects().next().unwrap().clone()
}

fn yields(o: &Object) -> bool {
    can_yield(
        o,
        o.mover.as_ref().unwrap(),
        &o.movement(crate::augmentation::Research::NONE).unwrap(),
    )
}

/// 0022 and 0023, Who yields: idle units with an owner; not on Hold or under any order, not while moving, not in
/// an attack sequence, not if they can't move. A slowing mode (Riot Shield) doesn't matter.
#[test]
fn only_idle_units_that_can_move_yield() {
    let idle = peacekeeper(Some(PlayerId(1)));
    assert!(yields(&idle));
    assert!(!yields(&peacekeeper(None)), "nobody's unit");

    let order = |kind| Order {
        kind,
        group: OrderGroup(1),
        slot: None,
    };
    let mut hold = idle.clone();
    hold.orders.as_mut().unwrap().current = Some(crate::orders::ActiveOrder {
        order: order(OrderKind::Hold),
        since: 1,
        state: crate::orders::Behaviour::Hold { target: None },
    });
    assert!(!yields(&hold), "Hold Position is a command");
    let mut queued = idle.clone();
    queued.orders.as_mut().unwrap().give(order(OrderKind::Stop), true);
    assert!(!yields(&queued));

    let mut moving = idle.clone();
    moving
        .mover
        .as_mut()
        .unwrap()
        .go_to(MoveGoal::Point(Vec2::ONE), None, 1);
    assert!(!yields(&moving));

    for (phase, busy) in [
        (AttackPhase::Aiming { target: ObjectId(9) }, true),
        (AttackPhase::AimingAt { at: Vec2::ONE }, true),
        (AttackPhase::Firing { target: None }, true),
        (
            AttackPhase::FiringAt {
                at: Vec2::ONE,
                target: Some(ObjectId(9)),
            },
            true,
        ),
        (AttackPhase::Cooldown, true),
        (AttackPhase::Reloading, false),
    ] {
        let mut attacking = idle.clone();
        attacking.attack.as_mut().unwrap().phase = Some(phase);
        assert_eq!(yields(&attacking), !busy, "{phase:?}");
    }

    // 0023: modes don't matter, unless the unit can't move in them (the hook for a deployed Artillery Rig).
    let mut shielded = idle.clone();
    shielded.mover.as_mut().unwrap().speed_factor = 0.5;
    assert!(yields(&shielded), "a raised Riot Shield isn't a task");
    let mut deployed = idle.clone();
    deployed.mover.as_mut().unwrap().speed_factor = 0.0;
    assert!(!yields(&deployed), "a unit that can't move is a fixed obstacle");

    let mut stepping = idle.clone();
    stepping.mover.as_mut().unwrap().sidestep = Some(Sidestep::new(Vec2::X, 24.0, 4.0, 0, None));
    assert!(!yields(&stepping), "one sidestep per request");
}

/// 0022: only to the owner's units. 0027: not to the group it arrived with.
#[test]
fn a_unit_yields_to_its_owners_units_but_not_to_the_group_it_arrived_with() {
    let body = |owner, arrived_with| Body {
        id: ObjectId(1),
        owner,
        pos: Vec2::ZERO,
        half: Vec2::splat(12.0),
        oriented: None,
        velocity: Vec2::ZERO,
        heading: Vec2::ZERO,
        stationary: true,
        progress: None,
        arrived_with,
        yields: true,
        gathers: false,
        leads: true,
        moved: 0,
        crushes: false,
        crushable: false,
    };
    let (p1, p2) = (Some(PlayerId(1)), Some(PlayerId(2)));
    assert!(yields_to(&body(p1, None), p1, Some(3)));
    assert!(!yields_to(&body(p1, None), p2, Some(3)), "enemies never yield");
    assert!(!yields_to(&body(None, None), None, None));
    assert!(!yields_to(&body(p1, Some(3)), p1, Some(3)), "its own group order");
    assert!(yields_to(&body(p1, Some(3)), p1, Some(4)), "another order");
    assert!(yields_to(&body(p1, Some(3)), p1, None));
    let busy = Body {
        yields: false,
        gathers: false,
        leads: true,
        moved: 0,
        ..body(p1, None)
    };
    assert!(!yields_to(&busy, p1, None));
}
