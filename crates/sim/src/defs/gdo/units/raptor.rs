//! The Raptor — `factions/gdo/units/raptor.md` (design change 0366, slice M6-GK): "GDO light missile vehicle produced
//! at the Factory — a Humvee-like wheeled vehicle with a missile launcher turret. Like a Rocketman whose missiles are
//! always homing. Does not carry infantry." Its numbers are the design's "Placeholder stats" (plan decision T84), its
//! movement, its turret and its homing missile among them. No player produces one until slice M6-GF makes its Factory
//! entry ([`RECIPE`], held) live.

use crate::defs::structures::Recipe;
use crate::defs::units::{
    AttackAttrs, AttackType, MovementAttrs, TargetDomain, TargetType, TurretAttrs, UnitBase, UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/gdo/units/raptor.md`: its placeholder stats (design change 0366).
pub(crate) const RAPTOR: ObjectDef = ObjectDef {
    kind: ObjectKind::Raptor,
    name: "Raptor",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Unit,    // Entity Type - Unit
    // "Silhouette 48x32 space units": its length along its facing (`systems/units.md > Silhouette Rotation`), turned
    // with the facing (`ObjectDef::silhouette_turns`).
    footprint: Footprint::Silhouette { w: 48.0, h: 32.0 },
    selectable: true, // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 7.0,    // SightRange 7 grid units
    max_hp: Some(110.0), // MaxHP 110
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 1.0,  // FullArmor 1
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        // UnitBase - WheeledVehicle (fast, fixed turn radius, can reverse, directional armor)
        base: UnitBase::WheeledVehicle,
        // "FixedTurnRadiusMovement: MinimumTurnRadius 80 su; ForwardMaxSpeed **10 su/frame** (very fast);
        // ForwardAcceleration 1; ReverseMaxSpeed 3; ReverseAcceleration 0.5; Deceleration 1."
        movement: MovementAttrs::FixedTurnRadius {
            minimum_turn_radius: 80.0,
            forward_acceleration: Some(1.0),
            forward_max_speed: 10.0,
            reverse_acceleration: Some(0.5),
            reverse_max_speed: 3.0,
            deceleration: Some(1.0),
        },
        // "TurnAngle: **60°** full arc (±30° from the vehicle's facing) — placeholder. Deliberately narrow"; "Turret
        // TurnRate 10°/frame."
        turret: Some(TurretAttrs {
            turn_angle: PI / 3.0,
            turn_rate: 10.0 * PI / 180.0,
        }),
        attack: Some(AttackAttrs {
            // "AttackType: HeadDisjointed (homing missiles — track the target, cannot miss)"
            attack_type: AttackType::HeadDisjointed,
            target_domain: TargetDomain::Universal, // "TargetDomain: Universal (ground and air)"
            // **Placeholder (not in the design: the Raptor's TargetType)**: SingleTarget, its Damage a missile's.
            target_type: TargetType::SingleTarget,
            // "Attack: Damage 14; Range 6 grid units; MinRange 0; ProjectileSpeed 14 su/frame; Aim 4, Firing 2,
            // Cooldown 4, Reload 22 frames (one missile per 32 frames)."
            damage: 14.0,
            range: 6.0,
            min_range: 0.0,
            projectile_speed: Some(14.0),
            aim_ticks: 4,
            firing_ticks: 2,
            cooldown_ticks: 4,
            reload_ticks: 22,
        }),
        unit_control_cost: Some(2), // UnitControlCost 2
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/factory.md > Produces`: "Raptor: 100 Space Crystals, 240 frames (15 seconds)
/// (placeholder)". Held until slice M6-GF (`crate::defs::PLANNED_RECIPES`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::Raptor,
    cost: 100,
    frames: 240,
    requires: &[],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::super::cycle;
    use super::*;

    /// The placeholder stats: a WheeledVehicle, 48×32, MaxHP 110, armour 1/1, SightRange 7, Unit Control 2; up to 10 su
    /// a frame on an 80 su turn radius; a 60° turret turning 10° a frame; a homing missile at ground and air, 14 damage
    /// up to 6 grid units at 14 su a frame, one every 32 frames. Its Factory entry, 100 Space Crystals and 240 frames, is
    /// held.
    #[test]
    fn the_raptor_follows_the_design() {
        let unit = super::super::assert_stat_block(
            ObjectKind::Raptor,
            "Raptor",
            (48.0, 32.0),
            UnitBase::WheeledVehicle,
            110.0,
            (1.0, 1.0),
            7.0,
            2,
        );
        assert_eq!((unit.movement.max_speed(), unit.can_reverse()), (10.0, true));
        let turret = unit.turret.expect("its launcher");
        assert!(
            (turret.turn_angle.to_degrees() - 60.0).abs() < 1e-4 && (turret.turn_rate.to_degrees() - 10.0).abs() < 1e-4
        );
        let attack = unit.attack.expect("its missile");
        assert_eq!(
            (attack.attack_type, attack.target_domain, attack.target_type),
            (
                AttackType::HeadDisjointed,
                TargetDomain::Universal,
                TargetType::SingleTarget
            )
        );
        assert_eq!(
            (attack.damage, attack.range, attack.projectile_speed),
            (14.0, 6.0, Some(14.0))
        );
        assert_eq!(cycle(&attack), ((4, 2, 4, 22), 32));
        let recipe = Recipe {
            kind: ObjectKind::Raptor,
            cost: 100,
            frames: 240,
            requires: &[],
            supplies: 0,
        };
        assert_eq!(RECIPE, recipe);
    }
}
