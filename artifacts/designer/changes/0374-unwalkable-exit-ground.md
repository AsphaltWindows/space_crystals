# 0374 — Exits over ground units can't walk on: the unit stays inside; unwalkable tiles shouldn't be Buildable

- **Date**: 2026-11-27T09:25:00-05:00
- **Answers**: Q0134
- **Docs**: `core/entities.md > Structure Type > Exit Doorway` (new point 6); `core/map.md > Tile`

## What changed and why
User: **(a)**, plus a guideline. A leaving unit may overlap its own building, but is never released onto ground it couldn't stand on. Such a building holds its finished unit indefinitely, like an occupied doorway. Placement does not prevent this.

The user's guideline for tile presets: **tiles that ground units can't walk on should generally not be Buildable**. It is advice for map makers, not an enforced rule; "any combination is valid" stands.

## Implementation impact
- None; this is as built in M5-Z1.
- The default presets already follow the guideline.
- If the map editor ever warns about preset combinations, Buildable without Traversible is one to flag. That is optional.

## Verification
—
