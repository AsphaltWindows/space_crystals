# 0209 — Laser Battery: Amplify trade as a tuning dial; narrow turret arc

- **Date**: 2026-11-25T04:35:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Laser Battery > Amplify`, `> TurretAttributes (mobile)`

## What changed and why
- **Amplify** (user): the efficiency loss is a reasonable trade, and its size depends on the numbers. If a full battery holds N normal surges, Amplify deals N^(−1/3) of the total: about 63% for N = 4, 50% for N = 8, about 33% for N = 27. The normal-surge share of the battery is the dial.
- **Mobile turret** (user): a narrow arc, so the tank must roughly face its target. Rotation within the arc is not singled out as slow.

## Implementation impact
Concept only; values TBD.

## Verification
—
