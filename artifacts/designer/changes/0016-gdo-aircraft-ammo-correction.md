# 0016 — Correction: Fighter carries 1 missile, Air-to-Ground Hovercraft carries 5

- **Date**: 2026-11-23T14:09:46-05:00
- **Answers**: —
- **Docs**: `factions/gdo/units.md > Air-to-Ground Hovercraft (name TBD) > AttackAttributes`, `> Air-to-Air Fighter (name TBD) > AttackAttributes`

## What changed and why
**Corrects 0015**, which had the ammo counts wrong. The user's intent:
- **Air-to-Air Fighter (Glider): 1 missile**, then it reloads at its assigned Airfield.
- **Air-to-Ground Hovercraft (the "hover chopper"): 5 missiles**, then it reloads at its assigned Airfield.

## Implementation impact
Replaces the "Fighter = 5 missiles" in 0015. Still design stubs.

## Verification
(For later.) The Fighter fires once and must return. The Hovercraft fires 5 missiles before it must return.
