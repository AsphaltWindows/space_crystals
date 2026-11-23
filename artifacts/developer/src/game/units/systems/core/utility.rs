use bevy::prelude::*;
use crate::types::*;
use crate::game::combat::types::{AttackState, Silhouette};
use crate::game::combat::utils::{create_turret_for_unit, spawn_turret_visual};
use crate::game::units::types::*;
use crate::game::units::types::movement::{TurnRateMovementParams, FixedTurnRadiusMovementParams, SpeedTurnRadiusMovementParams, DragMovementParams, GliderMovementParams};
use crate::game::units::utils::{world_to_grid, create_attack_capability};
use crate::game::entities::{EntityRegistry, SpawnParams};
use crate::game::entities::utils::{insert_entity_visuals, grid_to_spawn_position};
use crate::game::combat::types::{Armor, SeparationRadius};
use crate::game::types::ObjectInstance;

/// Spawn test units on the map -- GDO player units are now proper Peacekeepers
#[allow(dead_code)]
pub fn spawn_test_units(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    registry: Res<EntityRegistry>,
) {
    // Spawn GDO Peacekeepers for Player 0 (near DC at 30,30)
    let peacekeeper_positions = [(28, 32), (29, 32), (30, 32), (31, 32)];
    let gdo_owner = Owner::player(0);
    for (grid_x, grid_z) in peacekeeper_positions {
        let pos = grid_to_spawn_position(grid_x, grid_z, 1.0, 1.0);
        let entity = registry.spawn(&ObjectEnum::Peacekeeper, &mut commands, SpawnParams {
            position: pos,
            owner: gdo_owner,
            player: 0,
            rotation: None,
            under_construction: false,
            flip_horizontal: false,
            flip_vertical: false,
        });
        insert_entity_visuals(&mut commands, entity, &ObjectEnum::Peacekeeper, &gdo_owner, &mut meshes, &mut materials);
    }

    // Spawn Syndicate infantry (Player 1) -- Agent and Guard using registry
    let syn_owner = Owner::player(1);
    for (grid_x, grid_z, obj) in [(33, 32, ObjectEnum::SyndicateAgent), (34, 32, ObjectEnum::SyndicateGuard)] {
        let pos = grid_to_spawn_position(grid_x, grid_z, 1.0, 1.0);
        let entity = registry.spawn(&obj, &mut commands, SpawnParams {
            position: pos,
            owner: syn_owner,
            player: 1,
            rotation: None,
            under_construction: false,
            flip_horizontal: false,
            flip_vertical: false,
        });
        insert_entity_visuals(&mut commands, entity, &obj, &syn_owner, &mut meshes, &mut materials);
    }

    // Spawn test units for remaining UnitBaseEnum variants (Player 1).
    // TODO: These use ObjectEnum::Peacekeeper as placeholder -- need proper ObjectEnum variants.
    let vehicle_test_units: [(i32, i32, &str, UnitBaseEnum); 6] = [
        (36, 32, "Wheeled APC",    UnitBaseEnum::WheeledVehicle),
        (37, 32, "Heavy Tank",     UnitBaseEnum::TrackedVehicle),
        (38, 32, "Hover Vehicle",  UnitBaseEnum::HoverVehicle),
        (39, 32, "Battle Mech",    UnitBaseEnum::Mech),
        (40, 32, "Glider",         UnitBaseEnum::Glider),
        (41, 32, "Drill Unit",     UnitBaseEnum::DrillUnit),
    ];

    let owner = Owner::player(1);

    for (grid_x, grid_z, unit_name, unit_base) in vehicle_test_units {
        let world_x = (grid_x as f32 - 32.0) + 0.5;
        let world_z = (grid_z as f32 - 32.0) + 0.5;
        let base_data = unit_base.data();

        let mesh = meshes.add(Cuboid::new(0.5, 0.5, 0.5));
        let material = materials.add(StandardMaterial {
            base_color: owner.color(),
            ..default()
        });

        let mut entity_commands = commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(world_x, 0.5, world_z),
            Unit,
            // TODO: Use correct ObjectEnum variant when defined for each vehicle/mech/glider type
            ObjectInstance::destructible(ObjectEnum::Peacekeeper, 150.0),
            owner,
            UnitType { name: unit_name.to_string() },
            Selectable,
            SelectionBounds::unit(),
            GridPosition { x: grid_x, z: grid_z },
            unit_base,
            base_data.domain,
            MovementSpeed(5.0),  // Placeholder until ObjectEnum-based stats exist
            RotationSpeed(2.0),  // Placeholder until ObjectEnum-based stats exist
        ));

        entity_commands.insert((
            Velocity(Vec3::ZERO),
            create_attack_capability(&unit_base),
            AttackState::default(),
            UnitCommand::Idle,
        ));

        entity_commands.insert((
            CommandQueue::new(),
            BaseCommandState::default(),
            BaseBehaviorState::default(),
            LocomotionChannel::default(),
            OrientationChannel::default(),
            UnitControlCost(1), // TODO: Use actual cost from unit type data when defined
            SightRange(5),      // Placeholder until ObjectEnum-based sight_range exists
        ));

        // Armor and silhouette
        entity_commands.insert((
            Armor {
                point_armor: 5.0, // Placeholder
                full_armor: 3.0,  // Placeholder
                directional_armor: base_data.directional_armor,
            },
            Silhouette {
                width: 0.5,  // Placeholder
                height: 0.5, // Placeholder
            },
        ));

        // Movement params -- correct component per movement model
        match unit_base {
            UnitBaseEnum::WheeledVehicle => {
                entity_commands.insert(FixedTurnRadiusMovementParams {
                    minimum_turn_radius: 4.0,
                    forward_acceleration: 5.0,
                    forward_max_speed: 10.0,
                    reverse_acceleration: 3.0,
                    reverse_max_speed: 5.0,
                    deceleration: 8.0,
                });
            }
            UnitBaseEnum::TrackedVehicle => {
                entity_commands.insert(SpeedTurnRadiusMovementParams {
                    speed_to_turn_radius_ratio: 0.5,
                    acceleration: 5.0,
                    deceleration: 10.0,
                    max_speed: 12.0,
                });
            }
            UnitBaseEnum::HoverVehicle => {
                entity_commands.insert(DragMovementParams {
                    forward_acceleration: 6.0,
                    non_forward_acceleration: 4.0,
                    drag_ratio: 2.0,
                    turn_rate: 2.5,
                });
            }
            UnitBaseEnum::Mech => {
                entity_commands.insert(TurnRateMovementParams {
                    turn_rate: 3.0,
                    acceleration: 5.0,
                    deceleration: 10.0,
                    max_speed: 8.0,
                });
            }
            UnitBaseEnum::Glider => {
                entity_commands.insert(GliderMovementParams {
                    idle_speed: 5.0,
                    max_speed: 15.0,
                    acceleration: 3.0,
                    deceleration: 6.0,
                    max_centripetal_acceleration: 10.0,
                });
                // Air units need separation radius
                entity_commands.insert(SeparationRadius(1.25));
            }
            UnitBaseEnum::DrillUnit => {
                entity_commands.insert(SpeedTurnRadiusMovementParams {
                    speed_to_turn_radius_ratio: 0.5,
                    acceleration: 5.0,
                    deceleration: 10.0,
                    max_speed: 12.0,
                });
            }
            _ => {} // Infantry handled by spawn helpers above
        }

        // Turret components
        let has_turret = base_data.has_turret;
        if has_turret {
            entity_commands.insert((
                TurretCommandState::default(),
                TurretBehaviorState::default(),
                TurretOrientationChannel::default(),
                TurretAttackChannel::default(),
            ));
        } else {
            entity_commands.insert(BaseAttackChannel::default());
        }

        let entity_id = entity_commands.id();

        if let Some(turret) = create_turret_for_unit(&unit_base) {
            entity_commands.insert(turret);
        }

        spawn_turret_visual(&mut commands, entity_id, &mut meshes, &mut materials, &unit_base, owner.color());
    }

    info!("Spawned 4 Peacekeepers (Player 0), 2 Syndicate infantry, and 6 vehicle/mech/glider test units (Player 1)");
}

