//! Structures that attack — plan decision T49, slice M4-S1: the Laser Turret
//! (`factions/syndicate/structures/laser_turret.md`), whose attack is its def's (`StructureDef::attack`). A structure
//! takes no orders, so it picks its own target, in phase 5e right before its attack sequence runs (`super::phases`,
//! which it shares with the units, from its footprint). It has no turn rate ("it fires in any direction"): its barrel
//! faces its target at once ([`face`]).
//!
//! It attacks once complete (`Object::is_operational`: a site "does nothing but Cancel", design change 0234), which
//! `super::step` sees to.

use glam::Vec2;

use super::phases::Shooter;
use super::targeting::{TargetView, Targets, domain_allows};
use super::{AttackPhase, AttackState};
use crate::defs::Domain;
use crate::ids::ObjectId;
use crate::math;

/// Whether `t` is an enemy of the structure's: another player's (`systems/combat.md > ValidTarget` 1, every player its
/// own side: `economy::defeat::allied`). A neutral object is never picked, as auto-targeting ignores them
/// (`systems/controls/commands.md > Attack`: "auto-targeting ignores them").
fn enemy(shooter: &Shooter, t: &TargetView) -> bool {
    t.owner.is_some() && shooter.owner().is_some() && t.owner != shooter.owner()
}

/// The target a structure engages this tick (`state.target`), as a unit holding position picks its own (design change
/// 0298 point 8, `factions/syndicate/structures/laser_turret.md > Targeting`: "the turret has no commands and targets
/// like a unit holding position: BaseAutoTargeting within Range (systems/controls/behaviors.md), then the closest, ties
/// broken deterministically. It **keeps its target** while the target stays valid and in range"):
/// - **It keeps** the one it engages while that is still an enemy's ValidTarget (`systems/combat.md > ValidTarget`:
///   destructible, in a domain its attack allows, seen by its owner) within Range and beyond MinRange.
/// - **Else it picks** the best such target (`systems/controls/behaviors.md > BaseAutoTargeting > Target Selection
///   Priority`): **threatening** targets first, those whose own attack can target it (a structure stands on the
///   ground: `super::object_domain`), then the **closest**, by the gap between the shapes, then the lowest id (the
///   deterministic tie-break). BaseAutoTargeting's second criterion, the least rotation, counts for nothing: a
///   structure fires in any direction (plan decision T49). None in range: it engages nothing.
///
/// Nothing while auto-targeting is off (`DebugCommand::SetAutoTargeting`), every target of a structure's being one it
/// picked itself. A target it leaves is dropped as a unit's is (`AttackState::engage`): an aim at it is cancelled, a
/// shot at it under way lands.
pub(super) fn pick(state: &mut AttackState, shooter: &Shooter, targets: &Targets, auto_targeting: bool) {
    if !auto_targeting {
        state.engage(None);
        return;
    }
    let valid = |t: &TargetView| enemy(shooter, t) && shooter.sees_in_range(t);
    if state.target.and_then(|id| targets.get(id)).is_some_and(valid) {
        return;
    }
    let mut best: Option<(bool, f32, ObjectId)> = None;
    for t in targets.all().iter().filter(|t| valid(t)) {
        let harmless = !t.threat.is_some_and(|threat| domain_allows(threat, Domain::Ground));
        let key = (harmless, shooter.gap(t), t.id);
        let better = best.is_none_or(|b| {
            (key.0.cmp(&b.0))
                .then(key.1.total_cmp(&b.1))
                .then(key.2.cmp(&b.2))
                .is_lt()
        });
        if better {
            best = Some(key);
        }
    }
    state.engage(best.map(|(.., id)| id));
}

/// A structure's barrel turns at once to face what it fires at (plan decision T49: no turn rate): the target of the
/// shot under way, else the one it engages. Its `facing`, which nothing of the game reads of a structure (its shape is
/// its footprint), is where the client draws its barrel pointing; it stays as it was with no target.
pub(super) fn face(state: &AttackState, facing: &mut f32, shooter: &Shooter, targets: &Targets) {
    let aimed = match state.phase {
        Some(AttackPhase::Firing { target: Some(target) }) => Some(target),
        _ => state.target,
    };
    if let Some(t) = aimed.and_then(|id| targets.get(id)) {
        let to = t.pos - shooter.pos;
        if to != Vec2::ZERO {
            *facing = math::wrap_angle(math::vector_angle(to));
        }
    }
}

#[cfg(test)]
mod tests;
