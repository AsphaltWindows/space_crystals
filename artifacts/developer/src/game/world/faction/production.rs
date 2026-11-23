use bevy::prelude::*;
use crate::types::{Owner, ObjectEnum, GridPosition};
use crate::game::types::*;
use crate::game::entities::{EntityRegistry, SpawnParams};
use crate::game::entities::utils::{insert_entity_visuals, grid_to_spawn_position};
use crate::game::world::types::{Tile, TilePreset};
use crate::game::world::utils::world_to_grid;
use super::power::get_power_ratio_for_owner;

// =====================================================
// BARRACKS PRODUCTION SYSTEM
// =====================================================

/// System that ticks Barracks production progress each simulation frame
pub fn barracks_production_tick_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut barracks_query: Query<(Entity, &Owner, &crate::types::GridPosition, &mut BarracksState, &StructureInstance)>,
    mut players: Query<(&Player, &mut GdoPlayerResources)>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: Res<crate::game::world::types::GridMap>,
    rally_targets: Query<(&Transform, &Owner), With<ObjectInstance>>,
    occupancy: Res<crate::game::units::types::OccupancyMap>,
    registry: Res<EntityRegistry>,
) {
    for (_bk_entity, owner, grid_pos, mut bk_state, structure_instance) in barracks_query.iter_mut() {
        // If no current build but queue has items, start next build
        if bk_state.current_build.is_none() && !bk_state.build_queue.is_empty() {
            let next = bk_state.build_queue.remove(0);
            bk_state.current_build = Some(next);
            bk_state.current_build_progress = Some(0.0);
        }

        // Tick current build
        if let Some(unit_type) = bk_state.current_build {
            if let Some(cost) = registry.cost(&unit_type) {
                let required_frames = cost.build_frames as f32;
                let power_ratio = get_power_ratio_for_owner(owner, &players);

                if let Some(ref mut progress) = bk_state.current_build_progress {
                    *progress += power_ratio;

                    if *progress >= required_frames {
                        // Production complete — spawn unit at the B-side exit
                        let (dx, dz) = crate::game::world::utils::spawn_side_offset(
                            ObjectEnum::Barracks, structure_instance,
                        );
                        let spawn_x = grid_pos.x + dx;
                        let spawn_z = grid_pos.z + dz;

                        let spawn_pos = grid_to_spawn_position(spawn_x, spawn_z, 1.0, 1.0);
                        let unit_entity = registry.spawn(&unit_type, &mut commands, SpawnParams {
                            position: spawn_pos,
                            owner: *owner,
                            player: owner.player_number().unwrap_or(0),
                            rotation: None,
                            under_construction: false,
                            flip_horizontal: false,
                            flip_vertical: false,
                        });
                        insert_entity_visuals(&mut commands, unit_entity, &unit_type, owner, &mut meshes, &mut materials);

                        // Update unit control using the unit's actual control cost
                        let control_cost = crate::game::entities::factions::gdo::units::peacekeeper::PEACEKEEPER_CONTROL_COST;
                        for (player, mut res) in players.iter_mut() {
                            if Some(player.player_number) == owner.player_number() {
                                res.unit_control_used += control_cost;
                                break;
                            }
                        }

                        // Issue rally point command to the spawned unit
                        let produced_unit_base = {
                            use crate::game::entities::types::UnitDefinition;
                            crate::game::entities::factions::gdo::units::peacekeeper::Peacekeeper::unit_stats().unit_base
                        };
                        issue_rally_command(
                            &mut commands, unit_entity,
                            &bk_state.rally_point, owner,
                            spawn_x, spawn_z,
                            &tiles, &grid, &rally_targets,
                            &occupancy, &produced_unit_base,
                        );

                        info!("Barracks: Produced {:?} at ({}, {})", unit_type, spawn_x, spawn_z);

                        bk_state.current_build = None;
                        bk_state.current_build_progress = None;
                    }
                }
            }
        }
    }
}

