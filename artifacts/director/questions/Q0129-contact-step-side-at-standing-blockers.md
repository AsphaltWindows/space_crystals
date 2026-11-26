# Q0129 — The contact step's side at a standing blocker: right first, or the side the unit is already on?

- **Date**: 2026-11-25T21:13:32-05:00
- **Blocking**: no — proceeding on assumption: (a), as built. At a blocker that is standing still, a unit goes round on the side it is already on (the shorter way); right first decides only dead centre.
- **Docs**: `systems/units.md > Ground Collision` ("on contact it steps aside, to its right first, switching sides after 1 s"); entries 0323, 0324, 0336; Q0088, Q0127

## Question
When a unit steps aside round a blocker that is **standing still**, which way should it go: to its right first, as the doc says, or round the side of the blocker it is already on?

## Context
- **Where "right first" comes from.** Q0088 documented M1's movement:
  - a unit meeting a blocker squarely stepped right first;
  - one meeting it off-centre stepped the way it leaned;
  - since M3-G, a unit in motion takes the side it is on at a standing blocker.
- **The 0323 build measured four rules** for a standing blocker on the Agents' 30 hatch layouts, all with the 1 s commitment:

  | Rule at a standing blocker | Hatch income, worst layout |
  |---|---|
  | Off-centre: the way it leans; squarely: right first (as before 0323) | **locks up** at 1 GU beside the hatch: 1.2 / 0.0 / 4.2 SC/s with 4 / 5 / 6 Agents |
  | Off-centre: the way it leans; squarely: the side it is on | **locks up**: 1.3 / 0.0 / 0.3 |
  | Off-centre: the side it is on; squarely: right first | no lock-up: 11.2 or more |
  | **The side it is on, always; right first only dead centre (built)** | no lock-up: 10.7 or more |

- **Why the lean fails.** A whole-width step changes the unit's lean, so the next contact sends it straight back, and it shuttles to and fro. Seen from the blocker's middle, the side never changes. The 0323 result you accepted in 0336 depends on this.
- **Moving blockers are unchanged:** the lean, right first head-on, and passing behind a crossing unit. Two units meeting head-on still both step right.
- **What the side rule costs**, from the build's tests. None of these is a lock-up:
  - **Beside the target.** A unit stopped by a standing unit right beside where it is going may go round on the far side. Two Sabres attacking one building head for the same corner, and the second steps round the first, away from the building's face. It ends out of reach, and its Attack order ends by the Failure rule. With automatic targeting on, as in play, it takes the building up again by itself: all are fighting by tick 240 instead of 160.
  - **Converging units** settle later and elsewhere: a line ordered to one point closes into a column on it.
  - **A chase given up.** In 1 of 12 layouts, one of eight Guards attacking a building gives up its chase by the Failure rule and doesn't fire before the building falls.
  - **Crowded exits.** 40 Guards leaving one Tunnel with no orders crowd its exit: 31 come out, then one stands idle in the doorway, and the rest wait until the crowd is moved. That is the player's job, as in 0320 and 0336.
  - **Recruits crowding one Storage face** deliver a little less: five give 5.5 SC/s instead of 6.1.

## Options
- **(a) As built.** A unit goes round a standing blocker on the side it is already on. Right first decides only dead centre, and still decides head-on meetings of moving units. The doc would say so.
- **(b) The side it is on when off-centre, right first when nearly square** (a lean under about 30%). This is closer to the doc's words for a standing blocker met squarely. The hatch measured slightly better (11.2 or more against 10.7). It costs one more movement slice to re-tune about ten tests.
- **(c) Right first at every contact** (the doc's letter). This brings back the hatch lock-ups that 0323 fixed.

Proposed: (a). Take (b) if "right first" matters when a unit meets a standing blocker squarely. The two differ only in that case.
