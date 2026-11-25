# Q0055 — The Colonists' measured economy misses the table: 1:00, 3:00 and 4:00 outside ±15%, targets 1 and 4

- **Date**: 2026-11-24T18:54:06-05:00
- **Blocking**: no — proceeding on assumption: no game number is changed (design change 0108). The economy oracle (`crates/sim/tests/economy_oracle/colonists.rs`, `curve.rs`) pins the measured figures and targets as they are, and fails when one moves, until the answer comes.
- **Docs**: `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)` (the table, targets 1–4, the reference check); `factions/colonists/units.md > Drone > Patch Occupancy` ("saturate at roughly 18–22 working Drones"); design changes 0107 (reference geometry) and 0108 (±15%, measured table back to the designer)

## Question
On the reference layout, the simulated Colonists earn less than the table from 0:40 on. Three figures are outside ±15%: 1:00 (−16.2%), 3:00 (−15.3%) and 4:00 (−23.6%); 0:40 is on the edge (−15.0%). Target 1 fails, and target 4 fails at the design's "~24 working Drones" (it holds only from 27 Drones on). Which should change:

1. the table and targets, to what the game does (no game number changes); or
2. game numbers (which?) so the game meets the table; or
3. the benchmark line, if the table assumes something the line below doesn't do?

## Context
**The line measured** (slice M3-K1, `sc_sim::testing::colonists`): reference geometry of 0107 (8 adjacent patches in a row, the Egress Node centred on the long side, 2 GU edge to edge), Colonists Starting Conditions (Egress Node, one Robo Portal, 4 Drones, 100 SC). The portal trains Drones without a break and is rallied on the middle patch; the starting Drones get one Gather on it, and the occupancy rule deals everyone out. A Drone deploys as a Beacon whenever the Beacon Capacity room left falls to 1: the 5th, 11th, 17th and 23rd Drones made ("every 6th"). Neither money nor capacity ever holds the portal up: Drone k comes out at exactly 10·k s. Every command goes through the gate; the books balance every tick.

**Income** at a time = crystals delivered over the 21 s centred on it (the GDO measured the same way from its M2 reference line, plates every 25 s). At 0:00: the four starting Drones' rate once under way (their first walk out excluded).

| Time | Colonists measured | Table | Delta | Working Drones | Beacons | GDO measured | Table |
|---|---|---|---|---|---|---|---|
| 0:00 | 6.17 | 6.1 | +1.1% | 4 | 0 | 6.67 | 6.7 |
| 0:40 | 10.29 | 12.1 | −15.0% | 8 | 0 | 10.00 | 10.0 |
| 1:00 | 12.57 | 15.0 | **−16.2%** | 9 | 1 | 12.86 | 13.3 |
| 1:20 | 14.86 | 16.5 | −10.0% | 11 | 1 | 15.24 | 16.7 |
| 1:40 | 17.52 | 19.2 | −8.7% | 13 | 1 | 17.62 | 20.0 |
| 2:20 | 20.57 | 23.1 | −10.9% | 16 | 2 | 23.33 | 23.3 |
| 2:40 | 22.10 | 25.7 | −14.0% | 18 | 2 | 26.19 | 26.7 |
| 3:00 | 23.62 | 27.9 | **−15.3%** | 19 | 3 | 26.67 | 26.7 |
| 4:00 | 24.38 | 31.9 | **−23.6%** | 24 | 4 | 26.67 | 26.7 |

The Drone and Beacon counts match the design's (18 Drones made and 2 Beacons at "2 Drones per patch", 900 SC; ~24 working at 4:00). Played on, the Colonists pass the GDO's 26.67 at about 4:40 (27.4 with 28 working Drones), not around 3:00, and reach 32.0 at 5:40 (33 working).

**Saturation curve** (the Drones placed beside the node and dealt out over the row, income from 50 s on, over 4:10):

