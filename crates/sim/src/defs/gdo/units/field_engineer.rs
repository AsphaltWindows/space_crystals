//! The Field Engineer — `factions/gdo/units/field_engineer.md` (design change 0328, slice M5-N9): GDO infantry,
//! trained at the Barracks once its owner has a Career Campus (`factions/gdo/structures/barracks.md > Produces`), that
//! repairs its owner's buildings and vehicles for Space Crystals through the repairers' one mechanism
//! (`crate::repair`), and carries no weapon. Its Bunker and First Aid Tent aren't designed yet, so it builds nothing.

use crate::defs::units::{MovementAttrs, RepairDef, RepairTargets, UnitBase, UnitDef};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/gdo/units/field_engineer.md`: its placeholder stats (design change 0328).
pub(crate) static FIELD_ENGINEER: ObjectDef = ObjectDef {
    kind: ObjectKind::FieldEngineer,
    name: "Field Engineer",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Unit,    // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 24.0, h: 24.0 }, // Silhouette 24x24 space units
    selectable: true,            // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    groupable: true,    // Groupable true
    sight_range: 5.0,   // SightRange 5 grid units
    max_hp: Some(50.0), // MaxHP 50
    max_shields: None,
    armor: Armor {
        point: 0.0, // PointArmor 0
        full: 0.0,  // FullArmor 0
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::LightInfantry, // UnitBase LightInfantry
        // "Movement as the Peacekeeper (MaxSpeed 4 space units/frame)": UnitBaseAttributes[LightInfantry] (TurnRate
        // 180°/frame, infinite acceleration, RuggedTerrainDefenseBonus 50%)
        movement: MovementAttrs::TurnRate {
            turn_rate: PI,
            acceleration: None,
            deceleration: None,
            max_speed: 4.0,
        },
        rugged_terrain_defense_bonus: Some(0.5),
        turret: None,
        attack: None,               // "The Field Engineer carries **no weapon**."
        unit_control_cost: Some(1), // UnitControlCost 1
        tunnel_space_cost: None,
        beacon_capacity_cost: None,
        riot_shield: None,
        conversion: None,
        deploy: None,
        worker: None, // It gathers nothing.
        augmentable: None,
        // `factions/gdo/units/field_engineer.md > Abilities`: "**Repair**: repairs buildings and vehicles. Repair costs
        // Space Crystals (not free)"; "**D** Repair (targeted: a damaged own building or vehicle)".
        repairs: Some(RepairDef {
            targets: RepairTargets::OwnBuildingsAndVehicles,
        }),
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

    /// The placeholder stats: LightInfantry, 24×24, MaxHP 50, no armour, SightRange 5, Groupable, Unit Control 1; the
    /// Peacekeeper's movement; no weapon, turret or load; and it repairs its owner's buildings and vehicles, as the
    /// Recruit does.
    #[test]
    fn the_field_engineer_follows_the_design() {
        let def = ObjectKind::FieldEngineer.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Field Engineer", Category::Unit, Some(Faction::Gdo))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(24.0, 24.0)));
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(50.0), None, Armor::NONE, 5.0)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, true));
        let unit = def.unit.as_ref().unwrap();
        let peacekeeper = ObjectKind::Peacekeeper.def().unit.as_ref().unwrap();
        assert_eq!(unit.base, UnitBase::LightInfantry);
        assert_eq!(
            (unit.movement, unit.rugged_terrain_defense_bonus),
            (peacekeeper.movement, peacekeeper.rugged_terrain_defense_bonus),
            "as the Peacekeeper"
        );
        assert_eq!(unit.movement.max_speed(), 4.0);
        assert_eq!((unit.turret, unit.attack, unit.worker), (None, None, None));
        assert_eq!(unit.unit_control_cost, Some(1));
        let recruit = ObjectKind::Recruit.def().unit.as_ref().unwrap();
        assert_eq!(unit.repairs, recruit.repairs, "the Recruit's targets, paid");
    }
}
