//! The Sparrow — `factions/cults/units/sparrow.md`.

use crate::defs::units::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MovementAttrs, TargetDomain, TargetType, TurretAttrs, UnitBase,
    UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/cults/units/sparrow.md`: "A **small glider** built at the Airstrip, with a **machine-gun-style** attack
/// that hits **both air and ground**". Its numbers only (slice M5-G2, plan decision T69), its movement and its turret
/// as data: nothing reads its gliding nor its turret's arc and rate yet (slice H1 flies gliders; its SeparationRadius
/// and its flight rules come with it), and its production at the Airstrip comes with that building's slice, so no
/// building makes it yet. Its numbers are the design's "Placeholder stats".
pub(crate) static SPARROW: ObjectDef = ObjectDef {
    kind: ObjectKind::Sparrow,
    name: "Sparrow",
    faction: Some(Faction::Cults),                         // Faction - TheCults
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 40.0, h: 40.0 }, // Silhouette 40x40 space units
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,    // Groupable true
    sight_range: 6.0,   // SightRange 6 grid units
    max_hp: Some(50.0), // MaxHP 50
    max_shields: None,
    armor: Armor {
        point: 0.0, // PointArmor 0
        full: 0.0,  // FullArmor 0
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::Glider, // UnitBase - Glider (must keep moving; circles when idle; systems/units.md)
        // "GliderMovement: IdleSpeed 4 su/frame; MaxSpeed 10 su/frame; Acceleration 0.5 su/frame²; Deceleration 0.5
        // su/frame²; MaxCentripetalAcceleration **0.5 su/frame²** (was 1, "too much grip", user 2026-11-26 ...)"
        // (design change 0355).
        movement: MovementAttrs::Glider {
            idle_speed: 4.0,
            max_speed: 10.0,
            acceleration: Some(0.5),
            deceleration: Some(0.5),
            max_centripetal_acceleration: 0.5,
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        // "Turret: TurnAngle 45° (total), TurnRate 8°/frame": "**45° total cone** (22.5° to each side of its facing)".
        turret: Some(TurretAttrs {
            turn_angle: PI / 4.0,
            turn_rate: 8.0 * PI / 180.0,
        }),
        attack: Some(AttackAttrs {
            // AttackType: FullyConnected (machine gun similar to the Seeker's; **cannot miss**)
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
            target_domain: TargetDomain::Universal, // TargetDomain: **Universal** (air and ground)
            target_type: TargetType::SingleTarget,  // TargetType: SingleTarget
            // "Attack: Damage 3; Range 3 grid units; MinRange 0; Aim 1, Firing 1, Cooldown 1, Reload 1 frames (12 damage
            // per second before armor)."
            damage: 3.0,
            range: 3.0,
            min_range: 0.0,
            projectile_speed: None, // FullyConnected: no projectile
            aim_ticks: 1,
            firing_ticks: 1,
            cooldown_ticks: 1,
            reload_ticks: 1,
        }),
        // `> Produced at - Airstrip (consumes **1 Recruit**; Unit Control cost **1**)` (plan decision T55).
        unit_control_cost: Some(1),
        tunnel_space_cost: None,
        beacon_capacity_cost: None,
        riot_shield: None,
        conversion: None,
        deploy: None,
        worker: None,
        augmentable: None,
        repairs: None,
    }),
    structure: None,
    default_settings: None,
    energy: None,
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::defs::Domain;
    use crate::defs::cults::tests::timings;

    /// A 40×40 Glider, in the air, MaxHP 50, no armour, SightRange 6, Groupable; GliderMovement from an idle 4 su a
    /// frame to 10, with a MaxCentripetalAcceleration of 0.5 (design change 0355); a turret of a 45° cone turning 8° a
    /// frame; a machine gun that can't miss, at air and ground, 3 damage up to 3 grid units, a shot every 4 frames;
    /// Unit Control 1, the Recruit it is made of.
    #[test]
    fn the_sparrow_follows_the_design() {
        let def = ObjectKind::Sparrow.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Sparrow", Category::Unit, Some(Faction::Cults))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(40.0, 40.0)));
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(50.0), None, Armor::NONE, 6.0)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, true));
        let unit = def.unit.as_ref().unwrap();
        assert_eq!((unit.base, unit.base_def().domain), (UnitBase::Glider, Domain::Air));
        let glider = MovementAttrs::Glider {
            idle_speed: 4.0,
            max_speed: 10.0,
            acceleration: Some(0.5),
            deceleration: Some(0.5),
            max_centripetal_acceleration: 0.5,
        };
        assert_eq!(unit.movement, glider, "design change 0355: 0.5, was 1");
        let turret = unit.turret.expect("a turret");
        assert!((turret.turn_angle.to_degrees() - 45.0).abs() < 1e-4, "a 45° cone");
        assert!((turret.turn_rate.to_degrees() - 8.0).abs() < 1e-4, "8° a frame");
        let attack = unit.attack.expect("its machine gun");
        assert_eq!(
            attack.attack_type,
            AttackType::FullyConnected(FullyConnectedSubtype::Ranged)
        );
        assert_eq!(
            (attack.target_domain, attack.target_type, attack.damage),
            (TargetDomain::Universal, TargetType::SingleTarget, 3.0)
        );
        assert_eq!(
            (attack.range, attack.min_range, attack.projectile_speed),
            (3.0, 0.0, None)
        );
        assert_eq!(timings(&attack), ((1, 1, 1, 1), 4), "12 damage a second");
        assert_eq!((unit.rugged_terrain_defense_bonus, unit.worker), (None, None));
        assert_eq!(
            (
                unit.unit_control_cost,
                unit.tunnel_space_cost,
                unit.beacon_capacity_cost
            ),
            (Some(1), None, None)
        );
    }
}
