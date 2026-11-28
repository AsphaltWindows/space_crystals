# Task X4: Audit the sim for per-unit mechanics (read-only)

## Why
The user wants units to be compositions of shared mechanics: "most unit additions should more or less be a compilation of existing mechanics into a new entity. If it requires a new behavior [...] that mechanic should be introduced first as a dependency independently." Until now, each new mechanic was built inside the slice of the first unit that needed it, and shaped for that unit: for example the carrier with the Subterranean Transport, and the glider landing approach with the Dragonfly. Before refactoring, we need to know how much of the code is like that. **Change nothing:** this task only reads and reports.

## The codebase
- **Layout:** a Rust workspace. `crates/sim` (`sc_sim`) is the deterministic sim, `crates/game` the Bevy client, and `crates/ai` the computer players. The sim is about 300k lines with its tests; 38% of the source's characters are comments.
- **Units and structures are defs** (`crates/sim/src/defs/`, one module per kind, listed by `kind_registry!` in `defs.rs`), made of parts:
  - a unit base: the movement model and domain;
  - weapons and attack types;
  - abilities (the `Ability` enum);
  - optional parts such as `UnitDef::carries`, `vehicle_deploy`, `domain`, tanks, stock and ammo.

  The systems read those parts.
- **The design** (`artifacts/designer/design/`, read-only) is one file per unit or structure under `factions/<faction>/{units,structures}/`, plus `systems/` and `core/`.
- **The newest mechanics,** each built inside its first unit's slice: the glider landing approach (`crates/sim/src/movement/approach.rs`, the Dragonfly), the carrier (`crates/sim/src/carry.rs`, the Subterranean Transport), the Supply Chopper's cargo and its landing on an object (`Cargo`, `Landing.at`), and the unfold after a launch (`Mover::unfold`, the Prism).
- **The design now defines shared mechanics once,** in `artifacts/designer/design/systems/mechanics/` (0471, answering Q0193). Each doc defines a mechanic and its **archetypes**, and the unit docs name the archetype they use, with their parameters:
  - `carrying.md`: the Ground carrier (the Subterranean Transport, built), the Landing carrier (the Supply Chopper, slice GS2), the Docked carrier and Paradrop (the Pilgrim, slice CP);
  - `modes.md` (0472): Deploy (the Artillery Rig, built; the Laser Battery, the Transport's Burrow and Surface, the Prospector) and Toggle (the Riot Shield and Weapon Conversion, built). The Drone's Beacon deploy is neither.
  - **Coming, one doc at a time:** aircraft landing and rearming with runways, garrisoning, channelling, energy pools, heals and auras, hidden deployed objects, tanks and toggles.
  The design asks for **one module per mechanic, with its archetypes as parameters**.

## What to find
1. **Mechanics keyed by a unit's kind, not by its def.** Find code that tests for a specific `ObjectKind` (`grep -rn 'ObjectKind::' crates/sim/src`). Leave out `defs/`, the tests, `scenario/`, `scenarios.rs` and `testing/`. Sort each cluster into:
   - legitimate, such as one faction's economy rule that the design ties to one building;
   - a behaviour that a def field should switch on instead.
2. **Mechanics shaped for one unit:**
   - a module, order or ability that reads one unit's numbers or assumes its shape;
   - one that would need changes before a second unit could use it.

   Check these against the users the design gives them, now and upcoming. The upcoming M6 users:
   - the Supply Chopper carries units (`factions/gdo/units/supply_chopper.md`);
   - the Pilgrim lands along a runway and paradrops (`factions/cults/units/pilgrim.md`);
   - the Laser Battery and the Prism relay (`factions/syndicate/units/{laser_battery,prism}.md`);
   - the Twister's mines (`factions/gdo/units/twister.md`);
   - burrowing (`factions/syndicate/units/subterranean_transport.md`);
   - the Shadebringer's smoke;
   - the Specter (`factions/syndicate/units/specter.md`);
   - the Prospector;
   - the Field Engineer's Bunker and First Aid Tent.
   - the Prism's thrust (`factions/syndicate/units/prism.md`).
3. **The same mechanic built twice:** for example two landing paths, two deploy models, two heal or aura implementations, two stock or ammo systems, or two "hold units inside" models (buildings' holding, the Tunnel Network, the carrier).
4. **Abilities:** for each `Ability` variant, whether any unit could carry it through its def, or whether it is tied to one unit.

5. **The mechanics docs against the code.** For `carrying.md` and `modes.md`, and for each coming mechanic above:
   - which modules implement it now;
   - whether it is one module or several, shaped per unit;
   - what refactor would make it one module, with the archetype and its parameters in the defs;
   - which upcoming slice needs that refactor first.
6. **Hub files.** Of 257 slice commits, these files were touched by the most (count, lines):
   - `sim/testing/invariants.rs` 37 (563); `sim/defs.rs` 33 (477); `game/command_ui/interface/templates.rs` 29 (440);
   - `templates/planned.rs` 29 (270); `sim/defs/units.rs` 26 (376); `sim/sim.rs` 25 (564); `sim/sim/phase.rs` 23 (362);
   - `game/command_ui/resolution.rs` 23 (532); `sim/orders/run.rs` 22 (363); `sim/events.rs` 21 (586); `game/command_ui/feedback.rs` 21 (600);
   - `sim/sim/gate/orders.rs` 20 (559); `interface/conditions.rs` 20 (594); `interface/commands.rs` 20 (332);
   - `sim/movement.rs` 19 (559); `sim/combat.rs` 19 (388).

   Agents read these files whole: a slice read about 40–60k characters from each, much of it comments. Say which of them a new order, ability or unit must edit, and why (a central `match`, a list, a registration). Propose how a new mechanic could register in one place instead, for example one module per order kind behind a trait or a table, with a size and a risk.

## How to work
- **Budget:** 200 tool calls. If you reach it, report what you have, and list what you didn't get to.
- **Keep your context lean:** aim to stay under about 150k tokens.
  - Use `grep -n`, `grep -c` and `wc` to find things, then read only the lines you need (`sed -n 'A,Bp'`). Most modules start with a `//!` doc that says what they do: read those first.
  - Never read a whole large file, `ARCHITECTURE.md` (712 KB), or a log.
- Don't build, test or edit anything, and don't write files outside the scratchpad.
- Keep to what the code does now. Where you're unsure whether something is per-unit, say so rather than guessing.

## Report
At most 110 lines:
1. **Summary** (5 lines): how much of the sim is per-unit and how much data-driven, with numbers.
1b. **Mechanics docs against the code** (item 5), a table: the mechanic, its archetypes, the modules now, the refactor, its size, the slice that needs it first.
1c. **Hubs** (item 6): what a new mechanic must edit today, and the proposal.
2. **Refactor candidates,** a table with these columns:
   - the mechanic;
   - where it lives (files);
   - what it is keyed by (kind, or def field);
   - its users now and upcoming;
   - the shape you propose;
   - size (S, M or L);
   - whether a pure refactor of it should leave every golden unchanged.
3. **Duplicates:** the same mechanic built twice.
4. **Recommended order:** which refactors to do before which upcoming M6 slice (the ones it will build on), and which can wait until after M6.
5. **Legitimate per-kind code,** briefly, so we don't refactor it.
