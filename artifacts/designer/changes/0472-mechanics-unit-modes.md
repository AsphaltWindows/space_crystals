# 0472 — Mechanics doc: Unit Modes (Deploy and Toggle archetypes)

- **Date**: 2026-11-28T11:05:00-05:00
- **Answers**: Q0193 (migration, part 2)
- **Docs**:
  - `systems/mechanics/modes.md` (new)
  - `README.md > systems/mechanics/`
  - `factions/gdo/units/artillery_rig.md > Modes`
  - `factions/syndicate/units/laser_battery.md > Deploy`
  - `factions/syndicate/units/subterranean_transport.md > Burrowing and surfacing`
  - `factions/gdo/units/prospector.md`
  - `factions/gdo/units/peacekeeper.md > Riot Shield`
  - `factions/colonists/units/weapon_conversion.md`
  - `factions/gdo/units/README.md`

## What changed and why
The second mechanics doc. Units with two forms use one of two archetypes.
- **Deploy**: an order, queueable, uncancellable, with no reversing mid-way. The type changes on completion. The last order given while switching is held, and orders the new form can't do are dropped.
  - **Parameters:** permanent, targeted, what carries over.
  - **Used by:** the Artillery Rig, the Laser Battery, the Subterranean Transport's Burrow and Surface, and the Prospector (permanent and targeted).
- **Toggle**: a mode switch. The type changes at the press. It sets a target form, keeps orders, isn't queueable, and is deferred during Firing and Cooldown.
  - **Parameter:** what a second press does mid-switch: reverse from the current point (the Riot Shield), or complete and then switch back (Weapon Conversion).
  - **Used by:** the Riot Shield and the Colonists' Weapon Conversion.

**No rule changed.** The Artillery Rig's and Laser Battery's duplicated deploy rules were replaced by the reference. The Peacekeeper and Weapon Conversion docs keep their own parameters and examples, with a Uses line.

## Implementation impact
A single modes module with two archetypes. There are no behaviour changes. The Colonists Drone's Beacon deploy, which is immediate and cancellable, is explicitly not either archetype.

## Verification
No gameplay change.
