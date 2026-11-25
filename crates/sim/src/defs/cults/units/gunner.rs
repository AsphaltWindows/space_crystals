//! The Gunner — `factions/cults/units/gunner.md`.

use crate::defs::units::{AttackAttrs, AttackType, MovementAttrs, TargetDomain, TargetType, UnitBase, UnitDef};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/cults/units/gunner.md`: "the Cults' heavier-hitting combat infantry, trained from a Recruit at the
/// Armory", which "Shares the Militant's physical profile but trades air targeting and attack speed for a longer-range,
/// harder-hitting ground attack that **can miss**" (design change 0304). The first kind with a MinRange: a target
/// nearer than 2 grid units is too close to fire at (`systems/controls/behaviors.md > AttackingObject > Target inside
/// MinRange`, slice M3-L2's code, `crate::orders`). Its training comes with slice M5-F2, behind a Gunworks.
///
/// Its attack is TailDisjointed, which the combat code doesn't build yet (`crate::combat::phases`): until slice M5-O3
/// it hits as a FullyConnected attack does, and lets go of a location given by an Attack Ground (the miss, the Attack
/// Ground and the "Target too close" for a location are M5-O3's).
pub(crate) static GUNNER: ObjectDef = ObjectDef {
    kind: ObjectKind::Gunner,
    name: "Gunner",
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
            // AttackType: TailDisjointed (user, 2026-11-26; was FullyConnected; design change 0304, "All numbers are
            // unchanged"): "the target's position is locked at the end of Aiming and the shot lands there at the end of
            // Firing". CanTargetGround: true (derived, `crate::combat::can_target_ground`).
            attack_type: AttackType::TailDisjointed,
            target_domain: TargetDomain::Ground,   // TargetDomain: Ground
            target_type: TargetType::SingleTarget, // TargetType: SingleTarget
            damage: 16.0,                          // Damage: 16
            range: 5.0,                            // Range: 5 grid units
            min_range: 2.0,                        // MinRange: 2 (grid units)
            // No projectile: "Visually still an instant gunshot" (`systems/combat.md > AttackType`: "ProjectileSpeed -
            // number (HeadDisjointed / DoublyDisjointed only)").
            projectile_speed: None,
            aim_ticks: 4,      // AimDuration: 4 frames
            firing_ticks: 2,   // FiringDuration: 2 frames
            cooldown_ticks: 8, // CooldownDuration: 8 frames
            reload_ticks: 8,   // ReloadDuration: 8 frames
        }),
        // `factions/cults/economy.md > RecruitmentHall > Unit Control Tracking`: "A trained unit's Unit Control cost
        // equals the number of Recruits consumed to produce it": one, the Recruit an Armory trains it from
        // (`factions/cults/structures/armory.md > Training`; plan decision T55).
        unit_control_cost: Some(1),
        tunnel_space_cost: None,
        beacon_capacity_cost: None,
        riot_shield: None,
        conversion: None,
        deploy: None,
        worker: None,
    }),
    structure: None,
    default_settings: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{can_target_ground, domain_allows};
    use crate::defs::Domain;
    use crate::defs::cults::tests::{assert_infantry, timings};

    /// The Militant's body (LightInfantry, 24×24, MaxHP 40, no armour, SightRange 5, MaxSpeed 4) with its gun:
    /// TailDisjointed (design change 0304), so it can target the ground, ground only, single target, 16 damage from 2
    /// to 5 grid units, no projectile, one shot every 22 frames (Aim 4, Fire 2, Cooldown 8, Reload 8); Unit Control 1.
    #[test]
    fn the_gunner_follows_the_design() {
        let attack = assert_infantry(ObjectKind::Gunner);
        assert_eq!(attack.attack_type, AttackType::TailDisjointed, "design change 0304");
        assert!(can_target_ground(attack.attack_type), "CanTargetGround: true (derived)");
        assert_eq!(
            (attack.target_domain, attack.target_type, attack.damage),
            (TargetDomain::Ground, TargetType::SingleTarget, 16.0)
        );
        assert!(domain_allows(attack.target_domain, Domain::Ground));
        assert!(!domain_allows(attack.target_domain, Domain::Air), "ground only");
        assert_eq!(
            (attack.range, attack.min_range, attack.projectile_speed),
            (5.0, 2.0, None)
        );
        assert_eq!(timings(&attack), ((4, 2, 8, 8), 22), "one shot every 22 frames");
        assert_eq!(ObjectKind::Gunner.def().name, "Gunner");
    }

    /// "Shares the Militant's physical profile": the two are the same unit but for their kind, their name and their
    /// attack.
    #[test]
    fn the_gunner_has_the_militants_body() {
        let (militant, gunner) = (ObjectKind::Militant.def(), ObjectKind::Gunner.def());
        let unit = UnitDef {
            attack: militant.unit.as_ref().unwrap().attack,
            ..gunner.unit.clone().unwrap()
        };
        assert_eq!(militant.unit.as_ref(), Some(&unit), "but for the attack");
        let object = ObjectDef {
            kind: ObjectKind::Militant,
            name: militant.name,
            unit: Some(unit),
            ..gunner.clone()
        };
        // `ObjectDef`'s `PartialEq` compares every field.
        assert_eq!(&object, militant, "but for the kind and the name");
    }
}
