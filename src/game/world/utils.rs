use bevy::prelude::*;
use crate::types::{GridPosition, ObjectEnum, StructureRotation, SymmetryTypeEnum, VisibilityStateEnum};
use crate::game::types::StructureInstance;
use crate::game::types::objects::ObjectInstance;
use super::types::*;

// NOTE: viewport_offset() and cursor_pos_in_viewport() were removed.
// In Bevy 0.17, viewport_to_world() and world_to_viewport() handle viewport offsets
// internally (using logical_viewport_rect() which includes target_rect.min).
// All callers should use window.cursor_position() directly.

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

/// Compute the spawn offset for the B-side exit of a structure.
/// Returns (dx, dz) offset from the structure's grid_pos origin.
///
/// For structures with a 'B' label in their symmetry (e.g. Barracks ABAC),
/// the spawn point is placed 1 tile beyond the B-side edge.
/// For structures without a 'B' label (e.g. AAAA symmetry like Supply Tower),
/// the spawn point follows the rotation: R0→South, R90→East, R180→North, R270→West.
pub fn spawn_side_offset(
    object: ObjectEnum,
    si: &StructureInstance,
) -> (i32, i32) {
    let st = object.structure_type();
    let (base_w, base_h) = object.object_type().size;
    let (w, h) = rotated_building_size(base_w, base_h, &si.rotation);

    let symmetry = st.map(|s| s.symmetry_type).unwrap_or(SymmetryTypeEnum::AAAA);
    let labels = si.oriented_labels(symmetry);

    // labels: [N=0, E=1, S=2, W=3]
    let b_side = labels.iter().position(|&c| c == 'B');

    match b_side {
        Some(0) => (w as i32 / 2, -1),            // North: 1 tile above origin
        Some(1) => (w as i32, h as i32 / 2),       // East: 1 tile beyond width
        Some(2) => (w as i32 / 2, h as i32),       // South: 1 tile beyond height
        Some(3) => (-1, h as i32 / 2),             // West: 1 tile left of origin
        _ => {
            // No B side (e.g. AAAA) — use rotation to pick the "default" spawn side
            match si.rotation {
                StructureRotation::R0   => (w as i32 / 2, h as i32),    // South
                StructureRotation::R90  => (w as i32, h as i32 / 2),    // East
                StructureRotation::R180 => (w as i32 / 2, -1),          // North
                StructureRotation::R270 => (-1, h as i32 / 2),          // West
            }
        }
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
    structures: &Query<(&GridPosition, &crate::game::types::StructureInstance, &ObjectInstance)>,
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

            // 3. No overlap with existing structures (check full footprints)
            for (struct_pos, _si, obj) in structures.iter() {
                let (sx, sz) = obj.object_type.object_type().size;
                if check_x >= struct_pos.x && check_x < struct_pos.x + sx as i32
                    && check_z >= struct_pos.z && check_z < struct_pos.z + sz as i32
                {
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
    structures: &Query<(&GridPosition, &crate::game::types::StructureInstance, &ObjectInstance)>,
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

            // Check no overlap with existing structures (full footprint check)
            for (struct_pos, _si, obj) in structures.iter() {
                let (sx, sz) = obj.object_type.object_type().size;
                if check_x >= struct_pos.x && check_x < struct_pos.x + sx as i32
                    && check_z >= struct_pos.z && check_z < struct_pos.z + sz as i32
                {
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
        let entity = Entity::from_raw_u32(1).unwrap();
        let candidates = vec![make_candidate(entity, 100.0, 100.0)];
        let result = closest_to_center(&candidates, Vec2::new(50.0, 50.0));
        assert_eq!(result, entity);
    }

    #[test]
    fn closest_to_center_picks_nearest() {
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        let e3 = Entity::from_raw_u32(3).unwrap();
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
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
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
        // Spawn a 1x1 structure (ExtractionPlate) for overlap testing
        world.spawn((
            GridPosition { x, z },
            StructureInstance::default(),
            ObjectInstance::destructible(ObjectEnum::ExtractionPlate, 100.0),
        ));
    }

    fn spawn_structure_at_type(world: &mut World, x: i32, z: i32, object_type: ObjectEnum) {
        world.spawn((
            GridPosition { x, z },
            StructureInstance::default(),
            ObjectInstance::destructible(object_type, 100.0),
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
            structures: Query<(&GridPosition, &StructureInstance, &ObjectInstance)>,
        | {
            can_worker_place_structure(10, 10, 2, 2, &tiles, &structures)
        }).unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn worker_place_fails_on_non_buildable_tile() {
        let mut world = World::new();
        spawn_buildable_tiles(&mut world, &[(10, 10), (11, 10), (10, 11)]);
        spawn_non_buildable_tile(&mut world, 11, 11);

        let result = world.run_system_once(|
            tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
            structures: Query<(&GridPosition, &StructureInstance, &ObjectInstance)>,
        | {
            can_worker_place_structure(10, 10, 2, 2, &tiles, &structures)
        }).unwrap();
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
            structures: Query<(&GridPosition, &StructureInstance, &ObjectInstance)>,
        | {
            can_worker_place_structure(10, 10, 2, 2, &tiles, &structures)
        }).unwrap();
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
            structures: Query<(&GridPosition, &StructureInstance, &ObjectInstance)>,
        | {
            can_worker_place_structure(10, 10, 2, 2, &tiles, &structures)
        }).unwrap();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Overlaps existing structure");
    }

    #[test]
    fn worker_place_valid_1x1_on_single_tile() {
        let mut world = World::new();
        spawn_buildable_tiles(&mut world, &[(5, 5)]);

        let result = world.run_system_once(|
            tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
            structures: Query<(&GridPosition, &StructureInstance, &ObjectInstance)>,
        | {
            can_worker_place_structure(5, 5, 1, 1, &tiles, &structures)
        }).unwrap();
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
            structures: Query<(&GridPosition, &StructureInstance, &ObjectInstance)>,
        | {
            can_worker_place_structure(20, 20, 1, 1, &tiles, &structures)
        }).unwrap();
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
            structures: Query<(&GridPosition, &StructureInstance, &ObjectInstance)>,
        | {
            can_worker_place_structure(30, 30, 1, 1, &tiles, &structures)
        }).unwrap();
        assert!(result.is_ok(), "Worker placement should not require build area");
    }

    #[test]
    fn worker_place_fails_on_multi_cell_structure_overlap() {
        let mut world = World::new();
        // Spawn tiles covering 10..14 in both x and z (5x5 grid)
        let mut tiles_to_spawn = Vec::new();
        for x in 10..15 {
            for z in 10..15 {
                tiles_to_spawn.push((x, z));
            }
        }
        spawn_buildable_tiles(&mut world, &tiles_to_spawn);
        // Spawn a 4x4 Tunnel at (10, 10) — footprint covers (10,10) to (13,13)
        spawn_structure_at_type(&mut world, 10, 10, ObjectEnum::Tunnel);

        // Try placing a 2x2 at (12, 12) — overlaps with Tunnel's footprint
        let result = world.run_system_once(|
            tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
            structures: Query<(&GridPosition, &StructureInstance, &ObjectInstance)>,
        | {
            can_worker_place_structure(12, 12, 2, 2, &tiles, &structures)
        }).unwrap();
        assert!(result.is_err(), "Should detect overlap with Tunnel's full 4x4 footprint");
        assert_eq!(result.unwrap_err(), "Overlaps existing structure");
    }

    #[test]
    fn worker_place_succeeds_adjacent_to_multi_cell_structure() {
        let mut world = World::new();
        // Spawn tiles covering 10..16 in both x and z
        let mut tiles_to_spawn = Vec::new();
        for x in 10..16 {
            for z in 10..16 {
                tiles_to_spawn.push((x, z));
            }
        }
        spawn_buildable_tiles(&mut world, &tiles_to_spawn);
        // Spawn a 4x4 Tunnel at (10, 10) — footprint covers (10,10) to (13,13)
        spawn_structure_at_type(&mut world, 10, 10, ObjectEnum::Tunnel);

        // Place a 2x2 at (14, 10) — adjacent but not overlapping
        let result = world.run_system_once(|
            tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
            structures: Query<(&GridPosition, &StructureInstance, &ObjectInstance)>,
        | {
            can_worker_place_structure(14, 10, 2, 2, &tiles, &structures)
        }).unwrap();
        assert!(result.is_ok(), "Should succeed when adjacent but not overlapping");
    }

    // =====================================================
    // spawn_side_offset tests
    // =====================================================

    #[test]
    fn barracks_r0_b_side_is_south() {
        // Barracks 3x2, ABAC. R0: oriented_labels = [A,B,A,C] → B at East(1)
        // Wait — let's verify: base ABAC = [N=A, E=B, S=A, W=C], R0 shift=0
        // So B is at East(1) → offset (w, h/2) = (3, 1)
        let si = StructureInstance::new(StructureRotation::R0, false, false);
        let (dx, dz) = spawn_side_offset(ObjectEnum::Barracks, &si);
        assert_eq!(dx, 3); // Beyond the 3-wide building
        assert_eq!(dz, 1); // Midpoint of 2-deep building
    }

    #[test]
    fn barracks_r90_b_side_rotates() {
        // R90: shift right by 1 → labels become [C, A, B, A] → B at South(2)
        // Size rotated: (2, 3). Offset: (w/2, h) = (1, 3)
        let si = StructureInstance::new(StructureRotation::R90, false, false);
        let (dx, dz) = spawn_side_offset(ObjectEnum::Barracks, &si);
        assert_eq!(dx, 1); // Midpoint of rotated 2-wide
        assert_eq!(dz, 3); // Beyond the rotated 3-deep
    }

    #[test]
    fn barracks_r180_b_side_rotates() {
        // R180: shift right by 2 → labels become [A, C, A, B] → B at West(3)
        // Size: (3, 2). Offset: (-1, h/2) = (-1, 1)
        let si = StructureInstance::new(StructureRotation::R180, false, false);
        let (dx, dz) = spawn_side_offset(ObjectEnum::Barracks, &si);
        assert_eq!(dx, -1);
        assert_eq!(dz, 1);
    }

    #[test]
    fn barracks_r270_b_side_rotates() {
        // R270: shift right by 3 → labels become [B, A, C, A] → B at North(0)
        // Size rotated: (2, 3). Offset: (w/2, -1) = (1, -1)
        let si = StructureInstance::new(StructureRotation::R270, false, false);
        let (dx, dz) = spawn_side_offset(ObjectEnum::Barracks, &si);
        assert_eq!(dx, 1);
        assert_eq!(dz, -1);
    }

    #[test]
    fn supply_tower_r0_default_south() {
        // SupplyTower 3x3, AAAA — no B side. R0 defaults to South.
        // Offset: (w/2, h) = (1, 3)
        let si = StructureInstance::new(StructureRotation::R0, false, false);
        let (dx, dz) = spawn_side_offset(ObjectEnum::SupplyTower, &si);
        assert_eq!(dx, 1);
        assert_eq!(dz, 3);
    }

    #[test]
    fn supply_tower_r90_default_east() {
        // AAAA, R90 → East. Size still (3,3). Offset: (w, h/2) = (3, 1)
        let si = StructureInstance::new(StructureRotation::R90, false, false);
        let (dx, dz) = spawn_side_offset(ObjectEnum::SupplyTower, &si);
        assert_eq!(dx, 3);
        assert_eq!(dz, 1);
    }

    #[test]
    fn supply_tower_r180_default_north() {
        // AAAA, R180 → North. Offset: (w/2, -1) = (1, -1)
        let si = StructureInstance::new(StructureRotation::R180, false, false);
        let (dx, dz) = spawn_side_offset(ObjectEnum::SupplyTower, &si);
        assert_eq!(dx, 1);
        assert_eq!(dz, -1);
    }

    #[test]
    fn supply_tower_r270_default_west() {
        // AAAA, R270 → West. Offset: (-1, h/2) = (-1, 1)
        let si = StructureInstance::new(StructureRotation::R270, false, false);
        let (dx, dz) = spawn_side_offset(ObjectEnum::SupplyTower, &si);
        assert_eq!(dx, -1);
        assert_eq!(dz, 1);
    }

    #[test]
    fn barracks_flipped_horizontal_swaps_ew() {
        // R0 + flip_h: labels [A,B,A,C] → flip E↔W → [A,C,A,B] → B at West(3)
        // Size: (3,2). Offset: (-1, h/2) = (-1, 1)
        let si = StructureInstance::new(StructureRotation::R0, true, false);
        let (dx, dz) = spawn_side_offset(ObjectEnum::Barracks, &si);
        assert_eq!(dx, -1);
        assert_eq!(dz, 1);
    }

    #[test]
    fn deployment_center_aaaa_r0_south() {
        // DC is 4x4, AAAA. R0 → South. Offset: (w/2, h) = (2, 4)
        let si = StructureInstance::new(StructureRotation::R0, false, false);
        let (dx, dz) = spawn_side_offset(ObjectEnum::DeploymentCenter, &si);
        assert_eq!(dx, 2);
        assert_eq!(dz, 4);
    }

    #[test]
    fn spawn_offset_outside_building_footprint() {
        // Verify that for all Barracks rotations, the spawn tile is outside the footprint
        for rot in [StructureRotation::R0, StructureRotation::R90, StructureRotation::R180, StructureRotation::R270] {
            let si = StructureInstance::new(rot, false, false);
            let (dx, dz) = spawn_side_offset(ObjectEnum::Barracks, &si);
            let (w, h) = rotated_building_size(3, 2, &rot);
            // The spawn point should be outside the [0..w) x [0..h) footprint
            let inside_x = dx >= 0 && dx < w as i32;
            let inside_z = dz >= 0 && dz < h as i32;
            assert!(!(inside_x && inside_z),
                "Spawn point ({dx}, {dz}) is inside footprint ({w}x{h}) for rotation {rot:?}");
        }
    }
}
