use bevy::prelude::*;
use bevy::window::Window;
use crate::types::{GridPosition, ObjectEnum, StructureRotation, VisibilityStateEnum};
use super::types::*;

/// Compute the viewport offset in logical (window-space) pixels.
/// The camera viewport's `physical_position` is in physical pixels;
/// dividing by the window's scale factor gives logical pixels.
/// Returns `Vec2::ZERO` if the camera has no custom viewport.
pub fn viewport_offset(camera: &Camera, scale_factor: f32) -> Vec2 {
    if let Some(viewport) = &camera.viewport {
        Vec2::new(
            viewport.physical_position.x as f32 / scale_factor,
            viewport.physical_position.y as f32 / scale_factor,
        )
    } else {
        Vec2::ZERO
    }
}

/// Returns cursor position adjusted for the camera viewport offset.
/// `window.cursor_position()` returns window-space coordinates, but camera functions
/// (`viewport_to_world`, `world_to_viewport`) operate in viewport-space.
/// This converts window-space cursor position to viewport-space.
pub fn cursor_pos_in_viewport(window: &Window, camera: &Camera) -> Option<Vec2> {
    let cursor_pos = window.cursor_position()?;
    let offset = viewport_offset(camera, window.scale_factor());
    Some(cursor_pos - offset)
}

/// Helper function to convert world coordinates to grid coordinates (map version)
/// Assumes grid is centered at world origin (half-width = 32 for 64x64)
pub fn world_to_grid(world_pos: Vec3, cell_size: f32) -> (i32, i32) {
    let half_size = 32.0; // Half of 64x64 grid
    let x = ((world_pos.x + half_size * cell_size) / cell_size).floor() as i32;
    let z = ((world_pos.z + half_size * cell_size) / cell_size).floor() as i32;
    (x, z)
}

/// Helper function to convert grid coordinates to world coordinates (map version)
/// Assumes grid is centered at world origin (half-width = 32 for 64x64)
pub fn grid_to_world(grid_x: i32, grid_z: i32, cell_size: f32) -> Vec3 {
    let half_size = 32.0; // Half of 64x64 grid
    Vec3::new(
        (grid_x as f32 - half_size) * cell_size + cell_size / 2.0,
        0.0,
        (grid_z as f32 - half_size) * cell_size + cell_size / 2.0,
    )
}

/// Expand the build area around a placed building using Chebyshev distance
/// building_pos: top-left grid coordinate of the building
/// building_size: (width, height) in grid units
/// extension: BuildRadiusExtension value (Chebyshev distance in grid units)
pub fn expand_build_area(
    build_area: &mut GdoBuildArea,
    building_pos_x: i32,
    building_pos_z: i32,
    building_size_x: u32,
    building_size_z: u32,
    extension: u32,
) {
    let ext = extension as i32;

    // For each cell the building occupies, add all cells within Chebyshev distance
    for bx in 0..building_size_x as i32 {
        for bz in 0..building_size_z as i32 {
            let cell_x = building_pos_x + bx;
            let cell_z = building_pos_z + bz;

            for dx in -ext..=ext {
                for dz in -ext..=ext {
                    build_area.cells.insert((cell_x + dx, cell_z + dz));
                }
            }
        }
    }
}

/// Get the building footprint size, accounting for rotation
/// Returns (width, height) in grid units after rotation
#[allow(dead_code)]
pub fn rotated_building_size(
    base_size_x: u32,
    base_size_z: u32,
    rotation: &StructureRotation,
) -> (u32, u32) {
    match rotation {
        StructureRotation::R0 | StructureRotation::R180 => (base_size_x, base_size_z),
        StructureRotation::R90 | StructureRotation::R270 => (base_size_z, base_size_x),
    }
}

/// Screen-space click hit test: checks if a cursor position is within
/// a pixel radius of a projected screen position.
/// Returns the screen distance if within the radius, None otherwise.
pub fn screen_space_hit_test(
    cursor_pos: Vec2,
    screen_pos: Vec2,
    click_radius: f32,
) -> Option<f32> {
    let dist = (cursor_pos - screen_pos).length();
    if dist < click_radius {
        Some(dist)
    } else {
        None
    }
}

