# Implementation Plan

_Updated 2026-11-23. Spec: `artifacts/designer/design/`. Code rules: `ARCHITECTURE.md`._

Approach: thin vertical slices the user can run and see, each verified by tests and agent-taken screenshots.

## Built
- **Workspace scaffold** (`8ce3444`). The `sc_sim` core interface:
  - units and coordinates, ids, libm math
  - tile map with default presets, object defs and instances
  - commands, events, scenarios (`dev_skirmish`, `empty`)
  - the `Sim` tick pipeline and the Supply Delivery Station countdown

  Determinism lints are in `crates/sim/clippy.toml`. Bevy 0.20.1 client stub.
- **ARCHITECTURE.md**: the binding brief for implementing agents.
- **M0-A sim foundation** (`1d966e3`):
  - ASCII test maps; PCG32 `Rng`
  - `Sim::state_hash`, exhaustive: a new field won't compile until it is hashed
  - `Sim` serde; `Replay`; `sc_sim::testing`
  - versioned JSON scenario files; `ObjectKind::ALL`
  - `Scenario::validate` and `Sim::try_new`
  - `PlayerSetup::start_location`; non-finite positions rejected
- **M0-B client foundation** (`6593838`, merged in `731cc59`):
  - `build_app` with one plugin per feature
  - 16 Hz sim driver with recorded inputs
  - input layer (`InputAction` plus one key table)
  - camera: 28 GU wide, edge/arrow pan, clamped
  - map mesh with a grid toggle (G)
  - placeholder patches and stations; HUD frame
  - QA mode: offscreen PNG capture with no window, ~0.5 s per shot, byte-identical across runs
  - Note: when the display is asleep, vsynced windows run at 1 FPS; QA windows don't use vsync.
- **M0-C map editor** (`3ebe2c9`, merged in `a4a0945`; 148 tests across the workspace). Run `cargo run -- --editor [maps/x.json]`. It has:
  - the game's renderer and camera, plus zoom
  - terrain paint, elevation, flood fill and brushes
  - an object palette from `ObjectKind::ALL` with an owner picker, ghosts, select/move/delete
  - player slots with start and camera markers
  - undo/redo
  - a `maps/` list, save, name/seed/resize
  - a live problem list that focuses the camera
  - F5 play-test and back; live symmetry
  - an F1 help overlay
  - `--editor-script` for QA

  Sample: `maps/dev_skirmish.json`. `--map`/`--scenario` exit with code 2 and list the problems instead of panicking. The camera falls back from camera start to start location to map centre. The Director checked the screenshots.

- **M0-D corner-height terrain** (merged in `cffb46f`; 203 tests). It covers:
  - **Terrain model:** a corner heightmap with bilinear height, floor level, flatness and max-slope validation.
  - **File format v2:** base-17 height rows and explicit per-spawn `settings`; `MAX_MAP_SIDE` 256 is checked before allocating.
  - **Per-object values:** per-object patch and station values (0007); patches and stations need flat tiles (0013).
  - **Spawn timing:** an object spawned in tick t is first updated in t+1.
  - **Rendering:** height offset 0.25 GU per level; Lambert slope shading lit from the NW; cliff faces with strata; objects drawn at terrain height; `coords::ground_under` picks the frontmost surface.
  - **Camera:** middle-mouse grab pan (0010).
  - **Editor height tools:** corner tools whose slopes cascade and stop at cliffs; a slope overlay (K).
  - **Symmetry:** pure transforms for 6 modes in `sc_sim::symmetry`.
  - **Test map:** rebuilt with starts in the corners, hills, and a cliff plateau with slope entrances.
  - **Review fixes:** findings 1, 3, 4 and 6.

  The UI font has ASCII glyphs only.

