# Task M6-GL: Propulsion Labs and the Dragonfly

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** `artifacts/director/m6-plan.md > §B > GL: Propulsion Labs and the Dragonfly`, with T88 (`> §D > T88`, the glider's approach).

Read both before you start. **This slice has a Fable flag:** if the landing approach won't settle after two serious attempts, stop and report, and the Director decides.

## Goal
- **Propulsion Labs** (the Deployment Center's R, with an Airfield) unlocks the Dragonfly at the Airfield (W).
- **The Dragonfly** is a glider with one 90-damage homing anti-air missile, fired forward.
- **When empty,** it flies home, lands and rearms in 64 frames, as the Hornet does. It lands and takes off vertically.

## Running beside you
- **M6-ST1, the Subterranean Transport above ground:** the carrier (`UnitDef::carries`, `Place::Carrier`), boarding and Unload All, and `UnitDef::domain` in `combat::kind_domain`.
- **M6-GS1, the supply loop:** the Supply Tower, the Chopper's supply runs and schedule, and the GDO's starting Supply of 1 (0436), which moves every GDO golden. You both build on GH2's landing: GS1 adds the Tower's and a station's spots, you the glider's arm. If it merges first, re-bless `gdo_dragonflies` once more, for the starting Supply alone.

The lines you all share: the controls doc; the golden list and its counts; `ARCHITECTURE.md`. Whoever merges later rebases them, and checks the counts by hand (m6-common).

## The design: answers since the plan
Read `factions/gdo/units/dragonfly.md`, `factions/gdo/structures/propulsion_labs.md` and `factions/gdo/structures/airfield.md > Returning and rearming` as they stand.
- **0428 (Q0162) part 5: the Dragonfly lands and takes off vertically,** as the Specter at an air door. It approaches its slot at IdleSpeed, stops over it while it lands (16 frames), and takes off straight up, leaving at IdleSpeed. This is D34's placeholder, now the design.
  - 0428's Verification: "A Dragonfly returns: it slows to IdleSpeed over its slot, descends, rearms, lifts straight up."
  - 0428's parts 1–4 (air targets while landing, new aircraft landed, landed aircraft destroyed with the Airfield, an order during rearming) are GH2's, and apply to the Dragonfly as they do to the Hornet.
- **0435 (Q0169): its 4 Supplies,** carried by the recipe held in `defs::PLANNED_RECIPES` (`Recipe::supplies`). A test that trains it must give its player the Supplies.
- **0405 (Q0148): its SeparationRadius is 48 su,** and gliders push at a third of the HoverCraft's strength (0412).
- **0403: its SightRange is 8.** Check the def.
- **0457 (Q0182 part 4): its TargetType is SingleTarget,** confirmed (a homing missile). Replace the "Placeholder (not in the design: the Dragonfly's TargetType)" mark in `defs/gdo/units/dragonfly.rs` with 0457's citation.
- **Its forward shot** follows the Sparrow's rule (M5-H2's): no turret, it fires along its heading.
- **Anything the docs don't settle:** a placeholder, cited as m6-common says, and a line in your report. The Director files the question.

## Code
Follow `m6-plan.md > GL`'s Code list. Since the plan:
- **M6-GH2 built landing and rearming** (`landing.rs`, the slots, the landed domain, the rearm timer slowed by Power, `ExitRule::Pad`'s landed appearance). Read GH2's Built entry in `artifacts/director/plan.md` and its notes for you. The glider arm of `landing.rs` is yours.
  - **The approach:** M5-H1's glider (`movement/glider.rs`, read-only) must keep moving, so it can't hover. It approaches its slot at IdleSpeed and then stops over it to land, as 0428 part 5 says. Find where the glider's "must keep moving" rule lives, and let the landing phases override it, as the Specter's door descent will.
  - **Take-off:** straight up over the 16 frames, then it leaves at IdleSpeed.
  - **M6-SD built the Phantom's vertical door descent and rise** (`Mover::rise`, `Enter::descent`). Read them: the Dragonfly's vertical landing is the same shape. Reuse what fits, and say what in your report.
- **New aircraft** appear landed on a free slot (GH2's `ExitRule::Pad`), and take off for the rally point, vertically.
- **The menus:**
  - the Deployment Center's R: move Propulsion Labs' `RECIPE` out of `defs::PLANNED_RECIPES` into the Deployment Center's `constructs`, and turn `Planned::PropulsionLabs` into its `Kind`;
  - the Airfield's W: move the Dragonfly's `RECIPE` into the Airfield's `production`, and turn the planned W into its `Kind`.
- **New `Object` state,** if any: hash and save it only while present, under **Object tag 26**. The aircraft's state is GH1's tag 14, which GH2 extended; reuse it if it fits.
- **A new `OrderKind`,** if any, hashes as **tag 32** in `orders/hash.rs`.
- **The ruleset (X1):** the new def fields are structure, and the numbers are def numbers, by m6-common's checklist.
- **The controls doc:** regenerate it.

## Acceptance
`m6-plan.md > GL`'s Acceptance, plus:
- **0428's Verification** for the Dragonfly: it slows to IdleSpeed over its slot, descends, rearms, and lifts straight up.
- **GH2's landing tests,** repeated on the Dragonfly where they apply: air target while landing, ground target once landed, destroyed with its Airfield while landed, an order during rearming.
- **W** is greyed without Propulsion Labs. A Dragonfly costs 175 SC and 4 Supplies, and takes 320 frames.
- **Golden** `gdo_dragonflies`. No other golden should move. If `gdo_air` moves, find out why.
- **The computer players' games** are unchanged: none builds an Airfield.
- **Perf:** `air_perf` against master's binary (`target/perf/<hash>`).
- **QA screenshots:** the Deployment Center's R live; the Airfield's W live; a Dragonfly on its landing approach; one landed beside a Hornet.
- **Checks:** m5-common's "Done means".

## What you own
`m6-plan.md > GL`'s Owns list, and:
- Propulsion Labs' and the Dragonfly's lines in `defs::PLANNED_RECIPES`;
- the Deployment Center's R and the Airfield's W in their menu templates.

Read-only: `movement/glider.rs`, GH2's `landing.rs` beyond the glider arm, SD's rise and descent, and every other slice's files (`m6-plan.md > §C`).

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
- **M6-SB (d37b296d): the Vault and the Sludge.** The network's services run side by side (`network/services.rs`: the Vault's pass beside the Null Chamber's queue); the Shadebringer's tanks (`Object::tanks`, Object tag 24) and toggles (`Ability::Sludge/Cover`, in `modes.rs`); `Sim::patches` (Sim tag 4) for any `PatchKind`; `BuffKind::Sludge` slows. 33 goldens (`syndicate_sludge`).
- **GH2's notes for you:** reuse `landing::claims`, `slots` and `Landing`; a glider needs its own arrival check. `Landing` is hashed under `AircraftState`'s sub-tag 1 (Object tag 14): a new field of yours there takes sub-tag 2.
- **Your tags in the other namespaces** (`artifacts/director/hash-tags.md`): **Behaviour tag 20** and **Mover tag 13**, if needed.
- **M6-CP (the Cults' Pilgrim, 0462) comes later:** a glider that lands along an Airstrip's runway. Put a note in your report on what of your glider arm it can reuse.
- **Master's perf binary** for an A/B: `target/perf/d37b296d`, copied into your worktree with `target/`.

## Report
At most 12 lines:
- Propulsion Labs and the Dragonfly's production;
- the approach, the vertical landing and the take-off;
- its missile, its rearming, and returning home;
- separation;
- the golden, perf, and the computer players' games;
- notes for SA (the Specter's door landing) and placeholders for the Director to file.
