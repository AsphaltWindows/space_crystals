# 0332 — Cults production: first in, first trained; trained infantry keep the Recruit's damage

- **Date**: 2026-11-26T19:20:00-05:00
- **Answers**: Q0105 (points 1–2), Q0114 (point 2), Q0121 (points 1 and 3)
- **Docs**: `factions/cults/structures/armory.md > Training`; `factions/cults/augmentation.md > Augmentation Orders`

## What changed and why
User:
- **Infantry trained from a damaged Recruit keep its damage.** This covers Militants, Gunners and Disciples. Vehicles and aircraft come out at full HP (0330).
- **The unit that entered the building first is trained first.** Every pool building reserves the matching unit(s) in order of entry: Armory, Cathedral, Chop Shop (2 Recruits), Airstrip and the Altar.

## Implementation impact
- This matches the interim "inside the longest".
- Q0105 points 3–5 are still open.

## Verification
- Send in a hurt Recruit, then a healthy one, and queue a Militant: the Militant comes out with the hurt Recruit's missing HP.
