# 0304 — Cults Gunner can miss (TailDisjointed)

- **Date**: 2026-11-26T13:05:00-05:00
- **Answers**: —
- **Docs**: `factions/cults/units/gunner.md > AttackAttributes`

## What changed and why
User: the Gunner should be able to miss, with a **TailDisjointed** attack.
- The target's position locks at the end of Aiming, and the shot lands there at the end of Firing (2 frames).
- A target that moves in that window is missed, and whatever unit is at the spot (any owner) is hit instead.
- It gains **Attack Ground**, per the rule that every attack that can miss has it.

All numbers are unchanged.

## Implementation impact
- The Gunner's attack type changes from FullyConnected to TailDisjointed.
- Attack Ground is added to its panel.
- With 2-frame Firing, only moving targets really get missed.

## Verification
A Gunner firing at a Recruit that runs sideways sometimes misses; Attack Ground is available.
