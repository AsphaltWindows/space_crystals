//! The Null Chamber — `factions/syndicate/structures/null_chamber.md` (design change 0367's placeholders; slice M6-SK:
//! its numbers; slice M6-SN: a Tier 2 Tunnel builds it, and it buffs the units leaving the network).

use crate::defs::structures::{ChamberDef, ExpansionDef, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/syndicate/structures/null_chamber.md`: "A shielded dome that works like a Faraday cage against imprinting",
/// an underground expansion behind a Hideout. Its numbers came with slice M6-SK (plan decision T84, by T69's rule);
/// slice M6-SN makes it live: a Tier 2 Tunnel builds it ([`NULL_CHAMBER_EXPANSION`], its ExpandMenu entry its kind),
/// and while its owner has one, the manned units in their network take their turns in it and leave buffed
/// ([`NULL_CHAMBER_SERVICE`], `network::services`). It has no commands. Its numbers are the design's "Placeholder
/// stats".
pub(crate) const NULL_CHAMBER: ObjectDef = ObjectDef {
    kind: ObjectKind::NullChamber,
    name: "Null Chamber",
    faction: Some(Faction::Syndicate),          // `factions/syndicate/`
    category: Category::Structure,              // Entity Type - Structure Type (Underground)
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    // Not listed: every expansion's, `factions/syndicate/tunnel_menus.md > Expansions under construction` (design change
    // 0254): "Sight: none (underground). **Groupable**: true."
    groupable: true,
    sight_range: 0.0,
    max_hp: Some(400.0),                     // MaxHP 400
    armor: Armor { point: 6.0, full: 24.0 }, // PointArmor 6, FullArmor 24 (underground expansion, n = 6)
    // `core/map.md > Elevation` (design change 0005): "Syndicate underground expansions (inside a Tunnel Area) may be
    // placed on sloped tiles".
    needs_flat_ground: false,
    structure: Some(StructureDef {
        // Not given (plan decision T29): AAAA, since none of its sides has a function.
        symmetry: SymmetryType::AAAA,
        build_radius_extension: 0,
        power: 0,
        underground: true, // Entity Type - Structure Type (Underground)
        expansion: Some(NULL_CHAMBER_EXPANSION),
        // TechPrerequisite - player owns at least one Hideout (anywhere in the Tunnel Network)
        requires: &[ObjectKind::Hideout],
        chamber: Some(NULL_CHAMBER_SERVICE),
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// The Null Chamber's construction by a Tunnel (`StructureDef::expansion`, slice M6-SN): "Tier Requirement - 2".
pub(crate) const NULL_CHAMBER_EXPANSION: ExpansionDef = ExpansionDef {
    tier: 2,     // Tier Requirement - 2
    cost: 200,   // Cost 200 Space Crystals
    frames: 480, // Build Time 480 frames (30 seconds)
    // `factions/syndicate/tunnel_menus.md > Expansions under construction` (design change 0254): "the ConstructionHP
    // Rule applies to every expansion (starts at 10%, grows with progress)".
    start_hp: 0.1,
};

/// (M6-SN) The Null Chamber's service of the network (`StructureDef::chamber`, plan decision T98): `> Placeholder
/// stats`, "Exit buff: duration **20 s**; **+2 grid units SightRange**; **20%** damage-taken reduction; ends abruptly; no
/// research needed. Throughput: each Null Chamber buffs **4 units at a time**, each buff taking **2 s** inside before
/// the unit can leave buffed". The buff's Detection isn't built (`network::services`).
const NULL_CHAMBER_SERVICE: ChamberDef = ChamberDef {
    slots: 4,       // each Null Chamber buffs 4 units at a time
    turn: 2.0,      // each buff taking 2 s inside before the unit can leave buffed
    duration: 20.0, // duration 20 s
    sight: 2.0,     // +2 grid units SightRange
    reduction: 0.2, // 20% damage-taken reduction
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::syndicate::tests::assert_expansion;

    /// The stat block (design change 0367): an underground structure, 2×2, MaxHP 400, the expansions' 6 / 24, behind a
    /// Hideout, with no commands; a Tier 2 Tunnel's for 200 Space Crystals in 480 frames, live since slice M6-SN, and
    /// its exit buff's placeholders: 4 units at a time, 2 s each, for 20 s of +2 grid units of SightRange and 20% less
    /// damage.
    #[test]
    fn the_null_chamber_follows_the_design() {
        let expansion = ExpansionDef {
            tier: 2,
            cost: 200,
            frames: 480,
            start_hp: 0.1,
        };
        assert_expansion(
            ObjectKind::NullChamber,
            "Null Chamber",
            2,
            400.0,
            &[ObjectKind::Hideout],
        );
        assert_eq!(NULL_CHAMBER_EXPANSION, expansion);
        let structure = ObjectKind::NullChamber.def().structure.as_ref().unwrap();
        assert_eq!(
            (structure.expansion, structure.production.as_ref()),
            (Some(expansion), None)
        );
        let service = ChamberDef {
            slots: 4,
            turn: 2.0,
            duration: 20.0,
            sight: 2.0,
            reduction: 0.2,
        };
        assert_eq!(structure.chamber, Some(service));
        let others = ObjectKind::ALL
            .iter()
            .filter(|kind| kind.def().structure.as_ref().is_some_and(|s| s.chamber.is_some()));
        assert_eq!(
            others.copied().collect::<Vec<_>>(),
            [ObjectKind::NullChamber],
            "the one type with a chamber"
        );
    }
}
