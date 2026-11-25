# Q0109 — Agents at a hatch: who gives way? Four or more Agents whose patch lies to the side of the hatch gridlock for good

- **Date**: 2026-11-25T16:24:07-05:00
- **Blocking**: yes, for the fix of this jam, which must land before the playtest: `dev_skirmish`'s own crystal field lies to the side of the Syndicate's hatch. Nothing else waits on it.
- **Docs**:
  - `systems/units.md > Ground Collision > Who yields` ("only units with **no active command** yield") and `> Idle units push apart`;
  - `factions/syndicate/units/agent.md > Gathering > Space Crystals` (0294: Agents keep collision);
  - 0303 ("a player wanting the full 12.5 at 3 GU adds a fourth Agent");
  - `systems/controls/behaviors.md > Core principle: predictable, not optimal` (0223);
  - Q0076 and Q0088.

## Question
Four or more Agents gathering for one hatch, from a patch to the side of it (so they come to the hatch along the Tunnel's wall), lock up for good, and the Tunnel's income stops. Which rule should make one of them give way?

## Context
- **Found by M4's QA.** In the 10-minute soak of the golden game on `dev_skirmish`, four Agents locked up beside the hatch at 6:54, and the income stopped for the rest of the game.
- **Slice M4-W traced it.** Ignored tests that fail as built are in `crates/sim/tests/agent_gathering/jam.rs`.
  1. Every returning Agent heads for the same spot: the near end of the wall within reach of the hatch.
  2. The one unloading there ends up boxed in, by the wall and by two Agents waiting on its side.
  3. Its movement never steps back along a wall, and its re-routes bring back the same blocked path. It retries for ever (0255: never give up).
  4. The waiting Agents stand short of the hatch, out of reach, and retry for the spot it can't leave.
  5. All of them hold an order, so by "Who yields" none of them gives way.
- **It depends on where the patch is.** With the patch facing the hatch (the economy oracle's layout), nothing jams: 10.9–12.5 SC/s at 2–5 GU. To the side (north-west), measured in SC/s with 4 / 5 / 6 Agents:

  | | 2 GU | 3 GU | 4 GU | 5 GU |
  |---|---|---|---|---|
  | As built | 0.0 / 1.8 / 0.8 | 1.7 / 4.0 / 2.7 | 10.6 / 3.4 / 3.3 | 9.1 / 2.3 / 3.2 |
  | Option (a), prototyped | 8.7 / 8.5 / 9.2 | 8.6 / 9.0 / 9.5 | 8.6 / 9.2 / 10.2 | 9.4 / 9.0 / 9.4 |
  | Option (b), prototyped | 10.1 / 1.9 / 7.5 | 10.5 / 9.2 / 10.4 | 10.5 / 10.9 / 8.5 | 10.7 / 10.8 / 10.6 |

  As built, 7 of the 15 north-west cases go over 60 s without a delivery. Three Agents (the design's saturation) don't jam, except at 1 GU to the side, which delivers nothing even with three. 0294 calls rows 0–2 theoretical.
- **Precedent:** the Colonists' gathering Drones pass through each other (0106).

## Options
- **(a) A waiting Agent steps aside like an idle unit:** an Agent whose leg to the hatch ended short, or that is waiting its turn.
  - It never went 60 s without a delivery in any case (18 s at most).
  - It is an exception to "Who yields": an Agent with an order yields.
  - It costs the reference layout a little. Three Agents facing the hatch at 3 GU drop from 10.67 to 10.5 against the table's 12.5, just outside the oracle's ±15% (0303 calls the table an ideal).
  - "First to arrive delivers next" no longer holds, and both Syndicate goldens are re-recorded.
  - A variant where Agents waiting at a patch yield too is the only one that also keeps 1 GU working.
- **(b) Agents only wait in front of the hatch square, never beside it.** Better at 3–5 GU, but it still jams for good in places: 5 Agents at 2 GU to the side, and 5 or 6 facing the hatch at 2 GU. It doesn't settle the question alone.
- **(c) A stuck Agent's re-route goes round standing units** (M3-G2's detour, for every mover). This is Q0088's option 1a, which 0294 declined. Not prototyped; Q0088 rated it high risk.
- **(d) Gathering Agents pass through each other,** like the Colonists' Drones (Q0088's option 6). This removes every jam at the hatch, but 0294 declined it to keep Agents colliding.
- **(e) Accept it** as the player's job (0223): keep patches facing the hatch, or use three Agents. But `dev_skirmish`'s own field is to the side, and the jam stops the Tunnel's income with no sign but the income.

**The Director's note:** (a) is the smallest rule that resolved every measured case. (d) is the simplest overall, but reverses 0294. Each of (a), (b) and (d) is one slice of work, with the goldens re-recorded.
