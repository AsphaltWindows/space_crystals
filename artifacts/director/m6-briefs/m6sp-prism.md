# Task M6-SP: The Orbital Command and the Prism

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** `artifacts/director/m6-plan.md > §B > SP: The Orbital Command and the Prism`, with T96 (`> §D > T96`, the air doors you launch from).

Read both before you start. One design change since the plan replaces a placeholder (below).

## Goal
- **A Tier 3 Tunnel builds the Orbital Command,** once its owner has an Aerospace Assembly.
- **The Prism.** The Aerospace Assembly's E trains it, once its owner has an Orbital Command: an unmanned satellite, an air HoverCraft.
- **The launch.** It goes straight up out of the air exit door (32 frames), then unfolds (32 frames). It can be hit as an air unit throughout.
- **Moving.** It drifts at 1 su/frame on its drag numbers. It has three thrusts, each moving it 3 GU over 1 s toward a clicked point, and the charges never come back.
- **One way.** It can never go back into the network.
- **Sight.** It sees through smoke, once SC builds smoke.

## Running beside you
Filled in at launch.

## The design: answers since the plan
Read `factions/syndicate/units/prism.md` and `factions/syndicate/structures/orbital_command.md` as they stand.
- **0434 (Q0168): the panel, Thrust, and orders during the launch.** Read `artifacts/designer/changes/0434-prism-panel-thrust.md`.
  1. **The panel:** Q Move, R Thrust, X Stop. A Attack shows only while armed: that's a relay target (SR's), so the Prism shows no A in your slice.
  2. **Thrust is targeted:** click a point, and it thrusts the full 3 GU toward it, whatever the distance. **It queues with Shift, as every command does.** This **replaces** D47's "not queueable". Each queued thrust spends its charge when it starts. R greys once the charges are gone.
  3. **Orders given during the launch and the unfolding are held,** the last one winning, as in the vehicle deploy model. (D47, now the design.)
- **0456 (Q0181): its SeparationRadius is 48 su.** M6-SK registered it. S6b replaces the "Placeholder (Q0181)" marks; leave them.
- **0420 (Q0154): its DragMovement numbers** (ForwardAcceleration 0.05, Omni 0.05, DragRatio 0.1, TurnRate 4°/frame, top speed 1) are the doc's. M6-SK registered them.
- **"Manned - no (no Null Chamber buff)"** (`prism.md`), and `null_chamber.md`: "Manned units: all Syndicate units except satellites".
- **Anything the docs don't settle:** a placeholder, cited as m6-common says, and a line in your report. The Director files the question.

## Code
Follow `m6-plan.md > SP`'s Code list. Since the plan:
- **The air doors are built** (M6-SD): `Paths::air_exit`, the eject queue's air lane (`TunnelState::air_out`), and an aircraft's rise (`Mover::rise`, `UnitDef::air_door_ticks`).
  - SD's note for you: "Its 32-tick launch and 32-tick unfold are a second timed phase after the rise. `NoTransit::OneWay` must refuse only the way in, since `network::admits` also checks ejects."
  - The Prism's launch is its rise: `air_door_ticks` 32, then the unfold, 32 more. Reuse `Mover::rise`, and add the unfold after it. Don't build a second rise.
- **Held orders during the launch (0434, part 3).** M6-GA built the vehicle deploy model's held order: the last order given while a timed process runs is held and tried once it ends. Read GA's `vehicle_deploy.rs` and `orders/vehicle_deploy.rs`, and reuse its held-order slot if it fits. If it doesn't, say why in your report.
- **The thrusts:** `UnitDef::thrust` and `Object::thrust` (`thrust.rs`), a queued order or ability, by 0434 part 2. A thrust is a 16-tick displacement, 3 GU toward the point, on top of the drift.
  - **Hash and save `Object::thrust` only while present,** under its own Object tag. On master, tags 9 to 15 are taken (15 is CM's `illusion_of`), and 16 stays unused. **Use 17.**
  - **A new `OrderKind`** (Thrust) hashes as **tag 24** in `orders/hash.rs`. Master's highest is 21; the numbers between are unused or promised (`artifacts/director/hash-tags.md`).
- **"Unmanned":** the Null Chamber buffs manned units only. M6-SN merged with no `UnitDef::satellite`, so today every unit counts as manned.
  - Add `UnitDef::satellite` (false for every other unit), set it on the Prism, and make SN's buff skip satellites (`Sim::leave_network`).
  - Test that a Prism trained with a Null Chamber in the network leaves unbuffed.
- **The Aerospace Assembly's E** and **`EXPAND_MENU`'s Orbital Command:** turn their `Planned` entries into their `Kind`s (SD left E `Planned`).
- **The ruleset (X1):** the new def fields are structure, and the numbers (32 + 32 frames, 3 charges, 3 GU, 16 ticks) are def numbers, by m6-common's checklist.
- **The controls doc:** regenerate it.

## Acceptance
`m6-plan.md > SP`'s Acceptance, with 0434's Verification:
- an unarmed Prism shows Q, R and X, and no A;
- R and a click on a point 10 GU away: it thrusts 3 GU toward it;
- R and Shift-clicks on three points: three thrusts in a row, then R greys out;
- a Move given while it launches: it moves there once unfolded;
- a Null Chamber in the network doesn't buff it.

Also:
- **One way:** a right-click on an own Tunnel is refused ("can't re-enter", `NoTransit::OneWay`), and the Enter order too. The Prism still ejects by the air exit door.
- **No golden of your own** (SR's will cover the Prism). No golden should move.
- **The computer players' games are unchanged:** none builds an Orbital Command.
- **QA screenshots:** the ExpandMenu with the Orbital Command live; the Aerospace Assembly's E live; a Prism launching; one unfolded, with R greyed after three thrusts.
- **Checks:** m5-common's "Done means".

## What you own
`m6-plan.md > SP`'s Owns list, and the Prism's lines in `defs/syndicate/tests.rs` and `tests/syndicate_kinds.rs` (drop the Orbital Command and the Prism from `SK_KINDS`, as SV and SD did for theirs).

Read-only: SD's air lane (`network/paths.rs`, `sim/network/ejects.rs`), beyond the launch arm; `patches.rs`; GA's deploy model, which you reuse; and every other slice's files (`m6-plan.md > §C`).

## Landed since the plan (filled in at launch)
TBD.

## Report
At most 12 lines:
- the Orbital Command and the Prism's production;
- the launch, the unfold, and held orders;
- the thrusts, and Shift-queueing;
- one way, and "unmanned";
- the computer players' games, and any golden;
- notes for SR (the relay arms the Prism) and SC (it sees through smoke), and placeholders for the Director to file.
