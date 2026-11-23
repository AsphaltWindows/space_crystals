# 0013 — Crystal patches and Supply Delivery Stations need flat tiles

- **Date**: 2026-11-23T13:42:21-05:00
- **Answers**: Q0006
- **Docs**: `core/map.md > Elevation > Map resources only on flat terrain` (new)

## What changed and why
Q0006 option C: **both** Space Crystals patches and Supply Delivery Stations must be placed on flat tiles. Extraction Plates are structures and need flat ground (0004), so a patch on a slope would be unminable by GDO. Requiring flat patches keeps every patch usable by every faction with no plate exception. Stations are 2x2 building-like sprites, so keeping them level also avoids art problems.

## Implementation impact
- **Replaces the Director's assumption (A).** The editor's placement ghost for patches and stations goes red on any sloped tile, and map validation flags existing violations (e.g. if terrain under a patch is later reshaped).
- No change for Extraction Plates: the existing flatness rule always holds on patch tiles.

## Verification
- In the editor, try to place a patch or station on a slope: refused (red ghost). On flat ground: accepted.
- Place a patch, then raise one of its tile's corners: the map is flagged invalid until fixed.
