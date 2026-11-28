//! The Assault Bike — `factions/syndicate/units/assault_bike.md` (design change 0367's placeholders; slice M6-SK).

use crate::defs::units::{
    AttackAttrs, AttackType, BlastDomain, MovementAttrs, TargetDomain, TargetType, TurretAttrs, UnitBase, UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;
use crate::units::SU_PER_GU;

/// `factions/syndicate/units/assault_bike.md`: the "Syndicate fast attack vehicle. The **fastest wheeled vehicle in the
/// game**, with a **relatively wide turning radius**", "Armed with a **mortar**: like the Grenadier's grenade [...] but
/// with **slightly longer range**", on "a **turret with a narrow turn angle**". Its numbers only (slice M6-SK, plan
/// decision T84, by T69's rule): it drives by its FixedTurnRadiusMovement and fires from its turret as any wheeled unit
/// does, and nothing makes one yet: the Vehicle Assembly trains it with slice M6-SV
/// (`defs::syndicate::structures::vehicle_assembly`). Its numbers are the design's "Placeholder stats".
pub(crate) const ASSAULT_BIKE: ObjectDef = ObjectDef {
    kind: ObjectKind::AssaultBike,
    name: "Assault Bike",
    faction: Some(Faction::Syndicate), // Faction - TheSyndicate
    category: Category::Unit,          // Entity Type - Unit
    // "Silhouette 56x32 space units": its length along its facing, turned with the facing, as every vehicle's
    // (`ObjectDef::silhouette_turns`).
    footprint: Footprint::Silhouette { w: 56.0, h: 32.0 },
    selectable: true, // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Groupable - true (assumed)
    sight_range: 7.0,    // SightRange 7 grid units
    max_hp: Some(100.0), // MaxHP 100
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 1.0,  // FullArmor 1
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::WheeledVehicle, // UnitBase - WheeledVehicle (exits only through Tier 2+ Tunnels)
        // "FixedTurnRadiusMovement: MinimumTurnRadius **128 su** (wide for its speed); ForwardMaxSpeed **12 su/frame**
        // (the fastest wheeled vehicle); ForwardAcceleration 1; ReverseMaxSpeed 3; ReverseAcceleration 0.5;
        // Deceleration 1."
        movement: MovementAttrs::FixedTurnRadius {
            minimum_turn_radius: 128.0,
            forward_acceleration: Some(1.0),
            forward_max_speed: 12.0,
            reverse_acceleration: Some(0.5),
            reverse_max_speed: 3.0,
            deceleration: Some(1.0),
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        // "Turret: TurnAngle **40°** total; TurnRate 8°/frame": "it can aim a little to either side of the bike's
        // facing, but the bike must still roughly point toward its target".
        turret: Some(TurretAttrs {
            turn_angle: 40.0 * PI / 180.0,
            turn_rate: 8.0 * PI / 180.0,
        }),
        attack: Some(AttackAttrs {
            // AttackType: DoublyDisjointed (mortar; slow projectile to the target's position at the end of Aiming; can
            // be dodged). CanTargetGround: true (derived, `crate::combat::can_target_ground`).
            attack_type: AttackType::DoublyDisjointed,
            target_domain: TargetDomain::Ground, // TargetDomain: Ground (assumed)
            // TargetType: AoE; "AoERadius 48 su" (0.75 grid units), its blast its TargetDomain's (`systems/combat.md >
            // AoEDamage`), as the Grenadier's.
            target_type: TargetType::AoE {
                radius: 48.0 / SU_PER_GU,
                blast: BlastDomain::Attack,
            },
            damage: 120.0, // "Damage 120 total over the circle"
            range: 5.0,    // Range **5 grid units** (Grenadier: 4)
            // "**MinRange 1 grid unit** (user, 2026-11-27; Q0184: so it never fires close enough for its own blast to
            // hit it; value placeholder, just beyond the 48 su blast; was 0)": design change 0460.
            min_range: 1.0,
            projectile_speed: Some(8.0), // ProjectileSpeed 8 su/frame
            aim_ticks: 8,                // Aim 8
            firing_ticks: 2,             // Firing 2
            cooldown_ticks: 4,           // Cooldown 4
            reload_ticks: 34,            // Reload 34 frames (one round per 48 frames)
            surge: None,
        }),
        tunnel_space_cost: Some(2), // TunnelSpaceCost 2
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::combat::can_target_ground;
    use crate::defs::syndicate::tests::timings;

    /// A 56×32 WheeledVehicle, MaxHP 100, PointArmor 1, FullArmor 1, SightRange 7, Groupable ("true (assumed)"),
    /// TunnelSpaceCost 2; FixedTurnRadiusMovement at 12 su a frame forward, the fastest wheeled vehicle, on a turning
    /// radius of 128 su; a 40° turret turning 8° a frame; a mortar, DoublyDisjointed at the ground, 120 damage over 48
    /// su up to 5 grid units (the Grenadier's 4), its round at 8 su a frame, one per 48 frames, which can Attack Ground.
    #[test]
    fn the_assault_bike_follows_the_design() {
        let def = ObjectKind::AssaultBike.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Assault Bike", Category::Unit, Some(Faction::Syndicate))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(56.0, 32.0)));
        let one = Armor { point: 1.0, full: 1.0 };
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(100.0), None, one, 7.0)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, true));
        let unit = def.unit.as_ref().unwrap();
        assert_eq!(unit.base, UnitBase::WheeledVehicle);
        let movement = MovementAttrs::FixedTurnRadius {
            minimum_turn_radius: 128.0,
            forward_acceleration: Some(1.0),
            forward_max_speed: 12.0,
            reverse_acceleration: Some(0.5),
            reverse_max_speed: 3.0,
            deceleration: Some(1.0),
        };
        assert_eq!(unit.movement, movement);
        let fastest = (ObjectKind::ALL.iter())
            .filter_map(|kind| kind.def().unit.as_ref())
            .filter(|unit| unit.base == UnitBase::WheeledVehicle)
            .map(|unit| unit.movement.max_speed())
            .fold(0.0, f32::max);
        assert_eq!(fastest, 12.0, "the fastest wheeled vehicle in the game");
        let turret = unit.turret.expect("a turret");
        assert!((turret.turn_angle.to_degrees() - 40.0).abs() < 1e-4, "a 40° cone");
        assert!((turret.turn_rate.to_degrees() - 8.0).abs() < 1e-4, "8° a frame");
        let attack = unit.attack.expect("its mortar");
        assert_eq!(
            (attack.attack_type, attack.target_domain),
            (AttackType::DoublyDisjointed, TargetDomain::Ground)
        );
        let blast = TargetType::AoE {
            radius: 0.75,
            blast: BlastDomain::Attack,
        };
        assert_eq!((attack.target_type, attack.damage), (blast, 120.0));
        assert_eq!(0.75 * SU_PER_GU, 48.0, "48 space units");
        assert_eq!(
            (attack.range, attack.min_range, attack.projectile_speed),
            (5.0, 1.0, Some(8.0))
        );
        assert_eq!(timings(&attack), ((8, 2, 4, 34), 48), "one round per 48 frames");
        assert!(can_target_ground(attack.attack_type));
        assert_eq!(
            (
                unit.tunnel_space_cost,
                unit.unit_control_cost,
                unit.beacon_capacity_cost
            ),
            (Some(2), None, None)
        );
        assert_eq!((unit.worker, unit.deploy, unit.separation_radius), (None, None, None));
    }
}
