//! A unit's attack sequence, one tick at a time — `systems/combat.md > AttackPhase`.
//!
//! The tick's time goes to the first phase that hasn't run its full duration yet. A phase that has run it ends at
//! once, with its effect, and the next one begins, so each phase lasts exactly its duration in ticks, zero included.
//! A sequence that starts in tick `t` aims in ticks `t .. t + Aim`, and its hit lands at the end of tick
//! `t + Aim + Firing − 1`, the last tick of Firing (tick `t` if both are 0). A new sequence starts at most once per
//! tick, in the tick after the last one's Reloading ends, so a cycle takes Aim + Firing + Cooldown + Reload ticks.

use glam::Vec2;

use super::damage::{Hit, directional_multiplier};
use super::targeting::{TargetView, Targets, domain_allows, reaches};
use super::{AttackPhase, AttackState};
use crate::defs::{AttackAttrs, ObjectKind, UnitDef};
use crate::elevation::Elevation;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::math;
use crate::movement;
use crate::shape::Shape;
use crate::tuning::combat::FACING_TOLERANCE;
use crate::vision::PlayerVision;

/// The attacking unit, as its sequence needs it.
pub(super) struct Shooter<'a> {
    pub id: ObjectId,
    pub pos: Vec2,
    /// Its shape: Range is measured from its edge (`systems/combat.md > Range Measurement`).
    shape: Shape,
    elevation: Elevation,
    pub attrs: &'a AttackAttrs,
    /// (G) What is left of its TurnRate this tick (`systems/combat.md > LocomotionOrientationConstraints`), radians:
    /// one budget per tick, of which phase 5d may have spent some (`movement::Turns`). `f32::INFINITY` for the
    /// movement models whose turn rate depends on speed.
    turn_left: f32,
    /// The unit has a turret (`UnitDef::has_turret`, its TurretAttributes; design change 0154): the turret aims, not
    /// the base.
    turret: bool,
    /// Its owner's vision as of the end of the last tick; `None` for a unit without an owner, which sees nothing.
    sight: Option<&'a PlayerVision>,
}

impl<'a> Shooter<'a> {
    #[expect(
        clippy::too_many_arguments,
        reason = "the attacking object, split into the parts its sequence reads"
    )]
    pub(super) fn new(
        map: &Map,
        id: ObjectId,
        kind: ObjectKind,
        pos: Vec2,
        facing: f32,
        unit: &UnitDef,
        attrs: &'a AttackAttrs,
        sight: Option<&'a PlayerVision>,
        turned: f32,
    ) -> Self {
        Self {
            id,
            pos,
            shape: Shape::silhouette(kind, pos, facing),
            elevation: Elevation::of(map, kind, pos),
            attrs,
            turn_left: movement::turn_rate(&unit.movement) - turned,
            turret: unit.has_turret(),
            sight,
        }
    }

    /// Whether its attack can target `t` at all: not itself, and in a domain its TargetDomain allows
    /// (`systems/combat.md > ValidTarget` 3).
    fn can_target(&self, t: &TargetView) -> bool {
        t.id != self.id && domain_allows(self.attrs.target_domain, t.domain)
    }

    /// `systems/combat.md > ValidTarget` 2: whether its owner sees `t` (`PlayerVision::object_box_state`: `t` isn't
    /// hidden from the owner, plan decision T32, and a tile it covers is Visible, or it is revealed to the owner).
    /// Phase 5e reads the vision phase 5f left at the end of the last tick (`docs/movement.md > 4`).
    fn sees(&self, t: &TargetView) -> bool {
        let (lo, hi) = t.shape.bounds();
        self.sight.is_some_and(|v| v.sees_object_box(t.id, lo, hi, t.hiding))
    }

    /// Whether `t` is within its Range and beyond its MinRange from where it stands.
    fn reaches(&self, t: &TargetView) -> bool {
        reaches(self.attrs, &self.shape, self.elevation, &t.shape, t.elevation)
    }

    /// The direction to `t`'s centre, or `None` if it is right here.
    fn bearing(&self, t: &TargetView) -> Option<f32> {
        let to = t.pos - self.pos;
        (to != Vec2::ZERO).then(|| math::wrap_angle(math::vector_angle(to)))
    }

    /// Whether a unit facing `facing` faces `t` ([`FACING_TOLERANCE`]).
    fn faces(&self, facing: f32, t: &TargetView) -> bool {
        // TODO(turrets): a turret aims within its TurnAngle at its own TurnRate (`systems/units.md >
        // TurretAttributes`); turrets have no state yet, so they always face their target.
        if self.turret {
            return true;
        }
        self.bearing(t)
            .is_none_or(|wanted| math::wrap_angle(wanted - facing).abs() <= FACING_TOLERANCE)
    }

    /// `systems/combat.md > Aiming`: the unit turns towards `t`, at most its TurnRate this tick
    /// (`> LocomotionOrientationConstraints[TurnRateMovement]`, `Stationary + Turning`), landing exactly on the
    /// direction once within it. (G) The TurnRate is one budget per tick: the aim has what phase 5d left of it
    /// ([`Shooter::turn_left`]), and it turns a unit at most once per tick.
    fn turn_toward(&self, facing: &mut f32, t: &TargetView) {
        let Some(wanted) = self.bearing(t).filter(|_| !self.turret) else {
            return;
        };
        // TODO(movement models): only TurnRateMovement (every M1 unit) has a fixed rate; the others turn at once.
        math::turn_toward(facing, wanted, self.turn_left);
    }
}

