//! A unit's attack sequence, one tick at a time — `systems/combat.md > AttackPhase`. A structure that attacks runs the
//! same sequence (plan decision T49, slice M4-S1: the Laser Turret), as a turret does: it fires in any direction.
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

use super::area::{self, Blast};
use super::damage::{Hit, directional_multiplier};
use super::projectiles::Projectile;
use super::targeting::{TargetView, Targets, domain_allows, reaches, reaches_point};
use super::{AttackPhase, AttackState};
use crate::defs::{AttackAttrs, AttackType, TargetType};
use crate::elevation::Elevation;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::math;
use crate::movement;
use crate::object::Object;
use crate::shape::Shape;
use crate::tuning::combat::FACING_TOLERANCE;
use crate::units::{Tick, gu};
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
    /// The unit has a turret (`UnitDef::has_turret`, its TurretAttributes; design change 0154): the turret aims, not
    /// the base. So does a structure (plan decision T49: "A structure has no turn rate (it fires in any direction)").
    turret: bool,
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
            turret: unit.is_none_or(|unit| unit.has_turret()),
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
    fn can_target(&self, t: &TargetView) -> bool {
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
    // `systems/controls/command_behaviors.md > AttackingObject` 3: a sequence begins once the target is in range and no
    // sequence runs, and only at a target the unit's owner sees (`systems/combat.md > ValidTarget` 2).
    // `systems/controls/command_behaviors.md > AttackingLocation`: the same at a location in range, which no one needs
    // to see.
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
            // (M4-O) The same at a location: still the one engaged, and in range
            // (`systems/controls/command_behaviors.md > AttackingLocation`: "range/arc check against location").
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

/// `systems/combat.md > Firing`: the attack's effect takes place, at the end of the phase (a placeholder reading:
/// the design says only "during" it; TailDisjointed attacks apply theirs at its end). A FullyConnected attack can't
/// miss (`> FullyConnected`): it lands on the target wherever the target is now.
///
/// TODO(attack types): a HeadDisjointed attack launches a projectile that tracks its target instead (`>
/// HeadDisjointed`); no unit has one, and it lands like this, as one single-target hit.
fn fire(shooter: &Shooter, t: &TargetView, hits: &mut Vec<Hit>, events: &mut Vec<SimEvent>) {
    hit(shooter, t, hits);
    events.push(SimEvent::Attacked {
        attacker: shooter.id,
        target: t.id,
        from: shooter.pos,
        to: t.pos,
    });
}

/// The single-target hit of `shooter`'s attack on `t` (`systems/combat.md > SingleTargetDamage`): its Damage against
/// `t`'s PointArmor, from the side the shooter stands on (`systems/units.md > UnitBase > DirectionalArmor`).
fn hit(shooter: &Shooter, t: &TargetView, hits: &mut Vec<Hit>) {
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
}

/// (M5-O3) `systems/combat.md > TailDisjointed`: "The animation and effect are applied to that location ... The effect
/// applies to whatever units are at the target location at the end of the Firing phase": at the end of Firing, at
/// `at`, the location locked at the end of Aiming, where the object aimed at (`target`, while it is in the game) stood
/// then, or the location aimed at. Nothing asks whether the shooter's owner sees what is there.
/// - **A single-target attack** (the Gunner's) hits the object at the spot ([`at_the_spot`]) with its single-target
///   hit: an `Attacked` on it, its path drawn to the spot. With nothing there it misses (`systems/combat.md >
///   CanMiss`), its damage lost: a `Missed`, drawn to the spot as well (design change 0304: "Visually still an
///   instant gunshot").
/// - **An area attack** (no kind has one yet: the GDO's Artillery Rig, `factions/gdo/units/artillery_rig.md >
///   AttackAttributes`, "the blast lands there at the end of Firing (no tracking projectile)") blasts the spot, as a
///   projectile that lands does (`super::area`: everything in the circle, whoever owns it).
fn strike(
    shooter: &Shooter,
    at: Vec2,
    target: Option<ObjectId>,
    targets: &Targets,
    hits: &mut Vec<Hit>,
    events: &mut Vec<SimEvent>,
) {
    if let TargetType::AoE { radius } = shooter.attrs.target_type {
        let blast = Blast {
            attacker: shooter.id,
            at,
            radius: gu(radius),
            damage: shooter.attrs.damage,
            domain: shooter.attrs.target_domain,
        };
        events.push(SimEvent::Blast {
            at,
            radius: blast.radius,
        });
        area::strike(&blast, targets, hits);
        return;
    }
    let (from, to) = (shooter.pos, at);
    match at_the_spot(shooter, at, target, targets) {
        Some(t) => {
            hit(shooter, t, hits);
            events.push(SimEvent::Attacked {
                attacker: shooter.id,
                target: t.id,
                from,
                to,
            });
        }
        None => events.push(SimEvent::Missed {
            attacker: shooter.id,
            target,
            from,
            to,
        }),
    }
}

/// (M5-O3) What a single-target TailDisjointed shot at the spot `at` strikes ([`strike`]): an object whose shape covers
/// the point (`Shape::distance_to` is 0 inside it and on its edge), a unit's Silhouette or a structure's footprint (a
/// Gunner may be ordered to attack a structure), that the attack can target at all (`systems/combat.md >
/// ValidTarget` 3: its TargetDomain allows it; not the shooter itself), whoever owns it (design change 0304: "whatever
/// unit stands at the spot then is hit instead (any owner)"), the shooter's owner's own included. None: it misses.
///
/// Placeholder (not in the design: it doesn't say which of two objects at the spot a single-target shot hits, and two
/// can be there, their edges touching across the point): the one it was aimed at, `target`, while it covers the spot,
/// so that a target that stands still is always hit; else the first in id order.
fn at_the_spot<'t>(
    shooter: &Shooter,
    at: Vec2,
    target: Option<ObjectId>,
    targets: &'t Targets,
) -> Option<&'t TargetView> {
    let covers = |t: &&TargetView| shooter.can_target(t) && t.shape.distance_to(at) == 0.0;
    let aimed = target.and_then(|id| targets.get(id)).filter(covers);
    aimed.or_else(|| targets.all().iter().find(covers))
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
