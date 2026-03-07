use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::collections::HashSet;
use crate::types::{GridPosition, MainCamera, VisibleEntity, Owner, SightRange, LocalPlayer,
                   Unit, VisibilityStateEnum};
use crate::game::types::ObjectInstance;
use super::types::*;
use super::utils::{world_to_grid, vision_center, cursor_pos_in_viewport};

/// Determine tile type based on grid position for a 64x64 map
/// Creates a procedural terrain with water borders, scattered mountains/cliffs, and rugged areas
fn determine_tile_type(x: u32, z: u32, width: u32, height: u32) -> TilePresetEnum {
    // Water borders (2-tile border)
    if x < 2 || x >= width - 2 || z < 2 || z >= height - 2 {
        return TilePresetEnum::Water;
    }

    // Rugged terrain transition zone (tiles 2-4 from edge)
    if x < 5 || x >= width - 5 || z < 5 || z >= height - 5 {
        if (x + z) % 3 == 0 {
            return TilePresetEnum::Water;
        }
        return TilePresetEnum::RuggedTerrain;
    }

    // Scattered mountain clusters (using a simple hash to create clusters)
    let hash = simple_hash(x, z);
    let cluster_hash = simple_hash(x / 6, z / 6);

    // Mountain areas (a few clusters spread around the map)
    if cluster_hash % 17 == 0 && hash % 4 == 0 {
        return TilePresetEnum::Mountain;
    }

    // Cliff areas near mountains
    if cluster_hash % 17 == 0 && hash % 3 == 0 {
        return TilePresetEnum::Cliff;
    }

    // Scattered rugged terrain patches
    if hash % 11 == 0 {
        return TilePresetEnum::RuggedTerrain;
    }

    // Central water features (a few small lakes)
    let cx = width / 2;
    let cz = height / 2;
    // Lake 1: offset from center
    if dist_sq(x, z, cx - 12, cz + 8) < 9 {
        return TilePresetEnum::Water;
    }
    // Lake 2: another offset
    if dist_sq(x, z, cx + 15, cz - 10) < 6 {
        return TilePresetEnum::Water;
    }

    TilePresetEnum::Plane
}

