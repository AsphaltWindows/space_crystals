# Task M6-ST2: Burrowing

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** `artifacts/director/m6-plan.md > §B > ST2: Burrowing`, with T95 and T87 (`> §D`).

Read them before you start. Three design changes since the plan change it (below). **This slice has a Fable flag:** if surfacing and its reservation won't settle after two serious attempts, stop and report, and the Director decides.

## Goal
- **Burrow** (R, 48 frames) takes the Transport underground, into its Burrowed kind:
  - hidden from other players;
  - crossing any drillable tile at 3.5 su/f, under buildings and cliffs;
  - passing through every underground thing: expansions, lowered turrets and mines;
  - still hit by area damage.
- **Surface** (R, 48 frames) brings it up where a tracked vehicle could stand and no building or unit is. The spot is reserved meanwhile.
- **An Enter order while burrowed** surfaces it first, then drives it in.

## Running beside you
Filled in at launch.

## The design: answers since the plan
- **The mechanics doc comes first** (0472): `systems/mechanics/modes.md` defines Unit Modes once, with two archetypes, Deploy and Toggle; the unit doc gives only its parameters. Burrow and Surface are the **Deploy** archetype. Build it on the one modes module, shaped by the def, not by the unit's kind. A refactor phase before you may merge the existing deploy and toggle code into that module first; its handoff will say so.
Read `factions/syndicate/units/subterranean_transport.md` as it stands, from `> Burrow` to `> Destroyed with passengers`, and `systems/units.md > Underground Collision` and `> DrillUnit`.
- **0432 (Q0166) part 3: an Enter order given while burrowed** makes the Transport **surface by itself** (48 frames, where it is, by the Surface rules), then drive in through Side A like any vehicle. It never enters from below.
  - If it can't surface where it stands (a building, a tile a tracked vehicle can't cross, an overlapping unit), the order fails as a Surface would.
  - This **replaces** D45's placeholder and the plan's "a right-click on a Tunnel while burrowed is refused".
  - ST1 built Enter for the surfaced Transport. Chain Surface → Enter.
- **0426 (Q0160): mines are deployed units, not structures.** A burrowed Transport passes through them, and they don't go off. Nothing hidden can block it, so moving never reveals anything (`> Underground movement`).
- **0457: a mine has armour 12/12** (M6-GM builds it). A Grenadier's Attack Ground takes many grenades to destroy one; GM's report gives the number. Size the test below by it.
- **0419 (Q0153 part 4): underground, it never crushes.** ST1 made the surfaced kind crush, by its def. Check that the Burrowed kind doesn't.
- **Held orders** follow the deploy model (`> Burrowing and surfacing work like vehicle deploy`). Orders the unit can't carry out in its new mode are dropped. An Unload given while it burrows is dropped with the feedback, and one given while it surfaces runs once it's up.
- **Anything the docs don't settle:** a placeholder, cited as m6-common says, and a line in your report. The Director files the question.

## Code
Follow `m6-plan.md > ST2`'s Code list. Since the plan:
- **The pair** is data on M6-GA's `vehicle_deploy`, as SL's Battery is. GA made T44 data-driven for you. Read GA's and ST1's Built entries in `artifacts/director/plan.md`, and their notes for you.
- **The domain:** ST1's accessor puts the surfaced kind on Ground (`UnitDef::domain`). The Burrowed kind keeps its base's Underground.
- **ST1's notes for you:**
  - Burrow and Surface are a `vehicle_deploy` pair between the two types. Until you build it, R shows `Planned::Burrow`/`Surface`.
  - Passengers are keyed by the carrier's id, which a kind change keeps, so they stay aboard through both.
  - `carry::loads` is false underground: boarding a burrowed Transport is refused (`NotATunnel`), and so is its F (`NothingInside` when empty). Its passengers' deaths underground are already covered.
- **Nav:** the drillable class grid, on Z2's lazy grids.
- **Collision:** `Domain::hard_collision` is false for Underground. This closes gap G14.
- **Hiding:** an underground unit's arm in `Hiding::of`. GM built the mine's arm; add the unit's beside it.
- **Mines:** GM's trigger reads the enemy's base (hover bases don't trigger). Make sure an Underground unit doesn't trigger either, and that the blast still hurts it (area damage hits underground objects).
- **Surfacing:** its check (the tile itself: tracked-passable and free of buildings; an expansion is allowed) and its reservation, in new `surfacing.rs`.
- **Targeting:** resolve the TODO in `combat/targeting.rs` by kind.
- **New `Object` state** (the reservation, say): hash and save it only while present, under **Object tag 25** (14 to 24 are taken or promised).
- **A new `OrderKind`,** if any, hashes as **tag 31** in `orders/hash.rs`.
- **The ruleset (X1):** the new def fields are structure, and the numbers are def numbers, by m6-common's checklist.
- **The controls doc:** regenerate it.

## Acceptance
`m6-plan.md > ST2`'s Acceptance, with 0432 part 3 in place of the D45 case:
- **0432's Verification:** a burrowed Transport right-clicked on an own Tunnel surfaces (3 s), then drives into Side A. One that can't surface where it stands fails, as a Surface would.
- **0426:** a burrowed Transport crosses an enemy mine, and the mine doesn't go off. A Grenadier's Attack Ground over both hurts the Transport, and destroys the mine without a blast.
- **0419:** burrowed under enemy infantry, it passes with no effect.
- **Golden** `syndicate_burrow`. No other golden should move. If `syndicate_transport` moves, find out why.
- **The computer players' games** are unchanged: none builds a Forge.
- **Perf:** the drillable class grid and the underground arm in movement: `movement_perf` and `nav_perf` against master's binary (`target/perf/<hash>`).
- **QA screenshots:** the owner's view of a burrowed Transport under a cliff, and the enemy's; a Transport surfacing, with its reserved spot.
- **Checks:** m5-common's "Done means".

## What you own
`m6-plan.md > ST2`'s Owns list, and the Burrowed kind's lines in `defs/syndicate/tests.rs` and `tests/syndicate_kinds.rs`.

Read-only: `vehicle_deploy.rs` (GA's), `combat/mines.rs` (GM's), `nav/`, ST1's carrier, and every other slice's files (`m6-plan.md > §C`).

## Landed since the plan (filled in at launch)
TBD.

## Report
At most 12 lines:
- burrowing, and the Burrowed kind (domain, hiding, collision);
- underground movement: the drillable grid, passing through expansions, turrets and mines;
- surfacing: the check, the reservation, and Enter from below (0432 part 3);
- held orders;
- the golden, perf, and the computer players' games;
- placeholders for the Director to file.
