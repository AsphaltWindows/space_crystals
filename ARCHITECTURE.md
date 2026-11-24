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
There is one exception: an `#[ignore]` perf test under `crates/sim/tests/` may read the clock in a single timing helper marked
`#[expect(clippy::disallowed_methods)]`, as `nav_perf.rs` does. Its timings are printed and never asserted.

- **Order**: iterate objects in `ObjectId` order. `Sim` keeps them in a `BTreeMap<ObjectId, Object>`, and ids are
  allocated in increasing order and never reused. Break ties between candidates by `ObjectId` (`Iterator::min_by`
  keeps the first minimum), and compare floats with `total_cmp`.
- **No hash containers**: don't use `HashMap` or `HashSet`, because their iteration order is random. Use `BTreeMap`,
  `BTreeSet` or a sorted `Vec`.
- **No outside world**: no wall clock, no threads and no OS randomness. The only source of randomness is the sim's
  PRNG (`sc_sim::Rng`, PCG32), seeded from the game setup's seed (`GameSetup::seed`). It is part of the hashed
  state.
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
  `> Structures only on flat terrain`): every footprint tile of a kind with `ObjectDef::needs_flat_ground` must be
  flat. Validation flags it (`SpawnProblem::NotFlat`) and `Command::Debug(Spawn)` refuses it.
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

- **Core fields**: `id`, `kind`, `owner` (`None` means neutral), `pos`, `prev_pos`, `facing`, `prev_facing` and `hp`
  (`None` means indestructible).
- **Capabilities**: today these are `placement` (the tile footprint), `resource` (`ResourceState`), and for units
  `orders` (`Orders`, `orders.rs`: the current order with its behaviour's state, a queue of at most
  `MAX_QUEUED_ORDERS`, and what the unit does on its own while it has none; see *Behaviours*), `mover`
  (`Mover`, `mover.rs`: goal, status, path, the replanning state of `docs/movement.md > 3` and the sidestep of a
  yielding unit; `mover/` holds the stuck schedule, the sidestep and the hash), for units that
  can attack, `attack` (`AttackState`, `combat.rs`: see *Combat*) and, for units that carry a Riot Shield,
  `riot_shield` (`RiotShield`, `riot_shield.rs`: the shield's mode, see *Combat*); for a plate on a patch,
  `extractor` (`Extractor`, `economy/mining.rs`: the patch it mines and its cycle, see *Economy*); for a structure
  that produces units, `production` (`Production`, `production.rs`: its queue, the progress of the unit in
  production and its rally point, see *Production*); for a structure that constructs buildings, `constructor`
  (`Constructor`, `construction.rs`: idle, constructing or holding a building ready to place, see *Economy*); for
  a worker, the Drone, `worker` (`Worker`, `economy/gathering.rs`: its load, its AssignedDropOff and the patch it
  gathers, see *Economy > Gathering*). The
  rest arrives as more optional fields, each with its own module. Never add them as separate stores keyed by id. A
  system walks the objects in id order and skips those without its capability
  (`let Some(..) = &mut o.resource else { continue };`).
- **Per-type constants** sit in one `static ObjectDef` per type (`defs.rs`, `core/entities.md > Object Type`). Reach
  them through `ObjectKind::def()` or `Object::def()`. Every value cites its design section. Faction types go in
  per-faction modules (`defs/gdo.rs`, …). A unit type also has `ObjectDef::unit: Some(UnitDef)` (`defs/units.rs`,
  `systems/units.md`): its `UnitBase`, whose `static UnitBaseDef` holds the base's domain, movement model and flags
  (turret, crushable, …), the attributes of that movement model (`MovementAttrs`), and its turret, attack and costs.
  Its box is its silhouette (`ObjectDef::half_extents`). Armour (`ObjectDef::armor`, an `Armor` of PointArmor and
  FullArmor) is the same for units and structures; map resources have `Armor::NONE`.
- **Structures** (`defs/structures.rs`, `defs/gdo.rs`; `factions/gdo/structures.md`, `factions/gdo/economy.md`): the
  GDO's Deployment Center, Generator, Barracks, Extraction Facility and Extraction Plate (slice M2-A; their
  `ObjectKind`s follow the M1 ones, because the state hash numbers kinds in declaration order). A structure type has
  `ObjectDef::structure: Some(StructureDef)` (its `SymmetryType`, BuildRadiusExtension and Power, the plate's
  `MiningDef` in `StructureDef::mining`, and a constructor's `Constructs` list of `Recipe`s in
  `StructureDef::constructs`, see *Economy*) and a tile footprint, `ObjectDef::tile_size` as unturned;
  its `Placement` holds it as placed, `w` and `h` swapped by a quarter turn, with the orientation it was placed in
  (`Placement::orientation`). It has HP, armour and a SightRange, and no orders, mover or attack, so behaviours,
  movement and combat pass it by.
- **The Colonists** (`defs/colonists.rs`; `factions/colonists/units.md`, `factions/colonists/economy.md`, slice M3-A):
  the Drone, the Beacon, the Egress Node, the Robo Portal, the Bio Portal and the Sabre and Rifle Legionnaires, after
  the M2 kinds. Three def fields are data only until their slices: `ObjectDef::max_shields` (MaxShields, set for
  exactly the Colonists' kinds; Shields arrive with M3-B), `UnitDef::beacon_capacity_cost` (M3-C) and
  `StructureDef::warp_in`, the Beacon's Constructs List (M3-E). `warp_in` is not `constructs`, so the Beacon is no
  GDO-style constructor (plan decision T14). Their structures have no Power and no build area. The portals produce
  through their first A side clockwise from north, a placeholder until M3-D's exit rule. Until DragMovement (M3-G),
  the Drone, a HoverVehicle, moves on the stand-in, which reads only `MovementAttrs::max_speed` (the DragMovement
  equilibrium, `(ForwardAcceleration + OmniDirectionalAcceleration) / DragRatio` = 8 su per tick) and its TurnRate;
  Rugged tiles stay impassable to it (`HOVER_VEHICLE.rugged_terrain` is false, plan decision T25). Its base gives the
  navigation a `(Ground, 1)` grid next to the light infantry's `(GroundRugged, 1)`.
- **Changing kind** (`sim/kinds.rs`, plan decision T15): an object that transforms (a Drone deploying into a Beacon,
  a Beacon warping in a building, a Legionnaire converting its weapon) stays the same object, its id included.
  `Sim::change_kind(id, kind, placement)` is the one place a kind changes, and it emits `SimEvent::KindChanged`. It
  keeps the id, owner and facing; the position too, unless the object gets a placement, whose centre it then jumps
  to (`prev_pos` too); and the HP, clamped to the new MaxHP (callers interpolate HP themselves). It rebuilds the
  capabilities from the new def with `kinds::Capabilities::fresh`, which `Sim::insert_object` uses as well: a
  capability whose def part is unchanged keeps its state, a new one starts fresh, one the new kind lacks goes. A
  unit that stays a unit keeps its orders and its move (question Q0042 point 6), replanning if its grid changes; a
  kept attack keeps its target and restarts its sequence. Paid state the new kind has no place for (a production
  queue, a construction) is refunded first, as on removal. Call it before 5b (in the commands phase, or a step
  right after 5a), or sync the grids yourself (`paths::sync`). A caller whose new kind isn't functional yet (a
  building warping in) holds its functions back itself. Slices B, E and F are its callers; until then it is
  unit-tested only (`sim/kinds/tests.rs`).
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
1. Add an `ObjectKind` variant whose doc comment cites the design section, and add it to `ObjectKind::ALL`.
2. Add its `static ObjectDef`, with a citation for every value, and its arm in `ObjectKind::def()`. Missing numbers
   become named consts tagged with the question id.
3. Set up its capability state in `kinds::Capabilities::fresh` (`sim/kinds.rs`; units get idle `orders` and
   `mover`), and give it a placeholder shape (`object_render::object_mesh`, whose `match` is exhaustive). The other
   exhaustive matches on kinds name the rest: its short label (`hud::selection_panel::short_label`), its per-object
   values (`SpawnSettings::defaults`, the editor's `fields.rs`) and its interface
   (`command_ui/interface/templates.rs`).
4. Test it. Spawn it in a small inline scenario, then check its initial state and its behaviour.

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
- **Out**: a `PlacementCheck`: the `TileProblem`s of each footprint tile (off the map, not Buildable, sloped, not
  Visible, under a structure, under a resource, under a ground unit), row by row and in that order, and the
  `WholeProblem`s (not a structure, outside the build area, a plate not on a patch). Valid means both are empty: the
  ghost shows the tiles' problems tile by tile, and the gate refuses a placement with any problem. The check reads
  the true state, but a tile the player doesn't see has `NotVisible` as its only problem, so it tells nothing the fog
  hides.
- **The rules** are listed with their design sections in the module docs. Footprints share tiles only as
  `ObjectKind::may_overlap` allows. A ground unit of any player blocks the tiles its box overlaps, and is never pushed
  aside (design change 0094); air and underground units never block. A plate goes exactly onto a patch
  (`placement::Onto`), the one rule of the check that map validation applies to pre-placed objects too
  (`SpawnProblem::NotOnPatch`, slice M2-J2).
- **The build area** (`BuildArea`, `Sim::build_area(player)`; `factions/gdo/structures.md > GDOBuildArea`) is the
  union of one rectangle per GDO building the player owns: its footprint grown by its BuildRadiusExtension on every
  side, clamped to the map (design change 0091: square). A building with an extension of 0, the plate, adds nothing.
  It is derived, never stored or hashed: worked out from the objects whenever it is asked for, so it follows every
  placement and every loss at once (0092). It is the area only, apart from the tiles' validity (0093): an overlay
  draws `BuildArea::runs`, a ghost the check's tiles. A Starting Layout's area is that of its Deployment Center
  (`BuildArea::of_layout`). The Deployment Center seeds the area and needs no tile in it (`seeds_build_area`).
- **Not placement rules**: debug spawns bypass them (they keep the map's rules and apart from other footprints, see
  *Commands, events and replays*), and starting units follow the unit-start rule (`paths::misplaced`, see *Sim
  scenarios and tests*).
- **Tests**: `src/placement/tests.rs` (each rule on the pure check), `placement/build_area.rs`'s (the 28 × 28 area,
  each kind's extension, the runs), `tests/placement.rs` (the verification points of design changes 0004 and 0091 to
  0097 in games and layouts, determinism, a save) and the ignored `tests/placement_perf.rs`. Measured (release, 769
  objects, 101 buildings of the player's; slice M2-J2, on a quiet machine): 4 µs a check, 2 µs a build area.

## Tick pipeline

`Sim::step(&TickInputs)` advances exactly one tick. It runs fixed phases in this order (`sim.rs`):

1. **Clear events**, so that `Sim::events()` holds only this tick's events.
2. **Advance the tick** (`tick += 1`).
3. **Snapshot** `pos`/`facing` into `prev_pos`/`prev_facing` for every object (the client interpolates from these),
   and note who owns a structure as the tick starts (`economy::defeat::structure_owners`, for the defeat check).
4. **Commands**: apply each `PlayerCommand` in `TickInputs` order (`apply_command`). Each passes the gate against
   the state the commands before it left, so a cost is paid from the bank as they left it and a Place is checked
   with the footprints they placed (see *Economy > Construction*).
5. **Systems**, each walking the objects in id order, one function per module (`docs/movement.md > 4`):
   - Modes, `riot_shield::step`: a Riot Shield on its way up or down goes on one tick. After the commands, so that
     a toggle counts from its own tick; before 5a, so that the behaviours, the movement and the combat of the tick
     all see the mode it leaves.
   - 5a `orders::update`: a unit's finished order completes and its next one starts, until one is in progress, so
     legs follow each other without an idle tick; then that order's behaviour runs for the tick, or the idle one
     (see *Behaviours*). They write goals (`Mover::go_to`, `stop`) and the target the attack engages. Everything
     they read of other objects comes from one snapshot taken first (`orders::world::World`): the positions and the
     vision the last tick left.
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
   - 5e `combat::step`: attack phases, the tick's list of hits applied together, deaths (see *Combat*). The dead
     leave through `Sim::remove_objects`, then `paths::sync` runs again in case a footprint went with them.
   - Economy, `economy::step` (plan decision T5, see *Economy*): after 5e's removals and their `paths::sync`, so
     that a building destroyed in the tick counts for nothing in it; before 5f, so that what it brings into the game
     is seen in its own tick. In order: every player's power (the structures this tick's commands placed included),
     the constructors (`construction::step`; when a recipe's frames count: *Economy > Recipe timing*), the plates'
     mining, the workers' timed work (`gathering::step`, slice M3-H: mining takes its load from the patch, a
     drop-off pays the bank). `economy::step` returns the patches the workers emptied, and `step` takes them out
     through `Sim::remove_objects` and syncs the grids again, as after 5e's deaths.
   - Producers, `Sim::step_production` (slice M2-D2, see *Production*): part of the economy phase, run right after
     `economy::step` and not inside it, because a finished unit is inserted into the game (`Sim::insert_object`).
     The unit in production advances, and a finished one is released into its exit doorway, or held while that is
     occupied. Nothing in `economy::step` reads what the producers write, or the other way round, so the order
     between the two is free. After 5d, so a released unit never moves in its own tick; before 5f, so it is seen
     in it.
   - Defeat, `economy::defeat::step` (slice M2-F2, plan decision T8, see *Economy > Defeat and the end of the
     game*): the economy phase's last step, run by `step` after the producers, so that every way a structure left
     the game in the tick counts (the commands' removals, 5e's deaths). Before 5f, so that the units a defeat makes
     neutral are nobody's eyes in its own tick.
   - 5f `Vision::update`: every player's sight from the objects as the tick leaves them, those this tick's commands
     spawned included (see Vision). Combat in 5e reads the vision of the tick before.
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
- Insert a new phase at a deliberate position, with a comment in `step` saying why there, give it its `Phase`, and
  update this list.
