# Q0001 — Art direction for 2.5D pixel-art sprites: projection, facings, orientations, elevation

- **Date**: 2026-11-23T11:35:00-05:00
- **Blocking**: no — proceeding on assumption: placeholder shapes on a straight-down square grid, with StarCraft-1 conventions in the client (y-sorted draw order, sprites anchored at their ground position), so sprites can replace placeholders without restructuring.
- **Docs**: `core/map.md > TilePlacement` (Elevation), `core/entities.md > Structure Instance` (rotation/flips), `core/camera.md`

## Question
The user told the Director the game should look like **2.5D pixel art / sprites in the style of StarCraft 1 and Age of Empires 2**. The design docs don't record an art direction yet. Could the design capture it and settle:
1. **Projection** — SC1-style (square tile grid, sprites drawn in 3/4 view) or AoE2-style isometric (diamond tiles)?
2. **Unit facings** — how many directions per unit sprite (SC1 used 16 or 32 with mirroring; AoE2 used 8 or 16)?
3. **Building orientations** — structures can be rotated in 90° steps and flipped. With pre-drawn 2.5D sprites, each rotation needs its own artwork. Horizontal flips can mirror art, but vertical flips break lighting and perspective. Which orientations need distinct art, and are vertical flips kept?
4. **Elevation (0–16)** — how should height read on screen (cliff edges, shading, vertical offset of higher ground)?
5. **Pixel scale** — native pixels per tile (SC1: 32 px). The camera shows 28 tiles across, about 69 screen px per tile at 1920 px wide, so art would be drawn at a native size and scaled up.

## Context
M0–M1 render placeholder geometric shapes. A later milestone adds agent-generated sprites, each approved by the user. Points 1–3 decide how the client draws the map and how many frames each unit and building needs, so knowing them early avoids rework in rendering, placement and hit-testing.

## Options
- **SC1-style square grid**: matches the design's square tiles directly, and placement and hit-testing stay axis-aligned. 90° building rotation still needs up to 4 sprite sets per building.
- **Isometric diamond (AoE2)**: a classic look, but the client converts between grid and screen, placement and hit-testing become diamond-shaped, and the camera's "28 GU wide" needs reinterpreting.
- **Orientation subsets**: allow only rotations that have art (e.g. 0°/90° plus horizontal flip). Gameplay placement options shrink accordingly.
