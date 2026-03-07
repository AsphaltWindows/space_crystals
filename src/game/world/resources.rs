use bevy::prelude::*;
use crate::types::*;
use crate::game::types::ObjectInstance;
use crate::game::types::objects::StructureInstance;
use crate::ui::types::{CursorOverUi, ObjectInterfaceState};
use super::types::*;
use super::utils::{screen_space_hit_test, BoxCandidate, SelectionTier, closest_to_center, classify_selection_tier, cursor_pos_in_viewport, viewport_offset};

/// Spawn Space Crystal Patches on the map
pub fn spawn_space_crystal_patches(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tiles: Query<(&GridPosition, &mut TilePreset), With<Tile>>,
) {
    // Space Crystal Patches distributed across the 64x64 map
    let scp_data = [
        // Near DC starting position (30,30) — within build radius for early mining
        ((34, 28), 2500),   // East of DC, close
        ((26, 32), 2000),   // West of DC, close
        ((32, 36), 2500),   // Southeast of DC, close
        // Strategic map positions
        ((20, 20), 5000),   // NW quadrant
        ((44, 20), 3500),   // NE quadrant
        ((20, 44), 2000),   // SW quadrant
        ((44, 44), 4200),   // SE quadrant
        ((32, 15), 3000),   // North center
        ((32, 49), 3000),   // South center
        ((15, 32), 2500),   // West center
        ((49, 32), 2500),   // East center
    ];

    let crystal_mesh = meshes.add(Cuboid::new(0.6, 0.8, 0.6));

    let crystal_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.8, 1.0),
        emissive: Color::srgb(0.2, 0.6, 0.8).into(),
        ..default()
    });

    let mut spawned_count = 0;

    for ((grid_x, grid_z), amount) in scp_data {
        for (tile_pos, mut properties) in tiles.iter_mut() {
            if tile_pos.x == grid_x && tile_pos.z == grid_z {
                properties.buildable = false;
                properties.traversible = false;

                let world_x = (grid_x as f32 - 32.0) + 0.5;
                let world_z = (grid_z as f32 - 32.0) + 0.5;

                commands.spawn((
                    PbrBundle {
                        mesh: crystal_mesh.clone(),
                        material: crystal_material.clone(),
                        transform: Transform::from_xyz(world_x, 0.4, world_z),
                        ..default()
                    },
                    ObjectInstance::indestructible(ObjectEnum::SpaceCrystalsPatch),
                    Owner::neutral(),
                    SpaceCrystalPatch {
                        remaining_amount: amount,
                        initial_amount: amount,
                        has_plate: false,
                    },
                    Selectable,
                    SelectionBounds::from_dimensions(0.6, 0.8, 0.6),
                    GridPosition { x: grid_x, z: grid_z },
                ));

                spawned_count += 1;
                break;
            }
        }
    }

    info!("Spawned {} Space Crystal Patches", spawned_count);
}

