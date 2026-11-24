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
  `orders` (`Orders`, `orders.rs`: the current order with its behaviour's state, a queue of at most
  `MAX_QUEUED_ORDERS`, and what the unit does on its own while it has none; see *Behaviours*), `mover`
  (`Mover`, `mover.rs`: goal, status, path, the replanning state of `docs/movement.md > 3` and the sidestep of a
  yielding unit), for units that
  can attack, `attack` (`AttackState`, `combat.rs`: see *Combat*) and, for units that carry a Riot Shield,
  `riot_shield` (`RiotShield`, `riot_shield.rs`: the shield's mode, see *Combat*). Production and the rest arrive as
  more optional fields, each with its own module. Never add them as separate stores keyed by
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
5. Add the `Command`s that drive it, with validation, and the `SimEvent`s the client needs. A unit's mode (a
   toggled ability such as the Riot Shield) is not an order: it goes through `Command::Ability` (an `Ability`
   variant, see *Commands, events and replays*), never into the order queue.
6. Write unit tests in the module and a scenario test in `crates/sim/tests/`.

## Tick pipeline

`Sim::step(&TickInputs)` advances exactly one tick. It runs fixed phases in this order (`sim.rs`):

1. **Clear events**, so that `Sim::events()` holds only this tick's events.
2. **Advance the tick** (`tick += 1`).
3. **Snapshot** `pos`/`facing` into `prev_pos`/`prev_facing` for every object (the client interpolates from these).
4. **Commands**: apply each `PlayerCommand` in `TickInputs` order (`apply_command`).
5. **Systems**, each walking the objects in id order, one function per module (`docs/movement.md > 4`):
   - Modes, `riot_shield::step`: a Riot Shield on its way up or down goes on one tick. After the commands, so that
     a toggle counts from its own tick; before 5a, so that the behaviours, the movement and the combat of the tick
     all see the mode it leaves.
   - 5a `orders::update`: a unit's finished order completes and its next one starts, until one is in progress, so
     legs follow each other without an idle tick; then that order's behaviour runs for the tick, or the idle one
     (see *Behaviours*). They write goals (`Mover::go_to`, `stop`) and the target the attack engages. Everything
     they read of other objects comes from one snapshot taken first (`orders::world::World`): the positions and the
     vision the last tick left.
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
   - 5f `Vision::update`: every player's sight from the objects as the tick leaves them, those this tick's commands
     spawned included (see Vision). Combat in 5e reads the vision of the tick before.
   - `update_supply_stations` (`core/map.md > SupplyDeliveryStation`).
   - `Vision::remember`, last: each player's records of the structures and resources in its sight take the tick's
     final state, the deliveries included (see Vision).

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
  holds a unit that may not: it stands, keeping its goal, path and stuck clock (`movement::wait`). The behaviours
  start a chase only when `may_move` holds. A unit that moves in 5d and starts aiming in 5e (it aims in the tick it
  comes into range) may turn twice in a tick; harmless while every M1 unit turns 180° per tick.
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
- **Hits** (`combat/damage.rs`): every attacker acts against one snapshot of the targets, and an attack's effect
  lands at the end of Firing as a `Hit` on the tick's list. The list is applied once all have acted, so every hit
  counts, including those on a unit another hit has killed and those of a unit that dies in the same tick.
  `hit_damage` follows `> SingleTargetDamage` and `> DamageModifierOrder` (pre-armor multipliers, PointArmor,
  directional multiplier, MinimumDamage); HP is fractional, nothing is rounded. An object at 0 HP or less dies.
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