- Commands always run before movement. There is exactly **one** movement pipeline, and all movement goes through it.
  The old prototype had two pipelines, and they raced.
- Damage accumulates as a **list of hits** that is applied in one place. Never store it in a single slot that a second
  hit can overwrite, because the old prototype lost hits that way.
- Lessons from the prototype, for M1 and on:
  - **Movement**: the arrival radius must be at least one tick of travel. Otherwise units overshoot and oscillate.
  - **Pathfinding**: cap repath attempts (`mover::MAX_REPATHS`). A* needs a correct closed set, a best-cost map and
    deterministic tie-breaking. Overwriting a parent at equal cost made zigzag paths.
  - **Occupancy** is domain-aware. For example, underground expansions don't block surface units.

## Combat

Slice W (`combat.rs`, `combat/`, `elevation.rs`; `systems/combat.md`). A unit that can attack has an `AttackState`:
the target its behaviour engages (`target`, set by its Attack order in 5a), the phase of the running sequence and the
ticks spent in it. Hashed and serialized like all state.

- **Phases** (`combat/phases.rs`): Aiming → Firing → Cooldown → Reloading, each lasting exactly its duration in
  ticks (zero included). A sequence starts in 5e when none runs and the engaged target is valid and in range; the
  next one at the earliest in the tick after Reloading ends, so a cycle takes Aim + Firing + Cooldown + Reload ticks
  (Peacekeeper 17, Guard 8). Ordered in tick `t`, the first hit lands at the end of tick `t + Aim + Firing − 1`.
  Aiming turns the base towards the target at TurnRate (the attack writes `facing`) and lasts until it faces it
  within `FACING_TOLERANCE`. Aiming and Reloading are interruptible: another order, or a target that is gone or out
  of range, cancels an aim; a reload keeps counting. Firing and Cooldown run on whatever happens.
- **Queries**: `combat::may_move(&Object)` and `may_turn(&Object)`: whether the attack leaves the base free to move
  or turn this tick (`> UnitBaseSource`: not while Aiming, Firing or in Cooldown; turret units always). Phase 5d
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
  owner sees the target (`PlayerVision::sees_box`), by the vision 5f left at the end of the tick before. What a
  unit does about a target out of range or out of sight is its behaviour's; a neutral attacker sees nothing.
- **ElevationModifier** (`elevation.rs`, shared with vision): `Elevation::of` an object is its elevation level (the
  height rounded down), or `Exempt` for air units; `modifier(source, target)` is +1/−1/0 grid unit, binary.
  `attack_range` and `sight_range` apply it; melee attacks don't take it.
- **Melee** (`systems/combat.md > FullyConnected > Melee`: "adjacent contact"; the Sabre Legionnaire): every melee
  attack has the Range `defs::MELEE_CONTACT_GU`, a gap of 8 su, the one that counts as Proximity
  (`orders::PROXIMITY_SU`), and no MinRange (`defs_are_consistent` checks both). It is a placeholder (question Q0042
  point 5): exactly 0 would be fragile, since a unit closing in may stop a little short of touching. `max_range`
  gives melee no ElevationModifier.
- **Hits** (`combat/damage.rs`): every attacker acts against one snapshot of the targets, and an attack's effect
  lands at the end of Firing as a `Hit` on the tick's list. The list is applied once all have acted, so every hit
  counts, including those on a unit another hit has killed and those of a unit that dies in the same tick.
  `hit_damage` follows `> SingleTargetDamage` and `> DamageModifierOrder` (pre-armor multipliers, PointArmor,
  directional multiplier, MinimumDamage); HP is fractional, nothing is rounded. PointArmor is the target's
  `ObjectDef::armor`, a structure's too: a Peacekeeper's 10 damage takes 9 HP off a Generator and 8 off a plate. An
  object at 0 HP or less dies; a structure that dies frees its tiles (the navigation grids are rebuilt).
- **Removal path**: `Sim::remove_objects` (`sim/removal.rs`) is the one way objects leave the game, for deaths and
  debug removals alike. It emits `Removed` and clears every reference to the removed: a current order naming one
  completes, queued ones are dropped, a move towards one ends `TargetLost`, an attack on one stops (an aim is
  cancelled, a shot being fired hits nothing). `testing::dangling_references` checks that none are left.
- **Riot Shield** (`riot_shield.rs`; `factions/gdo/units.md > Peacekeeper > Riot Shield (toggled ability)`): a
  unit whose `UnitDef::riot_shield` is set starts with its shield `Lowered`; `Command::Ability` raises or lowers it,
  through `Raising { left }` or `Lowering { left }` for the toggle time (`RiotShieldDef::toggle_ticks`, 6 ticks, a
  placeholder: the design says under half a second), so a toggle in tick `t` is done at the end of tick
  `t + toggle_ticks − 1`, and a reversal turns back from where the shield is. While it isn't `Lowered` the unit
  moves at half speed (`Mover::speed_factor`, written when the mode changes) and can't attack: the gate drops it
  from Attack orders (`Reject::CannotAttack` when none is left), its behaviours see no attack (`Unit::attrs`: no
  auto-targeting, an Attack order gives up, Hold disengages), and 5e cancels an aim (`can_attack_now` is false); a
  shot already being fired lands. Only while it is `Raised` is the damage it takes halved, a pre-armor multiplier
  (`damage::pre_armor_multiplier`, with the Rugged bonus). A mode is not a task: the unit keeps its orders and
  yields like any other (design change 0023). The placeholders are listed in the module docs.
- **Not yet**: turrets, projectiles, AttackGround and the other location-targeted attacks, AoE.

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
  A tile is Visible while its count is above 0, and Explored once it has been Visible. `PlayerVision::revision`
  changes when any of the player's tiles changes state, so views redraw only then.
- **Memory** (`vision/memory.rs`, `core/vision.md > Explored`): a `LastKnown` record per tile-based object (a
  structure or a resource) the player has seen and doesn't own: kind, owner, footprint, HP, resource state and the
  tick it was last seen. An object is in a player's sight while a tile under its footprint is Visible; its record
  is rewritten at the end of every such tick (`Vision::remember`, the last phase) and stays as it was out of sight.
  A record whose object is gone, or has moved, is dropped once a tile under the record is Visible: an object
  destroyed in the fog is remembered until its place is seen. An object placed on Explored tiles gets no record, so
  it stays unknown until seen. Own objects are never recorded.
- **Memory updates follow the vision's changes**, never a scan of the map or of every object for every player:
  5f's id-order walk over the objects also merges the footprints of the tile-based objects (`Footprints`), so only
  a footprint that came or went touches the index from tiles to objects; `see` and `unsee` note the tiles whose
  state flips; after the walk a noted tile under no footprint and no record costs two bit tests, and on the others
  objects enter or leave the sight and the records seen gone are dropped; `remember` reads only the objects in each
  player's sight. Measured (`tests/vision_perf.rs`, ignored, release): 500 moving units and 168 crystal patches on
  256 × 256, `update` and `remember` 0.17 ms per tick on average, 0.2 ms at the 99th percentile.
