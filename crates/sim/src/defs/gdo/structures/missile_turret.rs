//! The Missile Turret — `factions/gdo/structures/missile_turret.md` (design change 0366, slice M6-GK): "GDO defensive
//! structure. Attacks both air and ground targets". Its numbers are the design's "Placeholder stats" (plan decision
//! T84), but for its attack: its homing missile ("**HeadDisjointed** [...] TargetDomain Universal, SingleTarget; Damage
//! 18; Range 6 grid units; MinRange 0; ProjectileSpeed 16 su/frame; Aim 4, Firing 2, Cooldown 4, Reload 14 frames"),
//! with the Laser Turret's targeting, is slice M6-GR's (`StructureDef::attack`), as is its Defense System entry
//! ([`RECIPE`], held). No player constructs one before.

use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/missile_turret.md`: its placeholder stats (design change 0366).
pub(crate) const MISSILE_TURRET: ObjectDef = ObjectDef {
    kind: ObjectKind::MissileTurret,
    name: "Missile Turret",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 1, h: 1 }, // Size 1x1
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 7.0,    // SightRange 7 grid units
    max_hp: Some(250.0), // MaxHP 250
    armor: Armor {
        point: 2.0, // PointArmor 2
        full: 2.0,  // FullArmor 2
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType AAAA
        build_radius_extension: 0,    // BuildRadiusExtension 0
        power: -10,                   // Power −10
        requires: &[],                // TechPrerequisite - None
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/defense_system.md > Constructs`: "Missile Turret: 100 Space Crystals, 240 frames (15
/// seconds) (placeholder)". Held until slice M6-GR (`crate::defs::PLANNED_RECIPES`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::MissileTurret,
    cost: 100,
    frames: 240,
    requires: &[],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block, with no TechPrerequisite and no attack yet (slice M6-GR's); its Defense System
    /// entry, 100 Space Crystals and 240 frames, is held.
    #[test]
    fn the_missile_turret_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::MissileTurret,
            (1, 1),
            SymmetryType::AAAA,
            250.0,
            (2.0, 2.0),
            7.0,
            true,
            0,
            -10,
        );
        assert_eq!(structure.requires, [] as [ObjectKind; 0]);
        assert_eq!((structure.attack, structure.popup), (None, None));
        assert_eq!(RECIPE, super::super::held(ObjectKind::MissileTurret, 100, 240));
    }
}
