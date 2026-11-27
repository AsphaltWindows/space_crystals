//! The Phantom — `factions/syndicate/units/phantom.md` (design change 0367's placeholders; slice M6-SK).

use crate::defs::units::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MovementAttrs, TargetDomain, TargetType, TurretAttrs, UnitBase,
    UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/syndicate/units/phantom.md`: the "Syndicate **hovering aircraft**", whose "**red laser** [...] hits
/// **both air and ground**. **Relatively manoeuvrable.**": "**Comparable to the Guard** [...] but **somewhat squishier
/// for its cost**, and **faster**". Its numbers only (slice M6-SK, plan decision T84, by T69's rule): it flies by its
/// DragMovement and fires from its turret as any HoverCraft does, and nothing makes one yet: the Aerospace Assembly
/// trains it behind an Optics Lab with slice M6-SD, which brings the air doors it rises out of and descends into. Its
/// numbers are the design's "Placeholder stats".
pub(crate) const PHANTOM: ObjectDef = ObjectDef {
    kind: ObjectKind::Phantom,
    name: "Phantom",
    faction: Some(Faction::Syndicate),                     // Faction - TheSyndicate
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 44.0, h: 44.0 }, // Silhouette 44x44 space units
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,    // Groupable - true (confirmed, Q0071)
    sight_range: 8.0,   // SightRange 8 grid units
    max_hp: Some(70.0), // MaxHP 70
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 1.0,  // FullArmor 1
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        // UnitBase - HoverCraft (enters and exits only through Tier 3 Tunnels' air doors)
        base: UnitBase::HoverCraft,
        // "HoverCraft (DragMovement; designer placeholder 2026-11-26, Q0154; "relatively manoeuvrable": more drag and
        // more sideways thrust than the Hornet, so it starts, stops and strafes quicker): ForwardAcceleration 0.78;
        // OmniDirectionalAcceleration 0.3; DragRatio 0.12 (top speed 9 su/frame); TurnRate 10°/frame."
        movement: MovementAttrs::Drag {
            forward_acceleration: 0.78,
            omni_directional_acceleration: 0.3,
            drag_ratio: 0.12,
            turn_rate: 10.0 * PI / 180.0,
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        // "Turret: TurnAngle **75°** total; TurnRate 8°/frame" ("**moderate, 60–90°** (user, 2026-11-25: the **total**
        // cone, i.e. 30–45° to each side of its facing").
        turret: Some(TurretAttrs {
            turn_angle: 75.0 * PI / 180.0,
            turn_rate: 8.0 * PI / 180.0,
        }),
        attack: Some(AttackAttrs {
            // AttackType: FullyConnected (laser like other Syndicate lasers: instant, cannot miss; confirmed, Q0071);
            // its Range of 5 grid units makes it the Ranged subtype (`systems/combat.md > FullyConnected > Melee` is
            // adjacent contact).
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
            target_domain: TargetDomain::Universal, // TargetDomain: Universal (ground and air)
            target_type: TargetType::SingleTarget,  // TargetType: SingleTarget (confirmed, Q0071)
            // "Laser: Damage 12; Range 5 grid units; Aim 2, Firing 1, Cooldown 3, Reload 10 frames (12 damage per
            // second before armor)."
            damage: 12.0,
            range: 5.0,
            min_range: 0.0,         // Not listed: none
            projectile_speed: None, // FullyConnected: no projectile
            aim_ticks: 2,
            firing_ticks: 1,
            cooldown_ticks: 3,
            reload_ticks: 10,
        }),
        tunnel_space_cost: Some(2), // TunnelSpaceCost 2
        // `systems/units.md > Air Collision`: "SeparationRadius — per unit type [...] Must be larger than the unit's
        // Silhouette", and the Phantom's isn't given: **Placeholder (Q0181)**, its Silhouette's side and 8 su, as each
        // radius the design gives is (the Sparrow's and the Dragonfly's 48 for 40, the Specter's 64 for 56).
        separation_radius: Some(52.0),
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::defs::Domain;
    use crate::defs::syndicate::tests::timings;

    /// A 44×44 HoverCraft, in the air, MaxHP 70, PointArmor 1, FullArmor 1, SightRange 8, Groupable, TunnelSpaceCost
    /// 2; DragMovement to a top speed of 9 su a frame, turning 10° a frame (Q0154); a 75° turret turning 8° a frame; a
    /// red laser that can't miss, at air and ground, 12 damage up to 5 grid units every 16 frames, 12 a second before
    /// armor; its SeparationRadius a placeholder of 52 su, more than its Silhouette (Q0181).
    #[test]
    fn the_phantom_follows_the_design() {
        let def = ObjectKind::Phantom.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Phantom", Category::Unit, Some(Faction::Syndicate))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(44.0, 44.0)));
        let one = Armor { point: 1.0, full: 1.0 };
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(70.0), None, one, 8.0)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, true));
        let unit = def.unit.as_ref().unwrap();
        assert_eq!((unit.base, unit.base_def().domain), (UnitBase::HoverCraft, Domain::Air));
        let MovementAttrs::Drag {
            forward_acceleration,
            omni_directional_acceleration,
            drag_ratio,
            turn_rate,
        } = unit.movement
        else {
            panic!("{:?} is not DragMovement", unit.movement);
        };
        assert_eq!(
            (forward_acceleration, omni_directional_acceleration, drag_ratio),
            (0.78, 0.3, 0.12)
        );
        assert!((turn_rate.to_degrees() - 10.0).abs() < 1e-4, "10° a frame");
        assert!((unit.movement.max_speed() - 9.0).abs() < 1e-4, "top speed 9 su/frame");
        let turret = unit.turret.expect("a turret");
        assert!((turret.turn_angle.to_degrees() - 75.0).abs() < 1e-4, "a 75° cone");
        assert!((turret.turn_rate.to_degrees() - 8.0).abs() < 1e-4, "8° a frame");
        let attack = unit.attack.expect("its laser");
        assert_eq!(
            attack.attack_type,
            AttackType::FullyConnected(FullyConnectedSubtype::Ranged)
        );
        assert_eq!(
            (attack.target_domain, attack.target_type, attack.damage),
            (TargetDomain::Universal, TargetType::SingleTarget, 12.0)
        );
        assert_eq!(
            (attack.range, attack.min_range, attack.projectile_speed),
            (5.0, 0.0, None)
        );
        assert_eq!(timings(&attack), ((2, 1, 3, 10), 16), "12 damage a second");
        assert_eq!(unit.separation_radius, Some(52.0), "Q0181");
        assert!(52.0 > def.silhouette().unwrap().max_element());
        assert_eq!(
            (
                unit.tunnel_space_cost,
                unit.unit_control_cost,
                unit.beacon_capacity_cost
            ),
            (Some(2), None, None)
        );
    }
}
