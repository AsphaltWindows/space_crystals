# Ticket: Headquarters Stats Correction

## Current State
The Headquarters expansion has placeholder stats. The codebase defines `HQ_MAX_HP: 200.0` (in `src/game/types/structures.rs`). Size, cost, build time, and armor values were previously open questions and may be missing or using defaults.

## Desired State
Update Headquarters stats to match the finalized specification:

- **Size**: 2x2
- **Cost**: 200 Space Crystals
- **Build Time**: 400 frames (25 seconds)
- **HP**: 400
- **PointArmor**: 1
- **FullArmor**: 4

All stat fields must be present and correct in the structure definition.

## Justification
`features/syndicate_objects.md` — Headquarters expansion section now has full stats defined. The previous placeholder HP of 200 was a temporary value while the design was open. These stats are now finalized per the 2026-03-09 feature update.

## QA Steps
1. [auto] Verify `HQ_MAX_HP` (or equivalent constant) is set to 400.0 in the structure stats module
2. [auto] Verify Headquarters PointArmor is 1 and FullArmor is 4
3. [auto] Verify Headquarters cost is 200 Space Crystals in the production cost function
4. [auto] Verify Headquarters build time is 400 frames
5. [human] Spawn a Headquarters in-game and select it — verify the info panel shows HP 400/400

## Expected Experience
The Headquarters displays correct HP (400) and armor values when selected. Building a new Headquarters costs 200 Space Crystals and takes 25 seconds. These are numerical corrections — no behavioral change.
