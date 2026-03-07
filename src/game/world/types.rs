#![allow(dead_code)]
use std::collections::{HashMap, HashSet};
use bevy::prelude::*;
use crate::types::{GridPosition, DomainEnum, ObjectEnum, VisibilityStateEnum};

/// Resource storing grid metadata
#[derive(Resource)]
pub struct GridMap {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
}

impl Default for GridMap {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
            cell_size: 1.0,
        }
    }
}

impl GridMap {
    /// Get the half-width offset for world-to-grid conversion
    pub fn half_width(&self) -> f32 {
        self.width as f32 / 2.0
    }

    /// Get the half-height offset for world-to-grid conversion
    pub fn half_height(&self) -> f32 {
        self.height as f32 / 2.0
    }
}

/// Tile terrain preset types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum TilePresetEnum {
    Plane,
    RuggedTerrain,
    Cliff,
    Mountain,
    Water,
}

impl TilePresetEnum {
    /// Get the preset properties for this tile type
    pub fn properties(&self) -> TilePreset {
        match self {
            TilePresetEnum::Plane => TilePreset {
                value: *self,
                name: "Plane".to_string(),
                texture: None,
                buildable: true,
                traversible: true,
                rugged: false,
                drillable: true,
                recruitable: true,
            },
            TilePresetEnum::RuggedTerrain => TilePreset {
                value: *self,
                name: "Rugged Terrain".to_string(),
                texture: None,
                buildable: false,
                traversible: true,
                rugged: true,
                drillable: true,
                recruitable: true,
            },
            TilePresetEnum::Cliff => TilePreset {
                value: *self,
                name: "Cliff".to_string(),
                texture: None,
                buildable: false,
                traversible: false,
                rugged: false,
                drillable: true,
                recruitable: true,
            },
            TilePresetEnum::Mountain => TilePreset {
                value: *self,
                name: "Mountain".to_string(),
                texture: None,
                buildable: false,
                traversible: false,
                rugged: false,
                drillable: false,
                recruitable: true,
            },
            TilePresetEnum::Water => TilePreset {
                value: *self,
                name: "Water".to_string(),
                texture: None,
                buildable: false,
                traversible: false,
                rugged: false,
                drillable: false,
                recruitable: false,
            },
        }
    }

    /// Get visual color for this tile type
    pub fn color(&self) -> Color {
        match self {
            TilePresetEnum::Plane => Color::srgb(0.5, 0.7, 0.4),
            TilePresetEnum::RuggedTerrain => Color::srgb(0.6, 0.4, 0.2),
            TilePresetEnum::Cliff => Color::srgb(0.5, 0.5, 0.5),
            TilePresetEnum::Mountain => Color::srgb(0.3, 0.3, 0.3),
            TilePresetEnum::Water => Color::srgb(0.2, 0.4, 0.7),
        }
    }
}

/// Tile preset properties — defines the gameplay characteristics of a tile type
#[derive(Component, Clone, Debug)]
pub struct TilePreset {
    pub value: TilePresetEnum,
    pub name: String,
    pub texture: Option<String>,
    pub buildable: bool,
    pub traversible: bool,
    pub rugged: bool,
    pub drillable: bool,
    pub recruitable: bool,
}

/// Maximum allowed tile elevation (0-16 inclusive)
pub const MAX_ELEVATION: u8 = 16;

/// Component representing a placed tile on the map with location and elevation
#[derive(Component, Clone, Debug)]
pub struct TilePlacement {
    pub tile_type: TilePresetEnum,
    pub location: GridPosition,
    pub elevation: u8,
}

impl TilePlacement {
    /// Create a new TilePlacement with validated elevation.
    /// Returns `Err` if elevation exceeds `MAX_ELEVATION` (16).
    pub fn new(tile_type: TilePresetEnum, location: GridPosition, elevation: u8) -> Result<Self, String> {
        if elevation > MAX_ELEVATION {
            return Err(format!(
                "Elevation {} exceeds maximum allowed elevation {}",
                elevation, MAX_ELEVATION
            ));
        }
        Ok(Self {
            tile_type,
            location,
            elevation,
        })
    }
}

/// Component to mark tile entities
#[derive(Component)]
pub struct Tile;

/// Component for Space Crystal Patch resource nodes
#[derive(Component)]
pub struct SpaceCrystalPatch {
    pub remaining_amount: u32,
    pub initial_amount: u32,
    /// Whether an extraction plate is currently on this patch
    pub has_plate: bool,
}

/// Component for Supply Delivery Station resource nodes
#[derive(Component)]
pub struct SupplyDeliveryStation {
    pub delivery_size: u32,
    pub delivery_interval: f32,
    pub current_supplies: u32,
    pub time_until_next_delivery: f32,
}

