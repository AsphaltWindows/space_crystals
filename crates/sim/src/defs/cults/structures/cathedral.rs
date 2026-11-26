//! The Cathedral — `factions/cults/structures/cathedral.md`.

use crate::defs::structures::{
    BuilderGoes, ExitRule, HoldsDef, ProductionDef, Recipe, SiteDef, SitePrice, StructureDef, SymmetryType,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/cults/structures/cathedral.md`: the "Tier 3 and last building of the Mystical/Human Enhancement branch.
/// Trains Recruits into Disciples, the Cults' spell caster". Its numbers, its site and its pool (slice M5-G2, plan
/// decision T69), and since slice M5-N4a its training (`> Training`: it "Works like the Armory's Training"), the
/// Armory's pool path as data (plan decision T71, `sim/producers/pool.rs`): Recruits walk into its pool (`holds`), and
/// it trains them into Disciples (`production`); Recruits build it from outside ([`CATHEDRAL_SITE`]) while their player
/// owns a Temple, as the Build is given (`Sim::check_build`). Its numbers are the design's "Placeholder stats".
pub(crate) static CATHEDRAL: ObjectDef = ObjectDef {
    kind: ObjectKind::Cathedral,
    name: "Cathedral",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Structure,
    // "Size 2x3": 2 tall, 3 wide in the default orientation, as the Armory's.
    footprint: Footprint::Tiles { w: 3, h: 2 },
    selectable: true, // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    // Groupable **false** (user, 2026-11-26; Q0105: Cults production buildings aren't groupable; design change 0348
    // point 5).
    groupable: false,
    sight_range: 4.0,    // SightRange 4 grid units
    max_hp: Some(400.0), // MaxHP 400
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 6.0,  // FullArmor 6
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        // "SymmetryType ABAC; EntranceSide B, ExitSide C (like the Armory)": A on top and bottom, B on the right, C on
        // the left, its short sides.
        symmetry: SymmetryType::ABAC,
        // The Cults have neither Power nor a build area (`factions/cults/economy.md > Cults Building Mechanics`).
        build_radius_extension: 0,
        power: 0,
        seeds_build_area: false,
        // `> Training`: "Works like the Armory's Training: Recruits enter the Cathedral and join an internal pool;
        // training orders are queueable and serial; queuing an order deducts its Space Crystals cost and reserves one
        // Recruit; when the order finishes, the Recruit is consumed and the trained unit exits to the RallyPoint", out of
        // its exit side ("ExitSide C (like the Armory)"), as the units it holds do (`holds`). Of the Recruits inside
        // (`sim/producers/pool.rs`), so it recruits nothing; its queue is bounded by its capacity ("so at most 10 orders
        // queued").
        production: Some(ProductionDef {
            exits: ExitRule::Side(SideLetter::C),
            recipes: &[
                // "**Disciple**: 1 Recruit, 75 Space Crystals, 320 frames (20 seconds)", the one unit it trains ("Panel
                // like the Armory's [...]: **Q: Train Disciple**"). The Disciple has no TechPrerequisite of its own: the
                // Cathedral's Temple is required to build it, not to train in it.
                Recipe {
                    kind: ObjectKind::Disciple,
                    cost: 75,
                    frames: 320,
                    requires: &[],
                },
            ],
            recruitment: None,
        }),
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        underground: false,
        tunnel: None,
        site: Some(CATHEDRAL_SITE),
        expansion: None,
        requires: &[ObjectKind::Temple], // TechPrerequisite - player owns at least one Temple
        attack: None,
        popup: None,
        // "EntranceSide B, ExitSide C (like the Armory)"; "InternalRecruitCapacity 10, counting reserved Recruits; so at
        // most 10 orders queued (bounded by capacity: reserved units count toward it; Q0104)", the pool buildings' rule
        // (design change 0347 point 5, slice M5-F2's); and `systems/controls/commands.md > Enter`: "Cults — Armory,
        // Cathedral: Recruits only (to be trained)".
        holds: Some(HoldsDef {
            entrance: SideLetter::B,
            exit: ExitRule::Side(SideLetter::C),
            capacity: 10,
            accepts: &[ObjectKind::Recruit],
        }),
        occupants_fire: None,
    }),
    default_settings: None,
    energy: None,
};

/// The Cathedral's construction by Recruits (`StructureDef::site`, slice M5-E's; "Space Crystals cost and construction
/// time are listed under the Recruit's Constructs list").
pub(crate) const CATHEDRAL_SITE: SiteDef = SiteDef {
    // `factions/cults/units/recruit.md > Constructs`: "Cathedral: 250 Space Crystals, 640 frames (40 seconds)
    // BaseConstructionTime (placeholder). Requires: Temple.", paid once as the order is given
    // (`factions/cults/economy.md > Build Command`).
    price: SitePrice::Crystals(250),
    frames: 640,
    // `factions/cults/economy.md > Cults Building Mechanics`: a Cults building under construction is "vulnerable to
    // attack, same as the structure itself (ConstructionHP Rule)", whose sites "start at 10% of their maximum HP"
    // (`core/entities.md > ConstructionHP Rule`).
    start_hp: 0.1,
    builder_goes: BuilderGoes::Outside,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::cults::tests::assert_structure;

    /// 2 tall and 3 wide, ABAC, MaxHP 400, PointArmor 1, FullArmor 6 (the building armour rule's n = 1), SightRange 4,
    /// not Groupable (design change 0348), behind a Temple. Its pool: Recruits walk in at side B and come out of side
    /// C, 10 of them, reserved ones counted; (M5-N4a) it trains them into Disciples, for 75 Space Crystals in 320
    /// frames each, out of side C, the recipe behind no other building. Recruits build it from outside for 250 Space
    /// Crystals in 640 frames with one of them, from 10% HP.
    #[test]
    fn the_cathedral_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::Cathedral,
            (3, 2),
            SymmetryType::ABAC,
            400.0,
            (1.0, 6.0),
            4.0,
            false,
        );
        assert_eq!(ObjectKind::Cathedral.def().name, "Cathedral");
        assert_eq!(structure.requires, [ObjectKind::Temple], "TechPrerequisite: a Temple");
        assert_eq!(structure.drop_off, None);
        let production = structure.production.expect("it trains");
        assert_eq!(production.exits, ExitRule::Side(SideLetter::C));
        let recipes: Vec<_> = (production.recipes.iter())
            .map(|r| (r.kind, r.cost, r.frames, r.requires))
            .collect();
        assert_eq!(recipes, [(ObjectKind::Disciple, 75, 320, &[][..])]);
        assert_eq!(production.recruitment, None);
        let holds = structure.holds.expect("its pool");
        assert_eq!(
            (holds.entrance, holds.exit),
            (SideLetter::B, ExitRule::Side(SideLetter::C))
        );
        assert_eq!((holds.capacity, holds.accepts), (10, &[ObjectKind::Recruit][..]));
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp, site.builder_goes),
            (SitePrice::Crystals(250), 640, 0.1, BuilderGoes::Outside)
        );
    }
}
