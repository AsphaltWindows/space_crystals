//! Data types for the declarative entity definition system.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::types::{ObjectEnum, FactionEnum, UnitBaseEnum, SymmetryTypeEnum, StructureRotation, Owner};
use crate::game::types::objects::ObjectType;
use crate::game::types::structures::StructureCost;
use crate::game::units::types::unit_data::AttackAttributesData;
use crate::game::units::types::movement::MovementParams;
use crate::ui::types::{ObjectInterfaceState, CommandButtonAction};

// === Shared Types ===

/// Base stats common to all entities (units, structures, resources).
#[derive(Clone, Debug)]
pub struct BaseStats {
    pub max_hp: u32,
    pub point_armor: u32,
    pub full_armor: u32,
    pub faction: FactionEnum,
}

/// Unit-specific stats.
#[derive(Clone, Debug)]
pub struct UnitStats {
    pub unit_base: UnitBaseEnum,
    pub silhouette_width: u32,
    pub silhouette_height: u32,
    pub control_cost: u32,
}

/// Structure-specific stats.
#[derive(Clone, Debug)]
pub struct StructureStats {
    /// Positive = power generator, negative = power consumer.
    pub power_value: i32,
    /// Build radius extension in grid units (0 if none).
    pub build_radius: u32,
    pub symmetry: SymmetryTypeEnum,
}

/// Runtime conditions that gate whether a command slot is visible.
#[derive(Clone, Debug, PartialEq)]
pub enum SlotCondition {
    /// Slot visible only when the structure has items in its production/training queue.
    HasQueue,
    /// Slot visible only when DC/EF has an active construction.
    HasActiveConstruction,
    /// Slot visible only when the tunnel is currently upgrading.
    TunnelIsUpgrading,
    /// Slot visible only when EF has a plate ready to place.
    HasReadyPlate,
}

/// Runtime context for evaluating slot conditions.
pub struct SlotConditionContext {
    pub has_queue: bool,
    pub has_active_construction: bool,
    pub tunnel_is_upgrading: bool,
    pub has_ready_plate: bool,
}

impl SlotConditionContext {
    /// Evaluate whether a condition is met in the current context.
    pub fn evaluate(&self, condition: &SlotCondition) -> bool {
        match condition {
            SlotCondition::HasQueue => self.has_queue,
            SlotCondition::HasActiveConstruction => self.has_active_construction,
            SlotCondition::TunnelIsUpgrading => self.tunnel_is_upgrading,
            SlotCondition::HasReadyPlate => self.has_ready_plate,
        }
    }
}

/// Maps an interface state + grid position to a command button action.
#[derive(Clone, Debug)]
pub struct CommandSlotDef {
    pub state: ObjectInterfaceState,
    pub row: u8,
    pub col: u8,
    pub action: CommandButtonAction,
    pub condition: Option<SlotCondition>,
}

impl CommandSlotDef {
    /// Create a slot with no condition (always visible when state matches).
    pub const fn new(
        state: ObjectInterfaceState,
        row: u8,
        col: u8,
        action: CommandButtonAction,
    ) -> Self {
        Self { state, row, col, action, condition: None }
    }

    /// Create a slot with a visibility condition.
    pub const fn with_condition(
        state: ObjectInterfaceState,
        row: u8,
        col: u8,
        action: CommandButtonAction,
        condition: SlotCondition,
    ) -> Self {
        Self { state, row, col, action, condition: Some(condition) }
    }
}