/// Component marking the drag-box UI element
#[derive(Component)]
pub struct DragBoxUI;

/// Resource tracking selection state for drag-box
#[derive(Resource, Default)]
pub struct SelectionState {
    pub drag_start: Option<Vec2>,
    pub is_dragging: bool,
}

/// Component for selection indicator visual
#[derive(Component)]
pub struct SelectionIndicator;

/// Resource tracking the GDO build area — grid cells where buildings can be placed
/// Uses Chebyshev distance (square extension) from building footprints
#[derive(Resource, Clone, Debug)]
pub struct GdoBuildArea {
    /// Set of grid coordinates within the build area
    pub cells: HashSet<(i32, i32)>,
}

impl Default for GdoBuildArea {
    fn default() -> Self {
        Self {
            cells: HashSet::new(),
        }
    }
}

impl GdoBuildArea {
    /// Check if a grid cell is within the build area
    pub fn contains(&self, x: i32, z: i32) -> bool {
        self.cells.contains(&(x, z))
    }

    /// Check if any cell of a building footprint overlaps the build area
    pub fn overlaps_footprint(&self, pos_x: i32, pos_z: i32, size_x: u32, size_z: u32) -> bool {
        for dx in 0..size_x as i32 {
            for dz in 0..size_z as i32 {
                if self.contains(pos_x + dx, pos_z + dz) {
                    return true;
                }
            }
        }
        false
    }
}

/// Resource providing O(1) elevation lookups by grid position.
/// Populated during startup after `spawn_grid`.
#[derive(Resource, Clone, Debug)]
pub struct ElevationMap {
    pub elevations: HashMap<(i32, i32), u8>,
}

impl Default for ElevationMap {
    fn default() -> Self {
        Self {
            elevations: HashMap::new(),
        }
    }
}

impl ElevationMap {
    /// Look up the terrain elevation at a grid position.
    /// Returns 0 if the position is not in the map.
    pub fn get(&self, x: i32, z: i32) -> u8 {
        self.elevations.get(&(x, z)).copied().unwrap_or(0)
    }

    /// Insert an elevation value for a grid position.
    pub fn insert(&mut self, x: i32, z: i32, elevation: u8) {
        self.elevations.insert((x, z), elevation);
    }
}

/// Returns the elevation modifier between source and target.
/// Returns +1 if source is higher, -1 if source is lower, 0 if equal.
/// Returns 0 if either entity is in the Air domain (air units are exempt).
/// Underground units use the surface tile elevation above them.
pub fn elevation_modifier(
    source_domain: DomainEnum,
    source_elevation: u8,
    target_domain: DomainEnum,
    target_elevation: u8,
) -> i32 {
    // Air units are exempt from elevation modifiers
    if source_domain == DomainEnum::Air || target_domain == DomainEnum::Air {
        return 0;
    }

    // Compare elevations (underground units already use surface elevation via lookup)
    match source_elevation.cmp(&target_elevation) {
        std::cmp::Ordering::Greater => 1,
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
    }
}

/// Resource tracking the last control group recall for double-tap detection.
/// Used to implement "Recall and Center" — double-pressing a control group key
/// centers the camera on the group's centroid.
#[derive(Resource, Default)]
pub struct LastRecallState {
    /// Which group index was last recalled
    pub group_index: Option<usize>,
    /// Time of the last recall (seconds since startup)
    pub timestamp: f64,
}

impl LastRecallState {
    /// Maximum time between two presses to count as a double-tap (seconds)
    pub const DOUBLE_TAP_THRESHOLD: f64 = 0.3;

    /// Check if this recall qualifies as a double-tap of the same group
    pub fn is_double_tap(&self, group_index: usize, current_time: f64) -> bool {
        self.group_index == Some(group_index)
            && (current_time - self.timestamp) <= Self::DOUBLE_TAP_THRESHOLD
    }
}

/// Per-player fog of war state for all tiles on the map.
/// Each player has their own visibility map tracking Unexplored/Explored/Visible per tile.
#[derive(Resource)]
pub struct FogOfWarMap {
    pub width: u32,
    pub height: u32,
    /// Map from player_id to a flat array of VisibilityStateEnum (row-major: index = z * width + x)
    pub player_maps: HashMap<u8, Vec<VisibilityStateEnum>>,
}

