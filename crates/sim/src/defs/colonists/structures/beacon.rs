//! The Beacon — `factions/colonists/economy.md > Beacon`.

use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/colonists/economy.md > Beacon`: a deployed Drone (`> Beacon Deployment`, slice M3-F), onto which every
/// other Colonists building is warped in (`> Warp In`, slice M3-E). "Full ObjectInterfaceState still TBD".
pub(crate) static BEACON: ObjectDef = ObjectDef {
    kind: ObjectKind::Beacon,
    name: "Beacon",
    faction: Some(Faction::Colonists),          // Faction - Colonists
    category: Category::Structure,              // Entity Type - Structure Type
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size - 2x2 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: false, // Groupable - false (design change 0122: "used up by its Warp In, like a worker")
    sight_range: 4.0, // SightRange - 4 grid units
    max_hp: Some(120.0), // MaxHP - 120
    max_shields: Some(10.0), // MaxShields - 10
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
        // `> Warp In > Constructs List`: the three buildings with a cost and a WarpInTime. The others (Alloy
        // Furnace, …, Structural Analyzer) are TBD, and deferred past M3 (question Q0043 point 1).
        warp_in: &[
            Recipe {
                kind: ObjectKind::EgressNode,
                cost: 75,    // Egress Node | 75 Space Crystals |
                frames: 320, // 320 frames (20 seconds)
                requires: &[],
            },
            Recipe {
                kind: ObjectKind::RoboPortal,
                cost: 250, // Robo Portal | 250 Space Crystals (was 400; user, 2026-11-24) | (design change 0167)
                frames: 640, // 640 frames (40 seconds)
                requires: &[],
            },
            Recipe {
                kind: ObjectKind::BioPortal,
                cost: 150,   // Bio Portal | 150 Space Crystals |
                frames: 480, // 480 frames (30 seconds)
                requires: &[],
            },
        ],
        drop_off: None,
        underground: false,
        tunnel: None,
        site: None,
        expansion: None,
        // `factions/colonists/structures/README.md > Tech Model`: "Colonists buildings have no TechPrerequisites"
        requires: &[],
        attack: None,
        popup: None,
        holds: None,
        occupants_fire: None,
        research: None,
    }),
    default_settings: None,
    energy: None,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The stat block, and its Warp In list (`> Warp In > Constructs List`): the three buildings with a cost and a
    /// WarpInTime. Ungroupable (design change 0122: "Beacon Groupable true → false").
    #[test]
    fn the_beacon_follows_the_design() {
        let structure = crate::defs::colonists::tests::assert_structure(
            ObjectKind::Beacon,
            (2, 2),
            SymmetryType::AAAA,
            120.0,
            10.0,
            (0.0, 0.0),
            4.0,
            false,
        );
        assert_eq!(structure.production, None);
        let recipe = |kind, cost, frames| Recipe {
            kind,
            cost,
            frames,
            requires: &[],
        };
        assert_eq!(
            structure.warp_in,
            [
                recipe(ObjectKind::EgressNode, 75, 320),
                recipe(ObjectKind::RoboPortal, 250, 640), // design change 0167 (was 400)
                recipe(ObjectKind::BioPortal, 150, 480),
            ]
        );
        // The Beacon is the one type with a Warp In list.
        for &kind in ObjectKind::ALL {
            let warps_in = kind.def().structure.as_ref().is_some_and(|s| !s.warp_in.is_empty());
            assert_eq!(warps_in, kind == ObjectKind::Beacon, "{kind:?}");
        }
    }
}
