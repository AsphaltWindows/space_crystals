# 0162 — Feedback when pressing an unavailable command

- **Date**: 2026-11-24T20:20:00-05:00
- **Answers**: Q0035 (point 3; Q0035 done)
- **Docs**: `systems/controls/selection_and_hud.md > CommandPanel` (Pressing an unavailable command)

## What changed and why
Option B. Pressing a greyed-out button or its hotkey issues nothing but gives the refusal feedback with its reason ("Not enough Space Crystals", "Queue is full"), via the same channel as invalid target clicks.

## Implementation impact
Hook greyed-out presses to the existing refusal-message path. Form (text/sound) TBD with notifications.

## Verification
With 20 SC, press Q on a Barracks: "Not enough Space Crystals" appears; nothing queues.