impl FogOfWarMap {
    /// Create a new FogOfWarMap with the given dimensions and no player maps
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            player_maps: HashMap::new(),
        }
    }

    /// Lazily create a player's visibility map filled with Unexplored if not yet present
    pub fn ensure_player(&mut self, player_id: u8) {
        self.player_maps.entry(player_id).or_insert_with(|| {
            vec![VisibilityStateEnum::Unexplored; (self.width * self.height) as usize]
        });
    }

    /// Get the visibility state for a player at a grid position.
    /// Returns Unexplored if the player has no map or position is out of bounds.
    pub fn get(&self, player_id: u8, x: i32, z: i32) -> VisibilityStateEnum {
        if x < 0 || z < 0 || x >= self.width as i32 || z >= self.height as i32 {
            return VisibilityStateEnum::Unexplored;
        }
        self.player_maps
            .get(&player_id)
            .map(|map| map[(z as u32 * self.width + x as u32) as usize])
            .unwrap_or(VisibilityStateEnum::Unexplored)
    }

    /// Set the visibility state for a player at a grid position.
    /// No-op if position is out of bounds.
    pub fn set(&mut self, player_id: u8, x: i32, z: i32, state: VisibilityStateEnum) {
        if x < 0 || z < 0 || x >= self.width as i32 || z >= self.height as i32 {
            return;
        }
        self.ensure_player(player_id);
        if let Some(map) = self.player_maps.get_mut(&player_id) {
            map[(z as u32 * self.width + x as u32) as usize] = state;
        }
    }

    /// Returns an iterator over all grid coordinates within Euclidean distance `range`
    /// of the center point (cx, cz), clamped to grid bounds.
    /// Uses circular (Euclidean) distance: dx*dx + dz*dz <= range*range.
    pub fn tiles_in_sight_range(&self, cx: i32, cz: i32, range: u32) -> Vec<(i32, i32)> {
        let r = range as i32;
        let r_sq = (range * range) as i64;
        let mut tiles = Vec::new();
        for dx in -r..=r {
            for dz in -r..=r {
                let dist_sq = (dx as i64) * (dx as i64) + (dz as i64) * (dz as i64);
                if dist_sq <= r_sq {
                    let x = cx + dx;
                    let z = cz + dz;
                    if x >= 0 && z >= 0 && x < self.width as i32 && z < self.height as i32 {
                        tiles.push((x, z));
                    }
                }
            }
        }
        tiles
    }
}

/// Information about a structure last seen on a tile (for Explored fog state)
#[derive(Clone, Debug)]
pub struct LastKnownStructure {
    pub object_type: ObjectEnum,
    pub hp_fraction: f32,
}

/// Tracks the last-known state of structures on explored tiles per player.
/// When a tile transitions from Visible to Explored, structures are snapshotted here.
#[derive(Resource, Default)]
pub struct LastKnownStructures {
    /// Map from (player_id, grid_x, grid_z) to last-known structure info
    pub entries: HashMap<(u8, i32, i32), LastKnownStructure>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_presets() -> Vec<TilePresetEnum> {
        vec![
            TilePresetEnum::Plane,
            TilePresetEnum::RuggedTerrain,
            TilePresetEnum::Cliff,
            TilePresetEnum::Mountain,
            TilePresetEnum::Water,
        ]
    }

    #[test]
    fn test_tile_preset_enum_has_five_variants() {
        assert_eq!(all_presets().len(), 5);
    }

    #[test]
    fn test_plane_properties() {
        let p = TilePresetEnum::Plane.properties();
        assert_eq!(p.value, TilePresetEnum::Plane);
        assert_eq!(p.name, "Plane");
        assert!(p.buildable);
        assert!(p.traversible);
        assert!(!p.rugged);
        assert!(p.drillable);
        assert!(p.recruitable);
        assert!(p.texture.is_none());
    }

    #[test]
    fn test_rugged_terrain_properties() {
        let p = TilePresetEnum::RuggedTerrain.properties();
        assert_eq!(p.value, TilePresetEnum::RuggedTerrain);
        assert_eq!(p.name, "Rugged Terrain");
        assert!(!p.buildable);
        assert!(p.traversible);
        assert!(p.rugged);
        assert!(p.drillable);
        assert!(p.recruitable);
    }

    #[test]
    fn test_cliff_properties() {
        let p = TilePresetEnum::Cliff.properties();
        assert_eq!(p.value, TilePresetEnum::Cliff);
        assert_eq!(p.name, "Cliff");
        assert!(!p.buildable);
        assert!(!p.traversible);
        assert!(!p.rugged);
        assert!(p.drillable);
        assert!(p.recruitable);
    }

    #[test]
    fn test_mountain_properties() {
        let p = TilePresetEnum::Mountain.properties();
        assert_eq!(p.value, TilePresetEnum::Mountain);
        assert_eq!(p.name, "Mountain");
        assert!(!p.buildable);
        assert!(!p.traversible);
        assert!(!p.rugged);
        assert!(!p.drillable);
        assert!(p.recruitable);
    }

