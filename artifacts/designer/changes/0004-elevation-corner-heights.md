# 0004 — Elevation: corner heights, gentle walkable slopes, cliffs, flat-only building

- **Date**: 2026-11-23T13:11:54-05:00
- **Answers**: Q0001 (point 4, elevation display; with 0003 this completes Q0001)
- **Docs**: `core/map.md > TilePlacement` (Elevation field removed), `core/map.md > Elevation` (new), `core/vision.md > ElevationModifier`, `core/entities.md > Placement Validation` (all three methods), `core/art_direction.md > Elevation Display`, `README.md`

## What changed and why
The user wants AoE2-like terrain with real slopes. Chokepoints come from cliffs combined with elevation differences, not from ramps.
- **Data model:** height is stored at **tile corners** ((W+1) x (H+1) integers, 0–16), replacing the per-tile `Elevation` on TilePlacement. Adjacent tiles share corners, so terrain is continuous.
- **Maximum slope:** on a Traversible tile, the corners differ by **at most 1 level**. Larger differences are only allowed on non-Traversible tiles (e.g. Cliff). The map editor enforces this.
- **Terrain height at a point:** interpolated from the tile's corners. The method is your choice, but it must be continuous across tile edges.
- **Elevation level:** terrain height at the object's position, **rounded down**. ElevationModifier (±1 sight/attack range, binary) compares elevation levels. The user accepted that the level flips abruptly partway up a slope.
- **Building:** surface structures only on **flat** tiles (all four corners equal), in addition to Buildable. This applies to Direct Placement, Worker-Built Structures (validated on arrival) and Warp In.
- **Display:** slopes shaded by facing direction, higher terrain drawn shifted up the screen, cliffs drawn as cliff faces. The vertical offset per level is TBD (art tuning).

## Implementation impact
- Replaces the per-tile elevation in `core/map.md > TilePlacement`. Map files and the editor store a corner heightmap instead.
- The map editor must reject Traversible tiles whose corners differ by more than 1.
- All structure placement checks gain a flatness check (ghost red / validation fails on arrival if any footprint tile is sloped).
- ElevationModifier uses floor(interpolated height) for ground and underground units (underground: the terrain above them).
- Slopes have **no** movement-speed effect for now (an open design question; it would be a later entry).
- Open, not decided: whether the flatness rule applies to Syndicate underground expansions (docs currently say "surface structures"). Assume it doesn't until a follow-up entry.

## Verification
- In the editor, raise one corner of a Plane tile by 2: the editor refuses (or flags it) unless the tile is non-Traversible.
- A hill made of 1-level steps renders as shaded slopes, and a unit walking up it visibly rises.
- Try to place a building with one footprint tile on a slope: the ghost is red. On a flat plateau it's green.
- A ranged unit standing above the next integer level gets +1 range against a unit below; just under that point, no bonus.
