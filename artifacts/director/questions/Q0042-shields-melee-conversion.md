# Q0042: Shields, melee range and Weapon Conversion

- **Date**: 2026-11-24T05:02:31-05:00
- **Blocking**: no. A and B build the proposals.
- **Docs**: `systems/combat.md > Shields`, `> DamageModifierOrder`, `> Range Measurement`, `> AttackPhase`; `factions/colonists/units.md > Weapon Conversion`, `> Sabre Legionnaire`; `systems/units.md > HoverVehicle`; `core/game_rules.md > On Defeat`.

## Question
1. **Shields of neutral objects** (a defeated player's units, 0101). **Proposal:** they keep regenerating.
2. **Regeneration while transforming.** **Proposal:** up to the current interpolated maximum (Q0039 point 2).
3. **Which modifiers touch Shields.** "At full value": **proposal:** no pre-armor multiplier and no DirectionalArmor applies to Shields. The Drone's DirectionalArmor (HoverVehicle) applies to the overflow only.
4. **Shields display.** **Proposal:** a bar above the HP bar, and "Shields a/b" in the InfoPanel.
5. **Melee range.** "Adjacent contact", "a gap of about 0". **Proposal:** 8 su (the Proximity gap), as a placeholder. Exactly 0 is fragile with collision margins.
6. **Convert while the unit has orders.** **Proposal:** the orders stay. The unit holds still for 4 frames, then carries on with the other weapon.
7. **Shift with Convert.** **Proposal:** it means nothing; Convert isn't queued.
8. **The weapon after conversion.** **Proposal:** the new weapon starts ready. A reload in progress isn't carried over.
9. **A mixed group in which some units are Firing or in Cooldown.** **Proposal:** those units are left out and the others convert.
10. **HoverVehicle `HasTurret - true`, but the Drone's `TurretAttributes - None`.** **Proposal:** read HasTurret as "the base can carry a turret".

## Context
The shield formulas are complete. These points are where Shields meet other rules, and the parts of conversion the docs leave open.

## Options
- **Point 6:** A, keep the orders, as proposed. B, Convert replaces the orders, and the unit idles afterwards.
- **Point 8:** A, ready, as proposed. B, carry the reload over.
- **Other points:** the proposal, or a rule you give.
