//! The Guard — `factions/syndicate/units/guard.md`.

use crate::defs::units::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MovementAttrs, TargetDomain, TargetType, UnitBase, UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/syndicate/units/guard.md`.
pub(crate) static GUARD: ObjectDef = ObjectDef {
    kind: ObjectKind::Guard,
    name: "Guard",
    faction: Some(Faction::Syndicate),                     // Faction - TheSyndicate
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 36.0, h: 36.0 }, // Silhouette - 36x36
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    // Not listed for the Guard; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`:
    // destructible types only).
    destructible: true,
    groupable: true,    // Groupable - true
    sight_range: 5.0,   // SightRange - 5 (grid units, `core/scale.md > GridUnit`)
    max_hp: Some(80.0), // MaxHP - 80
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 1.0,  // FullArmor - 1
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
        // The laser rifle (design change 0186): one shot every 24 frames, instant hit; the numbers are placeholders.
        attack: Some(AttackAttrs {
            // FullyConnected (laser; instant hit, cannot miss); its Range of 3 grid units makes it the Ranged subtype
            // (`systems/combat.md > FullyConnected > Melee` is adjacent contact).
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
            target_domain: TargetDomain::Universal, // TargetDomain: Universal (ground and air)
            target_type: TargetType::SingleTarget,  // TargetType: SingleTarget
            damage: 18.0,                           // Damage: 18 (placeholder)
            range: 3.0,                             // Range: 3 grid units
            min_range: 0.0,                         // MinRange: 0
            projectile_speed: None,                 // FullyConnected: no projectile
            aim_ticks: 4,                           // AimDuration: 4 frames (placeholder)
            firing_ticks: 1,                        // FiringDuration: 1 frame (placeholder)
            cooldown_ticks: 3,                      // CooldownDuration: 3 frames (placeholder)
            reload_ticks: 16,                       // ReloadDuration: 16 frames (placeholder)
        }),
        unit_control_cost: None,
        tunnel_space_cost: Some(1), // TunnelSpaceCost - 1
        beacon_capacity_cost: None,
        riot_shield: None,
        conversion: None,
        deploy: None,
        worker: None,
        augmentable: None,
        repairs: None,
        spells: &[],
        separation_radius: None,
        target_lock: None,
    }),
    structure: None,
    default_settings: None,
    energy: None,
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::combat::domain_allows;
    use crate::defs::Domain;
    use crate::defs::syndicate::tests::timings;

    /// The M1 unit: a 36×36 HeavyInfantry unit, MaxHP 80, PointArmor 1, FullArmor 1, TurnRateMovement at 180°/tick
    /// (every turn completes within a tick) and a MaxSpeed of 5, TunnelSpaceCost 1.
    #[test]
    fn the_guard_follows_the_design() {
        let guard = ObjectKind::Guard.def();
        assert_eq!(guard.half_extents(), Some(Vec2::new(18.0, 18.0)));
        assert_eq!(guard.max_hp, Some(80.0));
        let one = Armor { point: 1.0, full: 1.0 };
        assert_eq!(guard.armor, one, "PointArmor - 1, FullArmor - 1");
        assert_eq!(guard.faction, Some(Faction::Syndicate));
        let unit = guard.unit.as_ref().unwrap();
        assert_eq!(unit.base, UnitBase::HeavyInfantry);
        let MovementAttrs::TurnRate {
            turn_rate,
            acceleration,
            deceleration,
            ..
        } = unit.movement
        else {
            panic!("{:?} is not TurnRateMovement", unit.base);
        };
        assert_eq!(turn_rate, PI);
        assert_eq!((acceleration, deceleration), (None, None), "infinite");
        assert_eq!(unit.movement.max_speed(), 5.0);
        let attack = unit.attack.unwrap();
        assert_eq!((attack.damage, attack.range, attack.reload_ticks), (18.0, 3.0, 16));
        assert_eq!(unit.tunnel_space_cost, Some(1));
    }

    /// Design change 0186 (`> AttackAttributes`): the Guard's laser rifle hits ground and air (TargetDomain
    /// Universal), instantly, for 18 damage, one shot every 24 frames (Aim 4, Fire 1, Cooldown 3, Reload 16;
    /// placeholders). Its Range, its MinRange and the rest of the Guard are as before.
    #[test]
    fn the_guards_laser_rifle_hits_ground_and_air_every_24_frames() {
        let guard = ObjectKind::Guard.def();
        let attack = guard.unit.as_ref().unwrap().attack.unwrap();
        assert_eq!(attack.target_domain, TargetDomain::Universal);
        for domain in [Domain::Ground, Domain::Air] {
            assert!(domain_allows(attack.target_domain, domain), "{domain:?}");
        }
        assert_eq!(
            attack.attack_type,
            AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
            "instant hit: it can't miss"
        );
        assert_eq!(attack.projectile_speed, None);
        assert_eq!((attack.target_type, attack.damage), (TargetType::SingleTarget, 18.0));
        assert_eq!(timings(&attack), ((4, 1, 3, 16), 24), "one shot every 24 frames");
        assert_eq!((attack.range, attack.min_range), (3.0, 0.0));
        assert_eq!((guard.max_hp, guard.sight_range), (Some(80.0), 5.0));
    }
}
