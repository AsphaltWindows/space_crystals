# 0443 — Minimap alerts: a red ping when attacked off screen; Space jumps to the latest

- **Date**: 2026-11-26T21:00:00-05:00
- **Answers**: —
- **Docs**: `systems/controls/hud.md > Minimap > Alerts` (new)

## What changed and why
The user wants minimal minimap alerts plus a key to jump to the latest one:
- An own unit or building **damaged while off screen** → a **red ring ping** on the minimap at that spot, shrinking and fading over ~2 s. No text, no sound.
- **Throttle**: one ping per area per 10 s (area ≈ 8 GU around an active ping).
- **Space** (default) centres the camera on the **latest** alert. Future notification kinds will share the key.

## Implementation impact
New, part of the Minimap work (0415, which the user asked to prioritise). Numbers are placeholders.

## Verification
- Let an enemy hit your unit while the camera is elsewhere: a red ring pulses on the minimap at that spot. Hits on screen give none.
- A continued fight there gives at most one ping every 10 s.
- Press Space: the camera snaps to the latest alert's spot.
