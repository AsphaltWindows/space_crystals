# command_panel_directory_split

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-command_panel_split.md

## Task

Split the monolithic `src/ui/command_panel.rs` (6,357 lines) into a `command_panel/` directory module with 8 submodules plus mod.rs. This is a pure file reorganization -- no behavioral changes.

### Directory Structure to Create

```
src/ui/command_panel/
    mod.rs              -- pub mod declarations + pub use re-exports for all public systems
    grid_layout.rs      -- GRID_HOTKEYS, keycode_to_grid_slot(), get_grid_slot_action()
    state.rs            -- update_cursor_target(), update_command_panel_state(), selection_owned_by_local_player(), is_panel_visible()
    ui_rebuild.rs       -- rebuild_command_panel_ui(), spawn_panel_title(), spawn_progress_text(), spawn_info_text(), spawn_grid_button(), spawn_empty_grid_cell()
    hotkeys.rs          -- command_panel_hotkeys(), handle_command_button_clicks(), right_click_cancel_target(), right_click_cancel_submenu()
    actions.rs          -- execute_command_action(), update_command_panel_progress()
    tunnel_actions.rs   -- tunnel action helpers + tunnel grid builders + tunnel utility helpers
    pointer.rs          -- CursorEntityInfo, resolve_pointer_display(), resolve_awaiting_target(), resolve_default_state(), resolve_pointer_display_type()
    helpers.rs          -- compute_selected_unit_capabilities(), resource helpers, button enablement, action classification, selection helpers
```

### Verification

- `cargo check` must pass
- `cargo test --lib` must pass
- `cargo test --test qa` must pass

## Technical Context

### Primary file to split
- **`artifacts/developer/src/ui/command_panel.rs`** (6,357 lines) -- the entire file is replaced by a directory module

### Architecture document (READ THIS FIRST)
- **`artifacts/software_architect/architecture/command-panel-split.md`** -- contains the full analysis with exact line ranges, rationale, and expected file sizes for each submodule

### File that references command_panel (NO changes needed)
- **`artifacts/developer/src/ui/mod.rs`** -- `mod command_panel;` on line 8 will auto-resolve to `command_panel/mod.rs`. References 8 public systems via `command_panel::system_name` on lines 39-51. These must continue to work via re-exports from mod.rs.

### 8 public systems that must be re-exported from mod.rs
1. `update_cursor_target` (state.rs) -- line 203 in current file
2. `update_command_panel_state` (state.rs) -- line 325
3. `rebuild_command_panel_ui` (ui_rebuild.rs) -- line 516
4. `handle_command_button_clicks` (hotkeys.rs) -- line 915
5. `command_panel_hotkeys` (hotkeys.rs) -- line 946
6. `right_click_cancel_submenu` (hotkeys.rs) -- line 1223
7. `update_command_panel_progress` (actions.rs) -- line 1817
8. `resolve_pointer_display_type` (pointer.rs) -- line 2953

### Line ranges for each submodule (from current command_panel.rs)

**grid_layout.rs** (lines 1-202, ~200 code + ~200 test lines):
- `GRID_HOTKEYS` constant (line 19)
- `keycode_to_grid_slot()` (line 26)
- `get_grid_slot_action()` (line 43, ~160 lines)
- Tests: sections starting at line 3073 (conditional tests), 3249 (completeness), 3567 (StructureMenuState grid), 3604 (Tunnel Interface State), 3891 (Agent Interface), 4355 (HeadquartersMenu), 4530 (Inert Structure), 4640 (Standard Bottom-Row), 4864 (DC grid), 4932 (EF grid), 4970 (Mixed unit+structure), 5482 (RC menu grid), 5555 (SupplyChopper), 6174 (Armory), 6264 (Cults Recruit)

**state.rs** (lines 203-515, ~310 code + ~300 test lines):
- `update_cursor_target()` (pub, line 203)
- `selection_owned_by_local_player()` (line 292) -- called by state.rs and ui_rebuild.rs; make `pub(super)`
- `is_panel_visible()` (line 310) -- called by ui_rebuild.rs, hotkeys.rs; make `pub(super)`
- `update_command_panel_state()` (pub, line 325)
- Tests: line 3313 (CursorTarget), 3346 (is_panel_visible), 4106 (is_panel_visible), 4189 (selection_owned_by_local_player), 3281 (ObjectInterfaceState)

