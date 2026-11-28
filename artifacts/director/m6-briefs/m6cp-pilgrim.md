# Task M6-CP: The Cults' Pilgrim, a paratrooper glider

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** design change 0462 (`artifacts/designer/changes/0462-pilgrim-paratrooper-plane.md`) and `factions/cults/units/pilgrim.md`, with T94 (carriers and passengers) and T88 (aircraft landing) in `artifacts/director/m6-plan.md > §D`.

The Pilgrim was designed after M6's plan, so the plan has no section for it: this brief is the whole spec. It reuses four slices' work: M6-ST1's carrier, M6-GH2's landing, M6-GL's glider landing and M6-GS2's aircraft transport. Read their Built entries in `artifacts/director/plan.md` before you start.

## Goal
- **The Airstrip's W builds the Pilgrim**, once its owner has a Scrap Hangar: 2 Recruits, 300 Space Crystals, 960 frames, Unit Control 2.
- **On the runway:**
  - a new Pilgrim **appears landed** on the runway and stays until it is given an order;
  - any order makes it **take off along the runway** toward C, as the Sparrow does, and then carry the order out;
  - a right-click on an own Airstrip sends it there to **line up and land** along the runway, in 32 frames;
  - one Pilgrim per runway: one sent to an occupied runway circles over it until the runway is free;
  - while one is landed, the Airstrip's production carries on, but **a finished aircraft waits at 100%**.
- **Loading,** only while landed: up to 16 Cults infantry (Recruits, Militants, Gunners, Disciples and Disciples' illusions), by a right-click on the Pilgrim (Enter). They walk over and board.
- **F: Unload All,** only while landed: everyone comes out at once along the runway's side.
- **R: Drop,** targeted and Shift-queued:
  - the Pilgrim flies to the spot and drops one passenger every 2 frames, flying on;
  - each passenger falls for 48 frames, as an **air target that can't act**;
  - each lands below where it jumped, or at the nearest walkable spot.
- **Destroyed:** everyone aboard dies, landed or in the air.
- **Landed,** it is a ground target, and Recruits can repair it.

## Running beside you
Filled in at launch.

## The design
Read `factions/cults/units/pilgrim.md` (all of it), `factions/cults/structures/airstrip.md > Production` and `> Placeholder stats`, `factions/cults/structures/scrap_hangar.md > Unlocks`, and `factions/cults/units/recruit.md > Repair`, as they stand.
- **The designer's readings, now the design:**
  - an Airstrip holds one landed Pilgrim, and one sent to an occupied runway circles over it until the runway is free;
  - a falling passenger can't act: no attacks, spells or orders;
  - each passenger lands below where it jumped, or at the nearest spot walkable for infantry, as the Supply Chopper's landing does.
- **The designer's placeholder numbers** are in `pilgrim.md > Placeholder stats`: the Silhouette, HP, armour, sight and SeparationRadius; GliderMovement; the cost; the drop's 2 frames, the fall's 48 and the landing's 32; the panel (Q Move, X Stop, E HoldPosition, R Drop, F Unload All). Register them as they are.
- **The unload side:** the runway's long side to the left of the take-off direction, at the nearest free spots if it is crowded (a placeholder).
- **After a drop** it doesn't come back by itself: it circles where it is, as a glider's Stop does.
- **Recruits' right-click on a damaged landed Pilgrim:** Enter while it has room, and Repair once it is full, as for every unit holder (`recruit.md > Right-Click Resolution`: "Damaged own building or vehicle (completed) → Repair, **unless the building can be entered** by the selection: then **Enter** wins [...] **Except when it is full**"). D: Repair always repairs.
- **Anything the docs don't settle:** a placeholder, cited as m6-common says, and a line in your report. The Director files the question.

## Code
Filled in when ST1, GL and GS2 have merged. What is known now:
- **The kind:** register `ObjectKind::Pilgrim` with its def and recipe, and remove its line from `AWAITING_THEIR_SLICE` in `crates/sim/src/defs/cults/tests.rs`. The test then checks it like every other Cults unit.
- **The Airstrip** is M5-H2's: `ExitRule::Runway` and the Sparrow's take-off (`sim/holding/takeoff.rs`).
  - The Pilgrim appears landed and waits, where the Sparrow takes off at once.
  - A landed Pilgrim holds the runway, so a finished aircraft waits at 100%, as behind a blocked exit (`core/entities.md > Exit Doorway`).
  - W is greyed without a Scrap Hangar, the TechPrerequisite rule every producer uses.
- **The landing:** GH2's `landing.rs` lands aircraft on slots, and GL added the glider's arm. A runway landing (lining up along it) is a third arm. A landed Pilgrim is a ground target through `combat::object_domain`.
- **The carrier:** ST1's `UnitDef::carries` and `Place::Carrier`, with GS2's loading onto a landed aircraft. Loading only while landed on an Airstrip, Unload All only while landed, and everyone dies with the carrier.
  - **ST1's note:** the Pilgrim (16 places) loads while landed with no extra rule (`carry::loads` is true for a landed aircraft), but "out along the runway's side" needs its own spots in place of `carry::ring_spots`.
  - **Q0191 is open** (a crowded unload, who sees the passenger pips). Build as ST1 did.
- **The fall** is new: a dropped passenger is an air target (through `combat::object_domain`) that can't act for 48 frames, then is put on the map below its jump point, or at the nearest walkable spot (`sim/spot_work.rs::free_spot` or GS2's landing-spot search).
- **Recruits' repair** (M5): let its target filter accept a landed Pilgrim, and the right-click resolution weigh Enter against Repair by room, as it does for buildings.
- **Hash tags** (`artifacts/director/hash-tags.md`): new `Object` state hashes and saves only while present, under **Object tag 35**; the Drop order hashes as **`OrderKind` tag 43**; **Behaviour tag 32** and **Mover tag 18** (the runway landing and the fall), if needed. A sub-tag inside `AircraftState` (Object tag 14) takes the next free one after GH2's 1, GL's and SA's.
- **The ruleset (X1):** the new def fields are structure, and the numbers are def numbers, by m6-common's checklist.
- **The controls doc:** regenerate it.

