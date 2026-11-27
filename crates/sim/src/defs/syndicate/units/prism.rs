//! The Prism — `factions/syndicate/units/prism.md` (design change 0367's placeholders; slice M6-SK).

use crate::defs::units::{MovementAttrs, UnitBase, UnitDef};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/syndicate/units/prism.md`: "The Syndicate's **satellite**: an unmanned orbital unit whose function, **for
/// now, is only laser relay**", "**Very slow** by default". Its numbers only (slice M6-SK, plan decision T84, by T69's
/// rule): it drifts by its DragMovement as any HoverCraft flies, and has no attack of its own (an armed Prism attacks
/// with its relaying Batteries' surges, slice M6-SR). Nothing makes one yet: the Aerospace Assembly trains it behind an
/// Orbital Command with slice M6-SP, which brings its launch, its thrust charges and its one-way exit (`> One way
/// only`). Its numbers are the design's "Placeholder stats".
pub(crate) const PRISM: ObjectDef = ObjectDef {
    kind: ObjectKind::Prism,
    name: "Prism",
    faction: Some(Faction::Syndicate),                     // Faction - TheSyndicate
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 40.0, h: 40.0 }, // Silhouette 40x40 space units
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    // Not listed: true, as for every unit whose commands are used all together, not one unit at a time
    // (`systems/controls/selection.md > SelectionGroup > No smart-casting`: "Groupable = true for units whose abilities
    // are normally used **all together**").
    groupable: true,
    sight_range: 6.0,    // SightRange 6 grid units
    max_hp: Some(100.0), // MaxHP 100
    armor: Armor {
        point: 0.0, // PointArmor 0
        full: 0.0,  // FullArmor 0
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        // UnitBase - HoverCraft (Air domain; `factions/syndicate/structures/aerospace_assembly.md`)
        base: UnitBase::HoverCraft,
        // "Very slow drift: top speed **1 su/frame**. DragMovement (designer placeholder 2026-11-26, Q0154):
        // ForwardAcceleration 0.05; OmniDirectionalAcceleration 0.05; DragRatio 0.1; TurnRate 4°/frame. The thrust
        // charges below are separate bursts, not part of this."
        movement: MovementAttrs::Drag {
            forward_acceleration: 0.05,
            omni_directional_acceleration: 0.05,
            drag_ratio: 0.1,
            turn_rate: 4.0 * PI / 180.0,
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        turret: None,                       // Not listed
        attack: None,                       // its "function, **for now, is only laser relay**"
        tunnel_space_cost: Some(2),         // TunnelSpaceCost 2
        // `systems/units.md > Air Collision`: "SeparationRadius — per unit type [...] Must be larger than the unit's
        // Silhouette", and the Prism's isn't given: **Placeholder (Q0181)**, its Silhouette's side and 8 su, as each
        // radius the design gives is (the Sparrow's and the Dragonfly's 48 for 40, the Specter's 64 for 56).
        separation_radius: Some(48.0),
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::defs::Domain;

    /// A 40×40 HoverCraft, in the air, MaxHP 100, no armour, SightRange 6, Groupable (not listed: a unit used all
    /// together), TunnelSpaceCost 2; DragMovement to a top speed of 1 su a frame, turning 4° a frame (Q0154); no turret
    /// and no attack of its own; its SeparationRadius a placeholder of 48 su, more than its Silhouette (Q0181).
    #[test]
    fn the_prism_follows_the_design() {
        let def = ObjectKind::Prism.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Prism", Category::Unit, Some(Faction::Syndicate))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(40.0, 40.0)));
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(100.0), None, Armor::NONE, 6.0)
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
            (0.05, 0.05, 0.1)
        );
        assert!((turn_rate.to_degrees() - 4.0).abs() < 1e-4, "4° a frame");
        assert!((unit.movement.max_speed() - 1.0).abs() < 1e-5, "top speed 1 su/frame");
        assert_eq!((unit.turret, unit.attack), (None, None));
        assert_eq!(unit.separation_radius, Some(48.0), "Q0181");
        assert!(48.0 > def.silhouette().unwrap().max_element());
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
