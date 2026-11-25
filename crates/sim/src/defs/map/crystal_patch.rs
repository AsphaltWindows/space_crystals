//! The Space Crystals patch — `core/map.md > SpaceCrystalsPatch`.

use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};

/// `core/map.md > SpaceCrystalsPatch`, `> Resource`.
pub(crate) static CRYSTAL_PATCH: ObjectDef = ObjectDef {
    kind: ObjectKind::CrystalPatch,
    name: "Space Crystals",
    faction: None,
    category: Category::Resource,
    footprint: Footprint::Tiles { w: 1, h: 1 }, // Size - 1x1
    selectable: true,
    destructible: false,
    groupable: false,
    sight_range: 0.0,
    max_hp: None,
    max_shields: None,
    armor: Armor::NONE,
    needs_flat_ground: true, // `core/map.md > Elevation > Map resources only on flat terrain`
    unit: None,
    structure: None,
};
