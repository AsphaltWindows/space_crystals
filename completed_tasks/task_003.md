# Task 003: Implement Tile Properties and Types

## Description
Extend the grid system to support different tile types with associated properties as defined in the design document. Each tile should have a type (Plane, Rugged Terrain, Cliff, Mountain, Water) and corresponding properties (Buildable, Traversible, Drillable, Rugged, Recruitable) that will affect gameplay mechanics.

## Why Needed
Different terrain types are core to the RTS design. Units have different movement capabilities based on terrain, buildings can only be placed on certain tiles, and gameplay tactics depend on terrain variation. This provides the foundation for strategic map design and unit differentiation.

## Acceptance Criteria
- [ ] TileType enum exists with variants: Plane, RuggedTerrain, Cliff, Mountain, Water
- [ ] TileProperties struct exists with boolean flags: buildable, traversible, drillable, rugged, recruitable
- [ ] Each TileType has a method or mapping to get its default properties
- [ ] Tile component is extended to include TileType and TileProperties
- [ ] Different tile types are visually distinguishable (different colors or materials)
- [ ] Test map is generated with variety of tile types (at least one of each type)
- [ ] Hovering over a tile with mouse displays its type (console log for now)

## Relevant Files/Components
- `src/map.rs` or `src/grid.rs` - Add tile type definitions
- Tile component - Extend with type and properties
- Startup system - Generate a test map with varied terrain
- Add a system to handle mouse hover and display tile info

## Technical Considerations

**Tile Type Definitions**:
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum TileType {
    Plane,
    RuggedTerrain,
    Cliff,
    Mountain,
    Water,
}

#[derive(Component, Clone, Copy)]
struct TileProperties {
    buildable: bool,
    traversible: bool,
    drillable: bool,
    rugged: bool,
    recruitable: bool,
}

impl TileType {
    fn default_properties(&self) -> TileProperties {
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
            // ... other types
        }
    }
}
```

**Visual Differentiation**:
- Assign different colors to each tile type:
  - Plane: Light green
  - Rugged: Brown
  - Cliff: Gray
  - Mountain: Dark gray
  - Water: Blue
- Use Bevy's StandardMaterial with base_color

**Test Map Generation**:
- Create a simple pattern or use noise for varied terrain
- Ensure at least one tile of each type exists for testing
- Can be deterministic for now (no need for random generation yet)

**Mouse Interaction** (basic):
- Use raycast from camera through mouse position
- Check if ray hits a tile entity
- Log tile type to console

## Prerequisites
- [x] `task_002.md` - Grid system must exist to add properties to tiles

## Complexity
Medium
