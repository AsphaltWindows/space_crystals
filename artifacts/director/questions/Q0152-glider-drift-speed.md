# Q0152 — At a quarter strength, gliders take 11–17 s to drift apart, not "a few seconds"

- **Date**: 2026-11-26T15:33:00-05:00
- **Blocking**: no. Slice M5-H2 built 0405's placeholders: gliders push apart at a quarter of the HoverCraft strength, and the Sparrow's SeparationRadius is 48.
- **Docs**: `systems/units.md > Air Collision`; `factions/cults/units/sparrow.md` (Placeholder stats); design change 0405.

## Question
0405 asks for two things that the quarter strength doesn't give together:
- "They drift apart gradually, mostly while circling, without noticeably bending their attack runs";
- its Verification: "Six Sparrows sent to one point drift apart over a few seconds while circling, and end about 48 su apart."

At a quarter strength the attack runs barely bend, but six Sparrows take 11–17 s after arriving to spread out. Which should give: the strength, or "a few seconds"?

## Context
- **Six Sparrows sent to one point.** They arrive at 2.5 s as a crowd, 4.4 su apart at closest. Times are counted from the order:

  | Closest pair | Quarter strength (built) | Full HoverCraft strength |
  |---|---|---|
  | Within 10% of 48 su | 13.4 s | 3.4 s |
  | Within 1% | 20 s | 9.8 s |
  | Within 0.01 su | 28.75 s | 17.2 s |

  At a quarter strength all six end idle on the 64 su circle, 47.9997 su apart at closest.
- **Two Sparrows making passes on one target.** The worst case is a pair side by side, 60 su apart, converging on every pass:

  | | Quarter strength (built) | Full strength |
  |---|---|---|
  | First approach: farthest from a lone Sparrow's track | 11.5 su | 20.5 su |
  | Sideways push, summed over the first pass | 27.6 su | 61.5 su |
  | Over each later pass | up to 3.3 su | up to 12.5 su |

  - At a quarter strength each still makes 7 passes in 60 s, as a lone Sparrow does. Every pass lands 7 hits within 5.1 su of the target's centre; a lone Sparrow lands 6–7 within 4.3 su.
  - After the first pass each leaves about 12° off a lone Sparrow's line, 64 su off at 300 su past the target.
  - Staggered or crossing pairs get at most 1.5 su of push a pass.

## Options
- **(a) Keep a quarter,** and read the Verification as "over 10–20 seconds". Nothing changes in the build.
- **(b) A stronger push, such as half.** Both the drift time and the bending land between the two columns. The Director measures it before building.
- **(c) Full HoverCraft strength.** They drift apart within a few seconds, and attack runs bend as in the right-hand column.
- **(d) The strength depends on what the glider is doing.** For example, full strength while circling at the end of a move and a quarter during attack passes. This meets both lines of the Verification. It's a small change: the push reads the unit's current order.
