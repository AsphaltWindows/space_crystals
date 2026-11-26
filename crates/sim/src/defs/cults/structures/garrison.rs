//! The Garrison — `factions/cults/structures/garrison.md`.

use crate::defs::structures::{
    BuilderGoes, ExitRule, HoldsDef, OccupantFire, SiteDef, SitePrice, StructureDef, SymmetryType,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/cults/structures/garrison.md`: "The Cults' defensive structure. Infantry enter the Garrison and fire from
/// inside it: Recruits and Disciples fire **as if they were Militants** [...], while Militants and Gunners fire their
/// own weapons. Without units inside, the Garrison does not attack." Its numbers and its site came with slice M5-G2
/// (plan decision T69), its garrisoning with slice M5-N2 (plan decision T72): who may enter, through its one door
/// (`StructureDef::holds`), and how they fire from it (`StructureDef::occupants_fire`, `combat::occupants`). Its numbers
/// are the design's "Placeholder stats".
pub(crate) static GARRISON: ObjectDef = ObjectDef {
    kind: ObjectKind::Garrison,
    name: "Garrison",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`), and "when the
    // Garrison is destroyed, everyone inside comes out alive" (`> Garrisoning`).
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 6.0,    // SightRange 6 grid units
    max_hp: Some(400.0), // MaxHP 400
    max_shields: None,
    armor: Armor {
        point: 2.0, // PointArmor 2
        full: 8.0,  // FullArmor 8 (building armor rule, n = 2: sturdier as a defense)
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        // "SymmetryType AAAB, **the door on side B** (one door, entrance and exit; user, 2026-11-26; Q0116; side
        // placeholder)": A on top, right and bottom, B on the left in the default orientation.
        symmetry: SymmetryType::AAAB,
        // The Cults have neither Power nor a build area (`factions/cults/economy.md > Cults Building Mechanics`).
        build_radius_extension: 0,
        power: 0,
        seeds_build_area: false,
        production: None,
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        underground: false,
        tunnel: None,
        site: Some(GARRISON_SITE),
        expansion: None,
        requires: &[ObjectKind::Armory], // TechPrerequisite - player owns at least one Armory
        // Its occupants fire, not the building (`> Garrisoning`: "Without units inside, the Garrison does not attack").
        attack: None,
        popup: None,
        holds: Some(HoldsDef {
            // "the door on side B (one door, entrance and exit)": units walk in at its middle, as at every entrance
            // (`core/entities.md > Entrance and exit (production and research buildings)`, `holding::entry_path`).
            entrance: SideLetter::B,
            // "units leave **through the door**; on **Eject All** they all come out **at once**, ignoring collision
            // with each other until they are clear of the footprint".
            exit: ExitRule::AtOnce(SideLetter::B),
            capacity: 4, // GarrisonCapacity - 4 units
            // `> Garrisoning > Who can enter`: "Recruits, Militants, Gunners, and Disciples".
            accepts: &[
                ObjectKind::Recruit,
                ObjectKind::Militant,
                ObjectKind::Gunner,
                ObjectKind::Disciple,
            ],
        }),
        occupants_fire: Some(OccupantFire {
            // `> Garrisoning > Weapons`: "a Recruit or Disciple inside attacks with the Militant's AttackAttributes
            // (see Militant); Militants and Gunners use their own AttackAttributes".
            stand_in: &[
                (ObjectKind::Recruit, ObjectKind::Militant),
                (ObjectKind::Disciple, ObjectKind::Militant),
            ],
            // `> Garrisoning > Range bonus`: "**Militants and Gunners** inside get **+1 grid unit of Range** on their
            // attack".
            range_bonus_gu: 1.0,
            bonus_for: &[ObjectKind::Militant, ObjectKind::Gunner],
        }),
        research: None,
    }),
    default_settings: None,
    energy: None,
};

/// The Garrison's construction by Recruits (`StructureDef::site`, slice M5-E's; "Space Crystals cost and construction
/// time are listed under the Recruit's Constructs list").
pub(crate) const GARRISON_SITE: SiteDef = SiteDef {
    // `factions/cults/units/recruit.md > Constructs`: "Garrison: 150 Space Crystals, 400 frames (25 seconds)
    // BaseConstructionTime (placeholder). Requires: Armory.", paid once as the order is given
    // (`factions/cults/economy.md > Build Command`).
    price: SitePrice::Crystals(150),
    frames: 400,
    // `factions/cults/economy.md > Cults Building Mechanics`: a Cults building under construction is "vulnerable to
    // attack, same as the structure itself (ConstructionHP Rule)", whose sites "start at 10% of their maximum HP"
    // (`core/entities.md > ConstructionHP Rule`). Its panel's "**X: Cancel construction** while being built".
    start_hp: 0.1,
    builder_goes: BuilderGoes::Outside,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::cults::tests::assert_structure;

    /// 2×2, AAAB, MaxHP 400, PointArmor 2, FullArmor 8 (the building armour rule's n = 2), SightRange 6, Groupable,
    /// behind an Armory, with no attack of its own. Recruits build it from outside for 150 Space Crystals in 400 frames
    /// with one of them, from 10% HP. It holds 4 Recruits, Militants, Gunners and Disciples, in and out at its one
    /// door, B, everyone out at once; Recruits and Disciples inside fire the Militant's attack, Militants and Gunners
    /// their own with +1 grid unit of Range.
    #[test]
    fn the_garrison_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::Garrison,
            (2, 2),
            SymmetryType::AAAB,
            400.0,
            (2.0, 8.0),
            6.0,
            true,
        );
        assert_eq!(ObjectKind::Garrison.def().name, "Garrison");
        assert_eq!(structure.requires, [ObjectKind::Armory], "TechPrerequisite: an Armory");
        assert_eq!(
            (structure.production, structure.drop_off, structure.attack),
            (None, None, None)
        );
        let holds = structure.holds.expect("Infantry enter the Garrison");
        assert_eq!(
            (holds.entrance, holds.exit, holds.capacity),
            (SideLetter::B, ExitRule::AtOnce(SideLetter::B), 4),
            "one door, entrance and exit, on side B; GarrisonCapacity - 4 units"
        );
        let infantry = [
            ObjectKind::Recruit,
            ObjectKind::Militant,
            ObjectKind::Gunner,
            ObjectKind::Disciple,
        ];
        assert_eq!(holds.accepts, infantry);
        let fire = structure.occupants_fire.expect("its occupants fire");
        let weapons = infantry.map(|kind| (fire.weapon_kind(kind), fire.range_bonus(kind)));
        assert_eq!(
            weapons,
            [
                (ObjectKind::Militant, 0.0),
                (ObjectKind::Militant, 1.0),
                (ObjectKind::Gunner, 1.0),
                (ObjectKind::Militant, 0.0)
            ],
            "Recruits and Disciples fire as Militants, without the bonus"
        );
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp, site.builder_goes),
            (SitePrice::Crystals(150), 400, 0.1, BuilderGoes::Outside)
        );
    }
}
