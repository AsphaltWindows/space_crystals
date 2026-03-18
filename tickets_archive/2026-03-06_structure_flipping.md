# Ticket: Structure Flipping Support

## Current State
Structure Instance has a `Rotation` property (0/90/180/270 degrees) but no flipping support. Structures can only be oriented via rotation, giving a maximum of 4 orientations regardless of symmetry type.

## Desired State
Add horizontal and vertical flip properties to Structure Instance, and ensure the placement system accounts for flipping alongside rotation:

**Structure Instance** additions:
- `FlipHorizontal`: boolean (default false)
- `FlipVertical`: boolean (default false)

**Orientation calculation**:
- Rotation (4 positions) combined with flipping (H/V) yields up to 8 possible orientations for fully asymmetric buildings (SymmetryType::ABCD).
- More symmetric types yield fewer distinct orientations (e.g., AAAA has only 1 distinct orientation regardless of rotation/flip).
- During placement, the player can rotate in 90-degree increments AND flip across horizontal or vertical axis.

## Justification
Required by `features/entity_system.md` (Structure Type section, lines 29-31; Structure Instance section, lines 39-40). The design update `design_updates/2026-03-06_tunnel_stats_and_mechanics.md` introduced ABCD symmetry structures (Syndicate Tunnels) with per-side functions, making flipping necessary to access all 8 orientations.

## QA Steps
1. Verify `FlipHorizontal` and `FlipVertical` boolean fields exist on Structure Instance.
2. Spawn a Structure Instance with `FlipHorizontal: true, FlipVertical: false, Rotation: 0` and confirm all three fields are stored and queryable.
3. Spawn a Structure Instance with `FlipHorizontal: false, FlipVertical: true, Rotation: 90` and confirm fields are stored correctly.
4. Write a unit test that enumerates all distinct orientations for a SymmetryType::ABCD structure (4 rotations x 2 horizontal flip states = 8, though some rotation+flip combinations may be equivalent to others — verify the count of truly distinct orientations is 8).
5. Write a unit test that enumerates distinct orientations for SymmetryType::AAAA and confirms only 1 distinct orientation exists (all rotation+flip combinations produce the same result).
6. Write a unit test for SymmetryType::AABB and confirm the correct number of distinct orientations (should be 2: original and 90-degree rotation; flips are equivalent to rotations for this symmetry).

## Expected Experience
Structure Instances can be created with flip properties. Unit tests confirm that the combination of rotation and flipping produces the correct number of distinct orientations per symmetry type. ABCD structures have 8 distinct orientations, fully symmetric structures collapse to fewer.
