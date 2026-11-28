# Task M6-SL: The Laser Battery

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** `artifacts/director/m6-plan.md > §B > SL: The Laser Battery`, with T97 and T87 (`> §D`).

Read them before you start. The Relay through Prisms is SR's, later: not yours.

## Goal
- **The Vehicle Assembly trains the Laser Battery** (E), once its owner has an Optics Lab: 200 SC, 400 frames. It is tracked.
- **The battery** holds 100 charge, and charges 5 a second, always. Every 16 frames the laser surges for 4 × charge^(2/3), using 10, or whatever has charged.
- **Mobile,** it fires at ground targets only, with a 30° turret.
- **Deployed** (R, 48 frames), it can't move, and it gets a 360° turret and air targets only.
- **Amplify** (F) marks the next surge to use the whole battery.

## Running beside you
Filled in at launch.

## The design: answers since the plan
- **The mechanics doc comes first** (0472): `systems/mechanics/modes.md` defines Unit Modes once, with two archetypes, Deploy and Toggle; the unit doc gives only its parameters. The Laser Battery's Deploy is the **Deploy** archetype. Build it on the one modes module, shaped by the def, not by the unit's kind. A refactor phase before you may merge the existing deploy and toggle code into that module first; its handoff will say so.
Read `factions/syndicate/units/laser_battery.md` as it stands, `> Battery`, `> Attack`, `> Amplify`, `> Deploy` and `> Placeholder stats`.
- **No change entry since the plan touches the Battery** but these:
  - **0417 (Q0153): the tracked turning law.** Its numbers are TrackWidth 33 su and ReverseMaxSpeed 2.25 (spinning about 15.6° a frame). Check that M6-SK registered them, and add them if not.
  - **0433: smoke.** A Laser Battery firing from smoke at your unit becomes visible to you for 1.5 s after each hit. That's SC's: leave a note for SC on where a surge's hit is.
- **Amplify** is "usable mobile and deployed; not queueable" (`> Placeholder stats`). It marks the next surge; it doesn't fire.
- **The panel** isn't listed in the doc. Use R Deploy (Undeploy, deployed), F Amplify, and A Attack (A is Attack on every unit that can attack, 0397/0399), beside the standard Q Move and X Stop. Cite it as a placeholder, and put a line in your report.
- **D48** stands: the battery charges in every mode, while switching, and inside the network. The doc says "it charges in every mode, including while switching, as assumed".
- **Anything the docs don't settle:** a placeholder, cited as m6-common says, and a line in your report. The Director files the question.

## Code
Follow `m6-plan.md > SL`'s Code list. Since the plan:
- **M6-GA built the vehicle deploy model** (`vehicle_deploy.rs`, `orders/vehicle_deploy.rs`): two unit types, the timed deploy and undeploy, "Undeploy only once deployment is complete", and the held order. The Battery's pair is data on it. Read GA's Built entry in `artifacts/director/plan.md` and its notes for you: GA made T44 data-driven for you and ST2.
- **The battery:** `UnitDef::battery` and `Object::battery` (`battery.rs`). `change_kind` keeps it through the deploy.
  - Hash and save `Object::battery` only while present, under **Object tag 23** (14 to 22 are taken or promised).
  - It charges every tick, in every mode and inside the network (D48).
- **The surge:** in the FullyConnected phase. Each surge's damage is 4 × used^(2/3), where used is 10 or what has charged, or the whole battery after Amplify.
  - Compute it so that the tests can check 18.6, 11.7 and 86 "to f32 precision" on every platform. Use a deterministic power, not `f32::powf`, if the sim's rules forbid platform maths. Check `ARCHITECTURE.md` and the sim's other fractional powers.
- **Amplify:** `Ability::Amplify`, not queueable, usable in both modes.
- **The domains:** Ground mobile, Air deployed, per kind (`AttackAttrs::domain` on each def).
- **The Vehicle Assembly's E:** turn its `Planned` entry into its `Kind`, behind the Optics Lab.
- **A new `OrderKind`,** if any, hashes as **tag 29** in `orders/hash.rs`.
- **The ruleset (X1):** the new def fields are structure, and the numbers are def numbers, by m6-common's checklist.
- **The controls doc:** regenerate it.

- **A bug in GA's model, yours to fix:** an order Shift-queued behind a queued Undeploy is judged at the gate against the deployed type, so a Move there is refused (M6-GA's report). The Rig's doc checks each order once the switch has finished (`factions/gdo/units/artillery_rig.md > Deploy / Undeploy`: "At that point each order is checked against the deployed unit"). Judge a Shift-queued order against the type the unit will be when it runs, after any switch queued before it, with GA's `vehicle_deploy::can_take`.

## Acceptance
`m6-plan.md > SL`'s Acceptance, plus:
- **0369's Verification:** Amplify pressed on a battery between surges: the next surge empties it in one big hit, 86 from full.
- **Deploying** follows GA's tests, repeated on this pair, held orders included, and the battery charges while it deploys.
- **GA's queued-switch bug, fixed:** a Move Shift-queued behind a queued Undeploy is accepted, and runs once the Battery is mobile. Test it on both pairs, the Rig's and the Battery's.
- **Golden** `syndicate_lasers`. No other golden should move.
- **The computer players' games** are unchanged: none builds an Optics Lab.
- **Perf:** if the surge or the charge adds per-tick work for every unit, measure `movement_perf` against master's binary (`target/perf/<hash>`).
- **QA screenshots:** the Vehicle Assembly's E live; a mobile Battery firing at a ground target, with its battery bar; a deployed Battery's beam surging at an aircraft.
- **Checks:** m5-common's "Done means".

## What you own
`m6-plan.md > SL`'s Owns list, and the Battery's lines in `defs/syndicate/tests.rs` and `tests/syndicate_kinds.rs` (drop its two kinds from `SK_KINDS`, as SV and SD did for theirs).

Read-only: `vehicle_deploy.rs` and `orders/vehicle_deploy.rs` (GA's) beyond the queued-switch fix below, and every other slice's files (`m6-plan.md > §C`).

## Landed since the plan (filled in at launch)
TBD.

## Report
At most 12 lines:
- the battery, the surges and their damage;
- Amplify;
- the two modes, their domains, and the deploy;
- the golden, and the computer players' games;
- notes for SR (the Relay, from a deployed Battery) and SC (a surge from smoke reveals the Battery), and placeholders for the Director to file.
