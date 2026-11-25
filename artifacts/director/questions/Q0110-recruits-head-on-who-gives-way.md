# Q0110 — Recruits head-on in a lane: who gives way? More Recruits than a Storage face's lanes starve each other for minutes

- **Date**: 2026-11-25T17:35:27-05:00
- **Blocking**: yes, for this jam's fix, which must land before the playtest. A Recruitment Hall's rally on a patch sends every new Recruit to that one patch, so ordinary play overloads a face. Nothing else waits on it.
- **Docs**:
  - `factions/cults/units/recruit.md > Gathering` ("A 64 su gap fits two Recruits side by side (one lane in, one lane out)"; patch sharing);
  - `factions/cults/economy.md > Storage` ("each 2-tile face supports roughly 4–5 Recruits at full efficiency");
  - `systems/units.md > Ground Collision > Who yields` ("only units with **no active command** yield");
  - `factions/colonists/units/drone.md > Gathering Drones pass through each other` (0106: "Recruits mining a patch still bump into each other");
  - `systems/controls/behaviors.md > Core principle: predictable, not optimal` (0223);
  - Q0109 and its first-step answer 0315.

## Question
When more Recruits gather from a patch than its face has lanes, two Recruits meet head-on in one lane, with a working Recruit on either side, and neither can pass. All of them hold an order, so none gives way, and the pair stands for minutes. Which rule should make one of them give way?

## Context
- **Built and measured by slice M5-D** (Recruit gathering; tests in `crates/sim/tests/recruit_gathering/`).
  - **Within the design's numbers nothing jams.** Four or five Recruits on a two-patch face at a 1-tile gap give 5.3 and 6.1 SC/s, with the longest gap without a delivery 40 and 132 ticks. Ten Recruits on an 8-patch field with three Storages give 12.9 SC/s.
  - **Beyond them it jams.** The rates below are SC/s over 5 minutes, with the longest any Recruit went without delivering (16 ticks = 1 s):

  | Recruits | 3 | 4 | 5 | 6 | 7 | 8 |
  |---|---|---|---|---|---|---|
  | One patch facing the Storage, 1-tile gap | 3.77 / 110 | 2.30 / 2023 | 1.46 / 3086 | 1.60 / 3287 | 0.42 / 4597 | 0.41 / 4585 |
  | Two patches (the whole 2-tile face), 1-tile gap | 3.99 / 40 | 5.32 / 40 | 6.10 / 132 | 3.94 / 881 | 3.00 / 1193 | 2.19 / 2489 |

  The 8-patch field with 14 / 16 / 20 Recruits gives 15.7 / 15.8 / 13.6 SC/s, with gaps of 471 / 965 / 2618 ticks. At a 2-tile gap the lanes are long enough to pass in, and crowds only slow down (the worst gap is about 520 ticks).
- **The mechanism.**
  1. A patch's face holds two lanes of 24 su Recruits, and every leg heads for the point nearest the Recruit, so each Recruit shuttles in its own lane.
  2. With a fourth Recruit on one patch, two end up in one lane head-on: one loaded, wanting the Storage behind the other, and one empty, wanting the patch behind the first.
  3. A working Recruit shuttles on either side, so neither can step round. Both legs end short and retry for ever, for the same point.
  4. All four hold orders, so by "Who yields" nobody makes way, until some neighbour's jitter opens a lane, minutes later.
- **0315's rule (for Agents) doesn't fix it.** M5-D prototyped it for Recruits and reverted it, since the design doesn't extend it to them: a leg that never counts as arrived short, and retries at once. Of 99 cases, a Recruit went over 10 s without a delivery in 44 as built, 41 with the prototype, and 37 with the patch leg exempt too. The jam isn't a leg ending early. It is two units head-on with no room to pass, and "keep moving" doesn't make either give way.
- **Why ordinary play reaches it.** A Hall with its rally on a patch makes up to 20 Recruits, and each new one gathers from that patch (patch sharing).
- **The Agents' Q0109** is the same family: there, every return heads for one hatch point. Its option (a) is waiting on the user's decision after M4-W2's measurements, so both could be settled with one rule.

## Options
- **(a) A blocked worker steps aside like an idle unit:** a gathering worker whose leg is blocked short of its target makes way like a unit with no active command (Q0109's option (a), for every gathering worker). It is an exception to "Who yields". It resolved every measured Agent case in M4-W's prototype; it is not yet measured for Recruits.
- **(b) Right of way: the loaded worker goes first.** When two gathering workers block each other head-on, the one without a load steps aside. It is predictable, and narrower than (a). Not measured.
- **(c) Recruits pass through each other while gathering,** like the Drones. This removes the jam, but reverses 0106's "Recruits mining a patch still bump into each other".
- **(d) Accept it as the player's job** (0223): spread Recruits over patches, keep a face at 4–5. But a Hall's rally on a patch overloads one patch with no action by the player.

**The Director's note:** (a) and (b) are each one slice, measured against the table above with the same tests. (a) would also be the backstop the user is considering for the Agents (Q0109), so one rule could cover every gathering worker.
