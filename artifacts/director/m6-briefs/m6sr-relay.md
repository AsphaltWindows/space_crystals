# Task M6-SR: The Relay

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** `artifacts/director/m6-plan.md > §B > SR: The Relay`, with T97 (`> §D`, the Laser Battery's battery, Amplify and Relay).

Read both before you start. M6-SL built the Battery, and M6-SP the Prism.

## Goal
- **Deployed Laser Batteries relay their beams through Prisms.**
- **Relay** highlights the Prisms that every selected Battery can reach: a Prism within 8 GU of a Battery, and every Prism linked to one of those by a chain of Prisms 8 GU apart.
- **The chosen Prism is armed.** It fires at anything its player sees within 6 GU, ground or air. Each surge is the sum of the relaying Batteries' own, and discharges each of them as normal.
- **The Batteries are locked in,** each to its nearest Prism. A broken chain ends the relay.

## Running beside you
Filled in at launch.

## The design: answers since the plan
Read `factions/syndicate/units/laser_battery.md > Relay` and `> Placeholder stats`, and `factions/syndicate/units/prism.md`, as they stand.
- **0434 (Q0168): the Prism's panel** is Q Move, R Thrust, X Stop, and **A Attack while armed** (0397, 0399). This was D47's placeholder; it is the design now. SP built the panel without A: add it, live only while the Prism is armed.
- **"Directing the armed Prism":** selected, it takes an Attack on a chosen target, which it holds over its own picks.
- **While relaying,** the Batteries' panel offers Stop Relaying, Relay (another target Prism) and Amplify, the last "if the target Prism has an active target". SL built Amplify (not queueable, usable in both modes).
- **0433 (SC's smoke):** a Battery firing from smoke is revealed for 1.5 s after each hit. If SC has merged, check that a relayed surge reveals the Battery as SC's rule says, and say what it reveals (the Prism, the Batteries, or both) in your report: it is a reading for the Director.
- **Anything the docs don't settle:** a placeholder, cited as m6-common says, and a line in your report. The Director files the question.

## Code
Follow `m6-plan.md > SR`'s Code list. Since the plan:
- **M6-SL built the battery** (`battery.rs`, `UnitDef::battery`, `Object::battery`), the surge in the FullyConnected phase, and Amplify. Read SL's Built entry in `artifacts/director/plan.md`, and its notes for you. A surge's damage is SL's deterministic power: reuse it for the sum.
- **M6-SP built the Prism** (the Orbital Command, Thrust, `UnitDef::satellite` or `manned`). Read SP's Built entry, and its notes for you.
- **The order:** `OrderKind::Relay { prism }`, in new `orders/relay.rs`. It hashes as **`OrderKind` tag 40** in `orders/hash.rs`.
- **The chain,** derived each tick from positions, in new `relay.rs`. Keep it cheap: the Batteries and Prisms of one player, only while one relays.
- **The armed Prism's attack:** T49's picker, firing at the surge cadence. New `Object` state (the armed state, the lock) hashes and saves only while present, under **Object tag 32**.
- **The client:** the relay targeting, the highlights, the beams, and the panels.
- **The ruleset (X1):** the ranges (8, 8 and 6 GU) are def numbers, by m6-common's checklist.
- **The controls doc:** regenerate it.

## Acceptance
`m6-plan.md > SR`'s Acceptance, plus:
- **0434:** the armed Prism's A Attack is live, and greyed or absent while it isn't armed.
- **Perf:** if the chain adds per-tick work, measure `movement_perf` against master's binary (`target/perf/<hash>`).
- **Golden** `syndicate_relay`. No other golden should move.
- **The computer players' games** are unchanged: none builds an Orbital Command.
- **QA screenshots:** the highlights over a chain; the beams from two Batteries through the chain; the armed Prism's panel.
- **Checks:** m5-common's "Done means".

## What you own
`m6-plan.md > SR`'s Owns list, and the Prism's A in its panel template.

Read-only: `battery.rs` (SL's) beyond the relay's discharge, the Prism's def and Thrust (SP's), `combat/structures.rs`, and every other slice's files (`m6-plan.md > §C`).

## Landed since the plan (filled in at launch)
TBD.

## Report
At most 12 lines:
- Relay: the highlights, the chain, the lock;
- the armed Prism: its picks, its sight, directing it, its A;
- the summed surge, Amplify while relaying, and a broken chain;
- smoke and the relay (if SC has merged);
- the golden, perf, and the computer players' games;
- placeholders for the Director to file.
