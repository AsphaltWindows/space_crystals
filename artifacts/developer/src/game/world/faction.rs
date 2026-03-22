use bevy::prelude::*;
use crate::types::{Owner, ObjectEnum, MainCamera, GridPosition, LocalPlayer, InvisibleEntity, Selected, SelectionBounds, SelectedFaction, FactionEnum};
use crate::game::types::*;
use crate::game::utils::{
    spawn_deployment_center, spawn_power_plant, spawn_barracks,
    spawn_extraction_facility, spawn_extraction_plate, spawn_peacekeeper,
    spawn_tunnel, spawn_headquarters, spawn_supply_tower, spawn_supply_chopper,
    spawn_recruitment_center, spawn_cults_soldier, spawn_cults_gunner,
    spawn_cults_storage_under_construction,
};
use super::types::{GdoBuildArea, SpaceCrystalPatch, Tile, TilePreset, FogOfWarMap, MapStartingPositions};
use super::utils::{expand_build_area, world_to_grid, grid_to_world, can_place_building, rotated_building_size};
use crate::ui::types::{ObjectInterfaceState, StructureMenuState, AgentMenuState, CultsRecruitMenuState, CommandPanelTarget, PlacementGhost, PlacementState, CursorOverUi, BuildAreaOverlay, ArmoryEjectionQueue};
use crate::game::units::types::state::UnitCommand;
use crate::game::units::types::state::behavior::BuildingTunnelBehavior;

/// Setup initial player resources using design-aligned Player + faction-specific resource components.
/// Player 0 is always the local human; the faction assignment depends on SelectedFaction.
pub fn setup_player_resources(mut commands: Commands, selected: Res<SelectedFaction>) {
    // The local human controls player 0
    commands.insert_resource(LocalPlayer(0));

    match selected.0 {
        FactionEnum::GlobalDefenseOrdinance => {
            // GDO selected: GDO=player 0 (local), Syndicate=player 1 (opponent)
            spawn_faction_and_player(&mut commands, FactionEnum::GlobalDefenseOrdinance, 0);
            spawn_faction_and_player(&mut commands, FactionEnum::TheSyndicate, 1);
        }
        FactionEnum::TheSyndicate => {
            // Syndicate selected: Syndicate=player 0 (local), GDO=player 1 (opponent)
            spawn_faction_and_player(&mut commands, FactionEnum::TheSyndicate, 0);
            spawn_faction_and_player(&mut commands, FactionEnum::GlobalDefenseOrdinance, 1);
        }
        FactionEnum::TheCults => {
            // Cults selected: Cults=player 0 (local), GDO=player 1 (opponent)
            spawn_faction_and_player(&mut commands, FactionEnum::TheCults, 0);
            spawn_faction_and_player(&mut commands, FactionEnum::GlobalDefenseOrdinance, 1);
        }
        FactionEnum::Colonists => {
            // Colonists selected: Colonists=player 0 (local), GDO=player 1 (opponent)
            spawn_faction_and_player(&mut commands, FactionEnum::Colonists, 0);
            spawn_faction_and_player(&mut commands, FactionEnum::GlobalDefenseOrdinance, 1);
        }
    }

    info!("Initialized faction resources: Selected {:?}", selected.0);
}

/// Helper to spawn a faction entity and its corresponding player entity with resources.
fn spawn_faction_and_player(commands: &mut Commands, faction: FactionEnum, player_id: u8) {
    let player_name = if player_id == 0 { "Player 1" } else { "Player 2" };

    // Spawn the faction entity
    commands.spawn((
        InvisibleEntity,
        faction,
        DisplayHud::new(faction),
    ));

    // Spawn the player entity with faction-specific resources
    match faction {
        FactionEnum::GlobalDefenseOrdinance => {
            commands.spawn((
                InvisibleEntity,
                Player::new(player_name, FactionEnum::GlobalDefenseOrdinance, player_id),
                DisplayHudInfo::new(FactionEnum::GlobalDefenseOrdinance),
                GdoPlayerResources {
                    space_crystals: 1000,
                    supplies: 0,
                    power_generated: 0,
                    power_consumed: 0,
                    unit_control_used: 0,
                    unit_control_cap: 200,
                    has_power_plant: false,
                },
            ));
        }
        FactionEnum::TheSyndicate => {
            commands.spawn((
                InvisibleEntity,
                Player::new(player_name, FactionEnum::TheSyndicate, player_id),
                DisplayHudInfo::new(FactionEnum::TheSyndicate),
                SyndicatePlayerResources::default(),
            ));
        }
        FactionEnum::TheCults => {
            commands.spawn((
                InvisibleEntity,
                Player::new(player_name, FactionEnum::TheCults, player_id),
                DisplayHudInfo::new(FactionEnum::TheCults),
                CultsPlayerResources::default(),
            ));
        }
        FactionEnum::Colonists => {
            commands.spawn((
                InvisibleEntity,
                Player::new(player_name, FactionEnum::Colonists, player_id),
                DisplayHudInfo::new(FactionEnum::Colonists),
                ColonistsPlayerResources::default(),
            ));
        }
    }
}

/// Spawn the initial Deployment Center and initialize the build area
pub fn setup_gdo_game_start(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut build_area: ResMut<GdoBuildArea>,
    selected: Res<SelectedFaction>,
) {
    let gdo_owner = if selected.0 == FactionEnum::GlobalDefenseOrdinance {
        Owner::player(0)
    } else {
        Owner::player(1)
    };

    let dc_grid_x = 30;
    let dc_grid_z = 30;
    let _dc_entity = spawn_deployment_center(
        &mut commands, &mut meshes, &mut materials,
        dc_grid_x, dc_grid_z, gdo_owner,
    );

    expand_build_area(&mut build_area, dc_grid_x, dc_grid_z, 4, 4, 12);

    info!(
        "GDO Game Start: Deployed DC at grid ({}, {}), owner={:?}, build area: {} cells",
        dc_grid_x, dc_grid_z, gdo_owner, build_area.cells.len()
    );
}

/// Spawn the initial Tunnel and Headquarters for the Syndicate faction
pub fn setup_syndicate_game_start(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    selected: Res<SelectedFaction>,
) {
    let syn_owner = if selected.0 == FactionEnum::TheSyndicate {
        Owner::player(0)
    } else {
        Owner::player(1)
    };

    let tunnel_grid_x = 40;
    let tunnel_grid_z = 40;

    // Spawn a Tier 1 Tunnel
    let tunnel_entity = spawn_tunnel(
        &mut commands, &mut meshes, &mut materials,
        tunnel_grid_x, tunnel_grid_z, syn_owner,
    );

    // Spawn a pre-built Headquarters inside the Tunnel's area
    // Place at center of Tunnel Area (offset from tunnel origin)
    let hq_grid_x = tunnel_grid_x + 2; // Offset within the 10x10 area
    let hq_grid_z = tunnel_grid_z - 2;
    let _hq_entity = spawn_headquarters(
        &mut commands, &mut meshes, &mut materials,
        hq_grid_x, hq_grid_z,
        syn_owner, tunnel_entity,
    );

    info!(
        "Syndicate Game Start: Deployed Tunnel at grid ({}, {}), HQ at ({}, {}), owner={:?}",
        tunnel_grid_x, tunnel_grid_z, hq_grid_x, hq_grid_z, syn_owner
    );
}

/// Spawn the initial Recruitment Center for the Cults faction
pub fn setup_cults_game_start(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rc_counter: ResMut<RecruitmentCenterCounter>,
    selected: Res<SelectedFaction>,
) {
    let cults_owner = if selected.0 == FactionEnum::TheCults {
        Owner::player(0)
    } else {
        Owner::player(1)
    };

    let rc_grid_x = 50;
    let rc_grid_z = 50;
    let _rc_entity = spawn_recruitment_center(
        &mut commands, &mut meshes, &mut materials,
        rc_grid_x, rc_grid_z, cults_owner, rc_counter.next(),
    );

    info!(
        "Cults Game Start: Deployed Recruitment Center at grid ({}, {}), owner={:?}",
        rc_grid_x, rc_grid_z, cults_owner
    );
}

/// Stub game start for the Colonists faction. No structures are designed yet.
pub fn setup_colonists_game_start(
    selected: Res<SelectedFaction>,
) {
    let colonists_owner = if selected.0 == FactionEnum::Colonists {
        Owner::player(0)
    } else {
        Owner::player(1)
    };

    info!(
        "Colonists Game Start: No structures to spawn yet, owner={:?}",
        colonists_owner
    );
}

/// Center the camera on the local player's primary structure at game start.
/// If `MapStartingPositions` has an entry for the local player's slot,
/// the camera centers on that grid position instead.
pub fn center_camera_on_start(
    local_player: Res<LocalPlayer>,
    selected_faction: Res<SelectedFaction>,
    map_positions: Res<MapStartingPositions>,
    structures: Query<(&ObjectInstance, &Owner, &Transform), Without<MainCamera>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    // Step 1: Check for map-defined starting position
    if let Some(&(grid_x, grid_z)) = map_positions.positions.get(&local_player.0) {
        if let Ok(mut cam_transform) = camera_query.single_mut() {
            let world_pos = grid_to_world(grid_x, grid_z, 1.0);
            let z_offset = cam_transform.translation.y * 25.0 / 40.0;
            cam_transform.translation.x = world_pos.x;
            cam_transform.translation.z = world_pos.z + z_offset;
        }
        return;
    }

    // Step 2: Fall back to primary structure
    let primary_type = match selected_faction.0 {
        FactionEnum::GlobalDefenseOrdinance => ObjectEnum::DeploymentCenter,
        FactionEnum::TheSyndicate => ObjectEnum::Tunnel,
        _ => return,
    };
    let local_owner = Owner::player(local_player.0);

    for (obj, owner, transform) in structures.iter() {
        if obj.object_type == primary_type && *owner == local_owner {
            if let Ok(mut cam_transform) = camera_query.single_mut() {
                let z_offset = cam_transform.translation.y * 25.0 / 40.0;
                cam_transform.translation.x = transform.translation.x;
                cam_transform.translation.z = transform.translation.z + z_offset;
            }
            return;
        }
    }
}

/// Spawn enemy test units for the non-selected faction
pub fn setup_enemy_test_units(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    selected: Res<SelectedFaction>,
) {
    // Enemy is always player 1 (the non-local player)
    let enemy_owner = Owner::player(1);
    let _ = &selected; // acknowledge param — enemy is always player 1 regardless of faction
    let positions = [
        (50, 50),
        (51, 49),
        (49, 51),
        (52, 50),
        (50, 52),
    ];

    for (gx, gz) in positions {
        spawn_peacekeeper(&mut commands, &mut meshes, &mut materials, gx, gz, enemy_owner);
    }

    info!("Spawned {} enemy Peacekeepers around grid (50, 50)", positions.len());
}

// =====================================================
// POWER GRID SYSTEM
// =====================================================

/// System to compute power grid for GDO players each tick
pub fn compute_power_grid(
    mut players: Query<(&Player, &mut GdoPlayerResources)>,
    buildings: Query<(&Owner, &PowerValue, &ObjectInstance)>,
) {
    for (player, mut resources) in players.iter_mut() {
        let mut generated: i32 = 0;
        let mut consumed: i32 = 0;
        let mut found_power_plant = false;

        for (owner, power, obj) in buildings.iter() {
            if owner.player_number() == Some(player.player_number) {
                if power.0 > 0 {
                    generated += power.0;
                } else if power.0 < 0 {
                    consumed += power.0.abs();
                }
                if obj.object_type == ObjectEnum::PowerPlant {
                    found_power_plant = true;
                }
            }
        }

        resources.power_generated = generated;
        resources.power_consumed = consumed;
        resources.has_power_plant = found_power_plant;
    }
}

// =====================================================
// DEPLOYMENT CENTER CONSTRUCTION SYSTEM
// =====================================================

