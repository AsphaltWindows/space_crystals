//! The Sabre Legionnaire — `factions/colonists/units/sabre_legionnaire.md`: the melee form of the Legionnaire
//! (`factions/colonists/units/weapon_conversion.md`).

use super::weapon_conversion::LEGIONNAIRE_CONVERSION;
use crate::defs::units::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MELEE_CONTACT_GU, MovementAttrs, TargetDomain, TargetType,
    UnitBase, UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/colonists/units/sabre_legionnaire.md`: the melee form of the Legionnaire.
/// `factions/colonists/units/weapon_conversion.md`: it shares the Rifle Legionnaire's physical profile and differs
/// only in its AttackAttributes (a test keeps the two stat blocks so).
pub(crate) const SABRE_LEGIONNAIRE: ObjectDef = ObjectDef {
    kind: ObjectKind::SabreLegionnaire,
    name: "Sabre Legionnaire",
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
            // AttackType: FullyConnected (Melee)
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Melee),
            target_domain: TargetDomain::Ground,   // TargetDomain: Ground
            target_type: TargetType::SingleTarget, // TargetType: SingleTarget
            damage: 15.0,                          // Damage: 15
            // Range: melee (adjacent contact): a reach of 8 su, a placeholder (design change 0149).
            range: MELEE_CONTACT_GU,
            min_range: 0.0,         // none listed: contact is as near as a target gets
            projectile_speed: None, // FullyConnected: no projectile
            aim_ticks: 3,           // AimDuration: 3 frames
            firing_ticks: 3,        // FiringDuration: 3 frames
            cooldown_ticks: 1,      // CooldownDuration: 1 frame
            reload_ticks: 5,        // ReloadDuration: 5 frames
        }),
        beacon_capacity_cost: Some(2),            // BeaconCapacityCost - 2
        conversion: Some(LEGIONNAIRE_CONVERSION), // `factions/colonists/units/weapon_conversion.md`
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::gu;

    /// The Legionnaire's melee form, whose "Range: melee (adjacent contact)" is the placeholder reach of 8 su of
    /// design change 0149.
    #[test]
    fn the_sabre_legionnaire_follows_the_design() {
        let attack = crate::defs::colonists::tests::assert_legionnaire(ObjectKind::SabreLegionnaire);
        assert_eq!(
            attack,
            AttackAttrs {
                attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Melee),
                target_domain: TargetDomain::Ground,
                target_type: TargetType::SingleTarget,
                damage: 15.0,
                range: MELEE_CONTACT_GU,
                min_range: 0.0,
                projectile_speed: None,
                aim_ticks: 3,
                firing_ticks: 3,
                cooldown_ticks: 1,
                reload_ticks: 5,
            }
        );
        assert_eq!(gu(attack.range), 8.0, "8 space units, edge to edge");
    }
}
