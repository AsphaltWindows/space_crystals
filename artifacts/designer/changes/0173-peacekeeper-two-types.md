# 0173 — Peacekeeper and Peacekeeper (Shielded) are two unit types

- **Date**: 2026-11-24T22:00:00-05:00
- **Answers**: Q0020 (point 4)
- **Docs**: `factions/gdo/units.md > Peacekeeper > Riot Shield` (Two unit types)

## What changed and why
User's alternative to the "raise all / lower all" proposal: shield down and up become two unit types, **Peacekeeper** and **Peacekeeper (Shielded)**, like Sabre/Rifle Legionnaires. A mixed selection forms two SelectionGroups; the shield command acts on the ActiveGroup; after toggling, units of the same type merge into one group. Panels: Raise Shield (Peacekeeper) / Lower Shield, no Attack (Shielded).

## Implementation impact
- Toggling changes the unit's type at the **start** of the toggle (designer's reading; the opposite command is then available to reverse mid-way, 0171).
- Barracks produces Peacekeeper; same cost/UC for both types.
- Existing rules unchanged: 0074 (shielded right-click follows), 0075 (explicit Attack leaves Shielded out), 0150 (orders kept), 0172 (deferred mid-attack).

## Verification
Select 10 Peacekeepers, raise shields on 6: the SelectionPanel shows two groups. Tab to the 4, press the shield key: all 10 shielded, one group.
