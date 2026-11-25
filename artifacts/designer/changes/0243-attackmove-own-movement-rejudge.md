# 0243 — AttackMove: the unit's own movement also re-checks a rejected enemy

- **Date**: 2026-11-25T10:25:00-05:00
- **Answers**: Q0074
- **Docs**: `systems/controls/behaviors.md > AttackMovingToLocation > Algorithm` (Re-checking a rejected enemy)

## What changed and why
User chose **B** (to try): a reach-rule rejection is also re-checked once the unit has moved about 1 GU (placeholder) from where it judged that enemy. The same situation then always gets the same response: a Guard that rounds a wall attacks the building just as a fresh Guard would. Failure-rule and fog rejections keep 0230's triggers.

## Implementation impact
- Add the own-movement trigger.
- **Measure the cost** with large armies passing many unreachable targets. If it is too expensive, report back and we'll explore other options (e.g. a larger step).

## Verification
A Guard attack-moves along a wall past a building it rejected. After rounding the wall into the open, with the building reachable within leash and sight, it attacks.
