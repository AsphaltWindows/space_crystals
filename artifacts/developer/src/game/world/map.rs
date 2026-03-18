use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::collections::HashSet;
use crate::types::{GridPosition, MainCamera, VisibleEntity, Owner, SightRange, LocalPlayer,
                   Unit, VisibilityStateEnum};
use crate::game::types::ObjectInstance;
use super::types::*;
use super::utils::{world_to_grid, vision_center};

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
                Mesh3d(tile_mesh.clone()),
                MeshMaterial3d(tile_material),
                Transform::from_xyz(world_x, 0.0, world_z),
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

/// Maximum draw distance for grid lines from the camera's ground projection.
/// Set to 40.0 to ensure full coverage of the 64×64 map (half-extent = 32)
/// from any camera position, with headroom for edge panning.
const GRID_LINE_DRAW_RADIUS: f32 = 40.0;

/// Distance from camera beyond which grid lines begin to fade out.
/// Lines between FADE_START and DRAW_RADIUS fade linearly from full alpha to zero.
/// This prevents the dark band artifact caused by many semi-transparent lines
/// compounding under perspective compression at the far edge of the view.
const GRID_LINE_FADE_START: f32 = 20.0;

/// Base alpha for grid lines at full opacity (within FADE_START distance).
const GRID_LINE_BASE_ALPHA: f32 = 0.35;

/// Compute the opacity for a grid line at a given distance from the camera.
/// Returns `GRID_LINE_BASE_ALPHA` for distances ≤ `GRID_LINE_FADE_START`,
/// fades with a cubic curve to 0.0 at `GRID_LINE_DRAW_RADIUS`.
///
/// The cubic falloff (power of 3) is critical: under perspective projection,
/// grid lines far from the camera compress into a narrow screen-space band.
/// A linear fade leaves enough residual alpha that 10+ overlapping lines
/// compound into a visible dark band. The cubic curve ensures lines in the
/// compressed far zone have negligible individual alpha, preventing the
/// compounding artifact.
fn grid_line_alpha(distance: f32) -> f32 {
    if distance <= GRID_LINE_FADE_START {
        GRID_LINE_BASE_ALPHA
    } else if distance >= GRID_LINE_DRAW_RADIUS {
        0.0
    } else {
        let t = (distance - GRID_LINE_FADE_START) / (GRID_LINE_DRAW_RADIUS - GRID_LINE_FADE_START);
        let remaining = 1.0 - t;
        GRID_LINE_BASE_ALPHA * remaining * remaining * remaining
    }
}

/// System to draw grid cell outlines using Gizmos.
///
/// Draws grid lines within `GRID_LINE_DRAW_RADIUS` of the camera's ground
/// projection point. Lines beyond `GRID_LINE_FADE_START` fade out linearly,
/// reaching zero opacity at `GRID_LINE_DRAW_RADIUS`. This prevents the dark
/// band artifact from perspective compression while covering the full map.
pub fn draw_grid_lines(
    mut gizmos: Gizmos,
    grid: Res<GridMap>,
    camera_query: Query<&Transform, With<MainCamera>>,
) {
    let half_w = grid.width as f32 / 2.0;
    let half_h = grid.height as f32 / 2.0;
    let y = 0.005;

    // Get camera ground projection for distance culling
    let cam_x = camera_query.iter().next().map(|t| t.translation.x).unwrap_or(0.0);
    let cam_z = camera_query.iter().next().map(|t| t.translation.z).unwrap_or(0.0);
    let radius = GRID_LINE_DRAW_RADIUS;

    // Compute visible grid index range (clamped to grid bounds)
    let min_x = ((cam_x - radius + half_w).floor().max(0.0) as u32).min(grid.width);
    let max_x = ((cam_x + radius + half_w).ceil().max(0.0) as u32).min(grid.width);
    let min_z = ((cam_z - radius + half_h).floor().max(0.0) as u32).min(grid.height);
    let max_z = ((cam_z + radius + half_h).ceil().max(0.0) as u32).min(grid.height);

    // World-space clamp bounds for line endpoints
    let clip_min_x = (cam_x - radius).max(-half_w);
    let clip_max_x = (cam_x + radius).min(half_w);
    let clip_min_z = (cam_z - radius).max(-half_h);
    let clip_max_z = (cam_z + radius).min(half_h);

    // Vertical lines (along Z axis) — each with distance-based opacity
    for x in min_x..=max_x {
        let wx = x as f32 - half_w;
        let dist = (wx - cam_x).abs();
        let alpha = grid_line_alpha(dist);
        if alpha <= 0.0 {
            continue;
        }
        let color = Color::srgba(0.0, 0.0, 0.0, alpha);
        gizmos.line(
            Vec3::new(wx, y, clip_min_z),
            Vec3::new(wx, y, clip_max_z),
            color,
        );
    }

    // Horizontal lines (along X axis) — each with distance-based opacity
    for z in min_z..=max_z {
        let wz = z as f32 - half_h;
        let dist = (wz - cam_z).abs();
        let alpha = grid_line_alpha(dist);
        if alpha <= 0.0 {
            continue;
        }
        let color = Color::srgba(0.0, 0.0, 0.0, alpha);
        gizmos.line(
            Vec3::new(clip_min_x, y, wz),
            Vec3::new(clip_max_x, y, wz),
            color,
        );
    }
}

