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

- **M1-G2 QA tooling** (merged in `52f20a8`; 632 tests):
  - **Recording**: every client game is saved to `replays/<UTC>_<map>.json` (gitignored; not editor or playback runs), atomically at the start, every 160 ticks and on exit. QA runs record with `--record <path>`; `--no-record` turns it off.
  - **Playback**: `--replay <file>` plays only recorded inputs, checking a hash every tick against checkpoints taken every 64 ticks. A desync is logged, fails a QA run with exit 1 and shows red in the top bar.
    - Keys: Space pauses, `.` steps, `-`/`=` change the speed (×0.5 to max). F2 and F3 work.
    - It works with `--screenshot`, `--exit-at-tick`, `--dump-state`, `--headless`, `--fast` and `--input-script`.
  - **Goldens**: `crates/sim/tests/golden.rs` and `golden/*.json` hold six games, including the Riot Shield, and run in 0.01 s. Re-bless with `SC_BLESS=1 cargo test -p sc_sim golden`. `testing::every_command()` round-trips every Command variant and fails to compile when a variant is added.
  - **Fixes**:
    - The gate uses the GDO player's records.
    - Selection frames are drawn above structures.
    - 0057: GDO sees the real delivery timer on a remembered station.
    - 0038 tests.
    - Leaving max pace no longer runs a 64k-tick burst.
  - **Checked**: a 113k-tick game replays exactly.

- **M1-H2 command UI follow-ups** (merged in `2417f70`; 643 tests):
  - **Per-unit right-click** (`command_ui/resolution.rs`): a unit that can't attack moves to the point or follows the target. Shielded units move to the point (Q0020 default, the single rule `order_without_attack`). One `Command::Order` is sent per distinct order, so magic-box grouping is kept.
  - **Pointer**: Mixed when actions differ, shown as red/green stripes (placeholder). Attack targeting accepts own and neutral destructible objects (0064).
  - **Clicks while targeting**: the target is resolved at the press point (0068). An invalid click shows "Invalid target" for 2 s and keeps targeting (0065). 0066, 0067 and 0069 are confirmed by tests.
  - **Open**: targeting details went out as Q0021.

- **M1-G3 cleanup** (merged in `7be0f39`; 646 tests):
  - **One turn budget per tick**: movement records the turn angle (`movement::Turns`, which is transient and not hashed), and aiming gets only TurnRate minus that.
  - **Files split**: `mover.rs` (321 lines plus submodules), `collide/lanes.rs`, `vision/player.rs`, `sim/load.rs`, the cli, bindings and editor tests. Still over 500 lines: `command_ui/interface.rs` (649; M2-H1 splits it), `resolution.rs` (537) and `movement_yield.rs` (537).
  - **Soak tests**: they now assert invariants. An idle unit near a visible enemy engages within 1 tick, every order ends, and refused, failed and search shares stay bounded. They were checked by breaking the code on purpose.
  - **Guard blocks**: re-routes avoid the group's own earlier arrivals (`stuck.rs avoid_group`). Sweep failures went from 1.4% to 0.7%, and Guard 5×5 losses from 38 to 20 of 800. Perf is unchanged. Enclosed places remain, see Q0032.
  - **No golden changed**, and the reasons are explained.
  - **Found**: a unit ordered to Move onto itself follows itself forever (Q0031).

- **M1-H3 command UI follow-ups** (merged in `80b3d88`; 666 tests). **M1 is complete.**
  - **Shift keeps targeting while held** (0070), via `AwaitingTarget { pending, queued }`. Shift counts if it is held at the press, a placeholder.
  - **No smart-casting** (0071): every panel command goes through `recipients()`, common commands to the Selection and group commands to the ActiveGroup. The audit found nothing wrong.
  - **Targeting click allowance** of 16 px (0073), measured as the farthest the pointer travelled.
  - **Shield rules**: right-click follows (0074), explicit Attack leaves shielded units out (0075), and AttackMove goes to every unit (0076). No sim change was needed.
  - **Any control group recall** resets menus (0078).
  - **Waits for M2**: 0074 Verification 2, the enemy-structure case.

- **M2-A structure foundation** (merged in `cbeb804`; 692 tests):
  - The DC, Generator, Barracks, EF and plate are sim objects. `StructureDef` and `SymmetryType` live in `defs/structures.rs`, and `Armor` moved to `ObjectDef`.
  - `sides(kind, orientation)` follows 0079. The Barracks is [A,B,A,C] at R0, with B facing east.
  - `ObjectKind::may_overlap` allows only a plate on an uncovered patch. Debug spawns refuse overlaps (`Reject::Overlaps`).
  - Placeholder meshes with type tags; remembered structures are drawn dimmed. A click on a covered patch picks the plate (0099). Enemy structures are auto-targets (0103). QA scenario `dev_structures`.
  - Fixed: `World::pick`'s scan radius missed large footprints.
  - No golden changed.

- **M2-E path knowledge** (Fable; merged in `f76db38`; 704 tests):
  - **Knowledge.** `paths/knowledge.rs` holds `PathKnowledge`, which is hashed, serialized and checked on load: per player, a version plus the starting resources the player still assumes are there.
  - **Grids.** Players' known grids are built lazily: own footprints, plus records, plus assumed resources. Planning uses the owner's grids: serve, staggered checks, chase line of sight, `reachable_point` and the magic box. Movement, the gate and I2 use the true grids.
  - **Sync timing.** Known grids sync once per tick, at the end, after `Vision::remember`. That deviates from the plan's 5b, so that save and load stay exact.
  - **Ownerless and neutral units** plan on the true grids.
  - **`NavGrid::restamp`** uses union-find labelling and takes 0.17 ms per grid, against 1.4 ms for a full rebuild. In the 10-deaths case seen by 4 players, the tick takes 1.0 ms with 5 rebuilds (the target was 5 ms). The worst tick overall is 3.7 ms, at tick 1 with 412 spawns.
  - **Housekeeping.** New invariant K1: each player's grids equal a fresh build. `paths.rs` is split into `paths/` and `sim/navigation.rs`. `ROUTE_ON_TRUE_MAP` is deleted.
  - **Goldens.** Re-blessed, because the hash now includes path knowledge. Play is identical.
  - **Open.** The blocked-to-open cases went out as Q0033.

- **M2-C placement** (merged in `0f8cc70`; 730 tests):
  - **`sc_sim::placement`.** A pure `check(site, occupants, kind, origin, orientation) -> PlacementCheck { tiles, whole }`, in `Mode::Command { vision, player }` or `Mode::Layout`. It reads the true state. Supporting it: `Sim::placement_check`, `Sim::build_area`, `BuildArea`, and `check_layout`, ready for F1.
  - **Build area.** Derived on every call and never stored (0092). Square, per 0091. BRE 0 adds no area. The DC needs no area.
  - **Tiles.** A tile the player can't see reports only `NotVisible`, so nothing hidden leaks. Ground units of any player block, neutral ones included (0094).
  - **Layouts.** A layout's area is its DC's alone (`core/map_editor.md`).
  - **Speed.** 9 µs per check.

- **M2-B economy core** (merged in `39bcf66`; 756 tests):
  - **Banks.** `Player { crystals, supplies }`, with checked spending, saturating credit, and `economy::Refund::{Full, Ready}`.
  - **Power.** `Sim::power(p) -> PowerSummary { generated, consumed }`. `ratio()` is ⌈g·2^16/c⌉, capped at 1. Producers are never slowed.
  - **Unit Control.** `Sim::unit_control(p)`, capped at 200. `economy::unit_control::reserved` is the hook for D2.
  - **Mining.** `Object::extractor` and `StructureDef::mining`. Loads come in lumps every 48 ticks, scaled by the power ratio (0098), and follow 0099's last-load and residual rules. A depleted patch goes when its plate goes, in the same tick (0038). The economy step runs between 5e and 5f.
  - **Tests.** 0098 v1–v2, 0099 v1–v2, 0042 v2 (a depletion in the fog), 0089, 0090 and 0077 are verified.
  - **Goldens.** Re-blessed, because banks and extractors are now hashed. Play is identical.

- **M2-G HUD** (merged in `69e80a1`; 767 tests; client only):
  - **Resource bar** (`hud/resources.rs`): Space Crystals, Supplies, Power as consumed/generated (red when over), and Unit Control through `Sim::unit_control`. The top bar has three columns.
  - **Structure InfoPanels** (0105):
    - own: name, HP and an activity area, filled by `hud/info_panel/activity.rs::lines`, which is the hook for H1 and H2;
    - others: name, owner and HP;
    - a plate also shows its patch's crystals (0099), to every viewer; see Q0034.

- **M2-D2 production** (Fable; merged in `1a64e75`; 807 tests):
  - **Commands.** `Command::Production { objects, command: Train | CancelProduction | SetRally }`. Also `Object::production`, `StructureDef::production`, `Sim::production_status`, and the events Queued, Refused, Cancelled and UnitProduced.
  - **Queue.** The limit of 20 includes the current unit (0081). Unit Control is reserved at queue time (0082). When several Barracks are pressed, the least remaining work pays first (0083). Destruction refunds the queue (0084, 0085). Production is slowed by power (0089).
  - **Rally points.** Ground, unit, self-clear (0086), and an enemy that leaves sight becomes a ground rally (0088). A rally on a remembered structure becomes a ground rally at the record's centre, a placeholder for Q0017 point 2.
  - **Exit doorway (0080).** `Mover::exit` is hashed. `World::door_free` lets a unit only come out, through side B. I2, I5 and I7 allow exactly that. Paths start at the first clear way out. A unit spawns as far out as there is room. Tested in 8 orientations, walled in and freed, mid-exit saves, and a 3,000-tick random game.
  - **Deviations.** Producers run in `Sim::step_production` right after `economy::step`. A blocked rally Move fails after 104 ticks, and the unit then goes idle.
  - **Perf.** The first numbers, 0.88 ms mean and 10 ms worst, were machine load (load average about 22). Interleaved runs against master were within 2%. Absolute numbers will be re-measured on a quiet machine in M2-J.
  - **Goldens.** Re-blessed for the new hashed fields. Play is identical.

- **M2-F1 game start** (merged in `d5a7d29`; 817 tests):
  - **Starting state.** `scenario/starting.rs` holds the Starting Conditions table: GDO gets DC, EF, 2 plates, 100 SC and Supplies 0; other factions are `None`. Banks are set at game start.
  - **Layout validation.** `scenario/validate/layouts.rs` validates every layout on its own: `check_layout` for structures, `misplaced` for units (0097). On Skirmish Default, `LayoutExtra` and `LayoutMissing` apply to implemented factions only. `MapDef::outdated_layouts` is derived.
  - **Camera.** It starts on the camera start, else the DC, else the slot start.
  - **`dev_skirmish`.** The DC is at (5,5), the EF at (9,1), and plates on (6,2) and (7,2). Slot 2 is the R180 image. All 8 patches are within the DC's sight (0095) and build area. Director approved.
  - **Goldens.** Re-blessed, because banks now start at 100.
  - **Found.** `replay.rs::playback_pauses_steps_and_changes_speed` flakes under machine load: it assumes one update at Max pace reaches the end. M2-I fixes it.
  - **Known gap.** New Skirmish Default maps can't be play-tested until M2-I adds layout editing.

- **M2-D1 construction** (merged in `fda74d3`; 838 tests):
  - **State.** `Object::constructor`: `Idle | Constructing { kind, progress } | Ready { kind }`, with recipes in `StructureDef::constructs`.
  - **Commands.** `Command::Construction { objects, command: Construct | CancelConstruction | CancelReady | Place }`. Cost is paid when the command applies. Refunds are Full, or Ready at 75% rounded down; destruction refunds the same way (0084).
  - **Placing.** Place re-runs `placement::check`. A refusal keeps the building Ready and emits `PlacementRefused`. A Place naming several constructors is refused.
  - **Events.** `ConstructionCompleted`, `StructurePlaced` and `ConstructionCancelled { refund }`.
  - **Recipe timing** (decided by the Director; ARCHITECTURE.md > Economy > Recipe timing). Progress starts in the tick the order applies. A recipe of N frames ordered at t completes at the end of t+N−1, and the result is usable from t+N. D2 already worked this way.
  - **Mini-oracle on `dev_skirmish`.** The Barracks starts at 48.1 s.
  - **Goldens.** Re-blessed once for the hashed constructor field. Play is identical.

- **M2-H1 production UI** (merged in `eccaaa6`; 855 tests; client only):
  - **Panel.** `UnitCommand` became `PanelCommand`, adding Train, CancelProduction and SetRallyPoint. `command_ui/interface.rs` is split into commands, templates, conditions and panel. The Barracks panel is Q Train, X Cancel, C Rally.
  - **Train.** One press sends one `Command::Production` to every selected Barracks. Buttons are available per `Sim::validate` on the exact command.
  - **Rally points.** Set by right-click or by C. Rallying on the Barracks itself clears it. Indicators per 0087 and 0088.
  - **Activity and feedback.** InfoPanel lines show training progress, the queue and the rally point. Refusals show "Not enough Space Crystals" or "Unit Control limit reached".
  - **Open.** Shift with rallies, rallying on one of several selected Barracks, and feedback for a disabled button went out as Q0035.
  - **Flaky test.** `games_are_recorded_as_they_run_and_when_the_app_exits` flaked once under load: `load_replay` returned an Err. M2-I is fixing it.

- **M2-F2 game end** (merged in `e3f67a9`; 872 tests):
  - **Defeat in the sim.** `economy/defeat.rs` applies T8: a player who owned a structure and now owns none is defeated. Plates count. `Player::defeated_at` and `Sim::outcome` are hashed. Events: `PlayerDefeated`, `GameOver`.
  - **Commands.** The gate refuses with `Reject::Defeated` and `Reject::GameOver`.
  - **Units on defeat (0101).** They become neutral: orders cleared, stopped, repathed.
  - **Client.** The sim stops at game over. Banners for VICTORY, DEFEAT, and DEFEATED while spectating. A defeated player has no input and watches with `Sight::watching` (0102). The replay is saved at the end, and playback pauses there.
  - **Tests.** Some fuzz and vision tests now give player 2 an out-of-sight Generator so their games don't end mid-script.
  - **Open.** Three readings went out as Q0036: simultaneous last defeats (no winner), free-for-all spectating (own memories only), a shield left raised.

- **M2-I editor layouts** (merged in `6ae2b02`; 893 tests; client only):
  - **Layout tool (T).** A slot and faction picker. On Skirmish maps the palette offers only the Starting Conditions, with counts left. Objects can be placed, dragged, deleted, turned (R, Shift+R) and flipped (F, Shift+F).
  - **Checks.** Placement is checked by the sim's `MapDef::validate`; the editor has no rules of its own and no sight checks (0097).
  - **Display.** Other factions' layouts are dimmed, and the DC's build area is outlined. Problems are grouped into one line per layout, and a refused F5 shows that line. The outdated flag is shown, and plates go onto uncovered patches.
  - **Shared input.** `Placing`, `Bindings::resolve_placing` and `InputAction::is_placement` were added to `input/bindings.rs`; H2 reuses them.
  - **Replay flakes fixed.** The two recording tests shared a temp folder; now there is one per test, with 0 failures in 449 parallel runs. The recorder's temp file is now named per process.
  - **Found.** The sim accepts a layout plate that isn't on a patch; J1 fixes it. The editor reports `LayoutOverlap` between slots only when F5 is refused, which is left for a later editor pass.

- **M2-J1 sim QA** (merged in `c2b2a17`; 901 tests):
  - **Economy oracle** (`crates/sim/tests/economy_oracle.rs`) on `dev_skirmish`, using the `testing::build_order` driver. Measured vs design:

    | Check | Measured | Design |
    |---|---|---|
    | Fully plated | 2:32 | about 2:30 |
    | Bank at 1:30 / 2:00 | 280 / 555 | about 300 / 600 |
    | Generator-first opening | plate 12.0 s later, 9.4% less mined by 3:00 | about 11 s, about 8% |
    | Clean Barracks | starts 48.1 s, ready 58.0 s | about 48 / 58 with lumps |
    | All-in Barracks | starts 15.1 s at power −21, production at 58.8% | about 15 s, about 59% |

    - **Differences:** lumps of 10 shift the design's continuous-income timings by about 2 s, which the design anticipates.
    - **Exact checks:** costs, build frames, the power ratio, lumps, cycles (±1 tick) and refunds.
    - **Tolerances:** documented, and proven with mutations.
  - **Golden `gdo_opening`** (2:45). Player 1 plays the clean Barracks, then sends 10 Peacekeepers that destroy 4 of player 2's plates. Player 2 is not defeated. Both players play through the golden `Script` hook. There are now 7 goldens.
  - **Soak.** `behaviours_soak` now excuses only units whose scan lost an enemy that tick. It exposed nothing.
  - **Fix.** A layout structure over a layout unit is reported once. A layout plate off a patch was already refused, and is now pinned by a test.
  - **Bug, for J2.** `MapDef::validate` accepts a map-placed (non-layout) plate off a patch. The ignored repro is `game_start.rs::a_pre_placed_plate_off_a_patch_is_refused`.

