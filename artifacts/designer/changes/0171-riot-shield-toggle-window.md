# 0171 — Riot Shield: the toggle window

- **Date**: 2026-11-24T21:45:00-05:00
- **Answers**: Q0020 (point 2)
- **Docs**: `factions/gdo/units.md > Peacekeeper > Riot Shield` (During the toggle)

## What changed and why
The Director's proposal. While raising or lowering: half speed and no attacking; damage reduction only when fully up; a second press mid-toggle reverses from the current point. "Slowed" written as half speed (same as shield up) — designer's reading.

## Implementation impact
Shield state = progress 0..1 with direction; DR only at 1.0.

## Verification
Raise the shield and get hit mid-raise: full damage. Press again halfway: lowering takes half the toggle time.
