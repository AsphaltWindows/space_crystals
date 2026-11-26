# Q0146 — The economy oracle is wrapped up: its last figures, and closing the Economy check

- **Date**: 2026-11-26T13:22:49-05:00
- **Blocking**: no. The user has wrapped up automated economy testing (2026-11-26): "So much of it has to do with build orders and optimizations and that would be more effectively tested by me." The Director runs no more economy comparisons. The oracle's tests become on-demand measurements that tuning doesn't break, and the pairings tests keep checking that every faction's workers gather.
- **Docs**: `factions/cults/economy.md > Economy check`; the other factions' targets that the oracle checked (`factions/gdo/economy.md > Opening Economy Baseline`, and the Colonists' and the Syndicate's).

## Question
From now on the economy is tested by the user in playtests, not by the Director's scripted lines. Should the Economy check be closed: its targets kept as reference for playtests, no further measurements, and its "Still open" point and "Further targets may follow" left to playtests? The same goes for the other factions' oracle targets.

## Context
These are the last measurements (design change 0385, slice M5-S3), on the two-sided 8-patch field, nothing tuned.

| Line | Time | Income/s | Total mined | Bank | Spent |
|---|---|---|---|---|---|
| Cults, One Hall | 1:30 | 14.6 | 860 | 660 | 200 |
| | 2:48 | 21.4 | 2316 | 2016 | 300 |
| | 4:00 | 26.6 | 4162 | 3862 | 300 |
| | 5:00 | 26.4 | 5748 | 5448 | 300 |
| | 6:00 | 26.4 | 7338 | 7038 | 300 |
| Cults, Two Halls | 1:30 | 9.0 | 708 | 208 | 500 |
| | 2:48 | 27.2 | 2180 | 1480 | 700 |
| | 4:00 | 36.2 | 4676 | 3876 | 800 |
| | 5:00 | 39.0 | 6906 | 6106 | 800 |
| | 6:00 | 38.8 | 9150 | 8350 | 800 |
| GDO | 1:30 | 19.0 | 980 | 280 | 800 |
| | 2:48 | 29.0 | 2670 | 1320 | 1450 |
| | 4:00 | 29.0 | 4590 | 2740 | 1950 |
| | 5:00 | 29.0 | 6190 | 3940 | 2350 |
| | 6:00 | 29.0 | 7790 | 5440 | 2450 |
| Colonists | 1:30 | 16.0 | 1032 | 682 | 450 |
| | 2:48 | 24.8 | 2744 | 1994 | 850 |
| | 4:00 | 28.0 | 4624 | 3524 | 1200 |
| | 5:00 | 30.4 | 6312 | 4912 | 1500 |
| | 6:00 | 32.0 | 8224 | 6524 | 1800 |

Income is over the 160 frames before each checkpoint. Total mined leaves out the starting bank, and spent is net of refunds. Bank + spent = starting bank + total mined holds at every checkpoint.

- **Seats:** efficiency, against each Recruit's 1.33 SC/s alone, is 1.01 with 24 Recruits, 0.98 with 28, 0.92 with 32 and 0.66 with 40. Income peaks at 39.3 SC/s with 32 Recruits and falls with more, as head-on jams take the last tenth.
- **0385's Verification:** One Hall levels off at 20 Recruits mining, from 2:48. Two Halls reaches 32 mining, with 8 more that have no seat, and builds 5 Storages, 6 with the starting one.
- **Storages built:**
  - One Hall builds 3, done at 0:43, 1:46 and 2:47.
  - Two Halls finishes its second Hall at 1:21, then builds 5 Storages, done between 1:28 and 3:51.
- **The 4:00 target is met.** Two Halls leads One Hall on income from 2:34 and on total mined from 3:08. At 6:00 it is 1812 ahead, 9150 against 7338.
- **Out of tolerance:**
  - One Hall against the Colonists at 1:30 is 0.83.
  - At 2:48, One Hall is 0.87 of the GDO and 0.84 of the Colonists. The target is "slightly ahead by 2:48", and the doc's "Still open" asks whether ~0.9 is acceptable.
- **Readings where the design is silent,** built and marked in the code:
  - 1-tile gaps, with three Storage sites a side;
  - a Storage counts as "needed" when all the player's Recruits outnumber the room standing or ordered;
  - far-side Recruits wait for their own Storage;
  - Two Halls' 8 spare Recruits stay idle.

## Options
- **(a) Close it.** The targets stay in the docs as reference for playtests, and the Director measures nothing more. The oracle stays in the code as an on-demand tool whose pins aren't maintained. Any future economy targets come from playtests.
- **(b) Keep some targets as automatic checks**, saying which, for example the per-Recruit rate at a 1-tile gap. The Director keeps those as tests that must pass, and re-measures them when the numbers change.
