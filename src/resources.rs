use bevy::prelude::*;
use crate::map::{GridPosition, Tile, TileProperties};

/// Component for Space Crystal Patch resource nodes
#[derive(Component)]
pub struct SpaceCrystalPatch {
    pub amount: u32,
    pub initial_amount: u32,
}

/// Component for Supply Delivery Station resource nodes
#[derive(Component)]
pub struct SupplyDeliveryStation {
    pub delivery_size: u32,
    pub delivery_interval: f32,
    pub current_supplies: u32,
    pub time_until_next_delivery: f32,
}

/// Component marking an entity as selectable
#[derive(Component)]
pub struct Selectable;

/// Component marking an entity as currently selected
#[derive(Component)]
pub struct Selected;

/// Component for selection indicator visual
#[derive(Component)]
struct SelectionIndicator;

/// Component marking the drag-box UI element
#[derive(Component)]
struct DragBoxUI;

/// Resource tracking selection state for drag-box
#[derive(Resource, Default)]
struct SelectionState {
    drag_start: Option<Vec2>,
    is_dragging: bool,
}

/// Plugin for resource-related systems
pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectionState::default())
            .add_systems(Startup, (
                spawn_space_crystal_patches.after(crate::map::spawn_grid),
                spawn_supply_delivery_stations.after(crate::map::spawn_grid),
            ))
            .add_systems(Update, (
                selection_system,
                drag_box_system,
                draw_drag_box_ui,
                manage_selection_indicators,
                log_selection_changes,
                sds_delivery_timer,
            ));
    }
}

