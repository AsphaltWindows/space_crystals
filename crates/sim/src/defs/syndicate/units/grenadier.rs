//! The Grenadier — `factions/syndicate/units/grenadier.md` (design changes 0187 and 0260; slice M4-U).

use crate::defs::units::{AttackAttrs, AttackType, MovementAttrs, TargetDomain, TargetType, UnitBase, UnitDef};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;
use crate::units::SU_PER_GU;

/// `factions/syndicate/units/grenadier.md`: the Syndicate's explosives infantry, "Produced at - Hideout" behind a
/// Chemical Plant (its recipe there, `defs::syndicate::structures::hideout`, with its TechPrerequisite). Its grenade is
/// a DoublyDisjointed area attack: a projectile to where the target stood at the end of its Aiming, whose blast hits
/// whatever is there when it lands (`crate::combat::projectiles`, `crate::combat::area`, slice M4-O); and, like every
/// attack that can miss, it can Attack Ground (`systems/combat.md > AttackType`: "CanTargetGround - derived (... true
/// for TailDisjointed/DoublyDisjointed)"). Its numbers are the design's "Placeholder stats (designer, 2026-11-25,
/// Q0071; tune later)" (design change 0260).
pub(crate) static GRENADIER: ObjectDef = ObjectDef {
    kind: ObjectKind::Grenadier,
    name: "Grenadier",
    faction: Some(Faction::Syndicate),                     // Faction - TheSyndicate
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 36.0, h: 36.0 }, // Silhouette - 36x36
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    // Not listed for the Grenadier; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,    // Groupable - true (assumed)
    sight_range: 5.0,   // SightRange - 5
    max_hp: Some(70.0), // MaxHP - 70
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 1.0,  // FullArmor - 1
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::HeavyInfantry, // UnitBase - HeavyInfantry (assumed, like all Syndicate infantry)
        // UnitBaseAttributes[HeavyInfantry]
        movement: MovementAttrs::TurnRate {
            turn_rate: PI,      // TurnRate: 180 degrees/frame
            acceleration: None, // Acceleration: infinite
            deceleration: None, // Deceleration: infinite
            max_speed: 5.0,     // MaxSpeed: 5 space units/frame
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        turret: None,                       // Not listed: it throws from its base, as every infantry attacks
        // AttackAttributes: the explosive grenade, "a slow projectile that flies to where the target was when the
        // Grenadier finished aiming and explodes there, dealing moderate-high area damage. Ground only."
        attack: Some(AttackAttrs {
            // AttackType: DoublyDisjointed (slow projectile to the target's position at the end of Aiming; can be
            // dodged)
            attack_type: AttackType::DoublyDisjointed,
            target_domain: TargetDomain::Ground, // TargetDomain: Ground
            // TargetType: AoE (moderate-high damage); AoE radius: 40 space units (0.625 grid units)
            target_type: TargetType::AoE {
                radius: 40.0 / SU_PER_GU,
            },
            damage: 100.0,               // Damage: 100 (total over the circle)
            range: 4.0,                  // Range: 4 grid units
            min_range: 0.0,              // MinRange: 0
            projectile_speed: Some(8.0), // ProjectileSpeed: 8 space units/frame
            aim_ticks: 6,                // AimDuration: 6 frames
            firing_ticks: 2,             // FiringDuration: 2 frames
            cooldown_ticks: 4,           // CooldownDuration: 4 frames
            reload_ticks: 28,            // ReloadDuration: 28 frames
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
    }),
    structure: None,
    default_settings: None,
    energy: None,
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::combat::{can_target_ground, domain_allows, flight_ticks, hits_domain};
    use crate::defs::Domain;
    use crate::defs::syndicate::tests::timings;

    /// The stat block (design change 0260's placeholders): a 36×36 HeavyInfantry unit ("assumed, like all Syndicate
    /// infantry"), MaxHP 70, PointArmor 1, FullArmor 1, SightRange 5, TunnelSpaceCost 1, Groupable ("true (assumed)"),
    /// TurnRateMovement at 180° per frame with infinite acceleration and deceleration and a MaxSpeed of 5 space units
    /// per frame, no turret.
    #[test]
    fn the_grenadier_follows_the_design() {
        let def = ObjectKind::Grenadier.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Grenadier", Category::Unit, Some(Faction::Syndicate))
        );
        assert_eq!(def.half_extents(), Some(Vec2::new(18.0, 18.0)), "36x36");
        assert_eq!((def.max_hp, def.max_shields), (Some(70.0), None));
        let one = Armor { point: 1.0, full: 1.0 };
        assert_eq!((def.armor, def.sight_range), (one, 5.0));
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, true));
        let unit = def.unit.as_ref().unwrap();
        assert_eq!(unit.base, UnitBase::HeavyInfantry);
        let MovementAttrs::TurnRate {
            turn_rate,
            acceleration,
            deceleration,
            max_speed,
        } = unit.movement
        else {
            panic!("{:?} is not TurnRateMovement", unit.movement);
        };
        assert_eq!((turn_rate, acceleration, deceleration), (PI, None, None));
        assert_eq!(max_speed, 5.0);
        assert_eq!((unit.turret, unit.worker), (None, None));
        assert_eq!(
            (
                unit.tunnel_space_cost,
                unit.unit_control_cost,
                unit.beacon_capacity_cost
            ),
            (Some(1), None, None)
        );
    }

    /// `> AttackAttributes`: a DoublyDisjointed Ground attack of Damage 100 over a circle of 40 space units, 0.625 grid
    /// units, at a Range of 4 grid units and no MinRange; its projectile flies 8 space units a frame, so "a 4 GU throw
    /// takes 2 s" (32 frames: design change 0260's verification, "the grenade visibly arcs about 2 s to a target 4 GU
    /// away"); "one grenade per 40 frames, 2.5 s" (Aim 6, Firing 2, Cooldown 4, Reload 28). It targets ground units, not
    /// air ones ("Ground only"), and its blast hits the ground and the underground objects under it
    /// (`systems/combat.md > AoEDamage`: "Ground AoE hits Ground and Underground objects"), not the air; and it can
    /// target the ground ("CanTargetGround: true (derived)").
    #[test]
    fn its_grenade_is_a_slow_area_attack_that_can_target_the_ground() {
        let attack = ObjectKind::Grenadier.def().unit.as_ref().unwrap().attack.unwrap();
        assert_eq!(attack.attack_type, AttackType::DoublyDisjointed);
        assert_eq!(attack.target_domain, TargetDomain::Ground);
        assert_eq!(attack.target_type, TargetType::AoE { radius: 0.625 });
        assert_eq!(0.625 * SU_PER_GU, 40.0, "40 space units");
        assert_eq!((attack.damage, attack.range, attack.min_range), (100.0, 4.0, 0.0));
        assert_eq!(attack.projectile_speed, Some(8.0));
        let four_gu = flight_ticks(Vec2::ZERO, Vec2::new(4.0 * SU_PER_GU, 0.0), attack.projectile_speed);
        assert_eq!(four_gu, 32, "2 seconds at 16 frames a second");
        assert_eq!(timings(&attack), ((6, 2, 4, 28), 40), "one grenade per 40 frames");
        assert!(can_target_ground(attack.attack_type));
        let domain = attack.target_domain;
        assert!(domain_allows(domain, Domain::Ground) && !domain_allows(domain, Domain::Air));
        let blast = [Domain::Ground, Domain::Underground, Domain::Air].map(|target| hits_domain(domain, target));
        assert_eq!(blast, [true, true, false]);
    }
}
