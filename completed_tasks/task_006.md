# Task 006: Implement Basic Unit Selection System

## Description
Create a comprehensive unit selection system that allows players to select single units or multiple units using mouse clicks and drag-box selection. This system should provide visual feedback for selected units and handle selection/deselection properly. This is a core RTS interaction that will be used for all unit commands.

## Why Needed
Unit selection is the primary way players interact with their units in an RTS. Before implementing movement, attack, or other commands, players need to be able to select which units they want to command. This task implements the standard RTS selection mechanics.

## Acceptance Criteria
- [ ] Left-click on unit selects that unit (deselecting others)
- [ ] Left-click on empty space deselects all units
- [ ] Ctrl+Left-click adds/removes unit from current selection
- [ ] Click-drag creates a selection box (visual rectangle on screen)
- [ ] Units within completed drag box are selected
- [ ] Selected units have clear visual feedback (ring, outline, or highlight)
- [ ] Multiple units can be selected simultaneously
- [ ] Selection works correctly with camera at different angles
- [ ] Selected units display selection indicators that follow the units
- [ ] Console logs selected units count when selection changes

## Relevant Files/Components
- `src/units.rs` - Selection components and systems
- Extend camera systems to support raycasting
- Add new systems for drag-box selection
- Add selection indicator entities

## Technical Considerations

**Selection Components** (extend from Task 005):
```rust
// Already exists from Task 005
#[derive(Component)]
struct Selectable;

#[derive(Component)]
struct Selected;

// New for this task
#[derive(Component)]
struct SelectionIndicator; // Visual ring/outline entity

#[derive(Resource)]
struct SelectionState {
    drag_start: Option<Vec2>,  // Screen space
    is_dragging: bool,
}
```

**Single Selection (Click)**:
- Raycast from camera through mouse position
- If hits selectable entity:
  - If Ctrl not pressed: Clear all Selected components, add to clicked unit
  - If Ctrl pressed: Toggle Selected on clicked unit
- If hits nothing and Ctrl not pressed: Clear all selections

**Drag-Box Selection**:
- On mouse down: Store screen position as drag_start
- On mouse drag: Set is_dragging = true, draw selection box
- On mouse up: Query all selectable units, check if in box, select those units
- Selection box: Draw a rectangle outline on screen from drag_start to current mouse

**Checking Units in Box**:
- Project unit world positions to screen space
- Check if screen position is within box bounds
- Select all units inside box

**Visual Feedback**:
- Spawn a child entity for each selected unit containing selection indicator
- Selection indicator: Ring mesh around unit base, or glowing outline
- Position indicator to follow unit (as child entity or via system)
- Different material for selection (bright color, emissive)

**Screen-Space UI for Drag Box**:
- Use Bevy's UI system to draw selection rectangle
- Draw only while is_dragging = true
- Rectangle from drag_start to current mouse position
- Use a thin border, semi-transparent fill

**Input Handling**:
- Query for left mouse button state
- Query for Ctrl key state
- Get mouse position from window events

**Camera Raycasting**:
- Use camera projection and transform to create ray
- Cast ray against unit colliders or meshes
- Consider adding simple collider components to units for easier raycasting

## Prerequisites
- [x] `task_002.md` - Grid system for unit positions
- [x] `task_003.md` - Tile system
- [x] `task_004.md` - Basic selection foundation
- [x] `task_005.md` - Unit entities must exist to be selected

## Complexity
Medium
