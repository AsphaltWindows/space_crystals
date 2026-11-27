//! The Prospector — `factions/gdo/units/prospector.md` (design change 0366, slice M6-GK): "GDO expansion vehicle
//! produced at the Factory once the player owns Advanced Mechanics. A tracked vehicle that deploys into an
//! **Outpost**". Its numbers are the design's "Placeholder stats" (plan decision T84). Its deploy (`> Deploy into
//! Outpost (ability)`: "**Deploy 64 frames (4 s)**; it follows the vehicle deploy model") and its Factory entry
//! ([`RECIPE`], held) are slice M6-GP's; no player produces one before.

use crate::defs::structures::Recipe;
use crate::defs::units::{MovementAttrs, UnitBase, UnitDef};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/units/prospector.md`: its placeholder stats (design change 0366).
pub(crate) const PROSPECTOR: ObjectDef = ObjectDef {
    kind: ObjectKind::Prospector,
    name: "Prospector",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Unit,    // Entity Type - Unit
    // "Silhouette 64x48 space units": its length along its facing (`systems/units.md > Silhouette Rotation`), turned
    // with the facing (`ObjectDef::silhouette_turns`).
    footprint: Footprint::Silhouette { w: 64.0, h: 48.0 },
    selectable: true, // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 6.0,    // SightRange 6 grid units
    max_hp: Some(300.0), // MaxHP 300
    armor: Armor {
        point: 2.0, // PointArmor 2
        full: 2.0,  // FullArmor 2
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::TrackedVehicle, // UnitBase - TrackedVehicle
        // "SpeedTurnRadiusMovement: MaxSpeed 4 su/frame; Acceleration 0.4; Deceleration 1; TrackWidth 36 su;
        // ReverseMaxSpeed 2 (Q0153; spins about 12.7° a frame)."
        movement: MovementAttrs::SpeedTurnRadius {
            track_width: 36.0,
            acceleration: Some(0.4),
            deceleration: Some(1.0),
            max_speed: 4.0,
            reverse_max_speed: 2.0,
        },
        attack: None,               // "**No weapon** (as assumed)."
        unit_control_cost: Some(2), // UnitControlCost 2
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/factory.md > Produces`: "Prospector: 300 Space Crystals, 480 frames (30 seconds)
/// (placeholder). Requires: player owns at least one Advanced Mechanics", its Unit Prerequisite, and `> Placeholder
/// stats`: "Cost: Factory, 300 Space Crystals **+ 8 Supplies** (user, 2026-11-26; Q0169), 480 frames" (design change
/// 0435). Held until slice M6-GP (`crate::defs::PLANNED_RECIPES`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::Prospector,
    cost: 300,
    frames: 480,
    requires: &[ObjectKind::AdvancedMechanics],
    supplies: 8,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stats: a TrackedVehicle, 64×48, MaxHP 300, armour 2/2, SightRange 6, Unit Control 2; up to 4 su
    /// a frame, spinning about 12.7° a frame at rest; no weapon, no turret, no deploy yet. Its Factory entry, 300 Space
    /// Crystals and 8 Supplies in 480 frames behind Advanced Mechanics, is held.
    #[test]
    fn the_prospector_follows_the_design() {
        let unit = super::super::assert_stat_block(
            ObjectKind::Prospector,
            "Prospector",
            (64.0, 48.0),
            UnitBase::TrackedVehicle,
            300.0,
            (2.0, 2.0),
            6.0,
            2,
        );
        assert_eq!(unit.movement.max_speed(), 4.0);
        assert!((unit.movement.spin_rate().unwrap().to_degrees() - 12.7).abs() < 0.05);
        assert_eq!((unit.attack, unit.turret, unit.deploy), (None, None, None));
        let recipe = Recipe {
            kind: ObjectKind::Prospector,
            cost: 300,
            frames: 480,
            requires: &[ObjectKind::AdvancedMechanics],
            supplies: 8,
        };
        assert_eq!(RECIPE, recipe);
    }
}