/// System to handle clicking on selectable entities
pub fn selection_system(
    mut commands: Commands,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    selectables: Query<(Entity, &Transform, Option<&SelectionBounds>, Option<&SpaceCrystalPatch>, Option<&SupplyDeliveryStation>, &Owner), With<Selectable>>,
    selected: Query<Entity, With<Selected>>,
    selection_state: Res<SelectionState>,
    cursor_over_ui: Res<CursorOverUi>,
    interface_state: Res<ObjectInterfaceState>,
    local_player: Res<LocalPlayer>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    if cursor_over_ui.0 {
        return;
    }

    // Don't select during placement mode — clicks are for placing buildings
    if interface_state.is_placement_mode() {
        return;
    }

    // Don't select during command mode — clicks are for confirming command targets
    if interface_state.is_awaiting_target() {
        return;
    }

    if selection_state.is_dragging {
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = cameras.single();
    let ctrl_pressed = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if let Some(cursor_pos) = cursor_pos_in_viewport(window, camera) {
        // Screen-space hit testing: project each entity to screen coordinates
        // and check if the cursor click is within a pixel radius
        let click_radius = 25.0_f32; // pixels

        {
            let mut closest_entity = None;
            let mut closest_distance = f32::MAX;

            for (entity, transform, _bounds, scp, sds, owner) in selectables.iter() {
                let entity_pos = transform.translation;

                if let Some(screen_pos) = camera.world_to_viewport(camera_transform, entity_pos) {
                    if screen_space_hit_test(cursor_pos, screen_pos, click_radius).is_some() {
                        // Use distance from camera as tiebreaker (closer to camera wins)
                        let cam_distance = (entity_pos - camera_transform.translation()).length();
                        if cam_distance < closest_distance {
                            closest_distance = cam_distance;
                            closest_entity = Some((entity, scp, sds, *owner));
                        }
                    }
                }
            }

            if let Some((entity, scp, sds, owner)) = closest_entity {
                let is_owned = owner.0 == Some(local_player.0);

                if !is_owned {
                    // Non-owned entity: always single-select, ignore Ctrl
                    for other_entity in selected.iter() {
                        commands.entity(other_entity).remove::<Selected>();
                    }
                    commands.entity(entity).insert(Selected);
                } else if ctrl_pressed {
                    // Owned entity + Ctrl: check if non-owned is currently selected
                    // If so, deselect non-owned first, then select this owned entity
                    let has_non_owned_selected = selected.iter().any(|sel_entity| {
                        selectables.get(sel_entity)
                            .map(|(_, _, _, _, _, o)| o.0 != Some(local_player.0))
                            .unwrap_or(false)
                    });

                    if has_non_owned_selected {
                        // Transition: deselect all non-owned, select this owned entity
                        for other_entity in selected.iter() {
                            commands.entity(other_entity).remove::<Selected>();
                        }
                        commands.entity(entity).insert(Selected);
                    } else if selected.contains(entity) {
                        commands.entity(entity).remove::<Selected>();
                    } else {
                        commands.entity(entity).insert(Selected);
                    }
                } else {
                    for other_entity in selected.iter() {
                        if other_entity != entity {
                            commands.entity(other_entity).remove::<Selected>();
                        }
                    }
                    commands.entity(entity).insert(Selected);
                }

                if let Some(patch) = scp {
                    info!(
                        "Space Crystal Patch: {} / {} remaining ({:.1}%){}",
                        patch.remaining_amount,
                        patch.initial_amount,
                        (patch.remaining_amount as f32 / patch.initial_amount as f32) * 100.0,
                        if patch.has_plate { " [Plate Attached]" } else { "" }
                    );
                }

                if let Some(station) = sds {
                    if station.current_supplies > 0 {
                        info!(
                            "Supply Delivery Station: {} supplies available",
                            station.current_supplies
                        );
                    } else {
                        info!(
                            "Supply Delivery Station: Empty | Next delivery in {:.1} seconds (Size: {})",
                            station.time_until_next_delivery,
                            station.delivery_size
                        );
                    }
                }
            } else if !ctrl_pressed {
                for entity in selected.iter() {
                    commands.entity(entity).remove::<Selected>();
                }
            }
        }
    }
}

/// Determine the tier of the current selection by examining selected entities.
fn current_selection_tier(
    selected: &Query<Entity, With<Selected>>,
    selectables: &Query<(Entity, &Transform, &Owner, Option<&Unit>, Option<&StructureInstance>), With<Selectable>>,
    local_player_id: u8,
) -> Option<SelectionTier> {
    for entity in selected.iter() {
        if let Ok((_, _, owner, unit_marker, structure_marker)) = selectables.get(entity) {
            let is_owned = owner.0 == Some(local_player_id);
            let is_neutral = owner.is_neutral();
            let is_unit = unit_marker.is_some();
            let is_structure = structure_marker.is_some();
            return Some(classify_selection_tier(is_owned, is_neutral, is_unit, is_structure));
        }
    }
    None
}

/// System to handle drag-box selection with 5-tier priority system.
///
/// Priority tiers (highest to lowest):
/// 1. Own units — multi-select all
/// 2. Own structures — single-select closest to box center
/// 3. Enemy units — single-select closest to box center
/// 4. Enemy structures — single-select closest to box center
/// 5. Neutral objects — single-select closest to box center
pub fn drag_box_system(
    mut commands: Commands,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection_state: ResMut<SelectionState>,
    selectables: Query<(Entity, &Transform, &Owner, Option<&Unit>, Option<&StructureInstance>), With<Selectable>>,
    selected: Query<Entity, With<Selected>>,
    cursor_over_ui: Res<CursorOverUi>,
    interface_state: Res<ObjectInterfaceState>,
    local_player: Res<LocalPlayer>,
) {
    // Don't start drag during placement mode
    if interface_state.is_placement_mode() {
        return;
    }

    // Don't start drag during command mode — clicks are for confirming command targets
    if interface_state.is_awaiting_target() {
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = cameras.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if buttons.just_pressed(MouseButton::Left) && selection_state.drag_start.is_none() && !cursor_over_ui.0 {
            selection_state.drag_start = Some(cursor_pos);
            selection_state.is_dragging = false;
        }

        if buttons.pressed(MouseButton::Left) {
            if let Some(start_pos) = selection_state.drag_start {
                let drag_distance = (cursor_pos - start_pos).length();
                if drag_distance > 5.0 {
                    selection_state.is_dragging = true;
                }
            }
        }

        if buttons.just_released(MouseButton::Left) {
            if selection_state.is_dragging {
                if let Some(start_pos) = selection_state.drag_start {
                    // Convert drag box bounds from window-space to viewport-space
                    // for comparison with world_to_viewport results
                    let vp_offset = viewport_offset(camera, window.scale_factor());
                    let vp_start = start_pos - vp_offset;
                    let vp_cursor = cursor_pos - vp_offset;
                    let min_x = vp_start.x.min(vp_cursor.x);
                    let max_x = vp_start.x.max(vp_cursor.x);
                    let min_y = vp_start.y.min(vp_cursor.y);
                    let max_y = vp_start.y.max(vp_cursor.y);
                    let box_center = Vec2::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);

                    let ctrl_pressed = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

                    // Pass 1: Collect entities in box, categorized by tier
                    let mut own_units: Vec<BoxCandidate> = Vec::new();
                    let mut own_structures: Vec<BoxCandidate> = Vec::new();
                    let mut enemy_units: Vec<BoxCandidate> = Vec::new();
                    let mut enemy_structures: Vec<BoxCandidate> = Vec::new();
                    let mut neutrals: Vec<BoxCandidate> = Vec::new();

                    for (entity, transform, owner, unit_marker, structure_marker) in selectables.iter() {
                        if let Some(screen_pos) = camera.world_to_viewport(camera_transform, transform.translation) {
                            if screen_pos.x >= min_x && screen_pos.x <= max_x &&
                               screen_pos.y >= min_y && screen_pos.y <= max_y {
                                let candidate = BoxCandidate { entity, screen_pos };
                                let is_owned = owner.0 == Some(local_player.0);
                                let is_neutral = owner.is_neutral();
                                let is_unit = unit_marker.is_some();
                                let is_structure = structure_marker.is_some();

                                if is_owned && is_unit { own_units.push(candidate); }
                                else if is_owned && is_structure { own_structures.push(candidate); }
                                else if !is_neutral && is_unit { enemy_units.push(candidate); }
                                else if !is_neutral && is_structure { enemy_structures.push(candidate); }
                                else { neutrals.push(candidate); }
                            }
                        }
                    }

                    // Pass 2: Apply priority selection
                    let (entities_to_select, new_tier): (Vec<Entity>, Option<SelectionTier>) =
                        if !own_units.is_empty() {
                            (own_units.iter().map(|c| c.entity).collect(), Some(SelectionTier::OwnUnits))
                        } else if !own_structures.is_empty() {
                            (vec![closest_to_center(&own_structures, box_center)], Some(SelectionTier::OwnStructures))
                        } else if !enemy_units.is_empty() {
                            (vec![closest_to_center(&enemy_units, box_center)], Some(SelectionTier::EnemyUnits))
                        } else if !enemy_structures.is_empty() {
                            (vec![closest_to_center(&enemy_structures, box_center)], Some(SelectionTier::EnemyStructures))
                        } else if !neutrals.is_empty() {
                            (vec![closest_to_center(&neutrals, box_center)], Some(SelectionTier::Neutrals))
                        } else {
                            (vec![], None)
                        };

                    if entities_to_select.is_empty() {
                        // No entities in box — do nothing (keep current selection)
                    } else if ctrl_pressed {
                        // Ctrl-additive: enforce tier matching
                        let existing_tier = current_selection_tier(&selected, &selectables, local_player.0);

                        if let Some(existing) = existing_tier {
                            if let Some(new) = new_tier {
                                if existing != new {
                                    // Cross-tier mixing — no-op
                                } else if existing == SelectionTier::OwnUnits {
                                    // Same tier (own units) — add all to selection
                                    for entity in entities_to_select {
                                        commands.entity(entity).insert(Selected);
                                    }
                                } else {
                                    // Tiers 2-5 are single-select — replace
                                    for entity in selected.iter() {
                                        commands.entity(entity).remove::<Selected>();
                                    }
                                    for entity in entities_to_select {
                                        commands.entity(entity).insert(Selected);
                                    }
                                }
                            }
                        } else {
                            // Nothing currently selected — select new entities
                            for entity in entities_to_select {
                                commands.entity(entity).insert(Selected);
                            }
                        }
                    } else {
                        // No Ctrl — replace selection
                        for entity in selected.iter() {
                            commands.entity(entity).remove::<Selected>();
                        }
                        for entity in entities_to_select {
                            commands.entity(entity).insert(Selected);
                        }
                    }
                }
            }

            selection_state.drag_start = None;
            selection_state.is_dragging = false;
        }
    }
}

