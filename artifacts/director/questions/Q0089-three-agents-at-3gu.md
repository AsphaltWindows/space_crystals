# Q0089 — Three Agents at 3 GU reach 10.7 SC/s, not 12.5: is the opening target the third Agent at work, or the Tunnel at 12.5?

- **Date**: 2026-11-25T11:52:53-05:00
- **Blocking**: no. Proceeding on these assumptions: the opening target is met when the third Agent is at work by 0:40, and the table passes within ±15%. M4-L's oracle pins the measured figures.
- **Docs**:
  - `factions/syndicate/units/agent.md > Gathering > Space Crystals` (the table, the economy check's targets, the rationale);
  - `factions/syndicate/tunnels.md > Starting Conditions`;
  - 0294's verification.

## Question
With the patch 3 GU from the hatch, does "a starting Tunnel is saturated (3 Agents, 12.5 Space Crystals per second) around 0:40" mean:
- the third Agent is at work around 0:40, or
- the Tunnel takes in 12.5 SC/s from around 0:40?

And is 3 Agents at 3 GU measuring about 15% below the table's 12.5 acceptable?

## Context
M4-L built the Syndicate's economy oracle after 0294 (mining 4 s; Agents keep collision; checks from 3 GU). It measures with collision, through the real orders.

**The table** (SC/s with 2, 3 and 4 Agents; the trip in seconds):

| Distance | Trip | 2 Agents | 3 Agents | 4 Agents |
|---|---|---|---|---|
| 3 GU (design) | 12.0 | 8.3 | 12.5 | 12.5 |
| 3 GU (measured) | 11.0 | 9.0 | **10.67** | 11.83 |
| 4 GU (design) | 13.3 | 7.5 | 11.3 | 12.5 |
| 4 GU (measured) | 12.4 | 8.0 | 11.17 | 12.17 |
| 5 GU (design) | 14.7 | 6.8 | 10.2 | 12.5 |
| 5 GU (measured) | 13.75 | 7.17 | 10.33 | 12.50 |
| 6 GU (design) | 16.0 | 6.2 | 9.4 | 12.5 |
| 6 GU (measured) | 15.0 | 6.5 | 9.67 | 12.50 |

- **Every row passes ±15%,** so the oracle passes by the design's tolerance. But 3 Agents at 3 GU measure 10.67, which is −14.7%, at the edge.
- **Why:** with 3 or 4 Agents close in, the hatch sits idle for up to about 2 s while loaded Agents get past the one leaving. From 5 GU on, the walk spreads them out and 4 Agents reach the ceiling exactly.
- **The opening.** The third Agent comes out at 0:39.56, inside the 0:34–0:46 window. With 3 Agents at 3 GU, the Tunnel then runs at about 10.0 SC/s from 1:00 to 1:30, and never reaches 12.5.
- **Also checked, all passing:**
  - no two deliveries ever come less than 64 ticks apart (the 12.5 ceiling);
  - stacked Tunnels at 3 GU take in 23.33 (−6.7% against 25);
  - the one-base income is lowest for the Syndicate: 12.5 against the GDO's 26.67 and the Colonists' 32;
  - rows 0–2 are recorded only, by 0294. 0 and 1 GU jam at about 6; 2 GU gives 10.17, 11.00 and 11.33 with 2, 3 and 4 Agents.

## Options
- **(a) As built:** the opening target reads "the third Agent at work by 0:40", and the table stands, met within tolerance. A player wanting the full 12.5 at 3 GU adds a fourth Agent. That's close to it: 11.83 at 3 GU, 12.17 at 4.
- **(b)** Restate the targets as measured: 3 Agents give about 10.7 at 3 GU, and 4 Agents saturate from about 5 GU. The oracle then checks those numbers.
- **(c)** Change a number so 3 Agents reach 12.5 at 3 GU, for example a slightly shorter mining time. Collision near the hatch makes the result hard to predict; it would be re-measured.
