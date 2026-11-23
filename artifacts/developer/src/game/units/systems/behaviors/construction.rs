#![allow(dead_code)]
use bevy::prelude::*;
use crate::game::units::types::state::behavior::{
    BaseBehaviorState, BuildingStructureBehavior, BuildingTunnelBehavior, BuildTunnelPhase,
    LocomotionChannel, OrientationChannel,
};
use crate::game::units::types::state::commands::UnitCommand;
use crate::game::entities::factions::syndicate::units::agent::AGENT_TUNNEL_BUILD_FRAMES;
use crate::game::types::{ConstructionHP, ObjectInstance, StructureInstance, TunnelState, CultsConstructionState};
use crate::game::types::structures::tunnel_construction_cost;
use crate::game::types::factions::{Player, SyndicatePlayerResources};
use crate::game::world::types::{Tile, TilePreset};
use crate::game::world::utils::{can_worker_place_structure, world_to_grid};
use crate::game::entities::{EntityRegistry, SpawnParams};
use crate::game::entities::utils::{insert_entity_visuals, grid_to_spawn_position};
use crate::types::{GridPosition, Owner, Unit, ObjectEnum};

const BUILD_ARRIVAL_THRESHOLD: f32 = 1.0;

/// BuildingStructure behavior system.
///
/// Units with the `BuildingStructureBehavior` marker move toward the target build location.
/// On arrival, validates the placement with `can_worker_place_structure()`:
/// - If valid: removes the marker and sets UnitCommand::Idle (structure spawning will be
///   handled by a separate system that detects the arrival+validation).
/// - If invalid: removes the marker and sets UnitCommand::Idle (build cancelled).
///
/// Schedule: Update (same as other behavior systems)
pub fn building_behavior_system(
    mut commands: Commands,
    mut units: Query<(
        Entity,
        &Transform,
        &mut BuildingStructureBehavior,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &mut UnitCommand,
        &mut BaseBehaviorState,
    )>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    structures: Query<(&GridPosition, &StructureInstance, &crate::game::types::objects::ObjectInstance)>,
) {
    for (entity, transform, mut building, mut locomotion, mut orientation, mut command, mut behavior) in &mut units {
        let unit_pos = transform.translation;
        let target = building.target_location;
        let distance = Vec3::new(unit_pos.x - target.x, 0.0, unit_pos.z - target.z).length();

        if distance < BUILD_ARRIVAL_THRESHOLD && !building.arrived {
            // Mark as arrived
            building.arrived = true;

            // Validate placement
            let obj_type = building.object_to_build.object_type();
            let (size_x, size_z) = obj_type.size;
            let (grid_x, grid_z) = world_to_grid(target, 1.0);

            let valid = can_worker_place_structure(
                grid_x, grid_z, size_x, size_z,
                &tiles, &structures,
            );

            if valid.is_ok() {
                // Validation passed — mark command as Idle.
                // The actual structure spawning requires meshes/materials which are
                // not available in this system. A higher-level system will detect
                // BuildingStructureBehavior with arrived=true and valid placement
                // to spawn the structure. For now, we signal completion.
                *command = UnitCommand::Idle;
                *locomotion = LocomotionChannel::Stationary;
                *orientation = OrientationChannel::Maintaining;
                *behavior = BaseBehaviorState::None;
                // Keep the marker with arrived=true so a spawning system can read it
                // The spawning system will remove the marker after spawning.
                // For this task, we remove it since structure spawning integration
                // is deferred to the agent_object_interface_state task.
                commands.entity(entity).remove::<BuildingStructureBehavior>();
            } else {
                // Validation failed — cancel build, idle
                *command = UnitCommand::Idle;
                *locomotion = LocomotionChannel::Stationary;
                *orientation = OrientationChannel::Maintaining;
                *behavior = BaseBehaviorState::None;
                commands.entity(entity).remove::<BuildingStructureBehavior>();
            }
        } else if !building.arrived {
            // Not yet arrived — move toward target
            *locomotion = LocomotionChannel::Moving(vec![target]);
            *orientation = OrientationChannel::Turning(target);
        }
    }
}


