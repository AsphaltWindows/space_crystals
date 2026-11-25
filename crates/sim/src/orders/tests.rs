//! Unit tests of `orders`: the queue, and the behaviours of a lone unit in an empty world.

use super::*;
use crate::combat::AttackState;
use crate::defs::ObjectKind;
use crate::ids::PlayerId;
use crate::map::Map;
use crate::mover::Mover;
use crate::paths::NavState;
use crate::vision::Vision;

mod air;
mod leash;
mod min_range;
mod priority;

fn order(kind: OrderKind) -> Order {
    Order {
        kind,
        group: OrderGroup(0),
        slot: None,
    }
}

fn move_to(x: f32, y: f32) -> Order {
    order(OrderKind::Move(Target::Point(Vec2::new(x, y))))
}

/// `systems/controls/commands.md > Unit Command`.
#[test]
fn shift_appends_and_otherwise_the_order_replaces_the_queue() {
    let mut orders = Orders::default();
    assert!(orders.is_idle());
    assert!(orders.give(move_to(1.0, 1.0), true));
    assert!(orders.give(move_to(2.0, 2.0), true));
    orders.current = Some(ActiveOrder {
        order: move_to(0.0, 0.0),
        since: 1,
        state: Behaviour::Plain,
    });
    assert_eq!(orders.queue.len(), 2);
    assert!(orders.give(order(OrderKind::Stop), false));
    assert_eq!(orders.current, None);
    assert_eq!(orders.queue, [order(OrderKind::Stop)]);
}

#[test]
fn the_queue_is_capped() {
    let mut orders = Orders::default();
    for i in 0..MAX_QUEUED_ORDERS {
        assert!(orders.give(move_to(i as f32, 0.0), true));
    }
    assert!(!orders.give(move_to(99.0, 0.0), true));
    assert_eq!(orders.queue.len(), MAX_QUEUED_ORDERS);
    // Replacing always fits.
    assert!(orders.give(move_to(99.0, 0.0), false));
    assert_eq!(orders.queue.len(), 1);
}

/// One tick of phase 5a for a lone Peacekeeper of player 1 at the origin of an empty map: no other object, so
/// nothing to see, engage or follow.
fn tick(orders: &mut Orders, mover: &mut Mover, attack: Option<&mut AttackState>, tick: Tick) {
    let map = Map::from_ascii("....\n....").unwrap();
    let vision = Vision::new(&map, [PlayerId(1)]);
    let nav = NavState::unbuilt(&map);
    let world = World::new(&map, &vision, &nav, &BTreeMap::new(), tick, true);
    let mut unit = Unit {
        id: ObjectId(1),
        owner: Some(PlayerId(1)),
        kind: ObjectKind::Peacekeeper,
        pos: Vec2::new(40.0, 40.0),
        facing: 0.0,
        mover,
        attack,
        weapon: ObjectKind::Peacekeeper
            .def()
            .unit
            .as_ref()
            .and_then(|unit| unit.attack.as_ref()),
        may_attack: true,
        converting: false,
        worker: None,
    };
    update_unit(
        orders,
        &mut unit,
        &world,
        &mut gather::Claims::default(),
        &mut Reach::default(),
    );
}

#[test]
fn moves_complete_on_arrival_and_the_next_order_starts_at_once() {
    let mut orders = Orders::default();
    let mut mover = Mover::default();
    orders.give(move_to(10.0, 0.0), true);
    orders.give(move_to(20.0, 0.0), true);
    tick(&mut orders, &mut mover, None, 1);
    assert_eq!(mover.goal, MoveGoal::Point(Vec2::new(10.0, 0.0)));
    assert_eq!(orders.queue.len(), 1);
    // Still on the way: nothing changes.
    tick(&mut orders, &mut mover, None, 2);
    assert_eq!(orders.current.as_ref().map(|a| a.since), Some(1));
    mover.finish(MoveStatus::Arrived);
    tick(&mut orders, &mut mover, None, 3);
    assert_eq!(mover.goal, MoveGoal::Point(Vec2::new(20.0, 0.0)));
    assert_eq!(orders.current.as_ref().map(|a| a.since), Some(3));
    mover.finish(MoveStatus::Failed);
    tick(&mut orders, &mut mover, None, 4);
    assert!(orders.is_idle());
    assert_eq!(mover.status, MoveStatus::Failed, "the last outcome stays readable");
}

