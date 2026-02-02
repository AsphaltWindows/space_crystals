use bevy::prelude::*;
use bevy::window::PrimaryWindow;

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
            width: 20,
            height: 20,
            cell_size: 1.0,
        }
    }
}

/// Tile terrain types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum TileType {
    Plane,
    RuggedTerrain,
    Cliff,
    Mountain,
    Water,
}

impl TileType {
    /// Get default properties for this tile type
    pub fn default_properties(&self) -> TileProperties {
        match self {
            TileType::Plane => TileProperties {
                buildable: true,
                traversible: true,
                drillable: true,
                rugged: false,
                recruitable: true,
            },
            TileType::RuggedTerrain => TileProperties {
                buildable: false,
                traversible: true,
                drillable: true,
                rugged: true,
                recruitable: true,
            },
            TileType::Cliff => TileProperties {
                buildable: false,
                traversible: false,
                drillable: true,
                rugged: false,
                recruitable: true,
            },
            TileType::Mountain => TileProperties {
                buildable: false,
                traversible: false,
                drillable: false,
                rugged: false,
                recruitable: true,
            },
            TileType::Water => TileProperties {
                buildable: false,
                traversible: false,
                drillable: false,
                rugged: false,
                recruitable: false,
            },
        }
    }

    /// Get visual color for this tile type
    pub fn color(&self) -> Color {
        match self {
            TileType::Plane => Color::srgb(0.5, 0.7, 0.4),           // Light green
            TileType::RuggedTerrain => Color::srgb(0.6, 0.4, 0.2),  // Brown
            TileType::Cliff => Color::srgb(0.5, 0.5, 0.5),          // Gray
            TileType::Mountain => Color::srgb(0.3, 0.3, 0.3),       // Dark gray
            TileType::Water => Color::srgb(0.2, 0.4, 0.7),          // Blue
        }
    }
}

/// Tile gameplay properties
#[derive(Component, Clone, Copy, Debug)]
pub struct TileProperties {
    pub buildable: bool,
    pub traversible: bool,
    pub drillable: bool,
    pub rugged: bool,
    pub recruitable: bool,
}

/// Component to mark tile entities
#[derive(Component)]
pub struct Tile;

/// Component storing grid position
#[derive(Component, Debug, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub z: i32,
}

/// Plugin for map-related systems
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GridMap::default())
            .add_systems(Startup, spawn_grid)
            .add_systems(Update, tile_hover_system);
    }
}

/// Helper function to convert world coordinates to grid coordinates
pub fn world_to_grid(world_pos: Vec3, cell_size: f32) -> (i32, i32) {
    let x = (world_pos.x / cell_size).round() as i32;
    let z = (world_pos.z / cell_size).round() as i32;
    (x, z)
}

/// Helper function to convert grid coordinates to world coordinates
pub fn grid_to_world(grid_x: i32, grid_z: i32, cell_size: f32) -> Vec3 {
    Vec3::new(
        grid_x as f32 * cell_size,
        0.0,
        grid_z as f32 * cell_size,
    )
}

/// Determine tile type based on grid position (creates a test pattern)
fn determine_tile_type(x: u32, z: u32) -> TileType {
    // Create a deterministic test pattern with all tile types
    if x == 5 && z == 5 {
        TileType::Water
    } else if x == 10 && z == 10 {
        TileType::Mountain
    } else if x == 15 && z == 5 {
        TileType::Cliff
    } else if (x + z) % 7 == 0 {
        TileType::RuggedTerrain
    } else if x < 3 || x > 16 {
        TileType::Water
    } else if z < 3 || z > 16 {
        TileType::RuggedTerrain
    } else {
        TileType::Plane
    }
}

/// System to spawn the grid of tiles
pub fn spawn_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid: Res<GridMap>,
) {
    let tile_mesh = meshes.add(Plane3d::default().mesh().size(grid.cell_size, grid.cell_size));

    // Calculate grid offset to center it at origin
    let offset_x = -(grid.width as f32 * grid.cell_size) / 2.0 + grid.cell_size / 2.0;
    let offset_z = -(grid.height as f32 * grid.cell_size) / 2.0 + grid.cell_size / 2.0;

    let mut tile_type_counts = std::collections::HashMap::new();

    for x in 0..grid.width {
        for z in 0..grid.height {
            let grid_pos = GridPosition {
                x: x as i32,
                z: z as i32,
            };

            let tile_type = determine_tile_type(x, z);
            let properties = tile_type.default_properties();

            *tile_type_counts.entry(tile_type).or_insert(0) += 1;

            let tile_material = materials.add(StandardMaterial {
                base_color: tile_type.color(),
                ..default()
            });

            let world_x = offset_x + x as f32 * grid.cell_size;
            let world_z = offset_z + z as f32 * grid.cell_size;

            commands.spawn((
                PbrBundle {
                    mesh: tile_mesh.clone(),
                    material: tile_material,
                    transform: Transform::from_xyz(world_x, 0.0, world_z),
                    ..default()
                },
                Tile,
                tile_type,
                properties,
                grid_pos,
            ));
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

/// System to display tile info when hovering with mouse
fn tile_hover_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    tiles: Query<(&TileType, &TileProperties, &GridPosition), With<Tile>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let window = windows.single();
    let (camera, camera_transform) = cameras.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            // Calculate intersection with y=0 plane (where tiles are)
            if ray.direction.y.abs() > 0.001 {
                let t = -ray.origin.y / ray.direction.y;
                if t > 0.0 {
                    let intersection = ray.origin + ray.direction * t;

                    // Convert to grid coordinates
                    let (grid_x, grid_z) = world_to_grid(intersection, 1.0);

                    // Find tile at this position
                    for (tile_type, properties, grid_pos) in tiles.iter() {
                        if grid_pos.x == grid_x && grid_pos.z == grid_z {
                            // Only log on mouse click to avoid spam
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