/// Simple spatial hash for procedural terrain generation
fn simple_hash(x: u32, z: u32) -> u32 {
    let mut h = x.wrapping_mul(374761393).wrapping_add(z.wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

/// Squared distance between two grid positions
fn dist_sq(x1: u32, z1: u32, x2: u32, z2: u32) -> u32 {
    let dx = x1 as i32 - x2 as i32;
    let dz = z1 as i32 - z2 as i32;
    (dx * dx + dz * dz) as u32
}

/// System to spawn the grid of tiles and populate the ElevationMap resource
pub fn spawn_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid: Res<GridMap>,
    mut elevation_map: ResMut<ElevationMap>,
) {
    let tile_mesh = meshes.add(Plane3d::default().mesh().size(grid.cell_size, grid.cell_size));

    let offset_x = -(grid.width as f32 * grid.cell_size) / 2.0 + grid.cell_size / 2.0;
    let offset_z = -(grid.height as f32 * grid.cell_size) / 2.0 + grid.cell_size / 2.0;

    let mut tile_type_counts = std::collections::HashMap::new();

    for x in 0..grid.width {
        for z in 0..grid.height {
            let grid_pos = GridPosition {
                x: x as i32,
                z: z as i32,
            };

            let tile_type = determine_tile_type(x, z, grid.width, grid.height);
            let properties = tile_type.properties();

            *tile_type_counts.entry(tile_type).or_insert(0) += 1;

            let tile_material = materials.add(StandardMaterial {
                base_color: tile_type.color(),
                ..default()
            });

            let world_x = offset_x + x as f32 * grid.cell_size;
            let world_z = offset_z + z as f32 * grid.cell_size;

            let elevation = 0u8;

            commands.spawn((
                PbrBundle {
                    mesh: tile_mesh.clone(),
                    material: tile_material,
                    transform: Transform::from_xyz(world_x, 0.0, world_z),
                    ..default()
                },
                Tile,
                VisibleEntity,
                tile_type,
                properties,
                TilePlacement::new(tile_type, grid_pos, elevation)
                    .expect("Default elevation 0 is always valid"),
                grid_pos,
            ));

            elevation_map.insert(grid_pos.x, grid_pos.z, elevation);
        }
    }

    info!(
        "Spawned {}x{} grid ({} tiles)",
        grid.width,
        grid.height,
        grid.width * grid.height
    );
    info!("Tile distribution: {:?}", tile_type_counts);
}

/// System to draw grid cell outlines using Gizmos
pub fn draw_grid_lines(mut gizmos: Gizmos, grid: Res<GridMap>) {
    let half_w = grid.width as f32 / 2.0;
    let half_h = grid.height as f32 / 2.0;
    let color = Color::srgba(0.0, 0.0, 0.0, 0.35);
    let y = 0.005;

    // Vertical lines (along Z axis)
    for x in 0..=grid.width {
        let wx = x as f32 - half_w;
        gizmos.line(Vec3::new(wx, y, -half_h), Vec3::new(wx, y, half_h), color);
    }

    // Horizontal lines (along X axis)
    for z in 0..=grid.height {
        let wz = z as f32 - half_h;
        gizmos.line(Vec3::new(-half_w, y, wz), Vec3::new(half_w, y, wz), color);
    }
}

/// System to display tile info when hovering with mouse
pub fn tile_hover_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    tiles: Query<(&TilePresetEnum, &TilePreset, &GridPosition), With<Tile>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let window = windows.single();
    let (camera, camera_transform) = cameras.single();

    if let Some(cursor_pos) = cursor_pos_in_viewport(window, camera) {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            if ray.direction.y.abs() > 0.001 {
                let t = -ray.origin.y / ray.direction.y;
                if t > 0.0 {
                    let intersection = ray.origin + ray.direction * t;
                    let (grid_x, grid_z) = world_to_grid(intersection, 1.0);

                    for (tile_type, properties, grid_pos) in tiles.iter() {
                        if grid_pos.x == grid_x && grid_pos.z == grid_z {
                            if buttons.just_pressed(MouseButton::Left) {
                                info!(
                                    "Tile at ({}, {}): {:?} - Buildable: {}, Traversible: {}, Drillable: {}, Rugged: {}, Recruitable: {}",
                                    grid_x, grid_z, tile_type,
                                    properties.buildable,
                                    properties.traversible,
                                    properties.drillable,
                                    properties.rugged,
                                    properties.recruitable
                                );
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
}

// =====================================================
// FOG OF WAR VISION SYSTEMS
// =====================================================

/// System that recalculates fog of war visibility each fixed tick.
/// For each player, builds a set of currently-visible tiles from all their
/// vision sources (entities with SightRange + Owner), then updates the FogOfWarMap.
pub fn update_fog_of_war(
    mut fog_map: ResMut<FogOfWarMap>,
    mut last_known: ResMut<LastKnownStructures>,
    vision_sources: Query<(&SightRange, &GridPosition, &Owner, Option<&ObjectInstance>)>,
    structures_on_tiles: Query<(&GridPosition, &ObjectInstance, &Owner), Without<Unit>>,
) {
    // Collect all player IDs that have vision sources
    let mut player_visible_tiles: std::collections::HashMap<u8, HashSet<(i32, i32)>> =
        std::collections::HashMap::new();

    // Build visible tile sets for each player
    for (sight_range, grid_pos, owner, obj_instance) in vision_sources.iter() {
        if let Some(player_id) = owner.player_number() {
            // Compute vision center: for multi-tile structures, offset by half the footprint
            let size = obj_instance.map(|obj| obj.object_type.object_type().size);
            let (cx, cz) = vision_center(grid_pos.x, grid_pos.z, size);

            let visible_set = player_visible_tiles.entry(player_id).or_default();
            let tiles = fog_map.tiles_in_sight_range(cx, cz, sight_range.0);
            for tile in tiles {
                visible_set.insert(tile);
            }
        }
    }

    // Update fog state for each player
    for (&player_id, visible_set) in player_visible_tiles.iter() {
        fog_map.ensure_player(player_id);

        let width = fog_map.width;
        let height = fog_map.height;

        for z in 0..height as i32 {
            for x in 0..width as i32 {
                let current_state = fog_map.get(player_id, x, z);
                let is_visible_now = visible_set.contains(&(x, z));

                let new_state = if is_visible_now {
                    // Clear last-known snapshot when tile becomes visible again
                    last_known.entries.remove(&(player_id, x, z));
                    VisibilityStateEnum::Visible
                } else if current_state == VisibilityStateEnum::Visible {
                    // Transition from Visible to Explored — snapshot structures
                    for (struct_pos, obj_instance, _struct_owner) in structures_on_tiles.iter() {
                        if struct_pos.x == x && struct_pos.z == z {
                            last_known.entries.insert(
                                (player_id, x, z),
                                LastKnownStructure {
                                    object_type: obj_instance.object_type,
                                    hp_fraction: match (obj_instance.hp, obj_instance.max_hp) {
                                        (Some(hp), Some(max_hp)) if max_hp > 0.0 => hp / max_hp,
                                        _ => 1.0,
                                    },
                                },
                            );
                        }
                    }
                    VisibilityStateEnum::Explored
                } else {
                    // Unexplored stays Unexplored, Explored stays Explored
                    current_state
                };

                fog_map.set(player_id, x, z, new_state);
            }
        }
    }
}

/// System that applies fog-of-war visual effects for the local player.
/// - Hides enemy units on non-Visible tiles (sets Visibility::Hidden)
/// - Adjusts tile material colors based on fog state:
///   Unexplored = dark (near-black), Explored = dimmed (50%), Visible = full color
pub fn apply_fog_rendering(
    fog_map: Res<FogOfWarMap>,
    local_player: Res<LocalPlayer>,
    mut enemy_units: Query<
        (&GridPosition, &Owner, &mut Visibility),
        (With<Unit>, Without<Tile>),
    >,
    mut tile_visuals: Query<
        (&GridPosition, &TilePresetEnum, &mut Handle<StandardMaterial>),
        With<Tile>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_id = local_player.0;

    // Hide/show enemy units based on fog state
    for (grid_pos, owner, mut visibility) in enemy_units.iter_mut() {
        // Only affect non-local-player entities
        if owner.player_number() == Some(player_id) {
            continue;
        }

        let tile_state = fog_map.get(player_id, grid_pos.x, grid_pos.z);
        *visibility = if tile_state == VisibilityStateEnum::Visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    // Adjust tile colors based on fog state
    for (grid_pos, tile_type, material_handle) in tile_visuals.iter_mut() {
        let tile_state = fog_map.get(player_id, grid_pos.x, grid_pos.z);

        let base_color = tile_type.color();
        let fog_multiplier = match tile_state {
            VisibilityStateEnum::Unexplored => 0.1,
            VisibilityStateEnum::Explored => 0.5,
            VisibilityStateEnum::Visible => 1.0,
        };

        let fogged_color = Color::srgb(
            base_color.to_srgba().red * fog_multiplier,
            base_color.to_srgba().green * fog_multiplier,
            base_color.to_srgba().blue * fog_multiplier,
        );

        if let Some(mat) = materials.get_mut(material_handle.id()) {
            mat.base_color = fogged_color;
        }
    }
}

/// System that hides structures on unexplored tiles for the local player.
/// Structures on Explored tiles remain visible (last-known state).
/// Structures on Unexplored tiles are hidden entirely.
pub fn apply_structure_fog_rendering(
    fog_map: Res<FogOfWarMap>,
    local_player: Res<LocalPlayer>,
    mut structures: Query<
        (&GridPosition, &Owner, &mut Visibility),
        (With<ObjectInstance>, Without<Unit>, Without<Tile>),
    >,
) {
    let player_id = local_player.0;

    for (grid_pos, owner, mut visibility) in structures.iter_mut() {
        // Own structures are always visible
        if owner.player_number() == Some(player_id) {
            continue;
        }
        // Neutral structures (patches, SDS) are always visible
        if owner.is_neutral() {
            continue;
        }

        let tile_state = fog_map.get(player_id, grid_pos.x, grid_pos.z);
        *visibility = match tile_state {
            VisibilityStateEnum::Unexplored => Visibility::Hidden,
            VisibilityStateEnum::Explored => Visibility::Inherited, // Show in last-known state
            VisibilityStateEnum::Visible => Visibility::Inherited,
        };
    }
}
