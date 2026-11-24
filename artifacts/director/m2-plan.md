# M2 plan: GDO base and economy (planning spike M2-P)

> **Status (2026-11-23):** accepted by the Director.
> - Technical decisions T1–T13 are accepted as recommended.
> - The questions were filed as `artifacts/director/questions/Q0022`–`Q0030`. Q0024 point 3 was reworded after 0071.
> - In `plan.md` the slices are named M2-A to M2-J.

- **Date**: 2026-11-23. Read-only spike: no repository file was changed.
- **Code read at**: `91f9ee5` (master). M1-G3 (cleanup, file splits, golden re-bless) is still running.
- **Sources**:
  - `artifacts/director/plan.md`: M2 under Queued, the change table, the open questions.
  - Change entries 0004, 0005, 0007, 0008, 0009, 0012, 0013, 0038, 0040, 0041, 0042, 0048, 0057.
  - `ARCHITECTURE.md` and `docs/movement.md`.
  - The design docs cited below. Paths are relative to `artifacts/designer/design/`, written as `file.md > Section`.
- **"Silent"** means the design says nothing about the point. The plan then proposes a default and drafts a question (section 5).

---

## 0. Summary

- **Scope.** M2 covers:
  - five GDO structures: Deployment Center (DC), Generator, Barracks, Extraction Facility (EF), Extraction Plate;
  - Space Crystals, power, and Unit Control;
  - plate mining;
  - construction and placement, with the GDO build area;
  - Peacekeeper production with rally points;
  - path knowledge (0040–0042);
  - Starting Conditions and Starting Layouts (sim and editor);
  - defeat;
  - the HUD and command panels for structures.

  **Deferred:** the Supply Tower and Supply Chopper (air movement, a TBD Radar Station prerequisite, nothing to spend supplies on), tech prerequisites, the other GDO buildings and units, and Custom maps.
- **Slices.** There are 12, in 6 waves after G3:
  1. **A**
  2. **B** ∥ **C** ∥ **E** (Fable)
  3. **D1** ∥ **D2** ∥ **F** ∥ **G**
  4. **H1** ∥ **I**
  5. **H2** (gives the first playable)
  6. **J** (QA)
- **First playable** after A, B, C, D1, D2, F, G, H1, H2: on `dev_skirmish` you start with the GDO base, plate up, build Generators and a Barracks, train Peacekeepers to a rally point, and lose when your last building dies.
- **Questions.** Nine are drafted (Q0022–Q0030). Only **Q0022** (supplies) blocks, and only the supply part, which this plan defers. Everything else has a stated default.

---

## 1. Scope

### 1.1 In M2

| Area | Content | Design |
|---|---|---|
| Structures | DC, Generator, Barracks, EF and Extraction Plate, each with:<br>• stat block (size, SymmetryType, HP, armour, sight, Groupable, BuildRadiusExtension (BRE), power)<br>• orientation and footprint<br>• blocking, vision and fog records<br>• combat target with PointArmor | `factions/gdo/structures.md > Barracks`, `> DeploymentCenter`<br>`factions/gdo/economy.md > Generator`, `> ExtractionFacility`, `> ExtractionPlate`<br>`core/entities.md > Structure Type`, `> Structure Instance`<br>`systems/units.md > UnitCollision` (ground units collide with structures)<br>`systems/combat.md > ValidTarget` |
| Resources | Space Crystals bank; Supplies counter (always 0, Q0022); power (current/total, penalty ratio); Unit Control (hard cap 200) | `factions/gdo/overview.md > GDOResources`, `> DisplayHud` |
| Mining | 10 crystals per 48 frames, then a residual 1 per 48 frames; power penalty; on destruction a patch with crystals is uncovered and a depleted patch is removed | `factions/gdo/economy.md > ExtractionPlate`<br>`core/map.md > SpaceCrystalsPatch`<br>0038 |
| Construction | DC builds the Generator (100 SC / 240 frames), Barracks (200 / 160) and EF (200 / 320). The EF builds the plate (75 / 400). Both use CurrentConstruction → ReadyToPlace. Cancel refunds in full while constructing and 75% (rounded down) once ready. | `factions/gdo/structures.md > DeploymentCenter > Constructs`, `> Cancellation`<br>`factions/gdo/economy.md > ExtractionFacility > Constructs`, `> Cancellation` |
| Placement | Direct Placement: every tile Visible, Buildable and flat, no structure overlap, inside the GDO build area.<br>GDOBuildArea.<br>A plate goes on a patch that has no plate.<br>Ghost with rotate/flip and side labels, plus the area overlay. | `core/entities.md > Placement Validation > Direct Placement`<br>`core/map.md > Elevation > Structures only on flat terrain`<br>`factions/gdo/structures.md > GDOBuildArea`, `> DeploymentCenter > ObjectInterfaceState[DeploymentCenter]` (AwaitingPlacement)<br>`factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`<br>0004 |
| Production | Barracks: Peacekeeper (50 SC / 80 frames), queue of at most 20, CancelLast refunds in full, RallyPoint, Unit Control check, power penalty | `factions/gdo/structures.md > Barracks`<br>`factions/gdo/units.md > Peacekeeper > UnitControlCost` |
| Interface | DC, EF and Barracks panels; BuildMenu; AwaitingPlacement; AwaitingTarget[SetRallyPoint]; the production building's right-click; pointers; HUD resources; structure InfoPanels | `systems/controls/selection_and_hud.md > CommandPanel` (`> Standard Slot Assignments`, `> Production Building Default Right-Click`), `> PointerDisplayType > Resolution Rules`, `> HUD Bar`<br>`factions/gdo/overview.md > DisplayHud` |
| Path knowledge | Paths are planned only from what the owner knows; units re-route on discovery; map resources are known from the start | `systems/controls/behaviors.md > PathKnowledge`<br>0040, 0041, 0042 (plus the 0038 tests) |
| Start and end | Starting Conditions (DC, EF, 2 plates, 100 SC, power +9); Starting Layouts (sim rules and editor: overlay, Skirmish Default restriction, completeness, outdated flag); camera on the DC; defeat | `factions/gdo/economy.md > Starting Conditions`<br>`core/map_editor.md > Starting Layouts`, `> Map Types > Skirmish Default`<br>`core/camera.md > Starting Position`<br>`core/game_rules.md > Defeat Condition`<br>0008, 0009, 0012 |

### 1.2 Deferred

| Item | Why |
|---|---|
| **Supply Tower and Supply Chopper**. `plan.md` lists "GDO supplies" under M2. | • The Tower's TechPrerequisite is a Radar Station, and the Radar Station has "Full stats TBD" (`factions/gdo/structures.md > Radar Station`).<br>• The Tower's own cost and time are TBD (`> DeploymentCenter > Constructs`).<br>• The Chopper is an acceleration-driven air HoverCraft with no MaxSpeed (`factions/gdo/units.md > SupplyChopper > UnitBaseAttributes[HoverCraft]`). Air movement (0043–0047) and the acceleration model (0035) are scheduled for Later.<br>• Its transport is TBD (`> Cargo`).<br>• Starting Supplies are TBD and nothing GDO builds costs supplies yet (`factions/gdo/economy.md > Starting Conditions`).<br>→ **Q0022**. M2 shows Supplies 0, and stations keep their M1 behaviour (0057). |
| Tech prerequisites | No M2 kind has one. The first prerequisites gate TBD kinds: Arsenal for the Rocketman, Radar Station for the Supply Tower (`core/tech_and_research.md > Tech Prerequisites`). They land with the first such kind. |
| Other GDO buildings (Arsenal, Career Campus, Factory, Radar Station, Advanced Mechanics, Airfield, Industrial Powerplant, Propulsion Labs, Defense System, Missile Turret) | Costs, times and stats are TBD, and so are their BuildMenu entries (`factions/gdo/structures.md`; `> DeploymentCenter > ObjectInterfaceState`: "build menu entries TBD"). |
| Rocketman, Drone Operator, Field Engineer (Bunker, First Aid Tent), Prospector and Outpost | Stats and costs TBD (`factions/gdo/units.md`). The Outpost is the only other seed of the build area, so M2's area comes from the DC alone. |
| ConstructionHP | GDO buildings are built inside the constructor and placed whole. The rule applies only to structures that reference it (`core/entities.md > Structure Type > ConstructionHP Rule`). |
| Worker-Built and Warp In placement | These are M3 (Colonists) and M4 (Syndicate). The plan's flatness bullet covers all three methods, but M2 only has Direct Placement. |
| Other factions' Starting Conditions objects | Tunnel and Agents, Recruitment Hall and Recruits, Robo Portal and Drones arrive in M3–M5. Their layouts can't be filled until then (see T11). |
| Custom maps | "Not designed yet; deferred" (`core/map_editor.md > Map Types > Custom`). |
| Minimap, notifications, final HUD | Bookmarked for the HUD pass (`systems/controls/selection_and_hud.md > InfoPanel with 2+ objects selected`, `> SelectionPanel`). |

### 1.3 What already exists (so the slices don't redo it)

**Sim**
- **Orientation.** `object.rs` has `Placement { origin, w, h (after rotation), rotation, flip_h, flip_v }`. `BuildingOrientation` and `Orientation::of_building` flip first in the building's own frame and then rotate clockwise. `locate_oriented` swaps w and h at 90° and 270°.
- **Flatness.** `ObjectDef::needs_flat_ground` is checked by `scenario/validate.rs::place_object`.
- **Map format v3 already holds layouts**: `Slot::layouts: StartingLayouts { gdo, syndicate, cults, colonists: Vec<LayoutObject { kind, at, orientation }> }`.
  - `Scenario::start_objects` spawns only the picked faction's layout.
  - It flags `LayoutOverlap`, including overlaps with resources, so plate-on-patch must be exempted.
  - Symmetric maps regenerate slots with `slot_image` (`scenario/generate.rs`).
  - `Sim::preview` already builds layouts for the editor.
- **Nav.** Structures and resources block, via `paths::blocking_footprint` by Category.
  - The grids are rebuilt in full in 5b and again after the 5e removals.
  - `ROUTE_ON_TRUE_MAP` is const-asserted in `paths.rs` (comment: "per-player grids aren't built (question Q0011)").
  - `nav_version` is hashed.
- **Vision memory.** There is a `LastKnown` record for each seen tile-based object that the player doesn't own. The hooks for 0040 are `records_revision()`, `record_tiles()` and `is_record_tile()`. The `Footprints` index is `BTreeSet<(tile, ObjectId)>`, so it already allows two objects on one tile (plate plus patch). Map-placed resources are not seeded as records (0042 needs its own starting set; `plan.md`, M1-F).
- **Combat.**
  - `orders/world.rs::pick` accepts any destructible enemy object.
  - `combat/targeting.rs` gives non-units the Ground domain and armour 0, because armour lives on `UnitDef`.
  - Dead objects leave through `remove_objects`, followed by `paths::sync`.