/// Parameters passed to entity spawn functions.
#[derive(Clone, Debug)]
pub struct SpawnParams {
    pub position: Vec3,
    pub owner: Owner,
    pub player: u8,
    pub rotation: Option<StructureRotation>,
    pub under_construction: bool,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

/// Discriminant for what kind of entity a registry entry represents.
#[derive(Clone, Debug)]
pub enum EntityKind {
    Unit {
        unit_stats: UnitStats,
        attack_data: Option<AttackAttributesData>,
        movement_params: MovementParams,
    },
    Structure {
        structure_stats: StructureStats,
    },
    Resource,
}

// === Traits ===

/// Base trait for all declarative entity definitions.
pub trait EntityDefinition {
    const OBJECT: ObjectEnum;
    fn object_type() -> ObjectType;
    fn base_stats() -> BaseStats;
    fn cost() -> Option<StructureCost> { None }
    fn command_slots() -> &'static [CommandSlotDef] { &[] }
    fn spawn(commands: &mut EntityCommands, params: SpawnParams);
}

/// Extended trait for unit entities.
pub trait UnitDefinition: EntityDefinition {
    fn unit_stats() -> UnitStats;
    fn attack_data() -> Option<AttackAttributesData> { None }
    fn movement_params() -> MovementParams;
}

/// Extended trait for structure entities.
pub trait StructureDefinition: EntityDefinition {
    fn structure_stats() -> StructureStats;
}

// === Registry ===

/// A single entry in the entity registry.
pub struct RegistryEntry {
    pub object_type: ObjectType,
    pub base_stats: BaseStats,
    pub cost: Option<StructureCost>,
    pub command_slots: &'static [CommandSlotDef],
    pub spawn_fn: fn(&mut EntityCommands, SpawnParams),
    pub kind: EntityKind,
}

/// Runtime registry mapping `ObjectEnum` to entity metadata and spawn functions.
#[derive(Resource)]
pub struct EntityRegistry {
    entries: HashMap<ObjectEnum, RegistryEntry>,
}

impl EntityRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Register a unit definition. Panics on duplicate registration.
    pub fn register_unit<T: UnitDefinition>(&mut self) {
        let object = T::OBJECT;
        if self.entries.contains_key(&object) {
            panic!(
                "Duplicate entity registration for {:?}",
                object
            );
        }
        let entry = RegistryEntry {
            object_type: T::object_type(),
            base_stats: T::base_stats(),
            cost: T::cost(),
            command_slots: T::command_slots(),
            spawn_fn: T::spawn,
            kind: EntityKind::Unit {
                unit_stats: T::unit_stats(),
                attack_data: T::attack_data(),
                movement_params: T::movement_params(),
            },
        };
        self.entries.insert(object, entry);
    }

    /// Register a structure definition. Panics on duplicate registration.
    pub fn register_structure<T: StructureDefinition>(&mut self) {
        let object = T::OBJECT;
        if self.entries.contains_key(&object) {
            panic!(
                "Duplicate entity registration for {:?}",
                object
            );
        }
        let entry = RegistryEntry {
            object_type: T::object_type(),
            base_stats: T::base_stats(),
            cost: T::cost(),
            command_slots: T::command_slots(),
            spawn_fn: T::spawn,
            kind: EntityKind::Structure {
                structure_stats: T::structure_stats(),
            },
        };
        self.entries.insert(object, entry);
    }

    /// Register a resource entity definition. Panics on duplicate registration.
    pub fn register_resource<T: EntityDefinition>(&mut self) {
        let object = T::OBJECT;
        if self.entries.contains_key(&object) {
            panic!(
                "Duplicate entity registration for {:?}",
                object
            );
        }
        let entry = RegistryEntry {
            object_type: T::object_type(),
            base_stats: T::base_stats(),
            cost: T::cost(),
            command_slots: T::command_slots(),
            spawn_fn: T::spawn,
            kind: EntityKind::Resource,
        };
        self.entries.insert(object, entry);
    }

    /// Look up a registry entry by ObjectEnum.
    pub fn get(&self, object: &ObjectEnum) -> Option<&RegistryEntry> {
        self.entries.get(object)
    }

    /// Returns the number of registered entities.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if no entities are registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Look up the cost for a given ObjectEnum.
    /// Returns None if the entity is not registered or has no cost.
    pub fn cost(&self, object: &ObjectEnum) -> Option<StructureCost> {
        self.entries.get(object).and_then(|e| e.cost.clone())
    }

