# Tuning with data files

The game's numbers are written in Rust. The object types' defs are in `crates/sim/src/defs/`, the systems' tunables
in `crates/sim/src/tuning/`, and the client's feel in `crates/game/src/tuning/`. These are the committed rules, with
their design citations and tests.

Data files (slice M5-X2) are a layer over them for playtests: you change a number while the game runs and try it.
When a value should stay, it is baked into the Rust default (*Baking a value*). The tests and the golden replays
never read a data file.

## Export

```sh
cargo run -- --export-data tuning
```

This writes the built-in data into `tuning/` as JSON files, then exits. Each file is pretty-printed with its fields in
the Rust struct's order, so it reads like its module.

| File | Holds | Rust source |
|---|---|---|
| `defs/<faction>/units/<unit>.json` | a unit's def | `crates/sim/src/defs/<faction>/units/<unit>.rs` |
| `defs/<faction>/structures/<building>.json` | a building's def | `crates/sim/src/defs/<faction>/structures/<building>.rs` |
| `defs/map/<object>.json` | a def of no faction: the Crystal Patch, the Supply Delivery Station | `crates/sim/src/defs/map/<object>.rs` |
| `defs/<faction>.json` | a faction's own rules: the caps; the Cults' augmentations, techs and abilities | `crates/sim/src/defs/<faction>.rs` |
| `defs/bases/<base>.json` | a unit base: `light_infantry`, `mech`, `glider`, ... | `crates/sim/src/defs/units/bases.rs` |
| `tuning/<system>.json` | a system's tunables: `combat`, `movement`, `production`, ... | `crates/sim/src/tuning/<system>.rs` |
| `client/<module>.json` | the client's feel: `camera`, `feedback`, `input`, `look` | `crates/game/src/tuning/<module>.rs` |

File names are the kinds in snake case: the Peacekeeper's def is `defs/gdo/units/peacekeeper.json`. A form that a
kind turns into shares its kind's module, but has a file of its own beside it:

- `peacekeeper_shielded.json` (from `peacekeeper.rs`);
- `laser_battery_deployed.json` (from `laser_battery.rs`);
- `subterranean_transport_burrowed.json` (from `subterranean_transport.rs`).

## Edit

Each `.json` file in the directory is a **merge patch** (RFC 7386) over its built-in counterpart:

- **Objects merge** member by member. `{"unit": {"attack": {"damage": 100}}}` in `defs/gdo/units/peacekeeper.json`
  changes the Peacekeeper's Damage and nothing else.
- **Anything else replaces the whole value.** A list is given whole: a patch of a building's `recipes` lists every
  recipe.
- **`null`** sets an optional member to none. RFC 7386 would remove the member instead.

So you can copy in only the files you change and keep only the members you change. You can also keep the whole
export: an unchanged export changes nothing.

Files that aren't `.json` are skipped, and so are names that start with `.` or `#` or end with `~`: an editor's
backups and locks.

## Play and reload

```sh
cargo run -- --factions gdo,cults --map maps/playtest.json --tuning tuning
```

- **At the start.** The directory is read before the first tick. Its rules go to the game as a debug command in the
  first tick's inputs, so the game plays by them from the second tick on.
- **While the game runs.** Save a file, and within half a second the game reads the directory again, also while it
  is paused. Changed rules apply from the next tick on. The client's feel (`client/*.json`) applies at once.
- **Objects keep their state.** HP and Shields follow their new maxima but keep the damage taken, and energy stays
  within its MaxEnergy. Work under way reads the new numbers as it goes: a production's remaining time, a refund at
  the new price. What was fixed when it started stays: a projectile's damage, a planned path.
- **The top bar** says "Tuned: N files" while N files change something. A file that matches the built-in data isn't
  counted.
- **F10's Restart** starts the game over on the last data that passed. **F8** restarts the same way, but reads the
  directory again first. The new game gets its rules in its first tick.
- **Replays.** The rules travel in the game's commands, so the recording of a tuned game plays back with no directory
  (`--replay`).

`--tuning` is a dev tool. It needs debug commands, which are on by default, because the rules travel as a debug
command and a real match (`--no-debug`) rejects those. It is refused with `--no-debug`, `--editor` and `--replay`.
Debug commands are its only gate: no check on the build (`cfg(debug_assertions)`), so a `--release` build takes it
too.

A run takes at most 63 sets of rules besides the built-in ones. Each set counts once, so going back to an earlier
value takes no new one. After that, a change is refused until you quit and launch the game again.

