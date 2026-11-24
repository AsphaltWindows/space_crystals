# 0155 — A plate's remaining crystals: owner only

- **Date**: 2026-11-24T19:35:00-05:00
- **Answers**: Q0034
- **Docs**: `factions/gdo/economy.md > ExtractionPlate > InfoPanel`

## What changed and why
Option B, narrowed by the user to the **owner only** (allies don't see it either). Other players get the reduced structure panel (name, owner, HP; 0105).

## Implementation impact
Changes the current build (count shown to every viewer). Uncovered patches still show their count to everyone.

## Verification
Select your own plate: count shown. Select an ally's or enemy's plate: name, owner, HP only.
