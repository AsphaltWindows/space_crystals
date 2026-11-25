# Q0077 — Eject during the transit delay, and where ejected units wait

- **Date**: 2026-11-25T01:38:59-05:00
- **Blocking**: no. Proceeding as built in M4-C: refused, and 1 GU out.
- **Docs**: `factions/syndicate/tunnels.md > Tunnel Network` (Transit delay, 0245; Ejection, 0250 and 0251), `> EjectMenu`

## Question
1. **An Eject for a unit type whose units are all still in the transit delay** (entered under 32 frames ago, 0245): is it refused, or queued until the delay ends?
2. **An ejected unit with no rally:** where does it go?

## Context
- **Point 1.** 0251 ejects the longest-inside unit of the requested type, so a refusal happens only when every unit of that type is still in transit. As built, the command is refused (`InTransit`), and the EjectMenu can grey the button for those 2 seconds.
- **Point 2.** As built, an ejected unit with no rally walks 1 GU straight out of the exit path and idles there.
  - The next ejected unit needs the spot clear (0250), so idle units get nudged aside by friendly units coming out. The stream still flows, but slows to about 8–16 ticks per Guard, from 8.
  - Idle units near Side A can also brush the line entering on the other half.
  - A rally avoids all of it, which fits "managing bottlenecks is the player's job" (0223).

## Options
- **Point 1:**
  - **A. Refused,** as built: the button is grey while every unit of the type is in transit.
  - **B. Queued:** the eject waits in the Tunnel's queue and happens when the delay ends.
- **Point 2:**
  - **A. As built:** 1 GU straight out, then idle.
  - **B. Walk farther out,** for example 3 GU, so the next units rarely need to nudge. The distance is a placeholder.
  - **C. A different default,** to be described.