/// System to display tile info when hovering with mouse
pub fn tile_hover_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    tiles: Query<(&TilePresetEnum, &TilePreset, &GridPosition), With<Tile>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = cameras.single() else { return; };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
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
            // Compute vision center: for multi-tile structures, offset by half the footprint.
            // Only apply size offset for structures — unit sizes are in space units, not grid tiles.
            let size = obj_instance
                .filter(|obj| obj.object_type.is_structure())
                .map(|obj| obj.object_type.object_type().size);
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
        (&GridPosition, &TilePresetEnum, &mut MeshMaterial3d<StandardMaterial>),
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

        if let Some(mat) = materials.get_mut(material_handle.0.id()) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ObjectEnum;
    use crate::game::types::ObjectInstance;

    // === Vision center filtering tests ===

    #[test]
    fn unit_object_instance_is_not_structure() {
        // Peacekeeper has size (24,24) in space units — must NOT be treated as grid offset
        assert!(!ObjectEnum::Peacekeeper.is_structure());
        assert!(!ObjectEnum::SupplyChopper.is_structure());
        assert!(!ObjectEnum::SyndicateAgent.is_structure());
    }

    #[test]
    fn structure_object_instance_is_structure() {
        assert!(ObjectEnum::DeploymentCenter.is_structure());
        assert!(ObjectEnum::PowerPlant.is_structure());
        assert!(ObjectEnum::Barracks.is_structure());
    }

    #[test]
    fn unit_vision_center_has_no_size_offset() {
        // Simulate what update_fog_of_war does for a unit
        let obj = ObjectInstance::destructible(ObjectEnum::Peacekeeper, 100.0);
        let size = Some(&obj)
            .filter(|o| o.object_type.is_structure())
            .map(|o| o.object_type.object_type().size);
        // For units, size should be None (filtered out)
        assert!(size.is_none());
        let (cx, cz) = vision_center(32, 32, size);
        assert_eq!((cx, cz), (32, 32));
    }

    #[test]
    fn structure_vision_center_has_size_offset() {
        // Simulate what update_fog_of_war does for a structure
        let obj = ObjectInstance::destructible(ObjectEnum::DeploymentCenter, 500.0);
        let size = Some(&obj)
            .filter(|o| o.object_type.is_structure())
            .map(|o| o.object_type.object_type().size);
        // DeploymentCenter is (4,4), so offset by (2,2)
        assert!(size.is_some());
        let (cx, cz) = vision_center(30, 30, size);
        assert_eq!((cx, cz), (32, 32));
    }

    #[test]
    fn peacekeeper_size_is_space_units_not_grid() {
        // Peacekeeper size is 24x24 space units — applying this as grid offset would be wrong
        let obj_type = ObjectEnum::Peacekeeper.object_type();
        assert_eq!(obj_type.size, (24, 24));
        // If we naively used this, vision would be offset by (12,12) tiles — clearly wrong
        let (bad_cx, bad_cz) = vision_center(32, 32, Some((24, 24)));
        assert_eq!((bad_cx, bad_cz), (44, 44)); // 12 tiles off!
        // With the fix (filtering out units), no offset
        let (good_cx, good_cz) = vision_center(32, 32, None);
        assert_eq!((good_cx, good_cz), (32, 32));
    }

    #[test]
    fn supply_chopper_vision_center_no_offset() {
        let obj = ObjectInstance::destructible(ObjectEnum::SupplyChopper, 50.0);
        let size = Some(&obj)
            .filter(|o| o.object_type.is_structure())
            .map(|o| o.object_type.object_type().size);
        assert!(size.is_none());
        let (cx, cz) = vision_center(20, 20, size);
        assert_eq!((cx, cz), (20, 20));
    }

    #[test]
    fn syndicate_agent_vision_center_no_offset() {
        let obj = ObjectInstance::destructible(ObjectEnum::SyndicateAgent, 75.0);
        let size = Some(&obj)
            .filter(|o| o.object_type.is_structure())
            .map(|o| o.object_type.object_type().size);
        assert!(size.is_none());
        let (cx, cz) = vision_center(15, 40, size);
        assert_eq!((cx, cz), (15, 40));
    }

    #[test]
    fn entity_without_object_instance_no_offset() {
        // Entities with SightRange but no ObjectInstance (e.g. future sensor wards)
        let size: Option<(u32, u32)> = None;
        let (cx, cz) = vision_center(10, 10, size);
        assert_eq!((cx, cz), (10, 10));
    }

    #[test]
    fn tunnel_vision_center_has_structure_offset() {
        let obj = ObjectInstance::destructible(ObjectEnum::Tunnel, 200.0);
        let size = Some(&obj)
            .filter(|o| o.object_type.is_structure())
            .map(|o| o.object_type.object_type().size);
        assert!(size.is_some());
        let obj_type = ObjectEnum::Tunnel.object_type();
        let (cx, cz) = vision_center(25, 25, Some(obj_type.size));
        // Tunnel size determines offset
        let expected_cx = 25 + (obj_type.size.0 as i32) / 2;
        let expected_cz = 25 + (obj_type.size.1 as i32) / 2;
        assert_eq!((cx, cz), (expected_cx, expected_cz));
    }

    // === Grid line draw radius and opacity falloff tests ===

    #[test]
    fn grid_line_draw_radius_covers_full_map() {
        // Radius must be >= half the grid extent to cover full map from any camera position
        let grid = GridMap::default();
        let half_extent = (grid.width as f32 / 2.0).max(grid.height as f32 / 2.0);
        assert!(
            GRID_LINE_DRAW_RADIUS >= half_extent,
            "Draw radius {} must be >= map half-extent {} to cover full map",
            GRID_LINE_DRAW_RADIUS,
            half_extent
        );
    }

    #[test]
    fn grid_line_fade_start_less_than_draw_radius() {
        assert!(GRID_LINE_FADE_START < GRID_LINE_DRAW_RADIUS);
    }

    #[test]
    fn grid_line_alpha_full_within_fade_start() {
        assert_eq!(grid_line_alpha(0.0), GRID_LINE_BASE_ALPHA);
        assert_eq!(grid_line_alpha(10.0), GRID_LINE_BASE_ALPHA);
        assert_eq!(grid_line_alpha(GRID_LINE_FADE_START), GRID_LINE_BASE_ALPHA);
    }

    #[test]
    fn grid_line_alpha_zero_at_draw_radius() {
        assert_eq!(grid_line_alpha(GRID_LINE_DRAW_RADIUS), 0.0);
        assert_eq!(grid_line_alpha(GRID_LINE_DRAW_RADIUS + 5.0), 0.0);
    }

    #[test]
    fn grid_line_alpha_fades_cubically() {
        // At the midpoint of the fade zone (t=0.5), cubic fade gives (1-0.5)^3 = 0.125
        let midpoint = (GRID_LINE_FADE_START + GRID_LINE_DRAW_RADIUS) / 2.0;
        let mid_alpha = grid_line_alpha(midpoint);
        let expected = GRID_LINE_BASE_ALPHA * 0.125; // (0.5)^3
        assert!((mid_alpha - expected).abs() < 0.001,
            "Midpoint alpha {} should be ~{} (cubic falloff)", mid_alpha, expected);
    }

    #[test]
    fn grid_line_alpha_far_lines_negligible() {
        // Lines at 75% through the fade zone should have very low alpha,
        // preventing the dark band artifact from perspective compression.
        let far_dist = GRID_LINE_FADE_START + 0.75 * (GRID_LINE_DRAW_RADIUS - GRID_LINE_FADE_START);
        let alpha = grid_line_alpha(far_dist);
        // (1-0.75)^3 = 0.015625 * 0.35 ≈ 0.0055
        assert!(alpha < 0.01,
            "Far fade zone alpha {} should be < 0.01 to prevent compounding artifact", alpha);
    }

    #[test]
    fn grid_line_alpha_monotonically_decreasing_in_fade_zone() {
        let a1 = grid_line_alpha(GRID_LINE_FADE_START + 1.0);
        let a2 = grid_line_alpha(GRID_LINE_FADE_START + 5.0);
        let a3 = grid_line_alpha(GRID_LINE_FADE_START + 10.0);
        assert!(a1 > a2, "alpha at closer distance should be higher");
        assert!(a2 > a3, "alpha at closer distance should be higher");
    }

    #[test]
    fn grid_line_cull_range_centered_covers_full_grid() {
        // Camera at origin with radius 40 on 64x64 grid — should cover entire grid
        let grid = GridMap::default();
        let half_w = grid.width as f32 / 2.0; // 32.0
        let cam_x = 0.0_f32;
        let radius = GRID_LINE_DRAW_RADIUS;

        let min_x = ((cam_x - radius + half_w).floor().max(0.0) as u32).min(grid.width);
        let max_x = ((cam_x + radius + half_w).ceil().max(0.0) as u32).min(grid.width);

        // cam_x - radius + half_w = 0 - 40 + 32 = -8 → clamped to 0
        // cam_x + radius + half_w = 0 + 40 + 32 = 72 → clamped to 64
        assert_eq!(min_x, 0);
        assert_eq!(max_x, 64);
        // Full grid coverage: 65 lines for 64 cells
        assert_eq!(max_x - min_x + 1, grid.width + 1);
    }

    #[test]
    fn grid_line_cull_range_edge_camera_still_covers_far_side() {
        // Camera near the left edge — should still cover right edge
        let grid = GridMap::default();
        let half_w = grid.width as f32 / 2.0;
        let cam_x = -25.0_f32;
        let radius = GRID_LINE_DRAW_RADIUS;

        let min_x = ((cam_x - radius + half_w).floor().max(0.0) as u32).min(grid.width);
        let max_x = ((cam_x + radius + half_w).ceil().max(0.0) as u32).min(grid.width);

        // cam_x - radius + half_w = -25 - 40 + 32 = -33 → clamped to 0
        assert_eq!(min_x, 0);
        // cam_x + radius + half_w = -25 + 40 + 32 = 47
        assert_eq!(max_x, 47);
    }

    #[test]
    fn grid_line_clips_to_grid_bounds() {
        // Line endpoints clamped to grid world-space bounds
        let grid = GridMap::default();
        let half_w = grid.width as f32 / 2.0;
        let half_h = grid.height as f32 / 2.0;
        let cam_x = 0.0_f32;
        let cam_z = 0.0_f32;
        let radius = GRID_LINE_DRAW_RADIUS;

        let clip_min_x = (cam_x - radius).max(-half_w);
        let clip_max_x = (cam_x + radius).min(half_w);
        let clip_min_z = (cam_z - radius).max(-half_h);
        let clip_max_z = (cam_z + radius).min(half_h);

        // Radius 40 > half_w 32, so clamps to grid bounds
        assert_eq!(clip_min_x, -32.0);
        assert_eq!(clip_max_x, 32.0);
        assert_eq!(clip_min_z, -32.0);
        assert_eq!(clip_max_z, 32.0);
        // Full grid coverage
        assert_eq!(clip_max_x - clip_min_x, grid.width as f32);
    }

    #[test]
    fn grid_line_edge_lines_have_reduced_opacity() {
        // Lines at the map edge (32 units from center camera) should be faded
        let edge_dist = 32.0_f32;
        let alpha = grid_line_alpha(edge_dist);
        assert!(alpha < GRID_LINE_BASE_ALPHA,
            "Edge line at distance {} should be faded (alpha={}, base={})",
            edge_dist, alpha, GRID_LINE_BASE_ALPHA);
        assert!(alpha > 0.0, "Edge line should still be visible (alpha={})", alpha);
    }

    #[test]
    fn grid_line_near_lines_have_full_opacity() {
        // Lines close to camera should have full opacity
        let near_dist = 5.0_f32;
        let alpha = grid_line_alpha(near_dist);
        assert_eq!(alpha, GRID_LINE_BASE_ALPHA);
    }
}
