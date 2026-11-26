# Q0135 — The Cults' economy check: 5 figures miss tolerance, and the realistic opening is far from level

- **Date**: 2026-11-26T02:51:19-05:00
- **Blocking**: no — M5-L's oracle pins the measured figures (`economy_oracle/cults.rs`, `OUT_OF_TOLERANCE`), and the build goes on with today's numbers.
- **Docs**: `factions/cults/economy.md > Economy check`; `factions/cults/units/recruit.md > Gathering`; `factions/cults/structures/storage.md`; design change 0346

## Question
The check was measured as 0346 asked (edge to edge, ±15%, an 8-patch field, one Storage per face). Do the misses below call for different Cults numbers, or different targets? And what should the realistic opening, the user's build line, be measured against?

## Scenario 1: ideal mining (9 of 14 figures pass)
| Figure | Measured | Target | |
|---|---|---|---|
| SC/s per Recruit at a 1-tile gap | 1.33 | ≈ 1.45 | −8%, pass |
| at 2 / at 3 tiles | 0.57 / 0.36 | ≈ 0.59 / ≈ 0.37 | pass |
| 4 Recruits on a face | full efficiency | full efficiency | pass |
| **5 Recruits on a face** | **0.82 of full** | full efficiency ("4–5") | **out** |
| SC/s per Recruit gained, 2-tile gap to 1 | 0.76 | ~0.86 | −11%, pass |
| **SC/s per face gained, 2-tile gap to 1** | **3.04** | ~+4 | **−24%, out** |
| **A Storage's payback** (100 SC over that gain) | **32.9 s** | ~25 s | **+32%, out** |
| The starting Hall from 6 Recruits to 20 | 168.0 s | 168 s | pass |
| The earliest second Hall, complete | 1:07.6 | ~0:55–1:15 | pass |
| Crystals by 1:30, against the GDO / the Colonists | 0.97 / 0.92 | level | pass |
| **Crystals by 2:48, against the GDO / the Colonists** | **0.90 / 0.88** | slightly ahead | **out** |

The causes, as measured:
- **The trip** at a 1-tile gap is 24 ticks, not the 22 frames the figures assume. A Recruit stops at the first 4 su movement step that brings it within reach, so it walks 28 su each way instead of 24. That accounts for all of the −8% on 1.45. The other factions' workers move the same way, so their reference lines carry the same rounding.
- **A face has two lanes,** so a fifth Recruit waits for one: it adds 0.16 SC/s at a 1-tile gap (at a 2-tile gap, with longer trips, five reach 0.95 of full).
- **The gain per face** is the per-Recruit gain over 4 Recruits at the measured trips: 0.76 × 4 = 3.04. The "~+4" assumed 0.86 × 4.65.
- **Crystals by time:** the Cults are level until 1:30, then fall behind. By 0:30, 1:00, 1:30, 2:00, 2:30 and 2:48 they have 202, 538, 952, 1456, 2026 and 2408 SC. The GDO has 210, 540, 980, 1530, 2200 and 2670.
- **The earliest second Hall** now depends on how many Recruits build it (group construction, M5-E): complete at 2:13.5 with 1 builder, 1:28.9 with 2, 1:07.6 with 4, and 1:01.5 with 6.

## Scenario 2: the realistic opening (no target yet)
The user's line, as 0346 describes it:
1. one Storage;
2. new Recruits spread face by face to farther patches, all walking to the starting Storage;
3. the second Hall is ordered at 0:57.6 and built by the 4 far Recruits, complete at 1:33.9;
4. then a Storage per face, complete at 1:41.5, 1:52.3 and 2:14.6.

| By | GDO | Colonists | Cults, ideal | Cults, realistic | Realistic against the GDO / the Colonists |
|---|---|---|---|---|---|
| 1:30 | 980 | 1032 | 952 | 504 | **0.51 / 0.49** |
| 2:48 | 2670 | 2744 | 2408 | 1668 | **0.62 / 0.61** |

- **The cause:** until the extra Storages stand, every Recruit off the starting face walks 90–324 su to the one Storage, and they crowd it.
- **Other dealing orders did worse:** 438–504 SC by 1:30 and 1266–1668 by 2:48.

**So the answer to the user's question is no: the Cults don't stay level when they expand early this way.** They have about half the other factions' crystals by 1:30, and 0.6 by 2:48. The second Hall's extra production would pay off only later than 2:48.

## Options
- **(a) Accept the figures:** the doc's targets become the measured ones (for example, 4 Recruits per face, a payback of about 33 s, about 0.9 by 2:48), and the realistic opening is an accepted cost of expanding early.
- **(b) Tune the Cults' numbers** toward the targets, for example the Recruit's load, mining time or speed, the Storage's cost or build time, or the starting Storages. The oracle then re-measures every figure, and pins any that still miss.
- **(c) Give the realistic opening a target,** if the Cults should stay closer to level when they expand early, and say which numbers may move to reach it.
- **(d) Something else.**
