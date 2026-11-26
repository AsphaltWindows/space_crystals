# 0405 — Gliders push apart slowly; separation radii fixed above their silhouettes

- **Date**: 2026-11-27T14:30:00-05:00
- **Answers**: Q0148 (part 3; parts 1–2 in 0403, 0404)
- **Docs**:
  - `systems/units.md > Air Collision`;
  - `factions/cults/units/sparrow.md`, `factions/gdo/units/dragonfly.md`, `factions/syndicate/units/specter.md` (Placeholder stats).

## What changed and why
User, asked whether gliders should push apart at all: "maybe they should be, but slowly".
- **Gliders keep soft separation, but much weaker** than other air units. The placeholder is a quarter of the HoverCraft strength. They drift apart gradually, mostly while circling, without noticeably bending their attack runs.
- **SeparationRadius is fixed to exceed the silhouette**, as the rule requires. The numbers are the designer's placeholders:

| Unit | Silhouette | SeparationRadius before | Now |
|---|---|---|---|
| Sparrow | 40 su | 32 | **48** |
| Dragonfly | 40 su | 32 | **48** |
| Specter | 56 su | 40 | **64** |

## Implementation impact
- A per-base separation strength, with gliders at ¼.
- Three radius values change.

## Verification
- Six Sparrows sent to one point drift apart over a few seconds while circling, and end about 48 su apart.
- Two Sparrows making passes on the same target stay on their lines.
