# 0255 — Agent gathering: Drone rules, nearest Tunnel, 1-square hatch, Agents wait forever at the hatch

- **Date**: 2026-11-25T12:40:00-05:00
- **Answers**: Q0066 (and confirms Q0065 pt 4: the expansion's cells are reserved from construction start)
- **Docs**: `factions/syndicate/units.md > Agent > Gathering > Gathering rules` (new), `> Space Crystals` (DropOffReach); `factions/syndicate/tunnels.md > Tunnel > Side B` (hatch size)

## What changed and why
User:
1. **Yes**, the Drone's job rules apply to Agents, except for patch sharing and the absence of Set Drop-off.
2. **The nearest Tunnel, regardless of how busy it is** (no load balancing). As proposed, my reading: chosen when the Gather's first return starts and kept for that Gather.
3. **Agents waiting at the hatch wait forever** and never give up (an exception to the Failure rule). This replaces the Director's proposal.
4. A patch under a plate can't be mined.
5. Agents of any player may share patches.
6. **The hatch is 1 grid square, near the middle of the side.** Agents must be within DropOffReach of it (not the whole side), which replaces the Director's whole-side proposal. The Supplies hatch is the same (Q0067).

Q0065 clarification (user): point 4 was about when the expansion's space is reserved against other placements, which is from construction start (= ghost placement today), as written in 0254.

## Implementation impact
- The drop-off target is the 1-square hatch.
- Waiting Agents never time out.
- The Drones' code placeholder (retry every 2 s, never give up) is consistent with this; consider the same rule for Drones settled unless the user says otherwise.

## Verification
- Six Agents on one Tunnel queue at the hatch and all eventually deliver; none idles with a load.
- An Agent stands beside the hatch square, not at the side's far end.