| Drones | 4 | 8 | 12 | 16 | 20 | 24 | 25 | 26 | 27+ |
|---|---|---|---|---|---|---|---|---|---|
| SC/s | 6.34 | 11.65 | 17.22 | 23.10 | 24.45 | 26.24 | 26.91 | 30.72 | 32.00 |
| Design | 6.1 (+3.9%) | | | 23 (+0.4%) | | 32 (**−18.0%**) | | | |

So the reference check's points at 4 and 16 Drones hold, "~24 working Drones" for 32 does not (27 are needed), and units.md's "18–22" is further off.

**Targets as measured:**
- **1 fails.** The Colonists are barely ahead at 0:40 (10.29 vs 10.00), behind at 1:00 (12.57 vs 12.86), and grow less than the GDO by 2:20 (+14.4 vs +16.7).
- **2 holds.** 1.20 SC/s per working Drone added before 2:20, 0.48 after.
- **3 holds.** GDO 26.67 for 600 against the Colonists' 20.57 at 2:20 for 900 (23.10 for 16 Drones on the curve).
- **4 fails at 24 working Drones** (24.38 in the line, 26.24 on the curve, both under 26.67), **holds at full saturation** (32.0 from 27 Drones).

**Causes found:**
1. **A new Drone's first trip.** It first delivers about 150 ticks (9.4 s) after it comes out (the walk from the portal and its first load), against 81 ticks per trip once under way. With a Drone every 10 s, one Drone is always on its first trip: about 1.5 SC/s fewer than a model that counts each Drone at full rate from the moment it is made. The table looks like it does that: 0:40 has 8 Drones × 1.51 = 12.1, and 1:20 has 11 × 1.5 = 16.5.
2. **Longer trips at the ends of the row.** A trip takes 81–82 ticks on the four middle patches, 92 on the next two and 104 on the end two: 1.58 down to 1.23 SC/s per Drone. A patch saturates when its Drones' mining (32 ticks each) fills a trip: 3 Drones on the middle six patches, 4 on the end two, so 26 at best. The occupancy rule (one Drone per patch, the others re-check every 16 ticks) loses a little more, hence 27.
3. **Dealing Drones evenly doesn't help early.** Dealing each new Drone to the least-occupied patch gives 11.05 at 1:00 (−26%): an end-patch Drone earns less than a second Drone on a middle patch. From 2:20 it gives a little more (21.33, and 24.76 at 4:00).

**Not a cause:** money. The one-portal line banks 320 by 1:00 and 1272 by 2:20. A second Robo Portal (250, design change 0167) is affordable from about 0:55, but the targets compare "one Extraction Facility against one Robo Portal".

## Options
- **A. Take the measured economy as the design** (no game number changes): rewrite the Colonists column from the measurements (or a model with the first-trip delay and the per-patch trip times), set "fully saturated ≈ 32 per second (~27 working Drones)" and units.md's "18–22" to match, and restate targets 1 and 4. As measured, the Colonists don't pull ahead early and pass the GDO's ceiling at about 4:40. Code consequence: update the pinned constants and docs only.
- **B. Keep the table and change game numbers.** Each option has side effects to judge:
  - a shorter Drone production time (e.g. 8 s) helps target 1 and every figure, but needs Beacons sooner and costs more per minute;
  - a faster or nearer first trip (Drone speed or acceleration, or Drones released nearer the field) shrinks the 150-tick start-up;
  - a larger carry or shorter mining time raises every Drone's rate, including the 0:00 figure (6.17 now, against the GDO's 6.7: the "even start" of `factions/gdo/economy.md`) — a carry of 10 would put it at about 7.7, ahead of the GDO from the first second;
  - a 5th starting Drone helps target 1 directly.
  The geometry is fixed by 0107, so trip lengths can't move.
- **C. Change the benchmark line**, if the table assumes something else: for example, Drones counted from their first delivery, a second Robo Portal, or a different dealing of Drones over the row. The oracle would then measure that line instead.
- **D. Widen the tolerance** for the early, start-up-dominated figures (0:40 to 1:00) and for full saturation, keeping target 4 as "holds at full saturation" (true from 27 Drones on).
