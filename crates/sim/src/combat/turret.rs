//! Units' turrets — slice M5-U1, plan decision T80. A unit with TurretAttributes (`UnitDef::turret`, design change
//! 0154: "the attack source is the turret if the unit has TurretAttributes") attacks from its turret:
//! `systems/combat.md > TurretSource`: "The turret rotates independently to aim while the unit base is free to move and
//! turn during all attack phases" (`super::base_is_free`). Its state is a [`TurretState`] (`AttackState::turret`).
//!
//! - **Its facing** ([`TurretState::facing`]) is relative to the base's: `systems/units.md > TurretAttributes`: "The
//!   turret is centered on the unit's facing direction, with TurnAngle split equally clockwise and counter-clockwise
//!   from center". The base's own turn (phase 5d) carries the turret, and the turret's facing stays within ±TurnAngle/2.
//! - **Its LockedTarget** ([`TurretState::locked`], `systems/controls/commands.md > TurretCommandState`) is what it
//!   engages. The base's behaviours set it where the design does, and the turret's own behaviour scans for one while
//!   nothing is locked, a Move's included (phase 5a, `orders::turret`).
//! - **It turns** ([`step`]) in phase 5e, before the attack phases (`super::phases`), toward what it aims at
//!   (`systems/controls/action_channels.md > TurretTurning`: it "applies the turret's turn rate each tick, constrained by
//!   TurnAngle"): at most its TurnRate, the shorter way round for a full circle, never out of its arc. Not in Firing nor
//!   in Cooldown (`systems/combat.md > AttackPhase`: their TurretSourceActions are None), and with nothing to aim at it
//!   holds its facing (`systems/controls/action_channels.md > TurretMaintaining`).
//! - **It attacks** (`super::phases`): a sequence starts at its LockedTarget once its owner sees it and it is within
//!   Range, beyond MinRange and within the arc (`systems/controls/action_channels.md > TurretAttackChannel`, whose
//!   values are the phases); Aiming counts from then, lasts until the turret faces the target as well (within
//!   `tuning::combat::FACING_TOLERANCE`, as a base's aim), and is cancelled once the target leaves the arc or the range
//!   (plan decision T80). An Attack Ground's location is aimed at the same way, its turret's arc included
//!   (`systems/controls/command_behaviors.md > AttackingLocation`: "Range/arc check is against the target location").
//!
//! **Not shared with the Laser Turret** (`super::structures`, plan decision T49), whose sim stays as it was: a
//! structure has no arc and no turn rate ("it fires in any direction"), its barrel is its object's facing, which it
//! snaps to its target, and it picks its own target in phase 5e, by the gap alone. Only the client's barrel is shared.
//!
//! A unit's turret has no state until it first locks a target or turns: a [`TurretState`] at its default is the turret
//! as a unit starts with it, centred with nothing locked. So the units without a turret, and the states saved before
//! turrets existed, hash and load as they did (plan decision T65).

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::phases::Shooter;
use super::targeting::Targets;
use super::{AttackPhase, AttackState};
use crate::defs::{ObjectKind, TurretAttrs, UnitDef};
use crate::ids::ObjectId;
use crate::math;
use crate::object::Object;
use crate::tuning::combat::FACING_TOLERANCE;

/// A unit's turret (module docs): `systems/units.md > Unit Instance > TurretCommandState`, and its facing.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TurretState {
    /// Its facing relative to the base's, radians, clockwise (`math`): 0 straight ahead, within ±TurnAngle/2.
    pub facing: f32,
    /// `systems/controls/commands.md > TurretCommandState > LockedTarget`: the object it engages.
    pub locked: Option<ObjectId>,
}

/// Where an attack comes from (`systems/combat.md > AttackSource`; design change 0154), for its sequence
/// (`super::phases::Shooter`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) enum Source {
    /// `systems/combat.md > UnitBaseSource`: the whole unit turns to face its target.
    Base,
    /// `systems/combat.md > TurretSource`: its turret, of these attributes, aims (module docs).
    Turret(TurretAttrs),
    /// A structure that attacks (plan decision T49: "A structure has no turn rate (it fires in any direction)"): it
    /// faces its target at once (`super::structures::face`).
    Structure,
}

impl Source {
    /// The source of the attack of an object whose unit attributes are `unit` (`None`: a structure's).
    pub(super) fn of(unit: Option<&UnitDef>) -> Self {
        match unit {
            Some(unit) => unit.turret.map_or(Source::Base, Source::Turret),
            None => Source::Structure,
        }
    }
}

/// The TurretAttributes of a `kind` unit, if it has a turret (`UnitDef::turret`).
pub(crate) fn attrs_of(kind: ObjectKind) -> Option<TurretAttrs> {
    kind.def().unit.as_ref()?.turret
}

/// Whether the arc of `attrs` is a full turn, which the turret turns round the shorter way (the Seeker's "360° (full
/// rotation)", `factions/cults/units/seeker.md > TurretAttributes`). Technical: an arc whose gap behind is narrower than
/// [`FACING_TOLERANCE`] counts as one, so that a TurnAngle of 360° converted to radians is one whatever its rounding.
fn full_turn(attrs: &TurretAttrs) -> bool {
    attrs.turn_angle + FACING_TOLERANCE >= math::TAU
}

/// The direction of `at` from `from`, relative to `facing`, radians in `(-PI, PI]`; `None` if `at` is right there.
fn relative(facing: f32, from: Vec2, at: Vec2) -> Option<f32> {
    let to = at - from;
    (to != Vec2::ZERO).then(|| math::wrap_angle(math::vector_angle(to) - facing))
}

