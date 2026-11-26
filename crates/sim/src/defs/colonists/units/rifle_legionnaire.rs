//! The Rifle Legionnaire — `factions/colonists/units/rifle_legionnaire.md`: the ranged form of the Legionnaire
//! (`factions/colonists/units/weapon_conversion.md`).

use super::weapon_conversion::LEGIONNAIRE_CONVERSION;
use crate::defs::units::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MovementAttrs, TargetDomain, TargetType, UnitBase, UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/colonists/units/rifle_legionnaire.md`: the ranged form of the Legionnaire, "Not directly produced —
/// reached only by converting a Sabre Legionnaire" (slice M3-B).
pub(crate) static RIFLE_LEGIONNAIRE: ObjectDef = ObjectDef {
    kind: ObjectKind::RifleLegionnaire,
    name: "Rifle Legionnaire",
    faction: Some(Faction::Colonists),                     // Faction - Colonists
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 36.0, h: 36.0 }, // Silhouette - 36x36 space units (square)
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                                    // Destructible - true
    groupable: true,                                       // Groupable - true
    sight_range: 5.0,                                      // SightRange - 5 grid units
    max_hp: Some(70.0),                                    // MaxHP - 70
    max_shields: Some(10.0),                               // MaxShields - 10
    armor: Armor {
        point: 0.0, // PointArmor - 0
        full: 0.0,  // FullArmor - 0
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::HeavyInfantry, // UnitBase - HeavyInfantry
        // UnitBaseAttributes[HeavyInfantry]
        movement: MovementAttrs::TurnRate {
            turn_rate: PI,      // TurnRate: 180 degrees/frame
            acceleration: None, // Acceleration: infinite
            deceleration: None, // Deceleration: infinite
            max_speed: 5.0,     // MaxSpeed: 5 space units/frame
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        turret: None,                       // TurretAttributes - None
        attack: Some(AttackAttrs {
            // AttackType: FullyConnected (Ranged)
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
            target_domain: TargetDomain::Ground,   // TargetDomain: Ground
            target_type: TargetType::SingleTarget, // TargetType: SingleTarget
            damage: 10.0,                          // Damage: 10
            range: 4.0,                            // Range: 4 grid units
            min_range: 0.0,                        // MinRange: 0
            projectile_speed: None,                // FullyConnected: no projectile
            aim_ticks: 3,                          // AimDuration: 3 frames
            firing_ticks: 2,                       // FiringDuration: 2 frames
            cooldown_ticks: 7,                     // CooldownDuration: 7 frames
            reload_ticks: 4,                       // ReloadDuration: 4 frames
        }),
        unit_control_cost: None,
        tunnel_space_cost: None,
        beacon_capacity_cost: Some(2), // BeaconCapacityCost - 2
        riot_shield: None,
        conversion: Some(LEGIONNAIRE_CONVERSION), // `factions/colonists/units/weapon_conversion.md`
        deploy: None,
        worker: None,
        augmentable: None,
    }),
    structure: None,
    default_settings: None,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The Legionnaire's ranged form.
    #[test]
    fn the_rifle_legionnaire_follows_the_design() {
        let attack = crate::defs::colonists::tests::assert_legionnaire(ObjectKind::RifleLegionnaire);
        assert_eq!(
            attack,
            AttackAttrs {
                attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
                target_domain: TargetDomain::Ground,
                target_type: TargetType::SingleTarget,
                damage: 10.0,
                range: 4.0,
                min_range: 0.0,
                projectile_speed: None,
                aim_ticks: 3,
                firing_ticks: 2,
                cooldown_ticks: 7,
                reload_ticks: 4,
            }
        );
    }
}
