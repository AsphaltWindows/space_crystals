# Space Crystals — Architecture

Every implementing agent reads this core in full before touching code, and then only the system docs its task needs
(see *System docs*). Follow its rules as written. If this file or a system doc disagrees with the code or the design,
stop and tell the Director. Don't pick one of them on your own.

## The design is the spec

- The design docs in `artifacts/designer/design/` are the spec (index: its `README.md`). Implement what they say.
- Never edit anything under `artifacts/`, because the Designer and the Director own it.
- **Cite a design section with its full path**, `path/file.md > Section`, relative to `artifacts/designer/design/`,
  in doc comments, in this file and in `docs/`. A relative `` `> Section` `` is allowed only inside a module whose doc
  names exactly one file (a design file, or a `docs/` one), and is a section of that file. A bare file name
  (`` `economy.md > Section` ``) is never allowed.
- **Citations are checked** by `crates/sim/tests/design_citations/` over `crates/**/*.rs`, `docs/**/*.md` and this
  file. It fails, naming file and line, when a cited design path doesn't exist, when a relative section reference stands
  where the module's doc doesn't cite exactly one `.md` file, or when a code span begins with a design file's bare
  name. It also fails when a reference to `ARCHITECTURE.md` or to a `docs/architecture/` file with a section names
  no heading of that file. The design's own sections aren't checked, since the design names many by bold
  paragraphs; an ignored test reports the ones that aren't headings:
  `cargo test -p sc_sim --test sim design_citations -- --ignored --nocapture`.
- **Comments**:
  - Cite the design section a piece of code implements by its full path.
  - Mark a number or rule the design doesn't give `Placeholder (QNNNN point N)`, or
    `Placeholder (not in the design: why)`. A number the design doesn't give is a **named const** (or a tunable, see
    *Tunables*), so that the answer later changes one line. A purely presentational placeholder (colours, sizes and
    fonts of placeholder art and HUD) is yours to choose: say so in the comment and list it in your report.
  - Don't quote the design beyond a few words, and only where its exact wording matters.
  - Don't record history (which slice, design change or plan decision did what): git and the Director's plan hold it.
  - Say briefly what the code does and why.
- Never invent design silently. Report any gap that no open question covers, and any unclear or contradictory rule
  (see *Working as an implementing agent*).

```rust
/// `core/map.md > SupplyDeliveryStation`: the countdown runs only while empty.
fn update_supply_stations(&mut self) { /* … */ }

footprint: Footprint::Tiles { w: 2, h: 2 }, // Size - 2x2

/// Placeholder (Q0003 point 1): the design gives no patch amount.
pub const DEFAULT_PATCH_AMOUNT: u32 = 1500;
```

## Layout, toolchain and commands

| Path | Package | Role |
|---|---|---|
| `crates/sim` | `sc_sim` (library) | The deterministic game simulation. Engine-free: it never depends on Bevy. |
| `crates/game` | `space_crystals` (binary) | The Bevy client: draws the sim, handles input, the HUD and QA mode. |
| `crates/ai` | `sc_ai` (library) | The computer players: deterministic, on the sim alone, never on Bevy. The client runs them. |

