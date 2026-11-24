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

- **M1-Q queued group destinations** (merged in `bb867a5`; 519 tests):
  - 0030 was already true: each unit's destination is resolved at the click into `Order::slot` and never recomputed.
  - 0029: Shift-queued orders build the box from each unit's planned end (`destinations::planned_end`). That is the destination of its last Move or AttackMove in the current order plus queue. Stop and Hold are skipped. Object targets and Patrol fall back to the current position.

- **M1-E behaviours** (merged in `aa5a05f`; 554 tests):
  - **Structure**: `orders/{state,world,engage,travel}.rs`. Every behaviour reads one per-tick snapshot.
  - **Attack**: chases with `MoveGoal::Object` while `may_move` holds. It ends on the target's death, at the last seen position (0034), or when the approach fails.
  - **Idle auto-targeting**: by the design's priority within SightRange, with a 4 GU leash. HoldPosition uses weapon range only. After Stop the unit re-acquires at once.
  - **AttackMove**: 6 GU leash; PathReference is the first planned route.
  - **Patrol**: origin at execution (0036); two failed legs end it (0037).
  - **Move to an object**: proximity is 8 su edge to edge. Units are followed; static objects complete; the 2-GU blocked rule applies; "leaves the map" is a hook (0032/0033).
  - **Gate**: `Reject::NotVisible`, with the GDO station exception (Q0017 default).
  - **Range**: `shape.rs`, axis-aligned and ready for facing. `combat::reaches` is the one range check (0031).
  - **QA**: new hashed `DebugCommand::SetAutoTargeting`.
  - **Perf**: a 250 v 250 battle averages 0.49 ms per tick, 4.4 ms at worst (group fields on resume; G1).
  - **Open**: behaviour gaps went out as Q0018. The ignored `nav_perf` assertion fails (G1).

- **M1-F fog memory** (merged in `167d683`; 570 tests):
  - **Records**: per-player `vision::LastKnown` records of seen structures and resources. They are rewritten each tick the object is in sight, and dropped once a tile under them is Visible and the object is gone or has moved. Records are hashed and saved, and V1 rebuilds them.
  - **Cost**: no scans; the tile flips logged by `see`/`unsee` drive the updates. 500 units + 168 patches take 0.17 ms per tick.
  - **Hooks for 0040**: `records_revision()`, `record_tiles()` and `is_record_tile()`. Map-placed objects aren't seeded as known (0042), so nav needs its own starting set in M2.
  - **Client**: Explored tiles draw the records, dimmed, instead of live objects. Selection, markers and the InfoPanel use the records. Only GDO can select a remembered station; its InfoPanel shows "Last seen". A shot from the fog shows as a hit flash (Q0017 default).
  - **Follow-ups (in G2)**: the gate should use the records for the station exception; selection frames are drawn under structures; the remembered station's countdown is frozen (Q0019).

- **M1-S2 selection follow-ups** (merged in `f35f0f0`; 579 tests):
  - **Selection rules**:
    - Shift+box adds (0049).
    - Ctrl+Shift-click adds the type on screen (0052).
    - Add inputs do nothing while a non-owned object is selected (0053).
    - Every add goes through `ControlState::add_to_selection`.
  - **Control groups (0056)**: Ctrl+1 with an enemy selected used to put the enemy into the group. Fixed: Assign and Add need an all-own selection.
  - **Panel**: the portrait order is the Tab order (0055).
  - **Already matched**: 0048, 0051 and 0054.
  - **Kept as built, since the design is silent**: Shift+double-click adds the type; Ctrl on a box does nothing.

- **M1-R Riot Shield** (merged in `9aa41b6`; 598 tests):
  - **Mode**: a `riot_shield` capability with the states Lowered, Raising, Raised and Lowering, advanced in a new Modes phase before 5a.
  - **Command**: `Command::Ability { units, ability: RiotShield { raise } }`. It is not an order kind, because a mode isn't a task (0023).
  - **While up**: half speed and no attacking, from the start of raising to the end of lowering. Attack orders drop shielded units; there is no auto-targeting. Damage taken is halved before armor, only while fully Raised.
  - **Client**: an R button (Raise or Lower Shield). Shielded units right-clicked on an enemy move to the spot. A pale arc shows the raised shield.
  - **Open**: placeholders and the 0062 conflict went out as Q0020. The shield is not yet blocked inside a Bunker (a TODO in the gate).