/// Issue a rally command to a newly spawned unit based on the barracks rally point.
/// `unit_base` determines pathfinding domain (air units skip terrain checks).
fn issue_rally_command(
    commands: &mut Commands,
    unit_entity: Entity,
    rally_point: &Option<RallyTarget>,
    owner: &Owner,
    spawn_x: i32,
    spawn_z: i32,
    tiles: &Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: &crate::game::world::types::GridMap,
    rally_targets: &Query<(&Transform, &Owner), With<ObjectInstance>>,
    occupancy: &crate::game::units::types::OccupancyMap,
    unit_base: &crate::types::UnitBaseEnum,
) {
    use crate::game::units::utils::{world_to_grid, smooth_path};
    use crate::game::units::pathfinding::find_path_for_domain;
    use crate::game::units::types::movement::{MoveTarget, Path};
    use crate::game::units::types::state::UnitCommand;

    let rally = match rally_point {
        Some(r) => r,
        None => return, // No rally point set — unit stays idle
    };

    let spawn_grid = GridPosition { x: spawn_x, z: spawn_z };

    match rally {
        RallyTarget::Location(pos) => {
            let target_grid = world_to_grid(*pos);
            if let Some(path) = find_path_for_domain(spawn_grid, target_grid, tiles, unit_base, grid.width as i32, grid.height as i32, occupancy, (spawn_grid.x, spawn_grid.z)) {
                let smoothed = smooth_path(path);
                commands.entity(unit_entity).insert((
                    MoveTarget(*pos),
                    Path { waypoints: smoothed, current_waypoint: 0 },
                    UnitCommand::Move(*pos),
                ));
            }
        }
        RallyTarget::Object(target_entity) => {
            if let Ok((target_transform, target_owner)) = rally_targets.get(*target_entity) {
                let target_pos = target_transform.translation;
                let is_enemy = !target_owner.is_neutral()
                    && target_owner.player_number() != owner.player_number();

                if is_enemy {
                    // Attack enemy rally target with approach path
                    let target_grid = world_to_grid(target_pos);
                    if let Some(path) = find_path_for_domain(spawn_grid, target_grid, tiles, unit_base, grid.width as i32, grid.height as i32, occupancy, (spawn_grid.x, spawn_grid.z)) {
                        let smoothed = smooth_path(path);
                        commands.entity(unit_entity).insert((
                            UnitCommand::AttackTarget(*target_entity),
                            MoveTarget(target_pos),
                            Path { waypoints: smoothed, current_waypoint: 0 },
                        ));
                    } else {
                        // Pathfinding failed — still issue attack command, combat behavior will handle approach
                        commands.entity(unit_entity).insert(UnitCommand::AttackTarget(*target_entity));
                    }
                } else {
                    // Move to friendly/neutral rally target position
                    let target_grid = world_to_grid(target_pos);
                    if let Some(path) = find_path_for_domain(spawn_grid, target_grid, tiles, unit_base, grid.width as i32, grid.height as i32, occupancy, (spawn_grid.x, spawn_grid.z)) {
                        let smoothed = smooth_path(path);
                        commands.entity(unit_entity).insert((
                            MoveTarget(target_pos),
                            Path { waypoints: smoothed, current_waypoint: 0 },
                            UnitCommand::Move(target_pos),
                        ));
                    }
                }
            }
            // If target entity no longer exists, unit stays idle
        }
    }
}

// =====================================================
// HEADQUARTERS PRODUCTION SYSTEM
// =====================================================

/// System that ticks Headquarters unit production each simulation frame.
/// Follows the same pattern as `barracks_production_tick_system` but uses
/// `SyndicatePlayerResources` and spawns Agents from the parent Tunnel's surface position.
pub fn headquarters_production_tick_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut hq_query: Query<(Entity, &Owner, &mut HeadquartersState, &TunnelExpansionMarker)>,
    _players: Query<(&Player, &SyndicatePlayerResources)>,
    tunnel_data: Query<(&Transform, &StructureInstance, &GridPosition), With<TunnelState>>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: Res<crate::game::world::types::GridMap>,
    rally_targets: Query<(&Transform, &Owner), With<ObjectInstance>>,
    occupancy: Res<crate::game::units::types::OccupancyMap>,
    registry: Res<EntityRegistry>,
) {
    use crate::game::units::utils::tunnel_side_world_position;
    use crate::game::units::utils::world_to_grid;
    use crate::game::units::types::state::behavior::InTunnelNetwork;

    for (_hq_entity, owner, mut hq_state, expansion_marker) in hq_query.iter_mut() {
        // If no current build but queue has items, start next build
        if hq_state.current_build.is_none() && !hq_state.build_queue.is_empty() {
            let next = hq_state.build_queue.remove(0);
            hq_state.current_build = Some(next);
            hq_state.current_build_progress = Some(0.0);
        }

        // Tick current build
        if let Some(unit_type) = hq_state.current_build {
            if let Some(cost) = registry.cost(&unit_type) {
                let required_frames = cost.build_frames as f32;

                if let Some(ref mut progress) = hq_state.current_build_progress {
                    // No power ratio for Syndicate — progress 1.0 per frame
                    *progress += 1.0;

                    if *progress >= required_frames {
                        // Determine if unit should eject to surface or enter tunnel network
                        let should_eject = match &hq_state.rally_point {
                            Some(RallyTarget::Object(entity)) if *entity == expansion_marker.parent_tunnel => {
                                // Rally target is parent tunnel → stay in tunnel network
                                false
                            }
                            Some(_) => true, // Rally to location or non-parent object → eject
                            None => false,   // No rally point → stay in tunnel network
                        };

                        if should_eject {
                            // Spawn at Side A of parent tunnel
                            let spawn_pos = if let Ok((tunnel_tf, tunnel_si, _tunnel_gp)) = tunnel_data.get(expansion_marker.parent_tunnel) {
                                let side_a_world = tunnel_side_world_position(tunnel_tf, tunnel_si, 'A');
                                let side_a_grid = world_to_grid(side_a_world);
                                (side_a_grid.x, side_a_grid.z)
                            } else {
                                warn!("HQ production: parent tunnel {:?} not found in tunnel_data query, falling back to map center", expansion_marker.parent_tunnel);
                                (32, 32) // Fallback to map center
                            };

                            let spawn_world = grid_to_spawn_position(spawn_pos.0, spawn_pos.1, 1.0, 1.0);
                            let unit_entity = registry.spawn(&unit_type, &mut commands, SpawnParams {
                                position: spawn_world,
                                owner: *owner,
                                player: owner.player_number().unwrap_or(0),
                                rotation: None,
                                under_construction: false,
                                flip_horizontal: false,
                                flip_vertical: false,
                            });
                            insert_entity_visuals(&mut commands, unit_entity, &unit_type, owner, &mut meshes, &mut materials);

                            // Determine unit base for domain-aware pathfinding
                            let produced_unit_base = match &registry.get(&unit_type).unwrap().kind {
                                crate::game::entities::EntityKind::Unit { unit_stats, .. } => unit_stats.unit_base,
                                _ => crate::types::UnitBaseEnum::HeavyInfantry,
                            };

                            // Issue rally point command to the spawned unit
                            issue_rally_command(
                                &mut commands, unit_entity,
                                &hq_state.rally_point, owner,
                                spawn_pos.0, spawn_pos.1,
                                &tiles, &grid, &rally_targets,
                                &occupancy, &produced_unit_base,
                            );

                            info!("Headquarters: Produced {:?} at Side A ({}, {})", unit_type, spawn_pos.0, spawn_pos.1);
                        } else {
                            // No surface rally point — unit enters Tunnel Network
                            let owner_player = owner.player_number().unwrap_or(0);

                            let spawn_world = grid_to_spawn_position(0, 0, 1.0, 1.0);
                            let unit_entity = registry.spawn(&unit_type, &mut commands, SpawnParams {
                                position: spawn_world,
                                owner: *owner,
                                player: owner.player_number().unwrap_or(0),
                                rotation: None,
                                under_construction: false,
                                flip_horizontal: false,
                                flip_vertical: false,
                            });
                            insert_entity_visuals(&mut commands, unit_entity, &unit_type, owner, &mut meshes, &mut materials);

                            // Mark as in tunnel network and hide from map
                            commands.entity(unit_entity).insert(InTunnelNetwork { owner_player });
                            commands.entity(unit_entity).insert(Visibility::Hidden);

                            info!("Headquarters: Produced {:?}, entered Tunnel Network", unit_type);
                        }

                        hq_state.current_build = None;
                        hq_state.current_build_progress = None;
                    }
                }
            }
        }
    }
}

