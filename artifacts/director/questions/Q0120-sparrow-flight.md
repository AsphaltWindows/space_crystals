# Q0120 — The Sparrow's flight: separation, the map edge, takeoff, idling and strafing

- **Date**: 2026-11-25T19:18:39-05:00
- **Blocking**: no — proceeding on assumption: (a) on all seven.
- **Docs**: `systems/units.md > Air Collision`, `> GliderMovement`; `systems/controls/behaviors.md` (the GliderMovement exceptions); `factions/cults/structures/airstrip.md`; `factions/cults/units/sparrow.md`

## Question
The Sparrow is the game's first Glider. Open points:
1. **SeparationRadius** ("per unit type ... Must be larger than the unit's Silhouette") has no value. Is 32 su right? That is just over its half-diagonal of 28 su.
2. **Maximum overshoot** past the map edge ("TBD"). Is there no limit beyond what its turn needs (up to about 3 GU at full speed)?
3. **Takeoff.** A new Sparrow appears over the runway's B end, facing C, at IdleSpeed, then speeds up along the runway. Is that right?
4. **No rally point.** Where does it circle? 2 GU past the runway's C end?
5. **Idle circling.** At IdleSpeed with its full centripetal limit, the circle's radius is 16 su, a quarter of a tile: nearly a spin on the spot. Is that intended, or should idle circles have a set radius (e.g. 1 GU)?
6. **Strafing.** After a pass, how far does it fly on before looping back? Until the target is out of range, then turning at full speed?
7. **Stop and Hold Position.** It can't stop. Does Stop make it circle where it is? Does Hold make it circle there and fire only at what comes into range, without chasing?

## Context
- At 0319's numbers the full-speed turn radius is 100 su, so a pass and its loop take about 3 s.
- A set idle radius (5b) would be one more number per type.

## Options
1. **(a)** 32 su; **(b)** another value.
2. **(a)** no limit; **(b)** a cap (e.g. 2 GU), steering back earlier.
3. **(a)** as described; **(b)** it appears already at MaxSpeed.
4. **(a)** 2 GU past the C end; **(b)** over the Airstrip.
5. **(a)** the tightest circle (16 su); **(b)** a set idle radius of 1 GU.
6. **(a)** until out of range; **(b)** a fixed distance past the target (e.g. 2 GU).
7. **(a)** Stop and Hold both circle in place, and Hold doesn't chase; **(b)** no Hold for Gliders.

Proposed: (a) on all seven.