- **State**: the Explored bits and the records are history, so they are hashed and serialized (each player's rows of
  `#` and `.`, and its `last_known` records). Eyes, stamps, counts, the footprint index and the objects in sight are
  derived: a loaded save rebuilds them from the objects (`vision::load`, which rejects a record twice, of an own
  object, off the map, or seen after the save's tick), and invariant V1 (`testing::invariants`) rebuilds them from
  scratch, from the same records, and compares.
- **Queries**: `Sim::vision()`, `Sim::visibility(player, tile)` and `Sim::is_visible(player, object)`: an object is
  seen when any tile it covers is Visible (its silhouette box or footprint, `combat::object_box`).
  `Vision::player(id)` gives a `PlayerVision` with `state`, `states` (row-major, for the client's fog texture),
  `object_state`, `sees` and `sees_box`; and for the records `records` (by id), `record(id)`, `remembered` and
  `remembered_record(id)` (those not in sight, which Explored tiles show), `record_tiles` and `is_record_tile` (the
  tiles under any record) and `records_revision` (changes when a record comes, goes or moves; not when only its
  state changes).
- **Paths** are planned from what each player knows (design changes 0040 and 0042; *Navigation > Path knowledge*):
  the records' footprints are the other players' structures and the resources it knows of; its own come from the
  live objects, as they are never recorded. Map-placed patches and stations, known from the start for paths, are
  records only once seen: until then `PathKnowledge` holds them, so the fog shows nothing of them.

## Behaviours

Slice E (`orders.rs`, `orders/`; `systems/controls/behaviors.md`). Phase 5a runs one behaviour per unit and tick: its
current order's, or the idle one. A behaviour never moves or shoots anything itself: it writes the mover's goal and
the target the attack engages, and reads `Mover::status` and the snapshot.

- **State** (`orders/state.rs`, the design's BaseBehaviorState): `ActiveOrder::state` (`Behaviour`: `Plain`,
  `Attack`, `Approach`, `AttackMove`, `Patrol`, `Hold`, `Gather`) and `Orders::auto` (`Auto`: what a unit without an order
  does: `Watching`, `Engaging`, `Returning`). Hashed and serialized like all state; every order that starts resets
  `auto`. A state names objects only by id, and the removal path clears them (`Orders::forget`).
- **The snapshot** (`orders/world.rs`): `World` holds a `View` of every object (position, shape, owner, domain,
  elevation) as the last tick left it, the vision of its phase 5f, and a coarse grid of the objects a scan may
  pick. Every unit decides on the same world, whatever its id. `World::pick` is the scan of `> BaseAutoTargeting >
  Target Selection Priority`: among the ValidTargets in reach (enemies only: another player's destructible objects
  that the unit's owner sees and its attack can target), threatening ones first, then the least rotation from the
  unit's facing, then the smallest gap, then the lowest id. Enemy structures are ValidTargets too (design change
  0103), so units attack enemy bases on their own; the structures that can't attack come after the targets that
  shoot back. The grid holds candidates by their centres, so a scan looks as far as the unit's longest range with
  the modifier, plus its own half size and the largest candidate's (`World::candidate_half`): a footprint in reach
  is found however far its centre lies.
- **AttackingObject** (`orders/engage.rs`): `close_in` is its core, shared by all fighting. In range (`combat::
  reaches`) the unit stops and phase 5e attacks; out of range it heads for the target with `MoveGoal::Object`,
  starting only while `combat::may_move` holds. An **Attack order** adds its ends: the target destroyed (the removal
  path completes the order); the target out of its owner's sight (design change 0034: the unit walks to the **last
  seen position**, which the behaviour notes while it sees the target, and the order completes there; it never
  re-engages on the way); the approach failed, or ended as near as the unit gets to a target that hasn't moved.
- **Idle units** (`engage::idle`): scan their SightRange every tick; the acquired enemy is engaged with `close_in`
  from the IdleOrigin, where the unit stood. Farther than `IDLE_LEASH_GU` (4) from it the unit disengages and walks
  back without scanning; a target destroyed or out of SightRange leaves it idle where it is.
- **Hold Position** (`engage::hold`): scans its weapon's range only, never moves, and disengages the moment the
  target leaves that range.
- **Move to an object** (`orders/travel.rs::approach`; design changes 0032 and 0033): proximity is a gap of at most
  `PROXIMITY_SU` (8). A static object completes the order there (also at the nearest point the unit can reach, or
  stuck within 2 GU of it behind units that won't move). **A unit is followed**: the order never completes by
  itself; the follower waits within proximity (or where it was stopped near it, without re-routing) and sets off
  again when the target moves. It ends with a new order, the target's death, a failed move, or an enemy target
  going out of sight (to its last seen position, as for Attack). "Leaves the map" (transports, buildings, tunnels)
  is a documented hook in `approach`.
- **AttackMove** (`travel::march`): one `Leg` to the unit's own destination (the magic box). While moving it scans
  its SightRange and engages with `close_in`; it lets go, and resumes the move with a new path, when the target is
  destroyed or no longer valid, or when the unit is farther than `ATTACK_MOVE_LEASH_GU` (6) from its PathReference.
- **Patrol** (`travel::patrol`): legs to and fro between the position where the Patrol started executing (design
  change 0036) and the destination. A failed leg counts as arrived; `PATROL_FAILED_LEGS` (2) in a row end it (design
  change 0037).
- **Gathering** (`orders/gather.rs`, slice M3-H, plan decision T17; `factions/colonists/units.md > Drone`, whose
  open cases follow question Q0037's proposals, each marked with its point): one behaviour for the Gather and the
  Drop Off order, in the phases of `GatherPhase`: ToPatch, Waiting, Mining, ToDropOff, DroppingOff, Delivered.
  - **Occupancy is derived, never stored on the patch**: a patch is taken while a worker of any player mines it (the
    snapshot's `World::patch_mined`), once a worker with a lower id claimed it in this tick (`gather::Claims`, the
    one thing of 5a that isn't the snapshot), or while another worker is first in line for it
    (`World::first_waiter`: the longest wait, then the lowest id). An arrival at a taken patch moves the **order
    itself** on to the nearest free patch its owner knows within `REDIRECT_RADIUS_GU` (6, a placeholder), so
    `OrderKind::Gather` always names the patch the worker heads for; with none, it waits where it stands.
  - **Legs are moves to a point** the behaviour picks, where the worker stands against the nearest open face of
    its target (`gather::approach_point`), and the behaviour measures MiningReach and DropOffReach itself. Not
    `MoveGoal::Object`: that is planned to the point nearest the object's centre, the north side of every patch
    and node. A leg that ends short of its target is tried again after `LEG_RETRY_TICKS` (32, a placeholder) of
    standing; a Gather never fails or ends on its own, except with orders queued behind it, which start after its
    next drop-off.
  - **The return** reads the worker's AssignedDropOff every tick, so `Ability::SetDropOff` redirects it; without
    one it takes the owner's nearest, and with none the order ends and the worker idles with its load.
  - **A patch that leaves the game** sends its gatherers on first (`Sim::redirect_gatherers`, in the removal path):
    to another patch, or to deliver what they carry and idle.
  - **Gathering Drones pass through each other** (design change 0106; `gather::on_gather_job`, see *Movement and
    collision > Collision exceptions*): a worker of a type with `WorkerDef::pass_through` (the Drone) whose current
    order is a Gather or a Drop Off. They collide with everything else and, having an order, never yield. A leg
    that ends short (behind a unit that won't move) begins again after `LEG_RETRY_TICKS`, a back-off.
- **Not active during Move**: a Move never engages, and neither does the walk back to an IdleOrigin or to a last
  seen position.
- **Placeholders** (the design is silent; each is a comment or a named const in `orders/`): `GIVE_UP_RETRY_TICKS`
  (32): a unit doesn't re-acquire a target it couldn't get to for that long; "within SightRange" for a scan is
  measured edge to edge like Range; the PathReference is the route first planned for a leg and stays that when the
  unit resumes from elsewhere; a unit farther than the leash from it doesn't acquire; a queued Attack whose target
  was never seen while it ran completes at once.
- **Auto-targeting switch**: `DebugCommand::SetAutoTargeting { on }` (`Sim::auto_targeting`, hashed state) turns
  every scan off or on, for tests and QA runs about movement between hostile units; `TestSim::without_auto_
  targeting()` schedules it for tick 1. Attack orders work either way.
- **Adding a behaviour**: give it its state in `orders/state.rs` (with its hash, and its ids in `targets` and
  `forget`), start it in `orders::start`, run it in `orders::run`, and let it read only the `World` and its own
  unit. A behaviour that sends the unit somewhere must remember that it did (`chasing`, `marching`, `moving`):
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
  `(class, size class)` under every blocking footprint (structures and map resources), and each player's **known
  grids** (below). It is derived, like the `Nav`: not hashed, compared or serialized, built when a `Sim` is
  created or loaded. The true grids are rebuilt in phase 5b; `Sim::nav()` (the true grids), `nav_version()`,
  `nav_counters()` and `nav_rebuilds()` (sets of grids the latest step rebuilt) expose it to tests and overlays.
- **Path knowledge** (`paths/knowledge.rs`; `systems/controls/behaviors.md > PathKnowledge`, design changes 0040
  to 0042): paths are planned only from what the unit's owner knows, and units collide with what is there.
  - **What a player knows** to block the ground, besides the terrain: the footprints it owns; those of its
    vision's records (seen and not seen gone, so a structure destroyed in the fog stays a wall at its last-known
    position, and one never seen is open ground); and the map's resources it hasn't seen the place of yet, which
    it assumes (`PlayerKnowledge::assumed`: every resource the game started with, dropped the first time a tile
    under it is Visible; from then on the record says what the player knows). Only resources are "known from the
    start": a structure a Custom map pre-places is known to the others once seen.
  - **State**: `Sim::path_knowledge()` (`PathKnowledge`: per player the assumed resources and a `version`, which
    counts the changes of the set of footprints it knows) is history, hashed and serialized; a save's is checked
    on load. The known grids (`Sim::known_nav(player)`) are derived from it, the objects and the vision.
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
    game's. A resource seen for the first time moves from assumed to remembered without a rebuild.
  - **Lazy**: a player's grids exist only while it has a unit that walks on a grid; `ensure_known` builds them
    for an order in the tick of its first unit's spawn.
  - **No owner**: a neutral unit (and a defeated player's, design change 0101) plans on the true grids by
    `nav_version`: it has nobody's knowledge and sees nothing. Code that changes a unit's owner should call
    `Mover::repath`, since the unit's path was planned on other grids.
  - Measured (`tests/nav_perf.rs`, ignored, release; 4 players, 412 units and 160 patches on 256 × 256): the tick
    in which ten Generators go in sight of all four players rebuilds 5 sets of grids and takes 1.0 ms; the worst
    tick after it, with the units' path checks, 1.3 ms.
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
  start first (`Mover::next` 0). With no way out its owner knows of, the request fails like any from a blocked
  tile (see *Production*).
- **Avoid tiles** (`Mover::avoid_tiles`): a repath of a stuck unit treats them as blocked, and its smoothed path and
  its steering stay out of them (`nav::smooth_avoiding`, `nav::segment_avoids`). If they cut every route, the unit
  drops them and plans again.
- **Unit boxes** never overlap blocked tiles or another unit of their layer at spawn: the gate checks debug spawns and
  teleports against the sim's grids; scenario validation and the editor check pre-placed and layout units with
  `paths::misplaced` (terrain-only grids of the classes present, plus the footprints).
- `f32::mul_add` is a disallowed method (`docs/movement.md > 7`): write `a * b + c`.
- Timings: `cargo test -p sc_sim --release --test nav_perf -- --ignored --nocapture --test-threads=1`, like every
  `*_perf.rs` test (each reads the clock in its one timing helper, see *Determinism*). Measure on a quiet machine:
  note the load average (`uptime`) with the numbers.

## Movement and collision

The design is `docs/movement.md > 6` (binding; its **(D)** marks are where slice D changed it, **(M)** where the
magic box did, **(Y)** where friendly yielding and the answers to Q0008 did, **(Q)** where queued group orders did, **(G)** where
the hardening of crowds and nav performance did, **(H2)** where gathering Drones passing through each other did),
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
  or oncoming one (a glance along its face, a 45° step, a step along the face; to the side the unit leans to, which
  it keeps, else to its Right first, so head-on units pass like traffic; a crossing unit is passed behind); else
  close up or stay. Within one step of the
  goal the unit lands exactly on it. A unit without a goal moves only by its own sidestep (below): nothing pushes.
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
  them (`yielding::ask_ahead`): the group parts and isn't shoved along.
- **Collision exceptions** between units go in `space::blocks(mover, other)` and nowhere else (crushing will). The
  sweeps, lane seeking and `World::is_free` all ask it, so a unit ignored for a move is ignored for the whole of it.
  It is asked last, after the box test (few units near a move are in its way). Two exceptions (design change 0106,
  slice M3-H2), both read off the bodies each tick, with **no pair state**:
  - **Gathering Drones** (`systems/units.md > UnitCollision > Ground Collision > Exception, gathering Drones`):
    two bodies with `Body::gathers` (`orders::gather::on_gather_job`: a Drone whose current order is a Gather or a
    Drop Off, at every step of the job) don't block each other, whoever owns them. A gathering Drone and any other
    unit block each other as ever, an idle Drone included.
  - **Units that already overlap** (`> Units that already overlap`): two bodies whose boxes overlap as the mover's
    turn begins don't block each other. `space::overlaps` is I1's own predicate, so the pair is ignored exactly
    while I1 would name it, and normal collision is back the moment the boxes are apart (touching is apart).
    Nothing pushes them apart, and either may move any way meanwhile. Only the first exception ever brings two
    boxes to overlap, so in today's game such a pair is two Drones of which one left its job; the rule itself asks
    nothing about the units' kinds.
  A later exception (crushing) is one more clause there. A slice that adds a way for two boxes to come to overlap
  must add it to `testing::invariants::may_overlap` too, or I1 fails.
- **The exit doorway** (`core/entities.md > Structure Type > Exit Doorway`, design change 0080; `production/exit.rs`)
  is the one exception against tiles, and the one case in which a unit's box overlaps a structure. A unit a producer
  released has `Mover::exit`, the building's id, while its box still overlaps that footprint. For that unit
  alone, `World::door_free` replaces the tile tests of `tiles_free`, `is_free_for` and `along_axis`: the building's
  own tiles don't block it, every other blocked tile does (`Doorway::box_clear`), and it may only come out
  (`Doorway::allows`: never deeper in, and inside only within the exit side's span, so it leaves through that side
  and no other). Units still block it and it blocks them, as ever. Phase 5d clears `exit` in the tick the box is
  clear of the footprint (or the building is gone, or a teleport took the unit away), and nothing ever sets it
  again: a unit that is out can't re-enter. With a goal the unit moves by the ordinary rules (lane seeking, going
  around, yielding requests); without one, `doorway::leave` walks it straight out and stops it against the exit
  side (design change 0086), trying again every tick while something is in the way. **Never set `Mover::exit`
  anywhere but where a producer releases a unit, and never test a doorway unit's tiles anywhere but through
  `World::door_free`.**
- **Held by its attack**: a unit whose attack sequence holds its base (`combat::may_move` or `may_turn` false: see
  *Combat*) stands for the tick and keeps its goal, path and stuck clock (`wait`), like a unit waiting for its path.
- **Stuck and recovery** (`stuck.rs`; the numbers, in `mover/schedule.rs`, are the design's placeholders of changes
  0024 and 0026): progress is the remaining path length; waiting behind a unit that is getting on counts. A unit is
  stuck after `STUCK_TICKS` (24) without it (`Mover::is_stuck`); it then tries another route `MAX_REPATHS` times,
  after waits of `REPATH_WAITS` (8, 16, 32; `Mover::recovery_due`), around the tile of a blocker that stands, and
  gives up 24 ticks after the last: `MoveStatus::Failed` and `SimEvent::MoveFailed`, `GIVE_UP_TICKS` (104) after its
  last progress. **Occupied target**: within `OCCUPIED_ARRIVAL_GU` (2) of its goal behind a stationary unit that
  won't yield to it, the unit has arrived instead, without a repath: when stuck, or after `JAMMED_TICKS` without a
  way forward if the blocker has no goal. Two moving units head-on in a passage too narrow to pass both fail.
- **Whose progress a waiting unit shares** (`stuck::leads`, slice M3-H2): a unit blocked by one that is on its way
  takes over that unit's `last_progress`, but only if that unit **has got on since its leg began**. `Mover::go_to`
  sets a unit's own clock to the tick of the order, and that is no progress: before this rule, a blocked unit whose
  legs began again and again (a behaviour trying again, a player clicking on it) renewed the clock of every unit
  behind it, and none of them ever looked for a way round (`tests/movement_stuck.rs`). It is read off the mover
  (`repaths_left < MAX_REPATHS || last_progress > last_repath`), with no field of its own. A behaviour may begin a
  leg again as often as it likes; it need not stand in between for the sake of the units behind.
- **Object goals** (`MoveGoal::Object { id, reach }`, `docs/movement.md > 5.6`, `> 6.5`): the path is planned to
  the point nearest the object's centre that the unit can stand on (right at a structure's wall), and kept aimed at
  the object as it moves (`paths::chase_check`, on the stagger tick: straight at it while the unit's box has a
  clear line, else a new plan once it moved 1 GU or 25 % of the distance, at most every 8 ticks, never counted
  against the repath cap). The unit has `Arrived` when the gap between the two shapes is within `reach`, at the end
  of its path if the object hasn't moved, or by the occupied-target rule (the 2 GU measured as that gap). Progress
  towards a moving object is moving forward at all; towards one at rest it is measured as to a point. A unit
  chasing its attack's target behind stationary attackers of its owner on the same target **queues**
  (`stuck::queues_behind`): no repath, no failure.
- **Hooks for later slices**: other movement models replace the "direction → facing and velocity" part of
  `step_unit`.
- **Counters**: `Sim::move_counters()` (sweeps, box tests and lane tests of the latest tick) and `Sim::nav_counters()` are
  output, not state. Tests assert them; `cargo test -p sc_sim --release --test movement_perf -- --ignored
  --nocapture` prints timings.
- **Invariants** (`docs/movement.md > 8`, `sc_sim::testing::invariants`): I1 no two units of a layer overlap, except
  (design change 0106, `invariants::may_overlap`) two Drones on a Gather job, and two units that already overlapped
  a tick earlier, until they have been apart (a Drone counts as on its job for the tick in which the job ended
  without another order: a patch emptied in the economy phase ends it after the movement). Any other overlap fails,
  the first tick it exists (`tests/invariants_overlap.rs` puts units where they must not be and asks the checker); I2 no
  ground box overlaps a blocked tile or leaves the map, except a unit in an exit doorway (`Mover::exit`), which
  overlaps the tiles of the building that released it and no other blocked tile, overlaps that footprint for as
  long as `exit` is set, never moves deeper in, and never comes to be in a doorway by moving; I3 a unit moves at most its speed per tick; I4 every float
  is finite; I5 a unit without a goal doesn't move; I6 repaths per leg are capped and no path request waits longer
  than `MAX_REQUEST_AGE`; I7 an arrived unit never moves again without an order or a move of its behaviour, and a
  unit without a goal is at rest. A leg is one `Mover::go_to`: an order's move, or one a behaviour starts (a chase,
  a Patrol's next leg, the way back to an IdleOrigin). The exceptions to I5 and I7 are a unit taking its sidestep,
  which must have neither a goal nor an order, and a unit walking out of an exit doorway. K1 (`paths`): each player's known grids are those a fresh build
  from the state gives. A `TestSim` checks them after every tick and panics with a picture of the map (`TestSim::ascii`: units as
  letters, with a legend). The check is on by default; `check_invariants(false)` turns it off for timing runs. A
  test that steps a bare `Sim` keeps its own `Invariants` and calls `check` after each step.

## Economy

The players' banks, power and army caps, the plates' mining, construction, and defeat and the end of the game
(`economy.rs`, `economy/`, `construction.rs`; slices M2-B, M2-D1, M2-F2 and M3-C; `factions/gdo/overview.md >
GDOResources`, `factions/gdo/economy.md`, `factions/colonists/overview.md > ColonistResources`). Production has a
section of its own.

- **Banks** are the only resources stored: `Player::crystals` and `Player::supplies`, and the Colonists' refined
  banks `Player::alloys`, `essence` and `flux` (slice M3-C, plan decision T21; nothing earns or costs them yet). All
  are `u32`, hashed, and `#[serde(default)]` so that older states load. A game starts them at the player's faction's
  Starting Conditions (`Player::starting`, `StartingConditions`, slice M2-F1; see *Sim scenarios and tests*): a GDO or
  Colonists player's at 100 Space Crystals and the rest at 0, those of a faction not implemented yet at 0.
  `DebugCommand::SetResources` sets all five (`Player::set_banks`; the refined ones `serde(default)` in command files
  too), and `Player::banks` reads them as one `Banks`. Pay with
  `Player::spend_crystals`, which refuses when the bank is short and leaves it as it was; credit with
  `Player::credit_crystals`, which saturates (only a debug command fills a bank that far, and input never panics the
  sim). Every refund goes through `economy::Refund` (`Full`, or `Ready`: 75 %
  rounded down). Supplies are a counter only: nothing costs them yet (design change 0077).
- **Power and the army caps are derived** from the objects whenever they are asked for, never stored, so they can't
  drift (plan decision T7). `Sim::power(player)` is a `PowerSummary`: `generated` and `consumed`, the HUD's
  used / available (design change 0090). `Sim::army_cap(player)` is an `ArmyCap { resource, used, available }` for
  the player's faction (`economy/army.rs`, plan decision T20; `None` for the Syndicate and the Cults until their
  milestones): the GDO's Unit Control, out of `UNIT_CONTROL_CAP`, and the Colonists' Beacon Capacity, out of
  min(`BEACON_CAPACITY_CAP`, `BEACON_CAPACITY_PER_BUILDING` × their Colonists structures). Used is the
  `ArmyResource::cost_of` of the player's units plus what the units in their producers' queues reserve (design
  change 0082; Q0041 point 1 for Beacon Capacity: a unit counts from the moment it is queued, the one in production
  and one waiting inside included). Capacity that falls below what is used leaves the units and the queues alone
  and refuses new units (design change 0112; the queues, Q0041 point 2). `Sim::unit_control(player)` is the GDO's
  used figure alone, which the HUD reads. A building counts from the tick it is placed until the tick it leaves the
  game, that one excluded (design change 0089), because it is among the objects in between. Each call walks the
  objects once.
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
- **Gathering** (`economy/gathering.rs`, `orders/gather.rs`, `sim/workers.rs`; slice M3-H, plan decision T17;
  `factions/colonists/units.md > Drone > Gathering`, `factions/colonists/economy.md > Egress Node > Drop Off`): a
  unit type with a `WorkerDef` (`UnitDef::worker`: the Drone's CarryCapacity 8, MiningDuration 32, DropOffDuration 2
  and the two reaches of 8 su) is a worker, and a structure type with `StructureDef::drop_off` (the Egress Node)
  takes its loads. `Object::worker` (`Worker`) holds only what outlives an order: the load, the AssignedDropOff
  and the patch of its latest Gather. The job is an order and its behaviour (see *Behaviours > Gathering*), and
  **the timed work is part of the order's state** (`GatherPhase::Mining { done }`, `DroppingOff { done }`), so an
  order that ends takes its unfinished work along and frees the patch at once. `gathering::step` counts the
  frames in the economy phase, by *Recipe timing*: a worker that reaches its patch in tick `t` holds its load
  (`min(8, remaining)`, taken from the patch) at the end of tick `t + 31` and the patch is free in `t + 32`; one
  that reaches its drop-off in tick `t` is paid at the end of `t + 1` (`SimEvent::ResourcesDelivered`). A patch
  emptied this way leaves the map in the same tick, unless a plate stands on it; workers never gather a plated
  patch. `Sim::returning_drones(node)` is the Egress Node's F.
  **Measured on the stand-in mover** (`tests/gathering.rs`, `tests/gathering_soak.rs`; again when slice M3-G brings
  DragMovement): one Drone with its patch 2 GU from the node, edge to edge, delivers every 60 ticks, 2.13 Space
  Crystals per second. On the reference field (8 patches in a row, the node 2 GU south of the middle), with
  gathering Drones passing through each other (design change 0106), nothing jams and every Drone keeps delivering:
  4, 8, 12, 16, 20 and 24 Drones dealt out over the patches give 8.3, 15.1, 22.0, 26.8, 32.0 and 32.0 per second,
  and the longest any Drone goes without a delivery is 70, 82, 140, 200, 167 and 141 ticks; sent by one right-click
  they give 8.2, 15.0, 21.9, 27.4, 28.9 and 32.0, with gaps of 85 to 284 ticks (the acceptance bound is 480, 30 s).
  The 24-Drone soak costs 13 µs a tick in release.
  The patches are the only limit: 8 of them give 32 per second (the design's "fully saturated ≈ 32 (~24 working
  Drones)"). Below saturation the stand-in is ahead of the table (16 Drones: 27 for the table's 23), as expected
  until M3-G. Under hard collision, as slice M3-H first had it, the field gridlocked from 12 Drones on.
- **Removal** (`Sim::remove_objects`): a plate takes its patch along if that is depleted, in the same tick, which
  opens the tile (design change 0038; the caller's grid sync covers it); a patch with crystals left stays, uncovered.
  A plate whose patch is removed (only a debug command does that) loses its extractor. `testing::referenced_ids`
  lists the extractor's patch, so `dangling_references` checks it.
- **Construction** (`construction.rs`, `sim/construction.rs`; slice M2-D1;
  `factions/gdo/structures.md > DeploymentCenter`, `factions/gdo/economy.md > ExtractionFacility`): a structure type
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
  `Constructor::{current, ready, frames_done}` and `Sim::validate` are what a client needs for its menus. Not built:
  tech prerequisites (no building with a cost has one yet) and ConstructionHP.
- **Adding to the economy** (slices M2-D2, M2-F): put the new step into `economy::step` at its numbered place, advance
  progress by `PowerSummary::speed` from the tick the command applies (*Recipe timing*), take costs with
  `spend_crystals` when the command applies, and give refunds through `Refund`; `construction.rs` is the pattern.
- **Defeat and the end of the game** (`economy/defeat.rs`, slice M2-F2, plan decision T8; `core/game_rules.md >
  Defeat Condition`, `> On Defeat`, `> End of Game`; design changes 0100 to 0102):
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
    for them, so a defeated player owns nothing for good.
  - **Sides**: the sim has no team or alliance data yet (fixed teams are deferred, `core/map_editor.md > Map Types >
    Custom`), so every player is their own side (`defeat::allied`) and `Sim::allies` is empty. Teams go there.
  - **The end**: the defeat that leaves at most one side standing ends the game in its tick: `Sim::outcome()` is
    `Outcome { tick, winners }` (hashed, serialized, `#[serde(default)]`), after the tick's `PlayerDefeated`s comes
    `SimEvent::GameOver { winners }`. The winners are the side left standing, its defeated members included;
    nobody wins when the last sides fall in the same tick (a placeholder: the design doesn't say). From then on the
    gate refuses every command (`Reject::GameOver`). The sim would step on if asked; the client stops it (see
    *Client architecture > Sim driver*).

## Production

Producing structures, their rally points and the exit doorway (`production.rs`, `production/`, `sim/producers.rs`;
slice M2-D2; `factions/gdo/structures.md > Barracks`, `core/entities.md > Structure Type > Exit Doorway`).

- **State**: `Object::production` (`Production`: the BuildQueue, whose head is the unit in production, so the
  limit `QUEUE_LIMIT` (20) includes it, design change 0081; its progress in `RATIO_ONE`s; the rally point). What a
  type produces is `StructureDef::production` (`ProductionDef`: the exit side's letter and the `Recipe`s: kind,
  Space Crystals, frames). `Sim::production_status(id)` is what the interface shows: the kind, the progress out of
  its total, and `waiting`.
- **Commands**: `Command::Production { objects, command }` with `ProductionCommand::Train(kind)`,
  `CancelProduction` and `SetRally(Target)`; the gate's rules are in *Commands, events and replays*.
- **Costs** (plan decision T7): a unit's Space Crystals are paid and its army cap (Unit Control, Beacon Capacity; see
  *Economy*) reserved when it is queued (design change 0082). A unit past the cap is refused for every faction with
  the same reason (`Reject::UnitControlCap`, `Refusal::UnitControl`, plan decision T20). Cancelling refunds in full
  (`Refund::Full`) and takes the last entry. When a producer
  leaves the game, destroyed or removed, `Sim::remove_objects` refunds its whole queue the same way, a finished
  unit waiting inside included (design changes 0084, 0085; `Sim::refund_production`).
- **Group production** (design change 0083): a `Train` to several producers serves them by the work they have
  queued, least first (`Production::remaining_work`: remaining production time, a placeholder reading), ties by
  id, each check counting what the ones before it took, until a cost check (crystals, the army cap) fails. A full
  queue is passed over. The others get `SimEvent::ProductionRefused`.
- **The step** (`Sim::step_production`, see *Tick pipeline*): the unit in production advances by
  `PowerSummary::speed` (design change 0089), capped at its total. Trained in tick `t` at full power, a
  Peacekeeper (80 frames) is released in tick `t + 79`; at 30/51 in tick `t + 135`. The next one starts from 0 in
  the tick after a release.
- **The exit doorway** (design change 0080; `production::Doorway`, the one geometry for the release, the movement
  step and the invariants): the middle of the side lettered `ProductionDef::exit` (the Barracks' B), wherever it
  faces (`sides`). A finished unit comes into the game there **half inside the building**: its centre on the exit
  side, facing out, with `Mover::exit` set (see *Movement and collision* for the collision exception). When
  another unit or a blocked tile is right in front of the doorway it stands as far out as there is room, fully
  inside against the exit side at most (`Sim::doorway_spot`): the design's "half inside" would overlap what stands
  there, and its verification wants the unit to appear all the same. **The doorway is occupied** while a unit the
  building released still overlaps its footprint; meanwhile the next finished unit is held (progress at its
  total, `waiting`), the queue behind it doesn't advance, and it is released in the first tick the doorway is
  free. So at most one unit overlaps a building at a time.
- **Rally points** (`production/rally.rs`): `Rally::Point` or `Rally::Object { id, at }`. A released unit gets the
  order `rally::order_for` gives, through `Sim::give_order` like a player's: Move to the point; Attack an enemy
  object (if the unit attacks and the object is destructible); Gather, for a worker on a patch it can gather
  (question Q0037 point 9); Move to any other object. Without a rally point it
  walks out of the doorway and stops (design change 0086). The producer itself as the target clears the rally. A
  rally on an object that is removed resets to None (`Production::forget`). A rally on an **enemy** follows it only
  while the owner sees it: at the end of each tick `rally::follow` notes where the owner sees it, or turns the rally
  into a ground rally at the last such position (design change 0088), so an enemy dying in the fog changes nothing.
  **Placeholder** (0088's TBD, question Q0017 point 2): a rally set on a remembered object out of sight is a ground
  rally at the centre of its record's footprint.
- **Tests**: `src/production.rs` and `production/exit.rs` (the queue, the doorway's geometry in all 8
  orientations, its rule), and `tests/production/`: `queue.rs` (times, the power ratio, train and cancel, the limit
  of 20, Unit Control, the gate, group production, refunds, a game replayed and saved), `rally.rs` (each kind of
  target, clearing, an enemy leaving sight, a remembered structure), `exit.rs` (the verification of design change
  0080, all 8 orientations, a crowd of idle units, a half-blocked exit, a path from inside the footprint, no way
  back in) and `stalls.rs` (walled in by a building and by units and freed again, the waiting unit's refund, saves
  in the middle of an exit, and a random game of 3 000 ticks around four doorways).

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
  against the current state without changing it, and `step` applies a command only through the same check
  (`Sim::check`, which hands the application the objects and positions it resolved), so the two can't disagree. In
  order: the player is in the game (`Reject::UnknownPlayer`), the game isn't over (`GameOver`) and the player isn't
  defeated (`Defeated`, see *Economy > Defeat and the end of the game*), a debug command needs debug commands on
  (`DebugDisabled`), then the command's own rules: every id it names exists (`UnknownObject`) and, for a player's
  orders to objects, belongs to that player (`NotOwned`, through `Sim::referenced(id, Some(player))`; debug commands
  and targets may name anyone's objects), positions are finite and on the map (`OffMap`), an order's target
  object is in the issuing player's sight (`NotVisible`, below), spawns fit (`Spawn(SpawnProblem)`) and keep apart
  from other footprints (`Overlaps(id)`, see Object model). A rejected command changes nothing, neither the state
  nor the events, and never panics.
  The one rejection the sim reports is a Place the placement check refuses (plan decision T4): the client checked it
  before sending it, so the game changed in between, and `SimEvent::PlacementRefused` (output, outside the state
  hash) says why (`Sim::report_refusal`). Add another such event only when the UI needs it.
- **`Command::Order { units, kind, queue }`** (`systems/controls/commands.md > Unit Command`, `docs/movement.md > 3`):
  `OrderKind` is `Move(Target)` (a point or an object), `Attack(id)`, `AttackMove(point)`, `Patrol(point)`, `Hold`,
  `Stop`, and for workers `Gather(patch)` and `DropOff(node)` (`factions/colonists/units.md > Drone > Unit
  Commands`). The gate takes an order whole or not at all, but tolerates stale selections: at most `MAX_ORDER_UNITS`
  ids (`TooManyUnits`), repeated ids count once, ids of objects that are gone and objects that take no orders are
  dropped, and nothing left is `NoUnits`; any id of another player's object rejects it (`NotOwned`). Points must be
  finite and on the map, target objects must exist (`UnknownObject`) and an attack's target be destructible
  (`NotAttackable`). A Gather's target is a Space Crystals patch with no plate on it (`NotGatherable`) that the
  player sees, a Drop Off's a drop-off of the player's own (`NotDropOff`), and only the workers among the units
  take either (`NoWorkers` if there is none; `sim/workers.rs`). Without `queue` the order replaces the unit's queue and current order; with it, the order is
  appended (a full queue drops it). Each order command is one group (`Sim::next_group`); a location order gives
  each unit of a group its own destination (the magic box, see *Navigation*). What each order does is in
  *Behaviours*.
- **`Command::Ability { units, ability }`**: the units use an ability at once, beside their orders: no queue, no
  Shift, no group, and their orders go on. `Ability::RiotShield { raise }` says which way rather than "toggle", so a
  group and a replay mean the same whatever each unit's mode. `Ability::SetDropOff(node)` sets the workers'
  AssignedDropOff to a drop-off of the player's own (`NotDropOff`), and "does not interrupt the Drone's current
  action" (`factions/colonists/units.md > Drone > Assigned Drop-off`). The gate tolerates stale selections as for orders
  (`TooManyUnits`, repeats, gone ids, `NotOwned`, `NoUnits`) and drops the units without the ability
  (`Reject::LacksAbility` when none is left).
- **`Command::Production { objects, command }`** (`factions/gdo/structures.md > Barracks >
  ObjectInterfaceState[Barracks]`, see *Production*): the producers are the player's objects among `objects` that
  produce units, found as an order's units are (stale and repeated ids dropped, `NotOwned`); none is
  `Reject::NoProducers`. `Train(kind)` is taken when at least one producer queues the unit, else
  `CannotProduce(kind)`, `InsufficientCrystals`, `UnitControlCap` or `QueueFull`; `CancelProduction` when a producer
  has something queued, else `NothingQueued`; `SetRally` with a point on the map (`OffMap`) or an object that
  exists and that the player sees, as for an order's target (`UnknownObject`, `NotVisible`; the producer itself
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
  sends a command to the group the design says (no smart-casting, design change 0071).
- **Targets must be seen** (question Q0017 point 2, its default; `core/vision.md > Explored Selectability`): an
  Attack or a Move on an object is taken only while the issuing player sees it (a tile it covers is Visible), else
  `Reject::NotVisible`. The exception is the design's: a GDO player may target a Supply Delivery Station on Explored
  ground, through its last-known record of it (`PlayerVision::remembered_record`), as the client selects it there: a
  station it never saw, e.g. one debug-spawned on Explored ground, is no target. The check is on the sim's vision,
  whatever the client draws (`--no-fog` is display only). A queued order
  passed the gate when it was given; what its behaviour does if the target is out of sight when its turn comes is
  in *Behaviours*.
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
  *Behaviours*) and `SetResources` (any player's banks, see *Economy*; the gate checks that the player it names is in
  the game, `UnknownPlayer`), for development and QA. `Spawn` places a tile-based object in an `orientation`
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
  `PatchDepleted`, `ProductionQueued`, `ProductionRefused`, `ProductionCancelled`, `UnitProduced`,
  `ConstructionCompleted`, `ConstructionCancelled`, `StructurePlaced`, `PlacementRefused`, `PlayerDefeated`,
  `GameOver`, `KindChanged`, `ResourcesDelivered`, …) are output only, for effects, sounds and UI. The sim never reads them.
  `Sim::events()` holds only the latest step's events, and `Sim::new` discards the events from the scenario's
  initial spawns. `KindChanged { id, from, to }` (`Sim::change_kind`) tells the client to re-mesh an object and
  regroup the selection; that response arrives with its first caller, slice M3-B.

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
    slots no player takes, then each player's Starting Layout for its faction at its slot. A slot's objects belong to
    the player who takes it.
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
  (`InvalidLayoutObject`), and nothing else. Layouts of different factions may overlap. `Scenario::validate` adds the
  layouts of the players of one game in each other's way, once the map has no problems of its own.
- **Starting Conditions** (`scenario/starting.rs`, slice M2-F1, plan decision T11): `StartingConditions::of(faction)`
  is a faction's table (`factions/gdo/economy.md > Starting Conditions`; the Colonists' since slice M3-C,
  `factions/colonists/economy.md > Starting Conditions`: an Egress Node, a Robo Portal and four Drones): its starting
  objects with their counts, its starting banks, and its primary structure, which the camera starts on. A faction
  without one (the Syndicate and the Cults) is "not implemented yet". A game starts each player's banks from it
  (`Player::starting`). On a Skirmish Default map
  (`> Skirmish Default restriction`, design changes 0009 and 0012) a layout holds only its faction's Starting
  Conditions objects, no more of a kind than they do (`LayoutExtra`; for a faction not implemented yet, nothing), and
  every implemented faction's layout is complete at every start (`LayoutMissing`), so a new Skirmish Default map isn't
  playable until its layouts are placed. "Outdated" layouts (`MapDef::outdated_layouts`: holding objects, but not
  exactly the current Starting Conditions) are derived, never stored, so the map format stays at 3. When a faction
  gets its table, every Skirmish Default map in `maps/` needs its layouts (`the_maps_in_the_repository_are_playable`
  says which).
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
  Portal on (6, 6), the Drones at points clear of the node's sides and the portal's), gentle hills of slopes only,
  and a central plateau ringed by cliffs with a ramp up through a gap in the west and in the east
  (`maps/dev_skirmish.json` is the same map, which a test checks);
  `dev_structures`, a flat 30×12 Custom QA map with one of each GDO structure for each of two players (a plate on a
  patch in each base, slot 2's Barracks turned 90°); and `empty`, a flat 32×24 Custom map without symmetry.
  `scenarios::empty(w, h)` builds an empty scenario of any size, `empty_map(w, h)` its map.
- Unit tests sit in their module (`#[cfg(test)] mod tests`). Scenario and integration tests go in `crates/sim/tests/`
  and use only the public API.
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
  it changed by what the plates mined (read off their patches), less what the step's orders paid, plus the refunds
  its events report.
- **Movement scenario tests** are `crates/sim/tests/movement_collision.rs` (passing, head-on, choke, group arrival,
  idle units, repaths, oscillation bounds, save and load in a crowd), `group_destinations.rs` (the magic box),
  `queued_group_destinations.rs` (Shift-queued group orders),
  `movement_yield.rs` (friendly yielding, occupied targets, the give-up schedule), `movement_crowds.rs` (blocks
  meeting head-on, a group through an idle block, dense blocks of any shape), `movement_soak.rs` (200 units,
  random orders, 2 000 ticks; bounds on the share of failed moves), `movement_stuck.rs` (whose progress a waiting
  unit shares: a unit behind one whose legs begin again finds its way round), `invariants_overlap.rs` (I1's two
  exceptions and nothing more, on edited states) and the ignored `movement_perf.rs`.
- **Behaviour scenario tests** are `crates/sim/tests/behaviours_attack.rs` (the approach to a gap of Range, the
  chase, the last seen position, a target out of reach, queueing), `behaviours_idle.rs` (acquiring by priority,
  the leash and the return, Hold, Move doesn't engage, the auto-targeting switch), `behaviours_travel.rs` (Follow,
  static objects, AttackMove groups and the leash, a fight replayed and saved in its middle),
  `behaviours_patrol.rs`, `behaviours_gate.rs` (targets must be seen; a GDO player's remembered station, by its
  record), `behaviours_soak.rs` (120 units fighting under random orders of every kind, then without orders until
  those end; its checks are invariants that survive tuning: no dangling ids, no idle unit in reach of a visible
  enemy (but one whose scan held an enemy that died in that tick), every order but Hold, Patrol and Follow ends, and
  bounded shares of refused orders, failed moves and path searches) and the ignored `behaviours_perf.rs`.
  `riot_shield.rs` has the Riot Shield's: toggle timing, each raised-shield effect, the gate, a save in the middle
  of a toggle and its replay.
  Measured (release, 500 units on 256 × 256; slice M2-J2, within 7 % of M1's in an interleaved A/B): 0.13 ms a tick
  with everyone idle or on Hold in sight of the enemy, 0.29 ms on average and 0.65 ms at worst in a battle of 250
  against 250.
- **Resource collision tests** are `crates/sim/tests/resources_block.rs` (design change 0038's verification: units
  path around a crystal field, can't walk across a Supply Delivery Station, and walk through the tile of a patch
  removed by a debug command, standing in for depletion; `economy_mining.rs` has the real thing, below).
- **Path knowledge tests** are `crates/sim/tests/path_knowledge.rs` (the verification points of design changes
  0040 and 0042: a hidden wall of enemy buildings, a scouted building destroyed out of sight, a crystal field never
  seen, a patch gone in the fog; own structures; an unseen structure is open ground for plans and solid for
  units; lazy grids; saves with stale knowledge) and the ignored `nav_perf.rs` (`knowledge_rebuilds_256`).
- **Economy tests** are `crates/sim/tests/economy_mining.rs` (lumps every 48 ticks and the first 48 ticks after
  placement, the cycles under the power penalty and across a change of ratio, the loads down to the residual, a plate
  gone from a patch with crystals left or from a depleted one, by a debug removal or in combat, after which a unit
  walks through the opened tile, a patch mined out in the fog that another player assumes until it sees the tile
  (design change 0042), and the links in any order) and `economy_resources.rs` (the banks and
  `SetResources`, power and the ticks a building counts in, Unit Control, a mining game replayed and saved).
- **Construction tests** are `crates/sim/tests/construction.rs` (each building's frames at full power and the plate's
  680 ticks at 30/51 beside an unslowed Deployment Center, the refunds and their events, every refused command changing
  nothing, a group command in id order, a Place in its orientation with power, area and grids following, a refused Place
  for each problem of the check and one refused by an earlier command of its tick, destroyed constructors' refunds by a
  debug removal and in combat, a fuzz of the gate, a save and a replay). The clean Barracks opening, which slice
  M2-D1 measured there first, is the economy oracle's.
- **The economy oracle** is `crates/sim/tests/economy_oracle.rs` (slice M2-J1): GDO openings on
  `maps/dev_skirmish.json` from its real start, played with `testing::build_order` (player 2 idle), against
  `factions/gdo/economy.md > Opening Economy Baseline` and the GDO income of `> Economy Targets`: plates first (the
  reference line: income at the table's times, plates every 25 s, fully plated at about 2:30, the float at 1:30 and
  2:00), Generator first, the fastest clean Barracks (pinned: started in tick 769, ready at the end of 928), the
  all-in Barracks at −21, and the refunds. Income is the average over the 9 s (3 lump cycles) ending at a time; the
  tolerances on the design's trickle figures are named where they are used (a plate's first lump 3 s after it
  lands, a wait for money up to one cycle longer, a bank under a trickle's by less than a lump per plate), while
  costs, build times, lumps and the power penalty are checked exactly. `--nocapture` prints each line's timeline.
- **Game start tests** are `crates/sim/tests/game_start.rs` (slice M2-F1: each player's Starting Layout and banks,
  the Skirmish Default restriction and completeness, the layout rules of design change 0097, a layout plate off its
  patch refused by the game's rule, a pre-placed one by the same rule, two players' layouts in each other's way, the
  outdated flag, and `dev_skirmish`'s bases seeing and reaching their fields) and the unit tests of
  `scenario/starting.rs` (the table, and a layout made for other Starting Conditions). `colonists_start.rs` (slice
  M3-C) has the Colonists' start on `dev_skirmish` in either slot, its Drones clear of the node and the portal, the
  validation of every faction combination and of an incomplete layout, and Beacon Capacity: six Drones queued and
  the seventh refused, group production up to the cap, a fall in capacity, and the refined banks.
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
- **Gathering tests** are `crates/sim/tests/gathering.rs` (slice M3-H: one Drone's trip, pinned at 60 ticks; two
  Drones arriving together; waiting in the order they came; an enemy Drone on the patch; Set Drop-off Location in
  the middle of mining and of a return; depletion with the partial last load, the opened tile and the redirect, or
  the delivery and the idling when no patch is left; the Drop Off order and the way back to the patch; a lost
  Egress Node; interrupted mining; an order queued behind a Gather; every invalid worker command refused and
  changing nothing; a Drone rallied on a patch; a game replayed and saved in its middle) and
  `gathering_soak.rs` (the four starting Drones for 5 minutes; 24 Drones for 5 minutes, every one delivering at
  least every 30 s, the saturated income, nothing lost; the income growing with the Drones up to saturation; 9
  Drones on 3 patches, once the smallest gridlock, with a steady income; ignored: `saturation_curve`, which prints
  the measurements) and `gathering_pass_through.rs` (design change 0106's Verification: two players' Drones on one
  row; an idle Drone parked in the strip, an enemy's and the gatherer's own; a Drone ordered out of a mining Drone,
  and solid once apart; a Drone stopped inside another, not pushed apart).
- **Game end tests** are `crates/sim/tests/game_end.rs` (defeat by a removal and in combat, plates counting; no
  defeat without a structure ever owned; the units made neutral and idle; every command of a defeated player
  refused; a 1v1, three players defeated one after the other, and a last defeat shared by two; no producer handed
  over; a save across a defeat and the end, and old saves) and, in the client, `crates/game/tests/game_end.rs` (the
  banners, the stop, the saved recording, QA's end, the spectator's fog). The fuzz games of `robustness.rs` give
  each player a base it can't remove, so that they never end.
- **Determinism tests** compare `Sim::state_hash()`, an FNV-1a 64 hash of the whole state except the events. Run the
  same scenario and inputs twice, or play back a `Replay`, and compare the hashes (`determinism.rs`, with the replay
  checkpoints). Every state type's hash impl destructures it (`hash.rs`), so a new field doesn't compile until it is
  hashed. The golden games (`golden.rs`) pin the hashes from build to build: see Replays and golden hashes.

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
- **Draw order** follows StarCraft 1. The layers are terrain < ground decals < ground objects < air < overlays, each
  with its own z band. Within a layer, z grows with **ground** y, so southern objects draw in front. Each sprite is
  anchored at its ground position, the sim `pos`, and that point drives its y-sort; it is drawn at its height
  (`ground_translation`; a tile-based object at its footprint's mean corner height, `footprint_height`).
- **Terrain** (`map_render.rs`): one mesh. A flat tile is a quad, a sloped one four triangles around its center (at
  the corners' mean height), each shaded by Lambert's law towards `LIGHT_TOWARDS` (a placeholder: from the north-west,
  45° up), so flat ground keeps its colour. A non-Traversible tile at least `CLIFF_MIN_SLOPE` levels steep reads as a
  cliff face: rock-coloured, with a stratum line at every level. Every vertex's z grows with its ground y, so with the
  depth test southern terrain overdraws the terrain behind it. The grid (G) follows the surface in the terrain's z
  band, slightly in front, so terrain in front hides it too. Both meshes are rebuilt when the sim's map changes.
- **Units** (`object_render.rs`, `object_render/units.rs`) are placeholder shapes until the art (Q0001): a shape per
  kind (the Peacekeeper round, the Guard and the Legionnaires square, the Drone an octagon) of the silhouette's size,
  in the owner's colour (`player_color`, the editor's slot colours), with a dark rim and a light tick along the
  facing. Only a round body turns with the interpolated facing (`body_turns`). A square or an octagon stays
  axis-aligned like its collision box, and only its tick turns (a `FacingTick` child, `has_tick`). Structures and map
  resources never turn, whatever their facing (a structure keeps a unit's facing through `Sim::change_kind`). The
  Colonists' units carry a `UnitTag` child, a `Text2d` of the type's short label (DR, SL, RL), because the
  Legionnaire's two forms share a silhouette. One mesh per `MeshKey` (kind, owner, and a structure's size as placed)
  is shared, and one tick mesh per kind. The map resources' shapes are in `object_render/resources.rs`. A unit that
  carries a Riot Shield has a `RiotShieldVisual` child: a pale arc in front of it, hidden while the shield is down,
  growing to its full size as it goes up. It is `Visibility::Hidden` or `Inherited`, never `Visible`, so the fog that
  hides its unit hides it too.
- **Structures** (`object_render/structures.rs`) are placeholders too: the footprint as placed, in the owner's colour
  inside a dark rim, drawn `INSET_SU` (8) inside its edge so that a selection frame shows around it, and over a map
  resource's placeholder (a plate covers its patch). A `StructureTag` child, a `Text2d` of the type's short label
  (`hud::selection_panel::short_label`), names it; the fog hides it with its structure. A remembered structure
  (`remembered.rs`) looks the same, in its owner's colour, dimmed, its tag too.
- **Path overlay** (`path_overlay.rs`, P or `--show-paths`): each unit's remaining path (`Mover::remaining_path`) as
  a thin line from where the unit is drawn, cut into short pieces lifted by the terrain's height, in the overlays
  layer, and a frame around each unit in the colour of its move's status (`status_color`: green moving, amber held
  up, red failed, blue arrived; placeholder colours). Its one mesh is rebuilt every frame while it is shown.
- **Combat effects** (`combat_effects.rs`, placeholders until Q0001): a tracer for each `SimEvent::Attacked`
  (`TRACER_TICKS`), or a flash of the object hit if the local player sees it but not the tracer, an attacker in the
  fog (`shot_view`, `HIT_FLASH_TICKS`; Q0017 point 1's assumption), HP bars over damaged units, a fading marker for
  each `SimEvent::Died` (`DEATH_MARKER_TICKS`).
  One alpha-blended mesh in the overlays layer, rebuilt every frame while anything shows; effects age by the sim's
  tick and the render alpha, so screenshots stay deterministic.
- **Fog of war** (`fog.rs`, `core/vision.md`): `Sight` says what the local player is shown: its own objects, and
  everything on its Visible tiles (`Sight::sees`); on Explored tiles the terrain and the player's records of the
  structures and resources it saw there, in their last-known state, never the live objects (`Sight::remembered`,
  see Vision); on Unexplored tiles nothing. Object visuals it doesn't see get `Visibility::Hidden`; the records get
  visuals of their own (`object_render/remembered.rs`, `RememberedVisuals`: one entity per record shown, dimmed
  like the Explored ground, a placeholder; synced when the sight changes). Tracers, death markers, HP bars, the
  path overlay, the selection (`selectable_by`) and with it what a click or the pointer targets, the InfoPanel and
  the command indicators ask it too. The fog layer is one mesh that follows the terrain's surface tile
  by tile, in the terrain's z band `FOG_Z` in front of it (so no cliff stratum shows through), and one texture of
  one black texel per tile (`UNEXPLORED_ALPHA`, `EXPLORED_ALPHA`, clear when Visible), sampled linearly. The texture
  is written only when `Sight::revision`, the local player or the sim changes, the mesh only with the map.
  A `Sight` looks through the union of the visions it watches (`Sight::watching`): a tile in the best state any of
  them gives it, an object out of sight of all of them as the latest record any of them keeps. A player watches
  their own vision, and a defeated one their allies' too (`Sight::new`, `Sim::allies`; design change 0102: never a
  full map reveal); `Sight::revision` holds whom it watches. Without teams a defeated player watches their own
  vision alone, which has no Visible tile left.
  `FogOfWar` turns it off for QA (`--no-fog`, the debug key F3, which a replay's playback takes without debug
  commands too): display only, the sim computes every player's
  vision anyway. The editor shows no fog; its play-tests do.
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
(`core/camera.md > Starting Position`): the GDO's Deployment Center or the Colonists' Robo Portal, where the player's
Starting Layout places it (`Scenario::primary_structure`). Without one (a faction not implemented yet, or a layout
without it) the slot's
start position stands in, and the map centre for a player without a slot (`camera::start_center`, shown through
`start_view`). Only the editor zooms (`CameraZoom`).

**HUD**: built with bevy_ui (`core/camera.md > HUD Layout`). It has a top resource bar and, at the bottom, the HUD
Bar with five fixed sections. From left to right they are Minimap, Portrait, InfoPanel, SelectionPanel and
CommandPanel. All five have the same height and keep fixed horizontal slots, and a hidden panel's slot stays reserved
(`systems/controls/selection_and_hud.md > HUD Bar`). bevy_ui lays out each root node inside its camera's viewport,
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
constructor's (`activity::construction`, first) are the building under construction and its progress (frames done of
its recipe's, whole percent rounded down), or the building ready to place. A plate
adds the remaining crystals of the patch under it, for every viewer (0099). The top bar shows the player on the left,
the tick on the right (in a replay's playback also the replay's last tick and a note on the playback, `ReplayText`:
speed, paused or end, and a desync in red), and between them, centred, the local player's resources as their faction's
DisplayHud lists them (`hud/resources.rs`, one `ResourceFigure` text each, `resource_bar` for their readings): for
the GDO Space Crystals and Supplies (the banks), Power as consumed / generated (`Sim::power`), red while consumption
exceeds generation (0090), and Unit Control as used / `UNIT_CONTROL_CAP`, read through `Sim::unit_control` alone, so
that the queue's reserved Unit Control (0082) shows as soon as the sim counts it; for the Colonists (slice M3-C,
`factions/colonists/overview.md > DisplayHud`) Space Crystals, Alloys, Essence and Flux, and Beacon Capacity as used
/ available (`Sim::army_cap`; red over capacity, 0112), with no Power; the other factions Space Crystals only, until
their own resources exist. At the end of the game a banner across the world view (`hud/outcome.rs`,
placeholder look and wording) shows the local player's `Banner`: VICTORY for the winners, DEFEAT for everyone else,
and DEFEATED with a spectating line while a defeated player watches the game go on. A defeated player, and everyone
once the game is over, gives no commands: the command UI's input set doesn't run (`outcome::may_command`), and
nothing is `commandable` after the end.

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
- Double taps and double-clicks are timed on `Time<Real>`. Placeholders until the design gives values:
  `DRAG_THRESHOLD_PX`, `DOUBLE_TAP_SECONDS`, `DOUBLE_CLICK_SECONDS`, `DOUBLE_CLICK_DISTANCE_PX`, `EDGE_PAN_MARGIN_PX`,
  `PIXELS_PER_LINE` (wheels that report pixels).
- **Injection**: tests and QA scripts drive the layer through `InjectedInput`, a virtual keyboard and mouse, without
  winit. Press and release its `keys` and `mouse` (`ButtonInput`s) like Bevy's; the layer clears their press marks
  after reading them. `wheel` turns the wheel by that many lines before the next frame, `cursor` stands in for the
  mouse cursor, `over_ui` puts it over a UI node (headless apps have no UI picking), and `held` and `pressed` inject
  actions directly, past the bindings. Never write to Bevy's own `ButtonInput` resources: its input systems clear
  their presses before ours run. `RawInput` turns the real devices off in QA runs.

**Selection** (`selection.rs`, `selection/`; `systems/controls/selection_and_hud.md`): client UI state, never sim
state. `ControlStates` holds every player's `ControlState`, a `Selection`, 10 control groups and the
ObjectInterfaceState (see Commands UI); the input changes and the HUD shows the local player's (`LocalPlayer`).
- **Model** (`selection/model.rs`): a selection is a set of object ids with their kinds. Its SelectionGroups are
  derived: one per type, each by id, and one per instance of an ungroupable type (only resources so far, which are
  never selected with other objects, design change 0048). They come in one stable order, by `ObjectKind` and then by
  instance (`Selection::groups`), which is the order of the SelectionPanel's portraits and of Tab and Shift + Tab (the
  placeholder of 0055). The ActiveGroup is one SelectionGroup, named by its `GroupKey` (0048): Tab and Shift + Tab
  cycle it with wrap-around, adding objects keeps it, and it falls back to the first group when its group empties. At
  most `MAX_SELECTED` objects (a placeholder: the order limit).
- **Picking** (`selection/pick.rs`): `Selectables` is what the local player may select this frame (`selectable_by`:
  its own objects, others on Visible tiles; on Explored ones only a remembered Supply Delivery Station, at its
  record, and only for a GDO player, `remembered_selectable_by`, `core/vision.md > Explored Selectability`; with
  the fog off, everything), each object's box as it is drawn: interpolated, lifted by the height it stands at, in
  view space. A click picks the frontmost box under it (grown by `PICK_SLOP_SU`), and of boxes drawn at one depth
  anything before a map resource, so a click on a covered patch's tile picks its plate (design change 0099); a box
  (the input layer's `SelectionBox`) applies the priority tiers. `Relation::of` says own, enemy or neutral (no teams
  yet, so no allies); allies would count with enemies in the tiers.
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
- **Systems** (`SelectionSystems`): `Sync` (in `SyncSim`) removes objects that left the sim from every selection
  and group in the frame of their tick (`SimEvent::Removed`, deaths included, and anything else gone; a station
  the player still selects by its record stays), and those their player may no longer select (an enemy that walked
  into the fog; a placeholder, the design is silent), and
  starts every state over when the sim is replaced (`SimRes::generation`). `Input` (in `Actions`) applies map
  clicks, boxes, the control group keys (Recall and Center snaps the camera to the drawn centroid), Tab, and the
  debug player switch (F2, only while `SimConfig::debug`: the client acts as the next player, so "own" and the state
  in use change; a replay's playback switches without debug commands too). The HUD's portrait clicks run after it.
  Systems change a state only through `LocalControl::update`, which marks `ControlStates` changed only on a real
  change, and puts the ObjectInterfaceState back to DefaultState when the Selection or its ActiveGroup changes; a
  control group's recall puts it back even when the selection stays the same (`ControlState::recall_group`, design
  change 0078). While a target is awaited, map clicks and boxes select nothing (the drag box isn't drawn): the
  left-click is the target's.
- **Visuals** (`selection/markers.rs`): one marker entity per selected object: a ring around a unit, in the
  ground-decal layer under it, or a frame inside a footprint, which the object's visual would hide there, so it is
  drawn just in front of the object (`FRAME_Z`: half a su of ground y, under the objects south of it); coloured by
  relation, the ActiveGroup's solid and the others' dashed; meshes cached per shape, relation and activity. The drag
  box is four edges of one shared mesh in the overlay layer, shown while `Pointer::drag_box` is set.

**Commands UI** (`command_ui.rs`, `command_ui/`; `systems/controls/selection_and_hud.md > ObjectInterfaceState`
to `> CommandIndicators`, `systems/controls/commands.md > Unit Command`): how the local player orders their units,
their producing structures and their constructors, through one command flow (plan decision T10, slices M2-H1 and
M2-H2).
- **Rules** (pure and unit tested): `command_ui/interface.rs` and its children (`commands.rs`, `templates.rs`,
  `conditions.rs`, `panel.rs`) have the `PanelCommand`s (the units' and the producers' commands), the `InterfaceState`
  (DefaultState, or AwaitingTarget of a command, `queued` once a Shift-click has queued it: design change 0070) in
  each `ControlState`, the ActiveGroup's `InterfaceTemplate` (the Peacekeeper's and the Guard's BasicCombatUnit grid;
  the Barracks': Train Peacekeeper in Q, Cancel Production in X, Set Rally Point in C),
  the `Panel` a selection gets (common, group, pending, disabled and unavailable buttons; Back in Z while a target is
  awaited) and who receives a command (`recipients`: a common one the whole selection, a group one the whole
  ActiveGroup, by hotkey or button; never some of them, no smart-casting, 0071, apart from the units that can't carry
  it out, which the resolution leaves out). A type's grid is its template's plus its own commands (`grid`: the Riot
  Shield in R for a unit that carries one, a placeholder slot). `Modes` holds the selected units' modes as the sim has
  them: the Riot Shield button is on, and lowers the shields, while all its recipients' are up or going up, else it
  raises them; Attack is disabled while none of its recipients can attack. `Conditions` holds the `Modes` and what
  the command gate refuses: a production command is disabled exactly when `Sim::validate` refuses the
  `Command::Production` pressing it would send (the gate's own predicates, no copy of them: crystals, Unit Control, a
  full queue, nothing to cancel; 0081, 0082). `command_ui/resolution.rs` has what a click
  resolves to and the `PointerDisplay` that previews it. A `MapTarget` is what is under the cursor (`CursorTarget`),
  with the ground point there and, for an object, whether it is destructible and its domain. A right-click resolves
  per unit (design changes 0062, 0063), whichever group is active: `UnitCapabilities` (the unit's template, its
  attack's TargetDomain, whether its mode lets it attack) and `unit_right_click` give the template's order
  (`right_click`) when the unit can carry it out; a unit that can't attack the target moves to it instead (`move_to`:
  to the point, or Follow an object); a unit whose Riot Shield keeps it from an Attack gets `order_without_attack`,
  the right-click's shield rule: `move_to` too, Follow an enemy unit, Move to (and stop by) an enemy structure (design
  change 0074); objects without a template do nothing. A producer's right-click sets its rally point
  (`object_right_click` gives a `Resolved::Rally` beside the units' `Resolved::Order`s, so a mixed selection does
  both, 0062; `rally_target`: the object under the cursor, else the ground point; the producer itself clears it,
  0086); Set Rally Point's targeting click resolves the same way (`target_resolution`), and the pointer shows Move
  for a rally point. `group_orders` makes one order per distinct order, so units
  sent to one location go as one group (the magic box). A left-click while a target is awaited gives `target_order`:
  Attack takes any destructible object, own and neutral ones too (force-attack, 0059, 0064), never an indestructible
  one; a right-click never attacks them. `unit_target_order` is the targeting click's shield rule, per unit: an
  explicit Attack on an object leaves out a unit whose shield is up, which keeps its orders (0075; all of them
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
  point isn't queued). While a target is awaited (`> AwaitingTarget`, design changes 0065 to
  0070 and 0073): the other commands do nothing (`press`; the panel shows them disabled); a click counts at the press
  point, as the frame of the press drew the objects (`TargetPress`, recorded when the Select button goes down on the
  map, used when it comes up, `Pointer::released`); the press is a click while the pointer stayed within
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
- **Construction** (slice M2-H2; `factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState
  [DeploymentCenter]`, design change 0104; `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState
  [ExtractionFacility]`): the constructors' commands are `PanelCommand`s too: `BuildMenu(menu)` (the Deployment
  Center's Q Basic and W Advanced, unavailable while none of its buildings is in the game), `Construct(kind)`,
  `CancelConstruction` and `CancelReady` (X), and `PlaceReady(kind)`, "Select completed building", which takes its
  building's slot once it is ready. `InterfaceState` adds `BuildMenu(menu)` and `AwaitingPlacement(Placing
  { constructor, kind, orientation })`. A constructor's panel follows its state (`state_grid`, from
  `Conditions::constructor`, the ActiveGroup's one instance's `Constructor`), and its commands are that instance's
  own (`PanelCommand::instance_only`): they go to the active constructor only, never to the other selected ones. A
  construction command is one `Command::Construction` (`OrderOutbox::construct`, validated like the others), and
  Construct and the Cancels return to DefaultState. Back, Esc and a right-click go back one state
  (`InterfaceState::back`): from AwaitingPlacement to the Deployment Center's build menu or the Extraction Facility's
  DefaultState, from a build menu to DefaultState. While placing, the placement keys turn the ghost (`turn_ghost`,
  see Input), and a left-click on the map places it if `Sim::placement_check` allows it now: one Place at the
  footprint centred under the cursor (`Placing::origin_under`), as turned, and back to DefaultState; a Shift-click
  the same (0071); elsewhere nothing is sent, placing goes on, and an `InvalidPlacement` message says why. A Place
  the sim refuses as it applies it (`SimEvent::PlacementRefused`) leaves the building ready and the player in
  DefaultState. A build menu leaves the map's clicks to the selection; AwaitingPlacement takes them
  (`InterfaceState::takes_map_clicks`). `Validate` ends a build menu the ActiveGroup doesn't have, a placement whose
  constructor isn't active or holds the building no more, and both when the selection isn't commandable.
- **Visuals**: the panel's buttons (`hud/command_panel.rs`) are rebuilt when the panel, the bindings or the surface
  changes; each shows its command (`PanelSlot::label`: a toggle says what pressing it does) and its hotkey from the
  bindings, and a toggle that is on has a thick green border. The indicators (`command_ui/indicators.rs`) are
  one mesh in the overlays layer, rebuilt every frame while anything shows: markers at the targets of the selected
  own units' current orders (and queued ones while Shift is held), a flash where an order was just given
  (`FLASH_TICKS`), and while Shift is held each unit's route through its waypoints as dashed lines; colours follow
  `> Color language`; a target object the fog hides gets no marker and ends no leg (`Sight::sees`), but a Move to
  a remembered one is marked at its record. The selected own producers' rally points (`indicators/rally.rs`, design
  changes 0087, 0088) show a marker, green on the ground and around an own or neutral object, red around an enemy
  (the sim turns an enemy rally point it loses sight of into a ground one, so the marker turns green there), and a
  solid line from the footprint's edge to it; nothing while the producer isn't selected or has none. The pointer
  (`command_ui/cursor.rs`) shows the `PointerLook`: images drawn in code and cached per look, set as the window's
  `CursorIcon`, and drawn as a UI node at the injected cursor in QA runs, which render no system cursor; Mixed is
  striped in the Attack red and the Move green (`MIXED_COLORS`, `MIXED_STRIPE_PX`). The invalid-target feedback
  (`command_ui/feedback.rs`) is the message `INVALID_TARGET_TEXT` above the HUD Bar for `FEEDBACK_TICKS` sim ticks
  after the latest `InvalidTarget` (`TargetFeedback`), never in the pointer's way (`Pickable::IGNORE`). The same
  band gives the insufficient-resources feedback (0083): `NO_CRYSTALS_TEXT`, or for the army cap the player's
  faction's (`army_cap_text`: `UNIT_CONTROL_TEXT`, `BEACON_CAPACITY_TEXT`), after the sim's `ProductionRefused` for
  a local player's producer that a Train didn't reach for want of crystals or army cap (a full queue gives none),
  and a placement's (`placement_text`, the check's first problem): an `InvalidPlacement`
  or the sim's `PlacementRefused`.
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
  z band just in front of the fog layer (`overlay::AREA_Z`), rebuilt only when the area differs from the one it last
  drew (`BuildArea: PartialEq`) or the map meshes were rebuilt (`MapView::builds`).
- The design leaves some things open (labels, colours, the flash, the cursor's shapes and the Mixed look, the
  invalid-target and insufficient-resources feedbacks' form and duration, the targeting click allowance's value, when
  Shift counts for a targeting click, Shift with a rally point; the ghost's and the overlay's looks, the placement
  messages, the orientation a placement starts with, the hidden slots of buildings not yet in the game); each
  placeholder is marked where it is made.

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
- **Golden hashes** (`crates/sim/tests/golden.rs`): seven games, a 3v3 duel, a 40-unit AttackMove battle, a
  12-unit group move through a choke, a patrol that meets an enemy, a chase into the fog, three Peacekeepers that
  raise their Riot Shields (`Command::Ability`; one turns back half-way), walk up to a Guard under fire and lower
  them to kill it, and `gdo_opening` (slice M2-J1: `dev_skirmish` from its real start, to 2:45; player 1 plays the
  fastest clean Barracks and rallies ten Peacekeepers to player 2's crystal field, where they destroy four plates,
  while player 2 plates its field and builds Generators), are blessed as replays with a checkpoint every 32 ticks and
  at the end, in `crates/sim/tests/golden/<name>.json`. The test plays each game from its definition, checks that the
  definition still is the blessed one, and compares the checkpoints; a mismatch fails with the first diverging tick
  and the last one that matched. Each game also asserts what it is about, at its end (`expect`: one side of the duel
  is gone, the group passed the choke, …) and along the way (`during`: the shielded Peacekeepers held their fire), so
  a re-bless can't hide a broken game. `gdo_opening`'s players play by script (a `Script`, here two
  `testing::build_order`s), which gives their commands before each step from the game as it is and checks what it
  saw at the end: both players' books balance every step, the patch of a destroyed plate keeps what it held until a
  plate covers it again, player 2's income falls (210 crystals in the 9 s before its first plate fell, 130 in the
  last 9) and it is not defeated. Its commands are recorded like the others', so the file plays back without the
  script; as they follow the game, they are compared after the checkpoints, not as its definition. Any change to the
  sim's rules or state changes the hashes: when the change is intended, re-bless and commit the files with it:

  ```sh
  SC_BLESS=1 cargo test -p sc_sim golden
  ```

  The suite runs in well under a second in debug. The golden files are replays, so the client plays them:
  `cargo run -- --replay crates/sim/tests/golden/duel_3v3.json` (the camera starts over the map's middle).

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
  have no restrictions. A new Skirmish Default map lists its empty GDO and Colonists layouts as problems
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
    player may own; for a faction without Starting Conditions nothing ("not implemented yet", plan decision T11).
    Structures snap to tiles and units, such as the Colonists' Drones, go at the point clicked (`objects::snap_spawn`;
    `tests/editor/colonists.rs`).
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

## Adding a faction

What a faction needs before players can start with it, in the order M3 (the Colonists) built it. Started by slice
M3-C; the M3 slices after it add their steps (plan `artifacts/director/m3-plan.md`).

1. **Defs** (`defs/<faction>.rs`, see Object model; slice M3-A): its kinds at the end of `ObjectKind` (the state
   hash numbers kinds in declaration order), one `static ObjectDef` each with `faction: Some(..)`, each unit's
   army-cap cost in its `UnitDef`, and every exhaustive match over `ObjectKind` (sim and client) decided. The
   client draws each kind (`object_render.rs`, `object_render/units.rs`), and the editor's unit palette lists the
   faction's units by itself.
2. **Banks**: a resource the faction stores is a `Player` field (`u32`, hashed in `hash.rs`, `#[serde(default)]`),
   in `Banks` and `DebugCommand::SetResources` (`serde(default)` there too, so older command files load). Re-bless
   the goldens (`> Replays and golden hashes`).
3. **Starting Conditions** (`scenario/starting.rs`): a `static StartingConditions` table and its arm in
   `StartingConditions::of`. A game then starts the player's banks from it (`Player::starting`), and the camera on
   its primary structure (`Scenario::primary_structure`).
4. **Layouts**: from then on every Skirmish Default map needs the faction's layout at every start (plan decisions
   T11, T24): `scenarios::dev_skirmish_map` (slot 1's; `MapDef::regenerate` makes the images), every Skirmish
   Default file in `maps/` (place them with the editor's Layout tool and save;
   `the_maps_in_the_repository_are_playable` says which), and the tests that build a Skirmish Default map and expect
   it playable (the editor's use `tests/editor/colonists.rs::with_colonists`). Check each layout with
   `MapDef::validate` (its structures by `placement::check_layout`, its units by the unit-start rule), and keep the
   starting units clear of the structures' sides and exits.
5. **Army cap** (`economy/army.rs`): the faction's `ArmyResource` (`ArmyResource::of`, `name`, `cost_of`, and what
   is available in `cap`), or none. The gate's refusal stays the one for every faction (`Reject::UnitControlCap`,
   `Refusal::UnitControl`); the client's `army_cap_text` names the faction's resource.
6. **HUD figures** (`hud/resources.rs`): the faction's `ResourceFigure`s in its DisplayHud order (`ResourceFigure::of`)
   and each new figure's reading in `resource_bar`, in ASCII.
7. **Templates** (`command_ui/interface/templates.rs`): `InterfaceTemplate::of(kind)` for each kind with a command
   panel, `None` until its slice builds one.
8. **Tests and QA**: `ScenarioBuilder::faction` sets a test player's faction (plan decision T27), and
   `cargo run -- --factions <f1,f2>` starts a game with it, which its replay records.

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
