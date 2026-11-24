# 0098 — Plates pay in lumps

- **Date**: 2026-11-24T04:40:00-05:00
- **Answers**: Q0028 (point 1; points 2–4 still open)
- **Docs**: `factions/gdo/economy.md > ExtractionPlate` (new "Paid in lumps" paragraph)

## What changed and why
The user chose A. A plate delivers its whole payment at once at the end of each 48-frame cycle: +10, or +1 at the residual rate. A new plate pays first 48 frames after it is placed.

Under the power penalty, the cycle progresses at the power ratio. The lumps then come less often, but stay the same size. The Designer's reading of "reduced proportionally" is that the cycle slows; the amount per lump doesn't change.

The baseline was computed as a trickle, so its timings shift by about 2 s. For example, the fastest clean Barracks starts at ≈48 s. This is accepted.

## Implementation impact
- Matches the Director's proposal. Economy tests should expect lump timings.
- Power penalty: the cycle timer advances by the power ratio each frame.

## Verification
1. Place a plate. Crystals rise by exactly 10 at 48 frames, then every 48 frames after that.
2. At −21 power (≈59%), the payments come every ≈81 frames.
