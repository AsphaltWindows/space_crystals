# 0182 — Targets inside MinRange

- **Date**: 2026-11-24T23:40:00-05:00
- **Answers**: Q0018 (point 5)
- **Docs**: `systems/controls/behaviors.md > BaseAutoTargeting > Targets inside MinRange` (new); `> AttackingObject > Target inside MinRange` (new)

## What changed and why
- Auto-targeting skips enemies inside MinRange and picks a hittable one.
- Explicit Attack on a too-close target: mobile units back off until outside MinRange, then attack; immobile ones (deployed Artillery Rig) fail the order with "Target too close" feedback.

## Implementation impact
Replaces "attack fails and the order gives up" for mobile units.

## Verification
Mobile MinRange unit ordered to attack an adjacent enemy: it backs away, then fires. Deployed Artillery Rig: feedback, order ends. Auto-targeting ignores enemies hugging the Rig.
