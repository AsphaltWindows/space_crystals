# 0240 — Surface buildings and underground expansions never share a tile

- **Date**: 2026-11-25T10:00:00-05:00
- **Answers**: Q0062 points 3c and 4 (point 3d and point 5 pending)
- **Docs**: `factions/syndicate/structures.md > Tunnel Expansions`; `factions/syndicate/tunnels.md > AwaitingPlacement (Expansion)`

## What changed and why
User: no surface building on top of an expansion, and no expansion under a surface building. This applies to any player's, neutral included.
- An enemy's **direct placement** (GDO) over a hidden expansion is red at once. That leak is accepted, since GDO is limited by its build area.
- **Worker-built** ghosts judge by what the player knows, so they stay green and the order fails on the worker's arrival.
- Units still walk over expansions.

This **replaces the Director's proposals 3c and 4** (surface ignores underground, except the Tunnel).

## Implementation impact
- Placement checks in both layers consult the other layer's truth.
- For direct placement the leak is accepted. For worker-built placement the knowledge-based ghost is kept, with failure on arrival.

## Verification
- A GDO player's ghost over a hidden Syndicate Hideout is red.
- A Colonists Drone ordered to deploy there fails on arrival.
- A Syndicate player can't place an expansion under an enemy's building.
