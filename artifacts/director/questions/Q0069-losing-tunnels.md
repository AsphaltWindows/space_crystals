# Q0069 — Losing Tunnels: the open cases

- **Date**: 2026-11-24T22:01:11-05:00
- **Blocking**: no — proceeding on assumption: the proposals below, which slice M4-H builds, with a named placeholder for the decay rate (point 1).
- **Docs**: `factions/syndicate/tunnels.md > Losing Tunnels`; `core/game_rules.md > Defeat Condition`, `> On Defeat`; `systems/controls/selection_and_hud.md > RightClickResolution`.

## Question
1. **The decay rate** ("Decay rate: TBD"). **Proposal:** 1% of MaxHP per second, so a Hideout lasts 100 s. A named placeholder.
2. **Attacking neutral expansions.** **Proposal:** allowed with an explicit Attack. A right-click never attacks a neutral object (0059), and auto-targeting ignores them.
3. **When reclaiming happens.** **Proposal:** the moment the covering Tunnel is finished, or its upgrade is. A Tunnel under construction reclaims nothing.
4. **A neutral expansion partly inside an own Tunnel Area.** **Proposal:** it keeps its cells, so nothing can be built over them, but it doesn't count against that Tunnel's tunnel-region until it is reclaimed.
5. **A defeated Syndicate player's units in the network.** **Proposal:** they are destroyed, since no exit remains, rather than turned neutral.
6. **Does an enemy see a Tunnel's tier?** **Proposal:** yes. The Tunnel is on the surface, and its label shows the tier.

## Context
The rules for losing a Tunnel are stated in full, except for these edges. Point 6 matters if Q0062 point 1 is answered B, because the ghost then needs the enemy's tier.

## Options
- **Point 1:** A, as proposed. B, a rate you give.
- **Point 2:**
  - A, an explicit Attack only, as proposed;
  - B, also auto-targeted;
  - C, not attackable.
- **Point 4:** A, as proposed. B, it counts as occupied.
