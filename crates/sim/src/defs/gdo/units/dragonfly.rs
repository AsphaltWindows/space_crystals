//! The Dragonfly — `factions/gdo/units/dragonfly.md` (design change 0366, slice M6-GK): "GDO air-to-air fighter
//! produced at the Airfield once the player owns Propulsion Labs. Fires **air-to-air missiles**." Its numbers are the
//! design's "Placeholder stats" (plan decision T84), its GliderMovement and its homing missile among them; its idle
//! circle's radius, "1 grid unit", is every Glider's (`crate::tuning::air`). Its ammo (`> AttackAttributes`: "Ammo:
//! **1 missile**, then it must reload at its assigned Airfield"; "**RearmTimePerAmmo 64 frames**") and its Airfield
//! entry ([`RECIPE`], held) are slice M6-GL's; no player produces one before.

use crate::defs::structures::Recipe;
use crate::defs::units::{AttackAttrs, AttackType, MovementAttrs, TargetDomain, TargetType, UnitBase, UnitDef};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/units/dragonfly.md`: its placeholder stats (design change 0366).
pub(crate) const DRAGONFLY: ObjectDef = ObjectDef {
    kind: ObjectKind::Dragonfly,
    name: "Dragonfly",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Unit,    // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 40.0, h: 40.0 }, // Silhouette 40x40 space units
    selectable: true,            // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 8.0,    // SightRange 8 grid units
    max_hp: Some(100.0), // MaxHP 100
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 1.0,  // FullArmor 1
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        // UnitBase - Glider (must keep moving; circles when idle; wider turns at higher speed; with **vertical landing
        // and take-off at the Airfield**, Q0162)
        base: UnitBase::Glider,
        // "GliderMovement: IdleSpeed 6, MaxSpeed **14 su/frame**, Acceleration 0.6, Deceleration 0.6,
        // MaxCentripetalAcceleration 0.6 (about 330 su turn radius at full speed)"
        movement: MovementAttrs::Glider {
            idle_speed: 6.0,
            max_speed: 14.0,
            acceleration: Some(0.6),
            deceleration: Some(0.6),
            max_centripetal_acceleration: 0.6,
        },
        turret: None, // "No turret (fires forward)."
        attack: Some(AttackAttrs {
            // "Attack: **HeadDisjointed** (homing air-to-air missile; user, 2026-11-27)"
            attack_type: AttackType::HeadDisjointed,
            target_domain: TargetDomain::Air, // "TargetDomain: Air"
            // **Placeholder (not in the design: the Dragonfly's TargetType)**: SingleTarget, its Damage a missile's.
            target_type: TargetType::SingleTarget,
            // "Damage **90**; Range 6 grid units; ProjectileSpeed 20 su/frame; Aim 8, Firing 2, Cooldown 4, Reload 0 (1
            // missile)"; no MinRange given.
            damage: 90.0,
            range: 6.0,
            min_range: 0.0,
            projectile_speed: Some(20.0),
            aim_ticks: 8,
            firing_ticks: 2,
            cooldown_ticks: 4,
            reload_ticks: 0,
        }),
        unit_control_cost: Some(2), // UnitControlCost 2
        // "SeparationRadius **48** su (raised from 32, Q0148; must exceed the 40 su Silhouette)"
        separation_radius: Some(48.0),
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/airfield.md > Produces (so far)`: "Dragonfly: 175 Space Crystals, 320 frames (20 seconds)
/// (placeholder). Requires: player owns at least one Propulsion Labs", its Unit Prerequisite, and `> Placeholder
/// stats` of `factions/gdo/units/dragonfly.md`: "Cost: Airfield, 175 Space Crystals **+ 4 Supplies** (user,
/// 2026-11-26; Q0169), 320 frames" (design change 0435). Held until slice M6-GL (`crate::defs::PLANNED_RECIPES`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::Dragonfly,
    cost: 175,
    frames: 320,
    requires: &[ObjectKind::PropulsionLabs],
    supplies: 4,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::Domain;

    /// The placeholder stats: a Glider, 40×40, MaxHP 100, armour 1/1, SightRange 8, Unit Control 2; GliderMovement from
    /// an idle 6 su a frame to 14; no turret; a homing missile at air only, 90 damage up to 6 grid units at 20 su a
    /// frame, no reload (its one missile); a SeparationRadius of 48 su, more than its Silhouette. Its Airfield entry,
    /// 175 Space Crystals and 4 Supplies in 320 frames behind Propulsion Labs, is held.
    #[test]
    fn the_dragonfly_follows_the_design() {
        let unit = super::super::assert_stat_block(
            ObjectKind::Dragonfly,
            "Dragonfly",
            (40.0, 40.0),
            UnitBase::Glider,
            100.0,
            (1.0, 1.0),
            8.0,
            2,
        );
        assert_eq!(unit.base_def().domain, Domain::Air);
        let glider = MovementAttrs::Glider {
            idle_speed: 6.0,
            max_speed: 14.0,
            acceleration: Some(0.6),
            deceleration: Some(0.6),
            max_centripetal_acceleration: 0.6,
        };
        assert_eq!((unit.movement, unit.turret), (glider, None));
        let attack = unit.attack.expect("its missile");
        assert_eq!(
            (attack.attack_type, attack.target_domain, attack.target_type),
            (AttackType::HeadDisjointed, TargetDomain::Air, TargetType::SingleTarget)
        );
        assert_eq!(
            (attack.damage, attack.range, attack.projectile_speed),
            (90.0, 6.0, Some(20.0))
        );
        assert_eq!(super::super::cycle(&attack), ((8, 2, 4, 0), 14));
        assert_eq!(unit.separation_radius, Some(48.0));
        let recipe = Recipe {
            kind: ObjectKind::Dragonfly,
            cost: 175,
            frames: 320,
            requires: &[ObjectKind::PropulsionLabs],
            supplies: 4,
        };
        assert_eq!(RECIPE, recipe);
    }
}