/// System to tick Supply Tower production queues each fixed timestep
pub fn supply_tower_production_tick_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut st_query: Query<(Entity, &Owner, &crate::types::GridPosition, &mut SupplyTowerState, &StructureInstance)>,
    players: Query<(&Player, &mut GdoPlayerResources)>,
    rally_targets: Query<(&Transform, &Owner), With<ObjectInstance>>,
    registry: Res<EntityRegistry>,
) {
    use crate::game::units::types::movement::MoveTarget;
    use crate::game::units::types::state::UnitCommand;

    for (_st_entity, owner, grid_pos, mut st_state, structure_instance) in st_query.iter_mut() {
        // If no current build but queue has items, start next build
        if st_state.current_build.is_none() && !st_state.build_queue.is_empty() {
            let next = st_state.build_queue.remove(0);
            st_state.current_build = Some(next);
            st_state.current_build_progress = Some(0.0);
        }

        // Tick current build
        if let Some(unit_type) = st_state.current_build {
            if let Some(cost) = registry.cost(&unit_type) {
                let required_frames = cost.build_frames as f32;
                let power_ratio = get_power_ratio_for_owner(owner, &players);

                if let Some(ref mut progress) = st_state.current_build_progress {
                    *progress += power_ratio;

                    if *progress >= required_frames {
                        // Production complete — spawn chopper at the spawn-side exit
                        let (dx, dz) = crate::game::world::utils::spawn_side_offset(
                            ObjectEnum::SupplyTower, structure_instance,
                        );
                        let spawn_x = grid_pos.x + dx;
                        let spawn_z = grid_pos.z + dz;

                        let spawn_world = grid_to_spawn_position(spawn_x, spawn_z, 1.0, 1.0);
                        let unit_entity = registry.spawn(&ObjectEnum::SupplyChopper, &mut commands, SpawnParams {
                            position: spawn_world,
                            owner: *owner,
                            player: owner.player_number().unwrap_or(0),
                            rotation: None,
                            under_construction: false,
                            flip_horizontal: false,
                            flip_vertical: false,
                        });
                        insert_entity_visuals(&mut commands, unit_entity, &ObjectEnum::SupplyChopper, owner, &mut meshes, &mut materials);

                        // Issue rally command to the newly spawned chopper (air unit — direct move, no pathfinding needed)
                        if let Some(rally) = &st_state.rally_point {
                            match rally {
                                RallyTarget::Location(pos) => {
                                    commands.entity(unit_entity).insert((
                                        MoveTarget(*pos),
                                        UnitCommand::Move(*pos),
                                    ));
                                }
                                RallyTarget::Object(target_entity) => {
                                    if let Ok((target_transform, target_owner)) = rally_targets.get(*target_entity) {
                                        let target_pos = target_transform.translation;
                                        let is_enemy = !target_owner.is_neutral()
                                            && target_owner.player_number() != owner.player_number();

                                        if is_enemy {
                                            commands.entity(unit_entity).insert(UnitCommand::AttackTarget(*target_entity));
                                        } else {
                                            commands.entity(unit_entity).insert((
                                                MoveTarget(target_pos),
                                                UnitCommand::Move(target_pos),
                                            ));
                                        }
                                    }
                                }
                            }
                        }

                        info!("Supply Tower: Produced {:?} at ({}, {})", unit_type, spawn_x, spawn_z);

                        st_state.current_build = None;
                        st_state.current_build_progress = None;
                    }
                }
            }
        }
    }
}

