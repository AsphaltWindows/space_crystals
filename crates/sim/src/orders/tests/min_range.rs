//! Design change 0182's Verification (`systems/controls/behaviors.md > BaseAutoTargeting > Targets inside MinRange`,
//! `> AttackingObject > Target inside MinRange`), with a Peacekeeper given a MinRange of 2 grid units (Range 4), and
//! for the deployed Artillery Rig, which doesn't exist yet, one that can't move either (`defs::test_defs`); and the
//! Gunner, the first kind with a MinRange of its own (slice M5-A), as one that can't move.

use glam::Vec2;

use crate::command::DebugCommand;
use crate::defs::test_defs::TestDef;
use crate::defs::{MovementAttrs, ObjectKind};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::mover::{MoveGoal, MoveStatus};
use crate::orders::OrderKind;
use crate::shape::Shape;
use crate::testing::{ScenarioBuilder, TestSim};
use crate::units::{Tick, gu};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const UNIT: ObjectId = ObjectId(1);
const NEAR: ObjectId = ObjectId(2);
const FAR: ObjectId = ObjectId(3);
const MIN_RANGE_GU: f32 = 2.0;

/// 16 × 10 open tiles.
fn open() -> String {
    "................\n".repeat(10)
}

/// The Peacekeeper with a MinRange; `mobile: false`, one that can't move (a MaxSpeed of 0: `Mover::can_move`).
fn min_range_peacekeeper(mobile: bool) -> TestDef {
    TestDef::unit(ObjectKind::Peacekeeper, |unit| {
        unit.attack.as_mut().unwrap().min_range = MIN_RANGE_GU;
        if !mobile && let MovementAttrs::TurnRate { max_speed, .. } = &mut unit.movement {
            *max_speed = 0.0;
        }
    })
}

fn gap(t: &TestSim, a: ObjectId, b: ObjectId) -> f32 {
    let shape = |id| Shape::of(t.sim().object(id).expect("exists"));
    shape(a).gap(&shape(b))
}

/// The ticks of `attacker`'s attacks on `target`.
fn attacks(t: &TestSim, attacker: ObjectId, target: ObjectId) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(
            |(_, e)| matches!(e, SimEvent::Attacked { attacker: a, target: b, .. } if *a == attacker && *b == target),
        )
        .map(|&(tick, _)| tick)
        .collect()
}

fn current(t: &TestSim, id: ObjectId) -> Option<OrderKind> {
    let orders = t.sim().object(id).unwrap().orders.as_ref().unwrap();
    orders.current.as_ref().map(|active| active.order.kind)
}

/// "Mobile MinRange unit ordered to attack an adjacent enemy: it backs away, then fires." The Peacekeeper stands 16
/// su from a Guard of player 2 (auto-targeting off: the Guard doesn't fight back) and is ordered to attack it. It
/// walks straight away from the Guard, never firing while the Guard is inside its MinRange, and once it is out
/// of it, it stops, within its Range, and fires, its Attack order running all the while.
#[test]
fn a_mobile_unit_backs_off_from_a_target_inside_its_min_range_then_fires() {
    let _pk = min_range_peacekeeper(true);
    let start = Vec2::new(400.0, 320.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P1, start)
        .spawn_unit_at(1, ObjectKind::Guard, P2, start + Vec2::new(12.0 + 16.0 + 18.0, 0.0))
        .order_at(2, P1, &[UNIT], OrderKind::Attack(NEAR), false)
        .run_until(2);
    assert!(gap(&t, UNIT, NEAR) < gu(MIN_RANGE_GU));
    while attacks(&t, UNIT, NEAR).is_empty() {
        assert!(t.sim().tick() < 120, "it never fired\n{}", t.ascii(None));
        let before = gap(&t, UNIT, NEAR);
        t.step();
        assert_eq!(
            current(&t, UNIT),
            Some(OrderKind::Attack(NEAR)),
            "tick {}",
            t.sim().tick()
        );
        if before < gu(MIN_RANGE_GU) {
            assert!(t.pos(1).x <= start.x, "tick {}: away from the Guard", t.sim().tick());
        }
    }
    let fired = gap(&t, UNIT, NEAR);
    assert!(
        fired >= gu(MIN_RANGE_GU) && fired <= gu(4.0),
        "fired across a gap of {fired}"
    );
    assert!(
        t.pos(1).x < start.x && t.pos(1).y == start.y,
        "straight back: {}",
        t.pos(1)
    );
    let stood = t.pos(1);
    t.run(40);
    assert_eq!(t.pos(1), stood, "it stands while it attacks");
    assert!(attacks(&t, UNIT, NEAR).len() >= 2);
}

