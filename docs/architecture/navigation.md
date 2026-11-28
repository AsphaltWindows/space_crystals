# Navigation

Read this when you touch pathfinding, path knowledge or the nav grids.

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
  expansion budget, with one shared field per order group of `Paths::group_field_min` or more, anchored on the click's tile.
  A member whose goal the anchor can't see plans alone: through the anchor it would take a detour. The field is
  built only as far as the members stand (`Search::field_to`: towards their tiles' bounding rectangle, until each
  is expanded), and the members share the smoothing of the chains they share (`smooth_shared`, same waypoints as
  `smooth`). The budget's rule is unchanged: a job starts while budget remains and always finishes.
  `serve_on(navs, on, ..)` is the same for requests that plan on different sets of grids (the sim's: one per
  player), in the one order and budget; a field serves members of one set only.
- **`NavGrid::open_around(p)`**: the tile of `p` and its eight neighbours are free of solid tiles. The movement
  step asks it before `box_clear` and `los`, which hold there for a small box and a short move. A `Path` is unit
  state (serde, `PartialEq`, `StateHash`); `NavCounters` are output only.
- **Switches** of Q0011's answers: `Paths::resources_block` (design change 0038) and `Paths::rugged_impassable` (0039).
- **In the sim** (`paths.rs`, `paths/`): `NavState` holds the **true grids**, the `Nav` of the `(class, size class)`
  of each unit in the game (*Which grids*, below) under every blocking footprint (structures and map resources; an
  underground expansion's, `BlockerKind::UndergroundExpansion`, blocks none: units walk over it, plan decision T32
  and design change 0240; nor does a pop-up turret's while it is fully lowered, `paths::blocking`, slice M4-S2, nor a
  neutral turret's record, `paths::remembered_blocking`, slice M5-Y), and each player's **known grids** (below). It
  is derived, like the `Nav`: not hashed, compared or serialized, built when a `Sim` is created or loaded. The true
  grids are rebuilt in phase 5b; `Sim::nav()` (the true grids), `nav_version()`, `nav_counters()` and
  `nav_rebuilds()` (sets of grids the latest step rebuilt) expose it to tests and overlays.
- **Which grids** (slice M5-Z2, `paths/grids.rs`): every set of grids, the true ones and each player's, holds one for
  each `(class, size class)` that a unit in the game walks on, or one about to come into it (`paths::walking_keys`:
  on the map, held off it, or queued at a producer, a Recruitment Hall's Recruit from the start). A grid is built
  with the first such unit and never dropped (`NavState::cover`): the true one of the true footprints, each player's
  of its own, so a grid built late is the one an eager build would have had, which grids a game has never shows in
  its play, and no version or rebuild counts one. The grid sync steps cover every unit (`Sim::cover_grids`, in phase
  5b after the sync and at the end of the tick before `sync_known`); a debug spawn and a change of kind cover theirs
  at once (`Sim::cover_kind`); a start and a load build those of their units. Every reader of a unit's grid goes
  through `paths::unit_grid` or `NavState::plan_grid`, which fail in debug builds on a missing one (invariant I2
  reports it too). The gate, asked about a spawn of the first unit of a class, checks it on the grid it will have
  (`NavState::fresh_grid`), and `Sim::true_grid` gives tests the ground of a class no unit of which is in the game.
  A new unit type needs nothing: its definition gives its key.
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
  - **Lazy**: a player's grids exist only while it has a unit that walks on a grid, of the classes the true ones
    have (*Which grids*); `ensure_known` builds them for an order in the tick of its first unit's spawn.
  - **No owner**: a neutral unit (and a defeated player's, design change 0101) plans on the true grids by
    `nav_version`: it has nobody's knowledge and sees nothing. Code that changes a unit's owner should call
    `Mover::repath`, since the unit's path was planned on other grids.
  - Measured (`tests/nav_perf/`, ignored, release; 4 players, 412 units and 160 patches on 256 × 256): the tick
    in which ten Generators go in sight of all four players rebuilds 5 sets of grids and takes 1.0 ms; the worst
    tick after it, with the units' path checks, 1.3 ms. Its sets hold one grid, `(GroundRugged, 1)`, the
    Peacekeepers', which is also every other infantry's and the Drone's (a HoverVehicle crosses Rugged terrain,
    design change 0039; on its stand-in mover it had a grid of its own, which doubled every set). The Cults' wheeled
    and tracked vehicles (slice M5-G2) don't cross it and walk on `(Ground, 1)`, which slice M5-G2 put in every set,
    whether or not a vehicle was in the game; slice M5-Z2 builds it with the first one (*Which grids*). A/B against
    master, interleaved, 8 rounds (medians, load average 2.3 to 2.7): the rebuild tick from 1.64 to 1.02 ms, the
    ticks around it unchanged (0.32 and 0.35 ms); with a vehicle in the game (`knowledge_rebuilds_256_with_a_vehicle`,
    two grids a set) 1.59 ms, master's cost. `structures_perf` (no vehicle): the whole tick 0.358 ms on both sides,
    p99 0.71 to 0.68 ms, the grid syncs' p99 0.26 to 0.15 ms, a tick with a rebuild 0.48 to 0.41 ms.
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