/// System to log selection count changes
pub fn log_selection_changes(
    selected: Query<Entity, (With<Selected>, Changed<Selected>)>,
    all_selected: Query<Entity, With<Selected>>,
) {
    if !selected.is_empty() {
        let count = all_selected.iter().count();
        info!("Selection changed: {} unit(s)/entity(ies) selected", count);
    }
}

/// System to draw the drag-box UI
pub fn draw_drag_box_ui(
    mut commands: Commands,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    selection_state: Res<SelectionState>,
    drag_box: Query<Entity, With<DragBoxUI>>,
    ui_cam: Res<crate::ui::types::UiCameraEntity>,
) {
    let window = windows.single();

    if selection_state.is_dragging {
        if let Some(start_pos) = selection_state.drag_start {
            if let Some(cursor_pos) = window.cursor_position() {
                for entity in drag_box.iter() {
                    commands.entity(entity).despawn();
                }

                let min_x = start_pos.x.min(cursor_pos.x);
                let max_x = start_pos.x.max(cursor_pos.x);
                let min_y = start_pos.y.min(cursor_pos.y);
                let max_y = start_pos.y.max(cursor_pos.y);
                let width = max_x - min_x;
                let height = max_y - min_y;

                commands.spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Px(min_x),
                            top: Val::Px(min_y),
                            width: Val::Px(width),
                            height: Val::Px(height),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        border_color: Color::srgba(1.0, 1.0, 0.0, 1.0).into(),
                        background_color: Color::srgba(1.0, 1.0, 0.0, 0.1).into(),
                        ..default()
                    },
                    TargetCamera(ui_cam.0),
                    DragBoxUI,
                ));
            }
        }
    } else {
        for entity in drag_box.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// System to add/remove selection indicators for selected entities
pub fn manage_selection_indicators(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    selected: Query<(Entity, &Children), (With<Selected>, Changed<Selected>)>,
    newly_selected: Query<Entity, (With<Selected>, Without<Children>)>,
    deselected: Query<Entity, (Without<Selected>, With<Children>)>,
    indicators: Query<Entity, With<SelectionIndicator>>,
) {
    for entity in newly_selected.iter() {
        let indicator_mesh = meshes.add(Torus::new(0.4, 0.05));
        let indicator_material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 0.0),
            emissive: Color::srgb(1.0, 1.0, 0.0).into(),
            ..default()
        });

        let indicator = commands.spawn((
            PbrBundle {
                mesh: indicator_mesh,
                material: indicator_material,
                transform: Transform::from_xyz(0.0, -0.3, 0.0)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.0)),
                ..default()
            },
            SelectionIndicator,
        )).id();

        commands.entity(entity).add_child(indicator);
    }

    for entity in deselected.iter() {
        if let Some(_entity_commands) = commands.get_entity(entity) {
            if let Ok(children) = selected.get(entity).map(|(_, c)| c) {
                for &child in children.iter() {
                    if indicators.contains(child) {
                        commands.entity(child).despawn();
                    }
                }
            }
        }
    }
}

