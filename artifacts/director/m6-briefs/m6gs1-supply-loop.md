# Task M6-GS1: The Supply Tower, the Supply Chopper and the supply loop

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** `artifacts/director/m6-plan.md > §B > GS: The Supply Tower and the Supply Chopper`.

Read it before you start. The plan's GS is now **two slices**: yours, the supply loop, and GS2 later, the Chopper's unit transport (0442), which waits on ST1's carrier. Its "gated on D41" is settled: Q0169 gave GDO units Supply costs (0435), so the GDO needs this loop to train them.

## Goal
- **The Deployment Center builds the Supply Tower** (with a Radar Station).
- **The Tower** comes with one free, attached Supply Chopper, and trains more (Q: 100 SC, 160 frames).
- **Choppers** fly to a Supply Delivery Station, land, pick up all its Supplies, and drop them at a Tower.
- **Scheduled deliveries** (the Tower's S). An attached Chopper leaves in time to reach its station as the next delivery arrives.
- **The GDO starts with 1 Supply** (0436).

## Running beside you
- **M6-ST1, the Subterranean Transport above ground:** the carrier (`UnitDef::carries`, `Place::Carrier`), boarding and Unload All, and `UnitDef::domain` in `combat::kind_domain`. GS2 later reuses ST1's carrier for the Chopper's passengers: build the cargo as one kind at a time, as your brief says.
- **M6-GL, Propulsion Labs and the Dragonfly:** the glider's vertical landing on GH2's slots. You both build on GH2's landing: GL adds the glider's arm, you the Tower's and a station's spots. Keep to your arms.

The lines you all share: the controls doc; the golden list and its counts; `ARCHITECTURE.md`. Whoever merges later rebases them, and checks the counts by hand (m6-common).

## The design: answers since the plan
Read `factions/gdo/economy.md > SupplyTower` (all of it) and `factions/gdo/units/supply_chopper.md` as they stand, and `core/map.md > SupplyDeliveryStation`.
- **0435 (Q0169):** read `artifacts/designer/changes/0435-gdo-supply-costs.md`.
  1. **The six units' Supply costs** were built by their slices (GD, GH1, GA and the others), and the production buttons' second resource with them. Nothing of yours, except to check that a unit greyed for Supplies goes live once the loop brings them.
  2. **A Chopper landed on its own Tower is repaired free,** at the repairers' rate: 0 to full in 80 frames (half its build time), about 30 HP a second (a designer placeholder).
- **0436: the GDO starts with 1 Supply** (`factions/gdo/economy.md > Starting Conditions`). Change `StartingConditions::of(Gdo)`.
  - **Every golden with a GDO player moves,** by its bank's Supply. Re-bless them, and check that each one's play is unchanged: the Supply should be the only difference. Nothing in those games spends Supplies.
  - **0436's Verification:** a new GDO game's top bar shows Supplies 1. A Field Engineer can be queued once a Career Campus exists. A Prospector can't, until 7 more Supplies arrive.
- **0423 (Q0157): low Power slows "everything" a consumer does.** The Tower draws Power (−15), so its production slows, as every producer's does. Its free repair of the Chopper is something the Tower does, so slow it by the Power ratio too (`PowerSummary::speed`), citing 0423. Say so in your report: it's a reading, and the Director checks it with the Designer.
- **0456 (Q0181): the Chopper's SeparationRadius is 68 su.** M6-SK or GK registered it. S6b replaces the "Placeholder (Q0181)" marks: leave them.
- **0442 (the transport) is GS2's.** Leave the Chopper's Load, Enter and Unload All out, but keep its two refusals in mind: supply commands are refused while it carries units, and loading is refused while it carries Supplies. Build the cargo as one kind at a time, so that GS2 can add units beside Supplies.
- **The Chopper's panel** has no letters in the doc for Pick Up Supplies, Attach to Tower and Drop Off Supplies. Pick them from `systems/controls/interface.md > CommandPanel > Standard Slot Assignments`, cite them as placeholders, and put a line in your report. F stays free for GS2's Unload All.
- **Anything the docs don't settle:** a placeholder, cited as m6-common says, and a line in your report. The Director files the question.

## Code
Follow `m6-plan.md > GS`'s Code list. Since the plan:
- **M6-GH2 built landing** (`landing.rs`: 16 frames down and up, an air target meanwhile and a ground target once landed, the landed state). The Chopper lands on a Tower and on a station the same way. Read GH2's Built entry in `artifacts/director/plan.md` and its notes for you.
  - The Tower has one landing spot. The Airfield's slots are GH2's; give the Tower its own spot in its def.
  - Landing on a station: the station's centre, or a placeholder spot.
- **The pick-up** reuses M4-I's stations (`economy/stations.rs`'s `take` and its claims) for a whole station. Agents and a Chopper share a station by M4-I's claims.
- **The orders:** PickUpSupplies, DropOffSupplies and AttachToTower, in new `orders/supply_run.rs`, with the right-click resolution of `supply_chopper.md > ObjectInterfaceState`.
  - **Attachment:** a Chopper attaching evicts any landed Chopper that isn't the attached one, no other Chopper lands on a Tower that has one attached, and any command breaks the attachment.
