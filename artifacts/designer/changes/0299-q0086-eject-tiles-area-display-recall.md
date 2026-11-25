# 0299 — EjectMenu tiles fixed; when Tunnel Areas are shown; recall of an all-inside group

- **Date**: 2026-11-26T12:00:00-05:00
- **Answers**: Q0086 points 2, 4, 5 (point 3 pending with the user)
- **Docs**: `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState > EjectMenu`, `> Tunnel Area display` (new), `> Tunnel Network > On entering`

## What changed and why
User:
- **Point 2 (a):** every Syndicate unit type keeps its EjectMenu tile; types with none inside show 0, greyed.
- **Point 4 (new):**
  - selecting one of your Tunnels shows its area;
  - placing a Tunnel shows the Tier 1 area and the would-be Tier 2 and Tier 3 areas, with cells that would block an upgrade highlighted red (non-Drillable; off-map is my reading), so players know whether it can be fully upgraded;
  - **enemy areas are never shown**;
  - view-mode toggles maybe later.

  This differs from the Director's built version (a): no remembered enemy areas, and the upgrade previews are added.
- **Point 5 (a):** recalling an all-inside group does nothing.

## Implementation impact
- Remove the enemy-area drawing.
- Add the T2/T3 preview with red blocking cells to Tunnel placement.
- Open point for the user: whether overlap with your own other Tunnels' areas also shows red in the preview.

## Verification
- Place a Tunnel near Water: the T3 ring shows red over the Water.
- Select an enemy Tunnel: no area is drawn.
