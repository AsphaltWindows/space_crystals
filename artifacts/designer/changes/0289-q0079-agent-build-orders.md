# 0289 — Agent Tunnel orders: killed Agent refunds; pending ghosts and sites reserve their areas

- **Date**: 2026-11-26T09:40:00-05:00
- **Answers**: Q0079
- **Docs**: `factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`, `> Tunnel Area Non-Overlap Rule` (Pending ghosts and sites count)

## What changed and why
User, all as proposed:
1. If an Agent is killed before construction starts, its order is refunded at the current top price.
2. A pending ghost reserves its whole Tier 1 area against its owner's further Tunnel ghosts, which show red. It is the owner's only, since other players don't know about it.
3. A site counts with its whole area for every player from the moment it appears.

## Implementation impact
None; as proposed.

## Verification
- Shift-queue two Tunnels whose areas overlap: the second ghost is red.
- Kill an Agent walking to build: the Supplies come back.