#[test]
fn stop_completes_at_once_and_hold_never_does() {
    let mut orders = Orders::default();
    let mut mover = Mover::default();
    orders.give(move_to(10.0, 0.0), true);
    tick(&mut orders, &mut mover, None, 1);
    orders.give(order(OrderKind::Stop), true);
    orders.give(order(OrderKind::Hold), false);
    orders.give(move_to(30.0, 0.0), true);
    tick(&mut orders, &mut mover, None, 2);
    assert_eq!(mover.goal, MoveGoal::None);
    assert_eq!(orders.current.as_ref().map(|a| a.order.kind), Some(OrderKind::Hold));
    for t in 3..50 {
        tick(&mut orders, &mut mover, None, t);
    }
    assert_eq!(orders.queue.len(), 1, "orders queued behind Hold wait");
    assert_eq!(mover.goal, MoveGoal::None);

    let mut stopping = Orders::default();
    stopping.give(order(OrderKind::Stop), true);
    stopping.give(move_to(5.0, 5.0), true);
    tick(&mut stopping, &mut mover, None, 60);
    assert_eq!(
        mover.goal,
        MoveGoal::Point(Vec2::new(5.0, 5.0)),
        "Stop completed in its tick"
    );
}

/// `systems/controls/behaviors.md > AttackingObject`: an Attack engages its target; a target that doesn't exist
/// completes it, and the next order engages its own. Any other order disengages.
#[test]
fn attack_engages_its_target_and_another_order_disengages() {
    let mut mover = Mover::default();
    let mut attack = AttackState::default();
    // An empty world: no order here looks at it as it starts.
    let map = Map::from_ascii("....\n....").unwrap();
    let vision = Vision::new(&map, [PlayerId(1)]);
    let nav = NavState::unbuilt(&map);
    let world = |tick| World::new(&map, &vision, &nav, &BTreeMap::new(), tick, true);
    let mut state = start(
        &order(OrderKind::Attack(ObjectId(3))),
        &mut unit(&mut mover, Some(&mut attack)),
        &world(2),
    );
    assert_eq!(attack.target, Some(ObjectId(3)));
    assert_eq!(state, Behaviour::Attack(AttackOrder::default()));
    state = start(&move_to(5.0, 5.0), &mut unit(&mut mover, Some(&mut attack)), &world(3));
    assert_eq!(
        (attack.target, state),
        (None, Behaviour::Plain),
        "another order disengages"
    );
    // Nothing exists in this world: both Attacks complete at once, and the unit is idle with nothing engaged.
    let mut orders = Orders::default();
    orders.give(order(OrderKind::Attack(ObjectId(3))), true);
    orders.give(order(OrderKind::Attack(ObjectId(4))), true);
    tick(&mut orders, &mut mover, Some(&mut attack), 4);
    assert!(orders.is_idle());
    assert_eq!((attack.target, orders.auto), (None, Auto::default()));
    assert_eq!(mover.goal, MoveGoal::None);
}

fn unit<'a>(mover: &'a mut Mover, attack: Option<&'a mut AttackState>) -> Unit<'a> {
    Unit {
        id: ObjectId(1),
        owner: Some(PlayerId(1)),
        kind: ObjectKind::Peacekeeper,
        pos: Vec2::new(40.0, 40.0),
        facing: 0.0,
        mover,
        attack,
        weapon: ObjectKind::Peacekeeper
            .def()
            .unit
            .as_ref()
            .and_then(|unit| unit.attack.as_ref()),
        may_attack: true,
        converting: false,
        worker: None,
    }
}

