//! The Supply Delivery Station — `core/map.md > SupplyDeliveryStation`.

use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::scenario::SpawnSettings;
use crate::units::{TICKS_PER_SECOND, Tick};

/// `> StartingSupplies`: the default for newly placed stations (placeholder, user 2026-11-23).
pub const DEFAULT_STARTING_SUPPLIES: u32 = 0;
/// `> DeliverySize`: the default for newly placed stations (placeholder, user 2026-11-23).
pub const DEFAULT_DELIVERY_SIZE: u32 = 4;
/// `> DeliveryInterval`: the default for newly placed stations, 60 s (placeholder, user 2026-11-23).
pub const DEFAULT_DELIVERY_INTERVAL: Tick = 60 * TICKS_PER_SECOND;

/// `core/map.md > SupplyDeliveryStation`, `> Resource`.
pub(crate) const SUPPLY_DELIVERY_STATION: ObjectDef = ObjectDef {
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
    default_settings: Some(SpawnSettings::SupplyDeliveryStation {
        starting_supplies: DEFAULT_STARTING_SUPPLIES,
        delivery_size: DEFAULT_DELIVERY_SIZE,
        delivery_interval: DEFAULT_DELIVERY_INTERVAL,
    }),
    energy: None,
};
