# infopanel_fog_of_war_visibility

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

The InfoPanel must enforce per-field fog-of-war visibility rules as specified in the design documents. Currently, all InfoPanel data is shown regardless of whether the object is in the Visible state for the selecting player.

Per artifacts/designer/design/entities.md:
- SpaceCrystalsPatch InfoPanel: RemainingAmount should only display 'when visible to the selecting player'
- SupplyDeliveryStation InfoPanel: CurrentSupplies should only display 'when visible to the selecting player' (DeliverySize and DeliveryInterval always display)

The fix should apply a visibility check when rendering InfoPanel fields that are gated by visibility. For any object in an Explored (non-visible) area, visibility-gated fields should be hidden or show a placeholder (e.g., '???'). For objects in Visible areas, all fields display normally.

This affects the InfoPanel rendering system (likely ui/hud.rs based on forum discussion). Each InfoPanel field definition in the design docs specifies whether it requires visibility — the implementation must respect those per-field rules.

## QA Instructions

1. Start a game as GDO. Scout an area with Space Crystal Patches and at least one Supply Delivery Station using a unit.
2. Move the unit away so the area transitions from Visible to Explored (fog of war covers it, but terrain and structures remain shown in last-known state).
3. Click on a SpaceCrystalsPatch in the Explored area. If it is selectable, the InfoPanel should NOT display the RemainingAmount value. It should show a placeholder or simply omit the field.
4. Click on a SupplyDeliveryStation in the Explored area. The InfoPanel should display DeliverySize and DeliveryInterval (always visible) but should NOT display CurrentSupplies.
5. Move a unit back to make the area Visible again. Click the same objects — now all fields should display with current values.
6. Verify the same behavior with any other object types that have visibility-gated InfoPanel fields.
