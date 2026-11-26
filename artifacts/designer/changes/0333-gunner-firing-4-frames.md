# 0333 — Gunner: FiringDuration 4 frames so moving targets can dodge; structures count at the spot

- **Date**: 2026-11-26T19:30:00-05:00
- **Answers**: Q0126
- **Docs**: `factions/cults/units/gunner.md > AttackAttributes`; `systems/combat.md > TailDisjointed`

## What changed and why
- **Point 1** (user): "change it to 4 frames".
  - The Gunner's FiringDuration goes from 2 to **4 frames**. A walking infantry unit, at 16 su in 4 frames against a 12 su half-width, can now step off the locked point and be missed.
  - CooldownDuration goes from 8 to **6** (designer placeholder), so the cycle stays 26 frames.
- **Point 2:** structures count as "at the spot", so a Gunner damages buildings. The user accepted this.
- **Point 3:** your tie rule as built (the aimed target if it still covers the point, otherwise any one) is fine.

## Implementation impact
Numbers change only. The build's points 2 and 3 stand.

## Verification
- A Peacekeeper walking across a Gunner's line is sometimes missed: the tracer ends on empty ground.
- A standing target is always hit.
