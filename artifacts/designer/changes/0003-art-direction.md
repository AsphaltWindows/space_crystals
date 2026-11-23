# 0003 — Art direction: square grid, 3D-rendered sprites, 32 px/tile, all building orientations, 16/32 unit facings

- **Date**: 2026-11-23T13:00:03-05:00
- **Answers**: Q0001 (points 1, 2, 3, 5; point 4 "elevation display" still open, answer to follow)
- **Docs**: `core/art_direction.md` (new: Projection, Sprite Production, Native Pixel Scale, Building Orientations, Unit Facings, Elevation Display = TBD); `README.md` index

## What changed and why
The user picked a 2.5D pixel-art direction in the spirit of StarCraft 1 / Age of Empires 2:
1. **Projection:** square tile grid with 3/4-view sprites, not isometric diamonds.
2. **Sprite production:** sprites are **pre-rendered from 3D models**. This is what makes every building orientation and many unit facings affordable and consistent.
3. **Building orientations:** no restriction. All rotations and flips from `core/entities.md > Structure Type` stay, and each distinct orientation gets its own render. Flips are rendered from the mirrored model, so lighting stays correct and vertical flips are kept. Renders per SymmetryType: AAAA 1, ABAB 2, AABB/AAAB/ABAC 4, AABC/ABCD 8.
4. **Unit facings:** infantry 16; vehicles, turrets and aircraft 32. This is visual only, since simulation facing stays continuous.
5. **Native scale:** 32 px per tile (1 px = 2 space units), scaled up to the 28-tile-wide camera.

## Implementation impact
- Confirms the Director's square-grid assumption.
- The art pipeline must produce **3D models rendered to sprites**, not hand-drawn or generated 2D frames. The "agent-generated sprites, approved by the user" milestone should plan for models plus a render step at the fixed game camera angle.
- Building sprite lookup is keyed by (Rotation, FlipHorizontal, FlipVertical), collapsed by SymmetryType. Unit sprite lookup quantizes facing to 16 or 32 steps.
- Elevation display is still open. Don't commit to an elevation rendering scheme yet; the user is leaning towards AoE2-like sloped terrain, and that may change the map data model (an entry will follow).

## Verification
- Placing a building in each of its orientations shows a matching, correctly lit sprite (no mirrored lighting on flips).
- A turret tracking a moving target and a wheeled vehicle taking a curve rotate through 32 visible steps; infantry turn through 16.
- At 1920 px wide, one tile is about 69 screen px and the native pixels are visibly about 2× scaled.