- **M1-A input and command foundations** (merged in `fec5aec`; 224 tests):
  - **Input layer:** a shared input layer for the game (`ActionState`, `Pointer` with ground/view/over-UI, `MapClick` and `SelectionBox` messages, chords and double-tap/double-click on `Time<Real>`), plus headless injection.
  - **Bindings:** one rebindable `Bindings` table with the 0011 defaults; Esc = Cancel.
  - **Command gate:** `Sim::validate` runs inside `step`.
  - **Debug switch:** `SimConfig { debug }` is hashed and recorded in `Replay`; `--no-debug` turns it off.
  - **Tests:** client replay-hash and interpolation tests.
  - **Placeholders:** drag 4 px; double tap and double click 0.4 s; double-click distance 6 px.
  - **Open:** bindings aren't serde yet (that needs Bevy's `serialize` feature; decide when rebinding is built), and there is no rejection event.

- **M0-E map types and generated symmetry** (merged in `3251567`; 280 tests). `Scenario = { map: MapDef (format v3), setup: GameSetup }`:
  - **MapDef**: type, symmetry, terrain with preset looks, slots (start, camera, per-faction layouts), and neutral or slot-owned spawns.
  - **GameSetup**: players, seed, `config: SimConfig`. Replays are map + setup + inputs.
  - **Editor**: a creation dialog; Skirmish Default / Custom types; generated symmetry with the locked part dimmed; custom presets with looks; a play-test setup panel; debug on in play-tests.
  - **Placeholder rules, flagged in code**:
    - sides 16–256, at most 8 slots in whole symmetry sets, 90° only on square maps;
    - objects on an axis: a self-image object stays a single object, and one that overlaps its own image is refused;
    - preset removal is refused while the preset is in use.
  - **Fixed**: review findings 2, 5, 7, 10, the editor part of 11, and F5-only play-test exit.
  - **Not done**:
    - Starting Layout editing (M2);
    - a panel scrollbar (scroll with the wheel);
    - the right panel runs past the bottom on maps with 4 or more slots.

- **Editor on the shared input layer** (merged in `6bdc678`; 284 tests):
  - Every editor key, mouse button and the wheel are `InputAction::Editor(..)` entries in `Bindings`, with an `InputContext` of Game or Editor.
  - One over-UI test serves both modes, and focused text fields swallow typed keys.
  - `Pointer::press` handles strokes and drags over UI.
  - The editor QA screenshot is byte-identical to the one taken before the change.

- **M1-B unit foundation** (merged in `259f72c`; 324 tests):
  - **Units**: Peacekeeper and Guard, with every attribute the design gives (combat stats stored as data).
  - **Orders and movement state**: `Orders` (cap 32) and `Mover` capabilities. `Command::Order` goes through the gate: dead ids are dropped; not-owned or empty orders are rejected.
  - **Pipeline**: phases 5a–5f (`orders::update`, `paths::sync`, `paths::serve`, `movement::step`, plus combat and vision stubs).
  - **Movement**: a naive mover follows nav paths with an exact arrival snap. Group slots follow `docs/movement.md` 6.4.
  - **Validation**: units can't be placed on blocked ground or on other units. This covers debug spawns, scenarios and editor placement.
  - **Visuals**: placeholder units (circle and square), a P path overlay, and `--commands` QA input.
  - **Not done**: unit–unit collision (D) and the Attack/AttackMove/Patrol/Move-to-object behaviours (E).

- **M1-S selection** (`e14dd82`, merged in `b63e77f`):
  - **Selection state**: `ControlStates`, one per player: the selection grouped by kind, the ActiveGroup, and 10 control groups.
  - **Selecting**: view-space picking on drawn boxes, and box tiers. Click rules use the bindings: Shift toggles; Ctrl or a double-click selects that type on screen.
  - **Control groups and cycling**: Ctrl, Shift or # on a group, and a double-tap centres on it. Tab cycles the active group.
  - **Display**: markers, SelectionPanel, InfoPanel.
  - **Debug**: F2 switches the player.
  - **Removed objects** leave every selection and group.
  - **QA**: an `--input-script` QA flag.
  - **Open**: design gaps went out as Q0013.
