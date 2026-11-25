# 0258 — Losing Tunnels: decay %, attacking neutral expansions, reclaiming, blocked cells, defeat, visible tier

- **Date**: 2026-11-25T13:10:00-05:00
- **Answers**: Q0069
- **Docs**: `factions/syndicate/tunnels.md > Losing Tunnels`, `> Tunnel Area Non-Overlap Rule` (enemies see the tier); `core/game_rules.md` (defeated Syndicate units)

## What changed and why
User:
1. **Decay** is a percentage of MaxHP per second. 1% per second is a placeholder; the user isn't sure it's right, so tune it.
2. Neutral expansions can be attacked **only with explicit Attack**.
3. **Reclaiming** happens when the covering Tunnel, or its upgrade, finishes.
4. A neutral expansion partly inside an own Tunnel Area **still blocks its cells and counts as occupied for Tunnel Space**. This replaces the Director's proposal.
5. A defeated Syndicate player's units in the network are destroyed.
6. Enemies see a Tunnel's tier.

## Implementation impact
As proposed, except pt 4: count those cells as occupied.

## Verification
—
