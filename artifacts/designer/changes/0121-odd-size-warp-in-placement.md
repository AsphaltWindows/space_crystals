# 0121 — Warp In of odd-sized buildings: fixed default, rotate/flip picks the corner

- **Date**: 2026-11-24T14:30:00-05:00
- **Answers**: Q0040 (point 5)
- **Docs**: `factions/colonists/economy.md > Warp In > Warp In Command`; `core/entities.md > Placement Validation > Warp In`

## What changed and why
Neither of the offered options exactly (user's rule): an odd-sized footprint (3x3 Bio Portal on a 2x2 Beacon) starts with its extra row/column to the **bottom-right**. Rotate and flip transform the ghost around the Beacon's centre, so the extra tiles move with it (H flip: left/right; V flip: top/bottom; 90° rotation: next corner). Combined, every corner is reachable with either building orientation. No cursor-based snapping.

## Implementation impact
Ghost footprint = default offset footprint transformed by the current rotation/flip about the Beacon centre. Replaces the Director's proposed nearest-to-cursor snapping. "Bottom-right" was the designer's example default, accepted with the option; trivially changeable.

## Verification
- Select a Beacon, Warp In Bio Portal: the ghost covers the Beacon plus one extra column right and one extra row below.
- H flip: extra column moves left. V flip: extra row moves up. Rotate 90°: extra tiles move to the next corner, and exit sides rotate accordingly.
- Moving the mouse doesn't change the ghost's position.
