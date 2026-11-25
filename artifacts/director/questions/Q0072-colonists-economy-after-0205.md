# Q0072 — Q0055 re-measured after 0205: every figure within ±15%, but target 1 still fails at 2:20

- **Date**: 2026-11-24T22:20:13-05:00
- **Blocking**: no — proceeding on assumption: the oracle pins the new measured figures. No game number is changed beyond 0205, and the table and targets are left as they are until Q0055 is settled.
- **Docs**: `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`; `factions/colonists/units.md > Drone > Patch Occupancy` (the "18–22 working Drones" estimate); design change 0205; Q0055

## Question
0205 asked for the Colonists' economy to be re-measured with the faster Drone. Slice M3-K3 has done that, with the same line and method as Q0055.
- Every figure is now within ±15% of the table, and targets 2, 3 and 4 hold.
- Two points remain:
  1. **Target 1 fails narrowly, at 2:20 only.** The Colonists lead at 0:40 and 1:00, and grow faster up to 1:40. From 0:00 to 2:20, though, they grow by 15.9 against the GDO's 16.7, because they start higher: 6.93 against 6.67 at 0:00, which is +13.5% on the table's 6.1. The design says the two roughly match at the start.
  2. **The Patch Occupancy estimate is low.** `units.md` says the row saturates "at roughly 18–22 working Drones". 16–22 Drones give only 25.3–26.8, and the row saturates at 24.

How should Q0055 be settled on these figures?

## Context
**The measured line** (SC/s; the GDO figures are from M2's oracle):

| Time | Colonists | Table | Delta | Working Drones | Beacons | GDO |
|---|---|---|---|---|---|---|
| 0:00 | 6.93 | 6.1 | +13.5% | 4 | 0 | 6.67 |
| 0:40 | 11.43 | 12.1 | −5.5% | 8 | 0 | 10.00 |
| 1:00 | 13.33 | 15.0 | −11.1% | 9 | 1 | 12.86 |
| 1:20 | 16.00 | 16.5 | −3.0% | 11 | 1 | 15.24 |
| 1:40 | 18.67 | 19.2 | −2.8% | 13 | 1 | 17.62 |
| 2:20 | 22.86 | 23.1 | −1.1% | 16 | 2 | 23.33 |
| 2:40 | 25.14 | 25.7 | −2.2% | 18 | 2 | 26.19 |
| 3:00 | 26.29 | 27.9 | −5.8% | 19 | 3 | 26.67 |
| 4:00 | 27.81 | 31.9 | −12.8% | 24 | 4 | 26.67 |

**The saturation curve** on the reference row, in SC/s:

| Drones | 4 | 8 | 12 | 16 | 20 | 24 | 25+ |
|---|---|---|---|---|---|---|---|
| Dealt out over the row | 6.91 | 12.67 | 18.75 | 25.34 | 25.95 | 32.00 | 32.00 |
| Sent with one command | 6.91 | 12.74 | 19.23 | 25.41 | 25.86 | 29.60 | 32.00 |

**Targets as measured:**
1. **Fails at 2:20**, as above.
2. **Holds.** Each Drone added earns 1.33 SC/s early and 0.62 later.
3. **Holds.** The GDO earns 26.67 for 600 SC, against the Colonists' 22.86 for 900.
4. **Holds at 24 working Drones:** 27.81 on the line and 32.00 on the curve, against the GDO's 26.67.

**What changed since Q0055:**
- A trip from the middle patches outwards takes 73, 75, 83 and 95 ticks (was 81, 82, 92 and 104). A Drone now earns between 1.75 and 1.35 SC/s (was 1.58 to 1.23).
- 24 Drones dealt out saturate the row (was 27).
- A new Drone's first delivery comes barely sooner, after 139–154 ticks, since the walk from the portal is at top speed either way.
- The line passes the GDO's ceiling between 3:40 and 4:00 (was about 4:40).

## Options
- **A. Take the measured economy as the design.** No code change.
  - Write the measurements into the Colonists column.
  - Change the 18–22 estimate to about 24.
  - Restate target 1, for example "the Colonists lead from 0:40 to 1:40".
- **B. Bring the start into line with the GDO's.** Change a number, such as mining time or carry, so that the 0:00 figure is about 6.7. Every later figure would drop by about 4%: 4:00 would fall to about 26.7, −16% on the table.
- **C. Keep the table and the targets, and accept target 1's narrow miss.** The gap is 0.8 SC/s of growth at 2:20.