## Refusals

A refused directory is refused whole: the game keeps the data it had. The line above the HUD says why, naming the
file and the member, and so does the log. Fix the file and save it again.

| Message | Meaning |
|---|---|
| `defs/gdo/units/peacekeepr.json: unknown file` | no part of the data has that path: compare it with the export |
| `defs/...: can't be read: ...`, `not JSON: ...`, `expected an object` | the file itself |
| `defs/gdo/units/peacekeeper.json: unit.attack.damag: unknown field` | no such member at that place |
| `tuning/combat.json: minimum_damage: expected a number` | the wrong kind of value; also `a text`, `true or false`, `a list`, `an object`, `a whole number` |
| `...: unit.movement.TurnRate.max_speed: must be positive` | a number keeps the sign of its built-in value (below) |
| `...: structure.production.recipes[0].frames: must be at most 65535 ticks` | one of the defs' bounds |
| `...: must be a finite number` | a number too large for its field |
| `defs/gdo/structures/barracks.json: footprint.Tiles.w: structural, which needs a change to the Rust default and a rebuild` | a structural change (below) |

**The sign rule.** A number that the built-in data have positive stays positive, so a speed, a cost, a time or a
radius can't be 0 or negative. A number at 0 may grow but not go negative: an armour, or an immobile kind's speed. A
negative number stays negative. The client's feel follows the same rule.

**The bounds.** A recipe or a technology takes at most 65535 ticks. A Recruitment Area covers its building and spans
at most 255 tiles a side. An air unit's SeparationRadius is longer than its Silhouette. A default rally point lies
outside its building.

## Structural fields

A running game is built on some of the data. Those fields can't change while it runs (`Ruleset::structural_diff`,
`crates/sim/src/ruleset/structure.rs`):

- **Numbers aren't structural,** with one exception: a footprint's and a Silhouette's sizes (`w`, `h`), which the nav
  grids and the space every object takes are built on.
- **Names aren't structural:** the texts only label.
- **Everything else is structural:**
  - an enum's variant: a kind, a side, a domain, a movement model, an attack's type;
  - a flag (`true` or `false`);
  - whether an optional part is there (`null` or not): `energy`, `abilities`, a turret, a repairer's `repairs`;
  - what a list holds and how long it is: the recipes' kinds, a building's `accepts`, a research building's techs, a
    Tunnel's tiers.
- **A few fields are free anyway,** because the game reads them only as something starts, so a change takes what
  starts after it:
  - the `requires` of a recipe, a building and a tech (`requires_structures`);
  - a building's `default_rally`;
  - an area attack's `blast`.

A structural change is refused while the game runs and after F8 alike. Making one means changing the Rust default
and rebuilding.

Not available: rules of a scenario's own (`Scenario::rules`), which would let a new game start on a changed structure.
They are a possible follow-up, if playtesting asks to tune footprints or flags.

## The client's feel

| File | Holds |
|---|---|
| `client/camera.json` | the view's width at zoom 1 (`view_width_gu`, in a game), the pan speed, the edge-pan margin, the editor's zoom limits and step |
| `client/input.json` | the drag threshold, double-click and double-tap times, the double-click distance, the targeting click allowance, the pick slop, the marker grab radius, the wheel's pixels per line |
| `client/feedback.json` | how long things show: an order's flash, a refused command's message, a tracer, a death marker, a blast, a cast, a HUD notice, a data directory's refusal; the minimap's pings and their throttle |
| `client/look.json` | the Shroud's haze and a release's flash, the bars over the units (height, gap, narrowest width), the air units' lift over their shadows and where a takeoff leaves the runway |

The map editor and its play-tests always use the built-in feel, since `--tuning` is for games only.

The rest stays in code:

- colours, and the drawing sizes no playtest has asked to tune (rings, markers, fonts, the HUD's layout);
- technical constants: the z order, the Shroud's texels per tile, frame budgets, capture timeouts.

## Baking a value

When a tuned value should become the game's:

1. **Tell the Designer** the file, the member and the value, and why. The Designer records it in the design docs
   and publishes a change entry.
2. **The Director bakes it in.** The value goes into the Rust default the file mirrors, with the change entry cited
   in its doc comment. The Director then re-blesses the golden replays, giving the data diff as the reason.
3. **Drop it from your directory.** The file now matches the built-in data, and the top bar no longer counts it.
