# 0241 — Water/Mountain cells give no Tunnel Space; no expansions under resources

- **Date**: 2026-11-25T10:10:00-05:00
- **Answers**: Q0062 points 3d and 5 (Q0062 complete: 0237–0241)
- **Docs**: `factions/syndicate/tunnels.md > Tunnel Space`; `factions/syndicate/structures.md > Tunnel Expansions`

## What changed and why
User:
- **No expansions under resources** (as proposed): patches and Supply Delivery Stations above block expansions.
- **Non-Drillable cells (Water, Mountain) inside a Tunnel Area give no Tunnel Space.** This replaces the Director's proposal 5. Off-map cells still count (0238). That asymmetry is the user's choice: off-map cells are a minor edge case, while water and mountain are real map features.

## Implementation impact
Tunnel-region = area cells that are off the map or Drillable, minus the Tunnel footprint.

## Verification
A Tier 1 Tunnel whose area includes 8 Water tiles gives 2 fewer Tunnel Space than the same Tunnel on open ground.
