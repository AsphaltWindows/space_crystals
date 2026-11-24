# 0092 — GDO build area follows the current buildings

- **Date**: 2026-11-24T03:50:00-05:00
- **Answers**: Q0026 (points 2 and 3; point 4 still open)
- **Docs**: `factions/gdo/structures.md > GDOBuildArea`

## What changed and why
The user chose A. The build area is always the **union of the areas of the GDO buildings the player currently owns**, so it **shrinks** when a building is destroyed. Tiles another building also covers stay in the area.
- Buildings already standing outside the shrunken area remain, and plates keep mining. Only new placements need the area.
- **Point 3**, which follows from this: losing the Deployment Center doesn't remove the area. The other buildings' areas still count, so an Extraction Facility can still place plates within them.

Reasoning: edge buildings become worthwhile targets, because destroying a creep chain takes away GDO's ability to build there.

## Implementation impact
- Matches the Director's proposals for points 2 and 3. Recompute the area (or update it incrementally) when a building is placed or destroyed.

## Verification
1. Build a Generator at the edge of the base so it extends the area, then destroy it. The overlay shrinks back, and buildings that were placed in the lost part stay.
2. Destroy the DC while an Extraction Facility remains. The EF can still place plates on patches inside its own and other buildings' areas.
