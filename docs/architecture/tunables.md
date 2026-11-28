# Tunables — detail

Read this when you add a tunable or need the ruleset, its generations or the data files in detail; the core has the short rule.

## Tunables

The numbers a playtest tunes live apart from the logic that uses them, in tuning modules grouped by system, each
one struct that a data file loads (the data files come at playtest, plan decision T54; slice M5-X2). A number that
belongs to one object goes in its def instead (*Adding a unit or structure*).

- **The ruleset** (`crates/sim/src/ruleset.rs`, plan decision T66, slice M5-X1): every number the sim reads, in one
  value, `Ruleset { defs, bases, factions, tuning }`: the object types' defs by kind, the unit bases' by base, the
  factions' own rules (`defs::FactionRules`: the caps, the Cults' augmentation table, techs and abilities) and the
  systems' tunables (`tuning::Tuning`). The design's numbers are the built-in ruleset (`Ruleset::BUILTIN`), made of
  the def modules' defs and the tuning modules' `BUILTIN`s. The sim reads it through `rules()`: `kind.def()`,
  `base.def()`, `tech.def()` and `ability.def()` do, and a tunable reads `rules().tuning.<system>.<field>`.
  **A hot loop reads its tunables once**, before the loop (`let tuning = &rules().tuning.movement;`), and hands an
  inner helper what it needs (`elevation::sight_range_by`): each read is an atomic load and a branch, and a number the
  compiler can't see costs what a constant didn't (slice M5-X1 measured 2 to 5 % on the perf tests before hoisting).
