//! A unit's attack sequence, one tick at a time — `systems/combat.md > AttackPhase`.
//!
//! The tick's time goes to the first phase that hasn't run its full duration yet. A phase that has run it ends at
//! once, with its effect, and the next one begins, so each phase lasts exactly its duration in ticks, zero included.
//! A sequence that starts in tick `t` aims in ticks `t .. t + Aim`, and its hit lands at the end of tick
//! `t + Aim + Firing − 1`, the last tick of Firing (tick `t` if both are 0). A new sequence starts at most once per
//! tick, in the tick after the last one's Reloading ends, so a cycle takes Aim + Firing + Cooldown + Reload ticks.
//!
//! (M4-O, plan decision T48) A DoublyDisjointed attack (`systems/combat.md > DoublyDisjointed`) aims at an object or at
//! a location ([`AttackState::location`]), and locks the location at the end of Aiming: where its target stands then
//! ("the location the target was at at the end of the Aiming phase"), or the location aimed at. At the end of its
//! Firing it launches a projectile there (`super::projectiles`), which lands where it was locked, whatever the target
//! did since. TODO(attack types): the HeadDisjointed and TailDisjointed attacks aren't built (plan §3.3 O: no M4 kind
//! has one). They hit as a FullyConnected attack does, and a TailDisjointed one, which may be given an Attack Ground
//! ([`can_target_ground`]), lets go of its location.

use glam::Vec2;

use super::damage::{Hit, directional_multiplier};
use super::projectiles::Projectile;
use super::targeting::{TargetView, Targets, domain_allows, reaches, reaches_point};
use super::{AttackPhase, AttackState};
use crate::defs::{AttackAttrs, AttackType, ObjectKind, UnitDef};
use crate::elevation::Elevation;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::math;
use crate::movement;
use crate::shape::Shape;
use crate::tuning::combat::FACING_TOLERANCE;
use crate::units::Tick;
use crate::vision::PlayerVision;

/// What one tick of attack sequences puts out (`super::step`): the hits of single-target attacks, for the tick's list,
/// and the projectiles launched, for the store (plan decision T48).
pub(super) struct Volley {
    pub tick: Tick,
    pub hits: Vec<Hit>,
    pub launched: Vec<Projectile>,
}

impl Volley {
    pub(super) fn new(tick: Tick) -> Self {
        Self {
            tick,
            hits: Vec::new(),
            launched: Vec::new(),
        }
    }
}

