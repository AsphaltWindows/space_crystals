# M1 movement, pathfinding and collision: technical design

**Status: approved by the Director on 2026-11-23.** This is binding for M1 movement work, alongside `ARCHITECTURE.md`.
- **Decisions:**
  - The proposed gate change is adopted. Order commands drop ids that no longer exist, and reject only when no unit remains or one isn't owned.
  - f32 stays.
  - Paths are hashed state.
- **The gaps in section 2** went to the Designer as questions:

  | Gaps | Question |
  |---|---|
  | G1–G3 | Q0007 |
  | G4–G5 | Q0008 |
  | G6–G7 | Q0009 |
  | G8–G9 | Q0010 |
  | G10–G12, G14 | Q0011 |

  G13, the queue cap, is technical and set to 32. Until a question is answered, implement the default from the table, behind a named constant or switch that cites its question id.
- **The benchmark** cited in 5.8 was a throwaway spike file and isn't in the repo.

Design references are `file.md > Section`, relative to `artifacts/designer/design/`.
Every number marked *(placeholder)* becomes a named const tagged with the question that covers it (section 2).

## 1. Requirements from the design

| Topic | Rule | Source |
|---|---|---|
| Time, space | 16 ticks/s; speeds in su/tick; 64 su = 1 tile | `core/scale.md` |
| M1 units | Peacekeeper: LightInfantry, 24×24 su, 4 su/tick (2 with Riot Shield up), Range 4 GU, Sight 5 GU. Guard: HeavyInfantry, 36×36, 5 su/tick, Range 3 GU. Both: TurnRate 180°/tick, infinite acceleration and deceleration, so M1 kinematics are instant | `factions/gdo/units.md > Peacekeeper`, `factions/syndicate/units.md > Guard` |
| Later range | Silhouettes 24–60 su so far (vehicles TBD, may exceed a tile); speeds 4–8 su/tick; five movement models (TurnRate, FixedTurnRadius, SpeedTurnRadius, Drag, Glider) | `systems/units.md > MovementModel` ff., faction unit files |
| Domains | Ground, Air, Underground per UnitBase | `systems/units.md > UnitBase` |
| Ground collision | Hard, never overlapping; shape is the Silhouette rectangle; **idle units do not move aside**; moving units path around occupied space; ground units collide with ground units and structures | `systems/units.md > UnitCollision > Ground Collision` |
| Air collision | None with ground or structures; soft separation among air units (SeparationRadius) | `> Air Collision` |
| Crushing | Moving TrackedVehicle/Mech pass through and kill enemy LightInfantry; not friendly; stationary crushers are solid | `> Crushing` (not M1, but the collision filter must allow it) |
| Sub-tile lanes | A 64 su gap fits two 24 su units side by side, one lane each way | `factions/cults/units.md > Recruit` (throughput note) |
| Terrain | Non-Traversible tiles block ground units. RuggedTerrain = true bases (infantry, Mech) cross Rugged tiles. Drill units travel on Drillable tiles | `core/map.md > Tile`, `systems/units.md > UnitBase`, `> DrillUnit` |
| Elevation | Slopes change neither speed nor path cost; cliffs are non-Traversible tiles; corners are shared, so two adjacent Traversible tiles are always connected | `core/map.md > Elevation` |
| Underground | Expansions are walked over by surface units | `factions/syndicate/structures.md > Tunnel Expansions` |
| Move | To a location or an object; ends with Stopping, complete when stopped; completes "within proximity" of an object; on repeated failure to progress: stop, complete | `systems/controls/behaviors.md > MovingToLocation`, `> MovingToObject` |
| Chase | AttackingObject approaches until the target is within Range, beyond MinRange and in facing; resumes when it leaves range. Infantry (UnitBaseSource) cannot move during Aiming, Firing, Cooldown | `> AttackingObject`, `systems/combat.md > AttackPhase` |
| Range | ±1 GU by elevation level difference, ground and underground only | `core/vision.md > ElevationModifier` |
| AttackMove, Patrol | Leash 6 GU perpendicular to the path, recompute the path on resuming; Patrol swaps legs forever | `> AttackMovingToLocation`, `> Patrolling` |
| Idle | Auto-acquire in SightRange, leash 4 GU from IdleOrigin, return to it | `> BaseAutoTargeting` |
| Hold, Stop | Hold: Locomotion Stationary always, never completes. Stop: Stopping, then no behaviour | `> HoldingPosition`, `> StoppingBehavior` |
| Queue | Shift appends, otherwise the command replaces the whole queue; next order starts when the current behaviour completes | `systems/controls/commands.md > Unit Command` |
| Channels | Locomotion (Moving, Reversing, Stopping, Stationary) and Orientation (Turning, Maintaining) are named state; combat and crushing read them | `behaviors.md > BaseActionChannels`, `combat.md > LocomotionOrientationConstraints` |
| Group moves | **Nothing.** Selection has no size limit | `systems/controls/selection_and_hud.md` |

