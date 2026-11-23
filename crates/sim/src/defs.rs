//! Static object type definitions (`core/entities.md > Object Type`).
//!
//! One `ObjectKind` per object type in the design; its numbers live in a `static`
//! [`ObjectDef`] whose comments cite the design section they come from. Faction
//! types get their own modules (e.g. `defs/gdo.rs`) as they are added.

use crate::player::Faction;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ObjectKind {
    /// `core/map.md > SpaceCrystalsPatch`
    CrystalPatch,
    /// `core/map.md > SupplyDeliveryStation`
    SupplyDeliveryStation,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Category {
    Unit,
    Structure,
    /// `core/map.md > Resource`: indestructible, unowned, no vision.
    Resource,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Footprint {
    /// Structures and resources: whole tiles, unrotated `w × h` (`Size - Height x Width`).
    Tiles { w: u32, h: u32 },
    /// Units: silhouette rectangle in space units (`systems/units.md > Unit > Silhouette`).
    Silhouette { w: f32, h: f32 },
}

/// Attributes shared by every object type (`core/entities.md > Object Type`).
#[derive(Clone, Debug)]
pub struct ObjectDef {
    pub kind: ObjectKind,
    pub name: &'static str,
    pub faction: Option<Faction>,
    pub category: Category,
    pub footprint: Footprint,
    pub selectable: bool,
    pub destructible: bool,
    pub groupable: bool,
    /// Grid units. 0 = provides no vision.
    pub sight_range: f32,
    /// `None` for indestructible types.
    pub max_hp: Option<f32>,
    /// Every footprint tile must be flat (`core/map.md > Elevation > Map resources only on flat terrain`).
    pub needs_flat_ground: bool,
}

impl ObjectKind {
    /// Every object type, in declaration order, so tools such as the map editor's palette
    /// list new types automatically. A test checks that it stays complete.
    pub const ALL: &'static [ObjectKind] = &[ObjectKind::CrystalPatch, ObjectKind::SupplyDeliveryStation];

    pub fn def(self) -> &'static ObjectDef {
        match self {
            ObjectKind::CrystalPatch => &CRYSTAL_PATCH,
            ObjectKind::SupplyDeliveryStation => &SUPPLY_DELIVERY_STATION,
        }
    }
}

/// `core/map.md > SpaceCrystalsPatch`, `> Resource`.
static CRYSTAL_PATCH: ObjectDef = ObjectDef {
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
    needs_flat_ground: true, // `core/map.md > Elevation > Map resources only on flat terrain`
};

/// `core/map.md > SupplyDeliveryStation`, `> Resource`.
static SUPPLY_DELIVERY_STATION: ObjectDef = ObjectDef {
    kind: ObjectKind::SupplyDeliveryStation,
    name: "Supply Delivery Station",
    faction: None,
    category: Category::Resource,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size - 2x2
    selectable: true,
    destructible: false,
    groupable: false,
    sight_range: 0.0,
    max_hp: None,
    needs_flat_ground: true, // `core/map.md > Elevation > Map resources only on flat terrain`
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde::de::IntoDeserializer;
    use serde::de::value::{Error as ValueError, U32Deserializer};

    /// Every variant, found through serde's derived `Deserialize`, which accepts a unit
    /// variant by its declaration index and rejects the first index past the last variant.
    fn declared_kinds() -> Vec<ObjectKind> {
        (0u32..)
            .map_while(|i| {
                let index: U32Deserializer<ValueError> = i.into_deserializer();
                ObjectKind::deserialize(index).ok()
            })
            .collect()
    }

    #[test]
    fn all_lists_every_kind_in_declaration_order() {
        assert!(!ObjectKind::ALL.is_empty());
        assert_eq!(ObjectKind::ALL, declared_kinds().as_slice());
    }

    #[test]
    fn defs_are_consistent() {
        for &kind in ObjectKind::ALL {
            let def = kind.def();
            let name = def.name;
            assert_eq!(def.kind, kind, "{name}: def of the wrong kind");
            assert!(!name.is_empty(), "{kind:?}: empty name");
            match def.footprint {
                Footprint::Tiles { w, h } => {
                    assert!((1..=16).contains(&w) && (1..=16).contains(&h), "{name}: {w}×{h} tiles");
                    assert_ne!(def.category, Category::Unit, "{name}: units have silhouettes");
                }
                Footprint::Silhouette { w, h } => {
                    assert!(
                        w.is_finite() && w > 0.0 && h.is_finite() && h > 0.0,
                        "{name}: {w}×{h} su"
                    );
                    assert_eq!(def.category, Category::Unit, "{name}: only units have silhouettes");
                }
            }
            assert!(
                def.sight_range.is_finite() && def.sight_range >= 0.0,
                "{name}: sight range"
            );
            match def.max_hp {
                Some(hp) => assert!(def.destructible && hp.is_finite() && hp > 0.0, "{name}: max HP {hp}"),
                None => assert!(!def.destructible, "{name}: destructible without max HP"),
            }
            if def.category == Category::Resource {
                // `core/map.md > Resource`: indestructible, unowned (no faction), no vision.
                assert!(
                    !def.destructible && def.max_hp.is_none(),
                    "{name}: resources are indestructible"
                );
                assert_eq!(def.faction, None, "{name}: resources belong to no faction");
                assert_eq!(def.sight_range, 0.0, "{name}: resources provide no vision");
            }
        }
    }
}
