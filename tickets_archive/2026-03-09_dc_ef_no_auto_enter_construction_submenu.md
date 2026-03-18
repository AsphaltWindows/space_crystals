# Ticket: DC/EF should not auto-enter construction sub-menu on selection

## Current State
When a player selects a Deployment Center or Extraction Facility that has an active construction (`current_construction.is_some()`) or a ready-to-place building (`ready_to_place.is_some()`), the `update_command_panel_state` system forces the interface into the construction/ready sub-state every frame. This means:
- Selecting a constructing DC immediately puts the player in the DcConstructing sub-menu (showing Back/Cancel) instead of DefaultState.
- Selecting a constructing EF immediately puts the player in EfConstructing instead of EfIdle.
- Pressing Z/Escape to go back to DefaultState/EfIdle is immediately overridden on the next frame by the auto-sync.

This contradicts the feature spec, which defines DefaultState as the entry point when selecting a structure, with the player choosing to navigate into sub-menus.

## Desired State
Selecting a DC or EF always lands on DefaultState (DcIdle or EfIdle), regardless of construction state. The player navigates into build sub-menus voluntarily via the Build command (Q). The `update_command_panel_state` system should not force construction sub-states based on instance state — it should only set the initial state when the structure is first selected (or when the interface state is unset), not override it every frame.

The `DcConstructing`/`EfConstructing` sub-states should only be entered when the player initiates construction (via a build command), not re-imposed by the frame-by-frame sync.

## Justification
- The feature spec (`features/gdo_objects.md`) defines DC DefaultState as the entry point with Build (Q) navigating to BuildMenu. Auto-entering construction sub-state contradicts this.
- EF spec similarly defines DefaultState as the landing state with conditional commands based on instance state.
- Forum topic `dc_ef_construction_ux.md` — all respondents (QA, project_manager, product_analyst, task_planner) agree this is an implementation bug, not a spec gap. Task_planner confirmed the code location (`update_command_panel_state` at `command_panel.rs:319-320` for DC, `command_panel.rs:350` for EF).

## QA Steps
1. [human] Build a Power Plant from the DC and while it is constructing, deselect and re-select the DC — verify the interface shows DefaultState (DcIdle with Build Q and Cancel X), not the DcConstructing sub-menu.
2. [human] While DC is constructing, press Q to enter BuildMenu — verify you now see the Constructing sub-menu with Cancel (X) and Back (Z).
3. [human] Press Z or Escape from the BuildMenu Constructing view — verify you return to DefaultState (DcIdle) and stay there (not re-forced into sub-menu).
4. [human] Let a DC construction complete to ready-to-place state, deselect and re-select the DC — verify DefaultState is shown (with Cancel X and Build Q), not the ready-to-place sub-menu.
5. [human] Build an Extraction Plate from the EF and while it is constructing, deselect and re-select the EF — verify EfIdle is shown (with Cancel X), not EfConstructing.
6. [human] Press Z or Escape from EfConstructing — verify you return to EfIdle and stay there.

## Expected Experience
Selecting a constructing structure feels natural — the player sees the default command panel showing what's being built (with a Cancel shortcut right there), rather than being dumped into a sub-menu. Navigation into build menus is always the player's choice, never forced. Z/Escape reliably returns to the default view without being immediately overridden.
