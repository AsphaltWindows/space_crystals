# Task M6-SC: The Shadebringer's Cover (smoke)

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** `artifacts/director/m6-plan.md > §B > SC: The Shadebringer's Cover`, with T93 (`> §D`, ground patches), and D43 and D44 (`> §E`).

Read them before you start. Smoke is the game's first sight blocker, and vision is the most perf-sensitive system: measure as you go.

## Goal
- **F toggles Cover:** the Shadebringer lays a wall of smoke behind it, 64 su (1 GU) wide. Each piece lasts 12 s.
- **The smoke blocks sight** into it and past it, for every unit and structure except satellites (the Prism).
- **Units inside the smoke** see nothing. So do structures.

## Running beside you
Filled in at launch.

## The design: answers since the plan
- **The mechanics doc** (0483): `systems/mechanics/energy_and_effects.md`. The shared tank (0468) is a **Drain toggle** pool: drained once per toggle that is on, several toggles can be on at once, an empty pool switches its toggles off, and the refill is a parameter. Smoke is a **Ground field**: an object is in it when its centre is; it has a duration and may linger; it is shown to every player over the tiles Visible to them; who it affects is a parameter. If the sludge was built otherwise (the X4 audit reports it), bring it in line in the same phase.
Read `factions/syndicate/units/shadebringer.md` as it stands, `> Cover (toggle)`, `> Charges and recharging` and `> Placeholder stats`, and `core/vision.md`.
- **0425 (Q0159), D44's answer, as for the Sludge:** read `artifacts/designer/changes/0425-shadebringer-trail-rules.md`.
  1. A stationary Shadebringer with a toggle on lays its effect where it stands, and drains. This replaces D44's "lays only while moving".
  2. A unit is inside smoke while its centre is.
  3. Entering the Tunnel Network switches both toggles off. An empty tank switches both toggles off (0468).
  - M6-SB built these rules for the Sludge. Reuse them for Cover, with one rule for both toggles.
- **0433 (Q0167), D43's answer:** read `artifacts/designer/changes/0433-smoke-reveal-and-scan.md`.
  1. An attacker in or behind smoke is revealed after hitting your unit, as in the fog. The reveal isn't sight, so smoke doesn't block it.
  2. Smoke blocks a Scan for what is inside it. A Scan has no viewpoint, so it isn't blocked "behind" smoke.
- **0468 (Q0190 follow-up), one shared tank:** read `artifacts/designer/changes/0468-shadebringer-one-shared-tank.md`. Sludge and Cover draw from **one tank of 200** (placeholder), in place of SB's two tanks of 100. It drains **10/s per toggle on** (20/s with both), refills at **10/s** with a Vault (`factions/syndicate/structures/vault.md > Throughput`: 20 s from empty), and an empty tank switches both toggles off. Either toggle switches on with any charge left. A new Shadebringer starts full (0467 part 2). The panel shows one gauge.
  - **This is your mechanic phase's first step,** before smoke: change SB's tanks (`patches.rs`, `defs/units/tanks.rs`, the Shadebringer's def, the Vault's refill in `network/services.rs`, the client's gauge) to one tank that any number of toggles drain, each at its own rate. Its goldens (the Shadebringer's) move: re-bless those only, in that commit.
- **The Prism sees through smoke** (`factions/syndicate/units/prism.md > Sight`): a satellite.
- **Anything the docs don't settle:** a placeholder, cited as m6-common says, and a line in your report. The Director files the question.

## Code
Follow `m6-plan.md > SC`'s Code list. Since the plan:
- **M6-SB built the patches** (`patches.rs`, `tuning/patches.rs`) and the Shadebringer's tanks and toggles. Read SB's Built entry in `artifacts/director/plan.md`, and its notes for you. Smoke is a second kind of patch.
  - **SB's notes for you:** `Patches` takes any `PatchKind` and hashes each patch's kind, so smoke shares `Sim::patches` (Sim tag 4), and a game with no smoke hashes as before. Sim tag 5 stays free. `patches_view` draws only sludge: smoke needs its own look.
- **The occlusion,** in `vision/sight.rs`: each eye carries the revision of the smoke near it, so an eye far from any smoke costs nothing new.
- **Scan (0433 part 2):** GR's `vision/scans.rs`. A Scan reveals nothing inside smoke and everything else in its circle. If the occlusion in `sight.rs` can't give that, add the smoke check to the Scan's reveal, and say so in your report.
- **The reveal (0433 part 1):** `vision/reveal.rs` (read-only) stays as it is; add a test that smoke doesn't block it.
  - **M6-SL's note:** a Laser Battery's surge hits from where its notes for you say. Use it for 0433's Verification.
- **Satellites:** `UnitDef::satellite`. If M6-SN or M6-SP added it (or `manned`), use it. If not, add it, false for every unit but the Prism.
- **Cover's toggle:** SB built `Ability::Cover { on }` beside `Ability::Sludge` (in `modes.rs`, with `Ability::switching`), and the tanks (`Object::tanks`, Object tag 24). Cover drains the shared tank (0468, above), so the Shadebringer's def needs no second `TankDef`. Once Cover's toggle is in the def, the panel shows F by itself: remove `Planned::Cover`.
- **New `Object` state,** if any, hashes and saves only while present, under **Object tag 34**. A new `OrderKind`, if any, hashes as **tag 42**.
- **The client:** the smoke's look, and the fog with a smoke shadow.
- **The ruleset (X1):** the new def fields are structure, and the numbers are def numbers or tunables, by m6-common's checklist.
- **The controls doc:** regenerate it.

## Acceptance
`m6-plan.md > SC`'s Acceptance, plus:
- **0425:** a stationary Shadebringer with Cover on lays smoke where it stands and drains; entering the network switches Cover off; an empty tank switches it off.
- **0468's Verification:** with only Sludge on, a full tank lasts 20 s; with both on, 10 s, and then both switch off. Inside the network with a Vault, an empty tank refills in 20 s. Either toggle switches on with any charge left.
- **0433's Verification:**
  1. A Laser Battery firing from smoke at your unit becomes visible to you for 1.5 s after each hit.
  2. A Scan over an area half covered by smoke: units in the smoke stay hidden, and units beyond the smoke inside the circle are shown.
- **The Prism** sees through a wall, and into the smoke.
- **Perf:** the new `smoke_perf` (four Shadebringers' walls and 200 sight sources) within 20% of `vision_perf`'s mean, with its p99 recorded; and `vision_perf` itself against master's binary (`target/perf/<hash>`), which must not move.
- **Golden** `syndicate_smoke`. No other golden should move.
- **The computer players' games** are unchanged: none trains a Shadebringer.
- **QA screenshots:** the fog with a smoke shadow behind a wall; the enemy's view of a unit inside smoke; a Scan half over smoke.
- **Checks:** m5-common's "Done means".

## What you own
`m6-plan.md > SC`'s Owns list, and Cover's arm of the Shadebringer's toggles.

Read-only: `vision/scans.rs` (GR's) beyond the smoke check, `vision/reveal.rs`, SB's patches and toggles beyond smoke's arm, and every other slice's files (`m6-plan.md > §C`).

## Landed since the plan (filled in at launch)
TBD.

## Report
At most 12 lines:
- Cover: laying, the wall, the pieces' life, and 0425's rules;
- the occlusion: how it works, and what an eye pays;
- inside the smoke, structures, and satellites;
- Scan and the reveal (0433);
- the golden, perf (`smoke_perf`, `vision_perf`), and the computer players' games;
- placeholders for the Director to file.
