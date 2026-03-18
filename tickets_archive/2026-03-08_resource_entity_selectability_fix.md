# Ticket: Fix Crystal Patch and Supply Delivery Station Selectability

## Current State
The `selection_group_sync_system` in `src/game/world/resources.rs` (line ~790) filters selected entities with `Or<(With<Unit>, With<StructureInstance>)>`. Space Crystal Patches and Supply Delivery Stations have the `Selectable` component and receive the `Selected` marker when clicked, but they lack both `Unit` and `StructureInstance` markers. As a result, they are excluded from `Selection.groups`, making them effectively non-interactive — no InfoPanel displays, no selection feedback.

This was introduced by the phantom command panel fix (`phantom_command_panel_deployment_center`), which correctly prevented empty-tile clicks from showing a command panel but over-corrected by filtering at the selection group level instead of the command panel display level.

## Desired State
Crystal Patches and Supply Delivery Stations appear in `Selection.groups` when clicked or box-selected. Selecting them shows their InfoPanel (info-only, no commands):
- **SpaceCrystalsPatch**: displays RemainingAmount (when visible to selecting player)
- **SupplyDeliveryStation**: always displays DeliverySize and DeliveryInterval; displays CurrentSupplies when visible to selecting player

The CommandPanel does NOT appear when only resource entities are selected (they have no commands). The phantom command panel bug must not regress.

## Implementation Approach
**Option 1 (recommended by task_planner)**: Relax `selection_group_sync_system` filter to include resource entities. Two sub-approaches:
- Broaden the filter: `Or<(With<Unit>, With<StructureInstance>, With<SpaceCrystalPatch>, With<SupplyDeliveryStation>)>`
- Or use `With<ObjectInstance>` if all selectable entities have it (note: SCP may lack `ObjectInstance` — verify at `resources.rs:63`)

Then guard CommandPanel visibility (`src/ui/command_panel.rs`, `is_panel_visible`) to suppress the panel when the selection contains only non-commandable entities (no units, no structures with commands).

**Key files**:
- `src/game/world/resources.rs` — `selection_group_sync_system` (~line 790), `spawn_space_crystal_patches` (~line 11), SDS spawning (~line 594)
- `src/ui/command_panel.rs` — `is_panel_visible` (~line 210)
- Existing tests at `resources.rs` lines ~939-1021 that assert SDS exclusion — these need updating

## Justification
`design/entities.md`: Object Type has `Selectable=true`; both SpaceCrystalsPatch and SupplyDeliveryStation are Resource types inheriting Object Type with defined InfoPanels.
`features/entity_system.md` (lines 68-70): Confirms both as selectable Object Types with InfoPanel definitions.
`features/control_system.md` (line 33): BoxSelection tier 5 ("Neutral objects") explicitly covers unowned entities.
Forum topic: `crystal_patches_sds_not_selectable.md` — unanimous agreement this is a regression bug.

## QA Steps
1. [auto] Spawn a SpaceCrystalsPatch. Click on it. Verify `Selection.groups` is non-empty and contains an entry with `ObjectEnum::SpaceCrystalsPatch`.
2. [auto] Spawn a SupplyDeliveryStation. Click on it. Verify `Selection.groups` is non-empty and contains an entry with `ObjectEnum::SupplyDeliveryStation`.
3. [human] Click on a Crystal Patch in-game. Verify an InfoPanel appears showing RemainingAmount. Verify NO command panel appears.
4. [human] Click on a Supply Delivery Station in-game. Verify an InfoPanel appears showing DeliverySize, DeliveryInterval, and CurrentSupplies. Verify NO command panel appears.
5. [auto] Click on an empty tile (no entity). Verify `Selection.groups` is empty and no phantom command panel appears.
6. [auto] Select a unit, then select a Crystal Patch. Verify the selection switches cleanly — previous unit deselected, patch selected, no command panel.
7. [human] Box-select an area containing only Crystal Patches. Verify one patch is selected (tier 5 single-select per control_system spec). Verify InfoPanel displays, no command panel.
8. [auto] Verify the existing phantom command panel regression does not recur: click empty ground, assert `Selection.groups` is empty.

## Expected Experience
- Clicking a Crystal Patch highlights it and shows a small info display with remaining crystal amount. No command buttons appear.
- Clicking a Supply Delivery Station highlights it and shows delivery stats (size, interval, current supplies). No command buttons appear.
- Clicking empty ground still shows nothing — no phantom panel regression.
- Box-selecting over resource entities picks up one resource (single-select tier 5 behavior).
