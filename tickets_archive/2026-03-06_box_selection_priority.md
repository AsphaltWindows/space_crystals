# Ticket: Implement Box-Selection Priority System

## Current State
The `drag_select_system` in `src/game/world/resources.rs` only selects entities owned by the local player and skips all non-owned entities. There is no priority logic — if a box contains owned units, owned buildings, and enemy units, it selects everything owned indiscriminately. Enemy and neutral entities cannot be box-selected at all.

## Desired State
Box-selection evaluates all selectable entities within the drag rectangle and applies a 5-tier priority system (highest to lowest):

1. **Own units**: If any units owned by the local player are in the box, select ALL of them. Ignore everything else.
2. **Own buildings/structures**: If no owned units but owned structures are present, select ONE structure (e.g., closest to box center).
3. **Enemy units**: If no owned entities but enemy units are present, select ONE enemy unit.
4. **Enemy buildings**: If no owned entities and no enemy units but enemy structures are present, select ONE enemy structure.
5. **Neutral objects**: If nothing else is in the box, select ONE neutral selectable (mineral patch, supply station).

Only tier 1 (own units) supports multi-selection. All other tiers produce a single-entity selection.

Ctrl-click additive selection respects the same priority tiers — adding an entity from a lower tier when a higher tier is already selected is a no-op (no cross-tier mixing).

## Justification
The control system feature (`features/control_system.md`) defines Selection constraints: enemy/unowned selection must contain exactly one object in one group; own objects have no count limit. The current `drag_select_system` does not implement these constraints during box-select, and cannot select enemy/neutral entities at all. The 5-tier priority is the standard RTS implementation (StarCraft, C&C) of these constraints. Forum topic: `forum/box_selection_priority.md`.

## QA Steps
1. Box-select over your units only — verify all are selected.
2. Box-select over your units + your buildings — verify only units are selected (tier 1 wins).
3. Box-select over your buildings only (no units in box) — verify exactly one building is selected.
4. Box-select over your units + enemy units — verify only your units are selected (ownership priority).
5. Box-select over enemy units only (no owned entities in box) — verify exactly one enemy unit is selected.
6. Box-select over enemy buildings only — verify exactly one enemy building is selected.
7. Box-select over neutral objects only (mineral patches, supply stations) — verify exactly one is selected.
8. Box-select over enemy units + neutral objects — verify enemy unit wins (tier 3 > tier 5).
9. With your units already selected, Ctrl-box-select more of your own units — verify all are selected together (additive within tier).
10. With your units already selected, Ctrl-box-select area containing only enemy units — verify selection is unchanged (no cross-tier mixing).

## Expected Experience
Box-selecting in a battle zone always grabs the player's army, never accidentally selecting an enemy or a building mixed in. When no owned entities are present, box-select falls through to enemy/neutral entities with single-select behavior. Ctrl-click additive selection works within the same tier but refuses to mix tiers. The behavior matches standard RTS conventions that players expect.