- **The Tower's schedule** (S Schedule Deliveries, `> Scheduled Deliveries`):
  - the departure time, so that the Chopper arrives as the next delivery does. Estimate the flight from the distance and the Chopper's drag numbers, or simulate it ahead; say which;
  - "If the travel distance is too long or deliveries at the SDS are too frequent, the chopper departs immediately after dropping off supplies";
  - one Chopper at a time per station: the closest Tower with a Chopper ready goes first.
- **The Tower's free Chopper** at placement: spawned landed on it and attached, at no cost.
- **The Tower's production:** move its and the Chopper's `RECIPE`s out of `defs::PLANNED_RECIPES`, into the Deployment Center's `constructs` and the Tower's `production`, and turn the `Planned` entries into their `Kind`s.
- **The client:** `templates/gdo/{supply_tower,supply_chopper}.rs`, the Chopper's carried-Supplies pips, and the Tower's scheduled station shown when the Tower is selected.
- **Hash tags:** new `Object` state hashes and saves only while present: the Chopper's cargo and attachment under **Object tag 27**, the Tower's schedule under **28**. The three new `OrderKind`s hash as **tags 33, 34 and 35** in `orders/hash.rs`.
- **The ruleset (X1):** the new def fields are structure, and the numbers are def numbers or tunables, by m6-common's checklist.
- **The controls doc:** regenerate it.

## Acceptance
`m6-plan.md > GS`'s Acceptance, plus:
- **A scheduled run** arrives no later than the station's delivery, and no more than 16 ticks before it. It picks up everything and lands back at the Tower, and the GDO's Supplies rise by the delivery.
- **Too far, or too frequent:** the Chopper leaves at once after each drop-off.
- **Two Towers scheduled on one station:** only one Chopper flies at a time, the closer Tower's first.
- **0435's Verification,** part 2: a damaged Chopper landing on its Tower is back to full within about 5 s, at no cost. At 59% Power, it takes longer (0423, the reading above).
- **0435's Verification,** part 1, with the loop: a GDO game with 0 Supplies has the six units greyed. A Chopper's run brings Supplies, and a Drone Operator can then be trained.
- **0436's Verification** (above).
- **The landing rules:** eviction, a command breaking attachment, repair while landed, and air target while landing.
- **Golden** `gdo_supplies`. Re-bless only the GDO goldens that 0436's Supply moves, and list them in your report.
- **The computer players' games:** the GDO computer player starts with 1 Supply and spends none, so the 192 rows should stay identical. Check them.
- **QA screenshots:** the Tower's panel with S Schedule; a Chopper landing on a station; the top bar's Supplies rising after a run.
- **Checks:** m5-common's "Done means".

## What you own
`m6-plan.md > GS`'s Owns list, and:
- `StartingConditions::of(Gdo)`, and the goldens 0436 moves;
- the Supply Tower's and the Chopper's lines in `defs::PLANNED_RECIPES`;
- the Tower's landing spot, and the Chopper's arm in GH2's landing.

Read-only: `economy/stations.rs`, GH2's `landing.rs` beyond the Chopper's arm, and every other slice's files (`m6-plan.md > §C`).

