//! The Egress Node — `factions/colonists/economy.md > Egress Node`.

use crate::defs::structures::{DropOff, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/colonists/economy.md > Egress Node`: where Drones drop off what they gather (`> Drop Off`, slice M3-H).
pub(crate) static EGRESS_NODE: ObjectDef = ObjectDef {
    kind: ObjectKind::EgressNode,
    name: "Egress Node",
    faction: Some(Faction::Colonists),          // Faction - Colonists
    category: Category::Structure,              // Entity Type - Structure Type
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size - 2x2 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: false,                           // Groupable - false
    sight_range: 3.0,                           // SightRange - 3 grid units
    max_hp: Some(200.0),                        // MaxHP - 200
    max_shields: Some(10.0),                    // MaxShields - 10
    armor: Armor {
        point: 0.0, // PointArmor - 0
        full: 0.0,  // FullArmor - 0
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 0,    // none (see `defs/colonists.rs`)
        power: 0,                     // none (plan decision T21)
        seeds_build_area: false,
        production: None,
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: Some(DropOff::Anywhere), // `factions/colonists/economy.md > Egress Node > Drop Off`
        underground: false,
        tunnel: None,
        site: None,
        expansion: None,
    }),
    default_settings: None,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_egress_node_follows_the_design() {
        let structure = crate::defs::colonists::tests::assert_structure(
            ObjectKind::EgressNode,
            (2, 2),
            SymmetryType::AAAA,
            200.0,
            10.0,
            (0.0, 0.0),
            3.0,
            false,
        );
        assert_eq!((structure.production, structure.warp_in), (None, &[][..]));
    }
}