- **Commands.** `DebugCommand::Spawn { kind, owner, at: SpawnAt }` has **no orientation**. `testing::every_command` fails to compile when a new `Command` variant isn't covered.

**Client**
- `BoxTier::{OwnBuildings, EnemyBuildings}` exist.
- `InterfaceTemplate::of` returns `None` for non-units ("objects without a template do nothing").
- `command_ui/interface.rs` has `UnitCommand` and `InterfaceState { DefaultState, AwaitingTarget(UnitCommand) }`.
- The top bar shows the player and the tick.
- Editor: the Place tool refuses owned objects on Skirmish Default maps (`PlaceProblem::OwnedOnSkirmishMap`). There is no layout editing. Code comment: "Every tile-based type has a square footprint so far".

**Files near or over the ~500-line guideline** (`ARCHITECTURE.md > Code conventions`; G3 may change these):

| File | Lines |
|---|---|
| `sim.rs` | 552 |
| `vision.rs` | 583 |
| `paths.rs` | 502 |
| `command_ui/interface.rs` | 649 |
| `command_ui/resolution.rs` | 537 |
| `input/bindings.rs` | 644 |
| `cli.rs` | 764 |

M2 code should go into new modules.

---

## 2. First playable

**What the player can do** (after A, B, C, D1, D2, F, G, H1, H2):
1. `cargo run` starts GDO against GDO on `dev_skirmish`, with each player's GDO Starting Layout.
   - The camera starts on the player's DC (`core/camera.md > Starting Position`), unless the map sets a camera start.
   - The top bar reads `Space Crystals 100 | Supplies 0 | Power 9/30 | Unit Control 0/200` (`factions/gdo/overview.md > DisplayHud`; the "/30" reading is Q0025). Text is ASCII only, so the minus sign is written `-`.
2. **EF.** Select the EF and press Q to build a plate (-75). The InfoPanel shows progress. After 25 s, Q Place Plate shows a ghost that is green over an uncovered patch inside the overlay. Click to place it: power drops to 6, and income grows by 10 every 3 s.
3. **DC.** Select the DC, press Q, then pick the Generator (-100). After 15 s it is ready; select it to enter AwaitingPlacement.
   - The ghost is red on slopes, in fog, or outside the area, and green on flat Visible tiles inside the area.
   - R and Shift+R rotate it; F and Shift+F flip it.
   - Click to place: power +20, and the area grows by 1 around the Generator.
4. **Barracks.** Build two Generators, then a Barracks. On the clean line it starts at about 48 s (Q0028). R turns its footprint, and the side labels show the exit side B.
5. **Peacekeepers.** On the Barracks, press Q five times: 5 Peacekeepers at -50 each, one every 5 s.
   - Right-clicking the ground sets the rally point and shows an indicator (Q0024).
   - The units leave from side B and walk to it. Unit Control reads 5/200.
6. **Defeat.** Switch to the other player with F2 (M1 debug switch), or debug-spawn enemies. Destroying all of a player's structures, plates included, shows Defeated or Victory (Q0029).

**Not needed for the first playable:**
- **E**: units keep routing on the true map until E lands.
- **I**: the Director can write `dev_skirmish`'s GDO layout in JSON with F's validation; the plan says the Director authors the test map's layouts.
- **J**.

**Optional earlier checkpoint** (A, B, D2, G, H1, with a code scenario of a pre-placed Barracks and plates): train, rally, and watch crystals rise. Useful only if the user wants an early look.

---

## 3. Slices

### 3.1 Overview

| # | Slice | Content | After | Model |
|---|---|---|---|---|
| A | Structure foundation | 5 structure kinds with stat blocks, armour, orientation and sides, blocking, vision, records, combat, placeholder visuals, debug spawn with orientation | G3 merged | Opus |
| B | Economy core | banks, derived power and penalty, derived Unit Control, plate mining and depletion, economy phase | A | Opus |
| C | Build area and placement validation | one pure check shared by the gate, the ghost and layouts | A | Opus |
| D1 | Construction | DC and EF state machines, Construct, Cancel, CancelReady, Place, refunds | B, C | Opus |
| D2 | Production and rally | Barracks queue, Train, CancelLast, SetRally, exit spawn, Unit Control | B | Opus |
| E | Path knowledge | per-player knowledge grids for planning, the true map for collision, re-route on discovery | A | **Fable** |
| F | Start and end of a game (sim) | Starting Conditions, layout rules, restriction, completeness and outdated flag, game-start banks, camera, defeat; proposed `dev_skirmish` GDO layout | B, C | Opus |
| G | HUD: resources and structure info | resource bar, structure InfoPanel, labels, defeat and victory banner | B (D1 and D2 for progress lines) | Opus |
| H1 | Production UI | generalised command UI; Barracks panel; production right-click; rally targeting and indicator | D2 | Opus |
| H2 | Construction UI | DC and EF panels, BuildMenu, AwaitingPlacement ghost, overlay, R/F, side labels, Place | C, D1, H1 | Opus |
| I | Editor: Starting Layouts | slot and faction picker, overlay, restricted palette, missing list, outdated flag, M0-E panel leftovers | F | Opus |
| J | M2 QA | economy oracle, `gdo_opening` golden, QA screenshot set, structure perf, playtest build | all | Opus |

### 3.2 Waves and parallel tracks

```
W0  M1-G3 merged (mover split, goldens re-blessed)
W1  A
W2  B | C | E (Fable; own track, may run into W4)
W3  D1 (after B, C) | D2 (after B) | F (after B, C) | G (after B)
W4  H1 (after D2) | I (after F)
W5  H2 (after C, D1, H1)  -> first playable
W6  J (after everything, E included)  -> user playtest
```

**Shared files in W3.** D1, D2 and F all add commands, gate rules and `every_command` arms, so expect small merge conflicts. If the Director wants zero conflicts, run D2 after D1; that costs one wave.

**What each slice owns** (for the "don't touch" lists):

| Slice | Owns | Read-only |
|---|---|---|
| A | `defs/` | — |
| B | `player.rs`, `economy/`, the economy phase in `sim.rs`, the plate/patch removal hook | — |
| C | `placement/` (new) | everything else |
| E | `paths*`, `nav*`, the truth-grid switch in `movement*`, `orders/destinations.rs`, `mover*` | — |
| D1 | `constructor.rs` and its commands | — |
| D2 | `production/`, its commands, rally references in `sim/removal.rs` | — |
| F | `scenario/`, `starting.rs`, `economy/defeat.rs`, `maps/`, `camera.rs` | — |
| G | `hud/` | — |
| H1, H2 | `command_ui/`, the new client `placement/`, the bindings in AwaitingPlacement | — |
| I | `editor/` | — |

Every slice follows the Definition of done (`ARCHITECTURE.md > Definition of done`):
- clippy, fmt and tests are clean;
- doc comments cite the design;
- `ARCHITECTURE.md` is updated for new conventions: Object model, Tick pipeline, a new "Economy" section;
- goldens are re-blessed once after merging master, with each change explained (`> Replays and golden hashes`).

### 3.3 Slice details

#### A: Structure foundation (Opus)

**Goal.** The five GDO structure kinds exist in the sim with their full stat blocks. They can be created from scenarios, by debug spawn and through layouts. They block ground units, give their owner vision, appear in enemies' fog memory, can be attacked and destroyed (armour applies), and are drawn and selectable in the client. No economy yet.

**Design**
- Stat blocks:
  - `factions/gdo/structures.md > Barracks` and `> DeploymentCenter`;
  - `factions/gdo/economy.md > Generator`, `> ExtractionFacility` and `> ExtractionPlate`.
- `core/entities.md`:
  - `> Object Type > Size - Height x Width`;
  - `> Structure Type`, including SymmetryType and the 8 orientations;
  - `> Structure Instance`.
- `core/map.md > Elevation > Structures only on flat terrain`.
- `systems/combat.md > ValidTarget` (PointArmor for single-target hits).
- `core/vision.md`: structures are sight sources, take the ElevationModifier per 0004/0005, and are recorded in Explored areas.
- `systems/units.md > UnitCollision > Ground Collision`.

**Code**
- **Kinds.** In `defs/gdo.rs` and `defs.rs`, add five `ObjectKind` variants and add them to `ALL`.
- **`StructureDef`.** Add `ObjectDef::structure: Option<StructureDef>` with:
  - `symmetry`;
  - `build_radius_extension`;
  - `power: i32`;
  - `groupable` (already on `ObjectDef`).
