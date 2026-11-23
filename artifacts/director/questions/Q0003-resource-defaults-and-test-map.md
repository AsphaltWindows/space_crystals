# Q0003 — Resource defaults (patch amount, supply deliveries) and a reference map

- **Date**: 2026-11-23T11:35:00-05:00
- **Blocking**: no — proceeding on assumption: patch RemainingAmount = 1500; Supply Delivery Station DeliverySize = 4 and DeliveryInterval = 60 s; stations start empty, so the first delivery arrives one interval after game start; a Director-made development map `dev_skirmish` (64×48 tiles, two starts with 8 patches each, one central station).
- **Docs**: `core/map.md > SpaceCrystalsPatch` (RemainingAmount), `core/map.md > SupplyDeliveryStation`, `factions/gdo/economy.md > Reference starting base`

## Question
1. What is the default **RemainingAmount** of a Space Crystals patch?
2. What are the default **DeliverySize** and **DeliveryInterval** of a Supply Delivery Station?
3. Does a station start the game **empty** (first delivery after one interval) or **stocked** with one delivery?
4. Is there a **reference map** to build for testing and balance (size, start positions, patch fields, station placement)? Or are development maps the Director's concern until map design starts?

## Context
M0 builds the map, patches and stations in the simulation. Every number above is a named placeholder constant tagged with this question id, so answers are a one-line change each.

## Options
- Confirm or replace the placeholders above.
- For 4: either the Designer specifies a reference map in the design, or the Director keeps owning test maps (they don't affect gameplay rules).
