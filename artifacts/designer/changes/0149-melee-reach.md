# 0149 — Melee reach 8 su

- **Date**: 2026-11-24T18:50:00-05:00
- **Answers**: Q0042 (point 5)
- **Docs**: `systems/combat.md > Range Measurement` (melee bullet), `> Melee`

## What changed and why
Melee units have some reach, not exactly 0. All melee attacks: 8 su edge-to-edge for now (placeholder; may become per-attack later).

## Implementation impact
Melee range = 8 su gap. Applies to Sabre Legionnaire, Cults melee (Recruit), etc.

## Verification
A Sabre Legionnaire attacks when its edge is within 8 su of the target's edge; it never gets stuck "touching but out of range".