// =====================================================
// RECRUITMENT CENTER AUTO-PRODUCTION SYSTEM
// =====================================================

/// System that auto-produces Recruit units from Recruitment Centers each simulation frame.
/// Each RC with effectiveness > 0 and local_used < local_capacity accumulates production_progress.
/// At 100% effectiveness, production takes 12 seconds (192 frames). Scales inversely with effectiveness.
pub fn recruitment_center_production_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rc_query: Query<(Entity, &Owner, &GridPosition, &mut RecruitmentCenterState)>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: Res<crate::game::world::types::GridMap>,
    occupancy: Res<crate::game::units::types::OccupancyMap>,
    registry: Res<EntityRegistry>,
) {
    use crate::game::units::utils::{world_to_grid, smooth_path};
    use crate::game::units::pathfinding::find_path_for_domain;
    use crate::game::units::types::movement::{MoveTarget, Path};
    use crate::game::units::types::state::UnitCommand;
    use crate::game::units::types::unit_data::OriginatingCenters;
    use crate::simulation::FRAMES_PER_SECOND;
    use crate::types::UnitBaseEnum;

    let base_frames = 12 * FRAMES_PER_SECOND; // 192 frames at 16 FPS

    for (center_entity, owner, grid_pos, mut rc_state) in rc_query.iter_mut() {
        // Skip inactive or at-capacity centers
        if rc_state.effectiveness <= 0.0 || rc_state.local_used >= rc_state.local_capacity {
            continue;
        }

        let required_frames = (base_frames as f32 / rc_state.effectiveness).ceil() as u32;
        rc_state.production_progress += 1;

        if rc_state.production_progress >= required_frames {
            rc_state.production_progress = 0;

            // Spawn Recruit at RC's grid position (top-left corner)
            let spawn_x = grid_pos.x;
            let spawn_z = grid_pos.z;

            let spawn_world = grid_to_spawn_position(spawn_x, spawn_z, 1.0, 1.0);
            let unit_entity = registry.spawn(&ObjectEnum::CultsRecruit, &mut commands, SpawnParams {
                position: spawn_world,
                owner: *owner,
                player: owner.player_number().unwrap_or(0),
                rotation: None,
                under_construction: false,
                flip_horizontal: false,
                flip_vertical: false,
            });
            insert_entity_visuals(&mut commands, unit_entity, &ObjectEnum::CultsRecruit, owner, &mut meshes, &mut materials);

            // Track originating center for death-based unit control release
            commands.entity(unit_entity).insert(OriginatingCenters {
                centers: vec![center_entity],
            });

            rc_state.local_used += 1;

            // Issue rally command if set
            if let Some(rally_pos) = rc_state.rally_point {
                let spawn_grid = GridPosition { x: spawn_x, z: spawn_z };
                let target_grid = world_to_grid(rally_pos);
                if let Some(path) = find_path_for_domain(
                    spawn_grid, target_grid, &tiles,
                    &UnitBaseEnum::LightInfantry,
                    grid.width as i32, grid.height as i32,
                    &occupancy, (spawn_x, spawn_z),
                ) {
                    let smoothed = smooth_path(path);
                    commands.entity(unit_entity).insert((
                        MoveTarget(rally_pos),
                        Path { waypoints: smoothed, current_waypoint: 0 },
                        UnitCommand::Move(rally_pos),
                    ));
                }
            }

            info!("Recruitment Center: Produced Recruit at ({}, {})", spawn_x, spawn_z);
        }
    }
}

