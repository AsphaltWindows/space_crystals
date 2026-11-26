# 0347 — Entering pool buildings: dedicated entrance doorway, exit opposite the entrance, full means idle, loads kept, no entering sites, capacity counts reserved units

- **Date**: 2026-11-26T22:30:00-05:00
- **Answers**: Q0104
- **Docs**:
  - `core/entities.md > Entrance and exit`;
  - `factions/cults/structures/README.md > Entering pool buildings` (new);
  - `factions/cults/structures/{armory,cathedral,chop_shop,airstrip}.md`; `factions/cults/augmentation.md`.

## What changed and why
User:
1. **A dedicated entrance**: a doorway at the middle of the entrance side. **For every training, augmentation and unit-producing building, the exit is the side opposite the entrance.** The exception is the **Airstrip**, which puts the finished aircraft directly on the runway to take off. The current ABAC buildings (B entrance, C exit) already comply.
2. **Full**: the Enter ends and the unit idles beside the building.
3. **Loads**: kept inside and carried out; lost if the Recruit is trained (default).
4. **Can't enter a building under construction**; a right-click Assists (default).
5. **Capacity counts reserved units**: the queue can never exceed the capacity. The Armory, Cathedral and Airstrip can hold at most 10 orders, and the Chop Shop at most 5 Jackals. The user may raise capacities in the numbers pass.

## Implementation impact
- Queues are limited by capacity, not by a separate 20.
- Entrance doorway pathing.

## Verification
- An Armory with 10 Recruits inside and 10 queued refuses an 11th Recruit, which idles beside it.
- The Chop Shop greys Build Jackal once 5 are queued.
