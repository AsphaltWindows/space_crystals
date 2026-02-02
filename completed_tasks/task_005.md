# Task 005: Implement Unit Entity Foundation

## Description
Create the foundational structure for unit entities in the game. This includes base components that all units will have (position, health, owner/faction, selection state) and a system to spawn test units. This establishes the ECS architecture for units before implementing specific unit types and behaviors.

## Why Needed
Units are central to RTS gameplay. Before implementing specific unit types (Light Infantry, Wheeled Vehicle, etc.) or behaviors (movement, combat), we need the foundational entity structure that all units will share. This task creates that common foundation.

## Acceptance Criteria
- [ ] Unit component exists to mark unit entities
- [ ] UnitHealth component exists with current and max health
- [ ] Owner component exists to track which player/faction owns the unit
- [ ] Unit entities can be selected (using Selectable/Selected from Task 004)
- [ ] Test units spawn at specific grid positions
- [ ] Units have visual representation (simple capsule or cube mesh for now)
- [ ] Units render above the ground plane
- [ ] Clicking a unit selects it with visual feedback
- [ ] Selected unit shows basic info in console (type, health, owner)
- [ ] 3-5 test units spawn in the startup system

## Relevant Files/Components
- Create `src/units.rs` for unit-specific code
- Extend selection system from Task 004 to work with units
- Startup system - Spawn test units
- `src/main.rs` - Add units plugin/module

## Technical Considerations

**Core Unit Components**:
```rust
#[derive(Component)]
struct Unit;

#[derive(Component)]
struct UnitHealth {
    current: f32,
    max: f32,
}

#[derive(Component, Clone, Copy, Debug)]
enum Owner {
    Player(u8),  // Player 0, 1, 2, etc.
    Neutral,
}

#[derive(Component)]
struct UnitType {
    name: String,  // "Test Infantry", "Test Vehicle", etc.
}

// Reuse from Task 004
#[derive(Component)]
struct Selectable;

#[derive(Component)]
struct Selected;
```

**Visual Representation**:
- Use simple primitive mesh (capsule for infantry, cube for vehicles)
- Color-code by owner (e.g., Player 0 = blue, Player 1 = red)
- Position at grid location, elevated above ground (y = 0.5)
- Apply StandardMaterial with base_color based on owner

**Selection System Extension**:
- Extend raycast system to detect unit entities
- When unit is clicked, add Selected component
- Deselect previous selection (remove Selected component)
- Visual feedback: Ring around selected unit or glowing outline

**Unit Spawning**:
- Spawn units at specific grid positions
- Use helper function from Task 002 to convert grid to world coords
- Create test units with varying properties:
  - 2 units owned by Player 0 (blue)
  - 2 units owned by Player 1 (red)
  - 1 neutral unit
- Place units on different grid cells

**Console Info Display**:
- When unit is selected, log info:
  - "Unit: Test Infantry"
  - "Health: 100/100"
  - "Owner: Player 0"

**Module Organization**:
- Create `src/units.rs` module
- Move unit-related components and systems there
- Add to `src/main.rs` with `mod units;`
- Register unit systems in plugin

## Prerequisites
- [x] `task_002.md` - Grid system needed for unit positioning
- [x] `task_003.md` - Tile system needed for unit placement validation
- [x] `task_004.md` - Selection system foundation needed

## Complexity
Medium
