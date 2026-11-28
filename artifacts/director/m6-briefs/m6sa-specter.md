# Task M6-SA: The Specter

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** `artifacts/director/m6-plan.md > §B > SA: The Specter`, with T88, T96, T98 and T100 (`> §D`).

Read them before you start. **This slice has a Fable flag:** if the door approach won't settle after two serious attempts, stop and report, and the Director decides.

## Goal
- **The Aerospace Assembly's W trains the Specter,** once its owner has a Vault and a Chemical Plant.
- **A glider bomber** that lifts off and lands vertically through a Tier 3 Tunnel's air doors, 24 frames each, an air target meanwhile.
- **One bomb:** 400 damage over an 80 su circle, with a fixed 24-frame fall. It can Attack Ground.
- **Empty,** it flies to the nearest own Tier 3 Tunnel and lands. Inside, it re-arms in 160 frames given a Vault.
- **0272's attack-order rules** decide what it does with its queue before and after a bomb.

## Running beside you
Filled in at launch.

## The design: answers since the plan
Read `factions/syndicate/units/specter.md` as it stands, all of it, and `factions/syndicate/structures/vault.md`.
- **0431 (Q0165): the doors' timing.** A Phantom descends and rises over 16 frames, an air target meanwhile, "as the Specter (24 frames) and every aircraft landing or taking off (0428)". The Specter's 24 frames are in its doc.
- **0428 (Q0162) part 5: the Dragonfly lands "as the Specter at an air door":** it approaches at IdleSpeed, stops over its spot while it descends, and lifts straight up, leaving at IdleSpeed. M6-GL built that for the Dragonfly. Give the Specter the same approach to the entry door, citing 0428 part 5. This was D46's placeholder.
- **0424 (Q0158): network services run side by side.** A Specter's Vault turn, its Null Chamber turn and any repair run at once, in separate queues: "a Specter is ready in 10 s". The EjectMenu still takes the unit inside longest.
- **0405 (Q0148): its SeparationRadius is 64 su,** and gliders push at a third of the HoverCraft's strength (0412). Check the def M6-SK registered.
- **0458 (the Hornet's burst)** and **0445 (the Rocketman's)** are the game's other bursts at a spot. The Specter's bomb is area damage too: reuse the area code (`combat/area.rs`).
- **Anything the docs don't settle:** a placeholder, cited as m6-common says, and a line in your report. The Director files the question.

## Code
Follow `m6-plan.md > SA`'s Code list. Since the plan:
- **M6-SD built the air doors** (`UnitDef::air_door_ticks`, `Mover::rise`, `Enter::descent`, the air lane `TunnelState::air_in`/`air_out`). The Specter uses them at 24 frames. Read SD's Built entry in `artifacts/director/plan.md`.
- **M6-GL built the glider's vertical landing** (the approach at IdleSpeed, the stop over the spot, the lift), overriding the glider's "must keep moving" rule while it lands. Reuse it at the door. Read GL's Built entry and its notes for you.
  - **GL's notes for you:** call `Mover::approach_to(door)` and start the descent once `is_over()`. Phase 5d already holds a Glider that is over its point or rising (the one override of "must keep moving", invariant G1 amended), and `approach::stale` releases it if its order ends. SD's `Mover::rise` is the lift.
  - **Q0192 is open** on the approach (the 4 GU slow-down, lining up, circling when no spot is free). Build as GL did.
- **M6-GH1 and GH2 built ammo and rearming** (`UnitDef::ammo`, `Object::aircraft`). Use them for the single bomb; re-arming here is the Vault's service, not an Airfield's.
- **M6-SB built the Vault's service** in `network/services.rs` (2 at a time per Vault). The Specter's re-arm is a Vault turn: 160 frames.
- **The bomb:** DoublyDisjointed with a fixed fall (T100), `UnitDef::bomb_fall`. It lands 24 ticks after release, where it was aimed.
  - New projectile state hashes inside the projectiles' hash (`Sim` tag 2). A new variant must leave the existing ones' bytes unchanged, so no golden moves. If it needs `Sim` state of its own, use **`Sim` tag 6**.
- **0272's rules,** in new `orders/specter.rs`. A new `OrderKind`, if any, hashes as **tag 41** in `orders/hash.rs`.
- **New `Object` state,** if any, hashes and saves only while present, under **Object tag 33**.
- **The Aerospace Assembly's W:** turn its `Planned` entry into its `Kind`, behind the Vault and the Chemical Plant.
- **The ruleset (X1):** the new def fields are structure, and the numbers are def numbers, by m6-common's checklist.
- **The controls doc:** regenerate it.

## Acceptance
`m6-plan.md > SA`'s Acceptance, plus:
- **The approach:** it slows to IdleSpeed over the entry door, descends for 24 frames, and is an air target until it is in (0431, 0428 part 5).
- **0424:** in a network with a Vault and a Null Chamber, a Specter's re-arm and its Null Chamber turn run at once.
- **Without a Vault:** it flies home and waits inside, unarmed. With no Tier 3 Tunnel, it circles, and ignores Attack orders while unarmed.
- **Golden** `syndicate_specter`. No other golden should move. If `syndicate_air` moves, find out why.
- **The computer players' games** are unchanged: none builds an Aerospace Assembly.
- **Perf:** `air_perf` against master's binary (`target/perf/<hash>`).
- **QA screenshots:** the Aerospace Assembly's W live; a Specter descending into a door; a bomb's circle on the ground as it falls.
- **Checks:** m5-common's "Done means".

## What you own
`m6-plan.md > SA`'s Owns list.

Read-only: `landing.rs` (GH2's and GL's) beyond the door's arm, the air lane (SD's), `network/services.rs` (SN's and SB's) beyond the Specter's turn, and every other slice's files (`m6-plan.md > §C`).

## Landed since the plan (filled in at launch)
TBD.

## Report
At most 12 lines:
- the Specter's production;
- the approach, the door landing and the lift-off;
- the bomb: the fall, the miss, Attack Ground;
- 0272's rules, re-arming, and no Vault or no Tunnel;
- the golden, perf, and the computer players' games;
- placeholders for the Director to file.