/// Spawn Supply Delivery Stations on the map
pub fn spawn_supply_delivery_stations(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tiles: Query<(&GridPosition, &mut TilePreset), With<Tile>>,
) {
    // Supply Delivery Stations distributed across the 64x64 map
    let sds_data = [
        ((10, 32), 100, 60.0),   // West side
        ((54, 32), 200, 45.0),   // East side
        ((32, 10), 150, 90.0),   // North side
        ((32, 54), 150, 75.0),   // South side
    ];

    let platform_mesh = meshes.add(Cylinder::new(0.8, 0.2));

    let platform_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.6),
        metallic: 0.8,
        perceptual_roughness: 0.3,
        ..default()
    });

    let mut spawned_count = 0;

    for ((grid_x, grid_z), delivery_size, delivery_interval) in sds_data {
        for (tile_pos, mut properties) in tiles.iter_mut() {
            if tile_pos.x == grid_x && tile_pos.z == grid_z {
                properties.traversible = false;

                let world_x = (grid_x as f32 - 32.0) + 0.5;
                let world_z = (grid_z as f32 - 32.0) + 0.5;

                commands.spawn((
                    PbrBundle {
                        mesh: platform_mesh.clone(),
                        material: platform_material.clone(),
                        transform: Transform::from_xyz(world_x, 0.1, world_z),
                        ..default()
                    },
                    ObjectInstance::indestructible(ObjectEnum::SupplyDeliveryStation),
                    Owner::neutral(),
                    SupplyDeliveryStation {
                        delivery_size,
                        delivery_interval,
                        current_supplies: delivery_size,
                        time_until_next_delivery: delivery_interval,
                    },
                    Selectable,
                    SelectionBounds::from_dimensions(1.6, 0.2, 1.6),
                    GridPosition { x: grid_x, z: grid_z },
                ));

                spawned_count += 1;
                break;
            }
        }
    }

    info!("Spawned {} Supply Delivery Stations", spawned_count);
}

