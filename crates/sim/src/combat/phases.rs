//! A unit's attack sequence, one tick at a time — `systems/combat.md > AttackPhase`.
//!
//! The tick's time goes to the first phase that hasn't run its full duration yet. A phase that has run it ends at
//! once, with its effect, and the next one begins, so each phase lasts exactly its duration in ticks, zero included.
//! A sequence that starts in tick `t` aims in ticks `t .. t + Aim`, and its hit lands at the end of tick
//! `t + Aim + Firing − 1`, the last tick of Firing (tick `t` if both are 0). A new sequence starts at most once per
//! tick, in the tick after the last one's Reloading ends, so a cycle takes Aim + Firing + Cooldown + Reload ticks.

use glam::Vec2;

use super::damage::{Hit, directional_multiplier, hit_damage};
use super::targeting::{TargetView, Targets, domain_allows, range_distance, within_range};
use super::{AttackPhase, AttackState};
use crate::defs::{AttackAttrs, ObjectKind, UnitDef};
use crate::elevation::Elevation;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::math::{self, PI};

/// `systems/controls/behaviors.md > AttackingObject` 2: a unit attacking from its base must face its target ("unit
/// facing") before it fires. The largest angle, radians, between its facing and the direction to the target's centre
/// that counts as facing it. Placeholder, unspecified: 1°. Aiming turns the unit onto the exact direction once it is
/// within the TurnRate, so this only absorbs float rounding.
pub const FACING_TOLERANCE: f32 = PI / 180.0;

/// The attacking unit, as its sequence needs it.
pub(super) struct Shooter<'a> {
    pub id: ObjectId,
    pub pos: Vec2,
    /// Half its silhouette (`RangeMeasure::EdgeToEdge` measures from it).
    half: Vec2,
    elevation: Elevation,
    pub attrs: &'a AttackAttrs,
    /// Radians per tick (`systems/combat.md > LocomotionOrientationConstraints`); `None` for the movement models
    /// whose turn rate depends on speed.
    turn_rate: Option<f32>,
    /// `systems/units.md > UnitBase > HasTurret`: the turret aims, not the base.
    turret: bool,
}

impl<'a> Shooter<'a> {
    pub(super) fn new(
        map: &Map,
        id: ObjectId,
        kind: ObjectKind,
        pos: Vec2,
        unit: &UnitDef,
        attrs: &'a AttackAttrs,
    ) -> Self {
        Self {
            id,
            pos,
            half: kind.def().half_extents().unwrap_or(Vec2::ZERO),
            elevation: Elevation::of(map, kind, pos),
            attrs,
            turn_rate: unit.movement.turn_rate(),
            turret: unit.base_def().has_turret,
        }
    }

    /// Whether its attack can target `t` at all: not itself, and in a domain its TargetDomain allows
    /// (`systems/combat.md > ValidTarget` 3).
    fn can_target(&self, t: &TargetView) -> bool {
        t.id != self.id && domain_allows(self.attrs.target_domain, t.domain)
    }

