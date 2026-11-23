# fix_hud_ui_bugs

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Three HUD/UI bugs reported by the user need fixing. No design doc changes — the existing docs clearly specify the correct behavior.

**Bug 1: Peacekeeper missing command panel**
When a Peacekeeper is selected, no command panel appears. Per `artifacts/designer/design/gdo_objects.md`, Peacekeeper has `ObjectInterfaceState: BasicCombatUnitInterfaceState`. Per `artifacts/designer/design/control_system.md`, the CommandPanel should be visible when at least 1 player-owned object is selected, showing the BasicCombatUnitInterfaceState 4x3 grid: Q=Move, E=HoldPosition, A=Attack, S=Patrol, D=AttackGround (CanTargetGround=true), X=Stop. W=Reverse should be hidden (LightInfantry CanReverse=false).

**Bug 2: Extraction Facility build button label**
The button to build an Extraction Plate on the Extraction Facility just says 'Build'. Per `artifacts/designer/design/gdo_objects.md`, the idle DefaultState command is 'Q: Build Extraction Plate' — the label should be 'Build Extraction Plate', not just 'Build'.

**Bug 3: Extra tile / duplicate portrait in HUD bottom row**
An extra tile or duplicate portrait appears in the HUD bottom row. Per `artifacts/designer/design/control_system.md`, the HUD Bar has exactly 5 fixed-position sections (Minimap, Portrait, InfoPanel, SelectionPanel, CommandPanel). There should be no duplicate or extra elements.

## QA Instructions

1. **Bug 1 — Peacekeeper command panel:**
   - Spawn or select a player-owned Peacekeeper unit.
   - Verify the CommandPanel appears in the rightmost HUD section.
   - Verify buttons for Move (Q), HoldPosition (E), Attack (A), Patrol (S), AttackGround (D), and Stop (X) are visible.
   - Verify Reverse (W) is NOT shown (LightInfantry cannot reverse).
   - Click each button and verify it enters the correct AwaitingTarget state or issues the correct immediate command.

2. **Bug 2 — Extraction Facility button label:**
   - Select a player-owned Extraction Facility that is idle (not constructing, nothing ready to place).
   - Verify the Q button reads 'Build Extraction Plate' (not just 'Build').
   - Click it and verify it deducts 75 Space Crystals and starts construction.

3. **Bug 3 — No extra HUD tiles:**
   - Select various units and structures.
   - Verify the HUD bottom row has exactly 5 sections with no duplicate portraits or extra tiles.
   - Verify the Portrait section shows exactly one portrait for single-selection.
   - Verify the SelectionPanel only appears when 2+ units are selected.
