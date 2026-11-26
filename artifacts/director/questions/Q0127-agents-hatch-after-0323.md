# Q0127 — Agents at the hatch after 0323: the lock-ups are gone, but a crowded hatch leaves extra Agents waiting. Accept, or go on to step 2?

- **Date**: 2026-11-25T20:28:08-05:00
- **Blocking**: no — proceeding on assumption: (a). The hatch stays as built (one square, 0315, 0323), and the remaining layouts are documented as known behaviour.
- **Docs**: `factions/syndicate/units/agent.md > Gathering` (Hatch jams: the plan); `systems/units.md > Ground Collision`; entries 0315, 0323, 0324; Q0111

## Question
0324's step 1 is built and measured: 0323's 90° body-width contact step, with the 1-square hatch and 0315 kept. 0324 asks for the measurements to come back as a question if step 1 isn't enough. Should we stop here, or go on to step 2 (a 2-square hatch) or step 3 (stuck gathering Agents make way)?

## Context
- **The measurements:** 30 layouts (4, 5 or 6 Agents; the patch 1–5 GU away; facing the hatch or north-west of it), and the `dev_skirmish` soak over 10 minutes.

  | | Before (0315 only) | With 0323 |
  |---|---|---|
  | Layouts where the Tunnel's income stops for 60 s or more | 9 of 30 | 0 |
  | Layouts where some Agent goes 60 s or more without delivering | 20 of 30 | 6 |
  | Income across the 30 layouts | 0.0–12.5 SC/s | 10.7–12.5 SC/s |
  | Soak, 4 / 6 crystal Agents | stops for good at 6:53 / 4:59 | never stops (12.2 / 12.2 SC/s) |

- **The 6 remaining layouts aren't lock-ups.** The hatch keeps working at close to its full rate, but extra Agents don't get a turn:
  - **Facing the hatch at 1 GU, with 4, 5 or 6 Agents:** two Agents mine and unload, each from one spot, and keep the hatch at 10.7–12.5 SC/s. The others stand loaded 40–70 su away, outside reach. So they never join the line and never get a turn: they wait out the whole 10 minutes.
  - **Six Agents north-west of the hatch at 1 GU and 3 GU, and facing it at 4 GU:** they take turns at a saturated hatch, and the longest waits are 66, 66 and 76 s.
- **A precedent:** this resembles Recruits overloading a Storage face, which 0320 made the player's job. Extra workers add nothing, and the income doesn't drop.
- **The figures may still shift by a layout or so:** a follow-up is fixing side effects of 0323 on other crowd tests.

## Options
- **(a) Accept:** an oversaturated hatch is the player's job, as with Recruits (0320). The playtest notes say that with the patch close, more than two or three Agents per hatch add nothing.
- **(b) Step 2:** a 2-square crystal hatch (both middle squares of Side B), still with one Agent delivering at a time (0324).
- **(c) Step 3:** stuck gathering Agents make way like idle units, perhaps with "loaded has right of way" (0324).
- **(d) Join the line from farther away:** the waiting rule's "within reach" becomes a larger radius, so a loaded Agent standing 40–70 su away counts as waiting, gets its turn in order, and walks in when the hatch is free.

Proposed: (a). In these layouts the income is already at its maximum, and (b)–(d) change behaviour for a case where extra Agents can't add income anyway.
