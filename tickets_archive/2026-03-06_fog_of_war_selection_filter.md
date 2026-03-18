# Ticket: Filter Selection by Fog of War Visibility

## Current State
Both `selection_system` (click-selection) and `drag_box_system` (box-selection) in `src/game/world/resources.rs` iterate all entities with the `Selectable` component and perform hit-testing without checking whether the entity is visible to the local player. This allows players to select enemy units and structures that are hidden by fog of war (Unexplored or Explored tiles).

## Desired State
Selection hit-testing must filter out non-owned entities whose grid position is not in `VisibilityStateEnum::Visible` state for the local player in the `FogOfWarMap`. Own entities (where `owner.0 == Some(local_player.0)`) should always be selectable regardless of fog state, since the player always knows where their own units are.

Two systems need the fog check:
1. `selection_system` (click-selection at `resources.rs:81`) — add fog visibility check in the hit-test loop (around line 129) before confirming a hit on a non-owned entity.
2. `drag_box_system` (box-selection at `resources.rs:241`) — add fog visibility check when categorizing entities into enemy_units/enemy_structures buckets (around lines 317-318).

Both systems need `Res<FogOfWarMap>` added as a system parameter. Use `world_to_grid()` from `src/game/units/utils.rs:13` to convert entity `Transform.translation` to grid coordinates for the fog map lookup.

## Justification
`features/vision_system.md` specifies three visibility states: Unexplored (everything hidden), Explored (enemy units hidden), and Visible (everything shown in real time). Allowing selection of invisible enemies violates this fog of war contract and gives the player information they should not have. Originated from forum topic `select_enemies_through_fog.md`.

## QA Steps
1. Launch the game as GDO.
2. Note the enemy base location (or know where enemy units are from a previous exploration).
3. Without sending any units to explore, click on the area where enemy units exist (covered by Unexplored fog). Verify: no enemy unit is selected, no enemy info appears in HUD.
4. Send a unit to explore and reveal some enemy units (tiles become Visible). Verify: enemy units in Visible tiles CAN be selected by clicking on them.
5. Pull your unit back so the enemy area transitions to Explored state. Verify: enemy units in Explored tiles CANNOT be selected by clicking.
6. Repeat steps 3-5 using box-selection (click-drag) instead of click-selection.
7. Verify that your own units are always selectable regardless of fog state (e.g., if a unit is at the edge of your vision and its tile becomes Explored, you can still select it).

## Expected Experience
- Clicking or box-selecting in fogged areas produces no selection of enemy entities — it behaves as if nothing is there.
- Enemy entities in Visible tiles select normally with their info displayed in the HUD.
- Own entities always select normally regardless of fog state.
- No change to selection behavior for friendly/own units.
