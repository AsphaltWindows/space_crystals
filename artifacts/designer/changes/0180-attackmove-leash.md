# 0180 — AttackMove leash: limits chasing only; reference line rules

- **Date**: 2026-11-24T23:20:00-05:00
- **Answers**: Q0018 (point 3)
- **Docs**: `systems/controls/behaviors.md > AttackMovingToLocation` (PathReference; Algorithm step 3)

## What changed and why
- **Leash limits chasing only** (fixes a flaw the user spotted: beyond the leash a unit used to disengage even from an enemy in weapon range). Targets in weapon range are always fought. The unit just won't move beyond 6 GU from PathReference to chase; out-of-range targets that would need that are dropped and the unit resumes toward the destination.
- **PathReference** = route planned at AttackMove start; **kept after fights** (current build; walking path re-planned, leash line not), **replaced on a blockage re-route** (detour).

## Implementation impact
- Change disengage condition: only when (target out of weapon range) AND (approach would exceed leash).
- Re-route due to blocking updates PathReference; post-fight re-paths don't.

## Verification
- Attack-move past an enemy that shoots from 1 tile beyond the leash but within the unit's range: the unit fights back.
- Enemy artillery outranging from far off the route: the unit keeps walking.
- Block the route so the unit detours 10 GU sideways: it can still chase enemies near the detour.
