# 0088 — Rally points on enemies become ground rallies when the enemy leaves sight

- **Date**: 2026-11-24T03:20:00-05:00
- **Answers**: Q0024 (point 7; Q0024 is now fully answered)
- **Docs**:
  - `systems/controls/selection_and_hud.md > Rally points on enemy objects` (new)
  - pointers in the RallyPoint behavior of `factions/gdo/structures.md > Barracks` and `factions/colonists/structures.md` (Robo Portal, Bio Portal)

## What changed and why
The user chose B. A rally point on an enemy object follows the object **only while the owner can see it**. When the enemy leaves sight, the rally point **turns into a ground rally point** at the enemy's last-seen position:
- A unit produced while the enemy is visible attacks it.
- A unit produced after the enemy was lost from sight moves to the last-seen position. This matches the rule for chasing into the fog.
- This also closes an information leak. An enemy dying unseen no longer clears the rally point. The "reset to None when the object no longer exists" rule now applies to own and neutral objects, and to enemies only while visible.

## Implementation impact
- This replaces the Director's proposal, which was to move to the spot where the rally point was set.
- Each tick, or on a visibility change: if the RallyPoint is an enemy object that is not Visible to the owner, convert it to that object's last-seen Coordinates.
- The marker changes from a red object marker to a green ground marker (0087).
- **TBD:** rallying on a remembered (fogged) enemy structure. It depends on Q0017 point 2, which is still open.

## Verification
1. Rally a Barracks on a visible enemy unit. The next unit attacks it.
2. Let the enemy drive into the fog. The rally marker becomes a green ground marker where it was last seen. The next unit walks there.
3. Have the enemy unit die in the fog. The rally marker doesn't change.
