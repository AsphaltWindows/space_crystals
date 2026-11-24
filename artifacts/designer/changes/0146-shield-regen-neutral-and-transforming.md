# 0146 — Shield regeneration: neutral objects, transforming objects

- **Date**: 2026-11-24T18:35:00-05:00
- **Answers**: Q0042 (points 1 and 2)
- **Docs**: `systems/combat.md > Shields > ShieldRegenRate`

## What changed and why
- Pt 1 (user, A): neutral objects with Shields (defeated players' units) keep regenerating, like any object.
- Pt 2 (already decided in 0110): during Deploy/Warp In, Shields regenerate up to the current interpolated maximum. Now also stated in combat.md.

## Implementation impact
No ownership condition on regeneration.

## Verification
Defeat a Colonists player, damage one of their now-neutral Legionnaires' shields: they refill at 4/s.
