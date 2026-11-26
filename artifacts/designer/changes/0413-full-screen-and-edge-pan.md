# 0413 — Full screen by default; the cursor stays in the window; 8 px pan margin

- **Date**: 2026-11-26T16:35:00-05:00
- **Answers**: — (playtest feedback from the user)
- **Docs**: `core/camera.md > Panning > Arrow keys and screen edge`; `core/camera.md > Window Mode` (new)

## What changed and why
Playtest: in a window, moving the cursor up to pan slid it into the title bar and panning stopped.
- **Full screen by default** (user): the game starts in **borderless full screen**. **F11** or **Alt+Enter** toggles full screen / window (keys placeholder).
- **Windowed play** (designer's pick at the user's request):
  - **The cursor stays in the game window** while the game is the active window, in full screen too (several monitors). Losing focus (Alt-Tab etc.) frees it; clicking back confines it again.
  - **Scroll margin**: edge panning starts within **8 px** of the window's edge (placeholder).

## Implementation impact
New: window mode at startup, a toggle key, cursor confinement while focused, and the edge-pan margin. Pan speed is still TBD.

## Verification
- Launch: the game fills the screen with no window borders.
- F11 (or Alt+Enter): it becomes a window; again: full screen.
- In a window, push the cursor up: it stops at the top edge and the map keeps panning; the title bar is unreachable. Alt-Tab: the cursor is free.
- Hold the cursor 5 px from an edge (not touching): the map pans.
- On two monitors in full screen, the cursor can't move onto the other screen while the game is active.
