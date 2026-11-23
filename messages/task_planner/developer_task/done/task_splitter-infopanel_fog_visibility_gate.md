# infopanel_fog_visibility_gate

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-infopanel_fog_of_war_visibility.md

## Task

Add fog-of-war visibility gating to InfoPanel fields in ui/hud.rs.

Currently, update_selected_units_grid_system renders all InfoPanel fields for SpaceCrystalPatch and SupplyDeliveryStation regardless of fog state. Two fields must be gated:

1. SpaceCrystalPatch RemainingAmount (line ~729-733): Only display when the patch is in VisibilityStateEnum::Visible for the local player. When in Explored state, replace with a placeholder (e.g., '???') or omit the remaining/initial/percentage lines.

2. SupplyDeliveryStation CurrentSupplies (line ~807-812): Only display when the SDS is in VisibilityStateEnum::Visible. DeliverySize and DeliveryInterval always display. When in Explored state, omit the CurrentSupplies line and the 'Next delivery' countdown.

Implementation:
- Add FogOfWarMap as a Res parameter to update_selected_units_grid_system
- Add GridPosition to the selected_scp and selected_sds query tuples
- Add a query or resource for the local player's PlayerNumber (needed to index into FogOfWarMap)
- Before rendering gated fields, look up the entity's grid position in FogOfWarMap for the local player. If the tile is not Visible, skip those fields or show '???'
- The ExtractionPlate info panel (line ~487-500) also shows remaining_amount from the attached patch — apply the same visibility gate there (check the EP's own grid position)

Reference: artifacts/designer/design/entities.md lines 197-207 for the InfoPanel visibility rules.
