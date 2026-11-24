# 0120 — Worker-built ghost over fog judges by player knowledge

- **Date**: 2026-11-24T14:20:00-05:00
- **Answers**: Q0040 (point 4)
- **Docs**: `core/entities.md > Placement Validation > Worker-Built Structures`

## What changed and why
The Director's proposal. A worker-built ghost (Drone Deploy, Agent, Recruit builds) is green if, as far as the player knows, nothing blocks it: remembered terrain and structures on Explored tiles; hidden units ignored; never-explored tiles pass. The order is always allowed outside vision; the true validation happens on arrival.

## Implementation impact
Ghost validity reads the player's map memory, never the true state of non-visible tiles (no info leak).

## Verification
- Ghost over a never-explored area: green; the Drone walks there and validates on arrival.
- Ghost over a fogged area containing a remembered enemy building: red there.
- A hidden enemy unit standing in fog on the site: ghost still green; the order fails on arrival.
