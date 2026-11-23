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

## In progress
- **M0-E: map types and generated symmetry** (agent in a worktree; brief `scratchpad/briefs/m0e-map-types.md`). It covers:
  - the map/game-setup split, with slots and per-faction layouts (empty until M2);
  - the creation dialog and Skirmish Default / Custom map types;
  - generated symmetry with a locked region;
  - custom tile presets;
  - review fixes 2, 5, 7 and 10;
  - play-tests that end with F5 only;
  - a symmetric test map.

  It has been told to merge M1-A from master before finishing, folding `SimConfig.debug` into the game setup, with play-tests running with debug on.
- **M1-C navigation library: done** (`88dd9bc`, 244 tests). `sc_sim::nav` is pure and standalone:
  - NavGrid with components, rebuilt for 3 grids in 1.9 ms;
  - A* with retargeting;
  - LOS (Amanatides–Woo) and bisection smoothing;
  - group field;
  - budgeted `serve`;
  - Q0011 default switches.

  Release perf on a 256² worst-case map:
  - A* averages 1.8 ms on random pairs; the field takes 3.9 ms.
  - A **200-unit group order takes 9.4 ms**, against an 8 ms target.
  - **200 lone requests are served 4 per tick** under the 32k budget.

  **Tuning in G (hardening):** cheaper smoothing, budget tuning, or caching. Size class k > 1 is basic; per-player route grids aren't built (Q0011).

## M1 plan (from the movement spike; the approved design is in `docs/movement.md`)
The spike gaps went to the Designer as Q0007–Q0011. Until they're answered, the defaults from the doc apply.

| Slice | Content | After | Model |
|---|---|---|---|
| B Unit foundation | unit defs (Peacekeeper, Guard), `Orders`/`Mover`, `Command::Order` plus the gate (drop dead ids), spawn overlap checks, naive mover, phase stubs, placeholder visuals | M0-E merged | Opus |
| C Navigation | `sc_sim::nav` | none (running) | Fable |
| D Steering and collision | spatial grid, step/slide/follow/sidestep, group slots, stuck and recovery, invariant harness, movement scenarios | B, C; better with Q0007/Q0008 answers | Fable |
| S Selection | clicks, box tiers, control groups, Tab cycling | B | Opus |
| H Commands UI | command panel, right-click, cursors, indicators, Shift-queue | S | Opus |
| V Vision | per-player visibility, ElevationModifier, fog texture | B | Opus |
| W Combat | attack phases, hits, damage, deaths, range (Q0009) | B | Opus |
| E Behaviours | chase, AttackMove, Patrol, Hold, Stop, auto-target and leashes | D, W, V | Fable, or Opus with a Fable review |
| G QA and hardening | debug control of both sides, replay playback, path overlay, soak, perf, golden hashes | E, H | Opus |

Also queued: moving the editor onto the shared input layer (after M0-E).

## Queued
### M0-E: map types and generated symmetry (in progress; details below kept for reference)
- **Map/setup split**: the map file holds the map only:
  - type, name, description, size, terrain, symmetry;
  - N start positions (slots), each with empty per-faction layouts;
  - neutral pre-placed objects.

  Factions, player names and the seed belong to the game setup, picked for play-tests.
- **Creation dialog**: type, size, slot count, symmetry, name, description.
- **Map types** (0012):
  - **Skirmish Default**: symmetry is required and generated. Only the editable portion can be edited; the rest is locked and regenerated live, with everything mirrored, including object settings and building orientations. Owned objects are allowed only inside layouts.
  - **Custom**: no restrictions (its triggers and slot restrictions aren't designed, so not built).
- **Placeholder limits, flagged**: min/max size, max slots, slot count vs symmetry, objects on the symmetry axis.
- **Review fixes**: 2 (slot-name commit races the list rebuild), 5 (`--editor` player check), 7 (window close skips the unsaved-changes confirm; non-atomic save), 10 (per-frame UI work). Plus the editor test gaps.
- **Layout contents, completeness and outdated flags wait for M2**, when Starting Conditions exist.

### M1-A: input and command foundations (done; see Built)
- **Shared action layer**:
  - pointer state, cursor world position, modifiers, chords, drag/box state, an over-UI test;
  - one rebindable bindings table with the 0011 defaults (actual rebinding UI later).

  The editor moves onto this layer. Play-tests end with F5 only, since Esc = cancel (review finding 8).
- **Sim command gate**: `validate(&PlayerCommand)` checks the player exists, owns the ids, and the ids are alive. Debug commands only when debug is enabled (finding 9).
- **Client replay-hash test and interpolation test** (finding 11).

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
- **Vehicles**, turrets, air, area damage, research. The Factory's Artillery Rig and Prospector need Advanced Mechanics (0001). GDO air (0014): limited ammo with rearming at an Airfield; at most 4 aircraft per Airfield; Air-to-Ground Hovercraft and Air-to-Air Fighter (needs Propulsion Labs).
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
| Q0007 | Group moves: destination slots, head-on deadlock (idle units never yield, per spec) | no (needed before D) | open |
| Q0008 | Move failure numbers; unreachable or occupied targets | no (needed before D) | open |
| Q0009 | Range measured to large targets; proximity; silhouette rotation | no (needed before W) | open |
| Q0010 | Chase target lost in fog; Patrol origin; failed legs | no (needed before E) | open |
| Q0011 | Terrain/obstacles: resources block, Rugged, routes via the true map, underground/air | no | open; C builds the defaults as switches |

## Design change entries
Last triaged: 0020.

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
| 0014 GDO Airfield and aircraft | queued (Later: air) | |
