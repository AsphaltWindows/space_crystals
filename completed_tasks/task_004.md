# Task 004: Implement Resource Entities (Space Crystal Patches)

## Description
Implement Space Crystal Patch (SCP) entities as described in the design document. These are the primary resource nodes in the game that factions will harvest. Each SCP should occupy a tile, prevent building (with exceptions noted in design), block traversal, and display remaining resources when selected.

## Why Needed
Space Crystals are the core resource for all factions. Without resource nodes, there's no economy system. This task establishes the foundation for resource gathering mechanics that will be implemented by different factions in future tasks.

## Acceptance Criteria
- [ ] SpaceCrystalPatch component exists with amount field (remaining crystals)
- [ ] SCP entities spawn on specific tiles in the test map
- [ ] SCPs have visual representation distinct from regular tiles (glowing crystal material)
- [ ] SCPs are marked as not buildable and not traversible on their tiles
- [ ] Clicking/selecting an SCP displays its remaining amount (console log for now)
- [ ] SCPs are selectable entities with visual feedback (outline or highlight)
- [ ] Test map spawns 3-5 SCPs with varying amounts (1000-5000 crystals each)

## Relevant Files/Components
- `src/map.rs` or create `src/resources.rs` - SCP-specific code
- Tile properties - SCPs modify their tile's properties
- Selection system - SCPs need to be selectable
- Startup system - Spawn test SCPs during map generation

## Technical Considerations

**SCP Component Structure**:
```rust
#[derive(Component)]
struct SpaceCrystalPatch {
    amount: u32,
    initial_amount: u32,
}

#[derive(Component)]
struct Selectable;

#[derive(Component)]
struct Selected;
```

**Tile Interaction**:
- When SCP is spawned on a tile, modify that tile's properties:
  - Set buildable = false
  - Set traversible = false
- Consider storing a reference between SCP and its tile, or query by position

**Visual Representation**:
- SCP should be positioned slightly above the tile (y offset)
- Use a mesh that looks crystal-like (can be simple geo for now)
- Apply an emissive material to make it glow
- Color: Cyan/blue with emissive glow

**Selection Visual Feedback**:
- Add a ring or outline around selected SCP
- Can use a simple cylinder mesh as selection indicator
- When Selected component is added, spawn selection indicator as child entity

**Resource Amount Display**:
- For now, log to console when SCP is clicked
- Format: "Space Crystal Patch: 3500 / 5000 remaining"
- Future task will add UI display

**Placement on Map**:
- Place SCPs on Plane tiles (valid for resources)
- Distribute across map, not all clustered
- Use fixed positions for now (deterministic testing)

## Prerequisites
- [x] `task_002.md` - Grid system needed to place SCPs on tiles
- [x] `task_003.md` - Tile properties needed to mark SCP tiles as not buildable/traversible

## Complexity
Medium
