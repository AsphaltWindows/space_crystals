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
- **Slice D (steering and collision) is implemented.** Where the code differs from the first approved text, the
  text below was changed and the change is marked **(D)** with its reason. The code is `crates/sim/src/movement.rs`
  and `movement/`, `paths.rs` and `mover.rs`.
- **Group destinations follow the magic box** (design change 0021, answering G1): slice M replaced the slots of 6.4.
  Its changes are marked **(M)**. The code is `crates/sim/src/orders/destinations.rs`, `nav::reachable_point`, the
  group field of `nav::serve`, and one steering fix the new group shapes exposed (6.3, `movement/collide.rs`).
- **Friendly yielding, the give-up schedule and occupied targets follow the design** (design changes 0022–0028,
  answering G2–G5 and question Q0015): slice Y built 6.6, changed 5.6 and 6.5, and drew the magic box around the
  silhouettes (6.4). Its changes are marked **(Y)**. The code is `crates/sim/src/movement/yielding.rs`,
  `movement/stuck.rs` and `mover.rs`.
- **Shift-queued group orders follow the design** (design changes 0029 and 0030): slice Q builds the magic box of
  a queued order at the units' planned end positions, and confirmed that destinations are resolved at the click
  and kept per unit (6.4). Its changes are marked **(Q)**. The code is `crates/sim/src/orders/destinations.rs`.
- **Object goals and the behaviours are built** (design changes 0031–0034, 0036 and 0037, answering G7–G9; G6 is
  answered by 0033 and waits for the vehicles milestone): slice E gave `MoveGoal::Object` its meaning (3, 5.6, 6.5)
  and built the behaviours that set goals (`ARCHITECTURE.md > Behaviours`). Its changes are marked **(E)**. The code
  is `crates/sim/src/orders/`, `shape.rs`, `paths.rs` (`chase_check`), `movement.rs` and `movement/stuck.rs`.
