# The economy oracle: measured results

The economy oracle (`crates/sim/tests/economy_oracle/`, see `ARCHITECTURE.md > Testing`) plays openings from a real start, the way a player would, through the command gate, and checks them against `factions/gdo/economy.md` and the Syndicate's economy check (`factions/syndicate/units/agent.md > Gathering > Space Crystals`). This page records what it measured. The tests check these numbers against the design, each within the tolerance named where it is checked, so a change that moves one too far fails there. A change that moves one at all updates this page.

To print every line, run:

```sh
cargo test -p sc_sim --test sim economy_oracle -- --nocapture
cargo test --release -p sc_sim --test sim economy_oracle -- --ignored --nocapture
```

The second command runs the ignored measurements: the Colonists' lines to 6:00, the saturation curve, and the Syndicate's whole table.

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
| 1 | 8.38 / 9.3 (−9.9%) | 5.97 / 5.36 (+11.4%) | 16.8 / 19 (−11.8%) | 2.09 / 2.33 (−10.1%) | 12.00 / 10.7 (+12.1%) | 9.83 / 12.5 (−21.3%) | 5.83 / 12.5 (−53.3%) | recorded |
| 2 | 9.75 / 10.7 (−8.9%) | 5.13 / 4.69 (+9.3%) | 19.5 / 21 (−7.1%) | 2.44 / 2.67 (−8.7%) | 10.17 / 9.4 (+8.2%) | 11.50 / 12.5 (−8.0%) | 10.83 / 12.5 (−13.3%) | recorded |
| 3 | 11.00 / 12.0 (−8.3%) | 4.55 / 4.17 (+9.0%) | 22.0 / 24 (−8.3%) | 2.75 / 3.00 (−8.3%) | 9.00 / 8.3 (+8.4%) | 11.83 / 12.5 (−5.3%) | 12.17 / 12.5 (−2.7%) | pass |
| 4 | 12.38 / 13.3 (−7.0%) | 4.04 / 3.75 (+7.7%) | 24.8 / 27 (−8.3%) | 3.09 / 3.33 (−7.1%) | 8.00 / 7.5 (+6.7%) | 11.00 / 11.3 (−2.7%) | 12.00 / 12.5 (−4.0%) | pass |
| 5 | 13.75 / 14.7 (−6.5%) | 3.64 / 3.41 (+6.6%) | 27.5 / 29 (−5.2%) | 3.44 / 3.67 (−6.3%) | 7.17 / 6.8 (+5.4%) | 10.50 / 10.2 (+2.9%) | 12.50 / 12.5 (+0.0%) | pass |
| 6 | 15.00 / 16.0 (−6.2%) | 3.33 / 3.12 (+6.8%) | 30.0 / 32 (−6.2%) | 3.75 / 4.00 (−6.2%) | 6.50 / 6.2 (+4.8%) | 9.67 / 9.4 (+2.8%) | 12.50 / 12.5 (+0.0%) | pass |

0294's Verification, "At 3 GU, 3 Agents reach about 12.5 SC/s; at 4 GU about 11.3; at 6 GU, 4 Agents saturate": 11.83 (−5.3%), 11.00 and 12.50.

- **Trips** are shorter than the table's, which walks the whole distance both ways. The Agent's own 36 su and the two 8 su reaches come off. So the Agents' incomes run above the table, and the saturating counts below it.
- **3 Agents at 3 GU** fall short of the hatch's ceiling although their trips (176 ticks) are shorter than three drop-offs (192). Near the hatch they get in each other's way, and the hatch stands free at times while a loaded Agent finds its way past the others. (Before design change 0315 that cost more, 10.67: an Agent whose leg ended behind another stood for 2 s before it tried again.)
- **Rows 0 and 1** jam. With the patch that close, Agents can't pass each other between it and the hatch, and an Agent behind another in its lane is blocked for good (`tests/agent_gathering/trips.rs`). Income falls to one Agent's: 2 and 3 Agents at 0 GU, and 4 at 1 GU (3 at 1 GU too before design change 0315; 9.83 with it). Design change 0294 accepts this: "Very close to the hatch (0–2 grid units) they can get in each other's way and income drops".

**The ceiling** held exactly in every game: no two deliveries at a Tunnel came less than 64 ticks apart. Four Agents at 5 and 6 GU reach exactly 12.50.

**Stacked Tunnels** at 3 GU (`stacked_field`): two Tunnels on opposite sides of the field, each hatch 3 GU from its own patch, the two patches a free column apart, four Agents for each Tunnel. With the patches side by side, the ends of their north and south faces lie as near either hatch, and some Agents delivered at the other Tunnel. The Tunnels take in 12.00 and 11.83, 23.83 SC/s in all: −4.7% against twice the ceiling, and −2.1% against twice one Tunnel's 12.17 with four Agents at 3 GU (11.83 and 11.50 before design change 0315). Every Agent delivered at its own Tunnel. Pass.

**The opening** at 3 GU (`opening_field`) is played from the Syndicate's Starting Conditions on the reference layout: the Tunnel, the Hideout in its area, one Agent, 100 SC and 1 Supply. The Agent is sent to the patch, the Hideout is rallied on it, and the Hideout trains an Agent whenever it is free and the bank covers it. The second Agent starts in tick 1 ("the second Agent can start immediately"). The third is paid for at 0:24.6 and comes out to gather at 0:39.56, against 0:34–0:46: pass. The check reads "saturated (3 Agents, 12.5 Space Crystals per second) around 0:40" as the third Agent at work. Three Agents at 3 GU take in 11.83 at best, so the Tunnel never takes in 12.5 without a break with three; from 1:00 to 1:30 it takes in 11.67 SC/s (10.00 before design change 0315).

**The lowest one-base saturation**: one Tunnel takes in at most 12.50 SC/s, against the GDO's 26.67 fully plated and the Colonists' 32 (31.62 measured, above). Pass.