/// Spawn Space Crystal Patches on the map
fn spawn_space_crystal_patches(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tiles: Query<(&GridPosition, &mut TileProperties), With<Tile>>,
) {
    // Define SCP locations and amounts (on Plane tiles)
    let scp_data = [
        ((7, 7), 5000),
        ((12, 8), 3500),
        ((8, 14), 2000),
        ((14, 13), 4200),
    ];

    // Create crystal mesh (simple cube for now, can be improved later)
    let crystal_mesh = meshes.add(Cuboid::new(0.6, 0.8, 0.6));

    let crystal_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.8, 1.0),
        emissive: Color::srgb(0.2, 0.6, 0.8).into(),
        ..default()
    });

    let mut spawned_count = 0;

    for ((grid_x, grid_z), amount) in scp_data {
        // Find the tile at this position and modify its properties
        for (tile_pos, mut properties) in tiles.iter_mut() {
            if tile_pos.x == grid_x && tile_pos.z == grid_z {
                // Modify tile properties (SCPs make tiles not buildable/traversible)
                properties.buildable = false;
                properties.traversible = false;

                // Calculate world position (centered grid, elevated above tile)
                let world_x = (grid_x as f32 - 10.0) + 0.5;
                let world_z = (grid_z as f32 - 10.0) + 0.5;

                commands.spawn((
                    PbrBundle {
                        mesh: crystal_mesh.clone(),
                        material: crystal_material.clone(),
                        transform: Transform::from_xyz(world_x, 0.4, world_z),
                        ..default()
                    },
                    SpaceCrystalPatch {
                        amount,
                        initial_amount: amount,
                    },
                    Selectable,
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
fn selection_system(
    mut commands: Commands,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    selectables: Query<(Entity, &Transform, Option<&SpaceCrystalPatch>, Option<&SupplyDeliveryStation>), With<Selectable>>,
    selected: Query<Entity, With<Selected>>,
    mut selection_state: ResMut<SelectionState>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    // Don't process clicks if we're dragging
    if selection_state.is_dragging {
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = cameras.single();
    let ctrl_pressed = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let mut closest_entity = None;
            let mut closest_distance = f32::MAX;

            // Check each selectable entity
            for (entity, transform, scp, sds) in selectables.iter() {
                let entity_pos = transform.translation;

                // Simple sphere intersection check (using entity as sphere with radius 0.5)
                let to_entity = entity_pos - ray.origin;
                let projection = to_entity.dot(*ray.direction);

                if projection > 0.0 {
                    let closest_point = ray.origin + *ray.direction * projection;
                    let distance_to_entity = (closest_point - entity_pos).length();

                    if distance_to_entity < 0.5 && projection < closest_distance {
                        closest_distance = projection;
                        closest_entity = Some((entity, scp, sds));
                    }
                }
            }

            if let Some((entity, scp, sds)) = closest_entity {
                // Ctrl+click: Toggle selection
                if ctrl_pressed {
                    if selected.contains(entity) {
                        commands.entity(entity).remove::<Selected>();
                    } else {
                        commands.entity(entity).insert(Selected);
                    }
                } else {
                    // Normal click: Select only this entity
                    // Deselect all others
                    for other_entity in selected.iter() {
                        if other_entity != entity {
                            commands.entity(other_entity).remove::<Selected>();
                        }
                    }
                    commands.entity(entity).insert(Selected);
                }

                // Log SCP info
                if let Some(patch) = scp {
                    info!(
                        "Space Crystal Patch: {} / {} remaining ({:.1}%)",
                        patch.amount,
                        patch.initial_amount,
                        (patch.amount as f32 / patch.initial_amount as f32) * 100.0
                    );
                }

                // Log SDS info
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
                // Clicked empty space without Ctrl: Deselect all
                for entity in selected.iter() {
                    commands.entity(entity).remove::<Selected>();
                }
            }
        }
    }
}

/// System to handle drag-box selection
fn drag_box_system(
    mut commands: Commands,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection_state: ResMut<SelectionState>,
    selectables: Query<(Entity, &Transform), With<Selectable>>,
    selected: Query<Entity, With<Selected>>,
) {
    let window = windows.single();
    let (camera, camera_transform) = cameras.single();

    if let Some(cursor_pos) = window.cursor_position() {
        // Start dragging
        if buttons.just_pressed(MouseButton::Left) && selection_state.drag_start.is_none() {
            selection_state.drag_start = Some(cursor_pos);
            selection_state.is_dragging = false;
        }

        // Continue dragging
        if buttons.pressed(MouseButton::Left) {
            if let Some(start_pos) = selection_state.drag_start {
                let drag_distance = (cursor_pos - start_pos).length();
                if drag_distance > 5.0 {
                    selection_state.is_dragging = true;
                }
            }
        }

        // End dragging
        if buttons.just_released(MouseButton::Left) {
            if selection_state.is_dragging {
                if let Some(start_pos) = selection_state.drag_start {
                    // Calculate selection box bounds
                    let min_x = start_pos.x.min(cursor_pos.x);
                    let max_x = start_pos.x.max(cursor_pos.x);
                    let min_y = start_pos.y.min(cursor_pos.y);
                    let max_y = start_pos.y.max(cursor_pos.y);

                    let ctrl_pressed = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

                    // If not holding Ctrl, deselect all first
                    if !ctrl_pressed {
                        for entity in selected.iter() {
                            commands.entity(entity).remove::<Selected>();
                        }
                    }

                    // Select all units in box
                    for (entity, transform) in selectables.iter() {
                        if let Some(screen_pos) = camera.world_to_viewport(camera_transform, transform.translation) {
                            if screen_pos.x >= min_x && screen_pos.x <= max_x &&
                               screen_pos.y >= min_y && screen_pos.y <= max_y {
                                commands.entity(entity).insert(Selected);
                            }
                        }
                    }
                }
            }

            // Reset drag state
            selection_state.drag_start = None;
            selection_state.is_dragging = false;
        }
    }
}

/// System to log selection count changes
fn log_selection_changes(
    selected: Query<Entity, (With<Selected>, Changed<Selected>)>,
    all_selected: Query<Entity, With<Selected>>,
) {
    if !selected.is_empty() {
        let count = all_selected.iter().count();
        info!("Selection changed: {} unit(s)/entity(ies) selected", count);
    }
}

/// System to draw the drag-box UI
fn draw_drag_box_ui(
    mut commands: Commands,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    selection_state: Res<SelectionState>,
    drag_box: Query<Entity, With<DragBoxUI>>,
) {
    let window = windows.single();

    if selection_state.is_dragging {
        if let Some(start_pos) = selection_state.drag_start {
            if let Some(cursor_pos) = window.cursor_position() {
                // Remove old drag box if it exists
                for entity in drag_box.iter() {
                    commands.entity(entity).despawn();
                }

                // Calculate box dimensions
                let min_x = start_pos.x.min(cursor_pos.x);
                let max_x = start_pos.x.max(cursor_pos.x);
                let min_y = start_pos.y.min(cursor_pos.y);
                let max_y = start_pos.y.max(cursor_pos.y);
                let width = max_x - min_x;
                let height = max_y - min_y;

                // Spawn new drag box UI
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
                    DragBoxUI,
                ));
            }
        }
    } else {
        // Remove drag box when not dragging
        for entity in drag_box.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// System to add/remove selection indicators for selected entities
fn manage_selection_indicators(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    selected: Query<(Entity, &Children), (With<Selected>, Changed<Selected>)>,
    newly_selected: Query<Entity, (With<Selected>, Without<Children>)>,
    deselected: Query<Entity, (Without<Selected>, With<Children>)>,
    indicators: Query<Entity, With<SelectionIndicator>>,
) {
    // Add indicators to newly selected entities
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

    // Remove indicators from deselected entities
    for entity in deselected.iter() {
        if let Some(entity_commands) = commands.get_entity(entity) {
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
fn spawn_supply_delivery_stations(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tiles: Query<(&GridPosition, &mut TileProperties), With<Tile>>,
) {
    // Define SDS locations and properties (on Plane tiles, away from SCPs)
    // (grid_x, grid_z, delivery_size, delivery_interval)
    let sds_data = [
        ((3, 10), 100, 60.0),   // Small station, 60 second interval
        ((17, 10), 200, 45.0),  // Large station, 45 second interval
        ((10, 3), 150, 90.0),   // Medium station, 90 second interval
    ];

    // Create platform mesh (cylinder for landing pad)
    let platform_mesh = meshes.add(Cylinder::new(0.8, 0.2));

    let platform_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.6),  // Gray metallic
        metallic: 0.8,
        perceptual_roughness: 0.3,
        ..default()
    });

    let mut spawned_count = 0;

    for ((grid_x, grid_z), delivery_size, delivery_interval) in sds_data {
        // Find the tile at this position and modify its properties
        for (tile_pos, mut properties) in tiles.iter_mut() {
            if tile_pos.x == grid_x && tile_pos.z == grid_z {
                // Modify tile properties (SDSs make tiles not traversible)
                properties.traversible = false;

                // Calculate world position (centered grid)
                let world_x = (grid_x as f32 - 10.0) + 0.5;
                let world_z = (grid_z as f32 - 10.0) + 0.5;

                commands.spawn((
                    PbrBundle {
                        mesh: platform_mesh.clone(),
                        material: platform_material.clone(),
                        transform: Transform::from_xyz(world_x, 0.1, world_z),
                        ..default()
                    },
                    SupplyDeliveryStation {
                        delivery_size,
                        delivery_interval,
                        current_supplies: delivery_size, // Start with supplies
                        time_until_next_delivery: delivery_interval,
                    },
                    Selectable,
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
fn sds_delivery_timer(
    time: Res<Time>,
    mut stations: Query<&mut SupplyDeliveryStation>,
) {
    for mut sds in stations.iter_mut() {
        // Only count down when empty
        if sds.current_supplies == 0 {
            sds.time_until_next_delivery -= time.delta_seconds();

            // Deliver supplies when timer reaches 0
            if sds.time_until_next_delivery <= 0.0 {
                sds.current_supplies = sds.delivery_size;
                sds.time_until_next_delivery = sds.delivery_interval;
                info!("Supply Delivery: {} supplies delivered", sds.delivery_size);
            }
        }
    }
}
