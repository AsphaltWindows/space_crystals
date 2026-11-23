# fix_hud_ui_bugs

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-fix_hud_ui_bugs.md

## Task

Fix three HUD/UI bugs. All are in the UI layer (primarily `src/ui/`).

**Bug 1: Peacekeeper has no command panel when selected**
- When a Peacekeeper is selected, no command panel appears.
- The data layer is correct: `peacekeeper_default_commands_count` test passes with 6 grid commands (Move, HoldPos, Attack, Patrol, AtkGround, Stop).
- The bug is in the UI spawning/selection layer. Investigate `ui/command_panel/ui_rebuild.rs` and `ui/hud.rs` — the system that detects a Peacekeeper is selected and triggers the command panel rebuild is likely not matching the Peacekeeper entity correctly.
- Expected: BasicCombatUnitInterfaceState grid with Q=Move, E=HoldPosition, A=Attack, S=Patrol, D=AttackGround, X=Stop. W=Reverse hidden (LightInfantry CanReverse=false).

**Bug 2: Extraction Facility build button just says 'Build'**
- Root cause identified: In `ui/command_panel/helpers.rs` (around line 327), `grid_button_label()` has specific labels for PowerPlant ('PP'), Barracks ('BK'), SupplyTower ('ST'), but `DcBuild(ObjectEnum::ExtractionFacility)` falls through to the catch-all `DcBuild(_) => format!("[{}] Build", hotkey)`.
- Fix: Add a specific match arm for `DcBuild(ObjectEnum::ExtractionFacility)` before the catch-all, with label like `"[{}] EF\n{cost} SC"` (following the pattern of PP/BK/ST labels). The cost is 75 Space Crystals per design docs.

**Bug 3: Extra tile / duplicate portrait in HUD bottom row**
- Investigate `ui/hud.rs` portrait rendering section.
- Per design, the HUD Bar has exactly 5 sections (Minimap, Portrait, InfoPanel, SelectionPanel, CommandPanel) with fixed positions.
- Look for duplicate entity spawns, incorrect layout sizing, or extra UI nodes being created in the portrait area.

**QA Instructions (for reference):**
1. Spawn/select a Peacekeeper — verify command panel appears with correct buttons.
2. Select an idle Extraction Facility — verify Q button reads 'Build Extraction Plate' (not just 'Build').
3. Select various units/structures — verify no duplicate portraits or extra tiles in HUD bottom row.
