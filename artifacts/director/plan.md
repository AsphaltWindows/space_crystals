# Implementation Plan

_Updated 2026-11-26. Spec: `artifacts/designer/design/`. Code rules: `ARCHITECTURE.md`._

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
  - **The citation test,** `crates/sim/tests/design_citations/`. It fails, naming file and line, on any cited design or `docs/` path that doesn't exist, with a planted-path test. An ignored report lists cited sections that aren't headings. It passes on the whole repository.
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
  - **The follow-up pass** (`06d82614`, comments only):
    - 707 citations written out as full paths.
    - The rule is gated by `design_citations/rule.rs`: a relative `> Section` only inside a module whose doc names one design file, and no bare design file names. It has no false positives.
    - 1548 tests.

- **M4-J2 Tunnel and Hideout UI** (Opus; merged in `c5910e4d`; 1561 tests; client only, plus the EjectMenu's order as sim data; goldens unchanged):
  - **The Tunnel's panel** (0257): Q Upgrade and W Expand greyed "Not available yet" until G and F; E opens the EjectMenu; X Cancel greyed "Nothing to cancel". No right-click.
  - **The EjectMenu:**
    - one tile per kind in `defs::EJECT_MENU_ORDER` (Agent, Guard; U adds the Grenadier), in grid order, with Z back;
    - each tile shows how many are in the network and not yet queued;
    - a tile is greyed with the gate's reason: none inside, all in transit (Q0077), or the tier too low;
    - a click ejects the oldest, and the menu stays open;
    - Z, Escape and right-click go back.
  - **The Hideout's panel:**
    - Q trains the Agent.
    - W Guard is greyed "Requires an Optics Lab", and E Grenadier "Requires a Chemical Plant". Both are `Planned` entries (`templates/planned.rs`); K and U replace them in `templates/syndicate/hideout.rs`.
    - X cancels; C sets the rally.
  - **The Tunnel Area overlay** (`placement/tunnel_areas.rs`, through the shared `overlay::tint_area`):
    - own areas at their current tier;
    - enemy areas at their live tier, or their last-seen tier;
    - never an unknown Tunnel, an unexplored tile or an off-map tile.

    It shows while an own Tunnel is the active group (placeholder, Q0086).
  - **Held units** leave the selection and stay in their control groups. Recalling an all-held group changes nothing, as with an empty group (placeholder, Q0086).
  - **0252 on the client:** a rally on a Hideout puts the unit over it; a rally on a Tunnel keeps the unit inside.
  - **Open:**
    - Q0086: the EjectMenu's order (`units/README.md > Files` contradicts 0257), its zero-count tiles, staying open, when areas are drawn, and all-held groups.
    - The greyed reasons' wording is a placeholder.
    - A rally on an expansion still shows the object marker.

- **M4-O Combat spike: projectiles, area damage, Attack Ground** (Opus; merged in `356d1a96`; 1589 tests; the 9 goldens unchanged, plus the new `test_defs/area_attacks.json`):
  - **Projectiles (T45):**
    - `Sim::projectiles`, in launch order, hashed only while non-empty;
    - they land in phase 5e, before that tick's hits, even if the shooter has died;
    - only DoublyDisjointed attacks launch them, as planned; a unit-target slot is kept for HeadDisjointed and TailDisjointed.
  - **Area damage (T46):** `combat/area.rs` is the only AoE path.
    - The circle–rectangle overlap is exact, through `sc_sim::math` (libm), with a test pinning its bits.
    - Candidates are chosen by layer and domain, underground objects included, even unseen.
    - The share, the effective armour, the directional zone (from the blast's centre), the minimum, and Shields at full value.
    - 0264's "about 93" over a hidden Hideout is a test.
  - **The sequence (T48):**
    - the location locks at the end of Aiming;
    - the projectile launches at the end of Firing;
    - the base is free in Cooldown;
    - new events: `ProjectileLaunched` (with its target, for Q0083 point 4) and `Blast`.
  - **Attack Ground (T47):**
    - `OrderKind::AttackGround`, and `AttackState::location`, hashed only while set;
    - the gate's `CannotTargetGround`;
    - walking into range, and backing off inside MinRange;
    - it fires once and completes at its blast (Q0083 point 3).
  - **Client (T52):** `combat_effects/{projectiles,blasts}.rs`.
    - Projectiles are drawn only over Visible tiles, with a cosmetic arc.
    - A blast ring shows when its centre tile is Visible.
    - Nothing is drawn for a hidden victim. One leak was found and fixed: a hidden Hideout's death marker.
    - The D slot gives Attack Ground, with a red location marker.
  - **Q0083's placeholders** (points 1–4), each marked in the code.
  - **Perf** (interleaved A/B, release; load 1.1–8.1):
    - the behaviours battle 0.319 → 0.326 ms (+2%), and attack-move 5a 0.075 → 0.077 ms; the rest within noise;
    - `projectiles_perf` (200 in flight): mean 0.11–0.12 ms per tick, worst 0.16 ms; landing ticks mean 0.15–0.16 ms, worst 0.26 ms.
  - **Open:** see the M4 entry's technical follow-ups.

- **M4-E Agent-built Tunnels** (Opus; merged in `4933e094`; 1609 tests; goldens unchanged, and the Drone's deploy hashes and saves byte-identical):
  - **The Build order (T51):** `OrderKind::Build { kind, origin, orientation }`, kind-generic, so S1's turret reuses it. The lingering ghosts are `Sim::pending_builds`, derived from the orders.
  - **The price (0233):**
    - `Sim::build_price` counts Tunnels owned, ghosts and sites included, in Supplies;
    - the order pays when it is given, and the gate refuses one the player can't pay;
    - cancelled, replaced or failed orders, and cancelled sites, refund the current top price (Q0085's placeholder);
    - a property test checks the books over 8 seeds of random orders, Shift-queues, cancels and replacements.
  - **The site (0234, T31):**
    - `Object::construction`, with the shared `Progress`, and the Agent held inside (`Place::Site`);
    - the ConstructionHP rule gives the maximum, and damage stays;
    - a cancel refunds, and lets the Agent out at the centre; a destroyed site loses its Supplies and lets the Agent out;
    - the site counts for defeat;
    - at completion the Agent is in the network, free to leave at once (a reading, Q0087).
  - **One "operational" predicate,** `Object::is_operational`. Exits, Enter, drop-offs and Tunnel Space all leave sites out.
  - **Placement:**
    - 0237: no overlap with any player's Tunnel Area, judged by the player's knowledge at the order and by the truth on arrival;
    - 0242: the whole area on the map and Drillable;
    - 0240: a hidden expansion fails the build on arrival;
    - Q0079's and Q0078's placeholders.
  - **Hash tags:** Build is `OrderKind` tag 11 and `Behaviour` tag 10, after O's.
  - **Perf:** the placement check on the crowded map went 4.5 → 4.9 µs; Tunnel Areas are collected only for kinds that have them. structures_perf and drones_perf are unchanged.
  - **The client** already draws a site as a Tunnel, with its HP bar. Until J1:
    - a site shows the Tunnel's panel, and the sim refuses its Eject (`NotATunnel`);
    - the red Tunnel ghost can't be shown.
  - **A reading:** a defeated player's pending builds are cleared with no refund. Their bank no longer matters.

- **M4-H Losing Tunnels** (Opus; merged in `f456b0ca`; 1622 tests; goldens unchanged; the ignored suite passes):
  - **The removal path** (T38, `sim/tunnels.rs`, hooked into `remove_objects`). For each lost Tunnel, in id order:
    - its completed expansions in its current-tier area turn neutral;
    - the unit in production is lost, the queue is refunded, and the rally is cleared;
    - its unfinished expansions are destroyed (`built_by`; F adds the `Builder::Tunnel` arm and its test);
    - then the held units that no remaining exit admits are removed (`network::admits`).
  - **A site that falls** neutralises nothing. A Tunnel mid-upgrade still neutralises its expansions.
  - **Decay:** `Object::decay: Option<Progress>`, hashed and saved only while set.
    - The rate is `tuning/tunnel.rs`'s `DECAY_PERCENT_PER_SECOND` = 1 (placeholder, 0258).
    - It is exact: a full Hideout loses 0.25 HP a tick, and dies 1599 ticks after its Tunnel falls (100 s).
    - `Sim::step_decay` runs after the sites step.
  - **Reclaiming:** `Sim::reclaim(tunnel)`, at a site's completion, with `Vision::forget_own`. No event is emitted, so nothing leaks. G calls it when an upgrade completes. `tier_requirement` returns 1 until F's defs.
  - **Defeat:** a defeated player's network units are destroyed (0258 point 5). Only an explicit Attack hits a neutral expansion; a right-click moves to it. Both are tested through the gate.
  - **The client** needed no change: neutral objects already draw in the neutral colour with their HP bar. A test was added (`fog_memory/neutral.rs`).
  - **Perf:** 18 interleaved A/B rounds. The whole tick is unchanged (0.340 vs 0.342 ms mean); the economy phase is +1 µs a tick, the decay walk.

- **M4-J1 Agent UI** (Opus; merged in `dd14dc38`; 1635 tests; client only; goldens untouched). **With H, the Syndicate's first playable.**
  - **The Agent's panel** (0257): Q Move, A Build Tunnel (its price in the hint), S Drop Off, D greyed with "Requires an Optics Lab", X Stop. An A, or a click, the player can't pay for gives the feedback and nothing else (0233).
  - **The Tunnel ghost** is judged by E's check in knowledge mode, with red tiles and the reason line. It shows the Tunnel Areas while placing, and is green over unexplored Water (0242).
    - Shift sends one Agent per placement, and the next needs Tab then A (`interface.md > Shift while placing`).
    - 0289's verification is a test: a Shift-queued second ghost over the first's area is red.
  - **Sites** (`object_render/site.rs`): a braced look and a progress bar, for any site and anyone who sees it. The site's panel is X Cancel only (`SITE_GRID`), keyed on `Object::is_site`.
  - **Lingering ghosts** and their areas show to their owner only. The Enter and Drop Off pointers follow the right-click rule. A carried Supplies load is an amber crate (placeholder).
  - **Readings:** A is greyed by the gate's unqueued check; a refused click keeps placement going; unsent Builds don't count toward the price.
  - **Fixed since** (M4-I2, `4f31587f`): a remembered enemy site is drawn as a site, at its last-seen progress.

- **M4-F Expansions** (Opus; merged in `2cbb777e`; 1675 tests; goldens unchanged):
  - **Placement** (`Method::Expansion`, T32, T53). An expansion must be:
    - inside the Tunnel's current area, and off its footprint;
    - on Drillable tiles only (0239; `TileProblem` widened to u16);
    - clear of every structure, judged by the truth (0240), and of patches and stations (0241);
    - clear of own and neutral underground structures (0258).

    Ground units don't block it, and there is no ghost check (0290). A Colonists Warp In over a hidden enemy expansion is judged by the truth: red at once (0290).
  - **The operation slot:** `TunnelState::operation` (`Operation::Building(site)`; invariant O1), hashed only while set. `Builder::Tunnel` sites use E's Construction, ConstructionHP and Cancel.
  - **Expand:** `Sim::can_expand` checks the operation, the Tier Requirement, the prerequisites (a stub until K) and the bank, then the truth. The click pays, and Tunnel Space counts the expansion at once (0254).
  - **Cancel:** the site's X or the Tunnel's X (`TunnelCommand::Cancel`) refunds 75%, rounded down (`EXPANSION_CANCEL_REFUND`, 0254). Eject works during construction.
  - **H's arms:** an unfinished expansion dies with its Tunnel, with no refund; the defs carry the Tier Requirements.
  - **The ExpandMenu** (`defs::EXPAND_MENU`; 0257, 0291): the nine expansions in the Tech Tree's order, then the Laser Turret tenth (C), greyed with the gate's reasons while unavailable; Z goes back.
  - **The client:**
    - W opens the menu;
    - the `Builder::Tunnel` ghost is held within the area and judged by J1's `ghost::check_now`;
    - the Tunnel's X cancels, and the InfoPanel shows "Building Hideout: N%";
    - J1's site look, now with its scaffold for underground sites too.
  - **The comments for 0283–0285** are updated.
  - **Readings:**
    - the ghost is held inside the area rather than shown red outside it;
    - a Shift-click places and ends placement, since a Tunnel has no queue;
    - the wordings "This Tunnel is busy" and "Building Hideout: N%" are placeholders.

- **M4-I2 Answers sweep** (Opus; merged in `4f31587f`; 1682 tests; goldens unchanged):
  - **0292 point 2:** a Supplies Agent picks its Tunnel by the distance to the crystal hatch, and still unloads at Side D. One rule, `economy::drop_off::nearest` (the first among equals), serves both resources, the first return and the reassignment after a lost Tunnel. Two layouts where the measures differ are tests.
  - **Verifications:**
    - 0292: an Agent moved off a station mid-pick-up keeps its crystals.
    - 0289: Guards kill an Agent walking to build, and its Supply comes back.
    - 0288: the EjectMenu's Guard tile is grey for exactly 32 ticks after it walks in, with nothing queued.
  - **The marks:** Q0077, Q0079 and Q0082 now cite 0288, 0289 and 0292.
  - **Remembered sites:** `LastKnown::site: Option<Progress>`, hashed only while set; the remembered MaxHP follows ConstructionHP at that progress. The client draws a dimmed site with its bar fixed at the last-seen progress (`MeshKey::of_record`), and the live bar never shows through the fog.

- **M4-O2 Area damage answers** (Opus; merged in `2e0f31f4`; 1685 tests; `test_defs/area_attacks.json` re-blessed):
  - **Attack Ground keeps firing** (0293 point 3). The location stays engaged, and the order never completes by itself, so queued orders after it never start. It ends only by another order, Stop, the Failure rule, or when the unit can no longer attack the location. `World::in_flight` and the projectile snapshot are gone.
  - **No drawing exception** (0293 point 4, replacing 0244). Shots, tracers and projectiles are drawn only over the viewer's Visible tiles, a revealed attacker's included. `Tracer`/`Flight::target_owner` are gone, and five `fog_memory` tests are flipped.
  - **Points 1 and 2 were as built** (friendly fire; the reveal when the damage lands). They are now tested: a grenade into the thrower's own melee, one at its own feet, and a dodged grenade from the fog with a twin that stays put.
  - **The golden:** thrower 1 now shells the Hideout spot 12 times instead of once. Its 5th grenade kills the Hideout at tick 205, defeating player 2. The golden's description is corrected.
  - **Perf** (A/B, interleaved): `projectiles_perf`, `behaviours_perf` and `attack_move_perf` are within noise.
  - **Left as built:** an attacker killed in the tick its blast lands still gets a reveal entry. It has no visible effect, since the attacker is removed that tick.

- **M4-K Prerequisites, the Optics Lab and the Chemical Plant** (Opus; merged in `bbdbea73`; 1702 tests; goldens unchanged):
  - **Generic prerequisites** (T37):
    - `requires` on `Recipe` and `StructureDef`;
    - `tech.rs`: a prerequisite counts only if it is the player's own, complete and not transforming, so neutral expansions and enemy labs don't count;
    - one check, `Reject::Requires(kind)`, at Train (enqueue), Construct, Warp In, Expand (F's stub made real) and a worker's Build, always before the bank check.

    Nothing checks again, so queued or started work goes on when the prerequisite is lost. The Guard's recipe requires the Optics Lab.
  - **The Optics Lab and the Chemical Plant** (0229, 0260, 0264): their own def and client modules (OL, CP), last in the registry, in the ExpandMenu's W and E. Each is 2x2, 150 SC, 400 frames, HP 400, 6/24, underground, Groupable, sight 0, with no commands.
  - **The client:** `requires_text` ("Requires an Optics Lab" for any gate refusal). The Hideout's W is the real Guard entry, greyed by the gate. `Planned` loses the Guard and the two labs.
  - **Readings:**
    - a worker's Build is checked when ordered, which is when it is paid, and arrival doesn't re-check (the order read as "queued at the constructor"; S1 keeps it);
    - a placeholder entry with a prerequisite says "Requires …" while the player owns none, and "Not available yet" after (`Conditions::planned`);
    - a GDO building ready to place doesn't count until placed;
    - the Grenadier is a sim unit-test stand-in until U.
  - **Tests:** `sim/tech/tests.rs` and `tests/prerequisites/` cover:
    - 0229's verification (an Optics Lab ordered at tick 2 completes at 402, and only then does the Hideout train a Guard);
    - losing the lab, by destruction or neutralisation;
    - the Chemical Plant;
    - Tunnel Space 11 → 10 → 9;
    - the prerequisite reported before the bank;
    - other players' and neutral labs, and both hidden from the enemy.

    `building_armor` now covers 13 buildings and 3 expansions at 6/24. The client tests cover a GDO Construct and Train refused with "Requires a Generator".

- **M4-G Tunnel upgrades** (Opus; merged in `a737a69a`; 1735 tests; goldens unchanged):
  - **The sim:**
    - `TunnelCommand::Upgrade`, `Operation::Upgrading { progress, to }`, and Cancel's upgrade arm;
    - the frames and prices in `TierDef::upgrade` (`defs/syndicate/structures/tunnel.rs`);
    - one price function, `Sim::price_to` ("not counting itself"), for the order and every refund;
    - `Sim::can_upgrade`: Tier 3, operation, Supplies, then the truth (0242/0283's area rule at the target tier; any Tunnel's area, sites and other upgrades' reserved areas);
    - 480 frames, after `step_sites`; completion is `set_tier`, then `reclaim`.
  - **The reserved area:** `Occupant::of` uses the target tier for the truth and the owner. Other players get the current tier (`Occupant::as_seen`), so their build fails on arrival.
  - **Own pending ghosts don't block an upgrade** (the Director's reading). Their build then fails on arrival. This is tested, with a comment.
  - **The client:**
    - Q's hint shows the price ("Q: Upgrade Tunnel (2 Supplies)"), and Q greys with the gate's reasons ("Something blocks the upgrade", a placeholder);
    - the InfoPanel shows "Upgrading to Tier 2: N%", and X cancels;
    - the overlay shows own reserved areas while placing a Tunnel (M4-J3 reworks this for 0299/0300).
  - **Tests:** `tests/upgrades/` (21) covers 0229, 0232/0284 (both cancel orders; books balanced over random orders), 0237, 0242, 0283 (all four edges, 3 and 4 tiles), 0249 and the reading above. A unit test covers 0231's 500/600 → 700/800.
  - **Perf** (A/B): the placement check 5.55 → 5.40 µs, and the `structures_perf` battle 0.381 → 0.366 ms.
  - **Reading:** `DebugCommand::SetTier` ends an upgrade in progress with no refund.

- **M4-L Syndicate oracle and goldens** (Opus; merged in `6d05e084`; 1744 tests; two new goldens; three re-blessed for the stored map only):
  - **0294:**
    - The Agent's MiningDuration is 64 frames. The tests that follow from it are updated (`agent_gathering`, `agent_supplies/switching.rs`, the gathering unit test, `syndicate_start.rs`).
    - `dev_skirmish`'s Syndicate layout moved, not the patches: the Tunnel to (6, 6) and the Hideout to (10, 10), with the hatch (6, 7) 3 GU from the patches. The map JSON matches.
    - `gdo_opening`, `colonists_opening` and `colonists_vs_gdo` store the map, so they were re-blessed. Only the layout's positions changed, and no hash line moved.
    - No Q0076 mark remains (0294, 0295).
  - **The oracle** (`tests/economy_oracle/syndicate.rs`, `docs/economy_oracle.md`; one test per row):
    - every row from 3 GU passes ±15%;
    - the 12.5 ceiling is exact (no two deliveries less than 64 ticks apart);
    - stacked Tunnels take in 23.33 against 25;
    - the third Agent is out at 0:39.56, inside 0:34–0:46;
    - the Syndicate's one-base income is the lowest, at 12.5 against the GDO's 26.67 and the Colonists' 32;
    - rows 0–2 are recorded only.

    3 Agents at 3 GU measure 10.67 against 12.5, which is −14.7%; this went to the Designer as Q0089.
  - **The goldens** (`tests/golden/syndicate.rs`):
    - `syndicate_opening` runs to 2:15: Agents trained and rallied, the third builds the second Tunnel and brings 2 Supplies, and an Optics Lab is built.
    - `syndicate_vs_gdo` runs to 3:06.75: an Optics Lab, then 6 Guards against Peacekeepers. Both Tunnels fall, the expansions go neutral and decay, Guards are stranded, and player 1 is defeated.
    - The client's replay test now counts 11 goldens.
  - **`testing::syndicate`** has a build line through the gate (Train, Eject, Gather, Gather Supplies, Build Tunnel, Expand, Upgrade). `tests/syndicate_line.rs` upgrades a Tunnel with gathered Supplies.
  - **Perf:** the `economy_oracle` binary went from 2.85 s to about 4.9 s in debug; the goldens add 0.1 s.

- **M4-U Grenadier** (Opus; merged in `f34e4e6e`; 1752 tests; the 11 goldens unchanged):
  - **The kind** (`defs/syndicate/units/grenadier.rs`; `ObjectKind::Grenadier` last), every number cited from 0260. Its grenade is a DoublyDisjointed Ground AoE: radius 0.625 GU, 8 su per tick, and a 6/2/4/28-tick cycle.
  - **Training:** the Hideout's E, for 150 SC over 160 frames, requiring a Chemical Plant (K's `requires`). It is third in the EjectMenu (0257, 0285).
  - **The client:**
    - its module (GR; the BasicCombatUnit grid with Attack Ground);
    - `ShotStyle::Grenade`: a projectile takes its thrower's style through the new `Flight::weapon`, drawn with a lit fuse (placeholder; small at the default zoom);
    - the Hideout's panel is now the generic Production template.
  - **Tests** (`tests/grenadier.rs`):
    - 0260's verification: 32 ticks over 4 GU, and a walker escapes;
    - one grenade per 40 ticks;
    - 0264's verification: about 93 over a hidden Hideout, and its owner learns nothing;
    - a Tier 1 Tunnel's entry and eject.

    Training through the gate is in `tests/prerequisites/`, replacing K's stand-in. `defs_are_consistent` checks that only Disjointed attacks have a ProjectileSpeed.

- **M4-J3 Tunnel Area display** (Opus; merged in `5e2e9419`; 1760 tests; client only, plus comments; goldens untouched):
  - **Selecting own Tunnels** shows each one's area at its current tier, sites included, and the area hides when the selection changes. Enemy areas, seen or remembered, are never drawn (0299). The ghost still judges by the ones the player knows (0237).
  - **Placing a Tunnel** shows the ghost's Tier 2 and Tier 3 rings (`placement/tunnel_areas/rings.rs`). A ring cell is red when it is:
    - off the map (shown as a red line along the map's edge, since the camera can't show off-map cells);
    - on known non-Drillable ground (never-explored tiles pass, as on the ghost);
    - on an own area (0300).

    "Own areas" are what the ghost's own check counts: Tunnels and sites (at their reserved tier while upgrading) and pending Tunnel ghosts. That's stricter than `Sim::upgrade_blocked`, which doesn't count pending ghosts, because the preview anticipates them being built.
  - **G's `reserving` is gone.** The mesh is rebuilt only when what it shows changes.
  - **The marks:** every Q0086 and Q0087 cites 0285, 0299, 0300 or 0301. Colours and wording stay placeholders (Q0001).
  - **Tests:**
    - 0299: Water red only in the T3 ring, and unexplored Water not red; an enemy Tunnel selected draws nothing.
    - 0300: an own Tunnel's overlap is red in both rings.
    - 0301: the builder is out in the Eject's tick, and a unit rallied inside has no delay.
    - The tests of Q0086 points 1, 2, 3 and 5 cite their entries.

- **M4-S1 Laser Turret, upright and Agent-built** (Opus; merged in `50f51141`; 1782 tests; the 11 goldens unchanged):
  - **Structures that attack** (T49): `StructureDef::attack`, and a `Shooter` built from the object. A complete turret picks its own target in phase 5e, by 0298 point 8: threats first, then it keeps its target, then the closest, then the lowest id. It picks nothing when auto-targeting is off.
  - **The Agent-built turret:**
    - it requires an Optics Lab, checked at the order (K's reading);
    - it costs 100 SC at the order (a typed `BuildPrice`);
    - a cancel refunds 75 (0297 point 5, folded in mid-run), and a destroyed site refunds nothing;
    - the Agent rides inside, untargetable, for 320 ticks, then comes out beside it (0298 point 7). It comes out at the centre on a cancel, and waits inside if no spot is clear.
  - **The client:**
    - the Agent's D (greyed with the gate's reason; hint "(100 Space Crystals)");
    - the 1x1 ghost, the LT tag, and a red laser;
    - a barrel that turns to its target.

    The ExpandMenu's tenth slot stays a placeholder for S2.
  - **Tests:** combat structures (7), `tests/builds/turret.rs` (8), the turret case in `losing_tunnels`' free-rebuild test, and the defs, plus the client tests.
  - **Perf** (15 interleaved rounds at load 1.0): battle 0.314 → 0.317 ms, attack-move 1.000. Before `#[inline]` on `World::pick`, a codegen-unit split cost 2.9%.
  - **Readings:**
    - "the side it came from" is opposite the Agent's facing when it reached the site;
    - `PanelSlot::price` and `Conditions::pricing` take a `BuildPrice`.
  - **The robustness fuzz** asserts that nobody is defeated, but one random stream in sixteen lets a Grenadier destroy a base. S1 moved the seed to 0x5eee; M4-M makes the test robust.

- **M4-V One per-viewer event filter** (Opus; merged in `dbefb552`; 1784 tests; goldens unchanged; QA screenshots byte-identical to master's):
  - **The filter:** `crates/game/src/viewer_events.rs`.
    - `filter_events` (set `FilterEvents`, in SyncSim) is the only reader of `SimEventMessage`, apart from the QA state dump.
    - Each event gets an `Audience`:
      - the owner alone;
      - whoever sees the object;
      - whoever sees the place (a shot's path is clipped for every viewer, the shooter's owner included);
      - a removal, never shown to a player who remembers the object out of sight;
      - everyone.
  - **Moved onto it:** the combat effects (tracers, the Laser Turret's laser among them; projectiles, blasts and death markers), the feedback line, the station notice and the per-player selection sync.
  - **One latent leak closed:** death markers judged hiding by kind (`Hiding::of_kind`), which S2's lowered turret would have broken. `SimEvent::Died` now carries `hiding: Hiding::of(o)`, an additive field with a serde default; events are outside the state hash.
  - **S2's hook:** `Hiding::of` in `crates/sim/src/vision/hidden.rs` is the one place for a new arm. ARCHITECTURE names it under "Vision > Hidden objects" and in a new section, "Client architecture > Viewer events".
  - **Tests:**
    - a table test over every variant (`viewer_events/tests.rs`, `tests/table.rs`), for the owner, an enemy who sees, an enemy in the fog, and with the fog off, plus hidden objects and a turret firing;
    - a variant with no row fails the test;
    - `tests/fog_memory/station.rs`: a remembered station removed out of sight stays selected.
  - **One deliberate change:** a player's own event about an object that died or left in the same frame now reaches that player. Before, it reached no one.
  - **Open, as built:** Q0090 (an unseen shooter's colour, another player's defeat, marks seen late), marked `Placeholder (M4-V)`. A shot at a hidden object shows over Visible tiles up to it. That can only happen in the tick it hides, since nothing can target what it doesn't see.

- **M4-S2 Laser Turret, pop-up and Tunnel-built** (**Fable**; merged in `0cee8bc4`; 1814 tests, 21 ignored; the 11 goldens unchanged, with the pop-up's state hashed only while it has one):
  - **The form is derived** (T50; `popup.rs`):
    - `Object::popup: Option<PopUp>`, with `Lowered { blocked }`, `Rising`, `Raised { idle }` and `Retracting`; `None` means upright;
    - a turret is a pop-up if and only if its tile is in an own operational Tunnel's current area;
    - `Sim::step_popups` runs in its own phase (`Phase::PopUps`), right after 5e;
    - `PopUpDef`: 8, 32 and 8 frames; 6/6 while lowered.
  - **Hiding:** `Hiding::of`'s lowered arm, and nowhere else (M4-V's hook). M4-V's table now uses the real lowered turret, plus a row for a raised turret's death.
  - **Blocking (0296 point 3):**
    - the footprint blocks from the tick the turret starts to rise until it is fully lowered (the reading, marked);
    - the rise waits while any ground unit's box overlaps the tile, and the turret keeps its target meanwhile;
    - own idle units get a **yield request** (`yielding::make_way`, M1-Y's sidestep with its chain), not `separate.rs`'s step;
    - `paths::blocking(object)` makes a raised turret a navigation obstacle.
  - **The Tunnel-built turret:** the ExpandMenu's C is real. It requires the Optics Lab, and the gate greys it. It uses the operation slot (0267: Expand and Upgrade are refused with `OperationUnderWay` meanwhile), its site is hidden, and a cancel refunds 75%.
  - **Tunnel Space:** ¼ per pop-up, through the derived floor rule. Two pop-ups and an Optics Lab leave floor(42/4) = 10 on a Tier 1 Tunnel.
  - **Conversion:** a new Tunnel or an upgrade over a turret makes it a pop-up; a Tunnel's death makes its pop-ups upright.
  - **The client:** an outline while lowered, and a barrel that grows on the rise.
  - **Timing as built:** a target picked in tick t means raised at the end of t+8 and the first shot at t+13. The last target in tick t means lowered at the end of t+40.
  - **Readings:**
    - expansion placement now reserves the owner's pending worker-build sites. 0290's "no ghost check" was about Tunnel ghosts, which can't be in another Tunnel's area; an Agent's turret ghost can (0289's rule);
    - an Agent-built turret in an area completes raised, and retracts after 32 idle frames.

    Five more are filed as **Q0091**:
    - a Tunnel's death with a unit on the lowered turret;
    - a target while it retracts;
    - Buildable and flat ground for a Tunnel-built turret;
    - Tier 1;
    - when a site takes its ¼.
  - **Tests:**
    - `crates/sim/tests/popup/` (22: the cycle, blocking, hiding, building and conversion), and unit tests in `popup.rs`;
    - an area-damage case (a blast against a lowered or raised turret), and `building_armor.rs` for both forms;
    - the client's look and the ExpandMenu tests.

    0296's verifications are tests. The Agent case asserts only the outcome, since an idle Agent chases the enemy off the tile by itself. The push is proven with an unarmed Drone.
  - **Perf:** `popup_perf` (ignored) runs 12 pop-ups cycling under 200 walkers on 256×256: the mean goes 0.179 → 0.197 ms (+10%), and p99 0.353 → 0.406 ms; the worst is unchanged. It caused 227 grid changes in 1280 ticks. `nav_perf`, `movement_perf` and `structures_perf` are within noise.
  - **Accepted limits:**
    - idle units packed against a wall may fail to sidestep, and the turret then stays down (0223: predictable, not optimal);
    - an enemy's record of a turret is drawn raised, since that is the only way an enemy ever sees one.

- **M4-M M4 QA** (Opus; merged in `4e3fdd4a`; 1814 tests, 23 ignored; goldens unchanged). **M4 is complete with it.**
  - **Perf** (release, load 0.3–0.6), per sim tick:
    - `agents_perf`, 4 Tunnels with 16 Agents and 60 Guards cycling: mean 0.043 ms, p99 0.067 ms. With 40 Grenadiers shelling: 0.070 and 0.090 ms.
    - `popup_perf`: 0.194 and 0.406 ms, worst 1.44 ms.
    - `projectiles_perf` (200 grenades in flight): 0.110 and 0.192 ms.
  - **`syndicate_soak`** (ignored; 10 minutes of the golden game) delivered:
    - 5800 SC and 8 Supplies;
    - 12 grenades, 11 pop-up rises and 27 turret shots;
    - 15 units into the network and 34 out;
    - 4 expansions and 1 upgrade.

    Every projectile lands by its arrival tick, the network's invariants hold, and the rerun and replay hashes match.
  - **The robustness fuzz** accepts a game's end. Its seed is 0x5eed again, with lower thresholds: spawned and removed above 40, and unit steps above 400.
  - **The walkthrough:** 19 screenshots (`scratchpad/m4m/`, made by `m4m/walk/walkthrough.sh`), all looked at. They include the blast over a hidden Hideout from both sides, and the pop-up raised and lowered.
  - **The ignored suite:** all 23 pass in 56 s, at load 0.9.
  - **Docs:**
    - `docs/playtest/m4.md`;
    - ARCHITECTURE's "Adding a faction" gains items 15–18, and "Adding a unit or structure" covers M4's kinds;
    - M4-V's doubts now cite Q0090.
  - **Findings, for follow-ups:**
    - **Four Agents on one crystal hatch jam for good:** in the soak at 6:54, after which the income stops. Three never jam. Until it's fixed, the soak runs 3 crystal Agents and 1 on Supplies. Fix: **M4-W** (Fable).
    - **Strays,** fixed by **M4-Z**:
      - `Placeholder: Q0060 point 1` in `defs/syndicate/structures/tunnel.rs` (answered by 0229);
      - Q0091's as-built points carry no marks;
      - `testing::syndicate::eject_builders` treats a turret's `SiteCompleted` as a Tunnel's;
      - `Books`/`SupplyBooks` miss a turret's SC refunds and `ExpansionCancelled`.
    - **Small display notes:**
      - a site's InfoPanel shows its current construction maximum ("HP 74 / 74" at 10%) with a full bar;
      - a revealed shooter is drawn over unexplored ground.

- **M4-W The four-Agent hatch jam, diagnosed** (**Fable**; merged in `189d0162`; tests and docs only; no sim change). **The fix waits on Q0109.**
  - **Where it happens:** any four or more Agents whose patch lies to the side of the hatch, so that they come along the Tunnel's wall. `dev_skirmish`'s own field is like that. Facing the hatch (the oracle's layout) nothing jams.
  - **Why it locks:**
    - every return leg ends at one point (`legs::return_leg`, the near end of the wall within reach of the hatch);
    - the Agent unloading there is boxed in. Its steering never steps back along a face, and its re-routes repeat the same path;
    - the waiting Agents retry for ever (0255), and since all hold orders, none yields.
  - **Measured** north-west of the hatch, in SC/s with 4/5/6 Agents: as built, 0.0–4.0 at 2–3 GU, and 7 of the 15 cases go over 60 s without a delivery. Prototype (a), where a waiting Agent steps aside, gives 8.5–10.2. Prototype (b), where legs end only in front of the square, still jams in places.
  - **Tests:** two ignored tests that fail as built, in `crates/sim/tests/agent_gathering/jam.rs`: a focused repro, and a stress run with 4–6 Agents at 1–5 GU.
  - **Docs:** `docs/playtest/m4.md`'s Known issue carries the diagnosis and the table.
  - **The fix** is a new who-gives-way rule, so it is design: **Q0109**, with options (a)–(e). It must land before the playtest.

- **M4-Z The QA strays** (Opus; merged in `95b4bc6e`; comments and test helpers only; goldens unchanged; 1815 tests, 25 ignored).
  - **Marks:**
    - Q0060 point 1 now cites 0229 as the rule (the code matches it);
    - Q0091's five as-built points are marked `Placeholder (Q0091 point N)` where the code decides each one: `popup::step`, `placement/check.rs`, `LASER_TURRET_EXPANSION.tier` and `tunnel_space::occupies_tunnel_region`.
  - **`testing::syndicate::eject_builders`** acts only on a Tunnel's site (a turret's Agent leaves by itself, 0298 point 7), with a test in `syndicate_line.rs`.
  - **The test books** count every refund in its own resource (`build_order::refund_in`), including a Laser Turret's SC refunds and `ExpansionCancelled`. The books property test (`builds/price.rs`) now plays turret Builds and Tunnel 1's expansions, and balances both banks every tick.
  - **Left over:** `syndicate_soak.rs` keeps its own Eject workaround, which could now use the helper. It goes to the Q0109 fix, which reopens the soak.

- **M5-A The Cults foundation** (Opus; merged in `ed23874e`; 1830 tests, 25 ignored; goldens unchanged).
  - **The six designed kinds:** Recruit, Militant, Gunner, RecruitmentHall, Storage and Armory.
    - Each has its own def module with its design test, a registry line (last), a client kind (RT, MT, GU, RH, SE, AY) and an editor palette entry.
    - `unit_control_cost: Some(1)` on the three units (T55).
    - The Armory's `requires: [Storage]` is data only, and no system field is set yet.
  - **The Gunner** is TailDisjointed as data (0304), Range 5, MinRange 2. It hits as FullyConnected until O3.
  - **`Object::lineage`** (T56): optional, hashed only while set, and kept by `change_kind`. Nothing sets it yet; C does. It is left out of `testing/references.rs::referenced_ids` on purpose, since a lineage may name a Hall that is gone.
  - **The Storage's armour** is 1/6 as written (`Placeholder (Q0094)`), listed in the armour test's `EXCEPTIONS`.
  - **A fix found by its MinRange test** (`7cbd6652`, its own commit): a unit that can move but is cornered now gives up an Attack with "Target too close", as `systems/controls/command_behaviors.md > Target inside MinRange` and 0204 point 3 say. Before, it dropped the order silently. Only the Gunner has a MinRange.
  - **Tests:** `tests/cults_combat.rs` (a Militant fights a Peacekeeper to a result; the Gunner backs off, then fires, never from inside 2 GU; a cornered Gunner reports Target too close), the defs' serde round trip and labels, the armour test (17 buildings) and lineage's hash and save.
  - **Handed to C:** `unit_control_cost` counts for any owner, so debug-spawned Cults units cost a GDO player GDO Unit Control. The Director decided that a unit costs only in its own faction's resource, as `economy/army.rs`'s docs say.
  - **Left as is:** the client offers the Gunner Attack Ground (`can_target_ground`), and the order ends at once until O3.
  - **Perf** not run: no hot path changed (one Option per object in the hash, and a rare MinRange branch). Screenshot: `scratchpad/m5a/cults_kinds.png`.

- **M5-R Playtest pairings** (Opus; merged in `afd9c152`; tests and docs only; no game code changed; goldens unchanged).
  - **`crates/sim/tests/pairings/`** (`main.rs`, `openings.rs`): every ordered pair of factions with Starting Conditions (9 now; the Cults join with no edit after B), each with a scripted opening.
    - **The openings:** the GDO's golden Barracks line plus 6 Peacekeepers; the Colonists' golden opening; the Syndicate's 3 Agents on a hatch (Q0109; 0315 not built yet), an Optics Lab and 4 Guards.
    - **Checked every tick:** the invariants, held units and references, the gate taking every command, both books, and that nobody is defeated.
    - **The end:** player 2's buildings are removed by debug commands, and player 1 wins.
    - **Adding a faction** is one line in the `OPENINGS` table (M5-L adds the Cults').
    - **Naming:** each pairing runs on a thread named after its command line.
    - **Runs:** the default run is 9 × 30 s and takes 0.13 s. The ignored run is the built-in `dev_skirmish` and `maps/dev_skirmish.json`, 18 × 3 minutes.
  - **`crates/game/tests/pairings.rs`:** `--factions a,b --headless --exit-at-tick 64` with `--scenario` or `--map`, for every pair and map, in-process through `cli::parse`, `cli::into_config` and `build_app` (a spawned headless run paces by the clock). 18 runs.
  - **`docs/playtest/README.md`:** the 9 commands, hot-seat with F2, F3, Tab, the debug keys, replays, links to m2–m4, and a template for reporting a finding (design to the Designer, bugs to the Director). It was checked against `cli/usage.rs`, the key bindings and screenshots.
  - **Left for B:** the `--factions` line in `crates/game/src/cli/usage.rs` is out of date. B owns that file.

- **M5-D Recruit gathering** (**Fable**; merged in `36a06de4`; 1845 tests, 29 ignored; goldens unchanged; `movement/` untouched).
  - **The data:**
    - the Recruit's `WorkerDef`: 2 per load, 8 frames, 2 frames, 8 su, 8 su, `shares_patches`, no pass-through, no Supplies. `builds` stays empty for E;
    - the Storage's `DropOff::Anywhere`;
    - `economy/gathering.rs` is split into a directory.
  - **The drop-off choice** (`WorkerDef::drop_off: DropOffChoice`, which replaces the `keeps_drop_off` bool): `Assigned` for the Drone, `FirstReturn` for the Agent, and `EachReturn` for the Recruit.
    - **EachReturn** is `Placeholder (Q0102 point 1)`: the nearest Storage to the patch, edge to edge, chosen as each return starts. With none, the Recruit idles with its load.
    - **A Storage lost mid-return** is replaced by the nearest to the Recruit, by 0170's shared rule. That case is added to Q0102.
    - **The search radius** is 6 GU, a placeholder.
  - **Single-Recruit rates** at gaps of 1, 2 and 3 tiles: 1.33, 0.57 and 0.36 SC/s, against the design's 1.45, 0.59 and 0.37 (−8%, −4%, −2%). A Recruit stops a few su inside each reach, so its trip is 24 frames, not 22. **L's oracle should expect 24.**
  - **Crowds within the design's numbers don't jam:**
    - 4 or 5 Recruits on a 2-tile face give 5.3 and 6.1 SC/s;
    - 10 on an 8-patch field with 3 Storages give 12.9 SC/s;
    - one tile closer gains 0.77 SC/s per Recruit (the design says about 0.86).
  - **More Recruits than a face's lanes do jam,** head-on in a lane with nobody yielding: two ignored tests in `tests/recruit_gathering/jam.rs`, with the table. **Q0110** filed. 0315's rule, prototyped for Recruits and reverted, doesn't fix it.
  - **Perf** (`recruits_perf`, 60 Recruits): mean 0.062 ms, p99 0.089 ms per tick. `drones_perf` and `agents_perf` hold by interleaved A/B.
  - **Screenshot:** `scratchpad/m5d/recruits_field_400.png`.
  - **Left for C:** the rally-gather test. C, landing second, writes it un-ignored.

- **M4-W2 The hatch rule, 0315** (**Fable**; merged in `b81b2089`). **Built as designed, and it doesn't fix the jam:** Q0111 is filed.
  - **The rule:** `Mover::presses_on`, set by `orders/gather/legs.rs::return_leg` on the leg to a hatch only, exempts that leg from the occupied-target rule (`movement/stuck.rs`). When the mover gives the leg up, the next leg begins on the next tick, so the Agent never gives up. The flag is hashed while set.
  - **Measured** (10 minutes, north-west of the hatch, 4/5/6 Agents): better at 2 GU, much worse at 3 GU with 5–6 Agents (4.0/2.7 → 0.6/0.5 SC/s), and still nothing at 1 GU.
    - Facing the hatch, income never stops for 60 s, but 1–2 GU fall (5 Agents at 1 GU: 11.9 → 5.9).
    - Per Agent (0315's Verification), 20 of 30 layouts fail, against 19 before.
    - The soak with 4 crystal Agents still stops for good, at 6:53. The table is in `docs/playtest/m4.md`.
  - **The remaining mechanism:** the Agent that just unloaded is pocketed at the wall, the loaded Agents press toward points it covers, and nobody yields. W2 also found a technical cause: the Agents queued behind it are never judged stuck, because its shuffles count as their progress, so they never re-route. That is existing movement behaviour, left alone for now.
  - **Tests:**
    - `the_income_never_stops_where_it_holds` asserts the 21 layouts that hold;
    - the 9 that still jam (`STILL_JAM`) and the per-Agent check stay ignored, citing 0315;
    - new: `an_agent_blocked_from_the_hatch_presses_on_and_never_gives_up` and `only_the_leg_to_a_hatch_presses_on`;
    - the Recruits' tests are unchanged.
  - **The economy oracle** (3 Agents, 1–6 GU): 6.00/11.00/10.67/11.17/10.33/9.67 → 9.83/11.50/11.83/11.00/10.50/9.67, within ±15%. `docs/economy_oracle.md` is updated.
  - **Goldens:** `syndicate_opening` and `syndicate_vs_gdo` are re-blessed, diverging at tick 160 (the first return leg). `syndicate_vs_gdo` now ends at tick 2998 (`VERSUS_END`).
  - **Perf** holds by interleaved A/B (`agents_perf`, `nav_perf`, `movement_perf`).
  - **Not done, since the jam remains:** the soak's fourth crystal Agent, and swapping in `eject_builders`.

- **M5-C Recruitment Halls** (Opus; merged in `afd9b366`).
  - **Claims** (`recruitment.rs`), worked out every tick: completed Halls in `completed_at` order, then by id; Recruitable tiles under any footprint count; off-map tiles count toward the 100 but can't be claimed. The only state a Hall stores is `Production::completed_at`, hashed and saved only while not 0.
  - **Production** (`sim/recruitment.rs`): a Hall starts a Recruit while 100 × usage < 20 × claimed tiles, and queues the next at release. Progress gains the claimed tiles each tick, up to 192 × 100. X restarts production in the same tick; Train is refused with `CannotProduce`.
  - **Exits:** `ExitRule::Sides(A)` gives a door on each side: the one facing the rally first, else the south one, then clockwise. The Hall waits only while every door is occupied.
  - **Unit Control:** `ArmyResource::CultsUnitControl`. Available = floor(sum of capacities); used = the Halls' usage plus orphan points. It reserves nothing. `Object::lineage` is a `Vec<ObjectId>` (T56 amended). A unit costs only in its own faction's resource, tested both ways.
  - **Client:** the HUD's Unit Control; the Hall's InfoPanel (effectiveness and tiles, Unit Control, Recruiting %, the rally); X greyed while nothing is in production.
  - **Tests:** 17 in `tests/recruitment/` (e.g. `a_full_hall_fills_from_6_to_20_in_2688_ticks_then_stops`, `a_hall_at_fifty_tiles_makes_ten_at_384_ticks_each`, `the_second_hall_takes_the_freed_tiles_in_the_same_tick`, `a_destroyed_halls_recruits_still_count_and_its_capacity_goes`), 5 unit tests, and the client's panel, HUD and InfoPanel; 1877 tests in all.
  - **Goldens** unchanged. **Perf:** the Economy phase went from 0.009 to 0.039 ms a tick (claims are recomputed every tick); the whole tick is within noise.
  - **Open:** neutral Halls claim tiles but make nothing; the Cults' figure isn't shown red over capacity (the design gives no warning); a Recruit that terrain leaves in the rally side's doorway keeps that door occupied (`core/entities.md > Structure Type > Exit Doorway` point 5).

- **M5-F1 Buildings that hold units** (Opus; merged in `e0444eab`). One capability for every Cults building that holds units, built for the Armory.
  - **The def:** `StructureDef::holds` = `HoldsDef { entrance, exit: ExitRule, capacity, accepts }`; the Armory's is B in, C out, 10, Recruits. Units inside are held at `Place::Building` in T30's store.
  - **Enter** is M4-C's, generalized (`network::admits` became `holding::admits`; `network::EntryPath`). Room is judged on arrival, and `EnterFailed` gained `target` and `full`; the client shows "No room inside". A right-click on an own working Armory is Enter even when it is full.
  - **Eject All** (`Command::Holding`) lets units out through M2's doorway one at a time, about 12 ticks a unit while there's no rally (F2 adds it).
  - **Evacuation** runs in `remove_objects`: it reads the held units before the refunds (F2's hook) and places them before the defeat check, at T62's spots (`placement/spots.rs`, `Sim::spot_around`) in Q0106's order. An evacuee with no free spot anywhere on the map leaves the game (Placeholder). New events `EnteredBuilding`, `LeftBuilding` and `Evacuated`, each with its `viewer_events` row.
  - **The corner snag** (QA): a crowd pressed one Recruit along the wall toward a stand spot it never got round. A holder's stand spot is now 4 su off the wall (`EntryPath::standoff`; Placeholder, not in the design); the Tunnel's stays at 0, so its tests and the goldens are unchanged.
  - **After C:** `Doorway::of` accepts C's `Side | Sides`, and `Sim::release_held` takes the first free door in `Doorway::order(None)`.
  - **Placeholders:** Q0104 points 1–5 and Q0106 points 1–4.
  - **Tests** (`tests/holding/`, 18): ten Recruits enter from a block and from every side; the 11th fails with `full`; a loaded Recruit keeps its load inside and delivers it after; Eject All leaves side C with never two in the doorway; ten are evacuated alive at exact spots, clockwise from C's middle, before defeat; an Armory under construction refuses Enter; `control.rs`: a Recruit inside counts in its Hall's usage, inside and after evacuation, and is an orphan when its Hall goes. 1906 tests.
  - **Goldens** unchanged. **Perf:** `agents_perf` and `recruits_perf` unchanged; the new `holding_perf` (8 Armories, 80 Recruits) cycles at 0.028 ms mean, 0.071 ms p99, and evacuating all 80 takes 0.22 ms.
  - **Open:** the Tunnel's stand spot is still against the wall and may snag the same way in a crowd (not checked); the Garrison's "any side" needs an `ExitRule` variant (N2's `AnySide`).

- **M5-O3 TailDisjointed attacks** (Opus; merged in `e64cf8d2`; pulled forward from W4).
  - **TailDisjointed** (0304): the shot locks a point at the end of Aiming and, at the end of Firing, strikes whatever covers it, of any owner, the shooter's own units included. A hit is `Attacked` with `to` at the locked point; nothing there is the new `SimEvent::Missed`, and the damage is lost. An area TailDisjointed attack blasts the spot (the Artillery Rig's rule, tested with a test def only). The base stays held through Cooldown.
  - **Attack Ground** for the Gunner fires every 22 ticks until Stop or another order; a queued order never starts by itself. The client's tracer ends at the locked point, so a miss shows.
  - **"Target too close" for a location** reuses A's code: `engage::too_close` takes `TooNear` (an object or a location) and `BackOff`. `AttackGround` gains `backing_off` and `too_close`, hashed and saved only while set. `SimEvent::LocationTooClose` is shown to the owner only.
  - **The back-off fix (Q0054; the Director's call):** a back-off now keeps going while the target is within Range until its move ends, so the Gunner fires from the middle of the band (224 su, 3.5 GU), at unit targets and locations alike. A had stopped just past MinRange (128 su).
  - **Placeholder:** with two objects covering the spot, the one aimed at if it still covers it, else the lowest id. **Deviations:** structures count as at the spot (otherwise a Gunner could never damage a building); friendly yielding counts a unit aiming or firing as busy.
  - **Split (T28):** the attack-state hash moved to `combat/hash.rs`.
  - **Tests:** `tests/gunner.rs` (a standing target takes the hit; one that leaves the spot takes none; the Gunner's own Militant on the spot is struck; a structure is hit; Attack Ground repeats until Stop), `combat/phases/tests/tail_disjointed.rs`, `tests/cults_combat.rs` (3.5 GU for both targets; a cornered Gunner gets `LocationTooClose`), `orders/tests/min_range.rs`, and the client's miss tracer and feedback. 1919 tests.
  - **Goldens** unchanged; the pairings test is green. **Perf** (`projectiles_perf`, `behaviours_perf`) within noise by interleaved A/B.
  - **Open, filed as Q0126:** with today's numbers no ground unit can leave the locked point in the Gunner's 2-frame Firing (the closest is the Drone, 16 su of movement against an 18 su half-width), so the miss never happens in play.

- **M4-W3 The 90° contact step, 0323** (**Fable**; merged in `5305c2f3`): step 1 of 0324's plan for Q0111. **M4 is complete.**
  - **The step** (`movement/collide/contact.rs`, split out of `collide.rs` by T28). A blocked unit steps straight sideways along the blocker's face by its own width, over several ticks (`Mover::step_aside`, hashed and saved only while set).
    - Which bases take it is data, `UnitBaseDef::contact_step`: `Sideways` for both infantry bases, HoverVehicle, TrackedVehicle, the Mech and the DrillUnit (0338), and `Diagonal`, the old 45° step, for WheeledVehicle until M5-V2.
    - A step commits its side for 16 ticks ("switching sides after 1 s").
  - **The side at a standing blocker is the near side.** M3-G's rule for units with momentum now covers every base with the step, and right first decides only dead centre. Moving blockers keep the lean, right first head-on, and passing behind. Right first at standing blockers brought the hatch lock-up back (1.2 / 0.0 / 4.2 SC/s at 1 GU), so the deviation is filed as **Q0129**, proceeding on it.
  - **The hatch** (Q0111's 30 layouts):
    - layouts where the income stops: 9 → 0;
    - layouts with an Agent waiting 60 s or more without a load: 20 → 6. All six are oversaturated hatches, accepted by 0336: they assert income of at least 10.6 SC/s, and the others keep the 60 s assertion;
    - the soak with 4, 5 and 6 crystal Agents never stops (12.2 SC/s; `more_crystal_agents_never_stop_the_first_tunnel`, ignored, release);
    - W2's shared progress clock measured worse and was dropped.
  - **The 15 crowd tests that 0323 broke:**
    - **14 updated, each with a comment citing 0323:** converging groups settle by tick 83 (was 53); a line closes into a column on the click; the choke's bound goes from 160 to 192; corridor chases close up by tick 240; standing units drift up to 48 su; stacked Tunnels; Recruit crowds give 5.46 SC/s on a face (was 6.10).
    - **`deploy_gate` (0117) fixed** in `orders/build.rs`: a worker that begins a contact step on its footprint counts as stopped there, and deploys.
    - **Three real costs**, documented in the tests and in Q0129:
      - in 1 of 12 layouts, one of eight attacking Guards gives up;
      - Sabres converting one building lose their Attack order behind each other, and take it up again by automatic targeting (all fighting by tick 240, was 160);
      - 40 Guards out of one Tunnel jam its doorway after 31, until the crowd is moved.
  - **Goldens re-blessed:** attack_move_battle, syndicate_opening, colonists_vs_gdo, gdo_opening and syndicate_vs_gdo (`VERSUS_END` 2998 → 3060).
  - **Docs:** `docs/movement.md > 6.3`, and `docs/playtest/m4.md`, where the hatch's known issue becomes the accepted behaviour.
  - 1924 tests, 29 ignored.

- **M5-B Cults start** (Opus; merged in `98dfe65a`). **No faction is "not implemented" any more.**
  - **Totals (T63):** `StartingConditions::of` and `ArmyResource::of` are total. The Cults start is the Hall, the Storage and 6 Recruits at 0 SC, with the Hall as primary and the camera on it. `LayoutRefusal::NotImplemented` and every "not implemented" path are gone.
  - **Lineage:** `Scenario::start_objects` gives a layout's Cults units their first Hall as lineage, one entry per point of Unit Control cost. With no Hall they are orphans, which 0344 confirmed.
  - **Layouts** on `dev_skirmish` and every built-in Skirmish Default map: the Hall at (8,8), its 10×10 area wholly on the map, Recruitable and uncontested; the Storage at (5,4), 1 tile from 5 patches; the Recruits clear of the doors. Slot 2 is the Rotate180 image.
  - **Split (T28):** `validate.rs` was 503 lines, and `validate/start.rs` now holds the start checks.
  - **Tests:** `tests/cults_start.rs` (lineage; the 7th Recruit out at tick 192; LayoutMissing and LayoutExtra; 16 faction combinations), `crates/game/tests/factions.rs` (the HUD's "Space Crystals 0 / Unit Control 6/20" and the camera on the Hall, in both slots), `editor/cults.rs` (0012's Verification: the incomplete layout lists the missing Recruits), the CLI's `factions_start_the_cults_layout` and `the_usage_names_every_faction`, and both pairings tests run all 16 pairings.
  - **Goldens re-blessed by the Director, with the user's OK** (2026-11-25): gdo_opening, colonists_opening, colonists_vs_gdo, syndicate_opening and syndicate_vs_gdo. Each embeds `dev_skirmish`, and only the map's `slots` changed; inputs, setup and every checkpoint are identical. The permission system had denied the agent's own re-bless.
  - **Open:** replays recorded on Skirmish Default maps before M5-B no longer validate, as after M3-C and M4-B.
  - 1935 tests, 29 ignored.

- **M5-E Cults construction** (Opus; merged in `62dd3e7f`), with 0339 and 0340 folded in.
  - **Tickets (T59):** a group order is paid once, when given. It is refunded in full once its last holder drops it before the start, and so is each Shift-queued building (0340). X refunds 75%, and nothing comes back after the start.
  - **Assist** (`OrderKind::Assist`) targets only sites. A site starts at the first arrival within 8 su.
  - **Recruit-frames:** each builder on the order (Build or Assist) that is within reach and done walking adds one a frame. A site with none stalls, keeping its progress and HP.
  - **The wall (0339; `placement/wall.rs`).** Each Recruit claims a spot when given the order: the nearest part of the builders' wall (a box 4 su off the footprint, like F1's standoff), else the nearest free part along it, searched both ways in 4 su steps, clockwise first on a tie.
    - Waiters stand on rows 2 and beyond, keeping row 1 free as the way round.
    - Walks go round by the corners, never over the ghost.
    - Any unit on the footprint at the first arrival, the order's own Recruits included, fails the whole order, refunded once.
  - **Technical placeholders, marked in code:** the claim at the order; the tie-break and the 4 su step; the way round; re-spotting every `RESPOT_TICKS`. An order also fails when every holder's walk is over and none is in reach, which 0340's rule then refunds.
  - **Also built:**
    - the Armory needs a complete Storage;
    - Drones and Agents are unchanged, and `SeveralConstructors` stays;
    - a Recruit-built Hall claims, produces and gives lineage;
    - 2×3 and 2×4 ABAC test defs work at all four rotations;
    - the count and the spot claim are generic over (order, target), for N8's Repair.
  - **Tests:** `tests/cults_construction/` (21, every Verification of 0339 and 0340), `sim/spot_work/tests.rs` and `placement/wall/tests.rs`. 1967 tests, 29 ignored. Goldens unchanged.
  - **Perf:** `recruits_perf` and `structures_perf` within noise; 20 Recruits building a Hall cost about 0.01 ms a tick.
  - **Open:**
    - 24 Recruits coming from one side peak at 8 working on a Storage, which completes before the rest reach the far side;
    - the order's own Recruits on the ghost fail the order without warning (**Q0130**);
    - near the map's edge, a way round that leaves the map falls back to a straight walk, which can cross the ghost.

- **M5-T Build speed** (Opus; merged in `4ecfc18a`). No game change; the user asked about build time (2026-11-25).
  - **Test programs:** with `autotests = false`, the sim's integration tests are two programs, `sim` (`tests/sim.rs`) and `perf` (`tests/perf.rs`, the `*_perf` timings), and the client's are one, `game` (`tests/game.rs`). The files stay in place, as `#[path]` modules.
    - The guard, `crates/sim/tests/test_programs.rs` (3 tests, for both crates), names any test file that no program includes, with the line to add, and any file included twice.
    - 126 test programs became 8. On the same master, 1935 tests became 1938: the 3 new ones are the guard, and the names are otherwise identical.
  - **Debug info:** `debug = "line-tables-only"` for our crates and `debug = false` for dependencies. The client's test program and binary each shrink from 1.7 GB to under 0.25 GB. Backtraces keep file and line, though function names are bare. `split-debuginfo = "unpacked"` wasn't faster, so it isn't kept.
  - **Measured** (median of 3 interleaved rounds, 8 cores, 1-minute load in brackets):

    | After an edit | Before | After |
    |---|---|---|
    | sc_sim, then `cargo test --workspace --no-run` | 66.0 s (9) | 15.6 s (29) |
    | sc_sim, then one file's tests | 6.2 s | 5.5 s |
    | the client, then `cargo test --workspace --no-run` | 72.2 s (22) | 25.5 s (43) |
    | clippy | 6.3 s | 5.2 s |
    | the full test run, already built | 53.8 s (33) | 28.7 s (15) |

    The one-off rebuild after the profile change took 13.7 min, plus 4.3 for `-p sc_sim` and 2.5 for clippy. The main checkout's cache was seeded from M5-T's worktree.
  - **Perf A/B, adopted:** after each merge the Director builds master's release perf binary into `target/perf/<short sha>`, and agents run it as side A (`scratchpad/briefs/m5-common.md`).
  - `-p sc_sim` and `--workspace` build the sim with different features, so mixing them compiles it twice. This is documented in ARCHITECTURE.md.
  - 1970 tests, 29 ignored, in 6 programs.

- **M5-F2 Armory training** (Opus; merged in `d9492a69`). Design changes 0332, 0333, 0337, 0341 points 2–5, 0342, 0344, 0347 points 3 and 5, and 0348 points 3–5.
  - **The pool path, written once** over `holds` and `production` for every building that trains from its pool:
    - `QueueEntry` (`Unit` or `Release`);
    - `Production::reserved`, per entry;
    - `Held::leaving`, the units ready to walk out, in ready order through the one doorway;
    - `Object::augmentation`, two booleans with no effects yet.

    Each is hashed only while set. `QUEUE_LIMIT` no longer bounds a pool queue: the capacity does, reserved units included (0347 point 5).
  - **Training:** `Sim::train` changes the kind of the first reserved unit, which keeps its HP rule (infantry keep the damage, 0332), owner, lineage and augmentation. It appends the consumed units' lineages and drops the load (0347 point 3). The unit that entered first is trained first (0332).
  - **Kinds and Release (0337):** an order names a kind (Plain, Constitution, Metabolism or Both) and reserves the free unit of that kind that entered first. Release is a free 4-frame order. Eject All skips reserved units.
  - **Also new:**
    - `Reject::NoRecruit`;
    - `DebugCommand::SetAugmentation`;
    - the `ReleaseQueued` and `ReleaseCancelled` events;
    - invariant R1.

    `Lineages::of` now counts as orphaned the cost points no entry stands for.
  - **Evacuation:** every entry is refunded and the reserved units come out first, an order still marked `Placeholder (Q0106 point 2)`. **F3** replaces it with 0349.
  - **The Hall (0341, 0342, 0344):**
    - it exits from B, and has no X;
    - its Recruit in production counts;
    - capacity is rounded down per Hall;
    - tiles tied between Halls completed in the same tick go to the nearer centre, then the lower id.
  - **The Gunner (0333):** Firing 4 and Cooldown 6, with 0333's Verification as tests.
  - **Client:** `templates/cults/armory.rs` (a PoolProducer).
    - **DefaultState:** Q Plain, W Constitution, E Metabolism and R Both, each showing its free count and greyed at 0. A Eject All, X Cancel and C Rally.
    - **KindMenu:** Train Militant, the Gunner greyed "Requires a Gunworks", A Release and Z back. It stays open after a click.
    - X and C apply to the Tabbed Armory only. A right-click sets the rally of every selected producer.
    - The InfoPanel shows Inside, Free (per kind), Queue and Releasing.
  - **Perf** (interleaved A/B against `target/perf/4ecfc18a`, load 2.4): `holding_perf`, `structures_perf` and `recruits_perf` give the same games, with timings within noise.
  - 2010 tests passed, 29 ignored. The design-citation failure from the doc split (0370) is fixed on master in `36745275`.

- **M5-J1 Recruit UI** (Opus; merged in `2286859e`). Design changes 0322, 0327, 0341 point 1, 0342 (the ghost's rotation), 0347 point 4 (in the client), and 0362.
  - **The Recruit's panel:** Q Move, A Basic and W Advanced Construct, S Assist, D Repair (greyed until N8), X Stop.
    - The Construct menus are read from the sim's `CONSTRUCT_MENU`. Entries not in the game yet are `Planned` and greyed.
    - The InfoPanel shows the load a Recruit carries.
  - **`Builder::Recruits`:** one group Build, checked by `recruits_check` and paid once. Shift chains Builds, and each ticket leaves one lingering ghost.
  - **Pointers:**
    - Assist shows on a right-click and while S waits for a target. Its lime colour is a placeholder.
    - Repair has a pointer hook that nothing uses until N8.
    - Enter shows over an own Armory, never over a site; a right-click on a site Assists (0347 point 4).
  - **0341 point 1:** a Hall whose area leaves the map is refused with "The Recruitment Area reaches off the map", in the sim, on the ghost and in the editor's layout check. The game-start fixtures now place their Halls at least 3 tiles from the edge.
    - 0341's Verification says a Hall "3 tiles from the edge is refused". The doc's geometry accepts 3 tiles and refuses 2, which is how it's built and tested; the slip is noted in Q0131.
  - **0362:** the order's own Recruits on the footprint make the site wait; any other unit fails the order, refunded once. A Recruit whose walk ends on the footprint looks for a new spot, without which the site waited forever.
    - A Recruit that holds the order queued, behind another order, also makes the site wait. This is marked as a placeholder and asked as **Q0131**.
  - **Refactor:** the sim's right-click resolution is now a data table (`RULES`).
  - **QA:** an input script in which four Recruits build a Storage from the panel (`four_build_a_storage.json`). Screenshots in `scratchpad/m5j1/`: the panel, the menus, a Shift chain, the Assist and Enter pointers, and the refused Hall.
  - 2038 tests passed, 29 ignored. The goldens are unchanged.

- **M5-F3 Holding** (Opus; merged in `da08f46d`). Design changes 0347 points 1, 2 and 4, and 0349.
  - **0347:** F1 had already built points 1, 2 and 4 as placeholders, so the Q0104 marks in the sim and the client are now citations. New tests:
    - a unit gets in only through the doorway at the middle of the entrance side;
    - an Armory site refuses Enter at the gate, a right-click on it Assists, and once complete it takes the Recruit;
    - `defs/tests/holds.rs` checks that every holder's exit is opposite its entrance.
  - **0349:** `Sim::evacuate` (`sim/holding/evacuation.rs`) puts every unit inside on the former footprint in the tick of the destruction.
    - **Where:** on tile centres in row order from the north-west tile, wrapping round, each unit's box kept inside the footprint. Tiles are checked against the terrain alone (`nav::box_on_terrain`), since the nav grids are stale during removal.
    - **The queue:** a finished front entry is trained on the spot, with no refund. Every other entry is refunded in full, and its units come out as Recruits.
    - Evacuees idle and keep their facing. Invariant I1 gets a third exception, and I5 and I7 let an evacuee step apart in its evacuation tick.
    - **Removed,** as only evacuation used them: `Sim::rings_on_map`, `Spots::last_ring` and `Round::Clockwise`. Once Y lands, `placement::Spots`, `Sim::spot_around` and `Round` have no users and go at that merge, with ARCHITECTURE.md's Garrison note.
  - **Placeholders** (technical):
    - the spread order over the footprint;
    - a footprint with no tile a unit can stand on, where the unit leaves the game.
  - **Edge cases:**
    - a defeated owner's evacuees are neutral and never push apart (`can_yield` needs an owner);
    - a finished Release still waiting is dropped from the queue without `ReleaseCancelled`, and its unit comes out as it is.
  - **Perf** (A = `target/perf/dd29d89b`, load 1.96): `holding_perf` and `recruits_perf` are unchanged, and the evacuation tick drops from 0.22 ms to 0.14 ms.
  - 2041 tests passed, 29 ignored. The goldens are unchanged.

- **M5-Y Syndicate turrets** (Opus; merged in `c65cc1b0`). Design changes 0358, 0359 and 0360.
  - **0359:** an Agent builds only Tunnels (the turret's `site` is now None), and the Agent-built turret's rules and tests are gone.
    - The Agent's panel is Q Move, A Build Tunnel, S Drop Off, D Repair (`Planned::Repair`, greyed) and X Stop.
    - A turret is ordered only from a Tunnel's ExpandMenu.
  - **0360:** every turret starts lowered, and is active only while it has a parent Tunnel (`Sim::is_active_turret`). An inactive one is lowered at once and never fires.
    - When its Tunnel is lost, it turns neutral and visible to all, decays 3 HP/s, and is reclaimed by a covering Tunnel from Tier 1.
    - Turrets no longer keep a Syndicate player alive.
  - **0358:** points 2–4 are cited as built, and point 1 follows 0360. Its Verification is a test, adjusted so that the turret stays down once clear. The Q0091 marks are gone.
  - **F3's cleanup:** `placement::Spots`, `Sim::spot_around` and `Round` had no users left, and are deleted.
  - **Readings, marked in the code:**
    - "retracts at once" means lowered in the same tick;
    - an owned turret outside its owner's areas (from debug or a map) is inactive;
    - a neutral turret's fog record is drawn lowered;
    - spawn checks treat a turret as blocking, by its kind.
  - **Found:**
    - two doc lines still give the Syndicate Agent-built turrets: **Q0132**;
    - robustness fuzz seed 155 panics on invariant I2 for a Recruit, on master before Y: slice **Z1**.
  - **Perf** (A = `target/perf/dd29d89b`, quiet rounds at load about 2): `structures_perf` and `agents_perf` are within noise; `popup_perf` B ≤ A.
  - The ignored `syndicate_soak` passes: 6 pop-up rises, 21 shots, 2 neutral turrets decayed.
  - 2030 tests passed, 29 ignored. The tests of the removed rules went with them. The goldens are unchanged.

- **M5-J2 Recruitment Area display** (Opus; merged in `e548e6ca`). Design changes 0350, 0351 and 0352 (Q0107).
  - **The sim:** `recruitment::winners` is now the one claiming rule (0341's first-come order, ties by the nearer centre, then the lower id). `Claims::of`, the new `Sim::claimed_tiles` and the pure `recruitment::preview::Preview::of` all use it.
    - The preview takes an explicit list of claimants: complete Halls, then own pending Halls and ghosts in placement order (`PendingBuild::group`), then the Hall being placed.
    - It gives each tile Claimable, Taken (own, own pending, or other) or NotRecruitable, plus the Effectiveness rounded down.
    - The real claims are unchanged: the recruitment tests and the goldens pass untouched.
  - **The client:** the Tunnel display's mechanics are shared through `placement/area_overlay.rs` and `overlay::tint_tiles`/`outline`, with the Tunnel display's API and tests unchanged.
    - `placement/recruitment_areas.rs` draws the per-tile tints, "Effectiveness: N%" by the ghost, and a selected own Hall's claims. It never draws an enemy area.
    - `known_claimants` reads only what the player knows: own objects, enemy Halls it sees (`Sight::sees`) and remembers (`Sight::remembered`), and its own pending builds.
  - **Readings, marked Placeholder:**
    - other players' Halls rank as completed at tick 0, because the player doesn't know when they were completed. This decides only whose tile it is, never the Effectiveness;
    - own sites rank before own ghosts, because the sim keeps no placement time for a site. This can differ from 0351's placement order only when two groups build at once;
    - unexplored ground counts as Recruitable, as `Mode::Known` does;
    - without Shift, the placing Recruits' own ghosts are left out (their orders are replaced), as in `group_build_check`;
    - the label's wording and position.
  - **Found:** a selected own Hall's true claims (and its InfoPanel's Effectiveness, already on master) can reveal an enemy Hall the player has never seen: **Q0133**.
  - **Tests:** 7 sim unit tests (`recruitment/preview/tests.rs`: 0351's order, 0341's tie rule matching the real claims, rounding, off-map, Water against unexplored); 2 client unit tests; 4 headless `InjectedInput` tests (`tests/commands/recruitment_areas.rs`) for the three Verifications, a never-seen Hall, a Hall destroyed unseen, and the selected Hall.
  - **Perf:** `Claims::of` in the Economy step is no slower (`sixty_recruits_on_a_field_with_four_storages`, mean 0.024–0.025 ms against master's 0.026–0.038 ms at load 8.9).
  - 2043 tests passed, 29 ignored. The goldens are unchanged.

- **M5-Z1 Fuzz seed 155** (Opus; merged in `4d99074c`). A bug fix: in robustness fuzz seed 155, a Recruit broke invariant I2.
  - **The failure:** a Hall debug-spawned over Cliffs released a Recruit over the Cliff under its side B. When the Hall was removed, the Recruit stood on the Cliff.
  - **The root cause:** `Frame::box_clear` let a released unit overlap every tile of its building's footprint, including terrain it can't stand on. It came in with **M5-F2** (`a8a35bd0`, 0342: the Hall exits on B), found by bisecting the merges.
  - **The fix** (`production/exit.rs`, `exit/energy.rs`): `Frame::blocks` still lets the unit overlap the building's tiles, but not ground under them it can't stand on (`is_terrain_solid`, the test F3's evacuation uses). It covers side and energy doorways.
    - A doorway with no clear spot holds its finished unit, as an occupied one does.
    - I2 isn't weakened: the invariant checker applies the same rule to units in a doorway.
  - **Found:** the design allows Buildable ground that isn't Traversible (`core/map.md > Tile`: "Any combination of properties is valid"). A Hall whose side B lies over such ground holds its Recruits for good. Marked Placeholder: **Q0134**.
  - **Tests:**
    - a regression test through the gate (`recruitment::exits`);
    - two unit tests, one for side doorways and one for energy doorways;
    - all three fail without the fix.
  - **The seed sweep:** `robustness::seed_sweep`, ignored, fuzzes seeds 0–255 in about 6 s on the workspace build. `cargo test --workspace -- --ignored seed_sweep` is now in every slice's checks (`m5-common.md`). Before the fix only seed 155 failed; now none do.
    - `SEED`'s doc is updated: 0x5eed no longer ends a game, and 9 of the 256 seeds do.
  - **Perf:** `recruits_perf` and `holding_perf` are within noise of master (`e548e6ca`), at load 2.7–3.8.
  - 2046 tests passed, 30 ignored. The goldens are unchanged.

- **M5-L Cults oracle and goldens** (Opus; merged in `d729a418`). The Cults economy check, design changes 0345 and 0346 (Q0102, Q0103), on an 8-patch reference field (`testing::cults`, `economy_oracle/cults.rs`, `docs/economy_oracle.md`).
  - **Scenario 1, ideal mining:** 9 of 14 figures pass at ±15%. 5 are pinned out of tolerance (`OUT_OF_TOLERANCE`) and put to the Designer in **Q0135**:
    - 5 Recruits on a face reach 0.82 of full efficiency;
    - the gain per face from a 2-tile gap to 1 is 3.04 SC/s, against ~4;
    - a Storage's payback is 32.9 s, against ~25 s;
    - by 2:48 the Cults have 0.90 of the GDO's crystals and 0.88 of the Colonists', against "slightly ahead". By 1:30 they are level (0.97 and 0.92).
    - **Causes:**
      - a trip at a 1-tile gap is 24 ticks, because a Recruit stops at the first 4 su step within reach and walks 28 su each way instead of 24;
      - a face has two lanes.
    - **The rest pass:** the Hall fills from 6 to 20 in exactly 168.0 s, and the earliest second Hall is complete at 1:07.6.
  - **Scenario 2, the user's early-expansion line** (no target):
    - the second Hall is complete at 1:33.9, and the Storages at 1:41.5, 1:52.3 and 2:14.6;
    - **the Cults have 0.51 of the GDO's crystals by 1:30, and 0.62 by 2:48:** they don't stay level when they expand early (Q0135);
    - other dealing orders did worse.
  - **Found:**
    - with group construction, the earliest second Hall depends on how many build it: 1:07.6 with 4 builders, 2:13.5 with 1;
    - on 8 adjacent patches, an end patch is as near the next face's Storage as its own, and the lower id wins the tie, so faces share loads (14.4–15.9 SC/s against 20.5, from a throwaway measurement): **Q0136**. The reference field leaves a free column between patch pairs.
  - **0345:** D's `Placeholder (Q0102` marks now cite 0345, and its Verification is a test (`recruit_gathering/storages.rs`). There were no Q0103 marks.
  - **Goldens:** `cults_opening` (to tick 2640) and `cults_vs_syndicate` (a Cults win at tick 3529; the Syndicate's turret, ordered from the Tunnel's ExpandMenu, kills six Militants first). Both were blessed once, after the final merge, and play back in the client. The 11 existing goldens are unchanged.
  - **Also:** the pairing test's Cults side uses this opening, and every pairing on every map passes for 3 minutes.
  - 2053 tests passed, 31 ignored. The seed sweep passes.

- **M5-G2 Cults kinds and the Gunner** (Opus; merged in `9b7986e4`). G2 of the addendum, with what changed since; the last slice that adds a kind (T69).
  - **15 Cults kinds,** registered last in tech-tree order: the Gunworks, Garrison, Chop Shop, Machine Shop, Airstrip, Scrap Hangar, Altar, Temple, Cathedral and Shrine; then the Disciple, Seeker (the renamed Jackal, 0364), Sparrow, Hand of Heaven and Prayer-mobile (0361, 0363–0365).
    - Each has one module with its doc's numbers, `site` and `requires`.
    - A test checks the Cults' kinds against the design folder.
  - **Doc values, where the addendum's quotes differ** (the doc wins):
    - the Gunworks and Machine Shop are ABAC;
    - the Garrison is AAAB, with its door on B;
    - the Temple and Shrine are ABAC, and the Shrine isn't Groupable;
    - the Airstrip is a 2x4 runway;
    - the Sparrow's MaxCentripetalAcceleration is 0.5;
    - the Disciple has sight 6 and isn't Groupable.
  - **0348:** the Chop Shop, Airstrip, Cathedral and Altar are Groupable false. **0347 point 5:** the Chop Shop, Airstrip and Cathedral hold 10.
  - **Turret data** (`TurretAttrs`, in radians): the Seeker 360° at 12° a tick, the Sparrow 45° at 8°, the Hand of Heaven 20° at 4°. U1 reads it.
  - **The Gunworks, Machine Shop and Scrap Hangar** are built by Recruits from the Construct menus (the Machine Shop needs a Chop Shop, the Scrap Hangar an Airstrip), with no panel commands. A right-click on one is a Move.
  - **The Gunner:** the Armory's W trains it once a completed Gunworks exists, checked when queued. Losing the last Gunworks keeps the queued and training Gunners, and greys W (0335).
  - **The Storage's armour** is 1/4 (Q0094 a). The client's `WhyUnavailable::Placeholder` and the four `Planned` entries are gone.
  - **Perf, a small regression:** the vehicles' `NavClass::Ground` adds a `(Ground, 1)` grid to every game's grid set, because `paths::grid_keys` builds a grid for every registered kind.
    - `nav_perf`'s rebuild tick goes from 1.12 to 1.81 ms, and `structures_perf`'s mean tick rises about 3%.
    - Queued as **Z2**: build a class's grids only when it is in play.
  - **Not done:** the addendum's "G2 adds Gunners to `cults_vs_syndicate`", because G2's brief said no golden may move. It moved to N1a's first step.
  - **Readings, as placeholders:**
    - an editor-placed Chop Shop, Airstrip or Cathedral can be entered but has no panel yet;
    - the Airstrip's exit is on side C;
    - the labels are SKR and HOH.
  - 2077 tests passed, 31 ignored. No golden moved, the oracle's Cults figures are unchanged, and the seed sweep passes.

- **M5-U1 Unit turrets** (Opus; merged in `54b9c2fe`). U1 of the addendum, with T80.
  - **The sim:**
    - `AttackState::turret` holds the turret's facing and its LockedTarget. It is created on first use, and saved and hashed only while set.
    - In 5e the turret turns at its own rate within its arc before the attack phases, and holds still in Firing and Cooldown.
    - In 5a its behaviour runs after the base's: it keeps or drops its lock, and scans by the design's priority from the turret's facing (threat, then 45° rotation steps, then distance; MinRange skipped), during a Move too.
    - Attack, idle and AttackMove set the lock, Hold leaves it to the scan, and Stop clears it.
    - The scan and the end-of-sequence effects were split out of `orders/world.rs` first. `TurretAttrs` and the Laser Turret's code are unchanged.
  - **The client:** units with a turret carry the Laser Turret's barrel, interpolated by a client-side `TurretTrack`.
  - **Tests,** in-crate because the test defs are `#[cfg(test)]`: one per acceptance point (firing on the move, the arc, the 8-tick turn, the scan-order table with the design's example, MinRange, retention, a Move never stops, an Attack locks), plus Hold, Stop, hash and serde, and the real turrets' arcs. Each rule was broken once on purpose, and the tests caught it.
  - **Readings, marked Placeholder:**
    - a turret keeps a lock its base engages, and any other only while it can fire at it; otherwise it drops it and scans in the same tick;
    - it turns only toward a lock its owner sees;
    - an arc within 1° of 360° is a full turn.

    When a lock ends, and whether a Move clears a lock from an Attack, went to the Designer: **Q0137**.
  - **Perf:** `behaviours_perf` and `attack_move_perf` are within ±5% of master (`7088c06f`), at load 1.4–2.0. The new `turrets_perf` (96 turret units, 500 ticks, 2224 shots) runs 0.08–0.095 ms a tick.
  - 2096 tests passed, 32 ignored. All 13 goldens are unchanged, and the seed sweep passes.

- **M5-N8 Repair** (Opus; merged in `26501427`). N8 of the addendum, with T83 and the header's amendments (0321, 0322, 0325, 0326/0359, 0328, 0329): one repair mechanism for every repairer.
  - **The sim:** `crate::repair` decides who repairs what, and on what terms: half the target's build time, and half its price, paid gradually in whole crystals with no stored state.
    - `OrderKind::Repair` works from a spot beside a building (E's `spot_work.rs`), or follows a vehicle.
    - `Sim::step_repairs` runs after the sites. Repairers stack linearly.
    - A repair ends at full HP, or when the bank can't pay the next crystal (0329). Gliders, enemy targets, sites, full targets and, for Agents, the expansions are refused (`Reject::CantRepair`).
  - **The data:** `WorkerDef::repairs` for the Recruit (own buildings and vehicles) and the Agent (Tunnels free, Laser Turrets paid). The Field Engineer's is data for N9.
  - **Regeneration:** the Tunnel expansions regenerate 1% of MaxHP a second while their Tunnel lives, in combat too (`Sim::regenerate_expansions`). It shares `crate::regeneration` with the units held in the network.
    - The Laser Turret doesn't regenerate, since Agents repair it. The docs list it with the surface defensive structures, not the expansions, and 0359 keeps its repair.
  - **The client:** D is live on the Recruit's and the Agent's panels, with the Repair pointer and AwaitingTarget[Repair].
    - A right-click is Enter on what the unit can enter, and Repair on a damaged own building or vehicle otherwise (0325). The Repair row comes after Enter and Assist in the `RULES` table.
  - MaxHP is read only through `Object::max_hp()`, in three places, for N1a to adapt.
  - **Readings, marked Placeholder:**
    - the turret's terms come from its ExpansionDef, and a unit's from its first recipe;
    - a paying repairer pays crystals only;
    - free repairers count before paid ones;
    - a Repair ordered on an empty bank ends at its first crystal;
    - running out of crystals ends every paid Repair on the target;
    - a vehicle is followed within `PROXIMITY_SU`;
    - regeneration starts the tick after completion;
    - a Repair ends when its target starts transforming;
    - D is never greyed.
  - **Tests:** `tests/repair/` (rates, bank, Agents, refusals, regeneration), plus in-crate tests (a Seeker test recipe followed as it drives off, a glider refused by its movement) and the client's resolution and panel tests. The Hideout's "regenerates under fire" uses a Grenadier's Attack Ground, since an Attack on a hidden expansion is refused.
  - **Perf:** `structures_perf`'s whole tick is +1.3% (0.355 to 0.360 ms, at load 1.4), from two new walks over every object, for repairs and regeneration. The new `repairs_perf` (40 Recruits on 10 Armories) has a mean of 0.025 ms.
  - 2123 tests passed, 33 ignored. The goldens are unchanged, and the seed sweep passes.

- **M5-N2 The Garrison** (Opus; merged in `b706f6c0`). N2 of the addendum, with T72 as amended by 0335.
  - **Holding:**
    - the Garrison holds 4 of the Recruit, Militant, Gunner and Disciple, through one door on B, on F3's holding;
    - Eject All (`Sim::let_all_out`, `ExitRule::AtOnce`) brings everyone out at the door spot at once; they pass through each other until clear, then idle (invariant I1's exception 4);
    - destruction uses F3's `Sim::evacuate`, unchanged;
    - Recruits build it from D of Basic Construct.
  - **Fire** (`combat/occupants.rs`, `OccupantFire`, in 5e after the structures):
    - each occupant picks its own target as a structure does;
    - the Recruit and the Disciple fire the Militant's attack as it is, and the Militant and the Gunner get +1 GU;
    - Range and MinRange count from the footprint, and shots and directional armour use the footprint's edge point nearest the target;
    - no spells from inside, and leaving clears the unit's attack state.
  - **The client:**
    - the panel is A Eject All, with X while it is a site;
    - the InfoPanel shows "Inside" and an "Occupants" line (Placeholder);
    - a right-click with the four kinds is Enter, with the Enter pointer;
    - tracers start at the edge.
  - **Tests:** `tests/garrison/` (fire, entering, eject, protection), occupant unit tests and client tests, one per acceptance point.
  - **New golden `cults_garrison`** (14 goldens in all): 3 Militants and a Gunner inside against 4 Agents and 2 Grenadiers. A Grenadier's blast brings the Garrison down at tick 316, and the four come out unhurt and win. The other goldens are unchanged.
  - **Readings, two of them asked:**
    - a Garrison is never a "threat" to target scans, since it has no AttackAttributes of its own (**Q0139**);
    - a right-click goes by the type, as F3's Armory does, so a full damaged Garrison is Enter, not Repair (**Q0140**);
    - interface.md lists only the combat units for the Garrison's Enter; garrison.md and recruit.md let Recruits in too, which is built.
  - **Perf:** `structures_perf` 0.371 to 0.374 ms against `26501427`, combat 0.026 ms on both sides, at load 1.5–2.6. The new `garrisons_perf` (ten full Garrisons against ten waves of 30 Peacekeepers) has a mean of 0.064 ms a tick.
  - 2146 tests passed, 34 ignored, and the seed sweep passes.

- **M5-N1a Augmented stats** (Opus; merged in `698fbe78`). N1a of the addendum, with T73 as amended by 0330/0331, 0336, 0337 and 0338.
  - **Step 1:** Gunners in `cults_vs_syndicate` (G2's leftover), with a Gunworks and Gunners trained at the Armory's W through the gate. Only that golden was re-blessed.
  - **The accessors** take a `Research`, from `Sim::research()` or `Research::new(&players)`:
    - `Object::{max_hp_with, movement, movement_ref, max_speed, increments}` and `combat::{weapon, weapon_ref}`;
    - `Levels::of` is "No research" until N3 supplies the levels;
    - the data is `defs/cults/augmentation.rs` (`UnitDef::augmentable`, with `unarmed` for the Recruit).
    - ARCHITECTURE.md has a new section, *Augmented stats*.
  - **The HP rule** (`augmentation::follow_max`, applied by `augmentation::set`): when MaxHP rises, HP rises by the same amount; when it falls, HP is capped, with no floor.
    - `Sim::change_kind` keeps an augmentation for an augmentable kind and clears it otherwise.
    - `DebugCommand::SetAugmentation` works only with debug on.
  - **The guard:** `tests/augmentation/raw_reads.rs` fails on any new raw read of `.max_hp()`, `.movement` or `weapon_of(` outside `defs/` and `augmentation/`. Each allowed read gives its reason. `Object::max_hp()` stays the MaxHP before the Augmentations.
  - **Other slices' reads converted:**
    - N8's repair and regeneration heal up to `max_hp_with`;
    - N2's Militants and Gunners inside fire through `combat::weapon`, while a Recruit's or Disciple's borrowed attack stays the Militant's def, never augmented;
    - the client's HP bars and panel read the accessors.
  - **Perf:** the loops over every unit borrow a plain unit's numbers by reference, and the augmented path is a separate, non-inlined function. A/B against `463f2bda`, 10 interleaved rounds each way: `movement_perf` −0.1%, `recruits_perf` −0.1% (pinned, load about 2), instruction count +0.28%. A first version that copied the numbers was about 2% slower.
  - **Tests:** `tests/augmentation/{stats,hp,kinds,gate,raw_reads}.rs`, plus unit tests. They cover the addendum's table (Militant MaxHP 60, and 100 at L2; Recruit with both 50/4/7/4.5; the Metabolism-L2 Recruit 10/0/2/7), the armed units' parts (the Disciple counts as armed, which `disciple.md` itself still marks "To confirm with the user"), the kinds, the HP rules, the gate, and the borrowed attack.
  - **Readings, marked Placeholder:**
    - `faster` for movement models other than TurnRate (no augmentable kind uses one);
    - taking an Augmentation away through the debug command applies the same HP rule;
    - the repair's rate and price follow the augmented MaxHP, though no repairer repairs infantry.
  - 2169 tests passed, 34 ignored. Only `cults_vs_syndicate` moved (step 1), and the seed sweep passes.

- **M5-Z2 Lazy class grids** (Opus; merged in `31997e9e`). This fixes G2's perf regression: the `(Ground, 1)` grid was built in every game.
  - **Which grids exist:** a set, true or a player's, holds a grid only for a `(class, size)` that a unit in the game walks on, whether on the map, held, queued at a producer, or a Hall's Recruit.
    - `NavState::cover` builds a late grid from the same footprints as the other grids in its set, and never drops it. It bumps no version and no rebuild count.
    - It runs at 5b after the sync and at the end of the tick, and also on a debug spawn, on `change_kind`, at the start and on load. The code is in `paths/grids.rs`.
  - **Missing grids are loud:** every read goes through `paths::unit_grid` or `plan_grid`, which panic in debug builds on a missing grid, and I2 reports one. The gate checks a first-of-class debug spawn on a throwaway grid. A refused spawn builds nothing.
  - **New:** `Sim::true_grid(class, size)`, for tests that read a grid of a class with no unit in the game. nav_perf gains `knowledge_rebuilds_256_with_a_vehicle`.
  - **Tests** (`paths/grids/tests.rs`):
    - no `(Ground, 1)` grid, true or known, without a vehicle;
    - a debug-spawned or produced vehicle gets its grid from its tick or its queueing;
    - a late grid equals an eager one tile for tile, true and known, with the same hashes and paths;
    - saves before, at, during and after a vehicle load equal and play on the same;
    - the gate check works, and the debug panic fires.
  - **Perf,** against `698fbe78`, 8 interleaved rounds, at load 2.3–2.7:
    - nav_perf's rebuild tick goes from 1.64 to 1.02 ms, back to before G2;
    - with one vehicle it is 1.59 ms, master's cost;
    - `structures_perf`'s whole tick is 0.358 ms on both sides, with p99 from 0.71 to 0.68 ms and the grid sync's p99 from 0.26 to 0.15 ms.
  - **Accepted:** grids are never dropped, so a game whose vehicles all died still restamps their grid until a load, with no difference in play. A first-of-class debug spawn's check costs about 1.4 ms, for debug only.
  - 2176 tests passed, 35 ignored. The goldens are unchanged, and the seed sweep passes.

- **M5-N9 The Field Engineer and the Career Campus** (Opus; merged in `1cfb889c`). The 0328 header note: the GDO can repair.
  - **The sim:**
    - `CareerCampus` and `FieldEngineer` are the last two kinds in the registry, with 0328's placeholders;
    - the Deployment Center builds the Career Campus (150 SC, 320 f; needs a Barracks);
    - the Barracks trains the Field Engineer at E (75 SC, 160 f; needs a Career Campus).
  - **`repairs` moved from `WorkerDef` to `UnitDef`** (after `augmentable`, `#[serde(default)]`; the Director's call).
    - A `WorkerDef` gives the Worker capability, so a Field Engineer with one would have gathered crystals it can't deliver.
    - `repair::payment` was the only reader. The gate, the `RULES` Repair row and the client's pointer all go through it.
  - **The client:**
    - the Field Engineer's panel: Q Move, A Construct, D Repair, X Stop;
    - A opens its own `BuildMenu::Construct`, labelled "Constr" (the HUD's 6-character rule), with the hint "A: Construct". Its Q Bunker and W First Aid Tent are greyed;
    - a right-click on a damaged own building is Repair;
    - the Barracks' E is greyed "Requires a Career Campus" without one (`PanelTemplate::trains`), and W stays empty for the Rocketman (K);
    - the Deployment Center's D is the Career Campus.
  - **Tests:** `tests/field_engineer/` runs 0328's Verification end to end: one Field Engineer takes the Barracks from 150 to 300 HP in 40 ticks for 50 crystals, two in 20. It also covers the refusals, losing the Campus (E greys, queued ones still come out), Unit Control at production start, the 20 Power and the build area. Client tests cover the panel, the menu, the labels, the right-click, E and D.
  - **Notes:**
    - barracks.md's E line still says queuing needs "Unit Control", unlike Q, W and the GDO overview; it's built as the overview says (**Q0141**);
    - the Bunker and the First Aid Tent stay greyed, though 0366 gives them numbers (the user's decision on 0366–0369 is pending).
  - 2188 tests passed, 35 ignored. The goldens are unchanged, and the seed sweep passes.

- **M5-S1 The first answer sweep** (Opus; merged in `09940dab`):
  - **0335's rule needed no code change.** Every prerequisite is checked only when the work starts: Train when queued; Construct, Warp In and Expand when ordered; a worker's or a group's Build when ordered, which is when it is paid. Nothing checks again.
    - The new table `tech/tests/losing.rs` loses the only prerequisite in each case, finishes the work, and asserts no cancel, refund or fail: the Career Campus ReadyToPlace, a Warp In begun, a Laser Turret expanding, an Agent's paid Build still walking, a Gunworks site, a group or Shift-queued Build walking, and a complete Armory.
    - Cases tested elsewhere are counted, not duplicated (the Field Engineer, the Hideout's queue, the Gunner). The Altar and the Cathedral are left to N1b and N4a.
    - A worker's Build that is ordered but hasn't arrived, a Shift-queued Build, and a site waiting for its Recruits (0362) all count as started: "queued at the constructor" (`core/tech_and_research.md > Check Timing`), as the plan read it.
  - **0357:** a defeat notice, "<name> has been defeated" (Placeholder look), for every player still in the game, under F2 too (`hud/notices.rs`; tests with 3 and 4 players). The defeated player sees the results screen.
  - **0320:** the two ignored jam tests cite 0320 / Q0110 (d) as accepted behaviour; docs only.
  - **Q0108.1:** Enter's target reads "the structure being entered" in the code docs, with no behaviour change.
  - **Noted, no question:** a shot at an object hidden from the viewer is drawn over the viewer's Visible tiles up to the object (re-marked "Placeholder (not in the design)"). It can't matter before stealth is designed.
  - 2194 tests passed, 35 ignored. The goldens are unchanged, and the seed sweep passes.

- **M5-N1b The Altar** (Opus; merged in `8cd9020f`):
  - **The sim:**
    - `QueueEntry::Augment{order}` on F2's queue, with the recipes in `ProductionDef::augment`: one augmentation costs 25 SC and takes 160 frames; Both cost 50 SC and take 320.
    - New commands `ProductionCommand::Augment{kind, from, order}` and `ReleaseOf{kind, from}`.
    - The gate refuses AlreadyAugmented and CannotAugment, then the training refusals. The queue is bounded by the capacity of 10. The first unit in is reserved (0332).
    - The augmentation applies when the unit leaves, through `augmentation::set`; no level is stored (0336). X and destruction refund through F3's evacuation.
    - The shared pool code is general, for the Cathedral too (`Sim::complete`, `free_units_of`, `pool_counts_of`).
    - `sim/producers.rs` is split (T28), and so is the client's `commands.rs`.
  - **The client:**
    - `templates/cults/altar.rs`, the three-stage panel. Clicks stay in stage 3, and Z steps back.
    - Release is offered in every state; Fully augmented offers Release only (0337, Q0138).
    - The InfoPanel shows the free counts by type, the augmentation's progress, and the queue.
  - **The golden `cults_altar`:** EC, IM and Both Militants against 2 Agents. Its 100 SC come from a debug SetResources, since the Cults start with 0.
  - **Placeholders:**
    - QueueFull before NoRecruit at capacity;
    - a finished augmentation waiting at the doorway when the Altar falls comes out augmented, without a refund;
    - the labels, texts and InfoPanel wordings.
  - 2224 tests passed, 35 ignored. The other goldens are unchanged, and the seed sweep passes. No perf A/B: the producers' step is unchanged.

- **M5-M1 Soak invariants, pulled forward from M** (Opus; merged in `d0326c27`):
  - **H1** (`testing/invariants/hit_points.rs`): an object has HP exactly when it has a MaxHP, with `0 < HP <= max_hp_with(research)` and Shields within `[0, max]`, on the map and held.
    - A dead object exists only inside a tick: 5e leaves the killed at HP <= 0 until its own `remove_objects`, and the decay zeroes HP and removes the object in the same step. After `step`, every remaining object is alive.
  - **B1** (`banks.rs`): the banks are `u32` with checked debits, so "below 0" can't be stored. B1 catches a wrap instead, a change of 2^31 or more in one tick, unless a `SetResources` set it. It covers all five banks and leaves Power alone; debug builds panic on the underflow first.
  - **A1** (`augmentations.rs`): augmentations only on augmentable kinds, on the map and held. **R1 already covers** "reserved units are held in their building".
  - **Wider coverage:** the goldens now run the full `Invariants::check` after every tick. The fuzz games run `Invariants::check_rules` (P1, O1, R1, H1, B1, A1), since V1 and K1 would take the sweep from 5 s to 26 s.
  - Each invariant has a test that breaks it on purpose (`tests/invariants_state.rs`). Nothing fired anywhere, and no golden moved.
  - Perf: the seed sweep 4.9 → 5.2 s; the sim test program 15–16 → 16.6 s.
  - **Found:** a defeated player's units turn neutral and read "No research" amounts. Once N3's levels exist, their MaxHP can fall with nothing capping HP. N3 applies `follow_max` there (Q0142, built as (a)).
  - 2227 tests passed, 35 ignored. The seed sweep passes.

- **M5-N4a The Cathedral, the Disciple and energy** (Opus; merged in `aa220af2`):
  - **Energy (T75):** `Object::energy` in `energy::ONE` units, regenerating in `Phase::Energy` right after the Shields. It is hashed (tag 7) and saved only while set. The soak invariant E1 (energy within [0, max]) is in `Invariants::rules`.
  - **The Cathedral** is F2's pool path as data, on the Armory's template: Q costs 75 SC and takes 320 frames, and the first in is trained first. A Disciple leaves with 50 energy and reaches 200 in exactly 4800 ticks.
    - Recruits build it only while their player owns a Temple. Losing the Temple cancels no Disciple and no Cathedral already ordered (0335, tested).
    - Its queue is bounded by its capacity (0347 point 5), not by a `queue_limit` of 20.
  - **The Disciple:** the BasicCombatUnit panel with Attack greyed ("Not available for this unit") and R `Planned::Shroud`; a violet energy bar under the HP bar; the InfoPanel's "Energy N / 200", rounded down. Advanced C is live, greyed "Requires a Temple".
  - **Behaviour change, accepted:** the gate leaves unarmed units out of an explicit Attack (`combat::takes_attack_orders`: armed and not shielded), as 0202 already did for the Peacekeeper (Shielded). The gate, the right-click and the client's panel share it (T10). AttackMove still takes unarmed units along (0076). This changes the Field Engineer too.
  - **Placeholders:** held units regenerate; the bar's look; everyone who sees the Disciple sees its energy line; Patrol stays available to the Disciple.
  - **Noted for N5:** the addendum makes `energy.rs` read-only for N5, but has `energy::step` read its buff regeneration. N5's brief lets it add that hook.
  - Perf against 56225a93 (5 interleaved rounds, load about 1.1): `structures_perf` 0.361 → 0.361 ms; `movement_perf` 0.479 → 0.484 ms (+1%, the extra pass over 500 objects).
  - 2262 tests passed, 35 ignored. No golden moved, and the seed sweep passes.

- **M5-N3 The Temple and research** (Opus; merged in `63003d0c`):
  - **The sim:**
    - `QueueEntry::Research` on the queue engine, with the gate in T74's order: no duplicate, the previous level complete, the structure requirement at queue only, then the cost.
    - Progress is one `RATIO_ONE` per staffing Recruit; none without staff.
    - `Levels::of` reads the owner's techs. A completed level applies `follow_max` to every unit with that Augmentation, held ones included.
    - A/S/D/F are `HoldCommand::ReleaseOne`, a shared holding command for the tech buildings. The soak invariant T1 checks research.
  - **0379 (answering Q0142) arrived mid-slice and was built as (b),** in its own commit (`adcd806a`):
    - Defeat sets `Object::kept_research` on units with an Augmentation. `Research::levels_of` reads `owner.or(kept_research)`, so a neutral Enhanced Constitution Militant stays at 80/80. The field hashes under tag 8.
    - 0379 applies to every faction's research. Later research slices extend `kept_research` beyond Augmentations.
  - **A test checker fix:** invariant I3 allows the larger of the old and new MaxSpeed in the tick a level completes, since speed changes after the movement step.
  - **The client:** the Temple's panel (Q and W the next level, "InProg" while queued; A/S/D/F; V Eject All; X Cancel), the InfoPanel's staff and progress, and Construct X live.
  - **The golden `cults_temple`:** Enhanced Constitution L1 completes at tick 239 mid-fight, MaxHP 60→80 and HP +20. The other 15 goldens are unchanged.
  - **Placeholders:**
    - no rally point: released staff stop just outside side C;
    - with several Temples selected, A/S/D/F let one Recruit out of each;
    - a line's key is empty once both levels are done;
    - the labels and InfoPanel wordings.
  - `QueueFull` can't be tested: 4 technologies can't fill a queue of 5.
  - Perf (`movement_perf`, 7 rounds, load 7–9): 0.483–0.486 → 0.480–0.482 ms. 2295 tests passed, 35 ignored, and the seed sweep passes.

- **M5-S2b The second answer sweep, part 2** (Opus; merged in `49dc4b1a`):
  - **0376:** Stop already cleared a turret's lock (`orders::run::start`), so no behaviour changed. U1's marks now cite 0376. The new test Force-Attacks an own Drone, then Moves: the turret keeps firing; a Stop drops the lock, and the turret locks onto the enemy.
  - **0377:** `combat::Threats::of(objects, held)` makes one pass over the held units per snapshot (5a `World::with_occupants`, 5e `Targets::with_occupants`). A building threatens the domain its occupants fire at from inside, the Militant stand-in included; an empty one threatens nothing. Tested with a table per occupant, 0377's Verification (a Guard given an air-only weapon through TestDef), and a Laser Turret shooting a manned Garrison before a nearer Drone.
  - **0378:** `Aimed::full` (an own holding building in operation with no room, reserved units counted). `Rule::Enter` gives way to the Repair the unit would get there; the Tunnel Network is never full. The sim's right-click, rally points and the client's pointer share it.
  - **As built, no question:** units that can't repair (Militants, Gunners, Disciples) still get Enter on a full damaged building, and a full undamaged building is still Enter. Both end at the door, by the Full-building rule. Occupants mixing Ground and Air weapons would count as Universal; no building can hold that mix.
  - No golden moved: `cults_garrison`'s attackers only ever have the Garrison in reach. Perf (`garrisons_perf`, 5 rounds, load 3.4–3.7): 0.060–0.061 → 0.061–0.062 ms under the waves.
  - 2302 tests passed, 35 ignored. The seed sweep passes.

- **M5-N4b Shroud** (Opus; merged in `c0ca0956`):
  - **The sim:** `OrderKind::Shroud` holds the Disciple and drains its energy while regeneration continues; it is refused below 10. `Sim::clouds` is hashed under the Sim's tag 3, and saved only while a cloud exists.
    - Hits are absorbed and released in 5e. The clouds update at the end of 5a and again after the defeat check.
    - The per-target AoE share is one helper in `combat/area.rs`, shared by `strike` and the releases (bit-identical).
    - The soak invariant C1 checks clouds. The fuzz issues Shroud: the 256-seed sweep plays 137 channels and 15 releases.
  - **The client:** R is `PanelCommand::Shroud`, one Disciple per press (Tab to the next), greyed "Not enough energy" below 10. `shroud_view.rs` draws the clouds and a 6-tick release flash over the local player's Visible tiles only.
  - **The golden `cults_shroud`** (17 in all). The 3x3 building's ~32% share is a unit test, since no Disciple stands inside a footprint. The air-unit test is left to H1.
  - **Placeholders:**
    - an absorbed hit reveals its attacker; a release reveals nobody (Q0143);
    - a cloud ended by a death or a defeat releases in the next tick's 5e;
    - a projectile's origin is its launch point, judged when it lands;
    - a queued Shroud is checked again at its start;
    - Shift queues Shroud;
    - the cloud's look (Q0001).
  - **A Bevy 0.20 finding** (in `ARCHITECTURE.md`): a 2D material keeps the GPU texture it was prepared with. N4b suspected the fog layer has the same problem when the map size changes; Z3 checks it.
  - **Perf:**
    - `shroud_perf` with clouds: a whole tick averages 0.032–0.047 ms (p99 0.067–0.10).
    - A/B against 41e43d3d (load about 1): `structures_perf` 0.358 vs 0.360 ms, `movement_perf` 0.475 vs 0.472 ms. Without clouds the cost is within noise.
  - 2330 tests passed, 36 ignored (the air test for H1). No other golden moved, and the seed sweep passes.

- **M5-S2a The second answer sweep, part 1** (Opus; merged in `1801cf59`):
  - **0375:** a Recruit's drop-off is the own completed Storage nearest to the Recruit, edge to edge, when it finishes mining (`orders/gather/deliver.rs`). The Agent measures from itself. 0375's Verification is in `tests/recruit_gathering/storages.rs`.
  - **0372:** J1's mark is replaced, and 0372's Verification is a test. 0371's off-by-one is corrected where repeated.
  - **0373 and 0374:** J2's and Z1's marks now cite them (docs only).
  - **0384:** Agents re-pick their Tunnel at every return, Supplies included.
  - **0380:** the Cults Storage is 3x2 (ABAB, FullArmor 6). `maps/dev_skirmish.json` moves slot 2's Cults Storage to x 56 to keep the 180° symmetry.
  - **The oracle (0380–0383):**
    - **Its reference field** is now three faces of 3, 3 and 2 patches, with free columns between, seated 7/8/5.
    - **A test-harness fix:** the scripted Cults line keeps a Recruit's seat until it is idle. Without it, 40 Recruits jammed 20 seats. The fix changes `cults_opening`'s inputs (13 Gather orders instead of 19).
    - **The measurements:**
      - Every Scenario 1 per-face target passes on the 3x2: 6 on a long face and 4 on a short one, a gain of 4.54 SC/s per face, and a payback of 22.0 s.
      - By 2:48 ideal mining is 0.88 of the GDO and 0.86 of the Colonists, still out.
      - The realistic opening never overtakes ideal mining: both hit the 8-patch field's ceiling of about 21 SC/s by 2:48.
    - **The four-line table to 6:00** is pinned (`COMPARED`, `OVERTAKEN`), with bank + spent = start + mined asserted. It went to the Designer as **Q0144**.
  - **Goldens:**
    - `colonists_opening`, `colonists_vs_gdo`, `gdo_opening`, `syndicate_opening` and `syndicate_vs_gdo` differ only in their stored map: one field each, checked by the Director with a JSON diff.
    - `cults_opening` and `cults_garrison` diverge at tick 0.
    - `cults_vs_syndicate`'s defeat moves from tick 2853 to 2800.
    - `cults_temple`, `cults_altar` and `cults_shroud` are unchanged.
  - 2339 tests passed, 37 ignored. The seed sweep passes.

- **M5-Z3 The fog texture after a map size change** (Opus; merged in `29f2ee8e`; client only):
  - **The bug was real.** In Bevy 0.20 an image of a new size is a new GPU texture, but a 2D material only picks it up when the material itself changes. Before the fix:
    - every editor play-test started after an editor frame (every interactive F5) showed the whole map black, the 1x1 placeholder;
    - a play-test after Resize, New or Open to another size showed the previous play-test's fog, stretched and frozen;
    - F3 after `--no-fog` showed the map black.

    The start of a game was unaffected, which is why no QA screenshot showed it.
  - **The fix** (`fog/layer.rs`, as `shroud_view` does it): the texture is written in place while its size holds. A new size gets a new image, and the material (now in `FogAssets`) is pointed at it.
  - **Tests:** `tests/fog/texture.rs` covers editor play-test, Resize, play-test, and `--no-fog` then F3. Both fail without the fix. A guard in `tests/combat/shroud.rs` fails if `shroud_view` resizes under its handle.
  - **Same pattern elsewhere:** only the fog had it. The overlays, ghosts, shading, grid and effects replace untextured vertex-coloured meshes, which is safe. The note in ARCHITECTURE.md is updated.
  - **Found:** the editor play-test's banner, "PLAY-TEST: F5 returns to the editor", is drawn over the top bar's resource figures. Left for the next client slice that touches the top bar, M5-M (the playtest) at the latest.
  - 2341 tests passed, 37 ignored. The goldens are untouched, and the seed sweep passes.

- **M5-N5 The Shrine** (Opus; merged in `3197d711`):
  - **The sim:**
    - The Shrine holds 8 Recruits, in at B and out at C one at a time, and gains 0.25 energy/s per Recruit inside.
    - The gate on `Command::Cast` refuses LacksAbility, UnderConstruction, NotEnoughEnergy, OffMap and OutOfSight. `Sim::can_cast` and `Sim::sees_point` are new.
    - `Object::buffs` is hash tag 9, hashed and saved only while non-empty. A Buffs phase runs after Energy: the heal is 0.125 HP a tick, up to `max_hp_with`, and W adds to the Disciples' regeneration in the energy step.
    - Invariant F1, with its failing test.
  - **The client:**
    - Q and W cast; A/S/D/F release one Recruit of a kind (N3's `ReleaseOne`), and V is Eject All. It isn't groupable, so each press goes to one Shrine. A cast is greyed "Not enough energy".
    - While a cast waits for its point, a 4 GU ring follows the cursor, pink where seen and grey where not. A click out of vision says "Invalid target" and keeps targeting.
    - Buff marks sit beside the HP bars, and the InfoPanel shows "Participants: n / 8" and the energy.
  - **Tests:** 18 sim tests in `tests/shrine/`, and the client's in `interface/tests/shrine.rs`, `resolution/tests/shrine.rs` and `tests/commands/shrine.rs`.
  - **The golden `cults_shrine`** (18 in all): Q lands mid-fight on two wounded Militants, who win (without it they lose), and W boosts a Disciple. The client's playback count is now 18.
  - **Placeholders:**
    - the circle's edge counts as inside;
    - a buff works inside buildings and outlives its caster;
    - with several Shrines selected, the first in id order with the energy casts;
    - every Recruit held charges, those queued to leave too, since they are still inside;
    - Shift-click casts like a plain click;
    - the cast lands on the ground under the cursor;
    - the Shrine has no rally point or right-click;
    - the looks (Q0001).
- **M5-S4 A Recruitment Hall at the map edge** (Opus; merged in `cce50b54`; design change 0387):
  - **The sim:** `placement/recruitment_area.rs` and `WholeProblem::RecruitmentAreaOffMap` are gone, so a Hall takes only its footprint rules, in every placement mode. Off-map tiles count as non-Recruitable, out of 100: 70% flush against an edge, 49% in a corner. The Tunnel's on-map rule (0242) stays.
  - **The client:** the preview and the selected-Hall display already drew only on-map tiles. The Effectiveness label goes south of a Hall at the north edge (`label_ground`, a placeholder).
  - **Tests:** every edge and corner in all 4 placement modes; built Halls at an edge (capacity 14) and in a corner (capacity 9); the client's preview with the fog on.
  - **A Hall whose exit faces the map edge** keeps its finished Recruit. That follows `core/entities.md > Exit Doorway` point 5 (placement "does not keep exits clear"), with the edge blocking like terrain, so no question was filed.
- **M5-S3 The economy rerun and Shroud's reveal** (Opus; merged in `318234db`; design changes 0385 and 0386):
  - **0385:**
    - `testing::cults::line_field` is the two-sided 8-patch field, with three Storage sites a side.
    - The One Hall and Two Halls lines add Storages as needed.
    - The pins are `COMPARED` (the four lines to 6:00), `OVERTAKEN` (true, true) and `OUT_OF_TOLERANCE`. `economy_oracle/compared.rs` is new, and `docs/economy_oracle.md > The Cults` is rewritten.
    - The table went to the Designer in Q0146.
  - **0386:** its citation replaces N4b's placeholder. `tests/shroud/reveals.rs` tests the reveal by a direct shot and by an area hit, and that the release reveals nobody.
  - **The economy oracle is wrapped up** (user, 2026-11-26): "So much of it has to do with build orders and optimizations and that would be more effectively tested by me." There will be no more comparisons. Its tests become on-demand measurements, a task folded into M5-P3, and the pairings tests still check that every faction's workers gather.
- **M5-H1 Gliders and air** (Opus, the trial; merged in `e9ed2c37`):
  - **The sim:** glider movement (`movement/glider.rs`, `movement/air.rs`):
    - gliders spawn at IdleSpeed and idle on a 1 GU circle;
    - strafing flies on past the target and loops back at full speed, and Stop and Hold circle;
    - a glider may overshoot the map edge, and `Object::map_pos` places it at the nearest edge tile for sight and attacks;
    - air separation (`UnitDef::separation_radius`) is a soft, capped push;
    - HoverCraft are kept on the map by their half extents.
  - **Invariant G1:** the fuzz games give Sparrows orders and check G1 every tick.
  - **Tests:** `tests/air/`, covering flight, orders, strafing, Attack Move, the edge, separation and Shroud. An air unit under a Shroud cloud isn't protected.
  - **The client:** 3 flight lift levels and shadows, both placeholders (Q0001).
  - **Perf:**
    - `air_perf`, with 40 Sparrows: a whole tick takes 0.063 ms on average, 0.116 at p99.
    - `movement_perf` against 25edd170: about +2% in quiet rounds, with identical counters.
  - **One deviation:** six gliders sent to one point are 31.8 su apart at 10 s instead of at least 32, since the push is soft and capped. They are at least 31.999 su apart from 20 s.
  - **Placeholders:** clockwise circling, slowing for a point inside the turn, CIRCLE_PULL 0.5, SEPARATION_PUSH_SU 2, and no MinRange back-off for gliders.
  - **Sent to the Designer as Q0148:**
    - a lone Sparrow makes one pass, since its loop takes it out of sight and past Attack Move's leash;
    - a glider's Move never completes, so queued orders wait;
    - SeparationRadius 32 is under the 40 su Silhouette.
  - 2409 tests passed, 37 ignored. The goldens are unchanged.
- **M5-P3 The playtest map** (Opus; merged in `8eaf316e`):
  - **`maps/playtest.json`** keeps `dev_skirmish`'s terrain.
    - Each base's 8 patches form `line_field`'s two-sided line, at (3..10, 6) for slot 1. Rows 0–1 stay free behind the far Storage sites, and columns 0–2 round the west end.
    - Slot 2 is generated by the symmetry, and the editor reopens and saves the map unchanged.
  - **Slot 1's start layouts:**
    - **GDO:** the DC at (5,9), the EF at (12,5), and plates on (9,6) and (10,6).
    - **Syndicate:** the Tunnel at (5,10), R90 with flip_h, its hatch 3 GU from the line; the Hideout at (9,14).
    - **Cults:** the Storage on the near middle site (6,8), and the Hall at (5,12), R90, facing the line and 100% Recruitable.
    - **Colonists:** the Egress Node at (6,8), 1 tile from the line, and the Robo Portal at (8,10).
    - **The camera** starts at (7.5, 11) GU, with the line in view for every faction.
  - **Tests:**
    - `tests/playtest_map.rs` checks that slot 2 comes from slot 1, all 16 pairings, each field's 6 Storage sites, and that the layouts face the line.
    - The editor test now saves every map unchanged.
    - The 16 pairings gather over 3 min. The Cults gather 1298–1446, against 1078–1280 on `dev_skirmish`.
  - **The oracle is wrapped up** (0401):
    - Its 19 running tests are ignored, and its docs say it runs on demand with unmaintained pins.
    - Every non-economy rule it touched has other tests.
    - 2393 tests passed, 56 ignored.
- **M5-S6 Door arrows, rally points on unfinished buildings, and the top bar's notes** (Opus; merged in `5c0cc4e5`):
  - **Door arrows (0390, 0394):**
    - Ghosts and selected buildings show an arrow in at each entrance, out at each exit, and a two-way arrow at a shared door (the Garrison). The arrows turn and flip with the building.
    - Portals get an arrow out of each open side, and the Tunnel's In and Out arrows sit at side A. Both are placeholders.
    - Lingering ghosts and the editor's layout ghosts get none. A selected building shows its arrows whoever owns it.
  - **Rally points on unfinished buildings (0396):**
    - `production::takes_rally` is true for kinds with a ProductionDef and no ResearchDef: the Barracks, both portals, the Hideout, the Hall, the Armory, the Altar and the Cathedral.
    - An unfinished one holds a rally-only Production, and only while a rally is set, so the goldens are unchanged.
    - The gate's SetRally takes it. Completion keeps the point, and a cancelled Warp In drops it.
    - The unfinished panel gets C next to X, and a right-click sets the point.
    - The Temple, the Garrison, the Shrine and the Storage take none.
  - **The top bar (`hud::TopBarNote`):**
    - "Fog off (F3)" sits after the player's name, in games and replays.
    - In the editor's play-test, "Play-test (F5)" sits before the tick and replaces the overlay banner.
    - At 4:3 a long bar clips the end of a note.
  - **To the Designer:** Q0150, the Colonists doc's "no rally point" while warping in.
  - 2413 tests passed, 56 ignored. The goldens are unchanged.
- **M5-S5 Panels and Attack** (Opus; merged in `d6d1a1a2`):
  - **Attack (0397–0399, 0389):**
    - A is Attack whenever any selected unit can attack, whichever group is in focus.
    - It goes to every selected unit. Units that can't attack move to an explicit Attack's target, and an Attack Move takes everyone as one group.
    - The worker panels put their building on W:
      - Recruit: Q Move, W Basic, E Advanced, A Attack, S Assist, D Repair, X Stop;
      - Agent: W Build Tunnel;
      - Field Engineer: W Construct;
      - Drone: W Deploy.
  - **Place buttons (0388, and 0392's Extraction Facility half):**
    - The DC's E: Place shows while a building is under construction or ready to place: greyed with a progress bar, then lit.
    - The EF's Q: Place Plate works the same way.
    - The bar is a placeholder.
  - **Halls are groupable (0395).** A box still selects only one building (Q0151).
  - **Augmentations (0391)** take 48, 48 and 96 frames. The golden `cults_altar` was re-blessed: its Militants come out at ticks 147, 195 and 291, and none falls.
  - **Right-click (0400):**
    - In menus it does the DefaultState right-click, and the menu stays open.
    - In targeting and placement it goes straight to DefaultState.
    - Esc and Z still go back a step.
  - **To the Designer:** Q0151 asks about shielded Peacekeepers in an explicit Attack, box-selecting Halls, and A on a building's panel.
  - 2430 tests passed, 56 ignored. Only the golden cults_altar moved.
- **M5-H2 Glider follow-ups** (Opus; merged in `b825b923`):
  - **0403:** the Sparrow's SightRange is 8 GU. A lone Sparrow keeps strafing targets up to a Peacekeeper's size until they die. It loses bigger ones after 1–4 passes, since its loop grows with the target's size (Q0149).
  - **0404:** a glider's Move completes when it first reaches its point. The next queued order starts; with none, it circles there.
  - **0405:**
    - `UnitBaseDef::separation_strength` is 1.0 for HoverCraft, 0.25 for Glider (a placeholder citing 0405), and None for ground bases.
    - Each unit's push and the 2 su/tick cap scale by its base's share, so a Sparrow is pushed at most 0.5 su a tick. The invariants and the soak allow each unit its own share.
    - The Sparrow's SeparationRadius is 48. The Dragonfly's 48 and the Specter's 64 wait for their defs (0366, 0367).
  - **Measured:**
    - Six Sparrows sent to one point are within 10% of 48 su at 13.4 s and within 1% at 20 s, against 3.4 s and 9.8 s at full strength. That's slower than 0405's "a few seconds" (Q0152).
    - Two Sparrows on one target still make 7 passes in 60 s, as a lone one does, and every pass lands within 5.1 su of the centre.
  - **Perf:** air_perf's whole tick went from 0.0555 to 0.058 ms mean; movement_perf is unchanged.
  - 2438 tests passed, 56 ignored. No golden moved.
- **A1 Computer players: the framework and the GDO** (Opus; merged in `168dadc0`):
  - **The crate `sc_ai`** depends on the sim only. `Ai::think(&Sim)` runs once a tick before the step and returns ordinary `PlayerCommand`s. Each is checked at the gate first (`View::validate`), and the replay records them.
  - **`View`** keeps the `Sim` private. It shows the player's own objects and banks; other players' objects only while seen, or remembered as `LastKnown`; the map, with its patches known from the start; and the starts.
  - **Layers:** view, turn (the bank within a tick), build, army (staging, waves and defense, shared by every faction), and `factions/<f>.rs` behind `FactionAi`. A faction is added by setting its `HOOK`.
  - **Determinism:** no hash containers, no clock and no randomness; `crates/ai/clippy.toml` bans platform maths.
  - **The client:**
    - `--ai n[:standard|passive],...`; the command line refuses a faction with no computer player.
    - `ai_players.rs` runs `think` in FixedUpdate before StepSim. `--replay` and the editor never run it.
    - Input is dropped while the local player is a computer's (`CommandUiSystems::Input.run_if(local_player_is_human)`), and the top bar says "Player 2 (AI)".
  - **The GDO's play:**
    - Plate, Generator, Generator, Barracks; plates on every patch; Generators as Power needs.
    - A second Barracks at 6 plates, and Peacekeepers without pause.
    - Staging 12 GU toward the centre. Waves of 8, then 32, which go anyway after 2 minutes; a wave with nothing known searches the patches within 20 GU of an enemy start.
    - Defense when its buildings are hit.
    - Every number is in `crates/ai/src/tuning.rs`. Waves of 8 alone never beat a passive player on `dev_skirmish`, since they trade evenly with its whole army at its staging point.
  - **`ai_games`** (ignored, release), GDO against GDO; every invariant, the gate and 0402's fairness are checked each tick:
- **A2b The Syndicate's computer player** (Opus; merged in `73da4292`):
  - **What it plays:**
    - three Agents on its crystal hatch (0294);
    - one Agent runs Supplies and builds up to six Tunnels toward the staging point;
    - an Optics Lab, then Guards without pause, two queued, while Tunnel Space has room;
    - the Hideout's rally point follows the front of its queue: Agents go to their patch or the station, Guards to staging;
    - waves of 20, the first included, with 4 minutes' patience. Grenadiers were dropped, since mixing them in made the mirror slower.
  - **Shared-layer changes, all additive:**
    - `army::Waves`, with a default method `FactionAi::waves()` for per-faction wave numbers;
    - `View::network_counts` and `View::build_price`, read-only;
    - `ai_games` asserts standard-beats-passive only for same-faction pairs, and prints the cross-faction pairs as "Wanted".
  - **Results:**
    - The GDO-vs-GDO rows are unchanged.
    - In the Syndicate mirror, standard beats passive in 6:08–10:24.
    - The GDO's standard beats the Syndicate in 4:38–4:47.
    - **The Syndicate's standard can't beat the GDO's passive** (no winner at 20:00). It trains 96 units to the GDO's 249–253.
  - **The Director's finding:** the gap is in the AI's economy, not balance. Its new Tunnels go toward staging, and no Agents work their hatches, so income stays at one hatch's 12.5 SC/s. The design says the Syndicate "grows by adding Tunnels ... rather than by saturating one field" (`tunnel_area_and_space.md > Crystal intake and stacked Tunnels`). The fix is A2d's part 2.
  - **Perf:** `think` averages 15 µs over 624,744 calls. Its worst, 0.8–3 ms, depends on the machine's load.
  - 2455 tests passed, 57 ignored. No golden moved, and the sim is untouched.
- **A2c The Colonists' computer player** (Opus; merged in `157d6c28`):
  - **What it plays:**
    - Drones up to three per patch of its field;
    - Drones from the Robo Portal deploy into Beacons when Beacon Capacity, counting Beacons on their way, falls short of the queued units plus one of each portal's next. Every deploy is checked with `View::deploy_check`. Sites keep a free ring and stay off the field and the staging point. A Deploy stuck for 20 s is stopped, and that Drone and site are skipped for a minute;
    - warp-ins, checked with `View::warp_in_check`: a Bio Portal on the first Beacon, two more at 12 and 18 Drones, and the Egress Node or Robo Portal again if lost;
    - Sabres without pause, with Weapon Conversion turning every other one into a Rifle, since Sabres alone stalemated the mirror;
    - the army layer's default waves, 8 and then 32.
  - **Shared-layer change:** `build::Occupied`, `Occupied::of` and `Occupied::free` became `pub(crate)`. No sim or View change.
  - **Results:**
    - The GDO-vs-GDO rows are unchanged, and the Syndicate rows are as A2b left them.
    - In the Colonists mirror, standard beats passive in 6:38–12:21.
    - The GDO's standard beats the Colonists in 4:44–5:10.
    - The Colonists' standard beats the Syndicate's passive from both slots, in 6:04–8:59.
    - **The Colonists' standard can't beat the GDO's passive** (no winner at 20:00, 137 units trained to 253). A2c tried Rifle shares, a second Robo Portal and larger waves, and none changed it.
  - **The Director's call:** with one Robo Portal, Beacon Capacity sits at its cap from about 3:00, with 882 SC banked at 3:00 on `playtest.json`. A computer player that sits on its money is a weak opponent, so A2d's part 3 makes the Colonists spend it. Two Robo Portals did keep capacity ahead in A2c's trials, but the mirror then stalled against passive, so A2d must also find what breaks a passive Colonists defender.
  - **Perf:** `think` averages 8–44 µs on the Colonists' rows.
  - 2457 tests passed, 57 ignored. No golden moved, and the sim is untouched.
- **A2a The Cults' computer player** (Opus; merged in `946d6dff`):
  - **What it plays:**
    - Recruits fill the field's seats: 2 per patch for each Storage, across a one-tile gap. A new Storage goes up once all seats are taken;
    - spare Recruits enter the Armory, which trains Militants, then only Gunners once a Gunworks stands (built after 2 Halls);
    - Recruitment Halls go on open Recruitable land within 20 GU of the start, where each claims at least 90% of its area: 4 fit on `dev_skirmish`, 5 on `playtest.json`;
    - one Build at a time, by the nearest Recruits;
    - the default waves, 8 and then 32.
  - **Shared-layer changes:**
    - after searching the patches, a wave searches a grid of points 8 GU apart within 20 GU of an enemy's start (`army::search_points`), since Cults Halls can stand far from any patch. It applies only in games with a Cults player, to keep the other rows as they were;
    - `View::faction`;
    - the harness's `Aim::Around`.
  - **A sim fix, found by the AI games** (`7285eec0`):
    - **The bug:** a unit walking out of a doorway whose building is destroyed was left moving with no goal, which tripped invariant I7.
    - **The Director's reading,** since `core/entities.md > Exit Doorway` doesn't cover it: with no goal it stands where it is; toward a rally point it goes on there.
    - **Tests:** `crates/sim/tests/production/exit.rs`, where Peacekeepers destroy a Barracks mid-exit. The first test fails without the fix.
  - **Results:**
    - The 81 non-Cults rows are unchanged, the GDO rows included.
    - In the Cults mirror, standard beats passive in 6:59–13:54.
    - The Cults' standard beats every other faction's passive: the GDO's in 12:58–14:16, the Syndicate's in 6:18–8:21 and the Colonists' in 9:45–14:21.
    - The GDO's standard beats the Cults in 4:21–5:56.
  - **The Director's finding:** the Cults bank 4,000–6,000 SC by 7–9 minutes, while 40+ mining Recruits use the Unit Control their army needs (95/95 at the end of a mirror). That is A2d's part 4.
  - **Perf:** `think` averages 8–16 µs on the Cults rows.
  - 2461 tests passed, 57 ignored. No golden moved.
- **M5-V1 Oriented silhouettes** (Opus; merged in `011fdc79`):
  - **The geometry** (`shape.rs`):
    - a non-square Silhouette turns with its facing (`ObjectDef::silhouette_turns`);
    - overlap is a strict separating-axis test, the gap the least corner-to-shape distance, and collision a separating-axis sweep;
    - tile work uses the bounding box, with exact shape-against-tile tests where the box isn't clear;
    - two boxes keep the old arithmetic.
    - The module docs' promise that "oriented boxes change this module only" holds for range and proximity, but not for collision. `movement/space.rs`, invariants I1 and I2, the debug teleport, placement, pop-ups and release changed too, and picking uses the turned box (`selection/pick.rs`).
  - **A blocked turn** departs from T78, and the Director accepted it:
    - the turn is tried before the move, in parts along which the corners move at most 4 su (`TURN_TRY_SU`);
    - the first blocked part is halved 5 times, and the free part is kept;
    - a cut turn zeroes the speed, asks its blocker to yield, runs the stuck clock and creeps on (0035).
    - Why: halving a whole step tests only where it ends, and a long unit's corner can pass through another's corner in a 20° turn.
  - **Tests:**
    - shapes touching, corner to edge, edge to edge, and against tiles;
    - a wall cuts a turn at 8.59°, and the unit creeps off and arrives;
    - a 48 su gap: the unit passes when pointed through it, and its turn across it is cut at 11.3°;
    - range measured to a Seeker's long side and to its short side;
    - a new oriented soak (72 units, a third of them Seekers): I1 and I2 every tick, every move ends, deterministic.
  - **Perf:**
    - `movement_perf` A/B: 1.15 ms on master against 1.08 ms, within noise;
    - the new `vehicles_perf` (15 Seekers and 15 Hands of Heaven driven through each other): 0.038 ms a tick on average, 0.12–0.13 ms p99.
  - **For V2:** turned shapes jam. In an 8-seed soak, Seekers had 105 failed and 403 arrived moves, against 19 and 548 with turning off. V2's brief measures the same soak.
  - 2479 tests passed, 58 ignored (V1 merged after A2a and tested with it). No golden moved, and the computer players' games are unchanged.
- **M5-N7 The Airstrip and the Sparrow** (Opus; merged in `ce77e205`; with 0408, 0410, 0411's test and 0412 folded in):
  - **The Airstrip** trains Sparrows from pooled Recruits, as the Armory trains. Its panel is the pool building's, with Q: Build Sparrow. It is in the Advanced Construct menu (E, D), greyed with "Requires a Chop Shop" until N6 builds the Chop Shop.
  - **Takeoff** (`ExitRule::Runway`, `sim/holding/takeoff.rs`):
    - the Sparrow takes off the moment its training ends, never waiting for a doorway;
    - it appears over the B end facing C at IdleSpeed and flies straight past C (`Mover::takeoff`, hashed only while set);
    - it then takes the rally point's order, or circles 2 GU past C with none.
    - Ejected and released Recruits leave by the doorway at C.
    - A Sparrow on its run when the Airstrip is destroyed flies on. The test destroys it 1 and 12 ticks into the 28-tick run, for A2a's doorway fix.
  - **Placeholders:**
    - an order given during the run takes effect at its end, and Stop doesn't end the run (Q0170);
    - the runway's look;
    - door arrows in at B and out at C.
  - **0408:** a lone Sparrow with no spotter strafes until its target dies, every loop 488–495 su out. A Seeker took 10 passes, a Guard 7, a Barracks 20 and a Peacekeeper 5. `air::attack_move`'s `let_go` went from 2 to 1.
  - **0412:** gliders push apart at a third of the HoverCraft strength, up from a quarter.
    - Six Sparrows sent to one point come within 10 % of 48 su apart in 11.4 s, against 13.4 s before.
    - Two strafing side by side land 7 hits a pass, each pass within 4.2 su of the target's centre.
  - **0410 and 0411:** Shift-, Ctrl-, double- and Ctrl+Shift-click work on any own object, and "of that type" follows the SelectionGroup type. M5-S5's Halls test now boxes one Hall, double-clicks both, and sets a rally point both use.
  - **The golden:** new `cults_air`, three Sparrows against a Militant and a Guard. The Militant is spawned for the Syndicate player as a fixture. No other golden moved.
  - **Perf:** `air_perf` A/B against master's binary: a median of 0.049 ms a tick, against master's 0.050. No regression.
  - **Docs:** `docs/playtest/m5.md` (new): the Airstrip, the Sparrow and the Colonists' anti-air gap (0356).
  - 2505 tests passed, 58 ignored.
- **M5-MM The Minimap** (Opus; merged in `b5673277`; 0415, which the user asked to be prioritised). Client only.
  - **Drawing** (`hud/minimap.rs`, `hud/minimap/{frame,raster,input}.rs`):
    - a CPU-side image, one texel per physical pixel;
    - `MinimapFrame` letterboxes the map, with one tested mapping each way (`ground_at`, `screen_at`);
    - the terrain is baked once per map and size, from the terrain mesh's unshaded tile colours.
    - The fog and objects come only from `Sight`, with `Sight::tile_states` now shared with the fog layer. Memory records are dimmed, and live objects pass `Sight::sees`.
    - The layers repaint only when what they show changes, at most 15 times a second, and at once while the sim stands still, so screenshots are deterministic.
    - The camera's outline is a UI border, placed every frame.
  - **Perf** (release, `playtest.json` with 305 objects, fog on): a redraw takes 0.039 ms at 1920×1080 and 0.092 ms at 3840×2160. That is 0.010 and 0.023 ms a frame at 60 fps, against a 0.5 ms budget.
  - **Clicks** (`MinimapPointer`, filled before `CommandUiSystems::Input`):
    - a left press or drag centres the camera (`follow_minimap_press`);
    - a right press goes through one shared `right_click` function on `MapTarget::on_ground`, the same path as the map's;
    - in AwaitingTarget, `click_targets` claims the press and resolves it through the map's `PressedTarget` path;
    - in AwaitingPlacement, a left click only centres the camera.
    - No box selection, deselection or map click ever starts on the panel.
  - **Placeholders (MM's readings):**
    - only the map area takes presses, not the letterbox bars;
    - the outline is the view's extent around the ground under its centre, so it isn't exact on hills;
    - fog edges are hard, per tile;
    - unit dots are max(2, the longer side ÷ 80) texels;
    - the pointer stays the plain arrow over the minimap, even in AwaitingTarget.
  - **Left out:** attack alerts, undecided in 0415.
  - **For S7:** the panel test in `tests/commands/minimap.rs` checks that the far corner is Unexplored, which 0416's default changes.
  - **Tests:** 10 unit tests (frames, mappings, outline, throttle, and the fog layers against real sims) and 7 harness tests (centring, dragging, right-click and Attack Move equivalence with Shift, no selection change, placement, redraw).
  - 2522 tests passed, 59 ignored. No golden moved, and the computer players' games are unchanged.

    | Map | Standard as P1 vs passive | Passive vs standard as P2 | Standard vs standard |
    |---|---|---|---|
    | dev_skirmish | P1 wins 10:36 | P2 wins 11:25 | P2 wins 7:16 |
    | playtest.json | P1 wins 7:41 | P2 wins 12:28 | P1 wins 5:55 |

  - **Perf:** `think` takes 10.4 µs on average, 112 µs at worst, over 162,688 calls.
  - **The Director's reading:** a wave sent to search the map's patches is fair, since the map's patches are known from the start (`systems/controls/behaviors.md > PathKnowledge`).
  - 2453 tests passed, 57 ignored. No golden moved, and the sim is untouched.

    Whether enemies see buff marks went out as **Q0145**.
  - **Perf** (`movement_perf`, 6 pairs, load 8.7–10.3): 0.610 against 0.623 ms a tick, within the noise.
  - 2371 tests passed, 37 ignored. The 17 other goldens are unchanged, and the seed sweep passes.

## In progress
- **Running:**
  - **M5-S7 Full screen, the playtest menu and the explored start** (Opus; brief `scratchpad/briefs/m5s7-window-and-playtest-menu.md`; 0413, 0414 and 0416), launched in MM's slot.
  - **M5-V2 Wheeled movement and Reverse** (Opus; brief `scratchpad/briefs/m5v2-wheeled-movement.md`, with 0353's rules), launched in V1's slot. It heads M5's critical path.
  - **M5-K The GDO's anti-air: the Arsenal and the Rocketman** (Opus; brief `scratchpad/briefs/m5k-rocketman-arsenal.md`, with HeadDisjointed, Target Lock and 0407's Verification test), launched in N7's slot.
- **M6's plan is adopted** as `artifacts/director/m6-plan.md`, with Q0153–Q0169 filed (Queued > M6). M6-P found that no M5 slice builds tracked movement or crushing, which N10's Hand of Heaven needs, so **M5-V3** (tracked movement, after V2) and **M5-V4** (crushing, after V3) join M5. N10 now comes after N6 and V3.
- **Next, as slots free:** A2d (brief `scratchpad/briefs/a2d-rush-and-syndicate-tunnels.md`); V3 and N6 (after V2) → N10 and V4, then N11; then X1 → X2 → M.
- **Queued for the playtest:**
  - **Computer opponents.** The user asked for them for the next playtest (2026-11-26). 0402 answered Q0147: a playtest tool with fair knowledge (`core/game_rules.md > Computer players (playtest tool)`), built as planned. **A1 is merged** (`168dadc0`). It launched on 2026-11-26, taking the user's answer to Q0147 as the go-ahead: (a), "built as above", which 0402 records as "Build as planned". The user was told and can stop it. **A2a–c are running** (brief `scratchpad/briefs/a2-ai-factions.md`, launched 2026-11-26).
    - **Architecture** (the Director's):
      - A new crate, `crates/ai` (`sc_ai`), depends only on `sc_sim`. It has no Bevy and is deterministic like the sim, so an AI-vs-AI game replays from its seed.
      - Each computer player is an `Ai` whose `think(&Sim) -> Vec<PlayerCommand>` runs once a tick, like the pairings tests' `Opening::play`.
      - It reads the game only through a `View`: its own objects; the enemies its player sees or remembers (`Vision::sees`, `remember`); the map; and the start locations. Module privacy enforces this.
      - Its commands go through the gate into the same `TickInputs` as a player's (`sim_driver.rs`'s `PendingCommands`), so replays record them. `--replay` never runs it.
      - The client takes `--ai <player>[:<profile>]`, for example `--factions cults,gdo --ai 2`, or `--ai 1,2` to watch two play. F2 still switches the view, but an AI player's units take no keyboard commands.
      - QA:
        - `--headless --fast --ai 1,2`;
        - `crates/ai/tests/`, which plays every pairing AI against AI on every map, to a defeat or a time limit. It checks the invariants and the gate every tick and reports who won and when.
    - **Slices:**
      - **A1 The framework and the GDO** (Opus): all of the above, and the GDO's standard and passive profiles: the `gdo_opening` line, Barracks, and Peacekeepers in waves.
      - **A2a–c The Cults, the Syndicate, the Colonists** (Opus, in parallel once A1 lands): each faction's opening, made robust from the pairings openings, its production, and the shared army layer. Each keeps the GDO's games unchanged and reports its pairings against the GDO. The rush profile moved to A2d, so the three stay disjoint. **Done:** A2b (`73da4292`), A2c (`157d6c28`) and A2a (`946d6dff`).
      - **A2d The rush profile for all four factions, the Syndicate grows by Tunnels, and the Colonists spend their bank** (Opus, after A2a–c; brief `scratchpad/briefs/a2d-rush-and-syndicate-tunnels.md`).
      - **A3, later, from playtests:** defense, expansions, tech, spells and difficulty.
- **Next**:
  - **M4 is complete** (M4-M, `4e3fdd4a`; the strays, M4-Z, `95b4bc6e`; the hatch jam fixed by M4-W3's contact step, `5305c2f3`). The jam's history: M4-W diagnosed it (`189d0162`), M4-W2 built 0315's first step (`b81b2089`), which didn't fix it, and 0324 planned the 90° contact step, which did. 0336 accepted the oversaturated hatch and dropped 0324's steps 2 and 3.
  - **M5 wave 1 is done:** A (`ed23874e`) and R (`afd9c152`).
  - **M5 wave 2 is done:** D (`36a06de4`), C (`afd9b366`) and F1 (`e0444eab`), and O3, pulled forward from W4 (`e64cf8d2`).
  - **0319** put every designed Cults building and unit in M5, with placeholder numbers. The gated slices N1–N6 and G are now in scope. M5-P2's addendum plans them: 31 slices in all, with the playtest at the end of W12, or a first playtest of the Cults' infantry core at the end of W9 (the choice is with the user, 2026-11-25). The addendum is adopted as `artifacts/director/m5-plan-addendum.md`, with T69–T83, and its 14 question drafts are filed as Q0112–Q0125. Wave 3: B (`98dfe65a`) and E (`62dd3e7f`) have landed. **M5-T, build speed**, has landed (`4ecfc18a`). **F2, Armory training,** has landed (`d9492a69`). **J1, Recruit UI,** has landed (`2286859e`). **F3, holding,** has landed (`da08f46d`). **M5-Y, Syndicate turrets,** has landed (`c65cc1b0`). **J2, the Recruitment Area display,** has landed (`e548e6ca`). **Z1, a fuzz bug,** has landed (`4d99074c`). **L, the Cults oracle and goldens,** has landed (`d729a418`), and **G2, the remaining Cults kinds and the Gunner** (`9b7986e4`). **U1, unit turrets,** has landed (`54b9c2fe`), **N8, Repair** (`26501427`), **N2, the Garrison** (`b706f6c0`), **N1a, augmented stats** (`698fbe78`), **Z2, lazy nav grids** (`31997e9e`), **N9, the Field Engineer and the Career Campus** (`1cfb889c`), **S1, the first answer sweep** (`09940dab`), **N1b, the Altar** (`8cd9020f`), **M1, soak invariants** (`d0326c27`), **N4a, the Cathedral, the Disciple and energy** (`aa220af2`), and **N3, the Temple and research** (`63003d0c`). **H1**, gliders and air, is merged (`e9ed2c37`). Its Opus trial passed, so V1 and V2 run on Opus too. **S3**, the economy rerun (0385) and Shroud's reveal test (0386), is merged (`318234db`), and so is **S4**, a Hall at the map edge (0387, `cce50b54`). **N5**, the Shrine, is merged (`3197d711`). The user playtested what is built, hot-seat, on 2026-11-26 ("things seem reasonable"), and wants computer players for the next playtest (A1, running). **Z3**, the fog texture after a map size change, is merged (`29f2ee8e`). Merged since: **S2b**, answers 0376–0378 (`49dc4b1a`); **N4b, Shroud** (`c0ca0956`); and **S2a**, answers 0372–0375 and 0380–0384 (`1801cf59`), whose economy comparison is **Q0144**. **The other full-path slices wait on movement slices:** N6 on V2, N10 on V1, and N11 on N6 and K. N7 and K waited on H1, which has landed. **The user approved N7 and K** (2026-11-26: "N7 and K yes"). They start as the A2 slots free up. Then the user said: "I might not be able to playtest for the next 24 hours so just continue with the milestones work" (2026-11-26). The Director reads it as: build M5's remaining slices (V1, V2, N6, N10, N11, X1, X2, M) in dependency order, then M6 = 0366–0369. The user was told and can redirect. **Next:** A2a–c, then N7 ∥ K ∥ A2d (rush). The infantry-core playtest needs only X1, X2 and M. N3, N4b and N5 belong to the full one, which also needs V1, V2, H1 and the rest. **New since:** **M5-K, GDO anti-air** (0356: the Rocketman and the Arsenal), after H1. **The Field Engineer is in M5 now** (0328): slice N9, after N8 and before X1.
  - **Playtest:** the user plays once all four factions are built, in hot-seat from the command line (2026-11-25). The first playtest ran on 2026-11-26; the next adds computer players. The kit is `docs/playtest/` (m2–m4 and M5's README). This replaces separate M1–M4 playtests.

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
2. M4 Syndicate (done; `artifacts/director/m4-plan.md`).
3. M5 Cults (in progress; `artifacts/director/m5-plan.md`), ending at the user's first playtest.

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
- **M4 Syndicate** (**done**: M4-M at `4e3fdd4a`, with follow-ups M4-W and M4-Z; spec: `artifacts/director/m4-plan.md`, revision 2, from the design at `70d1bbc` (0282)).
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
    - Answered since revision 2: every M4 question, Q0076 to Q0088. Q0076 by 0294 and 0295; Q0077 by 0288; Q0078 by 0283; Q0079 by 0289; Q0080 by 0290; Q0081 by 0291; Q0082 by 0292; Q0083 by 0293; Q0084 by 0296–0298; Q0085 by 0284; Q0086 by 0285, 0299 and 0300; Q0087 by 0301; Q0088 by 0294. Q0089 by 0303 (as built). Open, neither blocking and both as built: Q0090 (what a player perceives of events they don't see) and Q0091 (the pop-up turret's details).
  - **Slice status:**
    - A: done (`fcd06dc`).
    - C: done (`f67df212`; Q0077).
    - D: done (`edc30e11`; Q0076).
    - I: done (`9aca8972`).
    - R1 (sim): done (`8eab2290`).
    - R2 (client, docs, citation test): done (`c60b8d0f`, and the citation pass `06d82614`).
    - B: done (`35e19634`).
    - J2: done (`c5910e4d`; Q0086).
    - O: done (`356d1a96`; Q0083's placeholders).
    - E: done (`4933e094`; Q0079's placeholders; Q0087).
    - H: done (`f456b0ca`).
    - J1: done (`dd14dc38`). **With H, the first playable.**
    - F: done (`2cbb777e`). **Wave 5 is complete.**
    - K: done (`bbdbea73`).
    - S1: done (`50f51141`).
    - V: done (`dbefb552`).
    - S2: done (`0cee8bc4`). **Wave 8 is complete.**
    - M: done (`4e3fdd4a`). **M4 is complete.**
    - G: done (`a737a69a`). **Wave 6 is complete.**
    - U: done (`f34e4e6e`).
    - J3: done (`5e2e9419`).
    - I2: done (`4f31587f`).
    - O2: done (`2e0f31f4`).
    - L: done (`6d05e084`).
    - The rest are queued in the waves above.
  - **Technical follow-ups** (the Director's):
    - **One per-player event filter in the client.** M4-O found that each event consumer filters for the viewer on its own, and fixed one leak (a hidden Hideout's death marker). Put the filter in one place before more hidden objects arrive (S2's pop-up turret, N4–N7). **Done: M4-V, `dbefb552`.**
    - **"Target too close" for a location** (`systems/controls/command_behaviors.md > Target inside MinRange`): an Attack Ground that can't back off gives up without the feedback (a TODO in `orders/attack_ground.rs`). No M4 kind can target the ground with a MinRange, so the first that can (the Gunner in M5, or the Artillery Rig) adds it. **Done: M5-O3, `e64cf8d2`.**
  - **Earlier scope note:** tunnels, Hideout, Agent, Guard economy, 0186's Optics Lab gate and Agent pistol, the Grenadier (0187) and the Assault Bike (0188), once their numbers exist. Underground expansions are exempt from the flatness rule (0005).
- **M5 Cults** (**in progress**): Recruitment Hall, Storage, Recruit, Armory, Militant (renamed from Soldier, 0302), Gunner.
  - **The plan: `artifacts/director/m5-plan.md`,** adopted from planning spike M5-P's draft, with the Director's decisions at its top.
    - **The slices:** 15, including O3 for 0304, plus answer sweeps S1 and S2, in 8 waves; D runs on Fable.
      1. A (Cults foundation) ∥ R (playtest pairings);
      2. C (Recruitment Halls) ∥ D (Recruit gathering) ∥ F1 (buildings that hold units);
      3. B (Cults start) ∥ E (group construction) ∥ F2 (Armory training);
      4. J1 (Recruit UI) ∥ L (oracle and goldens) ∥ O3 (TailDisjointed, 0304). **This is the first playable.**
      5. G (the Gunworks, if Q0092 and Q0093 say so) ∥ S1 ∥ a gated slice whose numbers have arrived;
      6. X1 (the sim's ruleset), alone;
      7. X2 (data files and live reload) ∥ S2;
      8. M (QA), then the user's playtest.
    - **Data files (T66):** JSON merge patches over the built-in Rust rules, swapped in at a tick boundary through a recorded command, and reloaded live in dev builds.
    - **Questions:** Q0092–Q0108 are filed; none blocks.
    - **The user decided (2026-11-25):** the playtest is **hot-seat** (both sides, switching with F2), and it starts from the **command line**. No AI and no start screen in M5. (2026-11-26: the user now wants basic computer opponents for the next playtest; see In progress.)
    - **M5 starts** after M4-M lands.
  - Recruits are Groupable: every selected Recruit builds (0072).
  - Shift-placement queues build orders while Shift is held, with the cost paid per order (0071).

### M6: the GDO's and the Syndicate's remaining content (planned: `artifacts/director/m6-plan.md`, adopted 2026-11-26)
- **Scope:** 0366–0369: every designed GDO and Syndicate building and unit that isn't built yet, on 0366's and 0367's placeholder numbers. This is the Director's reading of the user's "just continue with the milestones work" (2026-11-26); the user was told, and can redirect.
- **When:** after M5. M6 starts at X1's merge, since X1 and X2 stay in M5 and M6 is written in X1's form (T101).
- **Slices:** 27, about 73.5 agent hours. V3 and V4 moved into M5 at adoption.
  - **Kinds:** GK and SK.
  - **The GDO:** GR, GD, GF, GA, GM, GP, GB, GH1, GH2, GL and GS.
  - **The Syndicate:** SV, SN, SB, SC, SL, SD, ST1, ST2, SP, SA and SR.
  - **The tail:** the answer sweeps S6a and S6b, and QA, MQ.
- **Waves:** W1–W11 from X1's merge, three builds a wave; X2 and M share W1 and W2.
  - **The longest chains:** GK → GH1 → GH2 → GL; SK → SV → SD → SP → SR; SK → SV → SD → ST1 → ST2.
  - **A partway playtest, if the user wants one:** the ground war after W6, with QA slice MQa in W7.
- **Technical decisions:** T84–T101 (§D). For example, struct-update defaults for new def fields (T84), and aircraft that land and rearm (T88).
- **Questions:** Q0153–Q0169, none blocking. An answer that arrives before its slice launches goes into its brief; later ones go to S6a and S6b.
### Later
- **Vehicles**, turrets, air, area damage, research. The vehicles milestone starts with the movement model from 0033 (rotating silhouettes, oriented collision, turning near obstacles), 0035 (acceleration and collision response), 0039 (Rugged passability per base, hover drag), and tracked maneuver data. The Factory's Artillery Rig and Prospector need Advanced Mechanics (0001). GDO air (0014): limited ammo with rearming at an Airfield; at most 4 aircraft per Airfield; Air-to-Ground Hovercraft and Air-to-Air Fighter (needs Propulsion Labs).
- **Underground and air movement rules** (0043–0047):
  - underground units don't collide with each other;
  - surfacing is refused onto occupied spots and reserves its spot;
  - air ignores terrain, and HoverCraft are clamped to the map;
  - Gliders may overshoot the map edge, staying visible and attackable at the nearest edge tile.
- **Playtest prep** (after all four factions; the user's decision "Modules now, data files at playtest"): the defs and the `tuning/` modules load from data files, reloaded while the game runs. M4-R makes the defs serde-ready, so this is a loading layer, not a refactor.
- **Key rebinding UI** (0011): the bindings table is built rebindable in M1-A.
- **Online opponents**: decide with the user. Computer opponents are planned under In progress > Queued for the playtest.
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
| Q0076 | Agents jam head-on between a near patch (0–1 GU) and the hatch: 3+ Agents halve the income (5.33 against 12.5 SC/s). Pass through like Drones, move patches to 3 GU or more, or a right-of-way rule? | no (answered) | answered by 0294 (Agents keep collision, mining 4 s, reference distance 3 GU) and 0295 (the hatch square and the 6 GU search radius, as built); M4-L builds 0294 and updates the marks |
| Q0077 | Eject while every unit of the type is in transit: refused or queued? And where an ejected unit with no rally waits: 1 GU out, as built, or farther? | no | answered by 0288 (as built; M4-I2 `4f31587f`: the marks and the verifications) |
| Q0078 | Tunnel near the map edge: 0242's Verification (2 tiles from the edge is red) is one tile off its own rule | no | answered by 0283 (as built); F updates the placeholder comments |
| Q0079 | Agent build orders: a refund if the Agent is killed before the start; does a pending ghost or a site reserve its whole area? | no | answered by 0289 (as built; M4-J1 `dd14dc38`: the red ghost; M4-I2 `4f31587f`: the marks and the refund test) |
| Q0080 | Expansion placement: do ground units and lingering ghosts block it; is a Warp In over a hidden expansion red at once? | no | answered by 0290: units and ghosts don't block, and a Warp In is judged like direct placement; M4-F re-briefed |
| Q0081 | ExpandMenu slots: do expansions not yet in the game keep their greyed slots, so the turret's letter never moves? | no | answered by 0291 (as proposed); M4-F builds it |
| Q0082 | Supplies gathering: pick-up and drop-off reaches; nearest Tunnel measured to the Supplies hatch; the old load lost at the end of the new pick-up | no | answered by 0292 (points 1 and 3 as built; point 2 built by M4-I2 `4f31587f`) |
| Q0083 | Area damage: friendly fire, the reveal, Attack Ground's end, how grenades are drawn | no | answered by 0293 (points 1 and 2 as built; points 3 and 4 built by M4-O2 `2e0f31f4`) |
| Q0084 | Laser Turret: hidden when, sight, blocking, payment, refunds, sites, the Agent's exit, targeting | no | answered by 0296 (points 1–3: hidden only while lowered, sight 6, a new blocking rule), 0297 (points 4–6: paid at the order; **75% cancel refund for both**; sites as their flows) and 0298 (points 7–9, as proposed). M4-S1 (`50f51141`) built points 4–9 for the Agent's turret; M4-S2 (`0cee8bc4`) the pop-up rules and the Tunnel-built site. Follow-up details in Q0091 |
| Q0085 | Stale doc lines that later answers contradict (seven lines) | no | answered by 0284 (the docs corrected; as built); F updates the comments |
| Q0086 | The EjectMenu's order, zero-count tiles and state after a click; when Tunnel Areas are drawn; recalling an all-held group | no | answered by 0285 (point 1), 0299 (points 2, 4, 5) and 0300 (point 3, and the preview's red cells). Point 4 differs from the built version: enemy areas are never shown, and placing a Tunnel previews its Tier 2 and Tier 3 areas (M4-J3) |
| Q0087 | Does the Agent that finishes a Tunnel wait out the transit delay before it can leave? | no | answered by 0301: (a), as built; rallied produced units have no delay either (M4-J3 updates the marks) |
| Q0088 | The Agents' jam: what each fix costs (the reply to 0286 and 0287). It isn't head-on; the infantry sidestep exists in the code; only pass-through reaches 12.5 at 0–2 GU | no (answered) | answered by 0294: option (c) with a 4 s mining time; stage 0 documented; 1a not adopted |
| Q0089 | Three Agents at 3 GU measure 10.67 SC/s against the table's 12.5 (−14.7%, inside ±15%): is the opening target the third Agent at work by 0:40, or the Tunnel at 12.5? | no (answered) | answered by 0303: (a) as built; the third Agent at work by 0:40, and the table is an ideal ("It's better with some variation") |
| Q0090 | What a player perceives of events they don't see: an unseen shooter's colour, another player's defeat, marks seen late (M4-V's open doubts) | no | answered by 0357: (a) on all three, as built; S1 shows the defeat notice |
| Q0091 | Pop-up turret details: its Tunnel dying with a unit on top, a target while it retracts, a Tunnel-built turret's ground and tier, when a site takes Tunnel Space (M4-S2's readings) | no | answered by 0358, with point 1 replaced by 0360 (done, M5-Y) |
| Q0092 | Which TBD Cults buildings get placeholder numbers for the first playtest (decides whether M5-G runs) (M5) | no | answered by 0319: every designed Cults building and unit gets placeholder numbers, so all are in M5, in an order of the Director's choosing |
| Q0093 | Placeholder stats for the Gunworks (M5) | no | answered by 0319 (option a): the Gunworks at 150 SC / 480 f, 2x2, 300 HP, 1/4, with no commands and not enterable. These supersede the plan's interim numbers |
| Q0094 | The Storage's armour: 1/6, or the rule's 1/4 (M5) | no | answered by 0319 (option a): the Storage's armour is 1/4, since 1/6 was left over from its old 3x2 size. A built 1/6 as a placeholder; S1 changes it and removes the armour test's exception |
| Q0095 | The Recruit's panel letters, and its pointer over a building under construction (M5) | no | answered by 0322: Q Move, A Construct, S Assist, D Repair, X Stop, the same layout as the Agent's |
| Q0096 | The Recruit's Construct menu: 13 buildings, 11 slots (M5) | no | answered by 0327 with (a): A Basic and W Advanced Construct, fixed slots by branch. J1 builds both menus |
| Q0097 | Group construction: where Recruits build, when the site starts, who counts (M5) | no | answered by 0339: closest part and wall-hug, own Recruits on the footprint block, Recruit-frames; the rest as proposed |
| Q0098 | Group construction: payment and refunds (M5) | no | answered by 0340: all four (a)s |
| Q0099 | Recruitment Area: off-map tiles, rounding, priority, Halls under construction (M5) | no | answered by 0341: the area wholly on the map, capacity rounded down, same-tick ties by the nearer centre; points 4 and 5 as proposed |
| Q0100 | Hall production: the exit side, X, counting the Recruit in production (M5) | no | answered by 0342: AAAB with the exit on B, no X, the Recruit in production counts, restart from 0 (F2) |
| Q0101 | Cults Unit Control: the global rule, and Recruits no Hall produced (M5) | no | answered by 0344 with (a): the global figure is display only; orphans from the start |
| Q0102 | Which Storage a Recruit drops off at (M5) | no | answered by 0345 with (a) and its addendum, as built by M5-D |
| Q0103 | The Cults economy check: geometry, tolerance, targets, build line (M5) | no | answered by 0346: edge to edge, ±15%, one Storage per face, and a second scenario (the user's build line) reported without a target (L) |
| Q0104 | Entering the Armory: the spot, a full pool, loads, capacity (M5) | no | answered by 0347 (done: F2, J1 and F3) |
| Q0105 | Armory training: which Recruit, its HP, ejected Recruits, the doorway (M5) | no | answered: points 1–2 by 0332, points 3–5 by 0348 (to the rally, in ready order; Cults production buildings Ungroupable) |
| Q0106 | Evacuation: where and in what order units come out (M5) | no | answered by 0349 (done, F3) |
| Q0107 | Showing the Recruitment Area (M5) | no | answered by 0350 with (b), extended by 0351 (own pending Halls) and 0352 (remembered enemy Halls) (J2, running) |
| Q0108 | Stale lines in the Cults docs and the Enter command (M5) | no | answered: point 1 by 0319 (Enter's TargetObject), point 2 by 0322 (the Recruit's panel letters) |
| Q0109 | Agents at a hatch: who gives way? Four or more Agents whose patch lies to the side of the hatch gridlock for good (M4-W's diagnosis) | no (answered for now by 0315) | first step 0315 built by M4-W2 (`b81b2089`), and it doesn't fix the jam. The follow-up is Q0111 |
| Q0110 | Recruits head-on in a lane: who gives way? More Recruits than a Storage face's lanes starve each other for minutes (M5-D) | no (answered) | answered by 0320, option (d): crowding is the player's job, and Recruits keep their collision. Nothing to build; S1 changes the jam tests' citation |
| Q0111 | 0315 measured: Agents at a hatch still gridlock (mixed results). Add the backstop (a), accept (b), revert 0315 (c), or pass-through (d)? (M4-W2) | no (answered) | answered by 0324 with a plan: step 1, 0323's contact step, measured (M4-W3); steps 2 (a 2-square hatch) and 3 (stuck gathering Agents make way) only if needed, and the measurements go back as a new question |
| Q0112 | The Disciple's panel: Shroud's key, and casting with a group (M5: N4a, N4b) | no | answered: point 1 by 0328 (Shroud is on R), point 2 by 0334 (the Disciple is Ungroupable) |
| Q0113 | Shroud: what is under the cloud, and the release (M5: N4b) | no | answered by 0334 |
| Q0114 | Augmentation: HP when MaxHP changes, the reserved unit, and the weakest Recruit (M5: N1a, N1b) | no | answered: points 1 and 3 by 0330 and 0331 (a MaxHP increase keeps the damage; a decrease caps HP; no floors), point 2 by 0332 (first in, first trained) |
| Q0115 | The Garrison: targets, range, Eject All, casting, and augmented occupants (M5: N2) | no | answered by 0335 |
| Q0116 | Entering a building whose sides are all alike: Garrison, Temple, Shrine (M5: N2, N3, N5) | no | answered by 0335 |
| Q0117 | Temple research: queuing the next level, and the Cathedral check (M5: N3) | no | answered by 0335 |
| Q0118 | The Shrine's abilities: who is inside, recasting, and several Shrines (M5: N5) | no | answered by 0335 |
| Q0119 | Vehicles among other units: yielding and reversing; 0323 settled the grazing (M5: V2, N6) | no | answered by 0353: the defaults (V2, N6) |
| Q0120 | The Sparrow's flight: separation, the map edge, takeoff, idling and strafing (M5: H1, N7) | no | answered: point 1 by 0328, points 2–7 by 0354; 0355 halves the grip (H1, N7) |
| Q0121 | Pool production beyond the Armory: which Recruits, and the product's HP (M5: N4a, N6, N7) | no | answered: points 1 and 3 by 0332 (order of entry; infantry keep the damage), point 2 by 0330 (vehicles and aircraft at full HP) |
| Q0122 | The Sparrow at the playtest: the GDO and the Colonists can't hit air yet (M5: N7, M) | no | answered by 0356: the Sparrow stays; the GDO gets the Rocketman and the Arsenal (M5-K); the Colonists wait |
| Q0123 | Stale lines in the Cults docs after 0319 (M5: J1, G2) | no | answered by 0328: the doc lines are updated |
| Q0124 | Repair: when it ends, an empty bank, and which targets (M5: N8) | no | answered: points 1–4 by 0329 (stop at full HP; an empty bank ends the order; no glider repair; expansions regenerate instead of being repaired), 5–6 by 0328 |
| Q0125 | The GDO's repair in M5: the Field Engineer has no numbers (M5: N8, M) | no | answered by 0328 with (b): placeholder numbers for the Field Engineer and the Career Campus. The Director builds them in M5 as slice N9 |
| Q0126 | The Gunner's miss: no ground unit can leave the locked point in its 2-frame Firing; and do buildings count as at the spot? (M5-O3) | no | answered by 0333: FiringDuration 4 and CooldownDuration 6; structures count at the spot; the tie rule stands |
| Q0127 | Agents at the hatch after 0323: no lock-ups any more, but at a crowded hatch extra Agents wait 60 s or more in 6 of 30 layouts. Accept (a), a 2-square hatch (b), the backstop (c), or join the line from farther away (d)? (M4-W3) | no | answered by 0336 with (a): an oversaturated hatch is the player's job; steps 2 and 3 of 0324 are dropped |
| Q0128 | 0323's contact step per base: confirmation (0336), HoverCraft listed though it is an air base, and the DrillUnit's step (M4-W3) | no | answered: point 1 by 0337 (HoverCraft was a slip), point 2 by 0338 (the DrillUnit steps sideways) |
| Q0129 | The contact step's side at a standing blocker: the near side as built (a), the near side off-centre with right first when square (b), or right first always (c), which brings the lock-up back (M4-W3) | no | answered by 0343 with (a), as built |
| Q0130 | Placing a Cults building where the order's own Recruits stand: the order fails at once with no warning. Refuse on the ghost (a), warn (b), no warning (c), or wait for them to step off (d)? (M5-E) | no | answered by 0362 with (d): the site waits for its own Recruits to step off (done, J1) |
| Q0131 | 0362: does a Recruit that holds the Build queued, behind another order, make the site wait? (M5) | no | answered by 0372 (a): the site waits, as J1 built. 0371 confirmed the off-by-one in 0341's example. Done (S2a) |
| Q0132 | Two doc lines still give the Syndicate Agent-built turrets (`tunnels.md > Losing Tunnels`, `agent.md > Building`) (M5) | no | answered by 0371 (a): leftovers; built as 0359 and 0360 say |
| Q0133 | Can a selected own Hall's claims reveal an enemy Hall the player has never seen? (`recruitment_hall.md > Recruitment Area display`) (M5) | no | answered by 0373 (a): the true claims, as J2 built. Done (S2a) |
| Q0134 | Buildings on Buildable ground that units can't walk on: how do their units come out? (`core/map.md > Tile`, `core/entities.md > Structure Type > Exit Doorway`) (M5) | no | answered by 0374 (a): the doorway holds the unit, as Z1 built. Done (S2a) |
| Q0135 | The Cults' economy check: 5 figures miss tolerance, and the realistic opening is far from level (`factions/cults/economy.md > Economy check`) (M5) | no | answered by 0380–0383: the Storage goes 3x2, and the oracle measures to 6:00. Done (S2a); the table and its findings are **Q0144** |
| Q0136 | A patch equally near two Storages: which one does a Recruit use? (`recruit.md > Gathering`, 0345) (M5) | no | answered by 0375: measured from the worker when it finishes mining, ties to the earlier id. Done (S2a) |
| Q0137 | When does a unit's turret let go of its locked target, and does a Move clear it? (`commands.md > TurretCommandState`) (M5) | no | answered by 0376 (a); done (S2b): Stop already cleared the lock |
| Q0138 | Altar: can fully augmented units leave by Release (0337), when two older lines say only by Eject All? (`augmentation.md > Altar > Entry`, `> Eject All`) (M5) | no | answered by 0371 (a): Release or Eject All, as N1b built (`8cd9020f`) |
| Q0139 | Does a Garrison with armed units inside count as a threat to enemy target scans? (`behaviors.md > TurretAutonomousScanning > Target Selection Priority`) (M5) | no | answered by 0377 (b); done (S2b): a manned Garrison is a threat |
| Q0140 | A right-click on a full, damaged own building: Enter or Repair? (0325) (M5) | no | answered by 0378 (b); done (S2b): Repair when full at click time |
| Q0141 | Barracks E: does queuing a Field Engineer need free Unit Control? (`barracks.md > ObjectInterfaceState[Barracks]`, `gdo/overview.md > Unit Control`) (M5) | no | answered by 0371 (a): checked when production starts, as N9 built (`1cfb889c`) |
| Q0142 | Neutral units and research: does a defeated player's research still apply to their units once they are neutral? (`core/game_rules.md > On Defeat`, `core/tech_and_research.md > Research`) (M5) | no | answered by 0379 (b): neutral units keep their research; built by N3 (`63003d0c`) |
| Q0143 | Shroud: does an attack absorbed by a cloud reveal its attacker? (`core/vision.md > Attacks from outside vision`, `disciple.md > Spells > Shroud`) (M5) | no | answered by 0386 (a): as built in N4b |
| Q0144 | The economies compared to 6:00: the realistic opening never overtakes ideal mining on one 8-patch field, and ideal mining is still about 0.9 by 2:48. What targets now? (`factions/cults/economy.md > Economy check`) (M5) | no | answered in part by 0385: the economy is rerun on a two-sided 8-patch line, One Hall against Two Halls (M5-S3, queued); the 2:48 target waits for the new figures |
| Q0145 | Does a player see the Shrine's buffs on enemy units? (`factions/cults/structures/shrine.md`) (M5) | no | answered by 0393 (a): as built |
| Q0146 | The economy oracle is wrapped up (user, 2026-11-26): its last figures, and closing the Economy check (`factions/cults/economy.md > Economy check`) (M5) | no | answered by 0401 (a): closed, and the targets stay as playtest reference |
| Q0147 | Basic computer opponents for playtesting: what they know and how they play. The design has no computer players yet | no | answered by 0402: fair knowledge, and (a), a playtest tool |
| Q0148 | Gliders: a lone Sparrow makes one pass; a glider's Move never completes; SeparationRadius under the Silhouette (`sparrow.md`, `systems/units.md`) (M5-H1) | no | answered: part 1 by 0403 (sight 8), part 2 by 0404 (Moves complete), part 3 by 0405 (slow separation, radii above silhouettes) |
| Q0149 | With sight 8, a lone Sparrow still loses bigger targets, since its loop grows with the target's size (`sparrow.md`) (M5-H2) | no | answered by 0408 (c): the pass ends at the target's centre (N7) |
| Q0150 | The Colonists doc says a building under Warp In has no rally point, against 0396 (`factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel`) (M5-S6) | no | answered by 0406 (a): as built |
| Q0151 | After "Attack goes to everyone": shielded Peacekeepers in an explicit Attack, box-selecting groupable Halls, and A on a building's panel (`peacekeeper.md`, `selection.md > BoxSelection`) (M5-S5) | no | answered: part 1 by 0409 (a), part 2 by 0411 (a), part 3 by 0407 (a), all as built; 0410 followed from part 2 |
| Q0152 | At a quarter strength, gliders take 11–17 s to drift apart, not 0405's "a few seconds" (`systems/units.md > Air Collision`) (M5-H2) | no | answered by 0412: a third of the HoverCraft strength (N7) |
| Q0153 | Tracked vehicles: is the turning rate the same at every speed, as the SpeedTurnRadius law gives (at ratio 8 the Hand of Heaven turns on 40 su at full speed), or does slowing turn faster, as "trading speed for turning rate" suggests; a crusher turning in place beside an enemy crushable (the doc's TBD); the Subterranean Transport's above-ground numbers, and whether it crushes (`systems/units.md > SpeedTurnRadiusMovement`, `> Crushing`) (M5-V3, M5-V4; M6-ST1; from M6-P's D42) | no | answered: parts 1 and 3 by 0417 (a tread law with TrackWidth, folded into **M5-V3**'s brief), part 2 by 0418 (a spin crushes the enemies it sweeps) and part 4 by 0419 (the surfaced Transport crushes), both for **M5-V4** |
| Q0154 | HoverCraft movement: DragMovement numbers for the Hornet, the Phantom, the Prism and the drone (M6-GK, SK, GD, GH1, SD, SP; from M6-P's D32) | no | answered by 0420 (option c: DragMovement numbers of their own, in the Supply Chopper's pattern) |
| Q0155 | The Drone Operator: its Barracks key, and the drone's details (M6-GK, GD; from M6-P's D33) | no | answered by 0421 (R at the Barracks; no rebuy while the drone is out; the leash edge to edge) |
| Q0156 | Scan with several Radar Stations selected (M6-GR; from M6-P's D39) | no | answered by 0422 (option b: Groupable false) |
| Q0157 | Does low Power slow Scan's cooldown, the Missile Turret's fire and rearming? (M6-GR, GF, GH2; from M6-P's D40) | no | open |
| Q0158 | Inside the network: two services at once, and which unit the EjectMenu picks (M6-SN, SB, SA; from M6-P's D48) | no | open |
| Q0159 | The Shadebringer's trails: standing still, when a unit is in one, and the Tunnel Network (M6-SB, SC; from M6-P's D44) | no | open |
| Q0160 | Twister mines: what they count as, what sets them off, destroyed mines, and buying (M6-GM; from M6-P's D35) | no | open |
| Q0161 | The Outpost's HP when a damaged Prospector deploys (M6-GP; from M6-P's D36) | no | open |
| Q0162 | GDO aircraft at the Airfield: landing, take-off, new aircraft, rearming, and the Dragonfly (M6-GH1, GH2, GL; from M6-P's D34) | no | open |
| Q0163 | How the Field Engineer builds the Bunker and the First Aid Tent, and what cancelling refunds (M6-GB; from M6-P's D37) | no | open |
| Q0164 | A shielded Peacekeeper in a Bunker, and whom the First Aid Tent heals (M6-GB; could touch M5's Shrine casts; from M6-P's D38) | no | open |
| Q0165 | The Phantom at a Tier 3 Tunnel's air doors: descending in and rising out (M6-SD; from M6-P's D46) | no | open |
| Q0166 | The Subterranean Transport: firing out, passengers in the network, and Tunnels while burrowed (M6-ST1, ST2; from M6-P's D45) | no | open |
| Q0167 | Smoke: revealed attackers, and Scan (M6-SC; from M6-P's D43) | no | open |
| Q0168 | The Prism: its panel, aiming a thrust, and orders while it launches (M6-SP, SR; from M6-P's D47) | no | open |
| Q0169 | GDO Supplies: what they buy, and the Supply Chopper's repair on its Tower (M6-GS; from M6-P's D41) | no | open |
| Q0170 | Orders given to a Sparrow during its takeoff run (from M5-N7's placeholder: they wait for the run's end, and Stop doesn't end it) | no | open |
| Q0171 | Does a Shift-queued order end a Target Lock or a Shroud channel, or wait behind it? (as built for Shroud, and being built for Target Lock in M5-K: it waits, T76) | no | open |

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
| 0071 No smart-casting; Shift while placing | done (audit M1-H3 `80b3d88`; placement M2-H2 `2d507b4`); the Recruit Shift-queue queued (M5-E) | |
| 0072 Recruits are Groupable | queued (M5-E: every selected Recruit builds; M5-J1) | |
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
| 0203 The Rocketman's normal attack has a MinRange (value TBD) | queued (M5-K: MinRange 2, by 0356; the MinRange rules are built, M3-L2) | context for Q0054 |
| 0204 AttackMove's dropped target, the MinRange back-off spot and a cornered unit stay as M3-L2 built them; a locked Rocketman's fire ignores MinRange | done (as built M3-L2), except point 1, superseded by 0230 (M3-L4); the locked-fire part is queued with Target Lock | answers Q0054 |
| 0205 The Drone accelerates faster (ForwardAcceleration 0.9, OmniDirectionalAcceleration 0.3, DragRatio 0.15; were 0.6 / 0.2 / 0.1); re-run the Colonists oracle and report the table for Q0055 | done (M3-K3, `c4807b8`); figures re-reported as Q0072 | a step toward Q0055, which stays open; the table and targets stay unchanged |
| 0206 While an attacker in the fog is revealed, its own shots are drawn from it to the target; other shots stay clipped to Visible tiles | done (M3-J3, `827dfeb`); narrowed by 0244 (M3-L5); superseded by 0293 point 4 (M4-O2 `2e0f31f4`) | answers Q0056; client only |
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
| 0229 Placeholders: Optics Lab (2x2, 150 SC, 400 frames, HP 400, armour 1/4, Groupable, no commands); Tunnel upgrade takes 480 frames per tier, with Supplies paid when ordered | done (M4-K, `bbdbea73`: the Optics Lab; M4-G, `a737a69a`: the upgrade's 480 frames, paid when ordered) | answers Q0059 and Q0060 points 1–2; unblocks K and G |
| 0230 AttackMove takes only targets it can reach within the leash and sight; no dropped-target memory; a rejected enemy is re-judged only when it moves or PathKnowledge changes; performance a priority | done (M3-L4, `b261863`); own movement isn't a trigger (Q0074) | answers Q0058; replaces 0204 point 1 and M3-L3's placeholder |
| 0231–0232 Tunnel upgrades: an upgrade in progress counts toward the next one's cost; Cancel refunds the current top price, not counting itself (0232 corrects 0231); HP rises with max HP; Supplies are lost if the Tunnel is destroyed mid-upgrade; the larger area is reserved from the start | done (M4-G, `a737a69a`) | answers Q0060 points 3–6 |
| 0233 Agent-built Tunnels pay when the ghost is placed; ghosts and sites count toward the price; an order that can't be paid is refused with feedback; refunds at the current top price | done (M4-E, `4933e094`: the price, payment, the gate's refusal and the refunds; M4-J1, `dd14dc38`: the price in the A tile's hint and the refusal's feedback) | answers Q0061 points 1–2 |
| 0234 A Tunnel under construction: Cancel only, ConstructionHP is the maximum and damage stays, the Agent comes out at the centre, the site counts for defeat, the Agent keeps its load, enemies see the site and not the Agent | done (M4-E, `4933e094`: the site, its HP, Cancel, the Agent's exit, defeat, the load, enemies see the site and not the Agent; M4-J1, `dd14dc38`: the site's look and its X-only panel) | answers Q0061 points 3–5 and 7–9, as proposed |
| 0235 A chase given up by the Failure rule during an AttackMove counts as "can't reach": rejected until 0230's triggers fire | done (M3-L4, `b261863`) | answers Q0073 (A) |
| 0236 The measured Colonists economy is the design: the table is K3's figures, target 1 is "the Colonists lead from the start to about 1:40", saturation is about 24 Drones | done (M3-K4, `f47fb26`) | answers Q0055 and Q0072; no game number changes |
| 0237 Tunnel Areas may not overlap any player's Tunnels: crowding and blocking an enemy's Tunnels is allowed. The ghost judges known enemy Tunnels at their remembered position and tier; the check on arrival and upgrade checks use the truth | done (M4-E, `4933e094`; M4-J1, `dd14dc38`: the ghost; M4-G, `a737a69a`: the upgrade check by the truth) | answers Q0062 point 1; replaces the own-Tunnels-only proposal |
| 0238 A Tunnel Area's cells past the map edge count for Tunnel Space; expansions are placed on map cells only | superseded by 0242 | answers Q0062 point 2 |
| 0239 Every tile under an expansion must be Drillable (Plane, Rugged and Cliff; not Mountain or Water); own and neutral underground structures and the Tunnel's footprint block it | done (M4-F, `2cbb777e`) | answers Q0062 points 3a, 3b and 3e, as proposed |
| 0240 Surface buildings and underground expansions never share a tile, whatever the owner (neutral included). A direct placement (GDO) over a hidden expansion is red at once, an accepted leak. A worker-built ghost judges by what the player knows and fails on the worker's arrival. Units still walk over expansions | done (M4-E, `4933e094`: surface placement reads the underground truth, and a worker-built ghost fails on arrival; M4-F, `2cbb777e`: no expansion under a surface building) | answers Q0062 points 3c and 4; replaces T32's "the surface methods ignore them" |
| 0241 No expansions under resources (patches, Supply Delivery Stations); non-Drillable cells give no Tunnel Space | done (M4-F, `2cbb777e`: no expansions under resources); the Tunnel Space part is superseded by 0242 | answers Q0062 points 3d and 5 |
| 0242 A Tunnel's whole area must be on the map and Drillable, for placement and upgrades; every tunnel-region cell is usable; starting layouts must respect it | done (M4-B, `35e19634`; M4-E, `4933e094`; M4-J1, `dd14dc38`; M4-G, `a737a69a`: the upgrade check, 3 tiles from the edge for Tier 2 and 4 for Tier 3) | completes Q0062; replaces 0238 and 0241's Tunnel Space part |
| 0243 AttackMove: a reach-rule rejection is also re-checked once the unit has moved about 1 GU from where it judged; Failure-rule and fog rejections keep 0230's triggers; measure the cost | done (M3-L5, `a777ae3`); the 1 GU step is kept: at most 1.45× master's mean tick, about +0.05 ms for 100 Guards | answers Q0074 (B, "to try") |
| 0244 A revealed attacker's shot is drawn whole only when its target is the viewer's own object; everything else is clipped to Visible tiles | superseded by 0293 point 4 (M3-L5 `a777ae3` built it; M4-O2 `2e0f31f4` removed the exception) | answers Q0075; narrows 0206 |
| 0245 Tunnel Network: a flat transit delay of 32 frames (placeholder) after entering before a unit can exit; produced units have none | done (M4-C, `f67df212`) | answers Q0063 point 1 |
| 0246 Units walk into a Tunnel single file at normal speed with normal spacing, no fixed hold per unit ("one unit wide"); produced units exit without delay | done (M4-C, `f67df212`) | answers Q0063 point 7 and part of 2; replaces the 8-frame hold proposal |
| 0247 Entering a Tunnel uses the normal Failure rule; no special waiting rule | done (M4-C, `f67df212`) | answers Q0063 point 2, as proposed |
| 0248 Tunnel sides corrected: A entrance and exit, B crystal hatch, C back wall (no function), D Supplies hatch. Side A's exit path is the half nearer B, the entry path the half nearer D | done (A `fcd06dc`, C `f67df212`, D `edc30e11`, I `9aca8972`: Supplies at D) | answers Q0063 point 2 (which half of Side A); corrects 0224's B and C hatches |
| 0249 Inside the network: orders cleared, control groups and loads kept, slow HP regeneration; a Tunnel being upgraded keeps its old tier for entry and exit until it completes | done (M4-C, `f67df212`; M4-G, `a737a69a`: a Tunnel being upgraded keeps its old tier for transit) | Q0063 complete (0245–0249) |
| 0250 Ejection: ground units walk out single file, gated on the exit spot being clear; air doors have their own rate (8 frames) | done for ground units (M4-C, `f67df212`); the air doors wait for the first Syndicate air unit | answers Q0064 points 1, 2, 4, 10 and 11 |
| 0251 Ejection: the longest-inside unit goes first; queue rules; air doors on the roof over the matching path halves; Vault and Null Chamber limits noted | done for ground units (M4-C, `f67df212`); the air doors wait for the first Syndicate air unit | answers Q0064 points 3, 4 and 11 |
| 0252 Syndicate rallies: a rally on a Tunnel stays inside; one on an expansion means the surface spot above it | done (M4-C, `f67df212`; client tests M4-J2, `c5910e4d`) | answers Q0064 points 5–7 |
| 0253 Auto-ejected units share the exit queue | done (M4-C, `f67df212`) | Q0064 complete (0250–0253) |
| 0254 Expansions under construction, as proposed, except the 75% refund on cancel | done (M4-A, `fcd06dc`: sight 0 and Groupable; M4-H, `f456b0ca`; M4-F, `2cbb777e`: paid at the click, ConstructionHP, the 75% refund, the Tunnel's X, Eject meanwhile, and the expansion dying with its Tunnel) | answers Q0065 |
| 0255 Agent gathering: the Drone's rules, the nearest Tunnel, the 1-square hatch as the drop-off target, and Agents wait at the hatch for ever. The Drones' placeholder (retry every 2 s, never give up) is the settled rule | done (M4-D, `edc30e11`); patches 0–1 GU from the hatch jam with 3+ Agents (Q0076) | answers Q0066; confirms Q0065 point 4 (an expansion's cells are reserved from construction start) |
| 0256 Supplies gathering: shared stations, waiting at an empty station, switching resources loses the load; the notification waits for a notification system | done (M4-I, `9aca8972`); Q0082's three proposals as placeholders | answers Q0067 |
| 0257 Syndicate panel letters on the 4x3 grid | done (M4-J2, `c5910e4d`: the Tunnel's and the Hideout's panels and the EjectMenu; M4-J1, `dd14dc38`: the Agent; M4-F, `2cbb777e`: the ExpandMenu; M4-G, `a737a69a`: Q) | answers Q0068, as proposed |
| 0258 Losing Tunnels: decay %, attacking neutral expansions, reclaiming, blocked cells (counted as occupied), defeat, and the enemy sees the tier | done (M4-A, `fcd06dc`: the tier seen and remembered; M4-H, `f456b0ca`: neutral expansions, decay, explicit Attack only, reclaiming, stranded units, defeat; M4-F, `2cbb777e`: neutral expansions block expansion placement; M4-G, `a737a69a`: reclaiming at an upgrade's completion) | answers Q0069 |
| 0259 The Syndicate economy check: measured to the hatch square; the stacked layout puts the Tunnels on opposite sides | done (M4-L, `6d05e084`; the 1 GU opening layout replaced by 0294's 3 GU) | answers Q0070 |
| 0260 Placeholders for the Laser Turret, the Grenadier and the Chemical Plant, so they can be planned into M4; the Grenadier needs a projectile, AoE and Attack Ground combat spike | done (M4-O, `356d1a96`: the combat spike; M4-K, `bbdbea73`: the Chemical Plant; M4-U, `f34e4e6e`: the Grenadier; M4-S1, `50f51141`: the Laser Turret, upright; M4-S2, `0cee8bc4`: the pop-up) | answers Q0071 (scope B) |
| 0261 Area damage never hits underground objects unless targeted | void (replaced by 0262) | — |
| 0262 Area damage does hit underground objects; expansions and the pop-up turret have high armor | partly done (M4-A, `fcd06dc`: damage never asks the hidden predicate; M4-O, `356d1a96`: area damage hits underground objects, even unseen); queued (S2: the pop-up turret's armour, per 0264) | answers Q0071's AoE-underground gap |
| 0263 Confirmed: undeploy, held orders, the Transport's base, the Phantom's attack | queued (T44, the deploy model; gated N3 and N6) | answers Q0071's items to confirm |
| 0264 Building armor rule: PointArmor n, FullArmor n × size in grid squares; underground expansions 6/24 for 2x2; the Laser Turret 1/1 raised, 6/6 underground | done (M4-A, `fcd06dc`: the Hideout 6/24; M4-B, `35e19634`: the defs test across factions; M4-O, `356d1a96`: "about 93" through a hidden Hideout's 6/24; M4-K, `bbdbea73`: the Optics Lab and the Chemical Plant at 6/24; M4-S1, `50f51141`: the turret at 1/1; M4-S2, `0cee8bc4`: 6/6 while lowered) | — |
| 0265 Transport passengers always count against Tunnel Space | queued (gated N3) | answers Q0071's Tunnel Space gap |
| 0266 A burrowed Transport passes through underground structures; surfacing rules | queued (gated N3) | answers Q0071's underground and surfacing gaps |
| 0267 A Tunnel-built Laser Turret uses the operation slot; detection comes after the faction rosters | done (M4-F, `2cbb777e`: the operation slot; M4-S2, `0cee8bc4`: the turret uses it) | Q0071 complete (0260–0267) |
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
| 0283 Tunnel near the map edge: the rule as written (0242's verification was a slip) | done (as built, M4-B `35e19634`; M4-F updated the comments; M4-G `a737a69a`: the upgrade edges, 3 and 4 tiles, tested) | answers Q0078 |
| 0284 Stale doc lines corrected to the newer rules (refunds, Agent letters, Laser Turret, armour, AoE) | done (as built; F updates the comments) | answers Q0085 |
| 0285 EjectMenu order: Agent, Guard, Grenadier, …; `units/README.md > Files` reordered | done (as built, M4-J2 `c5910e4d`; F updates the comment) | answers Q0086 point 1 |
| 0286 The user asks for cost estimates of six fixes for the Agents' jam; no design change | done: the estimates went as Q0088 (a Fable investigation; prototypes and tables in `scratchpad/q0076-costs/`) | about Q0076 |
| 0287 The user's leanings for the jam estimates: the 2-square hatch keeps one drop-off at a time but is less wanted (looks); the moving unit's sidestep is wanted for all units, in general | done: folded into Q0088 (the general sidestep estimated first; the hatch briefly) | about Q0076 |
| 0288 An Eject during the transit delay is refused (its tile greyed); an ejected unit with no rally stops 1 GU out | done (as built, M4-C `f67df212` and M4-J2 `c5910e4d`; M4-I2 `4f31587f`: the verifications as tests, and the marks) | answers Q0077 |
| 0289 A killed Agent's Tunnel order refunds the top price; a pending ghost reserves its Tier 1 area against its owner's ghosts; a site's area counts for everyone | done (as built in the sim, M4-E `4933e094`; M4-J1 `dd14dc38`: the red second ghost; M4-I2 `4f31587f`: the refund test and the marks) | answers Q0079 |
| 0290 Expansion placement: ground units don't block it; no ghost check is needed; a Colonists Warp In over a hidden enemy expansion is red at once, like direct placement | done (M4-F, `2cbb777e`) | answers Q0080 |
| 0291 The ExpandMenu's nine expansions hold fixed slots in table order, greyed while unavailable; the Laser Turret is always tenth | done (M4-F, `2cbb777e`; the turret's tenth slot is a `Planned` entry until S2) | answers Q0081 |
| 0292 Supplies gathering: 8 su reaches; the nearest Tunnel measured to the crystal hatch; the old load lost when the new pick-up or mining ends | done (points 1 and 3 as built, M4-I `9aca8972`; point 2, M4-I2 `4f31587f`) | answers Q0082 |
| 0293 Area damage: friendly fire; the reveal when damage lands, to the damaged objects' owners; Attack Ground keeps firing until another order; every shot and projectile drawn only over Visible tiles (replaces 0244) | done (M4-O2, `2e0f31f4`; points 1 and 2 as built by M4-O `356d1a96`) | answers Q0083 |
| 0294 Agent MiningDuration 80 → 64 frames (4 s); Agents keep collision; the opening and the table checked from 3 GU (rows 0–2 theoretical); `dev_skirmish` and the reference layouts put the patch 3 GU from the hatch; the moving-unit sidestep is documented (stage 0) | done (M4-L, `6d05e084`); 3 Agents at 3 GU measure 10.67, inside ±15% (Q0089) | answers Q0076 (except its two "also to confirm" points) and Q0088 |
| 0295 The hatch is the middle square of its side farther from Side A; the Agent's patch search radius is 6 GU, like the Drone's (a placeholder value) | done (as built, M4-D `edc30e11`; M4-L `6d05e084`: the marks) | answers Q0076's two "also to confirm" points |
| 0296 Laser Turret: hidden at 6/6 only while fully lowered; sight 6 in both forms; upright turrets and raised pop-ups block like buildings, a lowered pop-up never; a pop-up can't rise while any unit stands over it, and the owner's idle units there are pushed off | done (M4-S1, `50f51141`: an upright turret blocks like a building; M4-S2, `0cee8bc4`: the pop-up rules, with the verifications as tests); the upright form is removed by 0360 (M5-Y) | answers Q0084 points 1–3 |
| 0297 Laser Turret: an Agent's turret is paid at the order, with the Tunnel order's refund rules; cancelling either kind of site refunds 75%; an Agent's site is visible and a Tunnel's is hidden | done (M4-S1, `50f51141`; M4-S2, `0cee8bc4`); the Agent's turret is removed by 0359 (M5-Y) | answers Q0084 points 4–6 |
| 0298 Laser Turret: the Agent exits at the first clear spot from its side and waits inside if none; holding-position targeting that keeps its target; an Agent's turret uses no operation slot | done (M4-S1, `50f51141`); superseded by 0359 (M5-Y removes the Agent's turret) | answers Q0084 points 7–9 (Q0084 complete) |
| 0299 Every Syndicate unit type keeps its EjectMenu tile (0, greyed); selecting an own Tunnel shows its area; placing a Tunnel previews its Tier 1–3 areas with blocking cells red; enemy areas never shown; recalling an all-inside group does nothing | done (points 2 and 5 as built, M4-J2 `c5910e4d`; point 4, M4-J3 `5e2e9419`) | answers Q0086 points 2, 4, 5 |
| 0300 The EjectMenu stays open after a click; the preview also marks cells overlapping own other Tunnels' areas; the area shows while placing an expansion | done (points 1 and 3 as built, M4-J2 and M4-F; point 2, M4-J3 `5e2e9419`) | answers Q0086 point 3 (Q0086 complete) |
| 0301 No transit delay for the Agent that finishes a Tunnel, nor for produced units rallied inside | done (as built, M4-E `4933e094`; M4-J3 `5e2e9419`: the verification as a test, and the marks) | answers Q0087 |
| 0302 The Cults' Soldier is renamed Militant (`factions/cults/units/militant.md`); the Armory's Q reads "Train Militant" | done (M5-A, `ed23874e`: the kind is `Militant`, labelled MT; no code says Soldier) | — |
| 0303 The Syndicate's opening target means the third Agent at work by about 0:40; the Agent table is an ideal, and real income varies a little below it | done (as built, M4-L `6d05e084`; no change) | answers Q0089 |
| 0304 The Cults' Gunner can miss: a TailDisjointed attack (the position locks at the end of Aiming; the shot lands there at the end of Firing and hits whatever unit is at the spot), and Attack Ground | done (M5-O3, `e64cf8d2`); the miss can't happen with today's numbers: Q0126 | — |
| 0305 The Cults' Acolyte is renamed Disciple (`factions/cults/units/disciple.md`) | queued (M5-N4, gated: no code mentions the Acolyte) | — |
| 0306–0308 The Disciple's channeled cloud spell (concept): damage on ground objects under connected clouds goes into a shared buffer, released as AoE every 0.5 s; attacks from under the same cloud do full damage | queued (M5-N4, gated: "Concept only, not for the current playtest"; size, drain, end, buffer splits, name and hotkey TBD) | — |
| 0309–0312 The Disciple's cloud spell, named **Shroud** (0312, after "Shared Burden" in 0311): a release's share is ordinary damage, with the receiver's modifiers, armor and minimum; the buffer is released at once when the channel ends or the cloud splits; the channel ends only on death, a cancel command, or running out of energy (0310 corrects 0309: taking damage doesn't end it) | queued (M5-N4, gated: concept only; size and drain TBD) | — |
| 0313 The Cults' light vehicle, the Chop Shop's first: a wheeled buggy with a 360° machine-gun turret, FullyConnected, low damage at a high rate of fire | queued (N6, gated: concept only; name, Recruits consumed, target domain, numbers and the Chop Shop's flow TBD; vehicles aren't built) | — |
| 0314 The light vehicle (0313) is the **Jackal**: it fires at ground and air (Universal) and consumes 2 Recruits (Unit Control 2). **The Chop Shop trains like the Armory:** a Recruit pool, serial queueable orders that reserve Recruits, Eject All and Cancel | queued (N6, gated: concept only; the Chop Shop's capacity, queue and sides, and the Jackal's numbers, TBD). It will reuse F1's holding capability and F2's training from a pool | — |
| 0315 **Q0109, first step:** an Agent waits only once it is within reach of the hatch. Until then it is never "arrived": it keeps moving toward the hatch, steering round blockers, exempt from MovingToLocation's occupied-target arrival rule, and never gives up. Option (a) is not adopted for now; the user decides on it after the re-measurement | done as designed (M4-W2, `b81b2089`), but it doesn't fix the jam: Q0111 | — |
| 0316 A new Cults aircraft, the Airstrip's first: a small machine-gun glider hitting air and ground (name TBD) | queued (N6, gated: concept only; name, Recruits, whether it can miss, gun or turret, ammo, numbers and the Airstrip's flow TBD) | — |
| 0317–0318 The glider (0316) is the **Sparrow**: a turret with a 45° cone in total that can't miss (FullyConnected, Universal), unlimited ammo (it never lands), 1 Recruit. **The Airstrip trains like the Armory**, and an aircraft takes off and never lands back | queued (N6, gated: concept only; its numbers TBD). It will reuse F1's holding and F2's training from a pool | — |
| 0319 **Placeholder stats for every designed Cults building and unit** (answers Q0092, Q0093, Q0094 and Q0108 point 1): Gunworks, Machine Shop, Scrap Hangar, Garrison, Chop Shop (Jackal), Airstrip (Sparrow), Temple (research), Cathedral (Disciple, Shroud's numbers), Shrine (two abilities), Altar (augmentation, Metabolism). The Gunworks and Machine Shop have no commands and can't be entered. The Storage's armour goes from 1/6 to 1/4. Enter's TargetObject is "the structure being entered" | queued: **all of it is in M5 now**. M5-P2's addendum slices it: G2 (the simple buildings, the Gunner, every remaining kind), N1a–N8, and the engine slices V1, V2, U1 and H1; the Storage's armour goes to G2. Folded in already: C's lineage becomes one entry per Recruit (the Jackal takes 2; the T56 amendment), and F1 was told the holding variations. **G2 done** (`9b7986e4`): every remaining kind registered, the Gunworks, Machine Shop and Scrap Hangar buildable, the Gunner unlocked, the Storage at 1/4. **U1 done** (`54b9c2fe`): unit turrets, on test defs. **N2 done** (`b706f6c0`): the Garrison | — |
| 0320 **Q0110 answered (d):** Recruit crowding at a Storage face is the player's job. Recruits keep their collision, with no give-way rule, the Hall's rally on a patch included. The reference layouts and the Cults economy check stay within 4–5 Recruits per face | done (M5-S1, `09940dab`): the jam tests cite 0320 as accepted behaviour. L keeps its layouts at 4–5 per face. No movement change | — |
| 0321 **Repair:** Recruits repair own buildings and vehicles (Space Crystals); Agents repair own Tunnels (free) and Laser Turrets (Space Crystals); the GDO Field Engineer's repair gets numbers; repairers stack linearly. Placeholders: 0 to full HP in half the build time with one repairer, at half the price, paid gradually | done (M5-N8, `26501427`). The Field Engineer's repair is data for N9. **N9 done** (`1cfb889c`): the Field Engineer repairs | — |
| 0322 **The Recruit's panel** (answers Q0095 and Q0108 point 2): Q Move, A Construct, S Assist, D Repair, X Stop. **Two new pointers**, Assist and Repair (also when awaiting a target). **Right-click** on a damaged, completed own building or vehicle resolves to Repair; a damaged building that can be entered keeps Enter for now (open with the user) | done (J1, `2286859e`: the panel and the Assist pointer; N8, `26501427`: D Repair and the Repair pointer) | — |
| 0323 **The contact step is 90° and one body width** (Q0111's context): an infantry, hover or tracked unit blocked on contact while moving steps straight sideways by its own width, replacing the 45° step. Right first, the other side after 1 s, the look-ahead and the stuck rule are unchanged. WheeledVehicles turn and graze; Mechs turn, then step sideways (0325) | done (M4-W3, `5305c2f3`), with the near side at standing blockers (Q0129). WheeledVehicle's turn and graze goes to M5-V2; Mechs turn first when one exists | — |
| 0324 **Q0111 answered with a plan:** step 1, 0323's contact step with the 1-square hatch and 0315 kept, measured on Q0111's grid and the soak with 4–6 Agents; step 2, only if Agents still lock up, a 2-square crystal hatch with one Agent delivering at a time; step 3, if still needed, stuck gathering Agents make way ("loaded has right of way" is a suggestion). The measurements go back as a new question if step 1 isn't enough | done: step 1 (M4-W3, `5305c2f3`) fixed the jam; steps 2 and 3 dropped by 0336 | — |
| 0325 **Enter beats Repair:** a right-click on a damaged own building the selection can enter means Enter; such a building is repaired through the panel. Buildings that can't be entered, and vehicles, resolve to Repair (settles 0322's open point). The Agent gets Repair: Tunnels and Laser Turrets; a right-click repairs a damaged Laser Turret, and a Tunnel stays Enter, or Drop Off when carrying | done (M5-N8, `26501427`): Enter beats Repair, and the Agent's Repair is on D | — |
| 0326 **The Agent's Construct menu:** A: Construct opens Q Tunnel, W Laser Turret (greyed until available) and Z Back, replacing the direct build buttons (Q0068). Repair moves to D. The Agent's panel: Q Move, A Construct, S Drop Off, D Repair, X Stop | superseded by 0359: the Agent's panel is A: Build Tunnel again, and J1 builds no Agent Construct menu. D stays Repair (N8) | — |
| 0327 **The Recruit's build menus** (answers Q0096 with (a)): Q Move, **A** Basic Construct, **W** Advanced Construct, S Assist, D Repair, X Stop. Basic: Q Recruitment Hall, W Storage; A Armory, S Gunworks, D Garrison; X Altar. Advanced: A Chop Shop, S Machine Shop, D Airstrip, F Scrap Hangar; X Temple, C Cathedral, V Shrine. Rows follow the branches; every building has a fixed key and is greyed until available; Z goes back | done (J1, `2286859e`) | — |
| 0328 **Placeholders and cleanup** (answers Q0123 and Q0125, and points of Q0112, Q0120 and Q0124): the **Field Engineer** (LightInfantry 24x24, HP 50, sight 5, speed 4, 1 Unit Control; Barracks E, 75 SC, 160 f; Q Move, A Construct with Bunker and First Aid Tent greyed, D Repair, X Stop; right-click as Recruits, Enter wins) and the **Career Campus** (2x2, HP 300, armour 1/4, sight 4, Power −20, 150 SC, 320 f, no panel). **Shroud is on R.** The Sparrow's SeparationRadius is 32 su. A Tunnel repairs at its 480-frame rate at every Tier; a vehicle's repair price is crystals only. Stale lines fixed (the Gunworks and Machine Shop can't be entered; "Own Garrison → Enter Garrison") | partly done: **N9** (`1cfb889c`), the Field Engineer and the Career Campus; **N8** (`26501427`), the repair rules. Queued: Shroud's R (N4a, N4b); 32 su (H1). The doc lines need no code. Q0141 asks about a stale Unit Control line in barracks.md's E. **N4a done** (`aa220af2`): R holds `Planned::Shroud`. **N4b done** (`c0ca0956`): R is live | — |
| 0329 **Repair ends at full HP or an empty bank** (answers Q0124 points 1–4): repairers stop at full HP; with the bank at 0 the order ends; gliders can't be repaired; Agents can't repair Tunnel expansions, which instead regenerate 1% of MaxHP a second while their Tunnel lives (placeholder; in combat too, 0330) | done (M5-N8, `26501427`): a repair ends at full HP or an empty bank, gliders are refused, and the expansions regenerate | — |
| 0330 **HP rules** (answers Q0114 points 1 and 3, Q0121 point 2): on a MaxHP change, HP above the new maximum drops to it and otherwise stays, with no floors, so an augmentation never kills; vehicles and aircraft come out at full HP; the expansions' regeneration runs in combat | done for N1a (`698fbe78`): the cap, with no floor (`augmentation::follow_max`). N8's regeneration is done (`26501427`). Queued: N6 and N7, full HP | — |
| 0331 **MaxHP increases keep the damage** (refines 0330's answer to Q0114 point 1): when MaxHP goes up, HP goes up by the same amount (25/40 becomes 45/60); when it goes down, HP is unchanged unless it is above the new maximum, which caps it (8/40 becomes 8/10, 40/40 becomes 10/10) | done (M5-N1a, `698fbe78`) | — |
| 0332 **Cults production: first in, first trained; trained infantry keep the Recruit's damage** (answers Q0105 points 1–2, Q0114 point 2, Q0121 points 1 and 3). Every pool building reserves in order of entry (the Armory, Cathedral, Chop Shop, Airstrip, Altar); vehicles and aircraft still come out at full HP (0330) | done (F2, `d9492a69`). Queued: N1b, N4a, N6 and N7 by the same path. **N1b done** (`8cd9020f`): the Altar reserves the first in within the type and state. **N4a done** (`aa220af2`): the Cathedral trains the first in | — |
| 0333 **The Gunner fires for 4 frames** (answers Q0126): FiringDuration 2 → 4 and CooldownDuration 8 → 6, so a walking unit can step off the locked point; structures count at the spot, and the tie rule stands | done (F2, `d9492a69`) | — |
| 0334 **Shroud's rules; the Disciple is Ungroupable** (answers Q0112 point 2 and Q0113): an object is under a cloud when its centre is; touching clouds join; releases ignore directional armour; underground objects are affected; the Disciple's regeneration continues while channelling; every player sees the cloud over their Visible tiles; each cloud has its own timer, releasing 8 frames after the first hit it absorbs; clouds that join release their buffers at once and start a fresh joint buffer | done: **N4a** (`aa220af2`) keeps the Disciple Ungroupable; **N4b** (`c0ca0956`) builds the cloud rules, amending T77 | — |
| 0335 **The Garrison, entrances, the Temple, the Shrine** (answers Q0115–Q0118). Garrison: each occupant picks its own target, Range from the Garrison's edge, no spells from inside, augmentation doesn't change the borrowed attack, **one door on side B** (AAAB), and Eject All lets everyone out at once through the door, passing through each other until clear. **Every production and research building has an entrance and an exit** (one at a time, like a Tunnel): the Temple, Gunworks and Machine Shop become ABAC, in at B and out at C. Temple: Level 2 only after Level 1 completes; the Cathedral is checked at queue only. **General rule: losing a prerequisite never cancels or disables anything already started or finished.** Shrine: in the circle by centre; a recast restarts the duration, with no stacking; Shrines are Ungroupable | queued (N2, G2, N3, N5; S1 sweeps the existing factions for the prerequisite rule). G2 done: the Gunworks gate never cancels queued or training Gunners (`9b7986e4`). **N2 done** (`b706f6c0`): the Garrison's door on B, Eject All at once, targets and Range from the edge, the borrowed attack. **S1 done** (`09940dab`): the prerequisite rule holds everywhere already, now tested in `tech/tests/losing.rs`. **N3 done** (`63003d0c`): the Temple is ABAC, Level 2 waits for Level 1, and the Cathedral is checked at queue **N5 done** (`3197d711`): the Shrine casts from one Shrine per press, by its centre, with recasts restarting. | — |
| 0336 **Augment research is retroactive; the Shrine is ABAC; Q0127 answered (a)**. When a Temple level completes, every unit with that augmentation gets the new amounts at once, with HP by 0331. Level Locking is removed, and AugmentationState is two booleans, the amounts read from the current research level. The Shrine is ABAC, in at B and out at C, one at a time. **An oversaturated hatch is the player's job** ("a big success"): steps 2 and 3 of 0324 are dropped. It also asks to confirm 0323 for every listed base | partly done: M4-W3 asserts the accepted hatch (`5305c2f3`). Queued: N1a, N1b and N3 lose T73's levels and T74's locking; N5, the Shrine's sides. The confirmation went out as Q0128, answered by 0337 and 0338. **N1a done** (`698fbe78`): amounts from the owner's levels, with no levels on the unit. **N1b done** (`8cd9020f`): the Altar sets the two booleans only. **N3 done** (`63003d0c`): levels apply retroactively, with HP by 0331 **N5 done** (`3197d711`): the Shrine's sides, ABAC, in at B and out at C. | — |
| 0337 **Recruits chosen by kind; the Release order; Eject All stays** (answers Q0128 point 1). The kinds are Plain, Constitution, Metabolism and Both, by augmentation. At pool buildings, Q/W/E/R pick a kind, then KindMenu offers the production options with that kind plus **A: Release**, a free 4-frame queued order whose unit walks out unchanged to the rally; Z goes back. Chop Shop and Airstrip pick a kind too. Tech buildings have A/S/D/F to send out one Recruit of a kind at once, and V for Eject All. The Altar has Release in every menu. Eject All stays everywhere, unreserved units only. HoverCraft is dropped from 0323's list | done for the Armory (F2, `d9492a69`). Queued: N4a (Cathedral), N6 (Chop Shop), N7 (Airstrip), N1b (Altar), N3, G2 and N5 (the tech buildings' buttons). **N1b done** (`8cd9020f`): Release in every Altar menu; Fully augmented offers Release only. **N3 done** (`63003d0c`): the Temple's A/S/D/F, `HoldCommand::ReleaseOne`, shared for the tech buildings. **N4a done** (`aa220af2`): the Cathedral's kind pick and Release, on the Armory's template as data **N5 done** (`3197d711`): the Shrine's A/S/D/F and V. | — |
| 0338 **All Cults stat upgrades are retroactive; the DrillUnit takes the 90° step** (answers Q0128 point 2). The Cults' "not retroactive" exception is removed entirely (T67's locked levels go), and staffing stays their one research exception | done for the DrillUnit (M4-W3, `5305c2f3`). Queued: G2, N3 and the Machine Shop's research apply to existing units, with no "trained after" bookkeeping. **N1a done** (`698fbe78`): augmentation applies to existing units through the accessors. **N3 done** (`63003d0c`): Temple levels apply to existing units | — |
| 0339 **Cults group construction rules** (answers Q0097). Each Recruit goes to the closest part of the footprint and wall-hugs to a free part if it is taken, working within 8 su. The site starts at the first arrival. Recruits of the order on the footprint block it, and a blocked site fails the whole order, refunded once. Progress counts Recruit-frames: each actively building Recruit adds 1 a frame, with no cap. Builders keep building under attack, and Assist targets only buildings under construction | done (M5-E, `62dd3e7f`); Q0130 asks about the order's own Recruits on the ghost | — |
| 0340 **Cults group construction refunds** (answers Q0098): Q0098's (a)s confirmed. A refund only once no Recruit of the group holds the order; a full refund if every builder dies before the start; each Shift-queued building refunded when its last holder drops it; after a cancel, Recruits go on to their queued orders | done (M5-E, `62dd3e7f`) | — |
| 0341 **Recruitment Area details** (answers Q0099). The whole 10×10 area must lie on the map, or the Hall can't be placed. Capacity is rounded down everywhere. Halls completed in the same tick split contested tiles by the nearer centre, then the lower id. A Hall under construction does nothing, and different players' Halls compete | done (F2: points 2–5, `d9492a69`; J1: point 1, `2286859e`). The Verification's "3 tiles" is off by one against the doc (Q0131's note) | — |
| 0342 **Recruitment Hall production** (answers Q0100). The Hall becomes AAAB, its exit on B, with Recruits one at a time through the Exit Doorway. No cancel (X goes). The Recruit in production counts toward LocalUnitControlUsage from its start (19/20 plus one in production shows 20/20). When capacity frees up, production starts from 0 | done (F2 in the sim, `d9492a69`; J1 the ghost's rotation, `2286859e`) | — |
| 0343 **The side at a standing blocker: the side the unit is on** (answers Q0129 with (a), as built). Right first decides only dead centre; moving blockers are unchanged | done (M4-W3, `5305c2f3`); the doc now matches the build | — |
| 0344 **The Cults' global Unit Control is display only** (answers Q0101). Only the Halls' local capacity limits, and Recruits that no Hall produced are orphans from the start | done (C and B built it; F2 dropped the placeholder marks) | — |
| 0345 **Which Storage: the nearest to the patch, chosen at every return** (answers Q0102 with (a) and its addendum). With none, the Recruit idles holding its load; a Storage lost mid-return is replaced by the one nearest the Recruit (0170) | done (M5-L, `d729a418`: the marks cite 0345, and its Verification is a test). A tie between equally near Storages: Q0136 | — |
| 0346 **The Cults economy check** (answers Q0103): edge to edge in whole tiles, ±15%, every figure of the doc with one Storage per face for the ideal; plus a second scenario, the user's build line (one Storage; a second Hall saved for without extra Storages, the new Recruits spread to farther patches; the Hall built by every non-optimal Recruit; then a Storage per face), measured against the GDO and the Colonists at 1:30 and 2:48, its numbers reported with no target | done (M5-L, `d729a418`): 9 of 14 figures pass, 5 pinned out of tolerance, and the realistic opening is at 0.51 by 1:30 and 0.62 by 2:48. Q0135 | — |
| 0347 **Entering pool buildings** (answers Q0104): a dedicated entrance doorway at the middle of the entrance side, and for every training, augmentation and unit-producing building the exit opposite it (the Airstrip excepted). When full, the Enter ends and the unit idles beside the building. Loads are kept inside and carried out, and lost if the Recruit is trained. Buildings under construction can't be entered (a right-click Assists). **The capacity counts reserved units, so the queue never exceeds the capacity**: 10 orders at the Armory, Cathedral and Airstrip, 5 Jackals at the Chop Shop | done (F2: points 3 and 5, `d9492a69`; J1: point 4 in the client, `2286859e`; F3: points 1, 2 and 4, `da08f46d`) | — |
| 0348 **Cults production buildings: leaving units go to the rally in ready order; they are Ungroupable** (answers Q0105 points 3–5). The Armory, Altar, Cathedral, Chop Shop and Airstrip are Groupable false | done for the Armory (F2, `d9492a69`). Queued: N1b, N4a, N6 and N7 set Groupable false. G2 set it in the Altar, Cathedral, Chop Shop and Airstrip defs (`9b7986e4`), so those slices only add behaviour. **N1b done** (`8cd9020f`): the Altar's leaving units go to the rally in ready order. **N4a done** (`aa220af2`): the Cathedral | — |
| 0349 **Evacuation inside the footprint** (answers Q0106): everyone appears at once inside the destroyed building's former footprint. Overlaps resolve by the existing rules (overlapping units ignore each other until separated, and idle units push apart). A unit that had finished training comes out as the trained unit, with no refund. Evacuated units idle | done (F3, `da08f46d`). Queued: N2's Garrison reuses it | — |
| 0350 **The Recruitment Area display** (answers Q0107 with (b)): the Hall's placement preview shows the area, the claimable, own-claimed and non-Recruitable tiles, and the Effectiveness %. The claims of enemy Halls the player can see count as unclaimable, and a selected own Hall shows its claims | done (M5-J2, `e548e6ca`). A selected own Hall's true claims can reveal an unseen enemy Hall: Q0133 | — |
| 0351 **The preview counts your own Halls under construction and placed ghosts** as claimants, in their order of placement; display only, since real claiming waits for completion (0341) | done (M5-J2, `e548e6ca`). Own sites rank before own ghosts, marked Placeholder: the sim keeps no placement time for a site | — |
| 0352 **The preview counts remembered enemy Halls**, from the player's memory in Explored fog, so nothing leaks | done (M5-J2, `e548e6ca`) | — |
| 0353 **Vehicles** (answers Q0119): an idle vehicle doesn't yield, and own idle infantry yield to vehicles; vehicles reverse only on W (Reverse); a point inside the turning circle is reached by a forward loop | queued (V2; N6's Jackal) | — |
| 0354 **Sparrow flight** (answers Q0120 points 2–7): no extra cap on overshooting the map edge; takeoff at IdleSpeed, speeding up along the runway; with no rally, it circles 2 GU past the C end; it strafes out of range and loops back at full speed; Stop circles in place, Hold circles and doesn't chase; the idle circle has a fixed 1 GU radius (IdleSpeed²/64 = 0.25 su/frame²) | queued (H1: the flight rules and the idle circle; N7: takeoff and the circling spot) | — |
| 0355 **Sparrow: less grip**: MaxCentripetalAcceleration 1 → 0.5 su/frame², so the full-speed turn radius is about 200 su; the 1 GU idle circle stays | queued (G2's Sparrow def, or H1 if G2 has run) | — |
| 0356 **The Sparrow stays; the GDO gets anti-air** (answers Q0122). The Rocketman (LightInfantry, HP 45, 1/1, sight 6, speed 3; a missile of Damage 20, Range 6, MinRange 2, 16 su/f, every 32 f; Target Lock R with LockOn 48 f and LockRange 7; 100 SC, 160 f, Barracks W) and the Arsenal it needs (2x2, HP 350, 1/4, Power −20, 150 SC, 320 f), with placeholder numbers. Target Lock can target structures. The Colonists wait until the Cults are done, and the playtest notes name their gap | queued (**M5-K**, new: GDO anti-air, after H1; it also settles 0203's MinRange and 0204's locked fire) | — |
| 0357 **Unseen events, as built** (answers Q0090 with (a) on all three): shots in the shooter's colour even unseen; every player learns of a defeat; late-seen marks show | done (M4-V as built; M5-S1, `09940dab`: every player sees the defeat notice) | — |
| 0358 **Pop-up turret details** (answers Q0091). A turret finishes retracting before it rises again; a Tunnel-built turret needs Buildable, flat and Drillable ground; Tier 1 builds it; every site takes its Tunnel Space from the start. Every expansion of a destroyed Tunnel becomes visible at once. Point 1 (a turret under a unit when its Tunnel dies) is replaced by 0360 | done (points 2–4 as built; point 1 by 0360; M5-Y, `c65cc1b0`) | — |
| 0359 **Only Tunnels build Laser Turrets; the Agent's panel reverts to A: Build Tunnel** (Q Move, A Build Tunnel, S Drop Off, D Repair, X Stop), undoing 0326's Construct menu. Every rule for Agent-built turrets goes (Q0084's payment, the Agent's exit, slot use, the visible site) | done (M5-Y, `c65cc1b0`). Two doc lines still mention Agent-built turrets: Q0132 | — |
| 0360 **Laser Turrets go neutral with their Tunnel, and have no upright form.** A turret is always a pop-up, active only inside its Tunnel's area. When its Tunnel is lost it becomes neutral and visible to all, keeps its high armour (a raised turret retracts at once), never fires, decays 1% MaxHP/s, and a new Tunnel covering it reclaims it (Tier 1 suffices). Turrets no longer keep a Syndicate player alive | done (M5-Y, `c65cc1b0`) | — |
| 0361 **New Cults unit (concept): the missile platform**, a tracked shoot-and-scoot launcher built at the Chop Shop from 3 Recruits. Its turret turns only about 20°, so it must stop and turn to aim. Its missile is DoublyDisjointed (dodgeable), area damage, and can't be shot down. A long aim, flight and reload, with a quick Cooldown. The name, target domain, MinRange and numbers are still open | named the Hand of Heaven by 0363, with numbers: queued (N10) | — |
| 0362 **The site waits for its own Recruits to step off the footprint** (answers Q0130 with (d)). Other units on it still fail the whole order. The fallback, (a) a red ghost, only if (d) proves costly | done (J1, `2286859e`). Q0131 asks whether a Recruit holding the order queued counts | — |
| 0363 **The missile platform is the Hand of Heaven** (0361's concept, now with numbers).<br>**Role and gun:** tracked and ground only. A 20° turret turning 4°/f. A DoublyDisjointed missile with 600 total area damage over 1.5 GU (a siege profile), Range 14, MinRange 4, travelling 8 su/f. Aim 48, Firing 2, Cooldown 4, Reload 96. It reveals on landing.<br>**Body:** 64x40, HP 200, armour 2/2, sight 7, speed 5.<br>**Cost:** 150 SC and 3 Recruits, 480 f, at the Chop Shop | queued (**N10**, new: after N6 and V1). Its base def is in (G2, `9b7986e4`) | — |
| 0364 **The Prayer-mobile, a wheeled caster vehicle; the Jackal is renamed the Seeker.**<br>**The vehicle:** no weapon; built at the Chop Shop from 2 Recruits; needs a Temple; has an energy pool.<br>**Inspire:** own infantry around it move faster, and Recruits also attack faster. It stacks with augmentation, and a recast restarts it.<br>**Suppress:** in a frontal arc, every unit (own and air included) can't use weapon attacks or weapon abilities for a while. A Rocketman keeps its lock but can't fire | queued (**N11**, new: after N6, V2, N3 and K). For the rename, F2 renames the code's few mentions, and the plans read "Jackal" as the Seeker. The Seeker and Prayer-mobile base defs are in (G2, `9b7986e4`) | — |
| 0365 **Prayer-mobile placeholders; Suppress fires along the vehicle's facing**, with no target.<br>**Body:** 56x40, HP 150, armour 1/1, sight 7, Ungroupable, speed 6. Energy 200, starting at 50, +0.5/s.<br>**Inspire (R):** 75 energy; 4 GU; +1.5 su/f MaxSpeed for 10 s; Recruits' attack cycle × 0.75.<br>**Suppress (F):** 100 energy; a 90° arc 5 GU long; units whose centre is inside can't attack for 4 s.<br>**Cost:** 125 SC and 2 Recruits, 400 f | queued (N11). The Prayer-mobile's base def is in (G2, `9b7986e4`) | — |
| 0366 **GDO placeholder pass**: every designed GDO building and unit gets numbers.<br>**Buildings:** Factory, Airfield, Advanced Mechanics, Propulsion Labs, Industrial Powerplant, Radar Station (Scan), Defense System, Supply Tower, Missile Turret, Outpost, Bunker, First Aid Tent.<br>**Units:** Raptor, Twister (with mines), Prospector, Artillery Rig, Hornet, Dragonfly, Drone Operator.<br>**Hotkeys** are placeholders. The only GDO design gap left is the Supply Chopper's transport | **M6**, planned in `artifacts/director/m6-plan.md` (adopted 2026-11-26), after M5. This is the Director's reading of the user's "just continue with the milestones work" (2026-11-26); the user was told. Its questions are Q0153–Q0169 | — |
| 0367 **Syndicate placeholder pass**: every designed Syndicate unit and expansion gets numbers.<br>**Expansions:** Vault, Vehicle Assembly, Forge, Null Chamber, Aerospace Assembly, Orbital Command.<br>**Units:** Assault Bike, Shadebringer, Laser Battery, Subterranean Transport, Phantom, Specter, Prism.<br>Research waits for the techs pass | **M6**, planned in `artifacts/director/m6-plan.md` (adopted 2026-11-26), after M5. This is the Director's reading of the user's "just continue with the milestones work" (2026-11-26); the user was told. Its questions are Q0153–Q0169 | — |
| 0368 **GDO rule confirmations**: the Dragonfly and the Missile Turret home; the Hornet is DoublyDisjointed (dodgeable, can Attack Ground); the drone explodes with area damage; Prospector deployment can't be cancelled; Bunker occupants see with its sight; the First Aid Tent heals own and allied infantry | with 0366 | — |
| 0369 **Syndicate rule confirmations**.<br>**Laser Battery:** Amplify marks the next surge to use the whole battery, and works mobile and deployed.<br>**Shadebringer:** a toggle switches on with any charge; the sludge slow refreshes while standing in it; toggles follow the mode-switch rule.<br>**Subterranean Transport:** holds 4 infantry, loads by right-click, and F unloads all.<br>**Expansions:** the Vault serves 2 at a time; the Null Chamber buffs 4 at a time for 2 s each, the buff ends abruptly, and no research is needed.<br>**Also:** the Specter can be hit during lift-off and landing, and the Hornet's dodgeability is confirmed | with 0366 and 0367 | — |
| 0370 **Doc reorganisation; no rule changed.** Four long files are split:<br>• `systems/controls/selection_and_hud.md` into `selection.md`, `interface.md` and `hud.md`;<br>• `behaviors.md` loses the command behaviours to `command_behaviors.md` and the channels to `action_channels.md`;<br>• `factions/syndicate/tunnels.md` loses the Tunnel's menus to `tunnel_menus.md`, and its Area and Space to `tunnel_area_and_space.md`;<br>• the Recruitment Hall and the Storage move from `factions/cults/economy.md` to `factions/cults/structures/`.<br>Also, the Hall's lost heading is restored | done (2026-11-26). About 1,000 citations were rewritten by script, in 256 files of code and docs, in `36745275`, and in the Director's notes. Comment lines were re-flowed to 120 columns. J1 and Y were told to take master's paths in conflicts | — |
| 0371 **Stale lines fixed** (answers Q0132, Q0138, Q0141). The Agent builds only Tunnels, and a player left with no Tunnel and not defeated exists only in a custom scenario. Fully augmented units leave by Release or Eject All. Barracks E's Unit Control is checked when production starts. Q0131's off-by-one note is confirmed; its question goes to the user | done as built: no implementation impact | — |
| 0372 **A Recruit holding a Build queued counts as one of the order's own Recruits** (answers Q0131). Standing on the footprint, it makes the site wait instead of failing the order | done (S2a, `1801cf59`): the mark cites 0372, and its Verification is a test | — |
| 0373 **A selected own Hall shows its true claims** (answers Q0133), even when an unseen enemy Hall has taken tiles; only the placement preview hides unseen Halls | done as built (J2); S2a (`1801cf59`) updated the code docs | — |
| 0374 **Exits over ground units can't walk on: the unit stays inside** (answers Q0134). Advice for map makers: unwalkable tiles should generally not be Buildable | done as built (Z1); S2a (`1801cf59`) updated the code docs. The optional editor warning isn't planned | — |
| 0375 **The drop-off is chosen by distance from the worker when it finishes mining; ties go to the earlier id** (answers Q0136). For the Recruit it replaces 0345's nearest-to-patch; for the Agent the measuring point is the Agent. Whether Agents re-pick at every return is being asked of the user | done (S2a, `1801cf59`): measured from the worker; the 8-adjacent-patch case is re-measured (in Q0144) | — |
| 0376 **When a turret's lock ends: as built, and Stop clears it** (answers Q0137). A Move keeps the lock, a forced Attack on a friendly unit included | done (S2b, `49dc4b1a`): Stop already cleared the lock; U1's marks cite 0376, and 0376's Verification is a test | — |
| 0377 **A Garrison counts as a threat while an occupant could attack the scanner** (answers Q0139), with the Militant stand-in for Recruits and Disciples. Replaces N2's "never a threat" | done (S2b, `49dc4b1a`): `combat::Threats::of` reads the occupants' weapons as fired from inside; no golden moved | — |
| 0378 **A right-click on a full, damaged own building is Repair** (answers Q0140), fullness counting reserved units at click time. Not full: Enter still wins (0325). Every building that holds units | done (S2b, `49dc4b1a`): `Aimed::full` at click time; `Rule::Enter` gives way to Repair on a full building | — |
| 0379 **A defeated player's neutral units keep their research, frozen at the defeat** (answers Q0142), for every faction's research | done (N3, `adcd806a` in `63003d0c`): `Object::kept_research`; later research slices extend it | — |
| 0380 **The Cults Storage goes back to 3x2** (answers Q0135 in part): ABAB, FullArmor 6; about 2 Recruits per tile of face length | done (S2a, `1801cf59`): 3x2, ABAB, FullArmor 6; the goldens re-blessed and the oracle re-measured | — |
| 0381 **Cults economy check: the realistic opening should overtake ideal mining by about 4:00; measure out to 6:00** (answers Q0135, with 0380). Four lines at 1:30, 2:48, 4:00, 5:00 and 6:00 | done (S2a, `1801cf59`): measured to 6:00, nothing tuned. The realistic opening never overtakes: **Q0144** | — |
| 0382 **Economy comparison metrics:** income/s over the preceding 10 s, total mined, bank, and spent on economy, per line and checkpoint | done (S2a, `1801cf59`): the four metrics, pinned | — |
| 0383 **Everything spent counts as economy, Generators included;** spent is everything spent, net of refunds. Replaces 0382's item list | done (S2a, `1801cf59`): "spent" is everything spent | — |
| 0384 **Agents choose their Tunnel at every return,** nearest crystal hatch to the Agent when it finishes mining; ties to the earlier id. Replaces the choice kept for the Gather | done (S2a, `1801cf59`): Agents re-pick at every return, Supplies included | — |
| 0385 **The Cults economy rerun** (answers Q0144 in part): an 8-patch line mined from both long sides, seating about 32 Recruits; One Hall against Two Halls, both starting with one Storage and adding Storages as needed; the same table to 6:00, with the GDO and the Colonists | done (**M5-S3**, `318234db`). Its table went to the Designer as Q0146, and the oracle is wrapped up (user, 2026-11-26) | Verification: One Hall levels off at 20 mining; Two Halls reaches about 32 and builds about 5–6 Storages |
| 0386 **A hit absorbed by Shroud reveals its attacker** to the targeted object's owner for 1.5 s; the release reveals nobody (answers Q0143) | done as built (N4b, `c0ca0956`). M5-S3 (`318234db`) cites it and tests its Verification (`tests/shroud/reveals.rs`) | — |
| 0387 **A Recruitment Hall can be placed at the map edge**; the area beyond the edge counts as non-Recruitable, out of 100 (at most 70% against an edge). Replaces 0341 point 1 | done (**M5-S4**, `cce50b54`) | Verification: a Hall flush against the edge is accepted, and the preview shows ≤ 70% |
| 0388 **The Deployment Center gets E: Place** in DefaultState: greyed with a filling progress bar while it constructs, lit once ready (enters AwaitingPlacement), hidden while idle. Whether the Defense System gets it too is still being asked | done (**M5-S5**, `d6d1a1a2`) | Verification: order a Generator, E shows it greyed and filling; when done, E then a click places it |
| 0389 **Recruits get E: Attack** (BasicCombatUnitInterfaceState's Attack: an enemy, force-attack on own or neutral, Attack Move on ground). Recruits already have their melee attack (Damage 3) and right-click Attack | done (**M5-S5**, `d6d1a1a2`) | Verification: E on an enemy, they hit it for 3 a swing; E on ground, they Attack Move |
| 0390 **Door arrows on building ghosts**: in at an entrance, out at an exit, and two-way at a shared door (Garrison, Bunker). They turn and flip with the ghost, for every faction | done (**M5-S6**, `5c0cc4e5`) | Verification: an Armory ghost shows in on B and out on C, and rotating turns them; a Garrison ghost shows one two-way arrow |
| 0391 **Faster Cults augmentations**: Enhanced Constitution and Improved Metabolism take 48 frames each, 96 for Both. Costs are unchanged | done (**M5-S5**, `d6d1a1a2`) | Verification: Enhanced Constitution queued in the Altar, the Recruit exits augmented about 3 s later |
| 0392 **Place buttons with progress**: the Extraction Facility's Q: Place Plate is greyed with a filling bar while the plate is built. The Defense System gets the one-slot flow (Q Build or Place Missile Turret, X Cancel), replacing 0366's "constructs like the Deployment Center" | EF half done (**M5-S5**, `d6d1a1a2`). The Defense System half goes with 0366, since it isn't built | Verification: order a plate, Q fills then lights; the same for a Missile Turret |
| 0393 **Shrine buff marks show to everyone** who sees the unit, enemies included (answers Q0145, a) | done as built (M5-N5, `3197d711`) | — |
| 0394 **Door arrows on a selected building**, as on ghosts | done (**M5-S6**, `5c0cc4e5`) | Verification: a selected Armory shows in on its entrance and out on its exit |
| 0395 **Recruitment Halls are groupable**: selected Halls form one SelectionGroup, and their commands, Set Rally Point among them, go to all of them. The other Cults production buildings stay ungroupable (Q0105) | done (**M5-S5**, `d6d1a1a2`) | Verification: box-select 2 Halls and set a rally point, and both send new Recruits there |
| 0396 **Rally points on unfinished buildings**, for every faction: C: Set Rally Point next to Cancel, and a right-click sets it. RallyPoint exists from the start of construction and is kept on completion | done (**M5-S6**, `5c0cc4e5`). The Colonists doc still says "no rally point" while warping in: Q0150 | Verification: start an Armory, select it under construction and right-click ground: a marker shows, and trained units go there after completion |
| 0397 **Attack is always A, and goes to every selected unit**: attackers Attack or Attack Move, others Move (0398). "Common command" means the same command wherever its button sits | done (**M5-S5**, `d6d1a1a2`) | Verification: Militants and a Prayer-mobile, A on ground: the Militants Attack Move and the Prayer-mobile moves with them |
| 0398 **Worker panels**: Recruit Q Move, W Basic Construct, E Advanced Construct, A Attack, S Assist, D Repair, X Stop; Agent W Build Tunnel, A Attack; Field Engineer W Construct; Drone W Deploy. Units that can't attack treat Attack and Attack Move as Move | done (**M5-S5**, `d6d1a1a2`) | Verification: Recruits W, E and A; Drones with Militants, A on ground: the Militants Attack Move and the Drones move |
| 0399 **A shows whenever any selected group can attack**, whichever group is in focus, and is hidden only when no selected unit can attack (replaces 0398's hidden A) | done (**M5-S5**, `d6d1a1a2`) | Verification: Drones and Militants with the Drones in focus show A; Drones alone don't |
| 0400 **Right-click in multi-stage menus** does the selection's normal right-click and leaves the menu open (BuildMenu, ExpandMenu, EjectMenu, KindMenu, AugmentMenu). In AwaitingTarget and AwaitingPlacement it cancels straight to DefaultState, and a ready building stays ready. Escape and Z still go back one step | done (**M5-S5**, `d6d1a1a2`) | Verification: Recruits in Basic Construct, right-click ground, they move and the menu stays; right-click in placement goes to DefaultState; Escape in placement goes back to the build menu |
| 0401 **The Economy check is closed** (answers Q0146, a): the Cults', GDO's and Syndicate's economy targets stay in the docs as reference for playtests, and the Director measures nothing more against them. The oracle stays as an on-demand tool whose pins aren't maintained, and the pairings tests stay | done (**M5-P3**, `8eaf316e`): the oracle's tests are ignored and its docs cite 0401 | — |
| 0402 **Computer players are a playtest tool, with fair knowledge** (answers Q0147: fair, a). They see only what their own objects see, remember what they have seen, and know the map and the start locations. They use a player's commands and rules, and their behavior is the Director's placeholder (`core/game_rules.md > Computer players (playtest tool)`) | done for all four factions: the GDO (**A1**, `168dadc0`), the Syndicate (**A2b**, `73da4292`), the Colonists (**A2c**, `157d6c28`) and the Cults (**A2a**, `946d6dff`). Its Verification is checked every tick in `crates/ai/tests/games.rs`. **A2d** adds the rush profile and makes three economies spend their income | Verification: in an AI-vs-AI game, an AI doesn't attack a building it has never seen or scouted, apart from the known start locations |
| 0403 **The Sparrow's SightRange is 8 GU** (answers Q0148 part 1), so a lone Sparrow keeps its target in sight through its loop. Attack Move is unchanged, and what Attack Move means for units that can't stop stays an open design question | done (**M5-H2**, `b825b923`) for targets up to a Peacekeeper's size. Bigger targets are lost after 1–4 passes: Q0149 | Verification: a lone Sparrow ordered to Attack a stationary target, with no spotter, makes repeated passes until it dies |
| 0404 **A glider's Move completes when it first reaches its point** (answers Q0148 part 2). The next queued order starts; with none, it circles there as when idle | done (**M5-H2**, `b825b923`) | Verification: Shift-queue Move A then Move B for a Sparrow: it passes A, flies on to B and circles at B |
| 0405 **Gliders push apart slowly, at a quarter of the HoverCraft strength, and their SeparationRadius exceeds their silhouette** (answers Q0148 part 3): Sparrow and Dragonfly 48, Specter 64 | done (**M5-H2**, `b825b923`): gliders at a quarter, and the Sparrow's radius is 48. The Dragonfly's and the Specter's radii wait for their defs (0366, 0367). The drift takes 11–17 s, not "a few seconds": Q0152 | Verification: six Sparrows sent to one point drift apart while circling and end about 48 su apart; two Sparrows passing on one target stay on their lines |
| 0406 **A portal under Warp In can have its rally point set** (answers Q0150, option a): the Colonists doc now carries 0396's exception, and a Beacon under deployment keeps only Cancel. Doc fix | done (as built, **M5-S6**, `5c0cc4e5`; tests `warp_in/rally.rs` and `commands/colonists/warp_in.rs`) | Verification: a warping Bio Portal shows X Cancel and C Set Rally Point, and a right-click sets the rally; a cancelled Warp In drops it; a deploying Beacon shows only Cancel |
| 0407 **A building in focus shows the selection's Attack on its free A** (answers Q0151 part 3, option a; from 0399's "whichever group is in focus"). No doc change | done (as built, **M5-S5**, `d6d1a1a2`); its Verification test is folded into **M5-K** | Verification: select a Barracks and some units, Tab to the Barracks: A: Attack shows, and A + click sends the units to attack |
| 0408 **A glider's strafing pass ends when the target's centre is out of weapon range** (answers Q0149, option c), measured from the glider's edge to the target's centre. Firing and every other range check stay edge to edge, so the loop no longer grows with the target | done (**M5-N7**, `ce77e205`) | Verification: a lone Sparrow with no spotter keeps making passes on a Seeker, a Guard and a building until each dies; its loop's farthest point is about 500 su from the target's centre for a Peacekeeper and a building alike |
| 0409 **Shielded Peacekeepers follow an explicit Attack's target** (answers Q0151 part 1, option a): "Attack goes to everyone" covers them; the held Attack resumes once the shield is lowered | done (as built, **M5-S5**, `d6d1a1a2`; tests `commands/shield_orders.rs` and `riot_shield/orders_kept.rs`) | Verification: shielded and unshielded Peacekeepers given A + click on an enemy all head for it, the unshielded fire, and the shielded attack once lowered |
| 0410 **Shift-, Ctrl- and double-click work on own buildings** (raised by the user: Shift-click wouldn't add a second Hall): Shift-click toggles; Ctrl- or double-click takes every own building of that type on screen, an unfinished one counting as another type; Ctrl+Shift-click adds them. Plain clicks and boxes are unchanged | done (**M5-N7**, `ce77e205`) | Verification: Shift-click a second Hall adds it and again removes it; double-click a Hall with three on screen, one unfinished, takes the two finished; Recruits plus a Shift-clicked Storage make two groups; Ctrl-click an Armory with two on screen takes both, each its own group |
| 0411 **A box still selects one building, groupable ones included** (answers Q0151 part 2, option a). 0395's "Box-select 2 Halls" becomes a double-click | done (the box as built; the Verification test in **M5-N7**, `ce77e205`) | Verification: a box over 2 Halls selects one; a double-click on one selects both as one group, and both use a rally point set then |
| 0412 **Gliders push apart at a third of the HoverCraft strength** (answers Q0152), up from a quarter (0405), the same in every activity. Spreading is expected to take roughly 10 s or more | done (**M5-N7**, `ce77e205`): six Sparrows within 10 % of 48 su in 11.4 s, against 13.4 s at a quarter | Verification: six Sparrows sent to one point drift apart while circling and end about 48 su apart, somewhat faster than at a quarter; two Sparrows side by side on one target still land near the centre |
| 0413 **Full screen by default; the cursor stays in the window; an 8 px pan margin** (the user's playtest finding: in a window, panning up slid the cursor into the title bar). Borderless full screen at start; F11 or Alt+Enter toggles a window (keys placeholder); the cursor is confined to the window while the game is focused, full screen included; edge panning starts within 8 px of the edge (placeholder) | queued (**M5-S7**, a small slice of its own, in the first free slot after K) | Verification: launch fills the screen with no borders; F11 or Alt+Enter toggles; in a window the cursor stops at the top edge and the map keeps panning; Alt-Tab frees it; 5 px from an edge pans; on two monitors the cursor stays on the game's |
| 0414 **A temporary playtest menu: Restart and Quit**, on F10 and at game end (the user: "when the game ends or if I press F10 I would like restart (and quit) options"). Restart replays the same map, slots, factions and computer players from 0:00; the F10 menu pauses the game and offers Resume; no confirmation (placeholders). The Director's call, as 0414 asks: a restarted game starts a new replay | queued (**M5-S7**, with 0413; brief `scratchpad/briefs/m5s7-window-and-playtest-menu.md`) | Verification: F10 mid-game shows the menu with units frozen, and F10 or Resume continues; Restart starts the same game again at 0:00; Quit closes it; a won or lost game shows the menu with the result |
| 0415 **The Minimap**, which the user asked to be prioritised (`systems/controls/hud.md > Minimap`): the whole map with simplified terrain; objects in player colours (buildings as footprints, units as dots), patches and neutrals in a neutral colour; the fog exactly as on the main map; the camera's view as a rectangle. Left-click or drag centres the camera; right-click is the selection's ground right-click there; in AwaitingTarget a left-click targets that ground point. It targets only ground and never places (placeholders); attack alerts aren't decided | done (**M5-MM**, `b5673277`; attack alerts left out, as 0415 leaves them undecided) | Verification: the whole map, player colours, neutral patches; enemies only where seen, remembered buildings as last seen, unexplored black; the rectangle follows the camera; a click moves the camera and a drag pans; a right-click moves the selection there; A then a left-click Attack Moves there |
| 0416 **The map starts Explored by default** (`core/vision.md > Starting exploration`): every tile Explored for every player, with the terrain and the neutral objects remembered as at game start and no player objects; the fog stays on. A command-line flag turns it off; Restart keeps it | queued (**M5-S7**, with 0413 and 0414; a `SimConfig` option that old replays read as off) | Verification: with no flag the map shows dimmed terrain and patches but no enemy base; with the flag it is black outside sight; a patch mined out of an enemy's sight keeps its starting amount in that enemy's memory; a restart keeps the setting |
| 0417 **Tracked vehicles turn by a tread law** (answers Q0153 parts 1 and 3): SpinRate = 2 × MaxSpeed ÷ TrackWidth when stopped, falling linearly to 0 at MaxSpeed (maxTurnRate = SpinRate × (1 − speed ÷ MaxSpeed)); reversing uses the same law, capped at a new ReverseMaxSpeed; turning costs no speed. TrackWidth replaces SpeedToTurnRadiusRatio, and the five tracked units get TrackWidth and ReverseMaxSpeed; the surfaced Transport takes the Laser Battery's Acceleration and Deceleration | queued (folded into **M5-V3**'s brief, which replaces T85's linear law and its corner choice) | Verification: a Hand of Heaven standing still turns 90° in about 5 frames; at full speed, ordered to a point 90° to its side, it slows first and never turns at MaxSpeed; its tightest radius is about 15 su at half speed and 60 su at 4 su/frame; reversing, it backs up at no more than 2.5 su/frame and turns by the same law |
| 0418 **A crusher turning in place crushes the crushable enemies its turning Silhouette sweeps** (answers Q0153 part 2, option b): the crush trigger becomes an overlap while Moving, Reversing or Stationary + Turning; own and allied crushables still block the spin, and a stopped crusher that isn't turning is still a hard obstacle | queued (**M5-V4**; it replaces V4's placeholder that a spin doesn't crush) | Verification: an enemy Light Infantry at a stopped Hand of Heaven's side dies when the tank turns to face it and the hull sweeps it; an own Recruit stops the turn at contact and nobody dies; a stopped, non-turning tank can't be walked into |
| 0419 **The surfaced Subterranean Transport crushes** (answers Q0153 part 4, option b): a DrillUnit above ground crushes like a TrackedVehicle, the spin rule (0418) included; underground it never crushes | queued (**M5-V4** adds the surfaced DrillUnit to the crushers; the Transport itself is **M6-ST1**'s) | Verification: a surfaced Transport driven through enemy Recruits kills them, and own infantry block it; burrowed, it passes under enemy infantry with no effect |
| 0420 **The HoverCraft aircraft move by DragMovement, with numbers of their own** (answers Q0154, option c, from the docs and the user's standing permission on placeholders): high thrust and high drag in the Supply Chopper's pattern, stopping by drag. Forward, Omni, DragRatio, top speed and TurnRate: the Hornet 0.8, 0.1, 0.1, 9, 8°; the Phantom 0.78, 0.3, 0.12, 9, 10°; the Prism 0.05, 0.05, 0.1, 1, 4°, its thrust charges staying separate bursts; the Operator's drone 0.9, 0.1, 0.1, 10, 10°. It replaces placeholder (a), the given numbers mapped | queued (**M6**: GK, SK, GD, GH1, SD and SP, as Q0154 lists) | Verification: a Hornet reaches about two thirds of top speed within about 1 s of thrust, and on arrival drifts to a stop over about 1.4 GU; a Phantom starts, stops and sidesteps visibly quicker than a Hornet; a Prism creeps at most 1 su/frame |
| 0421 **The Drone Operator's details** (answers Q0155): the Barracks' R: Build Drone Operator (100 SC, needing a Career Campus); the drone is a HoverCraft on 0420's numbers, Unit Control 0, PointArmor 0 and FullArmor 0, Groupable; Build Drone can start only once the operator's drone has ended (detonated, fallen, out of battery or lost to the leash); launching doesn't change the selection; the leash is measured edge to edge. It replaces the placeholders for points 3 (buy at once) and 5 (centre to centre) | queued (**M6**: GK and GD) | Verification: a Barracks with a Career Campus queues a Drone Operator on R; after a launch the operator stays selected, and Build Drone (F) is greyed until the drone ends, then buyable (10 s); a drone flown just past 8 GU from the operator's edge falls and explodes; drones count no Unit Control, and two selected together form one group |
| 0422 **Radar Stations are Groupable false** (answers Q0156, option b): with several selected each forms its own SelectionGroup, so one Scan press spends one station's 60 s cooldown; Tab moves to the next | queued (**M6-GR**) | Verification: two selected Radar Stations make two groups; Q and a click scans from one, which cools down; Tab, Q and another click scans from the second |
| 0014 GDO Airfield and aircraft | queued (Later: air) | |
