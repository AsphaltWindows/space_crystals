//! The Altar — `factions/cults/augmentation.md > Altar`.

use crate::augmentation::AugmentOrder;
use crate::defs::structures::{
    AugmentRecipe, BuilderGoes, ExitRule, HoldsDef, ProductionDef, SiteDef, SitePrice, StructureDef, SymmetryType,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/cults/augmentation.md > Altar`: "Tier 1 of the Mystical/Human Enhancement branch. The place where Cults
/// infantry consume Space Crystals and augment their flesh with them", which "works like the Armory: it holds an
/// internal pool of units, processes a serial queue of orders, and sends finished units out to its RallyPoint": the
/// Armory's pool (`holds`, `crate::holding`) and queue (`production`, plan decisions T70 and T71), the queue's entries
/// its augmentation orders (`ProductionDef::augment`, `QueueEntry::Augment`) and Releases, of the units inside
/// (`sim/producers/pool.rs`; slice M5-N1b). Its numbers and its site are slice M5-G2's (plan decision T69), and
/// Recruits build it from outside ([`ALTAR_SITE`]).
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
        // `> Altar > Augmentation Orders`: "Orders are queueable and serial — processed one at a time, in the order
        // queued"; "the unit **automatically exits** from the exit side and moves to the RallyPoint", as the units it
        // holds do (`holds`). It trains no unit and recruits none: its entries are its orders (and Releases, design
        // change 0337), each of one unit inside.
        production: Some(ProductionDef {
            exits: ExitRule::Side(SideLetter::C),
            recipes: &[],
            recruitment: None,
            // "Cost and time per order type (placeholder, designer 2026-11-26): **Enhanced Constitution** and
            // **Improved Metabolism**: 25 Space Crystals, 160 frames (10 s) each; **Both**: the sum, 50 Space
            // Crystals, 320 frames", in the order of `> Augmentation Orders`' "Order types".
            augment: &[
                AugmentRecipe {
                    order: AugmentOrder::EnhancedConstitution,
                    cost: 25,
                    frames: 160,
                },
                AugmentRecipe {
                    order: AugmentOrder::ImprovedMetabolism,
                    cost: 25,
                    frames: 160,
                },
                AugmentRecipe {
                    order: AugmentOrder::Both,
                    cost: 50,
                    frames: 320,
                },
            ],
        }),
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
        // `> Altar > EntranceSide / ExitSide - B / C (placeholder, like the Armory)`, `> Altar > InternalUnitCapacity -
        // 10 (placeholder)`, and `> Altar > Entry`: "Any Cults infantry (Recruit, Militant, Gunner, Disciple) can enter
        // the Altar to join its StoredUnits pool, up to InternalUnitCapacity", in the order of the type tiles of
        // `> Altar > ObjectInterfaceState[Altar]` ("**Q: Recruit**", "**W: Militant**", "**E: Gunner**", "**R:
        // Disciple**").
        holds: Some(HoldsDef {
            entrance: SideLetter::B,
            exit: ExitRule::Side(SideLetter::C),
            capacity: 10,
            accepts: &[
                ObjectKind::Recruit,
                ObjectKind::Militant,
                ObjectKind::Gunner,
                ObjectKind::Disciple,
            ],
        }),
        occupants_fire: None,
        research: None,
    }),
    default_settings: None,
    energy: None,
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
    /// not Groupable (design change 0348), behind a Storage. The four Cults infantry types walk in at its short side B
    /// and come out of the opposite one, C, and it holds 10 of them. Its orders come out of C too: Enhanced
    /// Constitution and Improved Metabolism, 25 Space Crystals and 160 frames each, and Both, 50 and 320; it trains no
    /// unit and recruits none (slice M5-N1b). Recruits build it from outside for 150 Space Crystals in 480 frames with
    /// one of them, from 10% HP.
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
        assert_eq!(structure.drop_off, None);
        let production = structure.production.expect("it augments");
        assert_eq!(production.exits, ExitRule::Side(SideLetter::C));
        assert_eq!((production.recipes, production.recruitment), (&[][..], None));
        let recipe = |order| production.augment_recipe(order).map(|r| (r.cost, r.frames));
        assert_eq!(recipe(AugmentOrder::EnhancedConstitution), Some((25, 160)));
        assert_eq!(recipe(AugmentOrder::ImprovedMetabolism), Some((25, 160)));
        assert_eq!(recipe(AugmentOrder::Both), Some((50, 320)), "the sum");
        let orders: Vec<AugmentOrder> = production.augment.iter().map(|r| r.order).collect();
        assert_eq!(orders, AugmentOrder::ALL);
        let holds = structure.holds.expect("the Altar holds Cults infantry");
        assert_eq!(
            (holds.entrance, holds.exit, holds.capacity),
            (SideLetter::B, ExitRule::Side(SideLetter::C), 10)
        );
        assert_eq!(
            holds.accepts,
            [
                ObjectKind::Recruit,
                ObjectKind::Militant,
                ObjectKind::Gunner,
                ObjectKind::Disciple
            ],
            "Q, W, E and R"
        );
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp, site.builder_goes),
            (SitePrice::Crystals(150), 480, 0.1, BuilderGoes::Outside)
        );
    }
}