pub fn building_tunnel_behavior_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut units: Query<(
        Entity,
        &Transform,
        &Owner,
        &mut BuildingTunnelBehavior,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &mut UnitCommand,
        &mut BaseBehaviorState,
        &mut Visibility,
    )>,
    tunnel_query: Query<(Entity, &Owner), With<TunnelState>>,
    tunnel_hp_query: Query<&ObjectInstance, With<ConstructionHP>>,
    mut syndicate_players: Query<(&Player, &mut SyndicatePlayerResources)>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    structures: Query<(&GridPosition, &StructureInstance, &ObjectInstance)>,
    registry: Res<EntityRegistry>,
) {
    // Single-Agent construction enforcement: collect locations where construction is in progress
    let constructing_locations: Vec<(Entity, Vec3)> = units.iter()
        .filter_map(|(e, _, _, b, ..)| {
            if matches!(b.phase, BuildTunnelPhase::Constructing { .. }) {
                Some((e, b.target_location))
            } else {
                None
            }
        })
        .collect();

    for (entity, transform, owner, mut building, mut locomotion, mut orientation, mut command, mut behavior, mut visibility) in &mut units {
        match building.phase.clone() {
            BuildTunnelPhase::MovingToSite => {
                let target = building.target_location;
                let unit_pos = transform.translation;
                let distance = Vec3::new(unit_pos.x - target.x, 0.0, unit_pos.z - target.z).length();

                if distance < BUILD_ARRIVAL_THRESHOLD {
                    // Validate placement — tiles must be buildable and unoccupied
                    let (grid_x_val, grid_z_val) = world_to_grid(target, 1.0);
                    let (size_x, size_z) = ObjectEnum::Tunnel.object_type().size;
                    let valid = can_worker_place_structure(grid_x_val, grid_z_val, size_x, size_z, &tiles, &structures);
                    if valid.is_err() {
                        info!("Agent: Build tunnel rejected — placement invalid at ({}, {}): {}", grid_x_val, grid_z_val, valid.unwrap_err());
                        *command = UnitCommand::Idle;
                        *behavior = BaseBehaviorState::None;
                        *locomotion = LocomotionChannel::Stationary;
                        *orientation = OrientationChannel::Maintaining;
                        commands.entity(entity).remove::<BuildingTunnelBehavior>();
                        continue;
                    }

                    // Single-Agent construction enforcement: reject if another agent
                    // is already constructing at this location
                    let location_taken = constructing_locations.iter().any(|(other_e, loc)| {
                        *other_e != entity
                            && (loc.x - target.x).abs() < 1.0
                            && (loc.z - target.z).abs() < 1.0
                    });
                    if location_taken {
                        info!("Agent: Build rejected — another Agent is already constructing at ({:.1}, {:.1})", target.x, target.z);
                        *command = UnitCommand::Idle;
                        *behavior = BaseBehaviorState::None;
                        *locomotion = LocomotionChannel::Stationary;
                        *orientation = OrientationChannel::Maintaining;
                        commands.entity(entity).remove::<BuildingTunnelBehavior>();
                        continue;
                    }
                    // Arrived at build site — count existing tunnels and compute cost
                    let agent_player = match owner.player_number() {
                        Some(p) => p,
                        None => {
                            // No player — cancel
                            *command = UnitCommand::Idle;
                            *behavior = BaseBehaviorState::None;
                            *locomotion = LocomotionChannel::Stationary;
                            *orientation = OrientationChannel::Maintaining;
                            commands.entity(entity).remove::<BuildingTunnelBehavior>();
                            continue;
                        }
                    };

                    let existing_count = tunnel_query.iter()
                        .filter(|(_, o)| o.player_number() == Some(agent_player))
                        .count();
                    let cost = tunnel_construction_cost(existing_count as u32);

                    // Check and deduct supplies
                    let mut can_afford = false;
                    for (player, mut resources) in &mut syndicate_players {
                        if player.player_number == agent_player {
                            if resources.supplies >= cost as i32 {
                                resources.supplies -= cost as i32;
                                can_afford = true;
                            }
                            break;
                        }
                    }

                    if !can_afford {
                        // Insufficient funds — cancel build
                        *command = UnitCommand::Idle;
                        *behavior = BaseBehaviorState::None;
                        *locomotion = LocomotionChannel::Stationary;
                        *orientation = OrientationChannel::Maintaining;
                        commands.entity(entity).remove::<BuildingTunnelBehavior>();
                        continue;
                    }

                    // Spawn partially-built tunnel via registry
                    let (grid_x, grid_z) = world_to_grid(target, 1.0);
                    let tunnel_pos = grid_to_spawn_position(grid_x, grid_z, 4.0, 4.0);
                    let tunnel_entity = registry.spawn(&ObjectEnum::Tunnel, &mut commands, SpawnParams {
                        position: tunnel_pos,
                        owner: owner.clone(),
                        player: owner.player_number().unwrap_or(0),
                        rotation: None,
                        under_construction: true,
                        flip_horizontal: false,
                        flip_vertical: false,
                    });
                    insert_entity_visuals(&mut commands, tunnel_entity, &ObjectEnum::Tunnel, &owner, &mut meshes, &mut materials);
                    // Override the default ConstructionHP with actual build frames
                    commands.entity(tunnel_entity).insert(ConstructionHP::new(AGENT_TUNNEL_BUILD_FRAMES));

                    // Hide the Agent (untargetable during construction)
                    *visibility = Visibility::Hidden;
                    *locomotion = LocomotionChannel::Stationary;
                    *orientation = OrientationChannel::Maintaining;

                    // Transition to Constructing phase
                    building.phase = BuildTunnelPhase::Constructing {
                        tunnel_entity,
                        frames_elapsed: 0,
                    };
                } else {
                    // Not yet arrived — move toward target
                    *locomotion = LocomotionChannel::Moving(vec![target]);
                    *orientation = OrientationChannel::Turning(target);
                }
            }
            BuildTunnelPhase::Constructing { tunnel_entity, frames_elapsed } => {
                *locomotion = LocomotionChannel::Stationary;

                // Check if tunnel still exists
                if tunnel_hp_query.get(tunnel_entity).is_err() {
                    // Tunnel was destroyed OR construction completed (ConstructionHP removed)
                    // Check if the tunnel entity itself still exists by trying to get anything
                    if commands.get_entity(tunnel_entity).is_ok() && frames_elapsed >= AGENT_TUNNEL_BUILD_FRAMES {
                        // Construction complete — tunnel still alive, ConstructionHP was removed
                        // Agent enters the tunnel network (despawn)
                        commands.entity(entity).despawn();
                    } else {
                        // Tunnel was destroyed — Agent emerges
                        *visibility = Visibility::Inherited;
                        *command = UnitCommand::Idle;
                        *behavior = BaseBehaviorState::None;
                        *locomotion = LocomotionChannel::Stationary;
                        *orientation = OrientationChannel::Maintaining;
                        commands.entity(entity).remove::<BuildingTunnelBehavior>();
                    }
                    continue;
                }

                // Tunnel is alive and still under construction — increment frame count
                let new_frames = frames_elapsed + 1;

                if new_frames >= AGENT_TUNNEL_BUILD_FRAMES {
                    // Construction complete — ConstructionHP system will remove the component
                    // Agent enters the tunnel network
                    commands.entity(entity).despawn();
                } else {
                    building.phase = BuildTunnelPhase::Constructing {
                        tunnel_entity,
                        frames_elapsed: new_frames,
                    };
                }
            }
        }
    }
}


// =====================================================
// CULTS RECRUIT CONSTRUCTION BEHAVIOR
// =====================================================

/// Distance threshold for a Recruit to be considered "arrived" at a construction site
const RECRUIT_CONSTRUCTION_ARRIVAL_THRESHOLD: f32 = 2.0;

