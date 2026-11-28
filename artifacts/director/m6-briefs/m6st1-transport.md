# Task M6-ST1: The Subterranean Transport, above ground

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** `artifacts/director/m6-plan.md > §B > ST1: The Subterranean Transport, above ground`, with T94 (`> §D > T94`).

Read both before you start. Two design changes since the plan add to it (below). Burrowing and surfacing are ST2's, later.

## Goal
- **The Vehicle Assembly trains the Transport** (R), once its owner has a Forge: a DrillUnit that drives as a tracked vehicle above ground, and carries 4 infantry.
- **A ground unit above ground.** Its base's domain is Underground, so its surfaced kind needs its own: Ground.
- **Loading.** The owner's infantry board it by right-click. **Unloading.** F unloads them all around it.
- **Into a Tunnel.** It can drive into a Tier 2+ Tunnel loaded, and its passengers stay aboard.
- **Its death.** Its passengers die with it.
- **It crushes** like a tracked vehicle (0419).

## Running beside you
- **M6-S6a, an answer sweep:** a braking crusher crushes (0453); the MinRange back-off and repositioning to aim (0454, 0460); the client's Attack on a target some weapons can't hit (0461); the client's QA mode. It owns the attack behaviours (`orders/engage.rs`, the attack orders, `orders/attack_ground/`). It changes `crush.rs` for braking crushers (0453); your crushing is in the def (your brief).
- **M6-SB, the Vault and the Sludge:** the Vault's turn in `network/services.rs`, the Shadebringer's tanks and toggles, and the sludge patches (`Sim` tag 4). It adds the Vault's pass, and 0463 (a finished turn always buffs), to `network/services.rs`: read it, don't change it.

The lines you all share: the controls doc; the golden list and its counts; `ARCHITECTURE.md`. Whoever merges later rebases them, and checks the counts by hand (m6-common).

## The design: answers since the plan
Read `factions/syndicate/units/subterranean_transport.md` as it stands.
- **0432 (Q0166):** read `artifacts/designer/changes/0432-transport-passengers-and-enter.md`. Its parts 1 and 2 are yours; part 3 (an Enter given while burrowed surfaces it first) is ST2's.
  1. **Passengers don't fire out.** (D45, now the design.)
  2. **Passengers aren't in the network:** no regeneration, no Null Chamber turn, and they come out unbuffed. The Transport itself is in the network: it regenerates, and it takes a Null Chamber turn (it is manned). (D45's assumption, now confirmed.)
