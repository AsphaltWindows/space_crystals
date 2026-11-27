//! `factions/gdo/units/drone_operator.md > Launching and piloting`: "The Drone Operator launches a drone from its stock.
//! The drone is a separate flying unit **controlled directly by the player**"; "Only **one drone can be active at a
//! time** per operator"; "The Drone Operator **cannot move** while it has a drone active" (`crate::tether`,
//! `OrderKind::Launch`, plan decision T91).

use sc_sim::tether::Tether;

use super::*;

/// R in tick 2: the drone (2) comes into the game in that tick over its operator (1), player 1's, tethered to it, its
/// battery to run out 320 ticks on, and the stock holds none. While it is out the gate refuses the operator a Move, a
/// second R and F (`Reject::DroneOut`, with feedback), and takes a Stop and a Hold; a group Move goes to the others of
/// the group, and the operator stays where it is, while the drone flies off on its own Move. Once the drone has ended
/// the operator walks again.
#[test]
fn the_drone_comes_out_over_its_operator_which_holds_still() {
    let mut t = field(|builder| builder.spawn(PK, Some(P1), SpawnAt::Point(from_home(0.0, 64.0))));
    next(&mut t, P1, launch(&[1]));
    let (operator, drone) = (1, 3);
    assert_eq!(launches(&t), [(2, id(operator), id(drone))]);
    let d = object(t.sim(), drone);
    assert_eq!((d.kind, d.owner, d.pos), (DRONE, Some(P1), HOME), "over the operator");
    let tether = Tether {
        operator: id(operator),
        battery_until: 2 + BATTERY,
    };
    assert_eq!(d.tether, Some(tether));
    assert_eq!(object(t.sim(), operator).drone, Some(id(drone)));
    assert_eq!(stock(t.sim(), operator).count, 0);

    let away = from_home(-300.0, 0.0);
    assert_eq!(gate(t.sim(), P1, move_to(&[operator], away)), Err(Reject::DroneOut));
    assert_eq!(
        gate(t.sim(), P1, order(&[operator], OrderKind::AttackMove(away))),
        Err(Reject::DroneOut)
    );
    assert_eq!(
        gate(t.sim(), P1, launch(&[operator])),
        Err(Reject::DroneOut),
        "one at a time"
    );
    assert_eq!(gate(t.sim(), P1, buy(&[operator])), Err(Reject::DroneOut));
    assert_eq!(gate(t.sim(), P1, order(&[operator], OrderKind::Stop)), Ok(()));
    assert_eq!(gate(t.sim(), P1, order(&[operator], OrderKind::Hold)), Ok(()));
    next(&mut t, P1, move_to(&[operator, 2], away));
    next(&mut t, P1, move_to(&[drone], from_home(200.0, 0.0)));
    t.run(60);
    assert_eq!(t.pos(operator), HOME, "it holds still");
    assert!(t.pos(2).x < HOME.x - 100.0, "the Peacekeeper walked: {}", t.pos(2));
    assert!(t.pos(drone).x > HOME.x + 150.0, "the drone flew: {}", t.pos(drone));
    assert_eq!(
        object(t.sim(), operator)
            .orders
            .as_ref()
            .and_then(|o| o.current.as_ref()),
        None
    );

    next(&mut t, P1, Command::Debug(DebugCommand::Remove { id: id(drone) }));
    assert_eq!(object(t.sim(), operator).drone, None);
    next(&mut t, P1, move_to(&[operator], away));
    t.run(30);
    assert!(t.pos(operator).x < HOME.x - 50.0, "free again: {}", t.pos(operator));
}

/// R is an order, untargeted and queueable (plan decision T91): Shift-queued behind a Move, the operator walks first
/// and launches where the walk ends. R is refused to a unit without a stock (`Reject::LacksAbility`) and, its drone
/// spent, to an operator with none in stock (`Reject::StockEmpty`).
#[test]
fn a_queued_launch_comes_after_the_walk() {
    let mut t = field(|builder| builder.spawn(PK, Some(P1), SpawnAt::Point(from_home(0.0, 64.0))));
    assert_eq!(gate(t.sim(), P1, launch(&[2])), Err(Reject::LacksAbility));
    let there = from_home(256.0, 0.0);
    next(&mut t, P1, move_to(&[1], there));
    let queued = Command::Order {
        units: vec![id(1)],
        kind: OrderKind::Launch,
        queue: true,
    };
    next(&mut t, P1, queued);
    while launches(&t).is_empty() {
        assert!(t.sim().tick() < 200, "no launch\n{}", t.ascii(None));
        t.step();
    }
    let [(_, operator, drone)] = launches(&t)[..] else {
        panic!("one launch: {:?}", launches(&t));
    };
    assert_eq!(operator, id(1));
    assert_eq!(t.pos(1), there, "launched where the walk ended");
    assert_eq!(object(t.sim(), drone.0).pos, there);
    next(&mut t, P1, Command::Debug(DebugCommand::Remove { id: drone }));
    assert_eq!(gate(t.sim(), P1, launch(&[1])), Err(Reject::StockEmpty));
}