/// Represents an entity candidate found within a drag-box selection area.
/// Used to categorize entities by selection tier before applying priority logic.
pub struct BoxCandidate {
    pub entity: Entity,
    pub screen_pos: Vec2,
}

/// Selection priority tier for box-selection.
/// Higher tiers take priority over lower tiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionTier {
    OwnUnits,
    OwnStructures,
    EnemyUnits,
    EnemyStructures,
    Neutrals,
}

/// Find the candidate whose screen position is closest to the given center point.
/// Returns the entity of the closest candidate.
/// Panics if candidates is empty — caller must check before calling.
pub fn closest_to_center(candidates: &[BoxCandidate], center: Vec2) -> Entity {
    candidates
        .iter()
        .min_by(|a, b| {
            let dist_a = (a.screen_pos - center).length_squared();
            let dist_b = (b.screen_pos - center).length_squared();
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("closest_to_center called with empty candidates")
        .entity
}

/// Classify an entity into a selection tier based on ownership and type markers.
pub fn classify_selection_tier(
    is_owned: bool,
    is_neutral: bool,
    is_unit: bool,
    is_structure: bool,
) -> SelectionTier {
    if is_owned && is_unit {
        SelectionTier::OwnUnits
    } else if is_owned && is_structure {
        SelectionTier::OwnStructures
    } else if !is_neutral && is_unit {
        SelectionTier::EnemyUnits
    } else if !is_neutral && is_structure {
        SelectionTier::EnemyStructures
    } else {
        SelectionTier::Neutrals
    }
}

/// Check if an underground expansion can be placed within a Tunnel's area.
/// Returns Ok(()) if valid, Err(reason) if not.
/// Validates: all cells are within the tunnel area, and no overlap with existing expansions.
pub fn can_place_expansion(
    pos_x: i32,
    pos_z: i32,
    size_x: u32,
    size_z: u32,
    tunnel_area: &crate::game::types::TunnelArea,
    existing_expansion_positions: &[(i32, i32)],
) -> Result<(), &'static str> {
    // All cells must be within the tunnel area
    if !tunnel_area.fits_expansion(pos_x, pos_z, size_x, size_z) {
        return Err("Expansion does not fit within Tunnel Area");
    }

    // No overlap with existing expansion cells
    for dx in 0..size_x as i32 {
        for dz in 0..size_z as i32 {
            let check_x = pos_x + dx;
            let check_z = pos_z + dz;
            if existing_expansion_positions.contains(&(check_x, check_z)) {
                return Err("Overlaps existing expansion");
            }
        }
    }

    Ok(())
}

/// Compute the vision center for an entity.
/// For multi-tile structures, offsets the GridPosition by half the footprint.
/// For units or entities without an ObjectInstance, returns the GridPosition as-is.
pub fn vision_center(grid_x: i32, grid_z: i32, size: Option<(u32, u32)>) -> (i32, i32) {
    match size {
        Some((sx, sz)) => (grid_x + (sx as i32) / 2, grid_z + (sz as i32) / 2),
        None => (grid_x, grid_z),
    }
}

/// Check if all tiles under a building footprint are Visible for a given player.
/// Returns true only if every cell in the footprint is in VisibilityStateEnum::Visible.
pub fn footprint_is_visible(
    fog_map: &FogOfWarMap,
    player_id: u8,
    pos_x: i32,
    pos_z: i32,
    size_x: u32,
    size_z: u32,
) -> bool {
    for dx in 0..size_x as i32 {
        for dz in 0..size_z as i32 {
            if fog_map.get(player_id, pos_x + dx, pos_z + dz) != VisibilityStateEnum::Visible {
                return false;
            }
        }
    }
    true
}

/// Check if a building can be placed at the given position
/// Returns Ok(()) if placement is valid, Err(reason) if not
pub fn can_place_building(
    pos_x: i32,
    pos_z: i32,
    size_x: u32,
    size_z: u32,
    object_type: ObjectEnum,
    build_area: &GdoBuildArea,
    tiles: &Query<(&GridPosition, &TilePreset), With<Tile>>,
    structures: &Query<(&GridPosition, &crate::game::types::StructureInstance)>,
    patches: &Query<(&GridPosition, &SpaceCrystalPatch)>,
    fog_map: &FogOfWarMap,
    player_id: u8,
) -> Result<(), &'static str> {
    let is_extraction_plate = object_type == ObjectEnum::ExtractionPlate;

    // 1. At least 1 grid cell of the proposed footprint must be within the build area
    if !build_area.overlaps_footprint(pos_x, pos_z, size_x, size_z) {
        return Err("No cells within build area");
    }

    // 2. All tiles under the footprint must be Visible to the placing player
    if !footprint_is_visible(fog_map, player_id, pos_x, pos_z, size_x, size_z) {
        return Err("Tile not visible");
    }

    // Special case: ExtractionPlate placement
    if is_extraction_plate {
        // Must be 1x1, placed on a SpaceCrystalsPatch without existing plate
        let mut found_patch = false;
        for (patch_pos, patch) in patches.iter() {
            if patch_pos.x == pos_x && patch_pos.z == pos_z {
                if patch.has_plate {
                    return Err("Patch already has a plate");
                }
                found_patch = true;
                break;
            }
        }
        if !found_patch {
            return Err("No Space Crystal Patch at this location");
        }
        return Ok(());
    }

    // 2. All grid cells must be on Buildable tiles
    for dx in 0..size_x as i32 {
        for dz in 0..size_z as i32 {
            let check_x = pos_x + dx;
            let check_z = pos_z + dz;

            let mut found_tile = false;
            for (tile_pos, properties) in tiles.iter() {
                if tile_pos.x == check_x && tile_pos.z == check_z {
                    if !properties.buildable {
                        return Err("Tile is not buildable");
                    }
                    found_tile = true;
                    break;
                }
            }
            if !found_tile {
                return Err("No tile at position");
            }

            // 3. No overlap with existing structures
            for (struct_pos, _) in structures.iter() {
                if struct_pos.x == check_x && struct_pos.z == check_z {
                    return Err("Overlaps existing structure");
                }
            }
        }
    }

    Ok(())
}

