# 0380 — Cults Storage back to 3x2

- **Date**: 2026-11-27T10:25:00-05:00
- **Answers**: Q0135 (in part: the user's response to the ideal-mining misses; the 2:48 target and the realistic opening are still being discussed)
- **Docs**:
  - `factions/cults/structures/storage.md` (Size, SymmetryType, FullArmor, rationale);
  - `factions/cults/economy.md > Economy check` (per-face target);
  - `factions/cults/units/recruit.md > Gathering`.

## What changed and why
User, on Q0135's measurements: **the Storage becomes 3x2** again (from 2x2). It has two 3-tile long faces and two 2-tile short faces, so it serves more Recruits per face, and an efficient field needs fewer Storages.

| | Before | Now |
|---|---|---|
| Size | 2x2 | **3x2** |
| SymmetryType | AAAA | **ABAB** |
| FullArmor | 4 | **6** (building armor rule) |
| MaxHP, PointArmor, cost, build time | 250, 1, 100 SC, 320 f | unchanged |

**Economy check per-face target:** "full efficiency at about 2 Recruits per tile of face length", i.e. ~6 on a long face and ~4 on a short one. This is the designer's estimate from the lane geometry; please re-measure.

## Implementation impact
- Storage footprint, rotation (2 orientations) and armor change.
- The oracle's reference field and per-face figures need re-measuring:
  - Recruits per face;
  - gain per face;
  - payback;
  - crystals by 1:30 and 2:48;
  - the realistic opening.
- Still open with the user, so don't tune yet:
  - whether ~0.9 by 2:48 is acceptable;
  - what the realistic opening should reach.

## Verification
- A Storage ghost is 3x2 and rotates between two orientations.
- Recruits mining a patch facing a long side use three lanes' worth of room.
