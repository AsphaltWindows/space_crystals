# Q0041: Portal exits and Beacon Capacity

- **Date**: 2026-11-24T05:02:31-05:00
- **Blocking**: no. C and D build the proposals.
- **Docs**: `factions/colonists/structures.md > Robo Portal`, `> Bio Portal > Exit Behavior`; `core/entities.md > Structure Type > Exit Doorway`, `> Structure Type` (lettering, 0079); `factions/colonists/overview.md > Beacon Capacity`.

## Question
1. **When is Beacon Capacity checked?** "The player cannot build units if doing so would exceed the total available Beacon Capacity." **Proposal:** at queue time, with the cost reserved while queued, as GDO Unit Control does (0082).
2. **Capacity falls below what is used** (buildings lost). **Proposal:** units stay, and new units are refused. Units already queued keep their reservation and complete.
3. **The Robo Portal's exit side.** "A warped-in Drone may emerge from any of the 4 sides", but which? **Proposal:** the Bio Portal's rule over all four sides: the side closest to the RallyPoint; with none, or with that side blocked, a random unblocked side.
4. **"Closest to the RallyPoint".** **Proposal:** straight-line distance from the middle of each side to the rally point (an object's centre). Ties go top, right, bottom, left.
5. **The Bio Portal's AABB.** Lettering runs clockwise from the top (0079), so both A sides are adjacent: top and right in the default orientation. Is that intended? ABAB would put them on opposite sides. **Proposal:** build AABB as written.
6. **"Materializes".** **Proposal:** a Drone appears half inside at the doorway, by the Exit Doorway rule (0080), like any produced unit.

## Context
Production reuses M2's queue, rally and doorway. These are the Colonists' differences.

## Options
- **Point 1:** A, at queue time, as proposed. B, at production start, as the Syndicate does.
- **Point 3:** A, as proposed. B, a fixed side. C, always random.
- **Point 5:** A, AABB as written. B, ABAB.
- **Other points:** the proposal, or a rule you give.