**ui_rebuild.rs** (lines 516-914, ~400 code + ~100 test lines):
- `rebuild_command_panel_ui()` (pub, line 516)
- `spawn_panel_title()` (line 681) -- only called within ui_rebuild; keep private or `pub(super)`
- `spawn_progress_text()` (line 694)
- `spawn_info_text()` (line 707)
- `spawn_grid_button()` (line 720)
- `spawn_empty_grid_cell()` (line 774)
- Calls into: `is_panel_visible` (state), `get_player_sc_from_owners` (helpers), `get_grid_slot_action` (grid_layout), `grid_button_label` (helpers), `grid_button_enabled_ext` (helpers), `is_action_active` (helpers), `is_common_command` (helpers), `spawn_grid_button`/`spawn_empty_grid_cell` (self), `build_tunnel_expand_grid`/`build_tunnel_eject_grid` (tunnel_actions), `get_tunnel_upgrade_cost`/`get_syndicate_supplies` (tunnel_actions), `get_dc_construction_info` (helpers)
- Tests: 3848 (panel_visible with Tunnel), 5022 (DC/EF State Preservation)

**hotkeys.rs** (lines 915-1259, ~340 code + ~400 test lines):
- `handle_command_button_clicks()` (pub, line 915) -- calls `execute_command_action` (actions)
- `command_panel_hotkeys()` (pub, line 946) -- calls `is_panel_visible` (state), `keycode_to_grid_slot` (grid_layout), `get_grid_slot_action` (grid_layout), `execute_command_action` (actions)
- `right_click_cancel_target()` (line 1172) -- called by hotkeys and right_click_cancel_submenu
- `right_click_cancel_submenu()` (pub, line 1223) -- calls `right_click_cancel_target` (self)
- Tests: 5381 (Right-click cancel submenu)

**actions.rs** (lines 1260-1891, ~630 code + ~600 test lines):
- `execute_command_action()` (line 1260) -- the 560-line dispatcher. Called by hotkeys.rs; make `pub(super)`
- `update_command_panel_progress()` (pub, line 1817)
- Calls into: `find_player_resources_mut` (helpers), `find_syndicate_resources_mut` (helpers), `active_group_entities` (helpers), `command_target_entities` (helpers), `execute_tunnel_upgrade`/`execute_tunnel_cancel_upgrade`/`execute_tunnel_select_expansion`/`execute_tunnel_eject_unit` (tunnel_actions)
- Tests: 4262 (Supply Tower tech), 4296 (Schedule Deliveries gate), 4322 (Selection bounds), 4334 (Air domain), 4572 (Common command visual), 5191 (Context-Aware Build Menu), 5219 (Placement flip), 5244 (EjectRequest), 5258 (Eject button), 5284 (Tunnel transit tier), 5331 (Tunnel upgrade cost), 5367 (Ejection queue cooldown), 5639 (ChopperDropOffSupplies)

**tunnel_actions.rs** (lines 1990-2343, ~380 code + ~200 test lines):
- `execute_tunnel_upgrade()` (line 1990) -- called from actions.rs; make `pub(super)`
- `execute_tunnel_cancel_upgrade()` (line 2054) -- called from actions.rs
- `execute_tunnel_select_expansion()` (line 2108) -- called from actions.rs
- `execute_tunnel_eject_unit()` (line 2150) -- called from actions.rs
- `build_tunnel_expand_grid()` (line 2162) -- called from ui_rebuild.rs
- `build_tunnel_eject_grid()` (line 2223) -- called from ui_rebuild.rs
- `tier_at_least()` (line 2288)
- `get_tunnel_upgrade_cost()` (line 2299) -- called from ui_rebuild.rs
- `get_syndicate_supplies()` (line 2327) -- called from ui_rebuild.rs
- Tests: 3744 (tier_at_least), 3773 (grid_button_label for Tunnel), 3820 (EjectionQueue), 3878 (is_common_command for Tunnel)

**pointer.rs** (lines 2780-3008, ~230 code + ~1,200 test lines):
- `CursorEntityInfo` struct (line ~2783)
- `resolve_pointer_display()` (line 2796) -- pure function
- `resolve_awaiting_target()` (line 2829) -- helper
- `resolve_default_state()` (line 2890) -- helper
- `resolve_pointer_display_type()` (pub, line 2953) -- system; calls `active_group_entities` (helpers), `resolve_pointer_display` (self)
- Tests: 5692-6173 (resolve_pointer_display tests), 6176-6357 (additional tests)

**helpers.rs** (lines 1893-2780 minus tunnel_actions and pointer sections, ~350 code + ~350 test lines):
- `compute_selected_unit_capabilities()` (line 1900) -- called from state.rs; make `pub(super)`
- `attack_type_can_target_ground()` (line ~1949 area)
- `get_player_sc_from_owners()` (line 1949) -- called from ui_rebuild.rs, actions.rs
- `find_player_resources_mut()` (line 1964) -- called from actions.rs
- `get_dc_construction_info()` (line 1977) -- called from ui_rebuild.rs
- `find_syndicate_resources_mut()` (line 2419) -- called from actions.rs
- `grid_button_enabled_ext()` (line 2344) -- called from ui_rebuild.rs
- `grid_button_enabled()` (line 2627)
- `grid_button_label()` (line 2551) -- called from ui_rebuild.rs
- `is_action_active()` (line 2659) -- called from ui_rebuild.rs
- `is_unit_action()` (line 2432)
- `object_type_supports_action()` (line 2457)
- `is_common_command()` (line 2512) -- called from ui_rebuild.rs
- `active_group_entities()` (line 2524) -- called from actions.rs, pointer.rs
- `command_target_entities()` (line 2532) -- called from actions.rs
- Tests: 3015 (attack_type_can_target_ground), 3054 (SelectedUnitCapabilities), 3199 (is_action_active), 3236 (grid_button_label), 3377 (is_common_command)