/// The duration of `phase`, ticks (`systems/combat.md > AttackAttributes`).
fn duration(attrs: &AttackAttrs, phase: AttackPhase) -> u32 {
    match phase {
        AttackPhase::Aiming { .. } => attrs.aim_ticks,
        AttackPhase::Firing { .. } => attrs.firing_ticks,
        AttackPhase::Cooldown => attrs.cooldown_ticks,
        AttackPhase::Reloading => attrs.reload_ticks,
    }
}

/// One tick of a unit's attack sequence (module docs): drop a target it can't attack, start a sequence if the
/// weapon is ready and the target in range, then spend the tick in the current phase, ending the phases that have
/// run their course. `facing` is the unit's; hits go to the tick's list.
pub(super) fn run(
    state: &mut AttackState,
    facing: &mut f32,
    shooter: &Shooter,
    targets: &Targets,
    hits: &mut Vec<Hit>,
    events: &mut Vec<SimEvent>,
) {
    // A target this unit can't attack at all (gone, itself, or in a domain its attack can't reach) is dropped, and
    // the order that engaged it completes (`orders::update`). One its owner doesn't see stays engaged here: what
    // to do about it is the behaviour's (`orders::engage`: an Attack completes, design change 0200).
    if let Some(target) = state.target
        && !targets.get(target).is_some_and(|t| shooter.can_target(t))
    {
        state.engage(None);
    }
    // `systems/controls/behaviors.md > AttackingObject` 3: a sequence begins once the target is in range and no
    // sequence runs, and only at a target the unit's owner sees (`systems/combat.md > ValidTarget` 2).
    if state.phase.is_none()
        && let Some(target) = state.target
        && targets
            .get(target)
            .is_some_and(|t| shooter.sees(t) && shooter.reaches(t))
    {
        state.phase = Some(AttackPhase::Aiming { target });
        state.elapsed = 0;
    }
    let mut tick_left = true;
    while let Some(phase) = state.phase {
        let aimed_at = match phase {
            // `systems/combat.md > Aiming`: the target must stay valid throughout (`> ValidTarget`: seen by the
            // owner too) and in range, or the sequence is cancelled; the unit keeps turning to face it. A different
            // command already cancelled it (`AttackState::engage`).
            AttackPhase::Aiming { target } => {
                let valid = targets.get(target).filter(|t| {
                    state.target == Some(target) && shooter.can_target(t) && shooter.sees(t) && shooter.reaches(t)
                });
                let Some(t) = valid else {
                    state.cancel();
                    break;
                };
                Some(t)
            }
            AttackPhase::Firing { .. } | AttackPhase::Cooldown | AttackPhase::Reloading => None,
        };
        // Aiming also lasts until the unit faces its target (a placeholder reading: the design gives Aiming a
        // duration and a turn, and doesn't say which wins).
        let over =
            state.elapsed >= duration(shooter.attrs, phase) && aimed_at.is_none_or(|t| shooter.faces(*facing, t));
        if over {
            state.phase = end(phase, shooter, targets, hits, events);
            state.elapsed = 0;
            continue;
        }
        if !tick_left {
            break;
        }
        if let Some(t) = aimed_at {
            shooter.turn_toward(facing, t);
        }
        state.elapsed = state.elapsed.saturating_add(1);
        tick_left = false;
    }
}

/// `phase` has run its course: its effect, and the phase that follows (`None` after Reloading: the sequence is
/// over).
fn end(
    phase: AttackPhase,
    shooter: &Shooter,
    targets: &Targets,
    hits: &mut Vec<Hit>,
    events: &mut Vec<SimEvent>,
) -> Option<AttackPhase> {
    match phase {
        AttackPhase::Aiming { target } => Some(AttackPhase::Firing { target: Some(target) }),
        AttackPhase::Firing { target } => {
            if let Some(t) = target.and_then(|id| targets.get(id)) {
                fire(shooter, t, hits, events);
            }
            Some(AttackPhase::Cooldown)
        }
        AttackPhase::Cooldown => Some(AttackPhase::Reloading),
        AttackPhase::Reloading => None,
    }
}

/// `systems/combat.md > Firing`: the attack's effect takes place, at the end of the phase (a placeholder reading:
/// the design says only "during" it; TailDisjointed attacks apply theirs at its end). A FullyConnected attack can't
/// miss (`> FullyConnected`): it lands on the target wherever the target is now.
///
/// TODO(attack types): HeadDisjointed and DoublyDisjointed attacks launch a projectile instead, and the disjointed
/// ones may target a location; AoE attacks (`> AoEDamage`) hit every unit in their circle. No M1 unit has them: every
/// attack lands like this, as one single-target hit.
fn fire(shooter: &Shooter, t: &TargetView, hits: &mut Vec<Hit>, events: &mut Vec<SimEvent>) {
    let directional = if t.directional_armor {
        directional_multiplier(t.facing, shooter.pos - t.pos)
    } else {
        1.0
    };
    hits.push(Hit {
        attacker: shooter.id,
        target: t.id,
        damage: shooter.attrs.damage,
        pre_armor: t.pre_armor,
        point_armor: t.point_armor,
        directional,
    });
    events.push(SimEvent::Attacked {
        attacker: shooter.id,
        target: t.id,
        from: shooter.pos,
        to: t.pos,
    });
}

#[cfg(test)]
mod tests;
