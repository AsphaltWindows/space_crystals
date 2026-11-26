//! (M5-U1) The test turret units of plan U1's acceptance (`defs::test_defs`): the sim's tests give the Guard a turret
//! on a Mech's base (`systems/units.md > Mech`: it can carry a turret, turns in place, on TurnRateMovement), and a
//! gun with fixed numbers, so that they test the turret apart from any real unit's tuning. The real turret units, the
//! Seeker, the Sparrow and the Hand of Heaven, are slice M5-G2's data, which their slices move (V1, V2, H1). Also what
//! the turret tests read of a game.

use glam::Vec2;

use super::TurretState;
use crate::defs::test_defs::TestDef;
use crate::defs::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MovementAttrs, ObjectKind, TargetDomain, TargetType, TurretAttrs,
    UnitBase,
};
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::math;
use crate::orders::OrderKind;
use crate::testing::TestSim;
use crate::units::Tick;

/// The kind the test defs give a turret.
pub(crate) const TURRET_UNIT: ObjectKind = ObjectKind::Guard;

/// The test turret's gun: FullyConnected, so it can't miss, Universal, Range 3 grid units, MinRange 0; Aim 0, Firing
/// 1, Cooldown 1 and Reload 2 frames, a shot every 4 ticks. Damage 0.5, so that the targets live through the tests:
/// a Drone (MaxHP 20) takes 40 shots.
pub(crate) const GUN: AttackAttrs = AttackAttrs {
    attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
    target_domain: TargetDomain::Universal,
    target_type: TargetType::SingleTarget,
    damage: 0.5,
    range: 3.0,
    min_range: 0.0,
    projectile_speed: None,
    aim_ticks: 0,
    firing_ticks: 1,
    cooldown_ticks: 1,
    reload_ticks: 2,
};

/// The base's TurnRate, degrees a tick: slow, so that a test sees the base turn over several ticks. MaxSpeed 5 su a
/// tick, as the Guard's.
pub(crate) const BASE_TURN_DEGREES: f32 = 5.0;

/// A turret whose full arc is `arc` degrees, turning `rate` degrees a tick.
pub(crate) fn turret(arc: f32, rate: f32) -> TurretAttrs {
    TurretAttrs {
        turn_angle: math::degrees_to_radians(arc),
        turn_rate: math::degrees_to_radians(rate),
    }
}

/// The addendum's test unit: "a turret unit on TurnRate movement (a 90° arc at 10° a tick)".
pub(crate) fn ninety_at_ten() -> TestDef {
    turret_unit(turret(90.0, 10.0), GUN)
}

/// While the guard lives, the Guard is a turret unit with `turret` and `gun` ([`TURRET_UNIT`]).
pub(crate) fn turret_unit(turret: TurretAttrs, gun: AttackAttrs) -> TestDef {
    TestDef::unit(TURRET_UNIT, |unit| {
        unit.base = UnitBase::Mech;
        unit.movement = MovementAttrs::TurnRate {
            turn_rate: math::degrees_to_radians(BASE_TURN_DEGREES),
            acceleration: None,
            deceleration: None,
            max_speed: 5.0,
        };
        unit.turret = Some(turret);
        unit.attack = Some(gun);
    })
}

/// `wide` × `high` open tiles.
pub(crate) fn open(wide: usize, high: usize) -> String {
    format!("{}\n", ".".repeat(wide)).repeat(high)
}

/// The point `distance` su from `from` in the direction `degrees` (clockwise from east, `math`).
pub(crate) fn toward(from: Vec2, degrees: f32, distance: f32) -> Vec2 {
    from + math::facing_vector(math::degrees_to_radians(degrees)) * distance
}

/// The shots `attacker` fired so far (`SimEvent::Attacked`): the tick and the target of each.
pub(crate) fn shots(t: &TestSim, attacker: ObjectId) -> Vec<(Tick, ObjectId)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match *e {
            SimEvent::Attacked {
                attacker: a, target, ..
            } if a == attacker => Some((*tick, target)),
            _ => None,
        })
        .collect()
}

/// Unit `id`'s turret, if it has any state yet.
pub(crate) fn turret_of(t: &TestSim, id: ObjectId) -> Option<TurretState> {
    let o = t.sim().object(id).expect("the unit");
    o.attack.as_ref().expect("a unit that attacks").turret
}

/// Unit `id`'s turret's LockedTarget.
pub(crate) fn locked(t: &TestSim, id: ObjectId) -> Option<ObjectId> {
    turret_of(t, id).and_then(|turret| turret.locked)
}

/// Unit `id`'s facing, degrees.
pub(crate) fn facing_degrees(t: &TestSim, id: ObjectId) -> f32 {
    t.sim().object(id).expect("the unit").facing.to_degrees()
}

/// Unit `id`'s current order, if any.
pub(crate) fn current(t: &TestSim, id: ObjectId) -> Option<OrderKind> {
    let orders = t.sim().object(id).expect("the unit").orders.as_ref().expect("a unit");
    orders.current.as_ref().map(|active| active.order.kind)
}
