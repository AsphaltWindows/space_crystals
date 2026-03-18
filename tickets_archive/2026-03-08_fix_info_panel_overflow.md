# Ticket: Fix info panel vertical overflow in HUD

## Current State
When selecting a unit or building, the info panel in the bottom HUD area extends vertically beyond the bounds of its container. The panel content is not properly clipped or sized to fit within the allocated HUD area.

## Desired State
The info panel content must be fully contained within its HUD container. Content that exceeds the available vertical space should be clipped or constrained (e.g., via `Overflow::clip()` or by constraining the node's height). The panel should never visually extend beyond the bottom HUD region.

## Justification
The bottom HUD area is specified in `features/camera_and_viewport.md` (line 22) as the region for "Info / Selection / Command panel." The InfoPanel is referenced in `features/control_system.md` as the display for single-unit selection info. The panel overflowing its container is a layout bug that breaks the intended HUD structure.

Bug reported via forum topic `info_panel_overflow.md`.

## QA Steps
1. [human] Launch the game and select a single unit or building.
2. [human] Observe the info panel in the bottom HUD area — verify all content is contained within the panel's visual bounds.
3. [human] Select different unit/building types (especially those with more info fields) and verify none cause vertical overflow.
4. [human] Resize the window to a smaller size and verify the info panel still respects its container bounds.

## Expected Experience
The info panel should display unit/building information entirely within the bottom HUD container. No text, icons, or panel background should extend beyond the HUD boundary. The panel content should be visually clipped or scrollable if it exceeds the available space.
