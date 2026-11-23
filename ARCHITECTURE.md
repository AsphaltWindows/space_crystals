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
- **Map resources need flat ground** (`core/map.md > Elevation > Map resources only on flat terrain`): every footprint
  tile of a kind with `ObjectDef::needs_flat_ground` must be flat. Validation flags it (`SpawnProblem::NotFlat`) and
  `Command::Debug(Spawn)` refuses it. Structures don't check flatness yet.
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
  `orders` (`Orders`, `orders.rs`: the current order and a queue of at most `MAX_QUEUED_ORDERS`), `mover`
  (`Mover`, `mover.rs`: goal, status, path and the replanning state of `docs/movement.md > 3`) and, for units that
  can attack, `attack` (`AttackState`, `combat.rs`: see *Combat*). Production and the rest arrive as more optional
  fields, each with its own module. Never add them as separate stores keyed by
  id. A system walks the objects in id order and skips those without its capability
  (`let Some(..) = &mut o.resource else { continue };`).
- **Per-type constants** sit in one `static ObjectDef` per type (`defs.rs`, `core/entities.md > Object Type`). Reach
  them through `ObjectKind::def()` or `Object::def()`. Every value cites its design section. Faction types go in
  per-faction modules (`defs/gdo.rs`, …). A unit type also has `ObjectDef::unit: Some(UnitDef)` (`defs/units.rs`,
  `systems/units.md`): its `UnitBase`, whose `static UnitBaseDef` holds the base's domain, movement model and flags
  (turret, crushable, …), the attributes of that movement model (`MovementAttrs`), and its armour, turret, attack and
  costs. Its box is its silhouette (`ObjectDef::half_extents`).

**Adding an object type**
1. Add an `ObjectKind` variant whose doc comment cites the design section, and add it to `ObjectKind::ALL`.
2. Add its `static ObjectDef`, with a citation for every value, and its arm in `ObjectKind::def()`. Missing numbers
   become named consts tagged with the question id.
3. Set up its capability state in `Sim::insert_object` (units get idle `orders` and `mover`), and give it a
   placeholder shape (`object_render::object_mesh`, whose `match` is exhaustive).
4. Test it. Spawn it in a small inline scenario, then check its initial state and its behaviour.

**Adding a capability**
1. Give it its own module with its state type (deriving `Clone, Debug, PartialEq, Serialize, Deserialize`) and logic.
2. Add `pub <name>: Option<State>` to `Object`, with a doc comment citing the design.
3. Put its per-type parameters on `ObjectDef`, with citations, and initialize the state in `Sim::insert_object`.
4. Insert its update phase into `Sim::step` at a deliberate position (see *Tick pipeline*).
5. Add the `Command`s that drive it, with validation, and the `SimEvent`s the client needs.
6. Write unit tests in the module and a scenario test in `crates/sim/tests/`.

## Tick pipeline

`Sim::step(&TickInputs)` advances exactly one tick. It runs fixed phases in this order (`sim.rs`):

1. **Clear events**, so that `Sim::events()` holds only this tick's events.
2. **Advance the tick** (`tick += 1`).
3. **Snapshot** `pos`/`facing` into `prev_pos`/`prev_facing` for every object (the client interpolates from these).
4. **Commands**: apply each `PlayerCommand` in `TickInputs` order (`apply_command`).
5. **Systems**, each walking the objects in id order, one function per module (`docs/movement.md > 4`):
   - 5a `orders::update`: a unit's finished order completes and its next one starts (a move sets `Mover::go_to`,
     Stop and Hold stop it), until one is in progress, so legs follow each other without an idle tick.
   - 5b `paths::sync`: if the blocking footprints changed, rebuild the navigation grids and bump `nav_version`.
   - 5c `paths::serve`: first each unit's staggered path checks, in the ticks where `(tick + id) % 4 == 0` (after
     the ground changed, line of sight along its remaining path, and a new plan only if that is cut; the farthest
     waypoint in sight becomes its steering target). Then every `PathState::Wanted` becomes a `nav::PathRequest`,
     served by `nav::serve` within `PATH_BUDGET` (group fields for move groups). A unit that gets no path fails its
     move (`SimEvent::MoveFailed`).
   - 5d `movement::step`: steering and hard collision, the only code that writes a unit's `pos` besides a teleport
     (see Movement and collision). Units move one after another in id order and never overlap.
   - 5e `combat::step`: attack phases, the tick's list of hits applied together, deaths (see *Combat*). The dead
     leave through `Sim::remove_objects`, then `paths::sync` runs again in case a footprint went with them.
   - 5f `vision::update` (slice V): empty so far.
   - `update_supply_stations` (`core/map.md > SupplyDeliveryStation`).