## 2. Design gaps and contradictions

Top five first. Each default is what this design implements until answered.

| # | Question | Proposed default |
|---|---|---|
| G1 | Several units are ordered to one point and cannot all stand on it. Where does each end up? | Each gets its own slot on a compact lattice around the point (pitch = largest silhouette in the group + 8 su), nearest free slots first. Units move at their own speed, not the slowest's |
| G2 | **Contradiction with the Director's brief**, which assumes idle units make way. `units.md > Ground Collision` says they don't. Confirm: no pushing or yielding at all, even for own units blocking a chokepoint? | As the spec says: no pushing. The design has a hook for a later "yield" (section 6) |
| G3 | With hard collision and no yielding, two units meeting head-on in a corridor too narrow to pass (two Guards, 72 su, in one tile) deadlock. Is "both give up and stop" intended? | Yes, by the Failure rule. Alternative to offer: moving friendly units may swap places |
| G4 | "Repeatedly fails to make progress" has no numbers. | Stuck after 24 ticks without progress; at most 3 repaths per order with 8/16/32-tick backoff; then fail (about 7 s in total) |
| G5 | Move target that cannot be reached (cliff, water, inside a structure, cut-off area), or whose point is occupied by a unit that won't move. | Go to the reachable point nearest the target and complete there. Complete when stuck within 1.5 GU of the goal behind stationary units |
| G6 | Does the Silhouette rectangle rotate with facing? All specified ones are square. | Axis-aligned, never rotates. Must be answered before the first non-square unit |
| G7 | How are Range and "within proximity" measured for large targets? | From the unit's centre to the nearest point of the target's silhouette or footprint. Proximity: boxes within 8 su (as MiningReach) |
| G8 | The attack target leaves the attacker's owner's vision during a chase. | Go to its last seen position, then the order completes |
| G9 | Patrol: is the origin the position at issue or when a Shift-queued Patrol starts (`commands.md > Patrol` vs `behaviors.md > Patrolling` step 1)? What if a leg fails? | When it starts executing. A failed leg counts as arrived; two failed legs in a row end the Patrol |
| G10 | Do map resources (patch, station) block ground units? "Structures" doesn't name them. | Yes, their footprints block |
| G11 | RuggedTerrain = false: is a Rugged tile impassable for that base? Implied, never stated. | Yes, impassable |
| G12 | Do units path with knowledge of structures their owner has never seen? | Yes, the true map, in M1 |
| G13 | Command queue length. Orders queued behind Patrol or Hold never run. | Cap 32 *(technical)*; the rest as written |
| G14 | Underground units: do they collide with each other? Can air cross every tile? | Not M1. Assume hard collision among themselves; air crosses everything inside the map |

Note for the Designer, not a question: `MovingToLocation` describes a Plan of channel pairs with an ExpectedPosition
and "recompute on deviation". Under hard collision units deviate constantly, and replanning on every deviation is the
repath storm the prototype died of. This design keeps the named channels as state but replans only by the rules in 5.6.

## 3. Sim data model

Two capabilities, each `Option` on `Object` with its own module (`ARCHITECTURE.md > Object model`), both fully hashed
and serialized. Paths are **state, not cache**: a save loaded mid-game continues bit-identically.

