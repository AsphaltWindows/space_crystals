# Q0023: Barracks footprint, sides and the exit

- **Date**: 2026-11-23T22:18:49-05:00
- **Blocking**: no. M2 builds the proposals. Please answer before production and placement are finished.
- **Docs**:
  - `factions/gdo/structures.md > Barracks` (Size 3x2, SymmetryType ABAC, "Units exit from one of the short sides (B side)");
  - `core/entities.md > Object Type > Size - Height x Width`, `> Structure Type`.

## Question
1. **Footprint and side letters at rotation 0.**
   - `core/entities.md` says Size is Height × Width, so "3x2" means 3 tall and 2 wide.
   - SymmetryType names the sides, but not which edge is A or which way the letters run. With ABAC, B and C face each other, and the Barracks text needs B on a short side.
   - The natural rule is "A is the top edge, letters run clockwise". With a 2-wide, 3-tall footprint, that puts B on a long side.
   - **Proposal:**
     - At rotation 0 the Barracks is 3 wide and 2 tall. Under Height × Width its size would then be written 2x3.
     - Sides run clockwise from the top: top A, right B, bottom A, left C. Units exit on the right.
     - The same rule applies to every building.
2. **Where a produced unit appears.**
   - **Proposal:** just outside the middle of side B. If that spot is taken by units, the unit appears at the nearest free spot, searching outward from there.
3. **A blocked exit.**
   - Buildings can be placed flush against side B, and the placement rules don't keep exits clear.
   - **Proposal:** the unit appears at the nearest free spot around the building, starting from side B. Production never stalls.

## Context
The orientation tools (R, Shift+R, F, Shift+F and the A/B/C/D labels) are built in M2. The Barracks is the first building that isn't square, and the first with a meaningful side. The art pipeline (0003) renders one sprite per distinct orientation, so the convention also matters for art.

## Options
- **Point 1:**
  - A (proposal): 3 wide × 2 tall, sides clockwise from the top.
  - B: 2 wide × 3 tall as written, with letters starting on a long side (for example, clockwise from the left: left A, top B, right A, bottom C).
- **Points 2 and 3:**
  - A (proposal): the nearest free spot around the building, starting at side B.
  - B: production waits until side B is clear.
  - C: placement refuses a building that would block another building's exit.
