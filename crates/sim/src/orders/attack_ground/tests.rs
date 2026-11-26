//! Plan §3.3 O's acceptance, Attack Ground: "The order walks into range", and "it is refused for a FullyConnected
//! unit"; and design change 0293's verification (answering Q0083 point 3, which replaced the acceptance's "fires once,
//! and completes at the blast"): "An Attack Ground keeps shelling until Stop". With the test grenade
//! (`combat::testing`).

use glam::Vec2;

use crate::combat::testing::{GRENADE, grenadier};
use crate::command::{Command, PlayerCommand};
use crate::defs::{AttackAttrs, ObjectKind};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::mover::MoveStatus;
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

/// The blasts so far: their ticks.
fn blasts(t: &TestSim) -> Vec<Tick> {
    (t.events().iter())
        .filter(|(_, e)| matches!(e, SimEvent::Blast { .. }))
        .map(|(tick, _)| *tick)
        .collect()
}

fn current(t: &TestSim, id: ObjectId) -> Option<OrderKind> {
    let orders = t.sim().object(id).unwrap().orders.as_ref().unwrap();
    orders.current.as_ref().map(|active| active.order.kind)
}

fn queued(t: &TestSim, id: ObjectId) -> Vec<OrderKind> {
    let orders = t.sim().object(id).unwrap().orders.as_ref().unwrap();
    orders.queue.iter().map(|order| order.kind).collect()
}

/// The gap from the thrower's shape to `at`: what its Range is measured against.
fn reach(t: &TestSim, at: Vec2) -> f32 {
    Shape::of(t.sim().object(THROWER).unwrap()).distance_to(at)
}

/// A thrower 600 su from the point clicked walks towards it (MovingToLocation), stops in range (4 grid units from its
/// edge) and fires at the point itself. Design change 0293's verification, "An Attack Ground keeps shelling until
/// Stop" (`systems/controls/command_behaviors.md > AttackingLocation`: "the behavior repeats its attack cycle at the
/// location until the player gives another order (or Stop)"): it throws again every cycle (Aim 6, Firing 2, Cooldown 4
/// and Reload 28 ticks) at the same point, long after its first blast, its order current all along and the Move
/// Shift-queued behind it still waiting ("Queued orders after an Attack Ground therefore never start on their own").
/// The Stop ends it: no grenade leaves after it, the one in the air lands, and the queued Move never starts.
#[test]
fn it_walks_into_range_and_keeps_shelling_until_stop() {
    let _grenadier = grenadier(GRENADE);
    let start = Vec2::new(100.0, 320.0);
    let at = Vec2::new(700.0, 333.5);
    let then = OrderKind::Move(Target::Point(Vec2::new(100.0, 500.0)));
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, ObjectKind::Guard, P1, start)
        .order_at(2, P1, &[THROWER], OrderKind::AttackGround(at), false)
        .order_at(2, P1, &[THROWER], then, true);
    t.run_until(3);
    assert!(t.status(1) == MoveStatus::Moving, "it walks");
    while launches(&t).is_empty() {
        assert!(t.sim().tick() < 300, "it never fired\n{}", t.ascii(None));
        t.step();
        assert_eq!(current(&t, THROWER), Some(OrderKind::AttackGround(at)));
    }
    let [(first, from, _)] = launches(&t)[..] else {
        panic!("one launch");
    };
    assert!(reach(&t, at) <= gu(GRENADE.range), "in range: {}", reach(&t, at));
    assert!(
        from.x < at.x - gu(GRENADE.range),
        "it stopped well short of the point: {from}"
    );
    let cycle = GRENADE.aim_ticks + GRENADE.firing_ticks + GRENADE.cooldown_ticks + GRENADE.reload_ticks;
    t.run_until(first + 4 * cycle);
    let thrown: Vec<(Tick, Vec2, Vec2)> = (0..5).map(|i| (first + i * cycle, from, at)).collect();
    assert_eq!(
        launches(&t),
        thrown,
        "one grenade a cycle, from where it stands, at the point clicked"
    );
    assert_eq!(blasts(&t).len(), 4, "the first four have landed");
    assert_eq!(current(&t, THROWER), Some(OrderKind::AttackGround(at)), "it goes on");
    assert_eq!(queued(&t, THROWER), [then], "the queued Move waits");
    let attack = t.sim().object(THROWER).unwrap().attack.clone().unwrap();
    assert_eq!(attack.location, Some(at), "the location stays engaged");

    let stopped = t.sim().tick() + 1;
    t.order_at(stopped, P1, &[THROWER], OrderKind::Stop, false)
        .run(3 * cycle);
    assert_eq!(launches(&t).len(), 5, "none after the Stop");
    assert_eq!(blasts(&t).len(), 5, "the last one landed");
    assert_eq!(current(&t, THROWER), None, "stopped");
    assert_eq!(queued(&t, THROWER), [], "the queued Move never started");
    assert_eq!(t.pos(1), from, "where it stood");
}

/// `systems/controls/commands.md > AttackGround`: "Only available to units whose AttackType has CanTargetGround". The
/// gate refuses the order for a Peacekeeper (FullyConnected) with `Reject::CannotTargetGround`, and nothing changes;
/// a group with a thrower in it gives the order to the thrower alone. (M5-H1) A point off the map is taken onto it,
/// to the nearest point of the nearest edge tile (plan decision T81, `Sim::check_order`); one that isn't finite is
/// refused.
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
    assert_eq!(t.sim().validate(&order(&[THROWER], off)), Ok(()));
    assert_eq!(
        t.sim().validate(&order(&[THROWER], Vec2::new(f32::NAN, 320.0))),
        Err(Reject::OffMap)
    );
    let mut edge = twin(Some((vec![THROWER], off)));
    edge.run(1);
    assert_eq!(
        current(&edge, THROWER),
        Some(OrderKind::AttackGround(Vec2::new(0.0, 320.0)))
    );
    let (mut refused, mut nothing) = (twin(Some((vec![peacekeeper], at))), twin(None));
    refused.run(5);
    nothing.run(5);
    assert_eq!(refused.sim().state_hash(), nothing.sim().state_hash(), "no change");
    let mut both = twin(Some((vec![THROWER, peacekeeper], at)));
    both.run(1);
    assert_eq!(current(&both, THROWER), Some(OrderKind::AttackGround(at)));
    assert_eq!(current(&both, peacekeeper), None, "the Peacekeeper ignores it");
}

/// Design change 0182, for a location (`systems/controls/command_behaviors.md > AttackingLocation`: as
/// AttackingObject): a thrower given a MinRange of 2 grid units, ordered to attack the ground 50 su from its edge,
/// backs off straight away from the point until it is beyond its MinRange, then fires at it.
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
