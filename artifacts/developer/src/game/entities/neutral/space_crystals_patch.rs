//! Space Crystals Patch entity definition — neutral resource node.

use bevy::prelude::*;

use crate::types::{ObjectEnum, FactionEnum};
use crate::game::entities::types::{EntityDefinition, BaseStats, SpawnParams};
use crate::game::types::objects::ObjectType;
use crate::game::types::ObjectInstance;
use crate::shared::types::{Selectable, SelectionBounds, GridPosition};
use crate::game::world::types::SpaceCrystalPatch;

/// Default initial amount of space crystals in a patch.
pub const DEFAULT_INITIAL_AMOUNT: u32 = 5000;

/// Space Crystals Patch entity definition marker.
pub struct SpaceCrystalsPatchDef;

impl EntityDefinition for SpaceCrystalsPatchDef {
    const OBJECT: ObjectEnum = ObjectEnum::SpaceCrystalsPatch;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Space Crystals Patch".to_string(),
            size: (1, 1),
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
            Transform::from_translation(params.position + Vec3::new(0.0, 0.4, 0.0)),
            ObjectInstance::indestructible(ObjectEnum::SpaceCrystalsPatch),
            params.owner,
            SpaceCrystalPatch {
                remaining_amount: DEFAULT_INITIAL_AMOUNT,
                initial_amount: DEFAULT_INITIAL_AMOUNT,
                has_plate: false,
            },
            Selectable,
            SelectionBounds::from_dimensions(1.2, 1.0, 1.2),
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
    fn scp_object_enum() {
        assert_eq!(SpaceCrystalsPatchDef::OBJECT, ObjectEnum::SpaceCrystalsPatch);
    }

    #[test]
    fn scp_object_type() {
        let ot = SpaceCrystalsPatchDef::object_type();
        assert_eq!(ot.name, "Space Crystals Patch");
        assert_eq!(ot.size, (1, 1));
        assert!(!ot.destructible);
        assert_eq!(ot.sight_range, 0);
        assert!(!ot.groupable);
    }

    #[test]
    fn scp_base_stats() {
        let stats = SpaceCrystalsPatchDef::base_stats();
        assert_eq!(stats.max_hp, 0);
        assert_eq!(stats.point_armor, 0);
        assert_eq!(stats.full_armor, 0);
        assert_eq!(stats.faction, FactionEnum::Colonists);
    }

    #[test]
    fn scp_cost_is_none() {
        assert!(SpaceCrystalsPatchDef::cost().is_none());
    }

    #[test]
    fn scp_command_slots_empty() {
        assert!(SpaceCrystalsPatchDef::command_slots().is_empty());
    }

    #[test]
    fn scp_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_resource::<SpaceCrystalsPatchDef>();
        let entry = registry.get(&ObjectEnum::SpaceCrystalsPatch);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Space Crystals Patch");
        assert_eq!(entry.base_stats.max_hp, 0);
        assert!(entry.cost.is_none());
        assert!(entry.command_slots.is_empty());
        assert!(matches!(entry.kind, crate::game::entities::types::EntityKind::Resource));
    }
}
