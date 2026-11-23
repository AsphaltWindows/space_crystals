# sds_infopanel_visibility_rework

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-fog_of_war_selectability.md

## Task

Rework the SupplyDeliveryStation InfoPanel to be visibility-context-aware, and ensure NextDeliveryTime is always displayed (not gated behind CurrentSupplies==0).

### Current State

In `ui/hud.rs` (~line 810-831), the SDS InfoPanel currently shows:
- DeliverySize and DeliveryInterval: always shown
- CurrentSupplies: shown only when tile is Visible
- NextDeliveryTime (`time_until_next_delivery`): shown only when Visible AND CurrentSupplies==0

The design requires NextDeliveryTime to be shown in BOTH Visible and Explored contexts, and CurrentSupplies only in Visible context.

### Required Changes

1. **InfoPanel display logic** (`update_selected_units_grid_system` in `hud.rs`, SDS single-selection branch ~line 810-831):
   - Add `SelectedFaction` resource to the system parameters
   - Get the fog state for the SDS tile position
   - **Visible** (any player): show DeliverySize, DeliveryInterval, NextDeliveryTime, CurrentSupplies (all four fields)
   - **Explored** (GDO player only -- this is the only case where Explored SDS is selectable): show DeliverySize, DeliveryInterval, NextDeliveryTime (three fields, no CurrentSupplies)
   - NextDeliveryTime should always be displayed as a countdown regardless of CurrentSupplies value (remove the `if sds.current_supplies == 0` gate)
   - Format: `Next delivery: {:.1}s` as currently done

2. **Note on Explored+non-GDO and Unexplored**: These cases will never reach the InfoPanel because the selection system (modified in the companion task) prevents selection. No defensive coding needed for these cases, but the code should be clean and not show stale data.

3. **Tests**: Add/update tests verifying:
   - SDS InfoPanel in Visible state shows all 4 fields
   - SDS InfoPanel in Explored state (GDO) shows 3 fields (no CurrentSupplies)
   - NextDeliveryTime is shown regardless of CurrentSupplies value
