# Task SLc: The Laser Battery, its panel, drawing, golden and QA (phase 3 of M6-SL, the entity phase)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is an entity phase, and your budget is 130 tool calls, a hard limit.** The mechanics are built: you assemble the unit from them.

## Where things stand
- **The defs exist,** both kinds, with the designer's placeholder numbers: `crates/sim/src/defs/syndicate/units/laser_battery.rs`. Their comments say "nothing makes one yet": make them true.
- **The Vehicle Assembly's E** is written but unused: `LASER_BATTERY_RECIPE` in `crates/sim/src/defs/syndicate/structures/vehicle_assembly.rs`, marked `expect(dead_code, reason = "slice M6-SL: ...")`.
- **`SK_KINDS`** (`crates/sim/src/defs/syndicate/tests.rs`, near 131) lists the kinds no player makes yet, the Battery's two among them.
- **The deploy model** (M6-GA): `crates/sim/src/vehicle_deploy.rs` and `orders/vehicle_deploy.rs`. A unit's two types and its switch times are data on its def; the Artillery Rig is the built example.
- **SLa** (`d5e7231e`) built the battery and the surge attack, as sim mechanics. From its handoff:
  - **The battery** is each kind's `EnergyDef`: 100, starting full, charging 5 a second. `energy::step` charges it in every mode, through the change of kind and inside the network. Read it with `sc_sim::energy::of(o)`: (charge, max) in points.
  - **The surge:** `SurgeDef { draw, constant }`, as `AttackAttrs::surge` (`defs/units/attack.rs`), and `crate::surge` (`surge.rs`): `has`, `used`, `damage`.
  - **The cadence:** one aim of 8 frames when it engages, then a surge every 16 frames while the beam holds (`Shooter::beam_on` in `combat/phases.rs`; Q0201 asks the Designer, and three placeholders wait on it).
  - **Amplify:** `Command::Ability(Ability::Amplify)`. It isn't queueable and keeps the orders. The gate refuses it to a unit without a surge (`Reject::LacksAbility`). The mark is `Object::amplified`, cleared by the next surge.
  - **A surge's hit** lands in `combat/phases.rs` `end()` (Firing, then `effects::fire` and `Shooter::spend_surge`). Its charge is spent in `combat/step.rs` (`surge::spend`).
  - **Left for you:** the scenario tests, the surge consistency check and the docs (Goal 6 and the Tests).
- **SLb** fixed the deploy model's queued switch. Its handoff: FILLED IN AT LAUNCH.

## The design
- `factions/syndicate/units/laser_battery.md`, all but `> Relay` (slice SR's). Its numbers are in the **Placeholder stats** paragraph at the end.
- `systems/mechanics/modes.md > Archetype: Deploy (an order)`.
- **The panel:** the Battery's doc gives only "**Amplify** (hotkey **F**)", and says its modes work like the Artillery Rig's. The Rig's panel is "BasicCombatUnitInterfaceState plus **R: Deploy** (mobile) / **R: Undeploy** (deployed)" (`factions/gdo/units/artillery_rig.md`, line 37). So the Battery's is the basic combat panel, R Deploy or Undeploy, and F Amplify in both modes. Cite both docs.
- `factions/syndicate/structures/vehicle_assembly.md`: **E** Laser Battery, greyed until its prerequisite, an Optics Lab.

## Goal
1. **The Vehicle Assembly trains it:** E goes live, behind the Optics Lab, for 200 SC and 400 frames.
2. **Its two kinds, live:**
   - the deploy pair on GA's model, 48 frames each way;
   - mobile: Ground targets only, a 30° turret;
   - deployed: immobile, Air targets only, a 360° turret;
   - drop the two kinds from `SK_KINDS`, and update the comments that say nothing makes one.
3. **The panel,** as above. Amplify is not queueable. The Relay's key is SR's: leave its slot free.
4. **Drawing:**
   - the beam: "while attacking, a **constant thin red laser** connects the Laser Battery to its target. Periodic **surges** along the beam" (`> Attack: constant beam with surges`). A simple placeholder look is fine.
   - the charge: the info panel already shows any energy pool as "Energy a / b". The design calls it the battery charge: label the Battery's line "Battery" if that is cheap, or leave it and say so.
5. **The golden `syndicate_lasers`,** made as the other Syndicate goldens are (`crates/sim/tests/golden/main.rs`): a Battery trained at a Vehicle Assembly with an Optics Lab fires at ground targets until its battery is empty, Amplifies, deploys and fires at an aircraft. No other golden may move.
6. **Docs:**
   - regenerate the controls doc: `UPDATE_DOCS=1 cargo test -p space_crystals --test game playtest_docs::controls`;
   - add the Battery to the docs that list units (`grep -rln 'Prism' docs` shows where).
   - the surge attack, a short paragraph in `docs/architecture/combat.md`, and Amplify in `docs/architecture/commands-events-replays.md` (SLa left them).
7. **QA screenshots** into `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/m6sl/`. Look at each yourself:
   - the Vehicle Assembly's panel with E live;
   - a mobile Battery firing at a ground target, with its charge in the info panel;
   - a deployed Battery's beam at an aircraft.

   The QA mode: `grep -n 'QA mode' ARCHITECTURE.md docs/architecture/*.md`, then read that section only. SP2c's recipe, `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/m6sp/cmds2.json`, is a worked example. Check that `cargo run` starts.

## Tests
In a new `crates/sim/tests/laser_battery/`, with the real unit:
- **the surges:** from a full battery, each deals 18.6 (to f32 precision), and the charge falls 5 a second while it fires. It is empty after about 20 s, and each surge then deals 11.7;
- **Amplify,** pressed between surges: the next surge deals 86 from full and empties the battery (0369's Verification);
- **domains:** mobile, it can't target a Sparrow; deployed, it targets only air;
- **deploying,** as GA's tests do for the Rig, held orders included, and the battery charges while it switches;
- **the queued switch** on this pair, as SLb's handoff says;
- **Amplify** neither fires nor drops the unit's orders, and the gate refuses it to a unit without a surge;
- **a consistency check** in `defs_are_consistent` for every def with a surge: a FullyConnected single-target attack, an energy pool, and 0 < draw ≤ the pool's maximum;
- client tests of the panel, Amplify included.

## Don't
- Don't change the surge attack or Amplify (SLa's), or the deploy model (GA's and SLb's). If they lack something, say so in your report.
- No Relay: that is slice SR's.
- Don't edit anything under `artifacts/`.

## Report
At most 10 lines, then **Handoff** for slice SR (the Relay, from a deployed Battery through the player's Prisms), at most 5 lines: where the Battery picks its target and fires, how the beam is drawn, and the panel slot left for Relay.