- **M2-H2 construction UI** (merged in `2d507b4`; 923 tests; client only). **M2 is playable end to end.**
  - **Interface states.** `InterfaceState` gains `BuildMenu` and `AwaitingPlacement(Placing)`.
  - **DC panel.** Q opens BuildMenu[Basic]: Q Generator, W Barracks, E EF. W opens BuildMenu[Advanced], which is unavailable (0104). X cancels construction, or a building that is ready.
  - **EF panel.** Build a plate, then place it.
  - **Placement.** The client `placement/` module has a ghost, green or red per tile from `placement_check`, with the side letters. The overlay shows the area only (0093) and is rebuilt only when the area changes. R and F turn and flip through the shared placement keys. A Shift-click places once (0071). Placement respects `commandable`.
  - **Activity lines** for construction (0105). Feedback for `PlacementRefused` and invalid clicks.
  - **Bevy quirk.** A `Text2d` changed in a frame when no camera saw text or sprites is lost. The ghost labels use `NoFrustumCulling`, stay visible, and are emptied rather than hidden.
  - **Placeholders.** A sim-refused Place returns to DefaultState with the building still ready. The ghost and overlay looks and the texts.

- **M2-J2 final QA** (merged in `3d98e29`; 926 tests). **M2 is complete.**
  - **Fixes:**
    - `placement::Onto` is now the single plate-on-patch rule, used by placement checks, layouts and `MapDef::validate`; a map-placed plate off a patch reports `SpawnProblem::NotOnPatch`.
    - The editor's Problems list is recomputed live (`sync_problems`), so `LayoutOverlap` shows up at once.
    - Defeat while placing is tested.
  - **Perf** (release, load 1.4 on 8 CPUs):
    - new `structures_perf` (4 bases × about 40 structures, 400 units): 0.28 ms mean; worst 7.8 ms in tick 1 from 20 group orders, 2.4 ms after that;
    - per-phase timings come from `Sim::step_observed` with `Phase`;
    - against `80b3d88`: movement +4%, behaviours +1.5%; no regressions.
    - `nav_perf::knowledge_rebuilds_256` had been failing since F2, unnoticed, because it's an ignored test; now fixed.
  - **Walkthrough.** 24 screenshots through the real UI, from the build menu to VICTORY at tick 3211, using one `SetResources`.
  - **Docs.** ARCHITECTURE.md reviewed. Playtest notes are in `docs/playtest/m2.md`.

- **M3-A Colonists foundation** (merged in `a22aa2d`; 949 tests):
  - **Kinds and data.** The seven kinds in `defs/colonists.rs`. `max_shields`, `beacon_capacity_cost` and `warp_in` are data only. `MELEE_CONTACT_GU` = 8 su (Q0042 point 5).
  - **The Drone** moves on the stand-in at 8 su per frame. It adds a (Ground, 1) grid, so nav rebuilds go from 1.0 to 1.4–1.7 ms until G.
  - **`Sim::change_kind`** (`sim/kinds.rs`) keeps the id and owner, rebuilds the capabilities, refunds a queue or construction, keeps orders, mover and target, and emits `KindChanged`.
  - **Client.** Placeholder meshes with tags; the Legionnaires use BasicCombatUnit. Goldens unchanged.

- **M3-H gathering** (Fable; merged in `5bd3b69`; 972 tests):
  - **Commands and state.** `OrderKind::Gather` and `DropOff`, `Object::worker`, `Ability::SetDropOff`, and gate rules in `sim/workers.rs`.
  - **Timed work and depletion.** Mining and drop-off are timed in `economy/gathering.rs`, with the bank credit and `ResourcesDelivered`. Depletion goes through the removal path, with `redirect_gatherers`.
  - **Rally and placeholders.** A rally on a patch gives Gather. Q0037's proposals are built and cited.
  - **Trip time** on the stand-in mover, with the patch 2 GU from the node: 60 ticks, 2.13 crystals per second per Drone.
  - **Soak under hard collision** (before 0106): 8 Drones 14.3/s; 12 Drones 9.8/s; 24 Drones gridlock at 0/s. The minimal jam is pinned, and 0106 answered the question.
  - **Found.** A blocked unit inherits the blocker's progress clock in `movement::stuck`; H2 checks it. Six open details went out as Q0044.

- **M3-C Colonists start** (merged in `5c75d40`; 988 tests):
  - **Army caps and banks.** `economy/army.rs` holds `ArmyCap` and `Sim::army_cap`, giving GDO Unit Control and Colonists Beacon Capacity. Syndicate and Cults are uncapped for now. Banks gain Alloys, Essence and Flux.
  - **Start.** The Colonists Starting Conditions: Egress Node, Robo Portal, 4 Drones, 100 SC; capacity 4/10. `dev_skirmish` has Colonists layouts in both slots (Egress Node (4,4), Robo Portal (6,6)).
  - **Client.** `--factions`, the Colonists bar, and the over-capacity red counter (0112). The camera starts on the Robo Portal.
  - **Tests and docs.** `ScenarioBuilder::faction`. ARCHITECTURE.md gets an "Adding a faction" checklist. The goldens were re-blessed for the bank fields.

- **M3-H2 Drone pass-through** (Fable; merged in `777b9a9`; 1005 tests):
  - **Rules (0106).** Both rules live in `movement::space::blocks`:
    - Drones on a gathering job (Gather or Drop Off, with `WorkerDef::pass_through`) don't block each other.
    - Units found overlapping at the start of the mover's turn ignore each other until apart.

    There is no pair state and nothing new is hashed. The goldens are unchanged.
  - **Invariants.** `invariants::may_overlap` holds I1's exceptions, with four negative tests. Any slice that adds another way for boxes to overlap must extend it.
  - **The stuck clock.** With `stuck::leads`, a blocked unit shares a blocker's progress clock only if the blocker has moved since its last re-path. Crowd paths changed: the `behaviours_perf` battle runs 1070 ticks instead of 880, with 16% more path searches.
  - **Saturation curve** on the reference row (stand-in mover, Drones dealt out):

    | Drones | 4 | 8 | 12 | 16 | 20 | 24 |
    |---|---|---|---|---|---|---|
    | Income per second | 8.3 | 15.1 | 22.0 | 26.8 | 32.0 | 32.0 |

    There are no jams. Below saturation the stand-in runs ahead of the design: 16 Drones give 27 against 23.1.
  - **Perf** (A/B): movement 0.421 → 0.430 ms a tick. The 24-Drone soak costs 13 µs a tick.
  - **Open.** The dead-centre steering oscillation went to G; two design details went out as Q0045.

- **M3-D portals** (merged in `20f584e`; 1025 tests):
  - **Exits (T19).** `ProductionDef::exits` is an `ExitRule`.
    - The Barracks keeps `Side(B)`.
    - The portals use `Nearest(&[A])`: every side for the Robo Portal (Q0041 point 3), and the 2 A sides for the Bio Portal.
    - A unit leaves by the door closest to the rally point, or else a random unblocked door. `Rng::below` is drawn only when two or more doors qualify.
    - A portal pauses only when every door is occupied.
  - **Rally.** A unit that can't attack, rallied on an enemy, moves to the point.
  - **Panel (T23).** The Production panel is built from `ProductionDef` for the Barracks and both portals: Q trains, X cancels, C sets the rally. It matches 0124. The unavailable W/E slots wait for J2.
  - **Fixed.** `Door::room_before` counted units behind a door as standing in front of it, which held releases 1–3 ticks. The same latent M2 bug could stall a Barracks.
  - **Reading.** When every free door is obstructed but none is occupied, the unit is still released, since the design pauses only when every door is occupied.

- **M3-E transformations and Warp In** (merged in `68ee747`; 1059 tests):
  - **Sim.** `Object::transform` (`Transform`, kept general for F), the WarpIn and CancelTransform commands, and `Method::WarpIn` in the one placement check. `Sim::step_transforms` runs in the economy phase, after depleted patches are removed and before the producers. `sim/gate.rs` is split by command family (T28).
  - **Design answers built:**
    - 0109–0111: the object is the target from frame 1, `follow_max` (HP), and `keep_fraction` with a 1-HP floor;
    - 0112: the retained 5;
    - 0113: counts for defeat;
    - 0115: no refund when destroyed;
    - 0116: functions held back, so production is None and drop-off skips a transforming node;
    - 0118–0119: units block, every tile must be Visible;
    - 0121: the footprint is derived by `transform::warp_in_footprint`, and `WarpIn` has no origin;
    - 0122: Beacons are Ungroupable.
  - **Client.** Visuals rebuild on a `MeshKey` change, with a cyan tint while transforming. The selection keeps the id through a kind change. The InfoPanel shows "Warping in: Robo Portal 45%".
  - **Goldens.** Re-blessed for the hashed `Object::transform`; the inputs are unchanged. Perf A/B shows no regression.
  - **Open.** A portal under Warp In still shows the portal panel; the Cancel-only panel is J1's (0124). A defeated player's units get visuals in the neutral colour.

- **M3-G Drone movement** (Fable; merged in `5d61dc7`; 1082 tests):
  - **DragMovement (T25).** `movement.rs` is split into `follow.rs` and `drag.rs`. The fixed scheme is turn → drag → thrust → position, with braking planned from the stopping distance. Equilibrium speed is 8, or 4 on Rugged with double drag. The stop takes 42 su and 16 ticks.
  - **0035.** Any blocked move zeroes the whole momentum, and a Drone creeps at 0.8 su a tick along a wall without tripping the stuck rule. A debug teleport zeroes momentum too.
  - **0126.** `movement/separate.rs`: an idle unit that yields steps off the lowest-id unit it overlaps, at 2 su a tick, until the boxes touch. Units with orders are never moved, and stacks of up to 8 separate.
  - **Steering.** H2's dead-centre oscillation is fixed for momentum units: at least 9 loads a minute at every offset, against 0–1 before.
  - **Nav.** One grid per set again (`GroundRugged, 1`), so nav rebuilds take 1.0–1.1 ms instead of 1.5–1.8.
  - **Curve** (income per second, worst gap in ticks):

    | Drones | 4 | 8 | 12 | 16 | 20 | 24 |
    |---|---|---|---|---|---|---|
    | Income | 6.34 | 11.65 | 17.86 | 23.10 | 25.57 | 27.26 |
    | Worst gap | 90 | 104 | 130 | 127 | 305 | 357 |

    The design's table gives 6.1, 23 and 32. 24 Drones are 15% under, and a redirect bounce remains; H3's 0132 rule should end it.
  - **Perf.** `movement_perf` shows no difference. The 24-Drone soak costs 25 µs a tick against 13.7 before.
  - **Accepted deviations.**
    - Momentum is hashed only while non-zero.
    - The Drone faces the step it wants.
    - Sidestep, exit and separation stay kinematic.
    - Corners are taken at speed.
    - Neutral units never separate.

- **M3-B Shields and Weapon Conversion** (merged in `61aa089`; 1111 tests):
  - **Shields (T18).** `Object::shields` is hashed. In `damage::resolve`, Shields absorb first at full value, and only the overflow takes modifiers, armour, direction and the minimum (0147). `Phase::Shields` regenerates 0.25 a tick up to `max_shields()`, for neutral and transforming objects too (0146). During a Warp In, Shields follow the maximum, and a cancel keeps their fraction (0110, 0111).
  - **Convert (T26).** `Ability::Convert` takes 4 ticks. Orders are kept (0150), Shift queues nothing (0151), and the new weapon starts ready (0152). In a group, units in Firing or Cooldown are left out (Q0042 point 9 proposal).
  - **Client.** A shield bar above the HP bar, "Shields a / b" in the InfoPanel (0148), and R Convert.
  - **Code.** The Modes step moved to `sim/modes.rs`. The goldens were re-blessed for the new hashed fields; the games are otherwise identical. `behaviours_perf` shows no measurable difference.

- **M3-F Beacon Deployment** (merged in `ff119a8`; 1140 tests):
  - **Order and gate.** `OrderKind::Deploy { origin }` goes to one Drone. The gate is `Sim::deploy_check` (`Method::WorkerBuilt`, `Mode::Known`, 0120). The truth is checked on arrival (`Mode::Arrival`; 0117, 0118), and a failure emits `SimEvent::DeployFailed`.
  - **The transform.** The Beacon exists from the first frame (0109), and deployment takes 48 frames. The load is kept during it (0114).
    - Capacity: the Drone's 1 is freed at the start and the Beacon's 5 arrives at completion (0112).
    - The Beacon counts for defeat (0113) and accepts only Cancel (0116).
    - A cancel restores the HP fraction (at least 1), the Shields fraction and the load.
  - **Reservations (0125).** `Sim::pending_deploys` reserves sites against the owner's Deploy and Warp In (`TileProblem::Reserved`), and doesn't block units. Drones are Groupable.
  - **Readings.**
    - A failed Deploy ends only that order.
    - A cancel returns the load but not the AssignedDropOff.
    - An own unit on Hold on the site fails the order.
    - A Deploy refused at the gate emits no event.

- **M3-H3 gathering rules** (merged in `1e43600`; 1156 tests):
  - **Occupancy.** It is never stored on the patch. A patch is taken by a Drone mining it, by a one-tick claim, or by a Drone waiting within MiningReach; the lowest id mines next (0133).
  - **The search.** 6 GU around the Drone, centre to centre, over known, unplated, reachable patches (0131, 0137, 0141).
  - **The loop.**
    - The Gather follows the last mined patch (0132), and waiting Drones re-check together every 16 ticks.
    - A gone patch's Drones head for its spot and search there (0134, `Gather::spot`).
    - A Gather ends only when no patch is left in range, delivering first (0123).
    - Drop Off is a delivery only, and the gate refuses an empty Drone (`NothingToDeliver`; 0127, 0140).
    - Loads survive other orders, and a loaded Drone sent to a patch turns round there (0128–0130).
  - **Tests.** `tests/gathering_rules/` has 17 tests.
  - **Curve** (per second): 12 → 17.2, 16 → 23.1, 20 → 24.5, 24 → 26.2. The worst gap is 428 ticks, within the restored 480 bound. The 0132 re-check costs about 1/s at 20–24 Drones.
  - **Open.** A node lost mid-return makes the Drone re-target at once, against 0135's "not continuously": Q0046. Empty Drones idle where the field ran out.

- **M3-P capacity at production start** (merged in `9f63a5f`; 1168 tests):
  - **Queuing (0143, every faction).** Queuing checks only crystals and queue room. `Reject::UnitControlCap` is gone, and 0083 ignores the cap.
  - **Production start.** `sim/producers/step.rs` checks `available − used ≥ cost` when production starts. "Used" counts units alive plus units in production.
  - **Waiting.** Without room, a producer waits at 0% (`Production::blocked`, hashed only while true) and starts by itself when room appears. A unit in production always finishes, and a destroyed producer frees only that unit.
  - **Notification placeholder.** `SimEvent::ProductionWaiting` shows the cap's name in the feedback band, and the InfoPanel says "Waiting for …". Train availability depends only on crystals and queue room.
  - **Unchanged.** The goldens and the economy oracle; `structures_perf` shows no regression.
  - **Open details.** Lowest id first, no start at power 0, and a zero cost still waits: Q0047.

- **M3-J1 Warp In and Deploy UI** (merged in `864c13f`; 1195 tests; client only, goldens unchanged):
  - **Beacon.** Q Basic and W Advanced Warp In menus; buildings not in the game hold unavailable slots. The Warp In ghost stands on the Beacon (`warp_in_footprint`), turns and flips about its centre with R/F, and is checked by `warp_in_check`. Tab picks which Beacon gets it.
  - **Drone.** Q Move, A Deploy, X Stop. The Deploy ghost follows the cursor, checked by `deploy_check` for the Drone the click goes to. Shift chains through the group (0125). Lingering ghosts show the local player's `pending_deploys`.
  - **Transforming objects** show Cancel (X) only (0116). **0162:** a greyed-out button or hotkey sends `CommandRefused`, shown in the feedback band.
  - **Structure.** `Placing` has a `Builder` (constructor, Drones, Beacon). The panel tests were split under T28; client tests are in `tests/commands/colonists/`.
  - **Readings.** Another command pressed while targeting says "Cancel the current command first", and a portal group only partly warping in shows the portal panel: both in Q0048.
  - **Cosmetic.** The highlighted "Warp RP" button wraps onto two lines.

- **M3-D2 portal energy doorway** (merged in `3576cbe`; 1202 tests; goldens unchanged):
  - **0145.** `ExitRule::Centre` replaces `Nearest`. `production/exit/energy.rs` (`EnergyDoorway`) takes the open arc from the placed sides.
  - **Heading.** Toward the rally point if that way out is free; else the nearest free open direction (32 headings, ties clockwise); else the nearest open one. With no rally point it draws with `Rng::below`.
  - **Walking out.** Units walk out straight along `Mover::exit_heading` (hashed only while set). A unit that can't get out stays half out, as `structures.md` says.
  - **Unchanged.** The Barracks keeps its side doorway. Perf (`structures_perf`, `movement_perf`) shows no regression.
  - **Readings.** A release also waits while a unit overlaps the centre box. The rally order is given at release. A half-out unit's new order fails until its exit point is clear. K2 checks that it can still attack, which `structures.md` requires.

