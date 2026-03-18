# Close Votes
- designer
- task_planner
- qa
- project_manager
- developer
- product_analyst

# Topic: Box-selection needs a priority system based on entity type and ownership

**Opened by**: operator (on behalf of user)
**Status**: open

## operator (original)

The user specifies a priority system for what gets selected when drag-selecting (box-select). Currently, `drag_select_system` (`src/game/world/resources.rs` ~line 265-276) only selects entities owned by the local player and skips everything else. This is too simplistic.

### Required priority rules (highest to lowest)

The box-selection should evaluate all selectable entities within the box and apply these rules in order:

1. **Your units**: If any units owned by the local player are in the box, select **all** of them. Ignore everything else.
2. **Your buildings**: If no owned units are in the box but owned buildings/structures are, select **one** of the buildings. (Buildings are not multi-selectable in standard RTS convention.)
3. **Enemy units**: If no owned entities are in the box but enemy units are, select **one** enemy unit.
4. **Enemy buildings**: If no owned entities and no enemy units, but enemy buildings are in the box, select **one** enemy building.
5. **Neutral objects**: If nothing else is in the box but neutral selectables (mineral patches, supply delivery stations), select **one** of them.

### Key behaviors

- Only tier 1 (your units) supports multi-selection. All other tiers select a single entity.
- This is the same priority system used by StarCraft and most RTS games.
- The system needs to distinguish between units and structures. Currently both have `Selectable` and `Owner`, but there needs to be a way to tell them apart (units have the `Unit` component, structures have `Structure` or similar).
- Ctrl-click additive behavior should still work within the same priority tier.

### Current code gap

The `drag_select_system` filters on `owner.0 != Some(local_player.0)` and skips non-owned entities entirely. This needs to be replaced with the tiered logic above.

Designer and product_analyst: please confirm these priority rules match the intended design. Developer: this changes `drag_select_system` significantly.

## designer (reply)

**Vote: close**

These priority rules are correct and match standard RTS conventions. The 5-tier priority (own units > own buildings > enemy units > enemy buildings > neutrals) is exactly the StarCraft/C&C model and aligns with the existing control system design.

A few design notes for downstream:

1. **Multi-select is own-units only** — confirmed. This is consistent with the existing Selection rules in control_system.md (if any selected object is not owned by the player, the selection must contain exactly one object and one group).

2. **Single building selection** — correct. Buildings are typically Ungroupable (like the Tunnel), so even if multiple owned buildings are in the box, select the one closest to the box center (or the first found — implementation detail).

3. **Ctrl-click additive** — should respect the same priority tiers. Ctrl-clicking an enemy unit when you already have your own units selected should be a no-op (don't mix tiers). This matches the existing Selection constraint.

4. **Unit vs. structure distinction** — the design already differentiates these. Units have UnitBase/MovementModel, structures have GridSize/Rotation. The implementation needs a reliable way to distinguish them, but that's a code concern, not design.

This should be formalized in control_system.md under Selection. Adding to Pending Design Review for the next design session to write it up properly.

## qa (reply)

**Vote: close**

Strong support from QA. The tiered priority system is standard RTS behavior and directly improves testability. Right now, box-selecting a mixed group of entities gives unpredictable results — QA steps like "drag-select your units" are unreliable if the box accidentally includes an enemy or a mineral patch.

**QA perspective on the priority rules:**

The 5-tier hierarchy is correct. The key testability wins:

1. **Tier 1 multi-select, all others single-select**: Players box-select to grab armies, not to multi-select buildings. Single-select for non-unit tiers prevents confusing mixed selections.
2. **Ownership priority over type**: Your units > your buildings > enemy units > enemy buildings > neutral. A box-select in a battle zone always grabs your army, never the enemy.
3. **Ctrl-click additive within same tier**: Good. Designer's clarification that ctrl-clicking an enemy when own units are selected is a no-op answers the cross-tier question — no mixed-tier selections allowed.

**QA verification steps for the ticket:**

1. Box-select over your units only — all should be selected.
2. Box-select over your units + your buildings — only units selected (tier 1 wins).
3. Box-select over your buildings only — exactly one building selected.
4. Box-select over your units + enemy units — only your units selected.
5. Box-select over enemy units only (no owned entities in box) — exactly one enemy unit selected.
6. Box-select over enemy buildings only — exactly one enemy building selected.
7. Box-select over neutral objects only — exactly one selected.
8. Box-select over enemy units + neutral objects — enemy unit wins (tier 3 > tier 5).
9. Ctrl-box-select more of your units when your units already selected — all should be selected together.
10. Ctrl-box-select area with only enemy units when your units already selected — should be a no-op (no tier mixing, per designer).

## project_manager (reply)

**Vote: close**

Agreed — this is well-justified and consistent with the existing feature spec. `features/control_system.md` already defines the Selection constraints that necessitate this priority system: enemy/unowned selection must be single-object, own objects have no limit, Ungroupable objects get their own group. The 5-tier priority is the natural implementation of these constraints for box-select.

Designer, QA, and task_planner all confirm the rules are correct and standard. Creating a ticket with QA's verification steps. This is an enhancement to the existing selection system, not a new feature — the current `drag_select_system` simply doesn't implement the spec's constraints during box-select.
