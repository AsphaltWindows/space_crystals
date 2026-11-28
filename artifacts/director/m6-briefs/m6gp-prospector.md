# Task M6-GP: The Prospector and the Outpost

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** `artifacts/director/m6-plan.md > §B > GP: The Prospector and the Outpost`, with T87 (`> §D > T87`, deploying from a site).

Read both before you start.

## Goal
- **The Factory trains the Prospector** (E), once its owner has Advanced Mechanics: 300 SC and 8 Supplies, 480 frames.
- **R Deploy into Outpost** targets a site on any Buildable ground. The Prospector drives there and, after 64 frames that can't be cancelled, becomes an Outpost.
- **The Outpost** is a 3×3 building that extends the GDO build area by 6 GU. It is permanent.

## Running beside you
Filled in at launch.

## The design: answers since the plan
- **The mechanics doc comes first** (0472): `systems/mechanics/modes.md` defines Unit Modes once, with two archetypes, Deploy and Toggle; the unit doc gives only its parameters. The Prospector's deploy is the **Deploy** archetype, permanent and targeted. Build it on the one modes module, shaped by the def, not by the unit's kind. A refactor phase before you may merge the existing deploy and toggle code into that module first; its handoff will say so.
Read `factions/gdo/units/prospector.md` and `factions/gdo/structures/outpost.md` as they stand.
- **0427 (Q0161): the damage carries over unchanged.** The Outpost starts at its MaxHP (600) minus the Prospector's damage, as Drone → Beacon does. This is D36's placeholder, now the design. Verification: a Prospector damaged to 200/300 deploys into an Outpost at 500/600.
- **0435 (Q0169): its 8 Supplies.** The recipe held in `defs::PLANNED_RECIPES` carries them (`Recipe::supplies`). A test that trains it must give its player the Supplies: the GDO starts with 1 (0436), and the supply loop is GS's, later.
- **0417: the tracked turning law** applies to the Prospector: TrackWidth 36 su, ReverseMaxSpeed 2. Check that M6-GK registered these numbers, and add them if not.
- **D36's other placeholders stand:** the footprint is centred on the target point, and its Unit Control (2) is freed at completion.
- **Anything the docs don't settle:** a placeholder, cited as m6-common says, and a line in your report. The Director files the question.

- **0457 (Q0182 part 2): the Outpost's Groupable is true,** as M6-GK assumed (a designer placeholder; it has no commands yet). Replace the "Placeholder (not in the design: the Outpost's Groupable)" mark in `defs/gdo/structures/outpost.rs` with 0457's citation.

## Code
Follow `m6-plan.md > GP`'s Code list. Since the plan:
- **M6-GA built the vehicle deploy model** (`vehicle_deploy.rs`, `orders/vehicle_deploy.rs`): the timed deploy that can't be cancelled, and the order held through it. GA left a site arm for you. Read GA's Built entry in `artifacts/director/plan.md`, and its notes for you.
  - The order's first half is `deploy.rs`'s Drone pipeline: the gate judging by what the player knows, the lingering ghost reserving the site, the walk, and the arrival check by the truth, with the deployer excluded (`Method::WorkerBuilt`).
  - Its second half is GA's deploy in place, 64 frames.
  - **Held orders:** an Outpost can carry out none, so each is dropped with the feedback.
- **Completion:** `change_kind` from unit to structure: the Drone's path to a Beacon, without its cancel. The damage carries over (0427).
- **The Outpost:** `seeds_build_area: true` with BRE 6, and its Power (−10) counted from completion.
- **The Factory's E:** move the Prospector's `RECIPE` out of `defs::PLANNED_RECIPES` into the Factory's `production`, and turn the menu's `Planned` entry into its `Kind`.
- **New `Object` state,** if any: hash and save it only while present, under **Object tag 22**. GA kept its deploy's progress in `Behaviour::VehicleDeploy` (Behaviour tag 13), and Object tag 16 stays unused: don't take it.
- **A new `OrderKind`,** if any (Deploy into Outpost with its point), hashes as **tag 28** in `orders/hash.rs`. If GA's `OrderKind::VehicleDeploy { into }` (tag 21) can carry a site, extend it instead, and hash the site only while present, so that `gdo_artillery` doesn't move.
- **The ruleset (X1):** the new def fields are structure, and the numbers are def numbers, by m6-common's checklist.
- **The controls doc:** regenerate it.

## Acceptance
`m6-plan.md > GP`'s Acceptance, plus:
- **0427's Verification:** a Prospector damaged to 200/300 deploys into an Outpost at 500/600.
- **The Factory's E** is live, greyed without Advanced Mechanics, and the Prospector's 8 Supplies are paid at the press.
- **Golden** `gdo_outpost`. No other golden should move.
- **The computer players' games** are unchanged: none builds a Factory.
- **QA screenshots:** the Factory's E live; the ghost, green and red; the deploy; the build-area overlay around the Outpost.
- **Checks:** m5-common's "Done means".

## What you own
`m6-plan.md > GP`'s Owns list, and the Prospector's line in `defs::PLANNED_RECIPES` and the Factory's menu template.

Read-only: `deploy.rs`, `transform.rs`, `placement/build_area.rs`, GA's deploy model beyond its site arm, and every other slice's files (`m6-plan.md > §C`).

## Landed since the plan (filled in at launch)
TBD.

## Report
At most 12 lines:
- the order: the ghost, the walk and the arrival check;
- the deploy, held orders, and completion (damage, Unit Control, Power);
- the build area;
- the golden, and the computer players' games;
- placeholders for the Director to file.