- **M3-V game end, costs, arrival radius, zero-cost capacity** (merged in `9516a0f`; 1213 tests; goldens unchanged):
  - **Draw (0163).** `Outcome::is_draw` is derived, and everyone sees DRAW.
  - **Results screen (0164).** `hud/results.rs` shows DEFEATED, the name and faction, and the defeat time. It is rebuilt every frame, so F2 and replays work.
  - **Robo Portal (0167)** costs 250, refund 187.
  - **Arrival radius (0169).** `OCCUPIED_ARRIVAL_GU` is 3.0, for points and objects. Crowd-sweep failures went from 43 to 7, kept as the ignored `movement_crowds.rs::crowd_sweep`.
  - **Zero-cost capacity (0176 point 3)** is in `ArmyCap::fits`. The `step.rs` comments cite 0175 and 0176.
  - **Soak seed.** The `production/stalls.rs` soak seed is now 0xd00d, because the old seed gave no waits at 3 GU.
  - **Placeholders.** The results screen's contents, and the end banner over it.

- **M3-R one default-order rule, remembered targets, Riot Shield orders** (merged in `e2975db`; 1243 tests):
  - **One rule (0136).** `orders::resolution::resolve` decides every default order. `Sim::default_order`, `Sim::rally_order` and the client's `command_ui/resolution.rs` all use it. A rally gives the new unit its own right-click order at release, keeping M3-D2's timing.
  - **Remembered targets (0139).** `targets::TargetView` (seen, remembered or gone). The gate takes seen or remembered targets. An order goes to the last-known position, and ends as for a vanished target when the record is dropped.
  - **Riot Shield orders (0150, 0151).** Orders are kept: Attack runs as a Follow while shielded, and Shift does nothing. `riot_shield::allows_attack` and `is_going_up` are the one shield check, for M3-S.
  - **Deferred Convert (0153).** A Convert pressed mid-attack waits in `AttackState::pending`, applied in `step_modes`. The pending ability is reusable for M3-S's 0172.
  - **CanHaveTurret (0154).** **Seeded records (0159):** `Vision::seed`; path knowledge no longer assumes objects. **Self-follow (0168)** is a Stop.
  - **Goldens.** `gdo_opening` was re-blessed for 0159 only; the per-tick object digests are unchanged. Perf: B/A 0.94–1.02.
  - **Found.** An AttackMove can loop forever: on `behaviours_soak` seed 1 a Guard takes and drops an edge-of-sight target every other tick. The soak now runs seeds 2–4, and M3-L2 fixes the loop. Master's soak also fails seeds 8, 12, 21 and 34 of 1–40: M3-X.

- **M3-U unfinished groups and polish** (merged in `d3814be`; 1255 tests; client only, goldens unchanged):
  - **0185.** SelectionGroups are keyed by `GroupKey { kind, unfinished, instance }`. An unfinished group shows Cancel only, with a tinted portrait and "(unfinished)" in the InfoPanel. `ControlState::set_type` moves a completed building into its finished group. The ActiveGroup on completion is a reading: Q0050.
  - **Draw order.** Units draw in their own layer above structures, with selection rings between, so a unit leaving a portal is visible. A click picks a unit over a structure it overlaps.
  - **Labels.** No command label wraps; long ones shrink. "Raise Shield", "Lower Shield" and "Cancel Ready" shrink to about 6 px at 720p: M3-S shortens the shield texts, K2 the rest.
  - **Splits.** `tests/game_end/` and `sim/tests/movement_yield/`, code moved only.

- **M3-L1 fog delay and attacker reveal** (merged in `aeb54ae`; 1266 tests):
  - **Fog delay (0178).** `vision/linger.rs`, `FOG_DELAY_TICKS` 8. A per-player bit and end tick per tile, plus a FIFO of expiries, so the cost follows only the tiles that change.
  - **Reveal (0177).** `vision/reveal.rs`, `REVEAL_TICKS` 24. A `BTreeMap<ObjectId, Tick>` per player, renewed by each shot, revealing to the victim's owner only.
  - **One query.** Both sit behind `PlayerVision::object_box_state` and `sees_object_box`, which `Vision::sees`, `World::sees`, combat, the gate and the client's `Sight` all use.
  - **Client.** Tracers are drawn only over Visible tiles (`visible_spans`), and the old hit flash is gone.
  - **Goldens.** All 7 re-blessed, since the hash covers the new history. Objects are identical at every checkpoint. Perf: no regression in vision, movement, behaviours or structures.
  - **Open.** A target that outruns sight is lost at once: Q0051. `fog_memory.rs` is 590 lines (K2 splits it).

- **M3-J2 gathering UI** (merged in `298f364`; 1283 tests; client only, goldens unchanged). **First playable:** the Colonists play from the UI alone (`docs/playtest/m3.md`).
  - **Right-click and pointers.** The Drone's right-click goes through M3-R's one rule. GatherResources (cyan), ReturnResources (gold) and Mixed pointers.
  - **Commands.** S Set Drop-off sends `Ability::SetDropOff`; an invalid target says "Invalid target". Egress Node F selects `Sim::returning_drones`, and is hidden when there are none.
  - **Load.** A load marker on the Drone, and the owner-only InfoPanel lines "Carrying 8 Space Crystals" and "Drop-off: Egress Node".
  - **Panels.** The 0124 grids, with the portals' TBD slots unavailable ("Not available yet"). The 0155 plate panel. A green ring on a patch a portal rallies to.
  - **Readings.** Q0052: enemy right-click, the Set Drop-off pointer, who sees the load, Gather indicators.
  - **Bug, K2.** Object markers (the rally ring, the Move marker) follow a remembered target's live position, which leaks it (0139).

- **M3-S Riot Shield as two unit types** (merged in `08f09a6`; 1298 tests):
  - **Two types (0173).** `ObjectKind::PeacekeeperShielded` has MaxSpeed 2 and the same cost and Unit Control. The Barracks trains only the Peacekeeper.
  - **The toggle (0171, 0174).** The type switches at the toggle's start via `change_kind`, which keeps orders, movement and attack state. The toggle takes 6 ticks, and a second press reverses it from where it is. During it the unit moves at half speed and can't attack; the reduction applies only when fully up.
  - **Mid-attack (0172).** A raise pressed during Firing or Cooldown waits in `AttackState::pending`. The conversion machinery isn't reused: it switches at the end and can't reverse. The two share `change_kind`, `pending` and the Modes step.
  - **Client.** R is Raise on the Peacekeeper and Lower on the Shielded, which hides Attack (`templates::hides_attack`, so 0075 and 0076 work unchanged). An InfoPanel hint names the hovered button in full. The portrait label is "PKS".
  - **Goldens.** Only `riot_shield` was re-blessed, at its shield-up checkpoints. Perf is unchanged.
  - **Readings.** Q0053: orders while lowering, Lower during a waiting raise, the ActiveGroup during a deferred raise.

- **M3-L2 target priority, AttackMove leash, MinRange, Guard laser** (merged in `f039e5c`; 1314 tests):
  - **0179.** `World::pick` ranks targets by `rotation_step` (45° steps), then distance, then id.
  - **0180.** A target in weapon range is always fought. An out-of-range one is chased only within 6 GU of the PathReference. The reference is kept after fights and replaced by a new path mid-leg.
  - **Loop fix.** A target dropped while alive is left alone for the rest of the leg (`Leg::dropped`), which ends soak seed 1's loop and seed 29's two-target variant. It isn't in the design: Q0054.
  - **0182.** Scans skip targets inside MinRange. An explicit Attack backs a mobile unit off to the middle of the band; an immobile one gets `SimEvent::TargetTooClose`, shown as "Target too close". Test-only defs are in `defs/test_defs.rs`.
  - **0186.** The Guard's laser: 18 damage, 4/1/3/16, Universal.
  - **Goldens.** duel_3v3, riot_shield, patrol, attack_move_battle and gdo_opening were re-blessed, each change traced to its entry.
  - **Soak.** Seed 1 is back in `behaviours_soak`, and an ignored `soak_sweep` covers seeds 1–40. Only seed 5 fails, on the settle bound with units still walking, not looping (master failed 5, 8, 11 and 34). Perf: same per-tick cost, longer battles.

- **M3-K1 Colonists economy oracle and goldens** (merged in `f73262a`; 1318 tests):
  - **Code.** `testing::colonists`: a build line with a Drone every 10 s and a deploy whenever Beacon room falls to 1 (Drones 5, 11, 17, 23), and `reference_field()` (0107). The oracle is in `tests/economy_oracle/`; `docs/economy_oracle.md` has both factions' tables.
  - **Measured.** 6.17 / 10.29 / 12.57 / 14.86 / 17.52 / 20.57 / 22.10 / 23.62 / 24.38 SC/s at 0:00–4:00.
    - 1:00 (−16.2%), 3:00 (−15.3%) and 4:00 (−23.6%) are outside ±15%.
    - Targets 1 and 4 fail; full saturation needs 27 Drones.
    - Pinned as measured, with no game number changed (0108): Q0055.
  - **Goldens.** `colonists_opening` (to 1:50) and `colonists_vs_gdo` (to 3:00, re-blessed after M3-L2 for 0179). There are 9 goldens now, in `tests/golden/`.
  - **Soak game.** `testing::colonists::skirmish` is the 10-minute game for K2.

- **M3-K2a fixes** (merged in `e481b24`; 1328 tests; goldens unchanged):
  - **Markers (0139 leak).** Markers and the InfoPanel's rally line go by `Sight::target_view`, so they no longer show a remembered target's live position.
  - **Drop-offs (0170).** `Sim::reassign_drop_offs` runs on the removal path, so nothing checks every tick. After a combat death the new drop-off shows one tick earlier than before (accepted: "immediately").
  - **Half-out units.** A unit half out of a portal attacks, takes an Attack, and can be selected and attacked; no fix was needed.
  - **Labels.** Every label is six characters at most, at full size at 720p ("Drop" for Set Drop-off), and a hover hint names each button in full.
  - **Cleanup.** `fog_memory.rs` is split, and `riot_shield::is_going_up` removed. Gathering perf is unchanged.

- **M3-K2 final QA** (merged in `3559b40`; 1329 tests, 14 ignored):
  - **Perf.** `drones_perf` (2 × 40 Drones gathering and 20 Legionnaires fighting, release) runs at 0.10 ms a tick, p99 0.12–0.21 ms. Movement takes 0.053 ms, vision 0.025 and behaviours 0.013. All 14 ignored tests pass in 49 s.
  - **Soak.** `skirmish_soak` plays 10 minutes of `colonists_vs_gdo` twice, plus the replay. There is no deadlock or stuck order, the invariants hold every tick, and the hashes match every 64 ticks. `soak_sweep`'s settle bound is now per order in flight, and 40/40 seeds pass.
  - **Fix.** Structure HP and Shields bars span the top edge of the footprint.
  - **Docs.**
    - `docs/playtest/m3.md`: how to start, F2, the debug keys, things to try, placeholders, known issues and open questions.
    - `m2.md` is refreshed.
    - ARCHITECTURE.md is checked against the code, and "Adding a faction" is finished.
    - No test file is over 500 lines.
  - **Walkthrough findings:**
    - the last Peacekeeper of an attack-move column is stuck and never fires (M3-G2);
    - a revealed attacker's tracer starts at the fog edge (Q0056);
    - M3-J3 polish:
      - a marker hides a unit's label;
      - the build-area tint turns unexplored ground lake-blue;
      - F2 keeps the camera, so the other side sees black;
      - Peacekeepers have no map label;
      - "SC" labels both the Scout button and a patch;
    - the Minimap and Portrait are empty (the design has no spec beyond "Always visible");
    - a conversion shows only a greyed R (placeholder).
  - **The waiting Drones' order.** At over-saturation, lowest id first starves late Drones (one waited 1984 ticks).
    - The design leaves the rule to us (`units.md > Patch Occupancy` point 4).
    - Consider longest-waiting first once Q0055 is answered, since it would move the pinned oracle figures.

- **M3-G2 stuck behind friends** (Fable; merged in `d98ae13`; 1343 tests):
  - **Cause.** A unit chasing its attack's target behind its own side's standing attackers "queued" (`docs/movement.md` 6.5 rule 2), and only waited. Units in a firing line stop a step out of line, so the unit behind stepped to and fro in the pocket between two of them.
  - **Fix.** A queueing unit detours round the row of standing units, to the end on the side facing the target (`movement/round.rs`, `stuck.rs`, `collide/hold.rs`, `Mover::go_round`, `Mover::detour_end`). Each attempt counts against the leg's three. With no way round, the unit stands still and never fails. Plain moves are unchanged.
  - **Also.** `sim/removal.rs::forget` now stops a walk to a last-seen point when its order ends. This was an older bug, found through soak seed 17.
  - **Tests.** `attack_behind_friends.rs` has 8:
    - four Peacekeepers in a column all fire within 200 ticks, in 16 layouts;
    - also covered: eight Peacekeepers, Guards, lines, and corridors with no room.

    One case isn't covered: eight Guards on an AttackMove, where the last is still walking round in 1 of 12 layouts.
  - **Goldens and perf.** `attack_move_battle` and `colonists_vs_gdo` were re-blessed. The perf counters are identical, and the timings are unchanged within the noise.
  - **Question Q0057:**
    - melee attackers behind their own side, within 3 GU, give the target up (2–3 of 8 Sabres never strike);
    - whether a unit should wait without limit;
    - whether three attempts at a way round are right.

- **M3-L3 target lost ends the attack** (merged in `33b3e3f`; 1349 tests):
  - **0200.** An Attack or Follow whose target leaves vision ends at once, after the 8-tick fog delay. The unit stops where it is, or its next queued order starts that tick.
    - 0034's walk to the last seen position is gone, along with `last_seen`/`lost`. The new `AttackOrder::seen` is `#[serde(default)]`.
    - A target remembered when the order is given is still walked to (0139).
    - A detour round friends (M3-G2) ends where the unit is.
  - **AttackMove.** It keeps M3-L2's placeholder: a target lost to the fog counts as dropped for the rest of the leg, and is taken again only within weapon range. Taking it again normally loops near walls (soak seeds 8 and 29 fail), so this went to Q0058.
  - **M3-G2's `removal.rs::forget` stop is removed**, since no walk to a last-seen point is left.
  - **Goldens.** `duel_3v3` changes only in a hash. In `fog_chase` the Peacekeeper stops at (404,352). `soak_sweep` passes 40/40.

- **M3-T toggles set a target state** (merged in `bcd66ab`; 1370 tests; goldens unchanged):
  - **0198.**
    - A press switches the unit's type (`Object::kind`) at once through `change_kind`, even while it waits behind a shot. The physical form (`Conversion{form,left}`, `RiotShield::up`) catches up in the Modes step.
    - **Per unit:**
      - a waiting unit takes the new target, or drops the change if it is back at its form;
      - a unit mid-change finishes, then changes back;
      - a unit already changed changes back.
    - **One read each:** `conversion::form`/`combat::weapon`, `riot_shield::mode`/`allows_attack`, and `takes_attack_orders(kind)` for the type.
    - `AttackState::pending` and `Reject::CannotConvert` are gone. ARCHITECTURE.md has a new section, "Object model > Type and form".
  - **0202.** The gate, the default command, the panel and right-click all go by type. A lowering Peacekeeper takes an Attack and holds it until the shield is down. A raising one shows no Attack, and right-click makes it follow.
  - **0199.** `Selection::fix_active`: when the ActiveGroup's last member dies or leaves the selection, the next group in Tab order becomes active, wrapping. A type change activates the joined group in DefaultState.
  - **Client.** `ControlState::toggle` shows the press at once, `PendingCommands::kind_of` covers the gap until the sim ticks, and units are drawn by form.
  - **Readings:**
    - Convert is each form's own button (`PanelCommand::Convert(target)`), so it goes only to the ActiveGroup.
    - The player deselecting a group's last unit counts as "left the selection".
    - A save from before 0198 loses a pending press.
  - **Perf.** `behaviours_perf` A/B shows no regression.

- **M3-K3 Drone acceleration** (merged in `c4807b8`; 1372 tests):
  - **0205.** The Drone's constants are 0.9 / 0.3 / 0.15. It reaches 67.9% of top speed after 7 ticks and 96.1% after 20. A Stop drifts 25.5 su, where it drifted 41.5 before.
  - **The oracle, re-measured:** every figure is within ±15% (6.93 / 11.43 / 13.33 / 16.00 / 18.67 / 22.86 / 25.14 / 26.29 / 27.81). Targets 2–4 hold, and target 1 fails at 2:20 only. 24 Drones saturate the row. The figures went to the Designer as Q0072.
  - **Goldens.** `colonists_opening` and `colonists_vs_gdo` were re-blessed. The soaks are re-pinned, and `docs/economy_oracle.md` keeps the tables from before 0205.

