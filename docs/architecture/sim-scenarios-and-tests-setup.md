# Sim scenarios and tests — Scenarios, maps and test helpers

Read this when you build a `Scenario`, a map file, a generated map, or need the sim test helpers and the test programs' layout.

## Sim scenarios and tests

- **A `Scenario` is a map plus a game setup** (`scenario.rs` and `scenario/`; the module docs have the details). The
  name stays because a scenario is still everything a game starts from (`Sim::try_new`, replays, the client, tests),
  but the two halves are separate:
  - The **map** (`MapDef`, `core/map_editor.md`) is what the editor edits and a map file holds: its type (`MapType`:
    Skirmish Default or Custom), name, description, terrain (`Map`), symmetry, player slots (`Slot`: a start
    position, an optional camera start, and a Starting Layout per faction) and pre-placed objects (`Spawn`s, neutral
    or owned by a slot).
  - The **game setup** (`GameSetup`) says who plays: the players (`PlayerSetup`: name, faction, slot), the RNG seed
    and the game options (`SimConfig`). `Scenario::new(map)` gives every slot a GDO player (`GameSetup::default_for`),
    as `--map` and `--scenario` do (`--factions` picks each slot's, `GameSetup::per_slot`); the editor's play-test
    panel builds its own.
  - `Sim::try_new` creates the map's objects first, in list order (so their ids ascend in it), leaving out those of
    slots no player takes, then each player's Starting Layout for its faction at its slot (`Scenario::start_objects`,
    `scenario/validate/start.rs`). A slot's objects belong to the player who takes it. A layout's Cults units start in
    the lineage of its first Recruitment Hall, an entry for each point of their Unit Control cost (slice M5-B;
    `factions/cults/economy.md > Starting Conditions`: "Six **Recruits**, all belonging to the starting Recruitment
    Hall"): `StartObject::lineage` holds the Hall's index among the start objects, which `try_new` turns into its id.
    A layout without a Hall (a Custom map's may) leaves them orphaned (design change 0344: "Recruits no Hall produced
    (custom maps, debug) are orphans from the start").
- `MapDef::validate()` and `Scenario::validate()` list every problem (`ScenarioProblem`) and never panic.
  `Sim::try_new` returns the problems as an error and `Sim::new` panics with them, so load files and editor maps with
  `try_new`. Units start on open ground and apart (`paths::misplaced`, in list order): a unit's box must not overlap
  blocked tiles, the map's edge or a blocking footprint (`SpawnProblem::Blocked`) or an earlier unit of its layer
  (`OverlapsUnit`), and a blocking footprint must not cover a ground unit (`CoversUnit`). Footprints keep apart
  (`OverlappingFootprints`, `LayoutOverlap`), but for a plate on an uncovered patch (see Object model), and a plate
  stands exactly on a patch (`NotOnPatch`, the placement check's `placement::Onto`).
  `MapDef::validate` checks the pre-placed objects, then every Starting Layout of every slot on its own
  (`scenario/validate/layouts.rs`; `core/map_editor.md > Starting Layouts > Layout validation`, design change 0097):
  among the map's objects, its structures by their faction's placement rules minus sight (`placement::check_layout`,
  `LayoutPlacement`; the GDO's need the area of the layout's own Deployment Center), its units by the unit-start rule
  (`InvalidLayoutObject`), and nothing else. Layouts of different factions may overlap. `Scenario::validate`
  (`scenario/validate/start.rs`) adds the layouts of the players of one game in each other's way, once the map has no
  problems of its own.
- **Starting Conditions** (`scenario/starting.rs`, slice M2-F1, plan decision T11): `StartingConditions::of(faction)`
  is a faction's table (`factions/gdo/economy.md > Starting Conditions`; the Colonists' since slice M3-C,
  `factions/colonists/economy.md > Starting Conditions`: an Egress Node, a Robo Portal and four Drones; the
  Syndicate's since slice M4-B, `factions/syndicate/tunnels.md > Tunnel > Starting Conditions`: a Tier 1 Tunnel, a
  Hideout inside its Tunnel Area and an Agent, 100 Space Crystals and 1 Supply, the camera on the Tunnel; the Cults'
  since slice M5-B, `factions/cults/economy.md > Starting Conditions`: a Recruitment Hall, a Storage and six Recruits
  of the Hall's lineage, 0 Space Crystals, the camera on the Hall): its starting objects with their counts, its
  starting banks, and its primary structure, which the camera starts on. Every faction has one, so `of` is total
  (plan decision T63): no faction is "not implemented", and a test that needs a faction without something uses
  `test_defs`. A game starts each player's banks from it (`Player::starting`). On a Skirmish Default map
  (`core/map_editor.md > Skirmish Default restriction`, design changes 0009 and 0012) a layout holds only its faction's
  Starting Conditions objects, no more of a kind than they do (`LayoutExtra`), and every faction's layout is complete
  at every start (`LayoutMissing`), so a new Skirmish Default map isn't playable until its layouts are placed. "Outdated" layouts (`MapDef::outdated_layouts`: holding
  objects, but not exactly the current Starting Conditions) are derived, never stored, so the map format stays at 3.
  When a faction gets its table, every Skirmish Default map in `maps/` needs its layouts
  (`the_maps_in_the_repository_are_playable` says which).
- **Map files** are a `MapDef`'s serde form, written with `serde_json::to_string_pretty`. A `version` field
  (`MAP_FORMAT_VERSION`, now 3) comes first; bump it on any incompatible change. The terrain is stored as rows of
  strings in the ASCII format below: `tiles` (H rows of W symbols) and `heights` (H + 1 rows of W + 1 corner digits),
  with the custom presets (and their looks) listed by symbol. Every spawn of a kind with per-object values stores them
  explicitly in `settings` (`SpawnSettings`, `core/map_editor.md > Per-object Resource Values`: a patch's starting
  amount, a station's starting supplies, delivery size and interval in ticks); `SpawnSettings::defaults` are the named
  placeholder values that new objects get. Reading rejects other versions, unknown fields, maps over the largest side,
  and a game setup: a map file holds a map only.
- **Map creation** (`MapSettings`, `scenario/create.rs`): type, size, slot count, symmetry, name and description, with
  placeholder limits (sides from `MIN_MAP_SIDE` 16 to `MAX_MAP_SIDE` 256, at most `MAX_SLOTS` 8 slots in whole sets of
  the symmetry's images, `MapSettings::slot_counts`). A Skirmish Default map needs a symmetry, and 90° rotation a
  square map. `MapSettings::check` names the first problem, and `create` makes a flat Plane map whose editable slots'
  starts are spread over the editable part, their images generated.
- **Generated symmetry** (`scenario/generate.rs`, `core/map_editor.md > Symmetry > Editing a symmetric map`): only
  the editable part of a symmetric map (`MapDef::region`) is edited, and `MapDef::regenerate` makes the rest its
  images: tiles, corner heights, spawns with their values, and the slots with their starts, camera starts and
  Starting Layouts. Placeholder rules for the design's TBDs: an object belongs to the part that holds its center; one
  that is its own image (a 2×2 station at the center of an even map) stays one object, and one that would overlap its
  own image is invalid; slots come in whole sets of images (slot `k·n + 1` is edited, slot `k·n + 1 + j` is its image
  `j`), and a slot's objects follow it. `first_asymmetry` finds where a map (a file edited by hand) differs from what
  its editable part generates; validation flags it (`ScenarioProblem::NotSymmetric`).
- Built-in scenarios live in `scenarios.rs`, behind `by_name(name)` and `NAMES`, each with its default setup. Keep
  the two in sync, because the client's `--scenario` flag uses them. Currently there are `dev_skirmish`, a 64×48
  Skirmish Default development map (Q0003) with 180° symmetry: starts in opposite corners, each with a GDO Starting
  Layout whose Deployment Center sees and reaches its 8-patch field (design change 0095) and two plates on it, and a
  Colonists one (slice M3-C, plan decision T24: the Egress Node on tile (4, 4) within the field's arc, the Robo
  Portal on (6, 6), the Drones at points clear of the node's sides and the portal's), and a Syndicate one (slice
  M4-B: the Tunnel on (6, 6) turned 180°, Side A south to the open map, Side D east to the middle, its crystal hatch
  (6, 7) 3 grid units from the nearest patches, (2, 6) and (2, 7), design changes 0248, 0255, 0259 and 0294; the
  Hideout on (10, 10) in its Tier 1 area; the Agent at (8, 11.5) grid units), gentle hills of slopes only,
  and a central plateau ringed by cliffs with a ramp up through a gap in the west and in the east
  (`maps/dev_skirmish.json` is the same map, which a test checks);
  `dev_structures`, a flat 30×12 Custom QA map with one of each GDO structure for each of two players (a plate on a
  patch in each base, slot 2's Barracks turned 90°); and `empty`, a flat 32×24 Custom map without symmetry.
  `scenarios::empty(w, h)` builds an empty scenario of any size, `empty_map(w, h)` its map.
- Unit tests sit in their module (`#[cfg(test)] mod tests`). Scenario and integration tests go in `crates/sim/tests/`
  (the client's in `crates/game/tests/`) and use only the public API.
- **Test programs** (slice M5-T): each crate's integration tests are modules of a few test programs, the `[[test]]`
  targets of its `Cargo.toml` (with `autotests = false`), so that an edit links one program rather than one for each
  file: the sim's `sim` (`tests/sim.rs`) and `perf` (`tests/perf.rs`, the `*_perf` timings, so that their release
  build compiles nothing else), the client's `game` (`tests/game.rs`); and the games on other rules than the built-in
  ones, in a process of their own, the sim's `rules` (`tests/rules.rs`, slice M5-X1b) and the client's `tuning`
  (`tests/tuning.rs`, slice M5-X2: *Tunables*). A new test file `tests/foo.rs` is a line
  `mod foo;` in its program's root, a folder `tests/foo/` (its root `main.rs`) is `#[path = "foo/main.rs"] mod foo;`,
  in name order; `crates/sim/tests/test_programs.rs` fails for any test file of either crate that no program
  includes, naming it and that line. One file's tests run by its module: `cargo test -p sc_sim --test sim holding::`
  (a filter matches any part of a test's path, so `gathering::` runs `agent_gathering::` too). The tests of all the
  files share one process and its threads: none changes the current folder or an environment variable, exits the
  process or keeps global state, and each writes files only in a folder of its own (the temp folder, a name of its
  own and the process id). serde_json is in scope in every module once one uses it, so an `assert_eq!` of a vector
  of numbers against `[]` names the type (`Vec::<Tick>::new()`). A module reaches its folder's root by `super::`:
  `crate::` is the program's root.
- **Test-only defs** (`defs/test_defs.rs`, slice M3-L2): a sim unit test can give a kind other numbers through the
  whole sim, on its own thread, while the guard `TestDef::set(def)` or `TestDef::unit(kind, change)` returns lives
  (`ObjectKind::def` reads them first): a MinRange, an artillery's Range, a unit that can't move, which no type of
  the design has yet. Only the sim's own unit tests can (`#[cfg(test)]`); they run the game through `TestSim` all
  the same (`orders/tests/leash.rs`, `orders/tests/min_range.rs`).
- Build test scenarios inline from an ASCII map or from `scenarios::empty(w, h)` plus tiles and spawns, not from
  `dev_skirmish`'s layout.
- **ASCII maps** (`Map::from_ascii`, `Map::to_ascii`; full rules in the `map` module docs) have one symbol per tile and
  one line per row, north to south: `.` Plane, `r` Rugged Terrain, `c` Cliff, `M` Mountain, `~` Water. An optional
  height block follows after a blank line, one row and one column larger than the tile block, with one base-17 digit
  per **corner** (`0`–`9`, then `a`–`g` for 10–16). Without it, every corner is at height 0. Lines are trimmed, so
  maps can be indented string literals.
- **Test helpers** live in the public `sc_sim::testing` module, so client tests can use them too.
  `ScenarioBuilder::new(ascii)` (`testing/builder.rs`) makes a scenario with the two slots of `scenarios::empty` and
  its default setup (players 1 and 2, GDO, in slots 1 and 2) and seed 1; `.faction(player, faction)` picks another
  faction for a player (plan decision T27). Add spawns with `.spawn(..)` (an owner takes the slot with its
  number), then call `.build()` or `.start()`. A `TestSim` runs with debug commands on
  (`TestSim::with_config` for other options). It schedules commands with `at(tick, player, cmd)` or
  `debug_at(tick, cmd)`, and each one applies in the step that reaches that tick; `spawn_unit_at(tick, kind, owner,
  pos)` and `order_at(tick, player, units, kind, queue)` are the shortcuts for units. It advances with `step`,
  `run(n)` and `run_until(tick)`, keeps every event with its tick, and records a `Replay` of the run. After every
  tick it checks the movement invariants (see Movement and collision); `pos(id)`, `mover(id)` and `status(id)` read
  a unit, and `ascii(focus)` draws the map. Units of different players fight when they see each other: a test
  about anything else calls `without_auto_targeting()` first, or keeps them apart or on Hold out of range.
  `testing::build_order` plays a player's opening by script (slice M2-J1), with the player's own commands through
  the gate: a `BuildOrder` is tracks of `Order`s (a building for one of the player's constructors, a unit for a
  producer) in priority order, and `commands(&sim)` gives the commands for the next tick: each ready building placed
  at the first origin, row by row, that the placement check takes (`first_spot`), then each track's next order once
  the bank covers it and its builder is free. `Books::open` before a step and `close` after it check a player's bank:
  it changed by what the plates mined (read off their patches) and the workers delivered (the step's
  `ResourcesDelivered`, plan decision T27), less what the step's orders paid, plus the refunds its events report.
  An `Order` whose builder is a Beacon is a Warp In (slice M3-K1), issued once `Sim::warp_in_check` takes it; a
  builder that transforms is never free. `testing::colonists` (slice M3-K1) plays a Colonists player the same way:
  a `Colonists` line sends the Drones to gather (one command, or each to a patch of its own), sets the portals'
  rally points, sends a Drone a portal releases to deploy at the next of its sites when the room under Beacon
  Capacity falls to 1 (the 5th Drone made and every 6th after it), and pays for the rest through a `BuildOrder`.
  `reference_field(drones)` is the reference geometry of design change 0107, and `colonists::skirmish` plays
  `dev_skirmish` with a Colonists line against the GDO's (the Colonists' golden games, and longer runs of them).
  `testing::syndicate` (slice M4-L) plays a Syndicate player the same way: a `Syndicate` line sends the Agents to
  gather (one command), sets the Hideouts' rally points, gives the player's `n`th Agent a job (Supplies at a station,
  or a Tunnel to build once the Supplies cover `Sim::build_price` and the build check takes its site), ejects an Agent
  that finished a Tunnel by it, has the `n`th Tunnel build the line's expansions once `Sim::can_expand` allows them
  and the bank covers them, and its upgrades once `Sim::can_upgrade` does and the Supplies cover them (slice M4-G;
  `tests/syndicate_line.rs`), trains units behind a prerequisite (the Guards behind an Optics Lab) only while the
  player owns it, and pays for its Agents through a `BuildOrder`; `SupplyBooks` balance the Supplies as `Books` do the
  crystals. `reference_field(distance, agents)`, `opening_field(distance)` and `stacked_field(distance, agents)` are
  the Syndicate economy check's reference layouts (design changes 0259 and 0294), and `syndicate::skirmish` plays
  `dev_skirmish` with a Syndicate line, alone or against the GDO's (the Syndicate's golden games).
  `testing::cults` (slice M5-L) plays a Cults player the same way: a `Cults` line deals each idle Recruit to a seat
  (a patch, in groups, near what the design says a face holds: since design change 0380 about 2 Recruits a tile of
  face length) once the player has a completed Storage, and keeps the seat it dealt until the Recruit is idle again
  (slice M5-S2a). A group (`Seats`, slice M5-S3, `seats.rs`) may face the Storage it is the room of, which the line
  builds once the player's Recruits outnumber the room standing, wait for it to be complete, and send its Recruits
  by a Move to the lane in front of their patch before the Gather. The line orders its buildings in turn, one at a
  time, each as one group Build from its crew (the Recruits whose seat has no room yet and those without one, or the
  `n` nearest the site) with each builder's way back to its seat queued after it, then the Storages its groups face
  as they're needed; sends idle Recruits into its first Armory and trains Militants of them
  (`ProductionCommand::TrainFrom`), and records what it pays by tick for `Books`. `line_field()`, `full_line(near,
  far)` and `face_field(side, gap, recruits)` are the Cults economy check's layouts (design change 0385: 8 patches in
  a row mined from both long sides, three Storage sites a side, `SITES`; one face of a lone 3x2 Storage), and
  `cults::skirmish` plays `dev_skirmish` with a Cults line, alone or against a Syndicate one (the Cults' golden
  games).
