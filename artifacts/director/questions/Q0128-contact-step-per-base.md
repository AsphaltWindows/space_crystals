# Q0128 — 0323's contact step per base: confirmation, HoverCraft, and the DrillUnit

- **Date**: 2026-11-25T20:43:38-05:00
- **Blocking**: no — proceeding on assumption: as built, below.
- **Docs**: `systems/units.md > Ground Collision`, `> HoverCraft`, `> DrillUnit`; entries 0323, 0325, 0336

## Question
0336 asks for confirmation that 0323's step is built for every listed base. It is, with two points that need the design:
1. **HoverCraft.** Ground Collision lists HoverCraft among the bases that sidestep, but `systems/units.md > HoverCraft` gives it Domain Air. Air units have no ground collision, so the step can never apply to it. Is HoverCraft in that list a slip?
2. **The DrillUnit**, the Syndicate's Subterranean Transport, isn't named in 0323. The build keeps the old 45° step for it. Which should it take: the 90° sidestep, or turn and graze like WheeledVehicles? It can reverse, like wheeled and tracked vehicles.

## Context
How 0323 is built (slice M4-W3, merging soon). The step is data on each base's definition:
- **LightInfantry, HeavyInfantry, HoverVehicle (the Drone) and TrackedVehicle:** a 90° step of the unit's own width, right first, switching sides after 1 s.
- **Mech:** the same sideways step. "Turn to face the side" first (0325) isn't built, since no Mech unit exists.
- **WheeledVehicle:** turn and graze comes with the Jackal's movement slice (M5-V2). Until then the old 45° step stays, but no wheeled unit exists yet.

## Options
1. **(a)** A slip: drop HoverCraft from the list. **(b)** HoverCraft means something else here.
2. **(a)** The 90° sidestep, like tracked vehicles. **(b)** Turn and graze, like wheeled vehicles. **(c)** Keep the 45° step.

Proposed: 1 (a). For 2, (b) suits a vehicle, but the Subterranean Transport isn't built yet, so it can wait.
