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

## In progress
- **M3 wave 2** (common brief `scratchpad/briefs/m3-common.md`; status `m3-w2-status.md`):
  - **M3-E transformations and Warp In** (Opus): splits `sim/gate.rs` first. The Director added the generic client response to `KindChanged` to E.
  - **M3-D portals** (Opus; brief `scratchpad/briefs/m3d-portals.md`): exit rules (T19), unarmed rally, the data-driven Production panel.
  - **M3-H2 Drone pass-through** (Fable; brief `scratchpad/briefs/m3h2-drone-pass-through.md`): 0106's pass-through and overlap rules, the invariant exceptions, the `movement::stuck` finding, and the saturation curve. G starts after it, because both own `movement*`.
- **Next**:
  - **M1 playtest**: M1 is complete at `80b3d88`. The user has been asked to play `cargo run` (dev_skirmish).
  - **M2 playtest** with the user, using `docs/playtest/m2.md`.
  - **Fix the Move-onto-itself bug** per Q0031's answer, in the next sim or command-UI slice.

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
- **M3 Colonists** (**planned**; spec: `artifacts/director/m3-plan.md`, from the M3-P spike). It covers the statted opening: Drone gathering, Beacon Deployment and Warp In (Egress Node, Robo Portal, Bio Portal), portals with exits on several sides, Sabre and Rifle Legionnaires with Weapon Conversion, Shields, Beacon Capacity, Colonists Starting Conditions and layouts, HUD and panels, and an economy oracle against `Economy Targets (GDO vs Colonists)`. Refining, the tech tier and the other units are deferred (Q0043).
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
    | E | running (wave 2) |
    | D | running (wave 3) |
    | H2 (0106) | running |
    | B (next free slot), F (after E), G (after H2), J1, J2, K1, K2 | queued |
- **M4 Syndicate**: tunnels, Hideout, Agent, Guard economy. Underground expansions are exempt from the flatness rule (0005).
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
| Q0017 | Fog: shots from hidden attackers (hit flash vs. a partial tracer); targeting remembered structures | no | open; F and E build the stated defaults |
| Q0018 | Behaviour gaps: sight-edge flicker, rotation in target priority, AttackMove leash reference, unreachable auto-targets, MinRange, walled-out spots | no | open; E built the stated defaults |
| Q0019 | A remembered station's delivery countdown: keep counting or freeze | no | answered by 0057: GDO sees the real timer (folded into G2) |
| Q0020 | Riot Shield: shield Move vs 0062 Follow, transitions, raising mid-shot, mixed groups, toggle time and hotkey | no | point 1 answered by 0074 (Follow wins; H3); points 2–5 open (R built the stated defaults) |
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
| Q0031 | A Move or Follow aimed at a selected unit itself (it currently follows itself forever) | no | open; default: the clicked unit is left out |
| Q0032 | Arrival radius behind an occupied destination: 2 or 3 grid units (data from G3) | no | open; M1 keeps 2 |
| Q0033 | Path knowledge, blocked to open: a destination behind a gone obstacle, shortcuts when a wall is seen gone, map-placed structures | no | open; M2-E built the stated current behaviour |
| Q0034 | Should an enemy plate's InfoPanel show its patch's remaining crystals? | no | open; M2-G shows it to everyone |
| Q0035 | Rally point UI: Shift with rally targeting, rallying on one of several selected Barracks, feedback when pressing an unavailable command | no | open; M2-H1 built the stated current behaviour |
| Q0036 | End of game edge cases: simultaneous last defeats, free-for-all spectating, a raised Riot Shield on defeat | no | open; M2-F2 built the stated current behaviour |
| Q0037 | Drone gathering: the cases the loop doesn't define (13 points) | no | open; M3-H builds the proposals |
| Q0038 | Gathering Drones, collision and the economy targets | was yes | answered by 0106 (pass-through), 0107 (geometry), 0108 (±15%) |
| Q0039 | Beacon Deployment and Warp In: what the transforming object is | no (answer before M3-F) | points 1–6 answered by 0109–0114; points 7–8 open |
| Q0040 | Deploy and Warp In placement, and the Colonists panels | no | open |
| Q0041 | Portal exits and Beacon Capacity | no | open |
| Q0042 | Shields, melee range and Weapon Conversion | no | open |
| Q0043 | M3 scope: the Colonists content left TBD (FYI) | no | open |
| Q0044 | Gathering details from the build: fog, plates, empty Drop Off, blocked Drones, redirect timing, the Egress Node's sight | no | open; M3-H built the stated current behaviour |

## Design change entries
Last triaged: 0114.

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
| 0080 Exit doorway: units appear half inside; production pauses while it is occupied | done (M2-D2, `1a64e75`) | answers Q0023 points 2–3; replaces the nearest-free-spot default |
| 0081 Queue limit of 20 includes the unit in production | done (M2-D2, `1a64e75`) | |
| 0082 GDO Unit Control reserved when a unit is queued | done (M2-D2, `1a64e75`) | |
| 0083 Group production: least remaining work pays first | done (sim M2-D2; UI M2-H1, `eccaaa6`) | replaces the fixed-order default |
| 0084 Destroyed GDO producers refund as if cancelled | done (M2-D2 producers, M2-D1 constructors, `fda74d3`) | |
| 0085 Finished unit waiting inside is refunded if the building dies | done (M2-D2, `1a64e75`) | |
| 0086 Rally point on the building itself clears it | done (sim M2-D2; UI M2-H1, `eccaaa6`) | |
| 0087 Rally point indicators | done (sim M2-D2; UI M2-H1, `eccaaa6`) | |
| 0088 Rally on an enemy becomes a ground rally when the enemy leaves sight | done (sim M2-D2; UI M2-H1, `eccaaa6`) | rally on a remembered structure TBD (Q0017 point 2) |
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
| 0106 Gathering Drones pass through each other; the general overlap rule | in progress (M3-H2) | answers Q0038 point 1 |
| 0107 Reference check geometry: a 2-GU gap, edge to edge | done in M3-H's tests (`5bd3b69`); queued (M3-K1) | answers Q0038 point 2 |
| 0108 Economy check tolerance ±15%; the reference layout is only a benchmark | queued (M3-K1) | answers Q0038 point 3 |
| 0109 The transforming object is the target structure from the first frame | in progress (M3-E; then M3-F) | answers Q0039 point 1 |
| 0110 Damage stays taken during the transform interpolation | in progress (M3-E; then M3-F) | answers Q0039 point 2 |
| 0111 A transform cancel keeps the HP and Shields fraction, at least 1 HP | in progress (M3-E; then M3-F) | answers Q0039 point 3 |
| 0112 Beacon Capacity during Deploy and Warp In; over capacity allowed and shown red | red counter and blocked production done (M3-C, `5c75d40`); Warp In in progress (M3-E); deploy queued (M3-F) | answers Q0039 point 4, and Q0041 point 2 in effect |
| 0113 Structures under transformation count for defeat | in progress (M3-E); deploy-cancel case queued (M3-F) | answers Q0039 point 5 |
| 0114 A Drone deploying with a load keeps it; lost on completion, returned on cancel | queued (M3-F) | answers Q0039 point 6 |
| 0014 GDO Airfield and aircraft | queued (Later: air) | |