- **Sources**: every object with an owner and a SightRange above 0; vision goes to its owner. Each sees a disc
  around its centre: a tile is seen when its centre (`TILE_SIGHT_POINT`) lies within the SightRange, boundary
  included, with the ElevationModifier between the object's elevation and the tile's level at its centre
  (`elevation::sight_range`: +1 GU towards lower tiles, −1 towards higher, none for air). No line of sight: terrain
  blocks nothing.
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
- **Not yet**: paths planned from what each player knows (design changes 0040 and 0042, with player-built
  structures): other players' structures from `record_tiles`, stale when `records_revision` changes; the player's
  own from the live objects, as they are never recorded. Map-placed patches and stations, known from the start for
  paths (0042), are records only once seen.

## Behaviours

Slice E (`orders.rs`, `orders/`; `systems/controls/behaviors.md`). Phase 5a runs one behaviour per unit and tick: its
current order's, or the idle one. A behaviour never moves or shoots anything itself: it writes the mover's goal and
the target the attack engages, and reads `Mover::status` and the snapshot.

- **State** (`orders/state.rs`, the design's BaseBehaviorState): `ActiveOrder::state` (`Behaviour`: `Plain`,
  `Attack`, `Approach`, `AttackMove`, `Patrol`, `Hold`) and `Orders::auto` (`Auto`: what a unit without an order
  does: `Watching`, `Engaging`, `Returning`). Hashed and serialized like all state; every order that starts resets
  `auto`. A state names objects only by id, and the removal path clears them (`Orders::forget`).
