# 0457 — Supply Chopper Unit Control, Outpost Groupable, mine sight and armour

- **Date**: 2026-11-27T14:00:00-05:00
- **Answers**: Q0182 (parts 1–3; part 4, the Hornet's missile, follows separately)
- **Docs**:
  - `factions/gdo/units/supply_chopper.md` (UnitControlCost)
  - `factions/gdo/structures/outpost.md > Placeholder stats` (Groupable)
  - `factions/gdo/units/twister.md > Placeholder stats` (mine)

## What changed and why
1. **Supply Chopper:** UnitControlCost **2**, as assumed (designer placeholder).
2. **Outpost:** Groupable **true**, as assumed (designer placeholder). It has no commands, so this has no effect yet.
3. **Mine** (user's numbers): **SightRange 1 grid unit**, **PointArmor 12, FullArmor 12**. This changes GK's assumption of 0 and 0/0. The user's reason: **mines shouldn't be easy to clear**. A mine sees a small circle around itself and shrugs off small-arms fire: weak hits do only the minimum damage against 40 HP.
4. **Raptor, Dragonfly, Twister:** SingleTarget, confirmed from the docs (homing missiles and a FullyConnected gun).

## Implementation impact
- The mine kind gets sight 1 GU and armour 12/12.
- The rest is as registered; remove the placeholder marks.

## Verification
- Lay a mine. Its owner sees a 1 GU circle around it.
- Shoot it with Peacekeepers, for example. Each hit does only the minimum damage.
