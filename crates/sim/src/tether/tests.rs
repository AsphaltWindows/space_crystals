use glam::Vec2;

use super::*;
use crate::ids::PlayerId;
use crate::scenario::SpawnAt;
use crate::sim::Sim;
use crate::testing::ScenarioBuilder;
use crate::units::SU_PER_GU;

/// An operator (1) at (100, 100) and its drone (2), tethered to it, `gap` space units from it edge to edge along x,
/// launched in tick 10.
fn pair(gap: f32) -> BTreeMap<ObjectId, Object> {
    // Both 24 × 24: centre to centre is the gap and a Silhouette's width.
    let builder = ScenarioBuilder::new(&"................\n".repeat(16))
        .spawn(
            ObjectKind::DroneOperator,
            Some(PlayerId(1)),
            SpawnAt::Point(Vec2::new(100.0, 100.0)),
        )
        .spawn(
            ObjectKind::OperatorDrone,
            Some(PlayerId(1)),
            SpawnAt::Point(Vec2::new(124.0 + gap, 100.0)),
        );
    let sim = Sim::new(&builder.build());
    let mut objects: BTreeMap<ObjectId, Object> = sim.objects().map(|o| (o.id, o.clone())).collect();
    objects.get_mut(&ObjectId(1)).unwrap().drone = Some(ObjectId(2));
    objects.get_mut(&ObjectId(2)).unwrap().tether = Some(Tether {
        operator: ObjectId(1),
        battery_until: 10 + 320,
    });
    objects
}

/// The leash is measured edge to edge (design change 0421): at 8 grid units between the Silhouettes the drone is on
/// it, a little more and it falls, and explodes; nothing ends before then.
#[test]
fn the_leash_is_measured_edge_to_edge() {
    let leash = 8.0 * SU_PER_GU;
    assert_eq!(def_of(ObjectKind::OperatorDrone).map(|def| def.leash), Some(8.0));
    assert_eq!(ends(&pair(leash), ObjectId(9), 11), []);
    let past = ends(&pair(leash + 0.5), ObjectId(9), 11);
    assert_eq!(past, [(ObjectId(2), ObjectId(1), Fall::Leash)]);
    assert!(Fall::Leash.explodes() && !Fall::Battery.explodes() && !Fall::Operator.explodes());
}

/// The battery runs out in its tick, 320 after the launch; an operator gone (or not naming the drone) drops it first;
/// a drone past its leash in its battery's tick explodes (the leash before the battery); the tick's new objects wait.
#[test]
fn a_drone_ends_with_its_battery_or_its_operator() {
    assert_eq!(ends(&pair(10.0), ObjectId(9), 10 + 319), []);
    assert_eq!(
        ends(&pair(10.0), ObjectId(9), 10 + 320),
        [(ObjectId(2), ObjectId(1), Fall::Battery)]
    );
    assert_eq!(
        ends(&pair(9.0 * SU_PER_GU), ObjectId(9), 10 + 320),
        [(ObjectId(2), ObjectId(1), Fall::Leash)]
    );
    let mut orphan = pair(10.0);
    orphan.remove(&ObjectId(1));
    assert_eq!(
        ends(&orphan, ObjectId(9), 11),
        [(ObjectId(2), ObjectId(1), Fall::Operator)]
    );
    let mut other = pair(10.0);
    other.get_mut(&ObjectId(1)).unwrap().drone = None;
    assert_eq!(
        ends(&other, ObjectId(9), 11),
        [(ObjectId(2), ObjectId(1), Fall::Operator)]
    );
    assert_eq!(
        ends(&pair(10.0), ObjectId(2), 10 + 320),
        [],
        "a drone of the tick's new objects"
    );
}

/// "The Drone Operator **cannot move** while it has a drone active": it takes only the orders that leave it where it
/// stands.
#[test]
fn an_operator_with_its_drone_out_takes_only_orders_that_dont_move_it() {
    let point = Vec2::new(10.0, 10.0);
    assert!(allows(OrderKind::Stop) && allows(OrderKind::Hold) && allows(OrderKind::Launch));
    for kind in [
        OrderKind::Move(crate::orders::Target::Point(point)),
        OrderKind::AttackMove(point),
        OrderKind::Patrol(point),
        OrderKind::Reverse(point),
        OrderKind::Enter(ObjectId(3)),
    ] {
        assert!(!allows(kind), "{kind:?}");
    }
    let objects = pair(10.0);
    assert!(holds(&objects[&ObjectId(1)]) && !holds(&objects[&ObjectId(2)]));
}