/// System to handle SDS delivery timers
pub fn sds_delivery_timer(
    time: Res<Time>,
    mut stations: Query<&mut SupplyDeliveryStation>,
) {
    for mut sds in stations.iter_mut() {
        if sds.current_supplies == 0 {
            sds.time_until_next_delivery -= time.delta_seconds();

            if sds.time_until_next_delivery <= 0.0 {
                sds.current_supplies = sds.delivery_size;
                sds.time_until_next_delivery = sds.delivery_interval;
                info!("Supply Delivery: {} supplies delivered", sds.delivery_size);
            }
        }
    }
}

// =====================================================
// CONTROL GROUP SYSTEM
// =====================================================

/// Maps a digit KeyCode to a control group index (0-9).
/// Digit1 → 0, Digit2 → 1, ..., Digit9 → 8, Digit0 → 9
fn digit_key_to_group_index(key: KeyCode) -> Option<usize> {
    match key {
        KeyCode::Digit1 => Some(0),
        KeyCode::Digit2 => Some(1),
        KeyCode::Digit3 => Some(2),
        KeyCode::Digit4 => Some(3),
        KeyCode::Digit5 => Some(4),
        KeyCode::Digit6 => Some(5),
        KeyCode::Digit7 => Some(6),
        KeyCode::Digit8 => Some(7),
        KeyCode::Digit9 => Some(8),
        KeyCode::Digit0 => Some(9),
        _ => None,
    }
}