/// A unit that can't attack (no `AttackState`) completes Attack at once; so does a Move to an object that is
/// gone. AttackMove and Patrol set off.
#[test]
fn orders_without_a_target_or_an_attack_complete_and_the_others_set_off() {
    let mut orders = Orders::default();
    let mut mover = Mover::default();
    mover.go_to(MoveGoal::Point(Vec2::ONE), None, 1);
    for kind in [
        OrderKind::Move(Target::Object(ObjectId(3))),
        OrderKind::Attack(ObjectId(3)),
    ] {
        orders.give(order(kind), true);
    }
    tick(&mut orders, &mut mover, None, 2);
    assert!(orders.is_idle());
    assert_eq!(mover.goal, MoveGoal::None, "they stop the unit");
    let to = Vec2::new(100.0, 40.0);
    orders.give(order(OrderKind::AttackMove(to)), true);
    orders.give(order(OrderKind::Patrol(Vec2::new(40.0, 100.0))), true);
    tick(&mut orders, &mut mover, None, 3);
    assert_eq!(mover.goal, MoveGoal::Point(to));
    assert!(matches!(
        orders.current.as_ref().map(|a| &a.state),
        Some(Behaviour::AttackMove(leg)) if leg.to == to && leg.marching
    ));
    // The AttackMove arrives: the Patrol starts from here (design change 0036) and never completes.
    mover.finish(MoveStatus::Arrived);
    tick(&mut orders, &mut mover, None, 9);
    let Some(Behaviour::Patrol(patrol)) = orders.current.as_ref().map(|a| &a.state) else {
        panic!("the Patrol runs: {orders:?}");
    };
    assert_eq!((patrol.origin, patrol.outbound), (Vec2::new(40.0, 40.0), true));
    assert_eq!(mover.goal, MoveGoal::Point(Vec2::new(40.0, 100.0)));
    // A leg arrives: the unit turns around in the same tick. A leg that fails counts as arrived (0037) ...
    mover.finish(MoveStatus::Arrived);
    tick(&mut orders, &mut mover, None, 20);
    assert_eq!(mover.goal, MoveGoal::Point(Vec2::new(40.0, 40.0)));
    mover.finish(MoveStatus::Failed);
    tick(&mut orders, &mut mover, None, 30);
    assert_eq!(mover.goal, MoveGoal::Point(Vec2::new(40.0, 100.0)));
    // ... an arrival in between starts the count again ...
    mover.finish(MoveStatus::Arrived);
    tick(&mut orders, &mut mover, None, 40);
    mover.finish(MoveStatus::Failed);
    tick(&mut orders, &mut mover, None, 50);
    assert!(!orders.is_idle());
    // ... and two failed legs in a row end the Patrol.
    mover.finish(MoveStatus::Failed);
    tick(&mut orders, &mut mover, None, 60);
    assert!(orders.is_idle());
}

/// `systems/controls/commands.md > Group Destinations (Magic Box)`: a location order's own destination
/// overrides the clicked point.
#[test]
fn a_units_own_destination_overrides_the_clicked_point() {
    let click = Vec2::new(1.0, 2.0);
    for kind in [
        OrderKind::Move(Target::Point(click)),
        OrderKind::AttackMove(click),
        OrderKind::Patrol(click),
    ] {
        let mut o = order(kind);
        assert_eq!(o.destination(), Some(click));
        o.slot = Some(Vec2::new(3.0, 4.0));
        assert_eq!(o.destination(), Some(Vec2::new(3.0, 4.0)));
    }
    assert_eq!(order(OrderKind::Hold).destination(), None);
    assert_eq!(order(OrderKind::Attack(ObjectId(1))).destination(), None);
}
