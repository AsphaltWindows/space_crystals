# 0169 — Occupied-arrival radius 3 grid units

- **Date**: 2026-11-24T21:25:00-05:00
- **Answers**: Q0032
- **Docs**: `systems/controls/behaviors.md > MovingToLocation > Occupied target location`; `> MovingToObject` (Blocked near the target)

## What changed and why
Option B. A unit stuck by non-moving units within **3 grid units** (was 2, 0026) of its target counts as arrived. Crowd sweep: Guard failures 43 → 17, Peacekeeper 12 → 0. Applied to MovingToObject's "blocked near the target" too, which is defined as the same rule. Still a placeholder.

## Implementation impact
Radius constant 2 → 3 GU in both behaviours.

## Verification
Rerun the M1-G3 crowd sweep: failures drop as in the experiment.