- **M3-G3 no indefinite wait** (merged in `8480ee0`; 1374 tests; goldens unchanged):
  - **0223.** A unit queueing behind its own side, with no way round or no attempt left, follows the Failure rule. It gives up STUCK_TICKS after its third attempt, about 84–112 ticks after it is blocked. M3-G2's wait is deleted (`movement/collide/hold.rs` and `World::hold`). Only `movement/` changed.
  - **Tests.** `attack_behind_friends/no_room.rs` covers 0223's Verification:
    - in the corridor, the three units behind give up and go idle, and fire once the front unit is gone;
    - of eight Sabres, 2–4 give up and take the target again by themselves.
  - **Handed to M3-L4:** the AttackMove re-take cycle (0235), and the explicit-Attack re-take in the same tick, which should be 32 ticks (0181).
  - **Perf.** `movement_perf` is unchanged, and its counters are identical.

- **M3-L4 AttackMove reach rule** (Fable; merged in `b261863`; 1380 tests):
  - **0230 and 0235.** In `orders/reach.rs`, an enemy out of weapon range is taken only if its chase path passes the check:
    - the path is planned on the owner's known grid, and walked in 16 su steps up to the first firing position;
    - every step stays within the leash and keeps the target within SightRange.

    `Leg::dropped`, `Leg::ignored` and M3-L3's fog-as-dropped are gone. Patrol uses the same rule.
  - **The rejection cache** (`Leg::rejected`, hashed only while it is set).
    - A rejection is judged again when:
      - the enemy moves `REJUDGE_MOVE_SU` (64) or more;
      - the knowledge version changes;
      - PathReference is replaced by a different route (`SAME_ROUTE_SU`, 32).
    - It holds three kinds of entry:
      - failed chases (0235);
      - chases that end beyond the leash;
      - targets lost to the fog during a chase, cached at the position where they were judged, never at the hidden one. This is a technical choice: the path check's sight is about 50 su more lenient than tile vision.
    - The cache holds at most 64 entries.
  - **Cost.**
    - Each unit plans at most 2 judgement paths per tick (`JUDGE_PATHS_PER_TICK`), in priority order.
    - A judgement's search stops past 2 × (SightRange + 1) tiles of path (`nav::find_path_within`), and the enemy is rejected.
    - `attack_move_perf` (100 Guards past 100 Drones across a river) plans 3.87 judgement paths per tick, at a mean of 0.11 ms per tick.
    - In the A/B runs, `behaviours_perf` and `movement_perf` show no regression.
  - **Folded in (0181).** A target given up by an explicit Attack waits 32 ticks before idle auto-targeting takes it again.
  - **Goldens.** `patrol`, `attack_move_battle` and `colonists_vs_gdo` were re-blessed. Each diverges where a leg first takes an out-of-range enemy under the new rule.
  - **Open:** Q0074, own movement as a trigger.
- **M3-J3 markers and polish** (merged in `827dfeb`; 1389 tests; client only, so the goldens are unchanged):
  - **0201 point 4, the markers:**
    - Gather: cyan corner brackets on the patch;
    - Drop Off: a gold ring with arrowheads on the node;
    - Set Drop-off: a violet dashed ring on the AssignedDropOff, shown while the workers are selected.

    The cursors use the same colours.
  - **0206.** A shot is drawn whole while `Sight::reveals(attacker)` holds; otherwise it is clipped as before. The case of a fogged target is Q0075.
  - **M3-K2's polish:**
    - unit labels sit above markers and stay upright;
    - the build tint leaves out unexplored tiles;
    - F2 keeps a camera for each player;
    - the Peacekeepers are labelled PK and PS, and the Scout button ST.
  - **Known issues left** (`docs/playtest/m3.md`):
    - a Move marker on a structure covers its label;
    - a soft tint fringe, about half a tile wide, next to unexplored ground.

- **M3-K4 economy table** (merged in `f47fb26`; 1389 tests; tests and docs only, so game numbers and goldens are unchanged):
  - **0236's table** is `economy_oracle/main.rs::TABLE`, and each figure is checked within ±15%. All are within 0.5%.
  - **Target 1, as restated, holds:** the Colonists lead from 0:00 to 1:40 by 0.26–1.43 SC/s.
  - **Saturation:** 24 Drones, 3 per patch, mine 32.00 SC/s.
  - **The GDO column** in the design table is the GDO line measured on the Colonists' 21 s window. M2's own 9 s check is kept as `gdo::model`.
  - **The docs** of `gathering_soak.rs` and `ARCHITECTURE.md` cite 0236.

- **M4-A Syndicate foundation** (merged in `fcd06dc`; 1412 tests; goldens unchanged):
  - **Sim.**
    - The Agent fights with its pistol.
    - The Tunnel is one kind with a tier (T39). `Object::tunnel` is hashed only while it is set, and MaxHP follows the tier. `tunnel::set_tier` keeps the damage taken, and `DebugCommand::SetTier` lets QA set a tier.
    - The Hideout is an underground expansion: no flat ground is needed (0005), and its `BlockerKind::UndergroundExpansion` blocks nothing.
    - **One hidden predicate,** `vision::hidden_from` (`Sim::hidden_from`). Targeting, `World::sees`, the gate, the records and the client's `Sight` use it: a hidden object is capped at Explored and never recorded. Damage never asks it (0218, 0262).
  - **Folded in:**
    - 0248: hatches on B and D;
    - 0254: the Hideout's sight 0, and Groupable;
    - 0258: enemies see and remember the tier;
    - 0262 and 0264: area damage hits underground objects, and the Hideout's armor is 6/24.
  - **Client.**
    - Labels AG, T1–T3 and HI.
    - Own underground structures are drawn as translucent outlines.
    - The Tunnel's look by tier and sides is in `object_render/tunnel.rs`: path strips on A, hatches on B and D, air-door circles at Tier 3.
    - Syndicate lasers are red, through `ShotStyle::of`.
  - **Perf.** Unchanged, except a +3% two-tick "group orders" sample whose cause is unconfirmed.
  - **Deviations.**
    - Maps have no per-object tier, so QA sets one with SetTier.
    - `sim.rs` is at 602 lines and wasn't split.
    - The no-fog debug views draw enemies' hidden Hideouts.

- **M3-L5 own-movement re-check and own-target shots** (merged in `a777ae3`; 1420 tests; goldens unchanged):
  - **0243.**
    - Reach-rule rejections, and chases that end beyond the leash, record where the unit judged (`Judged::from`, hashed only while set).
    - They stand only while the unit is within `REJUDGE_SELF_SU` (64 su) of that point.
    - Failure-rule give-ups and fog losses keep 0230's triggers. `no_room` guards that: with the trigger, the eight Sabres cycle.
  - **0243's cost** (release; 12 interleaved rounds; 100 Guards attack-moving past unreachable targets):

    | Targets | Judgement paths per tick | Mean tick |
    |---|---|---|
    | Drones | 3.9 → 28.5 | 0.107 → 0.144–0.154 ms (1.35–1.45×) |
    | Standing Generators | 2.8 → 6.9 | 1.06–1.07× |

    The worst tick is unchanged (1.93 ms, the same tick on master). Steps of 2 GU and 2.5 GU would give 1.16–1.24×. **The Director kept 1 GU:** the absolute cost is tiny.
  - **0244.** `Tracer::target_owner`. A shot is drawn whole only if its attacker is revealed and its target is the viewer's. Tests cover the neutral and three-player cases.
- **M3 is complete,** except 0203, which waits for the Rocketman's MinRange value.

- **M4-D Agent crystal gathering** (Fable; merged in `edc30e11`; 1439 tests; goldens unchanged):
  - **One Gather job for the Drone and the Agent** (T34). `gather.rs` is split into `claims.rs`, `patch.rs` and `deliver.rs`.
    - `WorkerDef` has a `LoadDef` per resource (Supplies `None` until I), plus `shares_patches` and `keeps_drop_off`.
    - `StructureDef::drop_off` is `Anywhere`, or `Hatches { crystals: B, supplies: D }`.
  - **The hatch.** `economy/drop_off.rs::hatch_square`, re-exported as `tunnel::hatch_square`, reads the square off `sides()`. It is the middle square farther from Side A (placeholder, Q0076).
  - **The line.**
    - `GatherPhase::AtHatch`: one Agent unloads at a time, in order of arrival tick, ties by id.
    - Waiting Agents stand side by side along the wall within reach.
    - An Agent out of reach retries every 32 ticks for ever (0255).
  - **The Tunnel.** The nearest by its hatch centre, taken as the first return starts and kept for the Gather. A new Gather picks again. Set Drop-off is refused for Agents. "Operational" is `!is_transforming()` for now: **E must exclude sites**.
  - **Measured.**
    - Trip times from 0 to 6 GU are within 15% of the table, and the 12.5 SC/s ceiling is exact.
    - From 3 GU on, incomes are within 15%.
    - **At 0–1 GU, 3 or more Agents jam head-on** (5.33 SC/s), and at 2 GU they reach only 9.8–10.0: Q0076.
    - The soak (stacked Tunnels on opposite sides, 2 GU, 10 min) holds 25.00 SC/s every minute.
  - **Perf.** `drones_perf` is unchanged, and the Colonists oracle passes.

- **M4-C Tunnel Network** (Fable; merged in `f67df212`; 1475 tests; goldens unchanged):
  - **The held store.** `network.rs`, `Sim::held`, and `take_off_map`, `put_on_map` and `remove_held`. Hashed and saved only while non-empty.
  - **Enter.** `OrderKind::Enter`, with `Sim::step_entries` after 5a. A unit goes in the tick its box is within 8 su of the entry spot, with no hold (0246): 20 Guards file in 8 ticks apart. Any unit of the Tunnel's owner may enter.
  - **Eject.** `Command::Tunnel { Eject }`, queued in `TunnelState::eject`. The longest-inside unit goes first, and a unit comes out once the spot is clear (0250).
    - An Eject while every unit of the type is in transit is refused (`InTransit`; Q0077).
  - **Production and rallies.** `ExitRule::Network`, the Hideout's Agent recipe, and rallies (0252). Auto-ejected units share the queue (0253).
  - **`Sim::step_network`.** Regeneration (0249), then ejection.
  - **Held units count** in `Sim::army_cap`.
  - **The client.** Enter on right-click. Held units leave the selection and stay in their control groups. The Hideout gets the Production template.
  - **Crowd:** 40 Guards all entered (0 gave up) and were ejected in 578 ticks. `movement_perf` is unchanged.
  - **Notes for later slices:**
    - B folds `network::area` and `network::parent` into `tunnel/area.rs`;
    - H calls `Sim::remove_held` for stranded units;
    - J1 owns the Enter pointer and indicator;
    - R splits `sim.rs` (648 lines);
    - M judges the units that idle 1 GU out (Q0077).

- **M4-B Syndicate start** (Opus; merged in `35e19634`; 1506 tests):
  - **The Starting Conditions:** a Tier 1 Tunnel, the Hideout, an Agent, 100 SC and 1 Supply, with the camera on the Tunnel.
  - **`dev_skirmish` Syndicate layouts** in both slots. The crystal hatch faces the field, 1 GU from the patches (Q0076 is noted in the layout comment).
  - **Tunnel Space** is a derived army cap (`economy/army/tunnel_space.rs`), and counts units in the network.
    - The HUD shows it, in red when over capacity, with "Tunnel Space limit reached" feedback.
    - Own areas that overlap (only spawns can make them) count their shared cells twice, as the formula reads.
  - **0242's area rule** in `placement::check` and `check_layout` (`TunnelAreaOffMap`, `TunnelAreaNotDrillable`), and the layout rule for expansions (`OutsideTunnelArea`). The editor has the Syndicate palette and the rule texts. A Tunnel 2 tiles from the edge is valid (Q0078).
  - **One area function.** C's `network::area` and `parent` are folded into `tunnel::area::parent`.
  - **The building-armor test** (0264): all 11 buildings follow PointArmor n / FullArmor n × size, with no exceptions.
  - **The Hideout** is judged as a Direct (Buildable) placement until M4-F (Q0080 may change it).
  - **Tests that used the Syndicate** as the unimplemented faction now use the Cults.
  - **Goldens re-blessed:**
    - attack_move_battle, duel_3v3, fog_chase, patrol and riot_shield: their P2 is the Syndicate, which now starts with 100 SC and 1 Supply. With those banks zeroed, all five match their old hashes.
    - gdo_opening, colonists_opening and colonists_vs_gdo: only the embedded map's slots changed. Every checkpoint hash is identical.
  - **Perf:** placement_check, structures_perf and drones_perf are unchanged in interleaved A/B runs.