- **M1-D steering and collision** (merged in `b689663`; 387 tests):
  - **Movement**: the real 5d step in `movement.rs` with `movement/{space,collide,stuck}.rs`. No overlap by construction; exceptions go through `space::blocks`.
  - **Recovery**: repath and stuck policy. Repaths at 24/48/72 ticks; Failed at 104.
  - **Invariants**: I1–I7 are checked after every tick in all `TestSim` tests.
  - **Testing**: a soak of 200 units × 2 000 ticks on 3 seeds.
  - **Performance**: 500 units take 0.19 ms per tick on average and 0.64 ms at worst. A 250-unit group order takes 6–10 ms (borderline).
  - **Deviations**: documented in `docs/movement.md` (D).
  - **Crowding**: dense head-on crowds clot and some moves fail, because the spec says no pushing. The evidence is in Q0012.
  - **Group arrival rule**: a unit stuck behind arrived members of its own group counts as arrived. This is a stand-in.

- **M1-W combat** (merged in `dd268bc`; 428 tests):
  - **Attack phases**: Aiming, Firing, Cooldown, Reloading, each lasting its duration in ticks. Aiming turns at TurnRate until the unit faces the target within 1°. A unit holds still while aiming, firing or cooling down (`combat::may_move`, wired into 5d).
  - **Range**: centre to the target's nearest point (Q0009 default), MinRange, TargetDomain, and the ±1 elevation rule in `elevation.rs`, shared with vision.
  - **Damage**: all hits of a tick apply together after everyone acts. Armor, then the directional multiplier, then minimum damage; HP is fractional.
  - **Deaths**: one removal path, `Sim::remove_objects`. Orders and attacks on the dead target end.
  - **Orders**: Attack on a target in range. An out-of-range target waits for E (`TODO(E)`).
  - **Client**: `combat_effects.rs` draws tracers, death markers and HP bars (damaged units and the local selection).
  - **Open**: edge cases went out as Q0014. A unit that moves in 5d and starts aiming in 5e can turn twice in one tick; harmless at 180°/tick, revisit in E.

- **M1-M magic box** (merged in `93e53ca`; 439 tests):
  - **The rule** (`orders/destinations.rs`) covers Move to a point, AttackMove and Patrol for 2+ units.
    - The box is the bounding rectangle of the unit centres; Q0015 asks to switch to footprints, which M1-Y is building. The group centre is the box centre.
    - A click inside the box (edges included) sends every unit to the exact click point. A click outside sends each unit to its position + (click − centre).
  - **Retargeting** (Q0008 default): destinations are clamped onto the map, then moved by `nav::reachable_point` to the nearest standable point in the unit's component.
  - **Group fields**: one field anchored on the click tile. A member joins only if the anchor sees its goal; the others use A*.
  - **Also fixed**: a steering limit cycle in `movement/collide.rs` (a leaning sidestep now sets `avoid_side`).
  - **Soak test**: arrivals dropped with converging random groups, so the `>= 100` threshold is fragile. M1-Y reworks it.

- **M1-H commands UI** (merged in `a71cee1`; 469 tests):
  - **Command panel**: a 4×3 grid for the ActiveGroup's interface state. Peacekeeper and Guard show Q Move, W Reverse (unavailable), E Hold, A Attack, S Patrol, D AttackGround (unavailable) and X Stop. While targeting, Z is Back.
  - **Right-click**: ground gives Move; an enemy gives Attack; an own unit, crystal patch or station gives Move to that object.
  - **Targeting**: Esc, right-click, Z or Back cancel it. A Shift-click queues the order and returns to default, as the design says.
  - **Shift-queue**: append or replace.
  - **Indicators**: flashes, plus dashed routes while Shift is held.
  - **Cursors**: placeholder cursors, also drawn in QA screenshots.
  - **Guard**: now drawn axis-aligned.
  - **Gaps**:
    - Riot Shield has no sim state or command yet (see R).
    - No OrderKind yet for AttackGround, Reverse or Enter (later milestones).
    - Patrol origin isn't kept in the sim (E).
    - The UI edge cases went out as Q0016.

