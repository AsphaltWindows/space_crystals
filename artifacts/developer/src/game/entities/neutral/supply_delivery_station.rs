//! Supply Delivery Station entity definition — neutral resource node.

use bevy::prelude::*;

use crate::types::{ObjectEnum, FactionEnum};
use crate::game::entities::types::{EntityDefinition, BaseStats, SpawnParams};
use crate::game::types::objects::ObjectType;
use crate::game::types::ObjectInstance;
use crate::shared::types::{Selectable, SelectionBounds, GridPosition};
use crate::game::world::types::SupplyDeliveryStation;

/// Default delivery size for a Supply Delivery Station.
pub const DEFAULT_DELIVERY_SIZE: u32 = 100;

/// Default delivery interval (in seconds) for a Supply Delivery Station.
pub const DEFAULT_DELIVERY_INTERVAL: f32 = 60.0;

/// Supply Delivery Station entity definition marker.
pub struct SupplyDeliveryStationDef;

impl EntityDefinition for SupplyDeliveryStationDef {
    const OBJECT: ObjectEnum = ObjectEnum::SupplyDeliveryStation;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Supply Delivery Station".to_string(),
            size: (2, 2),
            destructible: false,
            sight_range: 0,
            groupable: false,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: 0,
            point_armor: 0,
            full_armor: 0,
            // Neutral entity — uses Colonists as placeholder faction.
            // Actual neutrality is conveyed by Owner::neutral() in spawn().
            faction: FactionEnum::Colonists,
        }
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        commands.insert((
            Transform::from_translation(params.position + Vec3::new(0.0, 0.1, 0.0)),
            ObjectInstance::indestructible(ObjectEnum::SupplyDeliveryStation),
            params.owner,
            SupplyDeliveryStation {
                delivery_size: DEFAULT_DELIVERY_SIZE,
                delivery_interval: DEFAULT_DELIVERY_INTERVAL,
                current_supplies: DEFAULT_DELIVERY_SIZE,
                time_until_next_delivery: DEFAULT_DELIVERY_INTERVAL,
            },
            Selectable,
            SelectionBounds::from_dimensions(1.6, 0.2, 1.6),
            GridPosition {
                x: ((params.position.x - 0.5) + 32.0) as i32,
                z: ((params.position.z - 0.5) + 32.0) as i32,
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::entities::types::EntityRegistry;

    #[test]
    fn sds_object_enum() {
        assert_eq!(SupplyDeliveryStationDef::OBJECT, ObjectEnum::SupplyDeliveryStation);
    }

    #[test]
    fn sds_object_type() {
        let ot = SupplyDeliveryStationDef::object_type();
        assert_eq!(ot.name, "Supply Delivery Station");
        assert_eq!(ot.size, (2, 2));
        assert!(!ot.destructible);
        assert_eq!(ot.sight_range, 0);
        assert!(!ot.groupable);
    }

    #[test]
    fn sds_base_stats() {
        let stats = SupplyDeliveryStationDef::base_stats();
        assert_eq!(stats.max_hp, 0);
        assert_eq!(stats.point_armor, 0);
        assert_eq!(stats.full_armor, 0);
        assert_eq!(stats.faction, FactionEnum::Colonists);
    }

    #[test]
    fn sds_cost_is_none() {
        assert!(SupplyDeliveryStationDef::cost().is_none());
    }

    #[test]
    fn sds_command_slots_empty() {
        assert!(SupplyDeliveryStationDef::command_slots().is_empty());
    }

    #[test]
    fn sds_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_resource::<SupplyDeliveryStationDef>();
        let entry = registry.get(&ObjectEnum::SupplyDeliveryStation);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Supply Delivery Station");
        assert_eq!(entry.base_stats.max_hp, 0);
        assert!(entry.cost.is_none());
        assert!(entry.command_slots.is_empty());
        assert!(matches!(entry.kind, crate::game::entities::types::EntityKind::Resource));
    }
}
