//! The Twister — `factions/gdo/units/twister.md` (design change 0366, slice M6-GK): "GDO vehicle produced at the
//! Factory. A little slower than the Raptor. Mounts a machine gun on a fully rotating turret for anti-air defense, and
//! lays hidden mines." Its numbers are the design's "Placeholder stats" (plan decision T84), its machine gun among
//! them. Its mines (`> Mines`: a stock of 3, Buy Mine and Lay Mine; the mine is a kind of its own, `ObjectKind::Mine`)
//! are slice M6-GM's, and its Factory entry ([`RECIPE`], held) slice M6-GF's; no player produces one before.

use crate::defs::structures::Recipe;
use crate::defs::units::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MovementAttrs, TargetDomain, TargetType, TurretAttrs, UnitBase,
    UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/gdo/units/twister.md`: its placeholder stats (design change 0366).
pub(crate) const TWISTER: ObjectDef = ObjectDef {
    kind: ObjectKind::Twister,
    name: "Twister",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Unit,    // Entity Type - Unit
    // "Silhouette 52x36 space units": its length along its facing (`systems/units.md > Silhouette Rotation`), turned
    // with the facing (`ObjectDef::silhouette_turns`).
    footprint: Footprint::Silhouette { w: 52.0, h: 36.0 },
    selectable: true, // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 6.0,    // SightRange 6 grid units
    max_hp: Some(140.0), // MaxHP 140
    armor: Armor {
        point: 2.0, // PointArmor 2
        full: 2.0,  // FullArmor 2
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::WheeledVehicle, // UnitBase - WheeledVehicle (a little slower than the Raptor)
        // "FixedTurnRadiusMovement: MinimumTurnRadius 80 su; ForwardMaxSpeed **8 su/frame**; ForwardAcceleration 1;
        // ReverseMaxSpeed 3; ReverseAcceleration 0.5; Deceleration 1."
        movement: MovementAttrs::FixedTurnRadius {
            minimum_turn_radius: 80.0,
            forward_acceleration: Some(1.0),
            forward_max_speed: 8.0,
            reverse_acceleration: Some(0.5),
            reverse_max_speed: 3.0,
            deceleration: Some(1.0),
        },
        // "TurnAngle: 360° (full rotation)"; "Turret TurnRate 12°/frame."
        turret: Some(TurretAttrs {
            turn_angle: 2.0 * PI,
            turn_rate: 12.0 * PI / 180.0,
        }),
        attack: Some(AttackAttrs {
            // "AttackType: FullyConnected (Ranged) — machine-gun type weapon"
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
            target_domain: TargetDomain::Air, // "TargetDomain: **Air** (anti-air only)"
            // **Placeholder (not in the design: the Twister's TargetType)**: SingleTarget, a machine gun's (the
            // Seeker's and the Sparrow's).
            target_type: TargetType::SingleTarget,
            // "Machine gun: Damage 4; Range 6 grid units; Aim 1, Firing 1, Cooldown 1, Reload 1 frames (16 damage per
            // second before armor)."
            damage: 4.0,
            range: 6.0,
            min_range: 0.0,
            projectile_speed: None, // FullyConnected: no projectile
            aim_ticks: 1,
            firing_ticks: 1,
            cooldown_ticks: 1,
            reload_ticks: 1,
        }),
        unit_control_cost: Some(2), // UnitControlCost 2
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/factory.md > Produces`: "Twister: 125 Space Crystals, 320 frames (20 seconds)
/// (placeholder)", and `> Placeholder stats`: "Cost: Factory, 125 Space Crystals **+ 1 Supply** (user, 2026-11-26),
/// 320 frames" (design change 0441). Held until slice M6-GF (`crate::defs::PLANNED_RECIPES`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::Twister,
    cost: 125,
    frames: 320,
    requires: &[],
    supplies: 1,
};

#[cfg(test)]
mod tests {
    use super::super::cycle;
    use super::*;

    /// The placeholder stats: a WheeledVehicle, 52×36, MaxHP 140, armour 2/2, SightRange 6, Unit Control 2; up to 8 su
    /// a frame, slower than the Raptor, on its turn radius; a full turret turning 12° a frame; a machine gun at air
    /// only, 4 damage up to 6 grid units every 4 frames. Its Factory entry, 125 Space Crystals and 1 Supply in 320
    /// frames, is held.
    #[test]
    fn the_twister_follows_the_design() {
        let unit = super::super::assert_stat_block(
            ObjectKind::Twister,
            "Twister",
            (52.0, 36.0),
            UnitBase::WheeledVehicle,
            140.0,
            (2.0, 2.0),
            6.0,
            2,
        );
        let raptor = ObjectKind::Raptor.def().unit.as_ref().unwrap();
        assert!(unit.movement.max_speed() == 8.0 && unit.movement.max_speed() < raptor.movement.max_speed());
        let turret = unit.turret.expect("its machine gun's turret");
        assert_eq!(turret.turn_angle, 2.0 * PI);
        assert!((turret.turn_rate.to_degrees() - 12.0).abs() < 1e-4);
        let attack = unit.attack.expect("its machine gun");
        assert_eq!(
            (attack.attack_type, attack.target_domain, attack.target_type),
            (
                AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
                TargetDomain::Air,
                TargetType::SingleTarget
            )
        );
        assert_eq!(
            (attack.damage, attack.range, cycle(&attack)),
            (4.0, 6.0, ((1, 1, 1, 1), 4))
        );
        let recipe = Recipe {
            kind: ObjectKind::Twister,
            cost: 125,
            frames: 320,
            requires: &[],
            supplies: 1,
        };
        assert_eq!(RECIPE, recipe);
    }
}