- **M1-Y friendly yielding** (merged in `bc34d02`; 490 tests):
  - **Who yields** (`movement/yielding.rs`): an idle own unit with no orders, no attack phase and able to move. Hold Position counts as an order. Units never yield to the group order they arrived with (0027).
  - **Sidestep**: 45°, only as far as clears the lane, chained up to 4 units deep, with no return trip.
  - **Stuck schedule (0024)**: stuck at 24 ticks; re-routes at 32, 48 and 80; Failed at 104.
  - **Arrival (0026)**: a unit within 2 GU behind non-moving blockers counts as arrived. It arrives after 12 ticks when the blocker has no goal, otherwise at the stuck point. This replaces D's group-arrival stand-in.
  - **Magic box (0028)**: it bounds unit silhouettes.
  - **Soak test**: failed share dropped from 32% to 16%, and the test now asserts bounded ratios.
  - **Known issues** (for G):
    - tight own groups crossing head-on still jam, 14 of 36 failing at a 16 su gap;
    - walking through an idle group of the same width shoves it about 4 tiles;
    - an idle unit in a corridor as narrow as itself can't sidestep.
- **M1-V vision and fog** (merged in `83fec7d`; 513 tests):
  - **Vision state**: per player and tile: Unexplored, Explored or Visible.
  - **Sight rule**: tile centre within range, inclusive, with the elevation ±1 rule; no line of sight.
  - **Updates**: incremental; only moved objects are restamped.
  - **Hashing and checks**: Explored is hashed and saved; Visible is rebuilt on load. Invariant V1 checks it each tick.
  - **Perf**: 0.17 ms per tick with 500 units.
  - **Combat**: attacks need the target seen last tick.
  - **Client**: one fog mesh with one texel per tile, softened; hidden enemies are not drawn, selected, targeted or marked; F3 toggles fog; `--no-fog`.
  - **Gaps**:
    - Explored areas show live structures and resources instead of last-known ones (F builds this);
    - the gate accepts unseen targets (E builds this).

## In progress (three agents in parallel, kept to disjoint areas)
- **M1-E behaviours** (Fable; brief `scratchpad/briefs/m1e-behaviours.md`):
  - AttackingObject chase, including into fog to the last seen position (0034);
  - idle auto-targeting with leash; HoldPosition;
  - AttackMove with leash;
  - Patrol, with origin at execution (0036) and failed legs (0037);
  - MovingToObject, with proximity 8 su edge to edge, Move to a unit as Follow, and the 2-GU rule (0032, 0033);
  - edge-to-edge range (0031) through a new `shape` module;
  - the gate rejects unseen object targets (Q0017 default).
- **M1-Q queued group destinations** (Opus; brief `scratchpad/briefs/m1q-queued-group-destinations.md`): destinations resolved at issue and stored per unit (0030); queued orders build the box from planned end positions (0029).
- **M1-F fog memory** (Opus; brief `scratchpad/briefs/m1f-fog-memory.md`):
  - per-player last-known records of structures and resources in the sim, hashed;
  - Explored areas draw records rather than live objects;
  - the GDO station's Explored selection and InfoPanel;
  - a hit flash for shots from the fog (Q0017 default).
- **Next**:
  - **R Riot Shield mode**: after E.
  - **G QA and hardening**, after E:
    - golden hashes;
    - replay playback;
    - perf tuning of group orders (a 250-unit order takes 6–10 ms);
    - steering quality for dense head-on crowds (M1-Y known issues);
    - `mover.rs` size;
    - the turn-twice-in-a-tick note from W;
    - 0038 Verification tests.

## M1 plan (from the movement spike; the approved design is in `docs/movement.md`)
The spike gaps went to the Designer as Q0007–Q0011. Until they're answered, the defaults from the doc apply.