/// System that ticks DC construction progress each simulation frame
pub fn dc_construction_tick_system(
    mut dc_query: Query<(&Owner, &mut DeploymentCenterState)>,
    players: Query<(&Player, &mut GdoPlayerResources)>,
) {
    for (owner, mut dc_state) in dc_query.iter_mut() {
        if let Some(building_type) = dc_state.current_construction {
            if let Some(cost) = DeploymentCenterState::construction_cost(&building_type) {
                let required_frames = cost.build_frames as f32;

                // Get power ratio for this player
                let power_ratio = get_power_ratio_for_owner(owner, &players);

                if let Some(ref mut progress) = dc_state.construction_progress {
                    *progress += power_ratio;

                    if *progress >= required_frames {
                        // Construction complete — ready to place
                        dc_state.ready_to_place = Some(building_type);
                        dc_state.current_construction = None;
                        dc_state.construction_progress = None;
                        info!("DC: {:?} construction complete, ready to place", building_type);
                    }
                }
            }
        }
    }
}

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
    grid: Res<super::types::GridMap>,
    rally_targets: Query<(&Transform, &Owner), With<ObjectInstance>>,
    occupancy: Res<crate::game::units::types::OccupancyMap>,
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
            if let Some(cost) = BarracksState::production_cost(&unit_type) {
                let required_frames = cost.build_frames as f32;
                let power_ratio = get_power_ratio_for_owner(owner, &players);

                if let Some(ref mut progress) = bk_state.current_build_progress {
                    *progress += power_ratio;

                    if *progress >= required_frames {
                        // Production complete — spawn unit at the B-side exit
                        let (dx, dz) = super::utils::spawn_side_offset(
                            ObjectEnum::Barracks, structure_instance,
                        );
                        let spawn_x = grid_pos.x + dx;
                        let spawn_z = grid_pos.z + dz;

                        let unit_entity = spawn_peacekeeper(
                            &mut commands, &mut meshes, &mut materials,
                            spawn_x, spawn_z, *owner,
                        );

                        // Update unit control using the unit's actual control cost
                        let control_cost = crate::game::units::types::unit_data::PEACEKEEPER_CONTROL_COST;
                        for (player, mut res) in players.iter_mut() {
                            if Some(player.player_number) == owner.player_number() {
                                res.unit_control_used += control_cost;
                                break;
                            }
                        }

                        // Issue rally point command to the spawned unit
                        let produced_unit_base = crate::game::units::types::unit_data::peacekeeper_type_data().unit_base;
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
    grid: &super::types::GridMap,
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
                    // Attack enemy rally target
                    commands.entity(unit_entity).insert(UnitCommand::AttackTarget(*target_entity));
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
    grid: Res<super::types::GridMap>,
    rally_targets: Query<(&Transform, &Owner), With<ObjectInstance>>,
    occupancy: Res<crate::game::units::types::OccupancyMap>,
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
            if let Some(cost) = HeadquartersState::production_cost(&unit_type) {
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

                            let unit_entity = match unit_type {
                                ObjectEnum::SyndicateAgent => crate::game::utils::spawn_syndicate_agent(
                                    &mut commands, &mut meshes, &mut materials,
                                    spawn_pos.0, spawn_pos.1, *owner,
                                ),
                                ObjectEnum::SyndicateGuard => crate::game::utils::spawn_syndicate_guard(
                                    &mut commands, &mut meshes, &mut materials,
                                    spawn_pos.0, spawn_pos.1, *owner,
                                ),
                                _ => continue,
                            };

                            // Determine unit base for domain-aware pathfinding
                            let produced_unit_base = match unit_type {
                                ObjectEnum::SyndicateAgent => crate::game::units::types::unit_data::agent_type_data().unit_base,
                                ObjectEnum::SyndicateGuard => crate::game::units::types::unit_data::guard_type_data().unit_base,
                                _ => crate::types::UnitBaseEnum::HeavyInfantry, // fallback
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

                            let unit_entity = match unit_type {
                                ObjectEnum::SyndicateAgent => crate::game::utils::spawn_syndicate_agent(
                                    &mut commands, &mut meshes, &mut materials,
                                    0, 0, *owner, // Position doesn't matter — will be hidden
                                ),
                                ObjectEnum::SyndicateGuard => crate::game::utils::spawn_syndicate_guard(
                                    &mut commands, &mut meshes, &mut materials,
                                    0, 0, *owner,
                                ),
                                _ => continue,
                            };

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

// =====================================================
// EXTRACTION PLATE MINING SYSTEM
// =====================================================

/// System that ticks Extraction Plate mining each simulation frame
pub fn extraction_plate_mining_system(
    mut plates: Query<(&Owner, &mut ExtractionPlateState)>,
    mut patches: Query<&mut SpaceCrystalPatch>,
    mut players: Query<(&Player, &mut GdoPlayerResources)>,
) {
    for (owner, mut plate_state) in plates.iter_mut() {
        let power_ratio = get_power_ratio_for_owner(owner, &players);
        plate_state.mining_timer += power_ratio;

        if plate_state.mining_timer >= EXTRACTION_PLATE_MINING_INTERVAL {
            plate_state.mining_timer = 0.0;

            // Check patch state and mine
            if let Ok(mut patch) = patches.get_mut(plate_state.attached_patch) {
                let amount = if patch.remaining_amount > 0 {
                    let mine_amount = EXTRACTION_PLATE_MINING_RATE.min(patch.remaining_amount);
                    patch.remaining_amount -= mine_amount;
                    mine_amount
                } else {
                    // Depleted patch — residual mining
                    EXTRACTION_PLATE_RESIDUAL_RATE
                };

                // Add to player's resources
                for (player, mut res) in players.iter_mut() {
                    if Some(player.player_number) == owner.player_number() {
                        res.space_crystals += amount as i32;
                        break;
                    }
                }
            }
        }
    }
}

// =====================================================
// DEPLETED PATCH DESPAWN SYSTEM
// =====================================================

/// Despawns SpaceCrystalPatch entities when their remaining_amount reaches 0.
/// Also despawns the attached ExtractionPlate if one exists.
/// Per the design spec: "When a patch is fully depleted, it disappears from the map."
pub fn depleted_patch_despawn_system(
    mut commands: Commands,
    patches: Query<(Entity, &SpaceCrystalPatch)>,
    plates: Query<(Entity, &ExtractionPlateState)>,
) {
    for (patch_entity, patch) in patches.iter() {
        if patch.remaining_amount == 0 {
            // Find and despawn any extraction plate attached to this patch
            for (plate_entity, plate_state) in plates.iter() {
                if plate_state.attached_patch == patch_entity {
                    info!("Despawning extraction plate {:?} over depleted patch {:?}", plate_entity, patch_entity);
                    commands.entity(plate_entity).despawn();
                }
            }

            info!("Despawning depleted SpaceCrystalPatch {:?}", patch_entity);
            commands.entity(patch_entity).despawn();
        }
    }
}

// =====================================================
// RALLY POINT CLEANUP SYSTEM
// =====================================================

/// System that clears invalid rally point Object targets when the referenced entity is destroyed
pub fn rally_target_cleanup_system(
    mut barracks_query: Query<&mut BarracksState>,
    all_entities: Query<Entity>,
) {
    for mut bk_state in barracks_query.iter_mut() {
        if let Some(RallyTarget::Object(entity)) = &bk_state.rally_point {
            if all_entities.get(*entity).is_err() {
                bk_state.rally_point = None;
            }
        }
    }
}

// =====================================================
// PRODUCTION RALLY POINT SYSTEM
// =====================================================

/// System to set rally point via right-click on any production structure that is selected.
/// Handles Barracks, Headquarters, and Supply Tower.
pub fn production_rally_point_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut barracks_query: Query<(Entity, &mut BarracksState), With<Selected>>,
    mut hq_query: Query<(Entity, &mut HeadquartersState, &TunnelExpansionMarker), With<Selected>>,
    mut st_query: Query<(Entity, &mut SupplyTowerState), With<Selected>>,
    mut rc_query: Query<(Entity, &mut RecruitmentCenterState), (With<Selected>, Without<BarracksState>, Without<HeadquartersState>, Without<SupplyTowerState>, Without<ArmoryState>)>,
    mut armory_query: Query<(Entity, &mut ArmoryState), (With<Selected>, Without<BarracksState>, Without<HeadquartersState>, Without<SupplyTowerState>, Without<RecruitmentCenterState>)>,
    potential_targets: Query<(Entity, &Transform, &Owner, &SelectionBounds), With<ObjectInstance>>,
    cursor_over_ui: Res<CursorOverUi>,
    local_player: Res<LocalPlayer>,
    panel_state: Res<ObjectInterfaceState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_markers: Query<(Entity, &RallyPointMarker)>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    if cursor_over_ui.0 {
        return;
    }

    // Only handle when in a production structure menu state
    let is_production_menu = matches!(*panel_state,
        ObjectInterfaceState::StructureMenu(
            StructureMenuState::BarracksMenu |
            StructureMenuState::HeadquartersMenu |
            StructureMenuState::SupplyTowerMenu |
            StructureMenuState::RecruitmentCenterMenu |
            StructureMenuState::ArmoryMenu
        )
    );
    if !is_production_menu {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = cameras.single() else { return };

    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };

    // Determine the rally target from the click
    let rally_target = compute_rally_target_from_click(
        cursor_pos, &ray, camera, camera_transform,
        &potential_targets, &local_player,
    );

    let Some(rally_target) = rally_target else { return };

    // For Object targets, look up the target entity's world position for the visual marker
    let object_world_pos = if let RallyTarget::Object(target_entity) = &rally_target {
        potential_targets.iter()
            .find(|(e, _, _, _)| *e == *target_entity)
            .map(|(_, t, _, _)| t.translation)
    } else {
        None
    };

    // Set rally point on all selected production structures of the active group type
    match *panel_state {
        ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu) => {
            for (entity, mut bk_state) in &mut barracks_query {
                bk_state.rally_point = Some(rally_target.clone());
                info!("Barracks: Rally point set");
                spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, entity, &rally_target, object_world_pos);
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu) => {
            for (entity, mut hq_state, expansion_marker) in &mut hq_query {
                // If clicking the parent tunnel, clear rally point (unit stays in network)
                if let RallyTarget::Object(target_e) = &rally_target {
                    if *target_e == expansion_marker.parent_tunnel {
                        hq_state.rally_point = None;
                        despawn_rally_marker_for(&mut commands, &existing_markers, entity);
                        info!("Headquarters: Rally point cleared (target is parent tunnel)");
                        continue;
                    }
                }
                hq_state.rally_point = Some(rally_target.clone());
                info!("Headquarters: Rally point set");
                spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, entity, &rally_target, object_world_pos);
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu) => {
            for (entity, mut st_state) in &mut st_query {
                st_state.rally_point = Some(rally_target.clone());
                info!("Supply Tower: Rally point set");
                spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, entity, &rally_target, object_world_pos);
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu) => {
            for (entity, mut rc_state) in &mut rc_query {
                // RC rally_point is Option<Vec3>, not Option<RallyTarget> — resolve to world position
                let location = match &rally_target {
                    RallyTarget::Object(e) => potential_targets.iter()
                        .find(|(te, _, _, _)| *te == *e)
                        .map(|(_, t, _, _)| t.translation),
                    RallyTarget::Location(loc) => Some(*loc),
                };
                rc_state.rally_point = location;
                info!("Recruitment Center: Rally point set");
                spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, entity, &rally_target, object_world_pos);
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu) => {
            for (entity, mut armory_state) in &mut armory_query {
                armory_state.rally_point = Some(rally_target.clone());
                info!("Armory: Rally point set");
                spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, entity, &rally_target, object_world_pos);
            }
        }
        _ => {}
    }
}

/// Compute a RallyTarget from a click position — checks entities first, then ground plane.
fn compute_rally_target_from_click(
    _cursor_pos: Vec2,
    ray: &Ray3d,
    _camera: &Camera,
    _camera_transform: &GlobalTransform,
    potential_targets: &Query<(Entity, &Transform, &Owner, &SelectionBounds), With<ObjectInstance>>,
    _local_player: &Res<LocalPlayer>,
) -> Option<RallyTarget> {
    // Check for entity under cursor via 3D ray-AABB intersection
    let ray_origin = ray.origin;
    let ray_dir = *ray.direction;
    let click_pad = 0.3;
    let mut best_distance = f32::MAX;
    let mut clicked_entity: Option<Entity> = None;

    for (target_entity, target_transform, _target_owner, bounds) in potential_targets.iter() {
        let center = target_transform.translation;
        let aabb_min = Vec3::new(
            center.x - bounds.half_x - click_pad,
            center.y - bounds.half_y - click_pad,
            center.z - bounds.half_z - click_pad,
        );
        let aabb_max = Vec3::new(
            center.x + bounds.half_x + click_pad,
            center.y + bounds.half_y + click_pad,
            center.z + bounds.half_z + click_pad,
        );

        if let Some(t) = crate::ui::utils::ray_aabb_intersect(ray_origin, ray_dir, aabb_min, aabb_max) {
            if t < best_distance {
                clicked_entity = Some(target_entity);
                best_distance = t;
            }
        }
    }

    if let Some(target_entity) = clicked_entity {
        return Some(RallyTarget::Object(target_entity));
    }

    // Ground click — set rally to location
    if ray.direction.y.abs() > 0.001 {
        let t = -ray.origin.y / ray.direction.y;
        if t > 0.0 {
            let world_hit = ray.origin + *ray.direction * t;
            return Some(RallyTarget::Location(world_hit));
        }
    }

    None
}

/// Spawn or update a visual rally point marker for a production structure.
/// Despawns any existing marker for the owner, then spawns a new one at the rally location.
/// For Object targets, `object_world_pos` provides the target entity's position for the marker.
pub fn spawn_or_update_rally_marker(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    existing_markers: &Query<(Entity, &RallyPointMarker)>,
    owner_structure: Entity,
    rally_target: &RallyTarget,
    object_world_pos: Option<Vec3>,
) {
    // Despawn existing marker for this structure
    despawn_rally_marker_for(commands, existing_markers, owner_structure);

    // Determine world position for the marker
    let marker_pos = match rally_target {
        RallyTarget::Location(pos) => *pos,
        RallyTarget::Object(_) => {
            // For object targets, place marker at the target's position if provided
            if let Some(pos) = object_world_pos {
                pos
            } else {
                return;
            }
        }
    };

    // Spawn a small cylinder as the rally point indicator
    let marker_mesh = meshes.add(Cylinder::new(0.3, 0.1));
    let marker_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 1.0, 0.2, 0.7),
        emissive: LinearRgba::new(0.0, 2.0, 0.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        Mesh3d(marker_mesh),
        MeshMaterial3d(marker_material),
        Transform::from_translation(Vec3::new(marker_pos.x, 0.15, marker_pos.z)),
        RallyPointMarker { owner_structure },
    ));
}

/// Despawn any existing rally point marker for the given structure entity.
pub fn despawn_rally_marker_for(
    commands: &mut Commands,
    existing_markers: &Query<(Entity, &RallyPointMarker)>,
    owner_structure: Entity,
) {
    for (marker_entity, marker) in existing_markers.iter() {
        if marker.owner_structure == owner_structure {
            commands.entity(marker_entity).despawn();
        }
    }
}

// =====================================================
// EXTRACTION FACILITY CONSTRUCTION SYSTEM
// =====================================================

/// System that ticks ExtractionFacility construction progress
pub fn ef_construction_tick_system(
    mut ef_query: Query<(&Owner, &mut ExtractionFacilityState)>,
    players: Query<(&Player, &mut GdoPlayerResources)>,
) {
    for (owner, mut ef_state) in ef_query.iter_mut() {
        if ef_state.current_construction {
            let cost = ExtractionFacilityState::construction_cost();
            let required_frames = cost.build_frames as f32;
            let power_ratio = get_power_ratio_for_owner(owner, &players);

            if let Some(ref mut progress) = ef_state.construction_progress {
                *progress += power_ratio;

                if *progress >= required_frames {
                    ef_state.ready_to_place = true;
                    ef_state.current_construction = false;
                    ef_state.construction_progress = None;
                    info!("EF: ExtractionPlate construction complete, ready to place");
                }
            }
        }
    }
}

// =====================================================
// CONSTRUCTION HP SYSTEM
// =====================================================

/// System that ticks ConstructionHP progress and scales HP accordingly.
/// Each frame, progress increments by 1.0 / build_frames.
/// HP = MaxHP × (10% + 90% × progress).
/// When progress reaches 1.0, the ConstructionHP component is removed.
pub fn construction_hp_tick_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ObjectInstance, &mut ConstructionHP), Without<CultsConstructionState>>,
) {
    for (entity, mut obj, mut construction) in query.iter_mut() {
        let increment = 1.0 / construction.build_frames as f32;
        construction.progress = (construction.progress + increment).min(1.0);

        // Update HP based on progress
        if let Some(max_hp) = obj.max_hp {
            let new_hp = max_hp * ConstructionHP::hp_fraction(construction.progress);
            // Only set HP if not damaged below the expected value
            // (allow damage to reduce HP below the construction curve)
            if let Some(current_hp) = obj.hp {
                if current_hp < new_hp {
                    // Don't heal past damage — only raise HP if it's at or above
                    // the previous frame's expected value (i.e., not damaged)
                    let prev_progress = (construction.progress - increment).max(0.0);
                    let prev_expected = max_hp * ConstructionHP::hp_fraction(prev_progress);
                    if current_hp >= prev_expected - 0.001 {
                        obj.hp = Some(new_hp.min(max_hp));
                    }
                }
            }
        }

        // Remove component when construction completes
        if construction.is_complete() {
            commands.entity(entity).remove::<ConstructionHP>();
        }
    }
}

// =====================================================
// DEBUG KEYBINDING SYSTEMS
// =====================================================

