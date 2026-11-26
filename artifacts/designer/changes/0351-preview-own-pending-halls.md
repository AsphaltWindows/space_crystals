# 0351 — Hall placement preview also shows the would-be claims of your Halls under construction and ghosts

- **Date**: 2026-11-26T23:00:00-05:00
- **Answers**: — (extends 0350 / Q0107)
- **Docs**: `factions/cults/economy.md > RecruitmentHall > Recruitment Area display`

## What changed and why
User: "You should also show the claims made by your other in-progress and placed-ghost Recruitment Halls."
- In the preview, the tiles your **Halls under construction** and **placed ghosts** would claim are marked as taken by them.
- This is for display only: a Hall still claims nothing until it is complete (0341).

## Implementation impact
- The preview treats own pending Halls as claimants, in their order of placement.
- Real claiming is unchanged.

## Verification
- Place a ghost Hall, then preview a second one overlapping it: the overlap shows as taken.
