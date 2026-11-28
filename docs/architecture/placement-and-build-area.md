# Placement and the build area

Read this when you place structures or sites, or touch footprints, the build area or placement checks.

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
- **No Recruitment Area rule** (slice M5-S4; design change 0387, `factions/cults/structures/recruitment_hall.md >
  RecruitmentArea`: "a Hall can be placed anywhere its 4x4 footprint fits on the map, including right at the edge"):
  a Recruitment Hall takes its footprint's rules alone, in every mode, right up to the map's edge and into its
  corners. Slice M5-J1's rule, its whole area on the map (design change 0341 point 1,
  `WholeProblem::RecruitmentAreaOffMap`), is gone. The tiles of its area off the map count among its 100 and are
  never Recruitable (*Production > The Recruitment Halls*). Tunnel Areas keep their own edge rule (0242).
- **The files** (`placement/`): `site.rs` (`Site`, `Mode`, `Method`, `Occupant`), `problems.rs` (the problems,
  `PlacementCheck`, their texts), `check.rs` (the check), `footprint.rs` (its tiles), `tunnel_area.rs`,
  `tunnel_overlap.rs`, `build_area.rs` and `onto.rs`. The geometry of Tunnel Areas, their regions and an expansion's
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
  (`placement/wall.rs`, geometry alone): the loop round the footprint where a unit's box stands `Construction::build_standoff_su` off
  it, searched from its point nearest the unit (`Wall::hug`), then both ways along it, `Construction::hug_step_su` at a time, of two
  as far the clockwise one first; `Wall::out` is a row further out, `Wall::next_corner` where the way along it turns.
  (Slice M5-F1's spots around a footprint, plan decision T62's one search for a unit put on the map beside a building,
  `placement::Spots` and `Sim::spot_around`, went once nothing used them: the units of a destroyed building that holds
  units appear inside its former footprint since design change 0349, slice M5-F3, `Sim::evacuate`, and the Laser
  Turret's Agent, which came out beside the turret it finished, went with design change 0359, slice M5-Y. The
  Garrison's units leave through its door, all at once, and its destruction is the evacuation's: *Object model >
  Buildings that hold units > The Garrison*, slice M5-N2.)
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