```rust
// defs.rs: ObjectDef gains `unit: Option<UnitDef>`; one static UnitBaseDef per UnitBase (systems/units.md > UnitBase).
pub struct UnitBaseDef { domain: Domain, rugged_terrain: bool, crushable: bool, can_turn_in_place: bool, can_reverse: bool, .. }
pub struct UnitDef { base: UnitBase, movement: MovementAttrs /* enum per MovementModel; None accel = infinite */ }
pub enum Domain { Ground, Air, Underground }             // also the collision layer
pub enum NavClass { Ground, GroundRugged, Underground }  // passability predicate; air needs none
// size class: u8 = ceil(max(silhouette w, h) / 64 su); 1 for every unit specified so far.

// orders.rs: the command queue and behaviour state (commands.md, behaviors.md)
pub struct Orders { queue: VecDeque<Order> /* cap MAX_QUEUED_ORDERS */, current: Option<ActiveOrder> }
pub struct Order { kind: OrderKind, group: OrderGroup /* units of one command share it */ }
pub enum OrderKind { Move(Target), Attack(ObjectId), AttackMove(Vec2), Patrol(Vec2), Hold, Stop }
pub enum Target { Point { click: Vec2, slot: Vec2 }, Object(ObjectId) }  // click: indicator; slot: where this unit goes

// mover.rs: goal in, motion out. Behaviours write `goal` and read `status`; nothing else moves a unit.
pub struct Mover {
    goal: MoveGoal,            // None | Point(Vec2) | Object { id, reach: f32 }
    status: MoveStatus,        // Idle | Moving | Arrived | Failed | TargetLost
    locomotion: Locomotion, orientation: Orientation,   // the design's channels
    velocity: Vec2, speed_factor: f32,                  // Riot Shield: 0.5
    path: Vec<Vec2>, next: u16, path_state: PathState,  // PathState: None | Wanted { since: Tick } | Ready
    nav_version: u32, planned_target: Vec2,             // staleness checks
    repaths_left: u8, repath_ready_at: Tick, avoid_tiles: Vec<TilePos> /* cap 16 */,
    best_remaining: f32, last_progress: Tick, blocked_ticks: u16,
    avoid_side: Side, avoid_until: Tick,
}
```

- **Collision shape**: `half = silhouette / 2`, an axis-aligned box at `pos` (G6). Two boxes overlap only if
  `|dx| < hw_a + hw_b && |dy| < hh_a + hh_b`, strictly: touching is free. Contact positions are *assigned*
  (`obstacle edge ∓ half`), never accumulated, so resting units sit at exact values and don't drift.
- **Derived, not hashed** (`#[serde(skip)]`, rebuilt on load and on change, excluded from `PartialEq`): the `NavGrid`s
  and their component labels. **Scratch** (rebuilt every tick, carries nothing across ticks): the spatial grids and
  the A* arrays. A save/load test (section 8) proves neither leaks into behaviour.
- `Sim` gains `next_group: u32` and `nav_version: u32` (hashed).
- **Commands**: `Command::Order { units: Vec<ObjectId>, kind, queue: bool }`. Gate: owner check per unit, target
  finite and on the map, at most `MAX_ORDER_UNITS`, duplicates dropped. **Proposed gate change for the Director**:
  ids that no longer exist are *dropped* rather than rejecting the command (a unit can die in the tick the order
  arrives; under lockstep delay this is routine). Reject only if no unit remains or one isn't owned.
- **Spawn and teleport** must keep the invariants: `SpawnProblem::Blocked` / `Occupied` when a unit's box would
  overlap a blocked tile or another unit of its layer, in scenario validation, debug `Spawn` and `SetPosition`.

## 4. Movement in the tick pipeline

Phases 1–4 stay as they are. Order commands (phase 4) only write `Orders` queues and compute group slots (6.4); they
never move anything. Then, each walking units in id order over `..first_new`:

| Phase | Does | Why here |
|---|---|---|
| 5a `orders::update` | Pops completed orders, runs the behaviour state machine, writes `mover.goal`, reads `mover.status` and last tick's positions and visibility | After commands, so an order acts in its own tick; before movement, so there is one writer of goals |
| 5b `nav::sync` | If a tile-blocking object or the map changed: rebuild the NavGrids and components, bump `nav_version` | Paths below must see this tick's obstacles |
| 5c `paths::serve` | Serves `Wanted` requests within the budget (5.5) | After goals, before steering |
| 5d `movement::step` | **The one movement pipeline**: steer, resolve collisions, commit `pos`/`facing`/`velocity`/channels, set `status` | Single place that writes `pos` |
| 5e combat | Attack phases, hit list, damage, deaths (other slice) | Shoots from post-move positions |
| 5f vision | Visibility per player (other slice) | Sees final positions |

