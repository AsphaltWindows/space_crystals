# The economy oracle: measured results

**On demand only, its pins no longer maintained (design change 0401).** The economy checks are closed: `factions/cults/economy.md > Economy check`, `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)` and the Syndicate's in `factions/syndicate/units/agent.md`. Their targets stay in the design as reference for playtests. The oracle stays as a tool to run on demand: every one of its tests is ignored, so the default `cargo test` runs none of them, and its pins and this page are no longer updated, so a test may fail and a figure here may be out of date. The pairings tests (`crates/sim/tests/pairings/`: every faction's workers gather) still run.

The economy oracle (`crates/sim/tests/economy_oracle/`, see `docs/architecture/sim-scenarios-and-tests-setup.md > Sim scenarios and tests`) plays openings from a real start, the way a player would, through the command gate, and checks them against `factions/gdo/economy.md`, the Syndicate's economy check (`factions/syndicate/units/agent.md > Gathering > Space Crystals`) and the Cults' (`factions/cults/economy.md > Economy check`). This page records what it measured, last while its pins were maintained. Its tests check these numbers against the design, each within the tolerance named where it is checked.

To run it and print every line:

```sh
cargo test --release -p sc_sim --test sim economy_oracle -- --ignored --nocapture
```

That runs its checks and its measurements: the Colonists' lines to 6:00, the saturation curve, the Syndicate's whole table, and the Cults' measurements.

## The GDO (slice M2-J1)

The GDO player starts on `maps/dev_skirmish.json` with the reference base (`factions/gdo/economy.md > Opening Economy Baseline`).

| Check | Measured | Design |
|---|---|---|
| Fully plated | 2:32 | about 2:30 |
| Bank at 1:30 / 2:00 | 280 / 555 | about 300 / 600 |
| Generator-first opening | the plate comes 12.0 s later; 9.4% less mined by 3:00 | about 11 s; about 8% |
| Clean Barracks | starts at 48.1 s, ready at 58.0 s | about 48 / 58 with lumps |
| All-in Barracks | starts at 15.1 s at power −21; production at 58.8% | about 15 s; about 59% |

Lumps of 10 shift the design's continuous-income timings by about 2 s, which the design allows for. Costs, build frames, the power ratio, lumps, cycles and refunds are checked exactly. The reference line's income at the times of `factions/gdo/economy.md > Economy Targets` is checked against its plates landing every 25 s, each mining 10/3 SC/s from then. That model was the table's GDO column until design change 0236; the column is now the line measured as the Colonists' oracle measures it (below).

## The Colonists (slices M3-K1, M3-K3 and M3-K4)

The Colonists play the reference geometry of design change 0107: 8 adjacent patches in a row, with the Egress Node centred on the row's long side, 2 GU from it edge to edge. They start with the Colonists' Starting Conditions: one Robo Portal, four Drones and 100 SC.

- The portal trains Drones without a break and is rallied on the middle patch.
- A Drone deploys as a Beacon whenever the Beacon Capacity room falls to 1. That makes the 5th, 11th, 17th and 23rd Drones made deploy.
- Drone k comes out at exactly 10·k s.

Income at a time is the crystals delivered over the 21 s centred on that time. At 0:00 it is the starting Drones' rate once under way. The GDO figures come from M2's reference line (plates first), measured over the same window. M2's own oracle averages over the 9 s ending at the time; on that measure the GDO's figures are 6.67, 10.00, 13.33, 13.33, 16.67, 23.33, 25.56, 26.67 and 26.67.

### The measured economy is the design (design change 0236, slice M3-K4)

Design change 0236 answered Q0055 and Q0072: the measured economy is the design, and the Drone keeps the numbers of design change 0205. The table of `factions/gdo/economy.md > Economy Targets` is now the measurement below, slice M3-K3's, rounded. Its Colonists column is the reference line, and its GDO column is M2's reference line measured over the same window. No game number changed.

The oracle checks every figure of both columns against the table, within the ±15% of design change 0108. The table only rounds the measurement, so every figure is within 0.5% of it.

| Time | Colonists | Table | Delta | Working Drones | Beacons | GDO | Table | Delta |
|---|---|---|---|---|---|---|---|---|
| 0:00 | 6.93 | 6.9 | +0.4% | 4 | 0 | 6.67 | 6.7 | −0.5% |
| 0:40 | 11.43 | 11.4 | +0.3% | 8 | 0 | 10.00 | 10.0 | +0.0% |
| 1:00 | 13.33 | 13.3 | +0.3% | 9 | 1 | 12.86 | 12.9 | −0.3% |
| 1:20 | 16.00 | 16.0 | +0.0% | 11 | 1 | 15.24 | 15.2 | +0.3% |
| 1:40 | 18.67 | 18.7 | −0.2% | 13 | 1 | 17.62 | 17.6 | +0.1% |
| 2:20 | 22.86 | 22.9 | −0.2% | 16 | 2 | 23.33 | 23.3 | +0.1% |
| 2:40 | 25.14 | 25.1 | +0.2% | 18 | 2 | 26.19 | 26.2 | −0.0% |
| 3:00 | 26.29 | 26.3 | −0.1% | 19 | 3 | 26.67 | 26.7 | −0.1% |
| 4:00 | 27.81 | 27.8 | +0.0% | 24 | 4 | 26.67 | 26.7 | −0.1% |

