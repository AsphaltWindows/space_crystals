//! The Seeker — `factions/cults/units/seeker.md` (design change 0364: "Renamed from Jackal").

use crate::defs::units::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MovementAttrs, TargetDomain, TargetType, TurretAttrs, UnitBase,
    UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/cults/units/seeker.md`: "The Cults' **light vehicle**: a wheeled ATV/buggy built at the Chop Shop, with a
/// **machine-gun turret** that fires **low damage at a high rate**, **cannot miss**, and **rotates 360°**". Its numbers
/// only (slice M5-G2, plan decision T69), its movement and its turret as data: nothing reads its turning radius nor its
/// turret's arc and rate yet (slices V1 and V2 turn vehicles), and its production at the Chop Shop comes with that
/// building's slice, so no building makes it yet. Its numbers are the design's "Placeholder stats".
pub(crate) static SEEKER: ObjectDef = ObjectDef {
    kind: ObjectKind::Seeker,
    name: "Seeker",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Unit,      // Entity Type - Unit
    // "Silhouette 56x40 space units (length x width)": its length along its facing (`systems/units.md > Silhouette
    // Rotation`: "facing east it is wide and flat"), axis-aligned until slice V1 rotates it (`docs/movement.md`, gap G6).
    footprint: Footprint::Silhouette { w: 56.0, h: 40.0 },
    selectable: true, // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 6.0,    // SightRange 6 grid units
    max_hp: Some(120.0), // MaxHP 120
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 1.0,  // FullArmor 1
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::WheeledVehicle, // UnitBase - WheeledVehicle
        // "FixedTurnRadiusMovement: MinimumTurnRadius 64 su (1 grid unit); ForwardMaxSpeed 8 su/frame;
        // ForwardAcceleration 1 su/frame²; ReverseMaxSpeed 3 su/frame; ReverseAcceleration 0.5 su/frame²; Deceleration 1
        // su/frame²."
        movement: MovementAttrs::FixedTurnRadius {
            minimum_turn_radius: 64.0,
            forward_acceleration: Some(1.0),
            forward_max_speed: 8.0,
            reverse_acceleration: Some(0.5),
            reverse_max_speed: 3.0,
            deceleration: Some(1.0),
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        // "Turret: TurnAngle 360°, TurnRate 12°/frame."
        turret: Some(TurretAttrs {
            turn_angle: 2.0 * PI,
            turn_rate: 12.0 * PI / 180.0,
        }),
        attack: Some(AttackAttrs {
            // AttackType: FullyConnected (machine gun; cannot miss)
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
            target_domain: TargetDomain::Universal, // TargetDomain: **Universal** (ground and air)
            target_type: TargetType::SingleTarget,  // TargetType: SingleTarget
            // "Attack: Damage 3; Range 4 grid units; MinRange 0; Aim 1, Firing 1, Cooldown 1, Reload 1 frames (one shot
            // every 4 frames: 12 damage per second before armor)."
            damage: 3.0,
            range: 4.0,
            min_range: 0.0,
            projectile_speed: None, // FullyConnected: no projectile
            aim_ticks: 1,
            firing_ticks: 1,
            cooldown_ticks: 1,
            reload_ticks: 1,
        }),
        // "Produced at - Chop Shop (consumes **2 Recruits**; Unit Control cost **2**", and
        // `factions/cults/structures/README.md > Cults Vehicles`: "vehicles cost one Unit Control per Recruit consumed"
        // (plan decision T55).
        unit_control_cost: Some(2),
        tunnel_space_cost: None,
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
    use crate::defs::cults::tests::timings;

    /// A 56×40 WheeledVehicle (design change 0364's Seeker, once the Jackal), MaxHP 120, PointArmor 1, FullArmor 1,
    /// SightRange 6, Groupable; FixedTurnRadiusMovement (a turning radius of a grid unit, 8 su a frame forward and 3 in
    /// reverse); a turret that turns all the way round, 12° a frame; a machine gun that can't miss, at ground and air,
    /// 3 damage up to 4 grid units, a shot every 4 frames; Unit Control 2, the Recruits it is made of.
    #[test]
    fn the_seeker_follows_the_design() {
        let def = ObjectKind::Seeker.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Seeker", Category::Unit, Some(Faction::Cults))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(56.0, 40.0)), "length x width");
        let one = Armor { point: 1.0, full: 1.0 };
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(120.0), None, one, 6.0)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, true));
        let unit = def.unit.as_ref().unwrap();
        assert_eq!(unit.base, UnitBase::WheeledVehicle);
        let MovementAttrs::FixedTurnRadius {
            minimum_turn_radius,
            forward_acceleration,
            forward_max_speed,
            reverse_acceleration,
            reverse_max_speed,
            deceleration,
        } = unit.movement
        else {
            panic!("{:?} is not FixedTurnRadiusMovement", unit.movement);
        };
        assert_eq!(
            (minimum_turn_radius, forward_max_speed, reverse_max_speed),
            (64.0, 8.0, 3.0)
        );
        assert_eq!(
            (forward_acceleration, reverse_acceleration, deceleration),
            (Some(1.0), Some(0.5), Some(1.0))
        );
        let turret = unit.turret.expect("a turret");
        assert!((turret.turn_angle - 2.0 * PI).abs() < 1e-6, "360°");
        assert!((turret.turn_rate.to_degrees() - 12.0).abs() < 1e-4, "12° a frame");
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
            (4.0, 0.0, None)
        );
        assert_eq!(timings(&attack), ((1, 1, 1, 1), 4), "one shot every 4 frames");
        assert_eq!((unit.rugged_terrain_defense_bonus, unit.worker), (None, None));
        assert_eq!(
            (
                unit.unit_control_cost,
                unit.tunnel_space_cost,
                unit.beacon_capacity_cost
            ),
            (Some(2), None, None)
        );
    }
}