/// The attacking unit, as its sequence needs it.
pub(super) struct Shooter<'a> {
    pub id: ObjectId,
    /// Its owner, whom a projectile it launches remembers (`super::projectiles`).
    owner: Option<PlayerId>,
    pub pos: Vec2,
    /// Its shape: Range is measured from its edge (`systems/combat.md > Range Measurement`).
    shape: Shape,
    elevation: Elevation,
    /// For the ground's elevation at a location it aims at.
    map: &'a Map,
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
        map: &'a Map,
        id: ObjectId,
        owner: Option<PlayerId>,
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
            owner,
            pos,
            shape: Shape::silhouette(kind, pos, facing),
            elevation: Elevation::of(map, kind, pos),
            map,
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

    /// (M4-O) Whether its attack locks a location at the end of Aiming and launches a projectile there, and may aim
    /// at a location: a DoublyDisjointed one (module docs).
    fn locks(&self) -> bool {
        matches!(self.attrs.attack_type, AttackType::DoublyDisjointed)
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

    /// (M4-O) Whether the location `at` is within its Range and beyond its MinRange from where it stands
    /// (`systems/combat.md > Range Measurement`: "measured from the attacker's shape to the target point").
    fn reaches_point(&self, at: Vec2) -> bool {
        reaches_point(
            self.attrs,
            &self.shape,
            self.elevation,
            at,
            Elevation::ground(self.map, at),
        )
    }

    /// The direction to `at`, or `None` if it is right here.
    fn bearing(&self, at: Vec2) -> Option<f32> {
        let to = at - self.pos;
        (to != Vec2::ZERO).then(|| math::wrap_angle(math::vector_angle(to)))
    }

    /// Whether a unit facing `facing` faces the point `at`, its target's centre or a location ([`FACING_TOLERANCE`]).
    fn faces(&self, facing: f32, at: Vec2) -> bool {
        // TODO(turrets): a turret aims within its TurnAngle at its own TurnRate (`systems/units.md >
        // TurretAttributes`); turrets have no state yet, so they always face their target.
        if self.turret {
            return true;
        }
        self.bearing(at)
            .is_none_or(|wanted| math::wrap_angle(wanted - facing).abs() <= FACING_TOLERANCE)
    }

    /// `systems/combat.md > Aiming`: the unit turns towards the point `at`, at most its TurnRate this tick
    /// (`> LocomotionOrientationConstraints[TurnRateMovement]`, `Stationary + Turning`), landing exactly on the
    /// direction once within it. (G) The TurnRate is one budget per tick: the aim has what phase 5d left of it
    /// ([`Shooter::turn_left`]), and it turns a unit at most once per tick.
    fn turn_toward(&self, facing: &mut f32, at: Vec2) {
        let Some(wanted) = self.bearing(at).filter(|_| !self.turret) else {
            return;
        };
        // TODO(movement models): only TurnRateMovement (every M1 unit) has a fixed rate; the others turn at once.
        math::turn_toward(facing, wanted, self.turn_left);
    }
}

/// The duration of `phase`, ticks (`systems/combat.md > AttackAttributes`).
fn duration(attrs: &AttackAttrs, phase: AttackPhase) -> u32 {
    match phase {
        AttackPhase::Aiming { .. } | AttackPhase::AimingAt { .. } => attrs.aim_ticks,
        AttackPhase::Firing { .. } | AttackPhase::FiringAt { .. } => attrs.firing_ticks,
        AttackPhase::Cooldown => attrs.cooldown_ticks,
        AttackPhase::Reloading => attrs.reload_ticks,
    }
}

/// One tick of a unit's attack sequence (module docs): drop a target it can't attack, start a sequence if the
/// weapon is ready and the target in range, then spend the tick in the current phase, ending the phases that have
/// run their course. `facing` is the unit's; hits and projectiles go to `volley`.
pub(super) fn run(
    state: &mut AttackState,
    facing: &mut f32,
    shooter: &Shooter,
    targets: &Targets,
    volley: &mut Volley,
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
    // (M4-O) So is a location, for an attack that doesn't aim at one (module docs): the gate gives an Attack Ground
    // only to attacks that can target the ground, but a weapon may change under an order.
    if state.location.is_some() && !shooter.locks() {
        state.engage(None);
    }
    // `systems/controls/behaviors.md > AttackingObject` 3: a sequence begins once the target is in range and no
    // sequence runs, and only at a target the unit's owner sees (`systems/combat.md > ValidTarget` 2).
    // `systems/controls/behaviors.md > AttackingLocation`: the same at a location in range, which no one needs to see.
    if state.phase.is_none() {
        if let Some(target) = state.target
            && targets
                .get(target)
                .is_some_and(|t| shooter.sees(t) && shooter.reaches(t))
        {
            state.phase = Some(AttackPhase::Aiming { target });
            state.elapsed = 0;
        } else if let Some(at) = state.location
            && shooter.reaches_point(at)
        {
            state.phase = Some(AttackPhase::AimingAt { at });
            state.elapsed = 0;
        }
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
                Some(t.pos)
            }
            // (M4-O) The same at a location: still the one engaged, and in range (`systems/controls/behaviors.md >
            // AttackingLocation`: "range/arc check against location").
            AttackPhase::AimingAt { at } => {
                if state.location != Some(at) || !shooter.reaches_point(at) {
                    state.cancel();
                    break;
                }
                Some(at)
            }
            AttackPhase::Firing { .. }
            | AttackPhase::FiringAt { .. }
            | AttackPhase::Cooldown
            | AttackPhase::Reloading => None,
        };
        // Aiming also lasts until the unit faces its target (a placeholder reading: the design gives Aiming a
        // duration and a turn, and doesn't say which wins).
        let over =
            state.elapsed >= duration(shooter.attrs, phase) && aimed_at.is_none_or(|at| shooter.faces(*facing, at));
        if over {
            state.phase = end(phase, aimed_at, shooter, targets, volley, events);
            state.elapsed = 0;
            // `// Placeholder: Q0083 point 3`: an Attack Ground fires once. Its location is let go of as its one
            // projectile launches, and its behaviour completes at the blast (`orders::attack_ground`).
            if let AttackPhase::FiringAt { at, target: None } = phase
                && state.location == Some(at)
            {
                state.location = None;
            }
            continue;
        }
        if !tick_left {
            break;
        }
        if let Some(at) = aimed_at {
            shooter.turn_toward(facing, at);
        }
        state.elapsed = state.elapsed.saturating_add(1);
        tick_left = false;
    }
}