- 5d builds the spatial grids first from **all** units, including this tick's spawns (obstacles, though not updated),
  then moves units one by one, updating the grid at each commit. A unit tests against movers already at their new
  position and the rest at their old one. Sequential commit makes "no overlap" hold by construction, and id order
  makes it deterministic. The grid holds copies (`id, box, velocity, stationary, owner`), so no second borrow of the
  object map is needed.
- `status` set in 5d is read by 5a next tick. A Shift-queued unit that arrives at tick `t` requests, receives and
  follows its next path at `t + 1`, so it never stands idle between waypoints.
- **Events**: `MoveFailed { unit }` only. Indicators and paths are read from state. Per-tick counters (expansions,
  searches, collision queries) are output outside the hash, for tests and the perf overlay.

## 5. Pathfinding

**5.1 Algorithm.** Two levels. Global: tile A* over static obstacles, then any-angle smoothing. Local: steering and
collision against units (section 6). Rejected: a sub-tile global grid (1M cells on 256², too slow), navmesh (building
changes are costly, hard to verify), hierarchical/portal graphs (unneeded at these sizes, see 5.7; revisit only if
`MAX_MAP_SIDE` grows), flow fields as the *steering* model (tile-quantised directions, poor with hard collision).
A Dijkstra field is used only to mass-produce paths for group orders (5.5).

**5.2 NavGrid.** One per `(NavClass, size class)` in use: a `blocked` bit per tile plus a component label per tile
(flood fill, row-major seeds, so labels are deterministic).
- `Ground`: blocked if not Traversible, or Rugged (G11), or under a blocking footprint. `GroundRugged`: not
  Traversible, or under a footprint. `Underground`: not Drillable. Blocking footprints: surface structures and
  resources (G10), never underground expansions. Slopes never matter.
- Size class `k > 1`: a tile is passable if the `k × k` block anchored on it is (true-clearance); M1 only builds `k = 1`.
- Map edge: a unit's box stays inside the map; tiles outside are blocked.

**5.3 A\*.** 8-connected; a diagonal step needs both orthogonal neighbours free, so nothing squeezes between
diagonal obstacles. Integer costs 10 and 14, octile heuristic, all `u32`: no float enters the search.
- **Closed set and best-cost map**: per-tile `g`, `parent`, `stamp`, `closed` arrays with a generation counter
  (no clearing). A node is expanded once; a neighbour is relaxed only on a **strictly** lower `g`, so equal-cost
  parents are never overwritten (the zigzag bug).
- **Tie-break**: heap key `(f, h, tile index)`, neighbours in a fixed order (E, S, W, N, SE, SW, NW, NE).
- **Unreachable goals never search**: if the goal tile is blocked or in another component, first retarget to the
  tile of the unit's component nearest the goal (ring scan outward from the goal, ties by tile index). So no search
  can fail, and none floods the map.
- **Avoid tiles**: a repath treats the unit's `avoid_tiles` as blocked (6.5). Only this search can fail.

**5.4 Smoothing and line of sight.** `nav::los(class, half, a, b)`: walk the tiles the segment crosses
(Amanatides–Woo) and slab-test the segment against each blocked tile in their 3×3 neighbourhoods, inflated by `half`.
Exact for boxes up to one tile. Smoothing is greedy string-pulling: from an anchor, keep the farthest tile centre
with LOS. Waypoints are tile centres, plus the exact start and goal, which leaves `32 − half` su of corner clearance.

**5.5 Requests and budget.** `PathState::Wanted { since }` on the unit; each tick 5c sorts requests by `(since, id)`
and serves them until `PATH_BUDGET` = 32 000 node expansions is spent *(tuning const)*. A job starts while budget
remains and always finishes, so every request is served within bounded time. Units waiting for a path stand still.
- **Group orders**: when at least `GROUP_FIELD_MIN` = 4 requests share an `OrderGroup`, nav class and component, run
  **one** Dijkstra from the group's anchor tile (same code, `h = 0`), and give each member the parent chain from its
  own tile, smoothed, with its slot as the last waypoint. The field is scratch, dropped in the same tick.
