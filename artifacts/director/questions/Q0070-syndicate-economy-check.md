# Q0070 — The Syndicate economy check

- **Date**: 2026-11-24T22:01:11-05:00
- **Blocking**: no — proceeding on assumption: the proposals below. Slice M4-L builds and measures the oracle anyway; only its pass or fail waits for the answer.
- **Docs**: `factions/syndicate/units.md > Agent > Gathering` (the table); `factions/syndicate/tunnels.md > Starting Conditions`, `> Tunnel Area Non-Overlap Rule` ("Crystal intake and stacked Tunnels"); `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)` (0108).

## Question
1. **What "Distance to Side B" measures.** **Proposal:** the gap between the patch's edge and Side B, edge to edge, as 0107 did for Drones. Under that reading, an Agent walks d GU less its silhouette and both reaches, about 0.8 GU less, so trips at d ≥ 1 come out about 1 s shorter than the table.
2. **Tolerance.** **Proposal:** 0108's ±15% on every figure. The 12.5 SC per second ceiling must be exact, since it is a hard rule (one drop-off per 64 frames). A larger gap comes back as a question with the measured table.
3. **The targets.** **Proposal:**
   - the opening: "saturated (3 Agents, 12.5 Space Crystals per second) around 0:40", on a layout with the patch 1 GU from Side B: 0:34 to 0:46;
   - the table, row by row;
   - stacked Tunnels double a field's intake;
   - "the lowest one-base saturation income", checked as an ordering against the GDO's and the Colonists' measured one-base figures.
4. **The reference layout.** **Proposal:** for each row, a Tunnel with Side B facing a single patch at distance d. Agents share the patch, so the patch count doesn't matter.

## Context
The table's trip is "5 s mining + 4 s drop-off + walking both ways" at 1.5 GU per second. It ignores silhouettes, reaches, turning and the line at Side B. The 0:40 figure assumes the second Agent starts at once.

## Options
- **Point 1:**
  - A, edge to edge, as proposed;
  - B, the walk per leg is d GU (the oracle places each patch about 0.8 GU farther).
- **Point 2:** A, ±15%, as proposed. B, a tolerance you give.