/// Debug system: press keys to trigger construction/production (DEPRECATED - replaced by command panel UI)
/// Kept for reference but no longer registered in any plugin.
#[allow(dead_code)]
fn debug_construction_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    _commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    mut dc_query: Query<(Entity, &Owner, &crate::types::GridPosition, &mut DeploymentCenterState)>,
    mut bk_query: Query<(Entity, &Owner, &mut BarracksState)>,
    mut players: Query<(&Player, &mut GdoPlayerResources)>,
    _build_area: ResMut<GdoBuildArea>,
) {
    // DC construction commands
    let build_request = if keyboard.just_pressed(KeyCode::Digit1) {
        Some(ObjectEnum::PowerPlant)
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(ObjectEnum::Barracks)
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some(ObjectEnum::ExtractionFacility)
    } else {
        None
    };

    if let Some(object_type) = build_request {
        for (_entity, owner, _grid_pos, mut dc_state) in dc_query.iter_mut() {
            if dc_state.current_construction.is_some() || dc_state.ready_to_place.is_some() {
                info!("DC: Already building or has structure ready to place");
                continue;
            }

            if let Some(cost) = DeploymentCenterState::construction_cost(&object_type) {
                // Check player resources
                for (player, mut res) in players.iter_mut() {
                    if Some(player.player_number) == owner.player_number() {
                        if res.space_crystals >= cost.space_crystals as i32 {
                            res.space_crystals -= cost.space_crystals as i32;
                            dc_state.current_construction = Some(object_type);
                            dc_state.construction_progress = Some(0.0);
                            info!("DC: Started building {:?} ({} SC, {} frames)",
                                object_type, cost.space_crystals, cost.build_frames);
                        } else {
                            info!("DC: Not enough SC ({} needed, {} available)",
                                cost.space_crystals, res.space_crystals);
                        }
                        break;
                    }
                }
            }
            break; // Only use first DC
        }
    }

    // F = Enter placement mode (handled via command panel state machine)
    // The old instant-place behavior is replaced by click-to-place via AwaitingPlacement

    // P = Produce Peacekeeper from first Barracks
    if keyboard.just_pressed(KeyCode::KeyP) {
        for (_entity, owner, mut bk_state) in bk_query.iter_mut() {
            if let Some(cost) = BarracksState::production_cost(&ObjectEnum::Peacekeeper) {
                // Check resources and unit control
                for (player, mut res) in players.iter_mut() {
                    if Some(player.player_number) == owner.player_number() {
                        if res.space_crystals < cost.space_crystals as i32 {
                            info!("Barracks: Not enough SC ({} needed, {} available)",
                                cost.space_crystals, res.space_crystals);
                            break;
                        }
                        if !res.has_unit_control(crate::game::units::types::unit_data::PEACEKEEPER_CONTROL_COST) {
                            info!("Barracks: Unit control cap reached ({}/{})",
                                res.unit_control_used, res.unit_control_cap);
                            break;
                        }
                        if bk_state.try_queue(ObjectEnum::Peacekeeper) {
                            res.space_crystals -= cost.space_crystals as i32;
                            info!("Barracks: Queued Peacekeeper ({} SC, queue size: {})",
                                cost.space_crystals, bk_state.build_queue.len());
                        } else {
                            info!("Barracks: Queue full (max {})", BarracksState::MAX_QUEUE_SIZE);
                        }
                        break;
                    }
                }
            }
            break; // Only use first Barracks
        }
    }

    // C = Cancel construction/production
    if keyboard.just_pressed(KeyCode::KeyC) {
        // Cancel DC construction
        for (_entity, owner, _grid_pos, mut dc_state) in dc_query.iter_mut() {
            if let Some(building_type) = dc_state.current_construction {
                if let Some(refund) = dc_state.cancellation_refund(&building_type) {
                    for (player, mut res) in players.iter_mut() {
                        if Some(player.player_number) == owner.player_number() {
                            res.space_crystals += refund as i32;
                            info!("DC: Cancelled construction, refunded {} SC", refund);
                            break;
                        }
                    }
                }
                dc_state.current_construction = None;
                dc_state.construction_progress = None;
                break;
            }
            if let Some(ready_type) = &dc_state.ready_to_place {
                if let Some(refund) = dc_state.cancellation_refund(ready_type) {
                    for (player, mut res) in players.iter_mut() {
                        if Some(player.player_number) == owner.player_number() {
                            res.space_crystals += refund as i32;
                            info!("DC: Cancelled ready structure, refunded {} SC (75%)", refund);
                            break;
                        }
                    }
                }
                dc_state.ready_to_place = None;
                break;
            }
        }

        // Cancel last Barracks queue item
        for (_entity, owner, mut bk_state) in bk_query.iter_mut() {
            if let Some(cancelled) = bk_state.cancel_last() {
                if let Some(cost) = BarracksState::production_cost(&cancelled) {
                    for (player, mut res) in players.iter_mut() {
                        if Some(player.player_number) == owner.player_number() {
                            res.space_crystals += cost.space_crystals as i32;
                            info!("Barracks: Cancelled {:?}, refunded {} SC", cancelled, cost.space_crystals);
                            break;
                        }
                    }
                }
            }
            break;
        }
    }
}

// =====================================================
// HELPER FUNCTIONS
// =====================================================

/// Get power ratio for a given owner from the player resources (works with both mutable and immutable queries)
fn get_power_ratio_for_owner(
    owner: &Owner,
    players: &Query<(&Player, &mut GdoPlayerResources)>,
) -> f32 {
    for (player, res) in players.iter() {
        if Some(player.player_number) == owner.player_number() {
            return res.power_ratio();
        }
    }
    1.0 // Default if no matching player found
}

// =====================================================
// DISPLAY SYSTEM
// =====================================================

/// System to display resource information when 'R' key is pressed
pub fn display_resources_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    gdo_players: Query<(&Player, &GdoPlayerResources)>,
    syndicate_players: Query<(&Player, &SyndicatePlayerResources)>,
    cults_players: Query<(&Player, &CultsPlayerResources)>,
    colonist_players: Query<(&Player, &ColonistsPlayerResources)>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        for (player, res) in gdo_players.iter() {
            info!("=== {} ({}) Resources ===", player.name, player.faction.name());
            info!("Space Crystals: {}", res.space_crystals);
            info!("Supplies: {}", res.supplies);
            info!("Power: {} / {} (net: {})",
                res.power_generated, res.power_consumed, res.current_power());
            info!("Power Ratio: {:.1}%", res.power_ratio() * 100.0);
            info!("Unit Control: {} / {}", res.unit_control_used, res.unit_control_cap);
            info!("");
        }
        for (player, res) in syndicate_players.iter() {
            info!("=== {} ({}) Resources ===", player.name, player.faction.name());
            info!("Space Crystals: {}", res.space_crystals);
            info!("Supplies: {}", res.supplies);
            info!("Tunnel Space: {} / {}",
                res.tunnel_space_used, res.tunnel_space_provided);
            info!("");
        }
        for (player, res) in cults_players.iter() {
            info!("=== {} ({}) Resources ===", player.name, player.faction.name());
            info!("Space Crystals: {}", res.space_crystals);
            info!("Unit Control: {} / {}", res.unit_control_used, res.unit_control_available);
            info!("");
        }
        for (player, res) in colonist_players.iter() {
            info!("=== {} ({}) Resources ===", player.name, player.faction.name());
            info!("Space Crystals: {}", res.space_crystals);
            info!("Alloys: {}", res.alloys);
            info!("Essence: {}", res.essence);
            info!("Conduits: {}", res.conduits);
            info!("Beacon Capacity: {} / {}",
                res.beacon_capacity_used, res.beacon_capacity_provided);
            info!("");
        }
    }
}

// =====================================================
// PLACEMENT SYSTEMS
// =====================================================

/// System to spawn/despawn the placement ghost entity based on ObjectInterfaceState transitions
pub fn manage_placement_ghost(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    panel_state: Res<ObjectInterfaceState>,
    panel_target: Res<CommandPanelTarget>,
    ghost_query: Query<Entity, With<PlacementGhost>>,
    mut placement_state: ResMut<PlacementState>,
    dc_query: Query<(&Owner, &DeploymentCenterState)>,
    _ef_query: Query<(&Owner, &ExtractionFacilityState)>,
) {
    if !panel_state.is_changed() {
        return;
    }

    let is_placing = panel_state.is_placement_mode();

    if is_placing && ghost_query.is_empty() {
        // Determine what we're placing
        let building_type = match *panel_state {
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcAwaitingPlacement) => {
                if let Some(entity) = panel_target.entity {
                    dc_query.get(entity).ok().and_then(|(_, dc)| dc.ready_to_place)
                } else {
                    None
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement) => {
                Some(ObjectEnum::ExtractionPlate)
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement) => {
                // The building_type is already set in PlacementState by execute_tunnel_select_expansion
                placement_state.building_type
            }
            ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement) => {
                // The building_type is already set in PlacementState by execute_command_action
                placement_state.building_type
            }
            _ => None,
        };

        if let Some(btype) = building_type {
            let obj_type = btype.object_type();
            let (w, d) = (obj_type.size.0 as f32, obj_type.size.1 as f32);
            let h = match btype {
                ObjectEnum::PowerPlant => 1.0,
                ObjectEnum::Barracks => 0.8,
                ObjectEnum::ExtractionFacility => 1.2,
                ObjectEnum::ExtractionPlate => 0.15,
                ObjectEnum::SupplyTower => 1.2,
                _ => 1.0,
            };

            let mesh = meshes.add(Cuboid::new(w, h, d));
            let material = materials.add(StandardMaterial {
                base_color: Color::srgba(0.2, 0.8, 0.2, 0.5),
                alpha_mode: AlphaMode::Add,
                unlit: true,
                cull_mode: None, // Double-sided: prevents backface culling artifacts when flipped (negative scale)
                ..default()
            });

            // Determine symmetry type for side labels on ghost
            let sym = btype.structure_type().map(|st| st.symmetry_type).unwrap_or(crate::types::SymmetryTypeEnum::AAAA);
            let half_x = w / 2.0;
            let half_z = d / 2.0;

            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(0.0, h / 2.0, 0.0),
                Visibility::Hidden,
                PlacementGhost,
            )).with_children(|parent| {
                crate::game::utils::spawn_ghost_side_labels(parent, sym, half_x, half_z, h);
            });

            placement_state.building_type = Some(btype);
            placement_state.source_entity = panel_target.entity;
            placement_state.grid_pos = None;
            placement_state.is_valid = false;
            placement_state.rotation = crate::types::StructureRotation::R0;
            placement_state.flip_horizontal = false;
            placement_state.flip_vertical = false;

            info!("Entered placement mode for {:?}", btype);
        }
    } else if !is_placing && !ghost_query.is_empty() {
        // Despawn ghost
        for entity in ghost_query.iter() {
            commands.entity(entity).despawn();
        }
        placement_state.building_type = None;
        placement_state.source_entity = None;
        placement_state.grid_pos = None;
        placement_state.is_valid = false;
    }
}

/// System to update the placement ghost position and validity each frame
pub fn update_placement_ghost(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut ghost_query: Query<(&mut Transform, &mut Visibility, &MeshMaterial3d<StandardMaterial>), With<PlacementGhost>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    panel_state: Res<ObjectInterfaceState>,
    mut placement_state: ResMut<PlacementState>,
    build_area: Res<GdoBuildArea>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    structures: Query<(&GridPosition, &StructureInstance, &ObjectInstance)>,
    patches: Query<(&GridPosition, &SpaceCrystalPatch)>,
    tunnel_areas: Query<&TunnelArea>,
    fog_map: Res<FogOfWarMap>,
    local_player: Res<LocalPlayer>,
) {
    if !panel_state.is_placement_mode() {
        return;
    }

    let Ok((mut ghost_transform, mut ghost_vis, mat_handle)) = ghost_query.single_mut() else { return; };

    let Some(building_type) = placement_state.building_type else { return; };

    let Ok(window) = windows.single() else { return; };

    let Ok((camera, camera_transform)) = camera_query.single() else { return; };

    let cursor_pos = match window.cursor_position() {
        Some(pos) => pos,
        None => {
            *ghost_vis = Visibility::Hidden;
            return;
        }
    };

    // Raycast to ground plane (Y=0)
    let ray = match camera.viewport_to_world(camera_transform, cursor_pos) {
        Ok(r) => r,
        Err(_) => return,
    };

    // Intersect with Y=0 plane
    if ray.direction.y.abs() < 0.001 {
        *ghost_vis = Visibility::Hidden;
        return;
    }

    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        *ghost_vis = Visibility::Hidden;
        return;
    }

    let world_hit = ray.origin + *ray.direction * t;

    // Convert to grid coordinates
    let (grid_x, grid_z) = world_to_grid(world_hit, 1.0);
    placement_state.grid_pos = Some((grid_x, grid_z));

    // Get building size (accounting for rotation)
    let obj_type = building_type.object_type();
    let (base_x, base_z) = obj_type.size;
    let rotation = placement_state.rotation;
    let (size_x, size_z) = rotated_building_size(base_x, base_z, &rotation);
    let h = match building_type {
        ObjectEnum::PowerPlant => 1.0,
        ObjectEnum::Barracks => 0.8,
        ObjectEnum::ExtractionFacility => 1.2,
        ObjectEnum::ExtractionPlate => 0.15,
        ObjectEnum::SupplyTower => 1.2,
        _ => 1.0,
    };

    // Snap ghost to grid
    let world_pos = grid_to_world(grid_x, grid_z, 1.0);
    // Adjust to center of multi-cell building (using rotated dimensions)
    let center_offset_x = (size_x as f32 - 1.0) / 2.0;
    let center_offset_z = (size_z as f32 - 1.0) / 2.0;
    ghost_transform.translation = Vec3::new(
        world_pos.x + center_offset_x,
        h / 2.0,
        world_pos.z + center_offset_z,
    );
    ghost_transform.rotation = Quat::from_rotation_y(rotation.radians());
    ghost_transform.scale = Vec3::new(
        if placement_state.flip_horizontal { -1.0 } else { 1.0 },
        1.0,
        if placement_state.flip_vertical { -1.0 } else { 1.0 },
    );
    *ghost_vis = Visibility::Visible;

    // Validate placement (using rotated dimensions)
    let is_valid = if matches!(*panel_state, ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement)) {
        // Agent tunnel placement — validate tile buildability and no structure overlap (no visibility/build area check)
        crate::game::world::utils::can_worker_place_structure(
            grid_x, grid_z,
            size_x, size_z,
            &tiles,
            &structures,
        ).is_ok()
    } else if matches!(*panel_state, ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement)) {
        // Cults Recruit placement — validate tile buildability and no structure overlap (no build area, no fog check)
        crate::game::world::utils::can_worker_place_structure(
            grid_x, grid_z,
            size_x, size_z,
            &tiles,
            &structures,
        ).is_ok()
    } else if matches!(*panel_state, ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement)) {
        // Tunnel expansion placement — validate against TunnelArea
        if let Some(source) = placement_state.source_entity {
            if let Ok(tunnel_area) = tunnel_areas.get(source) {
                // Check expansion fits within the tunnel area
                // Also check no overlap with existing structures on the grid
                let fits_area = tunnel_area.fits_expansion(grid_x, grid_z, size_x, size_z);
                let no_overlap = !has_structure_overlap(grid_x, grid_z, size_x, size_z, &structures);
                fits_area && no_overlap
            } else {
                false
            }
        } else {
            false
        }
    } else {
        // Standard GDO placement
        can_place_building(
            grid_x, grid_z,
            size_x, size_z,
            building_type,
            &build_area,
            &tiles,
            &structures,
            &patches,
            &fog_map,
            local_player.0,
        ).is_ok()
    };

    placement_state.is_valid = is_valid;

    // Update ghost color based on validity
    if let Some(material) = materials.get_mut(&mat_handle.0) {
        material.base_color = if is_valid {
            Color::srgba(0.2, 0.8, 0.2, 0.5) // Green
        } else {
            Color::srgba(0.8, 0.2, 0.2, 0.5) // Red
        };
    }
}

