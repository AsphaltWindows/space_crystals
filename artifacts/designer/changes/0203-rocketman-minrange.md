# 0203 — Rocketman has a minimum range

- **Date**: 2026-11-25T03:20:00-05:00
- **Answers**: — (context for Q0054)
- **Docs**: `factions/gdo/units.md > Rocketman > AttackAttributes`

## What changed and why
User: the Rocketman's normal attack has a MinRange (value TBD). The deployed Artillery Rig already had one. These are the first real units the MinRange rules apply to (0182): the Rocketman is mobile and backs off; the deployed Artillery Rig is immobile and fails with "Target too close".

## Implementation impact
MinRange value TBD (numbers pass). Whether Target Lock's homing missiles also respect MinRange is being settled with the user.

## Verification
Once a value exists: an explicit Attack on an adjacent enemy makes the Rocketman step back, then fire. Auto-targeting skips enemies inside MinRange.
