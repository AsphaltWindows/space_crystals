# Q0066 — Agent gathering: the Drone's rules, the Tunnel it uses, and the line at Side B

- **Date**: 2026-11-24T22:01:11-05:00
- **Blocking**: no — proceeding on assumption: the proposals below, which slice M4-D builds.
- **Docs**: `factions/syndicate/units.md > Agent > Gathering`, `> Right-Click Resolution`; `factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD` (the hatches, 0224); `factions/colonists/units.md > Drone > Gathering` and its subsections, `> Assigned Drop-off`; `systems/controls/behaviors.md > Core principle: predictable, not optimal`.

## Question
1. **Do the Drone's job rules apply to the Agent?** These are `> Last load of a patch`, `> Interrupted mining`, `> Other orders keep the load`, `> Gather while carrying a load`, `> Drop Off order` and `> When a Gather ends` (0123, 0127–0141, 0170). There are two exceptions: patch occupancy (Agents share patches) and the Set Drop-off command (the Agent's panel has none; point 2). **Proposal:** yes.
2. **Which Tunnel an Agent delivers to.** **Proposal:** the Drone's AssignedDropOff, auto-assigned, without the Set Drop-off command.
   - As a Gather's first return starts, the Agent takes the nearest own operational Tunnel, by straight line to the middle of Side B.
   - It keeps that Tunnel for the rest of the Gather, even if another becomes nearer. If the Tunnel is lost, it takes the nearest again (0135, 0170).
   - A new Gather order takes the nearest again.
   - A Drop Off order delivers to the Tunnel clicked, once.
3. **The line at Side B.** **Proposal:**
   - an Agent stops as soon as it is within DropOffReach of Side B, wherever along the side that is, and waits there while another Agent drops off;
   - when the side frees, the waiting Agent that arrived first drops off next;
   - an Agent that can't get within reach, because others are in the way, follows the normal Failure rule (0223): it gives up after about 7 s and idles, still holding its load;
   - no Agent moves on to another side or Tunnel by itself, or looks for a gap.
4. **A patch under a GDO Extraction Plate.** **Proposal:** as for Drones (0137), Agents can't mine it.
5. **Agents of different players at one patch.** **Proposal:** allowed. Any number, from any player.
6. **Where the hatch is.** 0224 makes Sides B and C each "a hatch". **Proposal:** the whole side, as DropOffReach says ("within 8 su of Side B"). The hatch is drawn at the side's middle, but it doesn't narrow where Agents can deliver. Side C's Supply hatch is the same (Q0067).

## Context
The Agent's section gives the numbers, and says "The only queue in an Agent's trip is at the Tunnel's Side B". It gives none of the transitions that the Drone's section settled through Q0037. Agents keep normal collision while gathering (0106: "Drones only").

The proposals for points 2 and 3 follow 0223's principle: "prefer the simple, explainable rule over a cleverer one that handles more cases, as long as the simple rule fails in ways the player can see and fix".
- **Point 2.** An earlier draft re-chose the nearest Tunnel at every return. That would make Agents between two stacked Tunnels switch between them on small differences in where they stand. Keeping the Tunnel is the Drone's own rule (`> Assigned Drop-off`, "Auto-assignment").
- **Point 3.** Drones now try a blocked return again every 2 s and never give up. That is a placeholder in the code, since the design has no rule for a worker that can't reach its drop-off. For Agents the proposal uses the Failure rule instead, as 0223 does for attackers. An idle Agent holding a load is something the player can see and fix.

## Options
- **Point 1:** A, yes, as proposed. B, a list of the rules that differ.
- **Point 2:**
  - A, keep the Tunnel for the Gather, as proposed;
  - B, the nearest at every return. This follows a new Tunnel at once, but may switch between two that are nearly as near.
- **Point 3:**
  - A, as proposed;
  - B, an Agent that can't reach waits as close as it got and tries again until it can, never giving up. This is what Drones do now.
- **Point 6:**
  - A, the whole side, as proposed;
  - B, a fixed hatch spot at the side's middle. Fewer Agents then fit within reach at once, so a crowd forms sooner.
