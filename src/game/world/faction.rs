use bevy::prelude::*;
use crate::types::{Owner, ObjectEnum, MainCamera, GridPosition, LocalPlayer, InvisibleEntity, Selected, SelectionBounds, SelectedFaction, FactionEnum};
use crate::game::types::*;
use crate::game::utils::{
    spawn_deployment_center, spawn_power_plant, spawn_barracks,
    spawn_extraction_facility, spawn_extraction_plate, spawn_peacekeeper,
    spawn_tunnel, spawn_headquarters, spawn_supply_tower, spawn_supply_chopper,
};
use super::types::{GdoBuildArea, SpaceCrystalPatch, Tile, TilePreset, FogOfWarMap};
use super::utils::{expand_build_area, world_to_grid, grid_to_world, can_place_building, rotated_building_size, cursor_pos_in_viewport};
use crate::ui::types::{ObjectInterfaceState, StructureMenuState, AgentMenuState, CommandPanelTarget, PlacementGhost, PlacementState, CursorOverUi, BuildAreaOverlay};
use crate::game::units::types::state::UnitCommand;

/// Setup initial player resources using design-aligned Player + faction-specific resource components.
/// Player 0 is always the local human; the faction assignment depends on SelectedFaction.
pub fn setup_player_resources(mut commands: Commands, selected: Res<SelectedFaction>) {
    // The local human controls player 0
    commands.insert_resource(LocalPlayer(0));

    // Determine player numbers based on selected faction
    let (gdo_player, syn_player) = if selected.0 == FactionEnum::GlobalDefenseOrdinance {
        (0u8, 1u8) // Player selected GDO → GDO is player 0, Syndicate is player 1
    } else {
        (1u8, 0u8) // Player selected Syndicate → Syndicate is player 0, GDO is player 1
    };

    // Spawn Faction entities (invisible entities representing each faction in the game)
    commands.spawn((
        InvisibleEntity,
        FactionEnum::GlobalDefenseOrdinance,
        DisplayHud::new(FactionEnum::GlobalDefenseOrdinance),
    ));
    commands.spawn((
        InvisibleEntity,
        FactionEnum::TheSyndicate,
        DisplayHud::new(FactionEnum::TheSyndicate),
    ));

    // Spawn Player entities (invisible entities with faction-specific resources)
    commands.spawn((
        InvisibleEntity,
        Player::new("Player 1", FactionEnum::GlobalDefenseOrdinance, gdo_player),
        DisplayHudInfo::new(FactionEnum::GlobalDefenseOrdinance),
        GdoPlayerResources {
            space_crystals: 1000,
            supplies: 0,
            power_generated: 0,  // Will be computed by power grid system from buildings
            power_consumed: 0,
            unit_control_used: 0,
            unit_control_cap: 200,
        },
    ));
    commands.spawn((
        InvisibleEntity,
        Player::new("Player 2", FactionEnum::TheSyndicate, syn_player),
        DisplayHudInfo::new(FactionEnum::TheSyndicate),
        SyndicatePlayerResources::default(),
    ));

    info!("Initialized faction resources: Selected {:?}, GDO=player {}, SYN=player {}", selected.0, gdo_player, syn_player);
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
        &mut commands,
        hq_grid_x, hq_grid_z,
        syn_owner, tunnel_entity,
    );

    info!(
        "Syndicate Game Start: Deployed Tunnel at grid ({}, {}), HQ at ({}, {}), owner={:?}",
        tunnel_grid_x, tunnel_grid_z, hq_grid_x, hq_grid_z, syn_owner
    );
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
    buildings: Query<(&Owner, &PowerValue)>,
) {
    for (player, mut resources) in players.iter_mut() {
        let mut generated: i32 = 0;
        let mut consumed: i32 = 0;

        for (owner, power) in buildings.iter() {
            if owner.player_number() == Some(player.player_number) {
                if power.0 > 0 {
                    generated += power.0;
                } else if power.0 < 0 {
                    consumed += power.0.abs();
                }
            }
        }

        resources.power_generated = generated;
        resources.power_consumed = consumed;
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
    mut barracks_query: Query<(Entity, &Owner, &crate::types::GridPosition, &mut BarracksState)>,
    mut players: Query<(&Player, &mut GdoPlayerResources)>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: Res<super::types::GridMap>,
    rally_targets: Query<(&Transform, &Owner), With<ObjectInstance>>,
    occupancy: Res<crate::game::units::types::OccupancyMap>,
) {
    for (_bk_entity, owner, grid_pos, mut bk_state) in barracks_query.iter_mut() {
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
                        // Production complete — spawn unit
                        // Spawn at B side (short side): 1 grid unit beyond the building's edge
                        // Barracks is 3x2 at grid_pos, B side is the z+ short side
                        let spawn_x = grid_pos.x + 1; // Center of 3-wide
                        let spawn_z = grid_pos.z + 3; // 1 beyond the 2-deep building

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
                        issue_rally_command(
                            &mut commands, unit_entity,
                            &bk_state.rally_point, owner,
                            spawn_x, spawn_z,
                            &tiles, &grid, &rally_targets,
                            &occupancy,
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

/// Issue a rally command to a newly spawned unit based on the barracks rally point
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
) {
    use crate::game::units::utils::{world_to_grid, smooth_path};
    use crate::game::units::pathfinding::find_path;
    use crate::game::units::types::movement::{MoveTarget, Path};
    use crate::game::units::types::state::UnitCommand;

    let rally = match rally_point {
        Some(r) => r,
        None => return, // No rally point set — unit stays idle
    };

    let spawn_grid = GridPosition { x: spawn_x, z: spawn_z };
    // Default unit base for Peacekeeper
    let unit_base = crate::types::UnitBaseEnum::LightInfantry;

    match rally {
        RallyTarget::Location(pos) => {
            let target_grid = world_to_grid(*pos);
            if let Some(path) = find_path(spawn_grid, target_grid, tiles, &unit_base, grid.width as i32, grid.height as i32, occupancy, (spawn_grid.x, spawn_grid.z)) {
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
                    if let Some(path) = find_path(spawn_grid, target_grid, tiles, &unit_base, grid.width as i32, grid.height as i32, occupancy, (spawn_grid.x, spawn_grid.z)) {
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
// EXTRACTION PLATE MINING SYSTEM
// =====================================================

/// System that ticks Extraction Plate mining each simulation frame
pub fn extraction_plate_mining_system(
    mut plates: Query<(&Owner, &mut ExtractionPlateState)>,
    mut patches: Query<&mut SpaceCrystalPatch>,
    mut players: Query<(&Player, &mut GdoPlayerResources)>,
) {
    for (owner, mut plate_state) in plates.iter_mut() {
        plate_state.mining_timer += 1;

        if plate_state.mining_timer >= EXTRACTION_PLATE_MINING_INTERVAL {
            plate_state.mining_timer = 0;

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
// BARRACKS RALLY POINT SYSTEM
// =====================================================

/// System to set Barracks rally point via right-click when a Barracks is selected
pub fn barracks_rally_point_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut selected_barracks: Query<(&Owner, &mut BarracksState), With<Selected>>,
    potential_targets: Query<(Entity, &Transform, &Owner, &SelectionBounds), With<ObjectInstance>>,
    cursor_over_ui: Res<CursorOverUi>,
    local_player: Res<LocalPlayer>,
    panel_state: Res<ObjectInterfaceState>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    if cursor_over_ui.0 {
        return;
    }

    // Only handle when in Barracks menu state
    if !matches!(*panel_state, ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu)) {
        return;
    }

    // Must have a selected barracks
    let Ok((_owner, mut bk_state)) = selected_barracks.get_single_mut() else {
        return;
    };

    let Ok(window) = windows.get_single() else { return };
    let Ok((camera, camera_transform)) = cameras.get_single() else { return };

    let Some(cursor_pos) = cursor_pos_in_viewport(window, camera) else { return };
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };

    // Check for entity under cursor first
    let click_radius = 25.0_f32;
    let mut best_distance = f32::MAX;
    let mut clicked_entity: Option<(Entity, bool)> = None;

    for (target_entity, target_transform, target_owner, _bounds) in potential_targets.iter() {
        let target_pos = target_transform.translation;
        if let Some(screen_pos) = camera.world_to_viewport(camera_transform, target_pos) {
            let dx = cursor_pos.x - screen_pos.x;
            let dy = cursor_pos.y - screen_pos.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= click_radius {
                let cam_distance = (target_pos - camera_transform.translation()).length();
                if cam_distance < best_distance {
                    let is_enemy = !target_owner.is_neutral()
                        && target_owner.player_number() != Some(local_player.0);
                    clicked_entity = Some((target_entity, is_enemy));
                    best_distance = cam_distance;
                }
            }
        }
    }

    if let Some((target_entity, _is_enemy)) = clicked_entity {
        bk_state.rally_point = Some(RallyTarget::Object(target_entity));
        info!("Barracks: Rally point set to entity {:?}", target_entity);
        return;
    }

    // Ground click — set rally to location
    if ray.direction.y.abs() > 0.001 {
        let t = -ray.origin.y / ray.direction.y;
        if t > 0.0 {
            let world_hit = ray.origin + *ray.direction * t;
            bk_state.rally_point = Some(RallyTarget::Location(world_hit));
            info!("Barracks: Rally point set to ({:.1}, {:.1})", world_hit.x, world_hit.z);
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
    mut query: Query<(Entity, &mut ObjectInstance, &mut ConstructionHP)>,
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
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

            // Determine symmetry type for side labels on ghost
            let sym = btype.structure_type().map(|st| st.symmetry_type).unwrap_or(crate::types::SymmetryTypeEnum::AAAA);
            let half_x = w / 2.0;
            let half_z = d / 2.0;

            commands.spawn((
                PbrBundle {
                    mesh,
                    material,
                    transform: Transform::from_xyz(0.0, h / 2.0, 0.0),
                    visibility: Visibility::Hidden,
                    ..default()
                },
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
            commands.entity(entity).despawn_recursive();
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
    mut ghost_query: Query<(&mut Transform, &mut Visibility, &Handle<StandardMaterial>), With<PlacementGhost>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    panel_state: Res<ObjectInterfaceState>,
    mut placement_state: ResMut<PlacementState>,
    build_area: Res<GdoBuildArea>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    structures: Query<(&GridPosition, &StructureInstance)>,
    patches: Query<(&GridPosition, &SpaceCrystalPatch)>,
    tunnel_areas: Query<&TunnelArea>,
    fog_map: Res<FogOfWarMap>,
    local_player: Res<LocalPlayer>,
) {
    if !panel_state.is_placement_mode() {
        return;
    }

    let (mut ghost_transform, mut ghost_vis, mat_handle) = match ghost_query.get_single_mut() {
        Ok(g) => g,
        Err(_) => return,
    };

    let building_type = match placement_state.building_type {
        Some(bt) => bt,
        None => return,
    };

    let window = match windows.get_single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let (camera, camera_transform) = match camera_query.get_single() {
        Ok(c) => c,
        Err(_) => return,
    };

    let cursor_pos = match cursor_pos_in_viewport(window, camera) {
        Some(pos) => pos,
        None => {
            *ghost_vis = Visibility::Hidden;
            return;
        }
    };

    // Raycast to ground plane (Y=0)
    let ray = match camera.viewport_to_world(camera_transform, cursor_pos) {
        Some(r) => r,
        None => return,
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
    if let Some(material) = materials.get_mut(mat_handle) {
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
    mut build_area: ResMut<GdoBuildArea>,
    mut patches: Query<(Entity, &GridPosition, &mut SpaceCrystalPatch)>,
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
                *panel_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfReadyToPlace);
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement) => {
                *panel_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
            }
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement) => {
                *panel_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
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
                            spawn_supply_tower(
                                &mut commands, &mut meshes, &mut materials,
                                grid_x, grid_z, owner, rotation, flip_h, flip_v,
                            );
                            // Supply Tower also spawns a free Supply Chopper
                            spawn_supply_chopper(
                                &mut commands, &mut meshes, &mut materials,
                                grid_x + 1, grid_z + 3, owner,
                            );
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
                // Place tunnel expansion
                if let Ok((owner, mut tunnel_state)) = tunnel_query.get_mut(source_entity) {
                    let owner = *owner;
                    // Set the tunnel's current operation to building expansion
                    tunnel_state.current_operation = Some(TunnelOperation::BuildingExpansion {
                        object: building_type,
                        progress: 0.0,
                    });

                    // Spawn the expansion entity
                    match building_type {
                        ObjectEnum::Headquarters => {
                            spawn_headquarters(
                                &mut commands,
                                grid_x, grid_z, owner, source_entity,
                            );
                        }
                        _ => {
                            info!("Unknown tunnel expansion type: {:?}", building_type);
                        }
                    }

                    info!("Placed tunnel expansion {:?} at ({}, {})", building_type, grid_x, grid_z);
                    *panel_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
                }
            }
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement) => {
                // Agent placement: issue BuildTunnel command to the Agent, don't spawn structure yet
                let world_pos = grid_to_world(grid_x, grid_z, 1.0);
                commands.entity(source_entity)
                    .insert(UnitCommand::BuildTunnel(world_pos));
                info!("Agent: BuildTunnel command to ({}, {})", grid_x, grid_z);
                *panel_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
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

        if is_agent {
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
                        PbrBundle {
                            mesh: mesh_handle,
                            material,
                            transform: Transform::from_xyz(0.0, 0.01, 0.0),
                            ..default()
                        },
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
                PbrBundle {
                    mesh: mesh_handle,
                    material,
                    transform: Transform::from_xyz(0.0, 0.01, 0.0),
                    ..default()
                },
                BuildAreaOverlay,
            ));
        }
    } else if !is_placing && !overlay_query.is_empty() {
        // Despawn overlay
        for entity in overlay_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Check if any existing structure occupies any cell of the given footprint.
fn has_structure_overlap(
    pos_x: i32,
    pos_z: i32,
    size_x: u32,
    size_z: u32,
    structures: &Query<(&GridPosition, &StructureInstance)>,
) -> bool {
    for dx in 0..size_x as i32 {
        for dz in 0..size_z as i32 {
            let check_x = pos_x + dx;
            let check_z = pos_z + dz;
            for (struct_pos, _) in structures.iter() {
                if struct_pos.x == check_x && struct_pos.z == check_z {
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
    use bevy::render::mesh::PrimitiveTopology;

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
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    mesh
}

/// Build a single mesh from all cells in a TunnelArea (for overlay during expansion placement).
fn tunnel_area_mesh(tunnel_area: &TunnelArea) -> Mesh {
    use bevy::render::mesh::PrimitiveTopology;

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
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
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
    mut tunnel_query: Query<(Entity, &Owner, &mut TunnelState, &mut TunnelArea)>,
    mut syndicate_players: Query<(&Player, &mut SyndicatePlayerResources)>,
) {
    for (_entity, owner, mut tunnel_state, mut tunnel_area) in tunnel_query.iter_mut() {
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
            TunnelOperation::BuildingExpansion { object, progress } => {
                // TODO: Check Agent presence (builder_present) once enter_command task lands.
                // For now, construction always progresses.
                let required = syndicate_structure_stats::TUNNEL_CONSTRUCTION_FRAMES as f32;
                let new_progress = progress + 1.0;

                if new_progress >= required {
                    // Construction complete
                    tunnel_state.current_operation = None;
                    // TODO: Spawn the expansion entity once tunnel_expansions task defines types
                    info!("Tunnel expansion {:?} construction complete", object);
                } else {
                    tunnel_state.current_operation = Some(TunnelOperation::BuildingExpansion {
                        object,
                        progress: new_progress,
                    });
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
    mut st_query: Query<(Entity, &Owner, &crate::types::GridPosition, &mut SupplyTowerState)>,
    players: Query<(&Player, &mut GdoPlayerResources)>,
) {
    for (_st_entity, owner, grid_pos, mut st_state) in st_query.iter_mut() {
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
                        // Production complete — spawn chopper near the tower
                        let spawn_x = grid_pos.x + 1;
                        let spawn_z = grid_pos.z + 3;

                        spawn_supply_chopper(
                            &mut commands, &mut meshes, &mut materials,
                            spawn_x, spawn_z, *owner,
                        );

                        info!("Supply Tower: Produced {:?} at ({}, {})", unit_type, spawn_x, spawn_z);

                        st_state.current_build = None;
                        st_state.current_build_progress = None;
                    }
                }
            }
        }
    }
}