- **Generations** (`ruleset.rs`'s module docs): a ruleset in use is a generation of the process, the built-in one
  generation 0. `DebugCommand::SetRules` (slice M5-X1b, `sim/rules.rs`) installs another, 64 at most, and the sim
  makes its own the current one as each tick starts (`Sim::rules`): a tick reads one generation, and a change applies
  as the tick after its command's starts, in command order, the last of a tick's winning (`SimEvent::RulesChanged`).
  The ruleset travels in the command, so a replay reproduces a tuned game. The state hash leaves the rules out. A
  process runs one sim at a time on rules other than the built-in ones; their tests are a program of their own,
  `tests/rules.rs`, each test holding a lock for its games, and every other test plays on the built-in rules.
  - **Only numbers change**: `Ruleset::structural_diff` (`ruleset/structure.rs`) finds a change of kinds,
    footprints and Silhouettes, variants, lists or flags, and the gate refuses it (`Reject::RulesStructural`); the
    TechPrerequisites, a building's default rally point and a blast's BlastDomain are read only as something starts,
    so they aren't structure (`structure::FREE`).
  - **Objects keep their state**: HP and Shields follow their new maxima with the damage kept
    (`transform::follow_max`, never a kill), energy stays within its MaxEnergy, and the rest reads the new numbers as
    it goes (a refund at the new price) or keeps what it started with (a projectile's damage, a planned path).
  - **Saves** keep the game's rules (`rules`) and those its next tick starts on (`next_rules`) as the rules
    themselves, each left out while it is the built-in rules or none, so the saves of games on the built-in rules are
    as they were; a save whose rules have another structure than the build's doesn't load.
- **The sim's tunables** (`crates/sim/src/tuning/`, slices M4-R1 and M5-X1): one module per system, `air.rs`,
  `combat.rs`, `construction.rs`, `gathering.rs`, `holding.rs`, `movement.rs`, `orders.rs`, `patches.rs` (slice
  M6-SB), `paths.rs`,
  `production.rs`, `repair.rs`, `shields.rs`, `spells.rs`, `tunnel.rs` and `vision.rs`, each one struct
  (`tunables!`: the struct, its serde form and its `BUILTIN`) whose fields are named as its constants were
  (`Movement::jammed_ticks`). A faction's own rules are a struct in its `defs/<faction>.rs` (`GdoRules`,
  `SyndicateRules`, `CultsRules`, `ColonistsRules`). **A new tunable is a field** of its system's struct, with its
  doc and its built-in value, never a `const` the sim reads. A `const` context (an array's length, a `const`
  assertion, a test's constant) reads the struct's `BUILTIN` (`tuning::movement::BUILTIN.give_up_ticks()`).
- **The client's feel** (`crates/game/src/tuning/`, slice M4-R2): `camera.rs` (the view's width, the pan speed, the
  edge-pan margin, the editor's zoom), `input.rs` (the drag threshold, double-click and double-tap times, the
  targeting click allowance, the pick slop, the wheel's pixels per line, the editor's marker grab radius) and
  `feedback.rs` (how long an order's flash, a refused command's message, a tracer, a death marker, a blast, a cast and
  a HUD notice show; slice M5-MM2: the minimap's pings, how long one shows and the throttle's time and area).
  Slice M5-X2 made each module one struct, declared by the client's own `tunables!` (`CameraTuning`, `InputTuning`,
  `FeedbackTuning`), and added `look.rs` (`LookTuning`: the Shroud's haze and flashes, (M6-SB) the sludge's fade, the
  bars over the units, the
  air units' lift over their shadows). Together they are the `ClientTuning` resource, which the systems read
  (`Res<ClientTuning>`, every plugin that reads it initializing it) and the data files replace while a game runs. A
  function takes the module it reads (`&LookTuning`), or the whole `&ClientTuning` when it is public; a test or a
  `const` context reads `ClientTuning::BUILTIN` or a module's `BUILTIN`. In a game the view's width applies as the
  zoom, the tuned width over the built-in one (`camera::follow_view_width`); the editor runs on the built-in feel.
- **The data files** (`crates/game/src/data/`, slice M5-X2, `docs/tuning.md`): `--export-data <dir>` writes the
  ruleset and the client's feel as JSON files that mirror their modules (`data/layout.rs`: `defs/<faction>/{units,
  structures}/<kind>.json`, or `defs/<faction>/<kind>.json` for a kind that is neither, the GDO's Mine;
  `defs/<faction>.json`, `defs/bases/<base>.json`, `tuning/<system>.json`, `client/<module>.json`), each
  pretty-printed in its fields' order. `--tuning <dir>` (a game's, with debug commands
  on) reads each file as an RFC 7386 merge patch over its counterpart (`data/patch.rs`) and refuses a directory whole,
  naming the file and the member: an unknown file or member, a value of the wrong kind, a number out of its sign,
  structure (`Ruleset::structural_diff`), a number not finite, or one `Ruleset::check` refuses (`data/load.rs`). The
  rules go to the game as a `SetRules` in the first tick's inputs; then the directory is polled every 0.5 s of real
  time by its files' modification times (`data/poll.rs`), and a change is read and sent again (`data/live.rs`). F8
  (`InputAction::DebugRestartWithData`) restarts the game as the playtest menu's Restart does, with the directory
  read again; a Restart gives the new game the last rules that passed. The top bar says "Tuned: N files"
  (`TopBarNote::Tuned`) and the feedback line a refusal (`command_ui::DataRefused`). The sim never reads a file, and
  neither do the goldens; tests: `data/tests.rs` (the patch, the layout, the refusals, the poll) and the test
  program `tests/tuning.rs` (games on tuned rules, holding a lock as `crates/sim/tests/rules.rs` does).
- **The computer players'** (`crates/ai/src/tuning.rs`, slice A1): `army` (the staging point, the waves, the
  defense), `build` (where a building goes) and one module per faction with a computer player (`gdo`: the opening,
  the Barracks, the queues; `syndicate`: the Agents, the Tunnels and where they add income, the runners, the
  expansions, the waves; `cults`: the field's seats, the sites and crews of its buildings, the Halls, the Gunners, the
  bank that stops the crew growing; `colonists`: the Warp Ins, the Drones, Beacon Capacity's lead, the Beacon sites),
  each with its rush's (slice A2d). Each is "Placeholder (Q0147 point N)" (2: how it plays; 3: build orders) or
  "Placeholder (not in the design: why)": the behaviour is the Director's placeholder (*Computer players*).
- Each tunable and constant has a doc comment: a design citation, or "Placeholder (not in the design)" and, where
  there is one, why it has its value. `docs/playtest/placeholders.md` lists every such mark, the tunables' (`name =
  value`), the constants' and the rest of the code's (*Client architecture*, generated docs).
- Colours and drawing sizes stay with the drawing code. Technical constants stay where they are used: the sim's
  units of measure, hashing, file formats, and the caps and margins that only bound a computation or absorb rounding;
  the client's z offsets, frame budgets and capture timeouts.
- A slice adds its tunables at the end of its system's struct, and its constants at the end of its system's module.