/// The back-off is a walk to a point. When its target leaves the game, the removal path ends the order, and the walk
/// ends at the unit's next turn in 5a (`orders::engage::idle`): "the behavior completes, and the unit stops where it
/// is" (`systems/controls/behaviors.md > MovingToObject > Completion`). It doesn't walk on to the end of its back-off
/// without an order, as a walk to a last seen position once did (slice M3-G2, `behaviours_soak`'s seed 17; design
/// change 0200 removed that walk, and slice M3-L3 the stop M3-G2 gave it in `sim/removal.rs::forget`).
#[test]
fn a_back_off_ends_where_the_unit_is_when_the_target_leaves_the_game() {
    let _pk = min_range_peacekeeper(true);
    let start = Vec2::new(400.0, 320.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P1, start)
        .spawn_unit_at(1, ObjectKind::Guard, P2, start + Vec2::new(12.0 + 16.0 + 18.0, 0.0))
        .order_at(2, P1, &[UNIT], OrderKind::Attack(NEAR), false)
        .run_until(6);
    let mover = t.mover(1);
    assert!(
        matches!(mover.goal, MoveGoal::Point(_)) && mover.status == MoveStatus::Moving,
        "it backs off: {:?}",
        mover.goal
    );
    assert!(t.pos(1).x < start.x);
    let there = t.pos(1);
    t.debug_at(7, DebugCommand::Remove { id: NEAR });
    t.step();
    assert_eq!(current(&t, UNIT), None, "the order ended with its target");
    assert_eq!((t.mover(1).goal, t.mover(1).status), (MoveGoal::None, MoveStatus::Idle));
    assert_eq!(t.pos(1), there, "where it was");
    t.run(30);
    assert_eq!(t.pos(1), there, "and it stays");
}

/// "Deployed Artillery Rig: feedback, order ends." The Peacekeeper that can't move is ordered to attack a Guard
/// that hugs it, with a second Attack queued, on a Guard within its Range. The first order fails in its first tick
/// with a `SimEvent::TargetTooClose` (the client's "Target too close"); the unit never fires at the near Guard, the
/// queued order starts in the next tick and the unit fires at the far one.
#[test]
fn an_immobile_unit_fails_its_attack_on_a_target_too_close() {
    let _rig = min_range_peacekeeper(false);
    let at = Vec2::new(400.0, 320.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P1, at)
        .spawn_unit_at(1, ObjectKind::Guard, P2, at + Vec2::new(12.0 + 8.0 + 18.0, 0.0))
        .spawn_unit_at(1, ObjectKind::Guard, P2, at + Vec2::new(0.0, 12.0 + 3.0 * 64.0 + 18.0))
        .order_at(2, P1, &[UNIT], OrderKind::Attack(NEAR), false)
        .order_at(2, P1, &[UNIT], OrderKind::Attack(FAR), true)
        .run_until(2);
    let failed: Vec<&(Tick, SimEvent)> = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::TargetTooClose { .. }))
        .collect();
    assert_eq!(
        failed,
        [&(
            2,
            SimEvent::TargetTooClose {
                unit: UNIT,
                target: NEAR
            }
        )]
    );
    assert_eq!(current(&t, UNIT), None, "the order ended");
    t.run_until(3);
    assert_eq!(
        current(&t, UNIT),
        Some(OrderKind::Attack(FAR)),
        "the queued one goes on"
    );
    t.run_until(30);
    assert_eq!(attacks(&t, UNIT, NEAR), [] as [Tick; 0]);
    assert!(!attacks(&t, UNIT, FAR).is_empty());
    assert_eq!(t.pos(1), at, "it never moved");
}