| Slice | Content | After | Model |
|---|---|---|---|
| B Unit foundation | unit defs, `Orders`/`Mover`, `Command::Order` plus the gate, spawn checks, naive mover, phases, visuals, path overlay, `--commands` | done (`259f72c`) | Opus |
| C Navigation | `sc_sim::nav` | done (`88dd9bc`) | Fable |
| D Steering and collision | spatial grid, step/slide/follow/sidestep, stuck and recovery, invariant harness, movement scenarios | done (`b689663`) | Fable |
| S Selection | clicks, box tiers, control groups, Tab cycling, panels, debug player switch | done (`b63e77f`) | Opus |
| H Commands UI | command panel, right-click, cursors, indicators, Shift-queue | done (`a71cee1`) | Opus |
| R Riot Shield mode | the Peacekeeper's Riot Shield as sim state, command and hotkey; shielded speed and right-click per the design | after E (it touches orders) | Opus |
| M Magic box | group destinations per 0021 (replaces the 6.4 lattice) | done (`93e53ca`) | Opus |
| Y Friendly yielding | idle own units sidestep (0022/0023), stuck timing (0024), box silhouettes (0028), robust soak | done (`bc34d02`) | Fable |
| V Vision | per-player visibility, ElevationModifier for sight, fog texture | done (`83fec7d`) | Opus |
| W Combat | attack phases, hits, damage, deaths, range (Q0009), elevation helper | done (`dd268bc`) | Opus |
| E Behaviours | chase (fog: 0034), auto-target and leashes, AttackMove, Patrol (0036/0037), Hold, Stop, Follow and proximity (0032/0033), edge-to-edge range (0031), gate visibility | running | Fable |
| Q Queued group destinations | 0029/0030 | running | Opus |
| F Fog memory | last-known structures and resources, Explored selection, hit flash from the fog | running | Opus |
| G QA and hardening | debug control of both sides, replay playback, soak, perf (group orders), golden hashes, crowd steering quality | E | Opus or Fable |

After B: start D, S, V and W in parallel; then H, E, G.

## Queued
### M1: units move and fight (sandbox)
Units: Peacekeeper (GDO) and Guard (Syndicate). The work:
- **movement**:
  - pathfinding and hard collision; slopes don't change speed or path cost (0006)
  - Move, Stop, Hold, Attack, AttackMove and Patrol, with Shift-queueing
- **combat**: attack phases and damage
- **vision and fog of war**, with **ElevationModifier**:
  - the elevation level is the floor of the interpolated terrain height; ±1 sight/range, binary
  - units and ground/underground structures count as both source and target; air is exempt (0004, 0005)
- **selection**: clicks, box tiers, control groups, Tab cycling. Default bindings per 0011: Shift-click toggles; Ctrl-click and double-click select that type on screen; Ctrl/Shift/# for control groups, and a double-tap centres; Shift appends to the queue.
- **command panel** (BasicCombatUnitInterfaceState): standard slots by hotkey per 0002 (Z back/cancel, X cancel production, C rally point, V free)
- **right-click resolution**, cursors, command indicators
- **debug tools** to control both sides; replays

A movement design spike may come first. Fable is reserved for pathfinding, movement and collision.