/// `phase` has run its course: its effect, and the phase that follows (`None` after Reloading: the sequence is
/// over). `aimed_at`: where an Aiming phase was aimed, its target's centre or its location.
fn end(
    phase: AttackPhase,
    aimed_at: Option<Vec2>,
    shooter: &Shooter,
    targets: &Targets,
    volley: &mut Volley,
    events: &mut Vec<SimEvent>,
) -> Option<AttackPhase> {
    match phase {
        AttackPhase::Aiming { target } => match aimed_at {
            // (M4-O) The location is locked where the target stands now (module docs).
            Some(at) if shooter.locks() => Some(AttackPhase::FiringAt {
                at,
                target: Some(target),
            }),
            _ => Some(AttackPhase::Firing { target: Some(target) }),
        },
        AttackPhase::AimingAt { at } => Some(AttackPhase::FiringAt { at, target: None }),
        AttackPhase::Firing { target } => {
            if let Some(t) = target.and_then(|id| targets.get(id)) {
                fire(shooter, t, &mut volley.hits, events);
            }
            Some(AttackPhase::Cooldown)
        }
        AttackPhase::FiringAt { at, target } => {
            launch(shooter, at, target, volley, events);
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
/// TODO(attack types): a HeadDisjointed attack launches a projectile that tracks its target instead (`>
/// HeadDisjointed`); no unit has one, and it lands like this, as one single-target hit.
fn fire(shooter: &Shooter, t: &TargetView, hits: &mut Vec<Hit>, events: &mut Vec<SimEvent>) {
    let directional = if t.directional_armor {
        directional_multiplier(t.facing, shooter.pos - t.pos)
    } else {
        1.0
    };
    hits.push(Hit::single(
        shooter.id,
        t.id,
        shooter.attrs.damage,
        t.pre_armor,
        t.point_armor,
        directional,
    ));
    events.push(SimEvent::Attacked {
        attacker: shooter.id,
        target: t.id,
        from: shooter.pos,
        to: t.pos,
    });
}

/// (M4-O) `systems/combat.md > DoublyDisjointed`: "The Firing phase is short and spawns a projectile which travels to
/// the location the target was at at the end of the Aiming phase": at the end of Firing, from the shooter's centre to
/// `at`, the location locked (plan decision T48). `target`: the object it was aimed at, if any.
fn launch(shooter: &Shooter, at: Vec2, target: Option<ObjectId>, volley: &mut Volley, events: &mut Vec<SimEvent>) {
    let projectile = Projectile::launch(
        shooter.id,
        shooter.owner,
        shooter.attrs,
        shooter.pos,
        at,
        target,
        volley.tick,
    );
    events.push(projectile.launched());
    volley.launched.push(projectile);
}

#[cfg(test)]
mod tests;
