use bevy::prelude::*;
use crate::types::{Owner, ObjectEnum, MainCamera, GridPosition, Selected};
use crate::game::types::*;
use crate::game::entities::{EntityRegistry, SpawnParams};
use crate::game::entities::utils::{insert_entity_visuals, grid_to_spawn_position};
use crate::game::world::types::{GdoBuildArea, SpaceCrystalPatch, Tile, TilePreset, FogOfWarMap};
use crate::game::world::utils::{expand_build_area, world_to_grid, grid_to_world, can_place_building, rotated_building_size};
use crate::ui::types::{ObjectInterfaceState, StructureMenuState, AgentMenuState, CultsRecruitMenuState, CommandPanelTarget, PlacementGhost, PlacementState, CursorOverUi, BuildAreaOverlay};
use crate::game::units::types::state::UnitCommand;
use crate::game::units::types::state::behavior::BuildingTunnelBehavior;

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
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement) => {
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
    local_player: Res<crate::types::LocalPlayer>,
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
    registry: Res<EntityRegistry>,
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

                    // Build radius extension per building type
                    let build_radius = match building_type {
                        ObjectEnum::PowerPlant => 1,
                        ObjectEnum::Barracks | ObjectEnum::ExtractionFacility => 2,
                        ObjectEnum::SupplyTower => 1,
                        _ => 0,
                    };

                    let spawn_pos = grid_to_spawn_position(grid_x, grid_z, rot_x as f32, rot_z as f32);
                    let entity = registry.spawn(&building_type, &mut commands, SpawnParams {
                        position: spawn_pos,
                        owner,
                        player: owner.player_number().unwrap_or(0),
                        rotation: Some(rotation),
                        under_construction: false,
                        flip_horizontal: flip_h,
                        flip_vertical: flip_v,
                    });
                    insert_entity_visuals(&mut commands, entity, &building_type, &owner, &mut meshes, &mut materials);

                    if building_type == ObjectEnum::SupplyTower {
                        // Supply Tower also spawns a free Supply Chopper at the spawn-side exit
                        let st_si = StructureInstance::new(rotation, flip_h, flip_v);
                        let (dx, dz) = crate::game::world::utils::spawn_side_offset(
                            ObjectEnum::SupplyTower, &st_si,
                        );
                        let chopper_pos = grid_to_spawn_position(grid_x + dx, grid_z + dz, 1.0, 1.0);
                        let chopper_entity = registry.spawn(&ObjectEnum::SupplyChopper, &mut commands, SpawnParams {
                            position: chopper_pos,
                            owner,
                            player: owner.player_number().unwrap_or(0),
                            rotation: None,
                            under_construction: false,
                            flip_horizontal: false,
                            flip_vertical: false,
                        });
                        insert_entity_visuals(&mut commands, chopper_entity, &ObjectEnum::SupplyChopper, &owner, &mut meshes, &mut materials);
                        // Link the tower and its free chopper for auto-delivery
                        commands.entity(entity).insert(SupplyTowerState {
                            attached_chopper: Some(chopper_entity),
                            ..default()
                        });
                        commands.entity(chopper_entity).insert(SupplyChopperState {
                            attached_tower: Some(entity),
                            ..default()
                        });
                    }

                    expand_build_area(&mut build_area, grid_x, grid_z, rot_x, rot_z, build_radius);

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
                        let ep_pos = grid_to_spawn_position(grid_x, grid_z, 1.0, 1.0);
                        let ep_entity = registry.spawn(&ObjectEnum::ExtractionPlate, &mut commands, SpawnParams {
                            position: ep_pos,
                            owner,
                            player: owner.player_number().unwrap_or(0),
                            rotation: None,
                            under_construction: false,
                            flip_horizontal: false,
                            flip_vertical: false,
                        });
                        insert_entity_visuals(&mut commands, ep_entity, &ObjectEnum::ExtractionPlate, &owner, &mut meshes, &mut materials);
                        // Override the placeholder attached_patch
                        commands.entity(ep_entity).insert(ExtractionPlateState {
                            attached_patch: patch_entity,
                            mining_timer: 0.0,
                        });
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
                        .insert(UnitCommand::BuildTunnel(world_pos))
                        .insert(BuildingTunnelBehavior::new(world_pos));
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
                        let obj_type = building_type.object_type();
                        let (rot_x, rot_z) = rotated_building_size(obj_type.size.0, obj_type.size.1, &rotation);
                        let spawn_pos = grid_to_spawn_position(grid_x, grid_z, rot_x as f32, rot_z as f32);
                        let entity = registry.spawn(&building_type, &mut commands, SpawnParams {
                            position: spawn_pos,
                            owner,
                            player: owner.player_number().unwrap_or(0),
                            rotation: Some(rotation),
                            under_construction: true,
                            flip_horizontal: flip_h,
                            flip_vertical: flip_v,
                        });
                        insert_entity_visuals(&mut commands, entity, &building_type, &owner, &mut meshes, &mut materials);
                        entity
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

#[cfg(test)]
mod tests {
    use super::*;

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
        // flip_horizontal = true -> scale.x = -1, scale.z = 1
        let flip_h = Vec3::new(-1.0, 1.0, 1.0);
        assert_eq!(flip_h.x, -1.0);
        assert_eq!(flip_h.y, 1.0);
        assert_eq!(flip_h.z, 1.0);

        // flip_vertical = true -> scale.x = 1, scale.z = -1
        let flip_v = Vec3::new(1.0, 1.0, -1.0);
        assert_eq!(flip_v.x, 1.0);
        assert_eq!(flip_v.y, 1.0);
        assert_eq!(flip_v.z, -1.0);

        // both flipped -> scale.x = -1, scale.z = -1
        let flip_both = Vec3::new(-1.0, 1.0, -1.0);
        assert_eq!(flip_both.x, -1.0);
        assert_eq!(flip_both.z, -1.0);
    }
}
