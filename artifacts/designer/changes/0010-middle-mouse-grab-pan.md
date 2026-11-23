# 0010 — Camera: middle-mouse grab-and-drag panning

- **Date**: 2026-11-23T13:29:12-05:00
- **Answers**: Q0002 (point 3, partly: middle-mouse panning. Still open: map modifier clicks, control-group keys, arrow/edge pan + speed, queue modifier, rebinding)
- **Docs**: `core/camera.md > Panning > Middle-mouse grab` (new), `> Other panning inputs` (TBD placeholder), `> Centering` (existing text, now under its own heading)

## What changed and why
The user decided that holding the middle mouse button **grabs the map**. While held, the grabbed point stays under the cursor, so the map follows the mouse like dragging a page. Releasing the button stops panning. It's not joystick-style scrolling.

## Implementation impact
- New input: middle-mouse drag pans the camera 1:1 with the cursor movement, measured in world space at the fixed zoom.
- Arrow keys and screen-edge panning are not decided yet. Keep the current assumption (SC2-like) until a follow-up entry.

## Verification
- Press and hold the middle mouse button over a crystal patch, then move the mouse: the patch stays exactly under the cursor while the view moves. Releasing stops movement immediately, with no drift.