/// System to handle control group input:
/// - Ctrl+Number: assign current selection to group
/// - Shift+Number: add current selection to group
/// - Number (no modifier): recall group as selection (double-tap: center camera)
pub fn control_group_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut control_groups: ResMut<ControlGroups>,
    selected_entities: Query<(Entity, &Owner), With<Selected>>,
    all_entities: Query<Entity, With<ObjectInstance>>,
    interface_state: Res<ObjectInterfaceState>,
    local_player: Res<LocalPlayer>,
    time: Res<Time>,
    mut last_recall: ResMut<LastRecallState>,
    entity_transforms: Query<&Transform, (Without<MainCamera>, With<ObjectInstance>)>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<ObjectInstance>)>,
) {
    // Don't process during placement mode
    if interface_state.is_placement_mode() {
        return;
    }

    let ctrl_held = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let alt_held = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);

    // Only process number keys
    let digit_keys = [
        KeyCode::Digit0, KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
        KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6, KeyCode::Digit7,
        KeyCode::Digit8, KeyCode::Digit9,
    ];

    for &key in &digit_keys {
        if !keyboard.just_pressed(key) {
            continue;
        }

        let Some(group_idx) = digit_key_to_group_index(key) else {
            continue;
        };

        if alt_held {
            // Ignore Alt+Number
            continue;
        }

        if ctrl_held {
            // Ctrl+Number: Assign current selection to group (replace)
            // Only include entities owned by the local player
            let selected: Vec<Entity> = selected_entities.iter()
                .filter(|(_, owner)| owner.0 == Some(local_player.0))
                .map(|(e, _)| e)
                .collect();
            if !selected.is_empty() {
                control_groups.groups[group_idx] = selected;
                info!("Control group {}: assigned {} entities",
                    group_idx + 1, control_groups.groups[group_idx].len());
            }
        } else if shift_held {
            // Shift+Number: Add current selection to group (merge, no duplicates)
            // Only include entities owned by the local player
            for (entity, owner) in selected_entities.iter() {
                if owner.0 == Some(local_player.0) && !control_groups.groups[group_idx].contains(&entity) {
                    control_groups.groups[group_idx].push(entity);
                }
            }
            info!("Control group {}: now {} entities (after add)",
                group_idx + 1, control_groups.groups[group_idx].len());
        } else {
            // Number only: Recall group as selection
            // First, clean up dead entities
            control_groups.groups[group_idx].retain(|&entity| all_entities.get(entity).is_ok());

            let group = &control_groups.groups[group_idx];
            if group.is_empty() {
                continue;
            }

            let current_time = time.elapsed_seconds_f64();

            // Check for double-tap: center camera on group centroid
            if last_recall.is_double_tap(group_idx, current_time) {
                let mut centroid = Vec3::ZERO;
                let mut count = 0;
                for &entity in group {
                    if let Ok(transform) = entity_transforms.get(entity) {
                        centroid += transform.translation;
                        count += 1;
                    }
                }
                if count > 0 {
                    centroid /= count as f32;
                    if let Ok(mut cam_transform) = camera_query.get_single_mut() {
                        // Move camera X/Z to centroid, keep Y (height) unchanged
                        cam_transform.translation.x = centroid.x;
                        cam_transform.translation.z = centroid.z;
                        info!("Control group {}: centered camera at ({:.1}, {:.1})",
                            group_idx + 1, centroid.x, centroid.z);
                    }
                }
                // Reset recall state after centering
                last_recall.group_index = None;
                last_recall.timestamp = 0.0;
            } else {
                // First tap: recall the group
                // Deselect all currently selected
                for (entity, _) in selected_entities.iter() {
                    commands.entity(entity).remove::<Selected>();
                }

                // Select all entities in the group
                for &entity in group {
                    commands.entity(entity).insert(Selected);
                }

                info!("Control group {}: recalled {} entities", group_idx + 1, group.len());

                // Track this recall for double-tap detection
                last_recall.group_index = Some(group_idx);
                last_recall.timestamp = current_time;
            }
        }

        // Only process one digit key per frame
        break;
    }
}

