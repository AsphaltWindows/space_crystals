//! A unit's attack sequence, one tick at a time — `systems/combat.md > AttackPhase`. A structure that attacks runs the
//! same sequence (plan decision T49, slice M4-S1: the Laser Turret): it fires in any direction. (M5-U1) So does a unit
//! that attacks from its turret (`super::turret`), at its LockedTarget, which its turret turns toward before the
//! phases run; at a target within its arc, and Aiming lasts until the turret faces it.
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
//! did since. A location stays engaged after the launch, so its sequences follow one another as an attack on a target
//! that never dies does: an Attack Ground keeps firing until another order (design change 0293, answering Q0083
//! point 3; `orders::attack_ground`).
//!
//! (M5-O3) A TailDisjointed attack (`systems/combat.md > TailDisjointed`: the Cults' Gunner's, design change 0304)
//! aims, locks and stays engaged at a location the same way, with no projectile: "The effect applies to whatever units
//! are at the target location at the end of the Firing phase" ([`strike`]). A target that has left the spot by then
//! is missed ("Targets can dodge by moving out of the locked location"), and whatever stands there is hit in its
//! place, whoever owns it. TODO(attack types): the HeadDisjointed attack isn't built (no kind has one): it hits as a
//! FullyConnected attack does.

use glam::Vec2;

use super::damage::Hit;
use super::projectiles::Projectile;
use super::targeting::{TargetView, Targets, domain_allows, reaches, reaches_point};
use super::turret::{self, Source, TurretState};
use super::{AttackPhase, AttackState};
use crate::defs::{AttackAttrs, AttackType};
use crate::elevation::Elevation;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::math;
use crate::movement;
use crate::object::Object;
use crate::shape::Shape;
use crate::tuning::combat::FACING_TOLERANCE;
use crate::units::Tick;
use crate::vision::PlayerVision;

mod effects;

use effects::{fire, launch, strike};

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

/// The attacking unit, or structure, as its sequence needs it.
pub(super) struct Shooter<'a> {
    pub id: ObjectId,
    /// Its owner, whom a projectile it launches remembers (`super::projectiles`).
    owner: Option<PlayerId>,
    pub pos: Vec2,
    /// Its shape: Range is measured from its edge (`systems/combat.md > Range Measurement`), a structure's footprint.
    shape: Shape,
    elevation: Elevation,
    /// For the ground's elevation at a location it aims at.
    map: &'a Map,
    pub attrs: &'a AttackAttrs,
    /// (G) What is left of its TurnRate this tick (`systems/combat.md > LocomotionOrientationConstraints`), radians:
    /// one budget per tick, of which phase 5d may have spent some (`movement::Turns`). `f32::INFINITY` for the
    /// movement models whose turn rate depends on speed, and for a structure.
    turn_left: f32,
    /// Where its attack comes from: its base, its turret (`UnitDef::has_turret`, its TurretAttributes; design change
    /// 0154), or a structure's, which fires in any direction ([`Source`]).
    source: Source,
    /// Its owner's vision as of the end of the last tick; `None` for a unit without an owner, which sees nothing.
    sight: Option<&'a PlayerVision>,
}

impl<'a> Shooter<'a> {
    /// The object `o`, attacking with `attrs` (`super::weapon`), its owner's vision `sight`, after phase 5d turned it
    /// by `turned` this tick (plan decision T49: the attack and the object, a unit's or a structure's).
    pub(super) fn new(
        map: &'a Map,
        o: &Object,
        attrs: &'a AttackAttrs,
        sight: Option<&'a PlayerVision>,
        turned: f32,
    ) -> Self {
        let unit = o.def().unit.as_ref();
        Self {
            id: o.id,
            owner: o.owner,
            pos: o.pos,
            // A unit's silhouette where it stands, a structure's footprint (`Shape::of`).
            shape: Shape::of(o),
            elevation: Elevation::of_object(map, o),
            map,
            attrs,
            turn_left: unit.map_or(f32::INFINITY, |unit| movement::turn_rate(&unit.movement) - turned),
            source: Source::of(unit),
            sight,
        }
    }

    /// Its owner, if it has one.
    pub(super) fn owner(&self) -> Option<PlayerId> {
        self.owner
    }

    /// Whether `t` is within its Range and beyond its MinRange, and its owner sees it (`systems/combat.md >
    /// ValidTarget` 2): what a structure's own pick asks of a target (`super::structures`).
    pub(super) fn sees_in_range(&self, t: &TargetView) -> bool {
        self.can_target(t) && self.sees(t) && self.reaches(t)
    }

    /// The gap between its shape and `t`'s (`systems/combat.md > Range Measurement`).
    pub(super) fn gap(&self, t: &TargetView) -> f32 {
        self.shape.gap(&t.shape)
    }

    /// Whether its attack can target `t` at all: not itself, and in a domain its TargetDomain allows
    /// (`systems/combat.md > ValidTarget` 3).
    pub(super) fn can_target(&self, t: &TargetView) -> bool {
        t.id != self.id && domain_allows(self.attrs.target_domain, t.domain)
    }

