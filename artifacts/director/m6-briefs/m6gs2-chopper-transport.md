# Task M6-GS2: The Supply Chopper carries units

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** design change 0442 (`artifacts/designer/changes/0442-supply-chopper-transport.md`), with `artifacts/director/m6-plan.md > §B > GS` for the Chopper and T94 (`> §D`, carriers and passengers).

The plan's GS was split in two: M6-GS1 built the supply loop, and this slice adds the Chopper's unit transport. It reuses M6-ST1's carrier (`Place::Carrier`, the passengers) and M6-GH2's landing.

## Goal
- **Capacity:** 4 infantry (one place each, Light or Heavy) or 1 vehicle of any kind, or Supplies. One kind at a time.
- **Loading:** infantry or a vehicle right-clicked on the Chopper (Enter) makes it land where it is; the units walk in and board instantly. The Chopper's Load on an own unit flies it over to land beside the unit, which walks in. Units that don't fit stay out.
- **F: Unload All,** targeted and Shift-queued: it flies to the spot, lands, and everyone comes out at once around it.
- **Landing** on infantry-walkable ground free of buildings and units, or the nearest free spot: 16 frames to land and to take off, an air target during both, a ground target while landed. It takes off by itself after loading or unloading.
- **Passengers** can't fire out, and all die if it's destroyed, landed or in the air.

## Running beside you
Filled in at launch.

## The design: answers since the plan
Read `factions/gdo/units/supply_chopper.md` as it stands: `> Cargo`, `> Unit Transport` and `> ObjectInterfaceState[SupplyChopper]`.
- **0442** is the whole slice (above). Its two refusals:
  - the supply commands (Pick Up, Drop Off, Attach) are refused while it carries units;
  - loading is refused while it carries Supplies.
- **0428 (GH2's landing):** air target while landing and taking off, ground target once landed. The Chopper lands anywhere here, not only on a Tower or a station.
- **0432 (ST1's Transport):** its passengers don't fire out and aren't in the network. The Chopper's passengers are the same kind of thing: reuse ST1's carrier rather than a second one.
- **0457: the Chopper's Unit Control is 2** (a designer placeholder). Passengers keep their own Unit Control.
- **Anything the docs don't settle:** a placeholder, cited as m6-common says, and a line in your report. The Director files the question.

## Code
- **The carrier:** M6-ST1's (`Place::Carrier`, `UnitDef::carries`, the boarding and unloading). Read ST1's Built entry in `artifacts/director/plan.md`. Give the Chopper its `carries`: 4 infantry places, or 1 vehicle.
  - **ST1's notes for you:** `UnitDef::carries` is ST1's `CarrierDef { capacity, places }`, with places counted per base. A capacity of 4 with a vehicle taking 4 places already allows only one kind of passenger at a time.
  - The boarding gate and `Sim::step_unloads` act only on a carrier that `carry::loads`; a landed aircraft counts, through `object_domain`'s landed arm. So "Enter lands it" needs an arm in the gate, and your targeted F lands first and then calls `Sim::unload` (private now: make it `pub(crate)`).
  - The passengers come out on `carry::ring_spots`, rings around the carrier's turned shape.
  - **Q0191 is open** (a crowded unload, who sees the passenger pips, F on the move). Build as ST1 did.
- **The cargo, one kind at a time:** GS1 built the Chopper's cargo as Supplies only, shaped so that units can sit beside them (its notes for you are in its Built entry). Make the kinds exclude each other.
- **Landing anywhere:** GH2's `landing.rs` lands aircraft on slots, and GS1 added a Tower's spot and a station's. Add a free-ground landing: the spot's check (infantry-walkable, no building, no unit) and the nearest free spot, by a bounded search like `sim/spot_work.rs::free_spot`.
- **The orders:** Load (the Chopper's, on an own unit) and Unload All (F, targeted, Shift-queued), in GS1's `orders/supply_run.rs` or a new `orders/chopper.rs`. Enter on the Chopper (a unit's right-click) lands it where it is.
- **Death:** every passenger dies with the Chopper (`Place::Carrier`'s rule, as ST1 built it for the Transport).
- **Hash tags:** new `Object` state hashes and saves only while present, under **Object tag 29**. Load and Unload All hash as **`OrderKind` tags 36 and 37** in `orders/hash.rs`.
- **The ruleset (X1):** the new def fields are structure, and the numbers are def numbers or tunables, by m6-common's checklist.
- **The controls doc:** regenerate it.

## Acceptance
- **0442's Verification:**
  1. Four Peacekeepers right-clicked on a hovering Chopper: it lands, they walk in and vanish one by one, it takes off.
  2. A fifth Peacekeeper stays outside.
  3. F and a click across the map: it flies there, lands, all four appear around it, and it takes off.
  4. Shot down loaded, the passengers die.
  5. An air-only weapon hits it while it lands, but not once it is down.
- **One vehicle:** a Raptor boards alone, and a second unit of any kind is refused.
- **The refusals:** a loaded Chopper refuses Pick Up Supplies; one carrying Supplies refuses a Load.
- **Landing on a blocked spot:** Unload All on a building lands it at the nearest free spot.
- **Golden** `gdo_airlift`. No other golden should move. If `gdo_supplies` moves, find out why.
- **The computer players' games** are unchanged: none builds a Supply Tower.
- **QA screenshots:** the Chopper landed with Peacekeepers walking in; the F targeting; the passengers coming out.
- **Checks:** m5-common's "Done means".

## What you own
- the Chopper's transport: its `carries`, Load and Unload All, and the free-ground landing;
- the Chopper's panel's F, and its other transport letters.

Read-only: ST1's carrier beyond the Chopper's arm, GH2's `landing.rs` beyond the free-ground arm, GS1's supply orders beyond the refusals, and every other slice's files (`m6-plan.md > §C`).

## Landed since the plan (filled in at launch)
TBD.

## Report
At most 12 lines:
- loading: Enter on the Chopper, and Load on a unit;
- Unload All, and the landing spot;
- one kind at a time, and the refusals;
- passengers' deaths;
- the golden and the computer players' games;
- placeholders for the Director to file.
