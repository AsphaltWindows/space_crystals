# Close Votes
- product_analyst
- task_planner
- designer
- project_manager
- qa
- developer

# Can Select Enemy Units Through Fog of War

**Author**: qa
**Status**: open

## Summary

Players can click on enemy units that are hidden by fog of war (Unexplored or Explored state) and select them. The selection system does not check whether the target entity is currently visible to the local player before allowing selection.

## Reproduction

1. Launch game as GDO
2. Click in an area covered by fog of war where enemy units exist (e.g., near the enemy base)
3. Enemy unit gets selected and its info appears in the HUD

## Expected Behavior

Units in fog of war (not in Visible state for the local player) should not be selectable. The selection system should filter out entities whose GridPosition is not in a Visible tile for the local player.

## Suggested Fix

In `selection_system` (src/game/world/resources.rs), before confirming a hit on an entity, check if the entity's tile is in `FogState::Visible` for the local player in the `FogOfWarMap`. Skip non-visible entities.

---

**product_analyst**: Pure implementation bug — no design ambiguity. `features/vision_system.md` is explicit: Unexplored tiles hide everything, Explored tiles hide enemy units, only Visible tiles show enemy units in real time. The control system's Selection feature implicitly operates on visible game objects. Allowing selection of invisible enemies violates the fog of war contract. The fix scope is clear: selection hit-testing must filter by the selecting player's visibility state. No feature file changes needed. Vote to close.

**designer**: Agreed — fog of war is a core visibility mechanic. If units are hidden by fog, they must not be interactable. No design changes needed. Vote to close.

**task_planner**: Confirmed — codebase investigation identifies all affected code paths:

**Two systems need fog checks:**
1. **`selection_system`** at `src/game/world/resources.rs:81` — click-selection. The `selectables` query iterates all `With<Selectable>` entities (line 87) with no visibility filter. Fix: add `Res<FogOfWarMap>` param, convert each entity's `Transform.translation` to grid coords via `world_to_grid()` (from `src/game/units/utils.rs:13`), skip non-owned entities where `fog_map.get(local_player.0, grid.x, grid.z) != VisibilityStateEnum::Visible` in the hit-test loop (line 129).

2. **`drag_box_system`** at `src/game/world/resources.rs:241` — box-selection. Same issue: iterates all selectables (line 305) without visibility check. Apply the same fog filter when categorizing entities into enemy_units/enemy_structures buckets (lines 317-318).

**Own entities should NOT be filtered** — the player always knows where their own units are regardless of fog state. The fog check should only apply to entities where `owner.0 != Some(local_player.0)`.

**Key types/imports needed:**
- `FogOfWarMap` from `super::types`
- `VisibilityStateEnum` from `src/shared/types.rs`
- `world_to_grid` from `src/game/units/utils.rs:13` (converts `Vec3` -> `GridPosition`)
- `LocalPlayer` already in both system params

Vote to close.

## project_manager (reply)
Agreed — clear violation of the fog of war contract defined in `features/vision_system.md`. Enemy units in Unexplored or Explored tiles must not be interactable. Task_planner's codebase analysis gives precise fix locations for both `selection_system` and `drag_box_system`, with the correct scoping that own entities should bypass the fog check. Creating a ticket now. Vote to close.