    #[test]
    fn test_water_properties() {
        let p = TilePresetEnum::Water.properties();
        assert_eq!(p.value, TilePresetEnum::Water);
        assert_eq!(p.name, "Water");
        assert!(!p.buildable);
        assert!(!p.traversible);
        assert!(!p.rugged);
        assert!(!p.drillable);
        assert!(!p.recruitable);
    }

    #[test]
    fn test_all_presets_have_none_texture() {
        for preset in all_presets() {
            let p = preset.properties();
            assert!(p.texture.is_none(), "{:?} should have texture: None", preset);
        }
    }

    #[test]
    fn test_all_presets_produce_valid_properties() {
        for preset in all_presets() {
            let p = preset.properties();
            assert_eq!(p.value, preset);
            assert!(!p.name.is_empty());
        }
    }

    #[test]
    fn test_tile_preset_enum_color_returns_distinct_colors() {
        let colors: Vec<Color> = all_presets().iter().map(|p| p.color()).collect();
        // Each preset should have a unique color
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(colors[i], colors[j], "Presets {} and {} have the same color", i, j);
            }
        }
    }

    #[test]
    fn test_only_plane_is_buildable() {
        for preset in all_presets() {
            let p = preset.properties();
            if preset == TilePresetEnum::Plane {
                assert!(p.buildable, "Plane should be buildable");
            } else {
                assert!(!p.buildable, "{:?} should not be buildable", preset);
            }
        }
    }

    #[test]
    fn test_traversible_presets() {
        // Only Plane and RuggedTerrain are traversible
        let traversible: Vec<TilePresetEnum> = all_presets()
            .into_iter()
            .filter(|p| p.properties().traversible)
            .collect();
        assert_eq!(traversible, vec![TilePresetEnum::Plane, TilePresetEnum::RuggedTerrain]);
    }

    #[test]
    fn test_drillable_presets() {
        // Plane, RuggedTerrain, and Cliff are drillable
        let drillable: Vec<TilePresetEnum> = all_presets()
            .into_iter()
            .filter(|p| p.properties().drillable)
            .collect();
        assert_eq!(drillable, vec![TilePresetEnum::Plane, TilePresetEnum::RuggedTerrain, TilePresetEnum::Cliff]);
    }

    #[test]
    fn test_recruitable_presets() {
        // All except Water are recruitable
        let recruitable: Vec<TilePresetEnum> = all_presets()
            .into_iter()
            .filter(|p| p.properties().recruitable)
            .collect();
        assert_eq!(recruitable, vec![
            TilePresetEnum::Plane,
            TilePresetEnum::RuggedTerrain,
            TilePresetEnum::Cliff,
            TilePresetEnum::Mountain,
        ]);
    }

    // --- SpaceCrystalPatch component tests ---

    #[test]
    fn test_space_crystal_patch_creation() {
        let patch = SpaceCrystalPatch {
            remaining_amount: 5000,
            initial_amount: 5000,
            has_plate: false,
        };
        assert_eq!(patch.remaining_amount, 5000);
        assert_eq!(patch.initial_amount, 5000);
        assert!(!patch.has_plate);
    }

    #[test]
    fn test_space_crystal_patch_depletion() {
        let mut patch = SpaceCrystalPatch {
            remaining_amount: 100,
            initial_amount: 100,
            has_plate: true,
        };
        patch.remaining_amount = 0;
        assert_eq!(patch.remaining_amount, 0);
        assert!(patch.has_plate);
    }

    #[test]
    fn test_space_crystal_patch_partial_mining() {
        let mut patch = SpaceCrystalPatch {
            remaining_amount: 1000,
            initial_amount: 1000,
            has_plate: true,
        };
        patch.remaining_amount -= 250;
        assert_eq!(patch.remaining_amount, 750);
        assert_eq!(patch.initial_amount, 1000);
    }

    #[test]
    fn test_space_crystal_patch_plate_attachment() {
        let mut patch = SpaceCrystalPatch {
            remaining_amount: 2000,
            initial_amount: 2000,
            has_plate: false,
        };
        assert!(!patch.has_plate);
        patch.has_plate = true;
        assert!(patch.has_plate);
    }

    // --- SupplyDeliveryStation component tests ---

    #[test]
    fn test_supply_delivery_station_creation() {
        let sds = SupplyDeliveryStation {
            delivery_size: 100,
            delivery_interval: 60.0,
            current_supplies: 100,
            time_until_next_delivery: 60.0,
        };
        assert_eq!(sds.delivery_size, 100);
        assert_eq!(sds.delivery_interval, 60.0);
        assert_eq!(sds.current_supplies, 100);
        assert_eq!(sds.time_until_next_delivery, 60.0);
    }

    #[test]
    fn test_supply_delivery_station_empty_countdown() {
        let mut sds = SupplyDeliveryStation {
            delivery_size: 100,
            delivery_interval: 60.0,
            current_supplies: 0,
            time_until_next_delivery: 60.0,
        };
        // Simulate countdown
        sds.time_until_next_delivery -= 30.0;
        assert_eq!(sds.time_until_next_delivery, 30.0);
        assert_eq!(sds.current_supplies, 0);
    }

    #[test]
    fn test_supply_delivery_station_refill() {
        let mut sds = SupplyDeliveryStation {
            delivery_size: 150,
            delivery_interval: 90.0,
            current_supplies: 0,
            time_until_next_delivery: 0.0,
        };
        // Simulate delivery
        if sds.time_until_next_delivery <= 0.0 {
            sds.current_supplies = sds.delivery_size;
            sds.time_until_next_delivery = sds.delivery_interval;
        }
        assert_eq!(sds.current_supplies, 150);
        assert_eq!(sds.time_until_next_delivery, 90.0);
    }

    #[test]
    fn test_supply_delivery_station_collect_supplies() {
        let mut sds = SupplyDeliveryStation {
            delivery_size: 200,
            delivery_interval: 45.0,
            current_supplies: 200,
            time_until_next_delivery: 45.0,
        };
        // Collect all supplies
        let collected = sds.current_supplies;
        sds.current_supplies = 0;
        assert_eq!(collected, 200);
        assert_eq!(sds.current_supplies, 0);
    }

    // --- TilePlacement tests ---

    fn default_location() -> GridPosition {
        GridPosition { x: 0, z: 0 }
    }

    #[test]
    fn test_tile_placement_new_elevation_zero() {
        let tp = TilePlacement::new(TilePresetEnum::Plane, GridPosition { x: 0, z: 0 }, 0);
        assert!(tp.is_ok());
        let tp = tp.unwrap();
        assert_eq!(tp.tile_type, TilePresetEnum::Plane);
        assert_eq!(tp.location.x, 0);
        assert_eq!(tp.location.z, 0);
        assert_eq!(tp.elevation, 0);
    }

    #[test]
    fn test_tile_placement_new_elevation_max() {
        let tp = TilePlacement::new(TilePresetEnum::Mountain, GridPosition { x: 5, z: 3 }, 16);
        assert!(tp.is_ok());
        let tp = tp.unwrap();
        assert_eq!(tp.tile_type, TilePresetEnum::Mountain);
        assert_eq!(tp.location.x, 5);
        assert_eq!(tp.location.z, 3);
        assert_eq!(tp.elevation, 16);
    }

    #[test]
    fn test_tile_placement_new_elevation_exceeds_max() {
        let tp = TilePlacement::new(TilePresetEnum::Plane, default_location(), 17);
        assert!(tp.is_err());
        assert!(tp.unwrap_err().contains("17"));
    }

    #[test]
    fn test_tile_placement_new_elevation_255() {
        let tp = TilePlacement::new(TilePresetEnum::Plane, default_location(), 255);
        assert!(tp.is_err());
    }

    #[test]
    fn test_tile_placement_new_all_presets() {
        for preset in all_presets() {
            let tp = TilePlacement::new(preset, default_location(), 8);
            assert!(tp.is_ok(), "TilePlacement::new should succeed for {:?}", preset);
            assert_eq!(tp.unwrap().tile_type, preset);
        }
    }

    #[test]
    fn test_tile_placement_new_mid_elevation() {
        let tp = TilePlacement::new(TilePresetEnum::Cliff, GridPosition { x: 10, z: -5 }, 8);
        assert!(tp.is_ok());
        assert_eq!(tp.unwrap().elevation, 8);
    }

    #[test]
    fn test_tile_placement_max_elevation_constant() {
        assert_eq!(MAX_ELEVATION, 16);
    }

    #[test]
    fn test_tile_placement_boundary_elevations() {
        // All elevations 0..=16 should be valid
        for e in 0..=MAX_ELEVATION {
            assert!(TilePlacement::new(TilePresetEnum::Plane, default_location(), e).is_ok(),
                "Elevation {} should be valid", e);
        }
        // 17 and above should be invalid
        assert!(TilePlacement::new(TilePresetEnum::Plane, default_location(), MAX_ELEVATION + 1).is_err());
    }

    #[test]
    fn test_only_rugged_terrain_is_rugged() {
        for preset in all_presets() {
            let p = preset.properties();
            if preset == TilePresetEnum::RuggedTerrain {
                assert!(p.rugged, "RuggedTerrain should be rugged");
            } else {
                assert!(!p.rugged, "{:?} should not be rugged", preset);
            }
        }
    }

    // --- ElevationMap tests ---

    #[test]
    fn test_elevation_map_default_is_empty() {
        let map = ElevationMap::default();
        assert!(map.elevations.is_empty());
    }

    #[test]
    fn test_elevation_map_insert_and_get() {
        let mut map = ElevationMap::default();
        map.insert(5, 10, 8);
        assert_eq!(map.get(5, 10), 8);
    }

    #[test]
    fn test_elevation_map_get_missing_returns_zero() {
        let map = ElevationMap::default();
        assert_eq!(map.get(99, 99), 0);
    }

    // --- elevation_modifier tests ---

    #[test]
    fn test_elevation_modifier_higher_source() {
        assert_eq!(elevation_modifier(DomainEnum::Ground, 5, DomainEnum::Ground, 3), 1);
    }

    #[test]
    fn test_elevation_modifier_lower_source() {
        assert_eq!(elevation_modifier(DomainEnum::Ground, 3, DomainEnum::Ground, 5), -1);
    }

    #[test]
    fn test_elevation_modifier_equal() {
        assert_eq!(elevation_modifier(DomainEnum::Ground, 5, DomainEnum::Ground, 5), 0);
    }

    #[test]
    fn test_elevation_modifier_air_source_exempt() {
        assert_eq!(elevation_modifier(DomainEnum::Air, 5, DomainEnum::Ground, 3), 0);
    }

    #[test]
    fn test_elevation_modifier_air_target_exempt() {
        assert_eq!(elevation_modifier(DomainEnum::Ground, 5, DomainEnum::Air, 3), 0);
    }

    #[test]
    fn test_elevation_modifier_both_air() {
        assert_eq!(elevation_modifier(DomainEnum::Air, 5, DomainEnum::Air, 3), 0);
    }

    #[test]
    fn test_elevation_modifier_underground_uses_surface_elevation() {
        // Underground units use the surface tile elevation (looked up before calling this fn)
        assert_eq!(elevation_modifier(DomainEnum::Underground, 5, DomainEnum::Ground, 3), 1);
    }

    #[test]
    fn test_elevation_modifier_binary_not_proportional() {
        // Large elevation difference still only +1/-1
        assert_eq!(elevation_modifier(DomainEnum::Ground, 1, DomainEnum::Ground, 15), -1);
        assert_eq!(elevation_modifier(DomainEnum::Ground, 16, DomainEnum::Ground, 0), 1);
    }

    #[test]
    fn test_elevation_modifier_underground_vs_underground() {
        assert_eq!(elevation_modifier(DomainEnum::Underground, 10, DomainEnum::Underground, 5), 1);
        assert_eq!(elevation_modifier(DomainEnum::Underground, 5, DomainEnum::Underground, 10), -1);
    }

    #[test]
    fn test_elevation_modifier_ground_vs_underground() {
        assert_eq!(elevation_modifier(DomainEnum::Ground, 5, DomainEnum::Underground, 3), 1);
    }

    // === LastRecallState tests ===

    #[test]
    fn test_last_recall_state_default() {
        let state = LastRecallState::default();
        assert_eq!(state.group_index, None);
        assert_eq!(state.timestamp, 0.0);
    }

    #[test]
    fn test_last_recall_state_double_tap_same_group_within_threshold() {
        let state = LastRecallState {
            group_index: Some(3),
            timestamp: 10.0,
        };
        assert!(state.is_double_tap(3, 10.2));
    }

    #[test]
    fn test_last_recall_state_double_tap_same_group_at_threshold() {
        let state = LastRecallState {
            group_index: Some(3),
            timestamp: 0.0,
        };
        // Use 0.0 as base to avoid float precision issues
        assert!(state.is_double_tap(3, 0.3)); // exactly at threshold
    }

    #[test]
    fn test_last_recall_state_not_double_tap_different_group() {
        let state = LastRecallState {
            group_index: Some(3),
            timestamp: 10.0,
        };
        assert!(!state.is_double_tap(5, 10.2));
    }

    #[test]
    fn test_last_recall_state_not_double_tap_too_late() {
        let state = LastRecallState {
            group_index: Some(3),
            timestamp: 10.0,
        };
        assert!(!state.is_double_tap(3, 10.5)); // beyond 0.3s threshold
    }

    #[test]
    fn test_last_recall_state_not_double_tap_no_previous() {
        let state = LastRecallState::default();
        assert!(!state.is_double_tap(0, 1.0));
    }

    #[test]
    fn test_last_recall_state_threshold_constant() {
        assert_eq!(LastRecallState::DOUBLE_TAP_THRESHOLD, 0.3);
    }

    // === FogOfWarMap tests ===

    #[test]
    fn fog_map_new_creates_empty() {
        let map = FogOfWarMap::new(64, 64);
        assert_eq!(map.width, 64);
        assert_eq!(map.height, 64);
        assert!(map.player_maps.is_empty());
    }

    #[test]
    fn fog_map_default_state_is_unexplored() {
        let map = FogOfWarMap::new(10, 10);
        assert_eq!(map.get(0, 5, 5), VisibilityStateEnum::Unexplored);
    }

    #[test]
    fn fog_map_ensure_player_creates_all_unexplored() {
        let mut map = FogOfWarMap::new(4, 4);
        map.ensure_player(0);
        assert!(map.player_maps.contains_key(&0));
        assert_eq!(map.player_maps[&0].len(), 16); // 4x4
        for &state in &map.player_maps[&0] {
            assert_eq!(state, VisibilityStateEnum::Unexplored);
        }
    }

    #[test]
    fn fog_map_set_and_get() {
        let mut map = FogOfWarMap::new(10, 10);
        map.set(0, 3, 5, VisibilityStateEnum::Visible);
        assert_eq!(map.get(0, 3, 5), VisibilityStateEnum::Visible);
        assert_eq!(map.get(0, 3, 4), VisibilityStateEnum::Unexplored);
    }

    #[test]
    fn fog_map_out_of_bounds_returns_unexplored() {
        let map = FogOfWarMap::new(10, 10);
        assert_eq!(map.get(0, -1, 0), VisibilityStateEnum::Unexplored);
        assert_eq!(map.get(0, 0, -1), VisibilityStateEnum::Unexplored);
        assert_eq!(map.get(0, 10, 0), VisibilityStateEnum::Unexplored);
        assert_eq!(map.get(0, 0, 10), VisibilityStateEnum::Unexplored);
    }

    #[test]
    fn fog_map_set_out_of_bounds_is_noop() {
        let mut map = FogOfWarMap::new(10, 10);
        map.set(0, -1, 0, VisibilityStateEnum::Visible);
        map.set(0, 10, 0, VisibilityStateEnum::Visible);
        // Should not panic, no state changed
        assert_eq!(map.get(0, 0, 0), VisibilityStateEnum::Unexplored);
    }

    #[test]
    fn fog_map_multiple_players_independent() {
        let mut map = FogOfWarMap::new(10, 10);
        map.set(0, 5, 5, VisibilityStateEnum::Visible);
        map.set(1, 5, 5, VisibilityStateEnum::Explored);
        assert_eq!(map.get(0, 5, 5), VisibilityStateEnum::Visible);
        assert_eq!(map.get(1, 5, 5), VisibilityStateEnum::Explored);
    }

    #[test]
    fn fog_map_tiles_in_sight_range_zero() {
        let map = FogOfWarMap::new(10, 10);
        let tiles = map.tiles_in_sight_range(5, 5, 0);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0], (5, 5));
    }

    #[test]
    fn fog_map_tiles_in_sight_range_one() {
        let map = FogOfWarMap::new(10, 10);
        let tiles = map.tiles_in_sight_range(5, 5, 1);
        // Range 1 with Euclidean: center + 4 adjacent = 5 tiles
        assert_eq!(tiles.len(), 5);
        assert!(tiles.contains(&(5, 5)));
        assert!(tiles.contains(&(4, 5)));
        assert!(tiles.contains(&(6, 5)));
        assert!(tiles.contains(&(5, 4)));
        assert!(tiles.contains(&(5, 6)));
    }

    #[test]
    fn fog_map_tiles_in_sight_range_excludes_corners_for_small_radius() {
        let map = FogOfWarMap::new(20, 20);
        let tiles = map.tiles_in_sight_range(10, 10, 1);
        // Diagonal distance = sqrt(2) ≈ 1.41 > 1, so corners excluded
        assert!(!tiles.contains(&(9, 9)));
        assert!(!tiles.contains(&(11, 11)));
    }

    #[test]
    fn fog_map_tiles_in_sight_range_clamped_to_bounds() {
        let map = FogOfWarMap::new(10, 10);
        let tiles = map.tiles_in_sight_range(0, 0, 3);
        // Should not include negative coordinates
        for &(x, z) in &tiles {
            assert!(x >= 0 && z >= 0);
            assert!(x < 10 && z < 10);
        }
    }

    #[test]
    fn fog_map_tiles_in_sight_range_circular() {
        let map = FogOfWarMap::new(20, 20);
        let tiles = map.tiles_in_sight_range(10, 10, 3);
        // All tiles should be within Euclidean distance 3
        for &(x, z) in &tiles {
            let dx = (x - 10) as f64;
            let dz = (z - 10) as f64;
            let dist_sq = dx * dx + dz * dz;
            assert!(dist_sq <= 9.0, "Tile ({},{}) is outside range 3", x, z);
        }
    }

    #[test]
    fn fog_map_state_transitions() {
        let mut map = FogOfWarMap::new(10, 10);
        // Start unexplored
        assert_eq!(map.get(0, 5, 5), VisibilityStateEnum::Unexplored);
        // Become visible
        map.set(0, 5, 5, VisibilityStateEnum::Visible);
        assert_eq!(map.get(0, 5, 5), VisibilityStateEnum::Visible);
        // Transition to explored
        map.set(0, 5, 5, VisibilityStateEnum::Explored);
        assert_eq!(map.get(0, 5, 5), VisibilityStateEnum::Explored);
        // Back to visible
        map.set(0, 5, 5, VisibilityStateEnum::Visible);
        assert_eq!(map.get(0, 5, 5), VisibilityStateEnum::Visible);
    }

    #[test]
    fn fog_map_sight_range_5_tile_count() {
        let map = FogOfWarMap::new(64, 64);
        let tiles = map.tiles_in_sight_range(32, 32, 5);
        // Euclidean circle with r=5: count of tiles where dx^2 + dz^2 <= 25
        // This is 81 tiles in the square, minus corners outside the circle
        assert!(tiles.len() > 60); // reasonable lower bound
        assert!(tiles.len() < 100); // reasonable upper bound
    }

    // === LastKnownStructures tests ===

    #[test]
    fn last_known_structures_default_empty() {
        let lks = LastKnownStructures::default();
        assert!(lks.entries.is_empty());
    }

    #[test]
    fn last_known_structures_insert_and_retrieve() {
        let mut lks = LastKnownStructures::default();
        lks.entries.insert(
            (0, 10, 10),
            LastKnownStructure {
                object_type: ObjectEnum::PowerPlant,
                hp_fraction: 0.75,
            },
        );
        let entry = lks.entries.get(&(0, 10, 10)).unwrap();
        assert_eq!(entry.object_type, ObjectEnum::PowerPlant);
        assert!((entry.hp_fraction - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn last_known_structures_per_player() {
        let mut lks = LastKnownStructures::default();
        lks.entries.insert(
            (0, 5, 5),
            LastKnownStructure { object_type: ObjectEnum::Barracks, hp_fraction: 1.0 },
        );
        lks.entries.insert(
            (1, 5, 5),
            LastKnownStructure { object_type: ObjectEnum::Barracks, hp_fraction: 0.5 },
        );
        assert!((lks.entries[&(0, 5, 5)].hp_fraction - 1.0).abs() < f32::EPSILON);
        assert!((lks.entries[&(1, 5, 5)].hp_fraction - 0.5).abs() < f32::EPSILON);
    }

    // === SightRange component tests (in shared/types.rs) ===
    // Tested here because ObjectType.sight_range values are defined in game types

    #[test]
    fn sight_range_peacekeeper() {
        assert_eq!(ObjectEnum::Peacekeeper.object_type().sight_range, 5);
    }

    #[test]
    fn sight_range_deployment_center() {
        assert_eq!(ObjectEnum::DeploymentCenter.object_type().sight_range, 6);
    }

    #[test]
    fn sight_range_power_plant() {
        assert_eq!(ObjectEnum::PowerPlant.object_type().sight_range, 3);
    }

    #[test]
    fn sight_range_barracks() {
        assert_eq!(ObjectEnum::Barracks.object_type().sight_range, 4);
    }

    #[test]
    fn sight_range_extraction_facility() {
        assert_eq!(ObjectEnum::ExtractionFacility.object_type().sight_range, 3);
    }

    #[test]
    fn sight_range_extraction_plate_is_zero() {
        assert_eq!(ObjectEnum::ExtractionPlate.object_type().sight_range, 0);
    }

    #[test]
    fn sight_range_space_crystals_patch_is_zero() {
        assert_eq!(ObjectEnum::SpaceCrystalsPatch.object_type().sight_range, 0);
    }

    #[test]
    fn sight_range_supply_delivery_station_is_zero() {
        assert_eq!(ObjectEnum::SupplyDeliveryStation.object_type().sight_range, 0);
    }
}
