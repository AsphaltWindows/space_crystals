# Q0076 — Agents jam head-on between a near patch and the hatch

- **Date**: 2026-11-25T01:31:06-05:00
- **Blocking**: partly. It blocks the Syndicate economy check (M4-L, 0259's opening target) and the first playable layout. Other work proceeds as built: Agents keep collision.
- **Docs**: `factions/syndicate/units/agent.md > Gathering` (0255), `factions/syndicate/tunnels.md` (0259, the economy check), `factions/colonists/units/drone.md > Gathering Drones pass through each other` (0106)

## Question
With the patch 0–1 GU from the crystal hatch, Agents coming in meet Agents going out head-on, in a strip too narrow to pass. Agents keep hard collision and never give up (0255), so they stay jammed. What should happen?

## Context (measured in M4-D)
- **The strip** between the patch and the hatch is 1 GU (64 su) wide at the reference distance. An Agent is 36 su, so two can't pass each other in it.
- **Income against the table's 12.5 SC/s per Tunnel:**

  | Patch to hatch | 2 Agents | 3 Agents | 4 Agents |
  |---|---|---|---|
  | 0 GU | — | 5.5 | — |
  | 1 GU | 10.67 | 5.33 | 5.33 |
  | 2 GU | — | 9.8 | 10.0 |
  | 3 GU and more | within 15% | within 15% | within 15% |

- **The cause.** An Agent behind another in its lane meets one coming the other way. Neither yields (hard collision), and neither gives up (0255: waiting Agents never time out).
- **Where the distance comes from.** 0259's opening target puts the patch 1 GU from the hatch, and M4-B's `dev_skirmish` layout does the same. So the standard opening jams once a player has 3 Agents.
- **The Drone rule.** 0106 lets Drones on a Gather job pass through each other: "Drones only. Other factions' workers (e.g. Cults Recruits) keep normal collision while mining."

## Options
- **A. Agents on a Gather job pass through each other,** like Drones (0106). They still collide with everything else.
  - The hatch line still forms, since one Agent unloads at a time, and the 12.5 ceiling holds at any distance.
  - The code change is one flag on the Agent's worker definition.
- **B. Keep collision, and move the patch.** Reference layouts and the opening target put the patch at least 3 GU from the hatch.
  - A player who builds closer gets the jam. It is visible and fixable (move the Tunnel, or use fewer Agents), which fits "predictable, not optimal", but it is a trap at the most natural spot.
  - 0259's opening target changes.
- **C. A right-of-way rule.** An Agent carrying a load has priority, and an empty one waits or steps aside. This is crowd logic, which 0223 rules out, and it would need tuning.

## Also to confirm (M4-D's placeholders)
1. **Which square is the hatch.** It is the middle square of its side farther from Side A, to keep the line off the paths. 0255 says only "near the middle of the side".
2. **The search radius.** An Agent whose patch runs out looks for another within 6 GU, the Drone's value. The Agent's doc names none.
