# 0476 — Mechanics doc: Restoring HP (Repair, Healing area, Regeneration)

- **Date**: 2026-11-28T12:20:00-05:00
- **Answers**: Q0193 (migration, part 5)
- **Docs**:
  - `systems/mechanics/hp_restoration.md` (new)
  - `README.md > systems/mechanics/`
  - Repair sections of `factions/cults/units/recruit.md`, `factions/gdo/units/field_engineer.md` and `factions/syndicate/units/agent.md`
  - `factions/gdo/structures/first_aid_tent.md > Healing`
  - `factions/syndicate/tunnels.md` (Regeneration inside)
  - `factions/syndicate/structures/README.md` (expansions regenerate)
  - `factions/gdo/economy.md` (Chopper on its Tower)

## What changed and why
The fifth mechanics doc. It has three archetypes.
- **Repair:**
  - several repairers scale linearly;
  - from 0 to full takes half the build time and costs half the price, paid gradually; a vehicle's price counts its Space Crystals only;
  - it stops at full HP or when the resource runs out;
  - no Gliders, except the landed Pilgrim;
  - input is D or right-click, and Enter wins over it.
  - Each repairer's doc keeps only what it can repair and which targets are free.
  - This text was repeated word for word in three docs.
- **Healing area:** who, radius to the unit's centre, HP/s; sources don't stack; only units on the map; only once complete.
  - A **docked service** variant covers the Supply Chopper on its Tower.
- **Regeneration:** a percentage per second while a condition holds:
  - the Tunnel Network;
  - expansions;
  - the Shrine's regeneration buff.

**No rule changed.**

## Implementation impact
One HP-restoration module. No behaviour changes.

## Verification
No gameplay change.