- The budget counts expansions, never time, so it is deterministic.

**5.6 Repath policy.** Per order leg, `repaths_left = MAX_REPATHS` (3, G4). Every repath waits for
`repath_ready_at`; the backoff doubles (8, 16, 32 ticks).

| Trigger | Check | Counts against the cap |
|---|---|---|
| Stuck (6.5) | every tick, cheap | yes; at 0 the status becomes `Failed` |
| `nav_version` changed | once, on the unit's stagger tick: LOS along the remaining path; repath only if a segment is now blocked | no (bounded by how often the map changes) |
| Chase target moved | target is farther than `max(1 GU, 25 % of the distance)` from `planned_target`; at most every 8 ticks | no (rate-limited) |
| Lost LOS to the steering waypoint | on the stagger tick, after being pushed off the path by sidesteps | yes |

Stagger tick: `(tick + id) % 4 == 0`. While a chaser has LOS to its target it steers straight at it with no path.

**5.7 Dynamic obstacles.** A building placed or removed sets a dirty flag; 5b rebuilds grids and components in full
(65k tiles, about 1 ms, and rare). Units are never in the NavGrid. Stationary units reach the search only
through `avoid_tiles`.

**5.8 Performance.** Measured with a throwaway benchmark (`scratchpad/m1/bench/astar.rs`: this exact A\* on 256×256,
10 % random obstacles plus six map-high walls with one gap each, a deliberately bad case):

| Case | Expansions | Time (opt 3 / dev profile) |
|---|---|---|
| Open map, corner to corner | 256 | 19 µs / 26 µs |
| Pairs within 30 tiles | 2 200 avg | 160 µs / 190 µs |
| Random pairs, avg path 270 tiles | 15 000 avg, 49 000 max | 1.4 ms / 1.8 ms |
| Full Dijkstra field | 65 536 | 3.0 ms / 3.5 ms |
| Unreachable goal without the component check | 65 531 | 6.1 ms / 7.2 ms |

About 70–120 ns per expansion. Consequences: a 200-unit group order costs one field (3 ms), not 200 searches
(up to 280 ms); the component check is mandatory; a hierarchy is not needed at 256².
**Targets** (release, 500 moving units, 256×256): movement phase ≤ 1 ms; path serving ≤ 3 ms in a normal tick and
≤ 8 ms worst (budget plus one field); a group order of any size is pathed in the tick it is issued; whole sim tick
≤ 10 ms worst, leaving 6× fast-forward. Tests assert the deterministic counters, not wall time.

## 6. Collision and steering

**6.1 Layers.** One spatial grid per `Domain`, cell = one tile, a unit listed in every cell its box overlaps.
Ground: hard, vs ground units and the NavGrid of its class. Underground: its own layer (G14). Air: no hard
collision; soft separation is a velocity bias inside SeparationRadius (not M1). A filter
`blocks(mover, other) -> bool` is the single place for exceptions (crushing later).

**6.2 Steering target.** The farthest path waypoint with LOS, advanced on the stagger tick; a waypoint is never
required to be touched. This keeps a group's lateral spread through corridors instead of funnelling every unit
through the same tile centres.

**6.3 One unit's step.** `step = max_speed × speed_factor`; `d` = vector to the steering target, clamped to `step`.
- **Arrival**: if the distance to the final goal is ≤ `step`, set `pos = goal` exactly and stop. The arrival
  radius is one tick of travel by construction, so overshoot is impossible. A finite-deceleration model starts
  braking at `v² / (2 · decel) + v`.
- Sweep the box along `d` and find the first blocker (earliest contact; tiles before units, then lowest id):
  1. **Nothing**: commit `d`.
  2. **Tile or structure**: slide. Try the axis components `(d.x, 0)` and `(0, d.y)`, larger first.
  3. **Unit moving the same way** (`velocity · d > 0`): follow. Advance to contact and wait; no detour.
  4. **Unit stationary or oncoming**: sidestep. Try `d` rotated 45° then 90° towards `avoid_side`, same length.
     `avoid_side` starts **Right** for everyone, so head-on units pass like traffic with no randomness. Switch side
     only if both candidates fail and `avoid_until` has passed; a switch commits for 16 ticks (no left-right flutter).
  5. **All fail**: advance to contact or stay; `blocked_ticks += 1`.
