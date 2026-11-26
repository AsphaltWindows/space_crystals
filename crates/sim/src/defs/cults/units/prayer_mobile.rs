//! The Prayer-mobile — `factions/cults/units/prayer_mobile.md` (design changes 0364 and 0365).

use crate::defs::units::{MovementAttrs, UnitBase, UnitDef};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/cults/units/prayer_mobile.md`: "A Cults **energy-based caster vehicle**: a wheeled vehicle carrying a
/// **megaphone**, with **no weapon** (a pure caster)". Its numbers only (slice M5-G2, plan decision T69), its movement
/// as data: nothing reads its turning radius yet (slice V2 turns vehicles); its energy (`> Energy - has an energy pool
/// for its abilities (values TBD)`), its abilities (`> Abilities`: Inspire and Suppress) and its production at the Chop
/// Shop, where its TechPrerequisite is its recipe's (`> TechPrerequisite - player owns at least one **Temple**
/// (core/tech_and_research.md > Unit Prerequisites)`, `Recipe::requires`), come with their slice, so no building makes
/// it yet. Its numbers are the design's "Placeholder stats".
pub(crate) static PRAYER_MOBILE: ObjectDef = ObjectDef {
    kind: ObjectKind::PrayerMobile,
    name: "Prayer-mobile",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Unit,      // Entity Type - Unit
    // "Silhouette 56x40 space units", the Seeker's, its length along its facing (`systems/units.md > Silhouette
    // Rotation`), axis-aligned until slice V1 rotates it (`docs/movement.md`, gap G6).
    footprint: Footprint::Silhouette { w: 56.0, h: 40.0 },
    selectable: true, // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    // **Groupable false** (one cast spends one vehicle's energy, as for the Disciple)
    groupable: false,
    sight_range: 7.0,    // SightRange 7 grid units
    max_hp: Some(150.0), // MaxHP 150
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 1.0,  // FullArmor 1
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::WheeledVehicle, // UnitBase - WheeledVehicle
        // "FixedTurnRadiusMovement as the Seeker, but ForwardMaxSpeed **6 su/frame**"
        // (`factions/cults/units/seeker.md`: "MinimumTurnRadius 64 su (1 grid unit); ForwardMaxSpeed 8 su/frame;
        // ForwardAcceleration 1 su/frame²; ReverseMaxSpeed 3 su/frame; ReverseAcceleration 0.5 su/frame²; Deceleration
        // 1 su/frame²").
        movement: MovementAttrs::FixedTurnRadius {
            minimum_turn_radius: 64.0,
            forward_acceleration: Some(1.0),
            forward_max_speed: 6.0,
            reverse_acceleration: Some(0.5),
            reverse_max_speed: 3.0,
            deceleration: Some(1.0),
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        turret: None,                       // "with **no weapon** (a pure caster)": nothing to turn
        attack: None,                       // AttackAttributes - None (pure caster)
        // `> Produced at - Chop Shop (consumes **2 Recruits**; Unit Control cost **2**)` (plan decision T55).
        unit_control_cost: Some(2),
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
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;

    /// A 56×40 WheeledVehicle, MaxHP 150, PointArmor 1, FullArmor 1, SightRange 7, not Groupable; the Seeker's
    /// FixedTurnRadiusMovement but 6 su a frame forward; no weapon and no turret; Unit Control 2, the Recruits it is
    /// made of.
    #[test]
    fn the_prayer_mobile_follows_the_design() {
        let def = ObjectKind::PrayerMobile.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Prayer-mobile", Category::Unit, Some(Faction::Cults))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(56.0, 40.0)));
        let one = Armor { point: 1.0, full: 1.0 };
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(150.0), None, one, 7.0)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, false));
        let unit = def.unit.as_ref().unwrap();
        assert_eq!(unit.base, UnitBase::WheeledVehicle);
        // The Seeker's FixedTurnRadiusMovement, with a ForwardMaxSpeed of 6.
        let seekers = match ObjectKind::Seeker.def().unit.as_ref().unwrap().movement {
            MovementAttrs::FixedTurnRadius {
                minimum_turn_radius,
                forward_acceleration,
                reverse_acceleration,
                reverse_max_speed,
                deceleration,
                ..
            } => MovementAttrs::FixedTurnRadius {
                minimum_turn_radius,
                forward_acceleration,
                forward_max_speed: 6.0,
                reverse_acceleration,
                reverse_max_speed,
                deceleration,
            },
            other => panic!("the Seeker's is {other:?}"),
        };
        assert_eq!(unit.movement, seekers, "as the Seeker, but ForwardMaxSpeed 6");
        assert_eq!((unit.turret, unit.attack, unit.worker), (None, None, None));
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
