//! The GDO's structures — `factions/gdo/structures/` (the Deployment Center, the Barracks, the Career Campus, the
//! Arsenal) and the economy buildings of `factions/gdo/economy.md` (the Generator, the Extraction Facility, the
//! Extraction Plate), one module each.

pub(crate) mod arsenal;
pub(crate) mod barracks;
pub(crate) mod career_campus;
pub(crate) mod deployment_center;
pub(crate) mod extraction_facility;
pub(crate) mod extraction_plate;
pub(crate) mod generator;

/// A GDO structure's stat block (`factions/gdo/structures/*.md`, `factions/gdo/economy.md`): size (w × h in the
/// default orientation), SymmetryType, MaxHP, PointArmor and FullArmor, SightRange, Groupable, BuildRadiusExtension
/// and Power. It is a destructible, selectable GDO building on flat ground.
#[cfg(test)]
#[expect(clippy::too_many_arguments, reason = "one argument per line of the stat block")]
fn assert_stat_block(
    kind: crate::defs::ObjectKind,
    size: (u32, u32),
    symmetry: crate::defs::SymmetryType,
    hp: f32,
    armor: (f32, f32),
    sight: f32,
    groupable: bool,
    extension: u32,
    power: i32,
) -> &'static crate::defs::StructureDef {
    use crate::defs::{Armor, Category};
    use crate::player::Faction;
    let def = kind.def();
    let structure = def.structure.as_ref().unwrap();
    assert_eq!(
        (def.category, def.faction, def.tile_size(), structure.symmetry),
        (Category::Structure, Some(Faction::Gdo), Some(size), symmetry),
        "{kind:?}"
    );
    let (point, full) = armor;
    assert_eq!(
        (def.max_hp, def.armor, def.sight_range),
        (Some(hp), Armor { point, full }, sight),
        "{kind:?}"
    );
    assert_eq!(
        (def.destructible, def.groupable, def.selectable),
        (true, groupable, true),
        "{kind:?}"
    );
    assert_eq!(
        (structure.build_radius_extension, structure.power),
        (extension, power),
        "{kind:?}"
    );
    assert!(def.unit.is_none() && def.needs_flat_ground, "{kind:?}");
    structure
}
