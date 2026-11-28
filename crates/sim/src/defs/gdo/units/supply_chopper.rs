//! The Supply Chopper — `factions/gdo/units/supply_chopper.md` (design change 0366, slice M6-GK): "GDO supply
//! transport air unit. An unarmed hovercraft that collects supplies from Supply Delivery Stations and delivers them to
//! Supply Towers." Its numbers are the design's (plan decision T84). The Supply Tower produces it
//! ([`RECIPE`]); it carries Supplies, from the stations to the towers, and lands on both (`crate::supply_run`). Its
//! unit transport (`factions/gdo/units/supply_chopper.md > Cargo`) isn't in the game.

use crate::defs::structures::Recipe;
use crate::defs::units::{LandingDef, MovementAttrs, UnitBase, UnitDef};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind, SupplyChopperDef};
use crate::math::PI;
use crate::player::Faction;

/// `factions/gdo/units/supply_chopper.md`.
pub(crate) const SUPPLY_CHOPPER: ObjectDef = ObjectDef {
    kind: ObjectKind::SupplyChopper,
    name: "Supply Chopper",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Unit,
    footprint: Footprint::Silhouette { w: 60.0, h: 60.0 }, // Silhouette - 60x60 space units (square)
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                                    // Destructible - true
    groupable: true,                                       // Groupable - true
    sight_range: 5.0,                                      // SightRange - 5 grid units
    max_hp: Some(150.0),                                   // MaxHP - 150
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 1.0,  // FullArmor - 1
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::HoverCraft, // UnitBase - HoverCraft
        // `> UnitBaseAttributes[HoverCraft]`: "ForwardAcceleration: 0.9 space units/frame squared;
        // OmniDirectionalAcceleration: 0.1 space units/frame squared; DragRatio: 0.1 per frame; TurnRate: 10
        // degrees/frame".
        movement: MovementAttrs::Drag {
            forward_acceleration: 0.9,
            omni_directional_acceleration: 0.1,
            drag_ratio: 0.1,
            turn_rate: 10.0 * PI / 180.0,
        },
        turret: None, // TurretAttributes - None
        attack: None, // AttackAttributes - None
        // **Placeholder (not in the design: the Supply Chopper's UnitControlCost)**: 2, a vehicle's (the Raptor's, the
        // Twister's, the Hornet's); `factions/gdo/overview.md > Unit Control`: "Each GDO unit has a Unit Control
        // cost".
        unit_control_cost: Some(2),
        // **Placeholder (Q0181)**: SeparationRadius 68 su, its Silhouette's side and 8 su, as the Sparrow's and the
        // Dragonfly's 48 on their 40 (`systems/units.md > Air Collision`: "Must be larger than the unit's Silhouette").
        separation_radius: Some(68.0),
        // `> Unit Transport`: "**Landing and taking off take 16 frames (1 s) each** (as at the Airfield)"
        // (`crate::supply_run`, `crate::landing`).
        landing: Some(LandingDef {
            landing_ticks: 16,
            takeoff_ticks: 16,
        }),
        supply_chopper: Some(SupplyChopperDef {}),
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/economy.md > SupplyTower > Produces`: "Supply Chopper: 100 Space Crystals, 160 frames (10 seconds)".
/// In the Supply Tower's list (`crate::defs::gdo::structures::supply_tower`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::SupplyChopper,
    cost: 100,
    frames: 160,
    requires: &[],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::Domain;

    /// The stats: a HoverCraft in the air, 60×60, MaxHP 150, armour 1/1, SightRange 5, Unit Control 2 (a placeholder);
    /// DragMovement to a top speed of 10 su a frame, turning 10° a frame; unarmed, with no turret; a SeparationRadius
    /// of 68 su, more than its Silhouette; it lands and takes off in 16 frames. Its Supply Tower entry is
    /// 100 Space Crystals and 160 frames.
    #[test]
    fn the_supply_chopper_follows_the_design() {
        let unit = super::super::assert_stat_block(
            ObjectKind::SupplyChopper,
            "Supply Chopper",
            (60.0, 60.0),
            UnitBase::HoverCraft,
            150.0,
            (1.0, 1.0),
            5.0,
            2,
        );
        assert_eq!(unit.base_def().domain, Domain::Air);
        assert!((unit.movement.max_speed() - 10.0).abs() < 1e-4);
        assert!((unit.movement.turn_rate().unwrap().to_degrees() - 10.0).abs() < 1e-4);
        assert_eq!(
            (unit.attack, unit.turret, unit.separation_radius),
            (None, None, Some(68.0))
        );
        let recipe = Recipe {
            kind: ObjectKind::SupplyChopper,
            cost: 100,
            frames: 160,
            requires: &[],
            supplies: 0,
        };
        assert_eq!(RECIPE, recipe);
        assert_eq!(unit.supply_chopper, Some(SupplyChopperDef {}));
        let lands = LandingDef {
            landing_ticks: 16,
            takeoff_ticks: 16,
        };
        assert_eq!(unit.landing, Some(lands));
    }
}
