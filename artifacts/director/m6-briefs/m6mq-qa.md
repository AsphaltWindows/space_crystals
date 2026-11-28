# Task M6-MQ: M6's QA

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** `artifacts/director/m6-plan.md > §B > MQ: M6's QA`, with MQa's list (`> MQa`), which the partway playtest didn't take, so its checks are yours too.

M6 is every designed GDO and Syndicate building and unit, on placeholder numbers. You run after every other M6 slice has merged. M5-Mb's brief (`/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m5mb-qa.md`) is the model for the walkthrough and the playtest notes: read it.

## Goal
- **The walkthrough:** `qa/walkthrough/m6/run.sh`, up both factions' trees with every M6 unit and building, hot-seat, under `--fast` (M6-S6a fixed the lost key after a menu key).
- **Perf:** the M6 perf tests, with their means and p99 recorded.
- **Soak invariants** for M6's new state.
- **`docs/playtest/m6.md`:** what the user needs to playtest M6.

## Running beside you
Filled in at launch.

## The walkthrough
- **One scene per line of play,** each with its screenshots and the line in `run.sh` that says what it must show. Look at every screenshot, and check it shows that.
- **The GDO:** the Radar Station and Scan, the Defense System and Missile Turrets, the Drone Operator, the Factory with the Raptor, the Twister and its mines, the Artillery Rig deployed, the Prospector and an Outpost, the Field Engineer's Bunker and First Aid Tent, the Airfield with Hornets landing and rearming, Propulsion Labs and a Dragonfly, and the Supply Tower with a Chopper's run and a transport.
- **The Syndicate:** the Vehicle Assembly with the Assault Bike, the Laser Battery mobile and deployed, and the Shadebringer's Sludge and Cover; the Null Chamber and the Vault; the Forge, the Aerospace Assembly, the air doors and the Phantom; the Subterranean Transport loaded, burrowed and surfacing; the Orbital Command and the Prism's Thrust; the Relay; and the Specter's bombing run.
- **Anything that doesn't work as its slice's Built entry says** is a bug: report it with its scene, and don't fix it here unless it is a line or two.

## Perf
- `vehicles_perf`, `mines_perf` and `smoke_perf`;
- `air_perf` with Hornets landing and Phantoms at the doors;
- `movement_perf` with sludge, and `vision_perf`;
- each against M5-Mb's recorded numbers where it has them (`docs/perf.md`, or Mb's Built entry). Record the means and p99 in `docs/perf.md`.

## Soak invariants
In `tests/pairings/soak.rs`'s checks (M5-M1's), every tick:
- **MQa's:** a deploying unit holds no move; stocks stay within [0, max]; patches expire; mines never move.
- **MQ's:** ammo stays within [0, max]; a landed aircraft is on its own Airfield's pad, or its Tower's spot, or free ground (the Chopper); every passenger's carrier exists; a burrowed unit is hidden from every other player; the armed Prism has a live chain; battery charges stay within [0, capacity].
- **The Chopper:** it never carries units and Supplies at once.
- **The hash tags:** a test that no two `OrderKind`s, and no two optional `Object` states, hash under one tag (`artifacts/director/hash-tags.md` is the registry the slices followed).
- A soak that finds a real bug: report it with its seed; fix it only if it is small.

## `docs/playtest/m6.md`
- What's new in M6, by faction, and how to reach each unit (its building, its prerequisites, its Supplies for the GDO).
- **The placeholders:** every number and rule marked `Placeholder` in the M6 defs, grouped by unit, so the user can tune them (X2's data files).
- **What isn't in:** detection (no unit detects hidden ones yet), and anything a Built entry lists as left out.
- The computer players don't use M6 units: say so.

## Acceptance
- The walkthrough runs green, and every screenshot shows what its line says.
- The perf numbers are recorded, and none regresses more than 20% against M5-Mb's without a reason.
- The new soak invariants pass on `soak_sweep` and `behaviours_soak`.
- **The computer players' games:** the 192 rows, unchanged from master's.
- **Checks:** m5-common's "Done means".

## What you own
`qa/walkthrough/m6/`, `docs/playtest/m6.md`, the new soak checks, the hash-tag test, and `docs/perf.md`'s M6 lines.

Read-only: every slice's code, except for a fix of a line or two that you report.

## Landed since the plan (filled in at launch)
TBD.

## Report
At most 12 lines:
- the walkthrough: its scenes, and what didn't work;
- perf, against M5-Mb's;
- the soak invariants, and anything they found;
- the playtest notes;
- bugs for the Director, with their scenes or seeds.
