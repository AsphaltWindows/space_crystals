# Task 002: Implement Grid-Based Map System

## Description
Create a foundational grid-based map system that will serve as the terrain for the RTS game. This system should represent the game world as a 2D grid where each cell can hold tile data. The implementation should use Bevy ECS patterns with entities representing individual tiles, and should support querying tiles by grid coordinates.

## Why Needed
The map system is the foundation for all gameplay mechanics in an RTS. Units need to navigate on tiles, buildings need to be placed on tiles, and resources need to exist on specific tiles. This must be implemented before any gameplay features involving the map.

## Acceptance Criteria
- [ ] Grid resource exists storing grid dimensions (width, height) and cell size
- [ ] Tile entities are spawned in a grid pattern
- [ ] Each tile entity has a GridPosition component storing its (x, z) coordinates
- [ ] Helper function exists to convert world coordinates to grid coordinates
- [ ] Helper function exists to query a tile entity by grid position
- [ ] Visual representation of the grid exists (simple plane meshes for now)
- [ ] Camera is positioned to view the entire grid
- [ ] Grid dimensions are configurable (start with 20x20)

## Relevant Files/Components
- `src/main.rs` - Will need to add grid-related systems and components
- Consider creating `src/map.rs` or `src/grid.rs` for map-specific code
- `setup` system - Will need to spawn grid tiles
- Camera positioning in `setup` - Adjust to view the full grid

## Technical Considerations

**Grid Representation**:
- Use a 2D grid where each cell is a square tile
- Grid coordinates: (x, z) where y is elevation
- World space: Center the grid at origin for simplicity

**ECS Architecture**:
```rust
// Resource to store grid metadata
struct GridMap {
    width: u32,
    height: u32,
    cell_size: f32,
}

// Component to mark tile entities
#[derive(Component)]
struct Tile;

// Component storing grid position
#[derive(Component)]
struct GridPosition {
    x: i32,
    z: i32,
}
```

**Helper Functions**:
- `world_to_grid(world_pos: Vec3, cell_size: f32) -> (i32, i32)`
- `grid_to_world(grid_x: i32, grid_z: i32, cell_size: f32) -> Vec3`
- Query system to find tile at grid position

**Visual Representation**:
- Spawn a plane mesh for each tile
- Use a simple material (different color than background)
- Tiles should be clearly visible and distinguishable

**Performance Note**:
- For a 20x20 grid (400 tiles), spawning individual entities is fine
- Consider spatial indexing in future tasks if grid becomes very large

## Prerequisites
None - This builds on the basic setup from Task 001.

## Complexity
Medium
