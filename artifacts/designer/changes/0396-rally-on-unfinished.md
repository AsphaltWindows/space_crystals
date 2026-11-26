# 0396 — Rally points can be set on buildings still under construction

- **Date**: 2026-11-27T12:45:00-05:00
- **Answers**: —
- **Docs**: `systems/controls/selection.md > SelectionGroup` (Unfinished buildings form their own group)

## What changed and why
User: "in progress buildings probably should be able to have their rally point set".
- A building that will produce or release units can have its RallyPoint set while it is still under construction or transformation, for every faction. Examples: Cults buildings being built by Recruits, Syndicate expansions, Colonists portals warping in.
- Its unfinished panel adds **C: Set Rally Point**, next to Cancel. Right-clicking ground or an object sets it, as on the finished building.
- The rally point is kept when the building completes.

## Implementation impact
- Unfinished-building panels gain C, and the right-click sets the rally point.
- RallyPoint state must exist from the start of construction.

## Verification
- Start an Armory, select it while it's being built, and right-click ground: a rally marker appears.
- After completion, trained units go there.
