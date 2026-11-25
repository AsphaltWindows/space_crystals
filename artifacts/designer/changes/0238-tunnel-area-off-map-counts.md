# 0238 — Tunnel Area cells off the map still count for Tunnel Space

- **Date**: 2026-11-25T09:40:00-05:00
- **Answers**: Q0062 point 2
- **Docs**: `factions/syndicate/tunnels.md > Tunnel Space`

## What changed and why
User chose **B**: the whole square counts for Tunnel Space, including cells past the map edge. Expansions can't be placed there. The advantage of not being crowdable from that side is judged minor next to the lost expansion room.

## Implementation impact
This replaces the Director's on-map-only proposal. Tunnel Space uses the full area square, and placement still only allows on-map cells.

## Verification
A Tier 1 Tunnel against the map edge gives the same Tunnel Space as one in the middle.