| Command | Use |
|---|---|
| `cargo run [-- <flags>]` | Runs the game, the only binary. Flags: `docs/architecture/qa-mode.md > QA mode`. |
| `cargo test --workspace` | Runs all tests. `cargo test -p sc_sim` tests the sim alone and skips Bevy, but Cargo resolves the sim's dependencies apart from a `--workspace` build, so running both compiles the sim twice. |
| `cargo test -p sc_sim --test sim holding::` | Runs one test file's tests (here `tests/holding/`'s): the integration tests are modules of a few test programs, and the filter is the file's module. The client's: `cargo test -p space_crystals --test game commands::`. |
| `cargo test --workspace -- --ignored seed_sweep` | The seed sweep, about 6 s. |
| `cargo test --release --workspace -- --ignored --test-threads=1 --skip economy_oracle` | The other ignored tests (the computer players' `ai_games`, the soaks, the perf tests), about 24 minutes. For milestone boundaries only. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Runs the lints and turns every warning into an error. |
| `cargo fmt --all` | Formats the code; add `--check` to verify only. `rustfmt.toml` sets max width 120. |

- **Toolchain**: the Rust version is pinned in `rust-toolchain.toml`, with Edition 2024. The Director bumps the
  toolchain together with Bevy upgrades.
- **Bevy `=0.20.1`** is built with `default-features = false` and an explicit feature list in
  `crates/game/Cargo.toml`. It leaves out `audio`, `bevy_gilrs` and `wayland`, because their system dev headers are
  not installed. If a Bevy item seems to be missing, it may sit behind a disabled feature.
- **Shared dependencies** are `serde`, `libm` and `glam`. glam is the version `bevy_math` uses (so `sc_sim::Vec2` is
  Bevy's `Vec2`) and is built with its `libm` feature.
- **Ask the Director before adding a Bevy feature or any new dependency.** `serde_json` is pre-approved for the client.
- The dev profile optimizes dependencies at `opt-level = 3` and builds our crates at `opt-level = 1`. Integer overflow
  checks stay on in dev and test builds. Debug info is line tables for our crates and none for the dependencies; for a
  debugger, `CARGO_PROFILE_DEV_DEBUG=true` gives our crates full debug info, in a build of their own.

## Definition of done

A task is done when all of the following hold:
1. It builds (`cargo build --workspace`) and the tests cover the new behaviour.
2. Doc comments follow the comment rule (*The design is the spec*).
3. This file or the system doc is updated if you changed or added a convention or a system's rules.
4. **Test policy**:
   - While you work, run the tests for the code you touch (`cargo test -p <crate> <filter>`).
   - At the end, run each of these once: `cargo fmt --all --check`,
     `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and the seed sweep
     (`cargo test --workspace -- --ignored seed_sweep`).
   - The other ignored suites (`ai_games`, the soaks) and perf A/B runs are for milestone boundaries. Run them only
     when your brief says so.

## Determinism

`sc_sim` guarantees that the same `Scenario` plus the same sequence of `TickInputs` produces bit-identical state on
every run and every machine. Replays, reproducible bugs and a possible lockstep multiplayer mode depend on this.
`crates/sim/clippy.toml` (and `crates/ai/clippy.toml`) enforce the mechanical rules below as disallowed methods and
types. Never `#[allow]` them. The one exception: an `#[ignore]` perf test, or the computer players' game harness, may
read the clock in a single timing helper marked `#[expect(clippy::disallowed_methods)]` (`nav_perf/` is the model).
Its timings are printed and never asserted.

- **Order**: iterate objects in `ObjectId` order. `Sim` keeps them in a `BTreeMap<ObjectId, Object>`, and ids are
  allocated in increasing order and never reused. Break ties between candidates by `ObjectId` (`Iterator::min_by`
  keeps the first minimum), and compare floats with `total_cmp`.
- **No hash containers**: don't use `HashMap` or `HashSet`, because their iteration order is random. Use `BTreeMap`,
  `BTreeSet` or a sorted `Vec`.
- **No outside world**: no wall clock, no threads and no OS randomness. The only source of randomness is the sim's
  PRNG (`sc_sim::Rng`, PCG32), seeded from the game setup's seed (`GameSetup::seed`). It is part of the hashed
  state. Draw from it only where the design asks for chance, and only when there is a choice to make.
- **Floats**: `f32` is fine. `+ - * /`, `sqrt`, `abs`, `floor`/`round` and `as` casts give identical results
  everywhere, but transcendental functions (`sin`, `cos`, `atan2`, `exp`, `powf`, …) don't. Call them through
  `sc_sim::math`, which wraps the pure-Rust `libm` crate, and add any missing function there. `powi` is banned too,
  so multiply explicitly. glam is built with its `libm` feature, so glam's own methods are safe.
- **No engine, frames or logging**: the sim never depends on Bevy, never sees frame time, and never logs or prints.
  Only `Sim::step` advances it.
- **Money and progress are integers**: banks are `u32`, rates and progress fixed point in `RATIO_ONE` (2^16) a tick.
  No floats in the economy (`docs/architecture/economy.md > Economy`).
- A change to the sim's behaviour moves the golden hashes: `docs/architecture/replays-and-golden-hashes.md > Replays and golden hashes`.

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

## Code conventions

- No `unsafe`: the workspace sets `unsafe_code = "forbid"`. Every crate opts into the workspace lints with
  `[lints] workspace = true`, and under `-D warnings` any `dbg!` or `todo!` fails clippy.
- Modules use `foo.rs` plus `foo/child.rs`. Never use `mod.rs`, and never use both forms for one module.
- Keep files focused. As a guideline, stay under about 500 lines, and split by responsibility before a file grows
  past that.
- Sim types for state, input and scenarios derive `Serialize, Deserialize`, for replays and state dumps.
- If `(x - 1.0).abs()` fails with an ambiguous numeric type, write `f32::abs(x - 1.0)`.
- Per-object numbers are fields of its def; a system's numbers for every kind are tunables (see *Tunables*).

## Adding a unit or structure

One kind is its own files, and one line in each registry. The long version, with the examples of what a def holds, is
`docs/architecture/adding-a-unit-or-structure.md > Adding a unit or structure`.

1. **Def module**, `crates/sim/src/defs/<faction>/{units,structures}/<object>.rs`, named as the design's file of the
   object. Its doc comment cites that file. It holds the kind's `ObjectDef` (ending `..ObjectDef::NONE`, with its
   `UnitDef` or `StructureDef`), every number the design gives the object, each cited, and the object's tests. What
   the type does differently is a field of its def that the logic reads, never a `match` on its kind. Add its line
   at the **end** of `kind_registry!` in `defs.rs`: the state hash numbers kinds in that order, and maps and replays
   name them by their serialized names, which must not change. A rule of its faction goes in `defs/<faction>.rs`.
   Then see `docs/architecture/object-model-state.md > Object model > Adding an object type` and `Adding a capability`.
2. **Client module**, `crates/game/src/kinds/<faction>/<object>.rs`: its `ClientKind` (a label of two letters of its
   own, its `object_render::Look`, its shots' `ShotStyle` if it has a weapon, its panel template), and its line in
   `kinds::of`, whose `match` is exhaustive. `kinds::tests` check labels and looks against the def.
3. **Panel**, `crates/game/src/command_ui/interface/templates/<faction>/<object>.rs`: its `TEMPLATE`, its grid and
   menus, its `InterfaceTemplate` variant and arms in `InterfaceTemplate::grid` and the right-click
   (`command_ui/resolution.rs`). `PanelTemplate::NONE` until its panel is built. A builder also needs its `Builder`
   (`docs/architecture/adding-a-faction.md > Adding a faction`).
4. **Where numbers go**: the object's own in its def; a system's for every kind in that system's tuning module; the
   client's feel in `crates/game/src/tuning/`; colours and drawing sizes with the drawing code. Each has a design
   citation or a `Placeholder` mark.
5. **Tests**: its def's in its module; a scenario test of what it does in `crates/sim/tests/`; the client's, if it has
   a panel, beside the other panels' (`crates/game/tests/commands/`).

## Tunables

The numbers a playtest tunes live apart from the logic, in tuning modules grouped by system, each one struct that a
data file can load. A number that belongs to one object goes in its def instead. The detail (the ruleset, its
generations, the data files) is `docs/architecture/tunables.md > Tunables`.

- **The sim's** (`crates/sim/src/tuning/`): one module per system, each one struct declared by `tunables!` (the
  struct, its serde form and its `BUILTIN`). A faction's own rules are a struct in its `defs/<faction>.rs`.
  **A new tunable is a field** at the end of its system's struct, with its doc and its built-in value, never a
  `const` the sim reads. The sim reads it as `rules().tuning.<system>.<field>`. A `const` context (an array's length,
  a test's constant) reads the struct's `BUILTIN`. **A hot loop reads its tunables once**, before the loop
  (`let tuning = &rules().tuning.movement;`), because each read is an atomic load.
- **The client's feel** (`crates/game/src/tuning/`): `camera.rs`, `input.rs`, `feedback.rs` and `look.rs`, each one
  struct declared by the client's own `tunables!`, together the `ClientTuning` resource. A function takes the module
  it reads, or the whole `&ClientTuning` when it is public.
- **The computer players'** (`crates/ai/src/tuning.rs`): the Director's placeholders, each marked.
- Each tunable and constant has a doc comment: a design citation, or a `Placeholder` mark with its reason.
  `docs/playtest/placeholders.md` is generated from those marks.
- Colours and drawing sizes stay with the drawing code. Technical constants (units of measure, hashing, file formats,
  margins that only bound a computation) stay where they are used.

## Bevy 0.20: verify, don't recall

Bevy 0.20, released in October 2026, is newer than your training data, which ends around 0.18 or 0.19. Its APIs
differ from what you remember. Before you use any Bevy API, check it against the local sources, including the
examples:

```sh
ls -d ~/.cargo/registry/src/*/bevy_*-0.20.1/              # one crate per subsystem: bevy_time, bevy_ui, …
grep -rn 'pub fn overstep_fraction' ~/.cargo/registry/src/*/bevy_time-0.20.1/src
ls ~/.cargo/registry/src/*/bevy-0.20.1/examples/          # 2d/, ui/, time/, camera/, input/, …
```

Known trap: a 2D material keeps the GPU texture it was prepared with, so never resize an image under the same handle
(`Assets::get_mut` marks the asset changed only when dereferenced mutably). Point the material at a new image, or keep
the size and write `image.data` in place.

## Working as an implementing agent

- Read this file first, then the system docs below that your task needs, then the design sections your task cites. The
  old prototype (git tag `pipeline-archive`) is reference only: it diverges from the design.
- Stay inside your task's scope, and don't touch other crates' internals. If a client task needs a sim change,
  report the need instead of making the change.
- Don't invent design. If a **rule or behaviour** is unclear or contradictory, implement the unambiguous part and
  report the question; the Director files design questions. If a **gameplay number** is missing, use a named
  placeholder whose comment cites the open question (or says "not in the design"), and list it in your report.
- In an isolated git worktree, first make sure it contains the latest local `master` (`git merge --ff-only master`),
  and seed the build cache if the Director's brief says to (a cold Bevy build takes about 7 minutes).
- When you are done, commit on your worktree branch with explicit pathspecs (`git commit -- <paths>`). Never push,
  and never touch `master`.
- Your final report covers what changed, how you verified it and any open issues. List every placeholder and design
  question you ran into.

## System docs

The per-system rules are in `docs/architecture/`. Each file starts with a "Read this when" line. Read only the ones
your task touches. Cite a section as `docs/architecture/<file>.md > Section`.

| File | Covers; read it when |
|---|---|
| `docs/architecture/map-model.md` | The map: tiles, presets, corner heights, slopes, symmetry. You touch `map.rs`. |
| `docs/architecture/object-model-types.md` | `Object`, capabilities, `ObjectDef`, each faction's types. You add or change an object or a def. |
| `docs/architecture/object-model-holding.md` | The held store, sites, holding buildings, expansions, pop-ups, tech, the Tunnel Network. |
| `docs/architecture/object-model-state.md` | Carriers, kind changes, augmented stats, vehicle deploy, adding an object type or capability. |
| `docs/architecture/placement-and-build-area.md` | Footprints, the build area, placement checks. |
| `docs/architecture/tick-pipeline.md` | The order of `Sim::step`'s phases. You add or reorder a step. |
| `docs/architecture/combat.md` | Weapons, damage, projectiles, shields, energy, armor. |
| `docs/architecture/vision.md` | Sight and the Shroud's rules. |
| `docs/architecture/behaviours.md` | Orders and the behaviours that carry them out. |
| `docs/architecture/navigation.md` | Pathfinding, path knowledge, nav grids. |
| `docs/architecture/movement-and-collision.md` | Movement, steering, collision, jams, flight. |
| `docs/architecture/economy.md` | Banks, income, Supply, gathering, recipe timing. |
| `docs/architecture/production.md` | Producers, queues, Airfields, rally points. |
| `docs/architecture/commands-events-replays.md` | `Command`, `SimEvent`, the gate; how commands are recorded. |
| `docs/architecture/sim-scenarios-and-tests-setup.md` | `Scenario`, map files, generated maps, sim test helpers, test programs. |
| `docs/architecture/sim-scenarios-and-tests-systems.md` | Where the sim's tests of the core systems are. |
| `docs/architecture/sim-scenarios-and-tests-factions.md` | Where the sim's tests of the factions' features are. |
| `docs/architecture/client-architecture-visuals.md` | Client kinds, sim driver, drawing, fog, viewer events. |
| `docs/architecture/client-architecture-camera-and-hud.md` | Camera, window, HUD, minimap, playtest menu. |
| `docs/architecture/client-architecture-input-and-selection.md` | Input contexts and bindings, selection. |
| `docs/architecture/client-architecture-command-ui.md` | The command panel and each unit's and building's panel. |
| `docs/architecture/client-architecture-placement-and-tests.md` | Panel visuals, placement, Bevy code, client tests, generated docs. |
| `docs/architecture/qa-mode.md` | Running and scripting the game for QA; QA flags. |
| `docs/architecture/replays-and-golden-hashes.md` | Replay files, saves, the state hash, golden replays. |
| `docs/architecture/map-editor.md` | The map editor. |
| `docs/architecture/computer-players.md` | `sc_ai` and its tests. |
| `docs/architecture/adding-a-faction.md` | The steps to add a faction. |
| `docs/architecture/adding-a-unit-or-structure.md` | The long checklist for a unit or structure. |
| `docs/architecture/tunables.md` | The ruleset, its generations, the data files, each tunable module. |
