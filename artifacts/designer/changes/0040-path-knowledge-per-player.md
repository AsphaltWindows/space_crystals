# 0040 — Pathfinding uses only what the owner knows (per-player obstacle cache)

- **Date**: 2026-11-23T19:33:39-05:00
- **Answers**: Q0011 (part 3; only part 4, later domains, still open)
- **Docs**: `systems/controls/behaviors.md > PathKnowledge` (new); `systems/units.md > UnitBase > RuggedTerrain` (ground domain only, clarification of 0039)

## What changed and why
Option B, so that pathing can't be used to detect hidden enemy buildings:
- Terrain is always known, and so are your own structures.
- Other players' structures are known only once seen. A seen structure now in fog is a **wall at its last-known position** (even if since destroyed) until seen otherwise. Never-seen structures are open ground.
- **Per-player known-obstacle cache**, updated when a structure enters or leaves the player's view.
- When a walking unit gets stuck, a cache change since its plan was made triggers a **new path calculation** rather than counting toward giving up (0024).

Also, a clarification of 0039: the RuggedTerrain flag applies to the ground domain only. Air units ignore terrain.

## Implementation impact
- **Replaces** the Director's default A (true map).
- Per-player obstacle layer for pathfinding, built from the vision system's last-known structure state.
- TBD (assumption until decided): map-placed patches and Supply stations are known from the start like terrain, and a patch depleted in fog is only learned when seen.

## Verification
- Build a hidden wall of enemy buildings across a gap in fog. Order a unit through the gap: it heads for the gap, sees the wall, and re-routes. It never avoids the gap before seeing the wall.
- Scout an enemy building, leave, have it destroyed out of sight, then path through its spot: the unit routes around the remembered building until it sees the spot is empty.
