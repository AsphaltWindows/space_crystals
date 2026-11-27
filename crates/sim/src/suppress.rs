//! (M5-N11) Suppress, the Prayer-mobile's screech — `factions/cults/units/prayer_mobile.md > Abilities`: "**Suppress**:
//! a screech in an **arc in front of it**. Affected units **temporarily can't attack**";
//! `factions/cults/units/prayer_mobile.md > Placeholder stats`: "an arc of **90° total** in front of the vehicle's
//! current facing, **5 grid units** long; units whose centre is inside can't attack for **4 s**. The driver aims it by
//! pointing the vehicle". Its numbers are its def's (`defs::cults::units::prayer_mobile::SUPPRESS`).
//!
//! - **The status**: "suppressed until tick T", a buff (`Object::buffs`, `crate::buffs`: at most one, until its tick,
//!   hashed and saved only while a unit has one), which the cast (`Command::Cast`, in the commands phase) gives every
//!   unit whose centre is in its arc ([`in_arc`]): "**Affects every unit in the arc, enemies and own alike**", "**Air
//!   units in the arc are affected** too", "**Not buildings**". It lasts 64 ticks, the cast's own included, and the
//!   Buffs step ends it after its last (`ARCHITECTURE.md > Tick pipeline`). A unit already suppressed has it restarted
//!   by a recast: **Placeholder (not in the design)**, as Inspire's "Recasting on units that are already inspired
//!   restarts the duration". The vehicle isn't in its own arc, and has no weapon anyway.
//! - **The arc** ([`in_arc`]): from the vehicle's centre along its facing, as the last tick left them, 45° to each side,
//!   bounds included, out to 5 grid units from its centre, the edge included: **Placeholder (not in the design: the
//!   bounds)**, as a turret's arc (`combat::turret`) and the Shrine's circle (`crate::buffs`). A unit is in it by its
//!   centre, air units by where they count as being (`Object::pos`).
//! - **What it stops** ("**weapon attacks** and **abilities that use a weapon**"): a unit that holds fire
//!   ([`holds_fire`]) starts no attack sequence (`combat::phases::run`), on the map or firing from inside a building
//!   (`combat::occupants`: one suppressed outside that goes in stays so there), so it fires nothing, shot, projectile
//!   or strike. "A Rocketman's **Target Lock can be maintained**, but no missile can be fired while suppressed": its
//!   lock goes on counting, and holds or breaks as it would (`combat::target_lock`); locked, it engages its target, and
//!   its first missile's sequence starts once the status is over. It keeps the target its behaviour engages, its orders
//!   and its auto-targeting: it stands in range, or goes on chasing, and attacks once the status is over (the behaviours
//!   don't read the status: the attack waits, as a raised pop-up's would).
//! - **An attack under way** as it is suppressed: **Placeholder (not in the design: an Aiming cancelled or paused, and
//!   a Firing that has begun)**: an aim is cancelled, as a new command cancels one (`systems/combat.md > Aiming`), and a
//!   shot being fired, a Cooldown and a reload under way run on to their ends ("Firing" and "Cooldown" are not
//!   interruptible, `systems/combat.md > AttackPhase`; a reload keeps counting): the hit of a Firing under way lands. A
//!   pop-up turret that retracts does the same (`crate::popup`). The sequence's next aim waits for the status's end.
//! - **What it doesn't stop** ("Other abilities (e.g. a Disciple's Shroud) are unaffected"): the unit moves, takes and
//!   carries out its orders, channels Shroud, raises or lowers a Riot Shield, converts, casts, and builds, gathers and
//!   repairs. Its weapon's numbers are as they were.

use glam::Vec2;

use crate::defs::BuffEffect;
use crate::math;
use crate::object::Object;

/// Whether a unit at `pos` is in an arc of `arc` radians in all, from `at` along `facing`, `radius` space units long
/// (module docs): its centre within half the arc of the facing, bounds included, and at most `radius` from `at`. A
/// centre right at `at` is in it.
pub fn in_arc(at: Vec2, facing: f32, arc: f32, radius: f32, pos: Vec2) -> bool {
    let to = pos - at;
    if to.length_squared() > radius * radius {
        return false;
    }
    to == Vec2::ZERO || math::wrap_angle(math::vector_angle(to) - facing).abs() <= arc / 2.0
}

/// Whether `o` holds its fire (module docs): it is suppressed, a buff of Suppress's (`BuffEffect::HoldFire`) among its
/// buffs. What phase 5e asks of every attack before it starts one.
#[inline]
pub fn holds_fire(o: &Object) -> bool {
    !o.buffs.is_empty() && (o.buffs.iter()).any(|buff| buff.ability.effect() == Some(&BuffEffect::HoldFire))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::PI;

    /// Suppress's arc, 90° in all, 5 grid units (320 su) long, from a vehicle at `AT` facing east (0) or south (PI/2):
    /// a centre 44° off the facing is in it, one at 46° isn't; the edges at 45° and 320 su are in, 321 su isn't; a
    /// centre behind isn't; the vehicle's own centre is.
    #[test]
    fn a_unit_is_in_the_arc_by_its_centre() {
        const AT: Vec2 = Vec2::new(1000.0, 1000.0);
        let (arc, radius) = (PI / 2.0, 320.0);
        let off = |facing: f32, degrees: f32, distance: f32| {
            AT + math::facing_vector(facing + math::degrees_to_radians(degrees)) * distance
        };
        for facing in [0.0, PI / 2.0, -PI / 2.0, PI] {
            let inside = |degrees: f32, distance: f32| in_arc(AT, facing, arc, radius, off(facing, degrees, distance));
            assert!(inside(0.0, 100.0), "{facing}: straight ahead");
            assert!(
                inside(44.0, 300.0) && inside(-44.0, 300.0),
                "{facing}: just inside the 90°"
            );
            assert!(
                !inside(46.0, 100.0) && !inside(-46.0, 100.0),
                "{facing}: just outside the 90°"
            );
            assert!(!inside(180.0, 50.0) && !inside(90.0, 50.0), "{facing}: behind, beside");
            assert!(inside(0.0, 319.9), "{facing}: inside its length");
            assert!(!inside(0.0, 321.0), "{facing}: past its length");
        }
        assert!(
            in_arc(AT, 0.0, arc, radius, AT + Vec2::new(200.0, 200.0)),
            "on the 45° bound"
        );
        assert!(
            in_arc(AT, 0.0, arc, radius, AT + Vec2::new(320.0, 0.0)),
            "on its far edge"
        );
        assert!(in_arc(AT, 1.0, arc, radius, AT), "the centre");
    }
}
