//! The Specter — `factions/syndicate/units/specter.md` (design change 0367's placeholders; slice M6-SK).

use crate::defs::units::{
    AttackAttrs, AttackType, BlastDomain, MovementAttrs, TargetDomain, TargetType, TurretAttrs, UnitBase, UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;
use crate::units::SU_PER_GU;

/// `factions/syndicate/units/specter.md`: the "Syndicate **bomber** carrying a **single payload**. It flies as a
/// **Glider** (must keep moving), but **lifts off and lands vertically** through a Tier 3 Tunnel's air doors". Its
/// numbers only (slice M6-SK, plan decision T84, by T69's rule): it flies by its GliderMovement, circling when idle, and
/// drops its bomb as any DoublyDisjointed attack lands, and nothing makes one yet: the Aerospace Assembly trains it
/// behind a Vault and a Chemical Plant with slice M6-SA, which brings its single payload, its fixed fall (plan decision
/// T100's `UnitDef::bomb_fall`), its re-arming in a Tier 3 Tunnel and its vertical lift-off and landing. Its numbers are
/// the design's "Placeholder stats".
pub(crate) const SPECTER: ObjectDef = ObjectDef {
    kind: ObjectKind::Specter,
    name: "Specter",
    faction: Some(Faction::Syndicate),                     // Faction - TheSyndicate
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 56.0, h: 56.0 }, // Silhouette 56x56 space units
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    // Not listed: true, as for every unit whose commands are used all together, not one unit at a time
    // (`systems/controls/selection.md > SelectionGroup > No smart-casting`: "Groupable = true for units whose abilities
    // are normally used **all together**").
    groupable: true,
    sight_range: 6.0,    // SightRange 6 grid units
    max_hp: Some(140.0), // MaxHP 140
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 1.0,  // FullArmor 1
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        // UnitBase - Glider (with vertical lift-off and landing at the Tunnel air doors)
        base: UnitBase::Glider,
        // "GliderMovement: IdleSpeed 6, MaxSpeed 11 su/frame, Acceleration 0.5, Deceleration 0.5,
        // MaxCentripetalAcceleration 0.5; idle circle radius 1 grid unit" (every Glider's, `Air::idle_circle_gu`).
        movement: MovementAttrs::Glider {
            idle_speed: 6.0,
            max_speed: 11.0,
            acceleration: Some(0.5),
            deceleration: Some(0.5),
            max_centripetal_acceleration: 0.5,
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        // "Turret: TurnAngle 20° total" ("On a **turret with very low range**: it must be nearly over its target"). Its
        // TurnRate isn't given: **Placeholder (not in the design: the Specter's TurnRate)**, 8°/frame, as the Phantom's
        // and the Sparrow's.
        turret: Some(TurretAttrs {
            turn_angle: 20.0 * PI / 180.0,
            turn_rate: 8.0 * PI / 180.0,
        }),
        attack: Some(AttackAttrs {
            // "**Can miss**: the bomb falls to a locked location, so targets can dodge (DoublyDisjointed, assumed)"; "Like
            // every attack that can miss, it can **Attack Ground**" (`crate::combat::can_target_ground`).
            attack_type: AttackType::DoublyDisjointed,
            target_domain: TargetDomain::Ground, // TargetDomain: **Ground only** (user, 2026-11-25)
            // "**AoE damage**"; "AoERadius 80 su" (1.25 grid units), its blast its TargetDomain's (`systems/combat.md >
            // AoEDamage`).
            target_type: TargetType::AoE {
                radius: 80.0 / SU_PER_GU,
                blast: BlastDomain::Attack,
            },
            damage: 400.0, // "Damage **400** total over the circle"
            range: 1.0,    // Range 1 grid unit
            min_range: 0.0,
            // "a fixed **24-frame fall** (the dodge window)", which slice M6-SA builds (`UnitDef::bomb_fall`, plan
            // decision T100); until then its bomb flies as every projectile does, at a speed: **Placeholder (not in the
            // design: a speed for the bomb)**, 64 su in 24 frames, so that a drop at its full Range of 1 grid unit
            // falls for the design's 24 frames.
            projectile_speed: Some(SU_PER_GU / 24.0),
            aim_ticks: 4,      // Aim 4
            firing_ticks: 2,   // Firing 2
            cooldown_ticks: 4, // Cooldown 4
            // Not given: "A **single payload**: one bomb per sortie", re-armed inside the network in "160 frames"
            // (`> Placeholder stats`), which slice M6-SA builds. Until then: **Placeholder (not in the design: a Reload
            // for a single payload)**, a bomb every 160 frames, its re-arming time, Aim, Firing and Cooldown included.
            reload_ticks: 150,
            surge: None,
        }),
        tunnel_space_cost: Some(3), // TunnelSpaceCost 3
        // "SeparationRadius **64** su (raised from 40, Q0148; must exceed the 56 su Silhouette)."
        separation_radius: Some(64.0),
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::combat::{can_target_ground, flight_ticks};
    use crate::defs::Domain;
    use crate::defs::syndicate::tests::timings;

    /// A 56×56 Glider, in the air, MaxHP 140, PointArmor 1, FullArmor 1, SightRange 6, Groupable (not listed: a unit
    /// used all together), TunnelSpaceCost 3; GliderMovement from an idle 6 su a frame to 11; a 20° turret; a bomb,
    /// DoublyDisjointed at the ground, 400 damage over 80 su within a grid unit, which falls 24 frames at that Range and
    /// can Attack Ground; Aim 4, Firing 2, Cooldown 4, and a bomb every 160 frames, its re-arming time, until slice
    /// M6-SA gives it a single payload; a SeparationRadius of 64 su, more than its Silhouette.
    #[test]
    fn the_specter_follows_the_design() {
        let def = ObjectKind::Specter.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Specter", Category::Unit, Some(Faction::Syndicate))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(56.0, 56.0)));
        let one = Armor { point: 1.0, full: 1.0 };
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(140.0), None, one, 6.0)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, true));
        let unit = def.unit.as_ref().unwrap();
        assert_eq!((unit.base, unit.base_def().domain), (UnitBase::Glider, Domain::Air));
        let glider = MovementAttrs::Glider {
            idle_speed: 6.0,
            max_speed: 11.0,
            acceleration: Some(0.5),
            deceleration: Some(0.5),
            max_centripetal_acceleration: 0.5,
        };
        assert_eq!(unit.movement, glider);
        let turret = unit.turret.expect("a turret");
        assert!((turret.turn_angle.to_degrees() - 20.0).abs() < 1e-4, "a 20° cone");
        let attack = unit.attack.expect("its bomb");
        assert_eq!(
            (attack.attack_type, attack.target_domain),
            (AttackType::DoublyDisjointed, TargetDomain::Ground)
        );
        let blast = TargetType::AoE {
            radius: 1.25,
            blast: BlastDomain::Attack,
        };
        assert_eq!((attack.target_type, attack.damage), (blast, 400.0));
        assert_eq!(1.25 * SU_PER_GU, 80.0, "80 space units");
        assert_eq!((attack.range, attack.min_range), (1.0, 0.0));
        let full_range = flight_ticks(Vec2::ZERO, Vec2::new(SU_PER_GU, 0.0), attack.projectile_speed);
        assert_eq!(full_range, 24, "a 24-frame fall at its Range");
        assert!(can_target_ground(attack.attack_type));
        assert_eq!(timings(&attack), ((4, 2, 4, 150), 160), "a bomb every 160 frames");
        assert_eq!(unit.separation_radius, Some(64.0));
        assert!(64.0 > def.silhouette().unwrap().max_element());
        assert_eq!(
            (
                unit.tunnel_space_cost,
                unit.unit_control_cost,
                unit.beacon_capacity_cost
            ),
            (Some(3), None, None)
        );
    }
}