Played on, the line passes the GDO's 26.67 between 3:40 (25.90 SC/s, 23 working Drones) and 4:00 (27.81, 24 working Drones), as the design says. From 5:20 on (31 working Drones) it measures 31.62: 83 loads of 8 in the window, one short of the 84 that eight patches mined without a break would give.

**The targets** all hold:

- **Target 1, as 0236 restated it:** "Early on, the Colonists lead: from the start to about 1:40". The Colonists are ahead at every time of the table from 0:00 to 1:40, by 0.26, 1.43, 0.48, 0.76 and 1.05 SC/s.
- **Target 2:** "After that, the Colonists grow more slowly". Since 0236, "after that" follows the restated target 1, so the oracle checks both readings: after 1:40, and after 2 Drones per patch (2:20) as it read before. Each working Drone added brings 1.30 SC/s before 1:40 and 0.83 after; 1.33 before 2:20 and 0.62 after.
- **Target 3:** the GDO earns 26.67 for 600; the Colonists earn 22.86 at 2:20 for 900.
- **Target 4:** at 24 working Drones the Colonists earn 27.81 on the line and 32.00 on the curve, against the GDO's 26.67.

**The saturation curve.** For each count, the Drones are placed beside the node and dealt out over the row, or sent to the middle patch with one command. Income is measured from 50 s on, over 4:10:

| Drones | 4 | 8 | 12 | 16 | 18 | 20 | 22 | 24 | 25+ |
|---|---|---|---|---|---|---|---|---|---|
| Dealt out, SC/s | 6.91 | 12.67 | 18.75 | 25.34 | 25.34 | 25.95 | 26.78 | 32.00 | 32.00 |
| One command, SC/s | 6.91 | 12.74 | 19.23 | 25.41 | 25.38 | 25.86 | 26.37 | 29.60 | 32.00 |

Since 0236 the design's curve is this measurement too (`factions/colonists/units/drone.md > Patch Occupancy`). The row saturates at about 24 working Drones, 3 per patch, for 32 SC/s. 2 per patch (16 Drones, about 25 SC/s) is the efficient point. Drones 17–20 add almost nothing (about 26 at 20), a plateau the design accepts for now. The default test measures over 100 s:

