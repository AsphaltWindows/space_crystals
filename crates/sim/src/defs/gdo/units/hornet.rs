//! The Hornet — `factions/gdo/units/hornet.md` (design change 0366, slice M6-GK): "GDO air-to-ground attack aircraft
//! produced at the Airfield. A HoverCraft (a chopper) that fires **air-to-ground missiles**." Its numbers are the
//! design's "Placeholder stats" (plan decision T84), its DragMovement (design change 0420) and its missile among them.
//! Its ammo (`> AttackAttributes`: "Ammo: **5 missiles**, then it must reload at its assigned Airfield";
//! "**RearmTimePerAmmo 32 frames**") and its Airfield entry ([`RECIPE`], held) are slice M6-GH1's; no player produces
//! one before.

use crate::defs::structures::Recipe;
use crate::defs::units::{AttackAttrs, AttackType, MovementAttrs, TargetDomain, TargetType, UnitBase, UnitDef};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/gdo/units/hornet.md`: its placeholder stats (design change 0366).
pub(crate) const HORNET: ObjectDef = ObjectDef {
    kind: ObjectKind::Hornet,
    name: "Hornet",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Unit,    // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 48.0, h: 48.0 }, // Silhouette 48x48 space units
    selectable: true,            // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 7.0,    // SightRange 7 grid units
    max_hp: Some(120.0), // MaxHP 120
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 1.0,  // FullArmor 1
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::HoverCraft, // UnitBase - HoverCraft (can hover in place, turns in place)
        // "HoverCraft movement (DragMovement, as the HoverCraft base; designer placeholder 2026-11-26, Q0154: high drag
        // and high thrust, like the Supply Chopper): ForwardAcceleration 0.8; OmniDirectionalAcceleration 0.1;
        // DragRatio 0.1 (top speed 9 su/frame); TurnRate 8°/frame."
        movement: MovementAttrs::Drag {
            forward_acceleration: 0.8,
            omni_directional_acceleration: 0.1,
            drag_ratio: 0.1,
            turn_rate: 8.0 * PI / 180.0,
        },
        turret: None, // "No turret (fires forward; it turns in place to aim)."
        attack: Some(AttackAttrs {
            // "Attack: **DoublyDisjointed** (not homing: the missile flies to where the target was, so it can be
            // dodged; it can Attack Ground. user, 2026-11-27)"
            attack_type: AttackType::DoublyDisjointed,
            target_domain: TargetDomain::Ground, // "TargetDomain: Ground"
            // **Placeholder (not in the design: the Hornet's TargetType)**: SingleTarget, its Damage a missile's.
            target_type: TargetType::SingleTarget,
            // "Damage 30; Range 5 grid units; ProjectileSpeed 14 su/frame; Aim 4, Firing 2, Cooldown 4, Reload 22
            // frames"; no MinRange given.
            damage: 30.0,
            range: 5.0,
            min_range: 0.0,
            projectile_speed: Some(14.0),
            aim_ticks: 4,
            firing_ticks: 2,
            cooldown_ticks: 4,
            reload_ticks: 22,
        }),
        unit_control_cost: Some(2), // UnitControlCost 2
        // **Placeholder (Q0181)**: SeparationRadius 56 su, its Silhouette's side and 8 su, as the Sparrow's and the
        // Dragonfly's 48 on their 40 (`systems/units.md > Air Collision`: "Must be larger than the unit's Silhouette").
        separation_radius: Some(56.0),
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/airfield.md > Produces (so far)`: "Hornet: 150 Space Crystals, 320 frames (20 seconds)
/// (placeholder)", and `> Placeholder stats` of `factions/gdo/units/hornet.md`: "Cost: Airfield, 150 Space Crystals
/// **+ 2 Supplies** (user, 2026-11-26; Q0169), 320 frames" (design change 0435). Held until slice M6-GH1
/// (`crate::defs::PLANNED_RECIPES`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::Hornet,
    cost: 150,
    frames: 320,
    requires: &[],
    supplies: 2,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::Domain;

    /// The placeholder stats: a HoverCraft in the air, 48×48, MaxHP 120, armour 1/1, SightRange 7, Unit Control 2;
    /// DragMovement to a top speed of 9 su a frame, turning 8° a frame; no turret; a dodgeable missile at ground only,
    /// 30 damage up to 5 grid units at 14 su a frame, one every 32 frames; a SeparationRadius of 56 su, more than its
    /// Silhouette. Its Airfield entry, 150 Space Crystals and 2 Supplies in 320 frames, is held.
    #[test]
    fn the_hornet_follows_the_design() {
        let unit = super::super::assert_stat_block(
            ObjectKind::Hornet,
            "Hornet",
            (48.0, 48.0),
            UnitBase::HoverCraft,
            120.0,
            (1.0, 1.0),
            7.0,
            2,
        );
        assert_eq!(unit.base_def().domain, Domain::Air);
        assert!((unit.movement.max_speed() - 9.0).abs() < 1e-4, "top speed 9 su/frame");
        assert!((unit.movement.turn_rate().unwrap().to_degrees() - 8.0).abs() < 1e-4);
        assert_eq!(unit.turret, None);
        let attack = unit.attack.expect("its missiles");
        assert_eq!(
            (attack.attack_type, attack.target_domain, attack.target_type),
            (
                AttackType::DoublyDisjointed,
                TargetDomain::Ground,
                TargetType::SingleTarget
            )
        );
        assert_eq!(
            (attack.damage, attack.range, attack.projectile_speed),
            (30.0, 5.0, Some(14.0))
        );
        assert_eq!(super::super::cycle(&attack), ((4, 2, 4, 22), 32));
        assert_eq!(unit.separation_radius, Some(56.0));
        let recipe = Recipe {
            kind: ObjectKind::Hornet,
            cost: 150,
            frames: 320,
            requires: &[],
            supplies: 2,
        };
        assert_eq!(RECIPE, recipe);
    }
}
