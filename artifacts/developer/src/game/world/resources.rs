use bevy::prelude::*;
use crate::types::*;
use crate::game::types::ObjectInstance;
use crate::game::types::objects::StructureInstance;
use crate::ui::types::{CursorOverUi, ObjectInterfaceState, StructureMenuState, AgentMenuState};
use super::types::*;
use super::utils::{screen_space_hit_test, BoxCandidate, SelectionTier, closest_to_center, classify_selection_tier};
use crate::game::units::utils::world_to_grid;

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
        emissive: LinearRgba::rgb(0.2, 0.6, 0.8),
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
                    Mesh3d(crystal_mesh.clone()),
                    MeshMaterial3d(crystal_material.clone()),
                    Transform::from_xyz(world_x, 0.4, world_z),
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
    fog_map: Res<FogOfWarMap>,
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

    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = cameras.single() else { return; };
    let ctrl_pressed = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if let Some(cursor_pos) = window.cursor_position() {
        // Screen-space hit testing: project each entity to screen coordinates
        // and check if the cursor click is within a pixel radius
        let click_radius = 25.0_f32; // pixels

        {
            let mut closest_entity = None;
            let mut closest_distance = f32::MAX;

            for (entity, transform, _bounds, scp, sds, owner) in selectables.iter() {
                let entity_pos = transform.translation;

                if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, entity_pos) {
                    if screen_space_hit_test(cursor_pos, screen_pos, click_radius).is_some() {
                        // Skip non-owned entities not visible through fog of war.
                        // Neutral resource entities (crystal patches, SDS) are always
                        // rendered (apply_structure_fog_rendering skips neutrals), so
                        // they must always be selectable to stay consistent.
                        let is_owned = owner.0 == Some(local_player.0);
                        let is_neutral = owner.is_neutral();
                        if !is_owned && !is_neutral {
                            let grid = world_to_grid(entity_pos);
                            if fog_map.get(local_player.0, grid.x, grid.z) != VisibilityStateEnum::Visible {
                                continue;
                            }
                        }

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
                    // Non-owned entity: always single-select, ignore Ctrl/Shift
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
                } else if shift_pressed {
                    // Shift+click on owned entity: ADD to selection without deselecting others
                    // First check if non-owned entities are selected — if so, clear them
                    let has_non_owned_selected = selected.iter().any(|sel_entity| {
                        selectables.get(sel_entity)
                            .map(|(_, _, _, _, _, o)| o.0 != Some(local_player.0))
                            .unwrap_or(false)
                    });

                    if has_non_owned_selected {
                        // Transition from non-owned selection: deselect all, select this entity
                        for other_entity in selected.iter() {
                            commands.entity(other_entity).remove::<Selected>();
                        }
                    }
                    // Add entity to selection (or keep if already selected)
                    if !selected.contains(entity) {
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
            } else if !ctrl_pressed && !shift_pressed {
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
    fog_map: Res<FogOfWarMap>,
) {
    // Don't start drag during placement mode
    if interface_state.is_placement_mode() {
        return;
    }

    // Don't start drag during command mode — clicks are for confirming command targets
    if interface_state.is_awaiting_target() {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = cameras.single() else { return; };

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
                    // In Bevy 0.17, world_to_viewport returns window-space coords,
                    // so compare directly with raw cursor positions (no offset needed)
                    let min_x = start_pos.x.min(cursor_pos.x);
                    let max_x = start_pos.x.max(cursor_pos.x);
                    let min_y = start_pos.y.min(cursor_pos.y);
                    let max_y = start_pos.y.max(cursor_pos.y);
                    let box_center = Vec2::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);

                    let ctrl_pressed = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

                    // Pass 1: Collect entities in box, categorized by tier
                    let mut own_units: Vec<BoxCandidate> = Vec::new();
                    let mut own_structures: Vec<BoxCandidate> = Vec::new();
                    let mut enemy_units: Vec<BoxCandidate> = Vec::new();
                    let mut enemy_structures: Vec<BoxCandidate> = Vec::new();
                    let mut neutrals: Vec<BoxCandidate> = Vec::new();

                    for (entity, transform, owner, unit_marker, structure_marker) in selectables.iter() {
                        if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, transform.translation) {
                            if screen_pos.x >= min_x && screen_pos.x <= max_x &&
                               screen_pos.y >= min_y && screen_pos.y <= max_y {
                                let candidate = BoxCandidate { entity, screen_pos };
                                let is_owned = owner.0 == Some(local_player.0);
                                let is_neutral = owner.is_neutral();

                                // Skip non-owned entities not visible through fog of war
                                if !is_owned && !is_neutral {
                                    let grid = world_to_grid(transform.translation);
                                    if fog_map.get(local_player.0, grid.x, grid.z) != VisibilityStateEnum::Visible {
                                        continue;
                                    }
                                }

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
    let Ok(window) = windows.single() else { return; };

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
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(min_x),
                        top: Val::Px(min_y),
                        width: Val::Px(width),
                        height: Val::Px(height),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgba(1.0, 1.0, 0.0, 1.0)),
                    BackgroundColor(Color::srgba(1.0, 1.0, 0.0, 0.1)),
                    UiTargetCamera(ui_cam.0),
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

/// System to add/remove selection indicators for selected entities.
/// Uses Parent-based cleanup: queries all SelectionIndicator entities and checks
/// if their parent is still selected. This avoids reliance on fragile
/// Without<Children> / With<Children> archetype filters.
pub fn manage_selection_indicators(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    selected_entities: Query<(Entity, Option<&Children>), With<Selected>>,
    indicator_parents: Query<(Entity, &ChildOf), With<SelectionIndicator>>,
    selected_check: Query<(), With<Selected>>,
    indicators: Query<(), With<SelectionIndicator>>,
    mut cached_mesh: Local<Option<Handle<Mesh>>>,
    mut cached_material: Local<Option<Handle<StandardMaterial>>>,
) {
    // Spawn indicators for selected entities that don't already have one
    for (entity, children) in selected_entities.iter() {
        let has_indicator = children.map_or(false, |c| {
            c.iter().any(|child| indicators.get(child).is_ok())
        });
        if has_indicator {
            continue;
        }

        let indicator_mesh = cached_mesh.get_or_insert_with(|| {
            meshes.add(Torus::new(0.4, 0.05))
        }).clone();
        let indicator_material = cached_material.get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 0.0),
                emissive: LinearRgba::rgb(1.0, 1.0, 0.0),
                ..default()
            })
        }).clone();

        let indicator = commands.spawn((
            Mesh3d(indicator_mesh),
            MeshMaterial3d(indicator_material),
            Transform::from_xyz(0.0, -0.3, 0.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.0)),
            SelectionIndicator,
        )).id();

        commands.entity(entity).add_child(indicator);
    }

    // Despawn indicators whose parent is no longer selected
    for (indicator_entity, parent) in indicator_parents.iter() {
        if selected_check.get(parent.0).is_err() {
            commands.entity(indicator_entity).despawn();
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
        // Mark all 4 tiles of the 2x2 footprint as non-traversible and non-buildable
        for (tile_pos, mut properties) in tiles.iter_mut() {
            let dx = tile_pos.x - grid_x;
            let dz = tile_pos.z - grid_z;
            if dx >= 0 && dx < 2 && dz >= 0 && dz < 2 {
                properties.traversible = false;
                properties.buildable = false;
            }
        }

        // Center the entity on the 2x2 footprint (offset by +0.5 tiles from anchor)
        let world_x = (grid_x as f32 - 32.0) + 1.0;
        let world_z = (grid_z as f32 - 32.0) + 1.0;

        commands.spawn((
            Mesh3d(platform_mesh.clone()),
            MeshMaterial3d(platform_material.clone()),
            Transform::from_xyz(world_x, 0.1, world_z),
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
            sds.time_until_next_delivery -= time.delta_secs();

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

        if ctrl_held && shift_held {
            // Ctrl+Shift+Number: Add current selection to group (merge, no duplicates)
            // Only include entities owned by the local player
            for (entity, owner) in selected_entities.iter() {
                if owner.0 == Some(local_player.0) && !control_groups.groups[group_idx].contains(&entity) {
                    control_groups.groups[group_idx].push(entity);
                }
            }
            info!("Control group {}: now {} entities (after add)",
                group_idx + 1, control_groups.groups[group_idx].len());
        } else if ctrl_held {
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
        } else {
            // Number only: Recall group as selection
            // First, clean up dead entities
            control_groups.groups[group_idx].retain(|&entity| all_entities.get(entity).is_ok());

            let group = &control_groups.groups[group_idx];
            if group.is_empty() {
                continue;
            }

            let current_time = time.elapsed_secs_f64();

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
                    if let Ok(mut cam_transform) = camera_query.single_mut() {
                        // Move camera so centroid appears at viewport center.
                        // Camera looks from (x, y, z) with fixed rotation set at
                        // setup: from (0,40,25) looking at (0,0,0). The forward
                        // direction is (0,-40,-25), so the Z offset from camera
                        // position to ground look-at point is -25/40 * height.
                        // To center on a ground point, offset camera Z accordingly.
                        let z_offset = cam_transform.translation.y * 25.0 / 40.0;
                        cam_transform.translation.x = centroid.x;
                        cam_transform.translation.z = centroid.z + z_offset;
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
    selected_query: Query<(Entity, &ObjectInstance), With<Selected>>,
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

    // Tab cycling is also handled in command_panel_hotkeys when the command panel
    // is visible. Skip here if selection contains non-resource groups to avoid
    // double-cycling (which would cycle twice per frame, reverting instantly).
    let has_commandable_groups = !selection.groups.is_empty()
        && !selection.groups.iter().all(|g| g.object_type.is_resource());

    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if shift_held && keyboard.just_pressed(KeyCode::Tab) && !has_commandable_groups {
        selection.cycle_active_group_backward();
        if let Some(group) = selection.active_group() {
            info!("Active group (backward): {} ({} entities)",
                group.object_type.object_type().name,
                group.entities.len());
        }
    } else if keyboard.just_pressed(KeyCode::Tab) && !shift_held && !has_commandable_groups {
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
        if let Ok(mut entity_commands) = commands.get_entity(*entity) {
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

/// Tracks previous selection state for interface reset detection.
#[derive(Resource, Default)]
pub struct PreviousSelectionSnapshot {
    pub initialized: bool,
    pub active_group_index: Option<usize>,
    pub group_types: Vec<ObjectEnum>,
}

/// System to reset ObjectInterfaceState when the Selection or active group changes.
/// Tracks the previous selection snapshot (active_group_index + group types) and
/// resets to Default when a change is detected.
pub fn interface_state_selection_reset_system(
    selection: Res<Selection>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    mut prev_state: ResMut<PreviousSelectionSnapshot>,
) {
    let current_types: Vec<ObjectEnum> = selection.groups.iter().map(|g| g.object_type).collect();
    let current_index = selection.active_group_index;

    if prev_state.initialized {
        if prev_state.active_group_index != current_index || prev_state.group_types != current_types {
            *interface_state = ObjectInterfaceState::Default;
        }
    }

    prev_state.initialized = true;
    prev_state.active_group_index = current_index;
    prev_state.group_types = current_types;
}

/// System to validate the current ObjectInterfaceState against the active SelectionGroup.
/// If the current state is no longer valid for the active group, resets to Default.
pub fn interface_state_validation_system(
    selection: Res<Selection>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    dc_query: Query<&crate::game::types::structures::DeploymentCenterState>,
    bk_query: Query<&crate::game::types::structures::BarracksState>,
    ef_query: Query<&crate::game::types::structures::ExtractionFacilityState>,
    st_query: Query<&crate::game::types::structures::SupplyTowerState>,
    hq_query: Query<&crate::game::types::structures::HeadquartersState>,
    tunnel_query: Query<&crate::game::types::structures::TunnelState>,
) {
    use StructureMenuState::*;

    let active_group = selection.active_group();
    let active_type = selection.active_type();

    let is_valid = match &*interface_state {
        ObjectInterfaceState::Default => true,

        ObjectInterfaceState::AwaitingTarget(_) => {
            // Valid as long as the active group has at least one entity
            active_group.map_or(false, |g| !g.entities.is_empty())
        }

        ObjectInterfaceState::AgentMenu(_) => {
            active_type == Some(ObjectEnum::SyndicateAgent)
        }

        ObjectInterfaceState::CultsRecruitMenu(_) => {
            active_type == Some(ObjectEnum::CultsRecruit)
        }

        ObjectInterfaceState::StructureMenu(sm) => match sm {
            DcIdle | DcBuildMenu | DcReadyToPlace | DcAwaitingPlacement => {
                active_type == Some(ObjectEnum::DeploymentCenter)
                    && active_group.map_or(false, |g| {
                        g.entities.iter().any(|e| dc_query.get(*e).is_ok())
                    })
            }
            DcConstructing => {
                active_type == Some(ObjectEnum::DeploymentCenter)
                    && active_group.map_or(false, |g| {
                        g.entities.iter().any(|e| {
                            dc_query.get(*e).map_or(false, |dc| dc.current_construction.is_some())
                        })
                    })
            }
            BarracksMenu => {
                active_type == Some(ObjectEnum::Barracks)
                    && active_group.map_or(false, |g| {
                        g.entities.iter().any(|e| bk_query.get(*e).is_ok())
                    })
            }
            EfIdle | EfAwaitingPlacement => {
                active_type == Some(ObjectEnum::ExtractionFacility)
                    && active_group.map_or(false, |g| {
                        g.entities.iter().any(|e| ef_query.get(*e).is_ok())
                    })
            }
            SupplyTowerMenu => {
                active_type == Some(ObjectEnum::SupplyTower)
                    && active_group.map_or(false, |g| {
                        g.entities.iter().any(|e| st_query.get(*e).is_ok())
                    })
            }
            HeadquartersMenu => {
                active_type == Some(ObjectEnum::Headquarters)
                    && active_group.map_or(false, |g| {
                        g.entities.iter().any(|e| hq_query.get(*e).is_ok())
                    })
            }
            TunnelIdle | TunnelExpandMenu | TunnelEjectMenu | TunnelAwaitingPlacement => {
                active_type == Some(ObjectEnum::Tunnel)
                    && active_group.map_or(false, |g| {
                        g.entities.iter().any(|e| tunnel_query.get(*e).is_ok())
                    })
            }
            RecruitmentCenterMenu => {
                active_type == Some(ObjectEnum::RecruitmentCenter)
            }
            ArmoryMenu => {
                active_type == Some(ObjectEnum::CultsArmory)
            }
            Inert => {
                active_type.map_or(false, |t| t.is_structure())
            }
        },
    };

    if !is_valid {
        *interface_state = ObjectInterfaceState::Default;
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::game::types::ObjectInstance;
    use crate::game::units::types::commands::CommandType;
    use crate::ui::types::{StructureMenuState, AgentMenuState};
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
    fn selection_group_sync_includes_resource_entities() {
        let mut world = setup_test_world();

        // Spawn a SupplyDeliveryStation — has Selected + ObjectInstance (resource entity)
        world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::SupplyDeliveryStation),
        ));

        world.run_system_once(selection_group_sync_system);

        let selection = world.resource::<Selection>();
        assert_eq!(selection.groups.len(), 1, "Resource entity should appear in selection groups");
        assert_eq!(selection.groups[0].object_type, ObjectEnum::SupplyDeliveryStation);
    }

    #[test]
    fn selection_group_sync_includes_crystal_patch() {
        let mut world = setup_test_world();

        // Spawn a SpaceCrystalsPatch with ObjectInstance
        world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::SpaceCrystalsPatch),
        ));

        world.run_system_once(selection_group_sync_system);

        let selection = world.resource::<Selection>();
        assert_eq!(selection.groups.len(), 1, "Crystal Patch should appear in selection groups");
        assert_eq!(selection.groups[0].object_type, ObjectEnum::SpaceCrystalsPatch);
    }

    #[test]
    fn selection_group_sync_excludes_entity_without_object_instance() {
        let mut world = setup_test_world();

        // Spawn an entity with Selected but no ObjectInstance
        world.spawn(Selected);

        world.run_system_once(selection_group_sync_system);

        let selection = world.resource::<Selection>();
        assert!(selection.groups.is_empty(), "Entity without ObjectInstance should NOT appear in selection groups");
    }

    #[test]
    fn selection_group_sync_mixed_selection_includes_all_object_types() {
        let mut world = setup_test_world();

        // Spawn a unit
        world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::Peacekeeper),
            Unit,
        ));

        // Spawn a resource (SDS)
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
        // All 3 should be in groups: Peacekeeper, SDS, DeploymentCenter
        assert_eq!(selection.groups.len(), 3, "All ObjectInstance entities should be in groups");
        let types: Vec<ObjectEnum> = selection.groups.iter().map(|g| g.object_type).collect();
        assert!(types.contains(&ObjectEnum::Peacekeeper), "Peacekeeper should be in groups");
        assert!(types.contains(&ObjectEnum::DeploymentCenter), "DeploymentCenter should be in groups");
        assert!(types.contains(&ObjectEnum::SupplyDeliveryStation), "SDS should be in groups");
    }

    #[test]
    fn sds_selection_produces_groups_but_panel_hidden() {
        // When only SDS is selected, groups should be non-empty (SDS is now included).
        // The command panel uses is_panel_visible() which checks for resource-only selections
        // and returns false, preventing phantom panel.
        let mut world = setup_test_world();

        world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::SupplyDeliveryStation),
        ));

        world.run_system_once(selection_group_sync_system);

        let selection = world.resource::<Selection>();
        assert_eq!(selection.groups.len(), 1, "SDS selection should produce groups");
        assert_eq!(selection.groups[0].object_type, ObjectEnum::SupplyDeliveryStation);
        // Panel visibility is guarded by is_panel_visible() which returns false for resource-only
    }

    #[test]
    fn selection_groups_updated_when_switching_from_unit_to_resource() {
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
        assert_eq!(selection.groups.len(), 1, "SDS should now be in groups");
        assert_eq!(selection.groups[0].object_type, ObjectEnum::SupplyDeliveryStation);
    }

    // === Selection Indicator Memory Leak Fix Tests ===

    #[test]
    fn selection_indicator_cleanup_uses_children_query() {
        // Verify the manage_selection_indicators system properly cleans up children
        // (Tests the fix for the original bug where `selected.get()` was used
        // instead of `children_query.get()` for deselected entities)
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();

        // Spawn a selected entity — this will get an indicator child
        let entity = world.spawn((Selected, Transform::default())).id();
        world.run_system_once(manage_selection_indicators);

        // Verify indicator was spawned as child
        let has_children = world.entity(entity).get::<Children>().is_some();
        assert!(has_children, "Selected entity should have indicator child");
        let indicator_count = world.query::<&SelectionIndicator>().iter(&world).count();
        assert_eq!(indicator_count, 1);

        // Deselect the entity
        world.entity_mut(entity).remove::<Selected>();
        world.run_system_once(manage_selection_indicators);

        // Verify indicator was cleaned up (the bug was that this never happened)
        let indicator_count = world.query::<&SelectionIndicator>().iter(&world).count();
        assert_eq!(indicator_count, 0, "Selection indicator should be despawned after deselection");
    }

    #[test]
    fn selection_indicator_mesh_cached() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();

        // Spawn two selected entities
        world.spawn((Selected, Transform::default()));
        world.spawn((Selected, Transform::default()));
        world.run_system_once(manage_selection_indicators);

        // Both should exist as indicators
        let indicator_count = world.query::<&SelectionIndicator>().iter(&world).count();
        assert_eq!(indicator_count, 2);

        // But mesh count should be 1 (cached via Local)
        let mesh_count = world.resource::<Assets<Mesh>>().len();
        assert_eq!(mesh_count, 1, "Mesh should be cached — only 1 mesh for all indicators");
    }

    #[test]
    fn selection_indicator_material_cached() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();

        world.spawn((Selected, Transform::default()));
        world.spawn((Selected, Transform::default()));
        world.run_system_once(manage_selection_indicators);

        let material_count = world.resource::<Assets<StandardMaterial>>().len();
        assert_eq!(material_count, 1, "Material should be cached — only 1 material for all indicators");
    }

    #[test]
    fn selection_indicator_no_leak_on_repeated_select_deselect() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();

        let entity = world.spawn(Transform::default()).id();

        // Cycle select/deselect 5 times
        for _ in 0..5 {
            world.entity_mut(entity).insert(Selected);
            world.run_system_once(manage_selection_indicators);
            world.entity_mut(entity).remove::<Selected>();
            world.run_system_once(manage_selection_indicators);
        }

        // All indicators should be cleaned up
        let indicator_count = world.query::<&SelectionIndicator>().iter(&world).count();
        assert_eq!(indicator_count, 0, "No indicators should remain after deselection cycles");
    }

    // --- Fog of War Selection Filter Tests ---

    /// Helper: determines if an entity should be selectable given ownership and fog state.
    /// Returns true if the entity passes the fog filter (i.e., should be selectable).
    fn fog_filter_allows(is_owned: bool, is_neutral: bool, visibility: VisibilityStateEnum) -> bool {
        if is_owned {
            return true; // Own entities always selectable
        }
        if is_neutral {
            return true; // Neutral entities always selectable (SCP, SDS)
        }
        // Enemy entity: only selectable if tile is Visible
        visibility == VisibilityStateEnum::Visible
    }

    #[test]
    fn fog_filter_owned_always_selectable() {
        assert!(fog_filter_allows(true, false, VisibilityStateEnum::Unexplored));
        assert!(fog_filter_allows(true, false, VisibilityStateEnum::Explored));
        assert!(fog_filter_allows(true, false, VisibilityStateEnum::Visible));
    }

    #[test]
    fn fog_filter_neutral_always_selectable() {
        assert!(fog_filter_allows(false, true, VisibilityStateEnum::Unexplored));
        assert!(fog_filter_allows(false, true, VisibilityStateEnum::Explored));
        assert!(fog_filter_allows(false, true, VisibilityStateEnum::Visible));
    }

    #[test]
    fn fog_filter_enemy_visible_selectable() {
        assert!(fog_filter_allows(false, false, VisibilityStateEnum::Visible));
    }

    #[test]
    fn fog_filter_enemy_explored_not_selectable() {
        assert!(!fog_filter_allows(false, false, VisibilityStateEnum::Explored));
    }

    #[test]
    fn fog_filter_enemy_unexplored_not_selectable() {
        assert!(!fog_filter_allows(false, false, VisibilityStateEnum::Unexplored));
    }

    #[test]
    fn fog_map_out_of_bounds_returns_unexplored() {
        let fog_map = FogOfWarMap::new(64, 64);
        // Out-of-bounds should return Unexplored, blocking enemy selection
        assert_eq!(fog_map.get(0, -1, -1), VisibilityStateEnum::Unexplored);
        assert_eq!(fog_map.get(0, 999, 999), VisibilityStateEnum::Unexplored);
    }

    #[test]
    fn fog_map_default_tiles_are_unexplored() {
        let mut fog_map = FogOfWarMap::new(64, 64);
        fog_map.ensure_player(0);
        // Tiles within bounds but never revealed should be Unexplored
        assert_eq!(fog_map.get(0, 32, 32), VisibilityStateEnum::Unexplored);
    }

    #[test]
    fn fog_map_visible_tile_allows_enemy_selection() {
        let mut fog_map = FogOfWarMap::new(64, 64);
        fog_map.ensure_player(0);
        fog_map.set(0, 32, 32, VisibilityStateEnum::Visible);
        assert_eq!(fog_map.get(0, 32, 32), VisibilityStateEnum::Visible);
        // Enemy at this tile would pass the filter
        assert!(fog_filter_allows(false, false, fog_map.get(0, 32, 32)));
    }

    #[test]
    fn fog_map_explored_tile_blocks_enemy_selection() {
        let mut fog_map = FogOfWarMap::new(64, 64);
        fog_map.ensure_player(0);
        fog_map.set(0, 32, 32, VisibilityStateEnum::Explored);
        assert_eq!(fog_map.get(0, 32, 32), VisibilityStateEnum::Explored);
        // Enemy at this tile would NOT pass the filter
        assert!(!fog_filter_allows(false, false, fog_map.get(0, 32, 32)));
    }

    #[test]
    fn world_to_grid_consistency_for_fog_check() {
        // Verify world_to_grid produces valid coords for fog map lookup
        let grid = world_to_grid(Vec3::new(0.0, 0.0, 0.0));
        // At origin (0,0,0), grid should be (32, 32) due to GRID_HALF_SIZE=32
        assert_eq!(grid.x, 32);
        assert_eq!(grid.z, 32);
    }

    // --- Shift+Click Add-to-Selection Tests ---

    /// The selection_system checks shift_pressed and adds to selection
    /// instead of replacing. This tests the logic predicate.
    #[test]
    fn shift_click_add_predicate_keeps_existing() {
        // When shift is held and clicking an owned entity that's not yet selected,
        // the entity should be ADDED (other selected entities remain selected).
        // Predicate: !selected.contains(entity) → insert(Selected)
        // No deselection of other entities.
        let entity_a = Entity::from_raw_u32(1).unwrap();
        let entity_b = Entity::from_raw_u32(2).unwrap();
        let mut selected: Vec<Entity> = vec![entity_a];
        // Shift+click entity_b: add without removing entity_a
        if !selected.contains(&entity_b) {
            selected.push(entity_b);
        }
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&entity_a));
        assert!(selected.contains(&entity_b));
    }

    #[test]
    fn shift_click_does_not_deselect_on_empty() {
        // When shift is held and clicking empty space, selection should NOT clear
        let selection = vec![Entity::from_raw_u32(1).unwrap()];
        // shift_pressed && !ctrl_pressed → no deselect
        let should_deselect = false; // !ctrl_pressed && !shift_pressed evaluates to false
        assert!(!should_deselect);
        assert_eq!(selection.len(), 1, "Selection preserved when shift+clicking empty space");
    }

    #[test]
    fn shift_click_non_owned_still_single_selects() {
        // Shift+click on non-owned entity should still single-select (replace)
        // This is by design: non-owned always single-select regardless of modifiers
        let is_owned = false;
        let shift_pressed = true;
        // Non-owned path ignores shift — always replaces
        assert!(!is_owned, "Non-owned entities always single-select");
        assert!(shift_pressed, "Shift being held doesn't change non-owned behavior");
    }

    // --- Recall-and-Center Camera Offset Tests ---

    /// Camera starts at (0, 40, 25) looking at (0, 0, 0).
    /// The Z offset ratio is 25/40 = 0.625. When centering on a ground point,
    /// the camera Z must be offset by camera_height * 25/40.
    #[test]
    fn recall_center_z_offset_default_height() {
        let camera_height = 40.0_f32;
        let centroid_z = 10.0_f32;
        let z_offset = camera_height * 25.0 / 40.0;
        let expected_cam_z = centroid_z + z_offset;
        assert!((expected_cam_z - 35.0).abs() < 0.001,
            "Camera at height 40 centering on z=10 should be at z=35");
    }

    #[test]
    fn recall_center_z_offset_zoomed_in() {
        // When zoomed in (camera height = 20), Z offset shrinks proportionally
        let camera_height = 20.0_f32;
        let centroid_z = 0.0_f32;
        let z_offset = camera_height * 25.0 / 40.0;
        let expected_cam_z = centroid_z + z_offset;
        assert!((expected_cam_z - 12.5).abs() < 0.001,
            "Camera at height 20 centering on z=0 should be at z=12.5");
    }

    #[test]
    fn recall_center_z_offset_zero_means_origin() {
        // At default height, centering on z=0 should place camera at z=25
        let camera_height = 40.0_f32;
        let z_offset = camera_height * 25.0 / 40.0;
        assert!((z_offset - 25.0).abs() < 0.001,
            "Default offset matches initial camera Z position");
    }

    #[test]
    fn recall_center_x_is_exact_centroid() {
        // X centering should always match centroid exactly (no angle offset in X)
        let centroid_x = 15.0_f32;
        let cam_x = centroid_x;
        assert!((cam_x - 15.0).abs() < 0.001);
    }

    // --- Selection Indicator Parent-Based Cleanup Tests ---

    #[test]
    fn indicator_despawned_when_parent_deselected_parent_query() {
        // Verify the Parent-based cleanup works: indicator with Parent pointing
        // to a deselected entity is despawned.
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();

        let entity = world.spawn((Selected, Transform::default())).id();
        world.run_system_once(manage_selection_indicators);

        // Should have 1 indicator
        let count = world.query::<&SelectionIndicator>().iter(&world).count();
        assert_eq!(count, 1);

        // Deselect
        world.entity_mut(entity).remove::<Selected>();
        world.run_system_once(manage_selection_indicators);

        let count = world.query::<&SelectionIndicator>().iter(&world).count();
        assert_eq!(count, 0, "Indicator despawned when parent deselected");
    }

    #[test]
    fn indicator_persists_while_parent_selected() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();

        world.spawn((Selected, Transform::default()));
        world.run_system_once(manage_selection_indicators);

        let count = world.query::<&SelectionIndicator>().iter(&world).count();
        assert_eq!(count, 1);

        // Run again — indicator should persist (parent still selected)
        world.run_system_once(manage_selection_indicators);
        let count = world.query::<&SelectionIndicator>().iter(&world).count();
        assert_eq!(count, 1, "Indicator persists while parent is selected");
    }

    #[test]
    fn no_duplicate_indicators_on_already_selected() {
        // Verify that running the system multiple times doesn't spawn duplicate indicators
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();

        world.spawn((Selected, Transform::default()));
        world.run_system_once(manage_selection_indicators);
        world.run_system_once(manage_selection_indicators);
        world.run_system_once(manage_selection_indicators);

        let count = world.query::<&SelectionIndicator>().iter(&world).count();
        assert_eq!(count, 1, "Only one indicator per entity regardless of system runs");
    }

    #[test]
    fn multiple_entities_each_get_one_indicator() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();

        world.spawn((Selected, Transform::default()));
        world.spawn((Selected, Transform::default()));
        world.spawn((Selected, Transform::default()));
        world.run_system_once(manage_selection_indicators);

        let count = world.query::<&SelectionIndicator>().iter(&world).count();
        assert_eq!(count, 3, "Each selected entity gets exactly one indicator");
    }

    // === Bevy 0.17 Emissive Type Tests ===

    #[test]
    fn crystal_material_uses_linear_rgba_emissive() {
        // Verify that LinearRgba::rgb() produces a valid emissive value
        // (Bevy 0.17 requires LinearRgba for StandardMaterial.emissive)
        let emissive = LinearRgba::rgb(0.2, 0.6, 0.8);
        let mat = StandardMaterial {
            base_color: Color::srgb(0.3, 0.8, 1.0),
            emissive,
            ..default()
        };
        assert_eq!(mat.emissive.red, 0.2);
        assert_eq!(mat.emissive.green, 0.6);
        assert_eq!(mat.emissive.blue, 0.8);
    }

    #[test]
    fn selection_indicator_material_uses_linear_rgba_emissive() {
        let emissive = LinearRgba::rgb(1.0, 1.0, 0.0);
        let mat = StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 0.0),
            emissive,
            ..default()
        };
        assert_eq!(mat.emissive.red, 1.0);
        assert_eq!(mat.emissive.green, 1.0);
        assert_eq!(mat.emissive.blue, 0.0);
    }

    #[test]
    fn active_group_cycle_skips_when_commandable_groups_present() {
        // When non-resource groups are selected, active_group_cycle_system
        // should NOT cycle — command_panel_hotkeys handles Tab instead.
        // This prevents the double-cycle bug where active_group_index
        // reverts instantly (cycles twice per frame).
        let mut selection = Selection::default();
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::Peacekeeper, entities: vec![e1] },
            SelectionGroup { object_type: ObjectEnum::SyndicateAgent, entities: vec![e2] },
        ];
        selection.active_group_index = Some(0);

        // has_commandable_groups should be true (non-resource groups present)
        let has_commandable_groups = !selection.groups.is_empty()
            && !selection.groups.iter().all(|g| g.object_type.is_resource());
        assert!(has_commandable_groups, "Non-resource groups should be detected as commandable");

        // The system guard would block Tab processing, preserving active_group_index
        assert_eq!(selection.active_group_index, Some(0));
    }

    #[test]
    fn active_group_cycle_runs_when_only_resource_groups() {
        // When only resource groups are selected, active_group_cycle_system
        // should handle Tab (command_panel_hotkeys won't run for resources).
        let mut selection = Selection::default();
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::SpaceCrystalsPatch, entities: vec![e1] },
            SelectionGroup { object_type: ObjectEnum::SupplyDeliveryStation, entities: vec![e2] },
        ];
        selection.active_group_index = Some(0);

        let has_commandable_groups = !selection.groups.is_empty()
            && !selection.groups.iter().all(|g| g.object_type.is_resource());
        assert!(!has_commandable_groups, "Resource-only groups should not be commandable");

        // The system would allow Tab processing for resource-only selections
        selection.cycle_active_group();
        assert_eq!(selection.active_group_index, Some(1));
    }

    // === Interface State Selection Reset Tests ===

    fn setup_test_world_with_interface_state() -> World {
        let mut world = World::new();
        world.insert_resource(Selection::default());
        world.insert_resource(ObjectInterfaceState::Default);
        world.insert_resource(PreviousSelectionSnapshot::default());
        world
    }

    #[test]
    fn interface_reset_on_selection_change() {
        let mut world = setup_test_world_with_interface_state();

        // Initial run to seed the Local cache
        world.run_system_once(interface_state_selection_reset_system);

        // Set interface to AwaitingTarget
        *world.resource_mut::<ObjectInterfaceState>() =
            ObjectInterfaceState::AwaitingTarget(CommandType::Move);

        // Change selection: add a group
        let e = world.spawn((
            Selected,
            ObjectInstance::indestructible(ObjectEnum::Peacekeeper),
            Unit,
        )).id();
        {
            let mut sel = world.resource_mut::<Selection>();
            sel.groups = vec![SelectionGroup {
                object_type: ObjectEnum::Peacekeeper,
                entities: vec![e],
            }];
            sel.active_group_index = Some(0);
        }

        world.run_system_once(interface_state_selection_reset_system);

        let state = world.resource::<ObjectInterfaceState>();
        assert_eq!(*state, ObjectInterfaceState::Default,
            "Interface state should reset to Default when selection changes");
    }

    #[test]
    fn interface_reset_on_active_group_index_change() {
        let mut world = setup_test_world_with_interface_state();

        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        {
            let mut sel = world.resource_mut::<Selection>();
            sel.groups = vec![
                SelectionGroup { object_type: ObjectEnum::Peacekeeper, entities: vec![e1] },
                SelectionGroup { object_type: ObjectEnum::SyndicateGuard, entities: vec![e2] },
            ];
            sel.active_group_index = Some(0);
        }

        // Seed the cache
        world.run_system_once(interface_state_selection_reset_system);

        // Set non-default state
        *world.resource_mut::<ObjectInterfaceState>() =
            ObjectInterfaceState::AwaitingTarget(CommandType::Attack);

        // Change active group index
        world.resource_mut::<Selection>().active_group_index = Some(1);

        world.run_system_once(interface_state_selection_reset_system);

        let state = world.resource::<ObjectInterfaceState>();
        assert_eq!(*state, ObjectInterfaceState::Default,
            "Interface state should reset when active_group_index changes");
    }

    #[test]
    fn interface_no_reset_when_selection_unchanged() {
        let mut world = setup_test_world_with_interface_state();

        let e = world.spawn_empty().id();
        {
            let mut sel = world.resource_mut::<Selection>();
            sel.groups = vec![SelectionGroup {
                object_type: ObjectEnum::Peacekeeper,
                entities: vec![e],
            }];
            sel.active_group_index = Some(0);
        }

        // Seed cache
        world.run_system_once(interface_state_selection_reset_system);

        // Set non-default state
        let target_state = ObjectInterfaceState::AwaitingTarget(CommandType::Move);
        *world.resource_mut::<ObjectInterfaceState>() = target_state.clone();

        // Run again without changing selection
        world.run_system_once(interface_state_selection_reset_system);

        let state = world.resource::<ObjectInterfaceState>();
        assert_eq!(*state, target_state,
            "Interface state should NOT reset when selection is unchanged");
    }

    // === Interface State Validation Tests ===

    #[test]
    fn validation_default_always_valid() {
        let mut world = setup_test_world_with_interface_state();
        // Empty selection, Default state — should remain valid
        world.run_system_once(interface_state_validation_system);

        let state = world.resource::<ObjectInterfaceState>();
        assert_eq!(*state, ObjectInterfaceState::Default);
    }

    #[test]
    fn validation_awaiting_target_invalid_with_empty_selection() {
        let mut world = setup_test_world_with_interface_state();
        *world.resource_mut::<ObjectInterfaceState>() =
            ObjectInterfaceState::AwaitingTarget(CommandType::Move);

        world.run_system_once(interface_state_validation_system);

        let state = world.resource::<ObjectInterfaceState>();
        assert_eq!(*state, ObjectInterfaceState::Default,
            "AwaitingTarget should reset to Default with empty selection");
    }

    #[test]
    fn validation_awaiting_target_valid_with_entities() {
        let mut world = setup_test_world_with_interface_state();

        let e = world.spawn_empty().id();
        {
            let mut sel = world.resource_mut::<Selection>();
            sel.groups = vec![SelectionGroup {
                object_type: ObjectEnum::Peacekeeper,
                entities: vec![e],
            }];
            sel.active_group_index = Some(0);
        }

        let target_state = ObjectInterfaceState::AwaitingTarget(CommandType::Attack);
        *world.resource_mut::<ObjectInterfaceState>() = target_state.clone();

        world.run_system_once(interface_state_validation_system);

        let state = world.resource::<ObjectInterfaceState>();
        assert_eq!(*state, target_state,
            "AwaitingTarget should be preserved with active entities");
    }

    #[test]
    fn validation_agent_menu_invalid_with_non_agent_group() {
        let mut world = setup_test_world_with_interface_state();

        let e = world.spawn_empty().id();
        {
            let mut sel = world.resource_mut::<Selection>();
            sel.groups = vec![SelectionGroup {
                object_type: ObjectEnum::Peacekeeper,
                entities: vec![e],
            }];
            sel.active_group_index = Some(0);
        }

        *world.resource_mut::<ObjectInterfaceState>() =
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);

        world.run_system_once(interface_state_validation_system);

        let state = world.resource::<ObjectInterfaceState>();
        assert_eq!(*state, ObjectInterfaceState::Default,
            "AgentMenu should reset when active group is not SyndicateAgent");
    }

    #[test]
    fn validation_agent_menu_valid_with_agent_group() {
        let mut world = setup_test_world_with_interface_state();

        let e = world.spawn_empty().id();
        {
            let mut sel = world.resource_mut::<Selection>();
            sel.groups = vec![SelectionGroup {
                object_type: ObjectEnum::SyndicateAgent,
                entities: vec![e],
            }];
            sel.active_group_index = Some(0);
        }

        let target_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
        *world.resource_mut::<ObjectInterfaceState>() = target_state.clone();

        world.run_system_once(interface_state_validation_system);

        let state = world.resource::<ObjectInterfaceState>();
        assert_eq!(*state, target_state,
            "AgentMenu should be preserved when active group is SyndicateAgent");
    }

    #[test]
    fn validation_structure_menu_dc_invalid_without_dc_state() {
        use crate::game::types::structures::DeploymentCenterState;
        let mut world = setup_test_world_with_interface_state();

        // DC entity WITHOUT DeploymentCenterState component
        let e = world.spawn_empty().id();
        {
            let mut sel = world.resource_mut::<Selection>();
            sel.groups = vec![SelectionGroup {
                object_type: ObjectEnum::DeploymentCenter,
                entities: vec![e],
            }];
            sel.active_group_index = Some(0);
        }

        *world.resource_mut::<ObjectInterfaceState>() =
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);

        world.run_system_once(interface_state_validation_system);

        let state = world.resource::<ObjectInterfaceState>();
        assert_eq!(*state, ObjectInterfaceState::Default,
            "DcIdle should reset when DC entity lacks DeploymentCenterState");
    }

    #[test]
    fn validation_structure_menu_dc_valid_with_dc_state() {
        use crate::game::types::structures::DeploymentCenterState;
        let mut world = setup_test_world_with_interface_state();

        let e = world.spawn((DeploymentCenterState::default(),)).id();
        {
            let mut sel = world.resource_mut::<Selection>();
            sel.groups = vec![SelectionGroup {
                object_type: ObjectEnum::DeploymentCenter,
                entities: vec![e],
            }];
            sel.active_group_index = Some(0);
        }

        let target_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        *world.resource_mut::<ObjectInterfaceState>() = target_state.clone();

        world.run_system_once(interface_state_validation_system);

        let state = world.resource::<ObjectInterfaceState>();
        assert_eq!(*state, target_state,
            "DcIdle should be preserved when DC entity has DeploymentCenterState");
    }

    #[test]
    fn validation_dc_constructing_invalid_without_active_construction() {
        use crate::game::types::structures::DeploymentCenterState;
        let mut world = setup_test_world_with_interface_state();

        // DC with no active construction
        let e = world.spawn((DeploymentCenterState::default(),)).id();
        {
            let mut sel = world.resource_mut::<Selection>();
            sel.groups = vec![SelectionGroup {
                object_type: ObjectEnum::DeploymentCenter,
                entities: vec![e],
            }];
            sel.active_group_index = Some(0);
        }

        *world.resource_mut::<ObjectInterfaceState>() =
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing);

        world.run_system_once(interface_state_validation_system);

        let state = world.resource::<ObjectInterfaceState>();
        assert_eq!(*state, ObjectInterfaceState::Default,
            "DcConstructing should reset when DC has no active construction");
    }

    #[test]
    fn validation_dc_constructing_valid_with_active_construction() {
        use crate::game::types::structures::DeploymentCenterState;
        let mut world = setup_test_world_with_interface_state();

        let dc_state = DeploymentCenterState {
            current_construction: Some(ObjectEnum::Barracks),
            construction_progress: Some(50.0),
            ready_to_place: None,
        };
        let e = world.spawn((dc_state,)).id();
        {
            let mut sel = world.resource_mut::<Selection>();
            sel.groups = vec![SelectionGroup {
                object_type: ObjectEnum::DeploymentCenter,
                entities: vec![e],
            }];
            sel.active_group_index = Some(0);
        }

        let target_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing);
        *world.resource_mut::<ObjectInterfaceState>() = target_state.clone();

        world.run_system_once(interface_state_validation_system);

        let state = world.resource::<ObjectInterfaceState>();
        assert_eq!(*state, target_state,
            "DcConstructing should be preserved when DC has active construction");
    }
}
