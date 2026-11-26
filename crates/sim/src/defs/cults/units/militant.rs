//! The Militant — `factions/cults/units/militant.md`, named by design change 0302.

use crate::defs::units::{
    AttackAttrs, AttackType, Augmentable, FullyConnectedSubtype, MovementAttrs, TargetDomain, TargetType, UnitBase,
    UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/cults/units/militant.md`: "the Cults' basic combat infantry, trained from a Recruit at the Armory. A light
/// infantry unit with a fully connected attack effective against both ground and air targets." Its training comes with
/// slice M5-F2 (the Armory's `production`).
pub(crate) static MILITANT: ObjectDef = ObjectDef {
    kind: ObjectKind::Militant,
    name: "Militant",
    faction: Some(Faction::Cults),                         // Faction - TheCults
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 24.0, h: 24.0 }, // Silhouette - 24x24 space units (square)
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                                    // Destructible - true
    groupable: true,                                       // Groupable - true
    sight_range: 5.0,                                      // SightRange - 5 grid units
    max_hp: Some(40.0),                                    // MaxHP - 40
    max_shields: None,
    armor: Armor {
        point: 0.0, // PointArmor - 0
        full: 0.0,  // FullArmor - 0
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::LightInfantry, // UnitBase - LightInfantry
        // UnitBaseAttributes[LightInfantry]
        movement: MovementAttrs::TurnRate {
            turn_rate: PI,      // TurnRate: 180 degrees/frame
            acceleration: None, // Acceleration: infinite
            deceleration: None, // Deceleration: infinite
            max_speed: 4.0,     // MaxSpeed: 4 space units/frame
        },
        rugged_terrain_defense_bonus: Some(0.5), // RuggedTerrainDefenseBonus: 50%
        turret: None,                            // TurretAttributes - None
        attack: Some(AttackAttrs {
            // AttackType: FullyConnected; its Range of 4 grid units makes it the Ranged subtype (`systems/combat.md >
            // FullyConnected > Melee` is adjacent contact).
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
            target_domain: TargetDomain::Universal, // TargetDomain: Universal
            target_type: TargetType::SingleTarget,  // TargetType: SingleTarget
            damage: 4.0,                            // Damage: 4
            range: 4.0,                             // Range: 4 grid units
            min_range: 0.0,                         // MinRange: 0
            projectile_speed: None,                 // FullyConnected: no projectile
            aim_ticks: 2,                           // AimDuration: 2 frames
            firing_ticks: 2,                        // FiringDuration: 2 frames
            cooldown_ticks: 1,                      // CooldownDuration: 1 frame
            reload_ticks: 3,                        // ReloadDuration: 3 frames
        }),
        // `factions/cults/structures/recruitment_hall.md > Unit Control Tracking`: "A trained unit's Unit Control cost
        // equals the number of Recruits consumed to produce it": one, the Recruit an Armory trains it from
        // (`factions/cults/structures/armory.md > Training`; plan decision T55).
        unit_control_cost: Some(1),
        tunnel_space_cost: None,
        beacon_capacity_cost: None,
        riot_shield: None,
        conversion: None,
        deploy: None,
        worker: None,
        // (M5-N1a) `factions/cults/augmentation.md > Increments`: "Armed infantry (Militant, Gunner)".
        augmentable: Some(Augmentable { unarmed: false }),
        repairs: None,
    }),
    structure: None,
    default_settings: None,
    energy: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::domain_allows;
    use crate::defs::Domain;
    use crate::defs::cults::tests::{assert_infantry, timings};

    /// The Recruit's body (LightInfantry, 24×24, MaxHP 40, no armour, SightRange 5, MaxSpeed 4) with its gun:
    /// FullyConnected, ground and air (Universal), single target, 4 damage at up to 4 grid units, no MinRange, one shot
    /// every 8 frames (Aim 2, Fire 2, Cooldown 1, Reload 3); Unit Control 1. Its name is design change 0302's.
    #[test]
    fn the_militant_follows_the_design() {
        let attack = assert_infantry(ObjectKind::Militant);
        assert_eq!(
            attack.attack_type,
            AttackType::FullyConnected(FullyConnectedSubtype::Ranged)
        );
        assert_eq!(
            (attack.target_domain, attack.target_type, attack.damage),
            (TargetDomain::Universal, TargetType::SingleTarget, 4.0)
        );
        for domain in [Domain::Ground, Domain::Air] {
            assert!(domain_allows(attack.target_domain, domain), "{domain:?}");
        }
        assert_eq!(
            (attack.range, attack.min_range, attack.projectile_speed),
            (4.0, 0.0, None)
        );
        assert_eq!(timings(&attack), ((2, 2, 1, 3), 8), "one shot every 8 frames");
        assert_eq!(ObjectKind::Militant.def().name, "Militant");
    }
}
