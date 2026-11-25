//! The Bio Portal — `factions/colonists/structures/bio_portal.md`.

use crate::defs::structures::{ExitRule, ProductionDef, Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/colonists/structures/bio_portal.md`: the infantry's producer.
pub(crate) static BIO_PORTAL: ObjectDef = ObjectDef {
    kind: ObjectKind::BioPortal,
    name: "Bio Portal",
    faction: Some(Faction::Colonists),          // Faction - Colonists
    category: Category::Structure,              // Entity Type - Structure Type
    footprint: Footprint::Tiles { w: 3, h: 3 }, // Size - 3x3 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: true,                            // Groupable - true
    sight_range: 3.0,                           // SightRange - 3 grid units
    max_hp: Some(250.0),                        // MaxHP - 250
    max_shields: Some(15.0),                    // MaxShields - 15
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 9.0,  // FullArmor - 9
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        // SymmetryType - AABB: A on top and on the right, B at the bottom and on the left, unturned (design change
        // 0079), so the two A sides are adjacent ("AABB: neighbouring sides, top and right in the default
        // orientation", design change 0145, answering question Q0041 point 5).
        symmetry: SymmetryType::AABB,
        build_radius_extension: 0, // none (see `defs/colonists.rs`)
        power: 0,                  // none (plan decision T21)
        seeds_build_area: false,
        production: Some(ProductionDef {
            // `> Exit Behavior`: "The Bio Portal's two open sides are its "A" sides ... Units leave in any direction
            // through them" (the energy doorway at the centre, design change 0145): a half-circle of directions.
            exits: ExitRule::Centre(&[SideLetter::A]),
            recipes: &[Recipe {
                kind: ObjectKind::SabreLegionnaire, // Produces: "Sabre Legionnaire: 100 Space Crystals, 160 frames"
                cost: 100,
                frames: 160,
            }],
        }),
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
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

    /// It produces Sabre Legionnaires, which leave its energy doorway through its two A sides (`> Exit Behavior`,
    /// design change 0145), never Rifle Legionnaires ("Not directly produced").
    #[test]
    fn the_bio_portal_follows_the_design() {
        let structure = crate::defs::colonists::tests::assert_structure(
            ObjectKind::BioPortal,
            (3, 3),
            SymmetryType::AABB,
            250.0,
            15.0,
            (1.0, 9.0),
            3.0,
            true,
        );
        assert!(structure.warp_in.is_empty());
        // Produces: "Sabre Legionnaire: 100 Space Crystals, 160 frames (10 seconds)" (the Lancer's and the psionic
        // unit's cost and time are TBD).
        let production = structure.production.unwrap();
        assert_eq!(
            (production.exits, production.recipes),
            (
                ExitRule::Centre(&[SideLetter::A]),
                &[Recipe {
                    kind: ObjectKind::SabreLegionnaire,
                    cost: 100,
                    frames: 160,
                }][..]
            )
        );
        for &kind in ObjectKind::ALL {
            let produced = kind.def().structure.as_ref().and_then(|s| s.production.as_ref());
            assert!(
                produced.is_none_or(|p| p.recipe(ObjectKind::RifleLegionnaire).is_none()),
                "{kind:?} produces Rifle Legionnaires"
            );
        }
    }
}
