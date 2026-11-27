//! The Hand of Heaven — `factions/cults/units/hand_of_heaven.md` (design changes 0361 and 0363).

use crate::defs::units::{
    AttackAttrs, AttackType, MovementAttrs, TargetDomain, TargetType, TurretAttrs, UnitBase, UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;
use crate::units::SU_PER_GU;

/// `factions/cults/units/hand_of_heaven.md`: "The Cults' **long-range, mobile, semi-ballistic missile platform**, built
/// at the Chop Shop", "a **shoot-and-scoot** unit". Its numbers only (slice M5-G2, plan decision T69), its movement and
/// its turret as data: nothing reads its speed-dependent turning nor its turret's arc and rate yet (slice V2 turns
/// vehicles), and its production at the Chop Shop comes with slice M5-N10 (the Chop Shop trains the Seeker since slice
/// M5-N6), so no building makes it yet. Its missile is a DoublyDisjointed area attack, as the Grenadier's grenade is
/// (`crate::combat::projectiles`, `crate::combat::area`): "**no homing**: the missile flies to where the target was at
/// the end of Aiming, so targets can dodge; it can also be fired at the ground". Its numbers are the design's
/// "Placeholder stats".
pub(crate) static HAND_OF_HEAVEN: ObjectDef = ObjectDef {
    kind: ObjectKind::HandOfHeaven,
    name: "Hand of Heaven",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Unit,      // Entity Type - Unit
    // "Silhouette 64x40 space units (length x width)": its length along its facing (`systems/units.md > Silhouette
    // Rotation`), turned with the facing (slice M5-V1, `ObjectDef::silhouette_turns`).
    footprint: Footprint::Silhouette { w: 64.0, h: 40.0 },
    selectable: true, // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 7.0,    // SightRange 7 grid units (it needs spotters for its full range)
    max_hp: Some(200.0), // MaxHP 200
    max_shields: None,
    armor: Armor {
        point: 2.0, // PointArmor 2
        full: 2.0,  // FullArmor 2
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::TrackedVehicle, // UnitBase - TrackedVehicle (turns in place)
        // "SpeedTurnRadiusMovement: MaxSpeed 5 su/frame; Acceleration 0.5 su/frame²; Deceleration 1 su/frame²;
        // SpeedToTurnRadiusRatio 8 (turns in place when stopped)."
        movement: MovementAttrs::SpeedTurnRadius {
            speed_to_turn_radius_ratio: 8.0,
            acceleration: Some(0.5),
            deceleration: Some(1.0),
            max_speed: 5.0,
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        // "Turret: TurnAngle **20°** total; TurnRate 4°/frame": "the launcher barely turns, so to aim at most targets the
        // vehicle must **stop and turn its hull** to face them".
        turret: Some(TurretAttrs {
            turn_angle: 20.0 * PI / 180.0,
            turn_rate: 4.0 * PI / 180.0,
        }),
        attack: Some(AttackAttrs {
            // AttackType: **DoublyDisjointed** (user, 2026-11-27: **no homing**). CanTargetGround: true (derived,
            // `crate::combat::can_target_ground`).
            attack_type: AttackType::DoublyDisjointed,
            target_domain: TargetDomain::Ground, // TargetDomain: **Ground** only (user, 2026-11-27)
            // TargetType: **area damage** (AoE); "AoERadius **96 su (1.5 GU)**"
            target_type: TargetType::AoE {
                radius: 96.0 / SU_PER_GU,
            },
            // "Damage **600** total over the circle (AoEDamage; a 3x3 building wholly inside takes it all, a lone 24x24
            // infantry about 12, so it is a siege weapon, poor against spread or moving units)"
            damage: 600.0,
            range: 14.0,                 // Range **14 grid units**
            min_range: 4.0,              // MinRange **4 grid units**
            projectile_speed: Some(8.0), // ProjectileSpeed **8 su/frame** (a full-range shot flies about 7 s)
            aim_ticks: 48,               // AimDuration **48** frames (3 s)
            firing_ticks: 2,             // FiringDuration 2
            cooldown_ticks: 4,           // CooldownDuration **4**
            reload_ticks: 96,            // ReloadDuration **96** (6 s): one missile per 150 frames, about 9.4 s
        }),
        // `> Produced at - Chop Shop (consumes **3 Recruits**; Unit Control cost **3**)` (plan decision T55).
        unit_control_cost: Some(3),
        tunnel_space_cost: None,
        beacon_capacity_cost: None,
        riot_shield: None,
        conversion: None,
        deploy: None, // "Unlike the GDO's Artillery Rig it **never deploys**"
        worker: None,
        augmentable: None,
        repairs: None,
        spells: &[],
        separation_radius: None,
    }),
    structure: None,
    default_settings: None,
    energy: None,
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::combat::{can_target_ground, domain_allows};
    use crate::defs::Domain;
    use crate::defs::cults::tests::timings;

    /// A 64×40 TrackedVehicle, MaxHP 200, PointArmor 2, FullArmor 2, SightRange 7, Groupable; SpeedTurnRadiusMovement
    /// up to 5 su a frame at a ratio of 8; a launcher that turns 20° in all, 4° a frame; a DoublyDisjointed missile, so
    /// it can target the ground, ground only, 600 damage over a circle of 1.5 grid units, from 4 to 14 grid units, at 8
    /// su a frame, one every 150 frames (Aim 48, Fire 2, Cooldown 4, Reload 96); it never deploys; Unit Control 3, the
    /// Recruits it is made of.
    #[test]
    fn the_hand_of_heaven_follows_the_design() {
        let def = ObjectKind::HandOfHeaven.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Hand of Heaven", Category::Unit, Some(Faction::Cults))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(64.0, 40.0)), "length x width");
        let two = Armor { point: 2.0, full: 2.0 };
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(200.0), None, two, 7.0)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, true));
        let unit = def.unit.as_ref().unwrap();
        assert_eq!(unit.base, UnitBase::TrackedVehicle);
        let tracked = MovementAttrs::SpeedTurnRadius {
            speed_to_turn_radius_ratio: 8.0,
            acceleration: Some(0.5),
            deceleration: Some(1.0),
            max_speed: 5.0,
        };
        assert_eq!(unit.movement, tracked);
        let turret = unit.turret.expect("its launcher");
        assert!((turret.turn_angle.to_degrees() - 20.0).abs() < 1e-4, "20° in all");
        assert!((turret.turn_rate.to_degrees() - 4.0).abs() < 1e-4, "4° a frame");
        let attack = unit.attack.expect("its missile");
        assert_eq!(attack.attack_type, AttackType::DoublyDisjointed, "no homing");
        assert!(can_target_ground(attack.attack_type), "CanTargetGround");
        assert_eq!(
            (attack.target_domain, attack.target_type, attack.damage),
            (TargetDomain::Ground, TargetType::AoE { radius: 1.5 }, 600.0)
        );
        assert!(domain_allows(attack.target_domain, Domain::Ground));
        assert!(!domain_allows(attack.target_domain, Domain::Air), "ground only");
        assert_eq!(
            (attack.range, attack.min_range, attack.projectile_speed),
            (14.0, 4.0, Some(8.0))
        );
        assert_eq!(timings(&attack), ((48, 2, 4, 96), 150), "one missile per 150 frames");
        assert_eq!((unit.deploy, unit.worker), (None, None), "it never deploys");
        assert_eq!(
            (
                unit.unit_control_cost,
                unit.tunnel_space_cost,
                unit.beacon_capacity_cost
            ),
            (Some(3), None, None)
        );
    }
}