### Cross-submodule dependency map (callee visibility must be `pub(super)` or `pub(crate)`)

```
hotkeys.rs      -> state.rs (is_panel_visible)
                -> grid_layout.rs (keycode_to_grid_slot, get_grid_slot_action)
                -> actions.rs (execute_command_action)
ui_rebuild.rs   -> state.rs (is_panel_visible)
                -> helpers.rs (get_player_sc_from_owners, grid_button_label, grid_button_enabled_ext, is_action_active, is_common_command, get_dc_construction_info)
                -> grid_layout.rs (get_grid_slot_action)
                -> tunnel_actions.rs (build_tunnel_expand_grid, build_tunnel_eject_grid, get_tunnel_upgrade_cost, get_syndicate_supplies)
actions.rs      -> helpers.rs (find_player_resources_mut, find_syndicate_resources_mut, active_group_entities, command_target_entities)
                -> tunnel_actions.rs (execute_tunnel_upgrade, execute_tunnel_cancel_upgrade, execute_tunnel_select_expansion, execute_tunnel_eject_unit)
state.rs        -> helpers.rs (compute_selected_unit_capabilities)
pointer.rs      -> helpers.rs (active_group_entities)
```

### Imports pattern
Each submodule needs its own `use` block. Copy the relevant subset from the top of command_panel.rs (lines 1-16). The main imports are:
- `use bevy::prelude::*;`
- `use crate::types::*;` (brings in `ObjectEnum`, `Unit`, `Owner`, `Selected`, `Selection`, `SelectionGroup`, `PlacementState`, etc.)
- `use crate::game::types::{...};` (structure states, tunnel types, player resources)
- `use crate::game::units::types::commands::{CommandType, HoldingPosition, UnitCommand, CommandQueue};`
- `use crate::game::units::types::state::AgentCarryState;`
- `use crate::game::units::types::movement::{MoveTarget, Path, Velocity};`
- `use crate::game::combat::types::{AttackState, AttackCapability, AttackType};`
- `use crate::game::world::types::{SpaceCrystalPatch, SupplyDeliveryStation};`
- `use super::super::types::*;` (brings in `ObjectInterfaceState`, `CommandPanelTarget`, `CursorOverUi`, `CursorTarget`, `SelectedUnitCapabilities`, `PointerDisplayType`, `CommandButtonAction`, etc.)

Within submodules, cross-references use `use super::submodule::function_name;` -- for example, in hotkeys.rs: `use super::actions::execute_command_action;`

### mod.rs structure
```rust
mod grid_layout;
mod state;
mod ui_rebuild;
mod hotkeys;
mod actions;
mod tunnel_actions;
mod pointer;
mod helpers;

pub use state::{update_cursor_target, update_command_panel_state};
pub use ui_rebuild::rebuild_command_panel_ui;
pub use hotkeys::{handle_command_button_clicks, command_panel_hotkeys, right_click_cancel_submenu};
pub use actions::update_command_panel_progress;
pub use pointer::resolve_pointer_display_type;
```

### Test distribution
The single `#[cfg(test)] mod tests` block (lines 3009-6357) must be split into per-submodule test blocks. Each submodule gets `#[cfg(test)] mod tests { use super::*; ... }`. Test sections are clearly delimited with `// === section name ===` comments. Match each section to the function it tests and place it in that submodule. Tests that reference functions from other submodules will need `use super::super::other_module::function;` or rely on the module re-exports.

### Key implementation notes
1. **Delete command_panel.rs BEFORE creating command_panel/ directory** -- Rust cannot have both `command_panel.rs` and `command_panel/mod.rs`
2. **`pub(super)` visibility** for all cross-submodule functions -- they are not public API, only used within the command_panel module
3. **Test imports**: Current tests use `use super::*;` which brings in everything from command_panel.rs. After split, each test module uses `use super::*;` to get its own submodule's items, plus explicit imports for items from sibling submodules
4. **No changes to ui/mod.rs** -- Rust resolves `mod command_panel;` to `command_panel/mod.rs` automatically
5. **attack_type_can_target_ground** (helpers.rs line ~1949 area) needs `use crate::game::combat::types::FullyConnectedSubtype;` -- this is imported via `crate::types::*` in the current file but may need explicit import in helpers.rs depending on re-exports

## Dependencies

None -- this is a standalone structural refactoring task with no behavioral changes. The subsequent `command_panel_action_decomposition` task depends on this task being completed first (it assumes actions.rs exists as a separate file).
