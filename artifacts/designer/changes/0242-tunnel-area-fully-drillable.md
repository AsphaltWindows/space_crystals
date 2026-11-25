# 0242 — A Tunnel's whole area must be on the map and Drillable (placement and upgrades)

- **Date**: 2026-11-25T10:25:00-05:00
- **Answers**: Q0062 (revises 0238 and part of 0241)
- **Docs**: `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule > Drillable area`, `> Tunnel Space`

## What changed and why
User: a Tunnel can only be placed if its whole Tunnel Area is on the map and Drillable. An upgrade is blocked if the larger area would include a non-Drillable (Water, Mountain) or off-map cell.
- So there are no Tunnels near the map edge, and a Tunnel by a lake or a mountain may never be upgradable.
- Every tunnel-region cell is usable.
- **Replaces 0238** (off-map cells count) and 0241's "non-Drillable cells give no Tunnel Space"; that case can no longer arise.
- 0241's "no expansions under resources" stands.

## Implementation impact
- Tunnel placement validation (ghost by knowledge, final check on arrival) and the upgrade check include area drillability and map bounds.
- Tunnel Space no longer needs an off-map or non-Drillable case.
- Starting Layouts must respect this (the map editor's starting Tunnel placement); flag if any existing layout violates it.

## Verification
- A Tunnel ghost whose 8x8 area touches Water is red.
- A Tunnel 2 tiles from the map edge is red.
- Upgrading a Tier 1 Tunnel whose Tier 2 area would reach a Mountain is unavailable, with feedback.