/// System to sync the Selection resource from the Selected marker components.
/// Runs every frame after selection/drag-box/control-group systems have modified Selected markers.
/// Builds type-based SelectionGroups from all entities with the Selected component.
pub fn selection_group_sync_system(
    selected_query: Query<(Entity, &ObjectInstance), (With<Selected>, Or<(With<Unit>, With<StructureInstance>)>)>,
    mut selection: ResMut<Selection>,
) {
    // Collect current selected entities with their type info
    let entities: Vec<(Entity, ObjectEnum, bool)> = selected_query
        .iter()
        .map(|(entity, obj_instance)| {
            let object_type = obj_instance.object_type;
            let groupable = object_type.object_type().groupable;
            (entity, object_type, groupable)
        })
        .collect();

    // Check if selection actually changed to avoid unnecessary rebuilds
    let current_count = selection.total_entity_count();
    let new_count = entities.len();

    // Quick check: if counts differ, rebuild
    if current_count != new_count {
        selection.build_from_entities(&entities);
        return;
    }

    // Deeper check: verify all entities still match
    let all_match = entities.iter().all(|(e, _, _)| selection.contains_entity(*e));
    if !all_match {
        selection.build_from_entities(&entities);
    }
}

/// System to handle Tab key for cycling the active selection group
pub fn active_group_cycle_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<Selection>,
    interface_state: Res<ObjectInterfaceState>,
) {
    // Don't cycle during placement mode
    if interface_state.is_placement_mode() {
        return;
    }

    // Tab cycling is now also handled in command_panel_hotkeys, but this system
    // handles it when panel is hidden (no units selected, raw Tab behavior)
    if keyboard.just_pressed(KeyCode::Tab) {
        selection.cycle_active_group();
        if let Some(group) = selection.active_group() {
            info!("Active group: {} ({} entities)",
                group.object_type.object_type().name,
                group.entities.len());
        }
    }
}

