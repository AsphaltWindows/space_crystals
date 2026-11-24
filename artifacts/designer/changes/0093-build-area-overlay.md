# 0093 — Build area overlay shows the area only

- **Date**: 2026-11-24T03:55:00-05:00
- **Answers**: Q0026 (point 4; Q0026 is now fully answered)
- **Docs**:
  - `factions/gdo/structures.md > DeploymentCenter > AwaitingPlacement` (removed "showing all valid build cells")
  - `factions/gdo/economy.md > ExtractionFacility > AwaitingPlacement`

## What changed and why
The user chose A. During placement, the overlay shows the **GDO build area only**. Whether a tile is valid right now (flat, Visible, unoccupied, not a resource) is shown per tile on the ghost. This keeps the overlay as one clean shape, which doesn't flicker as units move.

## Implementation impact
- Matches the Director's proposal. Remove any per-tile validity holes from the overlay.

## Verification
While placing a Generator, the overlay is a solid square shape. A unit standing inside it doesn't make a hole. The ghost turns its tile red when hovered there.