### M2: GDO base and economy
- **buildings**: Deployment Center, Generator, Barracks, Engineering Facility, plates; power; unit control; build area
- **placement**: ghost with rotate/flip; the **flatness rule** (every footprint tile flat, for Direct Placement, Worker-Built on arrival and Warp In; 0004)
- **production**: queues and rally points; crystal harvesting; defeat conditions
- **GDO supplies**: Supply Tower, Chopper
- **Starting Conditions and Starting Layouts** (0008, 0009, 0012):
  - layout contents, overlay editing of all faction layouts;
  - on Skirmish Default maps, only Starting Conditions objects are allowed, and every layout must be complete (the editor lists what's missing);
  - a flag when a layout is outdated against Starting Conditions;
  - only the picked faction's layout spawns at game start;
  - the Director authors the test map's layouts.

### M3–M5: other factions, basics
- **M3 Colonists**: Drone, Beacon, Warp In, Egress Node, portals, Legionnaires, shields.
- **M4 Syndicate**: tunnels, Hideout, Agent, Guard economy. Underground expansions are exempt from the flatness rule (0005).
- **M5 Cults**: Recruitment Hall, Storage, Recruit, Armory, Soldier, Gunner.

### Later
- **Vehicles**, turrets, air, area damage, research. The vehicles milestone starts with the movement model from 0033 (rotating silhouettes, oriented collision, turning near obstacles) and 0035 (acceleration and collision response), and tracked maneuver data. The Factory's Artillery Rig and Prospector need Advanced Mechanics (0001). GDO air (0014): limited ammo with rearming at an Airfield; at most 4 aircraft per Airfield; Air-to-Ground Hovercraft and Air-to-Air Fighter (needs Propulsion Labs).
- **Key rebinding UI** (0011): the bindings table is built rebindable in M1-A.
- **AI or online opponents**: decide with the user.
- **Art pipeline** (0003):
  - 3D models pre-rendered to pixel-art sprites at the fixed camera angle; 32 native px per tile
  - one render per distinct building orientation (by SymmetryType); 16 facings for infantry, 32 for vehicles, turrets and air
  - per-asset user sign-off
  - needs Blender and a 3D-model source; set up with the user then
- **Audio**: needs `alsa-lib-devel`.

## Open design questions
| Q | Topic | Blocking | Status |
|---|---|---|---|
| Q0001 | Art direction | no | answered by 0003 and 0004 |
| Q0002 | Input bindings | no | answered by 0010 and 0011 (defaults as proposed; rebindable) |
| Q0003 | Patch amount, supply defaults, reference map | no | answered by 0007 |
| Q0004 | Map model: slots, start locations, starting bases | no | answered by 0008, 0009 and 0012 |
| Q0005 | CommandPanel standard slots vs the 4×3 grid | no | answered by 0002 (matches assumption A) |
| Q0006 | Do map resources need flat tiles? | no | answered by 0013 (yes, both) |
| Q0007 | Group moves: destination slots, head-on deadlock, do idle units yield | no | answered: part 1 by 0021 (magic box); parts 2 and 3 by 0022 (no swapping; idle own units yield) |
| Q0008 | Move failure: stuck timing, unreachable or occupied targets | no | answered by 0024 (timing), 0025 (nearest reachable point), 0026 (arrived within 2 GU behind non-moving units) |
| Q0009 | Range measured to large targets; proximity; silhouette rotation | no | answered by 0031 (edge to edge), 0032 (proximity 8 su) and 0033 (silhouettes rotate) |
| Q0010 | Chase target lost in fog; Patrol origin; failed legs | no | answered by 0034, 0036 and 0037 |
| Q0011 | Terrain/obstacles: resources block, Rugged, routes via the true map, underground/air | no | part 1 answered by 0038 (patches and stations block ground units, as built); Rugged passability, unseen-structure pathing and later domains open; C built the defaults as switches |
| Q0012 | Crowds clot without yielding: should friendly units yield or swap? (evidence from M1-D) | no | answered by 0022 (option B) and 0023 |
| Q0013 | Selection edge cases: ungroupables, allies, modified clicks, empty ground, InfoPanel for 2+ objects, control groups | no | open; S built the listed defaults |
| Q0014 | Combat edge cases: force-attacking own or neutral objects, a shot whose target dies mid-Firing, Rugged for units across tiles | no | open; W built the listed defaults |
| Q0015 | Magic box meets yielding (converging groups), box from footprints vs centres, Shift-queued group moves | no | answered by 0027, 0028 and 0029 |
| Q0016 | Command UI edge cases: who carries out a right-click, invalid targets, input while targeting, Shift keeps targeting | no | open; H built the listed defaults |
| Q0017 | Fog: shots from hidden attackers (hit flash vs. a partial tracer); targeting remembered structures | no | open; F and E build the stated defaults |

## Design change entries
Last triaged: 0038.

| Entry | Status | Notes |
|---|---|---|
| 0001 Artillery Rig requires Advanced Mechanics | queued (Later: Factory) | |
| 0002 CommandPanel slots by hotkey | queued (M1 command panel) | answers Q0005 |
| 0003 Art direction | queued (art pipeline) | confirms the square grid and SC1-style client conventions |
| 0004 Corner-height elevation | done (M0-D); flatness for structures → M2; ElevationModifier → M1 | |
| 0005 Underground exemption; structures get ElevationModifier | queued (M1, M4) | |
| 0006 Slopes don't affect speed | queued (M1, a check only) | confirms the default |
| 0007 Per-object resource values | done (M0-D) | answers Q0003 |
| 0008 Starting layouts; symmetry tools | symmetry in progress (M0-D); layouts queued (M2) | |
| 0009 Layouts from scratch; Skirmish Default | queued (M2); per-layout toggle superseded by 0012 | |
| 0010 Middle-mouse grab pan | done (M0-D) | |
| 0011 Default input bindings; rebinding | bindings table done (M1-A); selection/groups → M1; rebinding UI later | answers Q0002 |
| 0012 Map types, creation, generated symmetry | in progress (M0-E); layout rules → M2 | answers Q0004 |
| 0013 Patches and stations need flat tiles | done (M0-D) | answers Q0006 |
| 0015–0020 GDO aircraft: assignment, ammo, return/rearm, landed = ground target, rearm time, names (Hornet, Dragonfly) | queued (Later: air) | design stubs |
| 0021 Group destinations: magic box | done (M1-M, `93e53ca`); footprint follow-up in M1-Y (Q0015) | answers Q0007 part 1; replaces the lattice default |
| 0022 Friendly idle units sidestep for blocked movers | done (M1-Y, `bc34d02`) | answers Q0012, Q0007 parts 2–3 |
| 0023 Who yields: idle only; modes don't matter; deployed Artillery Rig fixed | done (M1-Y, `bc34d02`) | the Artillery Rig part is a hook until vehicles |
| 0024 Stuck 1.5 s, 3 re-route tries, give up | done (M1-Y, `bc34d02`) | answers Q0008 part 1; M1-Y reconciles D's schedule |
| 0025 Unreachable move target goes to the nearest reachable point | done (M1-Y, `bc34d02`) | already built by M1-M (`nav::reachable_point`) |
| 0026 Occupied target: arrived within 2 GU behind non-moving units | done (M1-Y, `bc34d02`) | replaces D's group-arrival stand-in |
| 0027 No yielding within the same group order | done (M1-Y, `bc34d02`) | answers Q0015 point 1 |
| 0028 Magic box drawn around silhouettes | done (M1-Y, `bc34d02`) | answers Q0015 point 2 |
| 0029 Shift-queued group orders build the box from planned positions | in progress (M1-Q) | answers Q0015 point 3 |
| 0030 Group destinations resolved at issue, stored per unit | in progress (M1-Q) | probably already true; Q verifies |
| 0031 Attack range measured edge to edge | in progress (M1-E) | answers Q0009 point 1; replaces default A |
| 0032 Move to a unit = Follow; proximity 8 su edge to edge | in progress (M1-E) | answers Q0009 proximity |
| 0033 Silhouettes rotate; 2-GU rule for static targets; follow end | split: the 2-GU rule and follow end in M1-E; **rotation queued for the vehicles milestone** | rotation can only be verified with long vehicles; square infantry is nearly unaffected |
| 0034 Attack target lost into fog: last seen position | in progress (M1-E) | answers Q0010 part 1 |
| 0035 Collision response: blocked motion dropped, velocity to 0 | **queued for the vehicles milestone** | every Verification point needs vehicles; infantry has no acceleration, so its behaviour is unchanged |
| 0036 Patrol origin at execution | in progress (M1-E) | answers Q0010 part 2 |
| 0037 Patrol failed legs | in progress (M1-E) | answers Q0010 part 3 |
| 0038 Crystal patches and supply stations block ground units | matches M1-C's default; G adds the Verification tests (path around a field, station not walkable, a removed patch reopens its tile) | answers Q0011 part 1; mining depletion is verified again with the economy |
| 0014 GDO Airfield and aircraft | queued (Later: air) | |
