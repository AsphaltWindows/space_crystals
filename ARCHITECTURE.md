# Space Crystals — Architecture

Every implementing agent reads this brief before touching code. Follow its rules as written. If this file disagrees with
the code or the design, stop and tell the Director. Don't pick one of them on your own.

## The design is the spec

- The design docs in `artifacts/designer/design/` are the spec (index: its `README.md`). Implement what they say.
- Never edit anything under `artifacts/`, because the Designer and the Director own it.
- Cite the design in doc comments as `path/file.md > Section`, with paths relative to `artifacts/designer/design/`.
- A number the design doesn't give becomes a **named const**. Its comment names the open Director question
  (`artifacts/director/questions/QNNNN-*.md`), so that the answer later changes one line.
- Never invent design silently. Report any gap that no open question covers, and any unclear or contradictory rule
  (see *Working as an implementing agent*).

```rust
/// `core/map.md > SupplyDeliveryStation`: the countdown runs only while empty.
fn update_supply_stations(&mut self) { /* … */ }

footprint: Footprint::Tiles { w: 2, h: 2 }, // Size - 2x2

/// Placeholder until the design gives one (question Q0003).
pub const DEFAULT_PATCH_AMOUNT: u32 = 1500;
```

## Layout, toolchain and commands

| Path | Package | Role |
|---|---|---|
| `crates/sim` | `sc_sim` (library) | The deterministic game simulation. Engine-free: it never depends on Bevy. |
| `crates/game` | `space_crystals` (binary) | The Bevy client: draws the sim, handles input, the HUD and QA mode. |

| Command | Use |
|---|---|
| `cargo run [-- <flags>]` | Runs the game, the only binary. Flags are listed under *QA mode*. |
| `cargo test --workspace` | Runs all tests. `cargo test -p sc_sim` tests the sim alone, which is fast and skips Bevy. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Runs the lints and turns every warning into an error. |
| `cargo fmt --all` | Formats the code; add `--check` to verify only. `rustfmt.toml` sets max width 120. |

- **Toolchain**: Rust 1.100.0 is pinned in `rust-toolchain.toml`, with Edition 2024. The workspace `rust-version` is
  1.97.1, the minimum Bevy 0.20.1 accepts. The Director bumps the toolchain, together with Bevy upgrades.
- **Bevy `=0.20.1`** is built with `default-features = false` and an explicit feature list in
  `crates/game/Cargo.toml`. It leaves out `audio`, `bevy_gilrs` and `wayland`, because their system dev headers are
  not installed. If a Bevy item seems to be missing, it may sit behind a disabled feature.