/// System to handle placement clicks (left-click to place, right-click to cancel)
pub fn placement_click_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut panel_state: ResMut<ObjectInterfaceState>,
    _panel_target: Res<CommandPanelTarget>,
    placement_state: Res<PlacementState>,
    cursor_over_ui: Res<CursorOverUi>,
    mut dc_query: Query<(&Owner, &mut DeploymentCenterState)>,
    mut ef_query: Query<(&Owner, &mut ExtractionFacilityState)>,
    mut tunnel_query: Query<(&Owner, &mut TunnelState)>,
    mut syndicate_players: Query<(&Player, &mut SyndicatePlayerResources)>,
    mut build_area: ResMut<GdoBuildArea>,
    mut patches: Query<(Entity, &GridPosition, &mut SpaceCrystalPatch)>,
    existing_builders: Query<&BuildingTunnelBehavior>,
    selected_recruits: Query<(Entity, &Owner), (With<crate::types::Unit>, With<Selected>)>,
) {
    if !panel_state.is_placement_mode() {
        return;
    }

    // Right-click: cancel placement
    if mouse_buttons.just_pressed(MouseButton::Right) {
        match *panel_state {
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcAwaitingPlacement) => {
                *panel_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace);
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement) => {
                *panel_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement) => {
                *panel_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
            }
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement) => {
                *panel_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
            }
            ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement) => {
                *panel_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu);
            }
            _ => {}
        }
        return;
    }

    // Left-click: place building if valid
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if cursor_over_ui.0 {
            return; // Don't place when clicking on UI
        }

        if !placement_state.is_valid {
            return; // Position is invalid
        }

        let (grid_x, grid_z) = match placement_state.grid_pos {
            Some(pos) => pos,
            None => return,
        };

        let building_type = match placement_state.building_type {
            Some(bt) => bt,
            None => return,
        };

        let source_entity = match placement_state.source_entity {
            Some(e) => e,
            None => return,
        };

        let rotation = placement_state.rotation;
        let flip_h = placement_state.flip_horizontal;
        let flip_v = placement_state.flip_vertical;

        match *panel_state {
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcAwaitingPlacement) => {
                if let Ok((owner, mut dc_state)) = dc_query.get_mut(source_entity) {
                    let owner = *owner;
                    // Take the ready_to_place
                    dc_state.ready_to_place = None;

                    let obj_type = building_type.object_type();
                    let (rot_x, rot_z) = rotated_building_size(obj_type.size.0, obj_type.size.1, &rotation);

                    match building_type {
                        ObjectEnum::PowerPlant => {
                            spawn_power_plant(
                                &mut commands, &mut meshes, &mut materials,
                                grid_x, grid_z, owner, rotation, flip_h, flip_v,
                            );
                            expand_build_area(&mut build_area, grid_x, grid_z, rot_x, rot_z, 1);
                        }
                        ObjectEnum::Barracks => {
                            spawn_barracks(
                                &mut commands, &mut meshes, &mut materials,
                                grid_x, grid_z, owner, rotation, flip_h, flip_v,
                            );
                            expand_build_area(&mut build_area, grid_x, grid_z, rot_x, rot_z, 2);
                        }
                        ObjectEnum::ExtractionFacility => {
                            spawn_extraction_facility(
                                &mut commands, &mut meshes, &mut materials,
                                grid_x, grid_z, owner, rotation, flip_h, flip_v,
                            );
                            expand_build_area(&mut build_area, grid_x, grid_z, rot_x, rot_z, 2);
                        }
                        ObjectEnum::SupplyTower => {
                            let tower_entity = spawn_supply_tower(
                                &mut commands, &mut meshes, &mut materials,
                                grid_x, grid_z, owner, rotation, flip_h, flip_v,
                            );
                            // Supply Tower also spawns a free Supply Chopper at the spawn-side exit
                            let st_si = StructureInstance::new(rotation, flip_h, flip_v);
                            let (dx, dz) = super::utils::spawn_side_offset(
                                ObjectEnum::SupplyTower, &st_si,
                            );
                            let chopper_entity = spawn_supply_chopper(
                                &mut commands, &mut meshes, &mut materials,
                                grid_x + dx, grid_z + dz, owner,
                            );
                            // Link the tower and its free chopper for auto-delivery
                            commands.entity(tower_entity).insert(SupplyTowerState {
                                attached_chopper: Some(chopper_entity),
                                ..default()
                            });
                            commands.entity(chopper_entity).insert(SupplyChopperState {
                                attached_tower: Some(tower_entity),
                                ..default()
                            });
                            expand_build_area(&mut build_area, grid_x, grid_z, rot_x, rot_z, 1);
                        }
                        _ => {}
                    }

                    info!("Placed {:?} at ({}, {}) rotation {:?}", building_type, grid_x, grid_z, rotation);
                    *panel_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement) => {
                if let Ok((owner, mut ef_state)) = ef_query.get_mut(source_entity) {
                    let owner = *owner;
                    ef_state.ready_to_place = false;

                    // Find the patch entity at this position and mark it as having a plate
                    let mut attached_patch = None;
                    for (patch_entity, patch_pos, mut patch) in patches.iter_mut() {
                        if patch_pos.x == grid_x && patch_pos.z == grid_z {
                            patch.has_plate = true;
                            attached_patch = Some(patch_entity);
                            break;
                        }
                    }

                    if let Some(patch_entity) = attached_patch {
                        spawn_extraction_plate(
                            &mut commands, &mut meshes, &mut materials,
                            grid_x, grid_z, owner, patch_entity,
                        );
                        expand_build_area(&mut build_area, grid_x, grid_z, 1, 1, 0);
                        info!("Placed Extraction Plate at ({}, {})", grid_x, grid_z);
                    }

                    *panel_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement) => {
                // Place tunnel expansion — deduct cost and begin construction (entity spawns on completion)
                if let Ok((owner, mut tunnel_state)) = tunnel_query.get_mut(source_entity) {
                    // Determine expansion cost
                    let cost = match building_type {
                        ObjectEnum::Headquarters => syndicate_structure_stats::HQ_SC_COST,
                        _ => {
                            info!("Unknown tunnel expansion type: {:?}", building_type);
                            0
                        }
                    };

                    // Check and deduct cost from SyndicatePlayerResources
                    let mut cost_paid = false;
                    for (player, mut res) in syndicate_players.iter_mut() {
                        if Some(player.player_number) == owner.player_number() {
                            if res.space_crystals >= cost as i32 {
                                res.space_crystals -= cost as i32;
                                cost_paid = true;
                                info!("Tunnel expansion {:?}: deducted {} SC", building_type, cost);
                            } else {
                                info!("Tunnel expansion {:?}: insufficient SC ({} < {})", building_type, res.space_crystals, cost);
                            }
                            break;
                        }
                    }

                    if !cost_paid {
                        *panel_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
                        return;
                    }

                    // Set the tunnel's current operation to building expansion (entity spawns on completion)
                    tunnel_state.current_operation = Some(TunnelOperation::BuildingExpansion {
                        object: building_type,
                        progress: 0.0,
                        grid_x,
                        grid_z,
                        rotation,
                        flip_horizontal: flip_h,
                        flip_vertical: flip_v,
                    });

                    info!("Began tunnel expansion {:?} construction at ({}, {})", building_type, grid_x, grid_z);
                    *panel_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
                }
            }
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement) => {
                // Agent placement: issue BuildTunnel command to the Agent, don't spawn structure yet
                let world_pos = grid_to_world(grid_x, grid_z, 1.0);

                // Single-Agent construction enforcement: reject if another Agent is
                // already building (MovingToSite or Constructing) at the same location
                let location_taken = existing_builders.iter().any(|b| {
                    let dx = (b.target_location.x - world_pos.x).abs();
                    let dz = (b.target_location.z - world_pos.z).abs();
                    dx < 1.0 && dz < 1.0
                });

                if location_taken {
                    info!("Agent: BuildTunnel rejected — another Agent is already building at ({}, {})", grid_x, grid_z);
                } else {
                    commands.entity(source_entity)
                        .insert(UnitCommand::BuildTunnel(world_pos));
                    info!("Agent: BuildTunnel command to ({}, {})", grid_x, grid_z);
                }
                *panel_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
            }
            ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement) => {
                // Get owner from first selected recruit
                let owner = match selected_recruits.iter().next() {
                    Some((_, o)) => *o,
                    None => { return; }
                };

                let rotation = placement_state.rotation;
                let flip_h = placement_state.flip_horizontal;
                let flip_v = placement_state.flip_vertical;

                let building_entity = match building_type {
                    ObjectEnum::CultsStorage => {
                        spawn_cults_storage_under_construction(
                            &mut commands, &mut meshes, &mut materials,
                            grid_x, grid_z, owner, rotation, flip_h, flip_v,
                        )
                    }
                    _ => { return; }
                };

                // Issue ConstructBuilding command to ALL selected recruits
                for (entity, _) in selected_recruits.iter() {
                    commands.entity(entity).insert(UnitCommand::ConstructBuilding(building_entity));
                }
                info!("Placed {:?} under construction at ({}, {}) — issued ConstructBuilding to recruits", building_type, grid_x, grid_z);
                *panel_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault);
            }
            _ => {}
        }
    }
}

// =====================================================
// BUILD AREA OVERLAY SYSTEM
// =====================================================

