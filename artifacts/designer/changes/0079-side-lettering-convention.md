# 0079 — Side lettering convention; Barracks and Armory are 3 wide

- **Date**: 2026-11-24T01:55:00-05:00
- **Answers**: Q0023 (point 1; points 2–3 still open)
- **Docs**:
  - `core/entities.md > Structure Type > Side lettering convention` (new)
  - `factions/gdo/structures.md > Barracks` (Size, SymmetryType, exit side)
  - `factions/cults/structures.md > Armory` (Size, SymmetryType)

## What changed and why
We accepted the Director's proposal. The user confirmed the intent: units come out of a **short side**.
- **Convention for all buildings:** SymmetryType letters name the sides **clockwise from the top edge**, in the default orientation (rotation 0, unflipped). The 1st letter is the top, the 2nd the right, the 3rd the bottom and the 4th the left. Size (Height × Width) also refers to the default orientation.
- **Barracks:** now **2x3** (2 tall, 3 wide) with ABAC. That gives top A, right B, bottom A, left C. Units exit from B, the right short side.
- **Cults Armory:** it is also ABAC with its entrance on B and its exit on C, both short sides. It changes to 2x3 the same way, so B is right and C is left.

## Implementation impact
- Footprints for both buildings are 3 wide × 2 tall at rotation 0. Sprites (0003) are rendered for that default orientation.
- Rotations and flips carry the side letters with them.

## Verification
1. Place a Barracks with no rotation. It is 3 tiles wide and 2 tall, and a produced Peacekeeper appears on the right end.
2. Rotate it 90° clockwise (R). It's now 2 wide and 3 tall, and units appear at the bottom end.
