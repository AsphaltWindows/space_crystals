# Q0064 — Ejection and production out of the network

- **Date**: 2026-11-24T22:01:11-05:00
- **Blocking**: no — proceeding on assumption: the proposals below, which slice M4-C builds.
- **Docs**: `factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD` (Side A's paths and the Tier 3 doors, 0224 and 0225), `> EjectMenu`; `factions/syndicate/structures.md > Rally Point Behavior`; `core/entities.md > Exit Doorway`; `systems/units.md > Underground Collision`; `systems/controls/selection_and_hud.md > What a rally point makes a new unit do`; `systems/controls/behaviors.md > Core principle: predictable, not optimal`.

## Question
1. **Where an ejected unit appears.** The Exit Doorway rule excludes Tunnels, and 0225 gives the exit path one fixed spot. **Proposal:**
   - the unit appears on the exit path's spot, just outside Side A, centred on the right half of the side (Q0063 point 2);
   - it can't appear where its silhouette would overlap a unit or a structure (as in Surfacing). The queue waits while the spot isn't clear;
   - no other spot is tried (0223).
2. **After it appears, with no order.** **Proposal:** it walks 1 GU straight out of Side A and idles, like the doorway's "straight out of the exit side if there is none". If something blocks it, the normal Failure rule applies, and the queue waits until the spot is clear.
3. **Which unit of a type comes out.** **Proposal:** the one that has been in the network longest.
4. **"A new unit begins ejecting every 8 frames minimum".** **Proposal:**
   - a unit appears at once, and at most one per 8 frames per Tunnel;
   - each click reserves a specific unit, so the tile's count drops at the click;
   - the queue can't be cancelled;
   - a Tunnel destroyed with a queue leaves the queued units in the network, where the stranded-unit rule applies.
5. **A rally on another own Tunnel.** The default right-click there is Enter, which ends in the network anyway. **Proposal:** the unit stays in the network, as for a rally on the parent Tunnel.
6. **A rally on an own expansion**, such as another Hideout. **Proposal:** the unit stays in the network.
7. **"Moves to the rally point".** **Proposal:** read it as 0136's default right-click against the rally target, so an Agent rallied on a patch gathers. Please confirm.
8. **Auto-ejected units.** **Proposal:** they join the parent Tunnel's eject queue in order with manual ejects, and don't show in the EjectMenu while they wait.
9. **The notification for a blocked queue** (Tunnel Space). Notifications aren't designed. M4 shows the waiting state on the producer's panel. FYI.
10. **How long a unit holds the exit path** (0225: TBD). **Proposal:** from when it appears until its silhouette has left the exit spot, and at least 8 frames ("every 8 frames minimum"). A slow unit holds the path longer, as the EjectMenu expects: "actual throughput is limited by unit speed and collision at Side A".
11. **The Tier 3 air doors.** 0224 adds "two horizontal circular sliding doors, one for flying units entering and one for flying units exiting", and 0225 leaves open "Whether the Tier 3 air doors also take one aircraft at a time". M4 has no Syndicate aircraft, so this isn't needed before the Phantom (0226). **Proposal:**
    - the same rules as the paths: one aircraft at a time per door, held as long as the matching path (Q0063 point 7, and point 10 here);
    - the doors are on the Tunnel's roof: the entry door over the entry path's half, and the exit door over the exit path's half.

## Context
The EjectMenu says that "Standard movement and collision mechanics apply as units emerge", but not what happens when the way out is blocked, or which unit of a type comes out. 0225 now fixes where units come out ("Each path takes one unit at a time and fixes where units go in and come out"). It leaves open how long a unit holds the path.

An earlier draft of point 1 searched along Side A for the clear spot nearest the middle. That is the kind of rule 0223 rules out ("re-checking for spots opening along a row"), so it is dropped. Now a unit standing on the exit spot, whoever owns it, holds up the queue, and the player can see that and fix it.

## Options
- **Point 1:** A, the exit spot only, as proposed. B, half inside the footprint, like a doorway. This contradicts Exit Doorway's last line.
- **Point 3:** A, the longest in the network, as proposed. B, the healthiest. C, the player picks a unit.
- **Points 5 and 6:** A, stays in the network, as proposed. B, comes out and walks to the target.
- **Point 10:** A, until the spot is clear and at least 8 frames, as proposed. B, another rule you give.
- **Point 11:**
  - A, one at a time, as proposed;
  - B, any number at once;
  - C, rules you give, with where the doors are.