/// Whether the point `at` is within the arc of a turret of `attrs` on a base at `from` facing `facing`: within
/// ±TurnAngle/2 of the base's facing, bounds included (`systems/units.md > TurretAttributes`; module docs). A point
/// right where the base is counts as in it.
pub(crate) fn in_arc(attrs: &TurretAttrs, facing: f32, from: Vec2, at: Vec2) -> bool {
    full_turn(attrs) || relative(facing, from, at).is_none_or(|angle| angle.abs() <= attrs.turn_angle / 2.0)
}

/// The direction a turret in `turret` faces on a base facing `facing`, radians in `(-PI, PI]`; the base's own for a
/// turret with no state yet, which is centred (module docs).
pub(crate) fn aim_facing(turret: Option<TurretState>, facing: f32) -> f32 {
    math::wrap_angle(facing + turret.map_or(0.0, |turret| turret.facing))
}

/// The direction the turret of `object` faces, radians in `(-PI, PI]`, if its type has a turret ([`aim_facing`]): for
/// the client's barrel.
pub fn turret_facing(object: &Object) -> Option<f32> {
    attrs_of(object.kind)?;
    let turret = object.attack.as_ref().and_then(|attack| attack.turret);
    Some(aim_facing(turret, object.facing))
}

/// `systems/controls/action_channels.md > TurretTurning`: one tick of the turret's turn toward the point `at`, on a base
/// at `from` facing `facing`, by at most its TurnRate, landing exactly on the direction once within it. A full turn's
/// turret goes the shorter way round; any other stays within ±TurnAngle/2, so it turns the only way its arc allows,
/// and toward a point outside the arc it turns as far as the arc's edge.
pub(super) fn turn(turret: &mut TurretState, attrs: &TurretAttrs, facing: f32, from: Vec2, at: Vec2) {
    let Some(wanted) = relative(facing, from, at) else {
        return;
    };
    if full_turn(attrs) {
        math::turn_toward(&mut turret.facing, wanted, attrs.turn_rate);
        return;
    }
    let half = attrs.turn_angle / 2.0;
    let (current, wanted) = (turret.facing.clamp(-half, half), wanted.clamp(-half, half));
    let turn = wanted - current;
    turret.facing = if turn.abs() <= attrs.turn_rate {
        wanted
    } else {
        current + attrs.turn_rate.copysign(turn)
    };
}

/// One tick of the turret of `state`'s unit in phase 5e, before its attack phases (`super::phases::run`), its base
/// facing `facing` after phase 5d: it lets go of a LockedTarget its attack can't target at all, as the phases drop
/// such a target (gone, itself, or in a domain its TargetDomain doesn't allow), then turns toward what it aims at
/// ([`turn`]).
/// - **What it aims at**: an Attack Ground's location while the unit engages one
///   (`systems/controls/command_behaviors.md > AttackingLocation`: "turret fires at location, not a unit"), else its
///   LockedTarget, at its centre. Placeholder (not in the design: a turret that turned toward a target its owner
///   doesn't see would show the player where the target is): only a LockedTarget its owner sees, by the vision the
///   last tick left (`systems/combat.md > ValidTarget` 2).
/// - Not while the phase it starts the tick in is Firing or Cooldown (`systems/combat.md > AttackPhase`:
///   TurretSourceActions None); in Aiming, Reloading and with no sequence it turns (Turning).
pub(super) fn step(state: &mut AttackState, attrs: &TurretAttrs, facing: f32, shooter: &Shooter, targets: &Targets) {
    if let Some(locked) = state.locked()
        && !targets.get(locked).is_some_and(|t| shooter.can_target(t))
    {
        state.lock(None);
    }
    let held = matches!(
        state.phase,
        Some(AttackPhase::Firing { .. } | AttackPhase::FiringAt { .. } | AttackPhase::Cooldown)
    );
    if held {
        return;
    }
    let at = match state.location {
        Some(at) => Some(at),
        None => (state.locked())
            .and_then(|id| targets.get(id))
            .filter(|t| shooter.sees(t))
            .map(|t| t.pos),
    };
    if let Some(at) = at {
        turn(state.turret.get_or_insert_default(), attrs, facing, shooter.pos, at);
    }
}

impl AttackState {
    /// (M5-U1) Its turret's LockedTarget (`systems/controls/commands.md > TurretCommandState`), if it has one (module
    /// docs).
    pub fn locked(&self) -> Option<ObjectId> {
        self.turret.and_then(|turret| turret.locked)
    }

    /// (M5-U1) The turret of a unit with TurretAttributes locks `target`, or nothing (module docs; the callers ask the
    /// unit's type). An aim at another target is interrupted (`systems/combat.md > Aiming`), as a new target interrupts
    /// a base's aim (`AttackState::engage`); a shot under way runs on, and a reload keeps counting.
    pub(crate) fn lock(&mut self, target: Option<ObjectId>) {
        if self.locked() == target {
            return;
        }
        self.turret.get_or_insert_default().locked = target;
        if matches!(self.phase, Some(AttackPhase::Aiming { target: aimed }) if Some(aimed) != target) {
            self.cancel();
        }
    }
}

#[cfg(test)]
mod perf;
#[cfg(test)]
pub(crate) mod testing;
#[cfg(test)]
mod tests;
