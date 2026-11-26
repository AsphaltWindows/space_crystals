# 0365 — Prayer-mobile placeholders; Suppress aims where the vehicle faces

- **Date**: 2026-11-27T02:00:00-05:00
- **Answers**: —
- **Docs**: `factions/cults/units/prayer_mobile.md > Placeholder stats`

## What changed and why
- **Suppress fires along the vehicle's current facing**, with no target click. The user chose this "for now"; clicking a direction may come later.
- **Placeholders** (designer):
  - **Basics:** 56x40, HP 150, armor 1/1, sight 7, **Ungroupable**, speed 6 su/f; energy 200, starting at 50, regenerating 0.5/s.
  - **Inspire:** R, 75 energy, radius 4 GU, +1.5 su/f MaxSpeed for 10 s; Recruits' attack cycle × 0.75.
  - **Suppress:** F, 100 energy, a 90° arc 5 GU long; units whose centre is inside can't attack for 4 s.
  - **Cost:** 125 SC + 2 Recruits, 400 f.

## Implementation impact
New unit:
- an energy caster vehicle with two instant abilities;
- an attack-disable status, which also blocks firing under Target Lock without breaking the lock.

## Verification
- Inspire next to Recruits: they move and punch faster for 10 s.
- Suppress facing a group of Peacekeepers and your own Militants: neither side fires for 4 s.
