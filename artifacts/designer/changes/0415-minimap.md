# 0415 — The Minimap (the user asks for it to be prioritised)

- **Date**: 2026-11-26T16:55:00-05:00
- **Answers**: —
- **Docs**: `systems/controls/hud.md > Minimap` (new)

## What changed and why
The Minimap slot had no spec beyond "Always visible". The user confirmed a design and **asked for the minimap to be prioritised**.
- **Shows**: simplified terrain; objects in player colour (buildings as footprint squares, units as dots); Space Crystal patches and neutral objects in a neutral colour; fog of war exactly as the main map (Unexplored black, Explored dimmed with remembered buildings/patches only, live objects only where Visible; hidden objects hidden); the camera's view as a rectangle.
- **Use**: left-click / drag centres the camera; right-click = the selection's ground right-click at that point (Move, etc.); in AwaitingTarget a left-click targets that ground point (e.g. Attack Move). Designer's placeholders: the minimap targets only ground, and placement isn't possible from it.
- Attack alerts on the minimap: not decided yet.

## Implementation impact
New: the whole Minimap panel. Priority requested by the user.

## Verification
- The minimap shows the whole map; units and buildings appear in player colours; crystal patches in a neutral colour.
- Enemy units appear only where you currently see; a remembered enemy building stays as last seen in Explored areas; unexplored areas are black.
- The rectangle follows the camera as you pan; clicking the minimap moves the camera there; dragging pans continuously.
- Select units, right-click the minimap: they move to that spot. Press A, left-click the minimap: they Attack Move there.
