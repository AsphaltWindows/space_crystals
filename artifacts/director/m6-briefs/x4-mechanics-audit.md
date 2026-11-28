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
- **Master is about to get the Dragonfly's landing approach** (`crates/sim/src/movement/approach.rs`). If that file is missing when you reach aircraft, read it with `git show worktree-agent-af1e93d23c0779a00:crates/sim/src/movement/approach.rs`.

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
3. **The same mechanic built twice:** for example two landing paths, two deploy models, two heal or aura implementations, two stock or ammo systems, or two "hold units inside" models (buildings' holding, the Tunnel Network, the carrier).
4. **Abilities:** for each `Ability` variant, whether any unit could carry it through its def, or whether it is tied to one unit.

## How to work
- **Keep your context lean:** aim to stay under about 150k tokens.
  - Use `grep -n`, `grep -c` and `wc` to find things, then read only the lines you need (`sed -n 'A,Bp'`). Most modules start with a `//!` doc that says what they do: read those first.
  - Never read a whole large file, `ARCHITECTURE.md` (712 KB), or a log.
- Don't build, test or edit anything, and don't write files outside the scratchpad.
- Keep to what the code does now. Where you're unsure whether something is per-unit, say so rather than guessing.

## Report
At most 80 lines:
1. **Summary** (5 lines): how much of the sim is per-unit and how much data-driven, with numbers.
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