/// System to sync GridPosition from Transform for all units each frame.
/// Movement systems update Transform but not GridPosition; this system keeps
/// GridPosition accurate so fog-of-war, combat range checks, and other
/// grid-based queries work correctly for moving units.
pub fn grid_position_sync_system(
    mut units: Query<(&Transform, &mut GridPosition), With<Unit>>,
) {
    for (transform, mut grid_pos) in &mut units {
        let new_pos = world_to_grid(transform.translation);
        if grid_pos.x != new_pos.x || grid_pos.z != new_pos.z {
            grid_pos.x = new_pos.x;
            grid_pos.z = new_pos.z;
        }
    }
}

/// System to display unit info when selected
pub fn unit_selection_display(
    units: Query<
        (&UnitType, &ObjectInstance, &Owner),
        (With<Unit>, Added<Selected>)
    >,
) {
    for (unit_type, obj, owner) in &units {
        info!(
            "Unit selected: {} | Health: {}/{} | Owner: {:?}",
            unit_type.name,
            obj.hp.unwrap_or(0.0),
            obj.max_hp.unwrap_or(0.0),
            owner
        );
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    /// Helper: spawn a Unit entity with Transform and GridPosition
    pub(crate) fn spawn_unit_at(world: &mut World, world_x: f32, world_z: f32, grid_x: i32, grid_z: i32) -> Entity {
        world.spawn((
            Unit,
            Transform::from_xyz(world_x, 0.5, world_z),
            GridPosition { x: grid_x, z: grid_z },
        )).id()
    }

    #[test]
    fn grid_position_sync_updates_moved_unit() {
        let mut world = World::new();
        // Spawn at grid (32,32) = world (0.5, 0.5), then move transform to world (5.5, 0.5, 3.5) = grid (37, 35)
        let entity = spawn_unit_at(&mut world, 5.5, 3.5, 32, 32);
        world.run_system_once(grid_position_sync_system).unwrap();
        let gp = world.entity(entity).get::<GridPosition>().unwrap();
        assert_eq!(gp.x, 37);
        assert_eq!(gp.z, 35);
    }

    #[test]
    fn grid_position_sync_no_change_when_stationary() {
        let mut world = World::new();
        // world (0.5, 0.5) maps to grid (32, 32) -- already correct
        let entity = spawn_unit_at(&mut world, 0.5, 0.5, 32, 32);
        world.run_system_once(grid_position_sync_system).unwrap();
        let gp = world.entity(entity).get::<GridPosition>().unwrap();
        assert_eq!(gp.x, 32);
        assert_eq!(gp.z, 32);
    }

    #[test]
    fn grid_position_sync_negative_world_coords() {
        let mut world = World::new();
        // world (-31.5, -31.5) = grid (0, 0)
        let entity = spawn_unit_at(&mut world, -31.5, -31.5, 32, 32);
        world.run_system_once(grid_position_sync_system).unwrap();
        let gp = world.entity(entity).get::<GridPosition>().unwrap();
        assert_eq!(gp.x, 0);
        assert_eq!(gp.z, 0);
    }

    #[test]
    fn grid_position_sync_multiple_units() {
        let mut world = World::new();
        let e1 = spawn_unit_at(&mut world, 0.5, 0.5, 32, 32); // stays at (32,32)
        let e2 = spawn_unit_at(&mut world, 10.5, -5.5, 0, 0); // should become (42, 26)
        world.run_system_once(grid_position_sync_system).unwrap();

        let gp1 = world.entity(e1).get::<GridPosition>().unwrap();
        assert_eq!((gp1.x, gp1.z), (32, 32));

        let gp2 = world.entity(e2).get::<GridPosition>().unwrap();
        assert_eq!((gp2.x, gp2.z), (42, 26));
    }

    #[test]
    fn grid_position_sync_ignores_non_unit_entities() {
        let mut world = World::new();
        // Entity without Unit marker -- should NOT be synced
        let entity = world.spawn((
            Transform::from_xyz(10.0, 0.5, 10.0),
            GridPosition { x: 5, z: 5 },
        )).id();
        world.run_system_once(grid_position_sync_system).unwrap();
        let gp = world.entity(entity).get::<GridPosition>().unwrap();
        // Should remain unchanged
        assert_eq!(gp.x, 5);
        assert_eq!(gp.z, 5);
    }

    #[test]
    fn grid_position_sync_sub_tile_movement_no_update() {
        let mut world = World::new();
        // world (0.1, 0.1) still maps to grid (32, 32) via floor
        let entity = spawn_unit_at(&mut world, 0.1, 0.1, 32, 32);
        world.run_system_once(grid_position_sync_system).unwrap();
        let gp = world.entity(entity).get::<GridPosition>().unwrap();
        assert_eq!((gp.x, gp.z), (32, 32));
    }

    #[test]
    fn grid_position_sync_tile_boundary_crossing() {
        let mut world = World::new();
        // world (1.0, 0.5) = grid (33, 32) -- just crossed tile boundary
        let entity = spawn_unit_at(&mut world, 1.0, 0.5, 32, 32);
        world.run_system_once(grid_position_sync_system).unwrap();
        let gp = world.entity(entity).get::<GridPosition>().unwrap();
        assert_eq!((gp.x, gp.z), (33, 32));
    }

    #[test]
    fn grid_position_sync_change_detection_guard() {
        let mut world = World::new();
        // Position already matches -- change detection should NOT fire
        // world (0.5, 0.5) = grid (32, 32)
        let entity = spawn_unit_at(&mut world, 0.5, 0.5, 32, 32);

        // Clear change ticks by running a frame
        world.increment_change_tick();
        world.clear_trackers();

        world.run_system_once(grid_position_sync_system).unwrap();

        // GridPosition should not have been mutated (change detection guard)
        // We verify the values are still correct
        let gp = world.entity(entity).get::<GridPosition>().unwrap();
        assert_eq!((gp.x, gp.z), (32, 32));
    }

    #[test]
    fn grid_position_sync_uses_world_to_grid_correctly() {
        // Verify the system produces the same result as world_to_grid
        let mut world = World::new();
        let test_positions = [
            (0.0, 0.0),    // grid center
            (-32.0, -32.0), // corner
            (31.0, 31.0),  // opposite corner area
            (15.5, -10.5), // arbitrary
        ];

        let entities: Vec<Entity> = test_positions
            .iter()
            .map(|&(x, z)| spawn_unit_at(&mut world, x, z, 0, 0))
            .collect();

        world.run_system_once(grid_position_sync_system).unwrap();

        for (i, &(x, z)) in test_positions.iter().enumerate() {
            let expected = world_to_grid(Vec3::new(x, 0.5, z));
            let gp = world.entity(entities[i]).get::<GridPosition>().unwrap();
            assert_eq!(
                (gp.x, gp.z), (expected.x, expected.z),
                "Mismatch at world ({}, {})", x, z
            );
        }
    }
}
