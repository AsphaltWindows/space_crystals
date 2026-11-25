# 0259 — The Syndicate economy check

- **Date**: 2026-11-25T13:20:00-05:00
- **Answers**: Q0070
- **Docs**: `factions/syndicate/units.md > Agent > Gathering > Space Crystals` (Syndicate economy check)

## What changed and why
The Director's proposals are accepted, with the user's layout correction:
- **Distance** is measured edge to edge, patch to the crystal **hatch** (the 1-square hatch from 0255).
- **Tolerance:** ±15%; the 12.5 SC/s ceiling must be exact.
- **Targets:** the opening at 0:34–0:46 (patch 1 GU from the hatch); the table rows; stacked Tunnels double intake; the Syndicate has the lowest one-base saturation.
- **Layout:** one Tunnel facing a single patch per distance. **Stacked check:** the two Tunnels stand on **opposite sides of the field**, so each Agent's nearest Tunnel is its own (nearest-Tunnel delivery, 0255).
- **User guidance for map makers:** don't encourage stacking Tunnels on one field; it remains allowed.

## Implementation impact
- The oracle measures to the hatch square.
- The stacked layout places the Tunnels on opposite sides.

## Verification
The oracle reports the table at ±15% and exactly 12.5 SC/s per saturated Tunnel.
