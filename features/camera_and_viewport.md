# Feature: Camera and Viewport

## Overview
Camera behavior and viewport configuration. Defines the fixed zoom level, visible game area, and HUD screen layout.

## Design Sources
- `design/camera.md`

## Specifications

### Zoom
- **Fixed zoom**: The camera displays exactly 28 GridUnits horizontally (1792 SpaceUnits).
- No zoom in or zoom out controls are available.

### Vertical Coverage
- The number of GridUnits visible vertically is determined by:
  - Screen aspect ratio
  - Minus vertical space occupied by HUD elements (top resource bar + bottom panel)

### HUD Layout
- **Top**: Resource bar
- **Bottom**: Info / Selection / Command panel
- **Sides**: No side panels

## Dependencies
- `simulation_core` (GridUnit definition, SpaceUnitsPerGridUnit ratio)
- `control_system` (CommandPanel, SelectionPanel, InfoPanel rendered in bottom HUD area)

## Open Questions
- Panning: How does the camera move? Arrow keys? Edge scrolling? Middle-mouse drag? (deferred per design)
- Minimum window resolution: Is there a minimum supported resolution, or does the game scale the 28 GridUnit constraint to any width?
- HUD sizing: Are HUD elements fixed-pixel height or proportional to window size? Affects vertical GridUnit coverage calculation.
