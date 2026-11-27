# 0451 — Newly released vehicles park around a taken rally point

- **Date**: 2026-11-27T12:35:00-05:00
- **Answers**: Q0177
- **Docs**: `core/entities.md > Entrance and exit > Exit Doorway`, point 7 (new "Parking around a taken rally point")

## What changed and why
The user chose option 3. When a newly released vehicle's rally point is already taken by parked vehicles, it parks at the **nearest free spot around the point** where it fits. It no longer stops in a line behind the last vehicle, back toward the door. More vehicles then fit before the doorway stalls, whatever the rally distance or the vehicle's length.

The user also wants a **warning when an exit stays blocked**. It is pinned until the notification system is designed. For now the only alert is the off-screen damage alert in `hud.md > Alerts`.

## Implementation impact
- **New:** the rally move of a newly released vehicle ends at the free spot nearest the rally point when the point is taken. This applies at any rally distance: the default 3 GU (unchanged) or one the player set.
- **Unchanged:**
  - infantry, which yield;
  - ordinary player-issued Moves;
  - idle vehicles still don't yield (0440).
- **Not yet:** the exit-blocked notification. A stall stays silent for now.

## Verification
1. Leave a Chop Shop's default rally point and queue five Seekers. They gather in a cluster around the point, and all five leave the building without the door stalling.
2. Do the same with Hands of Heaven. They cluster as well.
3. Set a rally point farther away. The vehicles cluster around it there.
