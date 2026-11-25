# Q0063 — The Tunnel Network: transit, entering single file, and units inside

- **Date**: 2026-11-24T22:01:11-05:00
- **Blocking**: no — proceeding on assumption: the proposals below, which slice M4-C builds.
- **Docs**: `factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD` (Side A's paths, 0224 and 0225), `> Tunnel Network`; `systems/controls/commands.md > Enter`; `systems/controls/selection_and_hud.md > Selection`, `> ControlGroups`; `systems/controls/behaviors.md > Core principle: predictable, not optimal`.

## Question
1. **Transit.** "Units inside the Tunnel Network can travel between Tunnels freely": instantly? **Proposal:** yes. A unit that went in at one Tunnel can come out of any other, in the same tick.
2. **Entering, through the entry path, one unit at a time** (0225). **Proposal:**
   - **The spots.** Each path has one fixed spot just outside Side A, centred on its half of the side. Seen from outside, facing Side A, the entry path is on the left half and the exit path on the right half. Both turn and flip with the Tunnel.
   - **Going in.** A unit goes in when its silhouette is within 8 su of the entry spot (DropOffReach's value, as a placeholder) and the path is free (point 7). If several units are within reach, the first to arrive goes first.
   - **Waiting its turn.** A unit stands where it stopped, with no formation. It doesn't switch to the exit path or to another Tunnel. A unit that can't get within reach follows the normal Failure rule (0223): it gives up after about 7 s and idles.
   - A Tunnel mid-upgrade takes units at its old tier.
   - If the target Tunnel is destroyed, or its tier doesn't suit the unit, the Enter order ends and the unit idles.
3. **Orders.** **Proposal:** entering clears the unit's orders, queued ones included. It idles in the network.
4. **Control groups.** Units leave the selection on entering ("leaves the selection by entering a building, transport or tunnel"). **Proposal:** they stay in their control groups, and a recall selects only the members on the map.
5. **Loads.** **Proposal:** an Agent keeps the load it carries in the network.
6. **Healing in the network.** **Proposal:** none; no healing exists yet.
7. **How long a unit holds the entry path** (0225: TBD). **Proposal:** 8 frames (0.5 s), whatever the unit, the same gap as ejection's "every 8 frames minimum". The unit leaves the map at once, and the path takes the next unit 8 frames later, so an army goes in at 2 units per second.

## Context
The design gives the network's tier rules and its two ways in and out (Enter, Eject), but not the time to travel or what happens to a unit inside. Instant transit is the biggest balance factor here: with it, any army can reach any Tunnel at once.

0224 and 0225 give Side A "two paths: one for entering, one for exiting". They also say: "Each path takes one unit at a time and fixes where units go in and come out ... an army enters or leaves single file." They leave open which half of Side A each path takes, and how long a unit holds one: 0225 says "TBD; e.g. until it has fully cleared it". The proposals follow 0223's principle ("prefer the simple, explainable rule"): a fixed spot, a fixed time, and no rule that moves waiting units round.

## Options
- **Point 1:** A, instant, as proposed. B, a delay you give, by distance or flat.
- **Point 2:**
  - A, as proposed;
  - B, the entry path on the right half instead;
  - C, a waiting unit keeps its turn however long the line takes, instead of following the Failure rule.
- **Point 4:**
  - A, units stay in their groups, as proposed;
  - B, they are removed, as a destroyed unit is ("destroyed entities are silently removed from the group").
- **Point 5:** A, the load is kept, as proposed. B, it is delivered on entering.
- **Point 7:**
  - A, 8 frames, as proposed;
  - B, as long as the unit takes to walk the path's length at its own speed, so slower units hold it longer;
  - C, a time you give, per unit or per unit base.
