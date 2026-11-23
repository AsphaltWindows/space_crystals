# 0027 — Arrived units don't yield to members of their own group order

- **Date**: 2026-11-23T18:52:17-05:00
- **Answers**: Q0015 (point 1; points 2 and 3 still open)
- **Docs**: `systems/units.md > UnitCollision > Ground Collision > Friendly yielding > No yielding within the same group order`

## What changed and why
Option A, as the Director assumed. A unit that arrived under a group order doesn't step aside for members of the **same** order who are still arriving. They stop against it (arrived within 2 GU, 0026). It still yields to all other own units, and a new order clears the exception. This prevents endless shuffling when a group converges.

## Implementation impact
Matches the Director's assumption.

## Verification
- Converge 12 units on a point: they settle into a calm blob with no shuffling at the centre.
- Then walk a different own unit through the blob: the blob's units step aside for it.
