//! The Altar — `factions/cults/augmentation.md > Altar`.

use crate::defs::structures::{BuilderGoes, SiteDef, SitePrice, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/cults/augmentation.md > Altar`: "Tier 1 of the Mystical/Human Enhancement branch. The place where Cults
/// infantry consume Space Crystals and augment their flesh with them", which "works like the Armory: it holds an
/// internal pool of units, processes a serial queue of orders, and sends finished units out to its RallyPoint". Its
/// numbers and its site only (slice M5-G2, plan decision T69): its pool (`> Altar > EntranceSide / ExitSide - B / C
/// (placeholder, like the Armory)`, `> Altar > InternalUnitCapacity - 10 (placeholder)`, "Any Cults infantry"), its
/// orders and its panel come with its slice (plan decisions T70 and T71), which adds it to the Recruits' builds, so it
/// holds no one and Recruits don't build it yet.
pub(crate) static ALTAR: ObjectDef = ObjectDef {
    kind: ObjectKind::Altar,
    name: "Altar",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Structure,
    // Size - 2x3 (placeholder): 2 tall, 3 wide in the default orientation, as the Armory's.
    footprint: Footprint::Tiles { w: 3, h: 2 },
    selectable: true,   // `core/entities.md > Object Type > Selectable - true`
    destructible: true, // Destructible - true
    // Groupable - false (user, 2026-11-26; Q0105: Cults production buildings aren't groupable; design change 0348
    // point 5)
    groupable: false,
    sight_range: 4.0,    // SightRange - 4 grid units (placeholder)
    max_hp: Some(300.0), // MaxHP - 300 (placeholder)
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor - 1 (placeholder)
        full: 6.0,  // FullArmor - 6 (placeholder)
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        // SymmetryType - ABAC (placeholder): A on top and bottom, B on the right, C on the left, its short sides.
        symmetry: SymmetryType::ABAC,
        // The Cults have neither Power nor a build area (`factions/cults/economy.md > Cults Building Mechanics`).
        build_radius_extension: 0,
        power: 0,
        seeds_build_area: false,
        // Its augmentation orders (`> Altar > Augmentation Orders`): with its slice.
        production: None,
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        underground: false,
        tunnel: None,
        site: Some(ALTAR_SITE),
        expansion: None,
        requires: &[ObjectKind::Storage], // TechPrerequisite - player owns at least one Storage
        attack: None,
        popup: None,
        // Its pool of Cults infantry: with its slice.
        holds: None,
        occupants_fire: None,
    }),
    default_settings: None,
};

/// The Altar's construction by Recruits (`StructureDef::site`, slice M5-E's; "Space Crystals cost and construction time
/// are listed under the Recruit's Constructs list, not here").
pub(crate) const ALTAR_SITE: SiteDef = SiteDef {
    // `factions/cults/units/recruit.md > Constructs`: "Altar: 150 Space Crystals, 480 frames (30 seconds)
    // BaseConstructionTime (placeholder). Requires: Storage.", paid once as the order is given
    // (`factions/cults/economy.md > Build Command`).
    price: SitePrice::Crystals(150),
    frames: 480,
    start_hp: 0.1, // `> Altar > ConstructionHP Rule - applies`: "start at 10% of their maximum HP"
    builder_goes: BuilderGoes::Outside,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::cults::tests::assert_structure;

    /// 2 tall and 3 wide, ABAC, MaxHP 300, PointArmor 1, FullArmor 6 (the building armour rule's n = 1), SightRange 4,
    /// not Groupable (design change 0348), behind a Storage; it holds no one and orders nothing yet. Recruits will
    /// build it from outside for 150 Space Crystals in 480 frames with one of them, from 10% HP.
    #[test]
    fn the_altar_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::Altar,
            (3, 2),
            SymmetryType::ABAC,
            300.0,
            (1.0, 6.0),
            4.0,
            false,
        );
        assert_eq!(ObjectKind::Altar.def().name, "Altar");
        assert_eq!(structure.requires, [ObjectKind::Storage], "TechPrerequisite: a Storage");
        assert_eq!(
            (structure.production, structure.drop_off, structure.holds),
            (None, None, None)
        );
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp, site.builder_goes),
            (SitePrice::Crystals(150), 480, 0.1, BuilderGoes::Outside)
        );
    }
}
