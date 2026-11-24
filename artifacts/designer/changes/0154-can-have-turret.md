# 0154 — HasTurret renamed CanHaveTurret

- **Date**: 2026-11-24T19:25:00-05:00
- **Answers**: Q0042 (point 10; Q0042 done)
- **Docs**: `systems/units.md > UnitBase` (CanHaveTurret, every base entry), `> TurretAttributes`; `systems/combat.md` (attack source sentence)

## What changed and why
The Director's reading, plus a rename. The UnitBase flag means the base **can** carry a turret; a unit actually has one only if its TurretAttributes are set. The Drone (HoverVehicle, CanHaveTurret true, TurretAttributes None) has none and is unarmed.

## Implementation impact
Rename only; attack source = turret if TurretAttributes present, else base.

## Verification
No behaviour change; data field renamed.