    /// Whether `t` is within its Range and beyond its MinRange from where it stands.
    fn reaches(&self, t: &TargetView) -> bool {
        let distance = range_distance(self.pos, self.half, t.lo, t.hi);
        within_range(self.attrs, distance, self.elevation, t.elevation)
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
    /// direction once within it.
    fn turn_toward(&self, facing: &mut f32, t: &TargetView) {
        let Some(wanted) = self.bearing(t).filter(|_| !self.turret) else {
            return;
        };
        // TODO(movement models): only TurnRateMovement (every M1 unit) has a fixed rate; the others turn at once.
        let rate = self.turn_rate.unwrap_or(PI);
        let turn = math::wrap_angle(wanted - *facing);
        *facing = if turn.abs() <= rate {
            wanted
        } else {
            math::wrap_angle(*facing + rate.copysign(turn))
        };
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
    // the order that engaged it completes (`orders::update`).
    if let Some(target) = state.target
        && !targets.get(target).is_some_and(|t| shooter.can_target(t))
    {
        state.engage(None);
    }
    // `systems/controls/behaviors.md > AttackingObject` 3: a sequence begins once the target is in range and no
    // sequence runs. TODO(slice V): and visible to the unit's owner (`systems/combat.md > ValidTarget` 2).
    if state.phase.is_none()
        && let Some(target) = state.target
        && targets.get(target).is_some_and(|t| shooter.reaches(t))
    {
        state.phase = Some(AttackPhase::Aiming { target });
        state.elapsed = 0;
    }
    let mut tick_left = true;
    while let Some(phase) = state.phase {
        let aimed_at = match phase {
            // `systems/combat.md > Aiming`: the target must stay valid throughout, or the sequence is cancelled; the
            // unit keeps turning to face it. A different command already cancelled it (`AttackState::engage`).
            AttackPhase::Aiming { target } => {
                let valid = targets
                    .get(target)
                    .filter(|t| state.target == Some(target) && shooter.can_target(t) && shooter.reaches(t));
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
        damage: hit_damage(shooter.attrs.damage, t.pre_armor, t.point_armor, directional),
    });
    events.push(SimEvent::Attacked {
        attacker: shooter.id,
        target: t.id,
        from: shooter.pos,
        to: t.pos,
    });
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::defs::{AttackType, FullyConnectedSubtype, TargetDomain, TargetType};
    use crate::object::Object;

    const ME: ObjectId = ObjectId(1);
    const FOE: ObjectId = ObjectId(2);

    fn attrs(aim: u32, firing: u32, cooldown: u32, reload: u32) -> AttackAttrs {
        AttackAttrs {
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
            target_domain: TargetDomain::Ground,
            target_type: TargetType::SingleTarget,
            damage: 10.0,
            range: 4.0,
            min_range: 0.0,
            projectile_speed: None,
            aim_ticks: aim,
            firing_ticks: firing,
            cooldown_ticks: cooldown,
            reload_ticks: reload,
        }
    }

    /// A Peacekeeper (`ME`) at the origin facing east, and a Guard (`FOE`) 100 su away in direction `bearing`.
    fn world(bearing: f32) -> (Map, Targets) {
        let map = Map::new(4, 4);
        let mut objects = BTreeMap::new();
        for (id, kind, pos) in [
            (ME, ObjectKind::Peacekeeper, Vec2::ZERO),
            (FOE, ObjectKind::Guard, math::facing_vector(bearing) * 100.0),
        ] {
            objects.insert(
                id,
                Object {
                    id,
                    kind,
                    owner: None,
                    pos,
                    prev_pos: pos,
                    facing: 0.0,
                    prev_facing: 0.0,
                    hp: kind.def().max_hp,
                    placement: None,
                    resource: None,
                    orders: None,
                    mover: None,
                    attack: None,
                },
            );
        }
        let targets = Targets::snapshot(&map, &objects);
        (map, targets)
    }

    /// Runs `ticks` ticks of a sequence engaging `FOE`; the ticks (from 1) whose hits landed, and the final facing.
    fn hit_ticks(attrs: &AttackAttrs, bearing: f32, turn_rate: f32, ticks: u32) -> (Vec<u32>, f32) {
        let (map, targets) = world(bearing);
        let unit = ObjectKind::Peacekeeper.def().unit.clone().unwrap();
        let mut shooter = Shooter::new(&map, ME, ObjectKind::Peacekeeper, Vec2::ZERO, &unit, attrs);
        shooter.turn_rate = Some(turn_rate);
        let mut state = AttackState::default();
        state.engage(Some(FOE));
        let mut facing = 0.0;
        let mut landed = Vec::new();
        for tick in 1..=ticks {
            let mut hits = Vec::new();
            run(&mut state, &mut facing, &shooter, &targets, &mut hits, &mut Vec::new());
            if !hits.is_empty() {
                landed.push(tick);
            }
        }
        (landed, facing)
    }

    #[test]
    fn each_phase_lasts_exactly_its_duration() {
        // Aim 2, Firing 1, Cooldown 2, Reload 12: hits in ticks 3, 20, 37 (a period of 17).
        assert_eq!(hit_ticks(&attrs(2, 1, 2, 12), 0.0, PI, 40).0, [3, 20, 37]);
        // Aim 2, Firing 1, Cooldown 1, Reload 4: a period of 8.
        assert_eq!(hit_ticks(&attrs(2, 1, 1, 4), 0.0, PI, 20).0, [3, 11, 19]);
        // Longer Firing: the hit lands at its end.
        assert_eq!(hit_ticks(&attrs(1, 3, 0, 0), 0.0, PI, 10).0, [4, 8]);
    }

    #[test]
    fn phases_may_last_no_time_at_all() {
        // No aim and no Firing: the hit lands in the tick the sequence starts, whose time goes to Cooldown. The
        // period is still the sum of the durations, 3 ticks.
        assert_eq!(hit_ticks(&attrs(0, 0, 1, 2), 0.0, PI, 9).0, [1, 4, 7]);
        // Every phase empty: still one attack per tick at most.
        assert_eq!(hit_ticks(&attrs(0, 0, 0, 0), 0.0, PI, 3).0, [1, 2, 3]);
    }

    #[test]
    fn aiming_lasts_until_the_unit_faces_its_target() {
        // The target is due south (a quarter turn); at 0.5 rad per tick the turn takes 4 ticks, longer than the aim.
        let (landed, facing) = hit_ticks(&attrs(2, 1, 0, 30), PI / 2.0, 0.5, 10);
        assert_eq!(landed, [5], "aims 4 ticks, fires in the 5th");
        assert_eq!(facing, PI / 2.0);
        // Fast enough: the aim's duration decides.
        let (landed, facing) = hit_ticks(&attrs(2, 1, 0, 30), PI / 2.0, PI, 10);
        assert_eq!(landed, [3]);
        assert_eq!(facing, PI / 2.0);
    }
}
