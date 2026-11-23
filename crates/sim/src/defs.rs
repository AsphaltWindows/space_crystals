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
}

impl ObjectKind {
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
};
