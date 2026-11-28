# Task SLa: An attack that spends from an energy pool, in surges, and Amplify (a mechanic, phase 1 of M6-SL)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a mechanic phase, in `crates/sim` only, and your budget is 100 tool calls, a hard limit.**

## Why
The Laser Battery's damage comes from its battery: each surge of its beam spends charge, and deals damage that grows with the charge spent to the power 2/3. The battery is the sim's generic energy pool (`crates/sim/src/energy.rs`): a maximum, a start, and regeneration every tick. What is new is an attack whose hits spend from that pool, and Amplify. Build both as mechanics, shaped by the def, so that any unit with an energy pool and a surge attack gets them. Nothing should test for the Laser Battery's kind.

Two later phases follow you: SLb fixes the deploy model's queued-switch bug, and SLc builds the Laser Battery as a unit (its rows, the Vehicle Assembly's E, the panel, the golden and QA).

## The design
- `factions/syndicate/units/laser_battery.md`: `> Battery`, `> Attack: constant beam with surges`, `> Amplify (ability)` and `> Placeholder stats` (the numbers).
- The rules, in short:
  - The battery is always charging, attacking or not, in every mode, while switching modes, and inside the Tunnel Network.
  - A normal surge draws a fixed charge (twice what charges in one surge interval) if the battery holds it, otherwise whatever is left. An empty battery keeps firing just as often, each surge using what charged since the last one.
  - The damage of a surge is a constant × (charge used)^(2/3). With the placeholder numbers (a constant of 4, a draw of 10, a battery of 100), a normal surge deals 18.6, a surge from 5 deals 11.7, and an Amplified full battery deals 86.
  - **Amplify** converts the entire remaining battery into the next surge. It is "usable mobile and deployed; not queueable" (`> Placeholder stats`), and marks the next surge; it doesn't fire.
- The change entry `artifacts/designer/changes/0369-*.md`: its Verification.

## The architecture
Read the core, then only these paragraphs, found by keyword:
- the energy pool: `grep -n -i 'energy' docs/architecture/object-model-state.md docs/architecture/combat.md`;
- attacks, their phases and cooldowns: `grep -n -i 'FullyConnected\|cooldown\|damage' docs/architecture/combat.md`;
- abilities and their gate: `grep -n -i 'Ability\|Reject' docs/architecture/commands-events-replays.md`.

## Goal
1. **The battery is the unit's `EnergyDef`.** The Laser Battery's defs (`crates/sim/src/defs/syndicate/units/laser_battery.rs`, both kinds) get one, with the placeholder numbers and their citations. Check that `energy::step` regenerates it:
   - while the unit attacks;
   - through the change of kind between the two modes, keeping the charge it had;
   - inside the Tunnel Network.
   
   Where it doesn't, make it do so by a field of the def, not by changing the Disciple's rule, and say so in your report.
2. **The surge,** a part of the attack's def, for example a `SurgeDef { draw, constant }`. The attack's own cooldown is the surge interval.
   - Each hit spends `min(draw, pool)` from the pool, or the whole pool if Amplify marked it, and deals `constant × used^(2/3)`.
   - Use a deterministic power, not `f32::powf`, if the sim's rules forbid platform maths: see how the sim computes its other fractional powers (`grep -rn 'cbrt\|powf\|fn pow' crates/sim/src`). The tests must check 18.6, 11.7 and 86 "to f32 precision" on every platform.
   - The beam is FullyConnected and can't miss; the existing attack phases handle that.
3. **Amplify:** a new ability, given by the surge part (any unit with a surge has it).
   - It marks the next surge to spend the whole pool. It is not queueable, and usable in both modes.
   - The gate refuses it to a unit without a surge (`Reject::LacksAbility`).
4. **The hash:**
   - Any new state on the `Object` (the Amplify mark) hashes as **Object tag 23**, only while set.
   - A new `OrderKind`, if Amplify needs one, hashes as **tag 29**.
   - Abilities may not need an `OrderKind`: follow how other unqueued abilities are given.
5. **Tests,** at unit level and in `crates/sim/tests/`:
   - the three damages: 18.6 from 10, 11.7 from 5, and 86 from an Amplified full battery;
   - an empty battery fires just as often, each surge spending what charged since the last;
   - the battery charges while attacking, while switching modes, and inside the network;
   - Amplify marks the next surge only, isn't queueable, and is refused without a surge;
   - 0369's Verification;
   - the X1/X2 tests for the new def fields.
6. **Goldens:** none should move, unless a golden plays a Laser Battery. If one does, say which and why.

## Don't
- No client work: the panel, the battery bar and the beam's drawing belong to SLc.
- Don't change the deploy model (`vehicle_deploy.rs`, `orders/vehicle_deploy.rs`): that is SLb's.
- Don't make the Vehicle Assembly train the Battery, or change its domains or turret arcs: SLc's.
- Don't edit anything under `artifacts/`.

## Report
At most 8 lines, then **Handoff** for SLb and SLc, at most 5 lines:
- the surge part and Amplify (types, files, the ability's name and its `Reject`s);
- where the battery's charge is read, for a battery bar;
- where a surge's hit lands, for SC's rule (0433: a Battery firing from smoke is revealed for 1.5 s after each hit);
- the hash tags used.
