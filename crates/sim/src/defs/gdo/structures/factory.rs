//! The Factory — `factions/gdo/structures/factory.md` (design change 0366, slices M6-GK and M6-GF): "GDO vehicle
//! production structure. Allows the building of vehicles." Its numbers are the design's "Placeholder stats" (plan
//! decision T84). (M6-GF) The Deployment Center constructs it ([`RECIPE`], in its `constructs`), behind a Barracks, and
//! it produces "as the Barracks (queue 20, refunds, rally)": its vehicles out of the doorway of its exit side, B, their
//! boxes turned to face out of it (plan decision T100, `ExitRule::Side`), and it starts with the default RallyPoint of a
//! building that releases vehicles through an exit side (`core/entities.md > Entrance and exit > Exit Doorway` point
//! 7, design changes 0446 and 0447: 3 grid units out, `ProductionDef::default_rally`). It trains the Raptor and the
//! Twister, and (M6-GA) the Artillery Rig behind Advanced Mechanics; the Prospector, the design's other one, is slice
//! M6-GP's, its entry held until then (`crate::defs::PLANNED_RECIPES`).

use crate::defs::gdo::units::{artillery_rig, raptor, twister};
use crate::defs::structures::{ExitRule, ProductionDef, Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/gdo/structures/factory.md`: its placeholder stats (design change 0366).
pub(crate) const FACTORY: ObjectDef = ObjectDef {
    kind: ObjectKind::Factory,
    name: "Factory",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 3, h: 3 }, // Size 3x3
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 4.0,    // SightRange 4 grid units
    max_hp: Some(500.0), // MaxHP 500
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 9.0,  // FullArmor 9
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        symmetry: SymmetryType::ABAC, // SymmetryType ABAC (exit side B, like the Barracks)
        build_radius_extension: 2,    // BuildRadiusExtension 2 grid units
        power: -30,                   // Power -30
        // (M6-GF) "Production as the Barracks (queue 20, refunds, rally)": the Barracks' queue, refunds and rally point
        // (`crate::production`), out of the doorway in the middle of side B, a vehicle's box turned to face out of it
        // (plan decision T100; `sim/producers/release.rs`'s `exit_half`, slice M5-N6's).
        production: Some(ProductionDef {
            exits: ExitRule::Side(SideLetter::B), // "exit side B, like the Barracks"
            // `factions/gdo/structures/factory.md > Produces`, in its order: "Raptor: 100 Space Crystals, 240 frames
            // (15 seconds) (placeholder)" and "Twister: 125 Space Crystals, 320 frames (20 seconds) (placeholder)",
            // its "+ 1 Supply" (design change 0441), each its def module's; (M6-GA) "Artillery Rig: 250 Space
            // Crystals, 480 frames (30 seconds) (placeholder). Requires: player owns at least one Advanced Mechanics",
            // its "+ 5 Supplies" (design change 0435). The Prospector's, behind Advanced Mechanics too, is held until
            // slice M6-GP (`crate::defs::PLANNED_RECIPES`).
            recipes: &[raptor::RECIPE, twister::RECIPE, artillery_rig::RECIPE],
            recruitment: None,
            augment: &[],
            // `core/entities.md > Entrance and exit > Exit Doorway` point 7: "Every building that releases **wheeled or
            // tracked vehicles** through an exit side (today the Cults **Chop Shop** and the GDO **Factory**)
            // therefore **starts with a RallyPoint 3 grid units straight out from the middle of its exit side**
            // (placeholder distance)" (design changes 0446 and 0447, answering Q0175).
            default_rally: Some(3),
        }),
        // TechPrerequisite - player owns at least one Barracks, wherever it is constructed (`crate::tech`).
        requires: &[ObjectKind::Barracks],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/deployment_center.md > Constructs`: "Factory: 250 Space Crystals, 480 frames (30 seconds)
/// (placeholder). Requires: player owns at least one Barracks" (its own TechPrerequisite). (M6-GF) In the Deployment
/// Center's `constructs`.
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::Factory,
    cost: 250,
    frames: 480,
    requires: &[],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block, ABAC, and its TechPrerequisite: a Barracks. Its Deployment Center entry, 250 Space
    /// Crystals and 480 frames, is live (M6-GF). It constructs nothing, and produces as the Barracks does, out of side
    /// B: the Raptor for 100 Space Crystals in 240 frames, then the Twister for 125 and 1 Supply in 320, each with no
    /// prerequisite of its own, then (M6-GA) the Artillery Rig for 250 and 5 Supplies in 480, behind Advanced Mechanics;
    /// with a default rally point 3 grid units out.
    #[test]
    fn the_factory_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::Factory,
            (3, 3),
            SymmetryType::ABAC,
            500.0,
            (1.0, 9.0),
            4.0,
            true,
            2,
            -30,
        );
        assert_eq!(structure.requires, [ObjectKind::Barracks]);
        assert!(structure.constructs.is_empty() && structure.holds.is_none() && structure.research.is_none());
        let recipe = |kind, cost, frames, supplies| Recipe {
            kind,
            cost,
            frames,
            requires: &[],
            supplies,
        };
        let rig = Recipe {
            requires: &[ObjectKind::AdvancedMechanics],
            ..recipe(ObjectKind::ArtilleryRig, 250, 480, 5)
        };
        assert_eq!(RECIPE, recipe(ObjectKind::Factory, 250, 480, 0));
        let dc = ObjectKind::DeploymentCenter.def().structure.as_ref().unwrap();
        assert!(dc.constructs.contains(&RECIPE), "the Deployment Center constructs it");
        let production = structure.production.expect("a producer");
        let barracks = ObjectKind::Barracks
            .def()
            .structure
            .as_ref()
            .unwrap()
            .production
            .unwrap();
        assert_eq!(production.exits, barracks.exits, "exit side B, like the Barracks");
        assert_eq!(
            production.recipes,
            [
                recipe(ObjectKind::Raptor, 100, 240, 0),
                recipe(ObjectKind::Twister, 125, 320, 1),
                rig
            ]
        );
        assert_eq!(
            (production.recruitment, production.augment, production.default_rally),
            (None, &[][..], Some(3))
        );
    }
}