- **Shared dependencies** are `serde`, `libm` and `glam`. glam is the same version `bevy_math` uses (so `sc_sim::Vec2`
  is Bevy's `Vec2`) and is built with its `libm` feature.
- **Ask the Director before adding a Bevy feature or any new dependency.** `serde_json` is pre-approved for the client.
- The dev profile optimizes dependencies at `opt-level = 3` and builds our crates at `opt-level = 1`. Integer overflow
  checks stay on in dev and test builds.

## Definition of done

A task is done when all of the following hold:
1. It builds (`cargo build --workspace`).
2. `cargo clippy --workspace --all-targets -- -D warnings` is clean.
3. `cargo fmt --all --check` passes.
4. `cargo test --workspace` passes, and tests cover the new behaviour.
5. Doc comments cite the design sections the code implements.
6. This file is updated if you changed or added a convention.

## Determinism

`sc_sim` guarantees that the same `Scenario` plus the same sequence of `TickInputs` produces bit-identical state on
every run and every machine. Replays, reproducible bugs and a possible lockstep multiplayer mode depend on this.
`crates/sim/clippy.toml` enforces the mechanical rules below as disallowed methods and types. Never `#[allow]` them.

- **Order**: iterate objects in `ObjectId` order. `Sim` keeps them in a `BTreeMap<ObjectId, Object>`, and ids are
  allocated in increasing order and never reused. Break ties between candidates by `ObjectId` (`Iterator::min_by`
  keeps the first minimum), and compare floats with `total_cmp`.
- **No hash containers**: don't use `HashMap` or `HashSet`, because their iteration order is random. Use `BTreeMap`,
  `BTreeSet` or a sorted `Vec`.
- **No outside world**: no wall clock, no threads and no OS randomness. The only source of randomness is the sim's
  PRNG (`sc_sim::Rng`, PCG32), seeded from `Scenario::seed`. It is part of the hashed state.
- **Floats**: `f32` is fine. `+ - * /`, `sqrt`, `abs`, `floor`/`round` and `as` casts give identical results
  everywhere, but transcendental functions (`sin`, `cos`, `atan2`, `exp`, `powf`, …) don't. Call them through
  `sc_sim::math`, which wraps the pure-Rust `libm` crate, and add any missing function there. `powi` is banned too,
  so multiply explicitly. glam is built with its `libm` feature, so glam's own methods are safe.
- **No engine, frames or logging**: the sim never depends on Bevy, never sees frame time, and never logs or prints.
  Only `Sim::step` advances it.

## Time, space and units

The design is `core/scale.md`, and the code is `crates/sim/src/units.rs`.

- **Time**: the sim runs at 16 ticks per second (`TICKS_PER_SECOND`,
  `core/scale.md > SimulationFrame > FramesPerSecond`). `Tick` is a `u32`. Tick 0 is the initial state, and each
  `Sim::step` advances one tick. Every duration and timer counts ticks. Convert design seconds once with
  `units::seconds(s)`, which rounds to the nearest tick.
- **Space**: positions are `Vec2` values in space units (su). 64 su make 1 grid unit (GU), which is 1 tile
  (`SU_PER_GU`, `core/scale.md > SpaceUnit > SpaceUnitsPerGridUnit`). Speeds are su per tick
  (`core/scale.md > SpaceUnit`). Strategic distances (Range, SightRange, …) come in GU (`core/scale.md > GridUnit`).
  Store them in GU as the design gives them, and convert with `units::gu()`.
- **Axes**: the origin is the map's top-left corner. +x points east and **+y points south**, the direction in which
  tile rows grow. Tile `(x, y)` covers `[64x, 64x+64) × [64y, 64y+64)`. Convert with `TilePos::containing`, `corner`
  and `center`.
- **Angles** are in radians. 0 points east (+x), and **positive angles turn clockwise on screen**, towards +y. Use
  `math::facing_vector`, `math::vector_angle` and `math::wrap_angle`, which wraps to `(-π, π]`.
- **Object positions**: `Object::pos` is the object's center. Tile-based objects also have a `Placement`. Its `origin`
  is the top-left tile, and its `w`/`h` are the footprint **after** rotation, so 90° and 270° swap them. Spawns use
  `SpawnAt::Tile(origin)` for tile-based objects and `SpawnAt::Point(center)` for units.

## Object model

Units, structures and resources are all `Object`s (`object.rs`, `core/entities.md > Object Instance`). The layout is
"ECS-lite": a plain struct with core fields plus one `Option` per capability.

- **Core fields**: `id`, `kind`, `owner` (`None` means neutral), `pos`, `prev_pos`, `facing`, `prev_facing` and `hp`
  (`None` means indestructible).
- **Capabilities**: today these are `placement` (the tile footprint) and `resource` (`ResourceState`). Movement,
  weapons, production and the rest arrive as more optional fields, each with its own module. Never add them as
  separate stores keyed by id. A system walks the objects in id order and skips those without its capability
  (`let Some(..) = &mut o.resource else { continue };`).
- **Per-type constants** sit in one `static ObjectDef` per type (`defs.rs`, `core/entities.md > Object Type`). Reach
  them through `ObjectKind::def()` or `Object::def()`. Every value cites its design section. Faction types go in
  per-faction modules (`defs/gdo.rs`, …).

**Adding an object type**
1. Add an `ObjectKind` variant whose doc comment cites the design section.
2. Add its `static ObjectDef`, with a citation for every value, and its arm in `ObjectKind::def()`. Missing numbers
   become named consts tagged with the question id.
3. Set up its capability state in `Sim::spawn`. The `match kind` there is exhaustive, so the compiler points you to it.
4. Test it. Spawn it in a small inline scenario, then check its initial state and its behaviour.

**Adding a capability**
1. Give it its own module with its state type (deriving `Clone, Debug, PartialEq, Serialize, Deserialize`) and logic.
2. Add `pub <name>: Option<State>` to `Object`, with a doc comment citing the design.
3. Put its per-type parameters on `ObjectDef`, with citations, and initialize the state in `Sim::spawn`.
4. Insert its update phase into `Sim::step` at a deliberate position (see *Tick pipeline*).
5. Add the `Command`s that drive it, with validation, and the `SimEvent`s the client needs.
6. Write unit tests in the module and a scenario test in `crates/sim/tests/`.

## Tick pipeline

`Sim::step(&TickInputs)` advances exactly one tick. It runs fixed phases in this order (`sim.rs`):

1. **Clear events**, so that `Sim::events()` holds only this tick's events.
2. **Advance the tick** (`tick += 1`).
3. **Snapshot** `pos`/`facing` into `prev_pos`/`prev_facing` for every object (the client interpolates from these).
4. **Commands**: apply each `PlayerCommand` in `TickInputs` order (`apply_command`).
5. **Systems**, each walking the objects in id order. Today the only one is `update_supply_stations`
   (`core/map.md > SupplyDeliveryStation`).

- Insert a new phase at a deliberate position, with a comment in `step` saying why there, and update this list.
- Commands always run before movement. There is exactly **one** movement pipeline, and all movement goes through it.
  The old prototype had two pipelines, and they raced.
- Damage accumulates as a **list of hits** that is applied in one place. Never store it in a single slot that a second
  hit can overwrite, because the old prototype lost hits that way.
- Lessons to apply when these systems arrive, from M1 on:
  - **Movement**: the arrival radius must exceed one tick of travel. Otherwise units overshoot and oscillate.
  - **Pathfinding**: cap repath attempts. A* needs a correct closed set, a best-cost map and deterministic
    tie-breaking. Overwriting a parent at equal cost made zigzag paths.
  - **Occupancy** is domain-aware. For example, underground expansions don't block surface units.

## Commands, events and replays

- All player intent is a `Command`. It is wrapped in a `PlayerCommand { player, command }` and collected in one
  tick's `TickInputs`. The client never mutates sim state. It reads `Sim` and submits commands.
- The sim validates every command (player, ownership, ids, costs, placement, and positions that are finite and on the
  map), and it **ignores invalid ones**. It must never panic on player input, so use checked or saturating arithmetic
  on numbers that come from input. Panics are only for programmer errors, such as an invalid scenario passed to
  `Sim::new` or an out-of-bounds `Map::set_tile`.
- Every new command gets tests showing that its invalid forms are ignored without a panic.
- `Command::Debug(DebugCommand::…)` covers `Spawn`, `Remove` and `SetPosition`, for development and QA. It bypasses
  costs and rules and is never offered in a real match. A teleport sets `prev_pos = pos`, so nothing interpolates
  across the map.
- **Replay** = the `Scenario` plus the `TickInputs` of every tick, empty ticks included: `sc_sim::Replay` (serde)
  with `record`/`step`/`run`. The client must never submit a command with a non-finite number. The sim would ignore
  it anyway, but JSON can't store NaN or ±∞, so the replay file would fail to load. Validate at the input layer.
- **Events**: `SimEvent`s (`Spawned`, `Removed`, `SuppliesDelivered`, …) are output only, for effects, sounds and UI.
  The sim never reads them. `Sim::events()` holds only the latest step's events, and `Sim::new` discards the events
  from the scenario's initial spawns.

## Sim scenarios and tests

- A `Scenario` (`scenario.rs`) holds a name, the `Map`, the players (`PlayerSetup`, including `camera_start` and
  `start_location`, a placeholder until Q0004), the spawns and the RNG `seed`. Spawns are created in order at tick 0,
  so their ids ascend in list order.
- `Scenario::validate()` lists every problem (`ScenarioProblem`) and never panics. `Sim::try_new` returns the problems
  as an error and `Sim::new` panics with them, so load files and editor maps with `try_new`.
- **Scenario files** are a `Scenario`'s serde form, written with `serde_json::to_string_pretty`. A `version` field
  (`SCENARIO_FORMAT_VERSION`) comes first; bump it on any incompatible change. The map is stored as rows of strings in
  the ASCII format below, with the custom presets listed by symbol. Reading rejects other versions and unknown fields.
- Built-in scenarios live in `scenarios.rs`, behind `by_name(name)` and `NAMES`. Keep the two in sync, because the
  client's `--scenario` flag uses them. Currently there are `dev_skirmish`, a 64×48 development map the Director made
  (see Q0003), and `empty`, a 32×24 map. `scenarios::empty(w, h)` builds an empty map of any size.
- Unit tests sit in their module (`#[cfg(test)] mod tests`). Scenario and integration tests go in `crates/sim/tests/`
  and use only the public API.
- Build test scenarios inline from an ASCII map or from `scenarios::empty(w, h)` plus tiles and spawns, not from
  `dev_skirmish`'s layout.
- **ASCII maps** (`Map::from_ascii`, `Map::to_ascii`; full rules in the `map` module docs) have one symbol per tile and
  one line per row, north to south: `.` Plane, `r` Rugged Terrain, `c` Cliff, `M` Mountain, `~` Water. An optional
  elevation block of the same size follows after a blank line, with one base-17 digit per tile (`0`–`9`, then `a`–`g`
  for 10–16). Without it, every tile is at elevation 0. Lines are trimmed, so maps can be indented string literals.
- **Test helpers** live in the public `sc_sim::testing` module, so client tests can use them too.
  `ScenarioBuilder::new(ascii)` makes a scenario with the two players of `scenarios::empty` and seed 1. Add spawns
  with `.spawn(..)`, then call `.build()` or `.start()`. A `TestSim` schedules commands with `at(tick, player, cmd)`
  or `debug_at(tick, cmd)`, and each one applies in the step that reaches that tick. It advances with `step`,
  `run(n)` and `run_until(tick)`, keeps every event with its tick, and records a `Replay` of the run.
- **Determinism tests** compare `Sim::state_hash()`, an FNV-1a 64 hash of the whole state except the events. Run the
  same scenario and inputs twice, or play back a `Replay`, and compare the hashes. Every state type's hash impl
  destructures it (`hash.rs`), so a new field doesn't compile until it is hashed.

## Client architecture

M0 builds this skeleton, so follow it. The art direction is 2.5D pixel-art sprites in the style of StarCraft 1 and
Age of Empires 2, with the details still open in Q0001. Until that art exists, everything is a placeholder shape laid
out by the same rules.

**Sim driver**
- The `Sim` lives in a Bevy `Resource`. That is a client newtype, because `sc_sim` can't derive Bevy traits. The sim
  is stepped only in `FixedUpdate`, whose timestep comes from `sc_sim::TICKS_PER_SECOND` (`Time::<Fixed>::from_hz`).
  One FixedUpdate run is one tick. Nothing else steps the sim, and no game logic reads frame time.
- Player input becomes `PlayerCommand`s in a pending-commands `Resource`. The stepping system drains it into that
  tick's `TickInputs`, which is what a replay records.
- Read `Sim::events()` in the same system, right after `step`, and forward them, for example as Bevy `Message`s. The
  next step clears them, and FixedUpdate can run several times in one frame.

**Visuals**
- The sim is the source of truth. Visuals are separate entities, each linked to its object by an
  `ObjectVisual(ObjectId)` component. Their `Transform`s are derived from the sim every frame and are never read back
  for gameplay.
- Create and despawn visuals from sim events, or by diffing ids against `Sim::objects()`. Build the initial set from
  `Sim::objects()`, because scenario spawns emit no events.
- **Interpolation**: draw each object at `prev_pos.lerp(pos, a)`, where `a` is `Time<Fixed>::overstep_fraction()`
  read in `Update`. Interpolate facing along the shortest arc.
- **Coordinates**: 1 world unit is 1 su. `bevy_x = sim_x` and **`bevy_y = -sim_y`**, because Bevy's +y points up.
  Convert in one helper, never ad hoc. A sim facing θ is a Bevy rotation of −θ about z.
- **Draw order** follows StarCraft 1. The layers are terrain < ground decals < ground objects < air < overlays, each
  with its own z band. Within a layer, z grows with sim y, so southern objects draw in front. Each sprite is anchored
  at its ground position, the sim `pos`, and that point drives its y-sort.
- Never make one entity per tile. Draw the terrain as one mesh or texture, or a few chunks. Fog of war will be one
  texture that is updated when visibility changes, never per-tile materials.
- Create mesh and material handles once and cache them in a `Resource`. Never create assets per frame, per object or
  per indicator, because the old prototype ran out of memory doing that.

**Camera**: the camera is orthographic and shows exactly 28 GU horizontally (`core/camera.md > Zoom`; see
`ScalingMode::FixedHorizontal`). Its viewport is the area between the UI bars (`ViewportInsets`: the top and bottom
HUD bars in the game, the editor's panels in the editor), so the visible height follows from the aspect ratio. It
pans with the arrow keys and at the screen edge, with bindings and speed pending in Q0002, and it stays clamped to
the map. Centering on something snaps instantly (`core/camera.md > Panning`). The camera starts at
`PlayerSetup::camera_start`, or else at the primary starting structure (`core/camera.md > Starting Position`). Until
starting structures exist, the slot's start location stands in for it, and the map centre for a slot without one
(`camera::start_center`). Only the editor zooms (`CameraZoom`).

**HUD**: built with bevy_ui (`core/camera.md > HUD Layout`). It has a top resource bar and, at the bottom, the HUD
Bar with five fixed sections. From left to right they are Minimap, Portrait, InfoPanel, SelectionPanel and
CommandPanel. All five have the same height and keep fixed horizontal slots, and a hidden panel's slot stays reserved
(`systems/controls/selection_and_hud.md > HUD Bar`). bevy_ui lays out each root node inside its camera's viewport,
so the HUD needs its own full-window UI camera (`IsDefaultUiCamera` / `UiTargetCamera`).

**Input**: raw Bevy input becomes our `InputAction`s through one binding table, and those become UI state changes
and `Command`s. Tests and QA scripts inject `InputAction`s, never raw `ButtonInput`. In a headless run, injected
presses are cleared before game systems see them.

**Bevy code**
- Write one plugin per feature module (camera, map rendering, HUD, input, …). Keep systems small. Bevy caps a system
  at 16 parameters, and the prototype's UI systems hit that cap.
- Keep any state a test must set or inspect in a `Resource`, not a `Local`.
- Send one-shot happenings as sim events or Bevy messages, never as marker components that are added and removed.

**Client tests** advance an exact number of ticks deterministically, never by wall-clock time. Insert
`TimeUpdateStrategy::FixedTimesteps(1)`, so that each `App::update()` runs FixedUpdate exactly once. The very first
update only starts the clock, so assert on `Sim::tick()`, not on the number of updates.

## Bevy 0.20: verify, don't recall

Bevy 0.20, released in October 2026, is newer than your training data, which ends around 0.18 or 0.19. Its APIs
differ from what you remember. Before you use any Bevy API, check it against the local sources, including the
examples:

```sh
ls -d ~/.cargo/registry/src/*/bevy_*-0.20.1/              # one crate per subsystem: bevy_time, bevy_ui, …
grep -rn 'pub fn overstep_fraction' ~/.cargo/registry/src/*/bevy_time-0.20.1/src
ls ~/.cargo/registry/src/*/bevy-0.20.1/examples/          # 2d/, ui/, time/, camera/, input/, …
```

## QA mode

Pass the flags with `cargo run -- <flags>`; `cargo run -- --help` lists them. The parser is `crates/game/src/cli.rs`.

| Flag | Effect |
|---|---|
| `--scenario <name>` | Starts a built-in scenario (`sc_sim::scenarios::NAMES`). Default `dev_skirmish`. |
| `--map <file.json>` | Starts a `Scenario` read from JSON (`sc_sim`'s serde format) instead. Excludes `--scenario`. Exits with code 2, listing the problems, if `Sim::try_new` rejects it. |
| `--editor [file.json]` | Opens the map editor (see Map editor) on that file, or on a new map. |
| `--editor-script <file.json>` | Applies a JSON list of `EditorAction`s in the editor's first frame. |
| `--player <n>` | Shows player `n`'s view: camera start, top bar. Default 1. |
| `--window-size <W>x<H>` | Window or offscreen image size in physical pixels. Default `1280x720`. |
| `--fast` | Runs as many ticks per frame as fit in 50 ms instead of 16 per second. Screenshots are unchanged. |
| `--screenshot <tick>:<path>` | Saves a PNG of exactly that tick. Repeatable, in any order; creates parent dirs. |
| `--exit-at-tick <n>` | Exits at tick `n`, after its screenshots. Windowless runs default to the last screenshot's. |
| `--dump-state <path>` | Writes the state as JSON when the run ends (fields below). |
| `--windowed` | Shows a QA run in a window on the user's display instead of rendering offscreen. |
| `--headless` | No window and no renderer: sim, client state and `--dump-state` only. Needs `--exit-at-tick`. |
| Later: `--script <file>`, replays | Run scripted `InputAction`s, and replay recorded sessions. |

Taking a screenshot (no window appears; about one second plus `tick / 16` seconds, or add `--fast`):

```sh
cargo run -- --scenario dev_skirmish --player 2 --screenshot 1:/path/to/p2.png --dump-state /path/to/p2.json
```

- Any of `--screenshot`, `--exit-at-tick`, `--dump-state` makes a *QA run*. It ignores keyboard and mouse, renders at
  scale factor 1, and renders offscreen: Bevy renders into an image on this machine's GPU (Vulkan, Intel Arc), with
  no window and no Xvfb. The sim waits at each screenshot tick until the PNG is saved, so the PNG shows that tick's
  exact state with no interpolation. Repeated runs give byte-identical PNGs.
- Exit codes: 0 when everything was saved, 1 when a QA step failed (the log says which), 2 for invalid flags.
- The `--dump-state` JSON holds `tick`, `scenario`, `players`, `objects` (every field of `sc_sim::Object`), the last
  tick's `events`, `client` (`local_player`, `camera_center` in su, `grid_visible`, `recorded_ticks`) and `map`.
- `--windowed` runs turn vsync off: with the display asleep, X11 paces a vsynced window at one frame per second.
- Verify visual work yourself. Capture screenshots and look at the PNGs with the Read tool before you report.
- To look at another part of a map, write a scenario JSON with a different `camera_start` and pass it with `--map`.

## Map editor

`cargo run -- --editor [file.json]` opens the map editor (`crates/game/src/editor.rs` and `editor/`). It edits a
`Scenario` and saves it in the scenario file format above. F1 in the editor lists the mouse and keys.

- **Starting**: with a file that exists, the editor opens it. With a file that doesn't exist yet, it starts a new map
  that Ctrl+S saves there. Without a file, it starts a new 64×48 map, and `--editor --scenario <name>` opens a
  built-in scenario. A file that doesn't parse exits with code 2. A file that parses but fails validation still
  opens, with its problems listed, so that it can be fixed.
- **Files**: maps live in `maps/` at the repository root (`editor::files::default_maps_dir`), and new maps are saved
  as `maps/<name>.json`. `maps/dev_skirmish.json` is the built-in `dev_skirmish` saved by the editor. A test checks
  that every map in `maps/` is playable and that this one still matches the built-in.
- **Actions**: every change is an `EditorAction` (`editor/actions.rs`), queued in the `EditorActions` resource and
  applied in `EditorSystems::Apply`. Keys, the mouse and the UI's buttons only queue actions. The edits are plain
  functions (`editor/terrain.rs`, `editor/objects.rs`) and sim-side helpers (`Map::resize`, `Map::connected_region`,
  `Scenario::resize_map`, `add_player`, `remove_player`).
- **Document**: `EditorDoc` holds the scenario, its file and the undo history, with one snapshot per completed action
  (a whole stroke, drag or field edit). When the document changes, the editor replaces the sim with
  `Sim::preview(scenario)`. That is a sim at tick 0 that shows every spawn that fits on the map, even when the map is
  invalid, so the game's map and object rendering show the document.
- **Modes**: `ClientMode` is `Game`, `Editor` or `PlayTest`, and only the HUD and the editor look at it. F5 starts a
  play-test: `Sim::try_new` on the document, as the player of the selected slot, at the game's zoom. Esc or F5 returns
  to the editor with the document, its history and the view unchanged.
- **Tests** (`crates/game/tests/editor/`) build a headless app with `AppConfig::editor`, push `EditorAction`s into
  `EditorActions`, call `app.update()`, and assert on `EditorDoc`, the sim, the view and the UI state. Positions are
  sim coordinates, such as `TilePos::center()`. Never inject raw input.
- **QA**: `--editor-script` applies its actions in the first frame, before any screenshot. The editor's sim stays at
  tick 0, so take screenshots at tick 0 unless the script play-tests (`"PlayTest"`).

```sh
echo '["FitMap", {"SetTool": "Paint"}, {"Press": [100.0, 80.0]}, "Release"]' > /path/to/edits.json
cargo run -- --editor maps/dev_skirmish.json --editor-script /path/to/edits.json --screenshot 0:/path/to/editor.png
```

- **UI**: bevy_ui with Bevy's `bevy_ui_widgets` feature (headless `Button`, `TextInput` and `ScrollArea`) and
  `ui_picking`. The player slot colours, UI sizes, `MAX_SLOTS`, `NEW_MAP_SIZE` and `MAX_MAP_SIDE` are placeholders,
  and so are start locations (Q0004).

## Code conventions

- No `unsafe`: the workspace sets `unsafe_code = "forbid"`. Every crate opts into the workspace lints with
  `[lints] workspace = true`, and under `-D warnings` any `dbg!` or `todo!` fails clippy.
- Modules use `foo.rs` plus `foo/child.rs`. Never use `mod.rs`, and never use both forms for one module.
- Keep files focused. As a guideline, stay under about 500 lines, and split by responsibility before a file grows
  past that.
- Doc comments cite the design sections the code implements.
- Sim types for state, input and scenarios derive `Serialize, Deserialize`, for replays and state dumps.
- If `(x - 1.0).abs()` fails with an ambiguous numeric type, write `f32::abs(x - 1.0)`.

## Working as an implementing agent

- Read this file first, then the design sections your task cites. The old prototype (git tag `pipeline-archive`) is
  reference only. It diverges from the design and its failures were architectural, so don't port it.
- Stay inside your task's scope, and don't touch other crates' internals. If a client task needs a sim change,
  report the need instead of making the change.
- Don't invent design. If a **rule or behaviour** is unclear or contradictory, implement the unambiguous part and
  report the question; the Director files design questions. If a **gameplay number** is missing, use a named
  placeholder constant whose comment cites the open question (or says "unspecified"), and list it in your report.
  **Purely presentational placeholder choices** (colours, sizes and fonts of placeholder art and HUD) are yours to
  make; list them briefly.
- In an isolated git worktree, first make sure it contains the latest local `master` (worktrees can be created
  from the last *pushed* commit): `git merge --ff-only master`, which is a no-op when already up to date.
- In an isolated git worktree, seed the build cache first with the command below. On btrfs it is near-instant, while
  a cold Bevy build takes about 7 minutes. If the command fails (no reflink support, e.g. tmpfs), skip seeding and
  build cold. Never fall back to a plain copy: the cache is over 5 GB.
  `cp -a --reflink=always /home/iv/dev/space_crystals/target ./target`
- When you are done, commit on your worktree branch with explicit pathspecs (`git commit -- <paths>`). Never push,
  and never touch `master`.
- Your final report covers what changed, how you verified it (commands, tests, screenshots viewed) and any open
  issues. List every placeholder constant and design question you ran into.
