# Q0028: Mining and depletion

- **Date**: 2026-11-23T22:18:49-05:00
- **Blocking**: no. M2 builds the proposals. Point 1 sets the economy tests' expectations.
- **Docs**:
  - `factions/gdo/economy.md > ExtractionPlate`, `> Opening Economy Baseline`;
  - `core/map.md > SpaceCrystalsPatch`;
  - change 0038.

## Question
1. **Paid in lumps or as a trickle?**
   - "10 Space Crystals per 48 frames" could mean 10 at once every 3 seconds, or 1 crystal every 4.8 frames.
   - The baseline's timings match a trickle: the clean Barracks starts at 45.8 s. Paid in lumps, it starts at about 48 s.
   - **Proposal:** lumps of 10 every 48 frames, as written. A new plate pays first 48 frames after it is placed.
2. **The last partial load.**
   - A map maker can give a patch an amount that isn't a multiple of 10, for example 1505.
   - **Proposal:** the last load takes what is left (5 here), and later loads mine the residual 1.
3. **A depleted patch under a plate.**
   - `core/map.md` says a depleted patch disappears. The plate rules say the plate keeps mining at the residual rate, and that the depleted patch is removed only when the plate is destroyed.
   - **Proposal:** the plate rules are GDO's exception. The patch stays under its plate with 0 crystals left. Other factions' harvesting follows the general rule.
4. **Selecting a covered patch.**
   - **Proposal:** a click on that tile selects the plate. The patch can't be selected while it is covered, and the plate's InfoPanel shows the patch's remaining crystals.

## Context
Plate mining is built in M2. Point 1 shifts the opening by about 2 seconds compared with the baseline's numbers.

## Options
- **Point 1:** A, lumps; B, a trickle.
- **Point 2:** A, the last load takes what is left; B, a full 10 is mined as long as anything is left.
- **Point 3:** A, the patch stays under the plate; B, the patch disappears and the plate mines from nothing.
