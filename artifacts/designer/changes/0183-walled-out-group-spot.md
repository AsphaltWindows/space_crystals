# 0183 — Walled out of a group spot: the arrival radius covers it

- **Date**: 2026-11-24T23:45:00-05:00
- **Answers**: Q0018 (point 6; Q0018 done)
- **Docs**: — (covered by `systems/controls/behaviors.md > MovingToLocation > Occupied target location`, now 3 GU per 0169)

## What changed and why
Confirmed acceptable: a unit walled out of its magic-box spot by its own group counts as arrived within the occupied-arrival radius (3 GU since 0169) and gives up beyond it. No extra rule.

## Implementation impact
None beyond 0169.

## Verification
—