- **Spawn tick**: an object that a command creates during tick `t` is first updated by the systems in tick `t + 1`.
  `step` notes `first_new = ObjectId(next_id)` before the commands, and each system walks only
  `objects.range_mut(..first_new)` (ids only grow). So a debug-spawned station with an interval of 960 delivers at
  `t + 960`, just as a scenario station, spawned before tick 1, delivers at tick 960. An order to a unit spawned
  earlier acts in the tick it arrives: a straight move of `d` su ordered in tick `t` arrives in tick
  `t − 1 + ⌈d / speed⌉`.
- **Grids match the objects between ticks**: `step` checks it in debug builds. A phase after 5b that adds or removes
  a blocking footprint (a death in combat, a new structure) must call `paths::sync` again.
- Insert a new phase at a deliberate position, with a comment in `step` saying why there, and update this list.
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
  holds a unit that may not: it stands, keeping its goal, path and stuck clock (`movement::wait`). Behaviours
  (slice E) should start a chase only when `may_move` holds. A unit that moves in 5d and starts aiming in 5e may
  turn twice in a tick; harmless while every M1 unit turns 180° per tick.
- **Range** (`combat/targeting.rs`): from the attacker's centre to the nearest point of the target's silhouette or
  footprint (`RANGE_MEASURE`, question Q0009's default); within Range and beyond MinRange, both inclusive; the
  TargetDomain must allow the target's domain. `can_attack_now(map, attacker, target)` is the check for behaviours.
- **ElevationModifier** (`elevation.rs`, shared with vision): `Elevation::of` an object is its elevation level (the
  height rounded down), or `Exempt` for air units; `modifier(source, target)` is +1/−1/0 grid unit, binary.
  `attack_range` and `sight_range` apply it; melee attacks don't take it.
- **Hits** (`combat/damage.rs`): every attacker acts against one snapshot of the targets, and an attack's effect
  lands at the end of Firing as a `Hit` on the tick's list. The list is applied once all have acted, so every hit
  counts, including those on a unit another hit has killed and those of a unit that dies in the same tick.
  `hit_damage` follows `> SingleTargetDamage` and `> DamageModifierOrder` (pre-armor multipliers, PointArmor,
  directional multiplier, MinimumDamage); HP is fractional, nothing is rounded. An object at 0 HP or less dies.
- **Removal path**: `Sim::remove_objects` (`sim/removal.rs`) is the one way objects leave the game, for deaths and
  debug removals alike. It emits `Removed` and clears every reference to the removed: a current order naming one
  completes, queued ones are dropped, a move towards one ends `TargetLost`, an attack on one stops (an aim is
  cancelled, a shot being fired hits nothing). `testing::dangling_references` checks that none are left.
- **Not yet**: chasing, AttackMove, auto-targeting and leashes (slice E: the Attack order waits while its target is
  out of range, `TODO(E): chase`), vision checks on targets (slice V), turrets, projectiles, AoE, Riot Shield.

## Navigation

The design is `docs/movement.md > 5` (binding for M1 movement), and the code is `crates/sim/src/nav.rs` and `nav/`.
The module is pure: it takes the `Map` and tile rectangles and knows nothing of `Object`, `Sim` or the tick.

- **`Nav`** holds one `NavGrid` per `(NavClass, size class)`: a blocked bit and a component label per tile. It is
  derived data, never hashed or serialized. `Nav::rebuild(map, blockers)` rebuilds it in full when the map or a
  blocking footprint changes. The caller chooses the blocking footprints (`NavRules::footprint_blocks`). Units are
  never in a grid.
- **`Search`** is the scratch for `astar` and the Dijkstra `field`. Keep one and reuse it; nothing carries over
  between searches. Searches are integer-only. An unreachable goal never searches: retarget it first
  (`NavGrid::nearest_in_component`), as `find_path` does.
- **`los`, `box_clear`, `smooth`**: line of sight and waypoints for a unit's box (half extents). Touching is free.
- **`serve(nav, search, requests, PATH_BUDGET)`** serves a tick's `PathRequest`s in `(since, id)` order within the
  expansion budget, with one shared field per order group of `GROUP_FIELD_MIN` or more, anchored on the click's tile.
  A member whose goal the anchor can't see plans alone: through the anchor it would take a detour. A `Path` is unit
  state (serde, `PartialEq`, `StateHash`); `NavCounters` are output only.
- **Open questions** (Q0011) are the switches `RESOURCES_BLOCK`, `RUGGED_IMPASSABLE` and `ROUTE_ON_TRUE_MAP`.
- **In the sim** (`paths.rs`): `NavState` holds the `Nav` of every unit type's `(class, size class)` and the
  footprints it was built from (structures and map resources block, Q0011). It is derived, like the `Nav`: not hashed,
  compared or serialized, built when a `Sim` is created or loaded and rebuilt in phase 5b. `Sim::nav()` (the grids),
  `nav_version()` and `nav_counters()` expose it to tests and overlays.
- **Group destinations** (`orders/destinations.rs`, `docs/movement.md > 6.4`, the design's magic box): a location
  order (Move to a point, AttackMove, Patrol) to several units looks at the bounding box of their positions. A click
  inside it sends every unit to the click; outside, each unit goes to the click plus its offset from the box's
  centre. The order keeps the unit's own destination (`Order::slot`; `None` is the click). Until question Q0008 is
  answered, a destination is clamped inside the map, and one the unit can't stand on or reach moves to the nearest
  point it can (`nav::reachable_point`, which never searches).
- **Avoid tiles** (`Mover::avoid_tiles`): a repath of a stuck unit treats them as blocked, and its smoothed path and
  its steering stay out of them (`nav::smooth_avoiding`, `nav::segment_avoids`). If they cut every route, the unit
  drops them and plans again.
- **Unit boxes** never overlap blocked tiles or another unit of their layer at spawn: the gate checks debug spawns and
  teleports against the sim's grids; scenario validation and the editor check pre-placed and layout units with
  `paths::misplaced` (terrain-only grids of the classes present, plus the footprints).
- `f32::mul_add` is a disallowed method (`docs/movement.md > 7`): write `a * b + c`.
- Timings: `cargo test -p sc_sim --release --test nav_perf -- --ignored --nocapture`. That test's `timed` helper is
  the one place in `crates/sim` that reads the clock, behind an `#[expect]` with its reason.

## Movement and collision

The design is `docs/movement.md > 6` (binding; its **(D)** marks are where slice D changed it, **(M)** where the
magic box did), and the code is `crates/sim/src/movement.rs` and `movement/`, with the unit's state in `mover.rs`.

- **One pipeline, sequential commit** (`movement::step`, phase 5d): the step builds one spatial grid per collision
  layer (ground, underground; air units collide with nothing) from every unit, then moves the units in id order and
  updates the grid at each commit. A unit sees earlier units where they now are and later ones where they were.
  The grids are scratch, rebuilt every tick and never hashed.
- **No overlap by construction**: a position is committed only if `World::is_free` holds, the same strict predicates
  as invariants I1 and I2 (touching is free). Contact positions are assigned from the blocker's edge
  (`space::resting`), never accumulated. Never write `pos` anywhere else, and never loosen the predicate with an
  epsilon.
- **One unit's tick** (`step_unit`, `collide.rs`): turn towards the steering waypoint (at most TurnRate), then move
  one step: free; **slide** along tiles (per axis); **follow** a unit moving the same way; **go around** a standing
  or oncoming one (a glance along its face, a 45° step, a step along the face; to the side the unit leans to, which
  it keeps, else to its Right first, so head-on units pass like traffic; a crossing unit is passed behind); else
  close up or stay. Within one step of the
  goal the unit lands exactly on it. Units without a goal never move: nothing pushes (question Q0007).
- **Collision exceptions** go in `space::blocks(mover, other)` and nowhere else (crushing will).
- **Held by its attack**: a unit whose attack sequence holds its base (`combat::may_move` or `may_turn` false: see
  *Combat*) stands for the tick and keeps its goal, path and stuck clock (`wait`), like a unit waiting for its path.
- **Stuck and recovery** (`stuck.rs`, numbers from question Q0008 in `mover.rs`): progress is the remaining path
  length; waiting behind a unit that is getting on counts. After `STUCK_TICKS` without it: arrived if the goal is
  within `CROWDED_ARRIVAL_GU` behind stationary units, or behind members of its own order group that have arrived;
  else a repath (`MAX_REPATHS` per leg, `REPATH_BACKOFF`), around the tile of a blocker that stands; else
  `MoveStatus::Failed` and `SimEvent::MoveFailed`. Two units head-on in a passage too narrow to pass both fail
  (question Q0007).
- **Hooks for later slices**: chasing and queueing behind attackers are `stuck::queues_behind` and the
  `MoveGoal::Object` arm of `paths::serve` (slice E); other movement models replace the "direction → facing and
  velocity" part of `step_unit`.
- **Counters**: `Sim::move_counters()` (sweeps and box tests of the latest tick) and `Sim::nav_counters()` are
  output, not state. Tests assert them; `cargo test -p sc_sim --release --test movement_perf -- --ignored
  --nocapture` prints timings.
- **Invariants** (`docs/movement.md > 8`, `sc_sim::testing::invariants`): I1 no two units of a layer overlap; I2 no
  ground box overlaps a blocked tile or leaves the map; I3 a unit moves at most its speed per tick; I4 every float
  is finite; I5 a unit without a goal doesn't move; I6 repaths per leg are capped and no path request waits longer
  than `MAX_REQUEST_AGE`; I7 an arrived unit never moves again without an order, and a unit without a goal is at
  rest. A `TestSim` checks them after every tick and panics with a picture of the map (`TestSim::ascii`: units as
  letters, with a legend). The check is on by default; `check_invariants(false)` turns it off for timing runs. A
  test that steps a bare `Sim` keeps its own `Invariants` and calls `check` after each step.

## Commands, events and replays

- All player intent is a `Command`. It is wrapped in a `PlayerCommand { player, command }` and collected in one
  tick's `TickInputs`. The client never mutates sim state. It reads `Sim` and submits commands.
- The sim validates every command (player, ownership, ids, costs, placement, and positions that are finite and on the
  map), and it **ignores invalid ones**. It must never panic on player input, so use checked or saturating arithmetic
  on numbers that come from input. Panics are only for programmer errors, such as an invalid scenario passed to
  `Sim::new` or an out-of-bounds `Map::set_tile`.
- **The command gate** (`sim/gate.rs`): `Sim::validate(&PlayerCommand) -> Result<(), Reject>` checks a command
  against the current state without changing it, and `step` applies a command only through the same check
  (`Sim::check`, which hands the application the objects and positions it resolved), so the two can't disagree. In
  order: the player is in the game (`Reject::UnknownPlayer`), a debug command needs debug commands on
  (`DebugDisabled`), then the command's own rules: every id it names exists (`UnknownObject`) and, for a player's
  orders to objects, belongs to that player (`NotOwned`, through `Sim::referenced(id, Some(player))`; debug commands
  and targets may name anyone's objects), positions are finite and on the map (`OffMap`), spawns fit
  (`Spawn(SpawnProblem)`). A rejected command changes nothing, neither the state nor the events, and never panics.
  There is no rejection event yet: add one as output outside the state hash when the UI needs it.
- **`Command::Order { units, kind, queue }`** (`systems/controls/commands.md > Unit Command`, `docs/movement.md > 3`):
  `OrderKind` is `Move(Target)` (a point or an object), `Attack(id)`, `AttackMove(point)`, `Patrol(point)`, `Hold`
  or `Stop`. The gate takes an order whole or not at all, but tolerates stale selections: at most `MAX_ORDER_UNITS`
  ids (`TooManyUnits`), repeated ids count once, ids of objects that are gone and objects that take no orders are
  dropped, and nothing left is `NoUnits`; any id of another player's object rejects it (`NotOwned`). Points must be
  finite and on the map, target objects must exist (`UnknownObject`) and an attack's target be destructible
  (`NotAttackable`). Without `queue` the order replaces the unit's queue and current order; with it, the order is
  appended (a full queue drops it). Each order command is one group (`Sim::next_group`); a location order gives
  each unit of a group its own destination (the magic box, see *Navigation*). `Move` to a point, `Stop` and `Hold` behave, and `Attack` without its approach (see *Combat*);
  the others complete at once (TODO slice E).
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
- `Command::Debug(DebugCommand::…)` covers `Spawn`, `Remove` and `SetPosition`, for development and QA. It bypasses
  costs and rules, only works while `SimConfig::debug` is on, and is never offered in a real match. A teleport sets
  `prev_pos = pos`, so nothing interpolates across the map.
- **Replay** = the `Scenario` (the map and the game setup, game options included) plus the `TickInputs` of every
  tick, empty ticks included:
  `sc_sim::Replay` (serde) with `start`/`record`/`step`/`run`. The client must never submit a command with a
  non-finite number. The sim would reject it anyway, but JSON can't store NaN or ±∞, so the replay file would fail to
  load. Validate at the input layer.
- **Events**: `SimEvent`s (`Spawned`, `Removed`, `SuppliesDelivered`, `MoveFailed`, `Attacked`, `Hit`, `Died`, …) are
  output only, for effects, sounds and UI. The sim never reads them. `Sim::events()` holds only the latest step's events, and `Sim::new`
  discards the events from the scenario's initial spawns.

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
    as `--map` and `--scenario` do; the editor's play-test panel builds its own.
  - `Sim::try_new` creates the map's objects first, in list order (so their ids ascend in it), leaving out those of
    slots no player takes, then each player's Starting Layout for its faction at its slot. A slot's objects belong to
    the player who takes it.
- `MapDef::validate()` and `Scenario::validate()` list every problem (`ScenarioProblem`) and never panic.
  `Sim::try_new` returns the problems as an error and `Sim::new` panics with them, so load files and editor maps with
  `try_new`. Units start on open ground and apart (`paths::misplaced`, in list order): a unit's box must not overlap
  blocked tiles, the map's edge or a blocking footprint (`SpawnProblem::Blocked`) or an earlier unit of its layer
  (`OverlapsUnit`), and a blocking footprint must not cover a ground unit (`CoversUnit`). `MapDef::validate` checks
  the pre-placed objects; `Scenario::validate` checks the Starting Layouts of the factions played, with the map's
  objects.
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
  Skirmish Default development map (Q0003) with 180° symmetry: starts in opposite corners, gentle hills of slopes
  only, and a central plateau ringed by cliffs with a ramp up through a gap in the west and in the east; and `empty`,
  a flat 32×24 Custom map without symmetry. `scenarios::empty(w, h)` builds an empty scenario of any size,
  `empty_map(w, h)` its map.
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
  `ScenarioBuilder::new(ascii)` makes a scenario with the two slots of `scenarios::empty` and its default setup
  (players 1 and 2, GDO, in slots 1 and 2) and seed 1. Add spawns with `.spawn(..)` (an owner takes the slot with its
  number), then call `.build()` or `.start()`. A `TestSim` runs with debug commands on
  (`TestSim::with_config` for other options). It schedules commands with `at(tick, player, cmd)` or
  `debug_at(tick, cmd)`, and each one applies in the step that reaches that tick; `spawn_unit_at(tick, kind, owner,
  pos)` and `order_at(tick, player, units, kind, queue)` are the shortcuts for units. It advances with `step`,
  `run(n)` and `run_until(tick)`, keeps every event with its tick, and records a `Replay` of the run. After every
  tick it checks the movement invariants (see Movement and collision); `pos(id)`, `mover(id)` and `status(id)` read
  a unit, and `ascii(focus)` draws the map.
- **Movement scenario tests** are `crates/sim/tests/movement_collision.rs` (passing, head-on, choke, group arrival,
  idle units, repaths, oscillation bounds, save and load in a crowd), `group_destinations.rs` (the magic box),
  `movement_soak.rs` (200 units, random orders, 2 000 ticks) and the ignored `movement_perf.rs`.
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
- **Units** (`object_render.rs`) are placeholder shapes until the art (Q0001): a shape per kind (Peacekeeper round,
  Guard square) of the silhouette's size, in the owner's colour (`player_color`, the editor's slot colours), with a
  dark rim and a light tick along the facing; the visual turns with the interpolated facing. One mesh per
  `(kind, owner)` is shared.
- **Path overlay** (`path_overlay.rs`, P or `--show-paths`): each unit's remaining path (`Mover::remaining_path`) as
  a thin line from where the unit is drawn, cut into short pieces lifted by the terrain's height, in the overlays
  layer, and a frame around each unit in the colour of its move's status (`status_color`: green moving, amber held
  up, red failed, blue arrived; placeholder colours). Its one mesh is rebuilt every frame while it is shown.
- **Combat effects** (`combat_effects.rs`, placeholders until Q0001): a tracer for each `SimEvent::Attacked`
  (`TRACER_TICKS`), HP bars over damaged units, a fading marker for each `SimEvent::Died` (`DEATH_MARKER_TICKS`).
  One alpha-blended mesh in the overlays layer, rebuilt every frame while anything shows; effects age by the sim's
  tick and the render alpha, so screenshots stay deterministic.
- Never make one entity per tile. Draw the terrain as one mesh or texture, or a few chunks. Fog of war will be one
  texture that is updated when visibility changes, never per-tile materials.
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
at the local player's slot's camera start (`Slot::camera_start`), or else at the primary starting structure
(`core/camera.md > Starting Position`). Until starting structures exist, the slot's start position stands in for
it, and the map centre for a player without a slot (`camera::start_center`, shown through `start_view`). Only the
editor zooms (`CameraZoom`).

**HUD**: built with bevy_ui (`core/camera.md > HUD Layout`). It has a top resource bar and, at the bottom, the HUD
Bar with five fixed sections. From left to right they are Minimap, Portrait, InfoPanel, SelectionPanel and
CommandPanel. All five have the same height and keep fixed horizontal slots, and a hidden panel's slot stays reserved
(`systems/controls/selection_and_hud.md > HUD Bar`). bevy_ui lays out each root node inside its camera's viewport,
so the HUD needs its own full-window UI camera (`IsDefaultUiCamera` / `UiTargetCamera`). The sections sit at the
rectangles `hud::layout` computes from the surface size alone (`section_rect`, `PortraitGrid`); the same geometry
finds the portrait under a click, because headless tests have no UI picking. The InfoPanel (`hud/info_panel.rs`) and
the SelectionPanel (`hud/selection_panel.rs`) show the selection (see Selection); a section with nothing to show
shows its name (`SectionLabel`).

**Input** (`input.rs`, `input/`): one shared layer turns the keyboard, the mouse, the wheel and the screen edge into
the input of the game and of the map editor, in `PreUpdate` (`ReadActions`, after Bevy's input, UI picking and the
focused text field's key handling), and gameplay and editor code read only what it produces: `InputAction`s, which
become UI state changes, `Command`s and `EditorAction`s. Never read raw keys, buttons, the wheel or the cursor
elsewhere.
- **Input contexts**: the mode picks whose bindings apply (`ClientMode::input_context`): the game's in a game and an
  editor play-test, the editor's in the editor. An action works in its own context only (`InputAction::context`), so
  one key can mean different things: V is a command slot in the game and the Select tool in the editor. The camera's
  pans and grab, the grid (G), `Cancel` (Esc) and the editor's play-test switch (F5) are shared by both. A button
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
  `InputContext::ctrl_blocks`). The action stays held until its button goes up. `Bindings::default()` holds the
  design's defaults (camera keys, G for the grid, P for unit paths, Select and ContextCommand on the left and right
  buttons, Esc to cancel, Tab and Shift + Tab, the control group keys, the 4 × 3 command grid QWER/ASDF/ZXCV with
  the standard slots Z, X and C, Shift to queue) and the editor's (its F1 help lists them). A rebinding UI edits it
  with `bind` and `unbind`. It isn't serializable yet: loading it from a file needs Bevy's `serialize` feature for
  `KeyCode` and `MouseButton`. The selection adds `CenterOnObject` (Alt) to the modifier actions, for clicks on
  portraits, and F2 for the debug player switch (a placeholder: the design has no debug keys).
- **Output**: `ActionState` (a `Resource`) holds this frame's actions (held, just pressed, just released), the wheel's
  turn, the modifiers, the context, the cursor in logical pixels, and the `Pointer`: the view point and the ground
  point (`coords::ground_under`) under the cursor, whether it is over the UI (a UI node by Bevy's picking, or outside
  the world view: the one over-UI test of the game and the editor), the press in progress and the drag box. A press of
  `Select` or the editor's `UseTool` that goes down on the map is followed until its release (`Pointer::press`, a
  `MapPress`: where it went down, whether it moved further than `DRAG_THRESHOLD_PX`, and the ground point under the
  cursor now, which keeps following it over UI nodes on the world view, as the editor's drags do). `MapClick`
  messages report clicks on the map: `Select` when released, unless it became a drag, and `ContextCommand` and the
  editor's `Eyedropper` when pressed, with the modifiers and a double-click flag. A `Select` drag ends as a
  `SelectionBox` message in view space, kept inside the world view. Nothing starts over the UI. What the modifiers of
  a click mean comes from the table (`Bindings::holds(InputAction::Queue, click.modifiers)`).
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
state. `ControlStates` holds every player's `ControlState`, a `Selection` and 10 control groups; the input changes and
the HUD shows the local player's (`LocalPlayer`).
- **Model** (`selection/model.rs`): a selection is a set of object ids with their kinds. Its groups are derived, one
  per type, in `ObjectKind` order (an ungroupable object, a patch or a station, is a group alone), each by id; the
  ActiveGroup is a group key, which Tab and Shift + Tab cycle with wrap-around and which falls back to the first group
  when its group empties. At most `MAX_SELECTED` objects (a placeholder: the order limit).
- **Picking** (`selection/pick.rs`): `Selectables` is what the local player may select this frame (`selectable_by`,
  the hook for vision's Explored rule), each object's box as it is drawn: interpolated, lifted by the height it
  stands at, in view space. A click picks the frontmost box under it (grown by `PICK_SLOP_SU`); a box (the input
  layer's `SelectionBox`) applies the priority tiers. `Relation::of` says own, enemy or neutral (no teams yet, so no
  allies); allies would count with enemies in the tiers.
- **Rules** (`selection/rules.rs`, pure and unit tested): map clicks, boxes, portrait clicks, control group keys and
  Tab on a `ControlState`, under the Constraints (another player's object or a resource is only ever selected alone).
  What a click's modifiers mean comes from the bindings (`ClickRoles`). Every rule the design leaves open is marked
  as a placeholder where it is made.
- **Systems** (`SelectionSystems`): `Sync` (in `SyncSim`) removes objects that left the sim from every selection
  and group in the frame of their tick (`SimEvent::Removed`, deaths included, and anything else gone), and starts
  every state over when the sim is replaced (`SimRes::generation`). `Input` (in `Actions`) applies map clicks, boxes,
  the control group keys (Recall and Center snaps the camera to the drawn centroid), Tab, and the debug player switch
  (F2, only while `SimConfig::debug`: the client acts as the next player, so "own" and the state in use change). The
  HUD's portrait clicks run after it. Systems change a state only through `LocalControl::update`, which marks
  `ControlStates` changed only on a real change.
- **Visuals** (`selection/markers.rs`): one marker entity per selected object in the ground-decal layer, under the
  object (a ring around a unit, a frame inside a footprint), coloured by relation, the ActiveGroup's solid and the
  others' dashed; meshes cached per shape, relation and activity. The drag box is four edges of one shared mesh in
  the overlay layer, shown while `Pointer::drag_box` is set.

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
`Sim` gives the same `state_hash` at every tick.

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
| `--editor [file.json]` | Opens the map editor (see Map editor) on that file, or on a new map. |
| `--editor-script <file.json>` | Applies a JSON list of `EditorAction`s in the editor's first frame. |
| `--player <n>` | Shows player `n`'s view: camera start, top bar. Default 1. In the editor: the slot play-tests start as. |
| `--window-size <W>x<H>` | Window or offscreen image size in physical pixels. Default `1280x720`. |
| `--fast` | Runs as many ticks per frame as fit in 50 ms instead of 16 per second. Screenshots are unchanged. |
| `--no-debug` | Runs the game with debug commands off (the setup's `SimConfig`), as a real match will; they are on by default. |
| `--screenshot <tick>:<path>` | Saves a PNG of exactly that tick. Repeatable, in any order; creates parent dirs. |
| `--exit-at-tick <n>` | Exits at tick `n`, after its screenshots. Windowless runs default to the last screenshot's. |
| `--dump-state <path>` | Writes the state as JSON when the run ends (fields below). |
| `--commands <file.json>` | Applies player commands at given ticks: a JSON list of `{"tick", "player", "command"}` (`qa::TimedCommand`; `command` is a `sc_sim::Command` in serde form). Each applies in the step that reaches its tick (1 or later), a tick's commands in file order. Not with `--editor`. |
| `--input-script <file.json>` | Drives the virtual keyboard and mouse (`InjectedInput`) at given ticks: a JSON list of `{"tick", "cursor", "press", "tap", "release"}` (`qa::InputStep`; buttons by name: `MouseLeft`, `ControlLeft`, `Digit1`, `KeyQ`, `F2`, …). The sim waits at each step's tick until it is applied, so a button held across steps drags and a chord spans steps (Ctrl + 1: `press` Ctrl and `tap` 1, release Ctrl in a later step). Not with `--editor`. |
| `--show-paths` | Starts with the unit path overlay on (P toggles it). |
| `--windowed` | Shows a QA run in a window on the user's display instead of rendering offscreen. |
| `--headless` | No window and no renderer: sim, client state and `--dump-state` only. Needs `--exit-at-tick`. |
| Later: replays | Replay recorded sessions. |

Taking a screenshot (no window appears; about one second plus `tick / 16` seconds, or add `--fast`):

```sh
cargo run -- --scenario dev_skirmish --player 2 --screenshot 1:/path/to/p2.png --dump-state /path/to/p2.json
```

- Any of `--screenshot`, `--exit-at-tick`, `--dump-state`, `--commands`, `--input-script` makes a *QA run*. It
  ignores keyboard and mouse, renders at scale factor 1, and renders offscreen: Bevy renders into an image on this
  machine's GPU (Vulkan, Intel Arc), with no window and no Xvfb. The sim waits at each screenshot tick until the PNG
  is saved, so the PNG shows that tick's exact state with no interpolation. Repeated runs give byte-identical PNGs.
- Exit codes: 0 when everything was saved, 1 when a QA step failed (the log says which), 2 for invalid flags.
- The `--dump-state` JSON holds `tick`, `scenario` (the map's name), `players`, `objects` (every field of
  `sc_sim::Object`), the last tick's `events`, `client` (`local_player`, `camera_center` as a view point in su,
  `grid_visible`, `recorded_ticks`, the local player's `selection`) and `map` (the terrain).
- `--windowed` runs turn vsync off: with the display asleep, X11 paces a vsynced window at one frame per second.
- Verify visual work yourself. Capture screenshots and look at the PNGs with the Read tool before you report.
- To look at another part of a map, give the slot a different camera start in the editor (or the file's
  `camera_start`) and pass the file with `--map`.
- To put units on a map in a QA run, spawn them with debug commands in a `--commands` file at tick 1 and order them
  from tick 2. Spawned objects take the ids after the starting objects, in file order (`--dump-state` lists them):

  ```json
  [{"tick": 1, "player": 1, "command": {"Debug": {"Spawn": {"kind": "Peacekeeper", "owner": 1, "at": {"Point": [1216.0, 1376.0]}}}}},
   {"tick": 2, "player": 1, "command": {"Order": {"units": [18], "kind": {"Move": {"Point": [2016.0, 1376.0]}}, "queue": false}}}]
  ```

## Map editor

`cargo run -- --editor [file.json]` opens the map editor (`crates/game/src/editor.rs` and `editor/`). It edits a map
(`MapDef`) and saves it as a map file (see Sim scenarios and tests). F1 in the editor lists the mouse and keys.

- **Starting**: with a file that exists, the editor opens it. With a file that doesn't exist yet, it starts a new map
  that Ctrl+S saves there. Without a file, it starts a new map with the creation dialog's defaults (Skirmish Default,
  64×48, 180° rotation, 2 slots), and `--editor --scenario <name>` opens a built-in scenario's map. A file that
  doesn't parse exits with code 2. A file that parses but fails validation still opens, with its problems listed, so
  that it can be fixed; `--player` is not checked against it (play-tests fall back to the first slot).
- **Files**: maps live in `maps/` at the repository root (`editor::files::default_maps_dir`), and new maps are saved
  as `maps/<name>.json`. Saves are atomic: `files::save` writes a hidden temporary file next to the target, flushes
  it and renames it over the target. `maps/dev_skirmish.json` is the built-in `dev_skirmish` saved by the editor. A
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
  have no restrictions.
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
  or, for resources, on a sloped tile (the ghost turns red). New objects get `SpawnSettings::defaults`; selecting one
  shows its values as text fields (the interval in seconds), which commit `SetSpawnSettings`, one undo step each.
- **View**: G toggles the grid, K the slope tint (flat green, sloped amber, too steep red, in the terrain's z band).
  The middle-mouse grab pans as in the game. The status line shows the tile under the cursor with its preset, the
  interpolated height there, its level, whether the tile is flat, sloped or too steep, and whether it is locked.
- **Selection** is a spawn index or a slot's marker. An undo, a redo or a resize may shift the indexes, so they clear
  the selection; so does switching documents.
- **UI work follows changes** (review finding): each list (`ui/lists.rs`) is rebuilt only when what it shows
  changed, keyed so that a field being typed into survives marker moves and preset edits; the buttons are restyled
  when the editor's state changed or the pointer moved over them; the texts and fields refresh only when their values
  changed. Text fields commit in `EditorSystems::Input`, before the frame's actions, so a click elsewhere (Save, a
  tool, another field) applies the typed text first; Enter commits, Esc gives up the text.
- **Tests** (`crates/game/tests/editor/`: `session`, `edits`, `creation`, `symmetry`, `review`, `input`) build a
  headless app with `AppConfig::editor`, push `EditorAction`s into `EditorActions`, call `app.update()`, and assert on
  `EditorDoc`, the sim, the view and the UI state. Positions are ground points, such as `TilePos::center()`. Tests of
  the input itself (`input`: keys, pointer, wheel, over the UI, typing) go through the layer's `InjectedInput`
  (`tap`, the virtual mouse, cursor and wheel). Never inject Bevy's raw input, except a text field's own keys
  (Enter, Esc) and the window's close button (`WindowCloseRequested`).
- **QA**: `--editor-script` applies its actions in the first frame, before any screenshot. The editor's sim stays at
  tick 0, so take screenshots at tick 0 unless the script play-tests (`"PlayTest"`).

```sh
echo '["FitMap", {"SetTool": "Paint"}, {"Press": [100.0, 80.0]}, "Release"]' > /path/to/edits.json
cargo run -- --editor maps/dev_skirmish.json --editor-script /path/to/edits.json --screenshot 0:/path/to/editor.png
```

- **UI**: bevy_ui with Bevy's `bevy_ui_widgets` feature (headless `Button`, `TextInput` and `ScrollArea`) and
  `ui_picking`. Its font has ASCII glyphs only, so UI texts stay ASCII ("Rotate 90", not "90°"). The player slot
  colours, the overlay, locked-part and slope tint colours, UI sizes, `NEW_MAP_SIZE`, `MAX_DESCRIPTION_CHARS` and the
  sim's creation limits are placeholders.

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
