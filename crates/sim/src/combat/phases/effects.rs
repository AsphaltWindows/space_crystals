//! What an attack sequence does as its Firing ends (`super::end`; `systems/combat.md > Firing`): a single-target hit
//! ([`fire`]), a TailDisjointed strike at the spot locked ([`strike`]) or a DoublyDisjointed projectile's launch
//! ([`launch`]).

use glam::Vec2;

use super::super::area::{self, Blast};
use super::super::damage::{Hit, directional_multiplier};
use super::super::projectiles::Projectile;
use super::super::targeting::{TargetView, Targets};
use super::{Shooter, Volley};
use crate::defs::TargetType;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::units::gu;

/// `systems/combat.md > Firing`: the attack's effect takes place, at the end of the phase (a placeholder reading:
/// the design says only "during" it; TailDisjointed attacks apply theirs at its end). A FullyConnected attack can't
/// miss (`> FullyConnected`): it lands on the target wherever the target is now.
///
/// TODO(attack types): a HeadDisjointed attack launches a projectile that tracks its target instead (`>
/// HeadDisjointed`); no unit has one, and it lands like this, as one single-target hit.
pub(super) fn fire(shooter: &Shooter, t: &TargetView, hits: &mut Vec<Hit>, events: &mut Vec<SimEvent>) {
    hit(shooter, t, hits);
    events.push(SimEvent::Attacked {
        attacker: shooter.id,
        target: t.id,
        from: shooter.muzzle(t.pos),
        to: t.pos,
    });
}

/// The single-target hit of `shooter`'s attack on `t` (`systems/combat.md > SingleTargetDamage`): its Damage against
/// `t`'s PointArmor, from the side the shooter stands on (`systems/units.md > UnitBase > DirectionalArmor`), where its
/// shot leaves from (`Shooter::muzzle`).
fn hit(shooter: &Shooter, t: &TargetView, hits: &mut Vec<Hit>) {
    let directional = if t.directional_armor {
        directional_multiplier(t.facing, shooter.muzzle(t.pos) - t.pos)
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
pub(super) fn strike(
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
    let (from, to) = (shooter.muzzle(at), at);
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
pub(super) fn at_the_spot<'t>(
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
/// the location the target was at at the end of the Aiming phase": at the end of Firing, from the shooter's centre
/// (`Shooter::muzzle`) to `at`, the location locked (plan decision T48). `target`: the object it was aimed at, if
/// any.
pub(super) fn launch(
    shooter: &Shooter,
    at: Vec2,
    target: Option<ObjectId>,
    volley: &mut Volley,
    events: &mut Vec<SimEvent>,
) {
    let projectile = Projectile::launch(
        shooter.id,
        shooter.owner,
        shooter.attrs,
        shooter.muzzle(at),
        at,
        target,
        volley.tick,
    );
    events.push(projectile.launched());
    volley.launched.push(projectile);
}