- **The snapshot** (`orders/world.rs`): `World` holds a `View` of every object (position, shape, owner, domain,
  elevation) as the last tick left it, the vision of its phase 5f, and a coarse grid of the objects a scan may
  pick. Every unit decides on the same world, whatever its id. `World::pick` is the scan of `> BaseAutoTargeting >
  Target Selection Priority`: among the ValidTargets in reach (enemies only: another player's destructible objects
  that the unit's owner sees and its attack can target), threatening ones first, then the least rotation from the
  unit's facing, then the smallest gap, then the lowest id.
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
  blocking footprint changes. The caller chooses the blocking footprints (`NavRules::footprint_blocks`). Units are
  never in a grid.
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
- **`NavGrid::open_around(p)`**: the tile of `p` and its eight neighbours are free of solid tiles. The movement
  step asks it before `box_clear` and `los`, which hold there for a small box and a short move. A `Path` is unit
  state (serde, `PartialEq`, `StateHash`); `NavCounters` are output only.
- **Open questions** (Q0011) are the switches `RESOURCES_BLOCK`, `RUGGED_IMPASSABLE` and `ROUTE_ON_TRUE_MAP`.
- **In the sim** (`paths.rs`): `NavState` holds the `Nav` of every unit type's `(class, size class)` and the
  footprints it was built from (structures and map resources block, Q0011). It is derived, like the `Nav`: not hashed,
  compared or serialized, built when a `Sim` is created or loaded and rebuilt in phase 5b. `Sim::nav()` (the grids),
  `nav_version()` and `nav_counters()` expose it to tests and overlays.
- **Group destinations** (`orders/destinations.rs`, `docs/movement.md > 6.4`, the design's magic box): a location
  order (Move to a point, AttackMove, Patrol) to several units looks at the bounding box of their silhouettes
  (design change 0028). A click inside it sends every unit to the click; outside, each unit goes to the click plus
  its offset from the box's centre. The order keeps the unit's own destination (`Order::slot`; `None` is the click),
  resolved once, at the click (design change 0030). A Shift-queued order takes each unit where its earlier orders
  leave it (`planned_end`, design change 0029).
  A destination is clamped inside the map, and one the unit can't stand on or reach moves to the nearest point it
  can (`nav::reachable_point`, which never searches; design change 0025).
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
magic box did, **(Y)** where friendly yielding and the answers to Q0008 did, **(Q)** where queued group orders did, **(G)** where
the hardening of crowds and nav performance did),
and the code is `crates/sim/src/movement.rs` and `movement/`, with the unit's state in `mover.rs`.

- **One pipeline, sequential commit** (`movement::step`, phase 5d): the step builds one spatial grid per collision
  layer (ground, underground; air units collide with nothing) from every unit, then moves the units in id order and
  updates the grid at each commit. A unit sees earlier units where they now are and later ones where they were.
  The grids are scratch, rebuilt every tick and never hashed.
- **No overlap by construction**: a position is committed only if `World::is_free` holds, the same strict predicates
  as invariants I1 and I2 (touching is free). Contact positions are assigned from the blocker's edge
  (`space::resting`), never accumulated. Never write `pos` anywhere else, and never loosen the predicate with an
  epsilon.
- **Lane seeking** (`World::lane`, `World::lane_step`; `docs/movement.md > 6.3`): before anything else, a unit
  looks 4 of its lengths ahead for the units it has to pass: oncoming ones (by `Body::heading`, where they are
  going) and idle units that yield to it. Each bars a range of sideways shifts, its lane. The unit steps, forward
  and sideways at once, towards the nearest clear lane within 3 of its widths, or, in front of a dense group,
  towards the middle of the widest gap; two dense groups mesh like combs and spread from the outside in. One idle
  unit alone is no lane. The constants are the `LANE_*` in `collide.rs`. It is deterministic by sorted lanes and
  fixed cell order, and keeps no state but `Mover::avoid_side`. Never let it write a position except through
  `World::clear`.
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
- **Collision exceptions** go in `space::blocks(mover, other)` and nowhere else (crushing will).
- **Held by its attack**: a unit whose attack sequence holds its base (`combat::may_move` or `may_turn` false: see
  *Combat*) stands for the tick and keeps its goal, path and stuck clock (`wait`), like a unit waiting for its path.
- **Stuck and recovery** (`stuck.rs`; the numbers, in `mover.rs`, are the design's placeholders of changes 0024
  and 0026): progress is the remaining path length; waiting behind a unit that is getting on counts. A unit is
  stuck after `STUCK_TICKS` (24) without it (`Mover::is_stuck`); it then tries another route `MAX_REPATHS` times,
  after waits of `REPATH_WAITS` (8, 16, 32; `Mover::recovery_due`), around the tile of a blocker that stands, and
  gives up 24 ticks after the last: `MoveStatus::Failed` and `SimEvent::MoveFailed`, `GIVE_UP_TICKS` (104) after
  its last progress. **Occupied target**: within `OCCUPIED_ARRIVAL_GU` (2) of its goal behind a stationary unit
  that won't yield to it, the unit has arrived instead, without a repath: when stuck, or after `JAMMED_TICKS`
  without a way forward if the blocker has no goal. Two moving units head-on in a passage too narrow to pass both
  fail.
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
- **Invariants** (`docs/movement.md > 8`, `sc_sim::testing::invariants`): I1 no two units of a layer overlap; I2 no
  ground box overlaps a blocked tile or leaves the map; I3 a unit moves at most its speed per tick; I4 every float
  is finite; I5 a unit without a goal doesn't move; I6 repaths per leg are capped and no path request waits longer
  than `MAX_REQUEST_AGE`; I7 an arrived unit never moves again without an order or a move of its behaviour, and a
  unit without a goal is at rest. A leg is one `Mover::go_to`: an order's move, or one a behaviour starts (a chase,
  a Patrol's next leg, the way back to an IdleOrigin). The one exception to I5 and I7 is a unit taking its sidestep,
  which must have neither a goal nor an order. A `TestSim` checks them after every tick and panics with a picture of the map (`TestSim::ascii`: units as
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
  and targets may name anyone's objects), positions are finite and on the map (`OffMap`), an order's target
  object is in the issuing player's sight (`NotVisible`, below), spawns fit (`Spawn(SpawnProblem)`). A rejected command changes nothing, neither the state nor the events, and never panics.
  There is no rejection event yet: add one as output outside the state hash when the UI needs it.
- **`Command::Order { units, kind, queue }`** (`systems/controls/commands.md > Unit Command`, `docs/movement.md > 3`):
  `OrderKind` is `Move(Target)` (a point or an object), `Attack(id)`, `AttackMove(point)`, `Patrol(point)`, `Hold`
  or `Stop`. The gate takes an order whole or not at all, but tolerates stale selections: at most `MAX_ORDER_UNITS`
  ids (`TooManyUnits`), repeated ids count once, ids of objects that are gone and objects that take no orders are
  dropped, and nothing left is `NoUnits`; any id of another player's object rejects it (`NotOwned`). Points must be
  finite and on the map, target objects must exist (`UnknownObject`) and an attack's target be destructible
  (`NotAttackable`). Without `queue` the order replaces the unit's queue and current order; with it, the order is
  appended (a full queue drops it). Each order command is one group (`Sim::next_group`); a location order gives
  each unit of a group its own destination (the magic box, see *Navigation*). What each order does is in
  *Behaviours*.
- **`Command::Ability { units, ability }`**: the units use an ability at once, beside their orders: no queue, no
  Shift, no group, and their orders go on. `Ability::RiotShield { raise }` says which way rather than "toggle", so a
  group and a replay mean the same whatever each unit's mode. The gate tolerates stale selections as for orders
  (`TooManyUnits`, repeats, gone ids, `NotOwned`, `NoUnits`) and drops the units without the ability
  (`Reject::LacksAbility` when none is left).
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
- `Command::Debug(DebugCommand::…)` covers `Spawn`, `Remove`, `SetPosition` and `SetAutoTargeting` (see
  *Behaviours*), for development and QA. It bypasses costs and rules, only works while `SimConfig::debug` is on, and
  is never offered in a real match. A teleport sets `prev_pos = pos`, so nothing interpolates across the map.
- **Replay** = the `Scenario` (the map and the game setup, game options included) plus the `TickInputs` of every
  tick, empty ticks included, and state-hash checkpoints: `sc_sim::Replay` (serde) with `start`/`record`/`step`/`run`,
  and `record_checked`/`verify`/`run_checked` for the checkpoints (see Replays and golden hashes, which also covers
  the client's recording and playback). A replay keeps every command whole: `testing::every_command` lists one of
  each kind (every variant of `Command`, `OrderKind`, `Target`, `Ability`, `DebugCommand` and `SpawnAt`), and the
  format tests round-trip it through the sim's serde and the client's file; its match stops compiling when a variant
  is added, so add the new one to it. The client must never submit a command with a non-finite number. The sim
  would reject it anyway, but JSON can't store NaN or ±∞, so the replay file would fail to load. Validate at the
  input layer.
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
  a unit, and `ascii(focus)` draws the map. Units of different players fight when they see each other: a test
  about anything else calls `without_auto_targeting()` first, or keeps them apart or on Hold out of range.
- **Movement scenario tests** are `crates/sim/tests/movement_collision.rs` (passing, head-on, choke, group arrival,
  idle units, repaths, oscillation bounds, save and load in a crowd), `group_destinations.rs` (the magic box),
  `queued_group_destinations.rs` (Shift-queued group orders),
  `movement_yield.rs` (friendly yielding, occupied targets, the give-up schedule), `movement_soak.rs` (200 units,
  random orders, 2 000 ticks; bounds on the share of failed moves) and the ignored `movement_perf.rs`.
- **Behaviour scenario tests** are `crates/sim/tests/behaviours_attack.rs` (the approach to a gap of Range, the
  chase, the last seen position, a target out of reach, queueing), `behaviours_idle.rs` (acquiring by priority,
  the leash and the return, Hold, Move doesn't engage, the auto-targeting switch), `behaviours_travel.rs` (Follow,
  static objects, AttackMove groups and the leash, a fight replayed and saved in its middle),
  `behaviours_patrol.rs`, `behaviours_gate.rs` (targets must be seen; a GDO player's remembered station, by its
  record), `behaviours_soak.rs` (120 units fighting
  under random orders of every kind; no dangling ids, no path-request storms) and the ignored `behaviours_perf.rs`.
  `riot_shield.rs` has the Riot Shield's: toggle timing, each raised-shield effect, the gate, a save in the middle
  of a toggle and its replay.
  Measured (release, 500 units on 256 × 256): 0.12 ms a tick with everyone idle, 0.14 ms on Hold in sight of the
  enemy, 0.5 ms on average in a battle of 250 against 250; the worst ticks, 4.4 ms, are those in which a group's
  path field is built (four or more of its units resuming their AttackMove in one tick).
- **Resource collision tests** are `crates/sim/tests/resources_block.rs` (design change 0038's verification: units
  path around a crystal field, can't walk across a Supply Delivery Station, and walk through the tile of a patch
  removed by a debug command, standing in for depletion).
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
  and QA's `hold_at`. `Pace` says how fast, changeable at any time: `Scaled(f)` runs `f` × 16 ticks per second of
  real time through `Time<Virtual>`'s relative speed, `Max` (`--fast`) as many ticks per frame as fit in
  `FAST_FRAME_BUDGET`, drawn exactly at their ticks. A change applies in `PostUpdate` (`apply_pace`), so the next
  frame's time update already goes at the new pace, and at a scaled pace the fixed loop never owes more than a
  frame's worth (`limit_fixed_loop`: 250 ms of scaled time), so leaving `Max` runs no burst of ticks. A replay's
  playback controls both (see Replays and golden hashes).

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
  dark rim and a light tick along the facing; the visual turns with the interpolated facing. The Guard's square
  doesn't: it stays axis-aligned like its collision box, and only its tick turns (a `FacingTick` child,
  `body_turns`). One mesh per `(kind, owner)` is shared, and one tick mesh per kind. A unit that carries a Riot
  Shield has a `RiotShieldVisual` child: a pale arc in front of it, hidden while the shield is down, growing to its
  full size as it goes up. It is `Visibility::Hidden` or `Inherited`, never `Visible`, so the fog that hides its unit
  hides it too.
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
  is written only when `PlayerVision::revision`, the local player or the sim changes, the mesh only with the map.
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
finds the portrait under a click, because headless tests have no UI picking (and `CommandGrid` the command button).
The InfoPanel (`hud/info_panel.rs`) and the SelectionPanel (`hud/selection_panel.rs`) show the selection (see
Selection), the CommandPanel (`hud/command_panel.rs`) its commands (see Commands UI); a section with nothing to show
shows its name (`SectionLabel`). A selected object the player doesn't see shows its record's last-known state and a
"Last seen … ago" line (a placeholder wording), except that a GDO player's remembered Supply Delivery Station shows
the live station's NextDeliveryTime (`core/map.md > SupplyDeliveryStation > InfoPanel`, design change 0057), "No
delivery scheduled" while it isn't empty, and never CurrentSupplies. The top bar shows the player and the tick; in a
replay's playback also the replay's last tick and a note on the playback (`ReplayText`: speed, paused or end, and a
desync in red).

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
  portraits, and F2 for the debug player switch, the fog of war F3 for its debug toggle (placeholders: the design
  has no debug keys), and the replay playback keys of the game's context: Space pauses and resumes, `.` plays one
  tick, `-` and `=` change the speed (`ReplayPause`, `ReplayStep`, `ReplaySlower`, `ReplayFaster`; placeholders).
- **Output**: `ActionState` (a `Resource`) holds this frame's actions (held, just pressed, just released), the wheel's
  turn, the modifiers, the context, the cursor in logical pixels, and the `Pointer`: the view point and the ground
  point (`coords::ground_under`) under the cursor, whether it is over the UI (a UI node by Bevy's picking, or outside
  the world view: the one over-UI test of the game and the editor), the press in progress and the drag box. A press of
  `Select` or the editor's `UseTool` that goes down on the map is followed until its release (`Pointer::press`, a
  `MapPress`: where it went down, whether it moved further than `DRAG_THRESHOLD_PX`, and the ground point under the
  cursor now, which keeps following it over UI nodes on the world view, as the editor's drags do). `MapClick`
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
  view space. A click picks the frontmost box under it
  (grown by `PICK_SLOP_SU`); a box (the input layer's `SelectionBox`) applies the priority tiers. `Relation::of`
  says own, enemy or neutral (no teams yet, so no allies); allies would count with enemies in the tiers.
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
  change, and puts the ObjectInterfaceState back to DefaultState when the Selection or its ActiveGroup changes. While
  a target is awaited, map clicks and boxes select nothing (the drag box isn't drawn): the left-click is the
  target's.
- **Visuals** (`selection/markers.rs`): one marker entity per selected object: a ring around a unit, in the
  ground-decal layer under it, or a frame inside a footprint, which the object's visual would hide there, so it is
  drawn just in front of the object (`FRAME_Z`: half a su of ground y, under the objects south of it); coloured by
  relation, the ActiveGroup's solid and the others' dashed; meshes cached per shape, relation and activity. The drag
  box is four edges of one shared mesh in the overlay layer, shown while `Pointer::drag_box` is set.

**Commands UI** (`command_ui.rs`, `command_ui/`; `systems/controls/selection_and_hud.md > ObjectInterfaceState`
to `> CommandIndicators`, `systems/controls/commands.md > Unit Command`): how the local player orders their units.
- **Rules** (pure and unit tested): `command_ui/interface.rs` has the `UnitCommand`s, the `InterfaceState`
  (DefaultState, or AwaitingTarget of a command) in each `ControlState`, the ActiveGroup's `InterfaceTemplate` (the
  Peacekeeper's and the Guard's BasicCombatUnit grid), the `Panel` a selection gets (common, group, pending, disabled
  and unavailable buttons; Back in Z while a target is awaited) and who receives a command (a common one the whole
  selection, a group one the ActiveGroup). A type's grid is its template's plus its own commands (`grid`: the
  Riot Shield in R for a unit that carries one, a placeholder slot). `Modes` holds the selected units' modes as the
  sim has them: the Riot Shield button is on, and lowers the shields, while all its recipients' are up or going up,
  else it raises them; Attack is disabled while none of its recipients can attack. `command_ui/resolution.rs` has
  what a click resolves to (`CursorTarget`, `right_click`, `target_order`, and `order_without_attack`: a Move to the
  clicked point for a unit that can't attack) and the `PointerDisplay` that previews it. Nothing is commandable
  unless the whole selection is the player's (`commandable`).
- **Systems** (`CommandUiSystems`): `Validate` (in `SyncSim`, after the selection's sync) ends an AwaitingTarget that
  no longer holds. `Input` (in `Actions`, after the selection's input) turns Esc, the command hotkeys
  (`InputAction::CommandSlot`), clicks on the panel (found by `CommandGrid`), left-clicks on the map while a target is
  awaited and right-clicks into state changes and orders. An order is a `Command::Order` in `PendingCommands` naming
  only the player's own units, given only if `Sim::validate` takes it, queued while `Queue` (Shift) is held; each one
  is also an `OrderIssued` message. A click that resolves to an Attack gives it to the units that can attack and a
  Move to the clicked point to those whose Riot Shield is up (`OrderOutbox::give_click`). An ability is a
  `Command::Ability`, validated the same way, never queued, and no `OrderIssued`.
- **Visuals**: the panel's buttons (`hud/command_panel.rs`) are rebuilt when the panel, the bindings or the surface
  changes; each shows its command (`PanelSlot::label`: a toggle says what pressing it does) and its hotkey from the
  bindings, and a toggle that is on has a thick green border. The indicators (`command_ui/indicators.rs`) are
  one mesh in the overlays layer, rebuilt every frame while anything shows: markers at the targets of the selected
  own units' current orders (and queued ones while Shift is held), a flash where an order was just given
  (`FLASH_TICKS`), and while Shift is held each unit's route through its waypoints as dashed lines; colours follow
  `> Color language`; a target object the fog hides gets no marker and ends no leg (`Sight::sees`), but a Move to
  a remembered one is marked at its record. The pointer
  (`command_ui/cursor.rs`) shows the `PointerLook`: images drawn in code and cached per look, set as the window's
  `CursorIcon`, and drawn as a UI node at the injected cursor in QA runs, which render no system cursor.
- The design leaves some things open (labels, colours, the flash, the cursor's shapes, who carries out a
  right-click, what an invalid target click does); each placeholder is marked where it is made.

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
| `--replay <file.json>` | Plays a recorded game back (see Replays and golden hashes) with its own map and game options: not with `--scenario`, `--map`, `--editor`, `--no-debug` or `--commands`. Pauses at the end, so no tick of the other flags may come after it. The other QA flags work with it. |
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
- The `--dump-state` JSON holds `tick`, `scenario` (the map's name), `players`, `objects` (every field of
  `sc_sim::Object`), the last tick's `events`, `client` (`local_player`, `camera_center` as a view point in su,
  `grid_visible`, `fog_of_war`, `recorded_ticks`, the local player's `selection` and `last_known` records, the
  `recording` file, and in a playback `replay`: its `file`, `end_tick`, number of `checkpoints` and the `desync`, if
  any) and `map` (the terrain).
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
  ticks and when the app exits (window closed, Ctrl+C, a QA run's end), atomically (`files::write_atomically`), so a
  crash loses a few seconds at most. QA runs record only with `--record`; `--no-record` turns recording off. Should
  the sim be replaced (`SimRes::generation`), the recording ends there.
- **Playback** (`--replay`, `crates/game/src/replay/playback.rs`): the game starts from the replay's scenario with its
  game options, and each step applies the recorded inputs of its tick and nothing else. The local player can't give
  commands: the command UI's input set (`CommandUiSystems::Input`) doesn't run, and anything else pending is dropped.
  It pauses at the end. The game is checked against the checkpoints at tick 0 and after every step; the first desync
  is logged as an error, fails a QA run (exit code 1) and shows in red in the top bar, and the playback goes on.
  Controls (Game context, placeholder keys): Space pauses and resumes, `.` plays one tick and pauses, `-` and `=`
  step through the speeds ×0.5, ×1, ×2, ×4 and max (`playback::SPEEDS`; `--fast` starts at max), F2 switches the
  player and F3 the fog, also when the recorded game had no debug commands. The top bar shows "Replay x2", ", paused"
  or ", end", and "Tick n / end". Screenshots, `--exit-at-tick`, `--dump-state`, `--input-script` (to press the
  playback keys), `--windowed` and `--headless` work with it; a windowless playback ends at the replay's end unless
  told otherwise. To check a replay quickly: `cargo run -- --replay <file> --headless --fast`.
- **Golden hashes** (`crates/sim/tests/golden.rs`): six scripted games, a 3v3 duel, a 40-unit AttackMove battle, a
  12-unit group move through a choke, a patrol that meets an enemy, a chase into the fog, and three Peacekeepers that
  raise their Riot Shields (`Command::Ability`; one turns back half-way), walk up to a Guard under fire and lower
  them to kill it, are blessed as replays with a checkpoint every 32 ticks and at the end, in
  `crates/sim/tests/golden/<name>.json`. The test plays each game from its definition, checks that the definition
  still is the blessed one, and compares the checkpoints; a mismatch fails with the first diverging tick and the
  last one that matched. Each game also asserts what it is about, at its end (`expect`: one side of the duel is
  gone, the group passed the choke, …) and along the way (`during`: the shielded Peacekeepers held their fire), so
  a re-bless can't hide a broken game. Any change to the sim's rules or state changes the hashes: when the change is
  intended, re-bless and commit the files with it:

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