/// System to validate the selection each tick.
/// Removes dead entities from Selection and keeps Selected markers in sync.
pub fn selection_validation_system(
    mut commands: Commands,
    mut selection: ResMut<Selection>,
    object_instances: Query<&ObjectInstance>,
    selected_query: Query<Entity, With<Selected>>,
) {
    let mut changed = false;

    // Collect entities to remove (dead or despawned)
    let mut to_remove = Vec::new();
    for group in &selection.groups {
        for &entity in &group.entities {
            let should_remove = match object_instances.get(entity) {
                Ok(obj) => !obj.is_alive(),
                Err(_) => true, // Entity no longer exists
            };
            if should_remove {
                to_remove.push(entity);
            }
        }
    }

    // Remove dead entities
    for entity in &to_remove {
        selection.remove_entity(*entity);
        // Remove Selected marker if entity still exists
        if let Some(mut entity_commands) = commands.get_entity(*entity) {
            entity_commands.remove::<Selected>();
        }
        changed = true;
    }

    if changed {
        // Ensure Selected markers are in sync
        for sel_entity in selected_query.iter() {
            if !selection.contains_entity(sel_entity) {
                commands.entity(sel_entity).remove::<Selected>();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::game::types::ObjectInstance;
    use crate::game::types::objects::StructureInstance;

    /// Helper to create a minimal test world with the Selection resource
    fn setup_test_world() -> World {
        let mut world = World::new();
        world.insert_resource(Selection::default());
        world
    }

    #[test]
    fn selection_group_sync_includes_unit_entities() {
        let mut world = setup_test_world();

        // Spawn a unit entity with Selected + ObjectInstance + Unit
        world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::Peacekeeper),
            Unit,
        ));

        world.run_system_once(selection_group_sync_system);

        let selection = world.resource::<Selection>();
        assert_eq!(selection.groups.len(), 1, "Unit should appear in selection groups");
        assert_eq!(selection.groups[0].object_type, ObjectEnum::Peacekeeper);
    }

    #[test]
    fn selection_group_sync_includes_structure_entities() {
        let mut world = setup_test_world();

        // Spawn a structure entity with Selected + ObjectInstance + StructureInstance
        world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::DeploymentCenter),
            StructureInstance::default(),
        ));

        world.run_system_once(selection_group_sync_system);

        let selection = world.resource::<Selection>();
        assert_eq!(selection.groups.len(), 1, "Structure should appear in selection groups");
        assert_eq!(selection.groups[0].object_type, ObjectEnum::DeploymentCenter);
    }

    #[test]
    fn selection_group_sync_excludes_neutral_objects_without_unit_or_structure() {
        let mut world = setup_test_world();

        // Spawn a SupplyDeliveryStation — has Selected + ObjectInstance but NOT Unit or StructureInstance
        world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::SupplyDeliveryStation),
        ));

        world.run_system_once(selection_group_sync_system);

        let selection = world.resource::<Selection>();
        assert!(selection.groups.is_empty(), "Neutral object without Unit/StructureInstance should NOT appear in selection groups");
    }

    #[test]
    fn selection_group_sync_excludes_entity_with_only_object_instance() {
        let mut world = setup_test_world();

        // Spawn an entity with ObjectInstance but no Unit/StructureInstance marker
        // (simulates any neutral/non-commandable object like SpaceCrystalPatch)
        world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::Tunnel),
        ));

        world.run_system_once(selection_group_sync_system);

        let selection = world.resource::<Selection>();
        assert!(selection.groups.is_empty(), "Entity with only ObjectInstance (no Unit/StructureInstance) should NOT appear in selection groups");
    }

    #[test]
    fn selection_group_sync_mixed_selection_only_includes_units_and_structures() {
        let mut world = setup_test_world();

        // Spawn a unit
        world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::Peacekeeper),
            Unit,
        ));

        // Spawn a neutral SDS (no Unit or StructureInstance)
        world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::SupplyDeliveryStation),
        ));

        // Spawn a structure
        world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::DeploymentCenter),
            StructureInstance::default(),
        ));

        world.run_system_once(selection_group_sync_system);

        let selection = world.resource::<Selection>();
        // Should have 2 groups: Peacekeeper and DeploymentCenter (SDS excluded)
        assert_eq!(selection.groups.len(), 2, "Only Unit and Structure entities should be in groups");
        let types: Vec<ObjectEnum> = selection.groups.iter().map(|g| g.object_type).collect();
        assert!(types.contains(&ObjectEnum::Peacekeeper), "Peacekeeper should be in groups");
        assert!(types.contains(&ObjectEnum::DeploymentCenter), "DeploymentCenter should be in groups");
    }

    #[test]
    fn sds_selection_produces_empty_groups() {
        // When only SDS is selected, groups should be empty.
        // The command panel checks `!selection.groups.is_empty()` to decide visibility,
        // so empty groups = no phantom panel.
        let mut world = setup_test_world();

        world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::SupplyDeliveryStation),
        ));

        world.run_system_once(selection_group_sync_system);

        let selection = world.resource::<Selection>();
        assert!(selection.groups.is_empty(), "SDS-only selection should produce empty groups (prevents phantom panel)");
    }

    #[test]
    fn selection_groups_cleared_when_switching_from_unit_to_neutral() {
        let mut world = setup_test_world();

        // First: select a unit
        let unit_entity = world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::Peacekeeper),
            Unit,
        )).id();

        world.run_system_once(selection_group_sync_system);
        let selection = world.resource::<Selection>();
        assert_eq!(selection.groups.len(), 1, "Unit should be in groups");

        // Now: deselect unit, select SDS instead
        world.entity_mut(unit_entity).remove::<Selected>();
        world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::SupplyDeliveryStation),
        ));

        world.run_system_once(selection_group_sync_system);
        let selection = world.resource::<Selection>();
        assert!(selection.groups.is_empty(), "After switching to neutral-only selection, groups should be empty");
    }
}