## Acceptance
- **0462's Verification:**
  1. With a Scrap Hangar, a Pilgrim built at an Airstrip appears landed and stays put, and a Sparrow finishing meanwhile waits at 100%.
  2. Right-click the Pilgrim with 20 Militants: 16 walk over and board, and 4 stay outside.
  3. R and a far spot: it takes off along the runway, and the waiting Sparrow comes out. Over the spot, 16 parachutes come out in about 2 s, and land 3 s later in a short line. Enemy anti-air shoots them while they fall.
  4. Afterwards it circles there. Right-click an Airstrip: it flies over, lines up and lands.
  5. Damage it, and Recruits repair it while it is landed. F: everyone aboard comes out along the runway's side.
  6. Shot down with passengers aboard: all of them die.
- **Also:** W is greyed without a Scrap Hangar; an illusion boards and drops like a Disciple; a Pilgrim sent to an occupied runway circles until it is free; a Drop with no passengers just flies to the spot; Shift-queued Drops run in turn.
- **Golden** `cults_pilgrim`. No other golden should move.
- **The computer players' games** are unchanged: no computer player builds an Airstrip.
- **QA screenshots:** the Airstrip's W live; a Pilgrim landed with Militants walking to it; the parachutes falling in a line; a Pilgrim lining up to land.
- **Checks:** m5-common's "Done means".

## What you own
- the Pilgrim's kind, def, recipe and panel, and the Airstrip's W;
- the runway landing, the runway's hold on the Airstrip's production, and the fall;
- the Drop order;
- the Recruits' repair filter's arm for a landed Pilgrim.

Read-only: ST1's carrier beyond the Pilgrim's arm, GH2's and GL's landing beyond the runway arm, GS2's transport, and every other slice's files.

## Landed since the plan (filled in at launch)
TBD.

## Report
At most 12 lines:
- the Pilgrim on the runway: appearing, taking off, landing, and the Airstrip's production held;
- loading, Unload All, and the passengers' deaths;
- the Drop and the fall;
- repair;
- the golden and the computer players' games;
- placeholders for the Director to file.
