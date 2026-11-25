//! Plan §3.3 O's acceptance, Attack Ground: "The order walks into range, fires once, and completes at the blast", and
//! "it is refused for a FullyConnected unit"; with the test grenade (`combat::testing`).

use glam::Vec2;

use crate::combat::testing::{GRENADE, grenadier};
use crate::command::{Command, PlayerCommand};
use crate::defs::{AttackAttrs, ObjectKind};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::{OrderKind, Target};
use crate::shape::Shape;
use crate::sim::Reject;
use crate::testing::{ScenarioBuilder, TestSim};
use crate::units::{Tick, gu};

const P1: PlayerId = PlayerId(1);
const THROWER: ObjectId = ObjectId(1);

/// 16 × 10 open tiles.
fn open() -> String {
    "................\n".repeat(10)
}

/// The launches so far: (tick, from, to).
fn launches(t: &TestSim) -> Vec<(Tick, Vec2, Vec2)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ProjectileLaunched { from, to, .. } => Some((*tick, from, to)),
            _ => None,
        })
        .collect()
}

fn current(t: &TestSim, id: ObjectId) -> Option<OrderKind> {
    let orders = t.sim().object(id).unwrap().orders.as_ref().unwrap();
    orders.current.as_ref().map(|active| active.order.kind)
}

/// The gap from the thrower's shape to `at`: what its Range is measured against.
fn reach(t: &TestSim, at: Vec2) -> f32 {
    Shape::of(t.sim().object(THROWER).unwrap()).distance_to(at)
}

/// A thrower 600 su from the point clicked walks towards it (MovingToLocation), stops in range (4 grid units from its
/// edge), fires once at the point itself, and its order completes in the tick after the blast; the Move queued behind
/// it follows. The attack lets go of the location as the grenade leaves (`// Placeholder: Q0083 point 3`).
#[test]
fn it_walks_into_range_fires_once_and_completes_at_the_blast() {
    let _grenadier = grenadier(GRENADE);
    let start = Vec2::new(100.0, 320.0);
    let at = Vec2::new(700.0, 333.5);
    let then = Vec2::new(100.0, 500.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, ObjectKind::Guard, P1, start)
        .order_at(2, P1, &[THROWER], OrderKind::AttackGround(at), false)
        .order_at(2, P1, &[THROWER], OrderKind::Move(Target::Point(then)), true);
    t.run_until(3);
    assert!(t.status(1) == crate::mover::MoveStatus::Moving, "it walks");
    while launches(&t).is_empty() {
        assert!(t.sim().tick() < 300, "it never fired\n{}", t.ascii(None));
        t.step();
        assert_eq!(current(&t, THROWER), Some(OrderKind::AttackGround(at)));
    }
    let [(launched, from, to)] = launches(&t)[..] else {
        panic!("one launch");
    };
    assert_eq!(to, at, "the point clicked, no snapping");
    assert!(reach(&t, at) <= gu(GRENADE.range), "in range: {}", reach(&t, at));
    assert!(
        from.x < at.x - gu(GRENADE.range),
        "it stopped well short of the point: {from}"
    );
    let attack = t.sim().object(THROWER).unwrap().attack.clone().unwrap();
    assert_eq!(attack.location, None, "let go of as the grenade left");
    let lands_at = t.sim().projectiles()[0].lands_at;
    t.run_until(lands_at);
    assert_eq!(
        current(&t, THROWER),
        Some(OrderKind::AttackGround(at)),
        "waits for the blast"
    );
    t.step();
    assert_eq!(
        current(&t, THROWER),
        Some(OrderKind::Move(Target::Point(then))),
        "complete"
    );
    t.run(80);
    assert_eq!(launches(&t).len(), 1, "once, at tick {launched}");
}

/// `systems/controls/commands.md > AttackGround`: "Only available to units whose AttackType has CanTargetGround". The
/// gate refuses the order for a Peacekeeper (FullyConnected) with `Reject::CannotTargetGround`, and nothing changes;
/// a group with a thrower in it gives the order to the thrower alone. A point off the map is refused.
#[test]
fn it_is_refused_for_units_that_cant_target_the_ground() {
    let _grenadier = grenadier(GRENADE);
    let peacekeeper = ObjectId(2);
    let at = Vec2::new(500.0, 320.0);
    let twin = |order: Option<(Vec<ObjectId>, Vec2)>| {
        let mut t = ScenarioBuilder::new(&open()).start();
        t.spawn_unit_at(1, ObjectKind::Guard, P1, Vec2::new(100.0, 320.0))
            .spawn_unit_at(1, ObjectKind::Peacekeeper, P1, Vec2::new(100.0, 400.0));
        if let Some((units, at)) = order {
            t.order_at(2, P1, &units, OrderKind::AttackGround(at), false);
        }
        t.run_until(1);
        t
    };
    let t = twin(None);
    let order = |units: &[ObjectId], at| PlayerCommand {
        player: P1,
        command: Command::Order {
            units: units.to_vec(),
            kind: OrderKind::AttackGround(at),
            queue: false,
        },
    };
    assert_eq!(
        t.sim().validate(&order(&[peacekeeper], at)),
        Err(Reject::CannotTargetGround)
    );
    assert_eq!(t.sim().validate(&order(&[THROWER, peacekeeper], at)), Ok(()));
    let off = Vec2::new(-10.0, 320.0);
    assert_eq!(t.sim().validate(&order(&[THROWER], off)), Err(Reject::OffMap));
    let (mut refused, mut nothing) = (twin(Some((vec![peacekeeper], at))), twin(None));
    refused.run(5);
    nothing.run(5);
    assert_eq!(refused.sim().state_hash(), nothing.sim().state_hash(), "no change");
    let mut both = twin(Some((vec![THROWER, peacekeeper], at)));
    both.run(1);
    assert_eq!(current(&both, THROWER), Some(OrderKind::AttackGround(at)));
    assert_eq!(current(&both, peacekeeper), None, "the Peacekeeper ignores it");
}

/// Design change 0182, for a location (`systems/controls/behaviors.md > AttackingLocation`: as AttackingObject): a
/// thrower given a MinRange of 2 grid units, ordered to attack the ground 50 su from its edge, backs off straight away
/// from the point until it is beyond its MinRange, then fires at it.
#[test]
fn a_location_inside_the_min_range_is_backed_off_from() {
    let _grenadier = grenadier(AttackAttrs {
        min_range: 2.0,
        ..GRENADE
    });
    let start = Vec2::new(400.0, 320.0);
    let at = start + Vec2::new(18.0 + 50.0, 0.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, ObjectKind::Guard, P1, start)
        .order_at(2, P1, &[THROWER], OrderKind::AttackGround(at), false);
    while launches(&t).is_empty() {
        assert!(t.sim().tick() < 300, "it never fired\n{}", t.ascii(None));
        t.step();
        assert!(t.pos(1).x <= start.x, "away from the point");
    }
    assert!(reach(&t, at) >= gu(2.0), "beyond its MinRange: {}", reach(&t, at));
    assert!(reach(&t, at) <= gu(GRENADE.range));
    assert_eq!(launches(&t)[0].2, at);
}