/// "Auto-targeting ignores enemies hugging the Rig." The Peacekeeper that can't move, idle, has a Guard of player 2
/// hugging it and another 3 grid units off, beyond its MinRange and within its Range: it picks the far one, though the
/// near one is closer and right in front of it. So does one on Hold, which scans its weapon's reach.
#[test]
fn auto_targeting_skips_enemies_inside_the_min_range() {
    let _rig = min_range_peacekeeper(false);
    for hold in [false, true] {
        let at = Vec2::new(400.0, 320.0);
        let mut t = ScenarioBuilder::new(&open()).start();
        t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, at)
            .spawn_unit_at(1, ObjectKind::Guard, P2, at + Vec2::new(12.0 + 8.0 + 18.0, 0.0))
            .spawn_unit_at(1, ObjectKind::Guard, P2, at + Vec2::new(0.0, 12.0 + 3.0 * 64.0 + 18.0))
            .order_at(2, P2, &[NEAR, FAR], OrderKind::Hold, false);
        if hold {
            t.order_at(2, P1, &[UNIT], OrderKind::Hold, false);
        }
        t.run_until(12);
        let target = t.sim().object(UNIT).unwrap().attack.as_ref().unwrap().target;
        assert_eq!(target, Some(FAR), "hold: {hold}");
        assert_eq!(attacks(&t, UNIT, NEAR), [] as [Tick; 0], "hold: {hold}");
        assert!(!attacks(&t, UNIT, FAR).is_empty(), "hold: {hold}");
    }
}

/// Slice M5-A: the Gunner (`factions/cults/units/gunner.md`: "Range: 5 grid units", "MinRange: 2"), the first kind of
/// the design with a MinRange, its own def but for a MaxSpeed of 0: a Gunner that can't move, as a garrisoned one will
/// be (`factions/cults/structures/garrison.md > Garrisoning`: "Militants and Gunners use their own AttackAttributes";
/// none can be yet). Ordered to attack a Peacekeeper of player 2 that hugs it, it fails the order in its first tick
/// with a `SimEvent::TargetTooClose`, and never fires at it. One that can move backs off instead
/// (`crates/sim/tests/cults_combat.rs`).
#[test]
fn a_gunner_that_cant_move_fails_its_attack_on_a_target_too_close() {
    let _rooted = TestDef::unit(ObjectKind::Gunner, |unit| {
        if let MovementAttrs::TurnRate { max_speed, .. } = &mut unit.movement {
            *max_speed = 0.0;
        }
    });
    let attack = ObjectKind::Gunner.def().unit.as_ref().unwrap().attack.unwrap();
    assert_eq!(
        (attack.min_range, attack.range),
        (MIN_RANGE_GU, 5.0),
        "the Gunner's own"
    );
    let at = Vec2::new(400.0, 320.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Gunner, P1, at)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, at + Vec2::new(12.0 + 8.0 + 12.0, 0.0))
        .order_at(2, P1, &[UNIT], OrderKind::Attack(NEAR), false)
        .run_until(2);
    assert!(gap(&t, UNIT, NEAR) < gu(MIN_RANGE_GU));
    let failed: Vec<&(Tick, SimEvent)> = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::TargetTooClose { .. }))
        .collect();
    let too_close = SimEvent::TargetTooClose {
        unit: UNIT,
        target: NEAR,
    };
    assert_eq!(failed, [&(2, too_close)]);
    assert_eq!(current(&t, UNIT), None, "the order ended");
    t.run_until(40);
    assert_eq!(attacks(&t, UNIT, NEAR), [] as [Tick; 0]);
    assert_eq!(t.pos(1), at, "it never moved");
}
