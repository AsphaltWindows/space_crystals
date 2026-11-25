# Q0111 — 0315 measured: Agents at a hatch still gridlock. Add the backstop (a), accept, revert, or pass-through?

- **Date**: 2026-11-25T18:23:00-05:00
- **Blocking**: yes, for the Agents' jam fix only, before the playtest. `dev_skirmish`'s own crystal field lies to the side of the Syndicate's hatch.
- **Docs**:
  - `factions/syndicate/units/agent.md > Gathering > Gathering rules (user, 2026-11-25; Q0066)` (Waiting at the hatch, 0315);
  - `systems/units.md > Ground Collision > Who yields`;
  - Q0109, its first step 0315, and 0320 (Recruits: crowding is the player's job).

## Question
0315's rule is built (slice M4-W2): an Agent waits only within reach of the hatch, and until then keeps pressing toward it. It doesn't fix the gridlock, and it makes some layouts worse. Which next step?

## Context
- **Measured by M4-W2,** in SC/s after the first minute of 10. The brackets give the longest stretch with no delivery at the hatch, in seconds (600 = the whole game). The patch is north-west of the hatch, so the Agents come along the Tunnel's wall, with 4 / 5 / 6 Agents:

  | Patch north-west | 1 GU | 2 GU | 3 GU | 4 GU | 5 GU |
  |---|---|---|---|---|---|
  | Before 0315 | 0.4 (356) / 0.0 (600) / 0.0 (600) | 0.0 (557) / 1.8 (42) / 0.8 (274) | 1.7 (38) / 4.0 (28) / 2.7 (39) | 10.6 (28) / 3.4 (35) / 3.3 (77) | 9.1 (33) / 2.3 (80) / 3.2 (43) |
  | With 0315 | 0.0 (600) / 0.0 (600) / 0.0 (600) | 4.4 (23) / 2.8 (39) / 2.9 (30) | 5.8 (168) / 0.6 (448) / 0.5 (462) | 10.4 (27) / 1.0 (335) / 2.1 (61) | 10.7 (19) / 3.1 (50) / 0.7 (171) |

  - **Facing the hatch** (the economy table's layout), income never stops for 60 s. But 0315 lowers 1–2 GU (5 Agents at 1 GU: 11.9 → 5.9), while 3–5 GU stay at 11.8–12.5.
  - **0315's own Verification** ("no Agent should go 60 s without delivering") fails in 20 of 30 layouts, against 19 before.
  - **The 10-minute soak on `dev_skirmish`,** with 3 / 4 / 5 / 6 crystal Agents:
    - before: 10.0 / 6.3 / 4.2 / 5.7 SC/s, with 4, 5 and 6 Agents stopping for good at 7:05, 4:37 and 6:16;
    - with 0315: 10.9 / 6.7 / 7.8 / 4.3 SC/s. Four Agents still stop for good at 6:53 and six at 4:59; five recover.
  - **The economy oracle** (3 Agents, 1–6 GU) stays within ±15%, and improves at 1 GU.
- **The mechanism that remains** (the trace is in `crates/sim/tests/agent_gathering/jam.rs`):
  1. The Agent that has just unloaded stands at the wall in a pocket, with loaded Agents north and west of it.
  2. Its steering never steps back, and its re-routes return the same path.
  3. The loaded Agents are out of reach, the nearest by 1 su. Under 0315 they press toward points it covers.
  4. All of them hold an order, so by "Who yields" none makes way.

  At 1 GU the Agents meet head-on between patch and hatch.
- **A technical cause, for completeness (the Director's, not a design matter):** the Agents queued behind the pocketed one are never judged stuck, because its small shuffles count as their progress, so they never re-route. Fixing it alone isn't expected to free the pocketed Agent itself.

## Options
- **(a) Add the backstop you were considering:** an Agent with a gathering order that is blocked, whether pressing toward the hatch or waiting at it, steps aside like a unit with no active command.
  - **M4-W's prototype of it, without 0315,** never went 60 s without a delivery in any layout (18 s at most), at 8.5–10.2 SC/s to the side.
  - **It costs a little in the reference layout:** 3 Agents facing the hatch at 3 GU gave 10.5 against the table's 12.5. "First to arrive delivers next" no longer holds strictly.
  - **Not yet measured on top of 0315.** One slice would measure both together.
- **(b) Accept it as the player's job,** as 0320 decided for Recruits: keep patches facing the hatch, or use three Agents a hatch. But `dev_skirmish`'s default field is to the side, and four Agents there lose the Tunnel's income for good, with no sign but the income.
- **(c) Revert 0315** to "wait within reach, or as close as it can get". The results are mixed: 0315 helps north-west at 2 GU and the 5-Agent soak, and hurts north-west at 3–4 GU with 5–6 Agents and facing the hatch at 1–2 GU.
- **(d) Gathering Agents pass through each other,** like the Colonists' Drones (0106). This removes every jam at the hatch, but reverses 0294's choice to keep Agents colliding.

**The Director's note:**
- (a) is one slice, measured on the same grid. Whether to keep 0315 under it can be decided from those numbers.
- (d) is the simplest and the only option certain to leave no jam.
- (b) is consistent with 0320, but leaves the default map's field jamming with four Agents.