/// System that handles Cults Recruits walking toward and entering buildings under construction.
/// When a Recruit with UnitCommand::ConstructBuilding reaches the target building, it is
/// added to the building's CultsConstructionState.assigned_recruits, hidden, and set to Idle.
pub fn cults_recruit_enter_construction_system(
    mut recruits: Query<(
        Entity,
        &Transform,
        &mut UnitCommand,
        &mut Visibility,
    ), With<Unit>>,
    mut buildings: Query<(&Transform, &mut CultsConstructionState), With<ConstructionHP>>,
) {
    // Collect recruit actions first to avoid borrow conflicts
    let mut recruit_actions: Vec<(Entity, Entity)> = Vec::new(); // (recruit_entity, building_entity)
    let mut recruit_movements: Vec<(Entity, Vec3)> = Vec::new(); // (recruit_entity, target_pos)

    for (recruit_entity, transform, command, _visibility) in recruits.iter() {
        if let UnitCommand::ConstructBuilding(target_building) = *command {
            // Check if the target building still exists and has CultsConstructionState
            if let Ok((building_transform, _)) = buildings.get(target_building) {
                let target_pos = building_transform.translation;
                let unit_pos = transform.translation;
                let distance = Vec3::new(unit_pos.x - target_pos.x, 0.0, unit_pos.z - target_pos.z).length();

                if distance < RECRUIT_CONSTRUCTION_ARRIVAL_THRESHOLD {
                    recruit_actions.push((recruit_entity, target_building));
                } else {
                    recruit_movements.push((recruit_entity, target_pos));
                }
            } else {
                // Target building no longer exists or no longer under construction — cancel
                recruit_movements.push((recruit_entity, Vec3::ZERO)); // sentinel for cancel
            }
        }
    }

    // Apply arrivals: add recruit to building and hide
    for (recruit_entity, building_entity) in recruit_actions {
        if let Ok((_, mut cults_state)) = buildings.get_mut(building_entity) {
            cults_state.assigned_recruits.push(recruit_entity);
        }
        if let Ok((_, _, mut command, mut visibility)) = recruits.get_mut(recruit_entity) {
            *command = UnitCommand::Idle;
            *visibility = Visibility::Hidden;
        }
    }

    // Apply movements: Recruits that haven't arrived yet need to keep moving.
    // Since Recruits are minimal stubs without LocomotionChannel, use simple direct movement
    // toward the target. The actual position update is handled by the movement pipeline.
    // For now, just cancel if the building is gone.
    for (recruit_entity, target_pos) in recruit_movements {
        if target_pos == Vec3::ZERO {
            // Building gone — cancel command
            if let Ok((_, _, mut command, _)) = recruits.get_mut(recruit_entity) {
                *command = UnitCommand::Idle;
            }
        }
        // If target_pos is a real position, the Recruit keeps its ConstructBuilding command.
        // Movement toward the target is handled by the movement pipeline if the Recruit
        // has movement components. For now, Recruits are stubs — actual movement will be
        // added when Recruit stats are formalized.
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::types::{GridPosition, Owner};
    use crate::game::types::{ObjectInstance, StructureInstance, ConstructionHP, TunnelState, CultsConstructionState};
    use crate::game::entities::registry::build_registry;
    
    use crate::game::types::factions::{Player, SyndicatePlayerResources};
    use crate::game::world::types::{Tile, TilePreset};
    
    use crate::game::units::types::state::behavior::*;
    use crate::game::units::types::state::commands::UnitCommand;
    
    
    

    // === BuildingStructureBehavior system tests ===

    #[test]
    fn build_arrival_threshold_is_positive() {
        assert!(BUILD_ARRIVAL_THRESHOLD > 0.0);
    }

    #[test]
    fn build_arrival_threshold_is_one_gu() {
        assert!((BUILD_ARRIVAL_THRESHOLD - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn building_behavior_not_arrived_sets_moving() {
        // When far from target, system should set Moving locomotion
        let mut world = World::new();
        let target = Vec3::new(10.0, 0.0, 10.0);
        let entity = world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0), // Far from target
            BuildingStructureBehavior::new(target, ObjectEnum::Tunnel),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::Build { target, object: ObjectEnum::Tunnel },
            BaseBehaviorState::None,
        )).id();

        world.run_system_once(building_behavior_system).unwrap();

        let locomotion = world.entity(entity).get::<LocomotionChannel>().unwrap();
        assert!(matches!(locomotion, LocomotionChannel::Moving(_)));
        let orientation = world.entity(entity).get::<OrientationChannel>().unwrap();
        assert!(matches!(orientation, OrientationChannel::Turning(_)));
        // Should still have the marker
        assert!(world.entity(entity).get::<BuildingStructureBehavior>().is_some());
    }

    #[test]
    fn building_behavior_arrived_no_tiles_cancels() {
        // At target but no tiles — validation fails → idle
        let mut world = World::new();
        let target = Vec3::new(0.5, 0.0, 0.5); // At grid ~(32,32)
        let entity = world.spawn((
            Transform::from_xyz(0.5, 0.0, 0.5), // At target
            BuildingStructureBehavior::new(target, ObjectEnum::Tunnel),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::Build { target, object: ObjectEnum::Tunnel },
            BaseBehaviorState::None,
        )).id();

        world.run_system_once(building_behavior_system).unwrap();

        let command = world.entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        // Marker should be removed
        assert!(world.entity(entity).get::<BuildingStructureBehavior>().is_none());
    }

    #[test]
    fn building_behavior_arrived_valid_tiles_completes() {
        // At target with valid tiles — validation passes → idle (structure spawning is separate)
        let mut world = World::new();
        let target = Vec3::new(0.5, 0.0, 0.5); // Maps to grid (32, 32)
        let entity = world.spawn((
            Transform::from_xyz(0.5, 0.0, 0.5),
            BuildingStructureBehavior::new(target, ObjectEnum::Tunnel),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::Build { target, object: ObjectEnum::Tunnel },
            BaseBehaviorState::None,
        )).id();

        // Spawn buildable tiles for 4x4 Tunnel at grid (32, 32)
        for dx in 0..4 {
            for dz in 0..4 {
                world.spawn((
                    GridPosition { x: 32 + dx, z: 32 + dz },
                    TilePreset {
                        value: crate::game::world::types::TilePresetEnum::Plane,
                        name: "Plane".to_string(),
                        texture: None,
                        buildable: true,
                        traversible: true,
                        rugged: false,
                        drillable: false,
                        recruitable: false,
                    },
                    Tile,
                ));
            }
        }

        world.run_system_once(building_behavior_system).unwrap();

        let command = world.entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        let locomotion = world.entity(entity).get::<LocomotionChannel>().unwrap();
        assert!(matches!(locomotion, LocomotionChannel::Stationary));
        assert!(world.entity(entity).get::<BuildingStructureBehavior>().is_none());
    }

    #[test]
    fn building_behavior_arrived_structure_overlap_cancels() {
        // At target with valid tiles but structure overlap → cancels
        let mut world = World::new();
        let target = Vec3::new(0.5, 0.0, 0.5); // Maps to grid (32, 32)
        let entity = world.spawn((
            Transform::from_xyz(0.5, 0.0, 0.5),
            BuildingStructureBehavior::new(target, ObjectEnum::Tunnel),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::Build { target, object: ObjectEnum::Tunnel },
            BaseBehaviorState::None,
        )).id();

        // Spawn buildable tiles
        for dx in 0..4 {
            for dz in 0..4 {
                world.spawn((
                    GridPosition { x: 32 + dx, z: 32 + dz },
                    TilePreset {
                        value: crate::game::world::types::TilePresetEnum::Plane,
                        name: "Plane".to_string(),
                        texture: None,
                        buildable: true,
                        traversible: true,
                        rugged: false,
                        drillable: false,
                        recruitable: false,
                    },
                    Tile,
                ));
            }
        }

        // Spawn an existing structure overlapping the footprint
        world.spawn((
            GridPosition { x: 33, z: 33 },
            StructureInstance::default(),
        ));

        world.run_system_once(building_behavior_system).unwrap();

        let command = world.entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        assert!(world.entity(entity).get::<BuildingStructureBehavior>().is_none());
    }

    #[test]
    fn building_behavior_already_arrived_no_double_process() {
        // If arrived is already true, system should not re-process
        let mut world = World::new();
        let target = Vec3::new(0.5, 0.0, 0.5);
        let mut behavior = BuildingStructureBehavior::new(target, ObjectEnum::Tunnel);
        behavior.arrived = true; // Already processed

        let entity = world.spawn((
            Transform::from_xyz(0.5, 0.0, 0.5),
            behavior,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::Idle,
            BaseBehaviorState::None,
        )).id();

        world.run_system_once(building_behavior_system).unwrap();

        // Should remain idle, marker should still be there (no re-processing)
        let command = world.entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
    }

    // === BuildingTunnelBehavior tests ===

    #[test]
    fn building_tunnel_constant_is_480_frames() {
        assert_eq!(AGENT_TUNNEL_BUILD_FRAMES, 480);
    }

    #[test]
    fn building_tunnel_behavior_new_defaults() {
        let target = Vec3::new(5.0, 0.0, 5.0);
        let behavior = BuildingTunnelBehavior::new(target);
        assert_eq!(behavior.target_location, target);
        assert_eq!(behavior.phase, BuildTunnelPhase::MovingToSite);
        assert!(behavior.path.is_empty());
        assert_eq!(behavior.path_index, 0);
    }

    #[test]
    fn building_tunnel_behavior_is_component() {
        let mut world = World::new();
        let target = Vec3::new(10.0, 0.0, 10.0);
        let entity = world.spawn(BuildingTunnelBehavior::new(target)).id();
        let b = world.entity(entity).get::<BuildingTunnelBehavior>().unwrap();
        assert_eq!(b.target_location, target);
    }

    #[test]
    fn build_tunnel_phase_moving_to_site_eq() {
        assert_eq!(BuildTunnelPhase::MovingToSite, BuildTunnelPhase::MovingToSite);
    }

    #[test]
    fn build_tunnel_phase_constructing_stores_data() {
        let phase = BuildTunnelPhase::Constructing {
            tunnel_entity: Entity::from_raw_u32(42).unwrap(),
            frames_elapsed: 100,
        };
        if let BuildTunnelPhase::Constructing { tunnel_entity, frames_elapsed } = phase {
            assert_eq!(tunnel_entity, Entity::from_raw_u32(42).unwrap());
            assert_eq!(frames_elapsed, 100);
        } else {
            panic!("Expected Constructing variant");
        }
    }

    /// Helper to spawn an Agent entity with all components needed by building_tunnel_behavior_system
    fn spawn_agent_with_build_tunnel(world: &mut World, pos: Vec3, target: Vec3, owner: Owner) -> Entity {
        world.spawn((
            Transform::from_translation(pos),
            owner,
            BuildingTunnelBehavior::new(target),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(target),
            BaseBehaviorState::None,
            Visibility::Inherited,
        )).id()
    }

    /// Helper to spawn 4x4 buildable tiles at a given grid position
    /// Use grid (32, 32) for world position Vec3(0.5, 0, 0.5)
    fn spawn_buildable_tiles_4x4(world: &mut World, grid_x: i32, grid_z: i32) {
        for dx in 0..4 {
            for dz in 0..4 {
                world.spawn((
                    GridPosition { x: grid_x + dx, z: grid_z + dz },
                    TilePreset {
                        value: crate::game::world::types::TilePresetEnum::Plane,
                        name: "Plane".to_string(),
                        texture: None,
                        buildable: true,
                        traversible: true,
                        rugged: false,
                        drillable: false,
                        recruitable: false,
                    },
                    Tile,
                ));
            }
        }
    }

    /// Helper to spawn a Player entity with SyndicatePlayerResources
    fn spawn_syndicate_player(world: &mut World, player_number: u8, supplies: i32) -> Entity {
        world.spawn((
            Player {
                name: format!("Player {}", player_number),
                faction: crate::types::FactionEnum::TheSyndicate,
                player_number,
            },
            SyndicatePlayerResources {
                space_crystals: 0,
                supplies,
                tunnel_space_provided: 100,
                tunnel_space_used: 0,
            },
        )).id()
    }

    #[test]
    fn building_tunnel_moving_to_site_sets_moving() {
        // Agent far from target — should set Moving locomotion
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let target = Vec3::new(10.0, 0.0, 10.0);
        let entity = spawn_agent_with_build_tunnel(app.world_mut(), Vec3::ZERO, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 100);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        let locomotion = app.world_mut().entity(entity).get::<LocomotionChannel>().unwrap();
        assert!(matches!(locomotion, LocomotionChannel::Moving(_)));
        let orientation = app.world_mut().entity(entity).get::<OrientationChannel>().unwrap();
        assert!(matches!(orientation, OrientationChannel::Turning(_)));
        // Phase should still be MovingToSite
        let behavior = app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().unwrap();
        assert_eq!(behavior.phase, BuildTunnelPhase::MovingToSite);
    }

    #[test]
    fn building_tunnel_arrival_spawns_tunnel() {
        // Agent at target with enough supplies — should spawn tunnel
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let target = Vec3::new(0.5, 0.0, 0.5);
        let entity = spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 100);
        spawn_buildable_tiles_4x4(app.world_mut(), 32, 32);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Agent should be hidden
        let vis = app.world_mut().entity(entity).get::<Visibility>().unwrap();
        assert_eq!(*vis, Visibility::Hidden);

        // Phase should be Constructing
        let behavior = app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().unwrap();
        if let BuildTunnelPhase::Constructing { frames_elapsed, .. } = &behavior.phase {
            assert_eq!(*frames_elapsed, 0);
        } else {
            panic!("Expected Constructing phase");
        }

        // Locomotion should be stationary
        let locomotion = app.world_mut().entity(entity).get::<LocomotionChannel>().unwrap();
        assert!(matches!(locomotion, LocomotionChannel::Stationary));
    }

    #[test]
    fn building_tunnel_cost_deducted() {
        // First tunnel is free (0 existing tunnels)
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let target = Vec3::new(0.5, 0.0, 0.5);
        spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        let player_entity = spawn_syndicate_player(app.world_mut(), 0, 10);
        spawn_buildable_tiles_4x4(app.world_mut(), 32, 32);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // First tunnel costs 0 — supplies should remain 10
        let resources = app.world_mut().entity(player_entity).get::<SyndicatePlayerResources>().unwrap();
        assert_eq!(resources.supplies, 10);
    }

    #[test]
    fn building_tunnel_cost_deducted_second_tunnel() {
        // Second tunnel costs 1 supply (1 existing tunnel)
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        // Spawn an existing tunnel owned by player 0
        app.world_mut().spawn((
            TunnelState::default_tier1(),
            Owner::player(0),
        ));

        let target = Vec3::new(0.5, 0.0, 0.5);
        spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        let player_entity = spawn_syndicate_player(app.world_mut(), 0, 10);
        spawn_buildable_tiles_4x4(app.world_mut(), 32, 32);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Second tunnel costs 1 supply
        let resources = app.world_mut().entity(player_entity).get::<SyndicatePlayerResources>().unwrap();
        assert_eq!(resources.supplies, 9);
    }

    #[test]
    fn building_tunnel_insufficient_funds_cancels() {
        // Agent can't afford — should cancel and idle
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        // 1 existing tunnel → cost = 1, but player has 0 supplies
        app.world_mut().spawn((
            TunnelState::default_tier1(),
            Owner::player(0),
        ));

        let target = Vec3::new(0.5, 0.0, 0.5);
        let entity = spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 0);
        spawn_buildable_tiles_4x4(app.world_mut(), 32, 32);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Should be idle, marker removed
        let command = app.world_mut().entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        assert!(app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().is_none());
    }

    #[test]
    fn building_tunnel_constructing_increments_frames() {
        // Agent in Constructing phase — frames should increment
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        // Create a fake tunnel entity with ConstructionHP
        let tunnel_entity = app.world_mut().spawn((
            ObjectInstance::under_construction(ObjectEnum::Tunnel, 600.0),
            ConstructionHP::new(AGENT_TUNNEL_BUILD_FRAMES),
        )).id();

        let mut behavior = BuildingTunnelBehavior::new(Vec3::ZERO);
        behavior.phase = BuildTunnelPhase::Constructing {
            tunnel_entity,
            frames_elapsed: 10,
        };

        let entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Owner::player(0),
            behavior,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(Vec3::ZERO),
            BaseBehaviorState::None,
            Visibility::Hidden,
        )).id();

        spawn_syndicate_player(app.world_mut(), 0, 10);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Frames should be 11
        let b = app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().unwrap();
        if let BuildTunnelPhase::Constructing { frames_elapsed, .. } = &b.phase {
            assert_eq!(*frames_elapsed, 11);
        } else {
            panic!("Expected Constructing phase");
        }
    }

    #[test]
    fn building_tunnel_destroyed_agent_emerges() {
        // Tunnel destroyed during construction — Agent emerges
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        // Don't spawn a tunnel entity — simulates it being destroyed
        let fake_tunnel = Entity::from_raw_u32(9999).unwrap();

        let mut behavior = BuildingTunnelBehavior::new(Vec3::ZERO);
        behavior.phase = BuildTunnelPhase::Constructing {
            tunnel_entity: fake_tunnel,
            frames_elapsed: 100,
        };

        let entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Owner::player(0),
            behavior,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(Vec3::ZERO),
            BaseBehaviorState::None,
            Visibility::Hidden,
        )).id();

        spawn_syndicate_player(app.world_mut(), 0, 10);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Agent should emerge — visible, idle, no marker
        let vis = app.world_mut().entity(entity).get::<Visibility>().unwrap();
        assert_eq!(*vis, Visibility::Inherited);
        let command = app.world_mut().entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        assert!(app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().is_none());
    }

    #[test]
    fn building_tunnel_completes_agent_despawned() {
        // Construction reaches 480 frames — Agent should be despawned
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        // Create tunnel entity with ConstructionHP still present
        let tunnel_entity = app.world_mut().spawn((
            ObjectInstance::under_construction(ObjectEnum::Tunnel, 600.0),
            ConstructionHP::new(AGENT_TUNNEL_BUILD_FRAMES),
        )).id();

        let mut behavior = BuildingTunnelBehavior::new(Vec3::ZERO);
        behavior.phase = BuildTunnelPhase::Constructing {
            tunnel_entity,
            frames_elapsed: AGENT_TUNNEL_BUILD_FRAMES - 1, // One frame away from completion
        };

        let entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Owner::player(0),
            behavior,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(Vec3::ZERO),
            BaseBehaviorState::None,
            Visibility::Hidden,
        )).id();

        spawn_syndicate_player(app.world_mut(), 0, 10);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Agent should be despawned
        assert!(app.world_mut().get_entity(entity).is_err());
    }

    #[test]
    fn building_tunnel_completion_with_construction_hp_removed() {
        // When ConstructionHP is removed (by construction_hp_tick_system) and frames >= 480,
        // Agent should be despawned (enters tunnel network)
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        // Create tunnel entity WITHOUT ConstructionHP (already completed)
        let tunnel_entity = app.world_mut().spawn((
            ObjectInstance::destructible(ObjectEnum::Tunnel, 600.0),
        )).id();

        let mut behavior = BuildingTunnelBehavior::new(Vec3::ZERO);
        behavior.phase = BuildTunnelPhase::Constructing {
            tunnel_entity,
            frames_elapsed: AGENT_TUNNEL_BUILD_FRAMES,
        };

        let entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Owner::player(0),
            behavior,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(Vec3::ZERO),
            BaseBehaviorState::None,
            Visibility::Hidden,
        )).id();

        spawn_syndicate_player(app.world_mut(), 0, 10);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Agent should be despawned (entered tunnel network)
        assert!(app.world_mut().get_entity(entity).is_err());
    }

    #[test]
    fn building_tunnel_no_player_cancels() {
        // Agent with no owner — should cancel
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let target = Vec3::new(0.5, 0.0, 0.5);
        let entity = app.world_mut().spawn((
            Transform::from_translation(target),
            Owner::neutral(),
            BuildingTunnelBehavior::new(target),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(target),
            BaseBehaviorState::None,
            Visibility::Inherited,
        )).id();

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        let command = app.world_mut().entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        assert!(app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().is_none());
    }

    #[test]
    fn build_tunnel_has_indicator() {
        use crate::game::units::types::types::{command_has_indicator, command_indicator_color};
        let cmd = UnitCommand::BuildTunnel(Vec3::new(5.0, 0.0, 5.0));
        assert!(command_has_indicator(&cmd));
        assert_eq!(command_indicator_color(&cmd), Color::srgb(0.0, 1.0, 0.0));
    }

    #[test]
    fn building_tunnel_arrival_spawns_tunnel_entity() {
        // Verify that a TunnelState entity is spawned when Agent arrives
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let target = Vec3::new(0.5, 0.0, 0.5);
        spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 100);
        spawn_buildable_tiles_4x4(app.world_mut(), 32, 32);

        // No tunnels before
        let tunnel_count_before = app.world_mut().query::<&TunnelState>().iter(app.world()).count();
        assert_eq!(tunnel_count_before, 0);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Should have 1 tunnel after (deferred commands need apply)
        app.world_mut().flush();
        let tunnel_count_after = app.world_mut().query::<&TunnelState>().iter(app.world()).count();
        assert_eq!(tunnel_count_after, 1);
    }

    #[test]
    fn building_tunnel_spawned_tunnel_has_construction_hp() {
        // Verify spawned tunnel has ConstructionHP component
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let target = Vec3::new(0.5, 0.0, 0.5);
        spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 100);
        spawn_buildable_tiles_4x4(app.world_mut(), 32, 32);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();
        app.world_mut().flush();

        let construction_count = app.world_mut()
            .query::<(&TunnelState, &ConstructionHP)>()
            .iter(app.world())
            .count();
        assert_eq!(construction_count, 1);
    }

    #[test]
    fn building_tunnel_spawned_tunnel_starts_at_10_percent_hp() {
        // Verify spawned tunnel starts at 10% HP
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let target = Vec3::new(0.5, 0.0, 0.5);
        spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 100);
        spawn_buildable_tiles_4x4(app.world_mut(), 32, 32);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();
        app.world_mut().flush();

        for (obj, _) in app.world_mut().query::<(&ObjectInstance, &TunnelState)>().iter(app.world()) {
            assert!((obj.hp.unwrap() - 60.0).abs() < 0.1, "HP should start at 10% of 600 = 60");
        }
    }

    // === Single-Agent Construction Enforcement tests ===

    #[test]
    fn building_tunnel_rejects_second_agent_at_same_location() {
        // Agent B arrives at a location where Agent A is already constructing —
        // Agent B should be rejected (idled) while Agent A continues
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let target = Vec3::new(0.5, 0.0, 0.5);

        // Agent A: already in Constructing phase
        let tunnel_entity = app.world_mut().spawn((
            TunnelState::default_tier1(),
            Owner::player(0),
            ObjectInstance::destructible(ObjectEnum::Tunnel, 600.0),
            ConstructionHP::new(AGENT_TUNNEL_BUILD_FRAMES),
        )).id();

        let mut behavior_a = BuildingTunnelBehavior::new(target);
        behavior_a.phase = BuildTunnelPhase::Constructing {
            tunnel_entity,
            frames_elapsed: 100,
        };
        let agent_a = app.world_mut().spawn((
            Transform::from_translation(target),
            Owner::player(0),
            behavior_a,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(target),
            BaseBehaviorState::None,
            Visibility::Hidden,
        )).id();

        // Agent B: arriving at same location (MovingToSite, at target)
        let agent_b = spawn_agent_with_build_tunnel(
            app.world_mut(), target, target, Owner::player(0),
        );

        spawn_syndicate_player(app.world_mut(), 0, 200);
        spawn_buildable_tiles_4x4(app.world_mut(), 32, 32);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();
        app.world_mut().flush();

        // Agent A should still be constructing
        let a_behavior = app.world_mut().entity(agent_a).get::<BuildingTunnelBehavior>();
        assert!(a_behavior.is_some(), "Agent A should still have BuildingTunnelBehavior");

        // Agent B should have been rejected — marker removed, command idle
        let b_behavior = app.world_mut().entity(agent_b).get::<BuildingTunnelBehavior>();
        assert!(b_behavior.is_none(), "Agent B should have BuildingTunnelBehavior removed");
        let b_cmd = app.world_mut().entity(agent_b).get::<UnitCommand>().unwrap();
        assert!(matches!(b_cmd, UnitCommand::Idle), "Agent B should be idle");
    }

    #[test]
    fn building_tunnel_allows_agent_at_different_location() {
        // Agent B builds at a different location than Agent A — should proceed
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let target_a = Vec3::new(0.5, 0.0, 0.5);
        let target_b = Vec3::new(10.5, 0.0, 10.5);

        // Agent A: already in Constructing phase at target_a
        let tunnel_entity = app.world_mut().spawn((
            TunnelState::default_tier1(),
            Owner::player(0),
            ObjectInstance::destructible(ObjectEnum::Tunnel, 600.0),
            ConstructionHP::new(AGENT_TUNNEL_BUILD_FRAMES),
        )).id();

        let mut behavior_a = BuildingTunnelBehavior::new(target_a);
        behavior_a.phase = BuildTunnelPhase::Constructing {
            tunnel_entity,
            frames_elapsed: 100,
        };
        app.world_mut().spawn((
            Transform::from_translation(target_a),
            Owner::player(0),
            behavior_a,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(target_a),
            BaseBehaviorState::None,
            Visibility::Hidden,
        ));

        // Agent B: arriving at different location
        let agent_b = spawn_agent_with_build_tunnel(
            app.world_mut(), target_b, target_b, Owner::player(0),
        );

        spawn_syndicate_player(app.world_mut(), 0, 200);
        // Tiles for target_b at grid (42, 42)
        spawn_buildable_tiles_4x4(app.world_mut(), 42, 42);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();
        app.world_mut().flush();

        // Agent B should have started constructing (behavior still present, phase = Constructing)
        let b_behavior = app.world_mut().entity(agent_b).get::<BuildingTunnelBehavior>();
        assert!(b_behavior.is_some(), "Agent B should still have BuildingTunnelBehavior");
        if let Some(b) = b_behavior {
            assert!(
                matches!(b.phase, BuildTunnelPhase::Constructing { .. }),
                "Agent B should be in Constructing phase"
            );
        }
    }

    #[test]
    fn building_tunnel_allows_after_first_agent_finishes() {
        // Agent A's tunnel was destroyed → Agent B should be able to build at the same location
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let target = Vec3::new(0.5, 0.0, 0.5);

        // No other agents constructing — Agent B should succeed
        let agent_b = spawn_agent_with_build_tunnel(
            app.world_mut(), target, target, Owner::player(0),
        );

        spawn_syndicate_player(app.world_mut(), 0, 200);
        spawn_buildable_tiles_4x4(app.world_mut(), 32, 32);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();
        app.world_mut().flush();

        // Agent B should have started constructing
        let b_behavior = app.world_mut().entity(agent_b).get::<BuildingTunnelBehavior>();
        assert!(b_behavior.is_some(), "Agent B should still have BuildingTunnelBehavior");
        if let Some(b) = b_behavior {
            assert!(
                matches!(b.phase, BuildTunnelPhase::Constructing { .. }),
                "Agent B should be in Constructing phase"
            );
        }
    }

    // === Tunnel Arrival Validation Tests ===

    #[test]
    fn building_tunnel_arrival_unbuildable_tile_cancels() {
        // Agent at target, but one tile is unbuildable → should cancel
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let target = Vec3::new(0.5, 0.0, 0.5);
        let entity = spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 100);

        // Spawn 4x4 tiles but make one unbuildable
        for dx in 0..4 {
            for dz in 0..4 {
                app.world_mut().spawn((
                    GridPosition { x: 32 + dx, z: 32 + dz },
                    TilePreset {
                        value: crate::game::world::types::TilePresetEnum::Plane,
                        name: "Plane".to_string(),
                        texture: None,
                        buildable: !(dx == 1 && dz == 1), // One tile unbuildable
                        traversible: true,
                        rugged: false,
                        drillable: false,
                        recruitable: false,
                    },
                    Tile,
                ));
            }
        }

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();
        app.world_mut().flush();

        // Agent should be idle, behavior removed
        let cmd = app.world_mut().entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(cmd, UnitCommand::Idle), "Agent should be idle after unbuildable tile");
        assert!(
            app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().is_none(),
            "BuildingTunnelBehavior should be removed"
        );

        // No tunnel should have been spawned
        let tunnel_count = app.world_mut().query::<&TunnelState>().iter(app.world()).count();
        assert_eq!(tunnel_count, 0, "No tunnel should be spawned");
    }

    #[test]
    fn building_tunnel_arrival_structure_overlap_cancels() {
        // Agent at target, tiles are buildable, but existing structure overlaps → should cancel
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let target = Vec3::new(0.5, 0.0, 0.5);
        let entity = spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 100);
        spawn_buildable_tiles_4x4(app.world_mut(), 32, 32);

        // Spawn an existing structure that overlaps the placement area
        app.world_mut().spawn((
            GridPosition { x: 33, z: 33 },
            StructureInstance::default(),
            ObjectInstance::destructible(ObjectEnum::Tunnel, 600.0),
        ));

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();
        app.world_mut().flush();

        // Agent should be idle, behavior removed
        let cmd = app.world_mut().entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(cmd, UnitCommand::Idle), "Agent should be idle after structure overlap");
        assert!(
            app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().is_none(),
            "BuildingTunnelBehavior should be removed"
        );
    }

    #[test]
    fn building_tunnel_arrival_no_tiles_cancels() {
        // Agent at target, no tiles spawned → should cancel
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let target = Vec3::new(0.5, 0.0, 0.5);
        let entity = spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 100);
        // No tiles spawned

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();
        app.world_mut().flush();

        // Agent should be idle, behavior removed
        let cmd = app.world_mut().entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(cmd, UnitCommand::Idle), "Agent should be idle when no tiles exist");
        assert!(
            app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().is_none(),
            "BuildingTunnelBehavior should be removed"
        );
    }

    // =====================================================
    // CULTS RECRUIT ENTER CONSTRUCTION SYSTEM TESTS
    // =====================================================

    fn spawn_test_recruit_for_construction(world: &mut World, pos: Vec3, target_building: Entity) -> Entity {
        world.spawn((
            crate::types::Unit,
            ObjectInstance::destructible(ObjectEnum::CultsRecruit, 50.0),
            Owner::player(0),
            UnitCommand::ConstructBuilding(target_building),
            Visibility::Inherited,
            Transform::from_translation(pos),
        )).id()
    }

    fn spawn_test_construction_building(world: &mut World, pos: Vec3, total_frames: u32) -> Entity {
        world.spawn((
            ObjectInstance::under_construction(ObjectEnum::CultsStorage, 200.0),
            ConstructionHP::new(total_frames),
            CultsConstructionState::new(total_frames),
            Transform::from_translation(pos),
            Owner::player(0),
        )).id()
    }

    #[test]
    fn recruit_enters_building_when_close() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let building_pos = Vec3::new(5.0, 0.0, 5.0);
        let building = spawn_test_construction_building(app.world_mut(), building_pos, 100);
        // Recruit is within arrival threshold (< 2.0)
        let recruit = spawn_test_recruit_for_construction(app.world_mut(), Vec3::new(5.5, 0.0, 5.5), building);

        app.world_mut().run_system_once(cults_recruit_enter_construction_system).unwrap();

        // Recruit should be hidden and idle
        let vis = app.world().get::<Visibility>(recruit).unwrap();
        assert_eq!(*vis, Visibility::Hidden, "Recruit should be hidden after entering");

        let cmd = app.world().get::<UnitCommand>(recruit).unwrap();
        assert!(matches!(*cmd, UnitCommand::Idle), "Recruit should be idle after entering");

        // Building should have the recruit in assigned_recruits
        let state = app.world().get::<CultsConstructionState>(building).unwrap();
        assert_eq!(state.assigned_recruits.len(), 1);
        assert_eq!(state.assigned_recruits[0], recruit);
    }

    #[test]
    fn recruit_keeps_command_when_far_away() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let building_pos = Vec3::new(5.0, 0.0, 5.0);
        let building = spawn_test_construction_building(app.world_mut(), building_pos, 100);
        // Recruit is far away (> 2.0)
        let recruit = spawn_test_recruit_for_construction(app.world_mut(), Vec3::new(20.0, 0.0, 20.0), building);

        app.world_mut().run_system_once(cults_recruit_enter_construction_system).unwrap();

        // Recruit should still have the ConstructBuilding command
        let cmd = app.world().get::<UnitCommand>(recruit).unwrap();
        assert!(matches!(*cmd, UnitCommand::ConstructBuilding(_)),
            "Recruit should keep ConstructBuilding when far away");

        // Building should NOT have the recruit assigned yet
        let state = app.world().get::<CultsConstructionState>(building).unwrap();
        assert!(state.assigned_recruits.is_empty());
    }

    #[test]
    fn recruit_cancels_when_building_gone() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn building, then despawn it
        let building = app.world_mut().spawn((
            ObjectInstance::under_construction(ObjectEnum::CultsStorage, 200.0),
            ConstructionHP::new(100),
            CultsConstructionState::new(100),
            Transform::from_xyz(5.0, 0.0, 5.0),
            Owner::player(0),
        )).id();

        let recruit = spawn_test_recruit_for_construction(app.world_mut(), Vec3::new(5.5, 0.0, 5.5), building);

        // Remove the building
        app.world_mut().despawn(building);

        app.world_mut().run_system_once(cults_recruit_enter_construction_system).unwrap();

        // Recruit should be set to Idle since building is gone
        let cmd = app.world().get::<UnitCommand>(recruit).unwrap();
        assert!(matches!(*cmd, UnitCommand::Idle),
            "Recruit should cancel when building no longer exists");
    }

    #[test]
    fn multiple_recruits_can_enter_same_building() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let building_pos = Vec3::new(5.0, 0.0, 5.0);
        let building = spawn_test_construction_building(app.world_mut(), building_pos, 100);
        let _recruit1 = spawn_test_recruit_for_construction(app.world_mut(), Vec3::new(5.3, 0.0, 5.3), building);
        let _recruit2 = spawn_test_recruit_for_construction(app.world_mut(), Vec3::new(4.8, 0.0, 5.1), building);

        app.world_mut().run_system_once(cults_recruit_enter_construction_system).unwrap();

        let state = app.world().get::<CultsConstructionState>(building).unwrap();
        assert_eq!(state.assigned_recruits.len(), 2, "Both recruits should enter");
    }
}
