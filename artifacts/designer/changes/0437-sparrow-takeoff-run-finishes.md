# 0437 — A Sparrow's takeoff run always finishes

- **Date**: 2026-11-26T20:10:00-05:00
- **Answers**: Q0170
- **Docs**: `factions/cults/units/sparrow.md > Placeholder stats > Flight rules` (Takeoff)

## What changed and why
The user chose option 1: the takeoff run is a fixed sequence. Any order given during it (Move, Attack, Attack Move, Stop, Hold Position) takes effect when the run ends past the runway's C end, replacing the rally point's order; Stop then circles there. Consistent with held orders during other take-offs (Airfield, Prism unfolding).

## Implementation impact
None: matches M5-N7 as built.

## Verification
Select a Sparrow as it appears and order it back over the base: it completes the run to C, then turns toward the order.