- **M4-I Supplies** (Opus; merged in `9aca8972`; 1529 tests; goldens unchanged):
  - **The one Gather job covers stations.**
    - The pick-up reuses Mining: 48 ticks. It claims one unclaimed Supply in 5a, and the Supply leaves in the pick-up's last tick.
    - Each worker takes a free spot round the station's faces, so several pick up at once.
    - The return goes to the nearest Supplies hatch (Side D), one Agent per side at a time.
  - **`Worker.resource`.** Hashed and saved only while it is Supplies. The Agent's Supplies `LoadDef` is filled in.
  - **Switching resources** loses the old load when the new pick-up or mining ends. An interrupted pick-up keeps it (Q0082 point 3).
  - **An empty station.** The sim event `SupplyStationEmpty { station, owner }` goes to each player with a worker gathering there, and the client shows it in `hud/notices.rs` for 80 ticks. Waiting Agents keep their job and never look for another station.
  - **Orders.** The default right-click and a rally on a station give Gather. A station Gather with no unit that gathers Supplies is refused (`NoWorkers`). A Supply is kept through the Tunnel Network.
  - **Tests:** `tests/agent_supplies/` (18) covers 0256's Verification, plus the client's right-click and notice tests. Stations stand 3 GU or more from the hatch (Q0076).
  - **Perf.** `drones_perf` and `behaviours_perf` are unchanged in interleaved A/B runs. An early version searched every object for stations each tick, costing 3–6% when idle, and was fixed.
  - **Notes for later slices:**
    - **J1:** the load marker draws the crystal diamond for a Supply too.
    - **R1:** `player.rs`'s `supplies` doc is stale.
    - **M:** six Agents coming from one side: one boxed in behind an Agent that is picking up waits for it to finish (the mover's occupied-arrival rule), so the test asserts 4 or more at once.

- **M4-R1 Code organization, the sim** (Opus; merged in `8eab2290`; 1543 tests; no golden moved, never re-blessed; 245 files):
  - **Defs per object:** `defs/<faction>/{units,structures}/<object>.rs` and `defs/map/`. Each faction's own rules (the caps and the divisor) are in `defs/<faction>.rs`. Each object's tests moved into its module.
  - **Numbers in the defs:**
    - `RiotShieldDef`, `ConversionDef` and `UnitDef::deploy` (the Drone);
    - the `TunnelDef` transit delay and regeneration;
    - the Agent's Supplies reaches;
    - the map objects' default settings.
    - Kind matches became fields: `StructureDef::onto`, `seeds_build_area`, and `Transit::of` as data.
  - **One kind registry macro** in `defs.rs`. The names and order are pinned by a test.
  - **Serde-ready defs.** `defs/tests/serde.rs` round-trips every def. The names and lists stay `&'static`, and the deserializer leaks them once (`defs/leak.rs`), so defs stay `Copy`. **Accepted by the Director for the playtest loader:** a few KB per reload, at dev time.
  - **60 behaviour constants** moved to `tuning/{combat,construction,gathering,movement,orders,paths,production,shields,tunnel,vision}.rs`, each cited or marked as a placeholder. The old names stay as aliases.
  - **Shared mechanics:**
    - `progress.rs` (`Progress`, used by Transform; hash and JSON byte-identical) and `toggle.rs` (Riot Shield and Conversion).
    - **Deferred,** because their hashed bytes would change: production and construction progress, the Extraction Facility cycle, Weapon Conversion, the Riot Shield, mining and unloading, and the network's held units. Each moves at the next re-bless that touches it.
  - **Splits:** `orders.rs` → `orders/run.rs`; `combat/phases.rs` → `phases/tests.rs`; `sim.rs` (648 lines) → `sim/commands.rs` and `sim/phase.rs`.
  - **Citations:** 204 rewritten by 0282's table, with 0 stale in the sim. The `player.rs` supplies doc is fixed.
  - **Perf:** `drones_perf` and `nav_perf` are unchanged in 12 interleaved ABBA runs per side, and the counters match.
  - **The client:** 7 `use` lines in 6 files.

- **M4-R2 Code organization, the client and docs** (Opus; merged in `c60b8d0f`; 1546 tests; QA screenshots byte-identical; 131 files):
  - **The citation test,** `crates/sim/tests/design_citations.rs`. It fails, naming file and line, on any cited design or `docs/` path that doesn't exist, with a planted-path test. An ignored report lists cited sections that aren't headings. It passes on the whole repository.
  - **Client kinds.** `kinds.rs` and `kinds/<faction>/<object>.rs` (label, `Look`, `ShotStyle`, `PanelTemplate`), behind the exhaustive `kinds::of`. The panels are in `command_ui/interface/templates/<faction>/<object>.rs`, and 10 kind matches are gone.
  - **Splits:**
    - `templates.rs`, `hud/selection_panel.rs`, `command_ui/indicators.rs`, `object_render.rs` and `combat_effects.rs`;
    - then `fog.rs` (→ `fog/{sight,layer}.rs`) and `editor/objects.rs`.
    - No client file is over 500 lines.
  - **Client tuning:** `crates/game/src/tuning/{camera,input,feedback}.rs`. The editor's limits stay where they are, with placeholder notes.
  - **ARCHITECTURE.md:**
    - "Adding a unit or structure", "Tunables" and "Client architecture > Kinds";
    - R1's sim layout;
    - one citation rule: full paths, with a relative `> Section` only inside a module whose doc names its design file.
  - **Citations:** 0 stale. The relative and bare-name citations in ARCHITECTURE.md and `docs/` are written out in full: 54, 10 of which pointed at the wrong file.
  - **A follow-up pass is running** in the same worktree, comments only: the remaining ~630 relative citations in modules that name several design files or none, and the rule gated if that's reliable.

## In progress
  - **M4-R2 follow-up: citations in full** (Opus; same worktree): comments only, before wave 4.
- **M3 waves 2 and 3** (common brief `scratchpad/briefs/m3-common.md`; status `m3-w2-status.md`):
- **Next**:
  - **Playtest:** the user plays once all four factions are built (2026-11-24), using `docs/playtest/m2.md`, `m3.md` and later notes. This replaces separate M1–M3 playtests.
  - **M4 Syndicate:** the plan is adopted (`artifacts/director/m4-plan.md`). M4-A is running, and every later slice waits for it.
  - **The Move-onto-itself bug** is being fixed in M3-R (0168).

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
| H2 Command UI follow-ups | right-click per unit (0062/0063), Mixed pointer, Attack targeting on own or neutral objects (0064), invalid-click feedback (0065), targeting rules (0066–0069), shield interplay (Q0020) | done (`2417f70`) | Opus |
| H3 Command UI follow-ups 2 | Shift keeps targeting (0070), no smart-casting audit (0071), 16 px targeting allowance (0073), shielded units follow on right-click (0074), left out of Attack (0075), AttackMove goes to every unit (0076), any group recall resets menus (0078) | done (`80b3d88`) | Opus |
| R Riot Shield | the Peacekeeper's toggled ability, end to end | done (`9aa41b6`) | Opus |
| Q Queued group destinations | 0029/0030 | done (`bb867a5`) | Opus |
| F Fog memory | last-known structures and resources, Explored selection, hit flash from the fog | done (`167d683`) | Opus |
| G1 Crowds and nav perf | head-on crowd steering, pass-through shoving, nav_perf, group-order and battle spikes | done (`4a982a0`) | Fable |
| G2 QA tooling | replay recording and playback, golden hashes, gate records fix, marker draw order, station timer (0057), 0038 tests | done (`52f20a8`) | Opus |
| G3 Cleanup | a per-tick turn budget shared by movement and aiming; oversize files; robust soak bounds; 5×5 Guard crowds; F2 in playback | done (`7be0f39`) | Opus |

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

### M2: GDO base and economy (**done** at `3d98e29`; spec: `artifacts/director/m2-plan.md`)
- **Scope**:
  - the Deployment Center, Generator, Barracks, Extraction Facility and Extraction Plates;
  - Space Crystals, power with its penalty, and Unit Control;
  - plate mining and depletion (0038);
  - construction and Direct Placement in the GDO build area, under the flatness rule (0004);
  - Peacekeeper production with rally points;
  - path knowledge (0040–0042);
  - Starting Conditions and Starting Layouts, in the sim and the editor (0008, 0009, 0012); the Director authors `dev_skirmish`'s layouts;
  - defeat;
  - the HUD and command panels for structures.
  - Also, from 0071: placement never moves on to another worker by itself, and a GDO Shift-click places once and leaves placement.
- **Deferred**:
  - GDO supplies (Supply Tower, Chopper; Q0022 blocks them);
  - tech prerequisites (no M2 kind has one);
  - the other GDO buildings and units (their stats are TBD);
  - ConstructionHP;
  - Custom maps.
- **Technical decisions** (spec §4, T1–T13), accepted as recommended:
  - one capability per structure role (constructor, production, extractor) plus `StructureDef`;
  - per-player knowledge grids for planning, built lazily, with the true grid used for collision;
  - one pure placement check shared by the gate, the ghost and layouts;
  - an economy step after 5e's removals and before vision;
  - 2^16 fixed-point progress, with the power ratio rounded up;
  - banks in `Player`; costs taken when the command applies; power and Unit Control derived, not stored;
  - defeat when a player's owned-structure count drops to 0; a player who never owned a structure is never defeated (this keeps sandboxes and goldens);
  - **commands per capability**: `Command::Construction` (D1) and `Command::Production` (D2). T9's single `Command::Structure` was split so the parallel slices never edit one enum. `PanelCommand` in the client;
  - layout rules checked for implemented factions only.

| Slice | Content | After | Model | Status |
|---|---|---|---|---|
| M2-A | Structure foundation: 5 kinds, stat blocks, armour, orientation and sides, blocking, vision, records, combat, placeholder visuals, oriented debug spawn | G3 | Opus | done (`cbeb804`) |
| M2-B | Economy core: banks, power and penalty, Unit Control, plate mining and depletion, the economy step | A | Opus | done (`39bcf66`) |
| M2-C | Build area and placement validation | A | Opus | done (`0f8cc70`) |
| M2-E | Path knowledge: per-player planning grids, re-route on discovery | A | Fable | done (`f76db38`) |
| M2-D1 | Construction: DC and EF, Construct, Cancel, Place, refunds | B, C | Opus | done (`fda74d3`) |
| M2-D2 | Production: Barracks queue, rally point, exit spawn, Unit Control | B | Fable (exit doorway: a unit overlapping its producer, the collision exception, invariants) | done (`1a64e75`) |
| M2-F1 | Game start: Starting Conditions, layout rules (restriction, completeness, outdated), game-start banks, camera on the DC, the `dev_skirmish` GDO layouts | B, C | Opus | done (`d5a7d29`) |
| M2-F2 | Game end: defeat (units become neutral and repath), game over when one side remains, spectating with team vision, the defeat and victory banner | B | Opus | done (`e3f67a9`) |
| M2-G | HUD: resources bar, structure InfoPanels (the banner moved to M2-F2) | B | Opus | done (`69e80a1`) |
| M2-H1 | Production UI: generalised command UI, Barracks panel, rally point | D2 | Opus | done (`eccaaa6`) |
| M2-I | Editor: Starting Layout editing | F | Opus | done (`6ae2b02`) |
| M2-H2 | Construction UI: build menu, ghost, area overlay, rotate and flip, side labels | C, D1, H1 | Opus | done (`2d507b4`) |
| M2-J1 | M2 sim QA: the economy oracle with documented tolerances, the `gdo_opening` golden, a tighter `behaviours_soak` check, F1's double layout report | B, D1, D2, F1 | Opus | done (`c2b2a17`) |
| M2-J2 | M2 QA, the rest: fix `MapDef::validate` for map-placed plates off a patch (ignored repro); `structures_perf`; perf re-measured on a quiet machine; walkthrough screenshots; ARCHITECTURE review; playtest notes | all | Opus | done (`3d98e29`) |

First playable after M2-H2.

### M3–M5: other factions, basics
**Order agreed with the user (2026-11-24):**
1. Finish M3.
2. M4 Syndicate (plan being drafted; it will become `artifacts/director/m4-plan.md`).
3. M5 Cults.

The user keeps feeding unit and other designs to the Designer, so expect incremental change entries throughout. The user playtests once all four factions are complete. The art pipeline starts once the gameplay design is mostly final.
- **M3 Colonists** (**done**, except 0203's value; last slice M3-L5 at `a777ae3`; spec: `artifacts/director/m3-plan.md`, from the M3-P spike). It covers the statted opening: Drone gathering, Beacon Deployment and Warp In (Egress Node, Robo Portal, Bio Portal), portals with exits on several sides, Sabre and Rifle Legionnaires with Weapon Conversion, Shields, Beacon Capacity, Colonists Starting Conditions and layouts, HUD and panels, and an economy oracle against `Economy Targets (GDO vs Colonists)`. Refining, the tech tier and the other units are deferred (Q0043).
  - **Slices:** 12, in 6 waves:
    1. A;
    2. C, E, H (Fable);
    3. B, D, F, G (Fable, DragMovement);
    4. J1, J2 (the first playable);
    5. K1;
    6. K2.
  - **User decisions (2026-11-24):** start M3 now, and build DragMovement (G) in M3, on Fable.
  - **Director decisions:**
    - T14–T28 are accepted as written in `m3-plan.md` §4;
    - at most three builds at once, so G waits for a free slot.
  - **Waiting on:** Q0038, gathering Drones and collision. It blocks H's final behaviour and K1, but not H's start.
  - **Slice status:**

    | Slice | Status |
    |---|---|
    | A | done (`a22aa2d`) |
    | H | done (`5bd3b69`) |
    | C | done (`5c75d40`) |
    | E | done (`68ee747`) |
    | D | done (`20f584e`) |
    | B | done (`61aa089`) |
    | H2 (0106) | done (`777b9a9`) |
    | F | done (`ff119a8`) |
    | G | done (`5d61dc7`) |
    | H3 | done (`1e43600`) |
    | J1 | done (`864c13f`) |
    | R | done (`e2975db`) |
    | D2 | done (`3576cbe`) |
    | P | done (`9f63a5f`) |
    | J2 | done (`298f364`) |
    | V | done (`9516a0f`) |
    | U | done (`d3814be`) |
    | S | done (`08f09a6`) |
    | K2a | done (`e481b24`) |
    | T | done (`bcd66ab`) |
    | L1 | done (`aeb54ae`) |
    | L2 | done (`f039e5c`) |
    | K1 | done (`f73262a`; Q0055) |
    | K2 | done (`3559b40`) |
    | L3 | done (`33b3e3f`; Q0058) |
    | G2 | done (`d98ae13`; Q0057) |
    | K3 | done (`c4807b8`; Q0072) |
    | L4 | done (`b261863`; Q0074) |
    | K4 | done (`f47fb26`) |
    | L5 | done (`a777ae3`) |
    | J3 | done (`827dfeb`; Q0075) |
    | G3 | done (`8480ee0`) |
- **M4 Syndicate** (**in progress**; spec: `artifacts/director/m4-plan.md`, revision 2, from the design at `70d1bbc` (0282)).
  - **Slices** (revised plan, adopted 2026-11-25, which folds in 0229–0282): A–M plus R1 and R2 (code organization), O (combat spike), U (Grenadier), and S1 and S2 (Laser Turrets). The waves:
    1. A
    2. B, C, D, then I
    3. R1, R2
    4. E, J2, O (O moved up from wave 6 by the Director, to de-risk the combat spike; its only conflict with E is one `OrderKind` line)
    5. J1, H, F (the first playable)
    6. K, G
    7. L, S1
    8. U, S2
    9. M

    The critical path is A → C → R1 → E → F → K → S1 → S2 → M.
  - **Scheduled from the old gated list:** N1, the Grenadier, is now U; N2, the Laser Turret, is now S1 and S2.
  - **Gated (not scheduled):**
    - N3: the Vehicle Assembly, Assault Bike, Laser Battery and Subterranean Transport;
    - N4: the Shadebringer;
    - N5: the Null Chamber, Vault, Forge, Aerospace Assembly and Orbital Command;
    - N6: the Phantom, with aircraft, the air doors and the Prism;
    - N7: the Specter.
  - **Questions:**
    - Q0059–Q0071 are all answered, by 0229–0267.
    - Q0076–Q0085 are open. Q0076, the Agents' jam near the hatch, partly blocks L's oracle; the others don't block, and their slices build the proposals.
  - **Slice status:**
    - A: done (`fcd06dc`).
    - C: done (`f67df212`; Q0077).
    - D: done (`edc30e11`; Q0076).
    - I: done (`9aca8972`).
    - R1 (sim): done (`8eab2290`).
    - R2 (client, docs, citation test): done (`c60b8d0f`). A comments-only citation pass runs before wave 4.
    - B: done (`35e19634`).
    - The rest are queued in the waves above. E, J2 and O come next, after R1 and R2. Their briefs are ready: `scratchpad/briefs/m4e-agent-built-tunnels.md`, `m4j2-tunnel-hideout-ui.md` and `m4o-combat-spike.md`.
  - **Earlier scope note:** tunnels, Hideout, Agent, Guard economy, 0186's Optics Lab gate and Agent pistol, the Grenadier (0187) and the Assault Bike (0188), once their numbers exist. Underground expansions are exempt from the flatness rule (0005).
- **M5 Cults**: Recruitment Hall, Storage, Recruit, Armory, Soldier, Gunner.
  - Recruits are Groupable: every selected Recruit builds (0072).
  - Shift-placement queues build orders while Shift is held, with the cost paid per order (0071).

### Later
- **Vehicles**, turrets, air, area damage, research. The vehicles milestone starts with the movement model from 0033 (rotating silhouettes, oriented collision, turning near obstacles), 0035 (acceleration and collision response), 0039 (Rugged passability per base, hover drag), and tracked maneuver data. The Factory's Artillery Rig and Prospector need Advanced Mechanics (0001). GDO air (0014): limited ammo with rearming at an Airfield; at most 4 aircraft per Airfield; Air-to-Ground Hovercraft and Air-to-Air Fighter (needs Propulsion Labs).
- **Underground and air movement rules** (0043–0047):
  - underground units don't collide with each other;
  - surfacing is refused onto occupied spots and reserves its spot;
  - air ignores terrain, and HoverCraft are clamped to the map;
  - Gliders may overshoot the map edge, staying visible and attackable at the nearest edge tile.
- **Playtest prep** (after all four factions; the user's decision "Modules now, data files at playtest"): the defs and the `tuning/` modules load from data files, reloaded while the game runs. M4-R makes the defs serde-ready, so this is a loading layer, not a refactor.
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
| Q0016 | Command UI edge cases: who carries out a right-click, invalid targets, input while targeting, Shift keeps targeting | no | answered by 0062–0070 (H2 and H3) |
| Q0017 | Fog: shots from hidden attackers (hit flash vs. a partial tracer); targeting remembered structures | no | answered: point 1 by 0177 (M3-L), point 2 by 0139 (M3-R) |
| Q0018 | Behaviour gaps: sight-edge flicker, rotation in target priority, AttackMove leash reference, unreachable auto-targets, MinRange, walled-out spots | no | answered: points 1–2 by 0178 (M3-L1) and 0179 (M3-L2), points 3–6 by 0180–0183 |
| Q0019 | A remembered station's delivery countdown: keep counting or freeze | no | answered by 0057: GDO sees the real timer (folded into G2) |
| Q0020 | Riot Shield: shield Move vs 0062 Follow, transitions, raising mid-shot, mixed groups, toggle time and hotkey | no | answered: point 1 by 0074 (H3); points 2–5 by 0171–0174 (M3-S) |
| Q0021 | Targeting click details: drift during a targeting click, shielded units in Attack targeting, recalling the same group | no | answered by 0073, 0075 and 0078 (H3) |
| Q0022 | GDO supplies: Supply Tower prerequisite and cost, Chopper flight, Starting Supplies | no | answered by 0077: the supply loop is deferred, Starting Supplies are 0 |
| Q0023 | Barracks footprint, side letters and exit; blocked exits | no | answered by 0079 (sides, Barracks 3 wide) and 0080 (exit doorway) |
| Q0024 | Production and rally details: queue limit, Unit Control when queued, several Barracks short of crystals, refunds on destruction, clearing and showing a rally point, rally on an unseen enemy | no | answered by 0081–0088 (M2-D2, M2-H1) |
| Q0025 | Power penalty: is the DC slowed, the ratio, the display, when power counts | no | answered by 0089 and 0090 (power display is used / available) |
| Q0026 | GDO build area: shape, shrinking, no DC, overlay | no | answered by 0091, 0092 and 0093 (square, follows the current buildings, overlay shows the area only) |
| Q0027 | Placement: units under the footprint, plates in fog, buildings on resources, layout rules | no | answered by 0094–0097 (M2-A, M2-C, M2-F, M2-I) |
| Q0028 | Mining: lumps or trickle, last partial load, depleted patch under a plate, selecting a covered patch | no | answered by 0098 (lumps) and 0099 (M2-A, M2-B) |
| Q0029 | What defeat does: the remaining units, victory and game end | no | answered by 0100–0102: units become neutral, the game ends when one side remains, defeated players watch with team vision (M2-F) |
| Q0030 | Structures as targets (auto-targeting buildings), build menu slots, building InfoPanels | no | answered by 0103 (structures are auto-targets; done in M2-A), 0104 (DC has two build menus; M2-H2), 0105 (structure InfoPanels; M2-G) |
| Q0031 | A Move or Follow aimed at a selected unit itself (it currently follows itself forever) | no | answered by 0168 (M3-R) |
| Q0032 | Arrival radius behind an occupied destination: 2 or 3 grid units (data from G3) | no | answered by 0169 (M3-V) |
| Q0033 | Path knowledge, blocked to open: a destination behind a gone obstacle, shortcuts when a wall is seen gone, map-placed structures | no | answered by 0156, 0158 and 0159 |
| Q0034 | Should an enemy plate's InfoPanel show its patch's remaining crystals? | no | answered by 0155 (M3-J2) |
| Q0035 | Rally point UI: Shift with rally targeting, rallying on one of several selected Barracks, feedback when pressing an unavailable command | no | answered by 0160–0162 |
| Q0036 | End of game edge cases: simultaneous last defeats, free-for-all spectating, a raised Riot Shield on defeat | no | answered by 0163–0165 |
| Q0037 | Drone gathering: the cases the loop doesn't define (13 points) | no | answered by 0123 and 0127–0138 |
| Q0038 | Gathering Drones, collision and the economy targets | was yes | answered by 0106 (pass-through), 0107 (geometry), 0108 (±15%) |
| Q0039 | Beacon Deployment and Warp In: what the transforming object is | no (answer before M3-F) | answered by 0109–0116 |
| Q0040 | Deploy and Warp In placement, and the Colonists panels | no | answered by 0117–0125 |
| Q0041 | Portal exits and Beacon Capacity | no | answered by 0143–0145 |
| Q0042 | Shields, melee range and Weapon Conversion | no | answered by 0146–0154 |
| Q0043 | M3 scope: the Colonists content left TBD (FYI) | no | answered by 0166 and 0167 |
| Q0044 | Gathering details from the build: fog, plates, empty Drop Off, blocked Drones, redirect timing, the Egress Node's sight | no | answered by 0134, 0137 and 0139–0141 |
| Q0045 | Pass-through details: idle units left stacked, and the Drop Off order | no | answered by 0126 and 0127 |
| Q0046 | A Drone's drop-off destroyed while it is on its way back | no | answered by 0170 (as built) |
| Q0047 | Waiting for capacity: several waiting buildings, no power, zero-cost units | no | answered by 0175 (as built) and 0176 (point 3 in M3-V) |
| Q0048 | Pressing other commands while targeting (0066 vs 0162); a portal group only partly warping in | no | answered: point 1 by 0184 (as built), point 2 by 0185 (M3-U) |
| Q0049 | Convert pressed again while a conversion waits for Cooldown (0153) | no | answered by 0198 (done, M3-T `bcd66ab`) |
| Q0050 | The ActiveGroup when an unfinished group's last building completes | no | answered by 0199 (done, M3-T `bcd66ab`) |
| Q0051 | A target that outruns sight is lost at once, despite the fog delay | no | answered by 0200 (done, M3-L3 `33b3e3f`) |
| Q0052 | Gathering UI: a Drone's right-click on enemies, the Set Drop-off pointer, who sees a Drone's load, Gather indicators | no | answered by 0201 (points 1–3 as built; markers M3-J3) |
| Q0053 | Riot Shield toggle edge cases: orders while lowering, Lower during a waiting raise, the ActiveGroup during a deferred raise | no | answered: points 2–3 by 0198, point 1 by 0202 (done, M3-T `bcd66ab`) |
| Q0054 | AttackMove: a dropped target not chased again on that leg; the MinRange back-off spot; a cornered unit | no | answered by 0204 (as built M3-L2) |
| Q0055 | The Colonists' measured economy misses the table: 1:00, 3:00 and 4:00 outside ±15%; targets 1 and 4 fail (full saturation needs 27 Drones) | no | answered by 0236 (A: the measured economy is the design; M3-K4) |
| Q0056 | Should a revealed attacker's shots be drawn from the attacker, or only over Visible tiles (a gap of about 140 px, M3-K2)? | no | answered by 0206 (B; M3-J3) |
| Q0057 | An attacker blocked by its own side near its target: melee units within 3 GU give the target up (2–3 of 8 Sabres never strike); should a unit wait without limit; three attempts at a way round | no | answered by 0223 (points 1 A, 2 B, 3 A; M3-G3) |
| Q0058 | AttackMove: is a target lost to the fog left alone for the rest of the leg, like one dropped at the leash? (taking it again loops near walls: soak seeds 8 and 29) | no | answered by 0230 (M3-L4) |
| Q0059 | Optics Lab: placeholder stats | yes | answered by 0229 (placeholders; unblocks M4-K) |
| Q0060 | Tunnel upgrade: duration, HP and payment | yes | answered by 0229, 0231 and 0232 (unblocks M4-G) |
| Q0061 | Agent-built Tunnels: payment, ownership and the site | no | answered: points 1–2 by 0233, points 3–5 and 7–9 by 0234; point 6 moot |
| Q0062 | Tunnel Areas and the two layers | no | answered by 0237–0242 |
| Q0063 | The Tunnel Network: transit, entering single file, and units inside | no | answered by 0245–0249 |
| Q0064 | Ejection and production out of the network | no | answered by 0250–0253 |
| Q0065 | Expansions under construction | no | answered by 0254 |
| Q0066 | Agent gathering: the Drone's rules, the Tunnel it uses, and the line at Side B | no | answered by 0255 |
| Q0067 | Supplies gathering | no | answered by 0256 |
| Q0068 | Panel letters outside the 4x3 grid | no | answered by 0257 |
| Q0069 | Losing Tunnels: the open cases | no | answered by 0258 |
| Q0070 | The Syndicate economy check | no | answered by 0259 |
| Q0071 | Deferred Syndicate content: which items will get their numbers or design in time for M4? | no | answered by 0260–0267 |
| Q0072 | Q0055 re-measured after 0205: every figure is within ±15%; target 1 fails at 2:20 only (the start is 6.93 against the GDO's 6.67); saturation at 24, not 18–22 | no | answered by 0236 (M3-K4) |
| Q0073 | AttackMove: does a chase given up by the Failure rule count as "can't reach" until 0230's triggers fire? (corridor cycle found by M3-G3) | no | answered by 0235 (A; M3-L4) |
| Q0074 | AttackMove: should the unit's own movement re-check an enemy it rejected? As built, a unit that rejected a standing enemy across a wall doesn't take it after rounding the wall, unless it comes within weapon range | no | answered by 0243 (B, to try; M3-L5 measures the cost) |
| Q0075 | A revealed attacker shooting at a target the player can't see: draw the shot whole (literal 0206) or clip it? | no | answered by 0244 (stricter than B; M3-L5) |
| Q0076 | Agents jam head-on between a near patch (0–1 GU) and the hatch: 3+ Agents halve the income (5.33 against 12.5 SC/s). Pass through like Drones, move patches to 3 GU or more, or a right-of-way rule? | partly (M4-L's oracle; the first playable) | open; as built, Agents collide |
| Q0077 | Eject while every unit of the type is in transit: refused or queued? And where an ejected unit with no rally waits: 1 GU out, as built, or farther? | no | open; as built in M4-C |
| Q0078 | Tunnel near the map edge: 0242's Verification (2 tiles from the edge is red) is one tile off its own rule | no | open; M4-B (`35e19634`) built the rule as written, marked `Placeholder: Q0078` |
| Q0079 | Agent build orders: a refund if the Agent is killed before the start; does a pending ghost or a site reserve its whole area? | no | open; M4-E builds the proposals |
| Q0080 | Expansion placement: do ground units and lingering ghosts block it; is a Warp In over a hidden expansion red at once? | no | open; M4-F builds the proposals |
| Q0081 | ExpandMenu slots: do expansions not yet in the game keep their greyed slots, so the turret's letter never moves? | no | open; M4-J2 and M4-F build the proposal |
| Q0082 | Supplies gathering: pick-up and drop-off reaches; nearest Tunnel measured to the Supplies hatch; the old load lost at the end of the new pick-up | no | open; M4-I (`9aca8972`) built A for all three points, marked `Placeholder: Q0082 point N` |
| Q0083 | Area damage: friendly fire, the reveal, Attack Ground's end, how grenades are drawn | no | open; M4-O and M4-U build the proposals |
| Q0084 | Laser Turret: hidden when, sight, blocking, payment, refunds, sites, the Agent's exit, targeting | no | open; M4-S1 and M4-S2 build the proposals |
| Q0085 | Stale doc lines that later answers contradict (seven lines) | no | open; the newer answer wins |

## Design change entries
Last triaged: 0282.

| Entry | Status | Notes |
|---|---|---|
| 0001 Artillery Rig requires Advanced Mechanics | queued (Later: Factory) | |
| 0002 CommandPanel slots by hotkey | done (no-op: matches the M1-H command panel) | answers Q0005 |
| 0003 Art direction | queued (art pipeline) | confirms the square grid and SC1-style client conventions |
| 0004 Corner-height elevation | done (terrain M0-D; ElevationModifier M1-V `83fec7d`; structure flatness M2-C `0f8cc70`) | |
| 0005 Underground exemption; structures get ElevationModifier | queued (M1, M4) | |
| 0006 Slopes don't affect speed | done (check: movement has no slope term) | confirms the default |
| 0007 Per-object resource values | done (M0-D) | answers Q0003 |
| 0008 Starting layouts; symmetry tools | done (symmetry M0-D/M0-E; layouts: sim M2-F1 `d5a7d29`, editor M2-I `6ae2b02`) | |
| 0009 Layouts from scratch; Skirmish Default | done (rules M2-F1 `d5a7d29`, editor M2-I `6ae2b02`); per-layout toggle superseded by 0012 | |
| 0010 Middle-mouse grab pan | done (M0-D) | |
| 0011 Default input bindings; rebinding | bindings table done (M1-A); selection/groups → M1; rebinding UI later | answers Q0002 |
| 0012 Map types, creation, generated symmetry | done (map types and symmetry M0-E; layout rules M2-F1; editor M2-I `6ae2b02`) | answers Q0004 |
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
| 0038 Crystal patches and supply stations block ground units | done (M1-C rule; Verification tests in M1-G2, `52f20a8`) | answers Q0011 part 1; mining depletion is verified again with the economy |
| 0039 Rugged terrain impassable for wheeled, tracked and drill vehicles; hover crosses with double drag | queued for the vehicles milestone | infantry unchanged (M1-C keeps Rugged passable for infantry) |
| 0040 Pathfinding uses only what the owner knows | done (M2-E, `f76db38`) | replaces default A (true map); M1 has no player-built structures, so nothing is observable yet; F was told to shape its records for it |
| 0041 The obstacle cache is only a suggestion | done (M2-E, `f76db38`) | how and when to re-path is the Director's call |
| 0042 Map-placed patches and stations known from the start | done (M2-E, `f76db38`) | confirms the 0040 assumption |
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
| 0057 GDO sees a remembered station's real delivery timer | done (M1-G2, `52f20a8`) | answers Q0019 |
| 0058 Allies can't be multi-selected; GDO station intel intended | matches the build (no teams yet) | answers Q0013 point 2 |
| 0059 Explicit Attack may target own and neutral objects; right-click doesn't | matches the build | answers Q0014 point 1 |
| 0060 Hits land at the end of Firing; overkill wastes the shot | matches the build | answers Q0014 point 2 |
| 0061 Rugged effects use the centre tile | matches the build | answers Q0014 point 3 |
| 0062 Right-click: units that can't carry it out move or follow | done (M1-H2, `2417f70`) | answers Q0016 point 1 |
| 0063 Right-click resolves per unit; Mixed cursor | done (M1-H2, `2417f70`) | |
| 0064 Attack targeting accepts own and neutral destructibles | done (M1-H2, `2417f70`) | |
| 0065 Invalid target click: ignored, targeting stays, with feedback | done (M1-H2, `2417f70`) | answers Q0016 point 2 |
| 0066 While targeting, other commands are unavailable | done (M1-H2, `2417f70`) | answers Q0016 point 3, first part; matches the build |
| 0067 Box-drag while targeting does nothing | done (M1-H2, `2417f70`) | matches the build |
| 0068 Targeting clicks count at the press point | done (M1-H2, `2417f70`) | |
| 0069 Selection changes while targeting: player changes cancel, deaths don't | done (M1-H2, `2417f70`) | Q0016 point 3 complete |
| 0070 Shift keeps targeting active while held | done (M1-H3, `80b3d88`) | Q0016 complete; Shift released before any click: targeting continues (proposed default) |
| 0071 No smart-casting; Shift while placing | done (audit M1-H3 `80b3d88`; placement M2-H2 `2d507b4`); the Recruit Shift-queue in M5 | |
| 0072 Recruits are Groupable | queued (M5 Cults) | |
| 0073 Targeting click allowance (16 px) | done (M1-H3, `80b3d88`) | answers Q0021 point 1 |
| 0074 Shielded Peacekeepers follow on right-click | done (M1-H3, `80b3d88`) | answers Q0020 point 1; replaces the Move-to-location default |
| 0075 Shielded Peacekeepers are left out of explicit Attack | done (M1-H3, `80b3d88`) | answers Q0021 point 2; differs from right-click on purpose |
| 0076 AttackMove goes to every unit, including units that can't attack | done (M1-H3, `80b3d88`) | closes 0075's TBD; check that the sim gate doesn't drop shielded units from AttackMove |
| 0077 GDO supply loop deferred; Starting Supplies 0 | done (M2-B, `39bcf66`) | answers Q0022; matches the plan |
| 0078 Any control-group recall resets menus | done (M1-H3, `80b3d88`) | answers Q0021 point 3 |
| 0079 Side lettering convention; Barracks 3 wide × 2 tall | done (`sides` M2-A `cbeb804`; ghost labels M2-H2 `2d507b4`) | answers Q0023 point 1 |
| 0080 Exit doorway: units appear half inside; production pauses while it is occupied | done (M2-D2, `1a64e75`) | answers Q0023 points 2–3; replaces the nearest-free-spot default; portals get their own energy doorway (0145) |
| 0081 Queue limit of 20 includes the unit in production | done (M2-D2, `1a64e75`) | |
| 0082 GDO Unit Control reserved when a unit is queued | done (M2-D2, `1a64e75`); superseded by 0143 (done, M3-P `9f63a5f`) | |
| 0083 Group production: least remaining work pays first | done (sim M2-D2; UI M2-H1, `eccaaa6`) | replaces the fixed-order default; Unit Control removed from it by 0143 (M3-P, `9f63a5f`) |
| 0084 Destroyed GDO producers refund as if cancelled | done (M2-D2 producers, M2-D1 constructors, `fda74d3`) | |
| 0085 Finished unit waiting inside is refunded if the building dies | done (M2-D2, `1a64e75`) | |
| 0086 Rally point on the building itself clears it | done (sim M2-D2; UI M2-H1, `eccaaa6`) | |
| 0087 Rally point indicators | done (sim M2-D2; UI M2-H1, `eccaaa6`) | |
| 0088 Rally on an enemy becomes a ground rally when the enemy leaves sight | done (sim M2-D2; UI M2-H1, `eccaaa6`) | a rally on a remembered enemy structure stays on it (0139, M3-R) |
| 0089 Power penalty scope, ratio and timing | done (M2-B, `39bcf66`) | matches the plan |
| 0090 Power display: used / available | done (M2-B accessor, M2-G display, `69e80a1`) | replaces "current / total" |
| 0091 GDO build area is square | done (M2-C, `0f8cc70`) | matches the plan |
| 0092 GDO build area follows the current buildings | done (M2-C, `0f8cc70`) | matches the plan |
| 0093 Build area overlay shows the area only | done (M2-H2, `2d507b4`) | answers Q0026 point 4; matches the plan |
| 0094 Ground units on the footprint block Direct Placement | done (M2-C, `0f8cc70`) | answers Q0027 point 1; matches the plan |
| 0095 Extraction Plates need visible tiles | done (M2-C check; M2-F1 layouts and `dev_skirmish`, `d5a7d29`) | answers Q0027 point 2 |
| 0096 No buildings on resources, except plates on patches | done (spawns M2-A `cbeb804`; placement check M2-C `0f8cc70`) | answers Q0027 point 3 |
| 0097 Starting Layout validation | done (M2-C check; M2-F1 layouts and `dev_skirmish`; M2-I editor `6ae2b02`) | answers Q0027 point 4: the proposal plus a rule for starting units |
| 0098 Plates pay in lumps; the cycle timer advances by the power ratio | done (M2-B lumps; M2-J1 oracle, `c2b2a17`) | answers Q0028 point 1 |
| 0099 Plate depletion: last partial load, depleted patch stays under its plate, the pick selects the plate | done (pick: M2-A; mining: M2-B, `39bcf66`) | answers Q0028 points 2–4 |
| 0100 A defeated player's units stay but can't be controlled | superseded by 0101 (units become neutral) | answers Q0029 point 1; replaces removing the units |
| 0101 On defeat, units become neutral; the game ends when one side remains | done (M2-F2, `e3f67a9`) | answers Q0029 points 1–2 |
| 0102 Defeated players in team games watch with team vision | done (M2-F2, `e3f67a9`) | closes 0101's TBD |
| 0103 Enemy structures are valid automatic targets | done (M2-A, `cbeb804`) | answers Q0030 point 1; matches M1 |
| 0104 Deployment Center has two build menus | done (M2-H2, `2d507b4`) | answers Q0030 point 2; M2 needs only Generator, Barracks and EF in BuildMenu[Basic] |
| 0105 Default InfoPanel for structures | done (M2-G `69e80a1`; activity lines M2-H1 `eccaaa6`, M2-H2 `2d507b4`) | answers Q0030 point 3, as proposed |
| 0106 Gathering Drones pass through each other; the general overlap rule | done (M3-H2, `777b9a9`) | answers Q0038 point 1 |
| 0107 Reference check geometry: a 2-GU gap, edge to edge | done in M3-H's tests (`5bd3b69`); queued (M3-K1) | answers Q0038 point 2 |
| 0108 Economy check tolerance ±15%; the reference layout is only a benchmark | queued (M3-K1) | answers Q0038 point 3 |
| 0109 The transforming object is the target structure from the first frame | Warp In done (M3-E, `68ee747`); deploy done (M3-F, `ff119a8`) | answers Q0039 point 1 |
| 0110 Damage stays taken during the transform interpolation | HP done (M3-E); Shields done (M3-B, `61aa089`); deploy done (M3-F, `ff119a8`) | answers Q0039 point 2 |
| 0111 A transform cancel keeps the HP and Shields fraction, at least 1 HP | HP done (M3-E); Shields done (M3-B, `61aa089`); deploy done (M3-F, `ff119a8`) | answers Q0039 point 3 |
| 0112 Beacon Capacity during Deploy and Warp In; over capacity allowed and shown red | done (M3-C red counter; M3-E Warp In; M3-F deploy; M3-P production side, `9f63a5f`) | answers Q0039 point 4, and Q0041 point 2 in effect |
| 0113 Structures under transformation count for defeat | Warp In done (M3-E, `68ee747`); deploy cancel done (M3-F, `ff119a8`) | answers Q0039 point 5 |
| 0114 A Drone deploying with a load keeps it; lost on completion, returned on cancel | done (M3-F, `ff119a8`) | answers Q0039 point 6 |
| 0115 A building destroyed mid-Warp-In refunds nothing | done (M3-E, `68ee747`) | answers Q0039 point 7 |
| 0116 Transforming objects show only Cancel; functions start at completion | done (sim M3-E, M3-F `ff119a8`; Cancel-only panel M3-J1 `864c13f`) | answers Q0039 point 8 |
| 0117 Deploy start position: the footprint centre, or stopped while overlapping it | done (M3-F, `ff119a8`) | answers Q0040 point 1 |
| 0118 Ground units block Worker-Built and Warp In sites | done (M3-E, M3-F `ff119a8`; air and underground untested, none exist yet) | answers Q0040 point 2 |
| 0119 Warp In needs Visible tiles | done (sim M3-E `68ee747`; ghost M3-J1 `864c13f`) | answers Q0040 point 3; differs from the proposal |
| 0120 The Worker-Built ghost in fog uses the player's knowledge | done (gate M3-F `ff119a8`; ghost M3-J1 `864c13f`) | answers Q0040 point 4 |
| 0121 Odd-size Warp In: extras at the bottom-right, turned and flipped about the Beacon centre | done (sim M3-E `68ee747`: `transform::warp_in_footprint`, `WarpIn` has no origin; ghost M3-J1 `864c13f`) | answers Q0040 point 5; replaces cursor snapping |
| 0122 Beacons are Ungroupable | done (def M3-E `68ee747`; Tab picks the Beacon, M3-J1 `864c13f`) | answers Q0040 point 6 |
| 0123 A Gather ends only when no patch is left within the Drone's search radius; queued orders wait | done (M3-H3, `1e43600`) | answers Q0040 point 7 and Q0037 point 12; Q0037 point 5 in part; replaces the after-the-next-drop-off proposal |
| 0124 Colonists panels: two Warp In menus on the Beacon; Cancel only while transforming; portal, Drone and Egress Node grids | done (portal grids M3-D `20f584e`; Beacon menus and Cancel M3-J1 `864c13f`; Drone S, Egress Node F and the unavailable portal slots M3-J2 `298f364`) | answers Q0040 point 8 |
| 0125 Lingering build ghosts; Drones Groupable; a Deploy goes to one Drone of a group; Shift-placement chains through the group | done (sim M3-F `ff119a8`: Groupable Drones, `Sim::pending_deploys`, `TileProblem::Reserved`; ghosts, dispatch and chaining M3-J1 `864c13f`) | answers Q0040 point 9; replaces 0071 for Drones |
| 0126 Idle overlapping units push apart; units with orders are never moved | done (M3-G, `5d61dc7`) | answers Q0045 point 1; amends 0106 |
| 0127 Drop Off is only a delivery; the Drone then idles at the drop-off or starts its next order | done (M3-H3, `1e43600`) | answers Q0045 point 2 and Q0037 point 1; replaces the return-to-patch proposal |
| 0128 A loaded Drone sent to Gather skips mining at the new patch and goes to deliver | done (M3-H3, `1e43600`) | answers Q0037 point 2 |
| 0129 Other orders keep the load | done (M3-H3, `1e43600`) | answers Q0037 point 3 |
| 0130 Interrupted mining gives no partial load; the patch frees at once | done (M3-H3, `1e43600`) | answers Q0037 point 4 |
| 0131 The patch search counts only known patches not under a structure | done (M3-H3, `1e43600`) | answers Q0037 point 5 |
| 0132 A waiting Drone waits at the patch it tried, re-checks every 16 frames, and returns to the patch it last mined | done (M3-H3, `1e43600`) | answers Q0037 point 6 in part |
| 0133 Which waiting Drone gets a freed patch is the implementation's choice, kept deterministic | done (M3-H3, `1e43600`) | answers Q0037 point 6 |
| 0134 A depleted patch counts as occupied; Drones walking to it search on arrival | done (M3-H3, `1e43600`) | answers Q0037 point 7 and Q0044 point 5 |
| 0135 A missing drop-off is replaced by the nearest own Egress Node when the return trip starts; with none, the Drone idles holding its load | done (M3-H3, `1e43600`) | answers Q0037 point 8 |
| 0136 A rally gives the new unit its default right-click command against the rally target; one rule for every producer | done (M3-R, `e2975db`: `orders::resolution::resolve`, `Sim::rally_order`; given at release) | answers Q0037 points 9–10 |
| 0137 A Drone mining a patch doesn't block a plate; a plated patch is gone for Drones | done (M3-H3, `1e43600`) | answers Q0044 point 2 and Q0037 point 11 |
| 0138 The last load takes the full 32 frames and the remaining crystals | done (M3-H3, `1e43600`) | answers Q0037 point 13 |
| 0139 Remembered objects can be targeted: orders go to the last-known position and resolve on sight | done (M3-R, `e2975db`: `targets::TargetView`; the gate takes seen or remembered targets) | answers Q0017 point 2, Q0044 points 1 and 6; replaces M2's "object targets only when visible" |
| 0140 No empty Drop Off order: an empty Drone right-clicking its own node moves there | done (gate M3-H3 `1e43600`; client Move M3-J2 `298f364`) | answers Q0044 point 3 |
| 0141 An unreachable patch is handled like an occupied one; the Drone never gives up | done (M3-H3, `1e43600`) | answers Q0044 point 4 |
| 0142 Beacon Capacity reserved at queue time | superseded by 0143 | answered Q0041 point 1 |
| 0143 Capacity (Unit Control and Beacon Capacity) is taken when production starts, not at queue time; a producer without room waits at 0% | done (M3-P, `9f63a5f`) | answers Q0041 points 1–2; replaces 0082 and 0142 |
| 0144 Robo Portal exits: the Bio Portal's rule on all four sides, closest by side midpoint, ties top, right, bottom, left | built (M3-D, `20f584e`); superseded by 0145 (done, M3-D2 `3576cbe`) | answers Q0041 points 3–4 |
| 0145 Portal energy doorway: units appear at the portal's centre and leave in any open direction, straight toward the rally point if possible | done (M3-D2, `3576cbe`; the Barracks keeps its side doorway) | answers Q0041 points 5–6; replaces 0144 |
| 0146 Shields regenerate for neutral and transforming objects | done (M3-B, `61aa089`) | answers Q0042 points 1–2 |
| 0147 Modifiers skip Shields; only the overflow takes them | done (M3-B, `61aa089`) | answers Q0042 point 3 |
| 0148 Shield bar above HP; "Shields a / b" in the InfoPanel | done (M3-B, `61aa089`); styling waits for the HUD pass | answers Q0042 point 4 |
| 0149 Melee reach: 8 su edge to edge for every melee attack | done (placeholder 8 su since M3-A; citations fixed in M3-F, `ff119a8`) | answers Q0042 point 5 |
| 0150 Convert and the Riot Shield are mode switches: orders stay | done (Convert M3-B `61aa089`; Riot Shield M3-R `e2975db`) | answers Q0042 point 6 |
| 0151 Mode switches happen at once; Shift has no effect | done (Convert M3-B `61aa089`; Riot Shield M3-R `e2975db`) | answers Q0042 point 7 |
| 0152 A converted Legionnaire's weapon starts ready | done (M3-B, `61aa089`) | answers Q0042 point 8 |
| 0153 Convert pressed during Firing or Cooldown happens right after Cooldown ends | done (M3-R, `e2975db`: `AttackState::pending`, applied in `step_modes`); second press: Q0049 | answers Q0042 point 9 |
| 0154 The flag is CanHaveTurret; a unit has a turret only if TurretAttributes are set | done (M3-R, `e2975db`: CanHaveTurret; the turret is the attack source; the Drone is unarmed) | answers Q0042 point 10 |
| 0155 A plate's remaining crystals are shown only to its owner | done (M3-J2, `298f364`) | answers Q0034; changes M2-G |
| 0156 A destination resolved at order time is final, even when the obstacle is later seen gone | done (M2-E, as built) | answers Q0033 point 1 |
| 0157 No re-plan when a remembered obstacle is seen gone | superseded by 0158 | answered Q0033 point 2 |
| 0158 Route recalculation timing is the Director's choice: no perf issues, predictable movement | done (M2-E's behaviour kept; the Director may change it later without a question) | answers Q0033 point 2 |
| 0159 Neutral map-placed objects are known from the start (pathing and memory); starting layouts are not | done (M3-R, `e2975db`: `Vision::seed`; records stay hidden on Unexplored tiles) | answers Q0033 point 3 |
| 0160 Shift with the rally point sets it and ends targeting | done (as built, M2-H1) | answers Q0035 point 1 |
| 0161 A rally on one of several selected producers: per-building rules | done (as built, M2-H1) | answers Q0035 point 2 |
| 0162 A greyed-out button or hotkey gives the refusal feedback with its reason | done (M3-J1 `864c13f`: `CommandRefused` in the feedback band) | answers Q0035 point 3 |
| 0163 Every remaining side defeated in one tick: a draw | done (M3-V, `9516a0f`: `Outcome::is_draw`, derived) | answers Q0036 point 1 |
| 0164 Defeated with no undefeated allies: leave to a results screen | done (M3-V, `9516a0f`: `hud/results.rs`; the team case is unit-tested only, since there are no teams yet) | answers Q0036 point 2 |
| 0165 Defeat clears orders but not modes | done (as built, M2-F2) | answers Q0036 point 3 |
| 0166 M3 scope confirmed as proposed; deferred content listed | done (the M3 plan) | answers Q0043 points 1, 3, 4; placeholder numbers or questions when deferred content is scheduled |
| 0167 Robo Portal costs 250 (was 400); Bio Portal stays 150 | done (M3-V, `9516a0f`; refund 187); watch it in K1 | answers Q0043 point 2 |
| 0168 A Move or Follow whose target is the unit itself is a Stop | done (M3-R, `e2975db`: `follow_itself.rs`) | answers Q0031; fixes M1-G3's self-follow |
| 0169 Arrival radius 3 GU (was 2) for MovingToLocation and MovingToObject | done (M3-V, `9516a0f`: `OCCUPIED_ARRIVAL_GU` 3.0; crowd-sweep failures 43 → 7) | answers Q0032 |
| 0170 A drop-off destroyed mid-return: the Drone switches at once to the nearest own node | done (as built M3-H3; event-driven `Sim::reassign_drop_offs`, M3-K2a `e481b24`) | answers Q0046 |
| 0171 Riot Shield toggle window: half speed and no attacking while toggling; damage reduction only when fully up; a second press reverses from the current point | done (M3-S, `08f09a6`) | answers Q0020 point 2 |
| 0172 A shield toggle pressed mid-shot waits for Cooldown to end, then starts | done (M3-S, `08f09a6`) | answers Q0020 point 3; like Convert's 0153 |
| 0173 Peacekeeper and Peacekeeper (Shielded) are two unit types; the type changes at the start of the toggle | done (M3-S, `08f09a6`) | answers Q0020 point 4 |
| 0174 Shield toggle 6 ticks (0.375 s), hotkey R for Raise and Lower | done (M3-S, `08f09a6`) | answers Q0020 point 5 |
| 0175 Several buildings waiting for capacity: the lowest id starts first | done (as built, M3-P `9f63a5f`); doc comments updated in M3-V | answers Q0047 point 1 |
| 0176 No start at power 0 (as built); a capacity cost of 0 always passes | done (point 2 as built, M3-P; point 3 M3-V `9516a0f`, in `ArmyCap::fits`) | answers Q0047 points 2–3 |
| 0177 Shots drawn only over Visible tiles; an attacker outside the victim's vision is revealed to the victim's owner for 24 ticks after each shot | done (M3-L1, `aeb54ae`: `vision/reveal.rs`, `REVEAL_TICKS` 24; tracers cut at Visible tile edges) | answers Q0017 point 1 |
| 0178 Fog delay: a tile stays Visible for 8 ticks after it leaves all sight | done (M3-L1, `aeb54ae`: `vision/linger.rs`, `FOG_DELAY_TICKS` 8); a target outrunning sight: Q0051 | answers Q0018 point 1 |
| 0179 Target priority by rotation in 45° steps, then closest, then a fixed order | done (M3-L2, `f039e5c`: `rotation_step`; exactly 45° is step 0) | answers Q0018 point 2 |
| 0180 AttackMove leash limits chasing only: targets in weapon range are always fought; PathReference kept after fights, replaced by a blockage detour | done (M3-L2, `f039e5c`: `travel::pursue`; the dropped-target rule is Q0054) | answers Q0018 point 3 |
| 0181 Unreachable auto-targets: approach, give up, retry after 32 ticks | done (as built) | answers Q0018 point 4. Since M3-L4, a target given up by an explicit Attack also waits 32 ticks before idle auto-targeting takes it again |
| 0182 MinRange: auto-targeting skips targets inside it; an explicit Attack backs a mobile unit off, and fails for an immobile one with "Target too close" | done (M3-L2, `f039e5c`: back-off, `Sim::step_too_close`, `SimEvent::TargetTooClose`; back-off spot is Q0054) | answers Q0018 point 5 |
| 0183 A unit walled out of its group spot counts as arrived within the 3 GU radius | done with 0169 (M3-V, `9516a0f`; tested with 7 Guards) | answers Q0018 point 6 |
| 0184 Another command pressed while targeting: "Cancel the current command first" | done (as built, M3-J1 `864c13f`) | answers Q0048 point 1 |
| 0185 Unfinished buildings form their own SelectionGroup (key: type, finished?) | done (M3-U, `d3814be`: `GroupKey { kind, unfinished, instance }`; the ActiveGroup on completion is Q0050) | answers Q0048 point 2; replaces M3-J1's reading |
| 0186 Syndicate Guard: laser rifle, Universal, 18 damage every 24 ticks, needs an Optics Lab; Agent: pistol, ground only, 5 damage every 20 ticks | Guard laser done (M3-L2, `f039e5c`); Optics Lab gate, Hideout and Agent queued (M4) | user redesign |
| 0187 Syndicate Grenadier: slow, dodgeable AoE grenade, ground only, Attack Ground; Hideout E, needs a Chemical Plant | queued (M4; concept only, numbers TBD) | user addition |
| 0188 Syndicate Assault Bike: fastest wheeled vehicle, wide turns, mortar on a narrow-angle turret | queued (M4; concept only, numbers TBD) | user addition |
| 0189–0196 Syndicate Shadebringer: support vehicle with Sludge (slows everyone except hover and air, Drill Units only on the surface (0196), laid behind it) and Cover (smoke that blocks line of sight and blinds units inside); separate tanks refilled in the Tunnel Network with a Vault | queued (M4; concept only, numbers and several rules TBD; line-of-sight blocking is new to the vision model) | user additions |
| 0197 Convert pressed again while a conversion is deferred cancels it; in a group, per unit (converted switch back, mid-conversion left out, waiting cancel) | superseded by 0198 (never built; M3-K2 told to drop it) | answers Q0049; changes M3-R's behaviour |
| 0198 A mode toggle (Convert, Riot Shield) sets a target state: type and group switch at the press, even when deferred; the ActiveGroup follows; only the ActiveGroup receives the press | done (M3-T, `bcd66ab`) | answers Q0049 (replaces 0197) and Q0053 points 2–3; changes M3-R and M3-S |
| 0199 The ActiveGroup when it disappears: on a type change the joined group becomes active, the panel resets and targeting ends; when the last member dies or leaves, the next group in Tab order (wrapping) | done (M3-T, `bcd66ab`) | answers Q0050 |
| 0200 A target lost from vision ends Attack and Follow at once; the unit stops or goes on to its next order (replaces 0034's walk to the last seen position) | done (M3-L3, `33b3e3f`); AttackMove keeps M3-L2's dropped-target placeholder (Q0058) | answers Q0051 |
| 0201 Gathering UI: a Drone's right-click on enemies, the Set Drop-off pointer and who sees the load confirmed; Gather, Drop Off and Set Drop-off get their own markers | done: points 1–3 as built in M3-J2, point 4 in M3-J3 (`827dfeb`) | answers Q0052 |
| 0202 Orders during the Riot Shield toggle follow the new type from the press; an Attack given while lowering is held until the shield is down | done (M3-T, `bcd66ab`) | answers Q0053 point 1 |
| 0203 The Rocketman's normal attack has a MinRange (value TBD) | queued (waiting for the value; the MinRange rules are built, M3-L2) | context for Q0054 |
| 0204 AttackMove's dropped target, the MinRange back-off spot and a cornered unit stay as M3-L2 built them; a locked Rocketman's fire ignores MinRange | done (as built M3-L2), except point 1, superseded by 0230 (M3-L4); the locked-fire part is queued with Target Lock | answers Q0054 |
| 0205 The Drone accelerates faster (ForwardAcceleration 0.9, OmniDirectionalAcceleration 0.3, DragRatio 0.15; were 0.6 / 0.2 / 0.1); re-run the Colonists oracle and report the table for Q0055 | done (M3-K3, `c4807b8`); figures re-reported as Q0072 | a step toward Q0055, which stays open; the table and targets stay unchanged |
| 0206 While an attacker in the fog is revealed, its own shots are drawn from it to the target; other shots stay clipped to Visible tiles | done (M3-J3, `827dfeb`); narrowed by 0244 (M3-L5) | answers Q0056; client only |
| 0207–0209 Laser Battery, a new Syndicate vehicle: always charging, weaker surges on an empty battery, Amplify, a narrow turret arc; Syndicate lasers are red | queued (M4: red lasers in M4-A; the Laser Battery is gated on numbers, N3) | concept; no numbers yet |
| 0210 The Laser Battery needs an Optics Lab; the Laser Battery and the Artillery Rig have mobile and deployed as two unit types | queued (M4 N3; the Artillery Rig part with the Artillery Rig, not yet built) | deploy times TBD |
| 0211, 0213, 0214 Vehicle deploy is a queueable order, can't be cancelled once started, switches type at completion; orders given meanwhile are held and checked against the deployed type | queued (with the first deploying vehicle: Laser Battery, Artillery Rig or the Transport's Burrow and Surface) | not the Drone's Beacon Deployment, which stays as M3 built it (0214) |
| 0212 Stop while deploying winds the deploy back | superseded by 0213 | never built |
| 0215–0220 Subterranean Transport, a new Syndicate infantry transport (Burrow and Surface on the deploy model, hidden underground unless detected, area damage still hits it, surfaces on tiles a tracked vehicle can cross that have no building, passengers die with it); every DrillUnit hidden underground unless detected | queued (M4 N3; hiding needs the detection system, TBD) | concept; capacity, timings and detection TBD |
| 0221–0222 The Subterranean Transport requires a Forge and drives into Tunnels loaded; passengers stay aboard in the network, and only its own Tunnel space counts | queued (M4 N3) | concept |
| 0223 Attackers blocked by their own side: no special crowd logic. Melee units within 3 GU give up (as built); farther out a unit goes round if it can, else the Failure rule applies (about 7 s), with no indefinite wait; a few attempts, no re-checking. New principle: default behaviour is predictable, not optimal | done (M3-G3, `8480ee0`); the AttackMove part through 0235 (M3-L4, `b261863`) | answers Q0057; removes M3-G2's and M1-E's indefinite wait |
| 0224 The Tunnel: Side A has an entry path and an exit path; Sides B and C are hatches for crystals and Supplies; Side D has no function; the look changes by tier; T3 adds air doors | queued (M4: A, C, D and I; air doors deferred with air units) | art direction for three tiers |
| 0225 Each Side A path takes one unit at a time at a fixed position; entering and exiting don't block each other | queued (M4-C) | how long a unit holds a path is TBD (an M4 question) |
| 0226–0227 The Phantom, a Syndicate hovering aircraft from the Aerospace Assembly: requires an Optics Lab, rises from a T3 Tunnel's air door, red laser against air and ground, a 60–90° turret arc, Guard-like but squishier and faster | queued (M4, deferred: numbers TBD) | concept |
| 0228 The Phantom's 60–90° is the total turret cone; the rally line no longer cites chasing into the fog (no rule change) | queued (M4, with the Phantom); the doc fix needs no code | follows Q0058's context note |
| 0229 Placeholders: Optics Lab (2x2, 150 SC, 400 frames, HP 400, armour 1/4, Groupable, no commands); Tunnel upgrade takes 480 frames per tier, with Supplies paid when ordered | queued (M4-K, M4-G) | answers Q0059 and Q0060 points 1–2; unblocks K and G |
| 0230 AttackMove takes only targets it can reach within the leash and sight; no dropped-target memory; a rejected enemy is re-judged only when it moves or PathKnowledge changes; performance a priority | done (M3-L4, `b261863`); own movement isn't a trigger (Q0074) | answers Q0058; replaces 0204 point 1 and M3-L3's placeholder |
| 0231–0232 Tunnel upgrades: an upgrade in progress counts toward the next one's cost; Cancel refunds the current top price, not counting itself (0232 corrects 0231); HP rises with max HP; Supplies are lost if the Tunnel is destroyed mid-upgrade; the larger area is reserved from the start | queued (M4-G) | answers Q0060 points 3–6 |
| 0233 Agent-built Tunnels pay when the ghost is placed; ghosts and sites count toward the price; an order that can't be paid is refused with feedback; refunds at the current top price | queued (M4-E) | answers Q0061 points 1–2 |
| 0234 A Tunnel under construction: Cancel only, ConstructionHP is the maximum and damage stays, the Agent comes out at the centre, the site counts for defeat, the Agent keeps its load, enemies see the site and not the Agent | queued (M4-E) | answers Q0061 points 3–5 and 7–9, as proposed |
| 0235 A chase given up by the Failure rule during an AttackMove counts as "can't reach": rejected until 0230's triggers fire | done (M3-L4, `b261863`) | answers Q0073 (A) |
| 0236 The measured Colonists economy is the design: the table is K3's figures, target 1 is "the Colonists lead from the start to about 1:40", saturation is about 24 Drones | done (M3-K4, `f47fb26`) | answers Q0055 and Q0072; no game number changes |
| 0237 Tunnel Areas may not overlap any player's Tunnels: crowding and blocking an enemy's Tunnels is allowed. The ghost judges known enemy Tunnels at their remembered position and tier; the check on arrival and upgrade checks use the truth | queued (M4-E, M4-G) | answers Q0062 point 1; replaces the own-Tunnels-only proposal |
| 0238 A Tunnel Area's cells past the map edge count for Tunnel Space; expansions are placed on map cells only | superseded by 0242 | answers Q0062 point 2 |
| 0239 Every tile under an expansion must be Drillable (Plane, Rugged and Cliff; not Mountain or Water); own and neutral underground structures and the Tunnel's footprint block it | queued (M4-F) | answers Q0062 points 3a, 3b and 3e, as proposed |
| 0240 Surface buildings and underground expansions never share a tile, whatever the owner (neutral included). A direct placement (GDO) over a hidden expansion is red at once, an accepted leak. A worker-built ghost judges by what the player knows and fails on the worker's arrival. Units still walk over expansions | queued (M4-E for surface placement: direct placement and the check on arrival read the underground truth; M4-F for expansions: none under a surface building) | answers Q0062 points 3c and 4; replaces T32's "the surface methods ignore them" |
| 0241 No expansions under resources (patches, Supply Delivery Stations); non-Drillable cells give no Tunnel Space | queued (M4-F: no expansions under resources); the Tunnel Space part is superseded by 0242 | answers Q0062 points 3d and 5 |
| 0242 A Tunnel's whole area must be on the map and Drillable, for placement and upgrades; every tunnel-region cell is usable; starting layouts must respect it | partly done (M4-B, `35e19634`: layout and editor validation, `TunnelAreaOffMap` and `TunnelAreaNotDrillable`, Tunnel Space; 2 tiles from the edge valid, Q0078); queued (M4-E: placement, the ghost by knowledge and the check on arrival; M4-G: the upgrade check) | completes Q0062; replaces 0238 and 0241's Tunnel Space part |
| 0243 AttackMove: a reach-rule rejection is also re-checked once the unit has moved about 1 GU from where it judged; Failure-rule and fog rejections keep 0230's triggers; measure the cost | done (M3-L5, `a777ae3`); the 1 GU step is kept: at most 1.45× master's mean tick, about +0.05 ms for 100 Guards | answers Q0074 (B, "to try") |
| 0244 A revealed attacker's shot is drawn whole only when its target is the viewer's own object; everything else is clipped to Visible tiles | done (M3-L5, `a777ae3`) | answers Q0075; narrows 0206 |
| 0245 Tunnel Network: a flat transit delay of 32 frames (placeholder) after entering before a unit can exit; produced units have none | done (M4-C, `f67df212`) | answers Q0063 point 1 |
| 0246 Units walk into a Tunnel single file at normal speed with normal spacing, no fixed hold per unit ("one unit wide"); produced units exit without delay | done (M4-C, `f67df212`) | answers Q0063 point 7 and part of 2; replaces the 8-frame hold proposal |
| 0247 Entering a Tunnel uses the normal Failure rule; no special waiting rule | done (M4-C, `f67df212`) | answers Q0063 point 2, as proposed |
| 0248 Tunnel sides corrected: A entrance and exit, B crystal hatch, C back wall (no function), D Supplies hatch. Side A's exit path is the half nearer B, the entry path the half nearer D | done (A `fcd06dc`, C `f67df212`, D `edc30e11`, I `9aca8972`: Supplies at D) | answers Q0063 point 2 (which half of Side A); corrects 0224's B and C hatches |
| 0249 Inside the network: orders cleared, control groups and loads kept, slow HP regeneration; a Tunnel being upgraded keeps its old tier for entry and exit until it completes | done (M4-C, `f67df212`); the upgrade case waits for G | Q0063 complete (0245–0249) |
| 0250 Ejection: ground units walk out single file, gated on the exit spot being clear; air doors have their own rate (8 frames) | done for ground units (M4-C, `f67df212`); the air doors wait for the first Syndicate air unit | answers Q0064 points 1, 2, 4, 10 and 11 |
| 0251 Ejection: the longest-inside unit goes first; queue rules; air doors on the roof over the matching path halves; Vault and Null Chamber limits noted | done for ground units (M4-C, `f67df212`); the air doors wait for the first Syndicate air unit | answers Q0064 points 3, 4 and 11 |
| 0252 Syndicate rallies: a rally on a Tunnel stays inside; one on an expansion means the surface spot above it | done (M4-C, `f67df212`) | answers Q0064 points 5–7 |
| 0253 Auto-ejected units share the exit queue | done (M4-C, `f67df212`) | Q0064 complete (0250–0253) |
| 0254 Expansions under construction, as proposed, except the 75% refund on cancel | partly done (M4-A, `fcd06dc`: the Hideout's sight 0 and Groupable); queued (M4-F) | answers Q0065 |
| 0255 Agent gathering: the Drone's rules, the nearest Tunnel, the 1-square hatch as the drop-off target, and Agents wait at the hatch for ever. The Drones' placeholder (retry every 2 s, never give up) is the settled rule | done (M4-D, `edc30e11`); patches 0–1 GU from the hatch jam with 3+ Agents (Q0076) | answers Q0066; confirms Q0065 point 4 (an expansion's cells are reserved from construction start) |
| 0256 Supplies gathering: shared stations, waiting at an empty station, switching resources loses the load; the notification waits for a notification system | done (M4-I, `9aca8972`); Q0082's three proposals as placeholders | answers Q0067 |
| 0257 Syndicate panel letters on the 4x3 grid | queued (M4-J1, M4-J2) | answers Q0068, as proposed |
| 0258 Losing Tunnels: decay %, attacking neutral expansions, reclaiming, blocked cells (counted as occupied), defeat, and the enemy sees the tier | partly done (M4-A, `fcd06dc`: the tier seen and remembered); queued (M4-H) | answers Q0069 |
| 0259 The Syndicate economy check: measured to the hatch square; the stacked layout puts the Tunnels on opposite sides | queued (M4-L); the stacked layout is in M4-D's soak | answers Q0070 |
| 0260 Placeholders for the Laser Turret, the Grenadier and the Chemical Plant, so they can be planned into M4; the Grenadier needs a projectile, AoE and Attack Ground combat spike | queued (planned in M4 revision 2: O the combat spike, U the Grenadier, S1 and S2 the Laser Turret, K the Chemical Plant) | answers Q0071 (scope B) |
| 0261 Area damage never hits underground objects unless targeted | void (replaced by 0262) | — |
| 0262 Area damage does hit underground objects; expansions and the pop-up turret have high armor | partly done (M4-A, `fcd06dc`: damage never asks the hidden predicate); queued (M4-O: area damage hits underground objects); armor per 0264 | answers Q0071's AoE-underground gap |
| 0263 Confirmed: undeploy, held orders, the Transport's base, the Phantom's attack | queued (T44, the deploy model; gated N3 and N6) | answers Q0071's items to confirm |
| 0264 Building armor rule: PointArmor n, FullArmor n × size in grid squares; underground expansions 6/24 for 2x2; the Laser Turret 1/1 raised, 6/6 underground | partly done (M4-A, `fcd06dc`: the Hideout 6/24; M4-B, `35e19634`: the defs test across factions, all 11 buildings pass, no exceptions); queued (S1, S2: the Laser Turret) | — |
| 0265 Transport passengers always count against Tunnel Space | queued (gated N3) | answers Q0071's Tunnel Space gap |
| 0266 A burrowed Transport passes through underground structures; surfacing rules | queued (gated N3) | answers Q0071's underground and surfacing gaps |
| 0267 A Tunnel-built Laser Turret uses the operation slot; detection comes after the faction rosters | queued (M4-S2) | Q0071 complete (0260–0267) |
| 0268 New Syndicate unit, a single-payload Bomber (concept only) | gated (concept only) | — |
| 0269 The Bomber needs a Vault and a Chemical Plant, attacks ground only, re-arms in the network; the Vault works network-wide | gated (concept only) | — |
| 0270 The Syndicate bomber is named Specter | gated (concept only) | — |
| 0271 The Specter returns home | gated (concept only) | — |
| 0272 Specter: an unarmed Specter ordered to attack flies to the nearest own Tier 3 Tunnel to re-arm; entering wipes its orders; what happens after bombing | gated (concept only) | — |
| 0273 Specter: with no Tier 3 Tunnel, an unarmed Specter ignores Attack orders | gated (concept only) | — |
| 0274 Satellites require the Orbital Command | gated (concept only) | — |
| 0275 Satellite: laser relay, launched like a rocket, can't return to the network, slow with thrust charges | gated (concept only) | — |
| 0276 Refract: deployed Laser Batteries relay through Satellites, chains allowed | gated (concept only) | — |
| 0277 Relay (renamed from Refract): how Laser Batteries relay through Satellites | gated (concept only) | — |
| 0278 Relay: damage adds up, and chain breaks | gated (concept only) | — |
| 0279 Relay: every selected Battery must reach a Satellite; the armed Satellite takes Attack orders | gated (concept only) | — |
| 0280 Satellite thrust charges never recharge; the armed Satellite fires at anything the player sees | gated (concept only) | — |
| 0281 The satellite is named Prism; it can be shot down while launching | gated (concept only) | — |
| 0282 The design docs are reorganized into one file per unit and per structure (`factions/<f>/units/*.md`, `structures/*.md`, shared sections in each folder's README); no design content changed. From the user: keep the code organized and modular too: per-object definitions, data and tunables apart from logic, shared mechanics written once | done (M4-R1 `8eab2290`, M4-R2 `c60b8d0f`): per-object defs and client kinds, tuning modules, serde-ready defs, 0 stale citations, gated by a test | — |
| 0014 GDO Airfield and aircraft | queued (Later: air) | |
