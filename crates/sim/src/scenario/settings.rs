//! Per-object values that a map sets on its spawns (`core/map_editor.md > Per-object Resource Values`): each patch's
//! starting amount and each Supply Delivery Station's supplies, delivery size and interval.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;
/// The defaults of the per-object values, each with the def of the type it belongs to (`defs/map/`): a patch's
/// StartingAmount, a station's StartingSupplies, DeliverySize and DeliveryInterval.
pub use crate::defs::{
    DEFAULT_DELIVERY_INTERVAL, DEFAULT_DELIVERY_SIZE, DEFAULT_STARTING_AMOUNT, DEFAULT_STARTING_SUPPLIES,
};
use crate::object::ResourceState;
use crate::units::Tick;

/// The values a map sets on one object (`core/map_editor.md > Per-object Resource Values`). Each variant is named
/// after the [`ObjectKind`] it belongs to. Map files store them explicitly on every spawn of such a kind.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpawnSettings {
    /// `core/map.md > SpaceCrystalsPatch`.
    CrystalPatch {
        /// `StartingAmount`: the patch's RemainingAmount at game start.
        starting_amount: u32,
    },
    /// `core/map.md > SupplyDeliveryStation`.
    SupplyDeliveryStation {
        /// `StartingSupplies`: CurrentSupplies at game start.
        starting_supplies: u32,
        /// `DeliverySize`: supplies per delivery.
        delivery_size: u32,
        /// `DeliveryInterval`, in ticks.
        delivery_interval: Tick,
    },
}

/// A value of [`SpawnSettings`] that makes no sense.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SettingsProblem {
    /// A patch with nothing in it would vanish at once (`core/map.md > SpaceCrystalsPatch`).
    EmptyPatch,
    /// A station whose deliveries bring nothing never gets supplies.
    EmptyDelivery,
    /// Deliveries come at least one tick apart.
    ZeroInterval,
}

impl fmt::Display for SettingsProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SettingsProblem::EmptyPatch => "the starting amount must be at least 1",
            SettingsProblem::EmptyDelivery => "the delivery size must be at least 1",
            SettingsProblem::ZeroInterval => "the delivery interval must be at least 1 tick",
        })
    }
}

impl SpawnSettings {
    /// The values newly placed objects of `kind` get (the map editor, debug spawns), its def's
    /// (`ObjectDef::default_settings`), or `None` for kinds without per-object values.
    pub fn defaults(kind: ObjectKind) -> Option<Self> {
        kind.def().default_settings
    }

    /// The kind of object these values belong to.
    pub fn kind(&self) -> ObjectKind {
        match self {
            Self::CrystalPatch { .. } => ObjectKind::CrystalPatch,
            Self::SupplyDeliveryStation { .. } => ObjectKind::SupplyDeliveryStation,
        }
    }

    /// Whether every value makes sense.
    pub fn check(&self) -> Result<(), SettingsProblem> {
        match *self {
            Self::CrystalPatch { starting_amount: 0 } => Err(SettingsProblem::EmptyPatch),
            Self::SupplyDeliveryStation { delivery_size: 0, .. } => Err(SettingsProblem::EmptyDelivery),
            Self::SupplyDeliveryStation {
                delivery_interval: 0, ..
            } => Err(SettingsProblem::ZeroInterval),
            _ => Ok(()),
        }
    }

    /// The resource state an object with these values starts the game with.
    pub(crate) fn starting_resource(&self) -> ResourceState {
        match *self {
            Self::CrystalPatch { starting_amount } => ResourceState::CrystalPatch {
                remaining: starting_amount,
            },
            // `core/map.md > SupplyDeliveryStation > StartingSupplies`: a station starting at 0 begins its first
            // countdown at game start; a stocked one only once it is emptied.
            Self::SupplyDeliveryStation {
                starting_supplies,
                delivery_size,
                delivery_interval,
            } => ResourceState::SupplyStation {
                delivery_size,
                delivery_interval,
                current_supplies: starting_supplies,
                next_delivery_in: (starting_supplies == 0).then_some(delivery_interval),
            },
        }
    }

    /// The values to create an object of `kind` with: `settings` if they belong to that kind, else the kind's
    /// defaults.
    pub(crate) fn resolve(kind: ObjectKind, settings: Option<&SpawnSettings>) -> Option<SpawnSettings> {
        settings
            .filter(|s| s.kind() == kind)
            .copied()
            .or_else(|| Self::defaults(kind))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_exist_for_every_kind_with_values_and_are_valid() {
        for &kind in ObjectKind::ALL {
            if let Some(settings) = SpawnSettings::defaults(kind) {
                assert_eq!(settings.kind(), kind);
                assert_eq!(settings.check(), Ok(()), "{kind:?}");
            }
        }
        assert_eq!(DEFAULT_DELIVERY_INTERVAL, 960);
    }

    #[test]
    fn nonsense_values_are_named() {
        let station = |starting_supplies, delivery_size, delivery_interval| SpawnSettings::SupplyDeliveryStation {
            starting_supplies,
            delivery_size,
            delivery_interval,
        };
        let patch = SpawnSettings::CrystalPatch { starting_amount: 0 };
        assert_eq!(patch.check(), Err(SettingsProblem::EmptyPatch));
        assert_eq!(station(5, 0, 10).check(), Err(SettingsProblem::EmptyDelivery));
        assert_eq!(station(5, 4, 0).check(), Err(SettingsProblem::ZeroInterval));
        assert_eq!(station(0, 1, 1).check(), Ok(()));
        assert_eq!(station(u32::MAX, u32::MAX, u32::MAX).check(), Ok(()));
    }

    #[test]
    fn stocked_stations_wait_and_empty_ones_count_down() {
        let stocked = SpawnSettings::SupplyDeliveryStation {
            starting_supplies: 6,
            delivery_size: 3,
            delivery_interval: 100,
        };
        assert_eq!(
            stocked.starting_resource(),
            ResourceState::SupplyStation {
                delivery_size: 3,
                delivery_interval: 100,
                current_supplies: 6,
                next_delivery_in: None,
            }
        );
        let empty = SpawnSettings::defaults(ObjectKind::SupplyDeliveryStation).unwrap();
        assert!(matches!(
            empty.starting_resource(),
            ResourceState::SupplyStation {
                current_supplies: 0,
                next_delivery_in: Some(960),
                ..
            }
        ));
        // Values of the wrong kind give way to the kind's defaults.
        let patch = SpawnSettings::resolve(ObjectKind::CrystalPatch, Some(&stocked));
        assert_eq!(patch, SpawnSettings::defaults(ObjectKind::CrystalPatch));
    }
}
