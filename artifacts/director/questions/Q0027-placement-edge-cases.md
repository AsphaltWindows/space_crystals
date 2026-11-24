# Q0027: Placement edge cases

- **Date**: 2026-11-23T22:18:49-05:00
- **Blocking**: no. M2 builds the proposals.
- **Docs**:
  - `core/entities.md > Placement Validation`;
  - `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`;
  - `core/map.md > Resource`, `> SpaceCrystalsPatch`;
  - `core/map_editor.md > Starting Layouts > Overlay editing`.

## Question
1. **Units under the footprint.**
   - Direct Placement rules out overlapping structures only. Worker-Built and Warp In require "unoccupied" tiles.
   - **Proposal:** a ground unit of any player that stands on the footprint makes those tiles red, and the building can't be placed there.
   - **Alternative:** own units step aside and the placement succeeds.
2. **Plates and visibility.**
   - Direct Placement requires every tile to be Visible. The DC's build area reaches 12 grid units beyond it, but its sight covers only about 4 beyond its edge, and the EF's sight is 3. Patches inside the area may be under fog.
   - **Proposal:** keep the rule as written. A plate can only go on a patch the player currently sees.
   - **Alternative:** plates are exempt from the visibility rule.
3. **Other buildings on patches and stations.**
   - Direct Placement forbids overlapping structures, but patches and stations are Resources, not structures.
   - **Proposal:** no building may cover a patch or a station, except a plate on a patch.
4. **Which rules a Starting Layout follows.**
   - A layout "only needs to be valid on its own (normal placement rules)".
   - **Proposal:** for GDO, the tiles must be Buildable and flat, with no overlaps except plates on patches, and the EF and plates must touch the build area of the layout's own DC. There is no visibility check.

## Context
The placement rules are built in M2. The ghost shows each failed rule per tile.

## Options
- **Point 1:** A, refuse; B, own units step aside.
- **Point 2:** A, plates need visible tiles; B, plates are exempt.
- **Point 3:** A, no building on resources except plates on patches; B, allowed.
- **Point 4:** A, as proposed; B, the Designer's rule.