    /// Spawn an entity of the given type using the registered spawn function.
    /// Returns the entity id. Panics if `object` is not registered.
    pub fn spawn(&self, object: &ObjectEnum, commands: &mut Commands, params: SpawnParams) -> Entity {
        let entry = self.entries.get(object)
            .unwrap_or_else(|| panic!("No registry entry for {:?}", object));
        let entity = commands.spawn_empty().id();
        (entry.spawn_fn)(&mut commands.entity(entity), params);
        entity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::units::types::movement::{TurnRateMovementParams, MovementParams};

    // Minimal test unit implementing UnitDefinition
    struct TestUnit;

    impl EntityDefinition for TestUnit {
        const OBJECT: ObjectEnum = ObjectEnum::Peacekeeper;

        fn object_type() -> ObjectType {
            ObjectType {
                name: "Test Unit".to_string(),
                size: (1, 1),
                destructible: true,
                sight_range: 5,
                groupable: true,
            }
        }

        fn base_stats() -> BaseStats {
            BaseStats {
                max_hp: 100,
                point_armor: 0,
                full_armor: 0,
                faction: FactionEnum::GlobalDefenseOrdinance,
            }
        }

        fn spawn(_commands: &mut EntityCommands, _params: SpawnParams) {
            // no-op for test
        }
    }

    impl UnitDefinition for TestUnit {
        fn unit_stats() -> UnitStats {
            UnitStats {
                unit_base: UnitBaseEnum::LightInfantry,
                silhouette_width: 1,
                silhouette_height: 1,
                control_cost: 1,
            }
        }

        fn movement_params() -> MovementParams {
            MovementParams::TurnRate(TurnRateMovementParams {
                max_speed: 2.0,
                acceleration: 4.0,
                deceleration: 6.0,
                turn_rate: 3.0,
            })
        }
    }

    // Minimal test structure implementing StructureDefinition
    struct TestStructure;

    impl EntityDefinition for TestStructure {
        const OBJECT: ObjectEnum = ObjectEnum::PowerPlant;

        fn object_type() -> ObjectType {
            ObjectType {
                name: "Test Structure".to_string(),
                size: (2, 2),
                destructible: true,
                sight_range: 3,
                groupable: false,
            }
        }

        fn base_stats() -> BaseStats {
            BaseStats {
                max_hp: 500,
                point_armor: 2,
                full_armor: 0,
                faction: FactionEnum::GlobalDefenseOrdinance,
            }
        }

        fn spawn(_commands: &mut EntityCommands, _params: SpawnParams) {
            // no-op for test
        }
    }

    impl StructureDefinition for TestStructure {
        fn structure_stats() -> StructureStats {
            StructureStats {
                power_value: 20,
                build_radius: 0,
                symmetry: SymmetryTypeEnum::AAAA,
            }
        }
    }

    #[test]
    fn test_register_and_lookup_unit() {
        let mut registry = EntityRegistry::new();
        registry.register_unit::<TestUnit>();

        let entry = registry.get(&ObjectEnum::Peacekeeper);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Test Unit");
        assert_eq!(entry.base_stats.max_hp, 100);
        assert!(matches!(entry.kind, EntityKind::Unit { .. }));
    }

    #[test]
    fn test_register_and_lookup_structure() {
        let mut registry = EntityRegistry::new();
        registry.register_structure::<TestStructure>();

        let entry = registry.get(&ObjectEnum::PowerPlant);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Test Structure");
        assert_eq!(entry.base_stats.max_hp, 500);
        assert!(matches!(entry.kind, EntityKind::Structure { .. }));
    }

    #[test]
    fn test_empty_registry_returns_none() {
        let registry = EntityRegistry::new();
        assert!(registry.get(&ObjectEnum::Peacekeeper).is_none());
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    #[should_panic(expected = "Duplicate entity registration")]
    fn test_duplicate_unit_registration_panics() {
        let mut registry = EntityRegistry::new();
        registry.register_unit::<TestUnit>();
        registry.register_unit::<TestUnit>();
    }

    #[test]
    #[should_panic(expected = "Duplicate entity registration")]
    fn test_duplicate_structure_registration_panics() {
        let mut registry = EntityRegistry::new();
        registry.register_structure::<TestStructure>();
        registry.register_structure::<TestStructure>();
    }

    #[test]
    fn test_entity_kind_is_unit_for_unit_registration() {
        let mut registry = EntityRegistry::new();
        registry.register_unit::<TestUnit>();
        let entry = registry.get(&ObjectEnum::Peacekeeper).unwrap();
        match &entry.kind {
            EntityKind::Unit { unit_stats, attack_data, movement_params } => {
                assert_eq!(unit_stats.unit_base, UnitBaseEnum::LightInfantry);
                assert_eq!(unit_stats.control_cost, 1);
                assert!(attack_data.is_none());
                assert!(matches!(movement_params, MovementParams::TurnRate(_)));
            }
            _ => panic!("Expected EntityKind::Unit"),
        }
    }

    #[test]
    fn test_entity_kind_is_structure_for_structure_registration() {
        let mut registry = EntityRegistry::new();
        registry.register_structure::<TestStructure>();
        let entry = registry.get(&ObjectEnum::PowerPlant).unwrap();
        match &entry.kind {
            EntityKind::Structure { structure_stats } => {
                assert_eq!(structure_stats.power_value, 20);
                assert_eq!(structure_stats.build_radius, 0);
                assert_eq!(structure_stats.symmetry, SymmetryTypeEnum::AAAA);
            }
            _ => panic!("Expected EntityKind::Structure"),
        }
    }

    #[test]
    fn test_multiple_registrations() {
        let mut registry = EntityRegistry::new();
        registry.register_unit::<TestUnit>();
        registry.register_structure::<TestStructure>();
        assert_eq!(registry.len(), 2);
        assert!(registry.get(&ObjectEnum::Peacekeeper).is_some());
        assert!(registry.get(&ObjectEnum::PowerPlant).is_some());
        assert!(registry.get(&ObjectEnum::Barracks).is_none());
    }

    #[test]
    fn test_default_cost_is_none() {
        let mut registry = EntityRegistry::new();
        registry.register_unit::<TestUnit>();
        let entry = registry.get(&ObjectEnum::Peacekeeper).unwrap();
        assert!(entry.cost.is_none());
    }

    #[test]
    fn test_cost_returns_none_for_no_cost_entity() {
        let mut registry = EntityRegistry::new();
        registry.register_unit::<TestUnit>();
        assert!(registry.cost(&ObjectEnum::Peacekeeper).is_none());
    }

    #[test]
    fn test_cost_returns_none_for_unregistered() {
        let registry = EntityRegistry::new();
        assert!(registry.cost(&ObjectEnum::Barracks).is_none());
    }

    #[test]
    fn test_default_command_slots_is_empty() {
        let mut registry = EntityRegistry::new();
        registry.register_unit::<TestUnit>();
        let entry = registry.get(&ObjectEnum::Peacekeeper).unwrap();
        assert!(entry.command_slots.is_empty());
    }

    #[test]
    fn test_spawn_creates_entity_with_registered_spawn_fn() {
        use bevy::ecs::system::RunSystemOnce;

        let mut app = App::new();
        let mut registry = EntityRegistry::new();
        registry.register_unit::<TestUnit>();
        app.insert_resource(registry);
        app.update();

        let entity = app.world_mut().run_system_once(
            |mut commands: Commands, registry: Res<EntityRegistry>| {
                registry.spawn(&ObjectEnum::Peacekeeper, &mut commands, SpawnParams {
                    position: Vec3::new(1.0, 2.0, 3.0),
                    owner: Owner::player(0),
                    player: 0,
                    rotation: None,
                    under_construction: false,
                    flip_horizontal: false,
                    flip_vertical: false,
                })
            },
        ).unwrap();
        app.world_mut().flush();

        // Entity should exist
        assert!(app.world().get_entity(entity).is_ok());
    }

    #[test]
    #[should_panic(expected = "No registry entry for")]
    fn test_spawn_panics_for_unregistered() {
        use bevy::ecs::system::RunSystemOnce;

        let mut app = App::new();
        let registry = EntityRegistry::new();
        app.insert_resource(registry);
        app.update();

        app.world_mut().run_system_once(
            |mut commands: Commands, registry: Res<EntityRegistry>| {
                registry.spawn(&ObjectEnum::Barracks, &mut commands, SpawnParams {
                    position: Vec3::ZERO,
                    owner: Owner::player(0),
                    player: 0,
                    rotation: None,
                    under_construction: false,
                    flip_horizontal: false,
                    flip_vertical: false,
                })
            },
        ).unwrap();
    }

    #[test]
    fn test_spawn_params_flip_fields_default() {
        let params = SpawnParams {
            position: Vec3::ZERO,
            owner: Owner::player(0),
            player: 0,
            rotation: None,
            under_construction: false,
            flip_horizontal: false,
            flip_vertical: false,
        };
        assert!(!params.flip_horizontal);
        assert!(!params.flip_vertical);
    }
}
