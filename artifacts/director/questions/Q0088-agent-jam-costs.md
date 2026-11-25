# Q0088 — The Agents' jam: what each fix costs (the reply to 0286 and 0287; follows Q0076)

- **Date**: 2026-11-25T09:10:19-05:00
- **Blocking**: partly, as Q0076: M4-L's economy oracle and the first playable layout. Until then Agents keep collision, as built.
- **Docs**:
  - `systems/units.md > Ground Collision`;
  - `systems/controls/behaviors.md > MovingToLocation > Failure` and `> Core principle: predictable, not optimal`;
  - `factions/colonists/units/drone.md` (0106);
  - `factions/syndicate/units/agent.md > Gathering`;
  - `factions/syndicate/tunnels.md` (0259);
  - the approved movement spec, `docs/movement.md > 6.3`.

## Question
With the costs below, which fix should the Agents get, if any? And should the design text describe the sidestep that moving infantry already make?

## Context: what the measurements found
A Fable investigation traced the jam and prototyped each option, measuring income over 10 minutes in 4 start layouts, with 3, 4 and 6 Agents. Three findings change the question.
1. **It isn't a head-on jam, and the stuck/re-route rule never fires.** At 1 GU with 3 Agents:
   - One Agent works the direct lane between the hatch and the patch, alone, for good.
   - A second, just emptied, heads for the patch point nearest to itself, which is the first one's spot.
   - A third, loaded, heads for the wall point nearest to itself by the hatch, which is where the second stands.
   - The second and third each want the spot the other is standing on. A unit near an occupied goal "has arrived" and waits (0026, 0169). So they wait and retry for ever, without ever being "stuck".
2. **The moving unit's sidestep already exists for every unit built** (M1, `docs/movement.md > 6.3`, approved then).
   - A mover looks 4 body lengths ahead and shifts to a clear lane. On contact it steps 45° or along the blocker's face, to its right first, switching sides after 1 s, before the stuck rule.
   - Measured: two Agents swapping places across 21 GU take 225 ticks alone, and 227 meeting head-on in the open or in a 2 GU corridor. **A head-on meeting costs 2 ticks, not 2 s.** In a 1 GU corridor they can't pass.
   - `systems/units.md > Ground Collision` doesn't describe it. It says moving units "re-route or eventually give up", which is what 0286 quotes.
3. **0286's layout question.** M4-D's test map is open and flat: one Tunnel and **one single 1x1 patch**, nothing else.
   - At 1 GU the "strip" is the one free tile between the Tunnel's wall and the patch, **open to the north and the south**.
   - The jam happens in open ground, over spots, not in a walled lane.

## Income measured (SC/s against the 12.5 ceiling; 3 / 4 / 6 Agents, the mean of 4 layouts)
| Option | 0 GU | 1 GU | 2 GU |
|---|---|---|---|
| As built | 5.6 / 7.0 / 8.4 | 6.7 / 6.7 / 8.5 | 10.0 / 9.7 / 10.6 |
| 6. Pass-through | **12.5 / 12.5 / 12.5** | **12.5 / 12.5 / 12.5** | **12.5 / 12.5 / 12.5** |
| 4. Two-square hatch | 5.6 / 7.0 / 8.4 | 7.1 / 8.1 / 8.6 | 9.9 / 10.2 / 10.3 |
| 3. Quick re-route | 5.6 / 7.0 / 7.0 | 9.5 / 9.3 / 10.3 | 10.3 / 10.6 / 10.6 |
| 5. Last drop-off spot | 5.6 / 7.0 / 8.4 | 5.3 / 5.3 / 8.4 | 11.2 / 12.0 / 11.8 |
| 1a. Wide step round standing units | 7.0 / 7.0 / 8.3 | 11.2 / 9.4 / 12.0 | 11.5 / 12.3 / 12.5 |
| 1a + 4 + 5 + a free-spot rule | 9.5 / 10.2 / 12.5 | 11.5 / 12.5 / 12.5 | 11.8 / 12.5 / 12.5 |

- Every row but 6 still has some layout at 5.3 at 0 or 1 GU, with Agents that deliver almost nothing.
- At 0 GU, collision alone caps a Tunnel at about 11: only two spots are within reach of both the patch and the hatch.

## What each option costs
| Option | Effort | Simulation cost | Risk | Fixes the jam? |
|---|---|---|---|---|
| **1, stage 0:** the design text describes the existing infantry sidestep | none in code | none | none | no |
| **1a:** a wider step round a row of standing units, for every mover | M | the jam case only | **high:** in the prototype, 17 tests and 2 of 9 replays moved, and the 250 v 250 battle never ended | partly: mostly at 1 GU, not at 0 GU |
| **1b:** vehicles graze round obstacles | L | per vehicle | new trigonometry paths | — |
| **2:** idle yielding for infantry only | S | none | low; today only the Drone changes | no, since nobody in the jam is idle |
| **3:** a quick re-route when the blocker is a unit | S, M to make safe | path requests in every crowd; worst tick 0.90 → 1.31 ms | **the highest:** 27 tests and 3 replays moved | no: the new route to a spot a tile away is the old one |
| **4:** a 2-square hatch, one drop-off at a time | S | none | low | no, not alone |
| **5:** workers return to their last drop-off spot | S to M | none | low | no: slightly worse at 0–1 GU, better at 2 GU |
| **6:** gathering Agents pass through each other, like Drones (0106) | S, one flag | lower than today | none | **yes**, at every distance and count measured |

On 1a and 1b:
- **1a** is what the user's "sidestep in general" would add beyond stage 0. It may be crowd logic in 0223's sense ("no special crowd logic (rings, queueing, re-checking for spots opening along a row)"). M3-G2 let the same thing in for attackers behind their own firing line only, as "pathfind around occupied space". A real version must be fenced against the occupied-goal rule (0026, 0169), the own-group rule (0027) and attack-move, and that fencing is the M.
- **1b** belongs to the vehicles milestone: no vehicle and no vehicle movement model is built yet. It is roughly a third of the vehicle-movement work.

**Interactions between options:**
- 2 makes 1a more needed: every vehicle that stops yielding becomes a fixed obstacle.
- 1a and 3 both act before the stuck rule; 3 adds nothing to 1a.
- With 6, none of 3, 4 or 5 adds anything for the jam.

## Options
- **(a) The Director's technical view:** 6 for the jam, plus stage 0 for the sidestep.
  - 6 is the cheapest, safest and only complete fix. In the waiting line at the hatch, idle Agents stand inside one another, as Drones do.
  - Stage 0 brings the design text up to the code.
  - Then judge 1a on its own merits, not as the jam fix. 1b comes with vehicles.
- **(b)** Keep collision, and take 1a + 4 + 5 + the free-spot rule. This is near the ceiling from 1 GU on, but it carries 1a's risk and still starves Agents at 0 GU.
- **(c)** Keep collision, and keep patches at least 3 GU from the hatch in the reference layouts (Q0076 option B). This is within 15% there; a player who builds closer jams.