## Landed since the plan (filled in at launch)
- **M6-GK (459cabbd) and M6-SK (f09e6eb7): the kinds,** with `Planned` entries for what isn't live.
- **M5-X1 (a8e9f9af) and M5-X2 (2b2f275c): the ruleset, data files and live tuning.** m6-common's checklist covers what you owe them.
- **M6-SV (5e0d600f, 36901a85): the Vehicle Assembly and the Assault Bike,** the first vehicle out of a Tunnel.
- **M6-GD (b35ce523): the Drone Operator,** its stock (`stock.rs`), launch, leash and battery.
- **M6-GR (5cb666c8): the Radar Station's scan and the Missile Turret,** and low Power slowing a consumer's timers (`PowerSummary::speed`, 0423).
- **M6-GF (1504f447): the Factory, the Raptor and the Twister;** vehicles park round a taken rally point (`sim/producers/park.rs`).
- **M6-SD (e3895173): the air doors and the Phantom** (`Mover::rise`, Mover tag 10; `Enter::descent`; the air lane). It split `network.rs` (`network/paths.rs`).
- **M6-GH1 (0cf082c4): the Airfield and the Hornet** (`AircraftState`, Object tag 14; Rebase; ReturnToBase).
- **M6-GA (df49b7f9): the vehicle deploy model.** `UnitDef::vehicle_deploy { into, frames }`, `OrderKind::VehicleDeploy` (tag 21), `Behaviour::VehicleDeploy` (Behaviour tag 13). The last order given while a unit switches is held and checked against the new type when the switch ends (`vehicle_deploy::can_take`); what it can't carry out is dropped with `SimEvent::OrderDropped`. A type with top speed 0 refuses every order that would move it (`Reject::CannotMove`).
- **M6-SN (c5469988): the Null Chamber.** `network/services.rs`; both eject lanes call `Sim::leave_network(unit, pos, facing)`; `BuffKind` wraps `AreaAbility` (`BuffKind::Area`) beside `BuffKind::NullChamber`.
- **M6-CM (f4651a44): the Disciple's Mirage,** and the Shroud cloud's 1.5 s linger (`Member::until`). The Cults' only.
- **M6-GH2 (800b81d5): landing and rearming.** Aircraft land on the Airfield's 4 slots in 16 frames (`landing.rs`: `users`, `claims`, `slots`, `Landing`; `Sim::step_landings`), rearm at the Power speed, and lift off at any order; new aircraft appear landed (`ExitRule::Pad`). A landed aircraft is a Ground target: `combat::object_domain` overrides `combat::kind_domain`, the type's, and every targeting reader asks `object_domain`. 32 goldens (`gdo_air`).
- **M6-S6a (69df0290): an answer sweep.** A crusher crushes at any speed and turning in place (0453, `crush::drives`); the MinRange back-off goes to MinRange + 1 GU, and a unit that can't aim moves to a spot it can (0454, 0460, `orders/aim.rs`); one check, `combat::can_hit`, decides who can hit a target, for the gate, the right-click and the client (0461), with GH2's domain. QA input scripts act at their tick under `--fast`.
- **GH2's notes for you:** reuse `landing::claims`, `slots` and `Landing`, with the Tower's spot and a station's beside the Airfield's slots. `Landing` is hashed under `AircraftState`'s sub-tag 1 (Object tag 14).
- **Your tags in the other namespaces** (`artifacts/director/hash-tags.md`): **Behaviour tag 21** and **Mover tag 16**, if needed.
- **The goldens 0436 moves:** every golden with a GDO player and a starting bank, `gdo_air` (GH2's) among them. List them in your report.
- **Master's perf binary** for an A/B: `target/perf/69df0290`, copied into your worktree with `target/`.

## Report
At most 12 lines:
- the Tower, its free Chopper and its production;
- the orders, attachment and eviction;
- the schedule: how the departure is timed, and how close the arrival is;
- the repair, and 0423's reading;
- the starting Supply, the goldens it moved, and the computer players' games;
- notes for GS2 (the transport: the cargo's shape, landing anywhere), and placeholders for the Director to file.
