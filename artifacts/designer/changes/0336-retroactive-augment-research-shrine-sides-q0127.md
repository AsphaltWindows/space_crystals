# 0336 — Augment research is retroactive; Shrine gets an entrance and exit; Q0127: accept the oversaturated hatch

- **Date**: 2026-11-26T20:35:00-05:00
- **Answers**: Q0127
- **Docs**:
  - `factions/cults/augmentation.md` (Research applies to everyone, replacing Level Locking; AugmentationState; Augmentation Orders);
  - `core/tech_and_research.md > Faction Exceptions`; `factions/cults/structures/temple.md`;
  - `factions/cults/structures/shrine.md` (sides).

## What changed and why
- **Temple augmentation research is retroactive** (user: "to avoid unnecessary complexity").
  - When a level completes, every unit with that Augmentation gets the new amounts at once.
  - HP follows 0331: a MaxHP increase raises HP by the same amount; a decrease caps it.
  - **Level Locking is removed.** AugmentationState becomes two booleans, with the amounts read from the current research level.
  - Other Cults stat upgrades (none designed yet) are still non-retroactive as written; the user is being asked.
- **Shrine** becomes ABAC, with entrance B and exit C. Participants leave one at a time.
- **Q0127: accept (a).** The user called 0323 "a big success". An oversaturated hatch, where extra Agents wait without a turn, is the player's job, as for Recruits (0320). No 2-square hatch, and no make-way rule. Steps 2 and 3 of 0324 are dropped.
- **Please confirm:** 0323's sidestep is implemented for **every** listed base (LightInfantry, HeavyInfantry, HoverVehicle, HoverCraft, TrackedVehicle; Mech), not only Agents. The user asked.

## Implementation impact
- Remove per-unit research levels.
- Augment amounts are computed from the player's current Temple levels.
- Shrine sides.

## Verification
- Augment a Recruit, then complete Enhanced Constitution L1: the existing Recruit's MaxHP rises from +20 to +40, and its HP rises by 20.