- **0419 (Q0153 part 4): the surfaced Transport crushes** like a TrackedVehicle, including the spin rule (0418). Underground it never crushes (ST2's).
  - V4's crushing (`movement/crush.rs`, `UnitBaseDef::crushes`) decides by the base, and the DrillUnit base doesn't crush. Make the surfaced kind crush, by its def, without changing the DrillUnit base for the burrowed kind.
  - M6-S6a (running beside you) changes `crush.rs` for braking crushers (0453). Keep your change in the def and in how `crush.rs` looks up whether a type crushes, not in its rule. Whoever merges later rebases.
- **0417: the tracked turning law** applies to the surfaced Transport. It drives "as tracked", on V3's tread law.
- **Anything the docs don't settle:** a placeholder, cited as m6-common says, and a line in your report. The Director files the question.

## Code
Follow `m6-plan.md > ST1`'s Code list. Since the plan:
- **The domain accessor.** M6-GH2 built it as two functions in `combat/targeting.rs`. Your `UnitDef::domain: Option<Domain>` is the per-kind override T94 names.
  - `object_domain(o)` is Ground for an aircraft down on a slot, and otherwise `kind_domain(o.kind)`. Every targeting reader asks it: the scans, ValidTarget, the blasts, the gate's `CannotHit` (`Sim::known_domain`, which takes a remembered object's recorded type), and the right-click.
  - `kind_domain(kind)` is the type's domain: a unit's base domain, and Ground for a structure. **Put your override there:** the kind's `UnitDef::domain` first, then its base's. Every targeting reader then sees the surfaced Transport as Ground with no change of its own.
  - T94's other readers go by the type, not the object: the collision layers, hiding, and the mines' trigger (M6-GM's, once it merges). Find each one, and make it read the kind's domain where it now reads the base's. A landed aircraft stays on the air layer for collision (GH2's note in `object_domain`'s docs).
  - M6-S6a (running beside you) is making the client's `unit_target_order` and the gate's `CannotHit` share one check, with its domain from GH2's functions. Your change is under `kind_domain`, so it reaches that check either way.
- **The carrier:** `UnitDef::carries` and `Place::Carrier` (`carry.rs`).
- **Boarding** goes through Enter on a carrier, with F1's `admits`. Only the owner's infantry board (D45).
  - M6-SD added an air arm to `orders/enter.rs`, and M6-SV a turned-box stand. Add the carrier arm beside them.
- **Unload All** (F): F3's evacuation spots, around the Transport's oriented shape.
- **Death:** passengers die on the removal path, and their Tunnel Space is freed.
- **The network:** a carrier is held with its passengers, and the passengers keep counting against Tunnel Space.
  - M6-SN's services (`network/services.rs`) serve the network's held units. Passengers are held in the carrier, not in the network, so they get no turn. Test it with a Null Chamber: the Transport comes out buffed, and its passengers don't (0432 part 2).
- **New `Object` state,** if any: hash and save it only while present, under **Object tag 18** (the numbers below it are taken, unused or promised (`artifacts/director/hash-tags.md`)).
- **A new `OrderKind`** (Unload All, say) hashes as **tag 25** in `orders/hash.rs`. Master's highest is 21; the numbers between are unused or promised (`artifacts/director/hash-tags.md`).
- **The menus:** the Vehicle Assembly's R goes live (turn its `Planned` entry into its `Kind`).
- **The ruleset (X1):** `UnitDef::carries` and `UnitDef::domain` are structure; the capacity of 4 is a def number.
- **The controls doc:** regenerate it.

## Acceptance
`m6-plan.md > ST1`'s Acceptance, plus:
- **0432's Verification,** parts 1 and 2:
  - a loaded Transport next to enemies: no shots from it;
  - wounded passengers ridden into the network for a minute come out still wounded and unbuffed, and the Transport itself healed.
- **0419's Verification,** above ground: a surfaced Transport driven through enemy Recruits kills them, and own infantry block it.
- **A ground unit:** a Peacekeeper can target the surfaced Transport, a Twister can't (GF's gate refuses the Attack: `Reject::CannotHit`), and it collides with ground units.
- **Golden** `syndicate_transport`. No other golden should move.
- **The computer players' games are unchanged:** none builds a Forge.
- **QA screenshots:** the Vehicle Assembly's R live; a loaded Transport with its passenger pips; an unload in a ring.
- **Checks:** m5-common's "Done means".

## What you own
`m6-plan.md > ST1`'s Owns list, and:
- the Transport's lines in `defs/syndicate/tests.rs` and `tests/syndicate_kinds.rs`;
- the kind arm of GH2's domain accessor.

Read-only: `holding.rs`, `network/services.rs` (SN's), `movement/crush.rs`'s rule (S6a's), GH2's landed arm, and every other slice's files (`m6-plan.md > §C`).

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
- **Your tags in the other namespaces** (`artifacts/director/hash-tags.md`): **Place tag 3** (`Place::Carrier`), and **Behaviour tag 16**, if boarding needs one.
- **Your carrier is reused twice:** M6-GS2 makes the Supply Chopper carry 4 infantry or 1 vehicle (0442), and M6-CP the Cults' Pilgrim 16 infantry, loading only while landed (0462). Shape `UnitDef::carries` and `Place::Carrier` so that a carrier that flies, and one with a vehicle place, need no second carrier. Put a note for each in your report.
- **Master's perf binary** for an A/B: `target/perf/800b81d5`, copied into your worktree with `target/`.

## Report
At most 12 lines:
- the domain override, and who reads it;
- boarding, unloading and death;
- the network: Tunnel Space, regeneration and the Null Chamber;
- crushing (0419);
- the golden, and the computer players' games;
- notes for ST2 (burrowing, 0432 part 3), and placeholders for the Director to file.
