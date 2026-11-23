# fix_hud_ui_bugs

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Bug 1: `owned_by_local_player` field overwrite in `update_command_panel_state`

**Root cause identified.** This bug affects ALL player-owned units (not just Peacekeeper), but Peacekeeper may be the only one tested recently.

**File:** `artifacts/developer/src/ui/command_panel/state.rs` — `update_command_panel_state()` function

The execution flow within `update_command_panel_state`:
1. **Line 154-157**: `owned` is computed correctly as `true` for player-owned units, and `unit_caps.owned_by_local_player = true` is set.
2. **Line 289**: `compute_selected_unit_capabilities()` returns a `SelectedUnitCapabilities::default()` where `owned_by_local_player` is `false`.
3. **Line 290-292**: `if *unit_caps != new_caps { *unit_caps = new_caps; }` — since `owned_by_local_player` differs (true vs false), the entire struct is overwritten, setting `owned_by_local_player = false`.

Then in `rebuild_command_panel_ui` (ui_rebuild.rs line 43):
```rust
if !is_panel_visible(&interface_state, &selection) || !unit_caps.owned_by_local_player {
    return; // <-- Returns early because owned_by_local_player is false
}
```

And in `hud_section_visibility_system` (hud.rs line 250-252):
```rust
*vis = if total >= 1 && owned { Visibility::Inherited } else { Visibility::Hidden };
// owned = unit_caps.owned_by_local_player = false, so panel is Hidden
```

**Fix:** In `update_command_panel_state` (state.rs), after the `*unit_caps = new_caps` assignment at line 292, re-apply the ownership flag:
```rust
let new_caps = compute_selected_unit_capabilities(&selected_units, &selection);
if *unit_caps != new_caps {
    *unit_caps = new_caps;
}
// Preserve ownership computed earlier (compute_selected_unit_capabilities doesn't track this)
if unit_caps.owned_by_local_player != owned {
    unit_caps.owned_by_local_player = owned;
}
```

Alternatively, refactor `compute_selected_unit_capabilities` to accept and preserve `owned_by_local_player`, but the simplest fix is moving the ownership set after the capability overwrite.

**System ordering context** (from `artifacts/developer/src/ui/mod.rs` lines 47-48):
- `update_command_panel_state` runs after `update_cursor_target`
- `rebuild_command_panel_ui` runs after `update_command_panel_state`
- `hud_section_visibility_system` runs after `update_command_panel_state`

**Verification:** After fix, also verify SyndicateAgent, SyndicateGuard, CultsRecruit, CultsSoldier command panels work (they all go through the same overwrite path at line 289-292).

**Test recommendation:** Add a unit test that calls `update_command_panel_state` with a player-owned unit entity selected, then asserts `unit_caps.owned_by_local_player == true` after the system runs. Test pattern: see existing hud.rs tests around line 2262 using `world.run_system_once()`.

---

### Bug 2: Missing label for `DcBuild(ObjectEnum::ExtractionFacility)`

**File:** `artifacts/developer/src/ui/command_panel/helpers.rs` — `grid_button_label()` function (line 316-327)

**Root cause:** The Deployment Center's build menu includes `DcBuild(ObjectEnum::ExtractionFacility)` at grid position (1,1) (defined in `artifacts/developer/src/game/entities/factions/gdo/buildings/base.rs` line 35). But `grid_button_label()` has no match arm for it. Existing labels:
- Line 324: `DcBuild(ObjectEnum::PowerPlant) => format!("[{}] PP\n150 SC", hotkey)`
- Line 325: `DcBuild(ObjectEnum::Barracks) => format!("[{}] BK\n200 SC", hotkey)`
- Line 326: `DcBuild(ObjectEnum::SupplyTower) => format!("[{}] ST\n200 SC", hotkey)`
- Line 327: `DcBuild(_) => format!("[{}] Build", hotkey)` ← ExtractionFacility falls here

**Fix:** Add before line 327:
```rust
CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility) => format!("[{}] EF\n200 SC", hotkey),
```

The cost is 200 Space Crystals (confirmed in `artifacts/developer/src/game/entities/factions/gdo/buildings/extraction.rs` test `ef_cost` at line 234-235, and design doc `gdo_objects.md` line 122).

**Note:** The task description says 75 SC but that is the ExtractionPlate cost, not the ExtractionFacility cost. The DC builds the ExtractionFacility (200 SC). The EF then builds ExtractionPlates (75 SC each, using `EfBuildPlate` action which already has correct label at helpers.rs line 336).

**Test recommendation:** Add a test for `grid_button_label` with `DcBuild(ObjectEnum::ExtractionFacility)` input, asserting it contains "EF" and "200 SC". Follow the pattern of existing tests in helpers.rs (e.g., the test at line 1819 that tests the DcBuild action resolution).

---

### Bug 3: Shared `UnitIcon`/`StructureIcon`/`ResourceIcon` markers between Info Panel and Selection Panel

**File:** `artifacts/developer/src/ui/hud.rs`

**Root cause:** The `update_info_panel_section_system` (line 441) uses global queries to find existing icon entities:
- Line 449: `existing_unit_icons: Query<(Entity, &UnitIcon)>`
- Line 450: `existing_struct_icons: Query<(Entity, &StructureIcon)>`
- Line 451: `existing_resource_icons: Query<(Entity, &ResourceIcon)>`

These queries match ALL entities in the world with those markers, not just info panel children. The selection panel also spawns entities with these same markers:
- `spawn_selection_portrait` (line 1192-1196): adds `StructureIcon` or `UnitIcon`
- `spawn_resource_portrait` (line 1349): adds `ResourceIcon`

This causes:
1. `existing_count` at line 464 includes selection panel icons, making `existing_count != total_selected` evaluate incorrectly
2. The despawn loop at lines 469-477 destroys ALL icon entities including selection panel's icons
3. The selection panel's icons get destroyed by the info panel system, and then the info panel recreates its own — resulting in visual glitches (extra/orphaned tiles from frame-to-frame rebuild races)

**Marker component definitions** (in `artifacts/developer/src/ui/types.rs`):
- Line 65: `pub struct UnitIcon { pub unit_entity: Entity }`
- Line 105: `pub struct StructureIcon { pub structure_entity: Entity }`
- Line 117: `pub struct ResourceIcon { pub resource_entity: Entity }`

**Fix options (recommended: Option A):**

**Option A: Scope queries to info panel children.** Add a new marker component `InfoPanelIcon` to `ui/types.rs`. When the info panel spawns icon entities (lines 529, 777, 905, 980), also insert `InfoPanelIcon`. Change the info panel system's existing_*_icons queries to filter `With<InfoPanelIcon>`. This leaves the selection panel's icons untouched.

**Option B: Use separate marker types.** Create `SelectionPanelUnitIcon`, `SelectionPanelStructureIcon`, `SelectionPanelResourceIcon` for the selection panel. More invasive but cleaner long-term separation.

**Test recommendation:** Add a test that spawns both info panel and selection panel icons, then runs `update_info_panel_section_system` and asserts selection panel icons are NOT despawned.

## Dependencies

None. All three bugs are independent fixes in the UI layer with no cross-task dependencies. They can be fixed in any order.
