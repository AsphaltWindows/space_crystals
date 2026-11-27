# Q0170 — Orders given to a Sparrow during its takeoff run

- **Date**: 2026-11-26T19:57:36-05:00
- **Blocking**: no — proceeding on assumption: an order given during the takeoff run takes effect when the run ends (at the runway's C end), and Stop doesn't cut the run short (slice M5-N7's placeholder).
- **Docs**: `factions/cults/units/sparrow.md > Placeholder stats > Flight rules` (Takeoff, Stop); `factions/cults/structures/airstrip.md` ("takes off along it toward C"); `core/entities.md` (the Airstrip exception, Q0104).

## Question
While a Sparrow is on its takeoff run, from the runway's B end past its C end (28 frames, about 1.75 s at the placeholder numbers), what happens when the player gives it an order: Move, Attack, Attack Move, Stop or Hold Position?

## Context
- The Sparrow's flight rules say it "appears over the runway's B end facing C at IdleSpeed and speeds up along the runway", then takes the rally point's order, or circles 2 GU past C with none. They don't say whether the run can be interrupted.
- A player can select a Sparrow the moment it appears, so this comes up whenever someone rallies Sparrows by hand.
- As built (M5-N7, the assumption above):
  - any order waits for the run's end, then replaces the rally point's order;
  - Stop doesn't end the run: the Sparrow finishes it, then circles where it is.

## Options
1. **The run always finishes** (as built): the takeoff is a fixed sequence, and orders take effect at C. It is predictable, and every Sparrow leaves the runway the same way.
2. **Orders take effect at once:** the Sparrow leaves the runway line and turns toward its order, within its glider turn limits. It responds sooner, but a Sparrow ordered back over the Airstrip loops around low over the base. Stop would circle over the runway.
3. **Mixed:** Move, Attack and Attack Move take effect at once; Stop and Hold Position wait for the run's end.

The implementation cost is small either way.