- Rotations use constants (`(x, y) → (−y, x)`, and `(x − y, x + y) × FRAC_1_SQRT_2`), no trigonometry. Facing follows
  the path direction, not the sidestep, turning by at most TurnRate; when the turn completes within the tick (all M1
  units) velocity is `d` itself. Other movement models replace only this "desired direction → velocity and facing"
  step; path following and collision stay shared.
- A unit counts as **stationary** for others when its locomotion is Stationary or `blocked_ticks ≥ 12`.
- Units with `goal = None` never move: idle units are never displaced (G2).

**6.4 Group slots (G1).** At command application, for a point target and more than one unit: anchor = the click's
tile, retargeted as in 5.3. Candidate slots spiral outward on a lattice around the click (pitch as G1); a slot is kept
if the largest member's box there is clear of blocked tiles and its tile is in the anchor's component. Take the first
`n` (examining at most `4n + 64`; leftover units get the anchor). Assign greedily: the slot farthest from the group's
centroid takes its nearest free unit, ties by id. If the Designer rejects slots, this step is dropped and 6.5 alone
handles the crowd.

**6.5 Stuck detection and recovery.** Progress = remaining path length to the goal. `last_progress` updates when it
beats `best_remaining` by half a step. Following a slower unit is progress. A unit is **stuck** after
`STUCK_TICKS` = 24 without it. Then, in order:
1. **Crowded arrival**: goal is a point within 1.5 GU and the blocker is stationary → `Arrived` (G5).
2. **Queueing**: chasing a target and blocked by stationary friendly units attacking the same target → keep waiting,
   re-check every 16 ticks; no repath, no failure.
3. **Repath** with the tiles of the stationary blockers it touched added to `avoid_tiles`, if `repaths_left > 0`.
4. Otherwise, or if that search fails → `Failed`; the behaviour stops and completes (`behaviors.md > Failure`).

An `Arrived` or `Failed` unit is Stationary and has no goal, so it cannot oscillate. Friendly and enemy units block
alike; a unit reacts to an enemy only on contact, so pathing leaks nothing about unseen units.

**6.6 If G2 or G3 come back "yes".** Yield: a mover blocked by an idle friendly unit sets a one-shot
`yield_request` on it, and that unit takes a single sidestep through the same 6.3 pipeline. Swap: two friendly
movers in mutual head-on block exchange positions. Both fit in 6.3 without restructuring; neither is built in M1.

## 7. Numbers: f32, not fixed point

**Decision: keep `f32` under the current rules, and keep every discrete decision in the pathfinder in integers.**
- Rust guarantees IEEE-754 results for `+ − × ÷` and `sqrt` at every optimisation level, never contracts to FMA
  and has no fast-math, so x86-64, aarch64 and wasm32 agree bit for bit. The remaining cross-machine risks are
  transcendentals (already routed through `libm`) and NaN payload bits (state must never hold a NaN: validated
  inputs, `normalize_or_zero`, an "all floats finite" invariant in tests).
- Precision is ample: the largest coordinate is 16 384 su, where f32 resolves 0.001 su.
- This design removes the usual float hazards: no epsilons in overlap tests (strict comparisons, assigned contact
  positions), no trig in collision, integer A\* costs and keys, `total_cmp` plus id for every ordering.
- Fixed point would mean replacing `glam::Vec2` across the sim, the client and the file formats, and writing our own
  `sqrt`/`atan2`, for a guarantee we already have. The design's fractional values (0.9 su/tick², 0.1 drag) fit both.
- Lockstep needs the same binary-level determinism replays need, nothing more. Insurance: golden hashes committed in
  tests (section 8) and, once a second architecture is available, running that one test there. Excluded: 32-bit x86
  without SSE2 (x87).
- Add `f32::mul_add` to `clippy.toml` (keeps arithmetic uniform and avoids the slow software fallback).

