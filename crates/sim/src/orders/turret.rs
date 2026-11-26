//! A unit's turret's behaviour in phase 5a — `systems/controls/behaviors.md > TurretBehavior` (slice M5-U1, plan
//! decision T80): "When the turret has a locked target, it engages that target. When the turret has no locked target,
//! it autonomously scans for and engages the best available target within range and turn angle." It runs for every
//! unit with a turret (`combat::turret`), right after its base's behaviour ([`run`]), whatever that is: during a Move
//! too (`systems/controls/behaviors.md > BaseAutoTargeting > Not active during`: "Turret units'
//! TurretAutonomousScanning still operates independently via TurretBehavior"). It never moves the unit or stops it.

use super::world::{Scan, Unit, View, World};
use crate::combat::{domain_allows, turret};
use crate::defs::{AttackAttrs, TurretAttrs};

/// One tick of the turret of `unit`, after its base's behaviour has set or left its LockedTarget
/// (`systems/controls/commands.md > TurretCommandState`: "When the base behavior identifies a target, it updates the
/// TurretCommandState. When the base behavior does not specify a target, the turret retains its previous
/// TurretCommandState. If the turret has no assigned target, it falls back to autonomous target scanning").
/// - **When a lock ends** (`systems/controls/commands.md > TurretCommandState > When a lock ends`, design change 0376,
///   answering Q0137): "A turret keeps a lock **while its base is engaging that target**, or otherwise **while it can
///   fire at it**: in range, in its arc, and seen by its owner. When neither holds, it drops the lock and scans for a
///   new target in the same tick." Its base engages that very object (`AttackState::target`: its Attack's,
///   AttackMove's CurrentTarget, the enemy its idle auto-targeting acquired) until the engagement's own rules end it
///   (out of sight, beyond the leash, destroyed); any other LockedTarget, one its scan locked or one an order before
///   left it, it keeps while it can fire at it ([`fireable`]): "a Move keeps a lock set by an earlier Attack, friendly
///   or enemy, for as long as the turret can fire at it (firing on the move)". A Stop clears it, and the turret scans
///   in that tick (`orders::run::start`: "**Stop clears the lock**").
/// - **With nothing locked it scans** (`systems/controls/behaviors.md > TurretAutonomousScanning`, [`World::turret_pick`])
///   and locks what it finds (step 4: "Set TurretCommandState.LockedTarget to the selected target"); nothing found,
///   it stays inactive (step 5). Nothing while auto-targeting is off (`DebugCommand::SetAutoTargeting`).
/// - **While an Attack Ground engages a location**, nothing: `systems/controls/command_behaviors.md >
///   AttackingLocation`: "TurretCommandState.LockedTarget is not set (turret fires at location, not a unit)".
/// - **A unit that can't attack now** (its Riot Shield up, `Unit::attrs`) locks nothing.
pub(super) fn run(unit: &mut Unit, world: &World) {
    let Some(turret) = unit.turret() else {
        return;
    };
    let Some(attack) = unit.attack.as_deref() else {
        return;
    };
    if attack.location.is_some() {
        return;
    }
    let (locked, engaged) = (attack.locked(), attack.target);
    let Some(attrs) = unit.attrs() else {
        unit.lock(None);
        return;
    };
    if let Some(id) = locked {
        let keeps = Some(id) == engaged || world.get(id).is_some_and(|t| fireable(unit, attrs, &turret, t, world));
        if keeps {
            return;
        }
        unit.lock(None);
    }
    if let Some(target) = world.turret_pick(unit, &turret) {
        unit.lock(Some(target));
    }
}

/// Whether the turret `turret` of `unit`, attacking with `attrs`, can fire at `target` from where the unit stands
/// (`systems/controls/commands.md > TurretCommandState > When a lock ends`: "in range, in its arc, and seen by its
/// owner"): its owner sees the target (`systems/combat.md > ValidTarget` 2), which is another object, destructible, in
/// a domain the attack can target (`systems/combat.md > ValidTarget` 1 and 3, of any owner: an Attack's target needn't
/// be an enemy), within Range and beyond MinRange ([`Scan::Weapon`]), and within the turret's arc
/// (`combat::turret::in_arc`).
fn fireable(unit: &Unit, attrs: &AttackAttrs, turret: &TurretAttrs, target: &View, world: &World) -> bool {
    target.id != unit.id
        && target.destructible
        && domain_allows(attrs.target_domain, target.domain)
        && world.sees(unit.owner, target)
        && world.within(unit, attrs, Scan::Weapon, target)
        && turret::in_arc(turret, unit.facing, unit.pos, target.pos)
}

#[cfg(test)]
mod tests;
