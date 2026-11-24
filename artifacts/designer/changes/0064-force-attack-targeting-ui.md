# 0064 — Consistency fix: Attack targeting accepts own and neutral destructibles

- **Date**: 2026-11-23T20:35:43-05:00
- **Answers**: —
- **Docs**: `systems/controls/selection_and_hud.md > PointerDisplayType > AwaitingTarget` (Attack rows), `> BasicCombatUnitInterfaceState > AwaitingTarget[Attack] resolution`

## What changed and why
Follow-through of 0059 (force-attack allowed). The targeting UI still said Attack over a friendly or neutral object was Inactive. Now, while Attack awaits a target, own or neutral **destructible** objects show the Attack pointer and a left-click issues Attack on them. Indestructible objects (crystal patches) stay Inactive. Ground still issues AttackMove.

## Implementation impact
Update the command gate and pointer for AwaitingTarget[Attack].

## Verification
- Press Attack, hover an own building: Attack cursor. Click: units attack it.
- Press Attack, hover a crystal patch: Inactive cursor.