- **M1-G1 crowds and nav perf** (merged in `4a982a0`; 603 tests):
  - **Lane seeking** (`World::lane`): before its sweep, a unit looks 4 lengths ahead and steps toward the nearest clear lane. It is deterministic: fixed orders, plus/minus/times only, no new state.
  - **Head-on crowds**: 18 v 18 at a 40 su pitch went from 14–18 of 36 failing to 0. Across 192 block layouts, failures went from 22% to 1.4%. Soak failures fell from 105/54/64 to 38/9/34.
  - **Passing through idle groups**: a mover aims between idle units, and they step straight across. Displacement is now 16 su on average (it was 204). `yielding::ask_ahead` asks two steps before contact; this is an implementation detail of "blocked", marked (G).
  - **Group field**: `Search::field_to` stops once every member's tile is expanded, and members share smoothing. Two 250-unit orders now take 5.9 ms in one tick (they took 7.1 + 6.1 ms). The battle's worst tick went from 4.2 to 0.67 ms.
  - **Accepted regression**: a 500-unit crossing tick averages 0.40 ms (it was 0.365); the target is under 1 ms.
  - **`nav_perf`**: the failure came from M1-M's sees-goal rule against its random goals. The test was adapted to the magic box.
  - **Leftovers (G3)**: 5×5 Guard blocks still lose a few units; `behaviours_soak` seed 2 is at 315 shots against a bound of > 300; members the anchor can't see plan alone.

## In progress (two agents in parallel, kept to disjoint areas)
- **M1-H2 command UI follow-ups** (Opus; brief `scratchpad/briefs/m1h2-command-ui-followups.md`), covering 0062–0069:
  - right-click per unit, where units unable to obey move or follow (0062/0063);
  - the Mixed pointer;
  - Attack targeting on own or neutral objects (0064);
  - invalid-click feedback (0065);
  - targeting at the press point (0068);
  - only player-driven selection changes cancel targeting (0069);
  - the shield's Move wins over Follow (Q0020 default).
- **M1-G2 QA tooling** (Opus; brief `scratchpad/briefs/m1g2-qa-tooling.md`):
  - every played game is recorded to `replays/`; `--replay` plays one back with pause, step and speed controls and desync checkpoints;
  - a golden-hash suite with `SC_BLESS=1` to re-bless;
  - the gate uses records for the station exception;
  - selection frames drawn above structures;
  - the remembered station countdown (Q0019 default);
  - 0038 tests.
- **Next**:
  - Then the M1 wrap-up: ask the user to playtest, and plan M2 with the Designer's current docs.

## M1 plan (from the movement spike; the approved design is in `docs/movement.md`)
The spike gaps went to the Designer as Q0007–Q0011. Until they're answered, the defaults from the doc apply.

| Slice | Content | After | Model |
|---|---|---|---|
| B Unit foundation | unit defs, `Orders`/`Mover`, `Command::Order` plus the gate, spawn checks, naive mover, phases, visuals, path overlay, `--commands` | done (`259f72c`) | Opus |
| C Navigation | `sc_sim::nav` | done (`88dd9bc`) | Fable |
| D Steering and collision | spatial grid, step/slide/follow/sidestep, stuck and recovery, invariant harness, movement scenarios | done (`b689663`) | Fable |
| S Selection | clicks, box tiers, control groups, Tab cycling, panels, debug player switch | done (`b63e77f`) | Opus |
| H Commands UI | command panel, right-click, cursors, indicators, Shift-queue | done (`a71cee1`) | Opus |