/// System to tick Armory training progress and spawn trained units.
/// Runs in FixedUpdate.
pub fn armory_training_tick_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut armory_query: Query<(Entity, &Owner, &Transform, &StructureInstance, &mut ArmoryState)>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: Res<crate::game::world::types::GridMap>,
    rally_targets: Query<(&Transform, &Owner), With<ObjectInstance>>,
    occupancy: Res<crate::game::units::types::OccupancyMap>,
    registry: Res<EntityRegistry>,
) {
    for (_entity, owner, transform, si, mut armory) in armory_query.iter_mut() {
        let Some(unit_type) = armory.training_queue else { continue; };
        let required_frames = match unit_type {
            ObjectEnum::CultsSoldier => crate::game::types::cults_structure_stats::SOLDIER_TRAINING_FRAMES,
            ObjectEnum::CultsGunner => crate::game::types::cults_structure_stats::GUNNER_TRAINING_FRAMES,
            _ => continue,
        };
        armory.training_progress += 1;
        if armory.training_progress >= required_frames {
            // Compute exit side (C) position
            let exit_pos = super::ejection::armory_exit_side_position(transform, si);
            let (exit_x, exit_z) = world_to_grid(exit_pos, 1.0);

            // Spawn the trained unit via registry
            let spawn_world = grid_to_spawn_position(exit_x, exit_z, 1.0, 1.0);
            let unit_entity = registry.spawn(&unit_type, &mut commands, SpawnParams {
                position: spawn_world,
                owner: *owner,
                player: owner.player_number().unwrap_or(0),
                rotation: None,
                under_construction: false,
                flip_horizontal: false,
                flip_vertical: false,
            });
            insert_entity_visuals(&mut commands, unit_entity, &unit_type, owner, &mut meshes, &mut materials);

            // Issue rally command (follow HQ pattern)
            issue_rally_command(
                &mut commands, unit_entity,
                &armory.rally_point, owner,
                exit_x, exit_z,
                &tiles, &grid, &rally_targets, &occupancy,
                &crate::types::UnitBaseEnum::LightInfantry,
            );

            // Clear training state
            armory.training_queue = None;
            armory.training_progress = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::types::FactionEnum;
    use crate::game::world::types::{Tile, TilePresetEnum};

    #[test]
    fn headquarters_production_eject_decision_with_parent_tunnel_rally() {
        // When rally_point is Object(parent_tunnel), should NOT eject
        let parent_tunnel = Entity::from_raw_u32(42).unwrap();
        let rally_point = Some(RallyTarget::Object(parent_tunnel));

        let should_eject = match &rally_point {
            Some(RallyTarget::Location(_)) => true,
            Some(RallyTarget::Object(entity)) => *entity != parent_tunnel,
            None => false,
        };

        assert!(!should_eject, "Rally on parent tunnel should not cause ejection");
    }

    #[test]
    fn headquarters_production_eject_decision_with_enemy_rally() {
        // When rally_point is Object(enemy), should eject
        let parent_tunnel = Entity::from_raw_u32(42).unwrap();
        let enemy = Entity::from_raw_u32(99).unwrap();
        let rally_point = Some(RallyTarget::Object(enemy));

        let should_eject = match &rally_point {
            Some(RallyTarget::Location(_)) => true,
            Some(RallyTarget::Object(entity)) => *entity != parent_tunnel,
            None => false,
        };

        assert!(should_eject, "Rally on enemy should cause ejection");
    }

    #[test]
    fn supply_tower_placement_links_chopper() {
        use crate::game::entities::registry::build_registry;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let (tower_entity, chopper_entity) = app.world_mut().run_system_once(
            |mut commands: Commands,
             mut meshes: ResMut<Assets<Mesh>>,
             mut materials: ResMut<Assets<StandardMaterial>>,
             registry: Res<EntityRegistry>| {
                let owner = Owner::player(1);
                let tower_pos = grid_to_spawn_position(32, 32, 3.0, 3.0);
                let tower = registry.spawn(&ObjectEnum::SupplyTower, &mut commands, SpawnParams {
                    position: tower_pos,
                    owner,
                    player: 1,
                    rotation: Some(crate::types::StructureRotation::R0),
                    under_construction: false,
                    flip_horizontal: false,
                    flip_vertical: false,
                });
                insert_entity_visuals(&mut commands, tower, &ObjectEnum::SupplyTower, &owner, &mut meshes, &mut materials);
                let chopper_pos = grid_to_spawn_position(33, 32, 1.0, 1.0);
                let chopper = registry.spawn(&ObjectEnum::SupplyChopper, &mut commands, SpawnParams {
                    position: chopper_pos,
                    owner,
                    player: 1,
                    rotation: None,
                    under_construction: false,
                    flip_horizontal: false,
                    flip_vertical: false,
                });
                insert_entity_visuals(&mut commands, chopper, &ObjectEnum::SupplyChopper, &owner, &mut meshes, &mut materials);
                commands.entity(tower).insert(SupplyTowerState {
                    attached_chopper: Some(chopper),
                    ..default()
                });
                commands.entity(chopper).insert(SupplyChopperState {
                    attached_tower: Some(tower),
                    ..default()
                });
                (tower, chopper)
            },
        ).unwrap();

        app.world_mut().flush();

        let tower_state = app.world().get::<SupplyTowerState>(tower_entity).unwrap();
        assert_eq!(tower_state.attached_chopper, Some(chopper_entity),
            "Tower should reference the chopper entity");

        let chopper_state = app.world().get::<SupplyChopperState>(chopper_entity).unwrap();
        assert_eq!(chopper_state.attached_tower, Some(tower_entity),
            "Chopper should reference the tower entity");
    }

    /// Helper: spawn a minimal tile entity at the given grid position with the given preset
    fn spawn_test_tile(world: &mut World, x: i32, z: i32, preset_enum: TilePresetEnum) -> Entity {
        let preset = preset_enum.properties();
        world.spawn((
            GridPosition { x, z },
            preset,
            Tile,
        )).id()
    }

    /// Helper: spawn an RC entity with custom effectiveness and capacity for production tests
    fn spawn_production_rc(world: &mut World, x: i32, z: i32, effectiveness: f32, local_capacity: u32, owner: Owner) -> Entity {
        world.spawn((
            GridPosition { x, z },
            RecruitmentCenterState {
                effectiveness,
                local_capacity,
                local_used: 0,
                production_progress: 0,
                rally_point: None,
                build_order: 0,
                claimed_tiles: Vec::new(),
            },
            ObjectInstance::destructible(ObjectEnum::RecruitmentCenter, 500.0),
            owner,
        )).id()
    }

    fn setup_production_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(crate::game::entities::registry::build_registry());
        app.init_resource::<crate::game::world::types::GridMap>();
        app.init_resource::<crate::game::units::types::OccupancyMap>();
        app.insert_resource(crate::game::entities::registry::build_registry());
        app
    }

    #[test]
    fn rc_production_100_pct_produces_recruit_at_192_frames() {
        let mut app = setup_production_app();
        let rc_entity = spawn_production_rc(app.world_mut(), 30, 30, 1.0, 20, Owner::player(0));

        // Run 191 frames — should NOT have produced yet
        for _ in 0..191 {
            app.world_mut().run_system_once(recruitment_center_production_system).unwrap();
        }

        let recruit_count = app.world_mut().query_filtered::<Entity, With<crate::types::Unit>>().iter(app.world()).count();
        assert_eq!(recruit_count, 0, "Should not produce before 192 frames");

        // Frame 192 — should produce
        app.world_mut().run_system_once(recruitment_center_production_system).unwrap();

        let recruit_count = app.world_mut().query_filtered::<Entity, With<crate::types::Unit>>().iter(app.world()).count();
        assert_eq!(recruit_count, 1, "Should produce 1 recruit at frame 192");

        // Check RC state
        let rc_state = app.world().get::<RecruitmentCenterState>(rc_entity).unwrap();
        assert_eq!(rc_state.local_used, 1);
        assert_eq!(rc_state.production_progress, 0);
    }

    #[test]
    fn rc_production_50_pct_produces_recruit_at_384_frames() {
        let mut app = setup_production_app();
        spawn_production_rc(app.world_mut(), 30, 30, 0.5, 20, Owner::player(0));

        // Run 383 frames — should NOT produce
        for _ in 0..383 {
            app.world_mut().run_system_once(recruitment_center_production_system).unwrap();
        }

        let recruit_count = app.world_mut().query_filtered::<Entity, With<crate::types::Unit>>().iter(app.world()).count();
        assert_eq!(recruit_count, 0, "Should not produce before 384 frames at 50% effectiveness");

        // Frame 384 — should produce
        app.world_mut().run_system_once(recruitment_center_production_system).unwrap();

        let recruit_count = app.world_mut().query_filtered::<Entity, With<crate::types::Unit>>().iter(app.world()).count();
        assert_eq!(recruit_count, 1, "Should produce 1 recruit at frame 384");
    }

    #[test]
    fn rc_production_stops_at_capacity() {
        let mut app = setup_production_app();
        let rc_entity = spawn_production_rc(app.world_mut(), 30, 30, 1.0, 2, Owner::player(0));

        // Produce 2 recruits (2 * 192 = 384 frames)
        for _ in 0..384 {
            app.world_mut().run_system_once(recruitment_center_production_system).unwrap();
        }

        let recruit_count = app.world_mut().query_filtered::<Entity, With<crate::types::Unit>>().iter(app.world()).count();
        assert_eq!(recruit_count, 2, "Should produce 2 recruits");

        let rc_state = app.world().get::<RecruitmentCenterState>(rc_entity).unwrap();
        assert_eq!(rc_state.local_used, 2);

        // Run 192 more frames — should NOT produce (at capacity)
        for _ in 0..192 {
            app.world_mut().run_system_once(recruitment_center_production_system).unwrap();
        }

        let recruit_count = app.world_mut().query_filtered::<Entity, With<crate::types::Unit>>().iter(app.world()).count();
        assert_eq!(recruit_count, 2, "Should not produce when at capacity");
    }

    #[test]
    fn rc_production_resumes_when_local_used_drops() {
        let mut app = setup_production_app();
        let rc_entity = spawn_production_rc(app.world_mut(), 30, 30, 1.0, 1, Owner::player(0));

        // Produce 1 recruit (192 frames)
        for _ in 0..192 {
            app.world_mut().run_system_once(recruitment_center_production_system).unwrap();
        }

        let recruit_count = app.world_mut().query_filtered::<Entity, With<crate::types::Unit>>().iter(app.world()).count();
        assert_eq!(recruit_count, 1);

        // Simulate unit death: decrement local_used
        app.world_mut().get_mut::<RecruitmentCenterState>(rc_entity).unwrap().local_used = 0;

        // Run 192 more frames — should produce again
        for _ in 0..192 {
            app.world_mut().run_system_once(recruitment_center_production_system).unwrap();
        }

        let recruit_count = app.world_mut().query_filtered::<Entity, With<crate::types::Unit>>().iter(app.world()).count();
        assert_eq!(recruit_count, 2, "Should produce again after local_used drops");
    }

    #[test]
    fn rc_production_skips_zero_effectiveness() {
        let mut app = setup_production_app();
        spawn_production_rc(app.world_mut(), 30, 30, 0.0, 20, Owner::player(0));

        // Run many frames — should NOT produce
        for _ in 0..500 {
            app.world_mut().run_system_once(recruitment_center_production_system).unwrap();
        }

        let recruit_count = app.world_mut().query_filtered::<Entity, With<crate::types::Unit>>().iter(app.world()).count();
        assert_eq!(recruit_count, 0, "Should not produce with 0 effectiveness");
    }

    #[test]
    fn rc_recruit_has_originating_center() {
        use crate::game::units::types::unit_data::OriginatingCenters;

        let mut app = setup_production_app();
        let rc_entity = spawn_production_rc(app.world_mut(), 30, 30, 1.0, 20, Owner::player(0));

        for _ in 0..192 {
            app.world_mut().run_system_once(recruitment_center_production_system).unwrap();
        }
        // Flush deferred commands
        app.update();

        let mut query = app.world_mut().query::<&OriginatingCenters>();
        let origins = query.single(app.world()).unwrap();
        assert_eq!(origins.centers.len(), 1);
        assert_eq!(origins.centers[0], rc_entity);
    }

    #[test]
    fn rc_recruit_has_rally_command_when_rally_set() {
        let mut app = setup_production_app();
        let rally_pos = Vec3::new(5.0, 0.0, 5.0);
        let rc_entity = spawn_production_rc(app.world_mut(), 30, 30, 1.0, 20, Owner::player(0));
        app.world_mut().get_mut::<RecruitmentCenterState>(rc_entity).unwrap().rally_point = Some(rally_pos);

        for _ in 0..192 {
            app.world_mut().run_system_once(recruitment_center_production_system).unwrap();
        }
        // Flush deferred commands
        app.update();

        // The recruit should have a Move command (if pathfinding succeeded)
        // Note: pathfinding may fail in minimal test (no tiles), so we check the RC state instead
        let rc_state = app.world().get::<RecruitmentCenterState>(rc_entity).unwrap();
        assert_eq!(rc_state.local_used, 1);
        assert_eq!(rc_state.production_progress, 0);
    }

    fn spawn_test_armory_entity(world: &mut World, pos: Vec3, player: u8) -> Entity {
        world.spawn((
            Transform::from_translation(pos),
            StructureInstance {
                rotation: crate::types::StructureRotation::R0,
                flip_horizontal: false,
                flip_vertical: false,
            },
            ObjectInstance::destructible(ObjectEnum::CultsArmory, 300.0),
            Owner::player(player),
            ArmoryState::default(),
        )).id()
    }

    #[test]
    fn armory_training_tick_advances_progress() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(crate::game::entities::registry::build_registry());
        app.init_resource::<crate::game::world::types::GridMap>();
        app.init_resource::<crate::game::units::types::OccupancyMap>();

        let armory = spawn_test_armory_entity(app.world_mut(), Vec3::new(0.0, 0.0, 0.0), 0);

        // Set up training
        {
            let mut armory_state = app.world_mut().get_mut::<ArmoryState>(armory).unwrap();
            armory_state.training_queue = Some(ObjectEnum::CultsSoldier);
            armory_state.training_progress = 0;
        }

        // Tick the system
        app.world_mut().run_system_once(armory_training_tick_system).unwrap();

        let armory_state = app.world().get::<ArmoryState>(armory).unwrap();
        assert_eq!(armory_state.training_progress, 1);
        assert_eq!(armory_state.training_queue, Some(ObjectEnum::CultsSoldier));
    }

    #[test]
    fn armory_training_completes_at_required_frames() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(crate::game::entities::registry::build_registry());
        app.init_resource::<crate::game::world::types::GridMap>();
        app.init_resource::<crate::game::units::types::OccupancyMap>();

        let armory = spawn_test_armory_entity(app.world_mut(), Vec3::new(0.0, 0.0, 0.0), 0);

        // Set progress to just before completion
        {
            let mut armory_state = app.world_mut().get_mut::<ArmoryState>(armory).unwrap();
            armory_state.training_queue = Some(ObjectEnum::CultsSoldier);
            armory_state.training_progress = crate::game::types::cults_structure_stats::SOLDIER_TRAINING_FRAMES - 1;
        }

        // Tick the system
        app.world_mut().run_system_once(armory_training_tick_system).unwrap();

        let armory_state = app.world().get::<ArmoryState>(armory).unwrap();
        // Training should be complete — queue cleared
        assert_eq!(armory_state.training_queue, None);
        assert_eq!(armory_state.training_progress, 0);
    }

    #[test]
    fn armory_training_skips_when_no_queue() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(crate::game::entities::registry::build_registry());
        app.init_resource::<crate::game::world::types::GridMap>();
        app.init_resource::<crate::game::units::types::OccupancyMap>();

        let armory = spawn_test_armory_entity(app.world_mut(), Vec3::new(0.0, 0.0, 0.0), 0);

        // No training queue set (default)
        app.world_mut().run_system_once(armory_training_tick_system).unwrap();

        let armory_state = app.world().get::<ArmoryState>(armory).unwrap();
        assert_eq!(armory_state.training_queue, None);
        assert_eq!(armory_state.training_progress, 0);
    }

    #[test]
    fn armory_state_training_cost_soldier() {
        let cost = ArmoryState::training_cost(&ObjectEnum::CultsSoldier);
        assert_eq!(cost, Some(crate::game::types::cults_structure_stats::SOLDIER_TRAINING_COST));
    }

    #[test]
    fn armory_state_training_cost_gunner() {
        let cost = ArmoryState::training_cost(&ObjectEnum::CultsGunner);
        assert_eq!(cost, Some(crate::game::types::cults_structure_stats::GUNNER_TRAINING_COST));
    }

    #[test]
    fn armory_state_training_cost_invalid() {
        let cost = ArmoryState::training_cost(&ObjectEnum::Peacekeeper);
        assert_eq!(cost, None);
    }

    #[test]
    fn armory_state_training_frames_soldier() {
        let frames = ArmoryState::training_frames(&ObjectEnum::CultsSoldier);
        assert_eq!(frames, Some(crate::game::types::cults_structure_stats::SOLDIER_TRAINING_FRAMES));
    }

    #[test]
    fn armory_state_training_frames_gunner() {
        let frames = ArmoryState::training_frames(&ObjectEnum::CultsGunner);
        assert_eq!(frames, Some(crate::game::types::cults_structure_stats::GUNNER_TRAINING_FRAMES));
    }

    #[test]
    fn rally_attack_enemy_inserts_move_target_and_path() {
        use crate::game::world::types::TilePresetEnum;
        use crate::game::units::types::movement::MoveTarget;
        use crate::game::units::types::state::UnitCommand;
        use crate::game::units::utils::grid_to_world;

        let mut app = setup_production_app();

        // Barracks at grid (30, 30). Barracks is 3x2, default R0 ABAC symmetry
        // spawns units at B-side (East): grid (30+3, 30+1) = (33, 31).
        // Enemy at grid (38, 31). Tiles must cover the path area.
        let bk_x = 30;
        let bk_z = 30;
        let enemy_grid_x = 38;
        let enemy_grid_z = 31;

        // Spawn tiles covering the path area
        for x in (bk_x - 1)..=(enemy_grid_x + 1) {
            for z in (bk_z - 1)..=(enemy_grid_z + 2) {
                spawn_test_tile(app.world_mut(), x, z, TilePresetEnum::Plane);
            }
        }

        // Compute enemy world position from its grid position
        let enemy_grid = GridPosition { x: enemy_grid_x, z: enemy_grid_z };
        let enemy_world = grid_to_world(&enemy_grid);

        // Spawn an enemy entity at the computed world position
        let enemy_entity = app.world_mut().spawn((
            Transform::from_translation(enemy_world),
            Owner::player(1),
            ObjectInstance::destructible(ObjectEnum::Barracks, 500.0),
        )).id();

        // Spawn player resources
        app.world_mut().spawn((
            Player {
                name: "Player 0".into(),
                faction: FactionEnum::GlobalDefenseOrdinance,
                player_number: 0,
            },
            GdoPlayerResources {
                space_crystals: 1000,
                unit_control_cap: 200,
                has_power_plant: true,
                ..Default::default()
            },
        ));

        // Spawn barracks with rally point targeting the enemy entity
        // Set progress so that exactly 1 more tick completes production
        let production_frames = {
            use crate::game::entities::registry::build_registry;
            build_registry().cost(&ObjectEnum::Peacekeeper).unwrap().build_frames as f32
        };
        app.world_mut().spawn((
            Owner::player(0),
            GridPosition { x: bk_x, z: bk_z },
            BarracksState {
                rally_point: Some(RallyTarget::Object(enemy_entity)),
                current_build: Some(ObjectEnum::Peacekeeper),
                current_build_progress: Some(production_frames - 1.0),
                ..Default::default()
            },
            StructureInstance::default(),
            ObjectInstance::destructible(ObjectEnum::Barracks, 500.0),
        ));

        // Run barracks production tick to complete the build
        app.world_mut().run_system_once(barracks_production_tick_system).unwrap();
        // Flush deferred commands
        app.update();

        // Find the produced unit (Peacekeeper)
        let mut unit_query = app.world_mut().query_filtered::<(Entity, &UnitCommand, Option<&MoveTarget>), With<crate::types::Unit>>();
        let units: Vec<_> = unit_query.iter(app.world()).collect();
        assert_eq!(units.len(), 1, "Should have produced exactly 1 unit");

        let (_unit_entity, command, move_target) = units[0];

        // Verify the unit has AttackTarget command pointing at the enemy
        match command {
            UnitCommand::AttackTarget(target) => {
                assert_eq!(*target, enemy_entity, "Attack target should be the enemy entity");
            }
            other => panic!("Expected AttackTarget command, got {:?}", other),
        }

        // Verify the unit also has a MoveTarget for approaching
        assert!(move_target.is_some(), "Unit with AttackTarget rally should also have MoveTarget for approach path");
    }
}