    /// (M4-O, M5-O3) Whether its attack locks a location at the end of Aiming, where its effect lands, and may aim at
    /// a location: a DoublyDisjointed one, which launches a projectile there, and a TailDisjointed one, which strikes
    /// there at the end of Firing (module docs). They are the attacks that can target the ground
    /// (`super::can_target_ground`).
    fn locks(&self) -> bool {
        matches!(
            self.attrs.attack_type,
            AttackType::DoublyDisjointed | AttackType::TailDisjointed
        )
    }

    /// `systems/combat.md > ValidTarget` 2: whether its owner sees `t` (`PlayerVision::object_box_state`: `t` isn't
    /// hidden from the owner, plan decision T32, and a tile it covers is Visible, or it is revealed to the owner).
    /// Phase 5e reads the vision phase 5f left at the end of the last tick (`docs/movement.md > 4`).
    pub(super) fn sees(&self, t: &TargetView) -> bool {
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

    /// Whether a unit facing `facing` faces the point `at`, its target's centre or a location ([`FACING_TOLERANCE`]):
    /// (M5-U1) its turret in `turret` does, on that base (`super::turret::aim_facing`); a structure always does.
    fn faces(&self, facing: f32, turret: Option<TurretState>, at: Vec2) -> bool {
        let aim = match self.source {
            Source::Base => facing,
            Source::Turret(_) => turret::aim_facing(turret, facing),
            Source::Structure => return true,
        };
        self.bearing(at)
            .is_none_or(|wanted| math::wrap_angle(wanted - aim).abs() <= FACING_TOLERANCE)
    }

    /// (M5-U1) Whether the point `at` is within its turret's arc, on its base facing `facing` (`super::turret::in_arc`);
    /// always for an attack from its base or a structure's.
    fn in_arc(&self, facing: f32, at: Vec2) -> bool {
        match self.source {
            Source::Turret(attrs) => turret::in_arc(&attrs, facing, self.pos, at),
            Source::Base | Source::Structure => true,
        }
    }

    /// (M5-U1) The object its sequence aims at: its turret's LockedTarget (`super::turret`), else what its behaviour
    /// engages.
    fn aimed(&self, state: &AttackState) -> Option<ObjectId> {
        match self.source {
            Source::Turret(_) => state.locked(),
            Source::Base | Source::Structure => state.target,
        }
    }

    /// `systems/combat.md > Aiming`: the unit turns towards the point `at`, at most its TurnRate this tick
    /// (`> LocomotionOrientationConstraints[TurnRateMovement]`, `Stationary + Turning`), landing exactly on the
    /// direction once within it. (G) The TurnRate is one budget per tick: the aim has what phase 5d left of it
    /// ([`Shooter::turn_left`]), and it turns a unit at most once per tick.
    fn turn_toward(&self, facing: &mut f32, at: Vec2) {
        let Some(wanted) = self.bearing(at).filter(|_| self.source == Source::Base) else {
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
    // (M5-U1) A turret turns first, by its own TurnRate (`super::turret::step`); the base turned in phase 5d.
    if let Source::Turret(attrs) = &shooter.source {
        turret::step(state, attrs, *facing, shooter, targets);
    }
    // `systems/controls/command_behaviors.md > AttackingObject` 3: a sequence begins once the target is in range (and
    // within a turret's arc) and no sequence runs, and only at a target the unit's owner sees (`systems/combat.md >
    // ValidTarget` 2). `systems/controls/command_behaviors.md > AttackingLocation`: the same at a location in range,
    // which no one needs to see.
    if state.phase.is_none() {
        if let Some(target) = shooter.aimed(state)
            && targets
                .get(target)
                .is_some_and(|t| shooter.sees(t) && shooter.reaches(t) && shooter.in_arc(*facing, t.pos))
        {
            state.phase = Some(AttackPhase::Aiming { target });
            state.elapsed = 0;
        } else if let Some(at) = state.location
            && shooter.reaches_point(at)
            && shooter.in_arc(*facing, at)
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
                    shooter.aimed(state) == Some(target)
                        && shooter.can_target(t)
                        && shooter.sees(t)
                        && shooter.reaches(t)
                        && shooter.in_arc(*facing, t.pos)
                });
                let Some(t) = valid else {
                    state.cancel();
                    break;
                };
                Some(t.pos)
            }
            // (M4-O) The same at a location: still the one engaged, and in range
            // (`systems/controls/command_behaviors.md > AttackingLocation`: "range/arc check against location").
            AttackPhase::AimingAt { at } => {
                if state.location != Some(at) || !shooter.reaches_point(at) || !shooter.in_arc(*facing, at) {
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
        let over = state.elapsed >= duration(shooter.attrs, phase)
            && aimed_at.is_none_or(|at| shooter.faces(*facing, state.turret, at));
        if over {
            state.phase = end(phase, aimed_at, shooter, targets, volley, events);
            state.elapsed = 0;
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
            match shooter.attrs.attack_type {
                AttackType::TailDisjointed => strike(shooter, at, target, targets, &mut volley.hits, events),
                _ => launch(shooter, at, target, volley, events),
            }
            Some(AttackPhase::Cooldown)
        }
        AttackPhase::Cooldown => Some(AttackPhase::Reloading),
        AttackPhase::Reloading => None,
    }
}

#[cfg(test)]
mod tests;