- **Armour.** Move armour to a shared `Armor { point, full }` on `ObjectDef`, used by both units and structures. Otherwise `combat/targeting.rs` keeps 0 for structures.
- **`defs_are_consistent`.** Require that `Category::Structure` ⇔ `structure.is_some()`, that AAAA ⇒ square, and that every value cites its design section.
- **Sides.** Add a new pure `sides.rs` with `sides(kind, orientation) -> [Side; 4]` for N, E, S and W, using the placeholder convention of Q0023. D2 uses it for the exit and H2 for the labels.
- **Debug spawn.** `DebugCommand::Spawn` gets `orientation: BuildingOrientation` with `#[serde(default)]`, so M1 replays still load.
- **Plate on a patch.** Allow it in `Scenario::start_objects`, scenario validation and the debug-spawn gate: a plate may share its tile with an uncovered patch, and only that pair may overlap.
- **Client**:
  - placeholder meshes in `object_render.rs` (a footprint rectangle in the owner's colour plus a 2-letter tag);
  - `hud/selection_panel.rs` labels;
  - `editor/ui/fields.rs` match arms;
  - `selection/pick.rs`: a structure wins over a resource on the same tile (Q0028 point 4);
  - `InterfaceTemplate::of` stays `None` until H1 and H2.

**Acceptance**
- **Defs.** Each kind spawns with its def values. The Barracks footprint is correct for all 8 orientations: w and h swap at 90° and 270°, and flips keep the size.
- **Nav.** Structures block the `(GroundRugged, 1)` tiles of their footprints. A Peacekeeper's path goes around a Generator.
- **Combat.**
  - A Peacekeeper attacking a Generator deals its damage minus PointArmor 1. Plates take damage minus 2.
  - At 0 HP the Generator is removed, its tiles reopen, and `nav_version` bumps.
- **Vision.** A DC gives sight 6, a Generator 3 and a Barracks 4, with the ElevationModifier. An enemy Barracks seen and then fogged stays as a record.
- **Plate on a patch.** A scenario with a plate on a patch is valid: both objects exist, the tile is blocked, and the vision index holds both. A Generator over a patch is refused (Q0027 point 3).
- **QA screenshot.** A dev scenario with one of each kind for two owners.
- **Goldens.** No golden change is expected. If one changes, explain it.

**After**: G3 merged. **Model**: Opus. The work is wide but shallow, mostly data.

#### B: Economy core: banks, power, Unit Control, mining (Opus)

**Goal.** Each player has a Space Crystals and Supplies bank. Power, the penalty ratio and Unit Control are derived from the objects. Plates mine their patch, with the residual rate and depletion. A plate's destruction follows the design rules. The economy phase is added to `Sim::step`.

**Design**
- `factions/gdo/overview.md > GDOResources`: Space Crystals, Supplies, Power, Unit Control.
- `factions/gdo/economy.md > ExtractionPlate`: MiningRate, ResidualMiningRate, PowerPenalty, On destruction.
- `factions/gdo/economy.md > Starting Conditions`: 100 SC; supplies 0 as a placeholder (Q0022); power +9. Starting banks are set in F; B only provides `DebugCommand::SetResources`.
- `core/map.md > SpaceCrystalsPatch`.
- 0038: a removed patch reopens its tile.

**Code**
- **`player.rs`.** `Player { crystals: u32, supplies: u32 }`, hashed and serialized with `#[serde(default)]`.
- **New `economy.rs` and `economy/`:**
  - `power.rs`: `PowerSummary { generated, consumed, ratio }`, using fixed point per T6;
  - `unit_control.rs`: the sum of `UnitControlCost` over owned units (reserved queue entries are added in D2);
  - `mining.rs`: an `Extractor { patch: ObjectId, cycle: u32 }` capability, `Object::extractor`, linked at insert to the patch under the plate.
- **`sim.rs`.** `economy::step` runs after the 5e removals and their `paths::sync`, and before 5f (T5).
- **`sim/removal.rs`.** When a plate is removed:
  - a patch whose `remaining == 0` is removed in the same tick, followed by `paths::sync`;
  - otherwise the patch is simply uncovered.

  When a patch under a plate is removed (debug only), the plate's extractor stops. The `testing::dangling_references` check covers `Extractor::patch`.
- **Commands and events.** Add `DebugCommand::SetResources { player, crystals, supplies }`. Events are optional, for example `PatchDepleted`.

**Acceptance**
- **Mining at full power.** Two plates give +20 at ticks 48, 96, and so on, paid in lumps (Q0028 point 1). The first payout comes 48 ticks after placement.
- **Mining under the penalty.** At ratio 30/51 (generated 30, consumed 51: the design's -21 case), a cycle lasts ⌈48 / r⌉ ticks with the remainder carried over, so the long-run rate is exactly 10 × 30/51 per 48 ticks.
- **Depletion.** A patch with 15 left yields 10, then 5 (Q0028 point 2), then 1 per cycle forever. `remaining` never underflows.
- **Plate destruction.**
  - On a patch with crystals: the patch stays and is uncovered.
  - On a depleted patch: the patch is gone in the same tick, its tile opens and `nav_version` bumps. This covers the 0038 verification "a patch mined out can be walked through".
- **Power.** Base (DC + EF + 2 plates) = +9 with generated 30 and consumed 21 (`> Starting Conditions`). Adding a Barracks gives -21 and ratio 30/51.
- **Unit Control** counts owned units' UnitControlCost (Peacekeeper 1).
- **Determinism.** The hash covers banks and extractors (a destructuring compile check), serde round-trips, and a replayed mining game matches.
- **Goldens** are re-blessed, because `Player` gains fields; explain it.

**After**: A. **Model**: Opus.

#### C: Build area and placement validation (Opus)

**Goal.** One pure sim function decides whether player P may place kind K with orientation O at origin T, and returns per-tile problems. The GDO build area is derived per player.

**Design**
- `core/entities.md > Placement Validation > Direct Placement`.
- `core/map.md > DefaultTilePresets` (Buildable) and `> Elevation > Structures only on flat terrain`.
- `factions/gdo/structures.md > GDOBuildArea` and the BRE of each kind.
- `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]` (AwaitingPlacement rule for plates).
- 0004 verification.

**Code**
- **New `placement.rs` and `placement/`.**
- **`build_area.rs`.**
  - Per GDO player, a `TileBits` that is the union of each own placed structure's footprint grown by its BRE. It is square (Chebyshev) per the Q0026 default.
  - It is derived and never hashed. It is cached with a per-player key and recomputed when the player's structures change.
- **`check.rs`.** `fn check(sim_view, player, kind, origin, orientation, mode) -> PlacementCheck { tiles: Vec<(TilePos, TileProblem)>, whole: Vec<WholeProblem> }`. `mode` is `Command`, or `Layout` (no visibility check; area seeded by the layout's own structures).
- **Rules for each footprint tile**:
  1. on the map;
  2. Buildable;
  3. flat;
  4. Visible to P (Command mode only);
  5. not under a structure;
  6. not under a resource, unless the kind is a plate (Q0027 point 3);
  7. not under a ground unit's box (Q0027 point 1).
- **Rules for the whole footprint**:
  1. at least one tile is in P's build area (GDO only);
  2. a plate must be exactly on a patch that has no plate.
- **API.** `Sim::placement_check(..)` is public for the client.

**Acceptance**
- **Unit tests**, one per rule.
- **0004 verification as a test.** A Generator footprint with one tile on a slope gives a NotFlat tile problem. A flat plateau gives none.
- **Build area.** A DC alone gives a (4 + 2×12) = 28×28 area. A Generator placed with one tile inside grows the area by 1 around itself. A plate is valid on an uncovered patch inside the area, and invalid outside it or on a covered patch.
- **Determinism.** The check is a pure function of sim state: the same inputs always give the same output.

**After**: A. Runs parallel with B and E. **Model**: Opus.

#### D1: Construction (DC and EF) (Opus)

**Goal.** A constructor builds one structure at a time and holds it in ReadyToPlace until it is placed. Cancel and the refunds work. The EF, a consumer, slows down under the power penalty; the DC does not (Q0025 point 1).

**Design**
- `factions/gdo/structures.md > DeploymentCenter`:
  - DeploymentCenterInstanceState, Constructs (Generator, Barracks and EF only) and Cancellation;
  - the command semantics of ObjectInterfaceState[DeploymentCenter]. The UI itself is H2.
- `factions/gdo/economy.md > ExtractionFacility`: ExtractionFacilityInstanceState, Constructs and Cancellation.
- `factions/gdo/overview.md > GDOResources > Power` ("construction speed").
- Prerequisites are not built (section 1.2).

**Code**
- **New `constructor.rs`.**
  - State: `Constructor { current: Option<ObjectKind>, progress: u32 (fixed point), ready: Option<ObjectKind> }` on `Object::constructor`.
  - `StructureDef::constructs: &[Recipe { kind, crystals, frames }]`.
- **Commands.** Add `Construct(kind)`, `CancelConstruction`, `CancelReady` and `Place { origin, orientation }` as `StructureCommand`s, sent in `Command::Structure { objects, command }` (T9).
- **Gate.** Checks:
  - the owner is the issuer and not defeated;
  - the constructor's state allows the command;
  - the cost, re-checked at apply time;
  - `Place` runs C's check.
- **Events.** `ConstructionCompleted`, `StructurePlaced` and `PlacementRefused { problems }`.
- **Placing.** It inserts the structure with its orientation. The 5b sync of the same tick adds it to the grids, and the build area changes.

**Acceptance**
- **Times at full power.** A Generator construct ordered at tick t is ready at t + 240. The Barracks takes 160 and the EF 320. A plate at the EF takes 400.
- **Under the penalty.** At ratio 30/51 a plate takes exactly 680 ticks (400 × 51/30; T6 makes this exact).
- **Refunds.** Cancel while constructing refunds 100%. Cancel when ready refunds 75%, rounded down: 75 → 56, 100 → 75, 200 → 150.
- **Refusals.** Construct is refused while busy or ready, and without the crystals; the bank is then unchanged.
- **Place.**
  - Valid: the structure exists with its orientation, and power and the build area update.
  - Invalid, for each problem from C: the command is refused, the building stays ready, and the event names the problem.
- **Mini-oracle.** Script the clean Barracks opening from the starting base: Plate, then Generator, Generator, Barracks.
  - The Barracks should start within tolerance of "~46 s" and finish "~10 s" later (`factions/gdo/economy.md > Opening Economy Baseline`).
  - With lumps, the expected start is about 48 s (section 6, R10).

**After**: B, C. **Model**: Opus.

#### D2: Production, rally points, Unit Control (Opus)

**Goal.** The Barracks trains Peacekeepers from its queue. Each unit appears at the B side and gets its rally order.

**Design**
- `factions/gdo/structures.md > Barracks`: BarracksInstanceState, RallyPoint behavior, Produces (Peacekeeper only), and the command semantics of ObjectInterfaceState[Barracks].
- `factions/gdo/overview.md > GDOResources > Unit Control` and `> Power` ("unit production speed").
- `factions/gdo/units.md > Peacekeeper > UnitControlCost`.
- `systems/controls/commands.md`: Move to a unit means follow; Attack.

**Code**
- **New `production.rs` and `production/`.**
  - State: `Production { queue: Vec<ObjectKind>, progress: u32, rally: Option<Rally> }`. The head of the queue is the current build (Q0024 point 1).
  - `rally.rs`: `Rally::{Point(Vec2), Object { id: ObjectId, at: Vec2 }}`, where `at` is the click point, kept for the Q0024 point 7 fallback. The spawn order:
    - ground → Move;
    - enemy object → Attack;
    - own or neutral object → Move to the object, which is Follow for units.
  - `exit.rs`: a deterministic search for the nearest free spot (no unit overlap, walkable on the true grid). It starts just outside the middle of side B and rings outward around the building (Q0023 points 3 and 4).
- **Commands** (as `StructureCommand`s, T9). `Train(kind)` applies per producer in id order, each checking the bank, queue < 20, and Unit Control with reservation (Q0024 point 2). Also `CancelProduction` and `SetRally(target)`.
- **`sim/removal.rs::forget`.** Rallies pointing at a removed object become None ("reset to None").
- **Events.** `UnitProduced` and `ProductionCancelled`.

**Acceptance**
- **Production time.** One Peacekeeper every 80 ticks at full power. At ratio 30/51 it takes exactly 136 ticks (80 / 0.588; the baseline says "~59%").
- **Train and cancel.**
  - Train deducts 50.
  - CancelProduction removes the last entry and refunds 50.
  - The queue stops at 20.
  - Train is refused at the Unit Control cap: debug-spawn 200 Peacekeepers to test it.
- **Rally.**
  - Ground: the unit moves there.
  - Enemy: the unit attacks it.
  - Own unit: the unit follows it.
  - Neutral station: the unit moves to it.
  - Target removed: the rally becomes None, and later units spawn with no command.
- **Exit.** The unit appears outside side B, even when 10 earlier units with no rally crowd the exit, and the invariant harness finds no overlap. Units leave on the correct side in all 8 orientations.
- **Determinism.** Two Barracks that finish in the same tick spawn in id order, and a replay matches.

**After**: B. Runs parallel with D1 (shared touch points: `command.rs`, the gate, `every_command`). **Model**: Opus. The exit search is simple; it is not pathfinding.

#### E: Path knowledge (Fable)

**Goal.** Paths are planned only from what the unit's owner knows (0040, 0041, 0042). Movement and collision stay on the true map, and units re-route when they discover an obstacle.

**Design**
- `systems/controls/behaviors.md > PathKnowledge`:
  - terrain is always known;
  - own structures are always known;
  - other players' structures are known once seen, and are walls at their last-known position until seen otherwise;
  - map-placed objects are known from the start;
  - a patch depleted in the fog stays known until seen gone.
- `core/vision.md > Explored`.
- `docs/movement.md > 5.2, 5.6, 5.7`.
- The verification points of 0038, 0040 and 0042.

**Code** (T3)
- **New `paths/knowledge.rs`.** Each player's known blockers are:
  - its own live structures;
  - every record's footprint (`record_tiles`, refreshed when `records_revision` changes);
  - the map's starting resources, minus those seen gone. This is a per-player set; it is hashed because it is history.
- **`NavState`.** It holds `truth: Nav` and `known: BTreeMap<PlayerId, Nav>`. A player's grids are built lazily, only for players that own ground units, and are rebuilt in 5b at most once per player per tick.
- **Planning uses the owner's grid.** That covers `serve`, `reachable_point`, the magic box (`orders/destinations.rs`), `chase_check`, the staggered line-of-sight re-check, and the avoid-tile repath.
- **`movement::step` and the spawn/teleport checks use `truth`.**
- **Versions.** A per-player `known_version`, hashed, takes over the role of `nav_version` for the staggered re-checks.
- **Clean-up.** Delete `ROUTE_ON_TRUE_MAP` and its const assert. Update `docs/movement.md`: G12 is now answered by 0040.

**Acceptance** (in `crates/sim/tests/`)
- **0040.**
  - A wall hidden in the fog: the unit heads for the gap, sees the wall, and re-routes with no failed move.
  - A scouted building destroyed out of sight: the unit routes around it until it sees the spot empty.
- **0042.**
  - Units route around a crystal field they have never seen.
  - A patch removed in the fog (its plate destroyed on a depleted patch) is still routed around until seen.
- **0038.** Units route around a crystal field. A removed patch's tile is walkable. Nobody walks across a station.
- **Collision.** An enemy structure never seen is open ground for planning, but no unit ever walks into it, because collision uses the true grid.
- **Perf.** Extend `nav_perf`: 4 players on 256², 10 structure deaths seen by all players in one tick. Report the worst tick in ms (target ≤ 5 ms in release) and the number of rebuilds.
- **Determinism.** Grids are rebuilt on load, so a save/load round trip compares equal. Goldens are re-blessed (version hashing); explain it.

**After**: A. It needs structure kinds for its tests, but not B–D. Own track. **Model**: **Fable**. This is the plan's pathfinding work (insights: Fable is reserved for pathfinding, movement and collision).

#### F: Start and end of a game (sim) (Opus)

**Goal.** A game starts from the picked faction's Starting Layout with the Starting Conditions resources. Layouts are validated per faction, including the Skirmish Default restriction and completeness. A player is defeated when they lose their last structure.

**Design**
- `factions/gdo/economy.md > Starting Conditions`.
- `core/map_editor.md > Starting Layouts`: Placing from scratch, Overlay editing ("valid on its own"), Skirmish Default restriction, Outdated layouts.
- `core/map_editor.md > Map Types > Skirmish Default`.
- `core/camera.md > Starting Position`.
- `core/game_rules.md > Defeat Condition`.
- The verification points of 0008, 0009 and 0012.

**Code**
- **New `scenario/starting.rs`.**
  - A `StartingConditions` per faction. GDO: `[(DC, 1), (EF, 1), (ExtractionPlate, 2)]`, 100 SC, supplies 0.
  - The other factions get `None`, meaning not implemented yet (T11).
- **`scenario/validate.rs`.**
  - Each layout object goes through C's check in Layout mode (Q0027 point 4).
  - On Skirmish Default maps: only Starting Conditions kinds, counts no higher than the conditions, and every implemented faction's layout complete at every start.
  - New problems: `LayoutMissing { slot, faction, kind, missing }`, `LayoutExtra` and `LayoutObjectInvalid`.
  - "Outdated" is derived (T11), so the map format stays v3.
- **`Sim::try_new`.** Sets the banks from the Starting Conditions and links layout plates to their patches.
- **Defeat (T8).** `Player::defeated_at: Option<u32>`, checked at the end of the economy phase, plus `SimEvent::PlayerDefeated`. The gate refuses a defeated player's commands.
- **`crates/game/src/camera.rs::start_center`.** Use the player's DC when the slot has no camera start (the current code comment says "No starting structures exist yet").
- **`maps/dev_skirmish.json`.** Propose a GDO layout for both starts, for the Director to approve; the plan says the Director authors it. Check that 8 patches lie inside the DC's build area (R13).

**Acceptance**
- **0009.** On a Skirmish Default map, the GDO layout allows only a DC, an EF and 2 plates. A third plate or a Barracks is a problem.
- **0012.**
  - An incomplete GDO layout lists what is missing, and the map can't start.
  - An owned object outside the layouts is refused (this already exists).
  - The GDO layout is mirrored onto the opposite start with its orientations, by 180° symmetry.
- **0008.** Starting as GDO puts each object exactly where the layout puts it. Only the picked faction's layout spawns.
- **Layout placement.** A plate must be on a patch inside the DC's area. An EF needs at least one tile inside it.
- **Game start.** A GDO player has 100 SC and power +9, and the camera is on their DC.
- **Defeat.**
  - Destroying a player's last structure defeats them in that tick, with one event. Plates count.
  - A player who never owned a structure, as in M1 sandbox tests and goldens, is never defeated.
- **Goldens.** Hashes change because of `defeated_at`; re-bless and explain. Golden replays embed their own map, so `dev_skirmish` edits don't affect them.

**After**: B (banks), C (layout checks). **Model**: Opus.

#### G: HUD: resources and structure info (Opus)

**Goal.** The top bar shows the faction's resources. The InfoPanel and SelectionPanel handle structures. A banner shows defeat or victory.

**Design**
- `factions/gdo/overview.md > DisplayHud`.
- `core/entities.md > Player > DisplayHudInfo[Faction]` and `> Object Type > InfoPanel`.
- `systems/controls/selection_and_hud.md > HUD Bar`, `> InfoPanel with 2+ objects selected` (placeholder) and `> SelectionPanel`.
- `factions/gdo/economy.md > ExtractionPlate > InfoPanel`, and `core/map.md > SpaceCrystalsPatch > InfoPanel`.
- The banner's look is not specified; this is Q0029.

**Code**
- **`hud.rs`.** A resources section in the top bar, through a new `hud/resources.rs`. It is per faction: GDO is implemented; other factions show crystals only for now.
- **`hud/info_panel.rs`.** Structure lines per the Q0030 point 3 default:
  - HP;
  - construction: current build, progress and the ready building;
  - production: current build, progress, queue length and rally;
  - plate: the patch's remaining crystals;
  - other players' structures: name, owner and HP.
- **`hud/selection_panel.rs`.** Labels for the new kinds.
- **New `hud/outcome.rs`.** The defeat and victory banner.

**Acceptance**
- **Client tests** (`FixedTimesteps`) on the texts:
  - at the start: `Space Crystals 100`, `Supplies 0`, `Power 9/30` and `Unit Control 0/200`;
  - the crystals line updates after the first payout (tick 48);
  - the plate's InfoPanel shows the remaining crystals;
  - a constructing DC shows its progress.
- **QA screenshots.** The HUD at the start, a DC InfoPanel mid-construction, and the defeat banner.
- **Font.** ASCII only (insights), so negative power is written `-21/30`.

**After**: B. The D1 and D2 lines can follow those slices, or the Director can schedule G after them. **Model**: Opus.

#### H1: Production UI and rally points (Opus)

**Goal.** The Barracks panel works end to end, and the command UI is generalised from units to structures.

**Design**
- `factions/gdo/structures.md > Barracks > ObjectInterfaceState[Barracks]`:
  - Q Build Peacekeeper; X Cancel Production; C Set Rally Point;
  - W Build Rocketman is hidden, because the Rocketman is deferred.
- `systems/controls/selection_and_hud.md`:
  - `> CommandPanel`: `> Standard Slot Assignments`, `> Production Building Default Right-Click`, and "Clicking a group-specific command issues it only to objects in the ActiveGroup";
  - `> PointerDisplayType > Resolution Rules`: "Selection is a production building: Move (rally point)", and AwaitingTarget[SetRallyPoint] shows Move;
  - `> CommandIndicators`: there is no rally entry, so the Q0024 point 6 default applies.
- Changes 0062/0063: in a mixed selection, each object carries out its own right-click.

**Code**
- **Split `command_ui/interface.rs`** (649 lines). `UnitCommand` becomes `PanelCommand`: the unit commands plus `Train(kind)`, `CancelProduction` and `SetRallyPoint`.
- **`InterfaceState`** becomes `{ DefaultState, AwaitingTarget(PanelCommand), BuildMenu, AwaitingPlacement(Placing) }`. H2 fills in the last two.
- **Templates.** Add `InterfaceTemplate::Barracks`, with button availability taken from the same sim predicates the gate uses (T10).
- **`command_ui/resolution.rs`.** A production building's right-click resolves to SetRally.
- **Rally indicator.**

**Acceptance**
- **Client tests:**
  - select a Barracks and press Q → `Train` is sent and the queue grows;
  - X → cancel;
  - C then a click on the ground → the rally is set and its indicator shows;
  - a right-click on an enemy → an Attack rally;
  - Peacekeepers and a Barracks selected, right-click the ground → the units Move and the Barracks sets its rally;
  - Q is greyed out below 50 SC;
  - every M1 command UI test still passes.
- **QA screenshot.** A Barracks panel with a queue and a rally indicator.

**After**: D2. **Model**: Opus.

#### H2: Construction UI: build menu, ghost, build-area overlay (Opus)

**Goal.** The DC and EF panels work. AwaitingPlacement shows the ghost with per-tile validity, the overlay, rotate/flip and the side labels, and sends Place.

**Design**
- `factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState[DeploymentCenter]`: DefaultState, the three BuildMenu states, and AwaitingPlacement.
- `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`.
- `systems/controls/selection_and_hud.md`:
  - `> PointerDisplayType > Resolution Rules > AwaitingPlacement`: no pointer; the ghost is the cursor;
  - `> Standard Slot Assignments`: Z Back.
- `core/entities.md > Structure Type`.
- The verification points of 0004 and 0048.

**Code**
- **New client `placement/`:**
  - `ghost.rs`: the footprint mesh. Its tiles are tinted from `Sim::placement_check`, and A/B/C/D labels come from `sides()`.
  - `overlay.rs`: one build-area mesh, rebuilt when the area's version changes.
- **Templates** `DeploymentCenter` and `ExtractionFacility`. Their grids depend on the constructor's state: idle, constructing or ready.
- **Input routing in AwaitingPlacement.**
  - R and Shift+R rotate; F and Shift+F flip. These override the slot bindings, and Shift is not Queue here.
  - Left-click places.
  - Right-click, Esc or Z goes back: to the BuildMenu for the DC, to DefaultState for the EF.
- **BuildMenu slots.** Use the Q0030 point 2 default.

**Acceptance**
- **Client tests:**
  - on the DC, Q then Generator sends Construct;
  - with a building ready, its slot enters AwaitingPlacement and the ghost follows the cursor, snapped to the grid;
  - R turns the Barracks ghost from 3 wide × 2 tall to 2 wide × 3 tall (Q0023 default), and the labels follow rotations and flips;
  - a valid click sends Place and returns to DefaultState;
  - an invalid click sends nothing and stays in AwaitingPlacement ("cannot be confirmed");
  - Esc goes back to the BuildMenu;
  - the EF ghost is green only over an uncovered patch inside the area.
- **0048 verification.** With the DC and EF selected through a control group, Tab switches the panel.
- **QA screenshots.** A green ghost on a flat plateau and a red one on a slope (0004 verification), the overlay, and a rotated Barracks with its labels.

**After**: C, D1, H1. **Model**: Opus.

#### I: Editor: Starting Layout editing (Opus)

**Goal.** A map maker places each faction's starting objects at each start position, sees all factions overlaid, and gets the restriction, missing-object and outdated feedback.

**Design**
- `core/map_editor.md > Starting Layouts`: all four subsections.
- `core/map_editor.md > Symmetry > Editing a symmetric map`: layouts are mirrored, and "Building orientations are rotated or flipped to match".
- The verification points of 0008, 0009 and 0012.
- M0-E leftovers: the panel scrollbar, and the right panel overflowing with 4 or more slots.

**Code**
- **New `editor/layouts.rs`**, as a Layout mode or its own tool:
  - a slot and faction picker;
  - a palette restricted to the faction's Starting Conditions, with remaining counts;
  - rotate and flip;
  - the other factions' layouts dimmed in the overlay;
  - a "missing" list built from F's problems;
  - an outdated flag.
- **Saving.** Uses the existing format; the generated half comes from `scenario/generate.rs`.

**Acceptance**
- **Client tests:**
  - place a DC in slot 1's GDO layout;
  - a second DC is refused on Skirmish Default;
  - the missing list shows `Extraction Plate x2`;
  - the mirrored start shows the objects with mirrored orientations;
  - save and load round-trip.
- **Overlay.** The 0008 verification ("the overlay of 4 layouts") is done for GDO. The other factions show "not implemented yet" (T11).
- **QA screenshot.** The overlay with the missing list.

**After**: F. **Model**: Opus.

#### J: M2 QA (Opus)

**Goal.** Pin the economy down with integrated tests and QA artefacts before the user playtests.

**Design**
- `factions/gdo/economy.md > Opening Economy Baseline` and the GDO column of `> Economy Targets (GDO vs Colonists)`.
- Every M2 verification point not covered yet.

**Code**
- **`crates/sim/tests/economy_oracle.rs`**, scripted build orders with documented tolerances:
  - plates first, the first Generator once 100 SC are banked: income per second matches the table (10.0 / 13.3 / 16.7 / 20.0 / 23.3 / 26.7) at 0:40, 1:00, 1:20, 1:40, 2:20 and 2:40, and the base is fully plated at about 2:30;
  - the all-in Barracks starts at about 15 s, leaves power at -21, and production runs at 30/51;
  - the clean Barracks starts at about 46 s (48 s with lumps) and finishes at about 56 s;
  - opening with a Generator delays the first plate by about 11 s.
- **`tests/golden.rs` game `gdo_opening`.** About 3 minutes: the clean line, then 10 Peacekeepers rallied into the enemy's plates.
- **QA input scripts and screenshots** for the first-playable walkthrough.
- **New `structures_perf`** (ignored, release): 4 GDO bases with 40 structures each and 400 units fighting through them. Report the nav rebuilds per tick, path knowledge, vision and the economy phase, in ms.
- **`ARCHITECTURE.md` review.**

**Acceptance**
- The oracle passes, with its tolerances explained.
- The golden is blessed.
- Perf numbers are reported; flag any worst tick over 10 ms.
- The screenshot set is reviewed.
- Playtest notes for the user.

**After**: everything. **Model**: Opus.

---

## 4. Technical decisions for the Director

Each decision gives a **recommendation** and its alternatives.

### T1. Structure state in the object model
- **Recommend** separate capabilities, per `ARCHITECTURE.md > Object model > Adding a capability`:
  - `constructor: Option<Constructor>` for the DC and EF. Their InstanceStates have the same shape: CurrentConstruction, ConstructionProgress, ReadyToPlace.
  - `production: Option<Production>` for the Barracks. The Supply Tower reuses it later (same BuildQueue, CurrentBuild and progress).
  - `extractor: Option<Extractor>` for the plate.
  - Per-type data in `ObjectDef::structure: Option<StructureDef>`: symmetry, BRE, power, recipes.
- **Alternatives:**
  - One `structure: Option<StructureState>` enum per kind. Systems would then match on kinds, and the Supply Tower's production plus chopper attachment wouldn't fit.
  - Stores keyed by id. ARCHITECTURE forbids them.

### T2. Size, orientation and sides
- **Recommend:**
  - Keep the existing orientation model: flip in the building's own frame, then rotate clockwise.
  - Add one pure `sides(kind, orientation)` that everything uses: the exit, the labels, and later the drop-off sides.
  - Until Q0023 is answered, use its proposed convention: side A is the top edge at rotation 0, letters run clockwise, and the Barracks is 3 wide × 2 tall. That puts B (the exit) on a short side, as `factions/gdo/structures.md > Barracks` requires.
- **Alternative:** read "3x2" as Height × Width (`core/entities.md > Object Type > Size - Height x Width`), giving 2 wide × 3 tall. Then clockwise lettering from the top puts B on a long side, which contradicts the Barracks text.
- **Cost of changing later:** low. No starting layout contains a Barracks.

### T3. Path knowledge (E)
- **Recommend:**
  - **Grids.** A **true grid** for movement, collision and spawn checks. **Per-player knowledge grids** for planning, built lazily and only for players with ground units.
  - **What a player knows:** terrain, plus its own live structures, plus every record (structures and resources; `record_tiles`), plus the starting map resources not yet seen gone.
  - **Starting resources.** The Director's plan already notes they need their own starting set (records aren't seeded). Keep it as a per-player set of ids, hashed. An id is dropped when the player sees the spot without the object; this mirrors how a record is dropped.
  - **Rebuilds.** Rebuild a player's grids in 5b when any of these inputs changed, at most once per player per tick.
  - **Re-routes.** A per-player `known_version`, hashed, drives the staggered line-of-sight re-check (`docs/movement.md > 5.6`), which gives the "re-route on discovery".
- **Alternatives:**
  - **Seed records for map resources at the start.** The fog display and the selectable stations (0057) would then show unexplored resources, which changes what M1-F built. Rejected.
  - **A search-time overlay** (the true grid plus a per-player diff). It saves memory, but components, `reachable_point`, `los` and smoothing would all need to know about the overlay. That is invasive.
  - **Incremental grid patching.** Only if E's perf numbers call for it.

### T4. Placement validation (C)
- **Recommend:**
  - One pure sim function with a Command mode and a Layout mode, returning per-tile problems.
  - The gate re-runs it when the command is applied; the client's ghost runs it every frame on the local sim. One rule set, no drift.
  - Ground units under the footprint make the placement fail (Q0027 point 1).
  - A refused Place leaves the building in ReadyToPlace and emits `PlacementRefused`.
- **Alternatives:**
  - Validate on the client only. The sim would then trust the client, which is a desync risk.
  - Push units out of the footprint. That needs a displacement behaviour; it is later work, if the Designer asks for it.

### T5. Where the economy runs in the tick
- **Recommend** one `economy::step`, after 5e's removals and `paths::sync`, and before 5f. It runs:
  1. power per player, from every owned structure present, including those placed this tick (Q0025 point 4);
  2. constructors;
  3. producers: spawns plus rally orders. Per the spawn-tick rule, the units act from t + 1;
  4. plates;
  5. the defeat check.

  Because 5f comes after it, units produced this tick are seen this tick. Structures placed by a command are inserted in step 4 and enter the grids in 5b of the same tick. Update the `ARCHITECTURE.md` tick list.
- **Alternatives:**
  - Run it before 5a. Spawned units would then act in their spawn tick, which breaks the spawn-tick rule.
  - Run it after 5f. New units would be unseen for one tick.

### T6. Progress and rates in fixed point
- **Recommend:**
  - Compute the penalty ratio as `r = min(2^16, ceil(generated × 2^16 / consumed))` in u64. A player with no consumers gets `2^16`.
  - Consumers (EF construction, Barracks production, plate cycles) add `r` per tick. The DC and the Generator add `2^16` (Q0025 point 1).
  - A build completes when its progress reaches `frames × 2^16`.
  - Mining cycles carry the remainder over, so the long-run rate is exact.
  - **Rounding the ratio up** makes exact ratios give the exact tick count: 80 frames at 30/51 → 136 ticks, and 400 frames → 680 ticks. Flooring would give 137 and 681.
  - Everything is integer and hashed.
- **Alternatives:**
  - f32 progress. It is deterministic with basic operations but sensitive at boundaries.
  - Exact rational accumulators. They get complicated once the ratio changes mid-build.

### T7. Resource accounting
- **Recommend:**
  - **Banks.** `Player { crystals: u32, supplies: u32, defeated_at: Option<u32> }`.
  - **Costs** are taken when the command applies. The gate re-checks the bank at that moment, in input order, and per issuing object in id order.
  - **Refunds** go through one helper: `Full`, or `Ready`, which is `cost * 3 / 4` rounded down (`> Cancellation`).
  - **Arithmetic.** `checked_sub` refuses a spend; additions saturate behind a `debug_assert`.
  - **Derived values.** Unit Control and power are computed, not stored as counters, so they can't drift.
  - **Tests.** New state goes into `state_hash`, whose destructuring makes an omission a compile error, and into serde with `#[serde(default)]`. Update `every_command` and `dangling_references`.
- **Alternative:** incremental counters for power and Unit Control. They are faster, but they can drift and would have to be hashed.

### T8. Defeat in the sim
- **Recommend:**
  - A player is defeated in the tick its owned-structure count drops to 0. This matches `core/game_rules.md > Defeat Condition` for every real game, because every faction starts with structures.
  - A player who never owned a structure is never defeated. This keeps the M1 sandbox, the tests and the goldens alive; M1 players own no buildings.
  - Record it as `defeated_at`, emit an event, and have the gate refuse the player's commands.
  - What happens to the defeated player's units and to the game is Q0029.
- **Alternative:** a game option `defeat: bool`, on for skirmish games and off for sandboxes. It is explicit, but every test setup then has to choose.

### T9. Commands for structures
- **Recommend** one new variant, `Command::Structure { objects: Vec<ObjectId>, command: StructureCommand }`, where `StructureCommand` is one of `Construct(kind)`, `CancelConstruction`, `CancelReady`, `Place { origin, orientation }`, `Train(kind)`, `CancelProduction` or `SetRally(target)`.
  - Each command is validated per object, in id order. This matches "issues it only to objects in the ActiveGroup".
  - `DebugCommand::SetResources` is added for tests and QA.
- **Alternative:** a top-level variant per command. It is flat, but it gives a longer `every_command` and more gate code.

### T10. Command UI for structures
- **Recommend:**
  - Generalise `UnitCommand` into `PanelCommand`.
  - `InterfaceState` becomes `{ DefaultState, AwaitingTarget(PanelCommand), BuildMenu, AwaitingPlacement(Placing { constructor, kind, orientation }) }`.
  - Each kind's template is computed from sim state every frame (CommandPanel: "derived from ControlState and game state each tick").
  - Button availability uses the same `sc_sim` predicates as the gate.
  - The ghost replaces the pointer.
  - The overlay mesh is cached by the area's version.
  - In AwaitingPlacement, R and F are placement keys, not panel slots.
- **Alternative:** a separate structure UI next to the unit UI. It duplicates the targeting, resolution and indicator code.

### T11. Starting Conditions and layouts while factions are missing
- **Recommend:**
  - The Starting Conditions table lives in the sim.
  - A faction counts as "implemented" once it has a table; only GDO does in M2.
  - On Skirmish Default maps, restriction and completeness are checked for implemented factions only. Unimplemented factions' layouts must stay empty: they have no objects yet. This is staging, not a design change. Tell the Designer as an FYI; no question is needed.
  - "Outdated" is derived: a layout is outdated when its objects don't match the faction's current Starting Conditions. This keeps the map format at v3, and golden replays embed their maps, so there is no migration.
- **Alternative:** store a Starting Conditions fingerprint per layout (format v4). It tells "outdated" apart from "unfinished", at the cost of a migration.

### T12. Not built in M2
- Tech prerequisites (no M2 kind has one).
- Supplies beyond a counter fixed at 0 (Q0022).
- ConstructionHP.
- A rally for buildings with no producer.

### T13. File hotspots
New code goes into new modules: `economy/`, `constructor.rs`, `production/`, `placement/` (sim and client), `paths/knowledge.rs`, `hud/resources.rs` and `editor/layouts.rs`. H1 splits `command_ui/interface.rs`. `sim.rs` (552 lines) only gains the phase call.

---

## 5. Design questions (drafts in the Director's format)

**Blocking status:**
- **Q0022** blocks, but only the supply part, which this plan defers.
- **No question blocks the M2 core.** Each point has a proposal, and the slices build it.
- **Answer before particular slices:**
  - Q0023 before D2 and H2;
  - Q0028 point 1 before J's oracle;
  - Q0026 point 1 before F's `dev_skirmish` layout.
- **Already open:** Q0017 point 2 (targeting remembered structures) becomes real in M2. M2 builds its stated assumption: visible targets only, and a right-click on a remembered structure moves to it.

### Q0022: GDO supplies in M2: Supply Tower, Supply Chopper and Radar Station are incomplete

- **Date**: 2026-11-23
- **Blocking**: yes, but only for GDO supplies. M2 is planned without them: the HUD shows Supplies 0, and Supply Delivery Stations work as in M1 (0057).
- **Docs**:
  - `factions/gdo/economy.md > SupplyTower`, `> Starting Conditions`;
  - `factions/gdo/structures.md > Radar Station`, `> DeploymentCenter > Constructs`;
  - `factions/gdo/units.md > SupplyChopper`;
  - `factions/gdo/overview.md > GDOResources > Supplies`.

#### Question
1. **The Supply Tower's prerequisite and cost.**
   - The Tower requires a Radar Station, which has "Full stats TBD". The Tower's own cost and build time are also TBD.
   - **Proposal:** build the Tower once the Radar Station is designed. Alternatively, give the Tower a cost and time now, and postpone its prerequisite.
2. **The Supply Chopper's flight.**
   - The Chopper is an air HoverCraft driven by accelerations, with no MaxSpeed. Air movement (0043–0047) and the acceleration model (0035) are planned for the vehicles and air milestone.
   - Its transport is TBD (`> Cargo`).
   - **Proposal:** build the Chopper with air movement, in that milestone.
3. **What supplies buy.**
   - Starting Supplies are TBD, and nothing GDO builds costs supplies yet. Collected supplies would have no use.
   - **Proposal:** Starting Supplies stay 0 until GDO supply costs exist.

#### Context
`plan.md` lists "GDO supplies: Supply Tower, Chopper" under M2. Planning M2 found that none of the supply loop can be finished without the items above. Everything else in M2 (buildings, crystals, power, production, layouts) is unaffected.

#### Options
- **A** (recommended): defer the supply loop to the air milestone. M2 shows Supplies 0.
- **B**: fill in the TBDs now (Radar Station stats, Tower cost and time) and build a simplified straight-line Chopper in M2. This costs about two more slices and rework when air movement arrives.
- **C**: build the Tower in M2 with no Chopper. It would collect nothing.

### Q0023: Barracks footprint, sides and the exit

- **Date**: 2026-11-23
- **Blocking**: no. M2 builds the proposals. Please answer before production and placement are finished.
- **Docs**:
  - `factions/gdo/structures.md > Barracks` (Size 3x2, SymmetryType ABAC, "Units exit from one of the short sides (B side)");
  - `core/entities.md > Object Type > Size - Height x Width`, `> Structure Type`.

#### Question
1. **Footprint and side letters at rotation 0.**
   - `core/entities.md` says Size is Height × Width, so "3x2" means 3 tall and 2 wide.
   - SymmetryType names the sides, but not which edge is A or which way the letters run. With ABAC, B and C face each other, and the Barracks text needs B on a short side.
   - The natural rule is "A is the top edge, letters run clockwise". With a 2-wide, 3-tall footprint, that puts B on a long side.
   - **Proposal:**
     - At rotation 0 the Barracks is 3 wide and 2 tall. Under Height × Width its size would then be written 2x3.
     - Sides run clockwise from the top: top A, right B, bottom A, left C. Units exit on the right.
     - The same rule applies to every building.
2. **Where a produced unit appears.**
   - **Proposal:** just outside the middle of side B. If that spot is taken by units, the unit appears at the nearest free spot, searching outward from there.
3. **A blocked exit.**
   - Buildings can be placed flush against side B, and the placement rules don't keep exits clear.
   - **Proposal:** the unit appears at the nearest free spot around the building, starting from side B. Production never stalls.

#### Context
The orientation tools (R, Shift+R, F, Shift+F and the A/B/C/D labels) are built in M2. The Barracks is the first building that isn't square, and the first with a meaningful side. The art pipeline (0003) renders one sprite per distinct orientation, so the convention also matters for art.

#### Options
- **Point 1:**
  - A (proposal): 3 wide × 2 tall, sides clockwise from the top.
  - B: 2 wide × 3 tall as written, with letters starting on a long side (for example, clockwise from the left: left A, top B, right A, bottom C).
- **Points 2 and 3:**
  - A (proposal): the nearest free spot around the building, starting at side B.
  - B: production waits until side B is clear.
  - C: placement refuses a building that would block another building's exit.

### Q0024: Production queue and rally point details

- **Date**: 2026-11-23
- **Blocking**: no. M2 builds the proposals.
- **Docs**:
  - `factions/gdo/structures.md > Barracks > BarracksInstanceState`, `> RallyPoint behavior`, `> ObjectInterfaceState[Barracks]`;
  - `factions/gdo/structures.md > DeploymentCenter > Cancellation`;
  - `factions/gdo/overview.md > GDOResources > Unit Control`;
  - `systems/controls/selection_and_hud.md > CommandPanel`, `> CommandIndicators`;
  - `core/tech_and_research.md > Research > Shared Rules`.

#### Question
1. **Does the limit of 20 include the unit in production?**
   - BuildQueue (max 20) and CurrentBuild are separate fields.
   - **Proposal:** 20 in total, the unit in production included.
2. **Unit Control for queued units.**
   - Q needs "sufficient ... Unit Control", and "the player cannot build units if doing so would exceed the cap".
   - **Proposal:** a queued unit takes its Unit Control when it is queued, as it pays its crystals then. The HUD's "used" count includes queued units, and cancelling gives the Unit Control back.
   - **Alternative:** only existing units count, and a unit waits at the front of the queue while the cap is full.
3. **Several Barracks selected.**
   - **Proposal:** each press of Q queues one Peacekeeper in every selected Barracks of the ActiveGroup. Each one pays, in a fixed order, while crystals last.
   - **Alternative:** each press queues one unit in total, in the Barracks with the shortest queue.
4. **A producer or constructor destroyed while working.**
   - The GDO docs are silent. Research is refunded in full when its building is destroyed.
   - **Proposal:** treat it like cancelling:
     - the Barracks queue is refunded in full;
     - a building under construction in the DC is refunded in full;
     - a ready building is refunded at 75%, rounded down.
   - **Alternative:** everything in progress is lost.
5. **Clearing a rally point.**
   - The design gives no way back to "None" except the target disappearing.
   - **Proposal:** setting the rally point on the Barracks itself clears it.
6. **Showing the rally point.**
   - CommandIndicators has no rally entry.
   - **Proposal:** while the building is selected, show a green location marker for a ground rally, a green object marker on a friendly or neutral object, and a red object marker on an enemy.
7. **A rally on an enemy that can't be seen when the unit appears.**
   - A unit can't be ordered to attack a target its owner doesn't see (Q0017's assumption).
   - **Proposal:** the unit attacks if the enemy is visible. Otherwise it moves to the spot where the rally point was set.

#### Context
These points come from planning Barracks production for M2. The proposals follow the crystal rules: costs are paid when queued, and cancelling refunds them in full.

#### Options
- **Point 1:** A, 20 in total; B, 20 waiting plus the unit in production.
- **Point 2:** A, reserved when queued; B, checked when production starts.
- **Point 3:** A, one per selected Barracks; B, one in total, to the shortest queue.
- **Point 4:** A, refunded as for a cancel; B, lost.
- **Point 5:** A, rally on itself clears it; B, a dedicated command.
- **Points 6 and 7:** A, as proposed; B, the Designer's alternative.

### Q0025: Power penalty details

- **Date**: 2026-11-23
- **Blocking**: no. M2 builds the proposals.
- **Docs**:
  - `factions/gdo/overview.md > GDOResources > Power`, `> DisplayHud`;
  - `factions/gdo/economy.md > Opening Economy Baseline`.

#### Question
1. **Is the Deployment Center slowed?**
   - The penalty applies to "power-consuming buildings and Extraction Plates". The DC produces power (+30), so read literally, its construction is never slowed.
   - The baseline's all-in Barracks mentions only "plates and Peacekeeper production" running at about 59%.
   - **Proposal:** the DC is never slowed. The EF (plate construction), the Barracks (production) and the plates (mining) are.
2. **The ratio.**
   - **Proposal:** generated power divided by consumed power, capped at 100%. The baseline's -21 case gives about 59% (30/51), which matches this rule.
3. **"Power: current / total".**
   - **Proposal:** current = generated − consumed, which can be negative; total = generated. The display reads `9/30` at the start and `-21/30` in the all-in case.
4. **When a building's power counts.**
   - **Proposal:** from the tick the building is placed until the tick it is destroyed.

#### Context
Power is built in M2. Point 1 decides how the opening plays out at negative power.

#### Options
- **Point 1:** A, the DC is never slowed; B, every GDO constructor and producer is slowed, the DC included.
- **Point 3:** A, total = generated; B, total = consumed.

### Q0026: GDO build area: shape and what happens when buildings are lost

- **Date**: 2026-11-23
- **Blocking**: no. M2 builds the proposals. Please answer point 1 before the test map's layouts are finalised.
- **Docs**:
  - `factions/gdo/structures.md > GDOBuildArea`;
  - `factions/gdo/structures.md > DeploymentCenter` (its description calls it the "build radius");
  - BuildRadiusExtension of each GDO building;
  - `factions/gdo/economy.md > Opening Economy Baseline` ("8 patches in GDO build range").

#### Question
1. **Shape.**
   - The area "expands outward from the placed building by that building's BuildRadiusExtension in all directions". It could be square (every tile within that many tiles of the footprint, diagonals included) or round (by distance from the footprint).
   - **Proposal:** square. With the DC's 12, the starting area is 28 × 28 tiles.
   - **Alternative:** round, which is closer to "radius".
2. **Losing buildings.**
   - Does the area shrink when a building, or the DC itself, is destroyed?
   - **Proposal:** the area is always the union of the current buildings' areas, so it shrinks.
   - **Alternative:** it only ever grows.
3. **No Deployment Center.**
   - The area is "seeded by each Deployment Center's and each Outpost's BuildRadiusExtension".
   - **Proposal:** with the DC gone, the other buildings' areas still count, so an EF can still place plates.
4. **What the overlay shows.**
   - The overlay shows "all valid build cells".
   - **Proposal:** it shows the build area only. Whether a given tile is valid (slope, fog, occupied) shows on the ghost.

#### Context
The build area is built in M2. It decides where buildings and plates can go and how many patches a starting base reaches. The reference base assumes 8.

#### Options
- **Point 1:** A, square; B, round.
- **Point 2:** A, follows the current buildings; B, only grows.
- **Point 4:** A, the area only; B, the area minus tiles that are invalid right now.

### Q0027: Placement edge cases

- **Date**: 2026-11-23
- **Blocking**: no. M2 builds the proposals.
- **Docs**:
  - `core/entities.md > Placement Validation`;
  - `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`;
  - `core/map.md > Resource`, `> SpaceCrystalsPatch`;
  - `core/map_editor.md > Starting Layouts > Overlay editing`.

#### Question
1. **Units under the footprint.**
   - Direct Placement rules out overlapping structures only. Worker-Built and Warp In require "unoccupied" tiles.
   - **Proposal:** a ground unit of any player that stands on the footprint makes those tiles red, and the building can't be placed there.
   - **Alternative:** own units step aside and the placement succeeds.
2. **Plates and visibility.**
   - Direct Placement requires every tile to be Visible. The DC's build area reaches 12 grid units beyond it, but its sight covers only about 4 beyond its edge, and the EF's sight is 3. Patches inside the area may be under fog.
   - **Proposal:** keep the rule as written. A plate can only go on a patch the player currently sees.
   - **Alternative:** plates are exempt from the visibility rule.
3. **Other buildings on patches and stations.**
   - Direct Placement forbids overlapping structures, but patches and stations are Resources, not structures.
   - **Proposal:** no building may cover a patch or a station, except a plate on a patch.
4. **Which rules a Starting Layout follows.**
   - A layout "only needs to be valid on its own (normal placement rules)".
   - **Proposal:** for GDO, the tiles must be Buildable and flat, with no overlaps except plates on patches, and the EF and plates must touch the build area of the layout's own DC. There is no visibility check.

#### Context
The placement rules are built in M2. The ghost shows each failed rule per tile.

#### Options
- **Point 1:** A, refuse; B, own units step aside.
- **Point 2:** A, plates need visible tiles; B, plates are exempt.
- **Point 3:** A, no building on resources except plates on patches; B, allowed.
- **Point 4:** A, as proposed; B, the Designer's rule.

### Q0028: Mining and depletion

- **Date**: 2026-11-23
- **Blocking**: no. M2 builds the proposals. Point 1 sets the economy tests' expectations.
- **Docs**:
  - `factions/gdo/economy.md > ExtractionPlate`, `> Opening Economy Baseline`;
  - `core/map.md > SpaceCrystalsPatch`;
  - change 0038.

#### Question
1. **Paid in lumps or as a trickle?**
   - "10 Space Crystals per 48 frames" could mean 10 at once every 3 seconds, or 1 crystal every 4.8 frames.
   - The baseline's timings match a trickle: the clean Barracks starts at 45.8 s. Paid in lumps, it starts at about 48 s.
   - **Proposal:** lumps of 10 every 48 frames, as written. A new plate pays first 48 frames after it is placed.
2. **The last partial load.**
   - A map maker can give a patch an amount that isn't a multiple of 10, for example 1505.
   - **Proposal:** the last load takes what is left (5 here), and later loads mine the residual 1.
3. **A depleted patch under a plate.**
   - `core/map.md` says a depleted patch disappears. The plate rules say the plate keeps mining at the residual rate, and that the depleted patch is removed only when the plate is destroyed.
   - **Proposal:** the plate rules are GDO's exception. The patch stays under its plate with 0 crystals left. Other factions' harvesting follows the general rule.
4. **Selecting a covered patch.**
   - **Proposal:** a click on that tile selects the plate. The patch can't be selected while it is covered, and the plate's InfoPanel shows the patch's remaining crystals.

#### Context
Plate mining is built in M2. Point 1 shifts the opening by about 2 seconds compared with the baseline's numbers.

#### Options
- **Point 1:** A, lumps; B, a trickle.
- **Point 2:** A, the last load takes what is left; B, a full 10 is mined as long as anything is left.
- **Point 3:** A, the patch stays under the plate; B, the patch disappears and the plate mines from nothing.

### Q0029: What defeat does

- **Date**: 2026-11-23
- **Blocking**: no. M2 builds the proposals.
- **Docs**: `core/game_rules.md > Defeat Condition`

#### Question
1. **The defeated player's remaining units.**
   - **Proposal:** they are removed at once.
   - **Alternatives:** they stay but can't be controlled; or they stay under the player's control until they are destroyed.
2. **Victory and the end of the game.**
   - **Proposal:** when only one player (or team) is left undefeated, that player wins. Every player sees Victory or Defeat, the game stops, and the replay is saved.
   - **Alternative:** the game continues, and defeated players watch.

#### Context
M2 builds defeat: a player is defeated when they no longer own a building, and Extraction Plates count. The design doesn't say what follows.

In sandbox games and tests, players start with no buildings. Defeat therefore applies only when a player loses their last building. This matches the rule for every real game, since every faction starts with buildings.

#### Options
- **Point 1:** A, removed; B, stay but can't be controlled; C, stay under control.
- **Point 2:** A, the game ends when one side is left; B, the game continues.

### Q0030: Structures as targets and in the interface

- **Date**: 2026-11-23
- **Blocking**: no. M2 builds the proposals.
- **Docs**:
  - `systems/combat.md > ValidTarget`;
  - `systems/controls/behaviors.md > BaseAutoTargeting`;
  - `factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState[DeploymentCenter]`;
  - `core/entities.md > Object Type > InfoPanel`;
  - `systems/controls/selection_and_hud.md > CommandPanel > Grid Layout`.

#### Question
1. **Do units attack enemy buildings on their own?**
   - ValidTarget begins "An enemy unit is a valid target", but its first condition is "a Destructible ObjectInstance".
   - **Proposal:** yes. Enemy buildings count as targets that can't shoot back, so units that can shoot back are picked first. M1 already behaves this way.
2. **Build menu slots.**
   - The DC's BuildMenu entries have no slots or hotkeys.
   - **Proposal:** Q Generator, W Barracks, E Extraction Facility, in the order of `> Constructs`, plus Z Back and X Cancel. When a building is ready, its own slot (for example, Q for a Generator) becomes "Select completed building".
3. **InfoPanels for buildings.**
   - Only the plate's InfoPanel is specified.
   - **Proposal:**
     - **For an own building:** name, HP, and what it is doing (construction or production with progress, queue length, the building ready to place, the rally point).
     - **For another player's building:** name, owner and HP.

#### Context
Buildings become targets and get panels in M2.

#### Options
- **Point 1:** A, enemy buildings are attacked automatically; B, only on an explicit Attack order.
- **Points 2 and 3:** A, as proposed; B, the Designer's layout.

---

## 6. Risks

| # | Risk | Mitigation |
|---|---|---|
| R1 | **Perf, nav rebuilds.** A full rebuild costs about 1 ms per grid on 256² (`docs/movement.md > 5.7`). With path knowledge, one structure death seen by 4 players can trigger up to 5 rebuilds (the true grid plus 4). A fight in a base can kill plates (85 HP) every few ticks. | E:<br>• rebuild at most once per player per tick;<br>• build knowledge grids lazily;<br>• measure the worst tick in `nav_perf` and J's `structures_perf`.<br>Add incremental patching only if the numbers demand it. |
| R2 | **Perf, vision and the client.** Structures are static vision sources, which cost nothing when unchanged (M1-V stamps). Records grow with plates, which is cheap. The ghost runs its check every frame (at most 16 tiles), and the overlay mesh is rebuilt only when the area changes. | Measure in J. |
| R3 | **Determinism.** New money, progress and spawn logic could diverge between runs. | • Integer-only economy (T6, T7).<br>• Commands apply in input order and re-check the bank.<br>• Producers spawn in id order, with a deterministic exit search.<br>• `state_hash` destructuring catches unhashed fields at compile time.<br>• `#[serde(default)]` keeps M1 replays loading (`deny_unknown_fields`). |
| R4 | **Golden churn.** B, E and F each add hashed state, so every golden changes several times. | • One re-bless per slice, after merging master, with each change explained (`ARCHITECTURE.md > Replays and golden hashes`).<br>• For parallel slices, the one that merges second re-blesses.<br>• J adds `gdo_opening`.<br>• No map-format bump (T11), because golden replays embed their maps. |
| R5 | **Defeat vs sandbox.** A literal "no buildings = defeated" would defeat every M1 test and golden player in tick 1. | The transition rule (T8). |
| R6 | **Merge conflicts.** G3 is splitting `mover.rs` and other files. Within M2, `command.rs`, `sim/gate.rs`, `sim.rs::step`, `testing.rs`, `object.rs`, `defs.rs` and `command_ui/interface.rs` are shared. | • Start M2 code only after G3 merges.<br>• Give each slice an ownership list (3.2).<br>• Run D2 after D1 if conflicts in W3 must be zero. |
| R7 | **The InterfaceState refactor (H1) regresses M1's command UI.** | The refactor is done only in H1. Every M1 UI test must stay green, and H2 builds on H1. |
| R8 | **A blocked Barracks exit.** Units spawning on top of other units would break the hard-collision invariant. | A ring search for a free spot around the building (Q0023). If nothing is free within a bound, hold the finished unit and retry each tick. Test with a fully walled Barracks. |
| R9 | **Layout completeness while factions are missing.** Read strictly, `dev_skirmish` would be unplayable until M5. | Check only implemented factions (T11), and tell the Designer as an FYI. |
| R10 | **Economy QA.** The design's opening numbers are approximate and use continuous income, while lumps shift the clean Barracks from about 46 s to about 48 s. Agents could "fix" the economy to match the numbers. | Oracle tests with documented tolerances (J). The golden pins the exact behaviour. Q0028 point 1 settles which reading is meant. |
| R11 | **UI QA.** Ghost colours, the overlay and the side labels need eyes. R, F and Shift in AwaitingPlacement clash with Shift meaning Queue (M1 bindings). | QA scripts with `--input-script` and `--screenshot`; the agent must look at the images. Add explicit binding tests in H2. |
| R12 | **0038 verification in M2.** A patch disappears only when its plate is destroyed after depletion. | B and E test exactly that, or use debug Remove. |
| R13 | **The test map.** The baseline assumes 8 patches in build range. `dev_skirmish` has 8 per side, but whether they are in range depends on the GDO layout and on the area shape (Q0026). | Check when authoring the layout. Move patches if needed (a Director task). |
| R14 | **Playability of "Visible" placement.** The DC's sight is 6 but its build area reaches 12, and the EF's sight is 3. Patches and spots inside the area may be under fog and unplaceable. | Raised in Q0027 point 2. Watch for it in the playtest. |
| R15 | **Typo in the plan.** `plan.md`'s M2 list says "Engineering Facility"; the design calls it the Extraction Facility. | Fix it when updating the plan. |

---

## 7. Director to-dos implied by this plan
1. Triage Q0022–Q0030 and send them; Q0022 is the only one that blocks, and only the supply part.
2. Update the M2 section of `plan.md`: the slice table, deferring supplies, and the typo.
3. Once G3 merges, brief A. Then brief B, C and E together.
4. Author `dev_skirmish`'s GDO layouts (with F's proposal and validation), and check the 8-patch reference (R13).

---

## 8. Design answers received after planning (they override the defaults above)

| Entry | Answers | Slices | What it settles |
|---|---|---|---|
| 0071 | none | H2 | Placement never moves on to another worker by itself. A GDO Shift-click places once and leaves placement. |
| 0077 | Q0022 | B, G | The supply loop is deferred. Starting Supplies are 0, and the HUD shows Supplies 0. |
| 0079 | Q0023 point 1 | A (`sides`), H2 (labels) | The side lettering convention. The Barracks is 3 wide × 2 tall at rotation 0. Rotations and flips carry the letters with them. |
| 0080 | Q0023 points 2–3 | D2 (spawn, pause, collision exception) | **The exit doorway.** A unit appears half inside the exit side. The releasing building ignores its own unit while the two overlap. Production pauses only while the doorway is occupied, and the finished unit is held. This replaces the nearest-free-spot default and R8's ring search. |
| 0081 | Q0024 point 1 | D2 | The limit of 20 includes CurrentBuild. |
| 0082 | Q0024 point 2 | D2, G | Unit Control is reserved when a unit is queued. It is released on cancel, on refund and on death. |
| 0083 | Q0024 point 3 | D2, H1 | **Group production.** The building with the least remaining production time pays first, ties broken by id, until a cost check fails. |
| 0084, 0085 | Q0024 point 4 | D1, D2 | A destroyed producer or constructor refunds everything as if cancelled, including a finished unit waiting at the doorway. |
| 0086 | Q0024 point 5 | D2, H1 | A rally point set on the building itself clears it. With no rally point, units walk out of the doorway and stop. |
| 0087 | Q0024 point 6 | H1 | Rally point indicators, shown while the building is selected. |
| 0088 | Q0024 point 7 | D2, H1 | A rally point on an enemy that leaves sight becomes a ground rally at its last-seen position, and the marker turns green. A rally point on a remembered structure is TBD (Q0017 point 2). |
| 0089 | Q0025 points 1, 2 and 4 | B | As proposed. |
| 0090 | Q0025 point 3 | G | The power text is consumed / generated, and turns red when consumption exceeds generation. This replaces "current / total". |
| 0091, 0092 | Q0026 points 1–3 | C | The build area is square and follows the current buildings. Without a DC, the other buildings still count. Point 4 (the overlay) is still open. |
| 0093 | Q0026 point 4 | H2 | The overlay shows the build area only. Whether a tile is valid right now shows on the ghost, tile by tile. |
| 0094 | Q0027 point 1 | C | A ground unit of any player standing on the footprint makes those tiles invalid, and own units are not pushed aside. |
| 0095 | Q0027 point 2 | C, F | Plates need Visible tiles too. Place `dev_skirmish`'s DCs so their sight (6 grid units plus the elevation modifier) covers the main patches. |
| 0096 | Q0027 point 3 | A, C | No building on a resource, except a plate on a patch. Resource tiles are invalid on every ghost. |
| 0097 | Q0027 point 4 | F, I | Starting Layout validation: the proposal, plus a rule for starting units. Read the entry. There are no visibility checks or sight warnings in the editor. |
| 0098 | Q0028 point 1 | B, J | Plates pay in lumps of 10. The cycle timer advances by the power ratio each frame. Economy tests expect lump timings. |
| 0099 | Q0028 points 2–4 | A, B | The last load takes what is left, then the residual 1 is mined. A depleted patch stays under its plate. A click on a covered patch selects the plate. |
| 0100 | Q0029 point 1 | F | A defeated player's units stay but can't be controlled. Clear their command queues and refuse every command to them. Alliances are unchanged (TBD). This replaces removing the units. |
| 0101, 0102 | Q0029 | F | **On defeat** (this replaces 0100's "units stay"): clear every unit's queue and transfer ownership to neutral; their vision no longer counts for anyone. The game ends when one side remains. In team games, a defeated player watches with the union of their allies' vision and has no command input. |
| 0103 | Q0030 point 1 | A (done) | Enemy structures are valid automatic targets. |
| 0104 | Q0030 point 2 | H2 | The DC has two build menus. M2 needs only Q Generator, W Barracks and E Extraction Facility, in BuildMenu[Basic]. Other slots are greyed placeholders or hidden. Idle, constructing and ready states work the same in both menus. |
| 0105 | Q0030 point 3 | G | The default structure InfoPanel, as proposed: own buildings show name, HP and activity; other players' buildings show name, owner and HP. |

---

## 9. Notes from merged slices for later slices

| From | For | Note |
|---|---|---|
| M2-E | F | When defeat transfers a unit to neutral (0101), call `Mover::repath`. Its path was planned on the old owner's knowledge grids, and neutral units plan on the true grids. |
| M2-E | J | `behaviours_soak` skips the "idle unit in reach has engaged" check in any tick with a death. Tighten it to skip only units whose target died that tick. |
| M2-E | B, D1, F | Own structures join their owner's planning grid at the end of the tick in which they are placed (known grids sync after `Vision::remember`). |
| M2-A | I | The editor places structures unturned and refuses a plate on a patch. M2-I adds orientation and the plate exception. |
| M2-C | F1 | Apply `paths::misplaced` to every layout's units. `MapDef::check` only checks a layout's terrain fit. `check_layout(map, others, layout, index)` is ready. |
| M2-C | F1, I | A layout's build area is its DC's alone, per `core/map_editor.md`'s "within the build area of the DC". The layout's EF doesn't extend it. |
| Director | D1, D2 | T9 is split into `Command::Construction` (D1) and `Command::Production` (D2), so the two slices can run in parallel. |
| Director | F | F is split into F1 (game start, wave 3) and F2 (game end, wave 4). F2 also takes the defeat and victory banner from G. |
| M2-D2 | F2 | A producer transferred to neutral keeps its queue, never advances it, and refunds nobody. Decide in F2 whether defeat should clear the queue, which 0101 says for units. |
| M2-D2 | J | Re-measure `movement_perf` and the other release perf tests on a quiet machine. Under parallel agent builds (load average about 22), master measured from 0.57 to 3.7 ms mean, with spikes up to 179 ms. |
| M2-D2 | H1 | API: `Sim::production_status`, plus `Command::Production { objects, command: Train(kind) | CancelProduction | SetRally(Target) }`. A rally on the building itself clears it. |
| M2-D1 | H2 | API: `construction::recipes`/`recipe`, `Constructor::{current, ready, frames_done}`, `Sim::validate`, `PlacementCheck::first_problem`. Events: `PlacementRefused`, `ConstructionCompleted`, `StructurePlaced`, `ConstructionCancelled { refund }`. Place takes exactly one constructor. |
| Director | all | Recipe timing: an order applied in tick t starts progress that tick; a recipe of N frames completes at the end of t+N−1, and the result is usable from t+N (ARCHITECTURE.md > Economy > Recipe timing). |