- **Crowds and nav performance are hardened** (slice G1; no design rule changed): units seek lanes among the units
  they have to pass (6.3), an idle group parts for a group that walks through it instead of being shoved along
  (6.6), and a group's field is built only as far as the group stands (5.5). Its changes are marked **(G)**. The
  code is `crates/sim/src/movement/collide/lanes.rs` (`World::lane`), `movement/yielding.rs` (`ask_ahead`),
  `movement/space.rs` (the cells' marks), `nav/search.rs` (`Search::field_to`) and `nav/serve.rs`
  (`smooth_shared`).

Design references are `file.md > Section`, relative to `artifacts/designer/design/`.
Every number marked *(placeholder)* becomes a named const tagged with the question that covers it (section 2).

## 1. Requirements from the design

| Topic | Rule | Source |
|---|---|---|
| Time, space | 16 ticks/s; speeds in su/tick; 64 su = 1 tile | `core/scale.md` |
| M1 units | Peacekeeper: LightInfantry, 24×24 su, 4 su/tick (2 with Riot Shield up), Range 4 GU, Sight 5 GU. Guard: HeavyInfantry, 36×36, 5 su/tick, Range 3 GU. Both: TurnRate 180°/tick, infinite acceleration and deceleration, so M1 kinematics are instant | `factions/gdo/units.md > Peacekeeper`, `factions/syndicate/units.md > Guard` |
| Later range | Silhouettes 24–60 su so far (vehicles TBD, may exceed a tile); speeds 4–8 su/tick; five movement models (TurnRate, FixedTurnRadius, SpeedTurnRadius, Drag, Glider) | `systems/units.md > MovementModel` ff., faction unit files |
| Domains | Ground, Air, Underground per UnitBase | `systems/units.md > UnitBase` |
| Ground collision | Hard, never overlapping; shape is the Silhouette rectangle; **(Y) friendly yielding**: an idle unit takes one short sidestep for a blocked moving unit of its owner (not on Hold, not busy, not if it can't move, not for its own group order); enemies never yield, and moving units never yield or swap; moving units path around occupied space; ground units collide with ground units and structures | `systems/units.md > UnitCollision > Ground Collision` (0022, 0023, 0027) |
| Air collision | None with ground or structures; soft separation among air units (SeparationRadius) | `> Air Collision` |
| Crushing | Moving TrackedVehicle/Mech pass through and kill enemy LightInfantry; not friendly; stationary crushers are solid | `> Crushing` (not M1, but the collision filter must allow it) |
| Sub-tile lanes | A 64 su gap fits two 24 su units side by side, one lane each way | `factions/cults/units.md > Recruit` (throughput note) |
| Terrain | Non-Traversible tiles block ground units. RuggedTerrain = true bases (infantry, Mech) cross Rugged tiles. Drill units travel on Drillable tiles | `core/map.md > Tile`, `systems/units.md > UnitBase`, `> DrillUnit` |
| Elevation | Slopes change neither speed nor path cost; cliffs are non-Traversible tiles; corners are shared, so two adjacent Traversible tiles are always connected | `core/map.md > Elevation` |
| Underground | Expansions are walked over by surface units | `factions/syndicate/structures.md > Tunnel Expansions` |
| Move | To a location or an object; ends with Stopping, complete when stopped; completes "within proximity" of an object; on repeated failure to progress: stop, complete. **(Y)** Stuck after 24 ticks without getting closer, 3 re-route attempts after waits of 8, 16 and 32 ticks, then give up (0024); an unreachable target becomes the nearest reachable point (0025); stuck within 2 GU of the target behind units that aren't moving is arrived (0026) | `systems/controls/behaviors.md > MovingToLocation`, `> MovingToObject` |
| Chase | AttackingObject approaches until the target is within Range, beyond MinRange and in facing; resumes when it leaves range. Infantry (UnitBaseSource) cannot move during Aiming, Firing, Cooldown | `> AttackingObject`, `systems/combat.md > AttackPhase` |
| Range | ±1 GU by elevation level difference, ground and underground only | `core/vision.md > ElevationModifier` |
| AttackMove, Patrol | Leash 6 GU perpendicular to the path, recompute the path on resuming; Patrol swaps legs forever | `> AttackMovingToLocation`, `> Patrolling` |
| Idle | Auto-acquire in SightRange, leash 4 GU from IdleOrigin, return to it | `> BaseAutoTargeting` |
| Hold, Stop | Hold: Locomotion Stationary always, never completes. Stop: Stopping, then no behaviour | `> HoldingPosition`, `> StoppingBehavior` |
| Queue | Shift appends, otherwise the command replaces the whole queue; next order starts when the current behaviour completes | `systems/controls/commands.md > Unit Command` |
| Channels | Locomotion (Moving, Reversing, Stopping, Stationary) and Orientation (Turning, Maintaining) are named state; combat and crushing read them | `behaviors.md > BaseActionChannels`, `combat.md > LocomotionOrientationConstraints` |
| Group moves | **(M)** The magic box: a location order (Move, AttackMove, Patrol) to several units keeps their arrangement when the click is outside the bounding box of their silhouettes (**(Y)**, 0028; each goes to the click plus its offset from the group's centre) and converges on the click when it is inside. No formation movement. Selection has no size limit | `systems/controls/commands.md > Group Destinations (Magic Box)`, `systems/controls/selection_and_hud.md` |

## 2. Design gaps and contradictions

Top five first. Each default is what this design implements until answered.

| # | Question | Proposed default |
|---|---|---|
| G1 | Several units are ordered to one point and cannot all stand on it. Where does each end up? | **(M) Answered by 0021** (Q0007, part 1): the magic box, 6.4. The default proposed here, a compact lattice of slots around the point, was rejected. Units move at their own speed, not the slowest's |
| G2 | **Contradiction with the Director's brief**, which assumes idle units make way. `units.md > Ground Collision` says they don't. Confirm: no pushing or yielding at all, even for own units blocking a chokepoint? | **(Y) Answered by 0022, 0023 and 0027**: idle own units yield with one short sidestep (6.6). The default here was "no pushing" |
| G3 | With hard collision and no yielding, two units meeting head-on in a corridor too narrow to pass (two Guards, 72 su, in one tile) deadlock. Is "both give up and stop" intended? | **(Y) Answered by 0022**: yes, both give up; moving units never yield or swap |
| G4 | "Repeatedly fails to make progress" has no numbers. | **(Y) Answered by 0024** with these numbers: stuck after 24 ticks without progress; at most 3 repaths per order after waits of 8, 16 and 32 ticks; then fail (about 7 s in total). The schedule is 5.6 |
| G5 | Move target that cannot be reached (cliff, water, inside a structure, cut-off area), or whose point is occupied by a unit that won't move. | **(Y) Answered by 0025 and 0026**: go to the reachable point nearest the target and complete there; arrived when stuck within **2 GU** of the goal behind units that aren't moving (6.5, rule 1) |
| G6 | Does the Silhouette rectangle rotate with facing? All specified ones are square. | **(E) Answered by 0033**: it rotates with the facing. Not built: it waits for the vehicles milestone, with oriented collision. Until then boxes are axis-aligned, and `shape.rs` is the one place that changes |
| G7 | How are Range and "within proximity" measured for large targets? | **(E) Answered by 0031 and 0032**: edge to edge, the gap between the two shapes (`shape::Shape::gap`), for Range, MinRange and proximity; proximity is a gap of 8 su. The default here was centre to nearest point |
| G8 | The attack target leaves the attacker's owner's vision during a chase. | **(E) Answered by 0034**, as proposed: go to its last seen position, then the order completes. A Follow of an enemy ends the same way |
| G9 | Patrol: is the origin the position at issue or when a Shift-queued Patrol starts (`commands.md > Patrol` vs `behaviors.md > Patrolling` step 1)? What if a leg fails? | **(E) Answered by 0036 and 0037**, as proposed: when it starts executing. A failed leg counts as arrived; two failed legs in a row end the Patrol |
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
pub struct Orders { queue: VecDeque<Order> /* cap MAX_QUEUED_ORDERS */, current: Option<ActiveOrder>,
                    auto: Auto /* (E) what the unit does on its own without an order */ }
pub struct ActiveOrder { order: Order, since: Tick, state: Behaviour /* (E) the behaviour's working state */ }
pub struct Order { kind: OrderKind, group: OrderGroup /* units of one command share it */,
                   slot: Option<Vec2> /* (M) this unit's own destination by the magic box (6.4); None: the click */ }
pub enum OrderKind { Move(Target), Attack(ObjectId), AttackMove(Vec2), Patrol(Vec2), Hold, Stop }
pub enum Target { Point(Vec2), Object(ObjectId) }  // the point is the click

// mover.rs: goal in, motion out. Behaviours write `goal` and read `status`; nothing else moves a unit.
pub struct Mover {
    goal: MoveGoal,            // None | Point(Vec2) | Object { id, reach: f32 } (E): reach is a gap between shapes
    status: MoveStatus,        // Idle | Moving | Arrived | Failed | TargetLost
    locomotion: Locomotion, orientation: Orientation,   // the design's channels
    velocity: Vec2, speed_factor: f32,                  // Riot Shield: 0.5
    group: Option<u32>,                                 // (D) the order group of this goal, kept after it ends (6.5)
    path: Vec<Vec2>, next: u16, path_state: PathState,  // PathState: None | Wanted { since: Tick } | Ready
    nav_version: u32, planned_target: Vec2,             // staleness checks
    repaths_left: u8, last_repath: Tick /* (Y) */, avoid_tiles: Vec<TilePos> /* cap 16 */,
    best_remaining: f32, last_progress: Tick, blocked_ticks: u16,
    avoid_side: Side, avoid_until: Tick,
    sidestep: Option<Sidestep>,                         // (Y) the sidestep of a yielding unit (6.6)
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

Phases 1–4 stay as they are. Order commands (phase 4) only write `Orders` queues and compute group destinations
(6.4); they never move anything. Then, each walking units in id order over `..first_new`:

| Phase | Does | Why here |
|---|---|---|
| 5a `orders::update` | Pops completed orders, runs the behaviour state machine, writes `mover.goal`, reads `mover.status` and last tick's positions and visibility. **(E)** Every behaviour reads one snapshot of the objects, taken before any of them runs (`orders::world::World`) | After commands, so an order acts in its own tick; before movement, so there is one writer of goals |
| 5b `nav::sync` | If a tile-blocking object or the map changed: rebuild the NavGrids and components, bump `nav_version` | Paths below must see this tick's obstacles |
| 5c `paths::serve` | Runs the staggered path checks (5.6, 6.2), then serves `Wanted` requests within the budget (5.5) | After goals, before steering; a repath a check asks for is served in the same tick |
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
  searches, collision queries) are output outside the hash, for tests and the perf overlay (`Sim::nav_counters`,
  `Sim::move_counters`).

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
- **Avoid tiles**: a repath treats the unit's `avoid_tiles` as blocked (6.5). Only this search can fail. **(D)**
  When it fails, the unit drops its avoid tiles and plans again in the next tick instead of failing its move: a
  tile is 64 su and a unit 24, so avoid tiles overstate what is blocked and can't prove that there is no way. The
  smoothed path and the steering target also stay out of the avoid tiles (`nav::smooth_avoiding`,
  `nav::segment_avoids`); string-pulling by terrain alone pulled every detour straight again.

**5.4 Smoothing and line of sight.** `nav::los(class, half, a, b)`: walk the tiles the segment crosses
(Amanatides–Woo) and slab-test the segment against each blocked tile in their 3×3 neighbourhoods, inflated by `half`.
Exact for boxes up to one tile. Smoothing is greedy string-pulling: from an anchor, keep the farthest tile centre
with LOS. Waypoints are tile centres, plus the exact start and goal, which leaves `32 − half` su of corner clearance.

**5.5 Requests and budget.** `PathState::Wanted { since }` on the unit; each tick 5c sorts requests by `(since, id)`
and serves them until `PATH_BUDGET` = 32 000 node expansions is spent *(tuning const)*. A job starts while budget
remains and always finishes, so every request is served within bounded time. Units waiting for a path stand still.
- **Group orders**: when at least `GROUP_FIELD_MIN` = 4 requests share an `OrderGroup`, nav class and component, run
  **one** Dijkstra from the group's anchor tile (same code, `h = 0`), and give each member the parent chain from its
  own tile, smoothed, with its own destination (6.4) as the last waypoint. The field is scratch, dropped in the same
  tick. **(M)** The anchor is the click's tile, and only requests whose goal the anchor tile's centre sees (for the
  unit's box) count and share the field; the others plan alone with A\*. With the magic box a destination is the
  unit's place in the group's shape, which can lie behind an obstacle from the click (on the near side of a wall
  the click is beyond, say), and the way through the anchor would be a detour. A converging group has every goal on
  the anchor's tile, and a group moved over open ground sees all of its goals, so both still cost one field.
- **(G) The field reaches only as far as the group stands** (`Search::field_to`). The search from the anchor heads
  for the bounding rectangle of the members' tiles (the heuristic is the octile distance to it, which is
  consistent, so an expanded tile has its final cost and parent) and stops when every member's tile is expanded.
  The paths cost what the whole field's would. A group standing together pays for the tiles between itself and
  the anchor, not for the map: the two 250-unit orders of the perf test cost 9 000 expansions each, where the whole
  field was 62 000, and both now fit into one tick's budget. Units of an AttackMove group that resume together
  after a fight no longer cost a field over the whole map (4.2 ms in a 250 v 250 battle; now 0.7 ms at worst).
  Members scattered all over the map still cost the whole field, the bound of 5.8.
- **(G) The members share their smoothing** (`nav::serve::smooth_shared`). The paths of a field are chains of one
  tree, so from a tile on two members walk the same tiles, and the waypoint that follows a tile's centre depends
  on that chain alone while the member's own goal is out of sight. It is found once per tile for the whole group.
  The waypoints are exactly those of `smooth`.
- The budget counts expansions, never time, so it is deterministic. **(G)** Its semantics are unchanged: a job
  starts while budget remains and always finishes, and a field is one job.
- **(G) Still open**: a member whose goal the anchor can't see plans alone, within the budget. A group ordered into
  cluttered ground, where the anchor sees few of the goals, is therefore pathed over several ticks, not in the
  tick of the order (the last case `nav_perf` prints: 4 of 200 served in the first tick on its bad-case map).

**5.6 Repath policy.** Per order leg, `repaths_left = MAX_REPATHS` (3, design change 0024). **(Y)** Every counted
repath waits `REPATH_WAITS` (8, 16, 32 ticks) since the unit became stuck or since the repath before
(`Mover::last_repath`), whichever is later.

| Trigger | Check | Counts against the cap |
|---|---|---|
| Stuck (6.5) | every tick, cheap | yes; with none left, `Failed` 24 ticks after the last one |
| `nav_version` changed | once, on the unit's stagger tick: LOS along the remaining path; repath only if a segment is now blocked | no (bounded by how often the map changes) |
| Chase target moved **(E)** | on the stagger tick (`paths::chase_check`): with a clear line for its box, the unit's path becomes the one leg straight to the target, no search; without one, a new plan once the target is farther than `max(1 GU, 25 % of the distance)` from `planned_target`, at most every 8 ticks | no (rate-limited) |
| Lost LOS to the steering waypoint | on the stagger tick, after being pushed off the path by sidesteps, **(Y)** and only when a stuck repath is due (the schedule below; slice D had 12 ticks without progress) | yes |

Stagger tick: `(tick + id) % 4 == 0`. While a chaser has LOS to its target it steers straight at it with no path.

**(E) Object goals.** A path to an object is planned to the point nearest its centre that the unit can stand on
(`nav::reachable_point`, as for an unreachable point, 6.4): for a structure or a map resource, whose own tiles are
blocked, that is right at its wall, within proximity. `planned_target` notes where the object's centre was. The unit
has `Arrived` when the gap between its shape and the object's is within the goal's `reach` (checked before it
moves); at the end of its path if the object is still where the path was planned to (the nearest point it can
reach: design change 0025); or by rule 1 of 6.5. At the end of a path to an object that has moved on, it plans
again (not counted). An object that is gone ends the move `TargetLost`.

**(Y)** Timing, as design change 0024 words it (`systems/controls/behaviors.md > MovingToLocation > Failure`). A
unit is **stuck** when 24 ticks have passed without progress (`Mover::is_stuck`; nothing restarts that clock but
progress). A stuck unit waits 8 ticks and tries another route, waits 16 and tries again, waits 32 and tries a third
time; if it is still stuck 24 ticks after the third attempt, it gives up (`Mover::recovery_due`). So a unit that
never gets on repaths 32, 48 and 80 ticks after its last progress and fails at 104 ticks (`GIVE_UP_TICKS`, 6.5 s:
the design's "about 7 s"). A unit that gets on after a repath is stuck again 24 ticks after its last progress, and
its next wait runs from then. Slice D had repaths at 24, 48 and 72 ticks, each restarting the stuck clock, with the
same 104. A unit blocked for less than 24 ticks never repaths. A unit that lost sight of its waypoint but is still
getting on slides back by itself and keeps its repaths.

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
**Measured after slice D** (`crates/sim/tests/movement_perf.rs`, release, two armies of 250 crossing a 256×256 map):
the whole sim tick takes 0.19 ms on average and 0.64 ms at worst with 450 or more units moving (2.7 sweeps and 5.9
box tests per unit and tick); the tick of a 250-unit group order (one field, 62 000 expansions, 250 smoothed paths)
takes 6 to 10 ms.
**(G) Measured after slice G1** (the same test, and `behaviours_perf.rs`; before → after):

| Case | Before | After |
|---|---|---|
| Group orders of the two armies of 250 | 7.1 ms and 6.1 ms, in two ticks (62 000 expansions each) | 5.9 ms, both in one tick (18 700 expansions) |
| Whole tick, 450 or more of 500 units moving, the armies crossing | 0.365 ms mean, 0.70 ms worst | 0.40 ms mean, 0.79 ms worst |
| 500 units attack-moving, up to the first shot | 0.35 ms mean, 0.41 ms worst | 0.33 ms mean, 0.38 ms worst |
| 250 v 250 AttackMove battle | 0.48 ms mean, 4.2 ms worst (65 700 expansions) | 0.28 ms mean, 0.67 ms worst (940 expansions) |
| The winners going on | 3.5 ms worst | 0.07 ms worst |

Lane seeking (6.3) costs the crossing armies 10 % of their tick: each unit looks through the cells ahead of it for
units coming its way. The cells' marks (6.1) keep that to one look at a few cells for a unit that has nobody to
pass, and the grid's hot path no longer calls `floor`.

## 6. Collision and steering

**6.1 Layers.** One spatial grid per `Domain`, cell = one tile, a unit listed in every cell its box overlaps.
Ground: hard, vs ground units and the NavGrid of its class. Underground: its own layer (G14). Air: no hard
collision; soft separation is a velocity bias inside SeparationRadius (not M1). A filter
`blocks(mover, other) -> bool` is the single place for exceptions (crushing later).
**(G)** A cell also notes the **kinds** of unit listed in it since the grid was built: the 45° sector of each
one's heading, and whether it yields (`Body::kinds`, `Layer::near_any`). A mark is never taken back within the
tick, so it means "maybe": lane seeking (6.3) asks the marks first and looks at the bodies only where a unit
coming its way, or an idle unit that would make way, may be. And `NavGrid::open_around` says whether a tile and
its eight neighbours are free of solid tiles: there a unit's box and its step are clear of the tiles without a
look at them.

**6.2 Steering target.** The farthest path waypoint with LOS, advanced on the stagger tick; a waypoint is never
required to be touched. This keeps a group's lateral spread through corridors instead of funnelling every unit
through the same tile centres. **(D)** The stagger tick scans forward from the current waypoint (at most 8), and
every tick a waypoint within one step is passed, so a unit rounds a corner without stopping on it.

**6.3 One unit's step.** `step = max_speed × speed_factor`; `d` = vector to the steering target, clamped to `step`.
- **Arrival**: if the distance to the final goal is ≤ `step`, set `pos = goal` exactly and stop. The arrival
  radius is one tick of travel by construction, so overshoot is impossible. A finite-deceleration model starts
  braking at `v² / (2 · decel) + v`.
- **(G) Lane seeking** (`World::lane`, `World::lane_step`), before the sweep. Two dense groups that met head-on
  jammed: 14 of 36 Peacekeepers in blocks 40 su apart gave up, 18 with two owners. Both groups are moving, so
  nobody yields (design change 0022), and the rules below only act on contact, one blocker at a time. Now a unit
  looks ahead for the units it has to pass and steps towards a lane between them, as pedestrians do:
  - **Whom it has to pass**: units within 4 of its own lengths ahead (`LANE_LOOK`; 96 su for a Peacekeeper) and
    nearer than the end of its way, that are **oncoming** (heading against its own by more than 120°, by where
    they are going, `Body::heading`, not by their latest step; moving or jammed), or **idle units that yield to
    it** (6.6). The object it heads for doesn't count, nor do units beyond its goal: units converging on one
    point aren't in each other's way.
  - **Lanes**: each such unit bars a range of sideways shifts of the mover, those at which the two boxes would
    meet, plus `LANE_MARGIN_SU` = 2 su of clearance. With the lanes that hang together around the unit:
    a **clear lane** within 3 of its widths (`LANE_REACH`) is taken, the nearer side, and on a tie (4 su) its
    `avoid_side`, so head-on units still pass on the Right. With **none within reach** (a dense group ahead) it
    takes the middle of the widest gap between two neighbouring lanes, a nearer gap before a slightly wider one
    (`LANE_NEAR`).
  - **The step**: forward and sideways at once, at most 45° off its way; if that isn't free, sideways alone,
    which counts as blocked like a step along a face. If neither is free, the rules below apply. The side taken
    becomes its `avoid_side`.
  - **Why dense groups get through**: the units of both groups aim at the gaps of the other, so the groups mesh
    like combs. The gaps are too narrow at first (16 su for 24 su units), but the units on the outside find a
    clear lane and move out, the gaps next to them widen, their neighbours centre themselves again, and the comb
    spreads until every lane is clear. Collision stays hard throughout: every step is committed through
    `World::is_free` like any other.
  - **Deterministic**: the lanes are sorted by their bounds, then by body index (bodies are in id order); the
    cells are visited in a fixed order; every decision compares `f32`s made with `+ − ×` only (the heading test
    is in squares, without a root); nothing is kept between ticks except `avoid_side`, which was state already.
  - **One idle unit alone** in the way is no lane: the unit walks on and asks it to step aside (6.6), as before.
  - **Results** (`movement_crowds.rs`): 18 v 18 Peacekeepers 40 su apart: 0 of 36 fail
    (14 before; 18 with two owners). Over 192 layouts of two blocks (Peacekeepers and Guards, three shapes, gaps
    of 8 to 24 su, four offsets, one and two owners) 1.4 % of the units fail, 8 of 50 at worst, where 22 % did,
    and 40 of 50 at worst. The soak's failed moves fall from 223 to 81 of about 1 080.
  - **Known limits**: Guards in deep square blocks (5 × 5) still lose a few units. In a passage too narrow for
    two lanes nothing changes: head-on movers fail there, by design.
- Sweep the box along `d` and find the first blocker (earliest contact; tiles before units, then lowest id):
  1. **Nothing**: commit `d`.
  2. **Tile or structure**: slide. **(D)** Both axis components `(d.x, 0)` and `(0, d.y)`, the larger first, each
     as far as tiles and units allow (the usual per-axis resolution; "try one, then the other" lost the second).
  3. **Unit moving the same way** (`velocity · d > 0`): follow. Advance to contact and wait; no detour.
  4. **Unit stationary or oncoming**: go around it. **(D)** Boxes are axis-aligned, so "sideways" is along the
     blocker's face: `d` turned 90° leans into the blocker whenever `d` isn't square to the face, and is then never
     free. `d` splits into a part straight at the face and a part along it, its lean:
     - *Glance* (the lean is at least half of `d`, squared): slide along the face, the lean alone.
     - *Leaning* (the lean is at least 30 % of `d`): go around on the side the unit leans to: `d` turned 45° that
       way, else a whole step along the face that way. Never a step back along the face. **(M)** The side taken
       becomes `avoid_side`. The unit's own step along the face can drop its lean below 30 %, and head-on with
       the other `avoid_side` then stepped it straight back, for good: a unit pressed against an idle Guard by the
       crossing lanes of two magic-box groups failed so.
     - *Head-on*: `d` turned 45° towards `avoid_side`, else a whole step along the face that way. `avoid_side`
       starts **Right** for everyone on every leg, so head-on units pass like traffic with no randomness. Switch
       side only if both candidates fail and `avoid_until` has passed; a switch commits for 16 ticks (no left-right
       flutter). A blocker that itself moves along its face is passed *behind*: once `avoid_until` has passed, the
       side switches to the one the blocker comes from. Without this an eastbound unit and a crossing or
       side-stepping one walked side by side to the map's edge.
     Every candidate is taken only if it is wholly free.
  5. **All fail**: advance to contact or stay; `blocked_ticks += 1`. **(D)** A whole step along a face (no
     progress) counts as blocked too, so units dancing in a corridor count as stationary after 12 ticks.
- Rotations use constants (`(x, y) → (−y, x)`, and `(x − y, x + y) × FRAC_1_SQRT_2`), no trigonometry. Facing follows
  the path direction, not the sidestep, turning by at most TurnRate; when the turn completes within the tick (all M1
  units) velocity is `d` itself. Other movement models replace only this "desired direction → velocity and facing"
  step; path following and collision stay shared.
- A unit counts as **stationary** for others when its locomotion is Stationary or `blocked_ticks ≥ 12`.
- **(Y)** A unit with `goal = None` moves only by its own sidestep for a blocked unit of its owner (6.6); nothing
  pushes it.
- **(D) No overlap by construction, in floats.** A position is committed only if it passes the overlap predicates
  of I1 and I2 against the tiles and every unit where it stands now (`World::is_free`); a candidate that fails by a
  rounding corner case is dropped and the unit stays. The sweep works on the same differences as the predicate
  (`a − b` against `half_a + half_b`), and an assigned contact that rounds to the inside is moved away float by
  float (`space::resting`).

**6.4 Group destinations: the magic box (G1).** **(M)** Design change 0021 replaced the slots first approved here
(`systems/controls/commands.md > Group Destinations (Magic Box)`). At command application (phase 4), a location
order (Move to a point, AttackMove, Patrol) to more than one unit gives each unit its own destination
(`Order::slot`, `orders/destinations.rs`):
- **(Y)** The **box** is the bounding rectangle of the units' full Silhouettes, their collision boxes (design
  change 0028; slice M took their centres). The **group's centre** is the box's centre, so a group arrives centred
  on the click.
- **Click inside the box, edges included: converge.** Every unit goes to the click itself (`slot = None`), and those
  the crowd blocks stop by 6.5 rule 1: **(Y)** within 2 GU of the click behind units that won't move, which the
  members that arrived before them are (they don't yield to their own group order, 6.6). A unit shut out farther
  than that re-routes and gives up. A group in a line has a box as high as its units, so a click on or between
  them converges it.
- **Click outside: keep the arrangement.** Every unit goes to its position plus the vector from the box's centre to
  the click, so the group arrives in the shape it had.
- **No formation movement**: each unit goes at its own speed, and a mixed group strings out.
- **(Q) Resolved at the click** (design change 0030): the destinations are computed once, as the command applies,
  queued or not, and each unit keeps its own in its order (`Order::slot`). A unit starts a queued leg alone, when
  its earlier orders are done, and nothing recomputes its destination then: as a group of one it would converge on
  the click. So the shape appears at every destination, a unit that falls short of a spot still heads for its own
  spot on the next leg, and a member that dies on the way moves nobody's spot. Slice M already worked this way.
- **(Q) Shift-queued orders: the box at the planned ends** (design change 0029, `destinations::planned_end`). For a
  queued order, each unit's position, for the box and for its offset, is where its earlier orders leave it: the
  destination of its last order that moves it, the current one included, if that order ends at a known location
  (a Move to a point or an AttackMove: its own spot, or the click it converges on). Stop and Hold move nobody, so
  the order before them decides. If no order moves the unit, or the last one ends somewhere unknown (an Attack or
  a Move to an object; a Patrol, which never completes), its current position stands in, 0029's fallback. The
  silhouettes are put at these positions (0028). So "far north, then back" brings a line back as the line; and
  after a queued converge the box is one silhouette around the converge point, so the next queued click outside
  it sends every unit to that click, and the blob moves on as a blob. Slice M took the current positions for
  queued orders too. A destination is still made reachable from the unit's current position (its component).
- **Unreachable destinations** (**(Y)** design change 0025 confirms this: the reachable point nearest the target): a
  destination beyond the map's edge is clamped so that the unit's box lies inside the map. A destination the unit
  can't stand on (a blocked tile, a box not clear of one, another component) moves to the nearest point it can stand
  on (`nav::reachable_point`): in rings of tiles outward from it, the point of each tile of the unit's component
  nearest to it where the box is clear, ties by tile index; exact for boxes up to a tile wide. That is 5.3's
  retargeting with points for tile centres, so a group that meets a cliff lines up along its edge instead of piling
  onto tile centres. It never searches. Air units are only clamped to the map.
- A single unit has no box: it goes to the click (`slot = None`), as before.
- **Paths**: the group shares one field anchored on the click's tile; a member whose destination the anchor can't
  see plans alone (5.5).
- The box has no size limit (TBD in the design): a widely scattered selection mostly converges.

**6.5 Stuck detection and recovery.** Progress = remaining path length to the goal. `last_progress` updates when it
beats `best_remaining` by half a step. Following a slower unit is progress. **(E)** Towards an object that is
moving (a unit that moved this tick or the last), progress is moving forward at all: the object may be as fast as
the unit, and the path is re-aimed as it moves, so its length says nothing; towards an object at rest it is the
remaining path length, measured afresh from when it came to rest. A unit is **stuck** after
`STUCK_TICKS` = 24 without it. Then, in order:
1. **(Y) Occupied target** (design change 0026, `systems/controls/behaviors.md > MovingToLocation > Occupied target
   location`): the goal is a point within `OCCUPIED_ARRIVAL_GU` = 2 GU, and the unit in the way is stationary and
   won't move for this unit → `Arrived`, with no repath. **Yielding comes first**: a blocker that yields to this
   unit (6.6) has been asked to step aside, so the rule is for blockers that stay: units that arrived with its own
   group order, units on Hold, busy units, enemies, units that can't move, and a yielding unit that found no room.
   **Timing**: this rule alone doesn't wait for the stuck clock when the blocker has no goal of its own (it will
   never move on): the unit has arrived once it found no way forward for `JAMMED_TICKS` = 12 in a row. Behind a
   stationary unit that has a goal (jammed for now) it applies when the unit is stuck, 8 ticks before the first
   repath. Slice D's stand-in "arrived at any distance behind a unit of its own order group, after a repath" is
   gone: a unit shut out farther than 2 GU from its destination re-routes and fails. **(E)** For an object goal the
   rule is the same, the 2 GU measured as the gap between the unit's shape and the object's (design changes 0032
   and 0033, "Blocked near the target"). What `Arrived` then means is the behaviour's: a static target completes
   the order, a followed unit is waited for where the follower stands, until it moves.
2. **Queueing (E)**: a unit chasing the target its attack engages, blocked by a stationary unit of its owner whose
   attack engages the same target → keep waiting; no repath, no failure (`movement::stuck::queues_behind`). The
   check is a few comparisons, so it runs every tick the unit is stuck instead of every 16.
3. **Repath**, if `repaths_left > 0`, when its wait is over (5.6). **(D)** The tile of the stationary unit in the way is added to `avoid_tiles`:
   at once if that unit has no goal; if it is on its way itself (only jammed for now), from the second repath on,
   the first being a plain retry. If the blocker stands in the unit's own tile (the start tile is never avoided),
   the neighbouring tile on the blocker's side is taken; the goal's tile is never taken.
4. Otherwise, 24 ticks after the last repath → `Failed`; the behaviour stops and completes (`behaviors.md >
   Failure`).

**(D)** Three refinements, each from a failing scenario:
- **Waiting behind a unit that is getting on is progress.** A unit blocked by one that has a goal takes over that
  unit's `last_progress` if it is later. A queue at a choke isn't stuck while its head moves; units that block each
  other for good share a clock that stands still. Without it, 20 units at a one-tile choke burnt their repaths
  waiting their turn and failed.
- **A new path is no progress.** Its first measure only sets `best_remaining`; **(Y)** and nothing else restarts
  the stuck clock (5.6).
- **Grace.** A unit due for a repath or for giving up that touches no unit in that tick (it just stepped around
  one) waits up to 8 ticks to meet its blocker again, so that rules 1 and 3 can see it.

An `Arrived` or `Failed` unit is Stationary and has no goal, so it cannot oscillate; **(Y)** it moves again only by
a sidestep (6.6). A unit reacts to an enemy only on contact, so pathing leaks nothing about unseen units.

**6.6 Friendly yielding. (Y)** Design changes 0022, 0023 and 0027 (`systems/units.md > UnitCollision > Ground
Collision > Friendly yielding`); the code is `movement/yielding.rs`. A moving unit blocked by an **idle** unit of
its own owner asks it to make way, and the idle unit takes **one short sidestep**. Collision stays hard: the
sidestep is a real move, committed through the same predicates as every other (`World::is_free`), in the unit's
own turn of phase 5d.
- **Who yields** (`can_yield`): a unit with an owner and **no active command** (no current order, none queued; Hold
  is a command, so units on Hold never yield), without a goal, not in an attack sequence (Aiming, Firing or
  Cooldown; `AttackState::phase`), and able to move in its current mode (`Mover::can_move`: modes that only slow
  it, such as a raised Riot Shield, don't matter, and it sidesteps at that speed; **the hook** for a deployed or
  deploying Artillery Rig, which will answer `false`). Enemies and nobody's units never yield. Moving units never
  yield or swap, friendly or not: two of them head-on in a passage too narrow to pass both still end `Failed`.
- **Not within the same group order** (0027, `yields_to`): a unit that `Arrived` under a group order doesn't yield
  to members of that order that are still arriving (`Mover::group`; a later order of either clears it). They stop
  against it by 6.5 rule 1. To every other unit of its owner it yields.
- **Requests** (`ask`, `apply`): the mover asks the first unit that blocked its way this tick (`Moved::blocker`),
  whatever it then did about it. Requests are collected while the units move, in id order, and applied when all
  have moved: the first one a unit gets wins (the lowest id that asked), and it sets `Mover::sidestep`, which is
  hashed and serialized state. The sidestep begins in the next tick. The spawn rule and the phase order are
  unchanged.
- **The sidestep** (`Sidestep`, `sidestep`): the unit steps at 45° to the mover's way, half across it and half
  along it, so it leaves the lane and the mover at once. Across the way it goes to the side of the mover's line it
  already stands on; within `YIELD_CENTRED_SU` = 4 su of that line, to the side the mover isn't going round on, so
  the two never chase each other. It goes as far as takes its box out of the mover's lane, at most
  `YIELD_SIDESTEP_WIDTHS` = 1 of its own width *(placeholder, TBD in the design)* and at least 1 su, at its current
  speed, without turning (every M1 unit turns in place; other movement models will need their own). It moves the
  whole step if that is free, else each axis as far as tiles and units allow. It ends when the distance is done or
  after the time the distance takes plus `YIELD_WAIT_TICKS` = 4, and the unit **stays where it got to**: there is
  no way back. A mover still blocked then asks again.
- **Dense idle groups**: a sidestep that another idle unit of the owner blocks passes the request on to it, the
  same way and for the distance still to go, at most `YIELD_CHAIN_DEPTH` = 4 units deep (one tick per link). A unit
  that finds no room stands, counts as stationary, and is an ordinary blocker to the mover (6.5). Chaining was
  chosen over "blocked means ordinary blocker" because a unit can't walk through a tight group of its own without
  it; the depth bounds the cost.
- **(G) Walking through a group.** A group walking through an idle group as wide as itself shoved it ahead, a few
  widths at a time: its units ended 204 su from their places on average and up to 373 su. Three changes:
  - The movers **aim between the idle units** (lane seeking, 6.3): a group of idle units that would yield is a
    row of lanes like an oncoming group, and each mover takes the middle of a gap, or walks around the group if a
    clear lane is within 4/3 of its width (`LANE_AROUND_IDLE`).
  - A unit that is only **clipped** (the mover's line passes beside its box), or that stands in the way of a mover
    walking in such a lane, steps **straight across** the mover's way, with no step along it, as far as its box
    reaches into the mover's lane. The step along the way is what did the shoving. A unit alone and squarely in
    the way still takes the 45° step.
  - The mover asks the idle units it is about to be blocked by, **within two steps**, both sides of its gap at
    once (`yielding::ask_ahead`), instead of one after the other as each stops it. It is still the blocked mover
    that asks, a moment early; a request is applied like any other, the lowest id first.

  Now the idle block parts: its units end 16 su from their places on average and at most 33 su, and all 18 movers
  arrive (16 before). One unit walks through a 5 × 5 block 4 su apart as fast as before (189 ticks; 186 before) and
  moves its units 16 su at most (68 before).
- **Known limits.** A sidestep straight across the mover's way was tried first, by slice Y: a line of movers
  facing an idle group as wide as itself then had nowhere to send it (each unit stepped into its neighbour's lane),
  and a third of them gave up. **(G)** With the movers in the gaps it works: a unit needs only a little to its
  side. An idle unit plugging a passage as narrow as itself has nowhere to step and stays an ordinary blocker.
  Swapping of two moving units is not built (0022 rules it out).

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
  **(Y)** I5 and I7 allow the one exception of 6.6: a unit without a goal may move while it has a `Mover::sidestep`,
  and only a unit without a goal and without an order may have one.
  **(D)** The switch (`TestSim::check_invariants`) is on by default, so every test that uses a `TestSim` runs them;
  `sc_sim::testing::invariants` has the checker for tests that step a bare `Sim`.
- **Scenario tests** (`crates/sim/tests/movement_*.rs`, inline ASCII maps): straight move arrives exactly at
  `ceil(d / speed)` ticks; around a wall; through a one-tile gap; two Peacekeepers pass in a 64 su corridor; two
  Guards head-on in it both end `Failed` within a bound; 20 units through a choke all arrive within
  `2 × path / speed + c`; a group arrives, every unit Stationary, and stays so for 100 ticks; an idle unit in the way
  is passed and never moved; goal on a cliff; enclosed unit; an object spawned on the path forces one repath; chase a
  moving target; three Shift-queued waypoints in order; Patrol for three cycles; Hold; Stop. **(M)** The magic box
  (`crates/sim/tests/group_destinations.rs`, 0021's Verification): a line ordered far away arrives as the same line
  centred on the click; a click inside the box converges with nobody failing; a mixed group strings out, fast units
  first; offset destinations on a cliff or off the map are retargeted; a single unit goes to the click as before.
  **(Y)** Yielding, occupied targets and the schedule (`crates/sim/tests/movement_yield.rs`, the Verification lists
  of 0022–0027): a unit walks through its owner's idle group and around an enemy's; two own units head-on in a
  corridor both give up; Hold units, attacking units and units that can't move don't budge; a slowed unit yields at
  its speed; twelve units converge without shuffling or re-routing and then step aside for another unit; a unit three
  tiles behind a wall of Hold units re-routes and gives up, one within 2 GU arrives; a boxed-in unit repaths at 32,
  48 and 80 ticks and gives up at 104; a unit blocked for 20 ticks never repaths; a save in the middle of a sidestep
  continues identically. The 18-v-18 runs (`movement_crowds.rs`) print their counts. **(Q)** Shift-queued group
  orders (`crates/sim/tests/queued_group_destinations.rs`, the Verification lists of 0029 and 0030): a line queued
  far north and back returns as the same line; a queued converge followed by a far queued click moves the blob as a
  blob; a mixed line queued north and back turns at different times and ends in its line; a unit removed on the way
  leaves the others' spots unchanged. **(G)** Crowds (`crates/sim/tests/movement_crowds.rs`): two blocks of 18 meet
  head-on and at most 2 of 36 give up, with one owner and with two; dense blocks of other shapes and of Guards pass
  with at most a fifth failing; a group walks through its owner's idle block of the same width, which ends within 96
  su of its places (40 su on average) with everyone standing, and an enemy's block never moves. The bounds are loose
  on purpose. `World::lane` has its own unit test. Nav: a group's field costs at most the component; shared smoothing
  gives the waypoints of `smooth`; `open_around` implies `box_clear` and `los`; the cells' marks never miss a heading
  that counts as oncoming.
- **Oscillation**: distance travelled ≤ 1.5 × path length + c; reversals of velocity along the path ≤ a small bound.
- **Determinism**: same run twice, per-tick hashes equal; replay round trip; **save at tick k, load, continue**:
  hashes equal (catches hidden unserialized state); **golden hashes** for three scenarios as committed constants.
- **Soak**: seeded random orders, 200 units, random maps, 2 000 ticks: invariants hold, no panic. **(Y)** Every
  move ends within `GIVE_UP_TICKS` and the grace of its last progress, and of the moves that end at most 60 % fail
  in a seed and 35 % overall (16 % with yielding, 32 % before; **(G)** 7 % with lane seeking: 38, 9 and 34 failed
  moves in the three seeds, where 105, 54 and 64 failed).
- **Perf**: an `#[ignore]` test printing ms per tick for 500 units on 256×256, plus CI assertions on the counters.
- **(E) Behaviours** (`crates/sim/tests/behaviours_*.rs`, the Verification lists of 0031–0034, 0036 and 0037): an
  attacker stops when the gap between the edges is its Range; a chase around a corner until the target is
  destroyed; a target lost in the fog is followed to its last seen position; a target across a moat ends the order
  at the water's edge; a second attacker queues behind the first for longer than a stuck move lasts; an idle unit
  chases to its leash and returns; Hold never moves; a follower keeps up, waits within 8 su, and waits behind a
  plugged corridor without re-routing; a patch ringed by units is reached within 2 GU; an AttackMove group
  engages and resumes; a Patrol cycles, engages, starts where the order before it ended, turns around after a
  failed leg and ends after two. A soak of 120 units fighting under random orders of every kind keeps the
  invariants, leaves no dangling id and asks for few paths. **I5–I7 changed**: a leg is one `Mover::go_to`, an
  order's or a behaviour's, so a unit may also move, and get its repaths back, when a behaviour began a move in
  that tick.
- **QA**: `TestSim::ascii()` renders the map with units as letters for readable failures; a client path overlay
  (waypoints, destinations, status colours) behind a debug key and flag; client replay playback (`--replay <file>`), so a
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