## 8. Test strategy

- **Nav unit tests** (ASCII maps): A\* cost equals a brute-force Dijkstra on seeded random maps; expansions ≤ passable
  tiles (closed set); a straight open-map path smooths to two waypoints (no zigzag); same call twice gives the same
  path; an unreachable goal costs 0 expansions and returns the nearest reachable tile; LOS against hand-drawn corners.
- **Invariants**, checked after every tick by a `TestSim` switch, in every scenario test:
  I1 no two units of a layer overlap; I2 no ground box overlaps a blocked tile or leaves the map;
  I3 `|pos − prev_pos| ≤ max_speed`; I4 all floats finite; I5 a unit with no goal doesn't move;
  I6 repaths per order ≤ `MAX_REPATHS`, request age bounded; I7 an `Arrived` unit never moves again without an order.
- **Scenario tests** (`crates/sim/tests/movement_*.rs`, inline ASCII maps): straight move arrives exactly at
  `ceil(d / speed)` ticks; around a wall; through a one-tile gap; two Peacekeepers pass in a 64 su corridor; two
  Guards head-on in it both end `Failed` within a bound; 20 units through a choke all arrive within
  `2 × path / speed + c`; a group arrives, every unit Stationary, and stays so for 100 ticks; an idle unit in the way
  is passed and never moved; goal on a cliff; enclosed unit; an object spawned on the path forces one repath; chase a
  moving target; three Shift-queued waypoints in order; Patrol for three cycles; Hold; Stop.
- **Oscillation**: distance travelled ≤ 1.5 × path length + c; reversals of velocity along the path ≤ a small bound.
- **Determinism**: same run twice, per-tick hashes equal; replay round trip; **save at tick k, load, continue**:
  hashes equal (catches hidden unserialized state); **golden hashes** for three scenarios as committed constants.
- **Soak**: seeded random orders, 200 units, random maps, 2 000 ticks: invariants hold, no panic.
- **Perf**: an `#[ignore]` test printing ms per tick for 500 units on 256×256, plus CI assertions on the counters.
- **QA**: `TestSim::ascii()` renders the map with units as letters for readable failures; a client path overlay
  (waypoints, slots, status colours) behind a debug key and flag; client replay playback (`--replay <file>`), so a
  failing sim test's `Replay` can be screenshotted with `--screenshot`.

## 9. M1 slicing

| Slice | Content | After | Model |
|---|---|---|---|
| **B Unit foundation** | `UnitBaseDef`/`UnitDef`, Peacekeeper and Guard, `Orders` and `Mover` types with hashing, `Command::Order` with gate and queue rules, spawn/teleport overlap checks, a naive straight-line mover, empty phase stubs 5a–5f in `step`, placeholder unit visuals | — | Opus |
| **C Navigation** | NavGrid, components, A\*, field, LOS, smoothing, request budget (`nav/`, pure and testable alone) | B | **Fable** |
| **D Steering and collision** | Spatial grid, 6.3 step, group slots, stuck and recovery, the invariant harness and movement scenarios; replaces the naive mover | C | **Fable** |
| **S Selection** | Clicks, box, control groups, Tab cycling (client) | B | Opus |
| **H Commands UI** | Command panel, right-click resolution, cursors, indicators, Shift-queue | S | Opus |
| **V Vision** | Visibility per player, ElevationModifier (sim); fog texture (client) | B | Opus |
| **W Combat** | Attack phases, hit list, damage, deaths; range checks per G7 | B | Opus |
| **E Behaviours** | Move-to-object, Attack chase and queueing, AttackMove, Patrol, Hold, Stop, idle auto-target and leashes | D, W, V | **Fable**, or Opus with a Fable review |
| **G QA and hardening** | Debug control of both sides, replay playback, path overlay, soak, perf test, golden hashes | E, H | Opus; Fable reviews movement |

- **Parallel**: after B, four tracks run at once: C → D (movement), S → H (client), V, W. E joins them.
- B creates every shared touch point (`Object` fields, `hash.rs`, the phase stubs, `Command::Order`), so the parallel
  slices each fill their own module and merges stay small.
- Before D starts, G1–G5 should be answered; C needs none of them. W needs G7, E needs G8 and G9.