| M Magic box | group destinations per 0021 (replaces the 6.4 lattice) | done (`93e53ca`) | Opus |
| Y Friendly yielding | idle own units sidestep (0022/0023), stuck timing (0024), box silhouettes (0028), robust soak | done (`bc34d02`) | Fable |
| V Vision | per-player visibility, ElevationModifier for sight, fog texture | done (`83fec7d`) | Opus |
| W Combat | attack phases, hits, damage, deaths, range (Q0009), elevation helper | done (`dd268bc`) | Opus |
| E Behaviours | chase (fog: 0034), auto-target and leashes, AttackMove, Patrol (0036/0037), Hold, Stop, Follow and proximity (0032/0033), edge-to-edge range (0031), gate visibility | done (`aa5a05f`) | Fable |
| S2 Selection follow-ups | 0048–0056 | done (`f35f0f0`) | Opus |
| H2 Command UI follow-ups | right-click per unit (0062/0063), Mixed pointer, Attack targeting on own or neutral objects (0064), invalid-click feedback (0065), targeting rules (0066–0069), shield interplay (Q0020) | running | Opus |
| R Riot Shield | the Peacekeeper's toggled ability, end to end | done (`9aa41b6`) | Opus |
| Q Queued group destinations | 0029/0030 | done (`bb867a5`) | Opus |
| F Fog memory | last-known structures and resources, Explored selection, hit flash from the fog | done (`167d683`) | Opus |
| G1 Crowds and nav perf | head-on crowd steering, pass-through shoving, nav_perf, group-order and battle spikes | done (`4a982a0`) | Fable |
| G2 QA tooling | replay recording and playback, golden hashes, gate records fix, marker draw order, station countdown, 0038 tests | running | Opus |
| G3 Cleanup | a per-tick turn budget shared by movement and aiming (W's note); `mover.rs` size; the fragile `behaviours_soak` bound; 5×5 Guard blocks | after G2 (so the goldens are blessed once) | Opus |

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
- **path knowledge** (0040/0041): paths planned from each player's knowledge. Terrain and own structures are always known; others' structures once seen; remembered ones are walls; units re-route on discovery. Built on M1-F's last-known records. Patch depletion reopens tiles (0038).
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
- **Vehicles**, turrets, air, area damage, research. The vehicles milestone starts with the movement model from 0033 (rotating silhouettes, oriented collision, turning near obstacles), 0035 (acceleration and collision response), 0039 (Rugged passability per base, hover drag), and tracked maneuver data. The Factory's Artillery Rig and Prospector need Advanced Mechanics (0001). GDO air (0014): limited ammo with rearming at an Airfield; at most 4 aircraft per Airfield; Air-to-Ground Hovercraft and Air-to-Air Fighter (needs Propulsion Labs).
- **Underground and air movement rules** (0043–0047):
  - underground units don't collide with each other;
  - surfacing is refused onto occupied spots and reserves its spot;
  - air ignores terrain, and HoverCraft are clamped to the map;
  - Gliders may overshoot the map edge, staying visible and attackable at the nearest edge tile.
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
| Q0011 | Terrain/obstacles: resources block, Rugged, routes via the true map, underground/air | no | answered by 0038–0045 |
| Q0012 | Crowds clot without yielding: should friendly units yield or swap? (evidence from M1-D) | no | answered by 0022 (option B) and 0023 |
| Q0013 | Selection edge cases: ungroupables, allies, modified clicks, empty ground, InfoPanel for 2+ objects, control groups | no | answered by 0048–0058 |
| Q0014 | Combat edge cases: force-attacking own or neutral objects, a shot whose target dies mid-Firing, Rugged for units across tiles | no | answered by 0059–0061 (all as built) |
| Q0015 | Magic box meets yielding (converging groups), box from footprints vs centres, Shift-queued group moves | no | answered by 0027, 0028 and 0029 |
| Q0016 | Command UI edge cases: who carries out a right-click, invalid targets, input while targeting, Shift keeps targeting | no | points 1–3 answered by 0062–0069 (H2); point 4 (Shift keeps targeting) open |
| Q0017 | Fog: shots from hidden attackers (hit flash vs. a partial tracer); targeting remembered structures | no | open; F and E build the stated defaults |
| Q0018 | Behaviour gaps: sight-edge flicker, rotation in target priority, AttackMove leash reference, unreachable auto-targets, MinRange, walled-out spots | no | open; E built the stated defaults |
| Q0019 | A remembered station's delivery countdown: keep counting or freeze | no | answered by 0057: GDO sees the real timer (folded into G2) |
| Q0020 | Riot Shield: shield Move vs 0062 Follow, transitions, raising mid-shot, mixed groups, toggle time and hotkey | no | open; R built the stated defaults |

## Design change entries
Last triaged: 0069.

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
| 0029 Shift-queued group orders build the box from planned positions | done (M1-Q, `bb867a5`) | answers Q0015 point 3 |
| 0030 Group destinations resolved at issue, stored per unit | done (M1-Q, `bb867a5`) | probably already true; Q verifies |
| 0031 Attack range measured edge to edge | done (M1-E, `aa5a05f`) | answers Q0009 point 1; replaces default A |
| 0032 Move to a unit = Follow; proximity 8 su edge to edge | done (M1-E, `aa5a05f`) | answers Q0009 proximity |
| 0033 Silhouettes rotate; 2-GU rule for static targets; follow end | the 2-GU rule and follow end done (M1-E, `aa5a05f`); **rotation queued for the vehicles milestone** | rotation can only be verified with long vehicles; square infantry is nearly unaffected |
| 0034 Attack target lost into fog: last seen position | done (M1-E, `aa5a05f`) | answers Q0010 part 1 |
| 0035 Collision response: blocked motion dropped, velocity to 0 | **queued for the vehicles milestone** | every Verification point needs vehicles; infantry has no acceleration, so its behaviour is unchanged |
| 0036 Patrol origin at execution | done (M1-E, `aa5a05f`) | answers Q0010 part 2 |
| 0037 Patrol failed legs | done (M1-E, `aa5a05f`) | answers Q0010 part 3 |
| 0038 Crystal patches and supply stations block ground units | matches M1-C's default; G adds the Verification tests (path around a field, station not walkable, a removed patch reopens its tile) | answers Q0011 part 1; mining depletion is verified again with the economy |
| 0039 Rugged terrain impassable for wheeled, tracked and drill vehicles; hover crosses with double drag | queued for the vehicles milestone | infantry unchanged (M1-C keeps Rugged passable for infantry) |
| 0040 Pathfinding uses only what the owner knows | queued for the structures milestone (M2): per-player known-obstacle layer from M1-F's last-known records, plus a re-route on discovery | replaces default A (true map); M1 has no player-built structures, so nothing is observable yet; F was told to shape its records for it |
| 0041 The obstacle cache is only a suggestion | noted with 0040 | how and when to re-path is the Director's call |
| 0042 Map-placed patches and stations known from the start | with 0040 in M2 | confirms the 0040 assumption |
| 0043 Underground units don't collide with each other | queued (Later: underground) | |
| 0044 No surfacing onto occupied spots | queued (Later: underground) | |
| 0045 Air units ignore terrain; Gliders may leave the map edge briefly | queued (Later: air) | Q0011 fully answered |
| 0046 Surfacing reserves the spot | queued (Later: underground) | |
| 0047 Gliders outside the map edge stay visible and attackable | queued (Later: air) | |
| 0048 Selection edge cases answered from the docs | done (M1-S2, `f35f0f0`) | Q0013 points 1, 3, 6 |
| 0049 Shift+box adds | done (M1-S2, `f35f0f0`) | Q0013 point 4 |
| 0050 Correction: empty clicks not decided | superseded by 0051 | |
| 0051 Empty-ground clicks decided | done (M1-S2, `f35f0f0`) | |
| 0052 Ctrl+Shift-click adds the type | done (M1-S2, `f35f0f0`) | |
| 0053 Add inputs do nothing with a non-owned selection | done (M1-S2, `f35f0f0`) | |
| 0054 InfoPanel for 2+ objects kept as a placeholder | no change | Q0013 point 5 |
| 0055 SelectionPanel grouping: placeholder order = Tab order | done (M1-S2, `f35f0f0`) | |
| 0056 Control groups hold only own objects | done (M1-S2, `f35f0f0`) | answers Q0013 point 7; matches the current behaviour |
| 0057 GDO sees a remembered station's real delivery timer | folded into M1-G2 (replaces the Q0019 assumption) | answers Q0019 |
| 0058 Allies can't be multi-selected; GDO station intel intended | matches the build (no teams yet) | answers Q0013 point 2 |
| 0059 Explicit Attack may target own and neutral objects; right-click doesn't | matches the build | answers Q0014 point 1 |
| 0060 Hits land at the end of Firing; overkill wastes the shot | matches the build | answers Q0014 point 2 |
| 0061 Rugged effects use the centre tile | matches the build | answers Q0014 point 3 |
| 0062 Right-click: units that can't carry it out move or follow | in progress (M1-H2) | answers Q0016 point 1 |
| 0063 Right-click resolves per unit; Mixed cursor | in progress (M1-H2) | |
| 0064 Attack targeting accepts own and neutral destructibles | in progress (M1-H2) | |
| 0065 Invalid target click: ignored, targeting stays, with feedback | in progress (M1-H2) | answers Q0016 point 2 |
| 0066 While targeting, other commands are unavailable | in progress (M1-H2 confirms with a test) | answers Q0016 point 3, first part; matches the build |
| 0067 Box-drag while targeting does nothing | in progress (M1-H2 confirms with a test) | matches the build |
| 0068 Targeting clicks count at the press point | in progress (M1-H2) | |
| 0069 Selection changes while targeting: player changes cancel, deaths don't | in progress (M1-H2) | Q0016 point 3 complete |
| 0014 GDO Airfield and aircraft | queued (Later: air) | |
