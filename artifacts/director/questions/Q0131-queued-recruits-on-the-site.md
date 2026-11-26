# Q0131 — 0362: does a Recruit holding the Build queued make the site wait?

- **Date**: 2026-11-26T00:52:09-05:00
- **Blocking**: no — proceeding on assumption (a): the site waits for it, as built in M5-J1 and marked there as a placeholder.
- **Docs**: `factions/cults/economy.md > Cults Building Mechanics > Group construction rules`

## Question
Design change 0362 says "Recruits of the same order standing on the footprint don't fail it. The site waits until they have stepped off." Does this include a Recruit that holds this Build **queued behind another order** (a Shift chain), and isn't working on it yet?

## Context
With Shift, a group of Recruits can hold several Builds in a row. For example, they build a Storage, then Shift-place a Hall right beside it.

When the first Recruit reaches the Hall's site, another Recruit of the same group may still be working on the Storage. Its work spot beside the Storage can lie on the Hall's footprint. That Recruit holds the Hall's order, but queued, behind the Storage.

## Options
- **(a) It counts as a Recruit of the same order (as built).** The Hall's site waits until the Recruit steps off. In practice that is when the Storage is finished and it walks over to build the Hall, so the site can wait a long time.
- **(b) Only a Recruit whose current order is this Build counts.** A Recruit holding it queued is "another unit", and fails the whole order, refunded once.

## Note (no answer needed)
0341's Verification says "Placing a Hall 3 tiles from the edge is refused". `factions/cults/structures/recruitment_hall.md > RecruitmentArea` says "The whole area must lie on the map", and the area is 10x10, centred on the 4x4 Hall. So a Hall 3 tiles from the edge has its area exactly on the map, and is accepted; a Hall 2 tiles from the edge is refused.

M5-J1 follows the doc and tests the refusal at 2 tiles. The Verification's example looks off by one.
