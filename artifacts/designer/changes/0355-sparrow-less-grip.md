# 0355 — Sparrow: less grip (MaxCentripetalAcceleration 1 → 0.5)

- **Date**: 2026-11-26T23:30:00-05:00
- **Answers**: —
- **Docs**: `factions/cults/units/sparrow.md > Placeholder stats`

## What changed and why
User: "Seems like it has too much grip." The placeholder MaxCentripetalAcceleration goes from 1 to **0.5 su/frame²**:
- the full-speed turn radius doubles to about 200 su (about 3 GU), with a loop in about 8 s;
- at IdleSpeed it would be 32 su, so the fixed 1 GU idle circle (0354) stays.

## Implementation impact
One number.

## Verification
- Strafing passes swing out about 3 tiles before coming back.