| Drones | Measured | Design | Delta |
|---|---|---|---|
| 4 | 6.88 | 6.9 (the table's 0:00) | −0.3% |
| 16 | 25.20 | about 25 | +0.8% |
| 20 | 26.00 | about 26 | +0.0% |
| 24 | 32.00 | 32, the row mined without a break | +0.0% |

Dealt out, the Drones fill the middle patches first. The 17th to 22nd are third Drones on the middle six patches, which two Drones already keep busy most of the time: a trip there takes 73 to 83 ticks, and two Drones mine for 64. So a third Drone mostly waits, or is sent on from patch to patch and loses time. The 23rd and 24th are the end patches' third Drones, and three Drones' 96 ticks of mining just cover the end patches' 95-tick trips, so the whole row is mined without a break. Sent with one command, the row takes 25 Drones.

### Design change 0205 (slice M3-K3)

Design change 0205 raised the Drone's thrust and drag by half: ForwardAcceleration 0.9, OmniDirectionalAcceleration 0.3 and DragRatio 0.15, where they were 0.6, 0.2 and 0.1. Its top speed stays 8 su a frame. It now reaches 68% of it after 7 frames and 96% after 20, and stops from it in 26 su (42 before). No other number changed, and the table and the targets stayed as they were: 0205 was a step toward the question Q0055. Slice M3-K3 measured the figures above, which went to the Designer as Q0072. Design change 0236 answered both questions.

Against the table as it was then, the design's estimate, every figure was within the ±15% of design change 0108:

| Time | Colonists | Table then | Delta | Working Drones | Beacons | GDO | Table then |
|---|---|---|---|---|---|---|---|
| 0:00 | 6.93 | 6.1 | +13.5% | 4 | 0 | 6.67 | 6.7 |
| 0:40 | 11.43 | 12.1 | −5.5% | 8 | 0 | 10.00 | 10.0 |
| 1:00 | 13.33 | 15.0 | −11.1% | 9 | 1 | 12.86 | 13.3 |
| 1:20 | 16.00 | 16.5 | −3.0% | 11 | 1 | 15.24 | 16.7 |
| 1:40 | 18.67 | 19.2 | −2.8% | 13 | 1 | 17.62 | 20.0 |
| 2:20 | 22.86 | 23.1 | −1.1% | 16 | 2 | 23.33 | 23.3 |
| 2:40 | 25.14 | 25.7 | −2.2% | 18 | 2 | 26.19 | 26.7 |
| 3:00 | 26.29 | 27.9 | −5.8% | 19 | 3 | 26.67 | 26.7 |
| 4:00 | 27.81 | 31.9 | −12.8% | 24 | 4 | 26.67 | 26.7 |

The GDO's column then was a model: its plates landing every 25 s, each mining 10/3 SC/s from then. The GDO oracle still checks its line against that model.

The saturation curve against the design's estimates then:

| Drones | Measured | Design then | Delta |
|---|---|---|---|
| 4 | 6.91 | 6.1 | +13.3% |
| 16 | 25.34 | 23 | +10.2% |
| 24 | 32.00 | 32 | +0.0% |

Full saturation took 24 Drones dealt out, as the design estimated (`factions/gdo/economy.md > Reference check`: about 24), and 25 sent with one command. The design's other estimate, 18–22 (`factions/colonists/units/drone.md > Patch Occupancy`), was low.

**The targets then:**

- **Target 1 failed, at 2 Drones per patch only.** It read "Early on, the Colonists' economy grows faster — until they reach about 2 Drones per patch". The Colonists were ahead at 0:40 (11.43 against 10.00) and at 1:00 (13.33 against 12.86). Their income grew faster than the GDO's up to 1:40 (+11.7 against +11.0). By 2:20 it had grown less (+15.9 against +16.7), because it starts higher: 6.93 against the GDO's 6.67.
- **Target 2 held.** Before 2:20 each working Drone added brought 1.33 SC/s; after 2:20, 0.62.
- **Target 3 held.** The GDO earned 26.67 for 600; the Colonists earned 22.86 at 2:20 for 900.
- **Target 4 held.** At 24 working Drones the Colonists earned 27.81 on the line and 32.00 on the curve, against the GDO's 26.67.

**What changed, and why:**

- **Shorter trips.** A trip takes 73 ticks on the middle patches, then 75, 83 and 95 ticks going out to the ends of the row. Before 0205 it took 81, 82, 92 and 104 ticks. A Drone brings 1.75 SC/s on a middle patch and 1.35 on an end patch, where it brought 1.58 and 1.23.
- **Saturation at 24 Drones.** Three Drones mine for 96 ticks between them, which now covers even an end patch's 95-tick trip. Before 0205 the end patches' 104-tick trips needed a fourth Drone, so the row took 27.
- **The first trip shortened only a little.** A new Drone's first delivery comes 139 to 154 ticks after it leaves the portal, or later when it is sent on from patch to patch. Before 0205 it came after about 150 ticks. Most of that time is the walk out from the portal, at top speed either way: only the Drone's starts and stops got shorter. So about one Drone is still always on its first trip.
- **A higher start.** The four starting Drones first deliver after 76, 81, 82 and 103 ticks (84, 90, 90 and 113 before 0205). Once under way they earn 6.93 SC/s, up from 6.17. That was 13.5% above the table's 6.1 then, within the tolerance but above the GDO's 6.67, which the design says it roughly matches. This higher start is what made target 1 fail, and why 0236 restated it.

### Before design change 0205 (slice M3-K1)

These are the figures that went to the Designer as Q0055, with the Drone's earlier numbers: ForwardAcceleration 0.6, OmniDirectionalAcceleration 0.2 and DragRatio 0.1. Figures outside the tolerance are in bold.

| Time | Colonists | Table then | Delta | Working Drones | Beacons | GDO | Table then |
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

Played on, the line passed the GDO's 26.67 at about 4:40 (27.4 SC/s with 28 working Drones). It reached 32.0 at 5:40 (33 working Drones).

**The saturation curve.** For each count, the Drones are placed beside the node and dealt out over the row. Income is measured from 50 s on, over 4:10:

| Drones | 4 | 8 | 12 | 16 | 18 | 20 | 22 | 24 | 25 | 26 | 27+ |
|---|---|---|---|---|---|---|---|---|---|---|---|
| SC/s | 6.34 | 11.65 | 17.22 | 23.10 | 23.94 | 24.45 | 25.57 | 26.24 | 26.91 | 30.72 | 32.00 |

Against the design's estimates then:

| Drones | Measured | Design then | Delta |
|---|---|---|---|
| 4 | 6.34 | 6.1 | +3.9% |
| 16 | 23.10 | 23 | +0.4% |
| 24 | 26.24 | 32 | **−18.0%** |

Full saturation (8 patches at 4 SC/s) took 27 Drones. The design estimated about 24 (`factions/gdo/economy.md > Reference check`) or 18–22 (`factions/colonists/units/drone.md > Patch Occupancy`).

**The targets then:**

- **Target 1 failed.** The Colonists were barely ahead at 0:40, behind at 1:00, and grew less than the GDO by 2 Drones per patch (+14.4 against +16.7).
- **Target 2 held.** Before 2:20 each working Drone added brought 1.20 SC/s; after 2:20, 0.48.
- **Target 3 held.** The GDO earned 26.67 for 600; the Colonists earned 20.57 at 2:20 for 900.
- **Target 4 failed at 24 working Drones and held at full saturation.** At 24 Drones the Colonists earned 24.38 on the line and 26.24 on the curve. At 27 Drones or more they earned 32.0 against 26.67.

**The causes:**

- **A new Drone's first delivery.** It came about 150 ticks after the Drone left the portal, against 81 ticks per trip once under way. So about one Drone was always on its first trip, roughly 1.5 SC/s below a table that counted each Drone from the moment it was made.
- **Longer trips at the row's ends.** Trips took 81–82 ticks on the middle patches, 92 on the next two out, and 104 on the end patches.
- **Saturation needed more Drones.** The end patches needed four Drones each to stay busy.

Following design change 0108, no game number was changed. The figures outside the tolerance and the failing targets went to the Designer as Q0055 (slice M3-K1), and the oracle pinned them as measured until an answer came. Design change 0205 then changed the Drone's numbers, and design change 0236 answered the question: the figures measured with 0205 are the design (above).

## The Syndicate (slice M4-L)

The Syndicate's economy check (`factions/syndicate/units/agent.md > Gathering > Space Crystals > Syndicate economy check`, design changes 0259 and 0294) is measured on its reference layout: one Tunnel with its crystal hatch facing a single patch, the distance counted edge to edge from the patch to the hatch square (`sc_sim::testing::syndicate::reference_field`). The Agent mines for 64 frames (design change 0294; 80 before) and unloads for 64, and Agents keep their collision.

- **A trip** is one Agent's gap between deliveries once it has settled. **Per Agent** is 50 SC per trip, **payback** is two trips (100 SC), and **Agents to saturate** is the trip over 4 s.
- **An income** is the average over minutes 5 to 10 of a ten-minute game. The Agents start in a row south-east of the Tunnel and are sent to the patch with one command. With collision they take minutes to settle into their lanes.

The oracle checks every figure from 3 GU at ±15% (0294: "the table from 3 GU outward"). Rows 0–2 are theoretical and only recorded here. Each cell is measured / design (delta). The incomes of 3 and 4 Agents are as measured with design change 0315 built (slice M4-W2: an Agent's leg back to the hatch presses on); before it they were 6.00, 11.00, 10.67, 11.17, 10.33 and 9.67 with 3 Agents at 1 to 6 GU, and 6.00, 11.33, 11.83, 12.17, 12.50 and 12.50 with 4.

| GU | Trip (s) | Per Agent | Payback (s) | To saturate | 2 Agents | 3 Agents | 4 Agents | |
|---|---|---|---|---|---|---|---|---|
| 0 | 8.00 / 8.0 (+0.0%) | 6.25 / 6.25 (+0.0%) | 16.0 / 16 (+0.0%) | 2.00 / 2.00 (+0.0%) | 6.17 / 12.5 (−50.7%) | 6.17 / 12.5 (−50.7%) | 12.50 / 12.5 (+0.0%) | recorded |
| 1 | 8.38 / 9.3 (−9.9%) | 5.97 / 5.36 (+11.4%) | 16.8 / 19 (−11.8%) | 2.09 / 2.33 (−10.1%) | 12.00 / 10.7 (+12.1%) | 12.00 / 12.5 (−4.0%) | 10.50 / 12.5 (−16.0%) | recorded |
| 2 | 9.75 / 10.7 (−8.9%) | 5.13 / 4.69 (+9.3%) | 19.5 / 21 (−7.1%) | 2.44 / 2.67 (−8.7%) | 10.33 / 9.4 (+9.9%) | 12.17 / 12.5 (−2.7%) | 12.50 / 12.5 (+0.0%) | recorded |
| 3 | 11.00 / 12.0 (−8.3%) | 4.55 / 4.17 (+9.0%) | 22.0 / 24 (−8.3%) | 2.75 / 3.00 (−8.3%) | 9.00 / 8.3 (+8.4%) | 12.17 / 12.5 (−2.7%) | 12.50 / 12.5 (+0.0%) | pass |
| 4 | 12.38 / 13.3 (−7.0%) | 4.04 / 3.75 (+7.7%) | 24.8 / 27 (−8.3%) | 3.09 / 3.33 (−7.1%) | 8.17 / 7.5 (+8.9%) | 11.33 / 11.3 (+0.3%) | 12.00 / 12.5 (−4.0%) | pass |
| 5 | 13.75 / 14.7 (−6.5%) | 3.64 / 3.41 (+6.6%) | 27.5 / 29 (−5.2%) | 3.44 / 3.67 (−6.3%) | 7.17 / 6.8 (+5.4%) | 10.50 / 10.2 (+2.9%) | 12.50 / 12.5 (+0.0%) | pass |
| 6 | 15.00 / 16.0 (−6.2%) | 3.33 / 3.12 (+6.8%) | 30.0 / 32 (−6.2%) | 3.75 / 4.00 (−6.2%) | 6.67 / 6.2 (+7.5%) | 9.83 / 9.4 (+4.6%) | 12.50 / 12.5 (+0.0%) | pass |

0294's Verification, "At 3 GU, 3 Agents reach about 12.5 SC/s; at 4 GU about 11.3; at 6 GU, 4 Agents saturate": 12.17 (−2.7%), 11.33 and 12.50 (re-measured by slice M5-S2a; the table had been left behind by earlier slices, and design change 0384 moves none of it).

- **Trips** are shorter than the table's, which walks the whole distance both ways. The Agent's own 36 su and the two 8 su reaches come off. So the Agents' incomes run above the table, and the saturating counts below it.
- **3 Agents at 3 GU** fall short of the hatch's ceiling although their trips (176 ticks) are shorter than three drop-offs (192). Near the hatch they get in each other's way, and the hatch stands free at times while a loaded Agent finds its way past the others. (Before design change 0315 that cost more, 10.67: an Agent whose leg ended behind another stood for 2 s before it tried again.)
- **Rows 0 and 1** jam. With the patch that close, Agents can't pass each other between it and the hatch, and an Agent behind another in its lane is blocked for good (`tests/agent_gathering/trips.rs`). Income falls to one Agent's: 2 and 3 Agents at 0 GU, and 4 at 1 GU (3 at 1 GU too before design change 0315; 9.83 with it). Design change 0294 accepts this: "Very close to the hatch (0–2 grid units) they can get in each other's way and income drops".

**The ceiling** held exactly in every game: no two deliveries at a Tunnel came less than 64 ticks apart. Four Agents at 5 and 6 GU reach exactly 12.50.

**Stacked Tunnels** at 3 GU (`stacked_field`): two Tunnels on opposite sides of the field, each hatch 3 GU from its own patch, the two patches a free column apart, four Agents for each Tunnel. With the patches side by side, the ends of their north and south faces lie as near either hatch, and some Agents delivered at the other Tunnel. Since design change 0384 an Agent takes the Tunnel nearest to it at every return: a few Agents go on at the other side's patch (on its first way back an Agent may step round another, `docs/movement.md > 6.3`), and deliver at that side's Tunnel, so over the game five Agents deliver at one Tunnel and three at the other, every one throughout. The Tunnels take in 12.50 and 12.17, 24.67 SC/s in all (12.50 and 12.33, 24.83, with each Agent's Tunnel kept for its Gather): −1.3% against twice the ceiling, and −1.3% against twice one Tunnel's 12.50 with four Agents at 3 GU. Pass.

**The opening** at 3 GU (`opening_field`) is played from the Syndicate's Starting Conditions on the reference layout: the Tunnel, the Hideout in its area, one Agent, 100 SC and 1 Supply. The Agent is sent to the patch, the Hideout is rallied on it, and the Hideout trains an Agent whenever it is free and the bank covers it. The second Agent starts in tick 1 ("the second Agent can start immediately"). The third is paid for at 0:24.6 and comes out to gather at 0:39.56, against 0:34–0:46: pass. The check reads "saturated (3 Agents, 12.5 Space Crystals per second) around 0:40" as the third Agent at work. Three Agents at 3 GU take in 11.83 at best, so the Tunnel never takes in 12.5 without a break with three; from 1:00 to 1:30 it takes in 11.67 SC/s (10.00 before design change 0315).

**The lowest one-base saturation**: one Tunnel takes in at most 12.50 SC/s, against the GDO's 26.67 fully plated and the Colonists' 32 (31.62 measured, above). Pass.

## The Cults (slices M5-L, M5-S2a and M5-S3)

The Cults' economy check (`factions/cults/economy.md > Economy check`, design changes 0346 and 0385) is measured edge to edge, patch to Storage, in whole tiles, at ±15%. Since slice M5-S3 (design change 0385, answering Q0144 in part) it is played on 8 patches in a row, mined from both long sides, in two scenarios: One Hall and Two Halls. These replace ideal mining and the realistic opening. The figures from before are at the end of this section. Where the design is silent, the field and the lines take the most literal reading. The code marks each one as slice M5-S3's choice. The field's are under "The reference field", and the lines' under "How the lines play".

### The reference field

`sc_sim::testing::cults::line_field`: the 8 patches on tiles (16, 12) to (23, 12) of a flat 32 × 28 map. The near side is the south, where the Cults start. The far side is the north.

- **Storage sites** (`SITES`): three a side, side by side along the row, each a 1-tile gap from it, and none past the row's ends ("Storages stand along both long sides only: none at the ends"). W, unturned, faces patches 1–3 with its long side, and M faces 4–6. E is turned a quarter and faces 7–8 with its short side. Together they cover the row's 8 tiles exactly.
- **Room**: two Recruits a patch on each side, the design's "about 2 Recruits per tile of face length". That is 6 at W and M and 4 at E, 16 a side and 32 in all.
- **The start** (`factions/cults/economy.md > Starting Conditions`): the starting Storage on near M, the Recruitment Hall centred under the row two free rows south of the near Storages, turned so that its exit faces the row, six Recruits of the Hall's, and 0 SC.
- **The second Hall's site** (`SECOND_HALL`): west of the row across four free columns, by the patches farthest from the starting Storage, turned so that its exit faces east. Its whole Recruitment Area is on the map and its own, so its Effectiveness is 100%.

**The seats the field holds** (`full_line`): all six Storages standing and no Hall, the Recruits spread evenly over each side's patches. The table gives their income over 0:20 to 5:00, and their efficiency against as many Recruits' rate alone at a 1-tile gap (1.33 SC/s):

| Recruits a side | 8 | 12 | 14 | 15 | 16 | 17 | 18 | 20 | 24 |
|---|---|---|---|---|---|---|---|---|---|
| SC/s | 21.81 | 32.26 | 36.59 | 36.31 | 39.31 | 37.88 | 35.19 | 35.25 | 27.59 |
| Efficiency | 1.02 | 1.01 | 0.98 | 0.91 | 0.92 | 0.84 | 0.73 | 0.66 | 0.43 |

- **About 32 seats.** The field takes in the most with 16 a side, 32 Recruits, at 0.92 of their rate alone: the design's "about 32 Recruits at full efficiency", within the tolerance. Up to 28 it is at full efficiency. Past 32 the income falls.
- **Jams decide the last tenth.** With 14 Recruits on the near side and none on the far, the near side takes in 16.39 SC/s (0.88). With 14 on the far side as well, it takes in 18.51 (0.99). In the first run and in the 16-a-side one, the same five near-side Recruits on patches 1 to 3 stand head-on and lose a fifth to a third of their loads. In the second run nothing holds them up. Where Recruits jam depends on how they arrive, not on the layout. Design change 0320 accepts this: there is no give-way rule.

### How the lines play

`sc_sim::testing::cults::Cults` plays both scenarios through the gate (`seats.rs` has what is new). Every point below is slice M5-S3's reading where the design is silent:

- **Seats.** Each idle Recruit gets the next free seat, the near side's first: near M (the starting Storage's), then near W, then near E. The far side's follow: M, W, E. Within a site, the patches nearest the starting Storage come first. Every Recruit is sent to its seat by a Move to the lane in front of its patch, and then a Gather. That sends the far side's round the row's end, as a player would ("Recruits on the far side are sent there by the scenario's orders").
- **Mining at a loss.** A Recruit on a near seat whose Storage doesn't stand yet mines anyway, taking its loads to the Storage nearest to itself. The far side's seats are dealt only once their own Storage is complete. Before then, a far-side Recruit's loads would go round the row to the near side, and its leg back would take it to the near face.
- **When a Storage is needed.** A Storage is needed whenever the player's Recruits outnumber the room at the Storages standing, a site counting from its order. Then the line orders the next site's Storage, once nothing else is being built and the bank covers it. All the Recruits count, also those with no seat yet. Counting only those mining, One Hall would stop at 16 mining, and 0385's Verification has it level off at 20.
- **Who builds.** The Recruits that mine at a loss (on a seat with no room yet) and those with no seat build. For the second Hall these are the Recruits on near W's patches, the farthest from the starting Storage ("built by the Recruits on the patches farthest from the starting Storage"). Each builder goes back to its seat after.
- **One Hall** builds only Storages. **Two Halls** orders its second Hall the moment the bank holds 300 SC, with no Storage before it, then Storages as needed. A Recruit with no seat is left idle: Two Halls' last 8.

### The targets

"Level" is a ratio of 1 within the tolerance; "slightly ahead" is above 1 by at most 15%. A figure out of tolerance is pinned in `economy_oracle/cults.rs` (`OUT_OF_TOLERANCE`) and brought to the designer. The setup is not changed to pass it.

| Figure | Measured | Target | |
|---|---|---|---|
| SC/s per Recruit at a 1-tile gap | 1.33 (a 24-tick trip) | ≈ 1.45 | −8.0%, pass |
| at a 2-tile gap | 0.57 (56 ticks) | ≈ 0.59 | −3.1%, pass |
| at a 3-tile gap | 0.36 (88 ticks) | ≈ 0.37 | −1.7%, pass |
| 6 Recruits on a long side (3 tiles), each against its rate alone | 1.00 (7.96 SC/s) | full efficiency (~6) | pass |
| 4 Recruits on a short side (2 tiles) | 1.00 (5.34 SC/s) | full efficiency (~4) | pass |
| 32 Recruits on the field's two sides | 0.92 (39.31 SC/s) | about 32 at full efficiency | −7.9%, pass |
| SC/s per Recruit gained, 2-tile gap to 1 | 0.76 | ~0.86 | −11.4%, pass |
| SC/s per face gained, 2-tile gap to 1 (6 Recruits on a long side: 7.96 against 3.43) | 4.54 | ~+4 | +13.4%, pass |
| A Storage's payback (100 SC over that gain) | 22.0 s | ~25 s | −11.8%, pass |
| The starting Hall from 6 Recruits to 20 (One Hall) | 168.0 s | 168 s | pass |
| The earliest second Hall, complete (Two Halls) | 1:21.12 | ~0:55–1:15 | +8.2% over 1:15, pass |
| One Hall's crystals by 1:30, against the GDO's | 860 / 980 = 0.88 | level | −12.2%, pass |
| against the Colonists' | 860 / 1032 = 0.83 | level | −16.7%, out |
| One Hall's crystals by 2:48, against the GDO's | 2316 / 2670 = 0.87 | slightly ahead | out |
| against the Colonists' | 2316 / 2744 = 0.84 | slightly ahead | out |

- **One face of a lone Storage** (`face_field`) gives the per-Recruit and per-face figures. It is the same as before 0385, and so are they. The trip is 24 ticks at a 1-tile gap, not the design's 22 frames. A Recruit stops at the first 4 su step that brings it within reach, so it walks 28 su each way, not 24 (`tests/recruit_gathering/trips.rs`). That is all of the 8% under 1.45. On a long side a seventh Recruit adds nothing (7.94 SC/s) and an eighth costs (7.28). On a short side a fifth adds 0.26 (5.60).
- **The earliest second Hall** is ordered at 0:44.75, the moment the bank holds 300 SC. Its crew is the 3 Recruits then on near W's patches, mining at a loss, and it is complete at 1:21.12. Before 0385, ideal mining had every Storage from the start: it ordered the Hall at 0:40.31, built it with the 4 Recruits nearest its site, and had it complete at 1:09.62.
- **One Hall replaces ideal mining** in the comparison with the GDO and the Colonists. It is 0.88 of the GDO's and 0.83 of the Colonists' by 1:30 (ideal mining's: 0.94 and 0.89), and 0.87 and 0.84 by 2:48 (0.88 and 0.86). It starts with one Storage, and the Recruits beyond its room build the next one or mine at a loss until it stands. These are the figures the design leaves open: "whether ~0.9 of the GDO and the Colonists by 2:48 is acceptable, to be decided on the new figures". By 0:30, 1:00, 1:30, 2:00, 2:30 and 2:48:

| Gathered since 0:00 | 0:30 | 1:00 | 1:30 | 2:00 | 2:30 | 2:48 |
|---|---|---|---|---|---|---|
| One Hall | 160 | 450 | 860 | 1330 | 1934 | 2316 |
| Two Halls | 170 | 430 | 708 | 1124 | 1724 | 2180 |
| GDO | 210 | 540 | 980 | 1530 | 2200 | 2670 |
| Colonists | 208 | 568 | 1032 | 1616 | 2304 | 2744 |

### The two scenarios

What each line builds: when it was ordered, by how many Recruits, and when it was complete.

| Line | Building | Ordered | By | Complete |
|---|---|---|---|---|
| One Hall | Storage, near W | 0:22.50 | 1 | 0:42.81 |
| | Storage, near E | 1:24.12 | 1 | 1:46.06 |
| | Storage, far M | 2:12.06 | 1 | 2:46.75 |
| Two Halls | the second Hall | 0:44.75 | 3 | 1:21.12 |
| | Storage, near W | 1:21.19 | 6 | 1:27.56 |
| | Storage, near E | 1:27.62 | 1 | 1:48.94 |
| | Storage, far M | 1:49.00 | 1 | 2:23.94 |
| | Storage, far W | 2:24.06 | 1 | 2:57.75 |
| | Storage, far E | 3:09.12 | 1 | 3:51.38 |

- **One Hall builds 3 Storages**, 4 with the starting one: room for 22, for its 20 Recruits. Each is ordered as a Recruit comes out to no room, the 7th, the 13th and the 17th, and built by that Recruit. All 20 mine from 2:48 on. This is 0385's Verification: "One Hall levels off at 20 Recruits mining, below the field's ~32 seats".
- **Two Halls builds 5 Storages**, every site, 6 with the starting one. This is 0385's Verification: "builds roughly 5–6 Storages along both sides". At 6:00 it has 40 Recruits: 32 mine, the field's seats ("Two Halls reaches ~32 mining"), and 8 have no seat. Near W is built in 6.4 s by the 6 Recruits then mining at a loss on its patches. Every other Storage is built by the one Recruit that came out to no room.

### The economies compared (design changes 0381 to 0383 and 0385)

Both Cults scenarios, the GDO's reference line and the Colonists' run to 6:00 (`economy_oracle/compared.rs`), each as above on its own reference field. At each checkpoint:

- **Income/s** is what the line gathered over the 160 frames before, divided by 10 (0382).
- **Total mined** is what it gathered since 0:00, the starting bank not included: the Recruits' and the Drones' deliveries, and what the GDO's plates mined.
- **Bank** is its unspent Space Crystals.
- **Spent** is everything it paid, net of refunds (0383: "everything spent", Generators included). These lines build nothing but economy.

At every checkpoint bank + spent = starting bank + total mined. The banks start at 0 for the Cults and 100 for the GDO and the Colonists. The test pins each figure (`COMPARED`): an income within 10% or 1 SC/s, an amount within 5% or 20 SC.

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

- **Two Halls has overtaken One Hall by 4:00** (`OVERTAKEN`), by income (36.2 against 26.6 SC/s) and by total mined (4676 against 4162 SC). It stays ahead from 2:34 by income and from 3:08 by total mined. At 6:00 it is 1812 SC ahead in total mined (9150 against 7338) and 1312 in the bank, and it takes in 38.8 SC/s against 26.4.
- **Two Halls falls behind early**, as the design allows. By 1:30 it has gathered 708 SC against One Hall's 860, and takes in 9.0 SC/s against 14.6: its Storages waited for its Hall, and six of its Recruits built near W from 1:21 to 1:27.
- **Against the other factions**: One Hall levels off at 26.4–26.6 SC/s with its 20 Recruits, under the GDO's 29.0 and the Colonists' 28.0–32.0. Two Halls takes in 36.2–39.0 from 4:00, above both. By 6:00 it has mined more than either (9150 against 7790 and 8224), and One Hall less (7338).
- **Ten seconds of income** move with the loads in the window. One Hall's 20 Recruits have all been at work since 2:48, and its figure moves by 0.2 SC/s from 4:00 to 6:00. Two Halls' rise from 4:00 to 5:00 is its last 4 seats at work: far E is complete at 3:51.
- **The GDO's plates pay 10 every 48 frames**, so 160 frames hold 3 or 4 payments a plate. From 2:48 on the checkpoints are whole numbers of cycles apart, so the window catches the same phase of every plate's cycle each time: 29 payments, 29.0 SC/s, against the 26.67 that its 8 plates (2 to start with and 6 built) mine on average.
- **What each line spends**: the GDO's reference line builds its 6 plates (450 SC) and a Generator whenever the bank covers one, 20 in all by 6:00 (2000 SC). The Colonists' trains a Drone every 10 seconds without a break, 36 paid for by 6:00 (1800 SC); its Beacons are Drones deployed, for nothing more. The Cults' Recruits are free. One Hall spends 300 on its 3 Storages; Two Halls spends 300 on its second Hall and 500 on its 5 Storages.

### Before design change 0385 (slices M5-L and M5-S2a)

The field was mined from one side only. Its 8 patches stood in faces a free column apart, each face with its own Storage a 1-tile gap south of it. From slice M5-S2a there were three faces, of 3, 3 and 2 patches, for the 3x2 Storage (design change 0380). Before that there were four 2-tile faces for the 2x2. The scenarios were ideal mining, which had every Storage from the start, and the user's realistic opening.

- **The field's ceiling** was about 16 Recruits: 20.31 SC/s with 16 and 21.05 with 20 (the 2x2 field's 20.49 and 21.97).
- **Ideal mining** (3x2) gathered 918 SC by 1:30, 0.94 of the GDO's and 0.89 of the Colonists'. By 2:48 it had 2362, 0.88 and 0.86 (out). Its income at 4:00 was 23.6 SC/s, and by 6:00 it had 6572 SC.
- **The realistic opening** (3x2) had its second Hall at 1:45.56 and its Storages at 1:52.44 and 1:59.94. It never overtook ideal mining: at 4:00 it took in 21.6 SC/s against 23.6 and had 3424 SC against 3904, and at 6:00 it was 630 SC behind. Its second Hall's Recruits found no seat on the one-sided field.
- **The 2x2 Storage** (slice M5-L): 5 Recruits on a 2-tile face brought 0.82 of five alone (out). A face's gain was 3.04 SC/s and a Storage's payback 32.9 s (both out). Ideal mining gathered 952 SC by 1:30 (0.97 and 0.92) and 2408 by 2:48 (0.90 and 0.88, out), and its earliest second Hall was complete at 1:07.62. The realistic opening gathered 504 SC by 1:30 and 1668 by 2:48.
- **Adjacent patches** (slice M5-S2a, one side): 8 adjacent patches with their Storages under the row took in as much as the faces apart with 16 Recruits (20.34 against 20.31 SC/s), and 8% less with 20. With one Storage north of the row the Recruits, coming from the south, mined every patch from the south, and that Storage took nothing. On the 2x2 field's adjacent patches, 0345's rule (nearest edge to edge from the patch) took in 15.74 SC/s with 16 Recruits, and 0375's (from the Recruit) 21.12.
- **A seat is kept** (slice M5-S2a). Before, a Recruit the game sent on to another patch left its seat free, and the realistic opening dealt all 40 Recruits to the field's 20 seats and jammed: 14.0 SC/s at 4:00.
