# Space Crystals — Architecture

Every implementing agent reads this brief before touching code. Follow its rules as written. If this file disagrees with
the code or the design, stop and tell the Director. Don't pick one of them on your own.

## The design is the spec

- The design docs in `artifacts/designer/design/` are the spec (index: its `README.md`). Implement what they say.
- Never edit anything under `artifacts/`, because the Designer and the Director own it.
- **Cite a design section with its full path**, `path/file.md > Section`, relative to `artifacts/designer/design/`,
  in doc comments, in this file and in `docs/`. A relative `` `> Section` `` is allowed only inside a module whose doc
  names exactly one file (a design file, or a `docs/` one), and is a section of that file. A bare file name
  (`` `economy.md > Section` ``) is never allowed.
- `crates/sim/tests/design_citations/` checks the citations in `crates/**/*.rs`, `docs/**/*.md` and this file, and
  fails naming file and line: every backticked design path must exist under `artifacts/designer/design/` (a `docs/`
  path under the repository's `docs/`); a relative `` `> Section` `` may stand only in a module whose doc (its leading
  `//!` lines) cites exactly one `.md` file; no code span may begin with a design file's bare name. Its ignored test
  reports the cited sections that aren't headings of their file (a report, not a gate: the design names many sections
  by bold paragraphs): `cargo test -p sc_sim --test sim design_citations -- --ignored --nocapture`.
- A number the design doesn't give becomes a **named const**. Its comment names the open Director question
  (`artifacts/director/questions/QNNNN-*.md`), so that the answer later changes one line. A tunable goes in a tuning
  module (see *Tunables*).
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
| `cargo test --workspace` | Runs all tests. `cargo test -p sc_sim` tests the sim alone and skips Bevy, but Cargo then resolves the sim's dependencies apart from a `--workspace` build, so the two never share a build of the sim: after an edit, running both compiles it twice. |
| `cargo test -p sc_sim --test sim holding::` | Runs one test file's tests, here `tests/holding/`'s: the integration tests are modules of a few test programs (*Sim scenarios and tests*), and the filter is the file's module. The client's are `cargo test -p space_crystals --test game commands::`. |
| `cargo test --release --workspace -- --ignored --test-threads=1` | Runs the ignored tests: the perf tests, soaks, sweeps and measurements (23 at M4's end, all in the sim, about 1 minute on a quiet machine). Run them at a milestone's end, since nothing else does. |
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
  checks stay on in dev and test builds. Its debug info (slice M5-T) is line tables for our crates and none for the
  dependencies, which took the client's test program and binary from 1.7 GB each to under 0.25 GB: a backtrace
  still gives each frame of ours its file and line, with its function's bare name (`pos` for `TestSim::pos`), and
  `split-debuginfo = "unpacked"` measured no faster. For a debugger, `CARGO_PROFILE_DEV_DEBUG=true` gives our
  crates full debug info, in a build of their own; the dependencies' builds stay as they are.

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
There is one exception: an `#[ignore]` perf test under `crates/sim/tests/` may read the clock in a single timing helper marked
`#[expect(clippy::disallowed_methods)]`, as `nav_perf/` does, and so may one that needs the sim's test defs and so is a
unit test (`combat/projectiles/perf.rs`, slice M4-O). Its timings are printed and never asserted.

- **Order**: iterate objects in `ObjectId` order. `Sim` keeps them in a `BTreeMap<ObjectId, Object>`, and ids are
  allocated in increasing order and never reused. Break ties between candidates by `ObjectId` (`Iterator::min_by`
  keeps the first minimum), and compare floats with `total_cmp`.
- **No hash containers**: don't use `HashMap` or `HashSet`, because their iteration order is random. Use `BTreeMap`,
  `BTreeSet` or a sorted `Vec`.
- **No outside world**: no wall clock, no threads and no OS randomness. The only source of randomness is the sim's
  PRNG (`sc_sim::Rng`, PCG32), seeded from the game setup's seed (`GameSetup::seed`). It is part of the hashed
  state. Draw from it only where the design asks for chance, and only when there is a choice to make; its one draw
  so far is a producer's exit door (see *Production*).
- **Floats**: `f32` is fine. `+ - * /`, `sqrt`, `abs`, `floor`/`round` and `as` casts give identical results
  everywhere, but transcendental functions (`sin`, `cos`, `atan2`, `exp`, `powf`, …) don't. Call them through
  `sc_sim::math`, which wraps the pure-Rust `libm` crate, and add any missing function there. `powi` is banned too,
  so multiply explicitly. glam is built with its `libm` feature, so glam's own methods are safe.
- **No engine, frames or logging**: the sim never depends on Bevy, never sees frame time, and never logs or prints.
  Only `Sim::step` advances it.
- **Money and progress are integers**: banks are `u32`, rates and progress fixed point in `RATIO_ONE` (2^16) a tick
  (plan decision T6, see *Economy*). No floats in the economy.

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

## Map model

The design is `core/map.md`, and the code is `crates/sim/src/map.rs` and `map/`.

- **Tiles**: a `W × H` map has one `Tile` per tile, which is just a `TilePresetId` into the map's preset table: the
  five default presets (`core/map.md > DefaultTilePresets`) at fixed indices, then the map's custom presets. A tile has
  no height of its own. Map sides run from 1 to `map::MAX_MAP_SIDE` (256, a placeholder), checked when a file is read,
  before anything is allocated, and by `MapDef::validate`.
- **Presets** (`core/map_editor.md > Custom Tile Presets`): a `TilePreset` has a name, its five properties and a
  `TileLook` (Plane, Rugged, Cliff, Mountain or Water), which is all the renderer draws by. The default presets are
  fixed. `Map::add_preset`, `set_preset` and `remove_preset` edit the custom ones; removing one moves the later ids
  down and renumbers their tiles, and `preset_use_count` says whether any tile still uses it.
- **Corner heights** (`core/map.md > Elevation`): a `(W + 1) × (H + 1)` grid of levels 0–16 (`CornerPos`,
  `Map::corner`, `TilePos::corners()` in NW, NE, SW, SE order), shared by adjacent tiles, so the terrain is
  continuous. `Map::height_at(p)` interpolates bilinearly inside the tile that contains `p` (written from the NW corner,
  so a flat tile gives its level exactly; off the map it takes the nearest point on it). `elevation_level` floors it. A
  tile is **flat** when its four corners are equal (`is_flat`, `tile_level`), else **sloped**; `tile_slope` is its
  highest corner minus its lowest.
- **Maximum slope**: a Traversible tile may slope at most `MAX_TRAVERSIBLE_SLOPE` (1) level; `is_too_steep` and
  `too_steep_tiles` find violations, and `MapDef::validate` lists them (`ScenarioProblem::TooSteep`). Non-Traversible
  tiles (Cliff, Mountain, Water) may be as steep as they like.
- **Map resources and structures need flat ground** (`core/map.md > Elevation > Map resources only on flat terrain`,
  `core/map.md > Structures only on flat terrain`): every footprint tile of a kind with `ObjectDef::needs_flat_ground`
  must be flat. Validation flags it (`SpawnProblem::NotFlat`) and `Command::Debug(Spawn)` refuses it.
- **Height edits**: `Map::set_corner_heights(targets)` pins the target corners and moves every other corner as little
  as needed for the maximum slope (two corners are linked when they share a Traversible tile, so slopes cascade
  outwards, AoE2-style, and stop at non-Traversible tiles). It returns the corners it changed. On a map without
  violations, targets that keep the rule among themselves (all at one level, or each one level from where it was)
  leave none. `Map::resize` keeps the corners that remain, and new ones continue the old edge flat outwards.
- **Symmetry** (`core/map_editor.md > Symmetry`): `sc_sim::Symmetry` (six modes; 90° rotation only on square maps) gives
  its `MapTransform`s, pure functions that map tiles, corners (the corner grid is one larger than the tile grid),
  points, footprints and building placements (rotation included) to their images, and its `EditableRegion`, the part
  a map maker edits. `MapDef::regenerate` (`scenario/generate.rs`) builds the rest of a symmetric map from that part:
  see Sim scenarios and tests.

## Object model

Units, structures and resources are all `Object`s (`object.rs`, `core/entities.md > Object Instance`). The layout is
"ECS-lite": a plain struct with core fields plus one `Option` per capability.

- **Core fields**: `id`, `kind`, `owner` (`None` means neutral), `pos`, `prev_pos`, `facing`, `prev_facing`, `hp`
  (`None` means indestructible) and `shields` (the current Shields of an object whose kind has MaxShields, the
  Colonists' kinds; `None` for the rest; see *Combat > Shields*).
- **Capabilities**: today these are `placement` (the tile footprint), `resource` (`ResourceState`), and for units
  `orders` (`Orders`, `orders.rs`: the current order with its behaviour's state, a queue of at most
  `MAX_QUEUED_ORDERS`, and what the unit does on its own while it has none; see *Behaviours*), `mover`
  (`Mover`, `mover.rs`: goal, status, path, the replanning state of `docs/movement.md > 3`, the sidestep of a
  yielding unit and a moving unit's contact step; `mover/` holds the stuck schedule, the two steps and the hash), for units that
  can attack, `attack` (`AttackState`, `combat.rs`: see *Combat*) and, for units that carry a Riot Shield,
  `riot_shield` (`RiotShield`, `riot_shield.rs`: how far up the shield is, the way it goes being the unit's type,
  see *Combat*); for a plate on a patch,
  `extractor` (`Extractor`, `economy/mining.rs`: the patch it mines and its cycle, see *Economy*); for a structure
  that produces units, `production` (`Production`, `production.rs`: its queue, the progress of the unit in
  production and its rally point, see *Production*); for a structure that constructs buildings, `constructor`
  (`Constructor`, `construction.rs`: idle, constructing or holding a building ready to place, see *Economy*); for
  a worker, the Drone or the Agent, `worker` (`Worker`, `economy/gathering.rs`: its load and its AssignedDropOff,
  see *Economy > Gathering*); for an object transforming into its kind (a Beacon warping in a building, a
  Beacon deploying from a Drone), `transform` (`Transform`, `transform.rs`: what it was and where it stood, how far it
  has come, what it paid and the load a deploying Drone carried, see *Economy > Warp In* and *> Beacon Deployment*);
  for a Legionnaire whose form isn't its type, `conversion` (`Conversion`, `conversion.rs`: the form it has, and the
  frames left of the conversion under way, none while it waits for its Cooldown; see *Type and form* below and
  *Combat > Weapon Conversion*); for a Tunnel, `tunnel` (`TunnelState`, `tunnel.rs`: its tier, plan decision T39;
  see *The Syndicate* below). The rest arrives as more optional fields, each with its own module. Never add them as separate
  stores keyed by id. A system walks the objects in id order and skips those without its capability
  (`let Some(..) = &mut o.resource else { continue };`).
- **Per-type constants** sit in one `static ObjectDef` per type (`core/entities.md > Object Type`), in the type's own
  def module, named as its design file: `defs/<faction>/{units,structures}/<object>.rs`, and `defs/map/<object>.rs`
  for the map's objects (*Adding a unit or structure*). Reach them through `ObjectKind::def()` or `Object::def()`.
  Every value cites its design section. A faction's own rules, which belong to no one type (its caps and divisors),
  are in its root module, `defs/<faction>.rs`. What a type does differently is a field of its def where it can be:
  slice M4-R1 turned three `match`es on `ObjectKind` into `StructureDef::onto` (the kind a structure is placed onto),
  `StructureDef::seeds_build_area` and `UnitBaseDef::transit` (which `Transit::of` reads). **The defs are plain serde
  data** (plan decision T54, for the data files of playtest; `defs/tests/serde.rs` takes every def through JSON and
  back): their names and lists stay `&'static str` and `&'static [T]`, which the deserializer leaks once
  (`defs/leak.rs`), so the defs stay `Copy` (the Director accepted this for the playtest loader). A unit type also
  has `ObjectDef::unit: Some(UnitDef)` (`defs/units.rs`, `systems/units.md`): its `UnitBase`, whose `static UnitBaseDef`
  holds the base's domain, movement model and flags (turret, crushable, …), the attributes of that movement model
  (`MovementAttrs`), and its turret, attack and costs. Its box is its silhouette (`ObjectDef::half_extents`). Armour
  (`ObjectDef::armor`, an `Armor` of PointArmor and FullArmor) is the same for units and structures; map resources have
  `Armor::NONE`.
- **Structures** (`defs/structures.rs`, `defs/gdo/`; `factions/gdo/structures/`, `factions/gdo/economy.md`): the
  GDO's Deployment Center, Generator, Barracks, Extraction Facility and Extraction Plate (slice M2-A; their
  `ObjectKind`s follow the M1 ones, because the state hash numbers kinds in declaration order). A structure type has
  `ObjectDef::structure: Some(StructureDef)` (its `SymmetryType`, BuildRadiusExtension and Power, the plate's
  `MiningDef` in `StructureDef::mining`, and a constructor's `Constructs` list of `Recipe`s in
  `StructureDef::constructs`, see *Economy*) and a tile footprint, `ObjectDef::tile_size` as unturned;
  its `Placement` holds it as placed, `w` and `h` swapped by a quarter turn, with the orientation it was placed in
  (`Placement::orientation`). It has HP, armour and a SightRange, and no orders, mover or attack, so behaviours,
  movement and combat pass it by.
- **The Colonists** (`defs/colonists/`; `factions/colonists/units/`, `factions/colonists/economy.md`, slice M3-A):
  the Drone, the Beacon, the Egress Node, the Robo Portal, the Bio Portal and the Sabre and Rifle Legionnaires, after
  the M2 kinds. Three def fields came before their slices: `ObjectDef::max_shields` (MaxShields, set for exactly
  the Colonists' kinds; their Shields came with M3-B, see *Combat > Shields*), `UnitDef::beacon_capacity_cost`
  (M3-C) and `StructureDef::warp_in`, the Beacon's Constructs List (M3-E). `warp_in` is not `constructs`, so the
  Beacon is no GDO-style constructor (plan decision T14). Their structures have no Power and no build area. The
  portals produce (slice M3-D) through an energy doorway, out through their A sides, all four of the Robo Portal's
  (slice M3-D2, design change 0145, see *Production*). The Drone, a
  HoverVehicle, moves by DragMovement (slice M3-G, see *Movement and collision > Movement models*): its equilibrium
  is `(ForwardAcceleration + OmniDirectionalAcceleration) / DragRatio` = 8 su per tick (`MovementAttrs::max_speed`).
  It crosses Rugged tiles with double drag (`HOVER_VEHICLE.rugged_terrain`, design change 0039), on the light
  infantry's `(GroundRugged, 1)` grid.
- **The Syndicate** (`defs/syndicate/`; `factions/syndicate/`, slice M4-A): the Guard (M3), then the Agent, the
  Tunnel and the Hideout, after the Colonists' kinds, the Optics Lab and the Chemical Plant (slice M4-K, *Tech
  prerequisites* below), and the Grenadier (slice M4-U), which throws grenades (*Projectiles*), trained at the Hideout
  behind a Chemical Plant. The Agent has its pistol; units enter and leave the Tunnel
  Network at a Tunnel's Side A and the Hideout produces into it (slice M4-C, *Units off the map* below); the rest is
  inert so far but for blocking, sight and being targets. **The Tunnel is one kind with a tier** (plan decision
  T39): `StructureDef::tunnel` holds
  its `TunnelDef`, the rows of `factions/syndicate/tunnels.md > Tiers` (`TierDef`: MaxHP, Tunnel Area Radius, Transit),
  and the object its `TunnelState` (`tunnel.rs`), from Tier 1. `Object::max_hp` reads its tier's row; `tunnel::set_tier`
  changes the tier, the damage kept (`transform::follow_max`, design change 0231 point 4), through `Sim::set_tier`: an
  upgrade's completion (slice M4-G, *Tunnel upgrades* below) and `DebugCommand::SetTier`, which ends an upgrade under
  way, unrefunded. Records carry the tier last seen (`LastKnown::tunnel`, 0237). **The Hideout is an underground
  expansion**, as are the Optics Lab and the Chemical Plant (`StructureDef::underground`, plan decision T32): it needs
  no flat ground (`ObjectDef::needs_flat_ground`, design change 0005), stands for the ElevationModifier at the level
  under its centre, blocks no movement (*Navigation*) and is hidden from the other players (*Vision > Hidden
  objects*). Surface and underground footprints never share a tile (design change 0240): `may_overlap` keeps them
  apart as any two footprints, and so do the placement methods of slices M4-E and M4-F (*Placement*).
- **The Cults** (`defs/cults/`; `factions/cults/`, slice M5-A, plan decision T55): the Recruit, the Militant and the
  Gunner, the Recruitment Hall, the Storage and the Armory, after the Syndicate's kinds, with their numbers and their
  client kinds, and none of the faction's systems: **each system's slice adds its own field** to their defs (the
  Recruit's `worker`, the Storage's `drop_off`, every structure's `site`, the Hall's and the Armory's `production`,
  the Armory's holding; plan §3.2, "one field per slice"), and `attack` and `popup` stay `None` on every Cults
  structure. The Armory's `requires: [Storage]` is data until Recruits build (slice M5-E). **A Cults unit's Unit
  Control cost** is the Recruits it takes (`factions/cults/structures/recruitment_hall.md > Unit Control Tracking`),
  kept in the GDO's `UnitDef::unit_control_cost`, 1 for each infantry unit (`defs/cults/tests.rs` checks every Cults
  unit's), and counted in the Cults' own Unit Control alone (`ArmyResource::CultsUnitControl`, slice M5-C; *Economy*).
  **The lineage**, `Object::lineage: Vec<ObjectId>` (plan decision T56 as the Director amended it for design change
  0319): one entry per Recruit the unit embodies, the Hall that made it, so a Recruit has one and a Seeker will have
  two, perhaps the same Hall twice; hashed (a tag) and saved only while it has an entry, kept whole by
  `Sim::change_kind`, to which a pool building's training adds the lineages of the other Recruits it consumes (slice
  M5-F2, *Production > Pool buildings*), and set by the Hall that releases the Recruit (slice M5-C, *Production > The
  Recruitment Halls*; by the layout at a game's start, M5-B). An entry whose Hall is gone stays: it is an orphaned
  point, and so is each point of a Cults unit's cost that no entry stands for (`recruitment::Lineages::of`: the cost
  less the entries), so a unit without an entry is orphaned whole (design change 0344: "Recruits no Hall produced
  (custom maps, debug) are orphans from the start"); `testing::references` doesn't check an entry as a reference for
  that reason. **Every other Cults kind** (slice M5-G2, plan decision T69: the last slice of M5 that adds a kind)
  follows, in the tech tree's order, then design changes 0361 to 0365's two vehicles: the Gunworks, the Garrison, the
  Chop Shop, the Machine Shop, the Airstrip, the Scrap Hangar, the Altar (`factions/cults/augmentation.md > Altar`),
  the Temple, the Cathedral and the Shrine; the Disciple, the Seeker (design change 0364, once the Jackal), the
  Sparrow, the Hand of Heaven and the Prayer-mobile; one def module each, with its design file's "Placeholder stats",
  its `site` and its `requires`. `defs/cults/tests.rs` lists them and checks the list against the design folder (every
  structure of `factions/cults/structures/README.md > Tech Tree Overview`, every file of `factions/cults/units/`). Each
  of the others' slices adds its behaviour: the Chop Shop, the Airstrip and the Cathedral hold 10 Recruits already
  (`HoldsDef`, design change 0347 point 5: an editor-placed one can be entered, with no panel yet, so no Eject All),
  and the Recruits build none of the seven until their slices (the Construct menus hold their slots, *Client
  architecture > The Recruit's panel*). The units' `MovementAttrs` (the Seeker's and the Prayer-mobile's
  `FixedTurnRadius`, the Hand of Heaven's `SpeedTurnRadius`, the Sparrow's `Glider`) are data that nothing reads yet:
  a model the movement doesn't build steps without a turn limit (*Movement and collision > Movement models*), and
  nothing produces those units before their buildings' slices. Their `TurretAttrs` are their turrets' since slice
  M5-U1 (*Combat > Units' turrets*). The wheeled and tracked
  vehicles don't cross Rugged terrain, so every game's set of grids holds a `(Ground, 1)` grid beside
  `(GroundRugged, 1)` (*Navigation*); the Sparrow is the first air unit kind (`nav_class` `None`: it crosses every
  tile). **The Augmentations** (slice M5-F2; `augmentation.rs`, `factions/cults/augmentation.md >
  AugmentationState (per unit instance)`): `Object::augmentation: Augmentation { constitution, metabolism }`, hashed (a
  tag) and saved only while one is set, so the states without one, the goldens among them, are as they were; its
  `RecruitKind` (Plain, Constitution, Metabolism or Both, design change 0337) is what a pool building's orders choose
  by. Nothing gives one yet but `DebugCommand::SetAugmentation` (the Altar's is a later slice); their effects are
  read through accessors since slice M5-N1a (*Augmented stats* below). The Gunner is the first
  kind with a MinRange, which M3-L2's code serves (*Behaviours*; M5-A gave a unit that can't back off its "Target too
  close", design change 0204 point 3; M5-O3 took the back-off on to the middle of the band, question Q0054, and gave
  an Attack Ground's location the same rule), and the first TailDisjointed attack (slice M5-O3, *Combat > TailDisjointed
  attacks*), which can miss; the Armory trains it for a player who owns a Gunworks (slice M5-G2, `Recipe::requires`,
  *Tech prerequisites*). The Storage is 1/4 on its 2x2 since slice M5-G2 (Q0094 a), so the building armour test has
  no exception left. The Cults start a game since slice M5-B (plan decision T63;
  `tests/cults_start.rs`, *Sim scenarios and tests*): no faction is "not implemented" any more.
- **Units off the map: the held store** (`network.rs`, `sim/network.rs`; slice M4-C, plan decision T30;
  `factions/syndicate/tunnels.md > Tunnel Network`): a unit inside a Tunnel Network is **not among the objects**. It
  is a `Held { object, place, since, entered }` in `Sim::held`, a second map by id beside `Sim::objects`, with the
  id it had. So every system that walks the objects (movement, collision, vision, targeting, auto-targeting, picking,
  records, the client's visuals, which diff against `Sim::objects()`) passes it by without a filter, and a forgotten
  case fails closed: it can't show, move or hit a unit that isn't there. The one exception to "never a separate store
  keyed by id" above, for that reason. **What must count held units says so**: `Sim::army_cap` (their army cost,
  Tunnel Space), the removal path's reference cleanup (`Sim::forget_references` walks them too, so a held unit never
  names an object that is gone), `Sim::clear_defeated_networks` (a defeated player's go, `Sim::remove_held`, which
  slice M4-H's stranded-unit rule calls too), the state hash and the saves, `testing::dangling_references`, the
  client's control groups (`ControlState::retain_on_map`). `Place` is where: `Network`, or `Site(site)`, a worker
  inside the site it builds (slice M4-E, *Sites* below: not in the network, so no Eject brings it out, and it doesn't
  regenerate), or `Building(building)`, a unit inside a building that holds units (slice M5-F1, *Buildings that hold
  units* below); match it exhaustively and never assume a held unit's place is on the map (a transport's passengers
  will be held at a carrier that may itself be held). `Sim::take_off_map(id, place, entered)` and `Sim::put_on_map(id,
  pos, facing)` are the only ways off and back: the first ends every reference to the unit as a removal does, without
  a death's effects (no `Died`, no `Removed`), and clears the unit's own orders (a Build among them refunded,
  `Sim::drop_orders`), move and attack (design change 0249), its HP, load and modes kept; call it before 5b, as a
  change of kind. `Sim::held()`, `held_unit(id)`, `network(player)`,
  `network_counts(player)` (the EjectMenu's tiles: unreserved units by kind), `reserved()` and `next_out(player,
  kind)` are what the interface reads. Hashed and saved **only while non-empty** (`#[serde(default,
  skip_serializing_if)]`, a tag in `Sim::state_hash`), as a Tunnel's eject queue is (`TunnelState::eject`), so the
  states without a held unit, the goldens among them, are as they were. Inside, a unit regenerates 1% of its MaxHP
  a second (`TunnelDef::regen_per_second`, placeholder, 0249; `crate::regeneration`, the rule of the expansions'
  regeneration too since slice M5-N8) and may leave `TRANSIT_DELAY_TICKS` (32, placeholder, 0245) after it walked in, a
  unit produced inside at once (0246; `Held::may_exit`). **Measured** (`tests/agents_perf.rs`, ignored, release; slice
  M4-M): 4 Tunnels with 4 Agents each on two shared fields and 60 Guards cycling through Enter and Eject, a mean of
  0.043 ms and a p99 of 0.067 ms a whole sim tick; then 40 Grenadiers shelling a Tunnel field, 0.070 and 0.090 ms (load
  average 0.3 to 0.6).
- **Sites** (`construction/site.rs`, `sim/sites.rs`; slice M4-E, plan decision T31; `factions/syndicate/units/agent.md
  > Building > Tunnel Construction Flow`, design change 0234): a structure a worker builds where it stands is, from its
  first frame, the object it will be, with `Object::construction: Option<Construction { progress, builder }>` while it
  is a site: a `Progress` over its def's frames (`SiteDef`, `StructureDef::site`) and `Builder::Held(worker)`, the
  worker held at `Place::Site(site)`. Hashed (a tag) and saved only while set, so the states without a site are as they
  were. `Object::max_hp` gives the ConstructionHP rule's maximum (`SiteDef::start_hp`, 10%, growing with the frames),
  and the HP follow it up with the damage kept (`transform::follow_max`). **`Object::is_operational`** (neither a site
  nor transforming) is the one predicate of a structure doing what its type does: the exits and Enter
  (`network::is_exit`), the drop-offs (the gate, `Sim::nearest_drop_off`, the resolution's facts) and Tunnel Space
  (`gives_tunnel_space`) all ask it, so that a site does nothing but Cancel. A site counts for defeat, blocks and is
  seen as any structure; the worker inside, held, is not. (Slice M4-S1's Laser Turret site of an Agent's went with
  design change 0359: a turret's site is a Tunnel's expansion's, hidden, *The pop-up form*; it holds its fire until
  complete, *Combat > Structures that attack*.) (M5-E) A Cults site is `Builder::Recruits`: nobody is held, and it
  gains a frame for each Recruit building it from outside (*Economy > Cults construction*); its functions are held
  back while it is a site (`sim/transform.rs`, `hold_back`) and start as it completes (`Sim::start_functions`), so that a Hall
  claims and produces from then on (`Production::completed_at`).
- **Buildings that hold units** (`holding.rs`, `sim/holding.rs`, `sim/holding/evacuation.rs`; slice M5-F1, plan
  decision T60; `factions/cults/structures/armory.md`, `factions/cults/economy.md > Cults Building Mechanics >
  Evacuation on Destruction`, `systems/controls/commands.md > Enter` with design change 0319's TargetObject): **one
  capability, data on the def**, `StructureDef::holds: Option<HoldsDef { entrance, exit, capacity, accepts }>`: the
  side lettered `entrance` is the way in, `exit` the `ExitRule` of the way out (`ExitRule::Side` only, for now; the
  defs test checks it, and that a type that also produces has the same rule in both), `capacity` bounds the pool,
  `accepts` the kinds let in. The Armory is the one so far (B in, C out, 10, Recruits). The units inside are in the
  held store at `Place::Building(building)` (`Sim::held_in`: longest inside first, by `Held::since` then id), so
  everything the store does for a Tunnel's (references, the hash, the saves, a defeat) holds for them too.
  - **In**: M4-C's Enter order, generalized, not forked. `network::admits(target, kind)` asks `holding::admits` of a
    building that holds units (placed, owned, `Object::is_operational`: a site refuses, `Reject::UnderConstruction`
    at the gate; design change 0347 point 4: "a building under construction **can't be entered**; a right-click on
    it Assists", *Economy > Cults construction*; `accepts` names the kind, else `NoTransit::NotAccepted`,
    `Reject::NotAccepted` when none of the units is let in). The walk ends at `network::EntryPath` (`outward`, `spot`;
    `stand(half)`, `reached(pos, half)`), `EntryPath::of_object`: a Tunnel's from `Paths::entry_path`, a holder's
    the middle of its entrance side (`holding::entry_path`; 0347 point 1: "a dedicated doorway at the middle of the
    entrance side": a unit goes in once its box is within `ENTRY_REACH_SU`, 8 su, of that one point, and nowhere
    else along the side, `entering.rs`). A Tunnel's units walk up
    against the wall (M4-C, unchanged); a holder's stop `holding::ENTRY_STANDOFF_SU` (4 su, within the reach) clear
    of it (`EntryPath::standoff`): against it, a Recruit pressed along the wall by the crowd coming round the corner of
    the entrance side slid ever more slowly towards a spot it couldn't get round the corner to, and gave up with the
    one behind it (`entering.rs`, the QA run's game). **The room is judged on
    arrival** (`Sim::room`: capacity less everyone inside, those an order reserves and those on their way out
    included, design change 0347 point 5: "Capacity counts reserved units"), not at the gate: a unit that gets there
    with no room loses its order and idles beside the building (`SimEvent::EnterFailed { full: true }`, the client's
    "No room inside" line; 0347 point 2: "a unit arriving when the building is full has its Enter **end** and idles
    beside the building (the normal Failure rule)"). The right-click resolves an own operational holder that takes
    the unit's kind to Enter (`Aimed::accepts`, `Abilities::kind`; "Own Armory → Enter Armory").
  - **Out by Eject All** (`Command::Holding { objects, command: HoldCommand::EjectAll }`, gate `Sim::check_holding`:
    `Reject::NoHolders`, `Reject::NothingInside`): it marks every unit of the pool ready to leave, `Held::leaving`
    (the tick it was marked; hashed and saved only while set), and `Sim::step_holding`, in the economy phase right
    after `Sim::step_network`, lets each building's unit ready first out, in id order of the buildings, through its
    exit doorway (`Sim::let_out`, `Sim::release_held`, a producer's release: `doorway_spot`, `Mover::exit`, one at a
    time, only once the doorway is free; of several doors, the first free one in `Doorway::order`, slice M5-C's rule;
    `SimEvent::LeftBuilding`): "in the order they are ready" (design change 0348 point 4), ties by the longest inside,
    then id. A pool building's trained and released units are made ready the same way (*Production > Pool
    buildings*), so one doorway serves them all in that order. A unit that goes in meanwhile isn't marked, and stays,
    and so do the reserved ones. Out, the unit goes to the building's RallyPoint with the rally's order (0348 point 3:
    "Ejected and released units walk to the RallyPoint"), or straight out to idle without one (`Sim::rally_order`).
    `production::exit_rule` gives a building's doorway its rule: production's, else the holder's.
  - **Out by destruction** (slice M5-F3, design change 0349: "evacuated units **all appear at the same time inside
    its former footprint**, unharmed, spread over it. Units that overlap ignore each other until separated, and idle
    ones push apart"): in `Sim::remove_objects`, `Sim::evacuations` reads each going holder's units first, longest
    inside first (`Sim::held_in`; 0349: "No placement order"), and takes a finished front entry off its queue
    unrefunded (`Sim::finished_training`: "A unit that had finished training [...] comes out **as the trained unit**,
    with no refund"); then `Sim::refund_production` refunds every other entry in full, so that the units they
    reserved come out as Recruits, a Recruit mid-training among them. `Sim::evacuate` puts them all on the map in that
    call, once the building has left the game and every reference to it is gone: each on the centre of the next tile
    of the footprint, row by row from its north-west tile, west to east (`Placement::tiles`), round again past the
    last (`Sim::footprint_spot`; Placeholder, not in the design: how they spread over it), its box moved in to lie
    inside the footprint, a tile whose terrain the unit can't stand on passed over (`nav::box_on_terrain`, the
    terrain alone: the grids still have the footprint until `paths::sync`). The finished training's unit is trained
    there (`Sim::train`, its `KindChanged`), the others are as they were; each faces as it did, idles (0349 point 4)
    and has a `SimEvent::Evacuated` after the building's `Removed`; so before the defeat check of the tick, and a
    defeated player's evacuees turn neutral on the map with its other units. They overlap, which I1 allows in the
    tick they come out (`invariants::may_overlap`, *Movement and collision > Collision exceptions*), and part by the
    movement's own rules: overlapping units ignore each other (`space::blocks`), idle ones step apart
    (`movement/separate.rs`), from the movement of the same tick on (ten Recruits in an Armory are apart within 5
    ticks). Neutral ones don't: no unit yields that has no owner (`yielding::can_yield`), so a defeated player's
    stay as they came out. One for which no tile of the footprint is ground it can stand on (buildable terrain it
    can't stand on) leaves the game (`Sim::remove_held`; Placeholder, not in the design).
  - **Measured** (`tests/holding_perf.rs`, ignored, release): 8 Armories with 10 Recruits each cycling through Enter
    and Eject All, a mean of 0.028 ms and a p99 of 0.071 ms a whole sim tick (the economy phase, `step_holding`
    with the rest, 0.001 ms); the tick that removes all 8 with 80 inside, 0.22 ms. `agents_perf` (the Tunnels'
    Enter) and `recruits_perf` ran A/B against master unchanged: the same games, the same timings (load average 3.9).
    Slice M5-F2 (its ready order, the pool producers' step) ran `holding_perf`, `structures_perf` and `recruits_perf`
    A/B against master: the same games, the same timings within noise (load average 2.4; `recruits_perf`'s tick 1, a
    single cold sample, varies with the build, 0.28 to 0.42 ms on either side). Slice M5-F3 (the evacuation inside
    the footprint) ran `holding_perf` and `recruits_perf` A/B against master, interleaved, three rounds each: the
    same games, the same timings within noise (cycling, 0.030 to 0.037 ms a tick on either side; `recruits_perf`,
    0.076 to 0.079 ms), but for the tick that removes the 8 Armories with 80 inside, 0.22 to 0.23 ms before and 0.135
    to 0.141 ms now (its commands phase, where the removal is, 0.14 ms before and 0.04 ms now: no search round the
    footprints) (load average 2.0). Slice M5-Z1 (the doorway's tile test, `Frame::blocks`) ran both A/B against
    master, interleaved, 13 rounds each: the same games, the same timings (medians on either side: cycling 0.030
    ms a tick, the evacuation's tick 0.129 and 0.131 ms; `recruits_perf` at work 0.076 ms, its Movement 0.032 ms)
    (load average 2.7 to 3.8).
  - **The pool** (`Sim::pool`, the Armory's StoredRecruits) is the units inside that no entry of the building's queue
    reserves and that aren't on their way out (slice M5-F2); `Sim::held_in` is everyone inside. The client's Enter
    pointer is slice M5-J1's (*Client architecture*, *The Recruit's panel*). Design change 0319's other holders fit
    the def as it is: the Chop Shop, the Cathedral and the Airstrip have their `HoldsDef` since slice M5-G2 (in at B,
    out at C, 10 Recruits), the Altar, the Temple and the Shrine (all ABAC now) get theirs with their slices, and the
    Gunworks and the Machine Shop (ABAC too) hold no one until their research is designed (design change 0328), so a
    right-click on one with Recruits is a Move. The Garrison's "units leave through any side, at the
    first clear spot next to the footprint" is either slice M5-C's `ExitRule::Sides` (a door on every side, the first
    free one, which `Sim::release_held` already serves; the defs test allows holders `ExitRule::Side` only, for now)
    or, as slice N2 will build it, the evacuation's (`Sim::evacuate`, *Out by destruction* above).
- **Expansions and a Tunnel's operation** (`sim/expansions.rs`, `tunnel/operation.rs`; slice M4-F;
  `factions/syndicate/tunnel_menus.md > ExpandMenu`, `factions/syndicate/tunnel_menus.md > AwaitingPlacement
  (Expansion)`, `factions/syndicate/tunnel_menus.md > Expansions under construction`; design changes 0240, 0254, 0290,
  0291): a Tunnel builds an underground expansion in its area as **its one operation**, `TunnelState::operation:
  Option<Operation>` (`Operation::Building(site)`, or `Operation::Upgrading { progress, to }`, *Tunnel upgrades* below),
  hashed (a tag) and saved only while set. The
  expansion is a site as above, `Builder::Tunnel(tunnel)`, no worker held.
  `TunnelCommand::Expand { kind, origin, orientation }`'s gate (`Sim::check_expand`) is first **`Sim::can_expand`** (the
  player's Tunnel in operation, no operation under way `Reject::OperationUnderWay`, the kind's Tier Requirement
  `Reject::TierRequirement`, its TechPrerequisite `Reject::Requires`, *Tech prerequisites* below, then
  the bank), which the ExpandMenu greys its entries by, then the placement by the truth (`Sim::expansion_truth`,
  `Reject::Unplaceable`). Paid at the click, Tunnel Space used and the ConstructionHP rule from the start (0254);
  completion (`Sim::complete_expansion`) ends the operation, and so does a cancel, the site's X or the Tunnel's
  (`TunnelCommand::Cancel`, `Sim::cancel_operation`; 75% back, `Refund::Expansion`), or the site gone any other way
  (`forget_references`, no refund). **The ExpandMenu's one table** is `defs::EXPAND_MENU` (`ExpandEntry::Kind(kind)` or
  `ExpandEntry::Planned(name)`): the nine expansions of the Tech Tree table in its order and the Laser Turret tenth
  (its `Kind` since slice M4-S2: a Tunnel builds it as an expansion, *The pop-up form* below; the only builder of it
  since design change 0359), each holding its slot (0291); the slice that brings one turns its entry into its `Kind`,
  and tests check the list against the design's table and against `StructureDef::expansion`.
  `Sim::operation_progress(tunnel)` gives the operation and its `Progress` for the Tunnel's InfoPanel.
- **The pop-up form** (`popup.rs`, `sim/popups.rs`; slices M4-S2 and M5-Y, plan decision T50;
  `factions/syndicate/structures/laser_turret.md > Pop-up turret`, `factions/syndicate/structures/laser_turret.md >
  Pop-up rules`, `factions/syndicate/structures/laser_turret.md > When its Tunnel is lost`; design changes 0267, 0296,
  0297 and 0358 to 0360). **Every turret is a pop-up** (0360: "There is no upright form any more"): an object of a
  type with `StructureDef::popup` (`PopUpDef`: the rise, idle and retract frames and the underground armour; the Laser
  Turret's 8, 32, 8 and 6 / 6) has **`Object::popup: Option<PopUp>`** (`Lowered { blocked }`, `Rising(Progress)`,
  `Raised { idle }`, `Retracting(Progress)`) from its first tick, lowered (`Sim::new_object`), its site too, so the
  state is hashed (a tag) and saved only for a turret. **Active is derived, never stored**: a turret in operation is
  active while it has a parent Tunnel (`popup::is_active`, `Sim::is_active_turret`: the expansions' rule,
  `tunnel::area::parent`, an own Tunnel in operation whose current area encloses it; a reading: "in operation", as for
  reclaiming, and an upgrading Tunnel's area is its current tier's), so a new Tunnel, an upgrade and a Tunnel's loss
  change it with no event. **An inactive turret is lowered at once** (a reading of "a raised turret retracts at
  once": in the tick, with no retract frames, so that its armour is the underground one from then) and never fires:
  a neutral one (*Losing Tunnels*), and one a map or a debug spawn puts outside its owner's areas. **The cycle** of an
  active one is `popup::step`, once a tick in `Sim::step_popups` (*Tick pipeline*): lowered with no target; rising for
  its frames once phase 5e picked one, then raised, where alone it fires (`popup::may_fire`, which `combat::step`
  asks: it keeps the target it picked and faces it meanwhile); retracting after its idle frames with no target. A
  phase started in tick `t` does its first frame in `t + 1`: a target picked in `t`, raised at the end of `t + 8`, the
  first shot in `t + 13`; the last target in `t`, down at the end of `t + 40`. Every phase runs its full length (0358
  point 2: "a new target during retracting doesn't interrupt it"). **Each reader asks one function**: hiding and the
  underground armour only while fully lowered (`popup::is_lowered`: `Hiding::of`'s arm, which hides an owned turret
  alone, *Vision > Hidden objects*; **`Object::armor()`**, which every hit meets through combat's snapshot), blocking
  (`popup::blocks`, below), Tunnel Space (`economy::army::occupies_tunnel_region`: a cell of the region, so ¼, for
  every expansion in the region, a turret's site from its first frame, 0358 point 5). **Blocking** (0296 point 3, a
  reading): the footprint blocks from the tick the turret starts to rise until it is fully lowered again, so that
  nothing steps under it while it rises; `paths::blocking(object)` is the one test of a live object's footprint (the
  true grids, its owner's known ones, the gate's spawn checks), `paths::remembered_blocking` a record's (a neutral
  turret's record, always lowered, is walked over: `popup::remembered_lowered`), and `step_popups` syncs the true
  grids when a turret starts or stops blocking, as a structure that appears or goes (*Navigation*). **Rising needs a
  clear footprint**: a lowered turret that wants up while a ground unit's box overlaps its footprint stays `Lowered {
  blocked: true }`, and phase 5d of the next tick asks its owner's idle units there to step off
  (`yielding::make_way`, *Movement and collision > Friendly yielding*); an enemy or a unit with an order keeps it
  down. **A Tunnel's turret** is the ExpandMenu's tenth entry (`StructureDef::expansion`, its one way to be built:
  slice M4-S1's `site`, an Agent's Build, went with 0359), placed by `Method::Expansion` with the ground a surface
  structure needs as well (Buildable, flat and Drillable, 0358 point 3), from Tier 1 (0358 point 4), its site lowered
  from its first frame: hidden, walked over, and taking the Tunnel's one operation (0267). Tests: `popup.rs`'s (the
  cycle), `tests/popup/` (`cycle`, `blocking`, `hidden`, `building`, `active`: 0358's and 0360's Verifications),
  `combat/area/tests.rs` (a blast against each armour). **Measured** (`tests/popup_perf.rs`, ignored, release; 12
  pop-ups in three areas on 256 × 256, their pairs rising and retracting 10 ticks apart every 64, under 200 walkers
  crossing them; four interleaved A/B rounds against the same file on a build without pop-ups, load 2.3): 227 changes
  of the true grids and 495 sets of grids rebuilt in 1280 ticks; a whole tick 0.197 ms on average against 0.179 ms
  (+10%), 0.41 ms against 0.35 ms at the 99th percentile, the worst tick (the group orders) unchanged: about 0.1 ms a
  tick in which a pop-up starts or stops blocking. `nav_perf`, `movement_perf` and `structures_perf`, which have no
  turret, are as they were.
- **Tech prerequisites** (`tech.rs`; slice M4-K, plan decision T37; `core/tech_and_research.md > Tech Prerequisites`,
  design changes 0229 and 0260): **one rule for every faction**. A structure type's TechPrerequisite is
  `StructureDef::requires`, a recipe's "Requires" (a unit's in `Produces`) `Recipe::requires`, kinds leaked from the
  defs (`leak::slice`); `tech::recipe_requires` gives both for a recipe, its own and its building's. **What counts**
  (`tech::counts`, `Sim::requirement_met`): a structure of the type, the player's own and in operation
  (`Object::is_operational`: no site, no transformation; a GDO building ready to place is no object yet), anywhere on
  the map, so anywhere in a Tunnel Network; a neutral one is nobody's. **The gate checks it only as the work starts**
  (`Sim::check_requires`, `Reject::Requires(kind)`): a Train as it is queued (`plan_training`: a producer whose recipe
  isn't met is passed over, and with none left the reason is the first unmet kind), a Construct, a Warp In, an Expand
  (`Sim::can_expand`) and a worker's Build as they are ordered (`Sim::check_build`; the worker's arrival doesn't check
  again: `core/tech_and_research.md > Tech Prerequisites > Check Timing`, "checked only at the moment construction is
  **started** (queued at the constructor, ...)", read as the order, which is when it is paid; slice M4-S1), each before
  the bank, so a greyed entry says "Requires ..." rather than the price. Nothing checks again:
  work queued or started goes on when the last one is lost
  (`core/tech_and_research.md > Tech Prerequisites > Losing a Prerequisite`). Today the Hideout's Guard and the Laser
  Turret (slice M4-S1; the ExpandMenu's alone since design change 0359) require the Optics Lab, and the Hideout's
  Grenadier the Chemical Plant (slice M4-U); the Optics Lab and the Chemical Plant are placeholder expansions (2 × 2,
  150 Space Crystals, 400 frames, HP 400, 6 / 24) that only count. `sim/tech/tests.rs` tests the rule with test defs (a
  GDO unit and building, a Colonists Warp In, a worker's Build, an expansion), `tests/prerequisites/` with the design's
  own types. The client shows `Reject::Requires` as "Requires a/an <name>" (`feedback::requires_text`), every faction's
  entries alike (*The Syndicate's panels*).
- **Losing Tunnels** (`sim/tunnels.rs`; slices M4-H and M5-Y; `factions/syndicate/tunnels.md > Losing Tunnels`,
  design changes 0254, 0258 and 0360): what a Tunnel leaves when it leaves the game, on the removal path (*Removal
  path*): its unfinished expansions (`built_by`: the sites it builds, `Builder::Tunnel`, slice M4-F) are destroyed with
  no refund; its completed ones, its owner's expansions wholly inside its area (`tunnel::is_expansion`: the
  underground structures and the Laser Turret; a Tunnel still a site has none), become **neutral**, seen by everyone
  from that tick: their unit in production lost, the rest of the queue refunded (`SimEvent::ProductionCancelled`, the
  lost one's with 0), the rally cleared, a turret lowered at once with its target dropped (it never fires while
  neutral, *The pop-up form*); then the units in the owner's network that no remaining Tunnel lets out
  (`network::admits`, the one exit rule: a site is none) are removed (`Sim::remove_held`). A neutral expansion decays:
  **`Object::decay: Option<Progress>`**, hashed (a tag) and saved only while set, steps
  `tuning::tunnel::DECAY_PERCENT_PER_SECOND` (1%, placeholder) of a 100-second decay a tick, and its HP follows the
  share of its MaxHP the decay has left (`decayed`, computed from the whole ticks in 64 bits, so that a full Hideout
  dies in its 1600th tick of decay, the first being its Tunnel's last; a turret loses 3 HP a second of its 300). An
  owned one under its Tunnel regenerates instead (slice M5-N8, *Economy > Expansions' regeneration*).
  **Reclaiming** is `Sim::reclaim(tunnel)`, which a site's completion calls (`Sim::complete_site`) and an upgrade's
  (slice M4-G): every neutral expansion wholly inside the finished Tunnel's area, at or below its tier (the Tier
  Requirement, `ExpansionDef::tier`: 1 for the Hideout and the turret), becomes the Tunnel owner's, stops decaying and
  is hidden again, with no event: a turret is active again. **No turret keeps a Syndicate player in the game** (0360:
  "losing its last Tunnel defeats the Syndicate unless it owns some other structure"): it is neutral with the Tunnel's
  other expansions, so the shared rule (*Economy > Defeat and the end of the game*) holds as it is. Tests:
  `tests/losing_tunnels/` (`neutral`: every expansion kind seen by every player at once, the defeat),
  `tests/popup/active.rs` (the turret's decay, fire and reclaim).
- **Tunnel upgrades** (`sim/upgrades.rs`; slice M4-G; `factions/syndicate/tunnels.md > Cost Scaling > Upgrade
  Costs`, `factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule`; design changes 0229, 0231, 0232,
  0237, 0242, 0249, 0283, 0284): `TunnelCommand::Upgrade` takes a Tunnel to the tier above as its one operation,
  `Operation::Upgrading { progress, to }`. **The data** is the tier table's: `TierDef::upgrade: Option<UpgradeDef {
  frames, base, per_tunnel }>` on the tier it reaches (480 frames, placeholder; 2 + 2 per Tunnel, 3 + 3 per Tunnel).
  **One price function**, `Sim::price_to`, counts the owner's other Tunnels whose `TunnelState::reserved_tier` (the
  tier, or the one it upgrades to) is the target or above, never the Tunnel itself: the order pays it
  (`Sim::upgrade_price`, which the panel shows), and the Tunnel's X refunds it as it is then (`Sim::cancel_upgrade`,
  "the current top price"); a Tunnel destroyed mid-upgrade loses it. **The gate**, `Sim::can_upgrade` (the panel's
  Q), as `Sim::can_expand`: the top tier (`Reject::TopTier`), an operation under way, the Supplies, then the truth
  (`Reject::UpgradeBlocked`, never what): the area at the target tier on the map and Drillable (`area::breaches`) and
  clear of every other Tunnel's `area::reserved`, any player's, a site's and another upgrade's target included. The
  owner's lingering ghosts don't block it (a reading: a ghost is no Tunnel, an upgrade no placement), and their builds
  then fail on arrival. **The reserved area**: from the start, the Tunnel holds its target tier's area against every
  placement and upgrade (`Occupant::of`, the truth and its owner's knowledge; the other players judge by the tier they
  see, `Occupant::as_seen`, and records keep only the tier), while transit, exits, Tunnel Space and reclaiming read
  the tier it has (0249). **The frames**: `Sim::step_upgrades` (*Tick pipeline*); completion is `Sim::set_tier` (the
  HP rising with the MaxHP), then `Sim::reclaim`. Events `UpgradeStarted`, `UpgradeCompleted`, `UpgradeCancelled`.
  Tests: `tests/upgrades/` (`costs`, with the books over random orders; `blocked`, every edge; `during`; `completion`)
  and `sim/upgrades.rs`'s (0231's 500/600 to 700/800).
- **Side A's paths** (`network::Paths`, design changes 0225 and 0248): the one geometry of a Tunnel's entry and
  exit, for the sim and the client's drawing (`object_render/tunnel.rs` reads `PATH_OFFSET_SU`): two spots on Side
  A's line, 30 su from its middle, the exit towards Side B and the entry towards Side D, wherever the sides face
  (`sides`). A unit stands on one with its box against the wall (`entry_stand`, `exit_stand`). `network::admits(tunnel,
  kind)` is the one tier rule (`factions/syndicate/tunnels.md > Transit Tier Requirements`; the tier a Tunnel has, which
  an upgrade leaves until it completes, 0249); **air units pass nowhere in M4** (`tuning::tunnel::AIR_DOORS`, whose doc
  holds the rule to come with the first Syndicate air unit, design changes 0250 and 0251). `tunnel::area::parent`
  derives an expansion's parent Tunnel (plan decision T32: the owner's Tunnel whose Tunnel Area, `tunnel::area::of`,
  fully contains it; `Sim::parent_tunnel`).
- **Changing kind** (`sim/kinds.rs`, plan decision T15): an object that transforms (a Drone deploying into a Beacon,
  a Beacon warping in a building, a Legionnaire converting its weapon, a Peacekeeper raising or lowering its Riot
  Shield) stays the same object, its id included.
  `Sim::change_kind(id, kind, placement)` is the one place a kind changes, and it emits `SimEvent::KindChanged`. It
  keeps the id, owner and facing; the position too, unless the object gets a placement, whose centre it then jumps
  to (`prev_pos` too); the HP, clamped to the new MaxHP (callers interpolate HP themselves); and the Shields, clamped
  to the new MaxShields (a kind without any has none, one that gains them starts full). It rebuilds the
  capabilities from the new def with `kinds::Capabilities::fresh`, which `Sim::insert_object` uses as well: a
  capability whose def part is unchanged keeps its state, a new one starts fresh, one the new kind lacks goes. A
  unit that stays a unit keeps its orders and its move (design change 0150), replanning if its grid changes; a
  kept attack keeps its target and restarts its sequence (0152), or keeps its whole state when the weapon in hand
  stays the same (`combat::weapon`: the Riot Shield's two types, and a Legionnaire pressed to convert, which keeps its
  form's weapon until its conversion ends; a shot or a reload goes on). Paid state the new kind has no place for (a production
  queue, a construction) is refunded first, as on removal. Call it before 5b (in the commands phase, or a step
  right after 5a), or sync the grids yourself (`paths::sync`). A caller whose new kind isn't functional yet (a
  building warping in) holds its functions back itself. Slice E's Warp In and its cancel are its first callers
  (`sim/transform.rs`), then a toggle's press (`Sim::use_ability` in `sim/modes.rs`: slice B's Weapon Conversion and
  slice M3-S's Riot Shield, see *Type and form*) and slice F's deployment (`sim/deploy.rs`, the step right after
  5a). `sim/kinds/tests.rs` unit-tests it. The client rebuilds the visuals of
  a changed object and keeps it selected (see *Commands, events and replays > Events*).
- **Type and form** (design change 0198, slice M3-T): a toggle, the Legionnaires' Convert or the Peacekeeper's Riot
  Shield, sets the state the unit works toward ("however fast R is pressed, every unit that received the presses
  ends up in the same form"). Its **type**, `Object::kind`, is that target: the press switches it at once
  (`Sim::use_ability`, `Ability::switched_kind`, through `Sim::change_kind`, so with a `SimEvent::KindChanged`),
  whatever the unit is doing, a shot it waits for included. What commands the unit reads the type: the gate, the
  default command (`orders::resolution`), the client's selection groups, panel and right-click, so a second press
  goes to the units the first reached, as they are now. Its **form** is what it has in the world now, which the
  mode's own state holds: a Legionnaire's weapon (`Conversion::form` while a conversion is pending, else its type),
  a Peacekeeper's shield (`RiotShield::up`, how far up it is). What the unit does in the world reads the form, each
  through one function: `conversion::form(o)` (what the client draws, `object_render::drawn_kind`) and
  `combat::weapon(o)` (its AttackAttributes: 5e, the behaviours, the threat it is), and `riot_shield::mode(o)`, with
  `allows_attack(o)`, `damage_factor(o)` and `speed_factor(o)` from it. The Modes step works the form toward the type
  (`conversion::step`, `riot_shield::step`): at once, or after a shot under way (Firing and Cooldown,
  `combat::mid_attack`; *Combat > Changes that wait for an attack*). Pressed again, a unit waiting takes the new
  target and drops a change that is back to its form (`conversion::retarget`; a deferred raise never starts), a
  conversion under way finishes and converts back, and a shield turns back from where it is. Which orders a unit
  takes follows its type from the press, carrying them out waits for its form (design change 0202:
  `riot_shield::takes_attack_orders(kind)` for the first, `allows_attack(o)` for the second). A Legionnaire's two
  forms share everything but the weapon, so its shape, sight and speed are its type's and its form's alike.
- **Augmented stats** (slice M5-N1a, plan decision T73 as design changes 0330, 0331, 0336 and 0338 amend it;
  `augmentation.rs`, `augmentation/stats.rs`, `augmentation/levels.rs`; `factions/cults/augmentation.md > Increments`):
  a unit's MaxHP, top speed and weapon are read through accessors that apply its Augmentations, and only through them:
  `Object::max_hp_with(research)`, `Object::movement(research)` and `Object::max_speed(research)` (before the mode's
  `Mover::speed_factor`), and `combat::weapon(o, research) -> Option<AttackAttrs>`, a copy; by reference,
  `Object::movement_ref` and `combat::weapon_ref` (below). The kind's numbers
  (`ObjectDef::max_hp`, `UnitDef::movement`, `UnitDef::attack`) and `Object::max_hp` (a tier's, a site's, a
  transformation's) are the values before them. The effects table is data (`defs/cults/augmentation.rs`), and which
  kinds take Augmentations, armed or not, is `UnitDef::augmentable` (the four Cults infantry types, the Recruit alone
  unarmed).
  - **The amounts follow the owner's current levels**, read whenever a stat is (design change 0336: research is
    retroactive; 0338: every Cults stat upgrade applies to the units that exist). A unit stores which Augmentations it
    has, never their amounts or a level. `Research<'a>` is a view of the players' research: `Levels::of(player)` is the
    one place a player's levels come from, "No research" until the Temple (slice M5-N3 fills it, and applies the HP
    rule below to the units it changes), and `Research::at(levels)` puts everyone at given levels, for tests.
  - **Why this shape.** The systems that hold the objects mutably (phases 5a, 5d and 5e, the network's regeneration)
    take a `Research` beside them, a borrow of the players alone, which a `Sim` method couldn't be; code that has the
    `Sim` asks `Sim::research()`. Stats stored on the object would go stale when a level completes or a def changes
    (T73's rejected alternative). A unit without an Augmentation costs a branch (`Augmentation::is_none`): the loops
    over every unit (phases 5a, 5d and 5e) borrow its kind's numbers (`Object::movement_ref`, `combat::weapon_ref`,
    which make an augmented unit's in a slot of the caller's), through accessors inlined into them, their augmented
    paths out of line (`augmented_increments`, `augmented_movement`); an augmented unit looks its owner up among the
    few players. The behaviours keep a unit's increments (`orders::world::Unit::increments`) and work out its top
    speed only where they need it.
  - **HP restored** goes up to `max_hp_with`: the repair's (`repair::damage`, whose MaxHP also sets its rate and its
    price's steps, slice M5-N8) and the regeneration's (`crate::regeneration`). No repairer repairs infantry yet.
  - **HP when MaxHP changes** (design changes 0330 and 0331): `augmentation::follow_max`, applied by
    `augmentation::set` (`DebugCommand::SetAugmentation` now, the Altar and the research later). Up, the HP rises as
    much: `transform::follow_max`'s rule, the damage staying. Down, the HP stays unless it is above the new MaxHP:
    there the two rules part, as a transformation's keeps the damage (design change 0110). No floor; an augmentation
    never kills.
  - **Kind changes** (`Sim::change_kind`): a unit keeps its Augmentations as a kind that can have them, loses them as
    any other, its HP within the new kind's MaxHP with those it keeps; `Sim::train` follows the MaxHP with them before
    and after (a Constitution Recruit's 60 is a Militant's 60).
  - **What they don't change** reads the weapon before them (`combat::weapon_in_hand`): the threat a unit is to the
    scans (`combat::threat`, its TargetDomain) and `can_attack_now` (its ranges); and the TurnRate. A weapon borrowed
    from another kind (a Recruit's in the Garrison, the Militant's, slice M5-N2) is that kind's: the effects apply to
    a unit's own attack only (`augmentation::augmented`; `factions/cults/structures/garrison.md`: "Augmentations don't
    change the borrowed attack").
  - **The guard**: `tests/augmentation/raw_reads.rs` scans the sim's and the client's code, outside the defs, the
    accessors and the tests, for the reads past the accessors (`.max_hp()`, a `.movement` field, `weapon_of(`), and
    fails on any its list doesn't allow with a reason (a structure's MaxHP, a turn rate, a kind's weapon). A new read
    goes through the accessor, or joins the list with its reason. A test rather than unreachable fields: the defs are
    public data that their own tests and the client's panels read, and `Object::max_hp` stays the structures' MaxHP.
- **Mechanics implemented once** (plan decision T54, slice M4-R1): `Progress` (`progress.rs`), ticks done of a total
  in the fixed point of plan decision T6, so that a tick at part speed counts its share of a tick; a transformation
  counts its frames with one (`Transform`). The toggle rule (`toggle.rs`, design change 0198, see *Type and form*):
  `toggle::pressed` switches the type at the press and `toggle::toward` works the form toward it, for the Riot Shield
  and the Weapon Conversion alike, each keeping its own state. The other timers don't use `Progress` yet, because
  their hashed bytes would change: production, construction, the Extraction Facility's cycle, the Weapon Conversion,
  the Riot Shield, a worker's mining and unloading, and the Tunnel Network's held units. Each moves at the next
  re-bless of the goldens that touches it. **New timers use `Progress`.**
- **Sides** (`sides.rs`, design change 0079): `sides(kind, orientation)` gives the letters of a structure's sides
  that face north, east, south and west (`[SideLetter; 4]`, `None` for other objects). Unturned, the letters of its
  `SymmetryType` (AAAA to ABCD) run clockwise from the top side; a flip mirrors them, then the rotation turns them
  with the building. A Barracks (ABAC) has its exit B on the east side unturned, on the south side turned 90°
  clockwise.
- **Footprints don't overlap** (design change 0096: no buildings on resources), with one exception, the pair of an
  Extraction Plate and the uncovered crystal patch it is placed onto (`ObjectKind::may_overlap`;
  `factions/gdo/economy.md > ExtractionPlate`). Scenario validation (`scenario/validate/occupied.rs`, for the map's
  objects and the Starting Layouts alike), debug spawns (`Reject::Overlaps`) and the placement check (see *Placement
  and the build area*) apply it. A plate must stand exactly on a patch (`placement::Onto`), which the placement check
  and map validation ask alike; only a debug spawn puts one on bare ground, where it mines nothing (see *Economy*).

**Adding an object type**
1. Add its def module with its `static ObjectDef`, a citation for every value (*Adding a unit or structure*), and its
   line at the end of the kind registry (`kind_registry!` in `defs.rs`), whose doc comment cites the design section.
   That one line makes the `ObjectKind` variant, its place in `ObjectKind::ALL` and its arm of `ObjectKind::def()`.
   Missing numbers become named consts tagged with the question id.
2. Set up its capability state in `kinds::Capabilities::fresh` (`sim/kinds.rs`; units get idle `orders` and
   `mover`), and give it its client module, with its line in the client's kind registry (*Adding a unit or
   structure*). Its per-object values, if it has any, are `SpawnSettings::defaults`' (the editor's fields follow
   them).
3. Test it. Spawn it in a small inline scenario, then check its initial state and its behaviour.

**Adding a capability**
1. Give it its own module with its state type (deriving `Clone, Debug, PartialEq, Serialize, Deserialize`) and logic.
2. Add `pub <name>: Option<State>` to `Object`, with a doc comment citing the design.
3. Put its per-type parameters on `ObjectDef`, with citations, and initialize the state in
   `kinds::Capabilities::fresh`. Decide in `Sim::change_kind`, whose destructuring of `Object` is exhaustive, what a
   changed object does with it: keep it, start it afresh, or refund it.
4. Insert its update phase into `Sim::step` at a deliberate position (see *Tick pipeline*).
5. Add the `Command`s that drive it, with validation, and the `SimEvent`s the client needs. A unit's mode (a
   toggled ability such as the Riot Shield) is not an order: it goes through `Command::Ability` (an `Ability`
   variant, see *Commands, events and replays*), never into the order queue.
6. Write unit tests in the module and a scenario test in `crates/sim/tests/`.

## Placement and the build area

`placement.rs` (slice M2-C; `core/entities.md > Placement Validation`) holds every rule of where a structure may be
placed, in one pure function, `placement::check`, so that the command gate, the client's placement ghost and Starting
Layout validation can't disagree. **Never test a placement rule anywhere else: add it to the check.**

- **In**: a footprint (kind, top-left tile, `BuildingOrientation`), a `Site` (the map, the `BuildArea` to reach and
  the `Mode`) and the `Occupant`s (the kind, centre and footprint of everything on the map). `Mode::Command { vision,
  player }` is a game: every tile must be Visible to the player as the latest tick left it. `Mode::Layout` is a
  Starting Layout: no sight (design change 0097). A game asks `Sim::placement_check(player, kind, origin,
  orientation)`, a layout object `placement::check_layout(map, others, layout, index)`.
- **Out**: a `PlacementCheck`: the `TileProblem`s of each footprint tile (off the map, not Buildable, not Drillable,
  sloped, not Visible, under a structure, under a resource, under a ground unit, reserved), row by row and in that
  order, as bits of a u16 (plan decision T53), and the `WholeProblem`s (not a structure, outside the build area, a
  plate not on a patch, a Tunnel Area off the map, on undrillable ground or over another Tunnel's, an expansion
  outside its Tunnel's area). Valid means both are
  empty: the ghost shows the tiles' problems tile by tile, and the gate refuses a placement with any problem. The
  check reads the true state, but a tile the player doesn't see has `NotVisible` as its only problem, so it tells
  nothing the fog hides.
- **The rules** are listed with their design sections in the module docs. Footprints share tiles only as
  `ObjectKind::may_overlap` allows. A ground unit of any player blocks the tiles its box overlaps, and is never pushed
  aside (design change 0094); air and underground units never block, and no unit blocks an underground expansion
  (design change 0290). A plate goes exactly onto a patch
  (`placement::Onto`), the one rule of the check that map validation applies to pre-placed objects too
  (`SpawnProblem::NotOnPatch`, slice M2-J2).
- **The Tunnel Area rules** (slice M4-B, `placement/tunnel_area.rs`): a kind with a tier table, the Tunnel, has its
  whole Tier 1 Tunnel Area (`tunnel::area`) on the map and Drillable (design change 0242:
  `WholeProblem::TunnelAreaOffMap`, `TunnelAreaNotDrillable`; Drillable is the preset's flag, never its look), in
  every mode and method; `Mode::Known` counts only the undrillable ground the player has explored, the map's edge
  always. A Tunnel 2 tiles from the edge is valid (design change 0283, as written). `check_layout` adds the
  layout rule for expansions: an underground structure is checked by the expansion method in the Tier 1 area of a
  Tunnel of its own layout (`OutsideTunnelArea` outside every one), as a layout's plates lie in its own Deployment
  Center's area. Map and debug spawns don't take the check: a spawned Tunnel by Water or the edge stays valid.
- **The overlap rule** (slice M4-E, `placement/tunnel_overlap.rs`; design change 0237,
  `factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule`): a kind with a tier table has its Tier 1
  area overlap no area of a Tunnel among the occupants, any player's, sites included (design change 0289 point 3), each
  at its tier as the check knows it (`Occupant::tier`: an object's tier now, a record's as last seen); touching is
  allowed. It is `WholeProblem::TunnelAreaOverlap`, in every mode: `Mode::Known` judges by what the player knows ("their
  remembered position and their tier as last seen"), `Mode::Arrival` by the truth, so a Tunnel upgraded unseen fails a
  build on arrival.
- **The Recruitment Area rule** (slice M5-J1, `placement/recruitment_area.rs`; design change 0341 point 1,
  `factions/cults/structures/recruitment_hall.md > RecruitmentArea`): a kind that recruits (`recruitment::def_of`),
  the Recruitment Hall, has its whole Recruitment Area (`recruitment::area`, centred on the footprint) on the map, in
  every mode, as the Tunnel Area's edge rule: `WholeProblem::RecruitmentAreaOffMap`, "The Recruitment Area reaches off
  the map". A Hall's 10-tile area holds its 4 × 4 footprint with 3 tiles a side, so an origin 3 tiles from the edge is
  valid and 2 is refused. Map and debug spawns don't take the check: such a Hall's tiles off the map count among its
  area's and can't be claimed (`recruitment.rs`, a placeholder).
- **The files** (`placement/`): `site.rs` (`Site`, `Mode`, `Method`, `Occupant`), `problems.rs` (the problems,
  `PlacementCheck`, their texts), `check.rs` (the check), `footprint.rs` (its tiles), `tunnel_area.rs`,
  `tunnel_overlap.rs`, `recruitment_area.rs`, `build_area.rs` and `onto.rs`. The geometry of Tunnel Areas, their regions and an expansion's
  parent Tunnel is `tunnel/area.rs`, the one place that works them out (Tunnel Space, these rules,
  `Sim::parent_tunnel`).
- **Methods** (plan decision T16): the `Site` also names how the structure comes to stand (`Method`), and the one
  check applies that method's rules. `Method::Direct` is the GDO's Direct Placement, with its build area.
  `Method::WarpIn { beacon }` (slice M3-E; `core/entities.md > Placement Validation > Warp In`) has the same tile
  rules, sight included (design change 0119: a tile out of sight is invalid whatever stands there), none but sight
  on the tiles of the Beacon the building consumes, ground units of any player blocking the others (0118), and no
  build area. A Warp In's footprint is never chosen: `transform::warp_in_footprint(beacon, kind, orientation)`
  derives it (0121: centred on the Beacon; an odd-sized building's extra row and column at the bottom-right, turned
  and flipped about the Beacon's centre as the building's sides are), and `Sim::warp_in_check(beacon, kind,
  orientation)` checks it. The gate and the Warp In ghost (slice M3-J1) call those two. `Method::WorkerBuilt`
  (slice M3-F; `core/entities.md > Worker-Built Structures`: a Drone's Deploy, and an Agent's Build since slice M4-E)
  has the tile rules without a build area;
  the caller leaves the worker out of the occupants (0118). Its order is judged in `Mode::Known { vision, player }`
  (0120: what the player knows: its own objects, the others it sees, its records of structures and resources in the fog;
  a tile never explored passes, but for the reservations), its arrival in `Mode::Arrival` (the truth, no sight rule).
  `Sim::deploy_check(drone, origin, queued)` and `Sim::build_check(worker, kind, origin, orientation, queued)` are the
  orders' checks, for the gate and the ghosts; `Sim::arrival_check` is the arrival's. `Method::Expansion { area }`
  (slice M4-F, plan decisions T32 and T53; `factions/syndicate/tunnel_menus.md > AwaitingPlacement (Expansion)`) is a
  Tunnel's underground expansion: wholly inside `area`, the Tunnel's current area (`OutsideTunnelArea`), on Drillable
  tiles in place of Buildable ones (`TileProblem::NotDrillable`, design change 0239), with no flatness and no sight
  in any mode. Every structure blocks it, whichever its layer and whoever owns it, the Tunnel's own footprint
  included (0240), and so do resources (0241); no unit does, and no reservation is checked against it, since no
  lingering ghost can be in its way (0290). `Sim::expansion_check(tunnel, kind, origin, orientation)` is the ghost's,
  in `Mode::Known` (what the owner knows); the gate checks the same by the truth (`Sim::expansion_truth`, in
  `Mode::Command` without its sight rule), so a surface building the owner doesn't know of refuses the Expand
  (0240). The other way round, a direct placement and a Warp In over a hidden expansion are red at once, since
  `Mode::Command` reads every object (0240's accepted leak; 0290 for the Warp In), while a worker-built ghost judges
  by what the player knows and fails on arrival.
- **Reservations** (design change 0125, `core/entities.md > Lingering build ghosts`): each worker build ordered and not
  yet started (a Deploy, a Build), current or Shift-queued, reserves its site against its owner's further placements,
  a worker build's and a Warp In's (`TileProblem::Reserved`, through `placement::check_reserving`, which takes them as
  `Occupant`s; an order without Shift replaces its worker's orders, so that worker's own sites don't count). A pending
  Tunnel's whole Tier 1 area is reserved too, by the overlap rule (design change 0289 point 2). They are derived from
  the order queues (`Sim::pending_builds(player)`: `PendingBuild { worker, kind, site }`, the lingering ghosts), never
  stored; another player's never count, and they block no unit.
- **The build area** (`BuildArea`, `Sim::build_area(player)`; `factions/gdo/structures/README.md > GDOBuildArea`) is the
  union of one rectangle per GDO building the player owns: its footprint grown by its BuildRadiusExtension on every
  side, clamped to the map (design change 0091: square). A building with an extension of 0, the plate, adds nothing.
  It is derived, never stored or hashed: worked out from the objects whenever it is asked for, so it follows every
  placement and every loss at once (0092). It is the area only, apart from the tiles' validity (0093): an overlay
  draws `BuildArea::runs`, a ghost the check's tiles. A Starting Layout's area is that of its Deployment Center
  (`BuildArea::of_layout`). The Deployment Center seeds the area and needs no tile in it (`seeds_build_area`).
- **Not placement rules**: debug spawns bypass them (they keep the map's rules and apart from other footprints, see
  *Commands, events and replays*), and starting units follow the unit-start rule (`paths::misplaced`, see *Sim
  scenarios and tests*).
- **Spots for units that work at a footprint** (slice M5-E, `crate::spot_work`, `Sim::free_spot` in
  `sim/spot_work.rs`; *Economy > Cults construction*): design change 0339's wall-hug, `placement::Wall`
  (`placement/wall.rs`, geometry alone): the loop round the footprint where a unit's box stands `BUILD_STANDOFF_SU` off
  it, searched from its point nearest the unit (`Wall::hug`), then both ways along it, `HUG_STEP_SU` at a time, of two
  as far the clockwise one first; `Wall::out` is a row further out, `Wall::next_corner` where the way along it turns.
  (Slice M5-F1's spots around a footprint, plan decision T62's one search for a unit put on the map beside a building,
  `placement::Spots` and `Sim::spot_around`, went once nothing used them: the units of a destroyed building that holds
  units appear inside its former footprint since design change 0349, slice M5-F3, `Sim::evacuate`, and the Laser
  Turret's Agent, which came out beside the turret it finished, went with design change 0359, slice M5-Y. Slice N2's
  Garrison will reuse the evacuation: *Object model > Buildings that hold units*.)
- **Tests**: `src/placement/tests/` (each rule on the pure check), `placement/build_area.rs`'s (the 28 × 28 area,
  each kind's extension, the runs), `tests/placement.rs` (the verification points of design changes 0004 and 0091 to
  0097 in games and layouts, determinism, a save), `tests/warp_in/` (the Warp In method, see *Economy > Warp In*),
  `tests/deploy_gate/` (the worker-built method, its modes and the reservations), `tests/builds/` (the Build's: the
  overlap rule by knowledge and by truth, the Drillable area, hidden expansions), `tests/expansions/placing.rs` (the
  expansion method in games, by knowledge and by truth; slice M4-F) and the ignored
  `tests/placement_perf.rs`. Measured (release, 769 objects, 101 buildings of the player's; slice M2-J2, on a quiet
  machine): 4 µs a check, 2 µs a build area. Slice M4-E's tiers and overlap rule, A/B against the master before it
  (four alternating runs at load 1.1): 4.5 µs a check before, 4.9 µs after; the rule gathers Tunnel Areas only for a
  kind with a tier table (`tunnel_overlap::Overlap`) and reads only a placed object's tier (`Occupant::of`). An
  upgrading Tunnel's is the tier it upgrades to for the truth and its owner, the tier it has for the others
  (`Occupant::as_seen`; slice M4-G).

## Tick pipeline

`Sim::step(&TickInputs)` advances exactly one tick. It runs fixed phases in this order (`sim/phase.rs`):

1. **Clear events**, so that `Sim::events()` holds only this tick's events.
2. **Advance the tick** (`tick += 1`).
3. **Snapshot** `pos`/`facing` into `prev_pos`/`prev_facing` for every object (the client interpolates from these),
   and note who owns a structure as the tick starts (`economy::defeat::structure_owners`, for the defeat check).
4. **Commands**: apply each `PlayerCommand` in `TickInputs` order (`apply_command`). Each passes the gate against
   the state the commands before it left, so a cost is paid from the bank as they left it and a Place is checked
   with the footprints they placed (see *Economy > Construction*).
5. **Systems**, each walking the objects in id order, one function per module (`docs/movement.md > 4`):
   - Modes, `Sim::step_modes` (`sim/modes.rs`, beside `Sim::use_ability`, which applies the ability commands and
     switches a toggled unit's type at the press): each unit's form goes a step toward its type (*Object model >
     Type and form*). A Riot Shield on its way up or down goes on one tick (`riot_shield::step`), and a Weapon
     Conversion one frame, or, its frames done, gives the unit its other form, and converts it back at once if its
     type changed meanwhile (`conversion::step`; see *Combat > Weapon Conversion*); a raise of the Riot Shield or a
     conversion that waits for the unit's shot begins in the first Modes step after its Cooldown (design changes 0172
     and 0153). After the commands, so that a toggle or a Convert counts from its own tick; before 5a, so that the
     behaviours, the movement and the combat of the tick all see the form it leaves.
   - 5a `orders::update`: a unit's finished order completes and its next one starts, until one is in progress, so
     legs follow each other without an idle tick; then that order's behaviour runs for the tick, or the idle one
     (see *Behaviours*), and a turret's own after it (slice M5-U1). They write goals (`Mover::go_to`, `stop`), the
     target the attack engages and a turret's LockedTarget (*Combat > Units' turrets*). Everything
     they read of other objects comes from one snapshot taken first (`orders::world::World`): the positions and the
     vision the last tick left.
   - Arrivals, `Sim::step_deployments` (slices M3-F and M4-E, `sim/builds.rs`; see *Economy > Beacon Deployment* and
     *Economy > Agent-built Tunnels*): part of 5a, right after `orders::update`, in id order. Each worker whose walk
     to its site ended this tick deploys there (a Drone: a change of kind) or starts its site there (an Agent: a new
     object, which it leaves the map for, `Sim::start_site`), before 5b, which claims the footprint; or its order fails
     (`SimEvent::DeployFailed`, `SimEvent::BuildFailed`, a Build refunded): the changes of kind, the new objects and
     the events that a behaviour may not make.
   - Spots, `Sim::step_spots` (slice M5-E, `sim/spot_work.rs`; *Economy > Cults construction*): part of 5a, right
     after the deployments, in id order. Each unit whose current order works at a footprint from a spot beside it (a
     group's Build, an Assist, a building's Repair) and has no spot gets one and walks there, round the footprint, a
     unit on its way heading on at each corner (`Sim::head_on`); an order on a structure that no longer suits it ends;
     every `RESPOT_TICKS`, a unit whose walk is over keeps where it stands within reach, or looks again along the
     builders' wall, and one whose walk ended on the footprint itself looks again as far out as needed (slice M5-J1: its
     site waits for it, design change 0362). Before 5b, so that its walks get their paths in the tick.
   - Entries, `Sim::step_entries` (slice M4-C, `sim/network.rs`; `factions/syndicate/tunnels.md > Tunnel Network >
     Walking in`): part of 5a, right after the deployments, in id order. Each unit whose current order is an Enter
     and whose box is within `ENTRY_REACH_SU` of its Tunnel's entry spot leaves the map for the network
     (`Sim::take_off_map`, `SimEvent::EnteredNetwork`), on its way or at the end of its walk alike: no hold between
     units (design change 0246). An Enter whose Tunnel is gone or no longer suits the unit, or whose walk ended short
     of the path (the normal Failure rule, 0247), ends (`SimEvent::EnterFailed`). Into a building that holds units
     (slice M5-F1, *Object model > Buildings that hold units*) the same, at its entry spot, if its pool has room then
     (`SimEvent::EnteredBuilding`); with none the Enter ends there. Before 5b to 5f, so that no grid, collision, shot
     or sight of the tick includes a unit that has gone in.
   - Attacks on a target too close, `Sim::step_too_close` (slice M3-L2, design change 0182): part of 5a, right after
     the deployments, in id order. Each unit that can't move, or can't back off further (design change 0204 point
     3), whose Attack order 5a found with its target inside its MinRange (`AttackOrder::too_close`) ends the order,
     and its next queued one starts in the next tick, with `SimEvent::TargetTooClose`; and so does each whose Attack
     Ground 5a found with its location inside (`AttackGround::too_close`, slice M5-O3), with
     `SimEvent::LocationTooClose`.
   - 5b `paths::sync`: if the blocking footprints changed, rebuild the true navigation grids and bump
     `nav_version`.
   - 5c `paths::serve`: first each unit's staggered path checks, in the ticks where `(tick + id) % 4 == 0` (after
     its owner's path knowledge changed, line of sight along its remaining path, and a new plan only if that is
     cut; the farthest waypoint in sight becomes its steering target). Then every `PathState::Wanted` becomes a
     `nav::PathRequest`, served by `nav::serve_on` within `PATH_BUDGET` (group fields for move groups). A unit that
     gets no path fails its move (`SimEvent::MoveFailed`). Each unit checks and plans on its owner's known grids,
     as of the end of the last tick (see *Navigation > Path knowledge*).
   - 5d `movement::step`: steering and hard collision, the only code that writes a unit's `pos` besides a teleport
     (see Movement and collision). Units move one after another in id order and never come to overlap, gathering
     Drones apart (design change 0106).
   - 5e `combat::step`: attack phases (a unit's turret turning first) and launches, the projectiles due landing in
     launch order, the tick's list of
     hits applied together (the blasts' first), deaths (see *Combat*). The dead
     leave through `Sim::remove_objects`, then `paths::sync` runs again in case a footprint went with them.
   - Pop-ups, `Sim::step_popups` (slice M4-S2, `sim/popups.rs`; *Object model > The pop-up form*): right after 5e
     and its removals. Each turret in operation from before the tick goes one tick through its cycle, in id order: by
     whether it is active now (an inactive one is lowered at once; a Tunnel lost in the tick left its turrets neutral
     and lowered already, on the removal path), the target 5e left it, and, to start rising, no ground unit's box over
     its footprint, where 5d left the units. A turret that starts to rise or is fully lowered changes what blocks:
     `paths::sync` runs again. Before 5f, so that a turret is seen from the tick it starts to rise and hidden from the
     tick it is down.
   - Shields, `shields::step` (slice M3-B, see *Combat > Shields*): every object's Shields regenerate
     `SHIELD_REGEN_PER_TICK`, up to its MaxShields now (`Object::max_shields`). Right after 5e's removals, so that the
     dead regenerate nothing and a hit's tick ends with the regeneration on top of it; before the economy, so that a
     transforming object regenerates up to the maximum its tick began with, which `Sim::step_transforms` then raises,
     its Shields with it.
   - Economy, `economy::step` (plan decision T5, see *Economy*): after 5e's removals and their `paths::sync`, so
     that a building destroyed in the tick counts for nothing in it; before 5f, so that what it brings into the game
     is seen in its own tick. In order: every player's power (the structures this tick's commands placed included),
     the constructors (`construction::step`; when a recipe's frames count: *Economy > Recipe timing*), the plates'
     mining, the workers' timed work (`gathering::step`, slice M3-H: mining takes its load from the patch, a
     drop-off pays the bank). `economy::step` returns the patches the workers emptied, and `step` takes them out
     through `Sim::remove_objects` and syncs the grids again, as after 5e's deaths.
   - Expansions' regeneration, `Sim::regenerate_expansions` (slice M5-N8, `sim/expansions.rs`; *Economy >
     Expansions' regeneration*): part of the economy phase, right after `economy::step` and its removals, before the
     transformations, the sites, the upgrades and the decay, so that an expansion completed or reclaimed in the tick
     regenerates from the next one, keeping in this one the damage it was built with or that its decay left it.
   - Transformations, `Sim::step_transforms` (slice M3-E, see *Economy > Warp In*): part of the economy phase, run
     right after the expansions' regeneration, before the producers: each transformation advances one frame,
     its interpolated maxima and its HP follow, and a complete one turns its functions on, so a portal takes a Train
     from the next tick's commands. It changes no footprint: a Warp In and a cancel change theirs in the commands
     phase, before 5b.
   - Sites, `Sim::step_sites` (slice M4-E, `sim/sites.rs`): part of the economy phase, right after the
     transformations and for the same reason. Each site from before the tick does a frame, its HP following its
     maximum, and a complete one works from the next tick's commands, its worker in the network for the network's
     step. A site is a new object, so by the spawn-tick rule one started in tick `t` does its first frame in `t + 1`
     and completes at the end of `t + frames`. A Tunnel that completes reclaims the neutral expansions it covers
     (`Sim::reclaim`, slice M4-H). (Slice M4-S1's Laser Turret's Agent, which came out beside the turret it
     finished, went with design change 0359.)
   - Repairs, `Sim::step_repairs` (slice M5-N8, `sim/repairs.rs`; *Economy > Repair*): right after the sites, whose
     builders its repairers are counted as (`spot_work::at_work`), and after 5e, so that the HP an object lost in the
     tick is what its repairers restore. Each object that repairers are at work on, in id order, regains its HP, its
     owner paying for it; the Repairs that are over end (`Sim::end_work`), their queued orders starting in the next
     tick's 5a. A site completed in the tick has no repairers yet.
   - Upgrades, `Sim::step_upgrades` (slice M4-G, `sim/upgrades.rs`): right after the repairs. Each Tunnel from before
     the tick that upgrades does a frame, in id order: the Tunnel exists already, so an upgrade ordered in tick `t`
     completes at the end of `t + frames - 1`, as a recipe; then it reclaims, before the decay.
   - Decay, `Sim::step_decay` (slice M4-H, `sim/tunnels.rs`): part of the economy phase, right after the sites, so
     that an expansion a Tunnel finished over in the tick is its owner's again first. Each neutral expansion that
     decays loses a tick of it, in id order; one whose HP reaches 0 dies (`SimEvent::Died`, owner `None`) and leaves
     through `Sim::remove_objects`, the grids synced again. An expansion its Tunnel left neutral in the tick (in the
     commands' removals or 5e's deaths) decays from that tick.
   - Producers, `Sim::step_production` (slice M2-D2, see *Production*): part of the economy phase, run right after
     `economy::step` and not inside it, because a finished unit is inserted into the game (`Sim::insert_object`).
     The Recruitment Halls first queue their Recruits by themselves (slice M5-C, `Sim::start_recruits`: their claims
     and lineages are read here, after 5e's deaths, so a Hall lost in the tick frees its tiles to the next Hall in
     it). A unit at the front starts only if its owner's army cap has room for it (design change 0143), the unit in
     production advances, and a finished one is released into its exit doorway, or held while that is occupied.
     Nothing in `economy::step` reads what the producers write, or the other way round, so the order between the
     two is free; after the transformations, so that a Beacon completed in the tick gives its room in it, as the
     deaths of 5e do theirs. After 5d, so a released unit never moves in its own tick; before 5f, so it is seen in
     it.
   - The Tunnel Network, `Sim::step_network` (slice M4-C, `sim/network.rs`): part of the economy phase, right after
     the producers. Every held unit regenerates; then each Tunnel with an eject queue, in id order, lets its first
     unit out onto the exit spot if that is clear of tiles, footprints and units of its layer (`Sim::check_unit_fits`,
     the rule of every unit put on the map), else the queue waits: no other spot, no minimum gap (design change
     0250), one unit a tick at most. The unit keeps its id (`Sim::put_on_map`, `SimEvent::LeftNetwork`) and gets its
     order through `Sim::give_order`: its rally's, or a Move 1 grid unit straight out. After the producers, so that a
     unit finished in the tick with a surface rally comes out in it; after 5d, so that it never moves in the tick it
     appears; before 5f, which sees it. **Old ids come back** (plan decision T35): nothing after this step may take
     `first_new` for "everything that arrived this tick".
   - Buildings that hold units, `Sim::step_holding` (slice M5-F1, `sim/holding.rs`): part of the economy phase,
     right after the Tunnel Network. Each building with units ready to leave (`Held::leaving`: ejected, or the unit of
     a pool building's finished entry, slice M5-F2), in id order, lets the one ready first out into its exit doorway
     if the doorway is free (`Sim::let_out`: a trained unit as the unit it trains, `SimEvent::UnitProduced`; an
     ejected or released one as it is, `SimEvent::LeftBuilding`), toward the rally point: one a tick at most, and none
     while the last is still in the doorway. After the producers, so that an entry finished in the tick lets its unit
     out in it, and for the same reasons as theirs after 5d, before 5f.
   - Defeat, `economy::defeat::step` (slice M2-F2, plan decision T8, see *Economy > Defeat and the end of the
     game*): the economy phase's last step, run by `step` after the producers, so that every way a structure left
     the game in the tick counts (the commands' removals, 5e's deaths). Before 5f, so that the units a defeat makes
     neutral are nobody's eyes in its own tick.
   - 5f `Vision::update`: every player's sight from the objects as the tick leaves them, those this tick's commands
     spawned included, then the fog delays and the reveals that end (see Vision). Combat in 5e reads the vision of
     the tick before, and its hits reveal their attackers on top of it.
   - `update_supply_stations` (`core/map.md > SupplyDeliveryStation`).
   - `Vision::remember`: each player's records of the structures and resources in its sight take the tick's
     final state, the deliveries included (see Vision).
   - Rally points, `production::rally::follow`: on the vision the tick leaves, a rally point on an enemy its owner
     no longer sees becomes a ground rally where it was last seen (design change 0088, see *Production*).
   - Path knowledge, last (`PathKnowledge::observe`, `paths::sync_known`): each player stops assuming the map
     resources whose place it now sees, and a player whose known footprints changed gets a new knowledge version
     and its planning grids rebuilt, at most once per player and tick. After everything it follows from: the
     objects, the vision and its records. Nothing after it may change any of them.

- **Spawn tick**: an object that a command creates during tick `t` is first updated by the systems in tick `t + 1`.
  `step` notes `first_new = ObjectId(next_id)` before the commands, and each system walks only
  `objects.range_mut(..first_new)` (ids only grow). So a debug-spawned station with an interval of 960 delivers at
  `t + 960`, just as a scenario station, spawned before tick 1, delivers at tick 960. An order to a unit spawned
  earlier acts in the tick it arrives: a straight move of `d` su ordered in tick `t` arrives in tick
  `t − 1 + ⌈d / speed⌉`.
- **Grids match the state between ticks**: the true grids match the objects (`step` checks it in debug builds),
  and each player's known grids match what it knows (invariant K1). A phase after 5b that adds or removes a
  blocking footprint (a death in combat, a new structure) must call `paths::sync` again; the known grids follow at
  the end of the tick.
- **Measuring a tick**: `Sim::step_observed(inputs, observe)` is `step` calling `observe` with each `Phase` as it
  starts and `Phase::End` when the tick is done (`sim/phase.rs`); `step` passes a no-op, and the sim never reads
  anything back. `TestSim::step_observed` wraps it for the perf tests. Measured (`tests/structures_perf.rs`, ignored,
  release; slice M2-J2: four GDO bases of 40 structures and 400 Peacekeepers fighting through two of them on 256 ×
  256): a whole tick takes 0.28 ms on average, 0.53 ms at the 99th percentile and at most 2.4 ms after the tick of
  the four players' 20 group orders at once, which takes 7.8 ms, nearly all path serving (each group order is pathed
  in its tick, `docs/movement.md > 5.8`). At worst the grid syncs (5b, 5e's, path knowledge) take 0.13 ms with up to
  3 sets rebuilt in a tick, vision 0.17 ms, the economy 0.11 ms and the defeat check's two walks 0.005 ms.
  Measured with Drones (`tests/drones_perf.rs`, ignored, release; slice M3-K2: two Colonists sides of 40 Drones
  gathering on two rows of 8 patches each, and 10 Legionnaires a side fighting between them, on 128 × 48 for 3
  minutes; three runs at load 1.1 to 1.2): a whole tick takes 0.10 ms on average, 0.12 to 0.21 ms at the 99th
  percentile and at most 0.21 ms after tick 1, whose 32 Gathers and two group orders take 0.5 to 0.9 ms; movement
  0.053 ms, vision 0.025 ms, the behaviours 0.013 ms. The waiting Drones' re-check ticks (every 16th) cost 0.003 ms
  more than the others. There is no M2 baseline to compare with, since M2 had no Drones.
- Insert a new phase at a deliberate position, with a comment in `step` saying why there, give it its `Phase`, and
  update this list.
- Commands always run before movement. There is exactly **one** movement pipeline, and all movement goes through it.
  The old prototype had two pipelines, and they raced.
- Damage accumulates as a **list of hits** that is applied in one place. Never store it in a single slot that a second
  hit can overwrite, because the old prototype lost hits that way.
- Lessons from the prototype, for M1 and on:
  - **Movement**: the arrival radius must be at least one tick of travel. Otherwise units overshoot and oscillate.
  - **Pathfinding**: cap repath attempts (`tuning::movement::MAX_REPATHS`). A* needs a correct closed set, a best-cost
    map and deterministic tie-breaking. Overwriting a parent at equal cost made zigzag paths.
  - **Occupancy** is domain-aware. For example, underground expansions don't block surface units.

## Combat

Slice W (`combat.rs`, `combat/`, `elevation.rs`; `systems/combat.md`). A unit that can attack has an `AttackState`:
the target its behaviour engages (`target`, set by its Attack order in 5a), the phase of the running sequence and the
ticks spent in it. Hashed and serialized like all state.

- **Phases** (`combat/phases.rs`, its tests in `combat/phases/tests.rs`): Aiming → Firing → Cooldown → Reloading, each
  lasting exactly its duration in ticks (zero included). A sequence starts in 5e when none runs and the engaged target
  is valid and in range; the next one at the earliest in the tick after Reloading ends, so a cycle takes Aim + Firing +
  Cooldown + Reload ticks (Peacekeeper 17, Guard 24: its laser rifle, design change 0186). Ordered in tick `t`, the
  first hit lands at the end of tick `t + Aim + Firing − 1`.
  Aiming turns the base towards the target at TurnRate (the attack writes `facing`) and lasts until it faces it
  within `FACING_TOLERANCE`. Aiming and Reloading are interruptible: another order, or a target that is gone or out
  of range, cancels an aim; a reload keeps counting. Firing and Cooldown run on whatever happens.
- **Queries**: `combat::may_move(&Object)` and `may_turn(&Object)`: whether the attack leaves the base free to move
  or turn this tick (`systems/combat.md > UnitBaseSource`: not while Aiming, Firing or in Cooldown; turret units
  always), and never while the unit converts its weapon (see Weapon Conversion). Phase 5d
  holds a unit that may not: it stands, keeping its goal, path and stuck clock (`movement::wait`). The behaviours
  start a chase only when `may_move` holds.
- **One turn budget per tick**: a unit turns at most its TurnRate per tick in all. Phase 5d turns it first and notes
  the angle (`movement::Turns`, in the movement scratch); the aim in 5e turns it only by what is left
  (`Shooter::turn_left`). Both turn through `math::turn_toward`. It matters in the tick a unit that moved comes into
  range and begins to aim. The record is transient: 5d writes it afresh and 5e of the same tick reads it, so it is
  neither hashed nor saved; it follows from the state the tick began with and the tick's inputs.
- **Range** (`shape.rs`, `combat/targeting.rs`; `systems/combat.md > Range Measurement`, design change 0031): **edge
  to edge**, the gap between the attacker's shape and the target's. `Shape::of(object)` is a unit's Silhouette or a
  tile-based object's footprint; `Shape::gap` measures between two, `Shape::distance_to` to a point (ground-location
  targets). Shapes are axis-aligned for now: Silhouettes that rotate with the facing (design change 0033) wait for
  the vehicles milestone, and only `shape.rs` changes then (`Shape::silhouette` already takes the facing).
  `combat::reaches(attrs, attacker shape, elevation, target shape, elevation)` is the one range check: within Range
  and beyond MinRange, both inclusive, with the ElevationModifier. `can_attack_now(map, vision, attacker, target)`
  adds the rest for two objects: destructible, seen, an allowed domain.
- **Vision** (`systems/combat.md > ValidTarget` 2): a sequence starts, and an aim goes on, only while the attacker's
  owner sees the target (`PlayerVision::sees_object_box`: the fog delay and a reveal count), by the vision 5f left
  at the end of the tick before. What a unit does about a target out of range or out of sight is its behaviour's; a
  neutral attacker sees nothing. Once the tick's hits are applied, each one reveals its attacker to the owner of the
  object hit if that player doesn't see it (`Vision::reveal`, see Vision).
- **ElevationModifier** (`elevation.rs`, shared with vision): `Elevation::of` an object is its elevation level (the
  height rounded down), or `Exempt` for air units; `modifier(source, target)` is +1/−1/0 grid unit, binary.
  `attack_range` and `sight_range` apply it; melee attacks don't take it.
- **Melee** (`systems/combat.md > FullyConnected > Melee`: "Short reach: 8 space units"; the Sabre Legionnaire):
  every melee attack has the Range `defs::MELEE_CONTACT_GU`, a gap of 8 su, the one that counts as Proximity
  (`tuning::orders::PROXIMITY_SU`), and no MinRange (`defs_are_consistent` checks both). It is a placeholder (design
  change 0149): exactly 0 would be fragile, since a unit closing in may stop a little short of touching. `max_range`
  gives melee no ElevationModifier.
- **Hits** (`combat/damage.rs`): every attacker acts against one snapshot of the targets, and an attack's effect
  lands at the end of Firing as a `Hit` on the tick's list. The list is applied once all have acted, so every hit
  counts, including those on a unit another hit has killed and those of a unit that dies in the same tick.
  `hit_damage` follows `systems/combat.md > SingleTargetDamage` and `systems/combat.md > DamageModifierOrder` (pre-armor
  multipliers, PointArmor, directional multiplier, MinimumDamage); HP is fractional, nothing is rounded. PointArmor is
  the target's `ObjectDef::armor`, a structure's too: a Peacekeeper's 10 damage takes 9 HP off a Generator and 8 off a
  plate. An object at 0 HP or less dies; a structure that dies frees its tiles (the navigation grids are rebuilt).
- **Shields** (`shields.rs`, `combat/damage.rs`; `systems/combat.md > Shields`, slice M3-B, plan decision T18): an
  object whose kind has MaxShields (the Colonists') has `Object::shields`, full when it is inserted (and when a save
  from before Shields loads). A `Hit` carries the raw damage and the target's modifiers as the shot saw them
  (pre-armor multiplier, PointArmor, directional multiplier), and `damage::resolve` settles it when the list is
  applied, hit by hit, so the second hit of a tick meets the Shields the first left. The Shields take the damage
  first, at full value; an overflow goes through `hit_damage` with the MinimumDamage capped at it (`systems/combat.md >
  MinimumDamage > Shield overflow`: 10.25 on 10 Shields and PointArmor 1 takes 0.25 HP); a hit the Shields take whole
  costs no HP, and no armour is consulted. No modifier touches the Shields: the pre-armor multipliers and the
  DirectionalArmor touch the overflow only (design change 0147). `SimEvent::Hit` says what each hit took (`damage` of
  the HP, `shield_damage`). The Shields regenerate 0.25 a tick (`SHIELD_REGEN_PER_TICK`; `systems/combat.md >
  ShieldRegenRate`: "continuously (not gated by a delay after taking damage)") up to the maximum now, a neutral object's
  too (design change 0146), in their own step (see *Tick pipeline*). A transforming object's follow its interpolated
  maximum as its HP do and regenerate up to it (design changes 0110 and 0146, `transform::follow_max`), and a cancel
  keeps their fraction, with no minimum (0111, `transform::keep_fraction`). The vision's records keep no Shields.
- **Weapon Conversion** (`conversion.rs`, `sim/modes.rs`; `factions/colonists/units/weapon_conversion.md`, plan
  decision T26, design change 0198): `Ability::Convert` flips the target form of each Legionnaire it goes to, a
  Sabre's to the Rifle Legionnaire and a Rifle's to the Sabre (`conversion::converts_to`): its type switches at the
  press, and the unit converts to it (*Object model > Type and form*). The gate drops the units without another form
  (`Reject::LacksAbility` when none has one) and takes every other, whatever it is doing: a press is never refused.
  `Object::conversion` holds the form the unit has (`Conversion::form`, whose weapon it fights with,
  `combat::weapon`) and the frames left of the conversion under way. **Pressed mid-attack** (design change 0153:
  "deferred, not rejected"), a unit Firing or in Cooldown fights on with its form's weapon
  (`conversion::waits_to_convert`, *Changes that wait for an attack* below), and its conversion begins in the first
  Modes step after its Cooldown's last tick; so one R converts a whole group in a fight, the mid-swing units 1 to 4
  frames later (a Sabre's Firing and Cooldown). **Pressed again** (`factions/colonists/units/weapon_conversion.md > A
  toggle sets a target form`): a unit that waits takes the new target, and doesn't convert at all if that is its form
  (`conversion::retarget`); one converting finishes, then converts back (8 frames for the round trip); one converted
  converts back. A conversion cancels an Aim or a Reload, keeping the target (design change 0152: the new weapon starts
  ready), and lasts `CONVERT_TICKS`, 4 frames: the unit holds still and doesn't turn (`may_move`, `may_turn`; its
  behaviours see it can't move; `Mover::speed_factor` 0, so it yields to nobody), its attack idles, and it can be
  attacked throughout (`factions/colonists/units/weapon_conversion.md > Vulnerability`). Pressed in tick `t`, it is of
  its new type from `t` on, converts through tick `t + 3`, and the Modes step of `t + 4` gives it its other form, HP and
  Shields unchanged (their maxima are equal), so it fights as that from `t + 4`. Nothing cancels a conversion under way
  (`factions/colonists/units/weapon_conversion.md > Completion`), a Stop changes nothing. The orders stay, and it
  carries on with the other weapon (`factions/colonists/units/weapon_conversion.md > Orders are kept`, design change
  0150). Convert isn't queued (design change 0151: "holding Shift changes nothing").
- **Changes that wait for an attack** (`sim/modes.rs`, design changes 0153 and 0172): a conversion or a raise of the
  Riot Shield whose unit is Firing or in Cooldown (`combat::mid_attack`: the phases that can't be interrupted) when it
  would begin waits, the unit's type switched already: the first Modes step after the Cooldown's last tick begins
  it, before the Reloading that followed runs a tick, so the unit acts as if pressed in that tick and the Reloading
  never runs (a raised shield's Reloading goes on behind it: the weapon is kept). No ability is stored: the type and
  the form's state say it (`conversion::waits_to_convert`, `ShieldMode::Deferred`). A lowering never waits (a shield
  off the ground has no shot to wait for), and one pressed while a raise waits drops it: the shield never starts
  rising (design change 0198).
- **Removal path**: `Sim::remove_objects` (`sim/removal.rs`) is the one way objects leave the game, for deaths and
  debug removals alike. It emits `Removed` and clears every reference to the removed: a current order naming one
  completes, queued ones are dropped, a move towards one ends `TargetLost`, an attack on one stops (an aim is
  cancelled, a shot being fired hits nothing). An order outlives its target by one rule (`Order::outlives_target`):
  an Attack, a Move to an object or a Gather on an object its holder's owner still remembers goes on to the
  last-known position (design change 0139: "nothing about the real state leaks"), and a Gather whose patch leaves
  heads for where the patch was (`Gather::spot`, see *Behaviours > Gathering*); the next tick's snapshot sees such a
  target Remembered until its place is seen, and Gone then (`sim/removal.rs::forget_seen_gone` ends what names it).
  `testing::dangling_references` checks that nothing else names a removed object. A Tunnel taken, whatever took it,
  leaves its expansions neutral, destroys its unfinished ones and strands the network units no remaining Tunnel lets
  out, once everything of the call has gone (`Sim::lost_tunnels` before, `Sim::lose_tunnels` after; slice M4-H,
  *Object model > Losing Tunnels*). The units inside a building that holds units come out alive inside its former
  footprint in the same way (`Sim::evacuations` first, before the refunds, `Sim::evacuate` once the building and
  every reference to it are gone; slices M5-F1 and M5-F3, *Object model > Buildings that hold units*).
- **Riot Shield** (`riot_shield.rs`; `factions/gdo/units/peacekeeper.md > Riot Shield (toggled ability)`, slice
  M3-S): shield down and shield up are two unit types, `Peacekeeper` and `PeacekeeperShielded` (design change 0173),
  which carry the same `RiotShieldDef`, naming both; the Barracks trains the first, and the Unit Control, a sum over
  the objects, is the same. `Object::riot_shield` (`RiotShield { up }`) is how many ticks of the toggle time the
  shield is up (`RiotShieldDef::toggle_ticks`, 6, design change 0174); the type says which way it goes (*Object model
  > Type and form*), and `riot_shield::mode` tells it as the design does (`ShieldMode`: Lowered, Deferred, Raising,
  Raised, Lowering). `Command::Ability` switches the type at the press, a raise that waits for a shot included
  (design change 0198; `Sim::change_kind` from `sim/modes.rs`: the orders, the move, the HP and the whole attack
  kept), and the Modes step takes the shield a tick on (`riot_shield::step`), so a toggle in tick `t` is done at the
  end of tick `t + toggle_ticks − 1`; a reversal turns back from where the shield is (0171). A raise pressed during
  Firing or Cooldown waits for the shot, the shield down meanwhile (0172, `ShieldMode::Deferred`, *Changes that wait
  for an attack*), and a lowering pressed then drops it. From the start of a raise to the end of a lowering the unit
  moves at half speed (`Mover::speed_factor`, from how far up the shield is: `riot_shield::speed_factor`) and can't
  attack (`riot_shield::allows_attack(object)`, the one "may it attack now" check: the behaviours, 5e and
  auto-targeting ask it; the client draws the shield from `riot_shield::mode`). Which orders it takes follows its
  type (design change 0202, `riot_shield::takes_attack_orders(kind)`: the gate, the default command and the client's
  panel and right-click ask it): a Peacekeeper (Shielded) takes no Attack (the gate drops it from new Attack orders,
  `Reject::CannotAttack` when none is left; design change 0075), and a right-click on an enemy makes it follow (0074);
  a Peacekeeper whose shield is on its way down takes one, "accepted and held", and attacks once the shield is down.
  Meanwhile its behaviours see no attack (`Unit::attrs`: no auto-targeting, Hold disengages), and 5e cancels an
  aim (`can_attack_now` is false); a shot already being fired lands. **Orders are kept** (design change 0150): an
  Attack it had goes on hold, following its target as a Move to it would (`AttackOrder::on_hold`), and attacks again
  once the shield is down; an AttackMove or a Patrol only moves meanwhile (0076); the toggle never touches the queue,
  and Shift means nothing for it (0151). Only once the shield is all the way up is the damage it takes halved, a
  pre-armor multiplier (`riot_shield::damage_factor` in `damage::pre_armor_multiplier`, with the Rugged bonus; 0171:
  "a half-raised shield gives none"). A mode is not a task: the unit keeps its orders and yields like any other
  (design change 0023). The placeholders are listed in the module docs. The toggle doesn't use the conversion's
  machinery (`conversion.rs`), whose change runs to its end once begun and holds the unit still; they share the
  type-and-form model, `Sim::change_kind` at the press, the wait for a shot and the Modes step.
- **Projectiles** (`combat/projectiles.rs`, slice M4-O, plan decisions T45 and T48; `systems/combat.md >
  DoublyDisjointed`): a DoublyDisjointed attack (`Shooter::locks`) locks its target's position at the end of Aiming
  (`AttackPhase::FiringAt { at, target }`) and at the end of Firing launches a `Projectile` into `Sim::projectiles`, in
  launch order: its shooter and the shooter's owner, the object it was locked on (cleared when that object leaves:
  `projectiles::forget`; a slot for the HeadDisjointed attack to come), its start, the locked
  point, its arrival tick (`flight_ticks`: the distance over the attack's ProjectileSpeed, rounded up: 32 ticks for 4
  GU at 8 su per tick), and the attack's Damage, AoE radius and TargetDomain, copied at launch. The unit goes on into
  Cooldown and Reloading, its base free from the launch (`base_is_free`). In 5e, after every attacker has acted, the
  due projectiles land in launch order (`projectiles::land`), whether their shooter lives or not: each a
  `SimEvent::Blast` and its area's hits, put on the tick's list before the single-target hits. The store is serialized
  and hashed only while it holds a projectile, and `AttackState::location` only while set, so a game without them
  hashes as before (the goldens unchanged). Its golden, a fight of test throwers (`combat::testing`), is a unit test
  (`combat/projectiles/golden.rs`, see *Replays and golden hashes*), and so is the ignored `projectiles_perf` (200 in
  flight). The game's thrower is the Grenadier (slice M4-U, `defs/syndicate/units/grenadier.rs`), played with its
  own numbers in `tests/grenadier.rs`; the mechanics' tests keep their test throwers, whose numbers stay put.
- **TailDisjointed attacks** (`combat/phases.rs`, slice M5-O3; `systems/combat.md > TailDisjointed`, the Cults'
  Gunner's, design change 0304): locked as a DoublyDisjointed attack is (`Shooter::locks`, `AttackPhase::FiringAt`),
  at an object or a location, with no projectile. At the end of Firing it strikes the locked spot (`phases::strike`):
  a single-target attack hits what stands there then (`phases::at_the_spot`: a destructible object whose shape, a
  Silhouette or a footprint, covers the point, in a domain the attack can target, whoever owns it, seen or not), with
  its single-target hit and a `SimEvent::Attacked` drawn to the spot; with nothing there it misses
  (`SimEvent::Missed`), so a target that has left the spot dodges. Placeholder (not in the design): of two objects at
  the spot, the one aimed at while it covers it, else the first in id order. An area one (the Artillery Rig's, not
  built) blasts the spot as a landing projectile does. The base stays held through Firing and Cooldown, as for a
  FullyConnected attack. No unit walks off the spot in the Gunner's two frames of Firing: each one's Silhouette
  reaches further from its centre than it moves in two frames (the nearest, the Drone's: 18 su against 16). So its
  tests move the target with a debug teleport (`tests/gunner.rs`), and the dodge on foot is tested with a longer
  Firing (`combat/phases/tests/tail_disjointed.rs`).
- **Area damage** (`combat/area.rs`, plan decision T46; `systems/combat.md > AoEDamage`): the one path of every area
  attack, a blast's hits. A blast takes every object whose domain its TargetDomain hits (`hits_domain`: Ground hits
  ground and underground objects, Air the air, Universal all; design change 0262: underground and hidden ones too),
  whoever owns it, its attacker and the attacker's own objects included (design change 0293, answering Q0083 point
  1: friendly fire), by the exact overlap of its circle with the object's shape (`area::overlap`, a closed form
  through `math::asin` and `math::sqrt`, whose bits a test pins): a share of the Damage of overlap / circle, the
  FullArmor times overlap / shape, the directional zone from the blast's centre, a minimum of 0.5 × the overlap
  fraction (`Hit::minimum`), the Shields first at full value (`damage::resolve`). Nothing in it asks `hidden_from`.
  The reveal after the hits (see Vision) shows the attacker, if it still exists, to each damaged object's owner, when
  the damage lands: a dodged grenade reveals nothing (design change 0293, point 2).
- **Attack Ground** (`orders/attack_ground.rs`, plan decision T47; `systems/controls/command_behaviors.md >
  AttackingLocation`): `OrderKind::AttackGround(point)` sets `AttackState::location`; 5e aims at a point in reach
  (`reaches_point`, from the unit's shape) and fires. The behaviour walks the unit into range, and backs it off a point
  inside its MinRange by an Attack's own rule and code (`engage::too_close`, slice M5-O3; *Behaviours*), letting go of
  the location while it does; a unit that can't back off gives the order up with `SimEvent::LocationTooClose` ("Target
  too close").
  **It keeps firing** (design change 0293, answering Q0083 point 3): the location stays engaged after each shot (a
  launch, or a TailDisjointed strike: the Gunner's, slice M5-O3), so
  the attack cycles at it until another order or a Stop, and the orders queued behind it never start on their own;
  only the Failure rule, or a weapon that can no longer attack the location, ends it. The gate takes it only for units
  whose weapon can target the ground (`combat::can_target_ground`, `Reject::CannotTargetGround`).
- **Structures that attack** (`combat/structures.rs`, plan decision T49; slice M4-S1, the Laser Turret,
  `factions/syndicate/structures/laser_turret.md`): **`StructureDef::attack`** is the type's AttackAttributes
  (`combat::weapon_of` reads a unit's or a structure's), and **`phases::Shooter::new(map, object, attrs, sight,
  turned)`** takes the attack and the object, a unit's silhouette or a structure's footprint (`Shape::of`,
  `Elevation::of_object`), so a structure runs the units' phases and deals their damage, with no turn rate (it fires
  in any direction: `turn_left` infinite, it faces its target as a turret does) and no orders. In 5e, before its
  sequence, a complete structure (`Object::is_operational`: a site holds its fire, and is no threat) **picks its own
  target** (`structures::pick`; design change 0298 point 8, `factions/syndicate/structures/laser_turret.md >
  Targeting`: "targets like a unit holding position: BaseAutoTargeting within Range ..., then the closest ... It
  **keeps its target** while the target stays valid and in range"): an enemy's ValidTarget within Range, threatening
  targets first (`TargetView::threat`, `combat::threat`: an attack that can target the ground), then the smallest
  gap, then the lowest id; none while auto-targeting is off. Its `facing` snaps to what it fires at
  (`structures::face`), for the client's barrel only. Its `AttackState` comes with its kind (`sim/kinds.rs`,
  `Capabilities`), so hashed and saved as a unit's; the goldens, which have no such structure, are as they were.
  **A pop-up fires only while raised** (slice M4-S2, `popup::may_fire`; "It can't fire while rising or retracting"):
  lowered, rising and retracting it picks and faces its target as ever and starts no sequence (an aim is cancelled, a
  reload runs on); **and only while active**, inside its Tunnel's area (design change 0360): an inactive one, a
  neutral one among them, stays down and never fires. Its armour is `Object::armor()`, the underground one while
  fully lowered (*Object model > The pop-up form*).
- **Units' turrets** (`combat/turret.rs`, `orders/turret.rs`; slice M5-U1, plan decision T80; `systems/combat.md >
  TurretSource`, `systems/controls/commands.md > TurretCommandState`): a unit whose type has TurretAttributes
  (`UnitDef::turret`) attacks from its turret (`turret::Source`). **`AttackState::turret: Option<TurretState
  { facing, locked }>`**: its facing relative to the base's, within ±TurnAngle/2, and its LockedTarget; none until it
  first locks or turns (a default one is a turret centred with nothing locked), so it is saved and hashed (a tag) only
  while set, and every state without one, the goldens', is as it was. `AttackState::target` stays what the base's
  behaviour engages; the turret's sequence runs at `locked` (`Shooter::aimed`). **In 5e**, before the phases
  (`turret::step`): a lock its attack can't target at all is let go, then the turret turns toward an Attack Ground's
  location or its LockedTarget, the latter only while its owner sees it (a placeholder: turning to an unseen one
  would show the player where it is), by its TurnRate, a full circle (`turret::full_turn`, within `FACING_TOLERANCE`
  of 360°) the shorter way, a limited arc only within it, to its edge toward a point outside; not when the tick
  starts in Firing or Cooldown (their TurretSourceActions are None). A sequence starts at a seen lock in range and
  in the arc (`turret::in_arc`, bounds included), Aiming lasts until the turret faces it too and is cancelled when
  it leaves the arc, and an Attack Ground's location takes the same arc. The base turns only in 5d: `base_is_free`
  holds throughout. **In 5a**, right after the base's behaviour, the turret's own (`orders/turret.rs::run`): it keeps
  a lock its base engages; any other it keeps only while it could fire at it from where it stands (`fireable`: seen,
  a valid domain, Range and MinRange, the arc; a placeholder, as the design never says when a turret lets go), else
  lets it go; with nothing locked it scans (`World::turret_pick`: `World::pick` with `Scan::Weapon` and the arc as
  `admit`, a Move's tick included) and locks the best. The rotation a scan measures, the base's
  (`BaseAutoTargeting`) and the turret's alike, is from the turret's facing (`Unit::aim_facing`, design change
  0179). **Who sets the lock**: `close_in` once the target is in range and in the arc (out of the arc the approach
  goes on, the move turning the base), the idle and the AttackMove acquisitions; a Hold's base engages nothing and
  leaves it to the turret's scan; a Stop clears it (`orders::run::start`); every other order leaves it ("When the
  base behavior does not specify a target, the turret retains its previous TurretCommandState"); the removal path
  clears it (`AttackState::forget`); a kind change keeps it only for a type with a turret (`sim/kinds.rs`). **Not
  shared with the Laser Turret** (*Structures that attack*), which has no arc and no turn rate, snaps its facing and
  picks by the gap in 5e: its code and its golden (`cults_vs_syndicate`) are as they were. Tests on test turret units
  (`combat/turret/testing.rs`: the Guard on a Mech's base, a 90°, 45° or 360° turret, a fixed gun), in
  `combat/turret/tests.rs` and `orders/turret/tests.rs`; the ignored `turrets_perf` (`combat/turret/perf.rs`, 96
  turret units passing each other) runs as `projectiles_perf` does.
- **Not yet**: the HeadDisjointed attack, which hits as a FullyConnected one does (a TODO in `combat/phases.rs`).

## Vision

Slices V and F (`vision.rs`, `vision/`; `core/vision.md`). Every tile is `VisibilityState::Unexplored`, `Explored`
or `Visible` for each player, and each player remembers the structures and resources it has seen.

- **Sources**: every object with an owner and a SightRange above 0 (units and structures; an Extraction Plate has
  none); vision goes to its owner. Each sees a disc around its centre (a structure's: its footprint's centre): a
  tile is seen when its centre (`TILE_SIGHT_POINT`) lies within the SightRange, boundary included, with the
  ElevationModifier between the object's elevation and the tile's level at its centre (`elevation::sight_range`: +1
  GU towards lower tiles, −1 towards higher, none for air). No line of sight: terrain blocks nothing.
- **Incremental stamps**: each source keeps its `Eye` (owner, centre, SightRange, elevation) and the sorted tiles
  it stamped. In 5f a source whose eye is unchanged costs nothing; one that changed stamps again and only the
  difference changes the per-player counts of sources seeing each tile; a source that is gone takes its tiles away.
  A tile is Visible while its count is above 0 and for the fog delay after, and Explored once it has been Visible.
  `PlayerVision::revision` changes when any of the player's tiles changes state, so views redraw only then.
- **Fog delay** (`vision/linger.rs`, `core/vision.md > Visible > Fog delay`, design change 0178): a tile that no
  source of the player sees any more in 5f of tick `t` lingers, genuinely Visible, at the end of ticks `t` to
  `t + FOG_DELAY_TICKS − 1` (8, a placeholder); 5f of `t + FOG_DELAY_TICKS` turns it Explored unless a source sees
  it again first. It is the one fix for a target blinking out at the edge of sight: Attack and Follow have no grace
  period of their own, and end once the delay is over (0200). Per player, `Linger` keeps a bit and an end tick
  per tile and a FIFO of `(end, tile)` in the order the delays end (a tile back in sight leaves a stale entry that
  the pop skips), so 5f costs the tiles that leave sight or come back, as before, plus the entries due, never a pass
  over the map. A delay that ends changes the revision, and the memory takes it as the tile leaving sight. It
  bridges sight that recedes from a target (a spotter walks off and a chaser's own sight takes over); a target that
  steps onto a tile never seen, ahead of the front edge of sight, is out of sight at once
  (`tests/fog_delay.rs`).
- **Attackers revealed** (`vision/reveal.rs`, `core/vision.md > Attacks from outside vision`, design change 0177):
  when a hit lands in 5e of tick `t` from an attacker that none of the victim's owner's sources sees (by the vision
  of the end of `t − 1`, the attacker's box where it fires from; the fog delay doesn't count: independent timers),
  the attacker alone, not its tiles, is Visible to that player at the end of ticks `t` to `t + REVEAL_TICKS − 1`
  (24, a placeholder); each such shot renews it. `PlayerVision::reveals` maps each attacker revealed to its end,
  and 5f drops those that end. Every attacker, whatever its range; a neutral victim reveals nothing.
- **Memory** (`vision/memory.rs`, `core/vision.md > Explored`): a `LastKnown` record per tile-based object (a
  structure or a resource) the player has seen and doesn't own: kind, owner, footprint, HP, resource state, a Tunnel's
  tier, a site's progress (`LastKnown::site`, slice M4-I2: the `Progress`, not who builds it; hashed only while set)
  and the tick it was last seen. An object is in a player's sight while a tile under its footprint is Visible; its record
  is rewritten at the end of every such tick (`Vision::remember`, the last phase) and stays as it was out of sight.
  A record whose object is gone, or has moved, is dropped once a tile under the record is Visible: an object
  destroyed in the fog is remembered until its place is seen. An object placed on Explored tiles gets no record, so
  it stays unknown until seen. Own objects are never recorded. **The map's neutral objects are known from the
  start** (design change 0159, `core/vision.md > Explored`: "like terrain"): as the game starts (`Sim::settle_start`)
  `Vision::seed` gives every player a record of each object without an owner (the crystal patches, the Supply
  Delivery Stations), `seen` 0, before the first sight; the players' Starting Layouts are never pre-known, and an
  object that appears later is unknown until seen.
- **Memory updates follow the vision's changes**, never a scan of the map or of every object for every player:
  5f's id-order walk over the objects also merges the footprints of the tile-based objects (`Footprints`), so only
  a footprint that came or went touches the index from tiles to objects; `see` and `unsee` note the tiles whose
  state flips; after the walk a noted tile under no footprint and no record costs two bit tests, and on the others
  objects enter or leave the sight and the records seen gone are dropped; `remember` reads only the objects in each
  player's sight. Measured (`tests/vision_perf.rs`, ignored, release): 500 moving units and 168 crystal patches on
  256 × 256, `update` and `remember` 0.17 ms per tick on average, 0.2 ms at the 99th percentile.
- **State**: the Explored bits, the records, the tiles that linger with their ends and the reveals are history, so
  they are hashed and serialized (each player's rows of `#` and `.`, its `last_known` records, and its `lingering`
  and `revealed`, both `#[serde(default)]`). Eyes, stamps, counts, the footprint index, the delay's queue and the
  objects in sight are derived: a loaded save rebuilds them from the objects (`vision::load`, which rejects a record
  twice, of an own object, off the map, or seen after the save's tick, and a delay or a reveal that ends in the past
  or beyond its length), and invariant V1 (`testing::invariants`) rebuilds them from scratch, from the same history,
  and compares.
- **Queries**: `Sim::vision()`, `Sim::visibility(player, tile)` and `Sim::is_visible(player, object)`: an object is
  seen while it is revealed or when any tile it covers is Visible (its silhouette box or footprint,
  `combat::object_box`). `Vision::player(id)` gives a `PlayerVision` with `state`, `states` (row-major, for the
  client's fog texture), `lingers(tile)`, `is_revealed(id)`, `object_state`, `sees`, `sees_box` (tiles only), and
  `object_box_state` and `sees_object_box`, the one object query that combat, `World::sees`, the gate and the
  client's `Sight` all go through; and for the records `records` (by id), `record(id)`, `remembered` and
  `remembered_record(id)` (those not in sight, which Explored tiles show), `record_tiles` and `is_record_tile` (the
  tiles under any record) and `records_revision` (changes when a record comes, goes or moves; not when only its
  state changes).
- **Hidden objects** (`vision/hidden.rs`, plan decision T32, slice M4-A): an owned underground structure (a
  Syndicate expansion, `StructureDef::underground`) is hidden from every player but its owner; a neutral one isn't
  (`factions/syndicate/tunnels.md > Losing Tunnels`). One predicate, `vision::hidden_from(object, player)`
  (`Sim::hidden_from` for the client), through `Hiding::of(object)`, which the snapshots carry (combat's
  `TargetView::hiding`, `World`'s views) and `object_box_state` takes: a hidden object's state is capped at Explored,
  however Visible its tiles are and whatever is revealed, so targeting, `World::sees`, the gate and the client's `Sight`
  never see it. The memory never records it (`remember` drops it, `Vision::seed` skips it). A neutral expansion (its
  Tunnel lost, slice M4-H) is seen and remembered by whoever sees its tiles; reclaimed, it is hidden again: the other
  players' records of it go when they see its tiles, and its new owner's at once (`Vision::forget_own`: a player never
  remembers its own objects, whether or not it sees them). **Damage never asks**
  (`systems/combat.md > AoEDamage`, design change 0262, keeping 0218: underground objects "take their share of any blast
  over them, even unseen"): a hit on the tick's list lands whoever sees its target (`combat/damage.rs`). Hidden units
  come with the Tunnel network (design change 0219). **A pop-up turret is hidden while fully lowered** (slice M4-S2,
  design change 0296 point 1; `popup::is_lowered`), by its state, which its kind doesn't tell: seen and remembered
  like any structure while it rises, stands or retracts, and its record dropped once its tile is seen with it down.
  A neutral one is lowered for good and hidden from nobody (design change 0360: "visible to everyone"), as a neutral
  expansion is. **The hook for a new arm of the rule** is `Hiding::of` in `vision/hidden.rs`, alone, where that arm is:
  vision, targeting, the records and the client's `Sight` ask it of the live objects, a death carries it
  (`SimEvent::Died::hiding`, slice M4-V), and the client's filter of events takes it for a removal (*Client
  architecture > Viewer events*).
- **Paths** are planned from what each player knows (design changes 0040 and 0042; *Navigation > Path knowledge*):
  the records' footprints are the other players' structures and the resources it knows of, the map's from the start
  (0159); its own come from the live objects, as they are never recorded.

## Behaviours

Slice E (`orders.rs`, `orders/`, phase 5a's loop in `orders/run.rs`; `systems/controls/behaviors.md`). Phase 5a runs one
behaviour per unit and tick: its current order's, or the idle one. A behaviour never moves or shoots anything itself: it
writes the mover's goal and the target the attack engages, and reads `Mover::status` and the snapshot.

- **State** (`orders/state.rs`, the design's BaseBehaviorState): `ActiveOrder::state` (`Behaviour`: `Plain`,
  `Attack`, `Approach`, `AttackMove`, `Patrol`, `Hold`, `Gather`) and `Orders::auto` (`Auto`: what a unit without an order
  does: `Watching`, `Engaging`, `Returning`). Hashed and serialized like all state; every order that starts resets
  `auto`. A state names objects only by id, and the removal path clears them (`Orders::forget`).
- **The snapshot** (`orders/world.rs`): `World` holds a `View` of every object (position, shape, owner, domain,
  elevation) as the last tick left it, the vision of its phase 5f, and a coarse grid of the objects a scan may
  pick. Every unit decides on the same world, whatever its id. `World::pick` is the scan of
  `systems/controls/behaviors.md > BaseAutoTargeting > Target Selection Priority`: among the ValidTargets in reach
  (enemies only: another player's destructible objects that the unit's owner sees and its attack can target),
  threatening ones first, then the least rotation from the unit's facing **in steps of 45°** (design change 0179,
  `world::rotation_step`, a placeholder step: each step reaches up to its upper bound, so a target exactly 45° off is in
  the first), then the smallest gap, then the lowest id. A scan of SightRange skips enemies inside the unit's MinRange
  (design change 0182) and picks one it can hit; the caller may leave out more (`pick`'s `admit`: the AttackMove's
  leash). Enemy structures are ValidTargets too (design change 0103), so units attack enemy bases on their own; the
  structures that can't attack come after the targets that shoot back. The grid holds candidates by their centres, so a
  scan looks as far as the unit's longest range with the modifier, plus its own half size and the largest candidate's
  (`World::candidate_half`): a footprint in reach is found however far its centre lies.
- **AttackingObject** (`orders/engage.rs`): `close_in` is its core, shared by all fighting. In range (`combat::
  reaches`) the unit stops and phase 5e attacks; out of range it heads for the target with `MoveGoal::Object`,
  starting only while `combat::may_move` holds. An **Attack order** adds its ends: the target destroyed (the removal
  path completes the order); the target **lost from its owner's vision** (design change 0200, which replaced 0034's
  walk to the last seen position): once its owner has seen it while the order ran (`AttackOrder::seen`), the order
  completes at once, as for a destroyed target, and the unit stops where it is (or the next queued order starts);
  the approach failed, or ended as near as the unit gets to a target that hasn't moved.
  **A remembered target** (design change 0139; the snapshot's `World::known`, the owner's `TargetView`) not seen
  since the order began: the unit walks to its last-known position, engaging nothing (`AttackOrder::remembered`),
  attacks it once its owner sees it standing, and stops where its owner sees its place empty, "the way it would for
  a vanished target" (the next queued order starts). Once seen, it is lost as above when the fog covers it again.
  The removal path leaves such an order alone (*Combat > Removal path*).
  **A target inside MinRange** (`engage::too_close`, design change 0182): a unit that can move backs straight away
  from it, to where the gap would be the middle of the band from MinRange to Range (`AttackOrder::backing_off`), and
  attacks once the back-off ends: it goes on while the target is within Range, until the move ends there or where
  the unit is blocked (question Q0054: "not just past MinRange"; slice M5-O3 fixed M5-A's back-off, which stopped
  as soon as the target was out of MinRange). The unit engages nothing meanwhile, so that 5e doesn't aim from just
  past MinRange and hold it there, and engages the target again as the back-off ends, or as the target leaves its
  Range. One that can't move (`Mover::can_move`) fails the order (`AttackOrder::too_close`,
  `Sim::step_too_close`, `SimEvent::TargetTooClose`; the client's feedback line says "Target too close"), and so
  does one whose back-off ended with the target still inside and where it was: it can't back off ("cornered, or
  blocked behind", design change 0204 point 3; slice M5-A, which gave it the feedback). An Attack Ground's location
  takes the same rule through the same code (slice M5-O3: `TooNear` is what the unit backs off from, `BackOff` the
  order's parts the rule works on), and fails with `SimEvent::LocationTooClose`.
- **Idle units** (`engage::idle`): scan their SightRange every tick; the acquired enemy is engaged with `close_in`
  from the IdleOrigin, where the unit stood. Farther than `IDLE_LEASH_GU` (4) from it the unit disengages and walks
  back without scanning; a target destroyed or out of SightRange leaves it idle where it is.
- **Hold Position** (`engage::hold`): scans its weapon's range only, never moves, and disengages the moment the
  target leaves that range. A unit with a turret engages nothing from its base: its turret scans and fires on its own
  (slice M5-U1, *Combat > Units' turrets*).
- **A turret's behaviour** (`orders/turret.rs`, slice M5-U1): runs for every unit with a turret right after its
  base's behaviour, whatever that is, a Move's included; it locks and lets go of the turret's target and never moves
  the unit (*Combat > Units' turrets*).
- **Move to an object** (`orders/travel.rs::approach`; design changes 0032 and 0033): proximity is a gap of at most
  `PROXIMITY_SU` (8). A static object completes the order there (also at the nearest point the unit can reach, or
  stuck within 3 GU of it behind units that won't move). **A unit is followed**: the order never completes by
  itself; the follower waits within proximity (or where it was stopped near it, without re-routing) and sets off
  again when the target moves. It ends with a new order, the target's death, a failed move, or an enemy target
  going out of sight: the follower stops where it is (0200). "Leaves the map" (transports, buildings, tunnels)
  is a documented hook in `approach`. A remembered object (0139) is approached at its last-known position, as an
  Attack's. **A Move or Follow whose target is the unit itself is a Stop** (design change 0168): it halts, and the
  order ends at once, as if arrived; a Shift-queued one completes in the tick it starts; the other units of the
  order follow it as usual (a right-click on one of the selected units).
- **AttackMove** (`travel::march`): one `Leg` to the unit's own destination (the magic box). While moving it scans
  its SightRange and engages with `close_in`, by **the reach rule** (design changes 0230 and 0235, `orders/reach.rs`,
  `travel::acquire`): an enemy within weapon range is always taken; one out of range only if the path to a firing
  position (the path phase 5c would serve the chase, `World::chase_path`, walked in quarter-tile steps,
  `reach::judge`) stays within `ATTACK_MOVE_LEASH_GU` (6) of its **PathReference** and keeps the target within the
  unit's SightRange, measured as the scan measures it. **The leash limits chasing only** (design change 0180): a
  target in weapon range is always fought, wherever the unit stands (`travel::pursue`); a chase goes on while the
  unit stands within the leash and its judgement stands (`Leg::chase`), and is judged again otherwise. The
  PathReference (`Leg::reference`, `Leg::follow_path`) is the first path the leg's move gets; it is kept after fights
  (the move's path then, the walking path, is `Leg::walking`) and replaced by a new path the unit gets while it walks
  its leg: a re-route round a blockage. The unit lets go, and resumes the move with a new walking path, when the
  target is destroyed, rejected or no longer valid (out of sight: its attack completes, design change 0200).
  - **No memory of targets let go** (0230). A rejection (`Leg::rejected`, `state::Judged`) only saves judging an
    enemy again, and stands only while nothing the judgement depends on has changed: the enemy within
    `REJUDGE_MOVE_SU` (1 GU, a placeholder) of where it was judged, the owner's `PlayerKnowledge::version` the same,
    the PathReference the same (`Leg::set_reference` clears them). A rejection by the reach rule (a chase the unit
    can't finish, or one that ended beyond the leash) also records where the unit stood (`Judged::from`), and stands
    only while it stands within `REJUDGE_SELF_SU` (1 GU, the design's placeholder) of there: design change 0243,
    answering Q0074, "so the same situation always gets the same response" (a unit that rounds a wall judges the
    building it passed over again, as a fresh unit there would). A chase that ends without the target reached is a
    rejection too: given up by the Failure rule or as near as the unit gets (0235), beyond the leash, or out of the
    owner's sight (then where the target was judged, never where it went); the Failure rule's and the fog's keep
    0230's triggers alone (`from` unset), so the units behind their own side in a corridor don't take the target up
    again as they shuffle. Any state that makes a unit pass over an enemy must follow this pattern: a cache of a
    judgement with its inputs, never a list of ids.
  - **The cost bound**: a unit plans at most `JUDGE_PATHS_PER_TICK` (2) judgement paths per tick, its candidates in
    target-priority order, the rest in later ticks (an enemy within weapon range is still taken at once); an enemy
    farther from the PathReference than any firing position could be costs none (`reach::near_route`); and a
    judgement's search gives up beyond `JUDGE_PATH_SIGHTS` (2) SightRanges of path (`nav::find_path_within`), so it
    never floods the map round a long wall. The searches run in phase 5a on their own scratch (`reach::Reach`, kept
    in `NavState`), outside `PATH_BUDGET`; `Sim::reach_counters` reports them. The rejections and the chase's
    judgement are hashed and serialized only while set, and so is a judgement's `from`. The own-movement trigger
    costs a path for each grid unit a unit walks with a rejected enemy in its sight, within the same bound
    (`attack_move_perf.rs` measures it).
- **Patrol** (`travel::patrol`): legs to and fro between the position where the Patrol started executing (design
  change 0036) and the destination. A failed leg counts as arrived; `PATROL_FAILED_LEGS` (2) in a row end it (design
  change 0037).
- **Gathering** (`orders/gather.rs` with `gather/patch.rs` (the job at its patch), `gather/station.rs` and
  `gather/station/spot.rs` (the job at a Supply Delivery Station), `gather/deliver.rs` (the way back and the line at
  a hatch), `gather/claims.rs`, `gather/search.rs`, `gather/legs.rs` and `gather/site.rs` (the patch or the station as
  the owner knows it, design change 0139), `orders/world/gathering.rs`;
  slices M3-H, M3-H3, M4-D, M4-I and M5-D, plan decisions T17, T34 and T58; `factions/colonists/units/drone.md`, whose rules
  design changes 0123 and 0127 to 0141 settled, answering question Q0037; `factions/syndicate/units/agent.md >
  Gathering`, design changes 0255 and 0256): one behaviour for the Gather and the Drop Off order of **every worker
  type** and both resources, in
  the phases of `GatherPhase`: ToPatch, Waiting, Mining, ToDropOff, AtHatch, DroppingOff, Delivered. What differs
  between the Drone and the Agent is data: the worker's `WorkerDef` and the drop-off's `StructureDef::drop_off`.
  - **Agents** (M4-D): `WorkerDef::shares_patches` takes a type out of the occupancy below altogether (it neither
    occupies, waits for nor claims a patch, and isn't in `World::patch_mined` or `waited_at`, so a Drone and an Agent
    mine one patch together); with `WorkerDef::drop_off` (`DropOffChoice`) `FirstReturn` and not `Assigned` (the
    Drone's) a Gather begins with no AssignedDropOff and takes the nearest
    drop-off as its first return starts, keeping it for that Gather (there is no Set Drop-off for the type: the gate's
    `Sim::sets_drop_off`). "Nearest" is by the centre of the crystal point, whatever the load
    (`economy::drop_off::nearest`, the one rule of the return leg and of the removal path): a Tunnel's crystal hatch
    (design change 0292 point 2: "the same measure as for crystals"), an Egress Node's own centre.
  - **Recruits** (M5-D, plan decision T58; `factions/cults/units/recruit.md > Gathering`,
    `factions/cults/structures/storage.md`): data again. `shares_patches`, no `pass_through` (design change 0106:
    Recruits collide while they gather), the Storage's `DropOff::Anywhere`, and
    `DropOffChoice::EachReturn` (design change 0345, answering Q0102): as each return starts (`state::Gather::node` is
    `None`) the Recruit takes the own completed Storage nearest to its patch, **edge to edge**
    (`deliver::nearest_to_patch`), and with none it idles holding its load. One lost on the way is replaced by
    the removal path as any worker's is. Within what the design says a face holds (4 to 5 Recruits a 2-tile face)
    nothing jams; with more Recruits than lanes they stand head-on and starve one another, which no existing rule
    resolves (question Q0109: `tests/recruit_gathering/jam.rs`, ignored).
    **Measured** (`tests/recruit_gathering/`): one Recruit at gaps of 1, 2 and 3 tiles makes a trip in 24, 56 and 88
    ticks (1.33, 0.57 and 0.36 Space Crystals per second; the design's formula gives 1.45, 0.59 and 0.37); 10 on an
    8-patch field with three Storages, 12.9. `tests/recruits_perf.rs` (ignored, release): 60 Recruits on 22 patches
    with four Storages, a mean of 0.062 ms and a p99 of 0.089 ms a whole sim tick (load average 2.7).
  - **Supplies** (M4-I; `factions/syndicate/units/agent.md > Gathering > Supplies`, design change 0256): a Gather on a
    Supply Delivery Station (`Site::Station`, the station as the owner knows it) runs the same phases, for a worker type
    with `WorkerDef::supplies` (the Agent; the gate's `Sim::gatherers_among`, `resolve`'s station rule). `Mining` at a
    station is the pick-up, PickUpDuration long; `economy::gathering::step` tells a station from a patch by the target
    (`economy::stations`). **No occupancy** (point 1): each worker heads for a free spot of its own round the station
    (`station::spot`: the nearest point against a face, or beside a face's end within reach, clear of known
    structures, of the units its owner sees there and of the spots claimed in this tick, `Claims::take_spot`), since
    Agents collide. **Claims on the Supplies** (point 2): a pick-up begins only while the station's Supplies outnumber
    the pick-ups running as 5a began (`World::unclaimed_supplies`) plus those begun in this tick
    (`Claims::take_supply`); the Supply leaves the station as the pick-up ends (`stations::take`), never before, and
    a pick-up stopped earlier takes nothing (0130). **An empty station** (point 3): the worker waits beside it
    (`Waiting`), keeping its Gather, for ever, and picks up as soon as a Supply is free; one that can't get within
    reach sets off again every `RECHECK_TICKS`, for a spot free by then (0141's rule). The pick-up that takes the last
    Supply emits `SimEvent::SupplyStationEmpty { station, owner }` once for each player whose workers' Gather names
    the station (the notification, a placeholder reading of "the owner"). **Switching resources** (point 4; design
    change 0292 point 3): `Worker::resource` says what the load is; a worker with crystals sent to a station (or
    with a Supply to a patch) goes straight there, and loses its load as the new pick-up or mining ends, with the new
    load in hand; one stopped before then keeps it. A worker with a Supply sent to a station turns round there and
    delivers it (0128). **The return** goes to the hatch of the load's resource: Side D for Supplies (design change
    0248), with its own one-at-a-time line, so a Tunnel takes a crystal and a Supply delivery in the same ticks; the
    nearest Tunnel is measured to its crystal hatch all the same, and both reaches are 8 su (design change 0292
    points 2 and 1).
  - **A hatch takes one worker at a time** (`defs::DropOff::Hatches`, the Tunnel; `deliver::at_hatch`). DropOffReach
    is measured to the hatch square. A worker within reach is `AtHatch { since }`: it unloads if nobody did as phase
    5a began (`World::hatch_busy`) or has begun since (`Claims::take_hatch`), and it is the first of the line
    (`World::first_in_line`: those within reach as the phase began, by arrival tick, then id). The line and the busy
    hatch are **derived from the workers' orders in the snapshot**, like a patch's occupancy, so they can't outlive
    an order. A hatch that frees at the end of tick `t` is taken in `t + 1`: a saturated Tunnel takes a load every 64
    ticks exactly, 12.5 Space Crystals per second. The leg back heads for the nearest point against the hatch's
    wall within reach of the square (`legs::approach_point` with `beside`), so waiting workers stand side by side
    (three or four fit). **One that isn't within reach doesn't wait: its leg presses on** (slice M4-W2, design
    change 0315: "Until it is within reach it is never 'arrived': it keeps actively moving toward the hatch"). The
    leg is exempt from the occupied-target rule (`Mover::presses_on`, set by `legs::return_leg` on the leg to a
    hatch only; `docs/movement.md > 6.5`, rule 1), so behind standing Agents it stays a unit on its way, and when
    the mover gives it up (stuck with no attempt left, 104 ticks) the next leg begins in the next tick, for ever
    ("It waits as long as it takes; it never gives up"). `LEG_RETRY_TICKS` (32) is left for a leg that ends out of
    reach with nothing in its way (the end of its path, or no path), and for the Drones' and Recruits' legs back. A
    worker pushed out of reach loses its place. **0315 doesn't end the jam of four or more Agents that come to the
    hatch along the wall** (`docs/playtest/m4.md > Known issues`, `tests/agent_gathering/jam.rs`).
  - **Hard collision at the hatch** (Agents don't pass through each other, 0106: "Drones only"): from 2 GU between
    hatch and patch on, Agents walk round each other and every one delivers (`tests/agent_gathering/`). At 0 and 1
    GU the strip is narrower than two Agents, each works in a lane of its own, and an Agent behind another in its
    lane blocks it head-on for good. Measured there with three and four Agents: one Agent's income. Open with the
    Designer (see the slice's report).
  - **Occupancy is derived, never stored on the patch** (design change 0133 and the Director's choice): a patch is
    taken while a worker of any player mines it (the snapshot's `World::patch_mined`), once a worker with a lower id
    claimed it in this tick (`gather::Claims`, the one thing of 5a that isn't the snapshot, kept for the tick only),
    and, for any other worker, while one waits at it within MiningReach (`World::waited_at`). A worker on its way
    occupies nothing ("Drones travelling to it do not count"); one that sets off for a free patch claims it for the
    tick, so that no other look in that tick picks it too. Of the workers waiting at a patch the lowest id mines it.
  - **Looking round** (`search::search`; 0131, 0141): among the patches the owner knows (seen or remembered, the
    map's from the start, 0159: `World::known_patches_near`, from its records and sight) that no plate it knows of
    covers, within the worker's `WorkerDef::search_radius` (6 GU, a
    placeholder, centre to centre) around the worker, the nearest free one it can reach (`World::reachable`: a tile
    against the footprint is in the unit's component of its owner's planning grid; the grids leave units out). An
    arrival at a taken patch, or a leg that ends short of one (0141), moves the **order itself** on to that patch,
    so `OrderKind::Gather` names the patch the worker heads for, and after a delivery the one it last mined (0132).
    With none free the worker waits, at its patch or as close to it as it got.
  - **Waiting** (`Waiting { since }`, 0132): the worker mines its patch as soon as it frees, and every
    `RECHECK_TICKS` (16, a placeholder), in the ticks that are multiples of it, every waiting worker looks round
    again: it goes to a patch that has freed up, or, short of an unreachable one, tries its leg again. One phase for
    all: counted from each worker's own start, the waiters looked round in different ticks and each went for the
    same freed patch (measured: 0.5 Space Crystals per second less with 20 and with 24 Drones).
  - **Patches by knowledge** (design change 0139, slice M3-R): the behaviour sees its patch as its owner does
    (`World::known_patch`: seen, or remembered at its last-known footprint). A Gather on a patch out of sight walks
    to its last-known spot (`legs::approach_point` on the known footprints, `World::known_fixed`), noting it in
    `Gather::spot` (`note_spot`), and mines it once its owner sees it there; one in reach but not yet seen holds a
    tick. A claim names the claimer's owner, and a patch out of sight counts as taken only by its owner's own claims
    and miners, so nothing of an unseen enemy leaks.
  - **A patch that leaves the game** (depleted, 0134; under a plate, 0137), or is seen gone: the removal path keeps
    its gatherers' orders on it and gives them its centre (`Sim::keep_gatherers_on_spots`, `Gather::spot`;
    `Order::outlives_target`, the one rule of *Combat > Removal path*). Those at the patch look round at once, one
    on its way when it gets to the spot, one carrying a load back after its drop-off ("the search radius is centred
    on the field"). With only occupied patches left it waits at the nearest; with none, the Gather ends.
  - **A Gather ends** only then (0123): the worker delivers what it carries (`Gather::ending`), the order completes
    and the queue goes on; orders Shift-queued behind a Gather, a Deploy as much as any, wait for that. **A Drop Off
    order** is only a delivery (0127): it completes at the drop-off, the worker idling there or starting its next
    order. A worker carrying a load that is sent to a patch walks there, doesn't mine, and returns to deliver
    (0128); other orders keep the load (0129).
  - **The return** reads the worker's AssignedDropOff every tick, so `Ability::SetDropOff` redirects it; without
    one as the return starts it takes the owner's nearest, and with none it idles holding its load (0135).
  - **A drop-off lost on the way** (0170, "switches at once") is replaced by the removal path, not looked for by
    the leg (`Sim::reassign_drop_offs`, like `keep_gatherers_on_spots`): a worker heading for it or unloading
    there takes its owner's nearest at the destruction, by the leg's rule, or none; the others forget it and take
    the nearest as their next return starts (0135). The loss is reacted to once, where it happens, and nothing
    checks every tick that the drop-off still stands. The leg turns in the first 5a after the loss, as it did; for
    a death in 5e the assignment shows from that tick's end.
  - **Legs are moves to a point** the behaviour picks, where the worker stands against the nearest open face of
    its target (`legs::approach_point`), and the behaviour measures MiningReach and DropOffReach itself. Not
    `MoveGoal::Object`: that is planned to the point nearest the object's centre, the north side of every patch
    and node. A leg to a patch that ends short counts as arriving there, out of reach (0141); a leg back to the
    drop-off that ends short begins again after `LEG_RETRY_TICKS` (32, a placeholder) of standing, a back-off;
    but an Agent's leg back to a hatch presses on instead (design change 0315, above).
  - **Gathering Drones pass through each other** (design change 0106; `gather::on_gather_job`, see *Movement and
    collision > Collision exceptions*): a worker of a type with `WorkerDef::pass_through` (the Drone) whose current
    order is a Gather or a Drop Off. They collide with everything else and, having an order, never yield.
- **Not active during Move**: a Move never engages, and neither does the walk back to an IdleOrigin.
- **Unreachable auto-targets** (design change 0181, "for now"): an idle unit doesn't re-acquire a target it couldn't
  get to for `GIVE_UP_RETRY_TICKS` (32). On an AttackMove or a Patrol such a target is rejected by the reach rule
  instead (0235). A target an explicit Attack gave up (`engage::attack`, the Failure rule) waits the same 32 ticks
  before the idle unit takes it again: the Attack leaves it in `Orders::auto` as the target given up.
- **Placeholders** (the design is silent; each is a comment or a named const in `orders/`): "within SightRange" for
  a scan is measured edge to edge like Range; a 45° step reaches up to its upper bound; a unit backing off
  from a target inside its MinRange goes to the middle of the band from MinRange to Range, and gives the order up
  when a back-off ends with the target inside and where it was (both written into the design since, by design
  change 0204, the second with the "Target too close" feedback, which slice M5-A added); a queued Attack or Follow
  whose target, a unit, is out of its owner's sight when its turn comes completes at once, as one lost from vision
  (0200).
- **Auto-targeting switch**: `DebugCommand::SetAutoTargeting { on }` (`Sim::auto_targeting`, hashed state) turns
  every scan off or on, for tests and QA runs about movement between hostile units; `TestSim::without_auto_
  targeting()` schedules it for tick 1. Attack orders work either way.
- **Adding a behaviour**: give it its state in `orders/state.rs` (with its hash, and its ids in `targets` and
  `forget`), start it in `run::start`, run it in `run::run` (`orders/run.rs`), and let it read only the `World` and its
  own unit. A behaviour that sends the unit somewhere must remember that it did (`chasing`, `marching`, `moving`):
  `Mover::status` is the outcome of the unit's latest move, whoever asked for it. Never call `Mover::go_to` tick
  after tick: each call is a path request and a fresh set of repaths.

## Navigation

The design is `docs/movement.md > 5` (binding for M1 movement), and the code is `crates/sim/src/nav.rs` and `nav/`.
The module is pure: it takes the `Map` and tile rectangles and knows nothing of `Object`, `Sim` or the tick.

- **`Nav`** holds one `NavGrid` per `(NavClass, size class)`: a blocked bit and a component label per tile. It is
  derived data, never hashed or serialized. `Nav::rebuild(map, blockers)` rebuilds it in full when the map or a
  blocking footprint changes; `Nav::restamp(blockers)` is the same for an unchanged map, keeping the terrain's
  part (about 0.17 ms a grid on 256 × 256, against 1.4 ms). The caller chooses the blocking footprints
  (`NavRules::footprint_blocks`). Units are never in a grid. Components are labelled by runs of free tiles, in the
  row-major order of their first tiles.
- **`Search`** is the scratch for `astar` and the Dijkstra `field`. Keep one and reuse it; nothing carries over
  between searches. Searches are integer-only. An unreachable goal never searches: retarget it first
  (`NavGrid::nearest_in_component`), as `find_path` does.
- **`los`, `box_clear`, `smooth`**: line of sight and waypoints for a unit's box (half extents). Touching is free.
- **`serve(nav, search, requests, PATH_BUDGET)`** serves a tick's `PathRequest`s in `(since, id)` order within the
  expansion budget, with one shared field per order group of `GROUP_FIELD_MIN` or more, anchored on the click's tile.
  A member whose goal the anchor can't see plans alone: through the anchor it would take a detour. The field is
  built only as far as the members stand (`Search::field_to`: towards their tiles' bounding rectangle, until each
  is expanded), and the members share the smoothing of the chains they share (`smooth_shared`, same waypoints as
  `smooth`). The budget's rule is unchanged: a job starts while budget remains and always finishes.
  `serve_on(navs, on, ..)` is the same for requests that plan on different sets of grids (the sim's: one per
  player), in the one order and budget; a field serves members of one set only.
- **`NavGrid::open_around(p)`**: the tile of `p` and its eight neighbours are free of solid tiles. The movement
  step asks it before `box_clear` and `los`, which hold there for a small box and a short move. A `Path` is unit
  state (serde, `PartialEq`, `StateHash`); `NavCounters` are output only.
- **Switches** of Q0011's answers: `RESOURCES_BLOCK` (design change 0038) and `RUGGED_IMPASSABLE` (0039).
- **In the sim** (`paths.rs`, `paths/`): `NavState` holds the **true grids**, the `Nav` of every unit type's
  `(class, size class)` under every blocking footprint (structures and map resources; an underground expansion's,
  `BlockerKind::UndergroundExpansion`, blocks none: units walk over it, plan decision T32 and design change 0240; nor
  does a pop-up turret's while it is fully lowered, `paths::blocking`, slice M4-S2, nor a neutral turret's record,
  `paths::remembered_blocking`, slice M5-Y), and each player's **known grids** (below). It is derived, like the `Nav`:
  not hashed, compared or serialized, built when a `Sim` is created or loaded. The true grids are rebuilt in phase 5b;
  `Sim::nav()` (the true grids), `nav_version()`, `nav_counters()` and `nav_rebuilds()` (sets of grids the latest step
  rebuilt) expose it to tests and overlays.
- **Path knowledge** (`paths/knowledge.rs`; `systems/controls/behaviors.md > PathKnowledge`, design changes 0040
  to 0042): paths are planned only from what the unit's owner knows, and units collide with what is there.
  - **What a player knows** to block the ground, besides the terrain: the footprints it owns, and those of its
    vision's records (seen and not seen gone, so a structure destroyed in the fog stays a wall at its last-known
    position, and one never seen is open ground). The map's neutral objects are records from the start (design
    change 0159, *Vision > Memory*), so a field never seen is routed around like terrain; the players' Starting
    Layouts are known once seen. (`PlayerKnowledge::assumed`, the resources assumed before 0159, stays only so that
    older saves load: a game starts it empty.)
  - **State**: `Sim::path_knowledge()` (`PathKnowledge`: per player a `version`, which counts the changes of the
    set of footprints it knows) is history, hashed and serialized; a save's is checked on load. The known grids (`Sim::known_nav(player)`) are derived from it, the objects and the vision.
  - **Plan on knowledge, collide on truth**: `NavState::planning(owner)` and `plan_grid` give the grids a unit
    plans on. Path requests, the staggered checks, a chase's line of sight and the nearest reachable point of a
    destination or an object goal use them. The movement step, the gate's spawn and teleport checks, scenario
    validation and invariant I2 use the true grids. Never plan on `Sim::nav()` for a player's unit, and never
    collide on a known grid.
  - **Re-route on discovery**: `Mover::nav_version` is the owner's knowledge version the path was planned on. On
    its staggered tick a unit whose owner's version moved on checks its remaining path on the known grid and plans
    again if it is cut, not counted against the repath cap. So a unit heading for a wall it didn't know about
    plans again within 4 ticks of its owner seeing it.
  - **Timing**: the known grids are brought up to date once per tick, at its end (*Tick pipeline*), so during a
    tick they are those of the end of the tick before (an own structure placed by a command joins them at the end
    of its tick), each player's are rebuilt at most once per tick, and a loaded save rebuilds exactly the running
    game's.
  - **Lazy**: a player's grids exist only while it has a unit that walks on a grid; `ensure_known` builds them
    for an order in the tick of its first unit's spawn.
  - **No owner**: a neutral unit (and a defeated player's, design change 0101) plans on the true grids by
    `nav_version`: it has nobody's knowledge and sees nothing. Code that changes a unit's owner should call
    `Mover::repath`, since the unit's path was planned on other grids.
  - Measured (`tests/nav_perf/`, ignored, release; 4 players, 412 units and 160 patches on 256 × 256): the tick
    in which ten Generators go in sight of all four players rebuilds 5 sets of grids and takes 1.0 ms; the worst
    tick after it, with the units' path checks, 1.3 ms. A set held one grid, `(GroundRugged, 1)`, until slice
    M5-G2: every unit type crossed Rugged terrain and was at most a tile wide, the Drone included (a HoverVehicle,
    design change 0039; on its stand-in mover it had a grid of its own, which doubled every set). The Cults'
    wheeled and tracked vehicles (slice M5-G2) don't cross it, so a set holds `(Ground, 1)` too, whether or not one
    is in the game (`paths::grid_keys`, every unit type's): A/B against master, interleaved, the rebuild tick went
    from 1.12 and 1.14 ms to 1.81 and 1.72 ms, the ticks around it unchanged (0.31 and 0.345 ms), and
    `structures_perf`'s whole tick from 0.349 and 0.344 ms to 0.360 and 0.356 (p99 0.66 to 0.71), its grid syncs'
    p99 about doubled and its other phases within noise (load average 2.9 to 6.5). Building a grid only once a unit
    of its class is in the game would take that back.
- **Group destinations** (`orders/destinations.rs`, `docs/movement.md > 6.4`, the design's magic box): a location
  order (Move to a point, AttackMove, Patrol) to several units looks at the bounding box of their silhouettes
  (design change 0028). A click inside it sends every unit to the click; outside, each unit goes to the click plus
  its offset from the box's centre. The order keeps the unit's own destination (`Order::slot`; `None` is the click),
  resolved once, at the click (design change 0030). A Shift-queued order takes each unit where its earlier orders
  leave it (`planned_end`, design change 0029).
  A destination is clamped inside the map, and one the unit can't stand on or reach moves to the nearest point it
  can (`nav::reachable_point`, which never searches; design change 0025).
- **A path out of a doorway** (`paths/serve.rs::doorway_start`, design change 0080): a unit still in the exit
  doorway of the building that released it stands on that building's tiles, which no search starts from. Its
  request starts at the first of the doorway's ways out where its box is clear on the grid it plans on
  (`Doorway::ways_out`: straight out, then in front of each tile of the exit side), and the unit heads for that
  start first (`Mover::next` 0). **(M3-D2)** Out of a portal's energy doorway the start is where the unit's walk
  along its heading comes clear of the footprint (`EnergyDoorway::outside`), if its box is clear there: the walk
  takes it there (see *Movement and collision > The exit doorway*). With no way out its owner knows of, the request
  fails like any from a blocked tile (see *Production*).
- **Avoid tiles** (`Mover::avoid_tiles`): a repath of a stuck unit treats them as blocked, and its smoothed path and
  its steering stay out of them (`nav::smooth_avoiding`, `nav::segment_avoids`). If they cut every route, the unit
  drops them and plans again.
- **Unit boxes** never overlap blocked tiles or another unit of their layer at spawn: the gate checks debug spawns and
  teleports against the sim's grids; scenario validation and the editor check pre-placed and layout units with
  `paths::misplaced` (terrain-only grids of the classes present, plus the footprints).
- `f32::mul_add` is a disallowed method (`docs/movement.md > 7`): write `a * b + c`.
- Timings: `cargo test -p sc_sim --release --test perf nav_perf -- --ignored --nocapture --test-threads=1`, like
  every `*_perf` test, all in the test program `perf` (each reads the clock in its one timing helper, see
  *Determinism*). Measure on a quiet machine:
  note the load average (`uptime`) with the numbers.

## Movement and collision

The design is `docs/movement.md > 6` (binding; its **(D)** marks are where slice D changed it, **(M)** where the
magic box did, **(Y)** where friendly yielding and the answers to Q0008 did, **(Q)** where queued group orders did, **(G)** where
the hardening of crowds and nav performance did, **(H2)** where gathering Drones passing through each other did,
**(M3-G)** where DragMovement and idle units moving apart did, **(M3-G2)** where a queueing unit going round
the row in its way did, **(M3-G3)** where one with no way round giving up did),
and the code is `crates/sim/src/movement.rs` and `movement/`, with the unit's state in `mover.rs`.

- **One pipeline, sequential commit** (`movement::step`, phase 5d): the step builds one spatial grid per collision
  layer (ground, underground; air units collide with nothing) from every unit, then moves the units in id order and
  updates the grid at each commit. A unit sees earlier units where they now are and later ones where they were.
  The grids are scratch, rebuilt every tick and never hashed.
- **No overlap by construction**: a position is committed only if `World::is_free` holds, the same strict predicates
  as invariants I1 and I2 (touching is free), with the exceptions of `space::blocks` below and no other. Contact positions are assigned from the blocker's edge
  (`space::resting`), never accumulated. Never write `pos` anywhere else, and never loosen the predicate with an
  epsilon.
- **Lane seeking** (`World::lane`, `World::lane_step`; `docs/movement.md > 6.3`): before anything else, a unit
  looks 4 of its lengths ahead for the units it has to pass: oncoming ones (by `Body::heading`, where they are
  going) and idle units that yield to it. Each bars a range of sideways shifts, its lane. The unit steps, forward
  and sideways at once, towards the nearest clear lane within 3 of its widths, or, in front of a dense group,
  towards the middle of the widest gap; two dense groups mesh like combs and spread from the outside in. One idle
  unit alone is no lane. The code and its constants, the `LANE_*`, are in `collide/lanes.rs`. It is deterministic
  by sorted lanes and fixed cell order, and keeps no state but `Mover::avoid_side`. Never let it write a position
  except through `World::clear`.
- **The grid's cells carry marks** (`Body::kinds`, `Layer::near_any`): which heading sectors and whether a yielding
  unit were listed in a cell this tick. They only ever say "maybe", and exist so that a unit with nobody to pass
  doesn't look at bodies. Keep them conservative: mark on every insert and whenever a body's kinds change.
- **One unit's tick** (`step_unit`, `collide.rs`): turn towards the steering waypoint (at most TurnRate), then move
  one step: a lane step (above), else free; **slide** along tiles (per axis); **follow** a unit moving the same way; **go around** a standing
  or oncoming one (`collide/contact.rs`: a glance along its face, else **the contact step** of design change 0323:
  straight sideways along the blocker's face by the unit's own width, taken over several ticks as `Mover::step_aside`,
  hashed only while set, during which the unit makes no other move; it commits its side for a second; at a standing
  blocker the side is the near one, at a moving one the side the unit leans to, else its Right first, so head-on
  units pass like traffic; a crossing unit is passed behind); else close up or stay. **Which contact step a base
  takes is data**: `UnitBaseDef::contact_step` (`ContactStep::Sideways`, or `Diagonal`, the 45° step as built before
  0323, kept for the WheeledVehicle until its turn and graze is built). Never match on the base in movement. Within one step of the
  goal the unit lands exactly on it. A unit without a goal moves only by its own sidestep (below), by braking off
  its momentum, or off a unit it overlaps (*Movement models*, *Idle units that overlap*): nothing pushes.
- **Friendly yielding** (`yielding.rs`, `docs/movement.md > 6.6`; design changes 0022, 0023, 0027): a moving unit
  asks the idle unit of its owner that blocked its way to step aside. Requests are collected during the step and
  applied after it in the order made (the lowest id that asked wins); the unit then takes one short sidestep
  (`Mover::sidestep`, hashed state) in its own turn from the next tick on, through `World::is_free` like every
  move. `yielding::can_yield` is the one place that says who yields (an owner, no order at all, no goal, not in an
  attack sequence, `Mover::can_move`: the hook for units that can't move in their mode), and `yields_to` to whom
  (its owner's units, but not the group order it arrived with). A blocked sidestep passes the request on, at most
  `YIELD_CHAIN_DEPTH` deep. Moving units never yield or swap. A lone unit squarely in the way steps at 45° to the
  mover's way; a unit that is only clipped, or stands in the way of a mover walking between the units of a group,
  steps straight across it, and such a mover asks the units on both sides of its gap two steps before it reaches
  them (`yielding::ask_ahead`): the group parts and isn't shoved along. **A pop-up turret that wants to rise asks the
  same way** (slice M4-S2, `yielding::make_way`; design change 0296 point 3: "The owner's own idle units over it are
  pushed off by the idle push"): after the units have moved, each lowered turret kept down (`popup::is_blocked`) asks
  the units over its footprint that yield to its owner for a sidestep off it, along the axis on which they are out
  soonest, to where their box touches its edge; the movers' requests come first, and it asks again every tick it is
  kept down. No other unit is ever moved for it.
- **Collision exceptions** between units go in `space::blocks(mover, other)` and nowhere else (crushing will). The
  sweeps, lane seeking and `World::is_free` all ask it, so a unit ignored for a move is ignored for the whole of it.
  It is asked last, after the box test (few units near a move are in its way). Two exceptions (design change 0106,
  slice M3-H2), both read off the bodies each tick, with **no pair state**:
  - **Gathering Drones** (`systems/units.md > UnitCollision > Ground Collision > Exception, gathering Drones`):
    two bodies with `Body::gathers` (`orders::gather::on_gather_job`: a Drone whose current order is a Gather or a
    Drop Off, at every step of the job) don't block each other, whoever owns them. A gathering Drone and any other
    unit block each other as ever, an idle Drone included.
  - **Units that already overlap** (`systems/units.md > Units that already overlap`): two bodies whose boxes overlap as
    the mover's turn begins don't block each other. `space::overlaps` is I1's own predicate, so the pair is ignored
    exactly while I1 would name it, and normal collision is back the moment the boxes are apart (touching is apart).
    Nothing pushes them apart, and either may move any way meanwhile; an idle one moves off by itself (design
    change 0126, below). Only the first exception and an evacuation ever bring two boxes to overlap (design change
    0349, slice M5-F3: everyone inside a destroyed building appears inside its former footprint at once), so in
    today's game such a pair is two Drones of which one left its job, or two evacuees; the rule itself asks nothing
    about the units' kinds.
  A later exception (crushing) is one more clause there. A slice that adds a way for two boxes to come to overlap
  must add it to `testing::invariants::may_overlap` too, or I1 fails.
- **The exit doorway** (`core/entities.md > Structure Type > Exit Doorway`, design change 0080; a portal's energy
  doorway, design change 0145; `production/exit.rs`, `production::ExitDoor`) is the one exception against tiles,
  and the one case in which a unit's box overlaps a structure. A unit a producer released (or a building that holds
  units let out, `Sim::release_held`, slice M5-F1) has `Mover::exit`, the building's id, while its box still
  overlaps that footprint. For that unit alone, `World::door_free` replaces the tile tests of `tiles_free`,
  `is_free_for` and `along_axis`: the building's own tiles don't block it, but for ground under them that the unit
  can't stand on (slice M5-Z1, `Frame::blocks`, `NavGrid::is_terrain_solid`: the building lets the unit overlap the
  building, not that ground, so that the unit never stands there once the building is gone; a debug spawn or a map's
  object may cover any terrain, and a custom preset may be Buildable and not Traversible), every other blocked tile
  does (`ExitDoor::box_clear`), and it may only come out (`ExitDoor::allows`: out of a side doorway, never deeper in,
  and inside only within the exit side's span, so it leaves through that side and no other; out of an energy
  doorway, never closer to the footprint's centre). A door whose every spot lies over such ground has none
  (`Sim::doorway_spot`), and its building waits as behind an occupied doorway (fuzz seed 155: a Hall spawned over
  Cliffs had left its Recruit on one). Units still block it and it blocks them, as ever.
  Phase 5d clears `exit` (and `Mover::exit_heading` with it) in the tick the box is clear of the footprint (or the
  building is gone, or a teleport took the unit away), and nothing ever sets it again: a unit that is out can't
  re-enter.
  - **Out of a side doorway** (the Barracks'), with a goal the unit moves by the ordinary rules (lane seeking,
    going around, yielding requests); without one, `doorway::leave` walks it straight out and stops it against the
    exit side (design change 0086), trying again every tick while something is in the way.
  - **Out of a portal's energy doorway** (slice M3-D2, `doorway::emerge`), goal or none, the unit moves **straight
    along `Mover::exit_heading`** (chosen at its release, see *Production*) and nothing else: no lane, no way round,
    no sidestep (`yielding::can_yield` leaves it out), so it never leaves the ray, and the ray leaves through an
    open side. It moves by its own model (one that stops within the tick turns to its heading first, then steps;
    the Drone thrusts along the ray, `Drag::along`), through `World::straight`: as far as tiles allow (the step
    halved up to 8 times) and up to the first unit. Without a path it lands exactly where it comes clear
    (`EnergyDoorway::outside`), a hair short counting (`CLEAR_SU`, 1/64 su), and stands there; with one it keeps
    its speed out of the footprint, and the next step is `follow`'s. It asks its owner's idle units in the way to
    step aside and tries again every tick; with a path its stuck clock runs, so a move that can't begin fails, and
    the unit stays where it got to, "half out at the edge of the footprint, as a normal unit" (0145 point 4).
  **Never set `Mover::exit` anywhere but where a producer releases a unit (`Sim::release_unit`) or a building that
  holds units lets one out (`Sim::release_held`), and never test a doorway unit's tiles anywhere but through
  `World::door_free`.**
- **Held by its attack**: a unit whose attack sequence holds its base (`combat::may_move` or `may_turn` false: see
  *Combat*) stands for the tick and keeps its goal, path and stuck clock (`wait`), like a unit waiting for its path.
- **Stuck and recovery** (`stuck.rs`; the numbers, in `mover/schedule.rs`, are the design's placeholders of changes
  0024 and 0026): progress is the remaining path length; waiting behind a unit that is getting on counts. A unit is
  stuck after `STUCK_TICKS` (24) without it (`Mover::is_stuck`); it then tries another route `MAX_REPATHS` times,
  after waits of `REPATH_WAITS` (8, 16, 32; `Mover::recovery_due`), around the tile of a blocker that stands, and
  gives up 24 ticks after the last: `MoveStatus::Failed` and `SimEvent::MoveFailed`, `GIVE_UP_TICKS` (104) after its
  last progress. **Occupied target**: within `OCCUPIED_ARRIVAL_GU` (3, design change 0169; 2 under 0026) of its goal
  behind a stationary unit that won't yield to it, the unit has arrived instead, without a repath: when stuck, or
  after `JAMMED_TICKS` without a way forward if the blocker has no goal. That covers a unit walled out of its
  magic-box spot by its own group's arrivals (design change 0183): arrived within the radius, given up beyond it.
  Two moving units head-on in a passage too narrow to pass both fail. **The crowd sweep** of slice M1-G3 is kept as
  the ignored `movement_crowds.rs::crowd_sweep` (two blocks crossing head-on, 288 layouts, run in release): for
  0169 it went from 43 to 7 failed units of 5 856 for each owner count (Peacekeepers 8 → 0, Guards 20 → 4,
  Peacekeepers against Guards 15 → 3).
- **Whose progress a waiting unit shares** (`stuck::leads`, slice M3-H2): a unit blocked by one that is on its way
  takes over that unit's `last_progress`, but only if that unit **has got on since its leg began**. `Mover::go_to`
  sets a unit's own clock to the tick of the order, and that is no progress: before this rule, a blocked unit whose
  legs began again and again (a behaviour trying again, a player clicking on it) renewed the clock of every unit
  behind it, and none of them ever looked for a way round (`tests/movement_stuck.rs`). It is read off the mover
  (`repaths_left < MAX_REPATHS || last_progress > last_repath`), with no field of its own. A behaviour may begin a
  leg again as often as it likes; it need not stand in between for the sake of the units behind.
- **Object goals** (`MoveGoal::Object { id, reach }`, `docs/movement.md > 5.6`, `docs/movement.md > 6.5`): the path is
  planned to the point nearest the object's centre that the unit can stand on (right at a structure's wall), and kept
  aimed at the object as it moves (`paths/checks.rs::chase_check`, on the stagger tick: straight at it while the unit's
  box has a clear line, else a new plan once it moved 1 GU or 25 % of the distance, at most every 8 ticks, never counted
  against the repath cap). The unit has `Arrived` when the gap between the two shapes is within `reach`, at the end
  of its path if the object hasn't moved, or by the occupied-target rule (its 3 GU measured as that gap). Progress
  towards a moving object is moving forward at all; towards one at rest it is measured as to a point. A unit
  chasing its attack's target behind stationary attackers of its owner on the same target **queues**
  (`stuck::queues_behind`): no path request; it fails as any stuck unit does once its attempts are used up (slice
  M3-G3, design change 0223; until then it never failed).
- **A queueing unit goes round the row** (slice M3-G2, `movement/round.rs`, `docs/movement.md > 6.5` rule 2;
  `systems/units.md > UnitCollision > Ground Collision`: "Moving units must pathfind around occupied space").
  The row is the unit in its way and the units standing for good next to it, nearer to each other than the unit
  is wide. The detour leads straight out of the row's box (grown by the unit's half extents and
  `ROW_CLEARANCE_SU`), round it from corner to corner, to an end of the side that faces the target: the shortest
  way whose legs are clear of blocked tiles and of the row's units. `Mover::go_round` puts its waypoints in front
  of the path, and `Mover::detour_end` (hashed and serialized only while set) keeps the staggered check from
  skipping them. An attempt counts against the leg's `MAX_REPATHS` on the repath schedule, or comes after
  `JAMMED_TICKS` behind a unit with no goal; it asks for no path. One after a lost race (the unit now in the way
  moved after this unit's last attempt, `Body::moved`) is free. **With no way round, or no attempt left, the
  Failure rule applies** (slice M3-G3, design change 0223: "it does not wait behind the blockers indefinitely"):
  the unit gives up still stuck `STUCK_TICKS` after its last attempt, and its behaviour gives the target up. M3-G2
  had it wait and stand there (`stuck::waits_in_queue`, `World::hold`, both gone). Measured
  (`tests/attack_behind_friends/`): four Peacekeepers in a column all fire on a building within 135 ticks of the
  order in all 16 layouts, where one of the four never fired in 5; eight within 255; in a corridor the ones behind
  give up within `GIVE_UP_TICKS` of their last progress, with the 8 ticks' grace (`no_room.rs`). **Never give a
  queueing unit a path request for the row**: no grid shows it. A melee unit behind its owner's attackers is
  within `OCCUPIED_ARRIVAL_GU` of its target, so the occupied-target rule comes first and its approach ends there:
  it gives the target up (design change 0223 point 1, as built).
- **Movement models** (`movement/follow.rs`, `movement/drag.rs`; `docs/movement.md > 6.7`): path following, lane
  seeking, the rules after the sweep, yielding and the stuck clock are shared (`follow`), and a model replaces only
  the "direction → facing and velocity" part. Two are built: the units that stop within the tick (every GDO unit
  and the Legionnaires: the step the rules give is their move) and **DragMovement** (the Drone, slice M3-G, plan
  decision T25).
  - **The integration scheme is fixed**: turn (at most TurnRate, towards the step the unit wants; it never stands
    to turn), drag (`v ← v × (1 − DragRatio)`, the ratio doubled while the tile under the unit's centre is Rugged:
    design change 0039), thrust (`v ← v + a`: OmniDirectionalAcceleration any way plus up to ForwardAcceleration
    along the facing, whichever brings `v` nearest to what the unit wants), position (`pos ← pos + v` as far as
    tiles and units allow). Drag is taken on the velocity the tick began with, so the equilibrium is the design's
    MaxSpeed `(Omni + Forward) / DragRatio`: 8 su a tick for the Drone, 4 on Rugged.
  - **What it wants** is the step the shared rules give a unit without momentum, at the **approach speed**
    (`drag::approach_speed`): the highest speed from which braking with the omni-directional thrust still stops
    it within the path that is left. So braking begins when the stopping distance reaches the remaining path
    (35 su before the goal from top speed with the Drone's numbers of design change 0205, 51 before them), and the
    last step lands exactly on the goal. No overshoot.
  - **Blocked means from rest** (`systems/units.md > UnitCollision > Collision Response (Un-sticking)`, design
    change 0035): a move that runs into a tile or a unit goes as far as it can, and the unit's **whole velocity is
    then 0**, not only the part into the obstacle. Where the rules' step is a slide along tiles (`Moved::slid`),
    the unit thrusts at the step it asked for, into the tile, and meets it every tick: it creeps along the wall
    at single-tick acceleration (1.2 su for the Drone) until its way is free, then accelerates normally. That
    creep is progress for the stuck clock wherever it gets closer (`movement/follow/tests.rs`). A wall it only
    touches, a unit it passes through (`space::blocks`) and a unit it moves off (0126) are no obstacles. A debug
    teleport zeroes the velocity too.
  - **`Mover::momentum`** is the velocity carried into the next tick: the displacement the unit made, or zero if
    it was blocked. It is hashed and
    serialized **only while it isn't zero**, so states of games without such a unit hash and load as before the
    field existed. `Mover::stop` and `finish` leave it: a unit without a goal that still has some brakes to a
    stop, `Locomotion::Stopping` (within 26 su and 9 ticks from top speed; 42 su and 16 ticks before design change
    0205). **Never zero it to stop a unit**, and
    never read `velocity` for it: `velocity` is the tick's displacement and `stop` clears it.
  - **Going round a standing unit**: a unit with momentum takes the side of the blocker's middle it is on
    (`World::around`, `near_side`), whatever its lean, and a step aside doesn't count as blocked while the unit
    moves (`Moved::aside`): the manoeuvre takes it a dozen ticks, and turning back costs it all its speed.
  - **Kinematic for every model**: a yielding unit's sidestep, the walk out of a side doorway and an idle
    unit's step off a unit it overlaps are plain steps without momentum. A corner is taken at speed: the unit
    swings wide, or meets the wall beyond and starts again from rest. **(M3-D2)** The walk out of a portal's energy
    doorway is the model's own (`Drag::along`: thrust along the heading, the speed carried from tick to tick, and
    without a path braking to land where the unit comes clear).
  - The other three models aren't built: their units would move like the first kind.
- **Idle units that overlap move apart** (`movement/separate.rs`; design change 0126, which amends 0106's "nothing
  is pushed apart"): in its turn, a unit that yields (`yielding::can_yield`: no command at all) and is at rest
  takes a step off the lowest-id unit its box overlaps, along the axis it is out soonest, at
  `SEPARATE_SPEED_SU` (2 su a tick, a placeholder), no farther than to where the boxes touch. Where two centres
  coincide the lower id goes west or north. Units with a command are never moved (Hold, a Gather, any order).
  The step goes through `World::shuffle`, so it never enters a unit it doesn't already overlap; `n` units stacked
  on one point are apart within `18 × (n − 1)` ticks for Drones.
- **Counters**: `Sim::move_counters()` (sweeps, box tests and lane tests of the latest tick) and `Sim::nav_counters()` are
  output, not state. Tests assert them; `cargo test -p sc_sim --release --test perf movement_perf -- --ignored
  --nocapture` prints timings.
- **Invariants** (`docs/movement.md > 8`, `sc_sim::testing::invariants`): I1 no two units of a layer overlap, except
  (design change 0106, `invariants::may_overlap`) two Drones on a Gather job, and two units that already overlapped
  a tick earlier, until they have been apart (a Drone counts as on its job for the tick in which the job ended
  without another order: a patch emptied in the economy phase ends it after the movement), and (design change 0349,
  slice M5-F3) a unit evacuated in the tick with any other (`SimEvent::Evacuated`: it wasn't on the map a tick
  earlier). Any other overlap fails,
  the first tick it exists (`tests/invariants_overlap.rs` puts units where they must not be and asks the checker); I2 no
  ground box overlaps a blocked tile or leaves the map, except a unit in an exit doorway (`Mover::exit`), which
  overlaps the tiles of the building that released it and no other blocked tile, nor ground under them it can't
  stand on (slice M5-Z1), overlaps that footprint for as
  long as `exit` is set, never moves deeper in (`ExitDoor::allows`), never stands off a portal's open arc
  (**M3-D2**: `EnergyDoorway::margin` at least −1e-3, `ARC_TOLERANCE`), and never comes to be in a doorway by
  moving (`tests/production/portal_stalls.rs` has the energy doorway's negative tests for I1 and I2); I3 a unit moves at most its speed per tick; I4 every float
  is finite (I3's speed is the model's maximum for a unit with momentum: a mode that slows it only weakens its thrust); I5 a unit without a goal doesn't move; I6 repaths per leg are capped and no path request waits longer
  than `MAX_REQUEST_AGE`; I7 an arrived unit never moves again without an order or a move of its behaviour, and a
  unit without a goal is at rest. A leg is one `Mover::go_to`: an order's move, or one a behaviour starts (a chase,
  a Patrol's next leg, the way back to an IdleOrigin). The exceptions to I5 and I7 are a unit taking its sidestep,
  which must have neither a goal nor an order, a unit walking out of an exit doorway, **(M3-G)** a DragMovement
  unit braking off the momentum it had (it must be `Stopping`, its velocity its momentum, and slower than a tick
  before; no other model may ever have momentum), and an idle unit that overlapped another a tick earlier, or was
  evacuated in the tick, moving off it (design changes 0126 and 0349). K1 (`paths`): each player's known grids are
  those a fresh build
  from the state gives. P1 (`projectiles`, slice M4-O, plan decision T42): every projectile in flight lands after the
  tick just played, between finite points. O1 (`operations`, slice M4-F): a Tunnel's operation and its site name each
  other (`TunnelState::operation`, `Builder::Tunnel`); an upgrading Tunnel is in operation, a tier below its target,
  over its target tier's frames (slice M4-G). A `TestSim` checks them after every tick and panics with a picture of
  the map (`TestSim::ascii`: units as letters, with a legend). The check is on by default; `check_invariants(false)`
  turns it off for timing runs. A test that steps a bare `Sim` keeps its own `Invariants` and calls `check` after each
  step.

## Economy

The players' banks, power and army caps, the plates' mining, construction, and defeat and the end of the game
(`economy.rs`, `economy/`, `construction.rs`; slices M2-B, M2-D1, M2-F2 and M3-C; `factions/gdo/overview.md >
GDOResources`, `factions/gdo/economy.md`, `factions/colonists/overview.md > ColonistResources`). Production has a
section of its own.

- **Banks** are the only resources stored: `Player::crystals` and `Player::supplies`, and the Colonists' refined
  banks `Player::alloys`, `essence` and `flux` (slice M3-C, plan decision T21; nothing earns or costs them yet). All
  are `u32`, hashed, and `#[serde(default)]` so that older states load. A game starts them at the player's faction's
  Starting Conditions (`Player::starting`, `StartingConditions`, slice M2-F1; see *Sim scenarios and tests*): a GDO or
  Colonists player's at 100 Space Crystals and the rest at 0, a Syndicate player's at 100 Space Crystals and 1 Supply
  (slice M4-B), a Cults player's all at 0 (slice M5-B).
  `DebugCommand::SetResources` sets all five (`Player::set_banks`; the refined ones `serde(default)` in command files
  too), and `Player::banks` reads them as one `Banks`. Pay with
  `Player::spend_crystals`, which refuses when the bank is short and leaves it as it was; credit with
  `Player::credit_crystals`, which saturates (only a debug command fills a bank that far, and input never panics the
  sim). Every refund goes through `economy::Refund` (`Full`, or `Ready`: 75 %
  rounded down). Supplies pay for an Agent's Tunnel (slice M4-E, *Agent-built Tunnels* below: `Player::spend_supplies`,
  `credit_supplies`); nothing else costs them yet.
- **Power and the army caps are derived** from the objects whenever they are asked for, never stored, so they can't
  drift (plan decision T7). `Sim::power(player)` is a `PowerSummary`: `generated` and `consumed`, the HUD's
  used / available (design change 0090). `Sim::army_cap(player)` is an `ArmyCap { resource, used, available }` for
  the player's faction (`economy/army.rs`, plan decision T20; `None` only for a player not in the game): the GDO's
  Unit Control, out of `UNIT_CONTROL_CAP`; the Colonists' Beacon Capacity, out of
  min(`BEACON_CAPACITY_CAP`, `BEACON_CAPACITY_PER_BUILDING` × their Colonists structures, a Beacon under deployment
  left out until it completes: design change 0112, *Beacon Deployment*); and the Syndicate's **Tunnel Space** (slice
  M4-B, `economy/army/tunnel_space.rs`; `factions/syndicate/tunnel_area_and_space.md > Tunnel Space`, design changes
  0242, 0258 and 0265), out of min(`TUNNEL_SPACE_CAP` 200, ⌊the unused tunnel-region cells of the player's Tunnels /
  `CELLS_PER_TUNNEL_SPACE` 4⌋): each Tunnel's region is its Tunnel Area at its current tier less its own footprint
  (`tunnel::area::region`: 48, 84 and 128 cells), every cell counted, on the map or off it, Drillable or not (0242);
  a cell is used when an underground structure of the player's or a neutral one stands on it, cell by cell (0258
  point 4), never another player's. The regions are summed Tunnel by Tunnel, as the formula reads. And the Cults'
  **Unit Control** (slice M5-C, plan decisions T56 and T57; `economy/army/cults.rs`, `recruitment.rs`;
  `factions/cults/structures/recruitment_hall.md > Cults Unit Control HUD Display`): available is the sum of their
  completed Recruitment Halls' capacities, each rounded down (design change 0341 point 2: "Capacity is rounded down
  everywhere", so two Halls of 6.6 give 12); used is their Halls' usage, the entries of every unit's lineage that
  name them and each Hall's Recruit in production (design change 0342 point 3), plus their orphaned points, the
  entries whose Hall is gone and the points of a Cults unit's cost that no entry stands for (design change 0344:
  "Recruits no Hall produced (custom maps, debug) are orphans from the start"), the units held off the map counted
  too. **A unit costs only in its own faction's
  resource** (`ArmyResource::cost_of`; the Director's decision, slice M5-C): another faction's unit, which only a
  debug or a Custom game gives a player, costs 0 in it, both ways (`economy/army.rs`'s
  `another_factions_unit_costs_nothing_in_either_direction`), so the Recruit's and the Peacekeeper's shared
  `unit_control_cost` field never mixes the GDO's and the Cults' Unit Control. Used is the `ArmyResource::cost_of` of
  the player's units plus, for a resource that `reserves`, their producers' units in production (design change 0143,
  for the GDO, the Colonists and the Syndicate: `Production::in_production`, the unit at the front once its progress
  has begun, a finished one waiting inside included; a unit only queued, or waiting at 0% for room, takes nothing).
  The Cults' Unit Control reserves nothing (`ArmyResource::reserves`): a Hall counts its own Recruit in production,
  and a unit trained of Recruits embodies their points (*Production > Pool buildings*). The cap is checked when a
  unit's production would start, never when it is queued (*Production*), but for the Cults', which is for display
  only and makes nothing wait (design change 0344: "Global total is for display only"). Capacity that falls below
  what is used leaves the units and the unit in production alone, and no new unit starts until there is room (design
  changes 0112 and 0143, Q0041 point 2). `Sim::unit_control(player)` is the GDO's used figure alone, which the HUD
  reads. A
  building counts from the tick it is placed until the tick it leaves the game, that one excluded (design change
  0089), because it is among the objects in between. Each call walks the objects once.
- **Speed in fixed point** (plan decision T6): `RATIO_ONE` (2^16) is 100 %. `PowerSummary::ratio()` is
  `min(RATIO_ONE, ⌈generated × 2^16 / consumed⌉)`, and `RATIO_ONE` without consumers. `PowerSummary::speed(kind)` is
  what a structure of that kind advances in a tick: the ratio for a consumer (negative Power), `RATIO_ONE` for any
  other ("power producers are never slowed", design change 0089). A task of `n` frames at full speed is done when its
  progress reaches `n × RATIO_ONE`, and the rest carries over. Rounding the ratio up gives an exact ratio its exact
  tick count: at 30/51 the k-th 48-frame cycle ends in tick ⌈81.6 k⌉.
- **Recipe timing** (the Director's convention, for every recipe: construction, production and any to come):
  progress accrues in the tick in which the command applies, since the commands apply before the economy phase of
  their tick (plan decision T5). A recipe of `N` frames ordered in tick `t` at full speed completes in tick
  `t + N − 1`, and what it makes can be used from tick `t + N`: a ready building placed, a produced unit acting (it
  comes into the game in tick `t + N − 1`, and the spawn-tick rule has it act from the next). So a Generator (240
  frames) ordered in tick `t` is ready at the end of tick `t + 239` and can be placed in `t + 240`, and a Peacekeeper
  (80 frames) trained in tick `t` is released in tick `t + 79`. A constructor or producer that comes into the game
  in tick `t` starts on its recipes in `t + 1` (spawn-tick rule).
- **Mining** (`economy/mining.rs`, `factions/gdo/economy.md > ExtractionPlate`): a structure with a `MiningDef`
  (`StructureDef::mining`: the plate) that stands on a patch has an `Extractor { patch, cycle }`, linked in
  `Sim::insert_object` once both are in the game, whichever comes first; a plate on bare ground (a debug spawn's) or
  without an owner mines nothing. Each tick the cycle advances by the owner's ratio. When it reaches
  `cycle_ticks × RATIO_ONE` (48 frames), the plate pays its owner a whole load: 10 while the patch holds 10, then what
  is left, then the residual 1 forever (design changes 0098, 0099; `SimEvent::PatchDepleted` when the patch reaches
  0, and it stays under the plate). Spawn-tick rule: a plate placed in tick `t` pays first in tick `t + 48`.
- **Gathering** (`economy/gathering.rs`, `orders/gather.rs`, `sim/workers.rs`; slices M3-H and M3-H3, plan
  decision T17; `factions/colonists/units/drone.md > Gathering`, `factions/colonists/economy.md > Egress Node >
  Drop Off`): a unit type with a `WorkerDef` (`UnitDef::worker`: the Drone's CarryCapacity 8, MiningDuration 32,
  DropOffDuration 2, the two reaches of 8 su and its search radius) is a worker, and a structure type with
  `StructureDef::drop_off` (the Egress Node, the Tunnel) takes its loads. `Object::worker` (`Worker`) holds only what outlives
  an order: the load and the AssignedDropOff (the patch a Gather goes back to is the one its order names, design
  change 0132). The job is an order and its behaviour (see *Behaviours > Gathering*), and
  **the timed work is part of the order's state** (`GatherPhase::Mining { done }`, `DroppingOff { done }`), so an
  order that ends takes its unfinished work along and frees the patch at once. `gathering::step` counts the
  frames in the economy phase, by *Recipe timing*: a worker that reaches its patch in tick `t` holds its load
  (`min(8, remaining)`, taken from the patch) at the end of tick `t + 31` and the patch is free in `t + 32`; one
  that reaches its drop-off in tick `t` is paid at the end of `t + 1` (`SimEvent::ResourcesDelivered`). A patch
  emptied this way leaves the map in the same tick, unless a plate stands on it; workers never gather a plated
  patch. `Sim::returning_drones(node)` is the Egress Node's F.
  **Worker types and drop-offs are data** (slice M4-D, plan decision T34): a `WorkerDef` has a `LoadDef` per
  resource (`crystals`, and `supplies`, the Agent's since slice M4-I: 1 a trip, 48 frames to pick up and 48 to unload,
  reaches of 8 su, `WorkerDef::load(Resource)`), `shares_patches`, `drop_off` (`DropOffChoice`: `Assigned`, the
  Drone; `FirstReturn`, the Agent; `EachReturn`, the Recruit, slice M5-D), `search_radius`
  and `pass_through`; `StructureDef::drop_off` is `None`, `DropOff::Anywhere` (the Egress Node, the Storage) or
  `DropOff::Hatches { crystals, supplies }` (the Tunnel: Sides B and D, design change 0248; `DropOff::hatch`).
  `Worker::resource` (`defs::Resource`, Crystals by default) is what the load is, kept in saves and hashed only while
  it is Supplies; a delivery credits the bank of its resource, and `SimEvent::ResourcesDelivered` says which.
  `economy/stations.rs` is the station's side of a pick-up: the Supply taken as the pick-up ends, and the
  notification of an empty station (slice M4-I, see *Behaviours > Gathering*).
  `economy/drop_off.rs` turns that into a `DropPoint` for a resource (`drop_point_of(object, resource)`: shape,
  centre, side) and has **the hatch square**
  (`hatch_offset`, `hatch_tile`, `hatch_square(tunnel, Resource)`, also `tunnel::hatch_square`): the middle tile of
  the side, of two middle tiles the one farther from Side A, read off `sides()` so that it turns and flips with the
  structure; the client draws the hatches there. The Agent: 50 a load, 64 frames of mining (design change 0294; 80
  before), 64 of unloading, reaches of 8 su; alone it delivers every 128, 134, 156, 176, 198, 220 and 240 ticks at 0
  to 6 GU from hatch to patch, edge to edge (the design's table: 8.0 to 16.0 s; all within 15%).
  **Measured with DragMovement, the rules of slice M3-H3 and the Drone's numbers of design change 0205**
  (`tests/gathering/`, `tests/gathering_soak.rs`; nothing is tuned to the design's table): one Drone with its
  patch 2 GU from the node, edge to edge, delivers every 73 ticks, 1.75 Space Crystals per second (81 ticks and
  1.58 before 0205; 60 ticks on the stand-in mover it had before DragMovement). On the reference field (8 patches
  in a row, the node 2 GU south of the middle), with gathering Drones passing through each other (design change
  0106), nothing jams: 4, 8, 12, 16, 20 and 24 Drones dealt out over the patches give 6.9, 12.7, 18.8, 25.3, 26.1
  and 31.4 per second (before 0205: 6.3, 11.7, 17.2, 23.1, 24.5, 26.2; before M3-H3: 17.9 for 12, 25.6 for 20,
  27.3 for 24; the stand-in: 8.3, 15.1, 22.0, 26.8, 32.0, 32.0; the design since design change 0236: 6.9 for 4,
  about 25 for 16, about 26 for 20, 32 for 24). Three findings for the economy oracle (slices K1 and K3): 24
  Drones dealt out nearly saturate 8
  patches (before 0205 they didn't: the end patches' trips were longer than three Drones' mining covered); a
  Drone sent on from an occupied patch to the nearest free one often finds that taken too when it gets there
  (dealt out, the longest one of 24 went between two deliveries is 393 ticks, within the plan's 480, 30 s; 357 to
  578 across earlier builds); and the waiting Drones' re-check (design change 0132) costs up to 0.6 per second
  below 16 Drones and up to 1.3 over 16 (measured in slice M3-H3): the patch a waiting Drone leaves for is
  usually one whose own Drone is on its way back to it. The 24-Drone soak costs 25.4 µs a tick in release (24.4
  before slice M3-H3's rules and 13 on the stand-in, each measured side by side: a Drone's tick is the rules' step
  and then its own); with the numbers of 0205, 25.8 against 27.0 before them, side by side (slice M3-K3). Under
  hard collision, as slice M3-H first had it, the field gridlocked from 12 Drones on.
- **Removal** (`Sim::remove_objects`): a plate takes its patch along if that is depleted, in the same tick, which
  opens the tile (design change 0038; the caller's grid sync covers it); a patch with crystals left stays, uncovered.
  A plate whose patch is removed (only a debug command does that) loses its extractor. `testing::referenced_ids`
  lists the extractor's patch, so `dangling_references` checks it.
- **Construction** (`construction.rs`, `sim/construction.rs`; slice M2-D1;
  `factions/gdo/structures/deployment_center.md`, `factions/gdo/economy.md > ExtractionFacility`): a structure type
  with a `Constructs` list (`StructureDef::constructs`: each `Recipe` a building, its cost and its frames) is a
  constructor, and its object has a `Constructor`: `Idle`, `Constructing { kind, progress }` or `Ready { kind }`, one
  building at a time and one ready at most (design change 0071). `Command::Construction` drives it (see *Commands,
  events and replays*): Construct pays the cost as it applies, the cancels refund through `Refund` (`Full` while
  constructing, `Ready` once ready), and so does a constructor that leaves the game (design change 0084, in
  `Sim::remove_objects`, for a destruction and a debug removal alike), each refund with a
  `SimEvent::ConstructionCancelled` (the constructor, the building and the refund) for the interface's feedback. Each
  tick `construction::step` advances a construction by its constructor's `PowerSummary::speed`: the Deployment Center, a
  producer, at full speed whatever the power, the Extraction Facility at the ratio (design change 0089); at
  `frames × RATIO_ONE` it is ready (`SimEvent::ConstructionCompleted`), by *Recipe timing*: a Generator ordered in tick
  `t` is ready at the end of tick `t + 239`, a plate at 30/51 at the end of `t + 679`, and a Place can apply from the
  next tick on. Place inserts the ready building where `Sim::placement_check` allows it as the command applies (plan
  decision T4): it counts for power and the build area at once, the true grids take it in phase 5b of its tick and its
  owner's planning grids at the end of it, and it works from the next tick on. `construction::recipes(kind)`,
  `Constructor::{current, ready, frames_done}` and `Sim::validate` are what a client needs for its menus. A Construct
  checks the building's prerequisites as it starts (slice M4-K, *Tech prerequisites*). Not built: ConstructionHP.
- **Warp In** (`transform.rs`, `sim/transform.rs`; slice M3-E, plan decisions T15, T16 and T22;
  `factions/colonists/economy.md > Warp In`, design changes 0109 to 0122): a Beacon transforms in place into a
  building of its Constructs List (`StructureDef::warp_in`, `transform::warp_in_recipe`).
  `ConstructionCommand::WarpIn { kind, orientation }` names one Beacon, which pays the cost at once and becomes the
  building through `Sim::change_kind`, on the footprint `transform::warp_in_footprint` derives (see *Placement and
  the build area > Methods*). The object is the building from its first frame, its whole footprint solid (0109),
  and `Object::transform` (`Transform`: what it was and where, its progress of `total` frames, what it paid) runs
  until it is complete. `Sim::step_transforms` advances it by `RATIO_ONE` a tick (the Colonists have no power), by
  *Recipe timing*: ordered in tick `t`, a WarpInTime of `N` frames completes at the end of `t + N − 1`
  (`SimEvent::TransformCompleted`) and the building works from `t + N`. Its maxima interpolate
  (`Object::max_hp`, `Object::max_shields`: "Beacon MaxHP + (Building MaxHP − Beacon MaxHP) × progress"), and its
  HP and Shields rise by what their maxima rise (`transform::follow_max`, 0110: damage stays taken, whenever it was
  taken; the Shields since slice M3-B, which regenerate up to the maximum now).
  Until it is complete it holds its functions back (0116: no production, drop-off, rally or abilities; the gate
  refuses a Train, a rally and a drop-off at it), but it counts for defeat (0113) and keeps providing its 5 Beacon
  Capacity (0112). `ConstructionCommand::CancelTransform` turns every transforming object named back into what it
  was, on its old tiles, with the fraction of its HP it had and at least 1, and the fraction of its Shields with no
  minimum (`transform::keep_fraction`, 0111), and
  refunds `Refund::Ready` of what it paid (`SimEvent::TransformCancelled`); a destroyed one refunds nothing (0115).
- **Beacon Deployment** (`deploy.rs`, `orders/build.rs`, `sim/deploy.rs`, `sim/builds.rs`; slice M3-F, plan decisions
  T15 to T17;
  `factions/colonists/economy.md > Beacon Deployment`, design changes 0109 to 0120 and 0125): a Drone walks to a site
  and transforms in place into a Beacon, by the Warp In's machinery. `OrderKind::Deploy { origin }` (the Beacon's
  2 × 2 footprint, `deploy::site`, never turned; Shift queues it) goes to one Drone; the gate checks its site on what
  the player knows (see *Placement and the build area > Methods*). In phase 5a the Drone heads for the footprint's
  centre (`orders::build`, the walk a Build shares); its walk is over once its move ended, or when it found no way
  forward for `JAMMED_TICKS`
  while overlapping the footprint (0117). Then `Sim::step_deployments` (see *Tick pipeline*) checks the site on the
  truth: if the Drone overlaps the footprint (`deploy::on_site`) and it passes, the Drone becomes the Beacon on the
  footprint, wherever it stood, the same object (0109: its footprint claimed in 5b of that tick), transforming `from`
  the Drone without `from_placement` (`deploy::BEACON_DEPLOYMENT`: 48 frames, no cost), its first frame that tick;
  else the order fails, the Drone idles and its next queued order starts (`SimEvent::DeployFailed`, its problems
  empty if the Drone stopped short). Its HP interpolate from 20 to 120, and its Shields, kept from the Drone, follow
  their maximum (10 to 10), as a Warp In's do; only Cancel Deployment (`CancelTransform`, 0116) applies, and turns it
  back into the Drone at the footprint's centre, with no order and the fractions of its HP and Shields (0111). It
  frees the Drone's 1 Beacon Capacity as it starts and provides the Beacon's 5 only at completion
  (`Object::is_deploying`, `economy::army`; a cancel takes the 1 back even over the cap, 0112). It counts for defeat
  (0113). The Drone's load stays in `Transform::load`, is lost at completion and returns with a cancel (0114).
- **Agent-built Tunnels** (`orders/build.rs`, `sim/builds.rs`, `sim/sites.rs`, `construction/site.rs`; slice M4-E,
  plan decisions T31 and T51; `factions/syndicate/units/agent.md > Building`, design changes 0233 and 0234):
  `OrderKind::Build { kind, origin, orientation }` is the kind-generic worker build (T51). It goes to the one unit
  named that builds `kind` (`WorkerDef::builds`, `construction::site::builds`), which walks to its site as a Drone
  does, and on arrival starts a site there (*Object model > Sites*) or fails. **The price**
  (`factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`; `SiteDef::price`, `SitePrice::SuppliesPerOwned`)
  is `Sim::build_price(player, kind)`: a Supply for each structure of the kind the player has, sites included, and
  for each Build of the kind among their units' orders (the lingering ghosts). It is paid as the order is given
  (`Sim::give_order`, `pay_build`); the gate refuses what the bank can't pay (`Reject::InsufficientSupplies`), counting
  what the orders it replaces give back. **Refunds** are the price once the order has gone, "the current top price",
  "not counting itself" (design change 0284): for an order replaced (`Sim::drop_orders`), gone with its worker
  off the map (`take_off_map`) or out of the game (`remove_objects`; design change 0289 point 1), or failed on
  arrival (`SimEvent::BuildRefunded`), and for a site cancelled (`Sim::cancel_site`, `SimEvent::SiteCancelled`); a
  destroyed site gives nothing back, nor do a defeated player's orders. So **the books** hold: the Supplies spent less
  those refunded equal the formula's total for what the player has, whatever the order of starts and cancels. A site
  completes after `SiteDef::frames` (480), its worker going where `SiteDef::builder_goes` says (into the network);
  cancelled or destroyed, it lets its worker out at the footprint's centre, idle, with its load
  (`Sim::release_builders`, on the removal path).
- **No Agent-built Laser Turrets** (design change 0359, slice M5-Y): an Agent builds Tunnels alone (its
  `WorkerDef::builds`), and the gate refuses its Build of anything else (`Reject::NoWorkers`). Slice M4-S1's turret of
  an Agent's (plan decision T51; design changes 0296 to 0298: its price in Space Crystals at the order, its Agent out
  beside it, no operation slot, its visible site) went with it; a turret is the Tunnel's ExpandMenu's alone (*Object
  model > The pop-up form*). What it brought stays where another builder uses it: the price in the def's resource
  (`construction::site::BuildPrice { resource, amount }`, `SitePrice::Crystals`), a Cults building's, refunded whole by
  an order that goes and 75% of it rounded down by a cancelled site (`BuildPrice::cancelled`, `Refund::Expansion`),
  the gate's `Reject::InsufficientCrystals`, and a worker's Build checked for its TechPrerequisite at the order (*Object
  model > Tech prerequisites*).
- **Cults construction** (`sim/builds/shared.rs`, `crate::spot_work`, `sim/spot_work.rs`, `orders/assist.rs`,
  `sim/sites.rs`, `placement/wall.rs`; slice M5-E, plan decisions T59 and T62; `factions/cults/economy.md > Cults
  Building Mechanics`, design changes 0071, 0072, 0339 and 0340): Recruits build together, from outside. The Recruit's `WorkerDef::builds` (Hall,
  Storage, Armory, and since slice M5-G2 the Gunworks, the Machine Shop and the Scrap Hangar,
  `tests/cults_construction/tech.rs`) and each structure's `SiteDef` (price, frames, `BuilderGoes::Outside`) are data,
  so a further Cults building is a def alone: every Cults structure has its site since slice M5-G2, and its slice adds
  it to `builds`; `WorkerDef::shares_builds` is what makes a worker's Builds shared.
  - **The ticket** (T59, `Ticket`): a Build given to several such workers is one order they each hold, current or
    queued, by its owner, kind, origin and orientation, derived from the orders: one ghost, one structure for the
    price, paid once as it is given (`Sim::give_shared_build`), refunded once when its last holder drops it before its
    site starts (re-ordered, killed, failed together; design change 0340), never after. The gate
    (`Sim::check_shared_build`) takes the whole group, no `Reject::SeveralConstructors` (it stays for Drones and
    Agents), and checks the site with the group left out (`Sim::group_build_check`).
  - **The site** starts at the first holder whose walk ends within `BUILD_REACH_SU` of the footprint
    (`Sim::arrive_together`, in the arrivals' step), on the truth check (`Sim::arrival_check_without`), checked twice
    (slice M5-J1, design change 0362): with nobody left out it starts; else with the ticket's holders left out, any
    other unit on it, of any player, or its ground fails the whole ticket, refunded once, and nothing retries (0339);
    else only its own Recruits stand on it, current or queued holders, and it waits, checked again each tick, as they
    walk off to their spots (a walk ended on the footprint looks again, `Sim::step_spots`, so none stands still there).
    It starts at 10% HP; every holder's Build becomes `OrderKind::Assist(site)`, queued ones too. It gains a frame a
    tick for each unit on an Assist of it within reach with its walk over (`spot_work::counts`; 0339's Recruit-frames:
    those walking or waiting add nothing), so n Recruits build n times as fast and none stall it, its progress and HP
    kept; complete, `SimEvent::SiteBuilt`, its builders' Assists end and they go on to their queued orders or idle
    (0340). X refunds 75% (`Sim::cancel_site`).
  - **The spots** (T62, 0339, `crate::spot_work`): each worker claims its own spot as it gets the order, in its
    order's state: the part of the builders' wall nearest it (`placement::Wall`, its box `BUILD_STANDOFF_SU` off the
    footprint), or else the nearest free part along the wall either way, clockwise first, a room (its box and the
    standoff) from the others' (`Claim::keeps`); with the wall full, it waits on a row from the third out (row 1 kept
    free, the way round) and doesn't count. It walks round the footprint and the others' rooms: along the row outside
    its spot's, by that row's corners, and in from there (`Sim::leg`); walks press on (`Mover::press_on`). The count
    of those at work and the claim are written over an order and its target, and slice M5-N8's Repair reuses both
    (*Repair* below).
  - **Assist** (`OrderKind::Assist(site)`, `orders/assist.rs`): of an own site built from outside, by its builders
    (`Reject::NotASite`, `Reject::NoWorkers`); the right-click on such a site (`orders::resolution`).
- **Repair** (`crate::repair`, `orders/repair.rs`, `sim/repairs.rs`, `tuning/repair.rs`; slice M5-N8, plan decision
  T83; `factions/cults/units/recruit.md > Repair`, `factions/syndicate/units/agent.md > Repair`, design changes 0321,
  0322, 0325, 0328, 0329 and 0360): one mechanism for every repairer, its rules in `crate::repair`'s docs.
  - **Who repairs what** is data, `WorkerDef::repairs` (`RepairDef`): the Recruit
    `RepairTargets::OwnBuildingsAndVehicles`, paid; the Agent `RepairTargets::Kinds`, the Tunnel free and the Laser
    Turret paid (`RepairPayment`); the Drone nothing, and slice N9's Field Engineer is a def. A vehicle
    (`repair::is_vehicle`) is a unit of neither infantry base that isn't a glider, by its movement; a building is a
    structure but for the underground expansions, which no one repairs (they regenerate, below).
    `repair::payment(repairer, target)` is the one rule, `repair::repairs` its answer.
  - **The terms** (`repair::terms`): from what built the kind, its site (`SiteDef`: a Cults building's; the Tunnel's
    480 frames at every tier, 0328), its expansion's build (`ExpansionDef`: the Laser Turret's) or else the first
    `Recipe` that makes it in the registry's order (a vehicle's Space Crystals only), shared out by
    `REPAIR_TIME_SHARE` and `REPAIR_COST_SHARE` (half and half, placeholders): MaxHP / frames a tick per repairer. A
    repairer that pays pays Space Crystals, so a price in another resource (the Tunnel's Supplies) is repaired for free
    or not at all (placeholder).
  - **The target** (`repair::can_repair`, `Sim::can_repair`): another object of the repairer's owner's, complete (no
    site, nothing transforming) and below its MaxHP (`repair::damage`, repair's one read of a MaxHP: the one with its
    Augmentations, `Object::max_hp_with`, slice M5-N1a), of a kind its type repairs. The gate (`Sim::check_repair`) keeps the units that can, else `Reject::CantRepair(target)`;
    `Sim::aimed` gives the kind as `Aimed::repair` for the right-click (*Commands, events and replays > Default
    orders*).
  - **The order** (`OrderKind::Repair(target)`, `RepairWork`): at a building the repairer works from its own spot beside
    the footprint (`RepairWork::Spot`: the spots' step, as an Assist's, *Cults construction*), a Tunnel and a lowered
    Laser Turret alike, which nothing of a repair raises; at a vehicle it follows it (`RepairWork::Follow`,
    `travel::approach` within `PROXIMITY_SU`, inside `BUILD_REACH_SU`).
  - **The step** (`Sim::step_repairs`, *Tick pipeline*): each unit whose current Repair's target it can still repair,
    within reach of its edge with its walk over (`spot_work::at_work`: a footprint, or a vehicle's Silhouette),
    restores its share, so that n repairers give n times the rate; the free ones first, then those that pay, in whole
    crystals as the HP enters each step of the price (`repair::step_of`, ceil(HP × price / MaxHP), and `top_of_step`),
    with no state of its own, a bank short of one stopping the HP at the top of the last step paid. The Repairs end
    (`Sim::end_work`, the queued orders going on): all of a target's at full HP, all its paid ones when the bank runs
    out (those on their way too, placeholder), any whose target no longer suits it (not its owner's: a neutral turret,
    0360; at full HP; no longer complete), and on the removal path any whose target left the game.
  - **Measured** (`tests/repairs_perf.rs`, ignored, release): 40 Recruits repairing 10 Armories from 1 HP, 12 rounds,
    a mean of 0.025 ms and a p99 of 0.09 ms a whole tick, the economy phase's 0.003 ms (load average 1.4 to 4.4).
    `structures_perf` (no repair in it) pays the repairs' and the regeneration's walks of every object: 3 µs more in
    the economy phase, 0.355 to 0.360 ms a whole tick (+1.3%, 8 interleaved rounds against slice M5-U1's build).
- **Expansions' regeneration** (`Sim::regenerate_expansions` in `sim/expansions.rs`, `crate::regeneration`,
  `tuning::tunnel::EXPANSION_REGEN_PER_SECOND`; slice M5-N8; `factions/syndicate/structures/README.md > Tunnel
  Expansions`, design changes 0329 and 0330): "Not repaired; they regenerate". Each complete underground expansion of
  its owner's with a parent Tunnel (`tunnel::area::parent`: one of its owner's in operation whose area contains it)
  regains 1% of its MaxHP a second (placeholder), always on, under fire too; a neutral one decays instead (*Object
  model > Losing Tunnels*); the Laser Turret, which isn't underground, regenerates nothing (Agents repair it,
  placeholder). `regeneration::regenerate` is the one rule of HP regained as a share of MaxHP a second, the network's
  held units' too (`network::Held::regenerate`), up to the MaxHP with a unit's Augmentations (`Object::max_hp_with`).
- **Adding to the economy** (slices M2-D2, M2-F): put the new step into `economy::step` at its numbered place, advance
  progress by `PowerSummary::speed` from the tick the command applies (*Recipe timing*), take costs with
  `spend_crystals` when the command applies, and give refunds through `Refund`; `construction.rs` is the pattern.
- **Defeat and the end of the game** (`economy/defeat.rs`, slice M2-F2, plan decision T8; `core/game_rules.md >
  Defeat Condition`, `core/game_rules.md > On Defeat`, `core/game_rules.md > End of Game`; design changes 0100 to 0102,
  and 0163 and 0164, slice M3-V):
  - **Defeat**: a player is defeated in the tick their count of owned structures drops to 0 (every structure
    counts, plates included): `defeat::step` compares the owners of structures as the tick started with those as it
    ends, so a player who never owned a structure (a sandbox's) is never defeated, and nobody is once the game is
    over. `Player::defeated_at` (hashed, `#[serde(default)]`), `SimEvent::PlayerDefeated`.
  - **On defeat** (0101): the player's remaining objects, units only since they own no structure, become neutral
    (`owner: None`): orders cleared, attack disengaged, mover stopped, and `Mover::repath` (see *Navigation > Path
    knowledge > No owner*). Neutral, they give no vision, never auto-target and are never auto-targeted, never
    yield, and any player may destroy them with an explicit Attack. A raised Riot Shield stays raised, and an
    attack already Firing or in Cooldown runs its course. No producer is ever handed over: a defeated player owns
    none. The gate refuses every command of a defeated player (`Reject::Defeated`), and a debug spawn of an object
    for them, so a defeated player owns nothing for good. Their units in a Tunnel Network are destroyed instead
    (design change 0258 point 5, `Sim::clear_defeated_networks`). A Syndicate player's expansions that a lost Tunnel
    left neutral no longer count, so losing the last Tunnel defeats a player with no other structure (slice M4-H,
    *Object model > Losing Tunnels*; whose stranded-unit rule takes the network units first, in the removal).
  - **Sides**: the sim has no team or alliance data yet (fixed teams are deferred, `core/map_editor.md > Map Types >
    Custom`), so every player is their own side (`defeat::allied`) and `Sim::allies` is empty. Teams go there.
  - **The end**: the defeat that leaves at most one side standing ends the game in its tick: `Sim::outcome()` is
    `Outcome { tick, winners }` (hashed, serialized, `#[serde(default)]`), after the tick's `PlayerDefeated`s comes
    `SimEvent::GameOver { winners }`. The winners are the side left standing, its defeated members included.
    When the last sides fall in the same tick nobody wins: the game is a **draw** (0163, `Outcome::is_draw`, the
    outcome without winners; derived, so the state and its files are as they were), and every player sees Draw,
    those defeated before it too. From then on the gate refuses every command (`Reject::GameOver`). The sim would
    step on if asked; the client stops it (see *Client architecture > Sim driver*).
  - **Leaving for the results screen** (0164) is the client's (`hud/results.rs`, see *Client architecture > HUD*):
    the sim runs on as before, and a player whose side fell while the game goes on is simply no longer shown it.

## Production

Producing structures, their rally points and the exit doorways (`production.rs`, `production/`, `sim/producers.rs`,
`sim/producers/step.rs`; slice M2-D2, the army caps at production start slice M3-P;
`factions/gdo/structures/barracks.md`, `core/entities.md > Structure Type > Exit Doorway`; the Colonists' portals, slice
M3-D: `factions/colonists/structures/robo_portal.md`, `factions/colonists/structures/bio_portal.md`, plan decision T19;
their energy doorway, slice M3-D2: design change 0145, `factions/colonists/structures/README.md > Portals > Portal Exits
(energy doorway)`).

- **State**: `Object::production` (`Production`: the BuildQueue, whose head is the unit in production, so the
  limit `QUEUE_LIMIT` (20) includes it, design change 0081; its entries `QueueEntry`s, a unit or a pool building's
  Release, slice M5-F2, a unit's written as its kind alone so the files and hashes from before are as they were; its
  progress in `RATIO_ONE`s, a Recruitment Hall's in tiles; the rally point; a Hall's completion tick, `completed_at`,
  hashed and saved only while not 0; a pool building's `reserved` units, entry by entry, hashed and saved only while
  any). What a type produces is `StructureDef::production` (`ProductionDef`: its exit rule, `ExitRule`, the
  `Recipe`s: kind, Space Crystals, frames, and for a Hall its `RecruitmentDef`). `Sim::production_status(id)` is what
  the interface shows: the entry, the progress out of its total, `waiting` (finished, held for the doorway) and
  `blocked` (the army cap the unit
  at the front waits for at 0%, design change 0143). `Production::blocked`, the stored mark of that wait, is hashed and
  saved only while true (`#[serde(default, skip_serializing_if)]`, as `Mover::momentum` is while non-zero), so that the
  games without a wait, the goldens among them, keep their hashes.
- **Commands**: `Command::Production { objects, command }` with `ProductionCommand::Train(kind)`,
  `CancelProduction` and `SetRally(Target)`, and a pool building's `TrainFrom { kind, from }` and `Release(from)`
  (slice M5-F2); the gate's rules are in *Commands, events and replays*.
- **Costs** (plan decision T7): a unit's Space Crystals are paid when it is queued, and only they and room in the
  queue are checked then. Its army cost (Unit Control, Beacon Capacity; see *Economy*) is taken when its production
  starts (design change 0143, which replaces 0082's reservation at queue time, for every faction): no `Reject` or
  `Refusal` is about the army cap. Cancelling refunds in full (`Refund::Full`) and takes the last entry, a unit
  waiting for room included. When a producer leaves the game, destroyed or removed, `Sim::remove_objects` refunds
  its whole queue the same way, a finished unit waiting inside included (design changes 0084, 0085;
  `Sim::refund_production`); of the cap it frees only what its unit in production took.
- **Group production** (design change 0083): a `Train` to several producers serves them by the work they have
  queued, least first (`Production::remaining_work`: remaining production time, a placeholder reading), ties by
  id, each check counting what the ones before it took, until the crystal check fails (0143: the army cap plays no
  part). A full queue is passed over. The others get `SimEvent::ProductionRefused`.
- **The start check** (design change 0143, `sim/producers/step.rs`): in the producers' step, before a unit at 0% at
  the front of its queue advances, its owner's `ArmyCap::fits(cost)` (`cost == 0 || available − used ≥ cost`: a
  unit without a cost always starts, even over the cap, design change 0176 point 3) must hold, with
  the caps worked out once a step for the players who need them and the units that start in the step added as they
  do. Without room the producer stays at 0%, `Production::blocked`, and checks again every tick; the tick the wait
  begins it raises `SimEvent::ProductionWaiting { producer, reason }`, once (placeholder for 0143's notification,
  whose system is TBD). Producers waiting for the same room are served in id order, the step's, lowest first
  (design change 0175). A unit in production is never checked again: it always finishes (Q0041 point 2). At a power
  ratio of 0 a unit that passes makes no progress, so it neither starts nor takes anything, and another producer may
  take the room (design change 0176 point 2).
- **The step** (`Sim::step_production`, see *Tick pipeline*): the start check, then the unit in production advances
  by `PowerSummary::speed` (design change 0089), capped at its total. Trained in tick `t` at full power, a
  Peacekeeper (80 frames) is released in tick `t + 79`; at 30/51 in tick `t + 135`; a Drone or a Sabre Legionnaire
  (160 frames, the Colonists have no Power) in tick `t + 159`. The next one starts from 0 in the tick after a
  release.
- **The Recruitment Halls** (slice M5-C, plan decision T57; `recruitment.rs`, `sim/recruitment.rs`;
  `factions/cults/structures/recruitment_hall.md`): a Hall's production is the producers', in an automatic mode that its
  `ProductionDef::recruitment` (`RecruitmentDef { area: 10, capacity: 20 }`) switches on. **The claims**
  (`recruitment::Claims`, derived each time, never stored): each Recruitable tile (`TilePreset::recruitable`: every
  default preset but Water) of a Hall's 10 × 10 area centred on its footprint goes to the first completed Hall whose
  area has it, by `Production::completed_at`, and of Halls completed in the same tick the one whose centre is nearer
  the tile, then the lower id (design change 0341 point 3), whoever owns it (0341 point 5); buildings never matter;
  the whole area lies on the map, the placement check refusing a Hall whose area would leave it (0341 point 1, slice
  M5-J1: `WholeProblem::RecruitmentAreaOffMap`, `placement/recruitment_area.rs`), and a debug spawn's tiles off the
  map count among its 100 and can't be claimed (Placeholder (not in the design: 0341 rules such a Hall out)); a Hall
  under construction claims nothing (0341 point 4, `Object::is_operational`). **One rule of claiming** (slice M5-J2,
  `recruitment::winners`): each competitor (`Contender`) ranked, every Recruitable tile of the areas to the least by
  (rank, distance² from its centre in half tiles, id); the game's claims rank the completed Halls by `completed_at`
  (`Claims::of`, `claimed_tiles` for a selected Hall: `Sim::claimed_tiles`), and **the placement preview**
  (`recruitment::preview`, `factions/cults/structures/recruitment_hall.md > Recruitment Area display`, design changes
  0350 to 0352) ranks an explicit list of `Claimant`s, a pure function: `Rank::Completed(tick)` first, then
  `Rank::Pending(n)` (own sites and ghosts in their order of placement, 0351), then the Hall placed;
  `Preview::of` gives each tile of the placed area `Claimable`, `Taken { claimer: Own | OwnPending | Other, by }` or
  `NotRecruitable` (known non-Recruitable ground, off the map; ground the player never explored passes, as in
  `Mode::Known`), and `effectiveness_percent`, rounded down (`recruitment::effectiveness_percent`, shared with
  `HallState`). Who the claimants are is the caller's: the client gathers what its player knows
  (`placement::recruitment_areas::known_claimants`). `PendingBuild::group`, its order's `OrderGroup`, gives the order
  the ghosts were placed in; `Sim::pending_without` is public for the display. `completed_at` is the tick
  `Capabilities::fresh` gives the producer, so a spawn's
  insertion tick, 0 for a map's, and the tick `Sim::start_functions` completes a site (slice M5-E). **The start**
  (`Sim::start_recruits`, first in the producers' step): a completed Hall with an owner and an empty queue queues its
  recipe's Recruit, free, while its usage (`recruitment::Lineages`, its Recruit in production counted: design change
  0342 point 3) is below its capacity, BaseUnitControlCapacity × claimed / 100 rounded down (`recruitment::Capacity`,
  design change 0341 point 2: 33 tiles stop at 6), whatever the global Unit Control (design change 0344); a Hall that
  releases its Recruit queues the next at once (`Sim::recruit_next`), so it shows 0% rather than nothing between
  them, and one at capacity starts again from 0% in the tick a unit of its lineage is lost (0342 point 4). **The
  progress**
  gains the claimed tiles each tick, whatever the power, to a total of 192 × 100 (`production::total_of`): 192 ticks
  a Recruit at 100 tiles, 384 at 50, the k-th of a full Hall in tick 192 k (6 → 20 in 2688 ticks). **The release**,
  out of its one exit side, B (design change 0342 point 1), sets the Recruit's lineage to the Hall. **No X** (0342
  point 2: "Recruits can't be cancelled"): a `CancelProduction` finds nothing to cancel at a Hall
  (`Reject::NothingQueued`), and its panel has no X. A `Train` to a Hall is `CannotProduce`.
  `Sim::claims` and `Sim::hall_state` (`HallState`: claimed and area tiles, effectiveness, usage, capacity) are the
  interface's.
- **Pool buildings** (slice M5-F2, plan decisions T61, T70 and T71; `sim/producers/pool.rs`, `production/queue.rs`;
  `factions/cults/structures/armory.md > Training`, `factions/cults/structures/README.md > Choosing Recruits, Release
  and Eject All`; design changes 0332, 0337, 0347 and 0348): a producer that also holds units (`holding::trains`: a
  `StructureDef` with both `holds` and `production`), the Armory first, makes its units of the units inside, one path
  for every such type, so that the Cathedral, the Chop Shop and the Airstrip are data. **An entry**, `QueueEntry::Unit`
  or `QueueEntry::Release` (`frames`: a recipe's, a Release's `RELEASE_FRAMES`, 4, the design's "very fast (**4
  frames**, placeholder)"; `cost`: a recipe's Space Crystals, a Release none), **reserves its units when it is
  queued**, `Production::reserved` beside the queue, entry by entry (`QueueEntry::units_reserved`: the unit's Unit
  Control cost, at least 1; 1 for a Release): the free units of its kind (`Sim::free_units`, `RecruitKind`, those
  that entered first), else `Refusal::NoRecruit`. They stay inside, out of the pool (`Sim::pool`), and the capacity
  counts them (0347 point 5), so the capacity alone bounds the queue: no `QUEUE_LIMIT` for a pool building. **The
  front** advances a frame a tick, the Cults having no power, and finished, its first unit is ready to leave
  (`Sim::ready_to_leave`, `Held::leaving`), the entry waiting at 100% while the units ready before it go out first
  (*Object model > Buildings that hold units*). **Out** (`Sim::let_out`), a training's unit is the Recruit changed in
  kind (`Sim::train`, plan decision T15: its id, its lineage): infantry keep its damage (`transform::follow_max`) and
  its Augmentations, vehicles and aircraft come out at full HP without them (design change 0332,
  `factions/cults/augmentation.md > Persistence`), and a load goes (0347 point 3); the other units it reserved leave
  the game into it, held, with no `Died` and their lineages appended, so the Halls' usage stays as it was. The entry
  then leaves the queue, and the next starts in the next tick. A Release's unit comes out unchanged. **X** takes the
  last entry back, refunding its Space Crystals and freeing its units into the pool (`Sim::unreserve`); a pool
  building that leaves the game refunds every entry but a finished one, whose unit comes out as the trained unit
  (design change 0349 point 3, slice M5-F3), its other units evacuated as they are (*Object model > Buildings that
  hold units*). **Invariant R1** (`testing/invariants/reservations.rs`): only a pool building reserves, an entry
  each, each of its units held in its building and reserved once, and only the front's first may be leaving. The
  Gunner's recipe requires a Gunworks (slice M5-G2), checked as it is queued and never again
  (`tests/armory/gunner.rs`: losing the last Gunworks keeps the Gunners queued and in training).
- **The exit doorways** (`production::ExitDoor`, the one geometry for the release, the movement step and the
  invariants; `ExitDoor::of` reads the building's `ProductionDef::exits`, an `ExitRule`, against its sides as
  placed, `sides`). Two kinds:
  - **A side doorway** (design change 0080; `production::Doorway`, `ExitRule::Side(B)`: the Barracks' one side B):
    a door (`Door`) in the middle of that side, wherever it faces. A finished unit comes into the game in it **half
    inside the building**: its centre on the exit side, facing out, with `Mover::exit` set (see *Movement and
    collision* for the collision exception). When another unit or a blocked tile is right in front of the door it
    stands as far out as there is room, fully inside against the exit side at most (`Sim::doorway_spot`;
    `Door::room_before` counts only what is in front, not what stands behind the door or beyond the building): the
    design's "half inside" would overlap what stands there, and its verification wants the unit to appear all the
    same. **Several exit sides** (slice M5-C, `ExitRule::Sides(A)`: the Recruitment Hall's four, until design change
    0342 gave it one, B, so that no type has them now; `core/entities.md > Structure Type > Exit Doorway`: "a side
    whose doorway is occupied counts as blocked, and the building pauses only when every eligible doorway is
    occupied"): a door on each, tried in `Doorway::order`, the side facing the rally point (the greatest cosine with
    the direction from the centre; of two alike, a diagonal, the clockwise one), the bottom one without a rally point,
    then clockwise (Placeholder (not in the design: no building has several exit sides now)); the unit
    takes the first door free of a released unit that has a spot for it. A unit in the doorway is in the door whose
    side it is least inside of (`Doorway::door_at`), which the movement step, the paths and the invariants ask.
  - **A portal's energy doorway** (slice M3-D2, design change 0145; `production::EnergyDoorway`,
    `production/exit/energy.rs`; `ExitRule::Centre(&[A])`: the Robo Portal's and the Bio Portal's A sides, Q0041
    point 5): a finished unit **materializes at the footprint's centre**, facing its heading
    (`Mover::exit_heading`, set and cleared with `exit`, hashed and saved only while set), and walks straight out
    along it (see *Movement and collision > The exit doorway*). **The open directions come from the sides**,
    never from a building's name: a direction is open when the ray from the centre leaves the footprint's box
    through an open side (`EnergyDoorway::margin`: how far out the ray reaches the farthest open side's line less
    the farthest closed side's, at least 0; `is_open`). The arc's two ends, exactly the diagonals of a square
    footprint, are open (**the Director's reading**). The Robo Portal (AAAA) is open all round; the Bio Portal
    (AABB) from its top-left corner's direction clockwise round to its bottom-right one unturned, turning and
    flipping with it. The Heavy and Star Portals' arcs will come from their sides the same way.
  - **No doorway: into the Tunnel Network** (slice M4-C, `ExitRule::Network`, the Hideout's;
    `factions/syndicate/structures/README.md > Rally Point Behavior`, design changes 0252 and 0253;
    `Sim::release_into_network`): the finished unit comes into the game held in its owner's network
    (`SimEvent::UnitProduced` without a `Spawned`), with no transit delay, and nothing ever holds the release up.
    With no rally point, or one on a Tunnel of its owner's, it stays inside. Any other rally is on the surface: the
    unit joins the end of its **parent Tunnel's** eject queue (`tunnel::area::parent`, derived) with its default command
    on the rally target, resolved at production (`Sim::rally_order_of`); a rally on an own underground structure is a
    Move to the point above it, not an object target (0252). Without a parent Tunnel, or one whose tier lets the unit
    out, it stays inside. An entry's order whose target leaves the game meanwhile is dropped (`forget`), and the unit
    then walks straight out and idles.
  - **Occupied** while a unit the building released is in its doorway (`Mover::exit` is the building's id). While
    it is, the next finished unit is held (progress at its total, `waiting`), the queue behind it doesn't advance,
    and it is released in the first tick the doorway is free (0080; for the portals 0145 point 5, "it waits fully
    inside until the previous unit is out"). So at most one unit is in a doorway, or in each door of a building with
    several exit sides, which waits only while every door is occupied.
- **The heading** (`EnergyDoorway::heading`, 0145 point 3; `release`, `Sim::release_unit`): **towards the rally
  point** (`Rally::point`, where the owner last saw an object) if that direction is open and its way out is free;
  else **the nearest open direction whose way out is free**: of the arc's ends and the open ones of 32 fixed
  headings (`exit::HEADINGS`, every 11.25° from east, exact under the square's eight symmetries), the one at the
  smallest angle from the rally point's direction, ties clockwise first (**the Director's reading**); with every
  way blocked, the nearest open one all the same (the unit stops half out, point 4). With **no rally point**: a
  random one of the open headings whose way out is free, uniformly (all the open ones if none is free). **A way
  out is free** (`EnergyDoorway::way_out_free`) when the unit's box, swept from the centre to one unit-length
  beyond where it comes clear (`outside`), enters no blocked tile of the grid of the unit's own class and size (the
  true grid; the footprint's own tiles don't count, the map's edge does) and no unit of its layer
  (`nav::segment_enters_box`). The release waits while the unit's box at the centre isn't clear, of tiles or of
  units (nearly never: only a unit in the doorway, which occupies it, can be there).
- **The sim's Rng in production** (its first use): the random heading is `Rng::below(n)` among the `n` candidates,
  drawn in `Sim::release_unit` (economy phase, producers in id order) **only when `n` is 2 or more**, and only by a
  portal without a rally point, in the tick it releases. A Barracks never draws, nor does a portal with a rally
  point, so the GDO goldens don't move. The Rng is hashed and saved, so replays and saves draw the same headings.
- **Rally points** (`production/rally.rs`): `Rally::Point` or `Rally::Object { id, at }`. A released unit gets its
  **own default right-click command** on the rally target (design change 0136, `systems/controls/interface.md
  > What a rally point makes a new unit do`: "No per-building lists override this"), at its release, from where it
  appears (out of a portal, the centre: its path starts there), through `Sim::give_order` like a player's:
  `Sim::rally_order` is `Sim::default_order` (`orders/resolution.rs`, the one rule of *Commands > Default orders*) on
  the target as the owner knows it, so a Drone rallied on a patch gathers it, on an enemy unit follows it and on an
  enemy structure moves to it (it is unarmed), a Peacekeeper attacks an enemy, and a rally on the ground is a Move
  there. Without a rally point it walks out of the doorway and stops (design change 0086; out of a portal, along its
  random heading, where it comes clear). The producer itself as the target clears the rally. A rally on an object
  that is removed and that the owner doesn't remember resets to None (`Production::forget`). At the end of each tick
  `rally::follow` (`Sim::follow_targets`) notes where the owner sees the object: an **enemy unit** leaving its sight
  turns the rally into a ground rally at its last-seen position (design change 0088), so an enemy dying in the fog
  changes nothing; a **structure** or a resource stays the rally out of sight, at its last-known position, until the
  owner sees its place without it (design change 0139: "A rally point on an enemy structure stays on it after it
  leaves sight"), and a rally may be set on one the owner only remembers.
- **Tests**: `src/production.rs` and `production/exit/tests.rs` (the queue, the Barracks' door in all 8
  orientations and its rule, the room before a door, the Hall's four doors and their order), `recruitment/tests.rs`
  (slice M5-C: the area, the exact capacity, the claims and their order, the lineages' points),
  `recruitment/preview/tests.rs` (slice M5-J2: the preview's claimants in their order, completed before pending,
  0351's placement order, 0341's tie rule as the game's claims split it, the Effectiveness rounded down, off-map tiles,
  known Water and unexplored ground; in the client, `placement/recruitment_areas/tests.rs`, the pending Halls' order
  and the mesh, and `crates/game/tests/commands/recruitment_areas.rs`, with the fog on: 0350's, 0351's and 0352's
  Verifications, a Hall never seen, the label, a selected Hall's claims and an enemy's none),
  `tests/recruitment/` (slice M5-C: a full Hall's 6 → 20 in 2688 ticks then none, 50 tiles' 10 at 384 ticks,
  buildings in the area, Halls sharing tiles and one lost in a tick, a destroyed Hall's Recruits and capacity, a lost
  Recruit restarting a full Hall, orphans, X, a `Train`, the rally on the ground and on a patch, a game replayed and
  saved, and the sides a Recruit leaves by, up to every door occupied), `production/exit/energy/tests.rs` (slice M3-D2:
  the 32 headings' symmetry, the Robo Portal open all round and the Bio Portal's arc in all 8 orientations, a wide
  doorway's arc from corner to corner, the walk out, the choice of heading and its ties and draws, a way out
  blocked by a cliff, by units and by the map's edge), and `tests/production/`: `queue.rs` (times, the power ratio, train and cancel, the limit
  of 20, Unit Control, the gate, group production, refunds, a game replayed and saved), `capacity.rs` (slice M3-P:
  the army caps at production start, design change 0143's and 0112's Verifications with a real deployment, the
  waits and their notification, id order, the refunds of a waiting and of a producing producer, a cancelled wait, a
  producer without power, waits saved and replayed, and slice M3-D's portal over capacity), `rally.rs` (each kind of
  target, clearing, an enemy leaving sight, a remembered structure), `exit.rs` (the verification of design change
  0080, all 8 orientations, a crowd of idle units, a half-blocked exit, a path from inside the footprint, no way
  back in), `stalls.rs` (walled in by a building and by units and freed again, the waiting unit's refund, saves
  in the middle of an exit, and a random game of 3 000 ticks around four doorways), `portals.rs` (slices M3-D and
  M3-D2, design change 0145's Verifications: a Drone's and a Sabre's times and their walk straight out of the
  centre, the heading for a rally point in every direction and on an object, the Bio Portal rallied behind it and
  its arc in all 8 orientations against 9 rallies, random headings by seed, a way walled off, rallies on an enemy,
  a destroyed portal's refund, a replay and a save in the middle of a walk), `portal_stalls.rs` (slice M3-D2: a
  portal ringed by units, its unit half out and the next held at 100% until the first is out, that unit's refund,
  and negative tests of I1 and I2 around the energy doorway) and `half_out.rs` (slice M3-K2a, 0145 point 4: a
  Sabre stuck half out is a normal unit, which auto-targets an enemy in reach, carries out an Attack and is
  attacked; a click picks it: the client's `Selectables::at`).

## Commands, events and replays

- All player intent is a `Command`. It is wrapped in a `PlayerCommand { player, command }` and collected in one
  tick's `TickInputs`. The client never mutates sim state. It reads `Sim` and submits commands.
- The sim validates every command (player, ownership, ids, costs, placement, and positions that are finite and on the
  map), and it **ignores invalid ones**. It must never panic on player input, so use checked or saturating arithmetic
  on numbers that come from input. Panics are only for programmer errors, such as an invalid scenario passed to
  `Sim::new` or an out-of-bounds `Map::set_tile`.
- **The command gate** (`sim/gate.rs`, with each command family's checks in its own file, plan decision T28: orders
  and abilities in `sim/gate/orders.rs`, debug commands in `sim/gate/debug.rs`, production in `sim/producers.rs`,
  construction in `sim/construction.rs`): `Sim::validate(&PlayerCommand) -> Result<(), Reject>` checks a command
  against the current state without changing it, and `step` applies a command (`Sim::apply_command`, `sim/commands.rs`)
  only through the same check (`Sim::check`, which hands the application the objects and positions it resolved), so the
  two can't disagree. In order: the player is in the game (`Reject::UnknownPlayer`), the game isn't over (`GameOver`)
  and the player isn't defeated (`Defeated`, see *Economy > Defeat and the end of the game*), a debug command needs
  debug commands on (`DebugDisabled`), then the command's own rules: every id it names exists (`UnknownObject`) and, for
  a player's orders to objects, belongs to that player (`NotOwned`, through `Sim::referenced(id, Some(player))`; debug
  commands and targets may name anyone's objects), positions are finite and on the map (`OffMap`), an order's target
  object is one the issuing player sees or remembers (`NotVisible`, below), spawns fit (`Spawn(SpawnProblem)`) and keep apart
  from other footprints (`Overlaps(id)`, see Object model). A rejected command changes nothing, neither the state
  nor the events, and never panics.
  The one rejection the sim reports is a Place or a Warp In the placement check refuses (plan decision T4): the
  client checked it before sending it, so the game changed in between, and `SimEvent::PlacementRefused` (output,
  outside the state hash) says why (`Sim::report_refusal`). Add another such event only when the UI needs it.
- **`Command::Order { units, kind, queue }`** (`systems/controls/commands.md > Unit Command`, `docs/movement.md > 3`):
  `OrderKind` is `Move(Target)` (a point or an object), `Attack(id)`, `AttackMove(point)`, `Patrol(point)`, `Hold`,
  `Stop`, `AttackGround(point)` (slice M4-O, for the units that can target the ground: `CannotTargetGround` if none
  can; see *Combat > Attack Ground*), and for workers `Gather(patch)` and `DropOff(node)` (`factions/colonists/units/drone.md > Unit
  Commands`). The gate takes an order whole or not at all, but tolerates stale selections: at most `MAX_ORDER_UNITS`
  ids (`TooManyUnits`), repeated ids count once, ids of objects that are gone and objects that take no orders are
  dropped, and nothing left is `NoUnits`; any id of another player's object rejects it (`NotOwned`). Points must be
  finite and on the map, target objects must exist (`UnknownObject`) and an attack's target be destructible
  (`NotAttackable`). A Gather's target is a Space Crystals patch the player sees or remembers, with no plate on it
  that the player knows of (`NotGatherable`; a remembered one judged by its record), a Drop Off's a drop-off of the
  player's own (`NotDropOff`), and only the workers among the units
  take either (`NoWorkers` if there is none; `sim/workers.rs`), a Drop Off only those that carry a load
  (`NothingToDeliver` if none does, design change 0140). `Deploy { origin }` (slice M3-F, see *Economy >
  Beacon Deployment*) goes to the one Drone among the units (`NoWorkers` if none, `SeveralConstructors` if more: the
  client picks one, design change 0125), with a site allowed as far as the player knows, its pending sites reserved
  (`Unplaceable(PlacementProblem)`, not reported: `Sim::deploy_check`). `Enter(tunnel)` (slice M4-C,
  `systems/controls/commands.md > Enter`, `Sim::check_enter`) names a Tunnel of the player's own in operation
  (`NotOwned`, `NotATunnel`), and goes to the units its tier lets in (`network::admits`; `NoTransit` if none: any
  unit of the Tunnel's owner may enter, whatever its type's faction). `Build { kind, origin, orientation }` (slice
  M4-E, `Sim::check_build`) goes to the one unit among them that builds `kind` (`NoWorkers`, `SeveralConstructors`),
  Shift-queued only to a unit with no Build yet (`ConstructorBusy`: each Agent takes one placement), and is paid for as
  it is given (`InsufficientSupplies`), its site allowed as far as the player knows (`Unplaceable`, not reported:
  `Sim::build_check`). Without `queue` the order replaces the unit's
  queue and current order; with it, the order is
  appended (a full queue drops it). Each order command is one group (`Sim::next_group`); a location order gives
  each unit of a group its own destination (the magic box, see *Navigation*). What each order does is in
  *Behaviours*.
- **`Command::Ability { units, ability }`**: the units use an ability at once, beside their orders: no queue, no
  Shift, no group, and their orders go on. `Ability::RiotShield { raise }` says which way rather than "toggle", so a
  group and a replay mean the same whatever each unit's mode. `Ability::SetDropOff(node)` sets the workers'
  AssignedDropOff to a drop-off of the player's own (`NotDropOff`), and "does not interrupt the Drone's current
  action" (`factions/colonists/units/drone.md > Assigned Drop-off`). `Ability::Convert` flips the Legionnaires'
  target form (see *Combat > Weapon Conversion*). A toggle switches its units' type at the press (*Object model >
  Type and form*, design change 0198), so a second one reaches them whatever their state. The gate tolerates stale
  selections as for orders (`TooManyUnits`, repeats, gone ids, `NotOwned`, `NoUnits`) and drops the units without
  the ability (`Reject::LacksAbility` when none is left); it refuses no toggle for the state its units are in.
- **`Command::Production { objects, command }`** (`factions/gdo/structures/barracks.md >
  ObjectInterfaceState[Barracks]`, see *Production*): the producers are the player's objects among `objects` that
  produce units, found as an order's units are (stale and repeated ids dropped, `NotOwned`); none is
  `Reject::NoProducers`. `Train(kind)` is taken when at least one producer queues the unit, else
  `CannotProduce(kind)`, `InsufficientCrystals` or `QueueFull` (the army cap is no reason: design change 0143 checks
  it when production starts); a pool building takes no `Train` (slice M5-F2) but `TrainFrom { kind, from }`, the same
  at pool buildings alone, each needing as many free units of kind `from` as the unit costs Unit Control
  (`NoRecruit(from)`; one without them is passed over as a full queue is, `Refusal::NoRecruit`) but no free queue
  slot, its capacity bounding its queue; `Release(from)` when a pool building has a free unit of the kind
  (`NoRecruit`, `NoHolders` for none that holds units), free; `CancelProduction` when a producer other than a
  Recruitment Hall (design change 0342) has something queued, else `NothingQueued`; `SetRally` with a point on the
  map (`OffMap`) or an object the
  player sees or remembers, as for an order's target (`UnknownObject`, `NotVisible`; the producer itself
  always). `Sim::validate` gives the interface the same answers for its buttons.
- **`Command::Construction { objects, command }`** (`construction.rs`, `sim/construction.rs`): the player's constructors
  carry out a `ConstructionCommand`: `Construct(kind)`, `CancelConstruction`, `CancelReady` or
  `Place { origin, orientation }` (the top-left tile of the turned footprint; the orientation optional in files, as for
  a debug spawn). The gate tolerates stale selections as for orders (`TooManyUnits`, repeats, gone ids, the player's
  objects that construct nothing; `NotOwned` for anyone else's, `NoConstructor` when none is left), then goes through
  them in id order: a Construct starts at each one that has the building on its list, is idle and can pay from what the
  ones before it left, and is rejected with the first one's reason if none can (`CannotConstruct(kind)`,
  `ConstructorBusy(id)`, `InsufficientCrystals`); a cancel goes to each one with something to cancel (`NotConstructing`,
  `NothingReady`) and refunds it (`ConstructionCancelled`). A Place names one constructor (`SeveralConstructors`; design
  change 0071) holding a ready building (`NothingReady`), and is rejected with the placement check's first problem
  (`Unplaceable(PlacementProblem)`), which is reported (above). Each object of a command is one constructor; the client
  sends a command to the group the design says (no smart-casting, design change 0071). `WarpIn { kind, orientation }`
  (slice M3-E, see *Economy > Warp In*) names one Beacon of the player's (`NoConstructor`, `SeveralConstructors`),
  not transforming itself (`ConstructorBusy`), a building of its list (`CannotConstruct`) and the bank to pay it
  (`InsufficientCrystals`), and no position: its footprint is derived, and refused like a Place's (`Unplaceable`,
  reported). `CancelTransform` goes to every transforming object of the player's named, in id order
  (`NotTransforming` when none is).
- **Targets must be known** (design change 0139, answering question Q0017 point 2; `core/vision.md > Explored >
  Targeting remembered objects`): an order (Attack, Move to an object, Gather) or a rally point may name an object
  the issuing player sees or remembers, else `Reject::NotVisible`. `Sim::target_view(player, id)` (`targets.rs`) is
  the one view of it: `Visible`, `Remembered { position, footprint }` (a record out of the player's sight: the order
  goes to the last-known position) or `Gone`. Whether the player sees an object is the one visibility query of the
  object's holder (`Vision::sees` through `Sim::is_visible`; `World::sees` for the behaviours' snapshot, the one
  ValidTarget goes by; `targets::Live::seen`), so that whatever slice M3-L adds to what a player sees (the fog's
  delay, 0178; an attacker revealed, 0177) reaches the targets too. It reads only the player's sight and records, so a remembered object
  destroyed in the fog is taken exactly as one that stands, and the gate's verdict leaks nothing. Remembered objects
  stay unselectable (the GDO player's Supply Delivery Station excepted, `core/vision.md > Explored Selectability`).
  The check is on the sim's vision, whatever the client draws (`--no-fog` is display only). A queued order passed
  the gate when it was given; what its behaviour does if the target is out of sight when its turn comes is in
  *Behaviours*.
- **`Command::Tunnel { tunnel, command }`** (slice M4-C, plan decision T36; `TunnelCommand`, to which Expand,
  Upgrade and the cancels come with slices M4-E to M4-G): the player's own Tunnel in operation (`NotOwned`,
  `NotATunnel`). **`Eject { kind }`** (`factions/syndicate/tunnel_menus.md > EjectMenu`): the tier lets the kind out
  (`NoTransit`), and a unit of the kind in the player's network is free to leave: not reserved on a queue already
  (`NotInNetwork`) and past its transit delay (`InTransit`: **a reading**, design change 0245's "For 2 s it can't be
  ejected anywhere" refuses the click rather than queueing the unit). The gate picks the one inside longest
  (`Sim::next_out`, design change 0251, ties by id) and the command reserves it at the end of the Tunnel's queue.
  The queue can't be cancelled, and goes with its Tunnel, its units staying in the network. **`Upgrade`** (slice
  M4-G): `Sim::can_upgrade` (*Tunnel upgrades*); **`Cancel`** ends the Tunnel's operation, an expansion or an
  upgrade (`NoOperation`).
- **`Command::Holding { objects, command }`** (slice M5-F1, `HoldCommand`; *Object model > Buildings that hold
  units*): the player's buildings among `objects` that hold units take it, the player's other objects are ignored
  (`NotOwned` for another's, `NoHolders` if none is left). **`EjectAll`** (`factions/cults/structures/armory.md >
  Eject All`): those whose pool has a unit (`NothingInside` if none has) mark their pool for Eject All.
- **Default orders** (`orders/resolution.rs`; `systems/controls/interface.md > RightClickResolution`, design
  changes 0062, 0063, 0074 and 0136): `resolve(Abilities, Aim)` is the one rule for what a unit does against a target
  by default: the ground gives Move; a worker on a patch it can gather Gather, on its owner's drop-off Drop Off while
  it carries a load (else a Move there, 0140); an own Tunnel whose tier lets the unit through Enter (slice M4-C:
  `Abilities::transit` against `Aimed::transit`, `network::passes`); an enemy it can attack (its attack's
  TargetDomain, the target
  destructible, its mode letting it attack) Attack; anything else a Move to the object, which follows a unit (an
  unarmed Drone follows an enemy, a shielded Peacekeeper too). `Sim::aimed(player, id)` is the target as the player
  knows it (`Aimed`: enemy, destructible, domain, gatherable, drop-off, and since slice M5-N8 `repair`, the kind of a
  damaged own object; a remembered one by its record) and `Sim::default_order(unit, target)` the sim's answer; the rally
  points use it (*Production > Rally points*), and the client's right-click calls `resolve` itself on what it shows
  (*Client architecture > Command UI*).
  `tests/default_order.rs` has the table of unit kinds against targets. The rules on an object are data (slice M5-J1):
  `RULES`, a list of `Rule`s tried in order (Gather, GatherSupplies, DropOff, Enter, Assist, Repair, Attack), the
  first that applies (`Rule::order`) giving the order; a new default order is a `Rule` in its place in the list
  (slice M5-N8's Repair is after Enter and Assist, design changes 0322 and 0325: a damaged own building that the unit
  can enter is entered; `Aimed::repair` against `repair::repairs`, *Economy > Repair*).
- **Debug spawns and teleports of units** keep the unit invariants: a box on blocked ground or across the map's edge
  is `Reject::Blocked`, one overlapping another unit of its layer `Reject::Occupied(id)`, and so is a blocking
  footprint spawned onto a ground unit. Touching is allowed. A teleported unit plans its path again.
- Every new command gets its rules in the gate and tests showing that its invalid forms are rejected without a panic
  and change nothing.
- **Game options** (`SimConfig`) are part of the game setup (`GameSetup::config`, see Sim scenarios and tests): the
  `Sim` keeps and hashes them, and the `Replay` records them with the setup; map files don't hold them. `debug` turns
  debug commands on. `SimConfig::default()` has it off, as a real match will. The client turns it on
  (`AppConfig::new`; off with `--no-debug`), editor play-tests run with it on, and so does `TestSim::new`
  (`TestSim::with_config` or `ScenarioBuilder::config` for other options).
- `Command::Debug(DebugCommand::…)` covers `Spawn`, `Remove`, `SetPosition`, `SetAutoTargeting` (see
  *Behaviours*), `SetResources` (any player's banks, see *Economy*; the gate checks that the player it names is in
  the game, `UnknownPlayer`) and `SetTier` (a Tunnel's tier, `NoSuchTier` for a tier it has no row for or an object
  without tiers; *Object model > The Syndicate*), and `SetAugmentation { id, augmentation }` (slice M5-F2: a unit's on
  the map, until the Altar gives them; *Object model > The Cults*; since slice M5-N1a its HP follows its MaxHP, and
  `NotAugmentable` refuses Augmentations to a unit that can't have them, *Object model > Augmented stats*), for
  development and QA. `Spawn`
  places a tile-based object in an
  `orientation`
  (`BuildingOrientation`), which files may leave out: unturned is the default and isn't written, so replays from
  before it load unchanged. It bypasses costs and rules, only works while `SimConfig::debug` is on, and
  is never offered in a real match. A teleport sets `prev_pos = pos`, so nothing interpolates across the map.
- **Replay** = the `Scenario` (the map and the game setup, game options included) plus the `TickInputs` of every
  tick, empty ticks included, and state-hash checkpoints: `sc_sim::Replay` (serde) with `start`/`record`/`step`/`run`,
  and `record_checked`/`verify`/`run_checked` for the checkpoints (see Replays and golden hashes, which also covers
  the client's recording and playback). A replay keeps every command whole: `testing::every_command` lists one of
  each kind (every variant of `Command`, `OrderKind`, `Target`, `Ability`, `ProductionCommand`,
  `ConstructionCommand`, `DebugCommand` and `SpawnAt`), and the format tests round-trip it through the sim's serde
  and the client's file; its match stops compiling when a variant is added, so add the new one to it. The client
  must never submit a command with a non-finite number. The sim would reject it anyway, but JSON can't store NaN or
  ±∞, so the replay file would fail to load. Validate at the input layer.
- **Events**: `SimEvent`s (`Spawned`, `Removed`, `SuppliesDelivered`, `MoveFailed`, `Attacked`, `Hit`, `Died`,
  `PatchDepleted`, `ProductionQueued`, `ProductionRefused`, `ProductionCancelled`, `ProductionWaiting`, `UnitProduced`,
  `ConstructionCompleted`, `ConstructionCancelled`, `StructurePlaced`, `PlacementRefused`, `PlayerDefeated`,
  `GameOver`, `KindChanged`, `TransformCompleted`, `TransformCancelled`, `ResourcesDelivered`, `DeployFailed`,
  `TargetTooClose`, `SupplyStationEmpty`, `ProjectileLaunched`, `Blast`, `Missed`, `LocationTooClose`, …) are output
  only, for effects,
  sounds and
  UI. The sim never reads them, and tells each to every player; the client shows a player only what they may
  perceive of it (*Client architecture > Viewer events*). `ProjectileLaunched { shooter, target, from, to, lands_at }`
  and `Blast { at, radius }` (slice M4-O) are a projectile leaving and landing; a blast's damage follows as a `Hit`
  for each object it took, which the client draws nothing of, and `Died` (see *Client architecture > Visuals*,
  Combat effects). A TailDisjointed shot (slice M5-O3) is an `Attacked` on what it struck, its `to` the spot it
  locked, or a `Missed { attacker, target, from, to }` when nothing was there: `target` is what it was aimed at, none
  for the ground. `DeployFailed { drone, origin, problems }` (slice M3-F) is a Deploy that failed on arrival; the
  client's feedback line says why (`command_ui/feedback.rs`). `TargetTooClose { unit, target }` (slice M3-L2, design
  change 0182) is an Attack order that failed because its target is inside the MinRange of a unit that can't move,
  or can't back off further (design change 0204 point 3); the feedback line says "Target too close". So does it for
  `LocationTooClose { unit, at }` (slice M5-O3), the same failure of an Attack Ground at the location `at`.
  `SupplyStationEmpty { station, owner }` (slice M4-I, design change 0256 point 3) is a station whose last Supply was
  picked up, for each player gathering there; the client's notice says so (`hud/notices.rs`).
  `EnteredNetwork { unit, tunnel }` and `LeftNetwork { unit, tunnel }` (slice M4-C) are a unit leaving the map
  for a Tunnel Network and coming back with its id: no `Removed` and no `Spawned` go with them. `EnterFailed { unit,
  target, full }` is an Enter that ended without the unit going in, `full` when a building that holds units had no
  room for it (the feedback line says "No room inside"). `EnteredBuilding { unit, building }`, `LeftBuilding { unit,
  building }` and `Evacuated { unit, building }` (slice M5-F1) are the same for a building that holds units: in, out
  of its doorway by Eject All or a Release, and out inside its former footprint as it is destroyed, after its
  `Removed` (and after the `KindChanged` of a unit whose training was finished, design change 0349).
  `ReleaseQueued { producer, unit }` and `ReleaseCancelled { producer, unit }` (slice M5-F2) are a pool building's
  Release, queued and taken back by X or a refund; its trained unit is a `UnitProduced`. `BuildFailed {
  worker, kind, origin, orientation, problems }`, `BuildRefunded { worker, kind, refund }`, `SiteStarted { worker,
  site }`, `SiteCompleted { site, worker }`, `SiteCancelled { site, kind, refund }` and `LeftSite { worker, site }`
  (slice M4-E) are a Build that failed on arrival, a Build refunded, and a site's life; `LeftSite` is its worker back
  on the map with its id. `SiteBuilt { site }` (slice M5-E) is a site built from outside complete: no worker comes
  out, its builders were never in.
  `UpgradeStarted { tunnel, to, price }`, `UpgradeCompleted { tunnel, tier }` and `UpgradeCancelled { tunnel, to,
  refund }` (slice M4-G) are a Tunnel upgrade's life; one destroyed mid-upgrade has no event of its own.
  `ProductionWaiting { producer, reason }` (slice M3-P, design change 0143) is a producer whose unit at the front
  began to wait for room under the army cap `reason`, raised once per wait: the placeholder of 0143's notification.
  `Sim::events()` holds only the latest step's events, and `Sim::new` discards the events from the scenario's
  initial spawns. `Hit { attacker, target, damage, shield_damage }` says what a hit took of the target's HP and of
  its Shields (see *Combat > Shields*). `KindChanged { id, from, to }` (`Sim::change_kind`) says that an object
  changed its kind in place (a toggled unit's type at the press, design change 0198).
  The client's response (slice M3-E) reads the objects themselves, so a replay's jump or a loaded game gets it too:
  the object's visual is made anew with the new kind's mesh, size and tag (*Client architecture > Visuals*), it
  stays selected with the same id as its new kind and its marker takes its new shape (*Selection*), and the panels
  re-resolve its type. The Warp In's own interface (its ghost, Cancel on its panel) is slice M3-J1's; a converted
  Legionnaire (slice M3-B) gets the same response and nothing else.

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
  build compiles nothing else), the client's `game` (`tests/game.rs`). A new test file `tests/foo.rs` is a line
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
  (a patch, in groups, at most a face's 4 to 5, design change 0320) once the player has a completed Storage, orders
  its buildings in turn, each as one group Build from its crew (the Recruits on patches with no own Storage a tile
  away, or the `n` nearest the site) with each builder's Gather queued back after it, sends idle Recruits into its
  first Armory and trains Militants of them (`ProductionCommand::TrainFrom`), and records what it pays by tick for
  `Books`. `reference_field(faces)`, `crowd_field(recruits)` and `face_field(gap, recruits)` are the Cults economy
  check's layouts (design change 0346), and `cults::skirmish` plays `dev_skirmish` with a Cults line, alone or against
  a Syndicate one (the Cults' golden games).
- **Movement scenario tests** are `crates/sim/tests/movement_collision/` (passing, head-on, choke, group arrival,
  idle units, repaths, oscillation bounds, save and load in a crowd; in modules by topic: `passing`, `crowds`,
  `paths`), `group_destinations.rs` (the magic box; a
  group crowding its own spots, 0183),
  `queued_group_destinations.rs` (Shift-queued group orders),
  `movement_yield/` (who yields, a group order's units, the give-up schedule, occupied targets; 0169's 3 GU for a
  point and for a structure), `movement_crowds.rs` (blocks meeting head-on, a group through an idle block, dense
  blocks of any shape; the ignored crowd sweep), `movement_soak.rs` (200 units,
  random orders, 2 000 ticks; bounds on the share of failed moves), `movement_stuck.rs` (whose progress a waiting
  unit shares: a unit behind one whose legs begin again finds its way round), `invariants_overlap.rs` (I1's two
  exceptions and nothing more, on edited states) and the ignored `movement_perf.rs`.
- **Behaviour scenario tests** are `crates/sim/tests/behaviours_attack.rs` (the approach to a gap of Range, the
  chase, a target lost from vision, a target out of reach, a unit blocked behind its owner's attacker giving up),
  `attack_behind_friends/` (slices M3-G2 and M3-G3: the way round a firing line, and the Failure rule with none,
  design change 0223's Verification), `target_lost.rs` (design change 0200: Attack,
  Follow, a queue, AttackMove and idle units as the target leaves vision), `behaviours_idle.rs` (acquiring by priority,
  the leash and the return, Hold, Move doesn't engage, the auto-targeting switch), `behaviours_travel.rs` (Follow,
  static objects, AttackMove groups and the leash, a fight replayed and saved in its middle), `attack_move_leash.rs`
  (design change 0180: the leash measured from the route planned at the start after a fight, a bait at the leash
  taken once, a detour round a blocked route as the new PathReference), `attack_move_reach.rs` (0230: enemies across
  walls passed without a chase, a rejected enemy taken within weapon range, an identical enemy swapped in, judging
  again when the enemy moves a grid unit or the known obstacles change, the paths per unit and tick),
  `attack_move_own_movement.rs` (0243: judging again once the unit has walked a grid unit from where it judged,
  tick by tick and within the bound, none while it stands, a chase that ended beyond the leash, and 0243's
  Verification against a fresh Guard on the same spot), `target_priority.rs` (0179:
  the 45° steps of rotation, the step's bound, ties by id), the unit tests in `orders/tests/` (with test-only defs:
  a unit fighting back beyond its leash, artillery out-ranging it left alone, MinRange: the back-off, the immobile
  unit's failed order, the scans skipping enemies inside it; the steps themselves),
  `behaviours_patrol.rs`, `behaviours_gate.rs` (targets must be seen; a GDO player's remembered station, by its
  record), `behaviours_soak.rs` (120 units fighting under random orders of every kind, then without orders until
  those end; its checks are invariants that survive tuning: no dangling ids, no idle unit in reach of a visible
  enemy (but one whose scan held an enemy that died in that tick), every order but Hold, Patrol and Follow ends, and
  bounded shares of refused orders, failed moves and path searches; the ignored `soak_sweep` runs seeds 1 to 40 and
  fails if any does. An order ends within the slowest unit's walk round the map's four sides for each order a unit
  still has at the last order, its current one and its queue: slice M3-K2 found seed 5's Peacekeepers with three,
  each across both walls, settling 2765 ticks after it at full speed) and the ignored `behaviours_perf.rs` and `attack_move_perf.rs` (the reach rule: 100 Guards past 100 Drones, and past 30 Generators, across a river).
  `riot_shield/` has the Riot Shield's: toggle timing, each raised-shield effect, the gate, a save in the middle
  of a toggle and its replay, the orders kept through a toggle (`orders_kept.rs`, design changes 0150, 0151), and
  the target state (`target.rs`, design changes 0198 and 0202: a second press before a deferred raise begins, fast
  presses ending in their parity, an Attack taken while lowering and carried out once down, none while raising).
  Slice M3-R's:
  `default_order.rs` (the table of unit kinds against targets, design change 0136), `remembered_targets.rs` (design
  change 0139: the gate, each order on a remembered target that stands and that was destroyed unseen, the rally
  point, no leak in lockstep games), `known_from_the_start.rs` (design change 0159's Verification),
  `follow_itself.rs` (design change 0168's Verification and the Shift-queued step) and `production/drone_rally.rs`
  (a Drone rallied on an enemy unit). In the client, `tests/commands/remembered.rs` plays them through the mouse
  with the fog on.
  Measured (release, 500 units on 256 × 256; slice M2-J2, within 7 % of M1's in an interleaved A/B): 0.13 ms a tick
  with everyone idle or on Hold in sight of the enemy, 0.29 ms on average and 0.65 ms at worst in a battle of 250
  against 250. The release build's split into codegen units moves these by a few percent with code that isn't on
  their path: slice M4-S1's battle ran 3% slower with no new work in it until `World::pick` was `#[inline]` (the
  split had left its call in `travel::march` out of line); with one codegen unit on both sides it was within noise.
- **Resource collision tests** are `crates/sim/tests/resources_block.rs` (design change 0038's verification: units
  path around a crystal field, can't walk across a Supply Delivery Station, and walk through the tile of a patch
  removed by a debug command, standing in for depletion; `economy_mining.rs` has the real thing, below).
- **Path knowledge tests** are `crates/sim/tests/path_knowledge/` (the verification points of design changes
  0040 and 0042: a hidden wall of enemy buildings, a scouted building destroyed out of sight, a crystal field never
  seen, a patch gone in the fog; own structures; an unseen structure is open ground for plans and solid for
  units; lazy grids; saves with stale knowledge; in modules by topic: `plans`, `remembered`, `collision`,
  `grids`) and the ignored `nav_perf/` (`knowledge_rebuilds_256`).
- **Economy tests** are `crates/sim/tests/economy_mining.rs` (lumps every 48 ticks and the first 48 ticks after
  placement, the cycles under the power penalty and across a change of ratio, the loads down to the residual, a plate
  gone from a patch with crystals left or from a depleted one, by a debug removal or in combat, after which a unit
  walks through the opened tile, a patch mined out in the fog that another player assumes until it sees the tile
  (design change 0042), and the links in any order) and `economy_resources.rs` (the banks and
  `SetResources`, power and the ticks a building counts in, Unit Control, a mining game replayed and saved).
- **Construction tests** are `crates/sim/tests/construction/` (each building's frames at full power and the plate's
  680 ticks at 30/51 beside an unslowed Deployment Center, the refunds and their events, every refused command changing
  nothing, a group command in id order, a Place in its orientation with power, area and grids following, a refused Place
  for each problem of the check and one refused by an earlier command of its tick, destroyed constructors' refunds by a
  debug removal and in combat, a fuzz of the gate, a save and a replay; in modules by topic: `timings`, `refunds`,
  `gate`, `placing`, `saves`). The clean Barracks opening, which slice
  M2-D1 measured there first, is the economy oracle's.
- **The economy oracle** is `crates/sim/tests/economy_oracle/`, one folder of modules; its measured results are in
  `docs/economy_oracle.md`. **The Colonists** (slices M3-K1 and M3-K4, `colonists.rs` and `curve.rs`): the
  reference line on `testing::colonists::reference_field` (one Robo Portal training Drones without a break, every
  6th deploying) and the GDO's reference line measured the same way (what each delivered or mined over the 21 s
  centred on a time) against both columns of the table of `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`
  (`main.rs`'s `TABLE`) within ±15% (design change 0108) and its targets 1 to 4, and the saturation curve of the row by
  number of Drones against `factions/colonists/units/drone.md > Patch Occupancy`. Since design change 0236 the table and
  the curve are this measurement (slice M3-K3's), rounded; a figure out of the tolerance or a failing target is a
  question to the Designer, not tuned. **The GDO** (slice M2-J1, `played.rs` and `gdo.rs`): GDO openings on
  `maps/dev_skirmish.json` from its real start, played with `testing::build_order` (player 2 idle), against
  `factions/gdo/economy.md > Opening Economy Baseline`: plates first (the reference line: income at the table's
  times by its plates landing every 25 s, fully plated at about 2:30, the float at 1:30 and
  2:00), Generator first, the fastest clean Barracks (pinned: started in tick 769, ready at the end of 928), the
  all-in Barracks at −21, and the refunds. Income is the average over the 9 s (3 lump cycles) ending at a time; the
  tolerances on the design's trickle figures are named where they are used (a plate's first lump 3 s after it
  lands, a wait for money up to one cycle longer, a bank under a trickle's by less than a lump per plate), while
  costs, build times, lumps and the power penalty are checked exactly. `--nocapture` prints each line's timeline.
  **The Syndicate** (slice M4-L, `syndicate.rs`): the economy check of `factions/syndicate/units/agent.md > Gathering >
  Space Crystals` (design changes 0259 and 0294) on `testing::syndicate`'s layouts: one Agent's trip and the income
  of 2, 3 and 4 Agents (minutes 5 to 10 of a ten-minute game) at 3 to 6 GU within ±15% of the table, a test for each
  row (rows 0 to 2 are theoretical: the ignored `syndicate_table` measures them for the record); the ceiling, no two
  deliveries at a Tunnel less than 64 ticks apart, in every game; two stacked Tunnels at 3 GU doubling the field's
  intake; the opening at 3 GU from the Starting Conditions, its third Agent out within 0:34–0:46, both books
  balanced every tick; and the lowest one-base saturation. **The Cults** (slice M5-L, `cults.rs`): the economy check
  of `factions/cults/economy.md > Economy check` (design change 0346) on `testing::cults`'s layouts, played by a
  `Cults` line through the gate with the books balanced every tick. Scenario 1, ideal mining (one Storage per face):
  every target within ±15%, those out of it pinned (`OUT_OF_TOLERANCE`: the fifth Recruit of a face, the gain per face
  of a Storage a tile closer and its payback, and "slightly ahead by 2:48"). Scenario 2, the user's realistic opening
  (a second Hall before any other Storage, built by the Recruits on far patches, then a Storage per face): measured
  against the GDO and the Colonists at 1:30 and 2:48 and reported, with no target.
- **Game start tests** are `crates/sim/tests/game_start/` (slice M2-F1: each player's Starting Layout and banks,
  the Skirmish Default restriction and completeness, the layout rules of design change 0097, a layout plate off its
  patch refused by the game's rule, a pre-placed one by the same rule, two players' layouts in each other's way, the
  outdated flag, and `dev_skirmish`'s bases seeing and reaching their fields; in modules by topic: `start`,
  `layout_rules`, `skirmish_default`) and the unit tests of
  `scenario/starting.rs` (the table, and a layout made for other Starting Conditions). `colonists_start.rs` (slice
  M3-C) has the Colonists' start on `dev_skirmish` in either slot, its Drones clear of the node and the portal, the
  validation of every faction combination and of an incomplete layout, and Beacon Capacity (design change 0143):
  seven Drones queued and the seventh waiting for room, group production past the capacity, a fall in capacity, and
  the refined banks. `syndicate_start.rs` (slice M4-B) has the Syndicate's start in either slot (the objects, 100
  Space Crystals and 1 Supply, Tunnel Space 1/11, the camera on the Tunnel), the crystal hatch 1 grid unit from a
  patch (`tunnel::hatch_square`), the nine faction combinations, an incomplete layout, a layout Tunnel by the lake
  or the edge refused (0242), and an expansion over full Tunnel Space removing nothing. `cults_start.rs` (slice M5-B)
  has the Cults' start in either slot (the objects, 0 Space Crystals, the six Recruits of the Hall's lineage, Unit
  Control 6/20, the camera on the Hall), the Hall's seventh Recruit started at once and out of its bottom door at tick
  192, `dev_skirmish`'s layout as the design asks for it (the Hall's area on the map, Recruitable and uncontested; the
  Storage a 1-tile gap from the patches; the Recruits beside it, clear of the doors), the 16 faction combinations, an
  incomplete layout and one with a Recruit too many, and a layout's units belonging to its first Hall. `pairings/`
  (slice M5-R) starts every ordered pair of the four factions, 16, on every Skirmish Default map and plays it to a win
  (*Adding a faction*, item 19). The Tunnel Area's rules are unit-tested in `placement/tests/tunnel_area.rs`, Tunnel
  Space in `economy/army/tunnel_space.rs`, the geometry in `tunnel/area.rs`; a unit inside the network keeping its
  Tunnel Space in `tests/tunnel_network/enter.rs`.
- **The building armor rule** (design change 0264, `core/entities.md > Structure Type > Building armor rule`):
  `crates/sim/tests/building_armor.rs` holds every faction's buildings, from the defs alone, to "PointArmor n,
  FullArmor n × size", with an `EXCEPTIONS` list (empty; a stale entry fails), and every underground expansion to
  PointArmor 6 (`structures/README.md > Tunnel Expansions > High armor underground`). A new building passes it as
  it is added, or is listed with its reason.
- **Structure tests** are `crates/sim/tests/structures.rs` (each type's stat block, the Barracks in all 16 ways to
  rotate and flip it and its sides, ground units walking around a footprint, PointArmor on hits, a destroyed
  Generator freeing its tiles, debug spawns kept apart but for a plate on its patch, flat ground, the optional
  orientation in files) and `structures_vision.rs` (each type's sight, the ElevationModifier, an enemy Barracks
  remembered in the fog, a plate on a patch in maps and layouts, the targets that shoot back first, idle and on an
  AttackMove, a scan reaching a large footprint by its edge), and the ignored `structures_perf.rs` (see *Tick
  pipeline > Measuring a tick*). The placement tests are listed under *Placement and the build area*.
- **Colonists tests** are `crates/sim/tests/colonists.rs` (slice M3-A: each kind spawned for two owners, its
  structures blocking every ground grid, each kind's sight, the Sabre Legionnaire striking at contact without the
  ElevationModifier, the Rifle Legionnaire's Range with it, the Drone's 8 su per tick and its path around a Robo
  Portal). Their stat blocks against the design are in `defs/colonists/tests.rs`, and `Sim::change_kind` is
  unit-tested in `sim/kinds/tests.rs`.
- **Shields and Weapon Conversion tests** are `crates/sim/tests/shields.rs` (slice M3-B: the design's cases in a
  game, two hits in one tick, the DirectionalArmor on the overflow only, the regeneration from 0 to full in 40 ticks
  and a neutral object's, the dead gone first, the hash, saves and an older save, the Shields of a portal warping
  in following its maximum and regenerating up to it, and a cancel's fraction with no minimum) and `conversion/`
  (the 4 frames, HP and Shields kept; moving or idle; a mixed group; no cancel and no shelter; the orders going on
  with the other weapon; a save and a replay; `mid_attack.rs`: the gate in every attack phase, a Convert waiting for
  the Cooldown's end with the old weapon, alone and in a group in a fight, design change 0153; `target.rs`: the
  target form, design change 0198: 4 Sabres and 6 Rifles, a press on a unit waiting, converting and converted, fast
  presses ending in their parity). The resolution's cases are unit-tested in `combat/damage.rs`, the regeneration
  in `shields.rs`, the conversion's pieces in `conversion/tests.rs`, a change waiting for the Cooldown's end and a
  second press dropping it in `sim/modes/tests.rs` (0153, 0198). In the client: `tests/combat/` (the shield bar),
  `tests/commands/convert.rs` (R: the type and the group switched at the press, a deferred unit's too, the drawn
  form, design change 0198), `tests/commands/riot_shield/` (`groups.rs`: the groups at the press; `target.rs`: 0198's
  and 0202's Verifications through the panel and the mouse) and the InfoPanel's Shields line in `hud/info_panel`.
- **Gathering tests** are `crates/sim/tests/gathering/` (slices M3-H and M3-H3, one folder of modules by topic,
  split by slice M3-K2: one Drone's trip, pinned at 81
  ticks; two Drones arriving together; waiting, the lowest id mining next; an enemy Drone on the patch; Set
  Drop-off Location in the middle of mining and of a return; depletion and the Drones going on at another patch,
  or delivering and idling when no patch is left; the Drop Off order and the idling at the node; a lost Egress
  Node; interrupted mining; every invalid worker command refused and changing nothing; a Drone rallied on a patch;
  a game replayed and saved in its middle), `gathering_rules/` (slice M3-H3, one folder: a test per Verification
  point of design changes 0123 and 0127 to 0141 that `gathering/` doesn't cover, in modules by topic: `ends`,
  `loads`, `waiting`, `reach`, `knowledge`, `drop_offs`, and slice M3-R's `remembered`: a Gather on a patch out of
  sight, scouted or known from the start, one gone unseen, one placed later and unknown) and
  `gathering_soak.rs` (the four starting Drones for 5 minutes; 24 Drones for 5 minutes, every one delivering at
  least every 30 s, the saturated income, nothing lost; the income growing with the Drones up to saturation; 9
  Drones on 3 patches, once the smallest gridlock, with a steady income; ignored: `saturation_curve`, which prints
  the measurements) and `gathering_pass_through.rs` (design change 0106's Verification: two players' Drones on one
  row; an idle Drone parked in the strip, an enemy's and the gatherer's own; a Drone ordered out of a mining Drone,
  and solid once apart; a Drone stopped inside another, not pushed apart).
- **Warp In tests** are `crates/sim/tests/warp_in/` (slice M3-E: each building's WarpInTime and the Train it takes
  from the next tick; the interpolated maxima; damage carried through, taken before or during it (0110); the
  derived footprint, its corners, flips, turns and exit sides (0121); the Robo Portal's ring, blocked by a unit of
  either player, a patch or a structure (0118); every tile Visible, an unseen unit leaking nothing (0119); the
  footprint solid from the first frame and the Beacon's 2 × 2 after a cancel (0109); no drop-off at an Egress Node
  warping in (0116); the cancels' refunds, 75 % rounded down (56, 112, and 187 for the Robo Portal, 250 since
  design change 0167), and HP (0111); Beacon Capacity kept (0112); defeat and destruction, with no
  refund for either portal (0113, 0115); the gate against `Sim::warp_in_check` in every orientation, a save, a
  replay, and a fuzz of the gate; in modules by topic: `timings`, `interpolation`, `footprints`, `under_warp_in`,
  `cancel`, `gate`, `saves`). The arithmetic and the footprint are unit-tested in `transform/tests.rs`. In the
  client: `tests/client/visuals/` (a visual rebuilt and tinted), `tests/selection/panels.rs` (a selected Beacon
  through its Warp In), and the InfoPanel's line in `hud/info_panel` and `activity.rs`.
- **Tunnel Network tests** (slice M4-C) are `crates/sim/tests/tunnel_network/` (`enter.rs`: the walk into the entry
  path, what leaving the map ends, cleared orders, the gate and the tiers with a vehicle, a Tunnel gone or
  unsuitable, the Failure rule, the transit delay of design change 0245 and the regeneration of 0249; `eject.rs`:
  the paths in all 8 orientations (0248), 10 Guards streaming out and a blocked exit spot (0250), the unit inside
  longest, the reservation and a destroyed Tunnel's queue (0251), the gate; `production.rs`: the Hideout's Agent
  into the network, the rallies of 0252 and the shared queue of 0253, a Hideout without a parent; `crowds.rs`: 20
  Guards filing in (0246), entries and exits side by side at two Tunnels, 40 Guards in and out; `saves.rs`: saves at
  every stage and a replay, the store hashed and saved only while it holds a unit), each tick under the held
  store's invariants (`check_held`). `network.rs`'s unit tests have the geometry in every orientation, the delay and
  the regeneration; `sim/network/tests.rs` an air unit refused at every tier (a test def), a unit's id, HP and load
  through `take_off_map` and `put_on_map`, and a defeated player's network. In the client:
  `tests/commands/tunnel_network.rs` (a right-click on an own Tunnel is Enter; units that go in leave the selection
  and stay in their control groups, whose recall selects those on the map).
- **Holding tests** (slice M5-F1) are `crates/sim/tests/holding/` (`entering.rs`: ten Recruits filing in, timed, from
  a block and from every side, the eleventh's Enter failing for want of room, the QA run's ejected ten going round
  to side B and back in, the gate, the right-click, an Enter whose Armory goes, a Recruit walled in against side B
  that gets in only once it can reach the doorway (slice M5-F3), a site its Recruits build that no Enter gets into
  and a right-click on assists; `eject.rs`: Eject All streaming the pool out of side C one at a time, a Recruit that
  goes in during the stream, the gate; `evacuation.rs` (slice M5-F3): design change 0349's Verification, ten
  inside the footprint in the tick, then apart and idle, the same whatever stands round it (a Storage, an enemy, a
  unit in the doorway), a tile no Recruit can stand on passed over, the evacuees of a defeated player; `loads.rs`: a
  load kept inside and out by real gathering, and through an evacuation; `saves.rs`: saves at every stage and a
  replay; `control.rs`: a Hall's Recruit inside an Armory still in its Hall's usage, inside and evacuated, and an
  orphaned point once its Hall is gone), each tick checking `testing::dangling_references`. `holding/tests.rs` has
  the entry path in every orientation, its reach only in the doorway, and the kinds let in; `sim/holding/tests.rs`
  the Eject All mark hashed and saved only while set; `defs/tests/holds.rs` every exit opposite its entrance (0347
  point 1). The ignored `holding_perf.rs` times eight Armories
  cycling, then evacuated (*Object model > Buildings that hold units*).
- **Armory tests** (slice M5-F2) are `crates/sim/tests/armory/` (`training.rs`: Q pays 20 and a Militant comes out
  of side C 32 ticks later toward the rally point, first in first trained, X's refund, the gate, Q to three Armories;
  `release.rs`: design change 0337's Verification, X taking a Release back, a Release queued like any order;
  `eject.rs`: Eject All leaving the reserved units inside, the order units are ready in; `capacity.rs`: 0347's
  Verification, the load a trained unit drops and a released one keeps; `destruction.rs`: an Armory destroyed
  mid-training, and one holding a trained Militant waiting for its doorway, which comes out a Militant, unrefunded,
  with the rest refunded (slice M5-F3); `control.rs`: the Unit Control unchanged through training, and a Militant
  trained of a destroyed
  Hall's Recruit an orphan; `saves.rs`: saves at every stage and a replay), each tick under the invariants, R1 among
  them, and checking `testing::dangling_references`.
  `sim/producers/pool/tests.rs` has a hurt Recruit's damage, a vehicle's full HP, a unit of two Recruits and an
  orphan's point (test defs); `production/tests.rs` the queue's entries. In the client:
  `tests/commands/armory.rs` (0337's and 0348's verifications, Q, X, Eject All and the right-click rally),
  `command_ui/interface/tests/cults.rs` (the panel as data) and `hud/info_panel/activity/tests/armory.rs`.
- **Cults construction tests** (slice M5-E) are `crates/sim/tests/cults_construction/` (`flow.rs`: four Recruits in
  80 frames paid once, five from one side spread along the wall (0339), two at twice the rate, one alone, a Shift
  chain, builders pulled off and back by Assist, the right-click; `refunds.rs`: two of three re-ordered and then the
  third (0340), killed, after the start, a queued building its last holder drops (0340), X, the queued building after
  X; `footprint.rs` (slice M5-J1, design change 0362): nine Recruits under a Hall walk off and it starts, a site
  waiting for one walking off its middle, an enemy and an own Recruit not on the order each failing it, refunded once,
  a Recruit holding it queued holding it back until it walks off; `gate.rs`: the Armory's Storage, Shift-clicks each
  paid, Assist's refusals; `crowds.rs`: twenty-four on a Storage; `hall.rs`: a built Hall's lineage, gatherers taken
  off and back), each tick checking a frame per builder at work. `sim/spot_work/tests.rs` (test defs): the wall full
  and those waiting further out moving in, oblong ABAC footprints, 2 × 3 and 2 × 4, at four rotations, and a walk
  ended on the footprint looking again (0362); `placement/wall/tests.rs` the wall, its search and the way round;
  `spot_work/tests.rs` the claim and the reach.
- **Repair tests** (slice M5-N8) are `crates/sim/tests/repair/` (`rates`: one to four Recruits on an Armory, linear, a
  crystal every 3 HP, each object by its own terms; `bank`: the bank running out, the queued orders going on, a Repair
  given with no crystals; `agents`: a Tunnel for free at Tiers 1 and 2, a Laser Turret lowered and raised, one left
  neutral; `refusals`: the gate's; `regeneration`: a Hideout to full in 100 s, under AoE fire, none once its Tunnel is
  gone, none for a turret), `sim/repairs/tests.rs` (test defs: a vehicle followed as it drives off, a glider refused by
  its movement), `repair/tests.rs` (the terms, who repairs what, the payment's steps) and `tests/default_order.rs`'s
  Repair rows. In the client: `command_ui/resolution/tests/repair.rs` (the right-click and D as data),
  `tests/commands/recruit_ui/repairing.rs` (a Storage repaired by a right-click, an Armory entered by one and repaired
  by D, D Inactive at full HP) and `tests/commands/agent_ui/repairing.rs` (a turret repaired, a Tunnel entered by a
  right-click and repaired by D, an expansion refused).
- **Beacon Deployment tests** (slice M3-F) are `crates/sim/tests/deploy/` (the walk to the footprint's centre and the
  start on arrival, the 48 frames, the HP from 20 to 120 and damage carried through (0110), Beacon Capacity 10/10 to
  9/10 to 9/15 and a cancel to 11/10 (0112), the cancel's fractions of HP and Shields and its place (0109, 0111), the
  Drone's Shields kept as it starts, the load lost or returned (0114), defeat (0113), a destruction, the footprint
  claimed, auto-targeting taking the footprint, Shift, a save and a replay; in modules by topic: `arrival`, `cancel`,
  `loads`, `beacon`, `saves`) and `crates/sim/tests/deploy_gate/` (one
  Drone per Deploy; the gate on what the player knows, remembered buildings, hidden units and never-explored ground
  (0120); failures on arrival for a structure, a unit of either player, a patch, a Drone walled off, and a Drone blocked
  while overlapping that deploys (0117, 0118); only Cancel on a deploying Beacon (0116); the reservations and
  `Sim::pending_builds` (0125); a fuzz of the gate and the arrival; in modules by topic: `gate`, `arrival`,
  `pending`, `fuzz`). Unit tests: `deploy.rs`'s, `placement/tests/`
  (reservations, `Mode::Known`) and `transform/tests.rs` (the load hashed).
- **Agent-built Tunnel tests** (slice M4-E) are `crates/sim/tests/builds/`, under `check_sites` (each site's worker held
  in it and nowhere else) after every tick: `flow` (the order paid at once, the site's HP and completion, the network
  and an Eject, the site's one command, the load kept, what enemies see and target, a cancel, a destruction, the
  defeat), `price` (0233's verification, the refusal, every refund, a price of 0, and the books: a property test over
  random orders, Shift-queues, cancels and replacements, with a fixed seed list; since slice M4-Z it also plays Tunnel
  1's expansions, its Laser Turrets among them (an Agent's turret Builds until slice M5-Y, design change 0359), and
  balances both banks every tick by `testing::build_order::Books` and `testing::syndicate::SupplyBooks`, each refund
  counted in its own resource by `build_order::refund_in`) and `rules` (the overlap rule by knowledge and by truth, the
  Drillable area, 0240's hidden expansions for the three factions, a unit on the site, the lingering ghosts'
  reservations). Unit tests: `construction/site.rs`'s (the maximum, who builds what).
- **Losing Tunnel tests** (slice M4-H) are `crates/sim/tests/losing_tunnels/`, under `check` (no reference dangles,
  each site's worker is held in it, and only a neutral expansion decays) after every tick: `neutral` (the only Tunnel
  falling in combat: the Hideout neutral, seen and remembered by the enemy, the defeat, the Agents neutral, the network
  destroyed, 4 HP a second and the death in the 1600th tick; one of two Tunnels: work lost, the queue refunded, the
  infantry inside kept), `stranded` (the vehicles at the last Tier 2 Tunnel, a site is no exit, the free rebuild with a
  debug-spawned Barracks), `attack` (auto-targeting and the right-click leave it, an explicit Attack hits it) and
  `reclaim` (at completion and never while a site, the enemy's view and the events of the tick, an enemy reclaiming
  and the old owner's record, partial cover blocking cells and taking Tunnel Space) and `saves` (saves at every
  stage, the replay, the decay hashed and saved only while set). Unit tests: `sim/tunnels.rs`'s
  (the decay's arithmetic). In the client, `crates/game/tests/fog_memory/neutral.rs` (the neutral colour, the falling
  HP bar, the record out of sight).
- **Expansion tests** (slice M4-F) are `crates/sim/tests/expansions/`, under O1 after every tick: `placing` (the
  expansion method by knowledge and by truth, 0290's units), `building` (paid at the click, Tunnel Space, 400 ticks
  with the damage kept, the second Hideout training, one operation, Eject meanwhile), `cancelling` (150 of 200 by
  either X, the site gone otherwise) and `hidden` (the enemy sees nothing; a Warp In over it red at once). The test
  kinds of another cost or Tier Requirement are `sim/expansions/tests.rs` (only a unit test can change a def), the
  ExpandMenu's list `defs/syndicate/tests.rs` (against the Tech Tree table, 0291). In the client,
  `command_ui/interface/tests/expansions.rs` (the menu's letters and greying, the placement, the clamp) and
  `crates/game/tests/commands/expansions.rs` (W, the ghost, the click, the bank, the operation and both X's).
- **Upgrade tests** (slice M4-G) are `crates/sim/tests/upgrades/`, under O1 after every tick: `costs` (2 then 4, 3
  then 6, a Tier 3 Tunnel in the Tier 2 price, two started together, 0232's refunds in either order, and the books
  over random starts and cancels), `blocked` (an own neighbour, an enemy's site and Tunnel by the truth, Mountain and
  Water, every map edge at 3 and 4 tiles, the gate's order, and an own lingering ghost that doesn't block and fails on
  arrival), `during` (the reserved area against the owner's ghost and an enemy's, 0249's vehicle, the X, the Tunnel
  destroyed) and `completion` (480 frames, the HP, Tunnel Space, a reclaim, saves and the replay); 0231's 500/600 is
  `sim/upgrades.rs`'s unit test. In the client, `command_ui/interface/tests/syndicate.rs` (Q and its reasons),
  `hud/info_panel/hint.rs`'s (the price), `hud/info_panel/activity.rs`'s (the line),
  `placement/tunnel_areas/tests.rs` (the reserved area among the own areas the rings count),
  `crates/game/tests/commands/tunnel_upgrade.rs` (Q, the progress, X's refund, the larger area, what blocks it) and
  `crates/game/tests/commands/agent_ui/ghost.rs` (the ghost red over the reserved area, the rings red over it, the
  current area while selected).
- **Game end tests** are `crates/sim/tests/game_end/` (defeat by a removal and in combat, plates counting; no
  defeat without a structure ever owned; the units made neutral and idle; every command of a defeated player
  refused; a 1v1, three players defeated one after the other, and the draws of 0163: two players defeated in one
  tick, and the last two of three; no producer handed over; a save across a defeat and the end, and old saves; in
  modules by topic: `defeat`, `on_defeat`, `end_of_game`, `saves`) and,
  in the client, `crates/game/tests/game_end/` (`outcome`: the banners, Draw for everyone, the stop; `recording`:
  the saved recording, QA's end; `results`: the results screen of 0164 in a three-player game, under the debug
  player switch and in a replay, the draw seen from it, the spectator's fog) with the unit tests of `hud/results.rs`
  (who leaves, allies included, which the sim can't make yet). The fuzz games of `robustness.rs` give each player a
  base the stream never removes, so that a debug removal doesn't end them; a fight still can (a Grenadier, or a Laser
  Turret in its owner's Tunnel Area, takes a base), and then the tick after the end checks that every command is refused
  and changes nothing, and the stream goes on in a new game (slice M4-M: any seed does, none is hunted for). One seed
  is one sample, which the next kind or rule reshuffles: **the ignored `robustness::seed_sweep`** (slice M5-Z1) plays
  the first 256 seeds' streams and lists every one that panics, with its message (each of the fuzz's own checks names
  what broke and the tick). A slice that changes the sim runs it before it reports: `cargo test -p sc_sim --release
  --test sim robustness::seed_sweep -- --ignored --nocapture`, about 4 s. At M5-Z1 all 256 pass, and 9 of them end a
  game, which the fixed seed no longer does.
- **Determinism tests** compare `Sim::state_hash()`, an FNV-1a 64 hash of the whole state except the events. Run the
  same scenario and inputs twice, or play back a `Replay`, and compare the hashes (`determinism.rs`, with the replay
  checkpoints). Every state type's hash impl destructures it (`hash.rs`), so a new field doesn't compile until it is
  hashed, and `sim/tests/hash.rs` changes each field in turn and checks the hash changes. The golden games (`golden/`) pin the hashes from build to build: see Replays and golden hashes.

## Client architecture

M0 builds this skeleton, so follow it. The art direction is 2.5D pixel-art sprites in the style of StarCraft 1 and
Age of Empires 2, with the details still open in Q0001. Until that art exists, everything is a placeholder shape laid
out by the same rules.

**Kinds** (`kinds.rs`, slice M4-R2): what the client knows of each object kind beside its def, one module per kind
(`kinds/<faction>/<object>.rs`, named as the design's file of the object is: `kinds/gdo/peacekeeper.rs` holds both
the Peacekeeper's types, `kinds/map/` the map's objects), behind one registry, `kinds::of`, an exhaustive `match`
with one line per kind. A `ClientKind` holds the kind's label (`kinds::short_label`; a Tunnel's names its tier,
`kinds::object_label`), its look (`object_render::Look`: a unit's body shape and whether it carries its label, a
structure, or a map resource's own shape), its shots' style (`combat_effects::ShotStyle`, for a kind with a weapon)
and its panel template (`command_ui::interface::templates::PanelTemplate`). Its name and numbers are the def's. The
drawing code stays in `object_render/` and `combat_effects/`, and nothing else in the client matches on kinds for
these.

**Sim driver**
- The `Sim` lives in a Bevy `Resource`. That is a client newtype, because `sc_sim` can't derive Bevy traits. The sim
  is stepped only in `FixedUpdate`, whose timestep comes from `sc_sim::TICKS_PER_SECOND` (`Time::<Fixed>::from_hz`).
  One FixedUpdate run is one tick. Nothing else steps the sim, and no game logic reads frame time.
- Player input becomes `PlayerCommand`s in a pending-commands `Resource`. The stepping system drains it into that
  tick's `TickInputs`, which is what a replay records.
- Read `Sim::events()` in the same system, right after `step`, and forward them, for example as Bevy `Message`s. The
  next step clears them, and FixedUpdate can run several times in one frame.
- `SimControl` says whether the step may run: `paused`, `step_once` (one step while paused, then it clears itself)
  and QA's `hold_at`; never once the game is over (`SimControl::may_step`, `Sim::outcome`: "the game stops",
  `core/game_rules.md > End of Game`), so a finished game stands at its last tick. `Pace` says how fast, changeable
  at any time: `Scaled(f)` runs `f` × 16 ticks per second of real time through `Time<Virtual>`'s relative speed,
  `Max` (`--fast`) as many ticks per frame as fit in `FAST_FRAME_BUDGET`, drawn exactly at their ticks. A change
  applies in `PostUpdate` (`apply_pace`), so the next frame's time update already goes at the new pace, and at a
  scaled pace the fixed loop never owes more than a frame's worth (`limit_fixed_loop`: 250 ms of scaled time), so
  leaving `Max` runs no burst of ticks. A replay's playback controls both (see Replays and golden hashes).

**Visuals**
- The sim is the source of truth. Visuals are separate entities, each linked to its object by an
  `ObjectVisual(ObjectId)` component. Their `Transform`s are derived from the sim every frame and are never read back
  for gameplay.
- Create and despawn visuals from sim events, or by diffing ids against `Sim::objects()`. Build the initial set from
  `Sim::objects()`, because scenario spawns emit no events.
- **Interpolation**: draw each object at `prev_pos.lerp(pos, a)`, where `a` is `Time<Fixed>::overstep_fraction()`
  read in `Update`. Interpolate facing along the shortest arc.
- **Coordinates** (`coords.rs`): 1 world unit is 1 su, in three spaces. **Ground** is the sim's (+y south).
  **View** is where a ground point is drawn: `drawn_position(map, ground)` shifts it up the screen (north) by
  `lift(height) = height × HEIGHT_OFFSET_PER_LEVEL` GU (`core/art_direction.md > Elevation Display`; 0.25 GU per
  level, a placeholder until art tuning). **Bevy world** is view space with **`bevy_y = -view_y`** (`view_to_world`),
  because Bevy's +y points up. Convert only through these helpers, never ad hoc. A sim facing θ is a Bevy rotation of
  −θ about z.
- **Picking**: every screen → world conversion ends in `coords::ground_under(map, view)`, the inverse of
  `drawn_position`: the ground point drawn at that view point. Where terrain overlaps itself on screen (a cliff or a
  hill in front of the ground behind it), the frontmost, i.e. southernmost, surface wins. It solves column by column
  (at a fixed x the drawn y is linear across a tile row) from south to north; beyond the map's edges the ground
  continues flat. The input layer's `Pointer::ground` is the cursor's ground point (`camera::cursor_to_ground`):
  hover, clicks, the editor's brush, ghosts, markers and object picking all use it.
- **Draw order** follows StarCraft 1. The layers (`coords::DrawLayer`) are terrain < ground decals < ground objects
  < unit decals < ground units < air < overlays, each with its own z band. Within a layer, z grows with **ground** y,
  so southern objects draw in front. Each sprite is anchored at its ground position, the sim `pos`, and that point
  drives its y-sort; it is drawn at its height (`ground_translation`; a tile-based object at its footprint's mean
  corner height, `footprint_height`). The tile-based objects (structures, map resources, the records of remembered
  ones) are ground objects, sorted among themselves (a plate over its patch: `structures::BASE_Z`); units are drawn
  above them whatever the depths (`object_render::draw_layer`, slice M3-U), so a unit crossing a footprint, as one
  leaving a portal's energy doorway does, is never hidden under it (`tests/selection/draw_order.rs`). Selection rings
  are unit decals, between the two; frames lie with their structures; ghosts, HP bars, path and command indicators are
  overlays, above everything. So are the units' tags, just above the command indicators and still y-sorted by their
  units (`units::tag_z`, slice M3-J3), so that no marker on a unit hides its label; a structure's tag stays with its
  structure, under the units that cross it.
- **Terrain** (`map_render.rs`): one mesh. A flat tile is a quad, a sloped one four triangles around its center (at
  the corners' mean height), each shaded by Lambert's law towards `LIGHT_TOWARDS` (a placeholder: from the north-west,
  45° up), so flat ground keeps its colour. A non-Traversible tile at least `CLIFF_MIN_SLOPE` levels steep reads as a
  cliff face: rock-coloured, with a stratum line at every level. Every vertex's z grows with its ground y, so with the
  depth test southern terrain overdraws the terrain behind it. The grid (G) follows the surface in the terrain's z
  band, slightly in front, so terrain in front hides it too. Both meshes are rebuilt when the sim's map changes.
- **Units** (`object_render.rs`, `object_render/units.rs`; the systems in `object_render/systems.rs`) are placeholder
  shapes until the art (Q0001): a shape per kind, its look's (*Kinds*: the Peacekeeper round, the Guard and the
  Legionnaires square, the Drone an octagon), of the silhouette's size,
  in the owner's colour (`player_color`, the editor's slot colours), with a dark rim and a light tick along the
  facing. Only a round body turns with the interpolated facing (`body_turns`). A square or an octagon stays
  axis-aligned like its collision box, and only its tick turns (a `FacingTick` child, `has_tick`). Structures and map
  resources never turn, whatever their facing (a structure keeps a unit's facing through `Sim::change_kind`). The
  Colonists' units, the Peacekeeper's two types, the Agent and the Grenadier carry a `UnitTag` child, a `Text2d` of
  the type's short label (DR, SL, RL, PK, PS, AG, GR), because the Legionnaire's two forms share a silhouette, the
  Peacekeeper's two types a body (slice M3-J3), and the Agent and the Grenadier the Guard's square (slices M4-A and
  M4-U). A tag's local z lifts it into the
  overlays band (`units::tag_z`, see Draw order), and it stays
  upright on a body that turns: `units::upright_tags`, chained after `place_object_visuals`, gives it the inverse of
  its visual's rotation. Every type's short label (`kinds::short_label`) is its own, and none is a
  planned unit's or building's button label (`Planned::label`: the Scout's is "ST", the patch's "SC"); a unit test
  checks it. One mesh per `MeshKey` (kind, owner, a structure's size as placed, and
  whether it is transforming) is shared, and one tick mesh per kind. A visual whose object is drawn otherwise now
  (its `MeshKey` changed: a kind changed in place, `SimEvent::KindChanged`; a transformation began or ended; a
  defeated player's unit turned neutral) is despawned with its children and made anew (`ObjectVisuals` keeps the
  key each was made from). A transforming object, a building being warped in, is tinted half way to a pale cyan
  (`transforming_color`, a placeholder until the art). The map resources' shapes are in
  `object_render/resources.rs`. A unit is drawn as the form it has (`object_render::drawn_kind`, *Object model >
  Type and form*): a Legionnaire whose conversion is pending keeps its old form's look and tag, its selection group
  its new type's. A unit that carries a Riot Shield has a `RiotShieldVisual` child: a pale arc in
  front of it, hidden while the shield is down (a deferred raise's too), growing to its full size as it goes up. It is `Visibility::Hidden` or
  `Inherited`, never `Visible`, so the fog that hides its unit hides it too. A worker has a `LoadMarker` child per
  resource it gathers (`object_render/load.rs`, `load::carried`; slices M3-J2 and M4-J1, placeholders): a crystal
  shard in the crystals' cyan with a dark rim at the top right of its body, and on an Agent an amber supply crate
  there too, each shown the same way while it carries a load of its resource (`carries_of`), to every player who sees
  it.
- **Structures** (`object_render/structures.rs`) are placeholders too: the footprint as placed, in the owner's colour
  inside a dark rim, drawn `INSET_SU` (8) inside its edge so that a selection frame shows around it, and over a map
  resource's placeholder (a plate covers its patch). A `StructureTag` child, a `Text2d` of the type's short label
  (`kinds::short_label`), names it; the fog hides it with its structure. A remembered structure
  (`remembered.rs`) looks the same, in its owner's colour, dimmed, its tag too. **Underground structures** (slice
  M4-A, the Hideout; `structures::underground`) are translucent outlines, a rim and a faint fill in the alpha-blended
  `underground` material, under the units (`artifacts/director/m4-plan.md` §3.3); the sim hides them from other
  players (*Vision > Hidden objects*), so only their owner and the fog-less views draw them. **The Tunnel's look**
  (`object_render/tunnel.rs`, design changes 0224, 0248 and 0251) follows its tier and its sides as placed
  (`TunnelLook`, part of the `MeshKey`): Side A's two paths, ragged at Tier 1 and paved from Tier 2, a hatch on Sides
  B (crystals' cyan) and D (Supplies' amber), none on C, and the Tier 3 air doors over the paths' halves; its tag
  names its tier (T1 to T3, `kinds::object_label`), to enemies too (design change 0258), a record's the tier last seen.
  **A site** (`Object::is_site`, slice M4-J1; `object_render/site.rs`, a placeholder) is drawn by being one, whatever
  its kind (`MeshKey::site`), on the surface or underground (an expansion's outline, slice M4-F): its footprint tinted
  half way to a pale sand and braced at its corners by a scaffold (clear of its tag) instead of its finished look,
  with a progress bar near its south edge (lower and thinner on a roof one tile high, a Laser Turret's, clear of its
  tag), a dark track in its mesh and a `SiteProgress` child stretched to its share done (`place_site_progress`,
  `Progress::ratio`), to everyone who sees it. A record of one is drawn as a site too (`MeshKey::of_record`, slice
  M4-I2), dimmed, its bar fixed at the progress last seen (`LastKnown::site`, `SiteProgress::remembered`, which
  `place_site_progress` leaves alone): the site may have grown or been finished since. **A structure that attacks**
  (`Look::Turret`, `object_render/turret.rs`; the Laser Turret, slice M4-S1, a placeholder) is a structure's footprint
  with a round head on its roof and, once finished, a `TurretBarrel` child turned by the object's facing
  (`place_barrels`, interpolated as a unit's facing tick), which the sim snaps to what it fires at; a record of one
  seen raised keeps the head and no barrel. **Always a pop-up** (slices M4-S2 and M5-Y, placeholders; design change
  0360): fully lowered (`MeshKey::lowered`, `popup::is_lowered`) it is an underground structure's outline in the
  blending material, with neither head nor barrel, so its owner sees the ground and the units over it and `Sight`
  shows an owned one to nobody else; from its first rising frame it is the raised body, its barrel facing its target
  and growing to its length with `PopUp::height`, and shrinking as it retracts. A neutral one, down for good, shows to
  everyone as its outline in the neutral colour, and so does its record (`MeshKey::of_record`,
  `popup::remembered_lowered`), blended as an underground structure's record is (`object_render/remembered.rs`).
  **A unit with a turret** (slice M5-U1) carries the same `TurretBarrel` on its body, with no head, turned to its
  turret's facing (`sc_sim::combat::turret_facing`); the sim keeps no turret facing of the tick before, so the
  barrel's `TurretTrack` remembers the last two ticks it saw and interpolates between them, and on a round body,
  which turns with the unit, the barrel turns by the turret's facing less the body's.
- **Path overlay** (`path_overlay.rs`, P or `--show-paths`): each unit's remaining path (`Mover::remaining_path`) as
  a thin line from where the unit is drawn, cut into short pieces lifted by the terrain's height, in the overlays
  layer, and a frame around each unit in the colour of its move's status (`status_color`: green moving, amber held
  up, red failed, blue arrived; placeholder colours). Its one mesh is rebuilt every frame while it is shown.
- **Combat effects** (`combat_effects.rs`, its mesh in `combat_effects/draw.rs`; placeholders until Q0001): a tracer for
  each `SimEvent::Attacked` and each `SimEvent::Missed` (`TRACER_TICKS`; a TailDisjointed shot's, the Gunner's, to the
  spot it locked, hit or miss, `Tracer::target` none for a miss: slice M5-O3), drawn only along the parts of its path over the local player's Visible
  tiles (`shot_spans`, through `visible_spans`, which cuts it at every tile edge it crosses; design change 0177: a shot
  from the fog shows from the fog edge, one all in the fog not at all; its attacker is revealed by the sim), **with no
  exception** (design change 0293, answering Q0083 point 4: every shot, tracer or projectile, whoever fires it and at
  whatever, a revealed attacker's at the player's own objects too; it replaced design changes 0206 and 0244, Q0056 and
  Q0075, which drew those whole, from the attacker to the target); a
  tracer's look is its weapon's (`ShotStyle::of` on the form that fired, its kind's `ClientKind::shot`; design change
  0207: the Syndicate's lasers, the Guard's and the Laser Turret's, red), HP bars over damaged and selected objects (a
  unit's clear of its silhouette's corners, a structure's along the top edge of its footprint and as wide as it), with a
  shield bar right above for an object that has Shields (`systems/combat.md > Shields > Display`, design change 0148;
  `SHIELD_FILL`, a placeholder colour), shown too while only its Shields are down, and a fading marker for each
  `SimEvent::Died` (`DEATH_MARKER_TICKS`), never for an object that was hidden from the local player
  (`shows_death`, the `Hiding` its death carries; plan decision T52: "nothing is drawn for a hidden victim", as a
  Hideout under a blast; no hit is drawn for any victim). Each effect keeps its event's `Sighting` (*Viewer events*):
  the drawing code only cuts a path to the Visible tiles. Slice M4-O adds the projectiles in flight
  (`combat_effects/projectiles.rs`, `Flight`, from `SimEvent::ProjectileLaunched`): each along the straight path to
  its locked point at the pace of the ticks, lifted by a cosmetic arc (`ARC`) over its shadow, shown while its point
  on the ground is over a Visible tile (`shows_flight`; `core/vision.md > Attacks from outside vision`), whoever
  threw it (design change 0293), in its thrower's style as a tracer is in its shooter's (`Flight::weapon`,
  `ShotStyle::of`; plan decision T52: the Grenadier's `ShotStyle::Grenade`, a grenade with its fuse lit, slice M4-U);
  and the
  blasts (`combat_effects/blasts.rs`, from `SimEvent::Blast`, `BLAST_TICKS`): a ring the size of the area, shown
  while the tile under its centre is Visible (`shows_blast`). One alpha-blended mesh in the overlays layer, rebuilt every frame while
  anything shows; effects age by the sim's tick and the render alpha, so screenshots stay deterministic.
- **Fog of war** (`fog.rs`, `core/vision.md`): `Sight` (`fog/sight.rs`) says what the local player is shown: its own
  objects, and everything on its Visible tiles (`Sight::sees`); on Explored tiles the terrain and the player's records
  of the structures and resources it saw there, in their last-known state, never the live objects (`Sight::remembered`,
  see Vision); on Unexplored tiles nothing (a map object known from the start, design change 0159, shows once a tile
  under it is Explored). Object visuals it doesn't see get `Visibility::Hidden`; the records get
  visuals of their own (`object_render/remembered.rs`, `RememberedVisuals`: one entity per record shown, dimmed
  like the Explored ground, a placeholder; synced when the sight changes). Tracers, death markers, HP bars,
  projectiles and blasts, the
  path overlay, the selection (`selectable_by`) and with it what a click or the pointer targets, the InfoPanel and
  the command indicators ask it too. The fog layer (`fog/layer.rs`) is one mesh that follows the terrain's surface tile
  by tile, in the terrain's z band `FOG_Z` in front of it (so no cliff stratum shows through), and one texture of
  one black texel per tile (`UNEXPLORED_ALPHA`, `EXPLORED_ALPHA`, clear when Visible), sampled linearly. The texture
  is written only when `Sight::revision`, the local player or the sim changes, the mesh only with the map.
  A `Sight` looks through the union of the visions it watches (`Sight::watching`): a tile in the best state any of
  them gives it, an object out of sight of all of them as the latest record any of them keeps. A player watches
  their own vision, and a defeated one their allies' too (`Sight::new`, `Sim::allies`; design change 0102: never a
  full map reveal); `Sight::revision` holds whom it watches. Without teams a defeated player watches their own
  vision alone, which has no Visible tile left once the fog delay of their last tiles is over.
  `FogOfWar` turns it off for QA (`--no-fog`, the debug key F3, which a replay's playback takes without debug
  commands too): display only, the sim computes every player's
  vision anyway. The editor shows no fog; its play-tests do.
- **Viewer events** (`viewer_events.rs`, slice M4-V): **one filter between the sim's events and every view of them.**
  The sim tells every event to everyone. `filter_events` (in `SyncSim`, set `FilterEvents`) alone reads that stream,
  besides the QA state dump (tooling). It gives each event an `Audience` as it arrives, and fills the frame's
  `ViewerEvents`, which every consumer reads instead. A consumer asks how a viewer perceives each event: `Whole`,
  `Clipped` (a path seen only over the viewer's Visible tiles, which the drawing code cuts), or `Nothing`. The
  audiences:
  - `Owner`, the owner alone: feedback, refusals, notices, refunds, production, construction and upgrades;
  - `Object`, whoever `Sight::sees` the object;
  - `Seen`, by place: every shot and projectile is clipped for every viewer, the shooter's owner too (0293); a blast
    shows where its centre is Visible; a death shows to its owner and where it is Visible, never to a player it was
    hidden from;
  - `Removal`, whoever was shown the object where it went, never a player who remembers it out of sight;
  - `Everyone`: defeats and the end of the game.

  The fog off shows everything but a player's own business. Whom an object is hidden from is `Hiding::of`'s alone
  (the hook, *Vision > Hidden objects*). A new `SimEvent` doesn't compile until `Arrival::audience` has its rule, and
  fails the table test (`viewer_events/tests.rs`) until it has a row (`tests/table.rs`). Never read
  `SimEventMessage` elsewhere.
- Never make one entity per tile. Draw the terrain as one mesh or texture, or a few chunks; the fog of war is one
  texture, never per-tile materials.
- Create mesh and material handles once and cache them in a `Resource`. Never create assets per frame, per object or
  per indicator, because the old prototype ran out of memory doing that.

**Camera**: the camera is orthographic and shows exactly 28 GU horizontally (`core/camera.md > Zoom`; see
`ScalingMode::FixedHorizontal`). Its viewport is the area between the UI bars (`ViewportInsets`: the top and bottom
HUD bars in the game, the editor's panels in the editor), so the visible height follows from the aspect ratio.
`CameraView::center` is a **view** point; to look at a ground point, center on its `drawn_position`. It pans with the
arrow keys and at the screen edge, with bindings and speed pending in Q0002, and it stays clamped to the map's drawn
area (`coords::drawn_bounds`, from its highest-drawn to its lowest-drawn corner, so raised ground at the north edge
stays reachable). The middle-mouse grab (`core/camera.md > Panning > Middle-mouse grab`, `camera::GrabPan`) keeps the
view point grabbed on the press exactly under the cursor while the button is held, and stops on the release; the
game and the editor share it. Centering on something snaps instantly (`core/camera.md > Panning`). The camera starts
at the local player's slot's camera start (`Slot::camera_start`), or else on the primary starting structure
(`core/camera.md > Starting Position`): the GDO's Deployment Center, the Syndicate's Tunnel, the Cults' Recruitment
Hall or the Colonists' Robo Portal, where the player's Starting Layout places it (`Scenario::primary_structure`).
Without one (a Custom map's layout may lack it) the slot's
start position stands in, and the map centre for a player without a slot (`camera::start_center`, shown through
`start_view`). Only the editor zooms (`CameraZoom`). Under the debug player switch (F2) each player has a camera of
their own (`camera::PlayerCameras`, `camera/players.rs`, slice M3-J3): when the local player changes within a sim,
the camera keeps the view of the one before and goes to the new one's, at their `start_view` the first time; a new
sim (`SimRes::generation`: a play-test, a loaded scenario) forgets them all and leaves the camera where its switch
put it.

**HUD**: built with bevy_ui (`core/camera.md > HUD Layout`). It has a top resource bar and, at the bottom, the HUD
Bar with five fixed sections. From left to right they are Minimap, Portrait, InfoPanel, SelectionPanel and
CommandPanel. All five have the same height and keep fixed horizontal slots, and a hidden panel's slot stays reserved
(`systems/controls/hud.md > HUD Bar`). bevy_ui lays out each root node inside its camera's viewport,
so the HUD needs its own full-window UI camera (`IsDefaultUiCamera` / `UiTargetCamera`). The sections sit at the
rectangles `hud::layout` computes from the surface size alone (`section_rect`, `PortraitGrid`); the same geometry
finds the portrait under a click, because headless tests have no UI picking (and `CommandGrid` the command button).
The InfoPanel (`hud/info_panel.rs`) and the SelectionPanel (`hud/selection_panel.rs`) show the selection (see
Selection), the CommandPanel (`hud/command_panel.rs`) its commands (see Commands UI); a section with nothing to show
shows its name (`SectionLabel`). A selected object the player doesn't see shows its record's last-known state and a
"Last seen … ago" line (a placeholder wording), except that a GDO player's remembered Supply Delivery Station shows
the live station's NextDeliveryTime (`core/map.md > SupplyDeliveryStation > InfoPanel`, design change 0057), "No
delivery scheduled" while it isn't empty, and never CurrentSupplies. A structure's panel follows design change 0105:
the local player's own shows its HP and, in the activity area under it (`InfoText::Activity`, hidden while empty),
what it is doing; another player's (or a neutral one) its owner and HP only. The activity's lines come from
`hud/info_panel/activity.rs::lines`, the one extension point for them: each structure capability adds its lines there
(construction with slice M2-H2, production with M2-H1), and nothing else of the panel changes. A producer's
(`activity::production`) are the unit in production and its progress from `Sim::production_status` (100% while it
waits at the doorway, 0085), the queue against `QUEUE_LIMIT`, and the rally point (placeholder wordings). A
constructor's (`activity::construction`) are the building under construction and its progress (frames done of
its recipe's, whole percent rounded down), or the building ready to place. A Recruitment Hall's
(`activity::recruitment`, slice M5-C, `Sim::hall_state`) are its Effectiveness with its claimed tiles, "Effectiveness:
49% (49/100 tiles)", and its "Unit Control: usage / capacity" rounded down, before its production's "Recruiting Recruit:
50%" and rally point, with no queue line. A building that holds units (`activity::holding`, slice M5-F2) shows "Inside:
3 / 10", everyone inside of its capacity, and a pool building its free units by kind, "Free: Plain 2, Constitution 1,
Metabolism 0, Both 0"; its queue line names the entries, "Queue: Militant, Release (2 / 10)", of its capacity, and a
Release under way is "Releasing Recruit: 50%". A transforming structure's
(`activity::transformation`, first; slice M3-E) is the building it becomes and its progress, "Warping in: Robo Portal
45%" ("Deploying" from a unit, slice M3-F), and its HP is out of its interpolated MaxHP (`Object::max_hp`), as its HP
bar is. A plate
adds the remaining crystals of the patch under it for its owner only (0099 as design change 0155 limits it, slice
M3-J2): another player's plate is another player's structure, its owner and HP; an uncovered patch shows its count to
everyone. A worker of the local player's adds what it carries, "Carrying 8 Space Crystals", and its AssignedDropOff,
"Drop-off: Egress Node" (`worker_lines`, placeholder lines its owner alone sees). The group of a type's unfinished
buildings (0185) is "(unfinished)" in the InfoPanel's title, and its portraits are dimmed as the map dims a transforming
object (`PortraitCell::Object::unfinished`, `transforming_color`). The top bar shows the player on the left,
the tick on the right (in a replay's playback also the replay's last tick and a note on the playback, `ReplayText`:
speed, paused or end, and a desync in red), and between them, centred, the local player's resources as their faction's
DisplayHud lists them (`hud/resources.rs`, one `ResourceFigure` text each, `resource_bar` for their readings): for
the GDO Space Crystals and Supplies (the banks), Power as consumed / generated (`Sim::power`), red while consumption
exceeds generation (0090), and Unit Control as used / `UNIT_CONTROL_CAP`, read through `Sim::unit_control` alone, so
that the units in production (design change 0143) count as the sim counts them; for the Colonists (slice M3-C,
`factions/colonists/overview.md > DisplayHud`) Space Crystals, Alloys, Essence and Flux, and Beacon Capacity as used
/ available (`Sim::army_cap`; red over capacity, 0112), with no Power; for the Syndicate (slice M4-B,
`factions/syndicate/overview.md > DisplayHud`) Space Crystals, Supplies and Tunnel Space as used / available
(`Sim::army_cap`; red over capacity, as `factions/syndicate/tunnel_area_and_space.md > Tunnel Space` has it, "12/10");
for the Cults (slice M5-C, `factions/cults/overview.md > DisplayHud`) Space Crystals and Unit Control as used /
available (`Sim::army_cap`: their Halls' usage and orphaned points over their Halls' capacities rounded down; over
capacity after a Hall is lost, not in red: the design gives the figure no warning). Under the top bar, a placeholder
notice (`hud/notices.rs`, `STATION_EMPTY_TEXT` for `NOTICE_TICKS` sim ticks) tells the local player that a Supply
Delivery Station where their Agents gather is empty (`SimEvent::SupplyStationEmpty`, design change 0256 point 3: "the
owner gets a notification", notifications not designed yet). At the end of the game a banner across the world view
(`hud/outcome.rs`, placeholder look and wording) shows the local player's `Banner`: VICTORY for the winners, DEFEAT for
everyone else, DRAW for everyone when nobody won (design change 0163), and DEFEATED with a spectating line while a
defeated player watches the game go on beside an ally (0102; never yet: the sim has no teams). A player defeated while
the game goes on with no ally left standing has **left for the results screen** (design change 0164, `hud/results.rs`;
`results::has_left`, the rule on its figures `results::left_the_game`): a placeholder, DEFEATED, their name and
faction, the game time and tick of their defeat, and whether the game goes on, on an opaque node over everything
below the top bar (`RESULTS_Z`), which takes the pointer, so nothing of the world shows or can be clicked. They stay
there after the end, and the end's banner shows over the screen (`BANNER_Z`). A side that falls in the game's last
tick never left. The screen is derived each frame from the sim and the local player, so the debug player switch and
a replay's playback show each player's view, and the sim isn't touched. A defeated player, and everyone once the
game is over, gives no commands: the command UI's input set doesn't run (`outcome::may_command`), and nothing is
`commandable` after the end.

**Input** (`input.rs`, `input/`): one shared layer turns the keyboard, the mouse, the wheel and the screen edge into
the input of the game and of the map editor, in `PreUpdate` (`ReadActions`, after Bevy's input, UI picking and the
focused text field's key handling), and gameplay and editor code read only what it produces: `InputAction`s, which
become UI state changes, `Command`s and `EditorAction`s. Never read raw keys, buttons, the wheel or the cursor
elsewhere.
- **Input contexts**: the mode picks whose bindings apply (`ClientMode::input_context`): the game's in a game and an
  editor play-test, the editor's in the editor. An action works in its own context only (`InputAction::context`), so
  one key can mean different things: V is a command slot in the game and the Select tool in the editor. The camera's
  pans and grab, the grid (G), `Cancel` (Esc), the editor's play-test switch (F5) and the placement keys (below) are
  shared by both. A button
  held across a context change ends its action. The editor's actions are `InputAction::Editor(EditorInput)`.
- **Bindings table** (`input::Bindings`, a `Resource`): the one list of `(InputAction, Binding)` entries for every
  game and editor action. A `Binding` is `Press(Chord)`, a key or mouse button with the modifiers it needs (Q,
  Ctrl + 1, Shift + Tab); `DoubleTap(Chord)`, the same chord pressed twice within `DOUBLE_TAP_SECONDS` with no other
  bound button in between; `Modifiers(..)`, held while those modifiers are, for the modifier actions (`Queue`,
  `ToggleSelection`, `SelectSameType`); or `Wheel(..)`, the mouse wheel while those modifiers are held (the editor's
  zoom; `ActionState::scrolled` says how far it turned, in lines). A press fires the action of its most specific
  binding in the context: of the bindings whose modifiers are all held, the one needing the most modifiers, then a
  double tap over a single press, then the earlier entry. So Shift + 1 adds to group 1 instead of recalling it, while
  Shift + Q presses slot Q with `Queue` held. One exception: in the editor Ctrl + key is a command of its own, so a
  held Ctrl blocks the editor's key bindings that don't need it (Ctrl + B is nothing, not the Paint tool;
  `InputContext::ctrl_blocks`). Another: the placement actions (`InputAction::is_placement`, today the editor's R,
  Shift + R, F and Shift + F that turn the structure being placed) take a press only while `Placing` says a structure
  is being placed, and then win over the press's other bindings, which they override (`Bindings::resolve_placing`):
  F flips then, and chooses the Fill tool otherwise. Whoever places reports `Placing` (the editor's
  `editor::input::report_placing`, in the editor only); a frame's input follows the report of the frame before. The
  game's AwaitingPlacement shares them (slice M2-H2, risk R11): the four belong to no context, so in a game they turn
  the ghost over the command slots R and F (the editor's `Turn`), and the game reports `Placing` from the local
  ControlState (`command_ui` `report_placing`, in the game modes, outside the command input set, so the keys end with
  the placement whatever stops the input: `Validate` ends a placement in the frame its selection stops being
  commandable, before the report, on a defeat as at the end). Shift + R and Shift + F are theirs while placing, not
  slot R or F with `Queue` held. The action stays held until its button goes up. `Bindings::default()` holds the
  design's defaults (camera keys, G for the grid, P for unit paths, Select and ContextCommand on the left and right
  buttons, Esc to cancel, Tab and Shift + Tab, the control group keys, the 4 × 3 command grid QWER/ASDF/ZXCV with
  the standard slots Z, X and C, Shift to queue) and the editor's (its F1 help lists them). A rebinding UI edits it
  with `bind` and `unbind`. It isn't serializable yet: loading it from a file needs Bevy's `serialize` feature for
  `KeyCode` and `MouseButton`. The selection adds `CenterOnObject` (Alt) to the modifier actions, for clicks on
  portraits, and F2 for the debug player switch, the fog of war F3 for its debug toggle (placeholders: the design
  has no debug keys), and the replay playback keys of the game's context: Space pauses and resumes, `.` plays one
  tick, `-` and `=` change the speed (`ReplayPause`, `ReplayStep`, `ReplaySlower`, `ReplayFaster`; placeholders).
- **Output**: `ActionState` (a `Resource`) holds this frame's actions (held, just pressed, just released), the wheel's
  turn, the modifiers, the context, the cursor in logical pixels, and the `Pointer`: the view point and the ground
  point (`coords::ground_under`) under the cursor, whether it is over the UI (a UI node by Bevy's picking, or outside
  the world view: the one over-UI test of the game and the editor), the press in progress and the drag box. A press of
  `Select` or the editor's `UseTool` that goes down on the map is followed until its release (`Pointer::press`, a
  `MapPress`: where it went down, the farthest the pointer has been from there (`moved`), whether that is further
  than `DRAG_THRESHOLD_PX`, and the ground point under the cursor now, which keeps following it over UI nodes on the
  world view, as the editor's drags do). In the frame its button goes up, the press is `Pointer::released`, for a
  rule of its own about clicks and drags (the command flow's targeting click allowance). `MapClick`
  messages report clicks on the map: `Select` when released, unless it became a drag, and `ContextCommand` and the
  editor's `Eyedropper` when pressed, with the modifiers and a double-click flag. A `Select` drag ends as a
  `SelectionBox` message in view space, kept inside the world view, with the modifiers held at its release. Nothing
  starts over the UI. What the modifiers of a click or a box mean comes from the table
  (`Bindings::holds(InputAction::Queue, click.modifiers)`).
- **Text fields**: while a text field has the keyboard (`TextFocus`), and in the frame after it lost it, keys type
  text and fire no binding, from the real keyboard or the injected one. So the Esc that gives a field up doesn't also
  cancel. A field's own keys (Enter commits, Esc reverts) are Bevy's keyboard input, read by the field
  (`editor/ui/fields.rs`), not bindings.
- Double taps and double-clicks are timed on `Time<Real>`. Placeholders until the design gives values, in
  `tuning/input.rs` and `tuning/camera.rs` (*Tunables*): `DRAG_THRESHOLD_PX`, `DOUBLE_TAP_SECONDS`,
  `DOUBLE_CLICK_SECONDS`, `DOUBLE_CLICK_DISTANCE_PX`, `EDGE_PAN_MARGIN_PX`, `PIXELS_PER_LINE` (wheels that report
  pixels).
- **Injection**: tests and QA scripts drive the layer through `InjectedInput`, a virtual keyboard and mouse, without
  winit. Press and release its `keys` and `mouse` (`ButtonInput`s) like Bevy's; the layer clears their press marks
  after reading them. `wheel` turns the wheel by that many lines before the next frame, `cursor` stands in for the
  mouse cursor, `over_ui` puts it over a UI node (headless apps have no UI picking), and `held` and `pressed` inject
  actions directly, past the bindings. Never write to Bevy's own `ButtonInput` resources: its input systems clear
  their presses before ours run. `RawInput` turns the real devices off in QA runs.

**Selection** (`selection.rs`, `selection/`; `systems/controls/selection.md`): client UI state, never sim
state. `ControlStates` holds every player's `ControlState`, a `Selection`, 10 control groups and the
ObjectInterfaceState (see Commands UI); the input changes and the HUD shows the local player's (`LocalPlayer`).
- **Model** (`selection/model.rs`): a selection is a set of object ids with the types they group as, a `GroupType`:
  the kind, and whether the object is unfinished, not in operation (`!Object::is_operational`): a site under
  construction (`Object::is_site`, slice M4-J1) or a transformation (`Object::is_transforming`: a Beacon a Drone
  deploys into, a building warped in). Its SelectionGroups are derived:
  one per `GroupType`, each by id, and one per instance of an ungroupable type (the map resources, which are never
  selected with other objects, design change 0048; the Colonists' Egress Node and Beacon, 0122; the Drone is
  Groupable, 0125). So a type's unfinished buildings are a group of their own, "as if it were another type", with
  their own panel (only Cancel), and one that completes joins the finished group (0185: the group key is "(type,
  finished?)"). A new dimension of the key goes into `GroupType`, which the rules, the panel (`offers`, `commands`)
  and the HUD read; a new kind needs nothing (0173's Peacekeeper (Shielded) is one). The groups come in one stable
  order, by the `GroupKey`: `ObjectKind`, the finished before the unfinished, then the instance (`Selection::groups`),
  which is the order of the SelectionPanel's portraits and of Tab and Shift + Tab (the placeholder of 0055). The
  ActiveGroup is one SelectionGroup, named by its `GroupKey` (0048): Tab and Shift + Tab cycle it with wrap-around,
  adding objects keeps it, and when its last member leaves the selection (dead, or no longer selectable) the next
  group in Tab order becomes active, wrapping around (design change 0199, `Selection::fix_active`); no group left,
  the selection is empty. At most `MAX_SELECTED`
  objects (a placeholder: the order limit).
- **Picking** (`selection/pick.rs`): `Selectables` is what the local player may select this frame (`selectable_by`:
  its own objects, others on Visible tiles; on Explored ones only a remembered Supply Delivery Station, at its
  record, and only for a GDO player, `remembered_selectable_by`, `core/vision.md > Explored Selectability`; with
  the fog off, everything), each object's box as it is drawn: interpolated, lifted by the height it stands at, in
  view space. A click picks the frontmost box under it (grown by `PICK_SLOP_SU`): a unit before a tile-based
  object, which it is drawn above (see Draw order), then the southernmost, and of boxes drawn at one depth anything
  before a map resource, so a click on a covered patch's tile picks its plate (design change 0099); a box
  (the input layer's `SelectionBox`) applies the priority tiers. `Relation::of` says own, enemy or neutral (no teams
  yet, so no allies); allies would count with enemies in the tiers. `Selectables::target_at` picks a target the same
  way among the selectables and the records the player remembers but may not select (design change 0139:
  targetable, not selectable), for a right-click, a targeting click and the pointer; with the fog off there are no
  records to add.
- **Rules** (`selection/rules.rs`, pure and unit tested): map clicks, boxes, portrait clicks, control group keys and
  Tab on a `ControlState`, under the Constraints: another player's object or a resource is only ever selected alone,
  and any click on one selects it alone (design change 0048). On an own unit a click selects it, Shift toggles it,
  Ctrl or a double-click selects its type on screen, and Ctrl + Shift adds that type (0052). A box selects what its
  tiers give, and Shift + box adds that (0049). The add inputs all go through `ControlState::add_to_selection`: they
  remove nothing and add nothing the Constraints forbid, so they do nothing while another player's or a neutral
  object is selected (0053). A plain click on empty ground or an empty box clears the selection; a modified click
  there or an empty Shift + box changes nothing (0051). Control groups take only own objects: Assign and Add with
  another player's or a neutral object selected change nothing (0056). What the modifiers of a click or a box mean
  comes from the bindings (`ClickRoles`; Shift is `ToggleSelection`, the add inputs' modifier). Every rule the design
  leaves open is marked as a placeholder where it is made.
- **Systems** (`SelectionSystems`): `Sync` (in `SyncSim`) removes objects that left the sim from every selection and
  group in the frame of their tick (`SimEvent::Removed`, deaths included, and anything else gone; a station the player
  still selects by its record stays), and those their player may no longer select (an enemy that walked into the fog;
  a placeholder, the design is silent). A selected object the player sees whose `GroupType` changed in place, a new
  kind (`SimEvent::KindChanged`) or a building that completes or starts transforming, stays selected as its new type
  (`ControlState::set_type`). The ActiveGroup stays while it exists (`systems/controls/interface.md >
  ObjectInterfaceState`: a change the player didn't make keeps the state unless the ActiveGroup becomes empty; design
  change 0199: changes in a group that isn't active never change it); one whose members changed type gives way to the
  group they joined (0199), and the ObjectInterfaceState resets with the ActiveGroup. A unit's type there is the sim's
  with the toggles already sent for its next tick applied (`PendingCommands::kind_of`), so the group a press switched
  (`ControlState::toggle`, below) stays switched in the frame before the sim's tick.
  `Sync` also starts every state over when the sim is replaced (`SimRes::generation`). `Input` (in `Actions`) applies
  map clicks, boxes, the control group keys (Recall and Center snaps the camera to the drawn centroid), Tab, and the
  debug player switch (F2, only while `SimConfig::debug`: the client acts as the next player, so "own" and the state
  in use change; a replay's playback switches without debug commands too). The HUD's portrait clicks run after it.
  Systems change a state only through `LocalControl::update`, which marks `ControlStates` changed only on a real
  change, and puts the ObjectInterfaceState back to DefaultState when the Selection or its ActiveGroup changes; a
  control group's recall puts it back even when the selection stays the same (`ControlState::recall_group`, design
  change 0078). While a target is awaited, map clicks and boxes select nothing (the drag box isn't drawn): the
  left-click is the target's.
- **Visuals** (`selection/markers.rs`): one marker entity per selected object: a ring around a unit, in the unit-decal
  layer under it (over the structures), or a frame inside a footprint, which the object's visual would hide there, so
  it is drawn just in front of the object (`FRAME_Z`: half a su of ground y, under the objects south of it); coloured
  by relation, the ActiveGroup's solid and the others' dashed; meshes cached per shape, relation and activity, and a
  marker whose object changed its shape (a kind changed in place) takes its new shape's mesh. The drag box is four
  edges of one shared mesh in the overlay layer, shown while `Pointer::drag_box` is set.

**Commands UI** (`command_ui.rs`, `command_ui/`; `systems/controls/interface.md > ObjectInterfaceState`
to `systems/controls/hud.md > CommandIndicators`, `systems/controls/commands.md > Unit Command`): how the
local player orders their units, their producing structures and their constructors, and the Colonists' Beacons and
Drones, through one command flow (plan decision T10, slices M2-H1, M2-H2 and M3-J1).
- **Rules** (pure and unit tested): `command_ui/interface.rs` and its children (`commands.rs`, `templates.rs` with
  its `production.rs`, `state.rs` and `offers.rs` and an object's own panel in `templates/<faction>/<object>.rs`,
  `conditions.rs`, `panel.rs`) have the `PanelCommand`s (the units' and the producers' commands), the `InterfaceState`
  (DefaultState, or AwaitingTarget of a command, `queued` once a Shift-click has queued it: design change 0070) in
  each `ControlState`, the ActiveGroup's `InterfaceTemplate` (the Peacekeeper's and the Guard's BasicCombatUnit grid;
  every producer's Production grid, built from its `ProductionDef` (plan decision T23): its units' Train from Q on,
  Cancel Production in X, Set Rally Point in C, so the Barracks trains the Peacekeeper in Q, the Robo Portal the
  Drone and the Bio Portal the Sabre Legionnaire, design change 0124; the Rocketman, not in the game yet, has no
  button, while the portals' units not in the game yet hold their slots unavailable, 0124's "W Scout" and "W Lancer,
  E psionic unit", as `PanelCommand::Planned`, slice M3-J2),
  the `Panel` a selection gets (common, group, pending, disabled and unavailable buttons; Back in Z while a target is
  awaited) and who receives a command (`recipients`: a common one the whole selection, a group one the whole
  ActiveGroup, by hotkey or button; never some of them, no smart-casting, 0071, apart from the units that can't carry
  it out, which the resolution leaves out). A type's grid is its template's plus its own commands (`grid`: in R,
  Raise Shield for the Peacekeeper and Lower Shield for the Peacekeeper (Shielded), design changes 0173 and 0174;
  Convert to Rifle Legionnaire for the Sabre and Convert to Sabre Legionnaire for the Rifle, `PanelCommand::Convert`
  of the target form, design change 0198: "It goes only to the ActiveGroup"; each its type's own, so in a selection
  of both it goes to the ActiveGroup). The Peacekeeper (Shielded)'s panel leaves out the Attack its type
  still offers (`hides_attack`, in `state_grid`; 0173: "no Attack"), so that Attack stays common in a selection with
  other units: an Attack on an object leaves the shielded units out (0075), an AttackMove takes them along (0076). A
  toggle's press switches its units' type, and group, at once (`ControlState::toggle`, by `Ability::switched_kind`,
  before the sim's tick has it; design change 0198): the group they join becomes the ActiveGroup, in its
  DefaultState (design change 0199), so the next press goes to every unit the last one reached. `Modes` holds the
  selected units' modes: Attack is disabled while none of its recipients takes one, by its type from the press
  (design change 0202, `riot_shield::takes_attack_orders`: a Peacekeeper whose shield is on its way down takes it and
  attacks once the shield is down). An Attack sent in the frame of a Lower Shield, before the sim's tick, goes too:
  the gate refuses it only because the unit is still a Peacekeeper (Shielded), and the tick applies the Lower first
  (`OrderOutbox::give`, `attack_once_toggled`). `Conditions` holds the `Modes` and what
  the command gate refuses: a production command is disabled exactly when `Sim::validate` refuses the
  `Command::Production` pressing it would send (the gate's own predicates, no copy of them: crystals, a full queue,
  nothing to cancel; 0081; never the army cap, which is checked when production starts, 0143); a toggle never is
  (design change 0198: a press is never refused). `command_ui/resolution.rs`
  has what a click resolves to, and `resolution/pointer.rs` the `PointerDisplay` that previews it (split off in slice
  M5-J1); `interface/commands/names.rs` has the commands' labels and names, beside `commands.rs`, the commands and
  what they give the sim. A `MapTarget` is what is under the cursor
  (`CursorTarget`, from `Selectables::target_at`: a remembered record too, 0139), with the ground point there and,
  for an object, the target as the local player knows it (`aimed`, the sim's `Sim::aimed`). A right-click resolves
  per unit (design changes 0062, 0063), whichever group is active, by the sim's one rule (`unit_right_click` calls
  `sc_sim::orders::resolution::resolve`, the rally point's rule too, design change 0136) on `UnitCapabilities` (the
  unit's template and the sim's `Abilities`: its attack's TargetDomain, whether its type takes an Attack, 0202, whether it is
  a worker and carries a load): an enemy it can attack gives Attack, a patch Gather for a Drone, anything else a Move
  to it, which follows a unit (a shielded Peacekeeper follows an enemy, 0074; a right-click on a selected unit makes
  it stop and the others follow it, 0168); the ground a Move. A Drone (`InterfaceTemplate::Drone`) resolves by the
  same rule (see Gathering below); the Beacon and the Egress Node have no right-click, nor has an object that
  transforms (no template, no abilities: slice M3-J1); objects without commands do nothing. A producer's right-click
  sets its rally point
  (`object_right_click` gives a `Resolved::Rally` beside the units' `Resolved::Order`s, so a mixed selection does
  both, 0062; `rally_target`: the object under the cursor, else the ground point; the producer itself clears it,
  0086); Set Rally Point's targeting click resolves the same way (`target_resolution`), and the pointer shows Move
  for a rally point. `group_orders` makes one order per distinct order, so units
  sent to one location go as one group (the magic box). A left-click while a target is awaited gives `target_order`:
  Attack takes any destructible object, own and neutral ones too (force-attack, 0059, 0064), never an indestructible
  one; a right-click never attacks them. `unit_target_order` is the targeting click's shield rule, per unit: an
  explicit Attack on an object leaves out a Peacekeeper (Shielded), which keeps its orders (0075, 0202; all of them
  shielded: nothing is given, and Attack is disabled anyway); an AttackMove goes to every unit, shielded ones too,
  which the sim walks on without engaging (0076: the sim's gate drops shielded units from Attack only). The pointer
  previews the set of per-unit right-click actions: one shows its pointer, several show `Mixed` (0063); while a target
  is awaited, the pending command's look over a target it can use, else Inactive. Nothing is commandable unless the
  whole selection is the player's, nor once the game is over (`commandable`).
- **Systems** (`CommandUiSystems`): `Validate` (in `SyncSim`, after the selection's sync) ends an AwaitingTarget that
  no longer holds. `Input` (in `Actions`, after the selection's input) turns Esc, the command hotkeys
  (`InputAction::CommandSlot`), clicks on the panel (found by `CommandGrid`), left-clicks on the map while a target is
  awaited and right-clicks into state changes and orders. An order is a `Command::Order` in `PendingCommands` naming
  only the player's own units, given only if `Sim::validate` takes it, queued while `Queue` (Shift) is held; each one
  is also an `OrderIssued` message. A click's per-unit orders go out grouped (`OrderOutbox::give_each`); in a
  targeting click each recipient's order is `unit_target_order`'s. An ability is a `Command::Ability`, validated the
  same way, never queued, and no `OrderIssued`. A producer's command is one `Command::Production` to every recipient
  (`OrderOutbox::produce`: the player's own producers, validated the same way), never queued, no `OrderIssued`: no
  smart-casting, the sim decides which producers a Train reaches and pays for (0083); a Shift-click while Set Rally
  Point awaits its target sets the rally point and ends targeting like a plain click (a placeholder reading: a rally
  point isn't queued). While a target is awaited (`systems/controls/interface.md > AwaitingTarget`, design
  changes 0065 to 0070 and 0073): the other commands do nothing (`press`; the panel shows them disabled); a click counts
  at the press point, as the frame of the press drew the objects (`TargetPress`, recorded when the Select button goes
  down on the map, used when it comes up, `Pointer::released`); the press is a click while the pointer stayed within
  `TARGETING_CLICK_ALLOWANCE_PX` of where it went down (16, a placeholder: more than the normal `DRAG_THRESHOLD_PX`,
  which holds everywhere else), and beyond it a drag, which does nothing (the selection's input ignores it, no box is
  drawn, the target is still awaited); a click that gives nothing (an invalid target, or an order the sim rejects)
  keeps targeting and sends an `InvalidTarget` message, Shift held or not. A click without Shift returns to
  DefaultState. A Shift-click (`Queue` held as the button went down, where the click counts) queues its order, one
  more in each unit's queue, and keeps the target awaited, marked `queued`; letting go of Shift then ends targeting
  (`end_queued_targeting`, first in `Input`, after a targeting press still held has had its click), while before
  the first queued click it changes nothing (0070's TBD, its proposed default). Esc, a right-click and Back cancel,
  Shift held or not. The player's changes of the selection or the ActiveGroup cancel targeting
  (`LocalControl::update`), and so does every control group recall, of the same selection too (0078); removals by
  the game (deaths) don't, unless the whole ActiveGroup is gone (`ControlState::retain`).
- **Construction** (slice M2-H2; `factions/gdo/structures/deployment_center.md >
  ObjectInterfaceState[DeploymentCenter]`, design change 0104; `factions/gdo/economy.md > ExtractionFacility >
  ObjectInterfaceState[ExtractionFacility]`): the constructors' commands are `PanelCommand`s too: `BuildMenu(menu)` (the
  Deployment Center's Q Basic and W Advanced, unavailable while none of its buildings is in the game),
  `Construct(kind)`, `CancelConstruction` and `CancelReady` (X), and `PlaceReady(kind)`, "Select completed building",
  which takes its building's slot once it is ready. `InterfaceState` adds `BuildMenu(menu)` and
  `AwaitingPlacement(Placing { builder, kind, orientation })`, the builder a constructor's `Builder::Constructor(id)`. A
  constructor's panel follows its state (`state_grid`, from `Conditions::constructor`, the ActiveGroup's one instance's
  `Constructor`), and its commands are that instance's own (`PanelCommand::instance_only`): they go to the active
  constructor only, never to the other selected ones. A construction command is one `Command::Construction`
  (`OrderOutbox::construct`, validated like the others), and Construct and the Cancels return to DefaultState. Back, Esc
  and a right-click go back one state (`InterfaceState::back`): from AwaitingPlacement to the Deployment Center's build
  menu or the Extraction Facility's DefaultState, from a build menu to DefaultState. While placing, the placement keys
  turn the ghost (`turn_ghost`, see Input), and a left-click on the map places it if `Sim::placement_check` allows it
  now: one Place at the footprint centred under the cursor (`Placing::origin_under`), as turned, and back to
  DefaultState; a Shift-click the same (0071); elsewhere nothing is sent, placing goes on, and an `InvalidPlacement`
  message says why. A Place the sim refuses as it applies it (`SimEvent::PlacementRefused`) leaves the building ready
  and the player in DefaultState. A build menu leaves the map's clicks to the selection; AwaitingPlacement takes them
  (`InterfaceState::takes_map_clicks`). `Validate` ends a build menu the ActiveGroup doesn't have, a placement whose
  constructor isn't active or holds the building no more, and both when the selection isn't commandable.
- **Warp In and Deploy** (slice M3-J1; `factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState`,
  `factions/colonists/units/drone.md > Drone ObjectInterfaceState`; design changes 0116, 0119 to 0122, 0124, 0125):
  `command_ui/interface/templates/colonists/` has the Colonists' panels, one file per object (the portals' have
  their planned units, `PanelTemplate::planned`). The Beacon's (`InterfaceTemplate::Beacon`)
  opens Warp In (Basic) in Q and Warp In (Advanced) in W, `BuildMenu`s whose entries are its own (`menu_grid`): a
  `WarpIn(kind)` for each building of its Constructs List (`transform::warp_in_recipes`), and a `Planned(..)` for
  each building not in the game yet, which holds its slot unavailable. A `WarpIn` enters `AwaitingPlacement` with
  `Builder::Beacon(id)`, for the Beacon Tabbed to (Beacons are Ungroupable, 0122; `instance_only`): the ghost is
  `transform::warp_in_footprint` of the Beacon's placement (`Placing::origin`), turned by the placement keys about
  its centre, never moved by the mouse (`Placing::follows_cursor`: the pointer shows); a left-click anywhere sends
  `ConstructionCommand::WarpIn` if `Sim::warp_in_check` allows it (`OrderOutbox::try_construct`; a gate refusal,
  the bank short since the entry was pressed, says why and placement goes on), and Back returns to its menu. A
  menu entry is greyed out by the gate's answer to its Warp In unturned, `Unplaceable` left out: the ghost shows
  where it can't stand. The Drone's grid (`InterfaceTemplate::Drone`) is Q Move, A Deploy, S Set Drop-off, X Stop (S
  and the right-click: Gathering below); A enters `AwaitingPlacement` with `Builder::Drones { queued }`: the ghost
  under the cursor, checked by `Sim::deploy_check` for the Drone the click would go to (`deploy_recipient`: a plain
  click's the ActiveGroup's first Drone in portrait order, whose orders the Deploy replaces; a Shift-click's the
  first with no Deploy, among `Sim::pending_builds` and the Deploys still in `PendingCommands`, `deploying`). A
  click gives that one Drone `OrderKind::Deploy { origin }`, queued with Shift; Shift-placement stays (`queued:
  true`) while another Drone of the group has none, and letting go of Shift ends it (`end_queued_placement`).
  Whatever transforms (`Object::is_transforming`) is unfinished, in its type's unfinished group (design change
  0185, `GroupType`): that group's key gives the panel `TRANSFORMING_GRID`, X `CancelTransform` alone
  (`ConstructionCommand::CancelTransform`, to the ActiveGroup), as `offers` and `commands` of an unfinished
  `GroupType` give the Cancel and nothing else, so the finished group's commands never go to it and `still_valid`
  ends a transforming Beacon's menu or placement; `Modes::transforming` gives the object no template in the
  resolution (no right-click, no rally point). A Drone that deploys stays selected as its Beacon in a group of its
  own (`ControlState::set_type`), and a Drones' placement ends once the ActiveGroup is no longer Drones. A greyed-out button (design change 0162) keeps its reason (`PanelSlot::why`, a
  `WhyUnavailable`: the gate's `Reject`, the shields, a command awaiting its target, the type, not in the game
  yet); pressing it, by hotkey or click, issues nothing and sends `CommandRefused`, whose message
  (`feedback::unavailable_text`) the feedback band shows.
- **Gathering** (slice M3-J2; `factions/colonists/units/drone.md > Right-Click Resolution`,
  `factions/colonists/units/drone.md > Set Drop-off Location`, `factions/colonists/economy.md > Egress Node > Egress
  Node ObjectInterfaceState`; design changes 0124, 0136, 0139, 0140, 0155): a Drone's right-click is the sim's one rule,
  per unit like every unit's: a patch (seen or remembered) gives Gather, an own drop-off a carrying Drone's `DropOff`
  and an empty one's Move to it (0140: no empty Drop Off), anything else a Move to it, a unit followed; the pointer
  previews Gather as GatherResources and Drop Off as ReturnResources ("Cursor over resource (resource gatherer
  selected)", "Cursor over drop-off point (resource gatherer carrying resources)"), so Drones that would do both show
  Mixed. S, Set Drop-off (`PanelCommand::SetDropOff`, a common command: every selected Drone's, Q0040 point 9), awaits a
  target: an own Egress Node that isn't transforming (`drop_off_target`, the `Aimed::drop_off` the gate checks) resolves
  to `Resolved::Ability`, one `Command::Ability` of `Ability::SetDropOff` that only updates the AssignedDropOff, the
  orders going on; anything else is an invalid target, and the pointer over a node shows ReturnResources (a placeholder:
  the rules have no AwaitingTarget\[SetDropOffLocation\] row). The Egress Node's grid (`InterfaceTemplate::EgressNode`)
  has F, Select Returning Drones (`PanelCommand::SelectReturningDrones`), a StateOnlyTransition
  (`SlotPress::SelectReturningDrones`, then `ControlState::replace_selection`) that replaces the selection with the
  node's `Sim::returning_drones` (`Conditions::returning`, asked for the ActiveGroup's node: `instance_only`); the
  button shows only while there are some (`Conditions::shows`). Its label, "Drones", keeps the full label size at 720p.
- **The Syndicate's panels** (slice M4-J2; `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`,
  `factions/syndicate/tunnel_menus.md > EjectMenu`, `factions/syndicate/structures/hideout.md >
  ObjectInterfaceState[Hideout]`; design changes 0252, 0257, 0260): `command_ui/interface/templates/syndicate/` has
  `tunnel.rs` (`InterfaceTemplate::Tunnel`, no right-click: W `PanelCommand::ExpandMenu`, E `PanelCommand::EjectMenu`, X
  `PanelCommand::CancelOperation`; Q `PanelCommand::UpgradeTunnel`, slice M4-G: greyed by `Sim::can_upgrade` with the
  gate's reason, "Something blocks the upgrade" never saying what, its price in the hint, "Q: Upgrade Tunnel (2
  Supplies)", `Conditions::price`), `hideout.rs` (the Production grid: W
  trains the Guard, greyed "Requires an Optics Lab" by the gate until the player owns one, slice M4-K; E the
  Grenadier, greyed "Requires a Chemical Plant" alike, slice M4-U), `eject_menu.rs` and `expand_menu.rs`. **A
  placeholder entry** (`templates/planned.rs`, `Planned`) holds its slot greyed with its reason
  (`Conditions::planned`): "Requires an Optics Lab" while the player owns no
  structure of the type its unit or building will require (`Planned::requires`; `Conditions::of` keeps the types
  owned), "Not available yet" otherwise, until the slice that brings its command replaces it. An entry in the game
  asks the gate, and **a prerequisite is the gate's** (`Reject::Requires`, *Tech prerequisites*): no template names
  one. **A menu's order is the sim's data**
  (`sc_sim::defs::EJECT_MENU_ORDER`, design change 0257's "Agent, Guard, Grenadier, …"; `sc_sim::defs::EXPAND_MENU`):
  a slice that adds a unit or an expansion adds it there, not in the client. E enters
  `InterfaceState::EjectMenu`, a StateOnlyTransition (`SlotPress::OpenEjectMenu`): a `PanelCommand::Eject(kind)` tile
  per kind of that list, from Q on, every slot but Z; each is greyed by the gate's answer to its `Command::Tunnel`
  (`Reject::NotInNetwork`, `InTransit` for the transit delay, design change 0288 point 1, `NoTransit` for the tier,
  each with its message) and carries the network's count (`PanelSlot::count`, `Conditions::network_count` from
  `Sim::network_counts`, so the count drops at the press); a kind with none inside keeps its tile, 0 and greyed (design
  change 0299, answering Q0086 point 2; the order is 0285's, point 1). A tile sends `TunnelCommand::Eject` to the Tunnel
  Tabbed to (Tunnels are Ungroupable; `SlotPress::Tunnel`, `OrderOutbox::tunnel`) and the menu stays open (0300, point
  3); Z, Escape and a right-click go back. Held units leave the selection and stay in their control groups
  (`ControlState::retain_on_map`); a recall selects the members on the map, and a group with none on the map recalls as
  an empty one does: nothing changes (`ControlState::recall_group`; 0299, point 5). A Hideout's rally pointer and marker
  on an own Tunnel or Hideout are every producer's (Move, a green Object marker); the sim decides what the unit does
  (0252).
- **The ExpandMenu and an expansion's placement** (slice M4-F; design changes 0257, 0290 and 0291): W enters
  `InterfaceState::ExpandMenu` (`SlotPress::OpenExpandMenu`), whose grid is `sc_sim::defs::EXPAND_MENU` in the
  EjectMenu's tile slots (`expand_menu::grid`): a `PanelCommand::Expand(kind)` per type in the game, greyed by
  `Sim::can_expand` (`Conditions::of`, its reasons "This Tunnel is busy", "This Tunnel's tier is too low", "Requires
  a ...", "Not enough Space Crystals"), and a `Planned` placeholder per entry not in the game yet, found by name
  (`Planned::expansion`), so a slice that brings an expansion changes the sim's list only: the Laser Turret's C is its
  `Expand` since slice M4-S2, greyed by the same gate ("Requires an Optics Lab", the operation slot), and the one place
  a turret is ordered from since design change 0359 (`tests/commands/agent_ui.rs`). An entry enters `AwaitingPlacement`
  with **`Builder::Tunnel`** for the Tunnel Tabbed to (`SlotPress::AwaitExpansion`); Z goes back to the menu, where the
  panel stays meanwhile (`state_grid`). **The ghost is held within the Tunnel's area** (`Placing::origin`,
  `placing::within`: the cursor outside leaves it at the nearest spot inside; a reading of "follows the cursor within
  the Tunnel Area"), and judged by `Sim::expansion_check`, what the owner knows, in `ghost::check_now` with every
  builder's; the Tunnel's area shows meanwhile, the Tunnel being selected (`TunnelAreaOverlay`; design change 0300: "as
  built"). A click on a green ghost sends `TunnelCommand::Expand` through `OrderOutbox::try_tunnel` and ends placement,
  Shift or not (one operation, no queue); the gate's refusal sends `CommandRefused` (the bank) or `InvalidPlacement`
  (the truth) and placement goes on. X is `CancelOperation` (`TunnelCommand::Cancel`), greyed while there is none
  (`Reject::NoOperation`, "Nothing to cancel"). The expansion's site is any site: `SITE_GRID`, and the look and progress
  bar of `object_render/site.rs`. The Tunnel's InfoPanel shows its operation, "Building Hideout: 25%" or "Upgrading to
  Tier 2: 25%" (slice M4-G, placeholder wording; `activity::operation`), which its X cancels either way.
- **The Agent's panel** (slice M4-J1; `factions/syndicate/units/agent.md > Agent ObjectInterfaceState`; design changes
  0233, 0237, 0257 and 0359): `templates/syndicate/agent.rs` (`InterfaceTemplate::Agent`) is Q Move, A
  `PanelCommand::Build(Tunnel)`, S `PanelCommand::DropOff`, D `PanelCommand::Repair` (slice M5-N8, as the Recruit's D)
  and X Stop, with no Build of a Laser Turret (slice M4-S1's D, which 0359 took away: a turret is
  the ExpandMenu's alone); its right-click is the sim's one rule (*Gathering* above). **A Build** is an instance command
  (`SlotPress::AwaitBuild`): `AwaitingPlacement` with `Builder::Agent` for the ActiveGroup's one Agent (Agents are
  Ungroupable: Tab picks the next one). It is greyed exactly when the gate refuses an unqueued Build of that Agent's
  (`conditions::build_probe`), the placement's own refusal left out; its price (`Sim::build_price`, a `BuildPrice` in
  its resource) is `PanelSlot::price`, which the hint names ("A: Build Tunnel (2 Supplies)"; a Recruit's "W: Build
  Storage (100 Space Crystals)"). A click checks the ghost (`Sim::build_check`; a red one sends `InvalidPlacement`),
  then gives the order through `OrderOutbox::try_give`, which returns the gate's `Reject`: a refused one (the bank,
  "Already building" for a second Build queued to an Agent) sends `CommandRefused` and placement goes on; a sent one,
  queued with Shift, ends it ("each Agent takes exactly one placement"). The client counts no Build still in
  `PendingCommands` in a price or a ghost. **S** awaits an own drop-off in operation (`drop_off_target`), greyed while
  none of its recipients carries a load (`Reject::NothingToDeliver`, "Not carrying resources"). A right-click that
  enters a Tunnel shows `PointerDisplay::Enter` (`cursor::ENTER_COLOR`, a placeholder), a damaged one too (Enter wins
  over Repair, design change 0325); on a damaged own Laser Turret, Repair; a carrying Agent's on its Tunnel
  ReturnResources; on a site, Move. **A site's panel** (`templates/site.rs`, `SITE_GRID`) is X
  `CancelConstruction` alone, for any site (`Conditions::site`: `Object::is_site`, never its kind), which slice M4-F
  reuses, a turret's site among its expansions': an unfinished `GroupType` offers both Cancels (`offers::UNFINISHED`),
  and the gate takes the one that fits.
- **The Armory's panel** (slice M5-F2; `factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`, design
  changes 0337 and 0348): `templates/cults/armory.rs` (`InterfaceTemplate::PoolProducer`, the pool buildings' one
  template, built from the building's `ProductionDef` as the Production template is) is Q, W, E and R
  `PanelCommand::KindMenu(kind)`, A `EjectAll`, X `CancelProduction` and C `SetRallyPoint`. A kind button counts its
  free Recruits (`PanelSlot::count`, `Conditions::pool_count` from `Sim::pool_counts`) and is greyed at 0 (the gate's
  `Reject::NoRecruit`, "No free Recruit of that kind"); it enters `InterfaceState::KindMenu(kind)`
  (`SlotPress::OpenKindMenu`), whose grid (`armory::kind_menu`) is a `PanelCommand::TrainFrom(unit, kind)` per recipe
  from Q (`ProductionCommand::TrainFrom`: the Armory's Q Militant and W Gunner, the Gunner greyed "Requires a
  Gunworks" by the gate's `Reject::Requires` while its player owns none, slice M5-G2), a unit not in the game yet in
  its slot (`PanelTemplate::planned`, none today), A `PanelCommand::Release(kind)` and Z Back; the menu stays open while
  its entries are used (`still_valid`), greyed by the gate. Eject All is a
  `Command::Holding` (`SlotPress::Hold`, `OrderOutbox::hold`), greyed by the gate ("No free Recruit inside"). The
  right-click sets the rally point, as a producer's (`object_right_click`). **One building at a time** (design change
  0348 point 5: the Armory is Ungroupable): every command of a pool building's panel goes to the one Tabbed to, X and C
  too (`is_common`, `holding::trains`), where a Barracks shares them with the others selected (0083); a right-click
  still rallies every selected producer, as each selected object resolves it for itself.
- **The Recruit's panel** (slice M5-J1; `factions/cults/units/recruit.md > Recruit ObjectInterfaceState`, design
  changes 0322, 0327 and 0347): `templates/cults/recruit.rs` (`InterfaceTemplate::Recruit`) is Q Move, A and W
  `PanelCommand::BuildMenu` (named "Basic Construct" and "Advanced Construct", `PanelTemplate::names`), S
  `PanelCommand::Assist`, D `PanelCommand::Repair` (slice M5-N8) and X Stop. Recruits are Groupable: every
  command goes to all the selected Recruits, the one ActiveGroup (Tab stops on them once). **The Construct menus**
  (`templates/cults/construct_menu.rs`) are the sim's `defs::CONSTRUCT_MENU`, the one table of their fixed slots: a
  type in the game is its `PanelCommand::Build(kind)`, greyed by the gate (its TechPrerequisite, the bank), one not
  yet built by the Recruits its `Planned` entry, found by name (`Planned::construct`), so a slice that brings a
  building changes the sim's table only (slice M5-G2 made the Gunworks, the Machine Shop and the Scrap Hangar
  `Build`s, greyed "Requires an Armory", "Requires a Chop Shop" and "Requires an Airstrip" until those stand); Z is
  Back. A Build enters `AwaitingPlacement` with **`Builder::Recruits { queued }`**
  (`Placing::recruits`): one ghost for the group, judged by `ghost::recruits_check` (`Sim::group_build_check`, the
  gate's own check as far as the player knows, the group left out of the occupants: design change 0362, a Recruit of
  the order on the footprint makes the site wait, so no warning). A click gives one `OrderKind::Build` to every
  Recruit of the group through `OrderOutbox::try_give`, paid once; a Shift-click queues it and placement stays while
  Shift is held (`end_queued_placement`), each Build its ticket's one lingering ghost. **S** awaits an own site that
  the recipients build from outside (`resolution::assist_target`) and gives `OrderKind::Assist`. The pointer shows
  `PointerDisplay::Assist` over such a site (`cursor::ASSIST_COLOR`, a placeholder), on a right-click as while S
  awaits. **D** (slice M5-N8, design changes 0322 and 0325) awaits a damaged own object that a recipient repairs
  (`resolution::repair_target`: `Aimed::repair`) and gives `OrderKind::Repair`, which the gate gives to the recipients
  that can repair it (`Reject::CantRepair`); it is never greyed, there being no target to judge before the click. The
  pointer shows `PointerDisplay::Repair` (`cursor::REPAIR_COLOR`, a placeholder) where the right-click gives a Repair,
  and while D awaits over an object that a selected type repairs (`resolution::repaired_by`,
  `sc_sim::repair::repairs`), else Inactive, where a click sends `InvalidTarget` and targeting goes on. A damaged own
  Armory that takes the Recruit in is entered by a right-click, repaired by D (Enter wins, 0325). Enter shows over an
  own Armory in operation that takes the Recruit in, never over its site (0347 point 4). The right-click is the sim's
  one rule (`sc_sim::orders::resolution::RULES`). The InfoPanel shows a carrying Recruit's load to its owner, as a
  Drone's ("Carrying 2 Space Crystals", `info_panel::worker_lines`).
- **Visuals**: the panel's buttons (`hud/command_panel.rs`) are rebuilt when the panel, the bindings or the surface
  changes; each shows its command's label (`PanelSlot::label`) and its hotkey from the bindings, and an EjectMenu tile
  its count in its bottom-right corner. **Every label keeps
  the full label size at 720p: six characters at most** (slice M3-K2a; a unit test measures every label of every
  panel on surfaces down to 800 × 600): what a button makes by its type's tag ("PK" trains a Peacekeeper, "GN" builds
  a Generator, "RP" warps in a Robo Portal, as `short_label` shows them on the map), "Place" for a ready building,
  "Cancel" for every Cancel, else the command's name, its first word or a shortening ("Raise", "Drop", "Rev.",
  "AtkGnd", "Adv."). **Never add a label longer than that**; its full name goes in `PanelCommand::name`. The
  InfoPanel names every button under the pointer in full, with its hotkey (`hud/info_panel/hint.rs`, `Panel::name`:
  "R: Raise Shield", "Q: Train Peacekeeper"; the client has no tooltips; slices M3-S and M3-K2a), after the
  ActiveGroup's type where the design names a command so (`PanelCommand::name_on`: a Beacon's "Warp In (Basic)",
  "Cancel Deployment" and "Cancel Warp In"; a Convert names its target form, "Convert to Rifle Legionnaire"), and with
  a Build's price, the sim's (`PanelSlot::price`: "A: Build Tunnel (1 Supply)", slice M4-J1). A label never wraps
  (`TextLayout::no_wrap`): one too long for its button at `LABEL_VH` (on a surface too small for it) is drawn smaller
  to fit on one line inside a lit button's thicker border (`label_px`, from the default font's fixed advance, 0.6 em,
  which a unit test checks against the font), so a label keeps its size when its button lights up. The indicators
  (`command_ui/indicators.rs`, the mesh in `indicators/draw.rs`) are
  one mesh in the overlays layer, rebuilt every frame while anything shows: markers at the targets of the selected
  own units' current orders (and queued ones while Shift is held), a flash where an order was just given
  (`FLASH_TICKS`), and while Shift is held each unit's route through its waypoints as dashed lines; colours follow
  `systems/controls/hud.md > Color language` (an Attack Ground's marker, slice M4-O, is a Location in
  red, `systems/controls/hud.md > Indicator assignments`, and its route leg ends at its point; the D
  slot's AwaitingTarget gives the order on the ground only, `command_ui/resolution.rs`). **A marker or a route leg that names an object goes by what
  the local player knows of it, never by the live object** (`indicators/targets.rs`, `Sight::target_view`: the sim's
  `TargetView` through the client's `Sight`; design change 0139, slice M3-K2a): where the object is drawn while the
  player sees it, around its record (last-known position and footprint) while the player remembers it, wherever the
  object really is (a Beacon whose deployment was cancelled walks away as a Drone) and whether it still exists, and
  nothing while the player knows nothing of it (a unit out of sight). `marker_ring` is where a marker is drawn; the
  rally lines end at it. The selected own producers' rally points (`indicators/rally.rs`, design
  changes 0087, 0088) show a marker, green on the ground and around an own or neutral object, red around an enemy
  (the sim turns an enemy rally point it loses sight of into a ground one, so the marker turns green there), and a
  solid line from the footprint's edge to it; nothing while the producer isn't selected or has none; a portal rallied
  on a patch rings the patch in green (slice M3-J2). The InfoPanel's rally line names the target as the player knows
  it too (`activity::rally_line`). The gathering markers (`indicators/gathering.rs`, design change 0201 point 4,
  slice M3-J3; placeholder looks, each a shape and a colour of its own, `Indicator::Gathering` with a
  `GatheringMarker`) take the Move marker's place: the Gather marker, cyan corner brackets round a Gather's source;
  the Return Resources marker, a gold ring with four arrowheads pointing in, round a Drop Off's drop-off; and the Set
  Drop-off marker, a wider dashed violet ring round each selected worker's AssignedDropOff (`drop_off_indicators`), as
  a rally point shows while its producer is selected (a reading: Set Drop-off applies at once and gives no flash; the
  design gives the marker, not when it shows). The pointer
  (`command_ui/cursor.rs`) shows the `PointerLook`: images drawn in code and cached per look, set as the window's
  `CursorIcon`, and drawn as a UI node at the injected cursor in QA runs, which render no system cursor; Mixed is
  striped in the Attack red and the Move green (`MIXED_COLORS`, `MIXED_STRIPE_PX`), GatherResources is the crystals'
  cyan (`GATHER_COLOR`) and ReturnResources gold (`RETURN_COLOR`), the Gather and Return Resources markers' colours. The invalid-target feedback
  (`command_ui/feedback.rs`) is the message `INVALID_TARGET_TEXT` above the HUD Bar for `FEEDBACK_TICKS` sim ticks
  after the latest `InvalidTarget` (`TargetFeedback`), never in the pointer's way (`Pickable::IGNORE`). The same
  band gives the insufficient-resources feedback (0083): `NO_CRYSTALS_TEXT` after the sim's `ProductionRefused` for
  a local player's producer that a Train didn't reach for want of crystals (a full queue gives none); the
  placeholder of design change 0143's notification, the army cap's name (`army_cap_text`: `UNIT_CONTROL_TEXT`,
  `BEACON_CAPACITY_TEXT`, `TUNNEL_SPACE_TEXT`), after the sim's `ProductionWaiting` for a local player's producer
  whose unit begins to wait for room (its InfoPanel shows "Waiting for Unit Control", "Waiting for Beacon Capacity"
  or "Waiting for Tunnel Space" while it does, `hud/info_panel/activity.rs`); a placement's (`placement_text`, the
  check's first problem, the Tunnel Area's three rules included): an
  `InvalidPlacement` or the sim's `PlacementRefused`; and `TARGET_TOO_CLOSE_TEXT` after the sim's `TargetTooClose`,
  or `LocationTooClose` (slice M5-O3), for a local player's unit (design change 0182).
- **Placement visuals** (`placement.rs`, `placement/`): while the local player places a building (`placing_now`: a
  game mode, AwaitingPlacement, a commandable selection). `PlacementGhost` holds the `Ghost` under the cursor (the
  footprint as turned, the `PlacementCheck` there now, the side letters of `sc_sim::sides`), worked out every frame;
  its one mesh is rebuilt in place, under one handle, only when that changed: each tile on the terrain's surface,
  green or red by its own problems, red throughout when the footprint breaks a whole rule (0093), and an outline
  green or red for the verdict; five `Text2d` labels carry the side letters (0079) and the building's tag, never
  hidden and never culled, only emptied (Bevy lays a `Text2d` out only while a camera saw a sprite or a text the frame
  before, and drops a change made in a frame without one: a text that changes must keep one in view). It lies
  over the map's other overlays, under the indicators: it is the cursor, so no pointer shows over the map
  (`PointerLook::placing`). `BuildAreaOverlay` shows `Sim::build_area`, the area only, as one mesh in the terrain's
  z band just in front of the fog layer (`overlay::AREA_Z`), but on the tiles the local player hasn't explored, which
  stay black (slice M3-J3, `BuildAreaOverlay::unexplored`), rebuilt only when the area differs from the one it last
  drew (`BuildArea: PartialEq`), its unexplored tiles changed or the map meshes were rebuilt (`MapView::builds`). The ghost's check is its
  builder's (`ghost::check_now`: `Sim::placement_check`, `warp_in_check`, `deploy_check` for the Drone a click
  would go to, or `build_check` for an Agent's Build; the last two by the player's knowledge, Reserved over a
  lingering ghost included); the overlay shows for a constructor's placement only. A Tunnel ghost (slice M4-J1) has
  its Tier 1 Tunnel Area drawn around it (`GhostArea`, `placement/ghost_area.rs`: `ShownArea::of`), green or red by
  the check's Tunnel Area problems alone (`area_is_clear`: the `WholeProblem::TunnelArea*`), on its explored tiles,
  just in front of the area overlays and behind the lingering ghosts (`GHOST_AREA_Z`). `LingeringGhosts`
  (`placement/lingering.rs`, `core/entities.md > Placement Validation > Lingering build ghosts`) draws the local
  player's pending worker builds (`Sim::pending_builds`, never another player's: Deploys, and an Agent's Tunnels with
  their Tier 1 areas, `lingering_sites`) in the game modes, whatever the interface: each area tinted on its explored
  tiles with its outline, then each site's footprint tinted on the terrain with an outline, one mesh rebuilt only when
  the sites, the areas' unexplored tiles or the map change, just in front of the build area overlay (`LINGERING_Z`).
  A ghost goes with its order, when
  the deployment or the site starts or the order is cancelled, replaced or fails. **The Tunnel Area display**
  (`TunnelAreaOverlay`, `placement/tunnel_areas.rs`; slices M4-J2 and M4-J3, `factions/syndicate/tunnels.md > Tunnel
  Area display`, design changes 0299 and 0300) shows, in the game modes, two things and never an enemy's area
  (neither seen nor remembered; the ghost still judges by the known ones, red with its reason), in:
  - **the selected areas** (`selected_areas`, a reading): the area of each own Tunnel in the selection, sites
    included, at its tier now (`tunnel::area::of`: an upgrading Tunnel's current area, not the one it reserves), in
    violet; it hides as the selection changes. An expansion's placement is its Tunnel's, selected all along (M4-F);
  - **a Tunnel ghost's rings** (`ghost_rings`, `tunnel_areas/rings.rs`): for each tier of the kind's table above the
    base, the cells its area adds to the tier below's (`Ring`: Tier 2's 36, Tier 3's 44), each ring in its tint.
    Red (`Ring::blocked`) are the cells that would block the upgrade (0242): off the map, not Drillable as far as the
    player knows (the sim's vision: an unexplored tile passes, as on the ghost, `Mode::Known`), or in an own area as
    the ghost's check counts them (`rings::own_areas`, a reading: `tunnel::area::reserved` of every own Tunnel and
    site, the upgrade's target tier's, and the Tier 1 areas of the own pending Tunnel ghosts but the placing Agent's
    without Shift, as `Sim::build_check` leaves them). Off-map cells can't be seen (the camera keeps to the map), so a
    red line along the map's edge marks where a ring runs off it (`rings::edge_marks`, a presentational reading).
    The Tier 1 area is the ghost's own (`GhostArea`).
  - **one mesh** under one handle, with the build area overlay's look (`overlay::tint_area`) and z, but on unexplored
    tiles and off the map, rebuilt only when the areas, the rings (as the ghost moves to another tile: it runs after
    `ghost::update_ghost`), their unexplored tiles or the map changed (`TunnelAreaOverlay::builds`); no view-mode
    toggles.

  **What the area displays share** (slice M5-J2, `placement/area_overlay.rs`), written once: the entity of one
  blended mesh under a reserved handle in the terrain's z band (`spawn_area_mesh`), its state (`AreaMesh`: rebuilt
  only when `is_stale`, what it shows or the map changed; `drawn` for its visibility, `show`), the unexplored tiles left
  out (`unexplored`), and the selection's own objects (`selected_own`). The tiles are drawn by `overlay::tint_tiles`
  (a tint per tile) and `overlay::outline` (an edge where the area ends), which `overlay::tint_area` combines. A new
  area display uses them rather than a copy. **The Recruitment Area display** (`RecruitmentAreaOverlay`,
  `placement/recruitment_areas.rs`; `factions/cults/structures/recruitment_hall.md > Recruitment Area display`,
  design changes 0350, 0351 and 0352, answering Q0107) shows, in the game modes, never an enemy's area:
  - **the preview of a Hall placed** (`ghost_preview`, the sim's `Preview::of`, after `ghost::update_ghost`): its 10 ×
    10 area tinted tile by tile by what the preview finds (`fill_of`) within its outline, and its Effectiveness as a
    `Text2d` north of the ghost (`EffectivenessLabel`, always visible and emptied rather than hidden, as the ghost's
    letters; "Effectiveness: 60%"). The claimants (`known_claimants`) are what the local player knows, never the hidden
    truth: their own completed Halls; the other players' completed Halls they see (`Sight::sees`) or remember
    (`Sight::remembered`, as last seen: a Hall destroyed unseen counts until its place is seen, one never seen doesn't,
    a site claims nothing), ranked as completed at tick 0 (Placeholder (not in the design: what the player knows holds
    no completion tick)); their own sites by id, then their pending Hall Builds by `PendingBuild::group` (Placeholder
    (not in the design: the sim keeps no placement time for a site, so the sites rank first)). Without Shift, the
    ghosts the placing Recruits alone hold (`command_ui::interface::recipients`) don't count, as in
    `Sim::group_build_check`;
  - **a selected own Hall's claims** (`selected_claims`): the tiles it claims in the game (`Sim::claimed_tiles`, what
    its InfoPanel's Effectiveness counts), in a selected Tunnel's violet; a site claims none;
  - its colours and z are its own constants in one place (`RECRUITMENT_AREA_Z`, just in front of the Tunnel Area
    overlay), its look the Tunnel display's.
- The design leaves some things open (labels, colours, the flash, the cursor's shapes and the Mixed look, the
  invalid-target and insufficient-resources feedbacks' form and duration, the targeting click allowance's value, when
  Shift counts for a targeting click, Shift with a rally point; the ghost's and the overlay's looks, the placement
  messages, the orientation a placement starts with, the hidden slots of buildings not yet in the game; the Warp In
  menus' labels, the lingering ghosts' look, a greyed-out button's messages); each placeholder is marked where it
  is made.

**Bevy code**
- Write one plugin per feature module (camera, map rendering, HUD, input, …). Keep systems small. Bevy caps a system
  at 16 parameters, and the prototype's UI systems hit that cap.
- Keep any state a test must set or inspect in a `Resource`, not a `Local`.
- Send one-shot happenings as sim events or Bevy messages, never as marker components that are added and removed.

**Client tests** advance an exact number of ticks deterministically, never by wall-clock time. Insert
`TimeUpdateStrategy::FixedTimesteps(1)`, so that each `App::update()` runs FixedUpdate exactly once and real time
advances by one tick. The very first update only starts the clock, so assert on `Sim::tick()`, not on the number of
updates. To draw between ticks, advance by part of a tick with `TimeUpdateStrategy::ManualDuration`. A client run's
`LoadedScenario` (whose setup holds the game options) and `RecordedInputs` are a `Replay`: playing it back in a bare
`Sim` gives the same `state_hash` at every tick. `tests/replay.rs` records games through the client and plays them
back (`AppConfig::record`, `AppConfig::playback`).

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
| `--scenario <name>` | Starts a built-in scenario (`sc_sim::scenarios::NAMES`) with its default setup. Default `dev_skirmish`. |
| `--map <file.json>` | Plays a map file (a `MapDef`, see Sim scenarios and tests) instead, with the default setup: a GDO player in every slot. Excludes `--scenario`. Exits with code 2, listing the problems, if `Sim::try_new` rejects it. |
| `--factions <f1,f2,...>` | The factions of the players in slots 1, 2, … (`gdo`, `colonists`, `syndicate`, `cults`, any case), for the default `--scenario`, a named one or a `--map`; the slots not named stay GDO (`GameSetup::per_slot`). More factions than slots is an error. Not with `--editor` or `--replay` (a replay records its setup). |
| `--editor [file.json]` | Opens the map editor (see Map editor) on that file, or on a new map. |
| `--editor-script <file.json>` | Applies a JSON list of `EditorAction`s in the editor's first frame. |
| `--player <n>` | Shows player `n`'s view: camera start, top bar. Default 1. In the editor: the slot play-tests start as. |
| `--window-size <W>x<H>` | Window or offscreen image size in physical pixels. Default `1280x720`. |
| `--fast` | Runs as many ticks per frame as fit in 50 ms instead of 16 per second. Screenshots are unchanged. |
| `--no-debug` | Runs the game with debug commands off (the setup's `SimConfig`), as a real match will; they are on by default. |
| `--screenshot <tick>:<path>` | Saves a PNG of exactly that tick. Repeatable, in any order; creates parent dirs. |
| `--exit-at-tick <n>` | Exits at tick `n`, after its screenshots. Windowless runs default to the last screenshot's, or else to the `--replay`'s end. |
| `--dump-state <path>` | Writes the state as JSON when the run ends (fields below). |
| `--commands <file.json>` | Applies player commands at given ticks: a JSON list of `{"tick", "player", "command"}` (`qa::TimedCommand`; `command` is a `sc_sim::Command` in serde form). Each applies in the step that reaches its tick (1 or later), a tick's commands in file order. Not with `--editor`. |
| `--input-script <file.json>` | Drives the virtual keyboard and mouse (`InjectedInput`) at given ticks: a JSON list of `{"tick", "cursor", "press", "tap", "release"}` (`qa::InputStep`; buttons by name: `MouseLeft`, `ControlLeft`, `Digit1`, `KeyQ`, `F2`, …). The sim waits at each step's tick until it is applied, so a button held across steps drags and a chord spans steps (Ctrl + 1: `press` Ctrl and `tap` 1, release Ctrl in a later step). Not with `--editor`. |
| `--show-paths` | Starts with the unit path overlay on (P toggles it). |
| `--no-fog` | Starts with the fog of war off: the client shows everything (F3 toggles it). Display only; not with `--no-debug`. |
| `--windowed` | Shows a QA run in a window on the user's display instead of rendering offscreen. |
| `--headless` | No window and no renderer: sim, client state and `--dump-state` only. Needs `--exit-at-tick`. |
| `--replay <file.json>` | Plays a recorded game back (see Replays and golden hashes) with its own map, players and game options: not with `--scenario`, `--map`, `--factions`, `--editor`, `--no-debug` or `--commands`. Pauses at the end, so no tick of the other flags may come after it. The other QA flags work with it. |
| `--record <file.json>` | Records the game to that file. A game is recorded to `replays/` anyway; a QA run only with this flag. Not with `--editor` or `--replay`. |
| `--no-record` | Doesn't record the game. |

Taking a screenshot (no window appears; about one second plus `tick / 16` seconds, or add `--fast`):

```sh
cargo run -- --scenario dev_skirmish --player 2 --screenshot 1:/path/to/p2.png --dump-state /path/to/p2.json
```

- Any of `--screenshot`, `--exit-at-tick`, `--dump-state`, `--commands`, `--input-script` makes a *QA run*. It
  ignores keyboard and mouse, renders at scale factor 1, and renders offscreen: Bevy renders into an image on this
  machine's GPU (Vulkan, Intel Arc), with no window and no Xvfb. The sim waits at each screenshot tick until the PNG
  is saved, so the PNG shows that tick's exact state with no interpolation. Repeated runs give byte-identical PNGs.
- Exit codes: 0 when everything was saved, 1 when a QA step failed (the log says which), 2 for invalid flags.
- Once the game is over the sim steps no more: the screenshots, input steps and exit scheduled after its last tick
  happen at that tick (`qa::stop_at_game_end`, logged).
- The `--dump-state` JSON holds `tick`, `scenario` (the map's name), `players`, `objects` (every field of
  `sc_sim::Object`), the last tick's `events`, `client` (`local_player`, `camera_center` as a view point in su,
  `grid_visible`, `fog_of_war`, `recorded_ticks`, the local player's `selection` and `last_known` records, the
  `recording` file, and in a playback `replay`: its `file`, `end_tick`, number of `checkpoints` and the `desync`, if
  any) and `map` (the terrain).
- `--windowed` runs turn vsync off: with the display asleep, X11 paces a vsynced window at one frame per second.
- Verify visual work yourself. Capture screenshots and look at the PNGs with the Read tool before you report.
- To look at another part of a map, give the slot a different camera start in the editor (or the file's
  `camera_start`) and pass the file with `--map`. In an `--input-script`, pan with middle-button drags (a press, a
  cursor move, a release: the view moves by the pixels dragged, clamped to the map): the arrow keys and the screen
  edge pan by real time, which `--fast` makes too short to rely on.
- To put units on a map in a QA run, spawn them with debug commands in a `--commands` file at tick 1 and order them
  from tick 2. Spawned objects take the ids after the starting objects, in file order (`--dump-state` lists them):

  ```json
  [{"tick": 1, "player": 1, "command": {"Debug": {"Spawn": {"kind": "Peacekeeper", "owner": 1, "at": {"Point": [1216.0, 1376.0]}}}}},
   {"tick": 2, "player": 1, "command": {"Order": {"units": [18], "kind": {"Move": {"Point": [2016.0, 1376.0]}}, "queue": false}}}]
  ```

## Replays and golden hashes

A replay file is `sc_sim::Replay` as JSON: `scenario` (the map in its file format and the game setup, game options
included), `inputs` (the `TickInputs` of every tick: `inputs[t]` is what the step from tick `t` to `t + 1` applied)
and `checkpoints` (`{"tick", "hash"}`: `Sim::state_hash` at the end of that tick, ascending). The client writes it
with one line per tick (`replay::replay_json`), as the golden files are; a file without `checkpoints` still plays,
unchecked.

- **Checkpoints** (`sc_sim::replay`): `Replay::record_checked` notes one at tick 0 and every `CHECKPOINT_INTERVAL`
  (64) ticks, and `Replay::checkpoint` one at the last tick (the client, whenever it writes the file).
  `Replay::verify(&sim)` compares a sim played back to some tick with that tick's checkpoint, and
  `Replay::run_checked` plays a whole replay that way. A `Desync { tick, expected, actual, previous }` names the first
  checkpoint that differs and the one before it, which matched: the game diverged in between.
- **Recording** (`crates/game/src/replay/record.rs`): every game the client plays is recorded, not the map editor, its
  play-tests or a playback. The file is `--record`'s, else `replays/<UTC date>_<time>_<map>.json` under the working
  directory (git-ignored; `_2`, `_3`, … when taken). It is written when the game starts, every `AUTOSAVE_TICKS` (160)
  ticks, at the end of the game (complete, with a checkpoint of its last tick: `Recorder::is_finished`,
  `core/game_rules.md > End of Game`) and when the app exits (window closed, Ctrl+C, a QA run's end), atomically
  (`files::write_atomically`), so a crash loses a few seconds at most. QA runs record only with `--record`;
  `--no-record` turns recording off. Should the sim be replaced (`SimRes::generation`), the recording ends there.
- **Playback** (`--replay`, `crates/game/src/replay/playback.rs`): the game starts from the replay's scenario with its
  game options, and each step applies the recorded inputs of its tick and nothing else. The local player can't give
  commands: the command UI's input set (`CommandUiSystems::Input`) doesn't run, and anything else pending is dropped.
  It pauses at the end, or where its game ended (a recording made outside the client may go on after it). The game
  is checked against the checkpoints at tick 0 and after every step; the first desync
  is logged as an error, fails a QA run (exit code 1) and shows in red in the top bar, and the playback goes on.
  Controls (Game context, placeholder keys): Space pauses and resumes, `.` plays one tick and pauses, `-` and `=`
  step through the speeds ×0.5, ×1, ×2, ×4 and max (`playback::SPEEDS`; `--fast` starts at max), F2 switches the
  player and F3 the fog, also when the recorded game had no debug commands (the selection's and the fog's own
  systems allow them while a `Playback` runs). The top bar shows "Replay x2", ", paused"
  or ", end", and "Tick n / end". Screenshots, `--exit-at-tick`, `--dump-state`, `--input-script` (to press the
  playback keys), `--windowed` and `--headless` work with it; a windowless playback ends at the replay's end unless
  told otherwise. To check a replay quickly: `cargo run -- --replay <file> --headless --fast`.
- **Golden hashes** (`crates/sim/tests/golden/`: the games in `units.rs`, `gdo.rs`, `colonists.rs`, `syndicate.rs`,
  `cults.rs` and `main.rs`, the recording and the comparison in `harness.rs`): thirteen games, a 3v3 duel, a 40-unit
  AttackMove battle, a 12-unit group move through a choke, a patrol that meets an enemy, a chase lost in the fog, three
  Peacekeepers that raise their Riot Shields (`Command::Ability`; one turns back half-way), walk up to a Guard under
  fire and lower them to kill it, and `gdo_opening` (slice M2-J1: `dev_skirmish` from its real start, to 2:45; player 1
  plays the fastest clean Barracks and rallies ten Peacekeepers to player 2's crystal field, where they destroy four
  plates, while player 2 plates its field and builds Generators), the Colonists' `colonists_opening` and
  `colonists_vs_gdo` (slice M3-K1, below), the Syndicate's `syndicate_opening` and `syndicate_vs_gdo` (slice
  M4-L, below) and the Cults' `cults_opening` and `cults_vs_syndicate` (slice M5-L, below), are blessed as replays
  with a checkpoint every 32 ticks and at the end, in `crates/sim/tests/golden/<name>.json`. A tenth, `area_attacks` (slice M4-O), is a unit test
  (`combat/projectiles/golden.rs`), as it plays test throwers (`combat::testing`): grenades in flight, blasts over a
  hidden Hideout and a group, an Attack Ground. Its file is `tests/golden/test_defs/area_attacks.json`, out of the
  client's playback of `tests/golden/*.json`, which knows only the game's types. The test plays each game from its definition, checks that the
  definition still is the blessed one, and compares the checkpoints; a mismatch fails with the first diverging tick
  and the last one that matched. Each game also asserts what it is about, at its end (`expect`: one side of the duel
  is gone, the group passed the choke, …) and along the way (`during`: the shielded Peacekeepers held their fire), so
  a re-bless can't hide a broken game. `gdo_opening`'s players play by script (a `Script`, here two
  `testing::build_order`s), which gives their commands before each step from the game as it is and checks what it
  saw at the end: both players' books balance every step, the patch of a destroyed plate keeps what it held until a
  plate covers it again, player 2's income falls (210 crystals in the 9 s before its first plate fell, 130 in the
  last 9) and it is not defeated. Its commands are recorded like the others', so the file plays back without the
  script; as they follow the game, they are compared after the checkpoints, not as its definition. The Colonists'
  games play `dev_skirmish` with player 1 as the Colonists, by `testing::colonists::skirmish`: in
  `colonists_opening` (to 1:50) player 1 gathers, trains Drones, deploys, warps in a Bio Portal on its first Beacon,
  trains two Sabre Legionnaires and converts the first; in `colonists_vs_gdo` (to 3:00) it trains eight Sabres and
  converts each to a Rifle Legionnaire to wait east of the Bio Portal, while player 2 plays `gdo_opening`'s Barracks
  line and sends its ten Peacekeepers to player 1's field in two waves of AttackMove: the Colonists' Shields take
  hits and units die on both sides. Both players' books balance every step, the Drones' deliveries included, and
  every command the scripts give is one the gate takes. The Syndicate's games play `dev_skirmish` with player 1 as
  the Syndicate, by `testing::syndicate::skirmish`: in `syndicate_opening` (to 2:15) its Agents gather the patch 3 GU
  from the crystal hatch, the Hideout, rallied on it, trains four more, the third of them builds a second Tunnel
  with the starting Supply, is ejected by it and gathers Supplies, and the first Tunnel expands an Optics Lab; in
  `syndicate_vs_gdo` (to 3:07.38, where it ends) six Guards follow behind the Optics Lab, the Hideout's rally point
  cleared so that they stay in the Tunnel Network, four of them ejected by the second Tunnel to hold the top of the
  plateau's west ramp, while player 2 plays the same Barracks line with sixteen Peacekeepers, sent in waves of six to
  the Syndicate's Tunnel nearest them: Guards and Peacekeepers die, the second Tunnel falls, then the first, which
  leaves the Hideout and the Optics Lab neutral, strands the two Guards in the network and defeats player 1; the game
  is over in its last tick, as the client's playback stops where a game ends. Both players' books balance
  every step, and player 1's Supplies too (`testing::syndicate::SupplyBooks`). The Cults' games play `dev_skirmish`
  with player 1 as the Cults, by `testing::cults::skirmish`: in `cults_opening` (to 2:45) its Recruits are dealt to
  the field, and the line builds a second Storage, a second Hall east of the hill (100% Effectiveness) and an Armory,
  and trains four Militants; in `cults_vs_syndicate` (to 2:58.31, where it ends) it builds the Armory first, then a
  Gunworks, and trains eight Militants and three Gunners (slice M5-N1a's first step) of the Recruits it has no seat
  for, which attack-move in a wave to player 2's Tunnel, while player 2 plays the Syndicate: two Agents, an Optics Lab,
  a Laser Turret from the Tunnel's ExpandMenu (slice M5-Y) and four Guards at its patch. The turret kills four
  Militants before it is destroyed, and then the Tunnel falls, which defeats player 2. Both players' books balance every step, player 2's Supplies too. The ignored
  `tests/skirmish_soak.rs`
  (slice M3-K2) plays the Colonists' two players for 10 minutes, twice, until player 1's field runs out: the
  movement invariants and the books every tick, no dangling id, no order stuck in one step beyond a bound (a waiting
  Drone re-checks within 16 ticks once its patch is free), the field's 12000 crystals all delivered, no game end, and
  the hash every 64 ticks equal over both runs and the replay
  (`cargo test -p sc_sim --release --test sim skirmish_soak -- --ignored --nocapture` prints what it saw: the longest
  step of each order, each Drone's longest gap between deliveries, the losses). Any change to the sim's rules or
  state changes the hashes:
  when the change is intended, re-bless and commit the files with it:

  ```sh
  SC_BLESS=1 cargo test -p sc_sim golden
  ```

  The suite runs in well under a second in debug. The golden files are replays, so the client plays them:
  `cargo run -- --replay crates/sim/tests/golden/duel_3v3.json` (the camera starts over the map's middle, or where
  the game sets it: the Colonists' games start it over player 1's base).

## Map editor

`cargo run -- --editor [file.json]` opens the map editor (`crates/game/src/editor.rs` and `editor/`). It edits a map
(`MapDef`) and saves it as a map file (see Sim scenarios and tests). F1 in the editor lists the mouse and keys.

- **Starting**: with a file that exists, the editor opens it. With a file that doesn't exist yet, it starts a new map
  that Ctrl+S saves there. Without a file, it starts a new map with the creation dialog's defaults (Skirmish Default,
  64×48, 180° rotation, 2 slots), and `--editor --scenario <name>` opens a built-in scenario's map. A file that
  doesn't parse exits with code 2. A file that parses but fails validation still opens, with its problems listed, so
  that it can be fixed; `--player` is not checked against it (play-tests fall back to the first slot).
- **Files**: maps live in `maps/` at the repository root (`editor::files::default_maps_dir`), and new maps are saved
  as `maps/<name>.json`. Saves are atomic: `files::save` writes a hidden temporary file next to the target (named
  after the process too, so that two processes writing one file never write into each other's), flushes it and
  renames it over the target. Replays are written the same way (`files::write_atomically`). `maps/dev_skirmish.json` is the built-in `dev_skirmish` saved by the editor. A
  test checks that every map in `maps/` is playable and that this one still matches the built-in.
- **Actions**: every change is an `EditorAction` (`editor/actions.rs`), queued in the `EditorActions` resource and
  applied in `EditorSystems::Apply`. The input layer's actions, the UI's buttons, the text fields and the window's
  close button only queue actions. The edits are plain functions (`editor/terrain.rs`, `editor/objects.rs`,
  `editor/presets.rs`, `editor/document.rs`) and sim-side helpers (`Map::resize`, `Map::connected_region`,
  `MapDef::resize`, `add_slots`, `remove_slots`, `regenerate`).
- **Input** (`editor/input.rs`): the editor reads only the shared input layer (see Client architecture > Input), in
  the editor's input context. Its keys are `InputAction::Editor(EditorInput)` entries of `Bindings`, rebindable like
  the game's, and `editor_keys` turns the pressed ones into `EditorAction`s (`editor_action`). `editor_pointer` turns
  the `Pointer` into the pointer's actions: its ground point into `Hover`; the `UseTool` press (left button,
  `Pointer::press`) into `Press`, a `Drag` every frame to `MapPress::drag_ground` (over UI nodes on the view too, not
  off it) and `Release`; `Eyedropper` clicks (right button, `MapClick`) into `Pick`; and the `Zoom` wheel
  (`ActionState::scrolled`) into `Zoom` at the cursor. Over the UI nothing starts, by the layer's over-UI test (the
  F1 help is a full-window UI node). While a text field has the keyboard no binding fires. The middle-mouse grab,
  the arrow keys, G and Esc are the shared actions.
- **Document**: `EditorDoc` holds the map, its file and the undo history, with one snapshot per completed action (a
  whole stroke, drag or field edit). Its dirty flag is worked out once per change, not per frame. When the document
  changes, the editor replaces the sim with `Sim::preview` of the map with its default setup. That is a sim at tick 0
  that shows every spawn that fits on the map, even when the map is invalid, so the game's map and object rendering
  show the document.
- **New maps** (`editor/creation.rs`): New opens the creation dialog (`NewMapDialog`, a modal panel), which edits a
  `MapSettings`: type, size, slot count, symmetry, name and description, with the placeholder limits shown and
  enforced (see Sim scenarios and tests). It offers only the slot counts and symmetries the other settings allow, and
  Create stays disabled until `MapSettings::check` passes. While it is open, other edits are refused; Cancel or Esc
  closes it. The map type and the symmetry are fixed once the map exists, and shown as such in the properties panel;
  a 90°-rotated map can only be resized to another square.
- **Map types** (`core/map_editor.md > Map Types`): on a Skirmish Default map, objects outside the Starting Layouts
  are neutral, so the editor offers no owner and refuses owned ones (`PlaceProblem::OwnedOnSkirmishMap`). Custom maps
  have no restrictions. A new Skirmish Default map lists its empty GDO, Syndicate and Colonists layouts as problems
  (`LayoutMissing`), so it can't be play-tested until they are placed with the Layout tool (see Starting Layouts
  below).
- **Starting Layouts** (`editor/layouts.rs` and `layouts/`, slice M2-I; `core/map_editor.md > Starting Layouts`,
  design changes 0008, 0009, 0012 and 0097): the Layout tool (T) edits the layout of `EditorTools::faction` at
  `EditorTools::slot`, an editable slot (the generated ones follow it, turned to match). A click places
  `EditorTools::layout_kind` turned as `EditorTools::orientation` says, or grabs an object of the layout to drag it;
  Delete removes the selected one (`Selection::LayoutObject`); each is one undo step. The palette's Starting layouts
  section (`ui/starting.rs`) picks the slot, the faction and the kind, says what the layout lacks or that it is
  complete, and flags outdated layouts (`layouts::outdated_text`); choosing the tool scrolls the palette to it.
  - **What it offers** (`layouts::palette`): on a Skirmish Default map the faction's Starting Conditions with how many
    the layout may still take (a placed kind moves the tool on to the next one missing); on a Custom map every kind a
    player may own. Every faction has Starting Conditions (plan decision T63), so every faction has a palette.
    Structures snap to tiles and units, such as the Colonists' Drones, go at the point clicked (`objects::snap_spawn`;
    `tests/editor/colonists.rs`). The Syndicate's palette is its Tunnel, Hideout and Agent (slice M4-B); a Tunnel
    whose Tier 1 Tunnel Area reaches off the map or onto undrillable ground, or a Hideout outside that area, shows a
    red ghost and is refused with the rule's text (`rule_text`; `tests/editor/syndicate.rs`). The Cults' is their
    Recruitment Hall, Storage and six Recruits (slice M5-B), and an incomplete Cults layout lists the Recruits it
    lacks and keeps the map from play-testing (0012; `tests/editor/cults.rs`).
  - **Checks** (`layouts/check.rs::check_object`): the editor's own rules first (the Starting Conditions' kinds and
    counts on a Skirmish Default map, the editable part, no overlap with the object's own image), then the sim's:
    `MapDef::validate` of the map with the object in place, keeping only that object's problems. The sim is the
    oracle; the editor has no placement rule of its own, so no sight checks or warnings (0097). The ghost
    (`LayoutPreview`, recomputed only when the document, the tools, the cursor or the gesture changed) shows the
    verdict, red on the refused tiles, and the status line says why.
  - **Overlay** (`overlay/layouts.rs`): while the tool is on, the other factions' layouts at the slot are drawn
    dimmed, the edited one outlined in the slot's colour, its Deployment Center's build area outlined, and the ghost
    with its images; the preview's sim plays the edited faction in every slot (`layouts::preview_scenario`), so it
    shows that faction's layouts. Layout objects with a problem are outlined in red with every tool.
  - **Problems** (`layouts/problems.rs::problem_lines`): the Problems list holds everything that keeps a play-test
    from starting (`PlayTestSetup::problems`: `Scenario::validate` of the map with the play-test setup, listed again
    whenever the document or the setup changes, `sync_problems`), so two slots' layouts in each other's way
    (`LayoutOverlap`) show as soon as the map has no problems of its own, not only when F5 is refused (slice M2-J2).
    It gives each layout that lacks objects one line that names them all and the tool ("S1 GDO layout: add Deployment
    Center, Extraction Facility, Extraction Plate x2 (Layout tool, T); S2 follows"), folding the generated slots'
    identical lines into their editable slot's. A click on a line chooses the slot, the faction and the first missing
    kind, or selects the broken object. A refused play-test (F5) shows the first line.
  - **Outdated layouts** are derived (T11): a non-empty layout that isn't its faction's current Starting Conditions
    is flagged, so one still being placed is too ("Outdated or unfinished"); on a Custom map the line only says that
    the layout differs. Nothing is fixed automatically.
- **Symmetry**: the editor edits only the editable part (`MapDef::region`); the locked part is dimmed
  (`overlay/tints.rs`), and the generated slots' markers are drawn faded. Every edit of the locked part is
  refused with a status message that names it: strokes and fills there, placing, moving or selecting what lies there,
  and the generated slots' markers and values. After every action (`Editor::regenerate`), `MapDef::regenerate`
  rebuilds the locked part when the document changed; a selected object keeps its selection through it. Slots are
  added and removed in whole sets. An object that would overlap its own image is refused
  (`PlaceProblem::OverlapsOwnImage`, the placeholder rule). A file whose locked part differs is listed as
  `NotSymmetric` until its first edit regenerates it. The placement previews show a ghost at each image.
- **Custom tile presets** (`editor/presets.rs`, `core/map_editor.md > Custom Tile Presets`): Add custom preset makes
  "Custom n" with Plane's properties and look and chooses it; the chosen preset's name, properties and look are edited
  in the palette, each one undo step. The default presets are fixed (shown in one line), and a preset some tile still
  uses can't be removed (the status line says how many tiles). `MAX_CUSTOM_PRESETS` and `MAX_PRESET_NAME_CHARS` are
  placeholders.
- **Modes**: `ClientMode` is `Game`, `Editor` or `PlayTest`, and only the HUD, the editor and the input layer (which
  takes the game's bindings in a play-test, the editor's in the editor) look at it. F5 starts a play-test of the
  document with the play-test setup (`PlayTestSetup`, the Play-test panel): each slot's faction, which slot you play,
  the seed, and debug commands on. It is not part of the map. `Sim::try_new` must accept the map; the game runs at
  the game's zoom. Only F5 returns (Esc is the game's Cancel), to the editor with the document, its history and the
  view unchanged.
- **Unsaved changes**: New, Open, Quit and the window's close button (`WindowCloseRequested`, through
  `editor/input.rs::close_requests`; `platform::ConfirmClose` keeps the platform from exiting by itself) ask once with
  a status warning and go ahead when asked again, also during a play-test.
- **Positions**: every position in an `EditorAction` is a ground point (su), the one drawn under the cursor
  (`coords::ground_under`). `CenterOn` centers the view on where its point is drawn, and `Zoom { at }` keeps that
  drawn point where it is on screen.
- **Terrain tools** (`editor/terrain.rs`): Paint and Fill set presets and keep the heights; a Traversible preset is
  refused, with a status warning, on tiles more than 1 level steep (`terrain::refuses`). The Height tool works on
  corners: brush N covers N × N tiles and changes their (N + 1)² corners. Set gives them a level; Raise and Lower move
  each one level, once per stroke however often the brush passes. `Map::set_corner_heights` then cascades the slopes,
  so a height edit never leaves a too-steep tile. It may slope the ground under a resource: that is allowed and
  listed as a problem. The overlay previews each dab: the tiles, the corners it sets (white) and the ones the cascade
  moves (amber), from where they are drawn now to where they will be, and in red the tiles a paint would refuse.
- **Objects**: Place refuses spots off the map, overlapping another footprint or its own image, in the locked part,
  or, for resources and structures, on a sloped tile (the ghost turns red). A plate may go onto a patch that no plate
  covers, the one overlap the sim allows (`ObjectKind::may_overlap`). Structures are placed turned as R, Shift+R, F
  and Shift+F (or the Turn buttons) turned them while the Place or the Layout tool holds one (`editor/orientation.rs`:
  a quarter turn either way, a flip left and right or top and bottom, as seen; `EditorTools::orientation`), their
  footprints and ghosts turned with them; the same keys as the game's placement ghost. New objects get
  `SpawnSettings::defaults`; selecting one shows its values as text fields (the interval in seconds), which commit
  `SetSpawnSettings`, one undo step each.
- **View**: G toggles the grid, K the slope tint (flat green, sloped amber, too steep red, in the terrain's z band).
  The middle-mouse grab pans as in the game. The status line shows the tile under the cursor with its preset, the
  interpolated height there, its level, whether the tile is flat, sloped or too steep, and whether it is locked.
- **Selection** is a spawn index, a slot's marker or a layout object (slot, faction, index). An undo, a redo or a
  resize may shift the indexes, so they clear the selection; so does switching documents.
- **UI work follows changes** (review finding): each list (`ui/lists.rs`) is rebuilt only when what it shows
  changed, keyed so that a field being typed into survives marker moves and preset edits; the buttons are restyled
  when the editor's state changed or the pointer moved over them; the texts and fields refresh only when their values
  changed. Text fields commit in `EditorSystems::Input`, before the frame's actions, so a click elsewhere (Save, a
  tool, another field) applies the typed text first; Enter commits, Esc gives up the text.
- **Tests** (`crates/game/tests/editor/`: `session`, `edits`, `map_edits`, `creation`, `symmetry`, `review`, `input`,
  `units`, `layouts`, `colonists`)
  build a headless app with `AppConfig::editor`, push `EditorAction`s into `EditorActions`, call `app.update()`, and
  assert on `EditorDoc`, the sim, the view and the UI state. Positions are ground points, such as
  `TilePos::center()`. Tests of the input itself (`input`: keys, pointer, wheel, over the UI, typing) go through the
  layer's `InjectedInput` (`tap`, the virtual mouse, cursor and wheel). Never inject Bevy's raw input, except a text
  field's own keys (Enter, Esc) and the window's close button (`WindowCloseRequested`).
- **QA**: `--editor-script` applies its actions in the first frame, before any screenshot. The editor's sim stays at
  tick 0, so take screenshots at tick 0 unless the script play-tests (`"PlayTest"`).

```sh
echo '["FitMap", {"SetTool": "Paint"}, {"Press": [100.0, 80.0]}, "Release"]' > /path/to/edits.json
cargo run -- --editor maps/dev_skirmish.json --editor-script /path/to/edits.json --screenshot 0:/path/to/editor.png
```

- **UI**: bevy_ui with Bevy's `bevy_ui_widgets` feature (headless `Button`, `TextInput`, `ScrollArea` and
  `Scrollbar`) and `ui_picking`. Both side panels scroll with the wheel and with a scrollbar along their inner edge
  (`ui/layout.rs::spawn_side_panel`); the right one lists the problems before the play-test's slots, which run long
  on maps with many slots. Its font has ASCII glyphs only, so UI texts stay ASCII ("Rotate 90", not "90°"). The
  player slot colours, the overlay, locked-part, slope tint, dimmed-layout and build-area colours, the scrollbar's,
  UI sizes, `NEW_MAP_SIZE`, `MAX_DESCRIPTION_CHARS` and the sim's creation limits are placeholders.

## Adding a unit or structure

One kind (plan decision T54, slices M4-R1 and M4-R2): its own files, and one line in each registry.

1. **Its def module**, `crates/sim/src/defs/<faction>/{units,structures}/<object>.rs`, named as the design's file of
   the object (`factions/<faction>/{units,structures}/<object>.md`; an economy building's is
   `factions/<faction>/economy.md`, and `defs/map/` holds the map's objects). Its doc comment cites that file. It
   holds the kind's `ObjectDef`, with every number the design gives the object, each cited, and the object's tests.
   What the type does differently goes in a field of its def, which the logic reads, rather than in a `match` on its
   kind. Then its line in the sim's kind registry (`kind_registry!` in `defs.rs`), **at the end**: that one line makes
   the `ObjectKind` variant, its place in `ObjectKind::ALL` and its arm of `ObjectKind::def`; the state hash numbers
   kinds in that order, and maps and replays name them by their serialized names, which must not change. A rule of
   its faction rather than of the type goes in `defs/<faction>.rs`. Then *Object model > Adding an object type* for
   its capabilities, and *Adding a capability* for anything new it does. What M4's kinds put in their defs, as
   examples: how a structure is built, `StructureDef::site` (a `SiteDef`: a worker builds it, the Tunnel) or
   `StructureDef::expansion` (an `ExpansionDef`: a Tunnel builds it in its area, the Hideout, the Optics Lab, the
   Chemical Plant and the Laser Turret, the turret's one way since design change 0359), with `StructureDef::underground`
   for an expansion; what must stand first, `StructureDef::requires` for a structure and `Recipe::requires` in its
   producer's `ProductionDef` for a unit (the Guard and the Grenadier, slice M4-K); a structure's weapon,
   `StructureDef::attack`, and its pop-up form, `StructureDef::popup` (the Laser Turret, slices M4-S1 and M4-S2; every
   object of such a type a pop-up since slice M5-Y). A new expansion also turns its `ExpandEntry::Planned` into its
   `Kind` in `defs::syndicate::EXPAND_MENU`, and a new Syndicate unit joins `EJECT_MENU_ORDER` there.
2. **Its client module**, `crates/game/src/kinds/<faction>/<object>.rs` (*Client architecture > Kinds*): its
   `ClientKind`, with
   - its label, two letters of its own (`kinds::tests::every_label_is_its_own` checks them against every other
     label, the planned buttons' too);
   - its look (`object_render::Look`): a unit's body shape and whether it carries its label, a structure, or a
     structure that attacks (`Look::Turret`). A new shape adds a `Look` or `UnitShape` variant and its drawing, in
     `object_render/`;
   - its shots' style (`combat_effects::ShotStyle`) if it has a weapon, none if not
     (`kinds::tests::every_kind_module_fits_its_def` checks both against its def);
   - its panel template. A shared one: `PanelTemplate::of(InterfaceTemplate::BasicCombatUnit)`, or `Production` for a
     producer, whose grid comes from its `ProductionDef`, or `PoolProducer` for a pool building's (slice M5-F2, the
     Armory's `templates/cults/armory.rs`, its KindMenus from its `ProductionDef`). `PanelTemplate::NONE` until its
     slice builds its panel.

   Then its line in the client's registry, `kinds::of`: the `match` is exhaustive, so the compiler names a kind
   without one.
3. **A panel of its own**, `crates/game/src/command_ui/interface/templates/<faction>/<object>.rs`: its `TEMPLATE`, its
   grid and menus, and its `InterfaceTemplate` variant, with its arms in `InterfaceTemplate::grid` and in the
   right-click (`command_ui/resolution.rs`). A unit or building of its panel that isn't in the game yet holds its slot
   (`PanelTemplate::planned`, `PanelCommand::Planned`, with the type it will require in `Planned::requires`), and a
   command the design names after the object has its name there (`PanelTemplate::names`). A menu whose entries
   follow a design list in grid order takes the list from the sim's data (`sc_sim::defs::EJECT_MENU_ORDER`, and the
   ExpandMenu's `EXPAND_MENU`), so a new unit or expansion joins it there. A builder also needs its `Builder`
   (*Adding a faction*, step 7). Of M4's eight kinds, three have a module here: the Tunnel, the Agent, and the
   Hideout (the Production template with the design's letters); the Guard and the Grenadier take the BasicCombatUnit
   template, and the Optics Lab, the Chemical Plant and the Laser Turret have no commands (`PanelTemplate::NONE`; a
   site shows its Cancel, `SITE_GRID`).
4. **Where numbers go**:
   - the object's own numbers in its def, as fields, not in logic files;
   - a number a system uses for every kind in that system's tuning module (*Tunables*);
   - the client's feel numbers in `crates/game/src/tuning/`;
   - colours and drawing sizes with the drawing code (`object_render/`, `combat_effects/`, `command_ui/indicators/`).

   Each has a design citation, or "Placeholder (not in the design)".
5. **Tests**: its def's in its module; a scenario test of what it does in `crates/sim/tests/`; the client's, if it
   has a panel, beside the other panels' (`crates/game/tests/commands/`).

## Adding a faction

What a faction needs before players can start with it, in the order M3 (the Colonists) built it. Started by slice
M3-C and finished by M3-K2 from what the M3 slices needed (plan `artifacts/director/m3-plan.md`, the Built entries of
`artifacts/director/plan.md`); reviewed against M4 (the Syndicate) by slice M4-M, which added items 15 to 18 and M4's
examples.

1. **Defs** (slice M3-A, laid out by M4-R1): the faction's root module, `defs/<faction>.rs`, with its own rules (its
   caps and divisors), and a def module per kind under `defs/<faction>/`, each with its line at the end of the kind
   registry (the state hash numbers kinds in that order), its `static ObjectDef` with `faction: Some(..)` and, for a
   unit, its army-cap cost in its `UnitDef`; every exhaustive match over `ObjectKind` (sim and client) decided. Each
   kind has its client module (*Adding a unit or structure*), and the editor's unit palette lists the faction's units
   by itself. A cost may share another faction's field when it means the same: the Cults' Unit Control cost, the
   Recruits a unit takes, is `unit_control_cost` (slice M5-A, plan decision T55), whose `defs_are_consistent` check
   names the factions that have one.
2. **Banks**: a resource the faction stores is a `Player` field (`u32`, hashed in `hash.rs`, `#[serde(default)]`),
   in `Banks` and `DebugCommand::SetResources` (`serde(default)` there too, so older command files load). Re-bless
   the goldens (see *Replays and golden hashes*). The Syndicate's Supplies (slice M4-I) took no new field: the bank
   was there, and what was new was gathering it (item 12's workers, from a neutral station).
3. **Starting Conditions** (`scenario/starting.rs`): a `static StartingConditions` table and its arm in
   `StartingConditions::of`. A game then starts the player's banks from it (`Player::starting`), and the camera on
   its primary structure (`Scenario::primary_structure`).
4. **Layouts**: from then on every Skirmish Default map needs the faction's layout at every start (plan decisions
   T11, T24, T63): `scenarios::dev_skirmish_map` (slot 1's; `MapDef::regenerate` makes the images), every Skirmish
   Default file in `maps/` (place them with the editor's Layout tool and save;
   `the_maps_in_the_repository_are_playable` says which), and the tests that build a Skirmish Default map and expect
   it playable (the editor's use `tests/editor/colonists.rs::with_colonists`,
   `tests/editor/syndicate.rs::with_syndicate` and `tests/editor/cults.rs::with_cults`). Check each layout with
   `MapDef::validate` (its structures by `placement::check_layout`, its units by the unit-start rule), and keep the
   starting units clear of the structures' sides and exits. The goldens on `dev_skirmish` embed its map, layouts and
   all, so they are re-blessed for the new layouts, every checkpoint the same (as slices M3-C and M4-B did; *Replays
   and golden hashes*). Every faction has Starting Conditions since slice M5-B (plan decision T63): a test that needs
   a faction without something uses `test_defs`. A faction's own layout rule goes into `check_layout` (the
   Syndicate's expansions inside their Tunnel's area), and its text into the editor's `rule_text` and the client's
   `placement_text`.
5. **Army cap** (`economy/army.rs`): the faction's `ArmyResource` (`ArmyResource::of`, `name`, `cost_of`, and what
   is available in `cap`), or none. A cap may be derived rather than built (the Syndicate's Tunnel Space, plan
   decision T33: its Tunnels' areas less their expansions' cells), and then counts the units off the map too
   (item 15). The start check at production start is the one for every faction (design change
   0143, `sim/producers/step.rs`), and so is its `SimEvent::ProductionWaiting`; the client's `army_cap_text` and the
   InfoPanel's waiting line name the faction's resource. A resource that only shows (the Cults' Unit Control, slice
   M5-C) says so in `ArmyResource::reserves`, and nothing waits for it. A unit costs only in its own faction's
   resource (`ArmyResource::cost_of`).
6. **HUD figures** (`hud/resources.rs`): the faction's `ResourceFigure`s in its DisplayHud order (`ResourceFigure::of`)
   and each new figure's reading in `resource_bar`, in ASCII.
7. **Templates** (`command_ui/interface/templates.rs`, an object's own panel in `templates/<faction>/<object>.rs`): each
   kind's panel template, in its client module (*Adding a unit or structure*), `PanelTemplate::NONE` until its slice
   builds one. A producer takes `InterfaceTemplate::Production`, whose grid comes from its `ProductionDef` (a new unit
   to train shows its label; a producer that recruits by itself, the Recruitment Hall, shows no Train and no X, only
   C), and a pool building `InterfaceTemplate::PoolProducer`, whose KindMenus come from it (slice M5-F2).
   A builder needs its `Builder` (`command_ui/interface/placing.rs`: where the ghost stands,
   which check colours it, what a click sends; slice M3-J1 has a constructor's, the Drones' and the Beacon's), and a
   building not in the game yet holds its menu slot as a `PanelCommand::Planned`. A type that transforms shows only its
   Cancel while it does (`TRANSFORMING_GRID`; `colonists::becomes` lists the types that may).
8. **Producers' exits** (`ProductionDef::exits`, slices M3-D and M3-D2): `ExitRule::Side` for a side doorway
   (the unit appears half inside that side), `ExitRule::Sides` for a door on every side of that letter, the first
   free one in `Doorway::order` taking the unit (slice M5-C), `ExitRule::Centre` for an energy doorway open through the
   sides with the letters given (the unit appears at the centre and walks out through them; its arc comes from the
   sides, so a new portal needs no code; see *Production*). `defs_are_consistent` checks that the letters name the
   type's sides.
   A doorway with a rally point gives the new unit its own right-click order at release (`orders::resolution`).
   `ExitRule::Network` is a producer with no doorway (the Hideout, slice M4-C): its units go into the held store
   (item 15), and a rally point brings them out of a Tunnel.

The faction's own mechanics come next, each only if it has them (M3's and M4's in brackets):

9. **Kinds that change** (slices M3-A, M3-E, M3-F; plan decision T15): every change of kind goes through
   `Sim::change_kind` (see *Object model > Changing kind*: same id and owner, capabilities rebuilt, orders, move and
   target kept, a `SimEvent::KindChanged`). A building that comes out of another object is an `Object::transform`
   (`transform.rs`, `Sim::step_transforms`: a Warp In, a Drone deploying as a Beacon), placed by its own `Method`
   in the one placement check, and the client rebuilds the visual on a `MeshKey` change and moves the object into
   its new group (`ControlState::set_type`; an unfinished building's group is its own, `GroupKey`).
10. **Abilities and modes** (M3-B Weapon Conversion, M3-S the Riot Shield, M3-T the target state): an `Ability`
    (`command.rs`), its rule in the gate (`sim/gate/orders.rs::check_ability`), its use in `sim/modes.rs` (a
    toggle switches the type at the press, `Ability::switched_kind`, and its mode's state holds the form, which the
    Modes step works toward the type, waiting through an attack phase that can't be interrupted,
    `combat::mid_attack`; *Object model > Type and form*); the client's `PanelCommand`, its template slot and a
    label of six characters at most with its full name in the hover hint, and the press shown at once
    (`ControlState::toggle`, `PendingCommands::kind_of`).
11. **Shields** (M3-B): `max_shields` in the def; `Object::shields` absorbs in `combat/damage.rs::resolve` and
    regenerates in `Phase::Shields`; the client draws the bar and the InfoPanel line.
12. **Workers** (M3-H, M3-H2, M3-H3, M3-K2a): a `WorkerDef` (its carry capacity, mining and drop-off times and
    reaches, search radius and `pass_through`), the Gather and Drop Off orders (`orders/gather.rs`, the gate in
    `sim/workers.rs`), the timed work and the bank credit (`economy/gathering.rs`, `SimEvent::ResourcesDelivered`),
    `StructureDef::drop_off` for the buildings that take loads, and a new drop-off for the workers of one that goes
    (`Sim::reassign_drop_offs`, on the removal path). Units that may overlap go in `invariants::may_overlap`. In
    the client: the pointers, the load marker, the InfoPanel lines and a select-the-workers command.
13. **A movement model** (M3-G, plan decision T25): a `MovementAttrs` variant with its mover in `movement/`
    (`drag.rs` beside `follow.rs`), the movement invariants amended in writing, and `movement_perf` run A/B.
14. **Orders of its own** (M3-F Deploy): an `OrderKind` with its behaviour in `orders/`, its gate check and its
    events (`SimEvent::DeployFailed`), reservations that others' placements respect (`Sim::pending_builds`), and the
    client's ghost (a `Builder` in `command_ui/interface/placing.rs`). **A worker-built structure** (M4-E) needs no
    order of its own: its def's `StructureDef::site` (`SiteDef`: the price, the frames, the share of its MaxHP it
    starts at, where the builder goes) and its builder's `WorkerDef::builds`, and the Build order, the price and the
    site take it (*Economy > Agent-built Tunnels*).
15. **Units off the map** (M4-C, plan decision T30): a unit inside something, the Syndicate's Tunnel Network, is not
    an object but a `Held` in `Sim::held` (*Object model > Units off the map: the held store*). Its ways in and out
    are `Sim::take_off_map` and `Sim::put_on_map` only (the Enter order and the Tunnel's eject queue,
    `sim/network.rs`), and whatever must count it says so: the army cap, the removal path's references
    (`Sim::forget_references`), a defeated player's units, the state hash and saves, `testing::dangling_references`,
    the client's control groups. A new kind of holder (a transport) gives `Place` a variant and matches it
    everywhere `Place` is matched. The client reads the store for its menus (`Sim::network_counts`, `next_out`).
16. **Sites** (M4-E, M4-F, M4-S1, M4-S2, plan decision T31): a structure built where it stands is the object from its
    first frame, with `Object::construction` while it is a site (*Object model > Sites*): a worker's
    (`StructureDef::site`, the builder held at `Place::Site`) or a Tunnel's expansion (`StructureDef::expansion`,
    `Builder::Tunnel`, the Tunnel's one operation, *Expansions and a Tunnel's operation*). Its maximum HP follows the
    ConstructionHP rule, every system asks `Object::is_operational` before treating it as working, and its cancel and
    refunds are the site's (`SimEvent::SiteCancelled`, `BuildRefunded` for an order that never became a site). A
    builder that isn't a worker or a Tunnel adds a `Builder` variant.
17. **The underground layer** (M4-A, M4-F, M4-H, plan decision T32): `StructureDef::underground` puts a structure in
    a layer of its own: hidden from every player but its owner while owned (*Vision > Hidden objects*), walked over
    (`BlockerKind::UndergroundExpansion`; surface and underground footprints never share a tile, `may_overlap`),
    hit by blasts over it (*Combat*), placed by the expansion method inside its Tunnel's area (*Placement and the
    build area*), and, when its Tunnel goes, neutral and decaying (*Object model > Losing Tunnels*). A form between
    the layers is a pop-up (`StructureDef::popup`, *Object model > The pop-up form*): one function decides each
    reader's answer. The client draws what only its owner sees as an outline.
18. **Prerequisites** (M4-K, plan decision T37): `StructureDef::requires` and `Recipe::requires` name the kinds that
    must stand; one rule for every faction counts them (`tech::counts`, *Object model > Tech prerequisites*), and the
    gate checks them only as the work starts (`Reject::Requires`). The client greys the entry with "Requires a/an
    <name>", so a faction's prerequisites need data only.
19. **Tests and QA** (plan decision T27; slices M3-K1 and M3-K2, M4-L and M4-M): `ScenarioBuilder::faction` sets a
    test player's faction, and `cargo run -- --factions <f1,f2>` starts a game with it, which its replay records. Then:
    - a build line through the gate beside `testing::build_order` (`testing::colonists`: Train, Gather, Deploy and
      Warp In; `testing::syndicate`: Gather, Job, Expand, Upgrade and After, with `SupplyBooks` for the Supplies;
      `testing::cults`: seats, group Builds and Train From),
      whose `Books` count what the faction's income adds;
    - goldens of its opening and of a game against another faction, both players scripted (`tests/golden/colonists.rs`,
      `testing::colonists::skirmish`; `tests/golden/syndicate.rs`, `testing::syndicate::skirmish`;
      `tests/golden/cults.rs`, `testing::cults::skirmish`);
    - its economy oracle against the design's targets (`tests/economy_oracle/`, `docs/economy_oracle.md`);
    - a perf test of its crowds (`tests/drones_perf.rs`, `tests/agents_perf.rs`, `tests/recruits_perf.rs`) and an ignored 10-minute soak of the
      golden game (`tests/skirmish_soak.rs`, `tests/syndicate_soak.rs`: invariants, books, no stuck order, the hash
      stable over two runs; the Syndicate's adds the network's invariants, `check_network`, and plays on past the
      golden's end with more on both sides);
    - its opening in the playtest pairings (slice M5-R), one line in `OPENINGS` (`tests/pairings/main.rs`, its hook in
      `openings.rs`): every ordered pair of the four factions (16) plays on every Skirmish Default map
      under the invariants, the books and the gate, until player 2's buildings are removed and player 1 wins; the
      client's `tests/pairings.rs` starts each pair from `--factions`. A faction without a line plays idle;
    - a scripted walkthrough of the first playable (`--factions` with `--input-script`, see *QA mode*), and its
      playtest notes in `docs/playtest/`, listed in `docs/playtest/README.md`.

## Tunables

The numbers a playtest tunes live apart from the logic that uses them, in tuning modules grouped by system, so that
each module can later become one loaded struct (the data files come at playtest, plan decision T54). A number that
belongs to one object goes in its def instead (*Adding a unit or structure*).

- **The sim's** (`crates/sim/src/tuning/`, slice M4-R1): one module per system, `combat.rs`, `construction.rs`,
  `gathering.rs`, `movement.rs`, `orders.rs`, `paths.rs`, `production.rs`, `repair.rs`, `shields.rs`, `tunnel.rs` and
  `vision.rs`, each mapping one to one onto the struct a data file will load at playtest prep. The code imports its
  constants from there (`use crate::tuning::movement::{JAMMED_TICKS, MAX_REPATHS};`). A faction's own rule is in its
  `defs/<faction>.rs`.
- **The client's feel** (`crates/game/src/tuning/`, slice M4-R2): `camera.rs` (the view's width, the pan speed, the
  edge-pan margin, the editor's zoom), `input.rs` (the drag threshold, double-click and double-tap times, the
  targeting click allowance, the pick slop, the wheel's pixels per line, the editor's marker grab radius) and
  `feedback.rs` (how long an order's flash, a refused command's message, a tracer, a death marker and a HUD notice
  show). Each kept its name, and the module it came from re-exports it (`pub use crate::tuning::...`) or imports it,
  so the code that uses it still names it there.
- Each constant has a doc comment: a design citation, or "Placeholder (not in the design)" and, where there is one,
  why it has its value.
- Colours and drawing sizes stay with the drawing code. Technical constants stay where they are used: the sim's
  units of measure, hashing, file formats, and the caps and margins that only bound a computation or absorb rounding;
  the client's z offsets, frame budgets and capture timeouts.
- A slice adds its constants at the end of its system's module.

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