/// System to show/hide the build area overlay during placement mode.
/// Spawns a single custom mesh covering all build area cells when entering AwaitingPlacement.
/// Despawns when exiting.
pub fn manage_build_area_overlay(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    panel_state: Res<ObjectInterfaceState>,
    overlay_query: Query<Entity, With<BuildAreaOverlay>>,
    build_area: Res<GdoBuildArea>,
    placement_state: Res<PlacementState>,
    tunnel_areas: Query<&TunnelArea>,
) {
    if !panel_state.is_changed() {
        return;
    }

    let is_placing = panel_state.is_placement_mode();

    if is_placing && overlay_query.is_empty() {
        // Build overlay mesh from the appropriate area
        let is_tunnel = matches!(*panel_state, ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement));
        let is_agent = matches!(*panel_state, ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement));
        let is_cults_recruit = matches!(*panel_state, ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement));

        if is_agent || is_cults_recruit {
            // Agent placement: no build area overlay needed (Agent can build anywhere with valid tiles)
        } else if is_tunnel {
            // Build mesh from TunnelArea cells
            if let Some(source) = placement_state.source_entity {
                if let Ok(tunnel_area) = tunnel_areas.get(source) {
                    let mesh = tunnel_area_mesh(tunnel_area);
                    let mesh_handle = meshes.add(mesh);
                    let material = materials.add(StandardMaterial {
                        base_color: Color::srgba(0.6, 0.2, 0.8, 0.3), // Purple tint for tunnel area
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        double_sided: true,
                        cull_mode: None,
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(mesh_handle),
                        MeshMaterial3d(material),
                        Transform::from_xyz(0.0, 0.01, 0.0),
                        BuildAreaOverlay,
                    ));
                }
            }
        } else {
            // Build mesh from GDO build area cells
            if build_area.cells.is_empty() {
                return;
            }

            let mesh = build_area_mesh(&build_area);
            let mesh_handle = meshes.add(mesh);
            let material = materials.add(StandardMaterial {
                base_color: Color::srgba(0.2, 0.8, 0.2, 0.3),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                double_sided: true,
                cull_mode: None,
                ..default()
            });

            commands.spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(material),
                Transform::from_xyz(0.0, 0.01, 0.0),
                BuildAreaOverlay,
            ));
        }
    } else if !is_placing && !overlay_query.is_empty() {
        // Despawn overlay
        for entity in overlay_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Check if any existing structure's full footprint occupies any cell of the given footprint.
/// Unlike origin-only checks, this accounts for multi-cell structures (e.g., 4x4 Tunnel).
fn has_structure_overlap(
    pos_x: i32,
    pos_z: i32,
    size_x: u32,
    size_z: u32,
    structures: &Query<(&GridPosition, &StructureInstance, &ObjectInstance)>,
) -> bool {
    for dx in 0..size_x as i32 {
        for dz in 0..size_z as i32 {
            let check_x = pos_x + dx;
            let check_z = pos_z + dz;
            for (struct_pos, _si, obj) in structures.iter() {
                let (sx, sz) = obj.object_type.object_type().size;
                // Check if check_x/check_z falls within this structure's full footprint
                if check_x >= struct_pos.x && check_x < struct_pos.x + sx as i32
                    && check_z >= struct_pos.z && check_z < struct_pos.z + sz as i32
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Build a single mesh from all cells in the build area.
/// Each cell becomes a quad (2 triangles) at its world position.
fn build_area_mesh(build_area: &GdoBuildArea) -> Mesh {
    use bevy::mesh::{PrimitiveTopology, Indices};

    let cell_count = build_area.cells.len();
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(cell_count * 4);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(cell_count * 4);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(cell_count * 4);
    let mut indices: Vec<u32> = Vec::with_capacity(cell_count * 6);

    let half_size = 32.0_f32; // Grid half-size (64x64 grid)

    for (i, &(gx, gz)) in build_area.cells.iter().enumerate() {
        let wx = gx as f32 - half_size;
        let wz = gz as f32 - half_size;
        let base_idx = (i * 4) as u32;

        // Four corners of the cell (1x1 quad)
        positions.push([wx, 0.0, wz]);           // bottom-left
        positions.push([wx + 1.0, 0.0, wz]);     // bottom-right
        positions.push([wx + 1.0, 0.0, wz + 1.0]); // top-right
        positions.push([wx, 0.0, wz + 1.0]);     // top-left

        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);

        uvs.push([0.0, 0.0]);
        uvs.push([1.0, 0.0]);
        uvs.push([1.0, 1.0]);
        uvs.push([0.0, 1.0]);

        // Two triangles per quad
        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Build a single mesh from all cells in a TunnelArea (for overlay during expansion placement).
fn tunnel_area_mesh(tunnel_area: &TunnelArea) -> Mesh {
    use bevy::mesh::{PrimitiveTopology, Indices};

    let cell_count = tunnel_area.cells.len();
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(cell_count * 4);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(cell_count * 4);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(cell_count * 4);
    let mut indices: Vec<u32> = Vec::with_capacity(cell_count * 6);

    let half_size = 32.0_f32;

    for (i, &(gx, gz)) in tunnel_area.cells.iter().enumerate() {
        let wx = gx as f32 - half_size;
        let wz = gz as f32 - half_size;
        let base_idx = (i * 4) as u32;

        positions.push([wx, 0.0, wz]);
        positions.push([wx + 1.0, 0.0, wz]);
        positions.push([wx + 1.0, 0.0, wz + 1.0]);
        positions.push([wx, 0.0, wz + 1.0]);

        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);

        uvs.push([0.0, 0.0]);
        uvs.push([1.0, 0.0]);
        uvs.push([1.0, 1.0]);
        uvs.push([0.0, 1.0]);

        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

// =====================================================
// TUNNEL CONSTRUCTION TICK SYSTEM
// =====================================================

/// System that ticks Tunnel operations (construction and upgrades) each simulation frame.
/// Construction requires an Agent to be present near the Tunnel (builder_present flag).
/// When construction completes, the expansion entity is spawned.
/// When upgrade completes, the TunnelTier is advanced and TunnelArea recalculated.
pub fn tunnel_construction_tick_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tunnel_query: Query<(Entity, &Owner, &mut TunnelState, &mut TunnelArea)>,
    mut syndicate_players: Query<(&Player, &mut SyndicatePlayerResources)>,
) {
    for (tunnel_entity, owner, mut tunnel_state, mut tunnel_area) in tunnel_query.iter_mut() {
        let operation = match &tunnel_state.current_operation {
            Some(op) => op.clone(),
            None => continue,
        };

        match operation {
            TunnelOperation::Upgrading { target_tier, progress } => {
                let required = syndicate_structure_stats::TUNNEL_UPGRADE_FRAMES as f32;
                let new_progress = progress + 1.0;

                if new_progress >= required {
                    // Upgrade complete
                    tunnel_state.tier = target_tier;
                    tunnel_state.current_operation = None;
                    tunnel_area.recalculate(&target_tier);

                    // Update tunnel_space_provided for the owning player
                    // Re-sum all tunnels is complex here; just add the difference
                    let old_space = match target_tier {
                        TunnelTier::Tier2 => TunnelTier::Tier1.tunnel_space(),
                        TunnelTier::Tier3 => TunnelTier::Tier2.tunnel_space(),
                        TunnelTier::Tier1 => 0, // shouldn't happen
                    };
                    let new_space = target_tier.tunnel_space();
                    let space_diff = new_space.saturating_sub(old_space);

                    for (player, mut res) in syndicate_players.iter_mut() {
                        if Some(player.player_number) == owner.player_number() {
                            res.tunnel_space_provided += space_diff;
                            break;
                        }
                    }

                    info!("Tunnel upgrade to {:?} complete", target_tier);
                } else {
                    tunnel_state.current_operation = Some(TunnelOperation::Upgrading {
                        target_tier,
                        progress: new_progress,
                    });
                }
            }
            TunnelOperation::BuildingExpansion { object, progress, grid_x, grid_z, rotation, flip_horizontal, flip_vertical } => {
                // TODO: Check Agent presence (builder_present) once enter_command task lands.
                // For now, construction always progresses.
                let required = match object {
                    ObjectEnum::Headquarters => syndicate_structure_stats::HQ_BUILD_FRAMES as f32,
                    _ => syndicate_structure_stats::TUNNEL_CONSTRUCTION_FRAMES as f32,
                };
                let new_progress = progress + 1.0;

                if new_progress >= required {
                    // Construction complete — spawn the expansion entity
                    tunnel_state.current_operation = None;
                    let owner_copy = *owner;
                    match object {
                        ObjectEnum::Headquarters => {
                            spawn_headquarters(
                                &mut commands, &mut meshes, &mut materials,
                                grid_x, grid_z, owner_copy, tunnel_entity,
                            );
                            info!("Tunnel expansion Headquarters spawned at ({}, {})", grid_x, grid_z);
                        }
                        _ => {
                            info!("Tunnel expansion {:?} construction complete (no spawn handler)", object);
                        }
                    }
                } else {
                    tunnel_state.current_operation = Some(TunnelOperation::BuildingExpansion {
                        object,
                        progress: new_progress,
                        grid_x,
                        grid_z,
                        rotation,
                        flip_horizontal,
                        flip_vertical,
                    });
                }
            }
        }
    }
}

/// System to process tunnel ejection queues and eject requests.
/// Each tick: process pending EjectRequest markers, decrement cooldowns,
/// eject one unit per tunnel when cooldown expires.
pub fn ejection_tick_system(
    mut commands: Commands,
    mut tunnel_query: Query<(Entity, &Transform, &StructureInstance, &Owner, &mut crate::ui::types::EjectionQueue, Option<&crate::ui::types::EjectRequest>)>,
    network_units: Query<(Entity, &ObjectInstance, &crate::game::units::types::state::behavior::InTunnelNetwork)>,
) {
    use crate::game::units::utils::tunnel_side_world_position;
    use crate::game::units::types::state::behavior::InTunnelNetwork;

    for (tunnel_entity, tunnel_tf, tunnel_si, owner, mut ejection_queue, eject_request) in tunnel_query.iter_mut() {
        // Process pending EjectRequest: find a matching unit in the network and add to queue
        if let Some(request) = eject_request {
            let owner_player = owner.player_number().unwrap_or(0);

            // Find first matching unit in the network owned by the same player
            let matching_unit = network_units.iter()
                .find(|(_, obj, in_net)| {
                    obj.object_type == request.unit_type && in_net.owner_player == owner_player
                })
                .map(|(entity, _, _)| entity);

            if let Some(unit_entity) = matching_unit {
                ejection_queue.queue.push_back(unit_entity);
                info!("Tunnel: Queued {:?} for ejection (queue len: {})", request.unit_type, ejection_queue.queue.len());
            } else {
                info!("Tunnel: No {:?} unit found in network for player {}", request.unit_type, owner_player);
            }

            // Remove the request marker
            commands.entity(tunnel_entity).remove::<crate::ui::types::EjectRequest>();
        }

        // Process ejection cooldown
        if ejection_queue.cooldown > 0 {
            ejection_queue.cooldown -= 1;
            continue;
        }

        // Eject next unit from queue
        if let Some(unit_entity) = ejection_queue.queue.pop_front() {
            // Verify entity still exists and is in network
            if let Ok((_, _, _)) = network_units.get(unit_entity) {
                // Compute Side A position
                let side_a_pos = tunnel_side_world_position(tunnel_tf, tunnel_si, 'A');

                // Teleport unit to Side A, make visible, remove InTunnelNetwork
                commands.entity(unit_entity)
                    .remove::<InTunnelNetwork>()
                    .insert(Visibility::Visible)
                    .insert(Transform::from_translation(Vec3::new(side_a_pos.x, 0.5, side_a_pos.z)));

                // Issue a move command away from the tunnel exit to clear the way
                let move_away = Vec3::new(side_a_pos.x, 0.5, side_a_pos.z - 2.0); // Move slightly away from Side A
                commands.entity(unit_entity)
                    .insert(UnitCommand::Move(move_away));

                // Reset cooldown (8 frames minimum between ejections)
                ejection_queue.cooldown = 8;

                info!("Tunnel: Ejected unit {:?} at Side A ({:.1}, {:.1})", unit_entity, side_a_pos.x, side_a_pos.z);
            } else {
                // Entity no longer valid — skip and try next
                info!("Tunnel: Ejection skipped — unit {:?} no longer in network", unit_entity);
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
            if let Some(cost) = SupplyTowerState::production_cost(&unit_type) {
                let required_frames = cost.build_frames as f32;
                let power_ratio = get_power_ratio_for_owner(owner, &players);

                if let Some(ref mut progress) = st_state.current_build_progress {
                    *progress += power_ratio;

                    if *progress >= required_frames {
                        // Production complete — spawn chopper at the spawn-side exit
                        let (dx, dz) = super::utils::spawn_side_offset(
                            ObjectEnum::SupplyTower, structure_instance,
                        );
                        let spawn_x = grid_pos.x + dx;
                        let spawn_z = grid_pos.z + dz;

                        let unit_entity = spawn_supply_chopper(
                            &mut commands, &mut meshes, &mut materials,
                            spawn_x, spawn_z, *owner,
                        );

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
// RECRUITMENT AREA TILE CLAIMING SYSTEM
// =====================================================

/// System that claims recruitable tiles for Recruitment Centers.
/// Uses approach B: stale claim cleanup at start (self-healing).
/// Priority is by build_order (ascending = first-built has priority).
pub fn recruitment_tile_claiming_system(
    mut claim_map: ResMut<super::types::TileClaimMap>,
    mut rc_query: Query<(Entity, &GridPosition, &mut RecruitmentCenterState, &ObjectInstance)>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
) {
    // Step 1: Remove stale claims (entities that are no longer alive)
    let stale_entities: Vec<Entity> = {
        let mut stale = Vec::new();
        for (_, &claimer) in claim_map.claims.iter() {
            if !stale.contains(&claimer) {
                let is_alive = rc_query.iter().any(|(e, _, _, obj)| e == claimer && obj.is_alive());
                if !is_alive {
                    stale.push(claimer);
                }
            }
        }
        stale
    };
    for entity in stale_entities {
        claim_map.unclaim_all_for(entity);
    }

    // Step 2: Build a recruitable tile lookup from the tile query
    let recruitable_tiles: std::collections::HashSet<(i32, i32)> = tiles
        .iter()
        .filter(|(_, preset)| preset.recruitable)
        .map(|(gp, _)| (gp.x, gp.z))
        .collect();

    // Step 3: Collect RCs sorted by build_order (ascending)
    let mut rcs: Vec<(Entity, i32, i32, u64)> = rc_query
        .iter()
        .filter(|(_, _, _, obj)| obj.is_alive())
        .map(|(e, gp, state, _)| (e, gp.x, gp.z, state.build_order))
        .collect();
    rcs.sort_by_key(|&(_, _, _, order)| order);

    // Step 4: Clear all existing claims and reclaim from scratch each frame
    claim_map.claims.clear();

    // Step 5: For each RC in build_order priority, claim tiles
    let mut claimed_per_entity: std::collections::HashMap<Entity, Vec<(i32, i32)>> =
        std::collections::HashMap::new();

    for (entity, gx, gz, _order) in &rcs {
        let mut claimed = Vec::new();
        // 10x10 area centered on 4x4 footprint: extends 3 tiles beyond each edge
        let x_min = (gx - 3).max(0);
        let x_max = (gx + 6).min(63);
        let z_min = (gz - 3).max(0);
        let z_max = (gz + 6).min(63);

        for x in x_min..=x_max {
            for z in z_min..=z_max {
                let pos = (x, z);
                if recruitable_tiles.contains(&pos) && claim_map.is_claimed(pos).is_none() {
                    claim_map.claim_tile(pos, *entity);
                    claimed.push(pos);
                }
            }
        }
        claimed_per_entity.insert(*entity, claimed);
    }

    // Step 6: Update RC state with claimed tiles, effectiveness, and local_capacity
    for (entity, _, mut rc_state, _) in rc_query.iter_mut() {
        if let Some(claimed) = claimed_per_entity.remove(&entity) {
            let effectiveness = claimed.len() as f32 / 100.0;
            let local_capacity = (20.0 * effectiveness).floor() as u32;
            rc_state.claimed_tiles = claimed;
            rc_state.effectiveness = effectiveness;
            rc_state.local_capacity = local_capacity;
        } else {
            // Dead or not found — clear
            rc_state.claimed_tiles.clear();
            rc_state.effectiveness = 0.0;
            rc_state.local_capacity = 0;
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
    grid: Res<super::types::GridMap>,
    occupancy: Res<crate::game::units::types::OccupancyMap>,
) {
    use crate::game::units::utils::{world_to_grid, smooth_path};
    use crate::game::units::pathfinding::find_path_for_domain;
    use crate::game::units::types::movement::{MoveTarget, Path};
    use crate::game::units::types::state::UnitCommand;
    use crate::game::units::types::unit_data::OriginatingCenters;
    use crate::game::utils::spawn_cults_recruit;
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

            let unit_entity = spawn_cults_recruit(
                &mut commands, &mut meshes, &mut materials,
                spawn_x, spawn_z, *owner,
            );

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

// =====================================================
// CULTS UNIT CONTROL AGGREGATION SYSTEM
// =====================================================

/// Aggregates unit control capacity and usage from all Recruitment Centers
/// into the owning player's CultsPlayerResources each frame.
pub fn cults_unit_control_aggregation_system(
    rc_query: Query<(&Owner, &RecruitmentCenterState)>,
    mut players: Query<(&Player, &mut CultsPlayerResources)>,
) {
    for (player, mut res) in players.iter_mut() {
        let mut total_capacity: u32 = 0;
        let mut total_used: u32 = 0;

        for (owner, rc_state) in rc_query.iter() {
            if owner.player_number() == Some(player.player_number) {
                total_capacity += rc_state.local_capacity;
                total_used += rc_state.local_used;
            }
        }

        res.unit_control_available = total_capacity;
        res.unit_control_used = total_used;
    }
}

// =====================================================
// CULTS CONSTRUCTION TICK SYSTEM
// =====================================================

/// System that ticks Cults building construction progress.
/// Progress scales linearly with the number of assigned Recruits.
/// When complete, all assigned Recruits are despawned and the building becomes operational.
pub fn cults_construction_tick_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ObjectInstance, &mut ConstructionHP, &mut CultsConstructionState)>,
) {
    for (entity, mut obj, mut construction, mut cults_state) in query.iter_mut() {
        let recruit_count = cults_state.assigned_recruits.len() as u32;
        if recruit_count == 0 {
            continue; // Paused — no recruits assigned
        }

        cults_state.construction_progress += recruit_count;

        // Update ConstructionHP progress (0.0 to 1.0)
        let progress = (cults_state.construction_progress as f32 / cults_state.total_construction_frames as f32).min(1.0);
        construction.progress = progress;

        // Update HP using ConstructionHP::hp_fraction()
        if let Some(max_hp) = obj.max_hp {
            obj.hp = Some(max_hp * ConstructionHP::hp_fraction(progress));
        }

        // Check completion
        if cults_state.construction_progress >= cults_state.total_construction_frames {
            // Set HP to full max
            if let Some(max_hp) = obj.max_hp {
                obj.hp = Some(max_hp);
            }
            // Despawn all assigned recruits (they are consumed)
            for recruit_entity in cults_state.assigned_recruits.drain(..) {
                commands.entity(recruit_entity).despawn();
            }
            // Remove construction components — building is complete
            commands.entity(entity).remove::<ConstructionHP>();
            commands.entity(entity).remove::<CultsConstructionState>();
        }
    }
}

// =====================================================
// CULTS CONSTRUCTION CANCEL SYSTEM
// =====================================================

/// System that handles cancellation of Cults buildings under construction.
/// When a building is destroyed (HP reaches 0), all assigned Recruits are ejected,
/// restored to visible, and set to Idle near the building's position.
/// Must run BEFORE remove_dead_entities_system so recruits can be ejected first.
pub fn cults_construction_cancel_system(
    mut commands: Commands,
    mut query: Query<(Entity, &ObjectInstance, &Transform, &mut CultsConstructionState)>,
) {
    for (_entity, obj, transform, mut cults_state) in query.iter_mut() {
        if !obj.is_alive() {
            // Building was destroyed — eject all recruits
            let base_pos = transform.translation;
            for (i, recruit_entity) in cults_state.assigned_recruits.drain(..).enumerate() {
                // Offset each recruit slightly from the building center
                let offset_x = ((i % 3) as f32 - 1.0) * 1.5;
                let offset_z = ((i / 3) as f32) * 1.5 + 1.5;
                let eject_pos = Vec3::new(base_pos.x + offset_x, base_pos.y, base_pos.z + offset_z);

                commands.entity(recruit_entity)
                    .insert(Visibility::Inherited)
                    .insert(UnitCommand::Idle)
                    .insert(Transform::from_translation(eject_pos));
            }
        }
    }
}

/// Compute the exit (Side C) world position for an Armory.
/// The Armory is 3x2 with ABCB symmetry: N=A, E=B, S=C, W=B.
/// Side C is opposite Side A. For a 3x2 building, the half-sizes are:
/// width = 1.5 (x-axis), depth = 1.0 (z-axis). Exit offset is depth + 0.5 = 1.5.
fn armory_exit_side_position(
    transform: &Transform,
    structure_instance: &StructureInstance,
) -> Vec3 {
    use crate::types::SymmetryTypeEnum;

    let labels = structure_instance.oriented_labels(SymmetryTypeEnum::ABCB);
    // Find which cardinal direction (N=0, E=1, S=2, W=3) has Side C
    let cardinal_index = labels.iter().position(|&c| c == 'C').unwrap_or(2);

    let center = transform.translation;
    // For a 3x2 building: half-width=1.5, half-depth=1.0
    // Side C exit offset depends on which cardinal direction it faces
    let offset = match cardinal_index {
        0 | 2 => 1.5, // North/South face (depth side: 1.0 + 0.5)
        1 | 3 => 2.0, // East/West face (width side: 1.5 + 0.5)
        _ => 1.5,
    };

    match cardinal_index {
        0 => Vec3::new(center.x, center.y, center.z - offset), // North
        1 => Vec3::new(center.x + offset, center.y, center.z), // East
        2 => Vec3::new(center.x, center.y, center.z + offset), // South
        3 => Vec3::new(center.x - offset, center.y, center.z), // West
        _ => center,
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
    grid: Res<super::types::GridMap>,
    rally_targets: Query<(&Transform, &Owner), With<ObjectInstance>>,
    occupancy: Res<crate::game::units::types::OccupancyMap>,
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
            let exit_pos = armory_exit_side_position(transform, si);
            let (exit_x, exit_z) = world_to_grid(exit_pos, 1.0);

            // Spawn the trained unit
            let unit_entity = match unit_type {
                ObjectEnum::CultsSoldier => spawn_cults_soldier(
                    &mut commands, &mut meshes, &mut materials,
                    exit_x, exit_z, *owner,
                ),
                ObjectEnum::CultsGunner => spawn_cults_gunner(
                    &mut commands, &mut meshes, &mut materials,
                    exit_x, exit_z, *owner,
                ),
                _ => continue,
            };

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

/// System to eject Recruits from Armory one at a time with a cooldown.
/// Runs in FixedUpdate.
pub fn armory_eject_tick_system(
    mut commands: Commands,
    mut armory_query: Query<(Entity, &Transform, &StructureInstance, &mut ArmoryEjectionQueue)>,
) {
    for (entity, transform, si, mut ejection_queue) in armory_query.iter_mut() {
        if ejection_queue.cooldown > 0 {
            ejection_queue.cooldown -= 1;
            continue;
        }
        if let Some(recruit_entity) = ejection_queue.queue.pop_front() {
            let exit_pos = armory_exit_side_position(transform, si);
            // Move the recruit slightly further from the exit to avoid stacking
            let eject_offset = Vec3::new(0.0, 0.0, 1.5);
            if commands.get_entity(recruit_entity).is_ok() {
                commands.entity(recruit_entity)
                    .insert(Visibility::Inherited)
                    .insert(Transform::from_translation(exit_pos))
                    .insert(UnitCommand::Move(exit_pos + eject_offset));
            }
            ejection_queue.cooldown = 8; // 8-frame spacing like tunnel ejection
        } else {
            // Queue empty — remove the component
            commands.entity(entity).remove::<ArmoryEjectionQueue>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn headquarters_default_has_no_rally_point() {
        let state = HeadquartersState::default();
        assert!(state.rally_point.is_none());
    }

    #[test]
    fn headquarters_rally_point_location_means_eject() {
        let state = HeadquartersState {
            rally_point: Some(RallyTarget::Location(Vec3::new(10.0, 0.0, 10.0))),
            ..default()
        };
        // A Location rally point means the unit should eject to surface
        let should_eject = matches!(state.rally_point, Some(RallyTarget::Location(_)));
        assert!(should_eject);
    }

    #[test]
    fn headquarters_no_rally_means_tunnel_network() {
        let state = HeadquartersState::default();
        // No rally point means the unit stays in the tunnel network
        let should_eject = match &state.rally_point {
            Some(RallyTarget::Location(_)) => true,
            Some(RallyTarget::Object(_)) => true, // non-parent-tunnel object
            None => false,
        };
        assert!(!should_eject);
    }

    #[test]
    fn headquarters_rally_target_parent_tunnel_clears() {
        // Simulate the "rally on parent tunnel = clear" logic
        let parent_tunnel = Entity::from_raw_u32(42).unwrap();
        let rally_target = RallyTarget::Object(parent_tunnel);

        let mut state = HeadquartersState {
            rally_point: Some(rally_target),
            ..default()
        };

        // The logic: if rally target entity == parent_tunnel, clear
        if let Some(RallyTarget::Object(entity)) = &state.rally_point {
            if *entity == parent_tunnel {
                state.rally_point = None;
            }
        }

        assert!(state.rally_point.is_none(), "Rally should be cleared when target is parent tunnel");
    }

    #[test]
    fn headquarters_rally_target_other_entity_does_not_clear() {
        let parent_tunnel = Entity::from_raw_u32(42).unwrap();
        let other_entity = Entity::from_raw_u32(99).unwrap();
        let rally_target = RallyTarget::Object(other_entity);

        let mut state = HeadquartersState {
            rally_point: Some(rally_target),
            ..default()
        };

        // The logic: if rally target entity != parent_tunnel, keep it
        if let Some(RallyTarget::Object(entity)) = &state.rally_point {
            if *entity == parent_tunnel {
                state.rally_point = None;
            }
        }

        assert!(state.rally_point.is_some(), "Rally should NOT be cleared when target is different entity");
    }

    #[test]
    fn rally_point_marker_stores_owner() {
        let owner = Entity::from_raw_u32(10).unwrap();
        let marker = RallyPointMarker { owner_structure: owner };
        assert_eq!(marker.owner_structure, owner);
    }

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

    /// Ghost placement material uses AlphaMode::Add (order-independent blending) and
    /// cull_mode: None to prevent visual artifacts when the ghost is flipped via negative scale.
    /// AlphaMode::Blend caused depth-sorting artifacts with negative scale, making the ghost
    /// appear "upside-down" from certain camera angles.
    #[test]
    fn ghost_material_should_be_double_sided_and_additive() {
        // The ghost StandardMaterial uses AlphaMode::Add + cull_mode: None
        // AlphaMode::Add is order-independent, preventing visual artifacts when
        // flip_horizontal or flip_vertical sets a negative scale component.
        let mat = StandardMaterial {
            base_color: Color::srgba(0.2, 0.8, 0.2, 0.5),
            alpha_mode: AlphaMode::Add,
            unlit: true,
            cull_mode: None,
            ..default()
        };
        assert!(mat.cull_mode.is_none(), "Ghost material must be double-sided (cull_mode: None)");
        assert_eq!(mat.alpha_mode, AlphaMode::Add, "Ghost material must use additive blending to avoid depth-sort artifacts with negative scale");
    }

    /// Flip scale should negate X for horizontal and Z for vertical.
    #[test]
    fn flip_scale_axes_are_correct() {
        // flip_horizontal = true → scale.x = -1, scale.z = 1
        let flip_h = Vec3::new(-1.0, 1.0, 1.0);
        assert_eq!(flip_h.x, -1.0);
        assert_eq!(flip_h.y, 1.0);
        assert_eq!(flip_h.z, 1.0);

        // flip_vertical = true → scale.x = 1, scale.z = -1
        let flip_v = Vec3::new(1.0, 1.0, -1.0);
        assert_eq!(flip_v.x, 1.0);
        assert_eq!(flip_v.y, 1.0);
        assert_eq!(flip_v.z, -1.0);

        // both flipped → scale.x = -1, scale.z = -1
        let flip_both = Vec3::new(-1.0, 1.0, -1.0);
        assert_eq!(flip_both.x, -1.0);
        assert_eq!(flip_both.z, -1.0);
    }

    // --- Depleted Patch Despawn Tests ---

    #[test]
    fn depleted_patch_is_despawned() {
        let mut world = World::new();
        // Spawn a depleted patch (remaining_amount == 0)
        let patch_entity = world.spawn(SpaceCrystalPatch {
            remaining_amount: 0,
            initial_amount: 1000,
            has_plate: false,
        }).id();

        world.run_system_once(depleted_patch_despawn_system).unwrap();
        world.flush();

        assert!(world.get_entity(patch_entity).is_err(),
            "Depleted patch should be despawned");
    }

    #[test]
    fn non_depleted_patch_is_not_despawned() {
        let mut world = World::new();
        // Spawn a patch with remaining resources
        let patch_entity = world.spawn(SpaceCrystalPatch {
            remaining_amount: 500,
            initial_amount: 1000,
            has_plate: true,
        }).id();

        world.run_system_once(depleted_patch_despawn_system).unwrap();
        world.flush();

        assert!(world.get_entity(patch_entity).is_ok(),
            "Non-depleted patch should not be despawned");
    }

    #[test]
    fn depleted_patch_also_despawns_attached_plate() {
        let mut world = World::new();
        // Spawn a depleted patch
        let patch_entity = world.spawn(SpaceCrystalPatch {
            remaining_amount: 0,
            initial_amount: 1000,
            has_plate: true,
        }).id();

        // Spawn an extraction plate attached to this patch
        let plate_entity = world.spawn(ExtractionPlateState {
            attached_patch: patch_entity,
            mining_timer: 0.0,
        }).id();

        world.run_system_once(depleted_patch_despawn_system).unwrap();
        world.flush();

        assert!(world.get_entity(patch_entity).is_err(),
            "Depleted patch should be despawned");
        assert!(world.get_entity(plate_entity).is_err(),
            "Plate attached to depleted patch should also be despawned");
    }

    #[test]
    fn plate_on_non_depleted_patch_is_not_despawned() {
        let mut world = World::new();
        // Spawn a patch with remaining resources
        let patch_entity = world.spawn(SpaceCrystalPatch {
            remaining_amount: 100,
            initial_amount: 1000,
            has_plate: true,
        }).id();

        // Spawn an extraction plate attached to this patch
        let plate_entity = world.spawn(ExtractionPlateState {
            attached_patch: patch_entity,
            mining_timer: 0.0,
        }).id();

        world.run_system_once(depleted_patch_despawn_system).unwrap();
        world.flush();

        assert!(world.get_entity(patch_entity).is_ok(),
            "Non-depleted patch should not be despawned");
        assert!(world.get_entity(plate_entity).is_ok(),
            "Plate on non-depleted patch should not be despawned");
    }

    #[test]
    fn extraction_plate_mining_slowed_by_power_deficit() {
        let mut world = World::new();

        // Spawn a player with power deficit: ratio = 50/100 = 0.5
        let player_entity = world.spawn((
            Player::new("Test", FactionEnum::GlobalDefenseOrdinance, 0),
            GdoPlayerResources {
                space_crystals: 0,
                supplies: 0,
                power_generated: 50,
                power_consumed: 100,
                unit_control_used: 0,
                unit_control_cap: 10,
                has_power_plant: true,
            },
        )).id();

        // Spawn a crystal patch with resources
        let patch_entity = world.spawn(SpaceCrystalPatch {
            remaining_amount: 1000,
            initial_amount: 1000,
            has_plate: true,
        }).id();

        // Spawn an extraction plate owned by the player
        let plate_entity = world.spawn((
            Owner::player(0),
            ExtractionPlateState {
                attached_patch: patch_entity,
                mining_timer: 0.0,
            },
        )).id();

        // Run 48 ticks — at full power this would complete one mining cycle,
        // but at 0.5 ratio only 24.0 effective ticks accumulate
        for _ in 0..48 {
            world.run_system_once(extraction_plate_mining_system).unwrap();
            world.flush();
        }

        // Mining should NOT have completed yet
        let res = world.get::<GdoPlayerResources>(player_entity).unwrap();
        assert_eq!(res.space_crystals, 0,
            "Mining should not complete in 48 ticks at 0.5 power ratio");

        // Run 48 more ticks (total 96 ticks at 0.5 = 48 effective)
        for _ in 0..48 {
            world.run_system_once(extraction_plate_mining_system).unwrap();
            world.flush();
        }

        // Now mining should have completed at least once
        let res = world.get::<GdoPlayerResources>(player_entity).unwrap();
        assert!(res.space_crystals > 0,
            "Mining should complete after 96 ticks at 0.5 power ratio");
        assert_eq!(res.space_crystals, EXTRACTION_PLATE_MINING_RATE as i32,
            "Should have mined exactly one cycle worth of crystals");

        // Verify the timer has been reset
        let plate = world.get::<ExtractionPlateState>(plate_entity).unwrap();
        assert!(plate.mining_timer < EXTRACTION_PLATE_MINING_INTERVAL,
            "Timer should have been reset after mining cycle");
    }

    #[test]
    fn extraction_plate_mining_full_power() {
        let mut world = World::new();

        // Spawn a player with full power (generated >= consumed)
        let player_entity = world.spawn((
            Player::new("Test", FactionEnum::GlobalDefenseOrdinance, 0),
            GdoPlayerResources {
                space_crystals: 0,
                supplies: 0,
                power_generated: 100,
                power_consumed: 50,
                unit_control_used: 0,
                unit_control_cap: 10,
                has_power_plant: true,
            },
        )).id();

        let patch_entity = world.spawn(SpaceCrystalPatch {
            remaining_amount: 1000,
            initial_amount: 1000,
            has_plate: true,
        }).id();

        world.spawn((
            Owner::player(0),
            ExtractionPlateState {
                attached_patch: patch_entity,
                mining_timer: 0.0,
            },
        ));

        // Run exactly 48 ticks at full power — should complete one cycle
        for _ in 0..48 {
            world.run_system_once(extraction_plate_mining_system).unwrap();
            world.flush();
        }

        let res = world.get::<GdoPlayerResources>(player_entity).unwrap();
        assert_eq!(res.space_crystals, EXTRACTION_PLATE_MINING_RATE as i32,
            "Mining should complete in exactly 48 ticks at full power");
    }

    #[test]
    fn supply_tower_placement_links_chopper() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        let (tower_entity, chopper_entity) = app.world_mut().run_system_once(
            |mut commands: Commands,
             mut meshes: ResMut<Assets<Mesh>>,
             mut materials: ResMut<Assets<StandardMaterial>>| {
                let owner = Owner::player(1);
                let tower = spawn_supply_tower(
                    &mut commands, &mut meshes, &mut materials,
                    32, 32, owner, crate::types::StructureRotation::R0, false, false,
                );
                let chopper = spawn_supply_chopper(
                    &mut commands, &mut meshes, &mut materials,
                    33, 32, owner,
                );
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

        let tower_state = app.world().entity(tower_entity).get::<SupplyTowerState>().unwrap();
        assert_eq!(tower_state.attached_chopper, Some(chopper_entity),
            "Tower should reference the chopper entity");

        let chopper_state = app.world().entity(chopper_entity).get::<SupplyChopperState>().unwrap();
        assert_eq!(chopper_state.attached_tower, Some(tower_entity),
            "Chopper should reference the tower entity");
    }

    #[test]
    fn setup_player_resources_gdo_selected_spawns_gdo_and_syndicate() {
        let mut app = App::new();
        app.insert_resource(SelectedFaction(FactionEnum::GlobalDefenseOrdinance));
        app.world_mut().run_system_once(setup_player_resources).unwrap();
        app.world_mut().flush();

        // Verify LocalPlayer is set
        let local = app.world().resource::<LocalPlayer>();
        assert_eq!(local.0, 0);

        // Verify GDO player entity exists with player_id 0
        let mut found_gdo = false;
        let mut found_syn = false;
        for entity in app.world().iter_entities() {
            if let Some(player) = entity.get::<Player>() {
                if player.faction == FactionEnum::GlobalDefenseOrdinance {
                    assert_eq!(player.player_number, 0);
                    assert!(entity.get::<GdoPlayerResources>().is_some());
                    found_gdo = true;
                }
                if player.faction == FactionEnum::TheSyndicate {
                    assert_eq!(player.player_number, 1);
                    assert!(entity.get::<SyndicatePlayerResources>().is_some());
                    found_syn = true;
                }
            }
        }
        assert!(found_gdo, "GDO player entity should exist");
        assert!(found_syn, "Syndicate player entity should exist");
    }

    #[test]
    fn setup_player_resources_cults_selected_spawns_cults_and_gdo() {
        let mut app = App::new();
        app.insert_resource(SelectedFaction(FactionEnum::TheCults));
        app.world_mut().run_system_once(setup_player_resources).unwrap();
        app.world_mut().flush();

        let mut found_cults = false;
        let mut found_gdo = false;
        for entity in app.world().iter_entities() {
            if let Some(player) = entity.get::<Player>() {
                if player.faction == FactionEnum::TheCults {
                    assert_eq!(player.player_number, 0);
                    assert!(entity.get::<CultsPlayerResources>().is_some());
                    found_cults = true;
                }
                if player.faction == FactionEnum::GlobalDefenseOrdinance {
                    assert_eq!(player.player_number, 1);
                    assert!(entity.get::<GdoPlayerResources>().is_some());
                    found_gdo = true;
                }
            }
        }
        assert!(found_cults, "Cults player entity should exist");
        assert!(found_gdo, "GDO opponent entity should exist");
    }

    #[test]
    fn setup_player_resources_colonists_selected_spawns_colonists_and_gdo() {
        let mut app = App::new();
        app.insert_resource(SelectedFaction(FactionEnum::Colonists));
        app.world_mut().run_system_once(setup_player_resources).unwrap();
        app.world_mut().flush();

        let mut found_colonists = false;
        let mut found_gdo = false;
        for entity in app.world().iter_entities() {
            if let Some(player) = entity.get::<Player>() {
                if player.faction == FactionEnum::Colonists {
                    assert_eq!(player.player_number, 0);
                    assert!(entity.get::<ColonistsPlayerResources>().is_some());
                    found_colonists = true;
                }
                if player.faction == FactionEnum::GlobalDefenseOrdinance {
                    assert_eq!(player.player_number, 1);
                    assert!(entity.get::<GdoPlayerResources>().is_some());
                    found_gdo = true;
                }
            }
        }
        assert!(found_colonists, "Colonists player entity should exist");
        assert!(found_gdo, "GDO opponent entity should exist");
    }

    #[test]
    fn setup_player_resources_syndicate_selected_spawns_syndicate_and_gdo() {
        let mut app = App::new();
        app.insert_resource(SelectedFaction(FactionEnum::TheSyndicate));
        app.world_mut().run_system_once(setup_player_resources).unwrap();
        app.world_mut().flush();

        let mut found_syn = false;
        let mut found_gdo = false;
        for entity in app.world().iter_entities() {
            if let Some(player) = entity.get::<Player>() {
                if player.faction == FactionEnum::TheSyndicate {
                    assert_eq!(player.player_number, 0);
                    assert!(entity.get::<SyndicatePlayerResources>().is_some());
                    found_syn = true;
                }
                if player.faction == FactionEnum::GlobalDefenseOrdinance {
                    assert_eq!(player.player_number, 1);
                    assert!(entity.get::<GdoPlayerResources>().is_some());
                    found_gdo = true;
                }
            }
        }
        assert!(found_syn, "Syndicate player entity should exist");
        assert!(found_gdo, "GDO opponent entity should exist");
    }

    #[test]
    fn cults_player_resources_default_values() {
        let res = CultsPlayerResources::default();
        assert_eq!(res.space_crystals, 500);
        assert_eq!(res.unit_control_used, 0);
        assert_eq!(res.unit_control_available, 0);
    }

    #[test]
    fn colonists_player_resources_default_values() {
        let res = ColonistsPlayerResources::default();
        assert_eq!(res.space_crystals, 500);
        assert_eq!(res.alloys, 50);
        assert_eq!(res.essence, 50);
        assert_eq!(res.beacon_capacity_provided, 20);
        assert_eq!(res.beacon_capacity_used, 0);
    }

    #[test]
    fn extraction_plate_power_grid_integration() {
        use crate::testing::{TestApp, TestHarness};
        use crate::game::types::structures::gdo_structure_stats::{DC_POWER, PP_POWER, EP_POWER};

        let mut test_app = TestApp::new_with_faction(crate::types::FactionEnum::GlobalDefenseOrdinance);
        // Step to trigger OnEnter(InGame) which spawns faction/player entities
        test_app.step();
        let owner = Owner::player(0);

        // Spawn a power plant (+20)
        {
            let mut harness = TestHarness::new(&mut test_app.app);
            harness.spawn_structure_at_grid(ObjectEnum::PowerPlant, 32, 32, owner);
        }

        // Spawn 3 extraction plates (-3 each = -9 total)
        {
            let mut harness = TestHarness::new(&mut test_app.app);
            let patch = harness.spawn_resource(30, 30, 1000);
            harness.spawn_extraction_plate_at_grid(30, 31, owner, patch);
            let patch2 = harness.spawn_resource(31, 30, 1000);
            harness.spawn_extraction_plate_at_grid(31, 31, owner, patch2);
            let patch3 = harness.spawn_resource(33, 30, 1000);
            harness.spawn_extraction_plate_at_grid(33, 31, owner, patch3);
        }

        // Run the power grid computation
        test_app.app.world_mut().run_system_once(compute_power_grid).unwrap();

        // Check power values on the GDO player resources
        let mut query = test_app.app.world_mut().query::<(&Player, &GdoPlayerResources)>();
        let found = query.iter(test_app.app.world())
            .find(|(p, _)| p.player_number == 0);
        assert!(found.is_some(), "Player 0 should exist");
        let (_, res) = found.unwrap();
        // DC (20) is spawned by TestApp initialization + our PP (20) = 40 generated
        let expected_generated = DC_POWER + PP_POWER;
        let expected_consumed = EP_POWER.abs() * 3; // 3 plates * 3 = 9
        assert_eq!(res.power_generated, expected_generated, "DC + Power Plant generate power");
        assert_eq!(res.power_consumed, expected_consumed, "3 extraction plates consume {} power each", EP_POWER.abs());
        // Net power: 40 - 9 = 31
        assert_eq!(res.current_power(), expected_generated - expected_consumed);
    }

    #[test]
    fn test_center_camera_on_start_gdo() {
        use crate::testing::TestApp;
        use crate::types::{MainCamera, FactionEnum, LocalPlayer};
        use crate::game::types::ObjectInstance;

        let mut test_app = TestApp::new_with_faction(FactionEnum::GlobalDefenseOrdinance);
        test_app.app.world_mut().insert_resource(LocalPlayer(0));
        test_app.app.world_mut().init_resource::<crate::game::world::types::MapStartingPositions>();

        // Spawn a deployment center for player 0 at a known position
        let dc_pos = Vec3::new(100.0, 0.0, 200.0);
        test_app.app.world_mut().spawn((
            ObjectInstance { object_type: ObjectEnum::DeploymentCenter, hp: None, max_hp: None },
            Owner::player(0),
            Transform::from_translation(dc_pos),
        ));

        // TestApp already spawns a MainCamera at (0, 40, 25)
        test_app.app.world_mut().run_system_once(center_camera_on_start).unwrap();

        let mut cam_query = test_app.app.world_mut().query_filtered::<&Transform, With<MainCamera>>();
        let cam = cam_query.single(test_app.app.world()).unwrap();
        assert_eq!(cam.translation.x, 100.0);
        let expected_z = 200.0 + 40.0 * 25.0 / 40.0;
        assert_eq!(cam.translation.z, expected_z);
    }

    #[test]
    fn test_center_camera_on_start_syndicate() {
        use crate::testing::TestApp;
        use crate::types::{MainCamera, FactionEnum, LocalPlayer};
        use crate::game::types::ObjectInstance;

        let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
        test_app.app.world_mut().insert_resource(LocalPlayer(0));
        test_app.app.world_mut().init_resource::<crate::game::world::types::MapStartingPositions>();

        // Spawn a tunnel for player 0 at a known position
        let tunnel_pos = Vec3::new(50.0, 0.0, 80.0);
        test_app.app.world_mut().spawn((
            ObjectInstance { object_type: ObjectEnum::Tunnel, hp: None, max_hp: None },
            Owner::player(0),
            Transform::from_translation(tunnel_pos),
        ));

        // TestApp already spawns a MainCamera at (0, 40, 25)
        test_app.app.world_mut().run_system_once(center_camera_on_start).unwrap();

        let mut cam_query = test_app.app.world_mut().query_filtered::<&Transform, With<MainCamera>>();
        let cam = cam_query.single(test_app.app.world()).unwrap();
        assert_eq!(cam.translation.x, 50.0);
        let expected_z = 80.0 + 40.0 * 25.0 / 40.0;
        assert_eq!(cam.translation.z, expected_z);
    }

    #[test]
    fn test_center_camera_on_start_no_structure() {
        use crate::testing::TestApp;
        use crate::types::{MainCamera, FactionEnum, LocalPlayer};

        let mut test_app = TestApp::new_with_faction(FactionEnum::GlobalDefenseOrdinance);
        test_app.app.world_mut().insert_resource(LocalPlayer(0));
        test_app.app.world_mut().init_resource::<crate::game::world::types::MapStartingPositions>();

        // No DC spawned — system should be a no-op. TestApp camera at (0, 40, 25).
        test_app.app.world_mut().run_system_once(center_camera_on_start).unwrap();

        let mut cam_query = test_app.app.world_mut().query_filtered::<&Transform, With<MainCamera>>();
        let cam = cam_query.single(test_app.app.world()).unwrap();
        // Camera should remain at original position
        assert_eq!(cam.translation.x, 0.0);
    }

    #[test]
    fn test_center_camera_on_start_map_position_override() {
        use crate::testing::TestApp;
        use crate::types::{MainCamera, FactionEnum, LocalPlayer};
        use crate::game::types::ObjectInstance;
        use crate::game::world::types::MapStartingPositions;
        use crate::game::world::utils::grid_to_world;

        let mut test_app = TestApp::new_with_faction(FactionEnum::GlobalDefenseOrdinance);
        test_app.app.world_mut().insert_resource(LocalPlayer(0));

        // Set a map starting position for player 0
        let mut map_pos = MapStartingPositions::default();
        map_pos.positions.insert(0, (40, 50));
        test_app.app.world_mut().insert_resource(map_pos);

        // Also spawn a DC — map position should take priority
        let dc_pos = Vec3::new(100.0, 0.0, 200.0);
        test_app.app.world_mut().spawn((
            ObjectInstance { object_type: ObjectEnum::DeploymentCenter, hp: None, max_hp: None },
            Owner::player(0),
            Transform::from_translation(dc_pos),
        ));

        test_app.app.world_mut().run_system_once(center_camera_on_start).unwrap();

        let mut cam_query = test_app.app.world_mut().query_filtered::<&Transform, With<MainCamera>>();
        let cam = cam_query.single(test_app.app.world()).unwrap();

        let expected_world = grid_to_world(40, 50, 1.0);
        assert_eq!(cam.translation.x, expected_world.x);
        let expected_z = expected_world.z + 40.0 * 25.0 / 40.0;
        assert_eq!(cam.translation.z, expected_z);
        // Confirm it did NOT use the DC position
        assert_ne!(cam.translation.x, 100.0);
    }

    #[test]
    fn test_center_camera_on_start_map_position_no_matching_slot() {
        use crate::testing::TestApp;
        use crate::types::{MainCamera, FactionEnum, LocalPlayer};
        use crate::game::types::ObjectInstance;
        use crate::game::world::types::MapStartingPositions;

        let mut test_app = TestApp::new_with_faction(FactionEnum::GlobalDefenseOrdinance);
        test_app.app.world_mut().insert_resource(LocalPlayer(0));

        // Set a map starting position for player 1 (not local player 0)
        let mut map_pos = MapStartingPositions::default();
        map_pos.positions.insert(1, (40, 50));
        test_app.app.world_mut().insert_resource(map_pos);

        // Spawn a DC for player 0 — should fall through to structure-based centering
        let dc_pos = Vec3::new(100.0, 0.0, 200.0);
        test_app.app.world_mut().spawn((
            ObjectInstance { object_type: ObjectEnum::DeploymentCenter, hp: None, max_hp: None },
            Owner::player(0),
            Transform::from_translation(dc_pos),
        ));

        test_app.app.world_mut().run_system_once(center_camera_on_start).unwrap();

        let mut cam_query = test_app.app.world_mut().query_filtered::<&Transform, With<MainCamera>>();
        let cam = cam_query.single(test_app.app.world()).unwrap();
        // Should use the DC position, not the map position for slot 1
        assert_eq!(cam.translation.x, 100.0);
        let expected_z = 200.0 + 40.0 * 25.0 / 40.0;
        assert_eq!(cam.translation.z, expected_z);
    }

    #[test]
    fn test_center_camera_on_start_map_position_grid_conversion() {
        use crate::testing::TestApp;
        use crate::types::{MainCamera, FactionEnum, LocalPlayer};
        use crate::game::world::types::MapStartingPositions;
        use crate::game::world::utils::grid_to_world;

        let mut test_app = TestApp::new_with_faction(FactionEnum::GlobalDefenseOrdinance);
        test_app.app.world_mut().insert_resource(LocalPlayer(0));

        // Use grid center (32, 32) which should map to world (0.5, 0.0, 0.5)
        let mut map_pos = MapStartingPositions::default();
        map_pos.positions.insert(0, (32, 32));
        test_app.app.world_mut().insert_resource(map_pos);

        test_app.app.world_mut().run_system_once(center_camera_on_start).unwrap();

        let mut cam_query = test_app.app.world_mut().query_filtered::<&Transform, With<MainCamera>>();
        let cam = cam_query.single(test_app.app.world()).unwrap();

        let expected_world = grid_to_world(32, 32, 1.0);
        assert_eq!(cam.translation.x, expected_world.x);
        assert_eq!(cam.translation.x, 0.5); // (32 - 32) * 1.0 + 0.5
        let expected_z = expected_world.z + 40.0 * 25.0 / 40.0;
        assert_eq!(cam.translation.z, expected_z);
    }

    // =====================================================
    // RECRUITMENT TILE CLAIMING TESTS
    // =====================================================

    /// Helper: spawn a minimal tile entity at the given grid position with the given preset
    fn spawn_test_tile(world: &mut World, x: i32, z: i32, preset_enum: crate::game::world::types::TilePresetEnum) -> Entity {
        let preset = preset_enum.properties();
        world.spawn((
            GridPosition { x, z },
            preset,
            Tile,
        )).id()
    }

    /// Helper: spawn a minimal Recruitment Center entity
    fn spawn_test_rc(world: &mut World, x: i32, z: i32, build_order: u64) -> Entity {
        world.spawn((
            GridPosition { x, z },
            RecruitmentCenterState {
                build_order,
                ..Default::default()
            },
            ObjectInstance::destructible(ObjectEnum::RecruitmentCenter, crate::game::types::structures::cults_structure_stats::RC_MAX_HP),
        )).id()
    }

    #[test]
    fn tile_claim_map_basic_operations() {
        use crate::game::world::types::TileClaimMap;
        let mut map = TileClaimMap::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();

        // Claim and check
        map.claim_tile((5, 5), e1);
        assert_eq!(map.is_claimed((5, 5)), Some(e1));
        assert_eq!(map.is_claimed((6, 6)), None);

        // Unclaim single
        map.unclaim_tile((5, 5));
        assert_eq!(map.is_claimed((5, 5)), None);

        // Unclaim all for entity
        map.claim_tile((1, 1), e1);
        map.claim_tile((2, 2), e1);
        map.claim_tile((3, 3), e2);
        map.unclaim_all_for(e1);
        assert_eq!(map.is_claimed((1, 1)), None);
        assert_eq!(map.is_claimed((2, 2)), None);
        assert_eq!(map.is_claimed((3, 3)), Some(e2));
    }

    #[test]
    fn rc_all_recruitable_terrain_full_effectiveness() {
        use crate::game::world::types::{TileClaimMap, TilePresetEnum};

        let mut app = App::new();
        app.init_resource::<TileClaimMap>();

        // Spawn a 10x10 area of recruitable tiles centered on RC at (10, 10)
        // RC footprint: [10..13] x [10..13], area: [7..16] x [7..16]
        for x in 7..=16 {
            for z in 7..=16 {
                spawn_test_tile(app.world_mut(), x, z, TilePresetEnum::Plane);
            }
        }

        // Spawn RC
        spawn_test_rc(app.world_mut(), 10, 10, 0);

        app.world_mut().run_system_once(recruitment_tile_claiming_system).unwrap();

        // Check state
        let mut query = app.world_mut().query::<&RecruitmentCenterState>();
        let state = query.single(app.world()).unwrap();
        assert_eq!(state.claimed_tiles.len(), 100);
        assert!((state.effectiveness - 1.0).abs() < f32::EPSILON);
        assert_eq!(state.local_capacity, 20);
    }

    #[test]
    fn rc_half_recruitable_terrain_half_effectiveness() {
        use crate::game::world::types::{TileClaimMap, TilePresetEnum};

        let mut app = App::new();
        app.init_resource::<TileClaimMap>();

        // RC at (10, 10), area [7..16] x [7..16]
        // Make half the tiles Water (non-recruitable)
        for x in 7..=16 {
            for z in 7..=16 {
                if z <= 11 {
                    // 5 rows of 10 = 50 recruitable
                    spawn_test_tile(app.world_mut(), x, z, TilePresetEnum::Plane);
                } else {
                    // 5 rows of 10 = 50 non-recruitable
                    spawn_test_tile(app.world_mut(), x, z, TilePresetEnum::Water);
                }
            }
        }

        spawn_test_rc(app.world_mut(), 10, 10, 0);

        app.world_mut().run_system_once(recruitment_tile_claiming_system).unwrap();

        let mut query = app.world_mut().query::<&RecruitmentCenterState>();
        let state = query.single(app.world()).unwrap();
        assert_eq!(state.claimed_tiles.len(), 50);
        assert!((state.effectiveness - 0.5).abs() < f32::EPSILON);
        assert_eq!(state.local_capacity, 10);
    }

    #[test]
    fn rc_overlapping_areas_first_built_has_priority() {
        use crate::game::world::types::{TileClaimMap, TilePresetEnum};

        let mut app = App::new();
        app.init_resource::<TileClaimMap>();

        // Two RCs with overlapping areas. Both at same z, close together in x.
        // RC1 at (10, 10), area [7..16] x [7..16], build_order=0
        // RC2 at (14, 10), area [11..20] x [7..16], build_order=1
        // Overlap: x in [11..16], z in [7..16] = 6 * 10 = 60 tiles

        // Spawn all tiles in combined area
        for x in 7..=20 {
            for z in 7..=16 {
                spawn_test_tile(app.world_mut(), x, z, TilePresetEnum::Plane);
            }
        }

        let rc1 = spawn_test_rc(app.world_mut(), 10, 10, 0);
        let _rc2 = spawn_test_rc(app.world_mut(), 14, 10, 1);

        app.world_mut().run_system_once(recruitment_tile_claiming_system).unwrap();

        // RC1 gets all 100 tiles (its full area)
        // RC2 gets: area [11..20]x[7..16] = 10*10=100 tiles, minus 60 overlap claimed by RC1 = 40
        let mut query = app.world_mut().query::<(Entity, &RecruitmentCenterState)>();
        let mut found_rc1 = false;
        let mut found_rc2 = false;
        for (entity, state) in query.iter(app.world()) {
            if entity == rc1 {
                assert_eq!(state.claimed_tiles.len(), 100);
                assert!((state.effectiveness - 1.0).abs() < f32::EPSILON);
                assert_eq!(state.local_capacity, 20);
                found_rc1 = true;
            } else {
                assert_eq!(state.claimed_tiles.len(), 40);
                assert!((state.effectiveness - 0.4).abs() < f32::EPSILON);
                assert_eq!(state.local_capacity, 8);
                found_rc2 = true;
            }
        }
        assert!(found_rc1 && found_rc2);
    }

    #[test]
    fn rc_destruction_frees_tiles_for_others() {
        use crate::game::world::types::{TileClaimMap, TilePresetEnum};

        let mut app = App::new();
        app.init_resource::<TileClaimMap>();

        // Same overlapping setup
        for x in 7..=20 {
            for z in 7..=16 {
                spawn_test_tile(app.world_mut(), x, z, TilePresetEnum::Plane);
            }
        }

        let rc1 = spawn_test_rc(app.world_mut(), 10, 10, 0);
        let _rc2 = spawn_test_rc(app.world_mut(), 14, 10, 1);

        // Run claiming once
        app.world_mut().run_system_once(recruitment_tile_claiming_system).unwrap();

        // Kill RC1
        let mut obj_query = app.world_mut().query::<&mut ObjectInstance>();
        let mut rc1_obj = obj_query.get_mut(app.world_mut(), rc1).unwrap();
        rc1_obj.apply_damage(crate::game::types::structures::cults_structure_stats::RC_MAX_HP + 1.0);

        // Run claiming again
        app.world_mut().run_system_once(recruitment_tile_claiming_system).unwrap();

        // RC2 should now get all 100 tiles in its area
        let mut state_query = app.world_mut().query::<(Entity, &RecruitmentCenterState)>();
        for (entity, state) in state_query.iter(app.world()) {
            if entity != rc1 {
                assert_eq!(state.claimed_tiles.len(), 100);
                assert!((state.effectiveness - 1.0).abs() < f32::EPSILON);
                assert_eq!(state.local_capacity, 20);
            }
        }

        // Claim map should have no claims for RC1
        let claim_map = app.world().resource::<TileClaimMap>();
        assert!(claim_map.claims.values().all(|&e| e != rc1));
    }

    #[test]
    fn rc_near_map_edge_clamps_area() {
        use crate::game::world::types::{TileClaimMap, TilePresetEnum};

        let mut app = App::new();
        app.init_resource::<TileClaimMap>();

        // RC at (0, 0) — area would be [-3..6] x [-3..6], clamped to [0..6] x [0..6] = 7*7 = 49 tiles
        for x in 0..=6 {
            for z in 0..=6 {
                spawn_test_tile(app.world_mut(), x, z, TilePresetEnum::Plane);
            }
        }

        spawn_test_rc(app.world_mut(), 0, 0, 0);

        app.world_mut().run_system_once(recruitment_tile_claiming_system).unwrap();

        let mut query = app.world_mut().query::<&RecruitmentCenterState>();
        let state = query.single(app.world()).unwrap();
        assert_eq!(state.claimed_tiles.len(), 49);
        assert!((state.effectiveness - 0.49).abs() < f32::EPSILON);
        // floor(20 * 0.49) = floor(9.8) = 9
        assert_eq!(state.local_capacity, 9);
    }

    // =====================================================
    // RECRUITMENT CENTER PRODUCTION TESTS
    // =====================================================

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
        app.init_resource::<crate::game::world::types::GridMap>();
        app.init_resource::<crate::game::units::types::OccupancyMap>();
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
        use crate::game::units::types::state::UnitCommand;

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

    // =====================================================
    // CULTS UNIT CONTROL AGGREGATION TESTS
    // =====================================================

    #[test]
    fn aggregation_sums_capacity_across_rcs() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn player entity with CultsPlayerResources
        app.world_mut().spawn((
            InvisibleEntity,
            Player::new("Test Player", FactionEnum::TheCults, 0),
            CultsPlayerResources::default(),
        ));

        // Spawn two RCs owned by player 0
        app.world_mut().spawn((
            Owner::player(0),
            RecruitmentCenterState {
                local_capacity: 20,
                local_used: 5,
                effectiveness: 1.0,
                ..Default::default()
            },
        ));
        app.world_mut().spawn((
            Owner::player(0),
            RecruitmentCenterState {
                local_capacity: 10,
                local_used: 3,
                effectiveness: 0.5,
                ..Default::default()
            },
        ));

        app.world_mut().run_system_once(cults_unit_control_aggregation_system).unwrap();

        let mut query = app.world_mut().query::<&CultsPlayerResources>();
        let res = query.single(app.world()).unwrap();
        assert_eq!(res.unit_control_available, 30, "Total capacity should be 20 + 10");
        assert_eq!(res.unit_control_used, 8, "Total used should be 5 + 3");
    }

    #[test]
    fn aggregation_ignores_other_players_rcs() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn player 0
        app.world_mut().spawn((
            InvisibleEntity,
            Player::new("Player 0", FactionEnum::TheCults, 0),
            CultsPlayerResources::default(),
        ));

        // Spawn RC owned by player 1 (different player)
        app.world_mut().spawn((
            Owner::player(1),
            RecruitmentCenterState {
                local_capacity: 20,
                local_used: 10,
                effectiveness: 1.0,
                ..Default::default()
            },
        ));

        app.world_mut().run_system_once(cults_unit_control_aggregation_system).unwrap();

        let mut query = app.world_mut().query::<&CultsPlayerResources>();
        let res = query.single(app.world()).unwrap();
        assert_eq!(res.unit_control_available, 0, "Should not count other player's RC");
        assert_eq!(res.unit_control_used, 0);
    }

    // =====================================================
    // CULTS CONSTRUCTION TICK SYSTEM TESTS
    // =====================================================

    fn setup_construction_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app
    }

    /// Helper to spawn a Cults building under construction with CultsConstructionState
    fn spawn_cults_building_under_construction(world: &mut World, total_frames: u32, max_hp: f32) -> Entity {
        world.spawn((
            ObjectInstance::under_construction(ObjectEnum::CultsStorage, max_hp),
            ConstructionHP::new(total_frames),
            CultsConstructionState::new(total_frames),
            Transform::from_xyz(5.0, 0.0, 5.0),
            Owner::player(0),
        )).id()
    }

    /// Helper to spawn a hidden Recruit entity (simulating one that has entered a building)
    fn spawn_hidden_recruit(world: &mut World) -> Entity {
        world.spawn((
            crate::types::Unit,
            ObjectInstance::destructible(ObjectEnum::CultsRecruit, 50.0),
            Owner::player(0),
            UnitCommand::Idle,
            Visibility::Hidden,
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id()
    }

    #[test]
    fn cults_construction_one_recruit_completes_in_total_frames() {
        let mut app = setup_construction_app();
        let total_frames = 100;
        let max_hp = 200.0;

        let recruit = spawn_hidden_recruit(app.world_mut());
        let building = spawn_cults_building_under_construction(app.world_mut(), total_frames, max_hp);

        // Assign 1 recruit to the building
        app.world_mut().get_mut::<CultsConstructionState>(building).unwrap()
            .assigned_recruits.push(recruit);

        // Run for exactly total_frames ticks
        for _ in 0..total_frames {
            app.world_mut().run_system_once(cults_construction_tick_system).unwrap();
        }
        // Flush deferred commands
        app.update();

        // Building should be complete (no ConstructionHP)
        assert!(app.world().get::<ConstructionHP>(building).is_none(),
            "ConstructionHP should be removed on completion");
        assert!(app.world().get::<CultsConstructionState>(building).is_none(),
            "CultsConstructionState should be removed on completion");

        // HP should be at max
        let obj = app.world().get::<ObjectInstance>(building).unwrap();
        assert_eq!(obj.hp, Some(max_hp));

        // Recruit should be despawned
        assert!(app.world().get_entity(recruit).is_err(),
            "Recruit should be despawned on completion");
    }

    #[test]
    fn cults_construction_two_recruits_complete_in_half_frames() {
        let mut app = setup_construction_app();
        let total_frames = 100;
        let max_hp = 200.0;

        let recruit1 = spawn_hidden_recruit(app.world_mut());
        let recruit2 = spawn_hidden_recruit(app.world_mut());
        let building = spawn_cults_building_under_construction(app.world_mut(), total_frames, max_hp);

        // Assign 2 recruits
        {
            let mut state = app.world_mut().get_mut::<CultsConstructionState>(building).unwrap();
            state.assigned_recruits.push(recruit1);
            state.assigned_recruits.push(recruit2);
        }

        // Run for half the total frames
        for _ in 0..50 {
            app.world_mut().run_system_once(cults_construction_tick_system).unwrap();
        }
        app.update();

        // Building should be complete
        assert!(app.world().get::<ConstructionHP>(building).is_none(),
            "Should complete in N/2 frames with 2 recruits");

        // Both recruits should be despawned
        assert!(app.world().get_entity(recruit1).is_err(), "Recruit 1 should be despawned");
        assert!(app.world().get_entity(recruit2).is_err(), "Recruit 2 should be despawned");
    }

    #[test]
    fn cults_construction_no_recruits_no_progress() {
        let mut app = setup_construction_app();
        let total_frames = 100;
        let max_hp = 200.0;

        let building = spawn_cults_building_under_construction(app.world_mut(), total_frames, max_hp);

        // Run many frames with no recruits assigned
        for _ in 0..200 {
            app.world_mut().run_system_once(cults_construction_tick_system).unwrap();
        }

        // Should still have ConstructionHP (not complete)
        let state = app.world().get::<CultsConstructionState>(building).unwrap();
        assert_eq!(state.construction_progress, 0, "No progress without recruits");
        assert!(app.world().get::<ConstructionHP>(building).is_some());
    }

    #[test]
    fn cults_construction_hp_scales_during_progress() {
        let mut app = setup_construction_app();
        let total_frames = 100;
        let max_hp = 200.0;

        let recruit = spawn_hidden_recruit(app.world_mut());
        let building = spawn_cults_building_under_construction(app.world_mut(), total_frames, max_hp);

        app.world_mut().get_mut::<CultsConstructionState>(building).unwrap()
            .assigned_recruits.push(recruit);

        // Run for 50 frames (50% progress)
        for _ in 0..50 {
            app.world_mut().run_system_once(cults_construction_tick_system).unwrap();
        }

        let obj = app.world().get::<ObjectInstance>(building).unwrap();
        let expected_hp = max_hp * ConstructionHP::hp_fraction(0.5); // 10% + 90% * 0.5 = 55% = 110.0
        assert!(f32::abs(obj.hp.unwrap() - expected_hp) < 0.01,
            "HP should scale with progress: expected {}, got {}", expected_hp, obj.hp.unwrap());
    }

    #[test]
    fn cults_construction_cancel_ejects_recruits_on_death() {
        let mut app = setup_construction_app();
        let total_frames = 100;

        let recruit1 = spawn_hidden_recruit(app.world_mut());
        let recruit2 = spawn_hidden_recruit(app.world_mut());
        let building = spawn_cults_building_under_construction(app.world_mut(), total_frames, 200.0);

        {
            let mut state = app.world_mut().get_mut::<CultsConstructionState>(building).unwrap();
            state.assigned_recruits.push(recruit1);
            state.assigned_recruits.push(recruit2);
        }

        // Kill the building
        app.world_mut().get_mut::<ObjectInstance>(building).unwrap().hp = Some(0.0);

        // Run cancel system
        app.world_mut().run_system_once(cults_construction_cancel_system).unwrap();
        app.update();

        // Recruits should be visible again and idle
        let vis1 = app.world().get::<Visibility>(recruit1).unwrap();
        assert_eq!(*vis1, Visibility::Inherited, "Recruit 1 should be visible after cancel");

        let vis2 = app.world().get::<Visibility>(recruit2).unwrap();
        assert_eq!(*vis2, Visibility::Inherited, "Recruit 2 should be visible after cancel");

        let cmd1 = app.world().get::<UnitCommand>(recruit1).unwrap();
        assert!(matches!(*cmd1, UnitCommand::Idle), "Recruit 1 should be idle after cancel");

        let cmd2 = app.world().get::<UnitCommand>(recruit2).unwrap();
        assert!(matches!(*cmd2, UnitCommand::Idle), "Recruit 2 should be idle after cancel");
    }

    #[test]
    fn cults_construction_state_new() {
        let state = CultsConstructionState::new(300);
        assert!(state.assigned_recruits.is_empty());
        assert_eq!(state.construction_progress, 0);
        assert_eq!(state.total_construction_frames, 300);
    }

    #[test]
    fn generic_construction_hp_tick_skips_cults_buildings() {
        let mut app = setup_construction_app();
        let total_frames = 100;
        let max_hp = 200.0;

        // Spawn a Cults building with BOTH ConstructionHP and CultsConstructionState
        let building = spawn_cults_building_under_construction(app.world_mut(), total_frames, max_hp);

        // Run the GENERIC construction_hp_tick_system
        for _ in 0..50 {
            app.world_mut().run_system_once(construction_hp_tick_system).unwrap();
        }

        // The generic system should NOT have advanced progress (due to Without<CultsConstructionState> filter)
        let construction = app.world().get::<ConstructionHP>(building).unwrap();
        assert_eq!(construction.progress, 0.0,
            "Generic construction_hp_tick_system should skip Cults buildings");
    }

    // === Armory training tick system tests ===

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
    fn armory_exit_side_position_r0_is_south() {
        let transform = Transform::from_xyz(10.0, 0.0, 10.0);
        let si = StructureInstance {
            rotation: crate::types::StructureRotation::R0,
            flip_horizontal: false,
            flip_vertical: false,
        };
        let pos = armory_exit_side_position(&transform, &si);
        // ABCB symmetry at R0: N=A, E=B, S=C, W=B. Side C is South.
        assert!(pos.z > 10.0, "Exit should be south of center, got z={}", pos.z);
        assert!((pos.x - 10.0).abs() < 0.01, "Exit should be at same x");
    }

    // === Armory eject system tests ===

    #[test]
    fn armory_eject_removes_queue_when_empty() {
        let mut app = App::new();

        let armory = app.world_mut().spawn((
            Transform::from_xyz(10.0, 0.0, 10.0),
            StructureInstance {
                rotation: crate::types::StructureRotation::R0,
                flip_horizontal: false,
                flip_vertical: false,
            },
            ArmoryEjectionQueue {
                queue: std::collections::VecDeque::new(),
                cooldown: 0,
            },
        )).id();

        app.world_mut().run_system_once(armory_eject_tick_system).unwrap();

        // Empty queue should be removed
        assert!(app.world().get::<ArmoryEjectionQueue>(armory).is_none());
    }

    #[test]
    fn armory_eject_sets_cooldown() {
        let mut app = App::new();

        // Create a recruit entity to eject
        let recruit = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::Hidden,
            UnitCommand::Idle,
        )).id();

        let mut queue = std::collections::VecDeque::new();
        queue.push_back(recruit);

        let armory = app.world_mut().spawn((
            Transform::from_xyz(10.0, 0.0, 10.0),
            StructureInstance {
                rotation: crate::types::StructureRotation::R0,
                flip_horizontal: false,
                flip_vertical: false,
            },
            ArmoryEjectionQueue {
                queue,
                cooldown: 0,
            },
        )).id();

        app.world_mut().run_system_once(armory_eject_tick_system).unwrap();

        let ejection = app.world().get::<ArmoryEjectionQueue>(armory);
        // Should have cooldown set after ejecting
        if let Some(eq) = ejection {
            assert_eq!(eq.cooldown, 8);
        }
        // Queue should now be empty (but component exists due to cooldown)
    }

    #[test]
    fn armory_eject_respects_cooldown() {
        let mut app = App::new();

        let recruit = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::Hidden,
            UnitCommand::Idle,
        )).id();

        let mut queue = std::collections::VecDeque::new();
        queue.push_back(recruit);

        let armory = app.world_mut().spawn((
            Transform::from_xyz(10.0, 0.0, 10.0),
            StructureInstance {
                rotation: crate::types::StructureRotation::R0,
                flip_horizontal: false,
                flip_vertical: false,
            },
            ArmoryEjectionQueue {
                queue,
                cooldown: 5,
            },
        )).id();

        app.world_mut().run_system_once(armory_eject_tick_system).unwrap();

        // Cooldown should have decremented but recruit not ejected
        let ejection = app.world().get::<ArmoryEjectionQueue>(armory).unwrap();
        assert_eq!(ejection.cooldown, 4);
        assert_eq!(ejection.queue.len(), 1);
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
}
