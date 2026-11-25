//! The Supply Delivery Station — `core/map.md > SupplyDeliveryStation`.

use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};

/// `core/map.md > SupplyDeliveryStation`, `> Resource`.
pub(crate) static SUPPLY_DELIVERY_STATION: ObjectDef = ObjectDef {
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
    max_shields: None,
    armor: Armor::NONE,
    needs_flat_ground: true, // `core/map.md > Elevation > Map resources only on flat terrain`
    unit: None,
    structure: None,
};
