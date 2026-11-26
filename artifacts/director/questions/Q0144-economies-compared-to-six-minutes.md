# Q0144 — The economies compared to 6:00: the realistic opening never overtakes, and what targets now?

- **Date**: 2026-11-26T11:16:55-05:00
- **Blocking**: no. Nothing is tuned. The oracle pins every figure below as measured (`crates/sim/tests/economy_oracle/cults.rs`), and they are re-pinned once the targets change. Full write-up: `docs/economy_oracle.md > The Cults` (merged in `1801cf59`).
- **Docs**: `factions/cults/economy.md > Economy check`; design changes 0380 (3x2 Storage), 0381 (targets, longer timeline), 0382 (metrics), 0383 (all spending)

## Question
This is the comparison 0381–0383 asked for, with two results that need a decision:
1. **The realistic opening never overtakes ideal mining.** The target is "it should **overtake ideal mining (scenario 1) by about 4:00**". On the 8-patch field it can't: both lines hit the field's ceiling of about 21 SC/s by 2:48, and the second Hall's Recruits have nowhere to mine. What should the realistic scenario, or its target, be?
2. **Ideal mining by 2:48 is still about 0.9 of the others** (0.88 of the GDO, 0.86 of the Colonists), against "slightly ahead by 2:48". 0380 left open "whether ~0.9 by 2:48 is acceptable". Is it?

The user also said further targets would follow from the table; it is below.

## Context

### The table
All four lines are measured on their own reference fields, with these definitions:
- **Income/s:** delivered over the 160 frames before the checkpoint, divided by 10 (0382).
- **Total mined:** delivered since 0:00, excluding the starting bank. Banks start at 0 for the Cults and 100 for the GDO and the Colonists.
- **Spent:** everything paid, net of refunds (0383).

At every checkpoint, bank + spent = starting bank + total mined; the test asserts it.

| Line | Time | Income/s | Total mined | Bank | Spent |
|---|---|---|---|---|---|
| Cults, ideal | 1:30 | 13.0 | 918 | 918 | 0 |
| | 2:48 | 19.0 | 2362 | 2362 | 0 |
| | 4:00 | 23.6 | 3904 | 3904 | 0 |
| | 5:00 | 21.2 | 5278 | 5278 | 0 |
| | 6:00 | 22.2 | 6572 | 6572 | 0 |
| Cults, realistic | 1:30 | 9.6 | 676 | 376 | 300 |
| | 2:48 | 21.4 | 1848 | 1348 | 500 |
| | 4:00 | 21.6 | 3424 | 2924 | 500 |
| | 5:00 | 20.0 | 4628 | 4128 | 500 |
| | 6:00 | 21.6 | 5942 | 5442 | 500 |
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

Notes on reading it:
- **The GDO's 29.0 from 2:48 is a window artefact.** Its plates pay 10 every 48 frames, and from 2:48 the checkpoints are whole cycles apart, so every window catches the same phase: 29 payments. Its 8 plates average 26.67 SC/s.
- **Ten seconds of Cults income moves** by up to 2.4 SC/s between checkpoints with the loads in the window, even with all 20 Recruits at work.
- **What each line spends:**
  - The GDO builds its 6 plates (450) and a Generator whenever the bank covers one, 20 by 6:00 (2000).
  - The Colonists train a Drone every 10 s without a break, 36 by 6:00 (1800).
  - Cults ideal spends nothing: its Recruits are free, and scenario 1's Storages stand from the start ("one Storage per face, all Recruits mining").
  - Cults realistic spends 300 on its second Hall and 200 on two Storages.
- **Every Cults figure runs about 8% under `recruit.md > Gathering`'s "≈ 1.45 SC/s per Recruit".**
  - A Recruit stops at the first 4 su movement step that brings it within reach, so it walks 28 su each way instead of 24, and the trip is 24 ticks instead of 22.
  - It measures 1.33 SC/s, inside the ±15% tolerance.

### Why the realistic opening can't overtake
- **The build line runs as the doc describes.** It starts with one Storage and saves for a second Hall, ordered at 0:48.56 and complete at 1:45.56. Then it builds Storages on the other faces, complete at 1:52.44 and 1:59.94.
- **The field's ceiling.** The 8-patch field holds about 16 Recruits at full efficiency, which is 2 per tile of face, as 0380 estimated. That is 20.31 SC/s with 16 Recruits and 21.05 with 20: a seventh Recruit on a long face adds nothing, and an eighth costs.
- **Both lines reach that ceiling by about 2:48.** From then on they take in about the same, 20–24 SC/s. At 2:48 the realistic line's 21.4 is even above the ideal's 19.0.
- **The realistic line can't win back what it lost before its Storages stood:** 1848 SC by 2:48, against 2362. At 4:00 it is behind by income (21.6 against 23.6) and by total mined (3424 against 3904). At 6:00 it is still 630 SC behind.
- **The investment has no return on one field.** At 6:00, 20 of the realistic line's 40 Recruits are idle: the second Hall's Recruits have no seat.

### Also measured (0375 asked for it; no decision needed unless it's unintended)
This is 8 adjacent patches without free columns, with 16 / 20 Recruits:
- Storages under the row: 20.34 / 19.43 SC/s, against the reference field's 20.31 / 21.05.
- With the middle face's Storage north of the row instead: 14.25 / 13.79 SC/s. That Storage takes nothing, because the Recruits come from the south and mine from the south lane. There, under 0375 ("nearest to the Recruit itself"), the Storages south of the row are the nearer.

## Options

### 1. The realistic opening
- **(a) Give it a second field.** The second Hall's Recruits go to a second 8-patch field at a distance you set, and the line builds Storages there. The ideal line stays on one field (about 21 SC/s), so the investment can pay off and the 4:00 target becomes a real test.
  - **Implementation:** a new reference layout, and the line's orders for the expansion.
  - **To set:** the distance, and when the line moves Recruits there.
- **(b) Keep one field and change the target.**
  - On one field both lines meet the same ceiling by about 2:48. The target could be "level income by about 3:00", which the line meets now, instead of overtaking in total.
  - The total stays 500–650 SC behind.
- **(c) Keep one field and change the line,** for example dropping or delaying the second Hall until the field is full. The comparison then becomes "Storages first" against "Storages later".

### 2. Ideal mining by 2:48
- **(a) Accept about 0.9:** the target becomes "about 0.9 of the GDO and the Colonists by 2:48", and the oracle pins it as a pass.
- **(b) Tune the Cults up to "slightly ahead".**
  - The levers include the Recruit's mining time or load, the Hall's 168 s fill, and the 6 starting Recruits. You'd set which ones and by how much.
  - The oracle re-measures every figure, all of them, against the new numbers.

### 3. Further targets
If the user wants targets for 4:00–6:00, for example the Cults' income against the GDO's and the Colonists' at 6:00 (about 22 against 26.7 and 32), the oracle can pin them as passes or misses in the same way.