/// Check if a worker-built structure can be placed at the given position.
/// Unlike `can_place_building()`, this does NOT check:
/// - Build area (Syndicate doesn't use GdoBuildArea)
/// - Fog of war visibility (worker is physically present)
/// Returns Ok(()) if placement is valid, Err(reason) if not.
pub fn can_worker_place_structure(
    pos_x: i32,
    pos_z: i32,
    size_x: u32,
    size_z: u32,
    tiles: &Query<(&GridPosition, &TilePreset), With<Tile>>,
    structures: &Query<(&GridPosition, &crate::game::types::StructureInstance)>,
) -> Result<(), &'static str> {
    for dx in 0..size_x as i32 {
        for dz in 0..size_z as i32 {
            let check_x = pos_x + dx;
            let check_z = pos_z + dz;

            // Check tile exists and is buildable
            let mut found_tile = false;
            for (tile_pos, properties) in tiles.iter() {
                if tile_pos.x == check_x && tile_pos.z == check_z {
                    if !properties.buildable {
                        return Err("Tile is not buildable");
                    }
                    found_tile = true;
                    break;
                }
            }
            if !found_tile {
                return Err("No tile at position");
            }

            // Check no overlap with existing structures
            for (struct_pos, _) in structures.iter() {
                if struct_pos.x == check_x && struct_pos.z == check_z {
                    return Err("Overlaps existing structure");
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================
    // Viewport offset tests
    // =====================================================

    #[test]
    fn viewport_offset_no_viewport_returns_zero() {
        let camera = Camera::default();
        let offset = viewport_offset(&camera, 1.0);
        assert_eq!(offset, Vec2::ZERO);
    }

    #[test]
    fn viewport_offset_with_viewport_at_origin() {
        use bevy::render::camera::Viewport;
        let mut camera = Camera::default();
        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(1280, 688),
            ..default()
        });
        let offset = viewport_offset(&camera, 1.0);
        assert_eq!(offset, Vec2::ZERO);
    }

    #[test]
    fn viewport_offset_with_top_bar_offset() {
        use bevy::render::camera::Viewport;
        let mut camera = Camera::default();
        // Simulates HUD top bar of 32px at scale_factor 1.0
        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 32),
            physical_size: UVec2::new(1280, 688),
            ..default()
        });
        let offset = viewport_offset(&camera, 1.0);
        assert!((offset.x - 0.0).abs() < 0.001);
        assert!((offset.y - 32.0).abs() < 0.001);
    }

    #[test]
    fn viewport_offset_with_scale_factor() {
        use bevy::render::camera::Viewport;
        let mut camera = Camera::default();
        // Physical 64px offset at scale_factor 2.0 → 32 logical pixels
        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 64),
            physical_size: UVec2::new(2560, 1376),
            ..default()
        });
        let offset = viewport_offset(&camera, 2.0);
        assert!((offset.x - 0.0).abs() < 0.001);
        assert!((offset.y - 32.0).abs() < 0.001);
    }

    #[test]
    fn viewport_offset_with_x_and_y() {
        use bevy::render::camera::Viewport;
        let mut camera = Camera::default();
        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(100, 50),
            physical_size: UVec2::new(1080, 670),
            ..default()
        });
        let offset = viewport_offset(&camera, 1.0);
        assert!((offset.x - 100.0).abs() < 0.001);
        assert!((offset.y - 50.0).abs() < 0.001);
    }

    #[test]
    fn viewport_offset_with_fractional_scale() {
        use bevy::render::camera::Viewport;
        let mut camera = Camera::default();
        // Physical 48px at scale 1.5 → 32 logical
        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 48),
            physical_size: UVec2::new(1920, 1032),
            ..default()
        });
        let offset = viewport_offset(&camera, 1.5);
        assert!((offset.x - 0.0).abs() < 0.001);
        assert!((offset.y - 32.0).abs() < 0.001);
    }

    #[test]
    fn cursor_adjustment_math_correctness() {
        // Verify that subtracting viewport offset converts window-space to viewport-space
        let cursor_window_space = Vec2::new(640.0, 392.0);
        let vp_offset = Vec2::new(0.0, 32.0); // 32px top bar
        let cursor_viewport_space = cursor_window_space - vp_offset;
        assert!((cursor_viewport_space.x - 640.0).abs() < 0.001);
        assert!((cursor_viewport_space.y - 360.0).abs() < 0.001);
    }

    #[test]
    fn cursor_at_viewport_top_edge() {
        // Cursor at y=32 (just at the top of the viewport) → viewport y=0
        let cursor_window = Vec2::new(500.0, 32.0);
        let offset = Vec2::new(0.0, 32.0);
        let result = cursor_window - offset;
        assert!((result.y - 0.0).abs() < 0.001);
    }

    // =====================================================
    // Screen-space hit tests
    // =====================================================

    #[test]
    fn screen_space_hit_test_exact_center() {
        let cursor = Vec2::new(100.0, 200.0);
        let screen = Vec2::new(100.0, 200.0);
        let result = screen_space_hit_test(cursor, screen, 25.0);
        assert!(result.is_some());
        assert!((result.unwrap() - 0.0).abs() < 0.001);
    }

    #[test]
    fn screen_space_hit_test_within_radius() {
        let cursor = Vec2::new(110.0, 200.0);
        let screen = Vec2::new(100.0, 200.0);
        let result = screen_space_hit_test(cursor, screen, 25.0);
        assert!(result.is_some());
        assert!((result.unwrap() - 10.0).abs() < 0.001);
    }

    #[test]
    fn screen_space_hit_test_at_boundary() {
        // Distance is exactly 25.0 — should miss (< not <=)
        let cursor = Vec2::new(125.0, 200.0);
        let screen = Vec2::new(100.0, 200.0);
        let result = screen_space_hit_test(cursor, screen, 25.0);
        assert!(result.is_none());
    }

    #[test]
    fn screen_space_hit_test_outside_radius() {
        let cursor = Vec2::new(130.0, 200.0);
        let screen = Vec2::new(100.0, 200.0);
        let result = screen_space_hit_test(cursor, screen, 25.0);
        assert!(result.is_none());
    }

    #[test]
    fn screen_space_hit_test_diagonal() {
        // Diagonal distance: sqrt(15^2 + 20^2) = sqrt(225 + 400) = sqrt(625) = 25.0
        // Exactly at boundary — should miss
        let cursor = Vec2::new(115.0, 220.0);
        let screen = Vec2::new(100.0, 200.0);
        let result = screen_space_hit_test(cursor, screen, 25.0);
        assert!(result.is_none());

        // Slightly inside
        let cursor_inside = Vec2::new(114.0, 219.0);
        let result_inside = screen_space_hit_test(cursor_inside, screen, 25.0);
        assert!(result_inside.is_some());
    }

    #[test]
    fn screen_space_hit_test_returns_distance() {
        let cursor = Vec2::new(103.0, 204.0);
        let screen = Vec2::new(100.0, 200.0);
        let result = screen_space_hit_test(cursor, screen, 25.0);
        assert!(result.is_some());
        let expected = (3.0_f32.powi(2) + 4.0_f32.powi(2)).sqrt(); // 5.0
        assert!((result.unwrap() - expected).abs() < 0.001);
    }

    #[test]
    fn world_to_grid_basic() {
        let pos = Vec3::new(0.5, 0.0, 0.5);
        let (x, z) = world_to_grid(pos, 1.0);
        assert_eq!(x, 32);
        assert_eq!(z, 32);
    }

    #[test]
    fn grid_to_world_basic() {
        let world = grid_to_world(32, 32, 1.0);
        assert!((world.x - 0.5).abs() < 0.001);
        assert!((world.z - 0.5).abs() < 0.001);
    }

    // =====================================================
    // Box-selection priority tests
    // =====================================================

    fn make_candidate(entity: Entity, x: f32, y: f32) -> BoxCandidate {
        BoxCandidate {
            entity,
            screen_pos: Vec2::new(x, y),
        }
    }

    #[test]
    fn closest_to_center_single_candidate() {
        let entity = Entity::from_raw(1);
        let candidates = vec![make_candidate(entity, 100.0, 100.0)];
        let result = closest_to_center(&candidates, Vec2::new(50.0, 50.0));
        assert_eq!(result, entity);
    }

    #[test]
    fn closest_to_center_picks_nearest() {
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let e3 = Entity::from_raw(3);
        let candidates = vec![
            make_candidate(e1, 100.0, 100.0),  // dist to center(50,50) = ~70.7
            make_candidate(e2, 55.0, 55.0),     // dist to center(50,50) = ~7.07
            make_candidate(e3, 200.0, 200.0),   // dist to center(50,50) = ~212.1
        ];
        let result = closest_to_center(&candidates, Vec2::new(50.0, 50.0));
        assert_eq!(result, e2);
    }

    #[test]
    fn closest_to_center_exact_center() {
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let candidates = vec![
            make_candidate(e1, 50.0, 50.0),  // exactly at center
            make_candidate(e2, 60.0, 60.0),
        ];
        let result = closest_to_center(&candidates, Vec2::new(50.0, 50.0));
        assert_eq!(result, e1);
    }

    #[test]
    fn selection_tier_own_unit() {
        assert_eq!(
            classify_selection_tier(true, false, true, false),
            SelectionTier::OwnUnits
        );
    }

    #[test]
    fn selection_tier_own_structure() {
        assert_eq!(
            classify_selection_tier(true, false, false, true),
            SelectionTier::OwnStructures
        );
    }

    #[test]
    fn selection_tier_enemy_unit() {
        assert_eq!(
            classify_selection_tier(false, false, true, false),
            SelectionTier::EnemyUnits
        );
    }

    #[test]
    fn selection_tier_enemy_structure() {
        assert_eq!(
            classify_selection_tier(false, false, false, true),
            SelectionTier::EnemyStructures
        );
    }

    #[test]
    fn selection_tier_neutral() {
        assert_eq!(
            classify_selection_tier(false, true, false, false),
            SelectionTier::Neutrals
        );
    }

    #[test]
    fn selection_tier_neutral_with_no_markers() {
        // Entity that is not a unit, not a structure, not owned — neutral
        assert_eq!(
            classify_selection_tier(false, true, true, false),
            SelectionTier::Neutrals
        );
    }

    #[test]
    fn selection_tier_owned_entity_no_markers_is_own_structure_fallthrough() {
        // Owned but neither unit nor structure — falls to Neutrals (shouldn't happen in practice)
        assert_eq!(
            classify_selection_tier(true, false, false, false),
            SelectionTier::Neutrals
        );
    }

    // === can_place_expansion tests ===

    #[test]
    fn can_place_expansion_valid_within_area() {
        use crate::game::types::{TunnelArea, TunnelTier};
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        let result = can_place_expansion(10, 10, 2, 2, &area, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn can_place_expansion_outside_area() {
        use crate::game::types::{TunnelArea, TunnelTier};
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        let result = can_place_expansion(50, 50, 2, 2, &area, &[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Expansion does not fit within Tunnel Area");
    }

    #[test]
    fn can_place_expansion_overlaps_existing() {
        use crate::game::types::{TunnelArea, TunnelTier};
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        let existing = vec![(10, 10), (11, 10), (10, 11), (11, 11)];
        let result = can_place_expansion(10, 10, 2, 2, &area, &existing);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Overlaps existing expansion");
    }

    #[test]
    fn can_place_expansion_adjacent_to_existing_ok() {
        use crate::game::types::{TunnelArea, TunnelTier};
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        let existing = vec![(10, 10), (11, 10)];
        // Place expansion at (12, 10) — no overlap
        let result = can_place_expansion(12, 10, 2, 2, &area, &existing);
        assert!(result.is_ok());
    }

    #[test]
    fn can_place_expansion_partial_outside_area() {
        use crate::game::types::{TunnelArea, TunnelTier};
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        // T1 area: 7..17 on both axes. 2x2 at (16,16) extends to 17 which is excluded
        let result = can_place_expansion(16, 16, 2, 2, &area, &[]);
        assert!(result.is_err());
    }

    // =====================================================
    // Vision center offset tests
    // =====================================================

    #[test]
    fn vision_center_no_object_returns_grid_pos() {
        let (cx, cz) = vision_center(30, 30, None);
        assert_eq!(cx, 30);
        assert_eq!(cz, 30);
    }

    #[test]
    fn vision_center_1x1_unit_no_offset() {
        let (cx, cz) = vision_center(15, 20, Some((1, 1)));
        // 1/2 = 0 in integer division, so no offset
        assert_eq!(cx, 15);
        assert_eq!(cz, 20);
    }

    #[test]
    fn vision_center_4x4_structure_offset_by_2() {
        // DeploymentCenter: size (4,4) at grid pos (30,30)
        // Center should be (30+2, 30+2) = (32, 32)
        let (cx, cz) = vision_center(30, 30, Some((4, 4)));
        assert_eq!(cx, 32);
        assert_eq!(cz, 32);
    }

    #[test]
    fn vision_center_2x2_structure_offset_by_1() {
        // PowerPlant: size (2,2) at grid pos (10,10)
        // Center should be (10+1, 10+1) = (11, 11)
        let (cx, cz) = vision_center(10, 10, Some((2, 2)));
        assert_eq!(cx, 11);
        assert_eq!(cz, 11);
    }

    #[test]
    fn vision_center_2x3_structure_asymmetric() {
        // Barracks: size (2,3) at grid pos (5,5)
        // Center should be (5+1, 5+1) = (6, 6) — integer division of 3/2 = 1
        let (cx, cz) = vision_center(5, 5, Some((2, 3)));
        assert_eq!(cx, 6);
        assert_eq!(cz, 6);
    }

    #[test]
    fn vision_center_3x3_structure_offset_by_1() {
        // ExtractionFacility: size (3,3) at grid pos (20,20)
        // Center should be (20+1, 20+1) = (21, 21) — integer division of 3/2 = 1
        let (cx, cz) = vision_center(20, 20, Some((3, 3)));
        assert_eq!(cx, 21);
        assert_eq!(cz, 21);
    }

    #[test]
    fn vision_center_at_origin() {
        let (cx, cz) = vision_center(0, 0, Some((4, 4)));
        assert_eq!(cx, 2);
        assert_eq!(cz, 2);
    }

    #[test]
    fn vision_center_negative_grid_pos() {
        let (cx, cz) = vision_center(-5, -5, Some((4, 4)));
        assert_eq!(cx, -3);
        assert_eq!(cz, -3);
    }

    // =====================================================
    // Footprint visibility tests
    // =====================================================

    fn make_fog_map_all_visible(width: u32, height: u32, player_id: u8) -> FogOfWarMap {
        let mut fog = FogOfWarMap::new(width, height);
        for x in 0..width as i32 {
            for z in 0..height as i32 {
                fog.set(player_id, x, z, VisibilityStateEnum::Visible);
            }
        }
        fog
    }

    #[test]
    fn footprint_visible_all_tiles_visible() {
        let fog = make_fog_map_all_visible(64, 64, 0);
        assert!(footprint_is_visible(&fog, 0, 10, 10, 2, 2));
    }

    #[test]
    fn footprint_visible_1x1_visible() {
        let fog = make_fog_map_all_visible(64, 64, 0);
        assert!(footprint_is_visible(&fog, 0, 30, 30, 1, 1));
    }

    #[test]
    fn footprint_not_visible_all_unexplored() {
        let fog = FogOfWarMap::new(64, 64);
        // No player map — all tiles default to Unexplored
        assert!(!footprint_is_visible(&fog, 0, 10, 10, 2, 2));
    }

    #[test]
    fn footprint_not_visible_one_tile_explored() {
        let mut fog = make_fog_map_all_visible(64, 64, 0);
        // Set one tile in the footprint to Explored
        fog.set(0, 11, 10, VisibilityStateEnum::Explored);
        assert!(!footprint_is_visible(&fog, 0, 10, 10, 2, 2));
    }

    #[test]
    fn footprint_not_visible_one_tile_unexplored() {
        let mut fog = make_fog_map_all_visible(64, 64, 0);
        fog.set(0, 10, 11, VisibilityStateEnum::Unexplored);
        assert!(!footprint_is_visible(&fog, 0, 10, 10, 2, 2));
    }

    #[test]
    fn footprint_not_visible_wrong_player() {
        let fog = make_fog_map_all_visible(64, 64, 0);
        // Player 1 has no map — all Unexplored
        assert!(!footprint_is_visible(&fog, 1, 10, 10, 2, 2));
    }

    #[test]
    fn footprint_visible_4x4_all_visible() {
        let fog = make_fog_map_all_visible(64, 64, 0);
        assert!(footprint_is_visible(&fog, 0, 20, 20, 4, 4));
    }

    #[test]
    fn footprint_not_visible_4x4_corner_explored() {
        let mut fog = make_fog_map_all_visible(64, 64, 0);
        // Bottom-right corner of a 4x4 at (20,20) is (23,23)
        fog.set(0, 23, 23, VisibilityStateEnum::Explored);
        assert!(!footprint_is_visible(&fog, 0, 20, 20, 4, 4));
    }

    #[test]
    fn footprint_visible_at_map_edge() {
        let fog = make_fog_map_all_visible(64, 64, 0);
        // 2x2 at (62, 62) — tiles (62,62), (63,62), (62,63), (63,63) all within bounds
        assert!(footprint_is_visible(&fog, 0, 62, 62, 2, 2));
    }

    #[test]
    fn footprint_not_visible_out_of_bounds() {
        let fog = make_fog_map_all_visible(64, 64, 0);
        // 2x2 at (63, 63) — tile (64,64) is out of bounds → Unexplored
        assert!(!footprint_is_visible(&fog, 0, 63, 63, 2, 2));
    }

    #[test]
    fn footprint_not_visible_negative_coords() {
        let fog = make_fog_map_all_visible(64, 64, 0);
        // Negative coords are out of bounds → Unexplored
        assert!(!footprint_is_visible(&fog, 0, -1, -1, 1, 1));
    }

    #[test]
    fn footprint_visible_different_player() {
        let mut fog = FogOfWarMap::new(64, 64);
        // Only player 2 has visibility
        for x in 0..64_i32 {
            for z in 0..64_i32 {
                fog.set(2, x, z, VisibilityStateEnum::Visible);
            }
        }
        assert!(footprint_is_visible(&fog, 2, 10, 10, 3, 3));
        // Player 0 still can't see
        assert!(!footprint_is_visible(&fog, 0, 10, 10, 3, 3));
    }

    // =====================================================
    // can_worker_place_structure tests
    // =====================================================

    use bevy::ecs::system::RunSystemOnce;
    use crate::game::types::StructureInstance;

    fn spawn_buildable_tiles(world: &mut World, positions: &[(i32, i32)]) {
        for &(x, z) in positions {
            world.spawn((
                GridPosition { x, z },
                TilePreset {
                    value: TilePresetEnum::Plane,
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

    fn spawn_non_buildable_tile(world: &mut World, x: i32, z: i32) {
        world.spawn((
            GridPosition { x, z },
            TilePreset {
                value: TilePresetEnum::Mountain,
                name: "Mountain".to_string(),
                texture: None,
                buildable: false,
                traversible: false,
                rugged: false,
                drillable: false,
                recruitable: false,
            },
            Tile,
        ));
    }

    fn spawn_structure_at(world: &mut World, x: i32, z: i32) {
        world.spawn((
            GridPosition { x, z },
            StructureInstance::default(),
        ));
    }

    #[test]
    fn worker_place_valid_on_buildable_tiles() {
        let mut world = World::new();
        // Spawn a 2x2 grid of buildable tiles at (10,10)
        spawn_buildable_tiles(&mut world, &[
            (10, 10), (11, 10), (10, 11), (11, 11),
        ]);

        let result = world.run_system_once(|
            tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
            structures: Query<(&GridPosition, &StructureInstance)>,
        | {
            can_worker_place_structure(10, 10, 2, 2, &tiles, &structures)
        });
        assert!(result.is_ok());
    }

    #[test]
    fn worker_place_fails_on_non_buildable_tile() {
        let mut world = World::new();
        spawn_buildable_tiles(&mut world, &[(10, 10), (11, 10), (10, 11)]);
        spawn_non_buildable_tile(&mut world, 11, 11);

        let result = world.run_system_once(|
            tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
            structures: Query<(&GridPosition, &StructureInstance)>,
        | {
            can_worker_place_structure(10, 10, 2, 2, &tiles, &structures)
        });
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Tile is not buildable");
    }

    #[test]
    fn worker_place_fails_on_missing_tile() {
        let mut world = World::new();
        // Only spawn 3 of 4 needed tiles
        spawn_buildable_tiles(&mut world, &[(10, 10), (11, 10), (10, 11)]);

        let result = world.run_system_once(|
            tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
            structures: Query<(&GridPosition, &StructureInstance)>,
        | {
            can_worker_place_structure(10, 10, 2, 2, &tiles, &structures)
        });
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No tile at position");
    }

    #[test]
    fn worker_place_fails_on_structure_overlap() {
        let mut world = World::new();
        spawn_buildable_tiles(&mut world, &[
            (10, 10), (11, 10), (10, 11), (11, 11),
        ]);
        spawn_structure_at(&mut world, 11, 11);

        let result = world.run_system_once(|
            tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
            structures: Query<(&GridPosition, &StructureInstance)>,
        | {
            can_worker_place_structure(10, 10, 2, 2, &tiles, &structures)
        });
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Overlaps existing structure");
    }

    #[test]
    fn worker_place_valid_1x1_on_single_tile() {
        let mut world = World::new();
        spawn_buildable_tiles(&mut world, &[(5, 5)]);

        let result = world.run_system_once(|
            tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
            structures: Query<(&GridPosition, &StructureInstance)>,
        | {
            can_worker_place_structure(5, 5, 1, 1, &tiles, &structures)
        });
        assert!(result.is_ok());
    }

    #[test]
    fn worker_place_no_visibility_check() {
        // This test confirms worker placement does NOT check visibility.
        // We place valid tiles and don't set up any fog of war.
        // If visibility were checked, this would fail (no FogOfWarMap param).
        let mut world = World::new();
        spawn_buildable_tiles(&mut world, &[(20, 20)]);

        let result = world.run_system_once(|
            tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
            structures: Query<(&GridPosition, &StructureInstance)>,
        | {
            can_worker_place_structure(20, 20, 1, 1, &tiles, &structures)
        });
        assert!(result.is_ok(), "Worker placement should not require visibility");
    }

    #[test]
    fn worker_place_no_build_area_check() {
        // This test confirms worker placement does NOT check build area.
        // We place valid tiles and don't set up any GdoBuildArea.
        let mut world = World::new();
        spawn_buildable_tiles(&mut world, &[(30, 30)]);

        let result = world.run_system_once(|
            tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
            structures: Query<(&GridPosition, &StructureInstance)>,
        | {
            can_worker_place_structure(30, 30, 1, 1, &tiles, &structures)
        });
        assert!(result.is_ok(), "Worker placement should not require build area");
    }
}
