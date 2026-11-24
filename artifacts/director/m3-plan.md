> **Status (Director, 2026-11-24):** This is the M3 planning spike's output, kept as the durable M3 spec. The drafts in section 5 were filed as Q0037–Q0043. The user approved M3 on 2026-11-24, with DragMovement (G) in M3. The Director accepted T14–T28 as written. Section 8 collects design answers, and section 9 collects notes from merged slices.

# M3 plan: the Colonists (planning spike M3-P)

> **Status (2026-11-24):** draft for the Director. Nothing filed or accepted yet.
> - Technical decisions are numbered T14–T28, after M2's T1–T13.
> - The questions are drafted as Q0037–Q0043. Q0038 is blocking (for slice H's acceptance and the oracle).

- **Date**: 2026-11-24. Read-only spike: no repository file was changed.
- **Code read at**: `11a3a4c` (master; M2 complete at `a956228`).
- **Sources**:
  - `artifacts/director/plan.md`: M3 under Queued ("Drone, Beacon, Warp In, Egress Node, portals, Legionnaires, shields"), the change table and the open questions. No change entry is queued for M3. Three entries queued for the vehicles milestone touch the Drone: 0033 (rotating silhouettes), 0035 (collision response) and 0039 (Rugged passability, hover double drag). Open questions run to Q0036.
  - `artifacts/director/insights.md`, `artifacts/director/m2-plan.md` (the template; T1–T13; §9).
  - Change entries up to 0105. None above 0105 was published while this spike ran (checked at the start and at the end).
  - `ARCHITECTURE.md`, all of it.
  - The design docs cited below. Paths are relative to `artifacts/designer/design/`, written as `file.md > Section`:
    - all of `factions/colonists/` (overview, economy, structures, units);
    - `core/entities.md`, `core/game_rules.md`, `core/tech_and_research.md`, `core/map.md`, `core/map_editor.md`, `core/camera.md`;
    - `systems/combat.md`, `systems/units.md`, `systems/controls/selection_and_hud.md`;
    - `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`.
  - The code:
    - sim: `object.rs`, `defs.rs` and `defs/`, `player.rs`, `economy/` (power, unit control, mining, defeat), `construction.rs`, `production/` and `sim/producers.rs`, `placement/`, `scenario/starting.rs`, `scenarios.rs`, `combat/damage.rs`, `orders.rs`, `command.rs`, `sim.rs` (the tick), `movement.rs`;
    - client: `command_ui/interface/` (templates, commands, panel, placing), `hud/resources.rs`, `cli.rs`, `editor/layouts.rs`.
- **"Silent"** means the design says nothing about the point. The plan then proposes a default and drafts a question (section 5).

---

## 0. Summary

- **Scope.** M3 builds the Colonists opening, the part of the faction the design states in full:
  - the Drone: gathering with patch occupancy, Set Drop-off, Drop Off, Deploy;
  - the Beacon, Beacon Deployment and Warp In (Egress Node, Robo Portal, Bio Portal);
  - the Egress Node as the drop-off;
  - Robo Portal (Drone) and Bio Portal (Sabre Legionnaire) production, with exits on several sides;
  - Sabre and Rifle Legionnaires with Weapon Conversion;
  - Shields on every Colonists object: damage resolution and regeneration;
  - Beacon Capacity;
  - Colonists Starting Conditions and Starting Layouts, the Colonists HUD and panels;
  - the Drone's DragMovement (own track);
  - an economy oracle against `Economy Targets (GDO vs Colonists)`.

  **Deferred**, because their stats are TBD in the design: refining (Alloys, Essence, Flux), the tech buildings and research, the Skylance and Plasma Turret, the Scout, the Lancer and the psionic unit, the Heavy and Star Portals.
- **Slices.** There are 12, in 6 waves:
  1. **A**
  2. **C** ∥ **E** ∥ **H** (Fable)
  3. **B** ∥ **D** ∥ **F** ∥ **G** (Fable, own track)
  4. **J1** ∥ **J2** (gives the first playable)
  5. **K1**
  6. **K2** (QA, then the user playtest)
- **First playable** after A–F, H, J1 and J2. On `dev_skirmish` with `--factions colonists,gdo`, you start with an Egress Node, a Robo Portal and four Drones. You gather by right-click, train Drones, deploy a Beacon, Warp In a Bio Portal, train Sabres and convert them, and fight GDO through F2.
- **Key decisions.**
  - **T15:** one object through every transformation. The kind changes in place, keeping the id, and the target footprint is claimed at the start.
  - **T16:** Warp In and Worker-Built are methods of the one placement check, plus a knowledge mode for the Deploy ghost and gate.
  - **T17:** Drones get Gather, Drop Off and Deploy orders and a Worker capability. Their timed work and bank credits run in the economy phase.
  - **T18:** Shields are resolved when a hit is applied, not when it is fired.
  - **T19:** exit rules for portals.
  - **T20:** Beacon Capacity is a derived army cap beside Unit Control.
  - **T25:** DragMovement is built in M3 on a Fable track, because the targets include Drone acceleration.
- **Readiness: mostly ready.**
  - The opening loop is statted: costs, times, HP, Shields, sizes, gathering numbers, Starting Conditions, the shield rules and the economy targets. About 45 rule gaps can take documented defaults (Q0037, Q0039–Q0042).
  - One gap is blocking: whether gathering Drones collide with each other (Q0038). The targets treat patch occupancy as the only limit, but under hard collision, with gathering Drones that never yield, crowding will lower them. It is blocking for H's final behaviour and the oracle, though not for starting.
  - Refining, the tech tier and the other units are not ready, so they are deferred.

---

## 1. Scope

### 1.1 In M3

| Area | Content | Design |
|---|---|---|
| Drone | Unit, HoverVehicle, 36x36, HP 20, Shields 10, armour 0/0, sight 5, Ungroupable, BeaconCapacityCost 1, unarmed. Commands: Gather, Drop Off, Set Drop-off, Deploy, Move, Stop. | `factions/colonists/units.md > Drone` |
| Gathering | Pick up at a patch, deliver to the AssignedDropOff (an Egress Node). CarryCapacity 8, MiningDuration 32, DropOffDuration 2, MiningReach and DropOffReach 8 su. One Drone per patch, from any player; redirect to the nearest unoccupied patch, or wait. | `units.md > Drone > Gathering`, `> Patch Occupancy (one Drone per patch)`, `> Assigned Drop-off` |
| Beacon | 2x2, HP 120, Shields 10, sight 4, Groupable, AAAA, provides 5 Beacon Capacity | `factions/colonists/economy.md > Beacon` |
| Beacon Deployment | Deploy order to any location, fog included. Worker-Built validation on arrival. 48 frames; HP and Shields interpolate; vulnerable; cancel reverts; completion is one-way. | `economy.md > Beacon Deployment`; `core/entities.md > Placement Validation > Worker-Built Structures` |
| Warp In | Beacon to Egress Node (75 SC, 320 frames), Robo Portal (400, 640) or Bio Portal (150, 480). Ghost centred on the Beacon, rotate and flip. HP and Shields interpolate; cancel refunds 75% rounded down and reverts; destroyed means both lost; every building keeps 5 capacity. | `economy.md > Warp In`; `core/entities.md > Placement Validation > Warp In` |
| Egress Node | 2x2, HP 200, Shields 10, sight 3, Ungroupable. Any number of Drones drop off at once. F: Select Returning Drones. | `economy.md > Egress Node` |
| Robo Portal | 4x4, HP 600, Shields 20, armour 1/16, sight 4, AAAA. Queue of 20, rally, Drone 50 SC and 160 frames; exits on any side. | `factions/colonists/structures.md > Robo Portal` |
| Bio Portal | 3x3, HP 250, Shields 15, armour 1/9, sight 3, AABB. Queue of 20, rally, Sabre 100 SC and 160 frames; exits on the A sides. | `structures.md > Bio Portal`, `> Exit Behavior` |
| Legionnaires | HeavyInfantry, 36x36, HP 70, Shields 10, cost 2, speed 5. Sabre: melee, 15 damage. Rifle: ranged, 10 damage, range 4. Weapon Conversion: 4 frames, no cost. | `units.md > Sabre Legionnaire`, `> Rifle Legionnaire`, `> Weapon Conversion` |
| Shields | MaxShields per object. Damage hits Shields first at full value; the overflow goes through the armour rules with the minimum capped at the overflow. Regen 0.25 per frame. | `systems/combat.md > Shields`, `> DamageModifierOrder`, `> MinimumDamage` |
| Beacon Capacity | Used / available, 5 per Colonists building, max 200. A unit that would exceed it can't be built. | `factions/colonists/overview.md > Beacon Capacity`; `economy.md > Beacon Capacity Retention` |
| Start | Egress Node, Robo Portal and 4 Drones; 100 SC; Alloys, Essence and Flux at 0; the camera starts on the Robo Portal | `economy.md > Starting Conditions`; `core/camera.md > Starting Position` |
| Layouts | Colonists Starting Layouts on Skirmish Default maps (`dev_skirmish`) | `core/map_editor.md > Starting Layouts` |
| HUD and panels | DisplayHud: SC, Alloys, Essence, Flux, Beacon Capacity. Panels for the Drone, Beacon, Egress Node, portals and Legionnaires. GatherResources and ReturnResources pointers. | `overview.md > DisplayHud`; the ObjectInterfaceStates above; `systems/controls/selection_and_hud.md > PointerDisplayType` |
| Drone movement | DragMovement (forward 0.6, omni 0.2, drag 0.1, 45° per frame, 8 su per frame); Rugged crossed with double drag; 0035's collision response | `units.md > Drone > UnitBaseAttributes[HoverVehicle]`; `systems/units.md > DragMovement`, `> HoverVehicle`; changes 0035 and 0039 |
| QA | Colonists oracle against the targets; goldens; Drone perf | `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)` |

### 1.2 Deferred

| Item | Why |
|---|---|
| Refining: Alloy Furnace, Essence Refinery, Flux Reactor; the feeding and hauling jobs; PickUpDuration; Waiting at a Full Refinery | Costs, WarpInTimes, stats, storage and rates are TBD (`economy.md > Constructs List`, `> Refining`). PickUpDuration is TBD (`units.md > Gathering`). The waiting Drone raises a notification, and the notification system isn't designed (`> Waiting at a Full Refinery`). The HUD shows Alloys, Essence and Flux at 0. |
| Exophysics Engine, Fusion Core, Relay Array, Navigation Spire, Psionic Amplifier, Structural Analyzer; research; shield regen upgrades | Stats, costs and research are TBD (`structures.md`); "per-level values TBD" (`systems/combat.md > ShieldRegenRate`) |
| Skylance, Plasma Turret; tech prerequisites | TBD. They are the faction's only prerequisites (`structures.md > Tech Model`), so M3 needs no prerequisite system. |
| Scout, Lancer, psionic unit | TBD (`units.md > Scout`, `> Lancer`; `structures.md > Bio Portal > Produces`) |
| Heavy Portal, Star Portal, portal upgrades | "Upgrade cost, upgrade time, and behavior during an upgrade ...: TBD" (`structures.md > Portal Upgrade Line`); footprints open (`> Open Questions`) |
| Costs in other resources (`Recipe::cost` is SC only) | Every M3 cost is in Space Crystals |
| 0033 rotating silhouettes | The Drone and the Legionnaires are square, as infantry is (`plan.md` change table: "square infantry is nearly unaffected"). Stays with vehicles. |
| Notifications (a Deploy that fails on arrival, a waiting Drone) | Not designed; M3 uses M2's feedback line |
| AI opponent | Undecided (insights) |

### 1.3 What already exists (so the slices don't redo it)

**Sim**
- **Object model.** An `Object` with optional capabilities (placement, resource, orders, mover, attack, riot_shield, extractor, production, constructor) and an `ObjectDef` static per kind. `ObjectKind` is appended to only, because a kind's position is part of the hash. `Faction::Colonists` exists.
- **Structures.**
  - `StructureDef { symmetry, build_radius_extension, power, production, mining, constructs }`.
  - `ProductionDef { exit: SideLetter, recipes }`: one exit side.
  - `Recipe { kind, cost (SC), frames }`.
  - `construction::constructor_for(kind)` gives every type with a non-empty `constructs` the GDO constructor state machine (idle, constructing, ready).
- **Production** (`production/`, `sim/producers.rs`):
  - a queue of 20 (0081) and progress in `RATIO_ONE`;
  - Train, CancelProduction and SetRally, with least work served first (0083);
  - refunds on destruction (0084, 0085), rally rules (0086, 0088);
  - one exit doorway (0080; `production/exit.rs`, `Mover::exit`, `release_unit`, `doorway_spot`).
- **Economy.**
  - Banks: SC and Supplies.
  - Power: `PowerSummary::speed` gives full speed unless power is negative, so a faction at Power 0 works at full speed.
  - Unit Control: GDO only (`economy/unit_control.rs`), reserved at queue time (0082).
  - Plate mining and depletion (`depleted_patch`, 0038).
  - `Refund::{Full, Ready}`: Ready is 75% rounded down, exactly the Warp In cancel rule.
- **Placement.**
  - One pure check (`placement::check`) in `Mode::Command { vision, player }` and `Mode::Layout`, giving tile and whole problems (T4).
  - The build area applies to GDO buildings only (`needs_build_area`), so Colonists kinds pass it.
- **Combat** (`combat/damage.rs`):
  - `Hit { attacker, target, damage }`, with the damage resolved when the shot is fired (`hit_damage`), then applied in attacker id order. It already carries `TODO(Colonists): Shields take the hit first`.
  - Melee uses `attrs.range` with no ElevationModifier.
  - The directional multiplier exists.
- **Orders.** `OrderKind { Move, Attack, AttackMove, Patrol, Hold, Stop }`; behaviours run in 5a on a `World` snapshot. `Command::Ability` exists for things that aren't orders (the Riot Shield).
- **Start and end of a game.**
  - Starting Conditions exist for the GDO only (T11).
  - `GameSetup::per_slot(map, factions, seed)` defaults to GDO; scenario files carry each player's faction.
  - `scenarios::dev_skirmish` and `maps/dev_skirmish.json` have GDO layouts and empty Colonists ones. `scenarios::empty` (the tests) is a Custom map.
  - Defeat (T8) compares structure owners at the start of the tick with those after the economy phase.
- **Movement.** One pipeline. `MovementAttrs::Drag` exists and gives the Drone `max_speed()` 8. Only TurnRateMovement is modelled ("TODO(movement models)"). `HOVER_VEHICLE` has `rugged_terrain: false`, which contradicts the design (0039, queued). `has_turret: true` meets the test `turret.is_some() == base.has_turret`, which the Drone breaks, because it has no turret.
- **The sim Rng**: PCG32, seeded from the setup, hashed.
- **Tests and QA.**
  - `TestSim`, `ScenarioBuilder`.
  - `testing::build_order`: GDO constructors and producers only.
  - The GDO oracle (`tests/economy_oracle.rs`); goldens with the `Script` hook; `structures_perf` at 0.28 ms mean; 926 tests.

**Client**
- **Templates.** `InterfaceTemplate::of(kind)` covers BasicCombatUnit, Barracks, DeploymentCenter and ExtractionFacility. `PanelCommand`; `is_common`, `recipients` and `instance_only` implement "no smart-casting".
- **Placement.** `Placing { constructor, kind, orientation }`: the ghost follows the cursor, with R and F turns shared with the editor.
- **HUD.** `ResourceFigure::of(faction)`: factions other than the GDO show SC only.
- **CLI and QA.** `--scenario`, `--map`, `--commands`, `--input-script`, `--screenshot`, `--dump-state`; F2 switches the player.
- **Editor.** Layouts with a palette from the Starting Conditions; units are drawn by their silhouette.

**Files near or over 500 lines** (split before adding to them): `sim.rs` 588, `sim/gate.rs` 517, `movement.rs` 506, `scenario/validate.rs` 502, `defs/units.rs` 497, `defs.rs` 495; client `command_ui/systems.rs` 592, `editor/layouts.rs` 585, `fog.rs` 562.

---

## 2. First playable

After A, B, C, D, E, F, H, J1 and J2 (end of wave 4):

1. `cargo run -- --scenario dev_skirmish --factions colonists,gdo` (C adds `--factions`). Player 1 starts as the Colonists in slot 1:
   - an Egress Node, a Robo Portal and four Drones;
   - 100 SC, Alloys, Essence and Flux at 0;
   - Beacon Capacity 4/10;
   - the camera on the Robo Portal.
2. Select the Drones and right-click a patch.
   - Each claims a patch; a Drone that arrives at an occupied patch goes to the nearest free one, or waits.
   - They carry 8 at a time to the Egress Node, and the bank rises at each drop-off.
3. Select the Robo Portal and press Q to train Drones (50 SC, 10 s each).
   - A rally on a patch sends new Drones to gather.
   - At 10/10 the next Drone is refused.
4. Select a Drone (Tab steps through Drones), press A and click a spot.
   - The Drone walks there and deploys into a Beacon in 3 s.
   - Beacon Capacity becomes 9/15.
5. Select the Beacon, choose Warp In, then Bio Portal.
   - The ghost is centred on the Beacon; R and F turn it.
   - A click starts the 30 s Warp In; HP and Shields rise.
   - X cancels with a 75% refund.
6. Select the Bio Portal and press Q to train Sabre Legionnaires. R converts them to Rifles and back.
7. Press F2 to play the GDO player: build a Barracks and Peacekeepers, then fight. Shields take hits first and regenerate.
8. Destroy every Colonists building (Beacons count): the Colonists are defeated and their Drones become neutral.

**Not needed for it:**
- G: until it lands, the Drone moves at its top speed of 8 su per frame with no acceleration, and Rugged stays impassable for it.
- K1 and K2.

**Earlier checkpoint (scripted, after A, C, D and H, around wave 3).**
- No gathering UI yet: a `--commands` QA file gives the Gather orders.
- Screenshots show Drones mining, the bank rising and a Robo Portal producing.
- The same as a test: a Colonists start whose 4 Drones gather for 60 s and deliver at least 6 loads each. The exact figure is measured and recorded in H.

**How it is played and tested without an AI**
- **Hot-seat.** The user plays both sides with F2 (insights: during development the user controls all sides).
- **Scripted QA.** `--commands` and `--input-script` files, with screenshots at fixed ticks.
- **Golden games through the `Script` hook.** `colonists_opening` scripts one player; `colonists_vs_gdo` scripts both players into a fight (K1).
- **The Colonists oracle** (K1) plays a build order through the gate and checks the targets.

---

## 3. Slices

### 3.1 Overview

| # | Slice | Content | After | Model |
|---|---|---|---|---|
| A | Colonists foundation | Seven kinds with stat blocks; `max_shields` and BeaconCapacityCost in the defs; `Sim::change_kind`; the HoverVehicle stand-in; the melee contact constant; placeholder visuals; debug spawns | M2 done; user go-ahead | Opus |
| B | Shields and Weapon Conversion | Shield state, damage resolution and regen; Convert ability, 4-frame mode and kind swap; shield bars; R on Legionnaires; the client's response to `KindChanged` | A | Opus |
| C | Colonists start | Beacon Capacity (a derived army cap), Colonists banks, Starting Conditions, `dev_skirmish` layouts, `--factions`, the Colonists resource bar | A | Opus |
| D | Portals | Exit rules (closest side to the rally, random fallback), per-door occupancy, Drone and Sabre production under Beacon Capacity, rally for unarmed units, portal panels | C | Opus |
| E | Transformations and Warp In | `Transform` capability, Warp In and Cancel commands with their gate, placement methods, progress, interpolation, cancel, destruction | A | Opus |
| F | Beacon Deployment | Deploy order, knowledge-mode gate, validation on arrival, unit to structure, Cancel Deployment | E | Opus |
| G | Drone movement | DragMovement, 0035's collision response for Drag units, Rugged with double drag for hover | A; own track | **Fable** |
| H | Gathering | Gather and Drop Off orders, SetDropOff ability, Worker capability, occupancy, mining and drop-off in the economy phase, depletion, soak | A | **Fable** |
| J1 | Warp In and Deploy UI | Beacon panel and Constructs menu, centred ghost, Drone Deploy ghost (knowledge mode), cancels, transform progress, regrouping after a kind change | E, F | Opus |
| J2 | Gathering UI | Drone right-click (Gather, Drop Off, Move), pointers, Set Drop-off targeting, Egress Node F, carried load shown | H, D | Opus |
| K1 | Colonists oracle and goldens | Colonists build orders, the reference map, the oracle against the targets, two goldens | D, F, H, G | Opus |
| K2 | M3 QA | Drone perf and soak, QA screenshots, ARCHITECTURE review, ignored tests, playtest notes | all | Opus |

### 3.2 Waves and parallel tracks

```
W0  M2 done (a956228); M2 playtest; user go-ahead for M3
W1  A
W2  C | E | H (Fable)
W3  B (after A) | D (after C) | F (after E) | G (Fable; own track, starts when a build slot is free)
W4  J1 (after E, F) | J2 (after H, D)           -> first playable (B and C are in by then)
W5  K1 (after D, F, H, G)
W6  K2 (after everything)                      -> user playtest
```

- **Critical path**: A → E → F → J1. H (Fable) runs alongside it, and so do C and D.
- **Machine load.** Insights record a load average of 22 with several agents building, and timing flakes under it. Run at most three builds at once: G waits for a slot, and B can move into W2 if the machine is quiet.
- **Shared files in W2–W3.** These touch shared files, so expect small merge conflicts, as in M2's W3:
  - `sim/gate.rs` and the `every_command` tests: B, E, F and H all add commands;
  - `object.rs`: B adds shields, E adds the transform and H adds the worker;
  - `events.rs` and `state_hash`.

  If the Director wants no conflicts, run F after H, at the cost of one wave.

**What each slice owns** (for the "don't touch" lists):

| Slice | Owns | Read-only |
|---|---|---|
| A | `defs/colonists.rs` (new), the Colonists entries in `defs.rs` and `defs/units.rs`, `Sim::change_kind`, placeholder visuals | — |
| B | `combat/damage.rs`, `shields.rs` (new), `conversion.rs` (new), the Modes step, client shield bars and the Legionnaire R | `defs/` |
| C | `economy/army.rs` (from `unit_control.rs`), `player.rs`, `scenario/starting.rs`, `scenarios.rs`, `maps/`, client `cli.rs` and `hud/resources.rs` | `production/` |
| D | `production/` (exit rules, doorways, rally), `sim/producers.rs`, the client's production template | `economy/army.rs` |
| E | `transform.rs` (new), `placement/` (methods), the Warp In commands in `construction.rs` and the gate | — |
| F | The Deploy order and behaviour in `orders/`, the knowledge mode in `placement/` | `transform.rs` |
| G | `movement*`, `mover.rs`, the HoverVehicle flags in `defs/units.rs`, the hover nav class | everything else |
| H | The Gather behaviour in `orders/`, `economy/gathering.rs` (new), the Worker capability, `Ability::SetDropOff`; `movement/collide*` only if Q0038's answer needs it (agree with G) | — |
| J1 | `command_ui/` (Beacon and Drone templates, `Placing`), client `placement/`, the selection regrouping | — |
| J2 | `command_ui/resolution*`, `command_ui/cursor.rs`, the Egress Node template, indicators | — |
| K1 | `testing/` (Colonists build orders), `tests/economy_oracle*.rs`, `tests/golden.rs` | — |
| K2 | perf tests, QA scripts, `ARCHITECTURE.md`, playtest notes | — |

**Every slice follows the Definition of done** (`ARCHITECTURE.md > Definition of done`):
- clippy, fmt and tests are clean;
- doc comments cite the design, and placeholders cite their question;
- `ARCHITECTURE.md` is updated for new conventions:
  - Object model: transform, worker, shields;
  - Tick pipeline: the transform step, shield regen, gathering;
  - Economy: Beacon Capacity, drop-offs;
  - Production: exit rules;
  - Placement: methods and the knowledge mode;
  - a new "Adding a faction" checklist, started by C;
- goldens are re-blessed once after merging master, with each change explained (`> Replays and golden hashes`).

### 3.3 Slice details

#### A: Colonists foundation (Opus)

**Goal.** The seven Colonists kinds exist with their full stat blocks. They can be created by debug spawn, in scenarios and in layouts.
- Their structures block ground units, give sight, are remembered in fog and can be destroyed.
- The Legionnaires fight with their weapons.
- The Drone moves on a stand-in model.

No Shields behaviour, economy or transformation yet.

**Design**
- Unit stat blocks: `factions/colonists/units.md > Drone` (UnitBaseAttributes[HoverVehicle], TurretAttributes None, AttackAttributes None), `> Sabre Legionnaire` and `> Rifle Legionnaire`.
- Structure stat blocks: `economy.md > Beacon` and `> Egress Node`; `structures.md > Robo Portal` and `> Bio Portal`.
- Unit bases: `systems/units.md > HoverVehicle` and `> HeavyInfantry`.
- Melee: `systems/combat.md > Range Measurement` ("Melee (adjacent contact) is a gap of about 0") and `> Melee` (no ElevationModifier).
- `core/entities.md > Structure Type`: AAAA and AABB, lettered clockwise from the top (0079).

**Code**
- **Kinds.** A new `defs/colonists.rs`, in the pattern of `defs/gdo.rs`: one static per kind, each value with its design line. Append Drone, Beacon, EgressNode, RoboPortal, BioPortal, SabreLegionnaire and RifleLegionnaire to `ObjectKind` and `ALL`.
- **New fields** (T14):
  - `ObjectDef::max_shields: Option<f32>`;
  - `UnitDef::beacon_capacity_cost: Option<u32>`;
  - `StructureDef::warp_in: &'static [Recipe]`, the Beacon's Constructs list. It is not `constructs`, which would make the Beacon a GDO-style constructor.
- **The four structures.** Power 0, no build radius. The portals get `production` recipes, with a single exit side as a placeholder until D.
- **Melee contact.** A named placeholder for the Sabre's range (Q0042 point 5).
- **Drone movement** (stand-in, T25). The Drone gets `MovementAttrs::Drag { 0.6, 0.2, 0.1, 45° }`. The existing step moves it at `max_speed()` = 8 su per frame within its turn rate.
  - `HOVER_VEHICLE` keeps `rugged_terrain: false` until G.
  - The turret rule in `defs_are_consistent` becomes `turret ⇒ base.has_turret` (Q0042 point 10).
- **`Sim::change_kind(id, kind, placement)`** (T15). The one function that changes an object's kind:
  - it rebuilds the capabilities from the new def;
  - it keeps the id, owner, position and facing;
  - it emits `SimEvent::KindChanged { id, from, to }`.

  Unit-tested in the sim; B, E and F use it.
- **Splits.** Split the tests out of `defs.rs` and `defs/units.rs` first; both are near 500 lines.
- **Client.**
  - Placeholder meshes with 2-letter tags (`object_render.rs`), selection panel labels and editor field arms.
  - `InterfaceTemplate::of`: the Legionnaires get BasicCombatUnit; the other kinds get `None` until D, J1 and J2.
  - Every exhaustive match over kinds.

**Acceptance**
- **Defs.** One test per kind checks the values against the design, as `the_m1_units_follow_the_design` does. The consistency test is extended:
  - Colonists structures have power 0 and no build radius;
  - `max_shields` is set for exactly the Colonists kinds;
  - AAAA ⇒ square;
  - the Bio Portal's sides follow 0079 in all 8 orientations.
- **Spawns.** Each kind spawns for two owners. Structures block the ground grids, with sight 4, 3, 4 and 3 for the Beacon, Egress Node, Robo Portal and Bio Portal. Drones and Legionnaires have sight 5.
- **Combat.**
  - A Sabre kills a Peacekeeper at contact, with no elevation bonus to its range.
  - A Rifle fires at 4 GU, with the ElevationModifier.
- **Movement.** A Drone moves at 8 su per frame on the stand-in, and paths around a Robo Portal.
- **`change_kind`.** The id is kept, the capabilities are rebuilt, the hash changes and the event is emitted.
- **QA screenshot.** One of each kind for two owners, from a scripted `--commands` file.
- **Goldens.** No change is expected: the new kinds go last, and def fields aren't state. If one changes, explain it.

**After**: M2 done and the user's go-ahead. **Model**: Opus. The work is wide but shallow, mostly data.

#### B: Shields and Weapon Conversion (Opus)

**Goal.** Colonists objects carry Shields, which take damage first at full value and regenerate. Legionnaires convert between Sabre and Rifle in 4 frames. The client shows shield bars, offers R, and redraws an object whose kind changed.

**Design**
- `systems/combat.md`:
  - `> Shields`: MaxShields; ShieldRegenRate, 4 per second or 0.25 per frame, "continuously (not gated by a delay after taking damage)"; `> Shield Damage Resolution`;
  - `> DamageModifierOrder`: "Colonists Shields are resolved before all of this";
  - `> MinimumDamage > Shield overflow`: "HP damage = max(overflow - armor, min(minimum, overflow))";
  - `> AttackPhase`: Firing and Cooldown are not interruptible.
- `units.md > Weapon Conversion`: Convert Command, Cost, HP/Shields, Vulnerability, Eligibility, Completion. Also `> Sabre Legionnaire > ObjectInterfaceState` (R).
- `selection_and_hud.md > CommandPanel > CommonCommands` (a mixed Sabre and Rifle selection).

**Code**
- **State.** `Object::shields: Option<f32>`, hashed with `serde(default)`. It is filled from `max_shields` when an object is inserted, and kept by `change_kind`.
- **Damage** (`combat/damage.rs`, T18).
  - `Hit` carries the raw damage and the target's modifiers: pre-armor, PointArmor and directional.
  - `apply` resolves each hit in order: Shields first at full value, then the overflow through `hit_damage`, with the minimum capped at the overflow.
  - `SimEvent::Hit` gets `shield_damage`.
- **Regen.** A new `shields.rs` with the regen step: 0.25 per tick, up to the maximum. It runs right after the dead are removed in 5e and before the economy. ARCHITECTURE > Tick pipeline is updated.
- **Conversion.** A new `conversion.rs` with `Ability::Convert` (T26).
  - The gate keeps the player's Legionnaires that aren't in Firing or Cooldown. It drops the others, and rejects the command if none remain.
  - The unit holds still and stays attackable for 4 ticks: a converting mode handled in the Modes step, next to `riot_shield::step`. Its attack sequence is reset.
  - Then `change_kind` makes it the other form. HP and Shields carry over unchanged (the maxima are equal), and the orders are kept (Q0042 point 6).
  - `sim.rs` is at 588 lines: move the Modes step into `sim/modes.rs` first.
- **Client.**
  - A shield bar above the HP bar (placeholder colour), and Shields in the selection panel and the InfoPanel.
  - `PanelCommand::Convert` in slot R for both Legionnaire kinds. Being one command, it is common for a mixed selection.
  - `KindChanged` re-meshes the object and regroups the selection. B is the first user of this.

**Acceptance**
- **The design's cases**, with exact binary values:
  - 15 damage on 10 Shields with armour 0: Shields reach 0 and HP drops by 5;
  - 25 damage on 20 Shields with PointArmor 1 (a Robo Portal): HP drops by 4;
  - 10.25 damage on 10 Shields with PointArmor 1: HP drops by 0.25, because the minimum is capped at the overflow;
  - 8 damage on 10 Shields: no HP loss, and armour isn't consulted;
  - non-Colonists targets are unchanged, so every M1 and M2 combat test passes.
- **Same-tick hits.** With two hits in one tick, the second sees the Shields the first left.
- **Regen.** Shields go from 0 to 10 in 40 ticks and never above the maximum. Dead objects don't regenerate. `state_hash` covers Shields.
- **Conversion.**
  - A Sabre becomes a Rifle in 4 ticks, keeping HP and Shields.
  - The command is rejected in Firing and Cooldown, and accepted while Aiming, Reloading, moving or idle.
  - A mixed group converts each unit to its other form.
  - A conversion can't be cancelled, and the unit is attackable throughout.
  - The replay is identical.
- **QA screenshots.** Shield bars on a Robo Portal after a hit and after regen; a Sabre and a Rifle side by side.
- **Goldens.** Every golden changes, because every object gets a new hashed field. Re-bless and explain.

**After**: A. **Model**: Opus.

#### C: Colonists start (Opus)

**Goal.** A Colonists player can start a game: banks, Beacon Capacity, Starting Conditions, layouts on `dev_skirmish`, a way to pick factions, and the Colonists resource bar.

**Design**
- `economy.md > Starting Conditions` and `> Beacon Capacity Retention`.
- `overview.md > DisplayHud` and `> ColonistResources > Beacon Capacity`: "Total Beacon Capacity caps at 200"; "The player cannot build units if doing so would exceed the total available Beacon Capacity".
- `core/map_editor.md > Starting Layouts`: the Skirmish Default restriction, layout validation and outdated layouts.
- `core/camera.md > Starting Position`: "Colonists: starting Robo Portal".
- `core/entities.md > Player > DisplayHudInfo[Faction]`.

**Code**
- **Army caps** (T20). `economy/unit_control.rs` becomes `economy/army.rs`, with `ArmyCap { used, available }` per faction:
  - GDO: Unit Control, as now;
  - Colonists: Beacon Capacity. Available is min(200, 5 × own Colonists structures that are complete or warping in). Used is the BeaconCapacityCost of own units plus what portal queues reserve (Q0041 point 1).

  Add `Sim::army_cap(player)`, and a refusal reason in `plan_training`.
- **Banks** (T21). `Player` gets `alloys`, `essence` and `flux` (u32, hashed, `serde(default)`, starting at 0). `DebugCommand::SetResources` gets them too, with `serde(default)`.
- **Starting Conditions** (`scenario/starting.rs`). The Colonists table is Egress Node 1, Robo Portal 1, Drone 4; crystals 100; supplies 0; primary Robo Portal. Under T11, every Skirmish Default map then needs Colonists layouts (T24).
- **Layouts.** `scenarios::dev_skirmish` and `maps/dev_skirmish.json` get Colonists layouts for both slots, slot 2 being the R180 image. Proposed for slot 1:
  - the Egress Node at (4, 4), 2.1 to 3.5 GU centre to centre from the eight patches at (2,5), (2,6), (2,7), (3,3), (4,2), (5,2), (6,2) and (7,2);
  - the Robo Portal at (6, 6), on tiles 6–9;
  - four Drones on points south and west of the node.

  C checks them with `check_layout` and the unit-start rule, and adjusts.
- **Client.**
  - `--factions <f1,f2,...>` for `--scenario` and `--map`: it sets `PlayerSetup::faction` per slot, and replays record the setup.
  - `ResourceFigure` for the Colonists: Space Crystals, Alloys, Essence, Flux and Beacon Capacity used/available, with no Power.
- **Editor.** The Colonists layout palette comes from the table, and Drones are placed as units. A test covers it.
- **ARCHITECTURE.md.** Start the "Adding a faction" checklist: defs file, Starting Conditions, layouts, army cap, HUD figures, templates.

**Acceptance**
- **Start.** A Colonists player starts with 100 SC and 0/0/0, with the Egress Node, Robo Portal and 4 Drones where the layout puts them. Beacon Capacity reads 4/10, and the camera starts on the Robo Portal.
- **Validation.** `dev_skirmish` validates with GDO and the Colonists in any slot combination, with no outdated flag. An incomplete Colonists layout is `LayoutMissing`.
- **Capacity.** With 10 available and 4 used, 6 Drones queue at the Robo Portal and the 7th is refused (the bank is set by debug). Group production serves least work first up to the cap (0083: "any cost").
- **HUD.** The GDO bar is unchanged. The Colonists bar reads "Space Crystals 100 ... Beacon Capacity 4/10", in ASCII.
- **QA screenshot.** A Colonists vs GDO start on `dev_skirmish`.
- **Goldens.** `Player` gets three hashed fields, so every golden changes. Re-bless and explain.

**After**: A. **Model**: Opus.

#### D: Portals (Opus)

**Goal.** The Robo Portal and the Bio Portal release their units through the side the design picks. Unarmed units follow the Colonists' rally rules. Both portals have panels.

**Design**
- `structures.md > Robo Portal`: "a warped-in Drone may emerge from any of the 4 sides". Its RallyPoint behavior includes "rally on enemy -> Move to that location".
- `structures.md > Bio Portal > Exit Behavior`: the eligible A side closest to the RallyPoint. With no RallyPoint, or with that side blocked, a random unblocked A side.
- `core/entities.md > Structure Type > Exit Doorway`: buildings with several exit-eligible sides choose by their own rules; "a side whose doorway is occupied counts as blocked; pause only when every eligible doorway is occupied".
- `selection_and_hud.md > CommandPanel > Standard Slot Assignments` (X, C) and `> Production Building Default Right-Click`.

**Code**
- **Exit rules** (T19). `ProductionDef::exit: SideLetter` becomes `exits: ExitRule`:
  - `Side(B)` for the Barracks, unchanged;
  - `Nearest(&[A])` for the portals. Under AAAA every side of the Robo Portal is an A side.
- **Doorways** (`production/exit.rs`). `Doorway::of(building, side)`, with occupancy per door by geometry (`Doorway::holds`); `Mover::exit` keeps the producer id. `release_unit` picks the door:
  - the free door closest to the rally point (Q0041 point 4);
  - otherwise a random free door from the sim Rng, drawn only when two or more are free;
  - if no door is free, it pauses.
- **Rally** (`production/rally.rs`).
  - An unarmed unit rallied on an enemy gets a Move to the point, not to the object.
  - A rally on a patch gives a Move to the object until H adds Gather.
- **Client.** A `Production` template built from the `ProductionDef` recipes: Train in Q, W and so on, X Cancel and C Rally (T23). It serves the Barracks, the Robo Portal and the Bio Portal.

**Acceptance**
- **Training.** The Robo Portal trains a Drone: 50 SC is paid at queue time. Ordered in tick t, the Drone is finished and appears at the door closest to the rally at the end of tick t+159, and acts from t+160 (the spawn-tick rule). The Bio Portal trains a Sabre the same way.
- **Door choice.**
  - A rally to the east gives the east door.
  - With no rally, the door is random by seed and the same across runs.
  - If the chosen door is blocked, another free door is used.
  - If all doors are blocked, the portal pauses at 100%.
- **Bio Portal.** It exits only from its A sides, in all 8 orientations.
- **Rally.** A Drone rallied on an enemy moves to the point. A Sabre rallied on an enemy attacks it.
- **Removal.** Destroying a producing portal refunds its queue (0084, 0085) and frees the reserved capacity.
- **QA screenshots.** Drones leaving a Robo Portal by several sides; the Bio Portal's two exits.
- **Goldens.** GDO goldens don't change, since the Barracks keeps `Side(B)`. Explain any change.

**After**: C (Beacon Capacity). **Model**: Opus.

#### E: Transformations and Warp In (Opus)

**Goal.** A Beacon warps into an Egress Node, a Robo Portal or a Bio Portal.
- The cost is paid at confirm.
- The target footprint is claimed at once.
- HP and Shields interpolate over the WarpInTime.
- A cancel refunds 75% and reverts the Beacon.
- Destruction loses both.

**Design**
- `economy.md > Warp In`: Warp In Command, Constructs List, HP/Shields Interpolation, Vulnerability, Cancel Warp In, Beacon Capacity Retention.
- `core/entities.md > Placement Validation > Warp In` and `> All placement methods: no buildings on resources`.
- `core/tech_and_research.md`: a Warp In counts only once complete. Nothing in M3 depends on it.
- `core/game_rules.md > Defeat Condition`: Beacons count.
- `ARCHITECTURE.md > Economy > Recipe timing`.

**Code**
- **Transform state** (T15). A new `transform.rs` with `Object::transform: Option<Transform>`. `Transform` holds:
  - `from` (kind) and `from_placement`;
  - `progress` in `RATIO_ONE`, and `total`;
  - `paid`;
  - for deployment, the Drone's load (F).
- **Transform step.** It runs in the economy phase, after construction (T5 position).
  - Progress advances one `RATIO_ONE` per tick: the Colonists have no power (T21).
  - The formulas set the maxima, and the current HP and Shields rise by the same amount (Q0039 point 2).
  - Completion clears `transform` and turns on the target's functions: Train is accepted, drop-offs count.
- **Commands** (T22). `ConstructionCommand::WarpIn { kind, origin, orientation }` names one Beacon. `ConstructionCommand::CancelTransform` goes to every transforming object of the selection (X).

  The gate checks that:
  - the Beacon is the player's, complete and not transforming;
  - the kind is in its `warp_in` list;
  - the bank covers the cost;
  - the placement check with `Method::WarpIn { beacon }` (T16) passes on the truth.
- **Placement** (`placement/check.rs`).
  - `Method` is one of Direct, `WarpIn { beacon }` or `WorkerBuilt { worker }`.
  - For Warp In, every footprint tile must be Buildable, flat and unoccupied (structures, resources and ground units), except the Beacon's own tiles.
  - There is no visibility rule (Q0040 point 3) and no build area.
  - The origin and orientation are inputs: centring the ghost is the client's job (J1).
- **Start.** `change_kind(beacon, kind, placement)` in the commands phase, before 5b, so no extra grid sync is needed. Then `transform = Some(...)`.
- **Cancel.** `Refund::Ready` (75% rounded down), then `change_kind` back to a Beacon on `from_placement`. HP and Shields follow Q0039 point 3.
- **Destroyed.** The normal removal path, with no refund (Q0039 point 7).
- **Beacon Capacity.** A building being warped in keeps providing 5.
- **Splits and client.** Split `sim/gate.rs` (517 lines) by command family first. On the client, the InfoPanel shows "Warping in: Robo Portal 45%", and the object gets a placeholder tint while it warps.

**Acceptance**
- **Timings.** For each of the three buildings: the cost is paid; ordered in tick t, the Warp In is complete at the end of t+N−1 and the building produces from t+N.
- **Interpolation.** HP and Shields at 0, 25, 50 and 100% follow the formulas. For example, a Robo Portal at 50% has HP 360 and Shields 15. With damage taken in between, the current values follow the default.
- **Footprints.**
  - A 4x4 Robo Portal centred on a 2x2 Beacon needs its ring of 12 tiles clear.
  - A unit, a patch or a structure on the ring refuses it, with the tile problem.
  - The Beacon's own tiles are ignored.
- **Cancel.** The refund is 75% rounded down: 75 → 56, 150 → 112, 400 → 300. The Beacon returns on its old tiles.
- **Destroyed mid-Warp-In.** The object is removed with no refund, capacity drops by 5, and the player is defeated if it was their last structure.
- **Consistency.** The gate and `Sim::placement_check` for Warp In agree. `every_command` is extended, and replays are covered.
- **Goldens.** The new hashed field changes every golden. Re-bless and explain.

**After**: A. **Model**: Opus.

#### F: Beacon Deployment (Opus)

**Goal.** A Drone deploys into a Beacon at a chosen footprint.
- The order is accepted unless the player knows the site to be invalid.
- The Drone walks there and validates the site on arrival.
- It transforms in 48 frames.
- A cancel reverts it to a Drone.

**Design**
- `economy.md > Beacon Deployment`: Deploy Command, Placement, Deployment Duration, HP/Shields Interpolation, Vulnerability, Cancel Deployment, Completion.
- `core/entities.md > Placement Validation > Worker-Built Structures`.
- `units.md > Drone > Deploying` and `> Drone ObjectInterfaceState > A: Deploy`. The Drone intro: the Drone's own 1 capacity is freed when it deploys.
- `selection_and_hud.md > Shift while placing`: "a Drone becomes the Beacon, so each takes exactly one placement".

**Code**
- **Order.** `OrderKind::Deploy { origin: TilePos }`, which can be Shift-queued.
- **Gate.** The Drone is the player's, and the footprint is on the map. The placement check runs in `Mode::Known { player }` (T16):
  - tiles never explored pass;
  - explored terrain, remembered structures and remembered resources count;
  - visible units count.

  This is the same answer as the ghost.
- **Behaviour.** In 5a, the Drone walks to the footprint's centre. On arrival (Q0040 point 1) it requests the deployment. A step right after 5a, before 5b, applies the requests in id order:
  - it validates on the truth with `Method::WorkerBuilt { worker }`, ignoring the Drone's own box;
  - if valid, it starts the transform: `change_kind(drone, Beacon, placement)`, dropping orders and mover, and keeping the load in `Transform` for a cancel;
  - if not, it cancels the order. The Drone idles, and a `SimEvent::DeployFailed` feeds the feedback line.
- **Progress.** E's transform step: 48 frames, interpolating from Drone to Beacon.
- **Cancel.** `CancelTransform` reverts to a Drone at the footprint's centre, with no order. Its box is free, because the tiles were claimed. HP and Shields follow Q0039 point 3.
- **Defeat.** A Beacon under deployment counts as a structure (Q0039 point 5).

**Acceptance**
- **Deploy in sight.** The Drone walks there, starts on arrival and completes in 48 ticks. HP rises from 20 to 120 linearly, and Shields stay at 10 (10 to 10). At completion, used capacity drops by 1 and available rises by 5.
- **Deploy into unexplored fog.** The order is accepted. If a structure stands there, the order is cancelled on arrival and the Drone idles.
- **Occupied site.** A ground unit on the footprint at arrival cancels the order. A patch refuses it at the gate if the player knows it, or on arrival if not.
- **Cancel at 50%.** A Drone stands at the centre again, with HP and Shields per the default, and capacity is unchanged.
- **Destroyed mid-deploy.** One object is removed, and capacity is unchanged.
- **Shift.** A Deploy queued behind a Move runs after it.
- **Footprint claimed.** A unit ordered across the footprint during deployment goes around it.
- **Coverage.** `every_command` and replays.
- **Goldens.** Re-bless if hashed state changed, and explain.

**After**: E. **Model**: Opus.

#### G: Drone movement (Fable)

**Goal.** The Drone moves as a HoverVehicle:
- DragMovement with its acceleration, drag and top speed of 8;
- 0035's collision response for Drag units;
- Rugged terrain crossed with double drag, the hover part of 0039.

Other units are unchanged.

**Design**
- `systems/units.md > DragMovement`: omni thrust any way, forward thrust along the facing, drag against the movement. "Ground hover units ... actively thrust in the opposing direction to decelerate and change direction."
- `systems/units.md > HoverVehicle`: RuggedTerrain true with double drag. `> Which tile counts`: the tile under the centre.
- `systems/combat.md > LocomotionOrientationConstraints[DragMovement]`.
- `units.md > Drone > UnitBaseAttributes[HoverVehicle]`.
- Change entries 0035 (blocked motion dropped, velocity to 0) and 0039.
- `docs/movement.md > 8`, the invariants. ARCHITECTURE > Movement and collision > Hooks: "other movement models replace the direction → facing and velocity part of `step_unit`".

**Code**
- **The model.** A Drag model in `movement/`: thrust towards the steering point, drag applied per tick, braking by opposing thrust, and arrival planned from the stopping distance. The integration scheme is fixed and documented (T25).
- **Collision response.** 0035 for Drag units: blocked motion is dropped and the velocity along it zeroed.
- **Rugged.** `HOVER_VEHICLE.rugged_terrain = true`, with the matching nav class. Drag doubles while the centre's tile is Rugged. `base_flags_follow_the_design` is updated.
- **Invariants.**
  - I3 (speed) uses the model's maximum.
  - I5 and I7 allow a Drag unit with no goal to brake to a stop. Amend the text in `docs/movement.md` explicitly.
- **Splits.** `movement.rs` is at 506 lines: split it first.

**Acceptance**
- **Speed.** From rest, the speed curve follows the scheme. Equilibrium is 8 su per frame within 1%, and 4 on Rugged.
- **Stopping and arrival.** The stopping distance from top speed is within the documented bound. A Drone arrives at a point and within the 8-su reach of a patch without overshoot or oscillation.
- **Rugged.** A Drone crosses a Rugged strip at half speed.
- **Collision.** A Drone driven into a wall slides, with the blocked motion dropped (the 0035 Verification points that apply to hover).
- **No regressions.** Every M1 and M2 movement test passes unchanged, and `movement_perf` is compared A/B with master.
- **QA.** A screenshot sequence of a Drone accelerating and stopping.

**After**: A, on its own track. **Model**: Fable.

#### H: Gathering (Fable)

**Goal.** Drones gather Space Crystals:
- they walk to a patch and mine it for 32 frames, one Drone per patch;
- they carry 8 to their AssignedDropOff and drop off in 2 frames;
- the bank is credited, and they repeat.

Set Drop-off, Drop Off, redirects and waiting work, and patches deplete.

**Design**
- `units.md > Drone`:
  - `> Gathering`: the job table, mining row only;
  - `> Patch Occupancy (one Drone per patch)`;
  - `> Assigned Drop-off`;
  - `> Unit Commands`.
- `economy.md > Egress Node > Drop Off`: "the only way resources enter the player's resource pool".
- `core/map.md > SpaceCrystalsPatch`: a depleted patch disappears; 0038 reopens its tile.
- `systems/units.md > Who yields`: units busy with a stationary task, mining included, don't yield.
- `systems/controls/commands.md` and `behaviors.md` define no Gather or Drop Off, so the behaviour comes from `units.md` alone.

**Code**
- **Orders** (T22).
  - `OrderKind::Gather(ObjectId)` targets a patch.
  - `OrderKind::DropOff(ObjectId)` targets an own Egress Node.
  - `Ability::SetDropOff(ObjectId)` doesn't interrupt (T17).
- **Worker capability.** `Object::worker: Option<Worker>` holds the load, the drop-off, the patch and the timed work. The behaviour is `Behaviour::Gather` with the phases ToPatch, Waiting, Mining, ToDropOff and DroppingOff.
- **5a.** The behaviour moves the Drone with `MoveGoal::Object` and an 8-su reach, for MiningReach and DropOffReach. On arrival at a patch, the Drone claims it if it is free. A patch is taken when a Drone is already mining it, or when a lower-id Drone claimed it earlier this tick. Otherwise the Drone redirects to the nearest free known patch within the default radius (Q0037 point 5), or waits.
- **Economy phase.** A new `economy/gathering.rs`, after plate mining:
  - advances Mining (32) and DroppingOff (2);
  - takes min(8, remaining) from the patch when mining ends;
  - removes a depleted patch through the removal path, then syncs;
  - credits the bank when a drop-off ends, with `SimEvent::ResourcesDelivered`.
- **Drop-off assignment.** A Gather assigns the nearest own Egress Node when the Drone has none. If the node is destroyed, the Drone re-assigns at its next return (Q0037 point 8).
- **Queries and rally.** `Sim::returning_drones(node)` serves the Egress Node's F (J2). A rally on a patch gives Gather (Q0037 point 9).
- **Collision.** Built as the design reads now: hard collision, and busy units don't yield. If Q0038 is answered with pass-through for gathering Drones, it goes into `movement/collide*`, agreed with G.
- **Perf.**
  - No per-tick searches.
  - The redirect scan runs only on arrival at an occupied patch.
  - Two path requests per trip.

**Acceptance**
- **One Drone,** with the patch 2 GU from the node: trips of 8, and the bank rises by 8 at the end of each drop-off. Measure the trip time and record it in the module docs.
- **Occupancy.**
  - Two Drones arriving together: the lower id mines, and the other goes to the nearest free patch.
  - With no patch free, a Drone waits and mines when the patch frees.
  - An enemy Drone blocks a patch; GDO plates and units ignore the rule.
- **Set Drop-off** mid-trip redirects the return without interrupting mining.
- **Depletion.** The last load is partial. The patch disappears and its tile reopens, and Drones redirect.
- **Drop Off order.** The Drone delivers its load, then goes back to its patch (Q0037 point 1).
- **Node destroyed.** Drones re-assign, or idle if no node remains.
- **Soak.** 24 Drones on 8 patches for 5 minutes. No deadlock: every Drone delivers at least once every 30 s. Report the income against the design's estimates (saturation at 18–22 Drones; about 32 per second with about 24 working Drones).
- **Determinism.** Replays, the hash and `every_command`.
- **Goldens.** Re-bless (new fields) and explain.

**After**: A. **Model**: Fable. The hard part is crowds of busy workers that never yield. If the soak needs real work, split H into H1 (the loop) and H2 (crowding).

#### J1: Warp In and Deploy UI (Opus)

**Goal.** The player deploys Drones and warps in buildings from the panels, with ghosts. Cancels work.

**Design**
- `economy.md > Beacon`: "Full ObjectInterfaceState still TBD (presumably exposes the Warp In option)". The proposal is Q0040 point 8.
- `economy.md > Warp In > Warp In Command`: the ghost centred on the Beacon, rotate and flip, green or red. Also `> Cancel Warp In`, `> Cancel Deployment`.
- `units.md > Drone ObjectInterfaceState > A: Deploy`: the ghost follows the cursor, snapped to the grid; Escape or right-click cancels.
- `selection_and_hud.md`:
  - `> ObjectInterfaceState`: reset rules, and the per-tick validation of the state;
  - `> Shift while placing`;
  - `> No smart-casting`;
  - `> CommandPanel > Standard Slot Assignments` (X).

**Code**
- **Placing** (T23). It becomes `Placing { source, kind, orientation }`, where the source is a constructor, a worker or a Beacon. `origin_under` depends on the source: a Beacon's ghost is centred on it, and for a 3x3 the placement nearest the cursor is taken (Q0040 point 5).
- **Templates.**
  - Beacon: Q Warp In opens a menu with Q Egress Node, W Robo Portal, E Bio Portal and Z Back. While warping: X Cancel Warp In.
  - Drone: A Deploy (`instance_only`), S Set Drop-off (J2), Stop.
  - A Beacon under deployment: X Cancel Deployment.
  - New `PanelCommand`s: `WarpInMenu`, `WarpIn(kind)`, `Deploy`, `CancelTransform`.
- **Ghosts.** Deploy asks the sim for the `Mode::Known` check; Warp In uses the Warp In method. Tiles are coloured as in M2-H2.
- **Selection.** On `KindChanged`, regroup the selection: a selected Drone that deploys joins the Beacon group. Reset the interface state when it is no longer valid.
- **InfoPanel.** Transform progress.
- **Split.** `command_ui/systems.rs` is at 592 lines: split it first.

**Acceptance**
- **Deploy** (headless interface tests). A, the ghost follows the cursor, and a click sends `Deploy { origin }` to the active Drone only. A Shift-click sends one queued Deploy and ends placement. Escape and right-click cancel.
- **Warp In.** Q, the menu, then W: the ghost is centred on the Beacon, R and F turn it, and it is red when blocked. A click sends WarpIn; X cancels.
- **Several Beacons selected**: Warp In follows the default of Q0040 point 6.
- **QA screenshots.** The Deploy ghost green and red; the Warp In ghost of each building; a Beacon at 50% of a Warp In; a cancel.

**After**: E, F. **Model**: Opus.

#### J2: Gathering UI (Opus)

**Goal.** Drones are controlled by right-click like any RTS worker. The Egress Node selects its Drones, and the player can see what each Drone carries.

**Design**
- `units.md > Drone > Right-Click Resolution`:
  - a patch gives Gather;
  - an own node, while carrying, gives Drop Off;
  - an enemy or other object gives "Move to that object's location";
  - ground gives Move;
  - "issued to all selected Drones".

  Also `> S: Set Drop-off Location`.
- `economy.md > Egress Node > F: Select Returning Drones`.
- `selection_and_hud.md > PointerDisplayType > Resolution Rules`: GatherResources over a resource with a gatherer selected; ReturnResources over a drop-off while carrying.

**Code**
- **Right-click** (`command_ui/resolution`). Each Drone's right-click resolves as above. "Move to that object's location" is a Move to the point under the click.
- **Pointers.** GatherResources and ReturnResources.
- **S, Set Drop-off.** AwaitingTarget[SetDropOffLocation], where an own Egress Node is the valid target. It sends `Ability::SetDropOff` to the selected Drones (Q0040 point 9).
- **Egress Node template.** F is a StateOnlyTransition that replaces the selection with `Sim::returning_drones`. It is hidden when there are none.
- **Carried load.** A placeholder marker on Drones that carry a load. The InfoPanel shows "Carrying 8 Space Crystals" and the drop-off.
- **Rally.** The rally indicator on a patch.

**Acceptance**
- **Right-click.** A headless interface test for each case. In a mixed selection of Drones and Sabres, each unit resolves its own right-click, as M2 does per unit.
- **F** selects exactly the returning Drones, in every phase of their Gather.
- **Set Drop-off** targeting works, and an invalid target is ignored, with the feedback line.
- **QA screenshots.** The pointers over a patch and over a node; Drones carrying loads; the result of F.

**After**: H, D. **Model**: Opus.

#### K1: Colonists oracle and goldens (Opus)

**Goal.** Measure the Colonists opening against `Economy Targets (GDO vs Colonists)`, and freeze two scripted games.

**Design**
- `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`: targets 1–4, the reference check and the income table. The GDO figures come from M2-J1's oracle.
- `units.md > Drone`: "roughly every 6th Drone must deploy as a Beacon to keep Drone production going".
- `units.md > Patch Occupancy`: the saturation estimate.

**Code**
- **Build orders.** Colonists build orders in `testing/`: Train at portals, Gather for new Drones (or a rally on a patch), Deploy at fixed spots, Warp In. They go through the gate, as `build_order` does, with books that count drop-offs (T27).
- **Reference map.** 8 adjacent patches in a row, the Egress Node 2 GU from the row's middle (Q0038 point 3), and one Robo Portal producing Drones continuously. Every 6th Drone deploys.
- **Oracle.**
  - It measures income per second at 0:00, 0:40, 1:00, 1:20, 1:40, 2:20, 2:40, 3:00 and 4:00 (SC delivered over a window), with the Drone and Beacon counts.
  - It compares them with the design's table and with M2's GDO oracle.
  - It asserts targets 1–4 and prints every figure with its delta. The tolerance follows Q0038 point 4.
- **Goldens.**
  - `colonists_opening` scripts gathering, a deploy, a Bio Portal Warp In, training and conversion.
  - `colonists_vs_gdo` scripts both players through the `Script` hook into a fight with Shields.

**Acceptance**
- **Targets.** The oracle passes targets 1–4 and prints the table with deltas. A deviation beyond the tolerance goes to the Director as a question draft with the measured table; the code isn't tuned to fit.
- **Goldens.** Both are stable across runs, and both replay.

**After**: D, F, H, and G for the table (the targets include acceleration; see T25). **Model**: Opus.

#### K2: M3 QA (Opus)

**Goal.** M3 is ready for the user's playtest.

**Tasks**
- **`tests/drones_perf.rs`.** 2 × 40 gathering Drones and 20 Legionnaires fighting, with phase timings (`step_observed`). Compare A/B with master on a quiet machine (insights).
- **Soak.** 10 minutes of the `colonists_vs_gdo` script: no deadlock, and the hash is stable.
- **QA screenshots.** A walkthrough of the first playable, from a scripted `--input-script`.
- **`ARCHITECTURE.md` review**, with the "Adding a faction" checklist finished.
- **Ignored tests.** `cargo test --release --workspace -- --ignored --test-threads=1` (insights: ignored tests rot).
- **Playtest notes.** How to start a Colonists game, F2, and the known placeholders.

**After**: everything. **Model**: Opus.

---

## 4. Technical decisions for the Director

### T14. Colonist structures in M2's model (A)
- **Recommend:** reuse `StructureDef` per kind.
  - **Power** 0 (T21), and no `build_radius_extension`.
  - **Production** for the portals.
  - **Warp In list.** A new `warp_in: &'static [Recipe]` holds the Beacon's Constructs list. Keep `constructs` for GDO-style constructors: `constructor_for` gives every type with `constructs` the idle, constructing and ready state machine, and a Beacon transforms itself instead.
  - **Beacon Capacity provided** is a faction rule, derived, not a def field (`economy.md > Beacon Capacity Retention`: every Colonists building provides 5).
  - **Shields:** `ObjectDef::max_shields`.
  - **BeaconCapacityCost:** `UnitDef::beacon_capacity_cost`, next to `unit_control_cost` and `tunnel_space_cost`.
- **Alternatives:**
  - put the Warp In list in `constructs` and branch `constructor_for` by faction: two state machines in one field;
  - a generic `ArmyCost` on `UnitDef`: tidier for M4 and M5, but it churns M2 code now.

### T15. One object through every transformation (A, E, F, B)
- **Recommend:** Drone to Beacon, Beacon to building, and Sabre to Rifle all keep the `ObjectId`.
  - **`Sim::change_kind`** is the only place a kind changes. It rebuilds the capabilities from the new def, keeps id, owner, position and facing, and emits `SimEvent::KindChanged`.
  - **`Object::transform`** holds the progress and what to revert to: the kind, the placement, and the Drone's load.
  - **The object is the target kind from the first frame.** Its footprint is claimed at once, so no unit can walk onto a building in the making (Q0039 point 1). It counts as a structure for defeat, auto-targeting (0103) and records. Its functions wait for completion: production, drop-off, and capacity for a deploy.
  - **Timing.** Kinds change in the commands phase (Warp In, cancel) or in a step right after 5a (deploy start). Both come before 5b, so the grids need no extra sync. Completion doesn't change the footprint.
- **Alternatives:**
  - remove and insert, with a new id at each stage. The state is simpler, but selection, control groups, rally references and records lose the object, and the design says "the Drone unit itself transforms into the Beacon in place";
  - a separate site object beside the Drone or Beacon: two objects to keep consistent for HP, death and selection.

### T16. Placement methods in the one check (E, F)
- **Recommend:** `placement::check` takes a `Method`:
  - `Direct`: the GDO's, as now;
  - `WarpIn { beacon }`: the Beacon's own tiles are ignored;
  - `WorkerBuilt { worker }`: the worker's own box is ignored.

  It also takes a mode:
  - `Command`: truth plus visibility, as now;
  - `Known { player }`: the inputs are filtered to what the player knows. Never-explored tiles pass, remembered structures and resources count, visible units count. The Deploy ghost and the Deploy gate use it;
  - `Arrival`: the truth with no visibility, for the Drone's check on arrival;
  - `Layout`, as now.

  It stays one check with different inputs, so the ghost, the gate and the arrival can't drift (T4).

  Colonists layouts use the Colonists' rules: Buildable, flat, no overlap, not on resources, and starting units by the unit-start rule.
- **Alternatives:**
  - a function per method: the drift that T4 rejected;
  - a Deploy gate that checks only "on the map": the ghost and the gate would disagree.

### T17. Drones as workers (H, F)
- **Recommend:**
  - **Orders.** `Gather(patch)`, `DropOff(node)` and `Deploy { origin }` are orders, queued like the others. Gather never ends on its own: with orders queued behind it, it completes at the next drop-off (Q0037 point 12).
  - **Set Drop-off.** `Ability::SetDropOff(node)` isn't an order, because it "does not interrupt the Drone's current action" (`units.md > Assigned Drop-off`).
  - **State.** A `Worker` capability on Drones. `Behaviour::Gather` runs in 5a for the legs and the claims.
  - **Timed work.** Mining (32) and DroppingOff (2), the patch decrement and the bank credit run in a new economy-phase step after plate mining (T5). All bank changes stay in the economy phase, as M2's books expect.
  - **Occupancy** is derived: the Drones mining at the start of 5a, plus the claims made in id order during the tick. No owner is stored on the patch.
  - **Depletion** uses the removal path (`depleted_patch`, 0038), then `paths::sync`.
  - **Perf.** Two path requests per trip; a redirect scan, over the known patches, only on arrival at an occupied patch; nothing per tick.
- **Alternatives:**
  - gathering as a mode outside orders: Gather replaces orders and is queued with Shift, so it is an order;
  - crediting in 5a on arrival: the bank changes in two phases;
  - an occupant stored on each patch: one more reference to clean up on removal.

### T18. Shields (B)
- **Recommend:**
  - **State.** `Object::shields: Option<f32>` beside `hp`, and `ObjectDef::max_shields`.
  - **Resolution at application.** The shot builds a `Hit` with the raw damage and the target's modifiers. `damage::apply` resolves hits in attacker id order: Shields first at full value, then the overflow through `hit_damage` with the minimum capped at the overflow. Two hits in one tick then see each other's effect on Shields.
  - **Regen.** 0.25 per tick, in a step right after the dead are removed in 5e.
  - **Determinism.** f32 in a fixed order is deterministic, as HP already is, and 0.25 is exact.
  - **Persistence.** Hashed, with `serde(default)`.
- **Alternatives:**
  - one "effective HP": breaks the rule that armour doesn't reduce damage to Shields;
  - resolving Shields at the shot, as `hit_damage` does now: wrong with two hits in one tick;
  - integer Shields in fixed point: they must combine with f32 HP.

### T19. Portals and exits (D)
- **Recommend:**
  - **The rule.** `ProductionDef::exits: ExitRule`: `Side(letter)` for the Barracks (B, unchanged), `Nearest(&[letters])` for the portals (both A).
  - **Free doors.** At release, a door is occupied while a unit it released still overlaps the footprint through that side, judged by geometry. `Mover::exit` stays the producer id.
  - **Choice.** With a rally point, the free door closest to it: straight-line distance from the middle of the door; ties go top, right, bottom, left. Otherwise, or when the closest door is occupied, a random free door from the sim Rng.
  - **Rng draws** happen only when two or more doors qualify: fewer draws, fewer golden changes.
  - **Pause** when no door is free.
- **Alternatives:**
  - a door list stored per building: state that can go stale;
  - randomness by a hash of tick and id: deterministic, but a second source of randomness.

### T20. Army caps per faction (C, D)
- **Recommend:** generalise `economy/unit_control.rs` into `economy/army.rs`, with `ArmyCap { used, available }` per faction.
  - **GDO:** Unit Control, 200 available.
  - **Colonists:** Beacon Capacity. Available is min(200, 5 × own Colonists structures that are complete or warping in). A Beacon under deployment doesn't count.
  - **Used** is the units' cost plus the units queued, reserved at queue time as in 0082 (Q0041 point 1).
  - **Derived each time**, never stored, like power.
  - **One refusal reason,** with the faction's name in the feedback.
- **Alternatives:**
  - a second module beside Unit Control: two copies of one rule;
  - checking at production start (the Syndicate's rule): differs from the GDO for no stated reason.

### T21. Power and banks for the Colonists (A, C)
- **Recommend:**
  - **Power.** Colonists structures have power 0. `PowerSummary::speed` gives full speed at power 0 or above, so nothing changes. The Colonists bar shows no Power.
  - **Banks.** Add `alloys`, `essence` and `flux` to `Player` now: 0, hashed, `serde(default)`. They show on the bar, and `SetResources` sets them for QA.
- **Alternatives:** zeros shown in the HUD until refining. That saves a re-bless now and costs one later.

### T22. Commands (B, E, F, H)
- **Recommend:**
  - **Warp In.** `ConstructionCommand::WarpIn { kind, origin, orientation }` names one Beacon, as Place names one constructor.
  - **Cancels.** `ConstructionCommand::CancelTransform` goes to every transforming object of the selection: Cancel Warp In and Cancel Deployment, both on X.
  - **Orders:** Gather, DropOff, Deploy.
  - **Abilities:** SetDropOff, Convert.
  - `systems/controls/commands.md` defines none of these, so the Colonists docs are the spec.
- **Alternatives:** a new `Command::Transform` family: a cleaner name, but more gate code.

### T23. Faction HUD and command panels (C, D, J1, J2)
- **Recommend:**
  - **HUD.** `ResourceFigure::of(faction)` gets the Colonists' figures.
  - **Production template.** Built from `ProductionDef`, so the Barracks grid becomes data and the portals need no new grid.
  - **New templates:** Beacon, with a Warp In menu built like the DC's BuildMenu (0104); Drone; Egress Node. The Legionnaires use BasicCombatUnit plus R.
  - **`Placing`** gets a source (constructor, worker or Beacon), so the ghost knows whether it follows the cursor or stays centred.
  - **Selection.** The client regroups the selection on `KindChanged`.
- **Alternatives:** per-faction UI modules: duplicates the template machinery.

### T24. Starting Conditions and layouts for a second faction (C)
- **Recommend:**
  - **In one slice.** Add the Colonists table in C, and give `dev_skirmish` (in `scenarios.rs` and `maps/dev_skirmish.json`) Colonists layouts for both slots in the same slice. T11 makes every implemented faction's layout mandatory on Skirmish Default maps (`ScenarioProblem::LayoutMissing`).
  - **Factions.** Player factions already come from `GameSetup`, and scenario files carry them. Add `--factions` to the client for `--scenario` and `--map`; replays carry the setup.
  - **Test maps** from `scenarios::empty` are Custom, so they're unaffected. Maps newly created in the editor are Skirmish Default and will list the Colonists layouts as missing; the editor already shows that.
- **Alternatives:** keep the Colonists "not implemented" for maps until later. A game couldn't start as the Colonists on `dev_skirmish`.

### T25. Drone movement (A, G)
- **Recommend:** build DragMovement in M3, on its own Fable track (G).
  - **Why.** `Economy Targets` include "Drone acceleration", and momentum is the Drone's identity (`systems/units.md > HoverVehicle`: "momentum-heavy and slidey").
  - **Size of the effect.** From rest, with v ← v + 0.8 − 0.1·v, a Drone covers a 2-GU leg (128 su) in about 24 frames before braking; at constant top speed it takes 16. A round trip at 2 GU (two legs, 32 frames of mining, 2 of drop-off) then takes about 85 frames, against about 66 on the stand-in, so the stand-in's income per Drone runs about 30% high.
  - **The design's figures fit the slower trip.** Four Drones at about 84 frames per trip give 6.1 per second, the table's 0:00 figure, and 16 working Drones give about 23 per second, its "2 Drones per patch" figure. The occupancy note's "about 2 at 2 grid units" is the no-acceleration figure, and says so ("ignoring acceleration").
  - **Until G lands,** the Drone uses the stand-in: 8 su per frame, instant acceleration, 45° per frame. Rugged stays impassable for hover.
  - **G pulls the hover parts of 0035 and 0039 forward.** 0033 stays with vehicles, because the Drone is square. G fixes and documents the integration scheme: thrust, then drag, then position, with braking when the stopping distance reaches the remaining path.
- **Alternatives:** defer DragMovement to the vehicles milestone. K1 would then only assert the targets' shape and report the inflated table. A movement rework would land after the economy was tuned against the wrong numbers.

### T26. Weapon Conversion (B)
- **Recommend:** an ability (`Ability::Convert`), like the Riot Shield. It doesn't replace the unit's orders.
  - **Gate.** It leaves out units in Firing or Cooldown (`systems/combat.md > AttackPhase`: not interruptible).
  - **During.** The unit holds still for 4 ticks in a Modes step, and its attack sequence restarts.
  - **After.** `change_kind` to the other form; HP and Shields carry over. The unit resumes its order with the new weapon.

  Q0042 point 6 asks whether orders survive.
- **Alternatives:** an order (`OrderKind::Convert`) that replaces the queue unless Shift is held, after which the unit idles. Simpler, but the unit drops its attack target in the middle of a fight.

### T27. Tests and QA for a second faction (C, K1, K2)
- **Recommend:**
  - **Builders.** `ScenarioBuilder` and `TestSim` take a faction per player.
  - **Build orders.** A sibling of `testing::build_order`, `testing::colonists`: Train, Gather, Deploy and Warp In through the gate, with books that count drop-offs.
  - **Goldens** with the `Script` hook for both players.
  - **Perf:** `drones_perf`.
  - **Oracle:** the Colonists oracle beside the GDO one.
- **Alternatives:** debug-spawned bases in tests. Faster, but they don't exercise the gate.

### T28. Hashes, replays and file hotspots (all)
- **Recommend:**
  - **Kinds** go at the end of `ObjectKind`, and every new field gets `serde(default)`.
  - **Re-bless per slice.** Each slice that adds hashed state re-blesses the goldens once after merging master, explaining each change: B (Shields), C (Player banks), E and F (transform), H (worker). A changes none.
  - **Split files near 500 lines** before adding to them:
    - `defs.rs` and `defs/units.rs` (A);
    - `sim.rs`, moving the Modes step (B);
    - `sim/gate.rs`, by command family (E);
    - `movement.rs` (G);
    - `command_ui/systems.rs` (J1);
    - `editor/layouts.rs`, if C touches it.
- **Alternatives:** add every hashed field in A and re-bless once. That couples A to the designs of later slices.

---

## 5. Design questions (drafts in the Director's format)

**Blocking status**
- **Q0038 is blocking**, for H's final crowd behaviour and for K1's acceptance. H starts with the rules as the design reads now.
- **Q0039 is not blocking**, but answering points 1–3 before F starts avoids rework.
- **The others are not blocking.** The slices build the proposals as the current behaviour and cite the question.
- **Q0043 is FYI**, for the content M3 defers.

### Q0037: Drone gathering: the cases the loop doesn't define
- **Date**: 2026-11-24
- **Blocking**: no. Slice H builds the proposals.
- **Docs**: `factions/colonists/units.md > Drone > Gathering`, `> Patch Occupancy (one Drone per patch)`, `> Assigned Drop-off`, `> Unit Commands`; `factions/colonists/structures.md > Robo Portal > RallyPoint behavior`; `core/map.md > SpaceCrystalsPatch`.

#### Question
1. **After a Drop Off order** (right-click an own Egress Node while carrying), what does the Drone do? The docs are silent. **Proposal:** it goes back to the patch it was mining, if any; otherwise it idles.
2. **Gather while carrying a load.** **Proposal:** it delivers the load to its AssignedDropOff first, then goes to the new patch.
3. **A load when the Drone gets another order** (Move, Stop). **Proposal:** the Drone keeps it and delivers it at its next drop-off.
4. **Mining interrupted** (ordered away during the 32 frames). **Proposal:** nothing is collected, and the patch frees at once.
5. **"The nearest unoccupied patch to itself"**: among which patches? With no limit, a Drone would leave for any free patch on the map, even in an enemy base. **Proposal:** patches the owner knows (seen or remembered), not under a structure, within 6 grid units of the patch it arrived at (placeholder). Otherwise it waits.
6. **Waiting at an occupied patch: where, and in what order?** **Proposal:** where it stands, within MiningReach. The first Drone to wait gets the patch next; ties go by id.
7. **A patch that disappears** (depleted) while Drones go to it or wait. **Proposal:** treated as occupied, so the redirect rule applies. With no patch left, the Drone delivers any load and idles.
8. **No Egress Node**: Gather when the player has none, or the AssignedDropOff is destroyed. **Proposal:** when the Drone starts its return, it assigns the nearest own Egress Node. With none, it idles holding its load.
9. **Robo Portal rally on a patch.** `> Robo Portal > RallyPoint behavior` first says the unit gets "the default right-click command resolved against the RallyPoint target", which for a Drone on a patch is Gather. It then lists "rally on friendly/neutral object -> Move to object". These disagree for a patch. **Proposal:** Gather.
10. **A Drone rallied on an enemy**: "Move to that location". **Proposal:** a Move to the point where the enemy stands when the Drone appears, not a Follow. Please confirm.
11. **A patch under a GDO Extraction Plate.** **Proposal:** Drones can't gather it, because a right-click hits the plate, and it is never a redirect target. The GDO side of the same patch is Q0034.
12. **Orders Shift-queued behind a Gather.** Gather never ends, so a queued order would never start. **Proposal:** a queued order starts after the Drone's next drop-off.
13. **The last load of a nearly empty patch.** **Proposal:** the Drone mines the full 32 frames and takes what is left, fewer than 8.

#### Context
Gathering is new: `systems/controls/commands.md` and `behaviors.md` define no Gather or Drop Off, so `units.md` is the whole spec. The loop's numbers and the occupancy rule are clear. These points are the transitions around them.

#### Options
- **Point 1:** A, as proposed: back to its patch. B, idle.
- **Point 5:**
  - A, a radius around the patch, as proposed;
  - B, the same patch field (patches connected through neighbours);
  - C, any known patch.
- **Point 9:** A, Gather: the default right-click. B, Move to the patch, as listed.
- **Point 12:** A, after the next drop-off, as proposed. B, at once: the Gather is dropped and the load is kept.
- **Other points:** the proposal, or a rule you give.

### Q0038: Gathering Drones, collision and the economy targets
- **Date**: 2026-11-24
- **Blocking**: **yes**, for slice H's final behaviour and for the Colonists oracle (K1). H starts with the rules as they read now.
- **Docs**: `factions/colonists/units.md > Patch Occupancy (one Drone per patch)`; `systems/units.md > UnitCollision`, `> Who yields`; `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`.

#### Question
1. **Do Drones on a Gather job collide with each other?** Today every ground unit collides, and units busy with a stationary task, mining included, never yield (`> Who yields`). A Drone on a Gather job always has an active command, so it never yields, whether it is mining, waiting at an occupied patch or travelling.
   - At saturation the reference check puts about 24 working Drones in the 2-GU (128 su) strip between the row and the Egress Node.
   - There, one Drone mines in front of each patch and others wait beside it, while the rest cross the strip both ways.
   - Returning Drones from a 512-su row converge on a 2x2 node.

   **Proposal:** Option B. Drones on a Gather job pass through other Drones on a Gather job, and still collide with everything else.
2. **Reference check geometry.** "Egress Node 2 grid units from the row's center": edge to edge, or centre to centre?
   - Centre to centre leaves a 32 su gap between the node and the middle patches, narrower than a Drone (36 su).
   - The occupancy note's figures, about 2 Drones at 2 GU and 2.5 at 3 GU, fit legs of 2 and 3 GU of travel at 8 su per frame.

   **Proposal:** a gap of 2 GU between the node's edge and the edge of the row's middle patches, with the node on the row's long side.
3. **How close must the oracle come to the table?** **Proposal:** targets 1–4 must hold, and each income figure must be within ±15% of the table. A larger gap comes back to you as a question with the measured table, not as a code change.

#### Context
The targets (about 23 per second at 2 Drones per patch, about 32 at full saturation with about 24 working Drones) treat patch occupancy as the only limit. Under hard collision with busy units that never yield, crowding at the row and the node will lower the saturation and can jam. Every RTS with walking workers faces this; many let gathering workers pass through each other. This shapes slice H and is the main reason the oracle may miss the targets.

#### Options
- **Point 1:**
  - A, hard collision as now. Expect saturation and income below the targets, possible jams, and more pathing work.
  - B, gathering Drones pass through each other, as proposed. The targets become reachable; they still collide with other units and structures.
  - C, gathering Drones yield to each other only. Partial relief, and still congested at the node.
- **Point 2:** A, edge to edge, as proposed. B, centre to centre.
- **Point 3:** A, as proposed. B, a tolerance you give.

### Q0039: Beacon Deployment and Warp In: what the transforming object is
- **Date**: 2026-11-24
- **Blocking**: no. E and F build the proposals. Answers to points 1–3 before F starts avoid rework.
- **Docs**: `factions/colonists/economy.md > Beacon Deployment`, `> Warp In`, `> Beacon Capacity Retention`; `units.md > Drone` intro (the Drone's 1 capacity is freed when it deploys); `core/game_rules.md > Defeat Condition`; `core/tech_and_research.md`.

#### Question
1. **During deployment, is the object a unit or a structure, and when is the Beacon's footprint claimed?** **Proposal:** from the first frame it is a Beacon under deployment.
   - It is a structure on its 2x2 footprint: units can't walk onto it, it can't move, and it counts as a structure.
   - It keeps the Drone's identity, so it stays selected.
   - A cancel turns it back into the Drone at the centre of the footprint.

   Warp In works the same way: the larger footprint is claimed at the start.
2. **HP and Shields interpolation when damaged.** The formulas give the values of an undamaged object. **Proposal:** the formula sets the maximum, and the current value rises by the same amount each frame, so damage taken stays taken. Shields regenerate up to the current maximum.
3. **What a cancel restores** ("no HP/Shields loss"). **Proposal:** the original form's maxima, with the same fraction of HP and Shields it has when cancelled. Damage taken during the transformation stays.
4. **Beacon Capacity during transformations.** **Proposal:** a deploying Drone still uses 1, and its Beacon provides nothing until complete. A building being warped in keeps providing 5 (`> Beacon Capacity Retention`).
5. **Defeat.** Beacons count. **Proposal:** a Beacon under deployment counts too. A player who cancels their last such Beacon, owning no other structure, is defeated in that tick.
6. **A Drone carrying a load deploys.** **Proposal:** the load is lost when the deployment completes, and given back on a cancel.
7. **Destroyed during a Warp In: any refund?** **Proposal:** none. The docs say only that the Beacon is destroyed with it.
8. **A building being warped in** shows only Cancel, and its functions (production, drop-off) start at completion. **Proposal:** yes.

#### Context
The docs fix the duration, the formulas, vulnerability, cancel and the 75% refund. They don't say what the object is in between. These answers decide how E and F model it (one object whose kind changes at the start, plan decision T15), and they are costly to change after F.

#### Options
- **Point 1:**
  - A, a structure from the first frame, as proposed;
  - B, a unit until complete. Units may then walk onto the footprint during the 3 s, and completion would need a second check.
- **Point 2:**
  - A, the maximum interpolates and the current value rises with it, as proposed;
  - B, the current value keeps its fraction of the maximum.
- **Point 3:**
  - A, keep the fraction, as proposed;
  - B, restore the values from when the transformation started (damage during it is forgiven);
  - C, take away what the interpolation added, which could kill it.
- **Other points:** the proposal, or a rule you give.

### Q0040: Deploy and Warp In placement, and the Colonists panels
- **Date**: 2026-11-24
- **Blocking**: no. F, E, J1 and J2 build the proposals.
- **Docs**: `core/entities.md > Placement Validation > Worker-Built Structures`, `> Warp In`, `> Direct Placement`; `factions/colonists/economy.md > Warp In > Warp In Command`, `> Beacon`; `units.md > Drone ObjectInterfaceState`; `structures.md > Robo Portal`, `> Bio Portal` ("Full ObjectInterfaceState ... still TBD"); `systems/controls/selection_and_hud.md > No smart-casting`, `> Shift while placing`.

#### Question
1. **Deploy: where must the Drone be to start?** "The Drone walks to the location, then begins deploying in place." **Proposal:** it walks to the centre of the footprint. It deploys when it arrives there, or when it stops, blocked, with its box overlapping the footprint. Otherwise the order fails.
2. **"Unoccupied" for Worker-Built and Warp In: do ground units count?** Direct Placement says they do (0094); the other two methods just say "unoccupied". **Proposal:** yes. Every ground unit counts except the deploying Drone; air and underground units don't.
3. **Warp In and visibility.** Direct Placement requires Visible tiles; Warp In lists no such rule. **Proposal:** no visibility rule. Every tile lies within about 3 GU of the Beacon's centre and its sight is 4, so it rarely matters. Where a tile isn't visible, the ghost could reveal a hidden unit.
4. **The Deploy ghost over fog.** The ghost is "tinted green when valid / red when invalid", but the order may target fog. **Proposal:** the ghost and the command check what the player knows (never-explored tiles pass), and the arrival checks the truth.
5. **Centring a 3x3 ghost (Bio Portal) on a 2x2 Beacon.** No placement is exactly centred. The 4x4 Robo Portal and the 2x2 Egress Node centre exactly. **Proposal:** of the four 3x3 placements that cover the Beacon, the one nearest the cursor.
6. **Several Beacons selected.** Beacons are Groupable, and with no smart-casting a command goes to the whole ActiveGroup, but a Warp In ghost belongs to one Beacon. **Proposal:** Warp In is available only when the ActiveGroup holds one Beacon. The alternative is to make Beacons Ungroupable, like the Drone and the Extraction Facility.
7. **Shift with Deploy.** "A Shift-click sends it and ends placement like a plain click." **Proposal:** the Deploy is queued behind the Drone's orders; behind a Gather, it starts after the next drop-off (Q0037 point 12).
8. **Panels** (the docs say TBD). **Proposal:**
   - Beacon: Q Warp In, which opens Q Egress Node, W Robo Portal, E Bio Portal and Z Back. X Cancel Warp In while warping.
   - A Beacon under deployment: X Cancel Deployment.
   - Robo Portal: Q Drone, X Cancel Production, C Set Rally Point.
   - Bio Portal: Q Sabre Legionnaire, X, C.
   - Drone: A Deploy, S Set Drop-off, Stop in its BasicCombatUnit slot, Move in Q.
   - Egress Node: F, as written.
9. **Set Drop-off with several Drones selected.** The docs say "the selected Drone(s)", but Drones are Ungroupable and the panel shows one Drone. **Proposal:** every selected Drone.

#### Context
Worker-Built and Warp In are stated as lists of rules; these are the gaps between them and the interface. None blocks the sim work.

#### Options
- **Point 6:** A, one Beacon only, as proposed. B, make Beacons Ungroupable. C, a Warp In per Beacon from one ghost (not possible with centred footprints).
- **Point 9:** A, every selected Drone, as proposed. B, only the Drone the panel shows.
- **Other points:** the proposal, or a rule you give.

### Q0041: Portal exits and Beacon Capacity
- **Date**: 2026-11-24
- **Blocking**: no. C and D build the proposals.
- **Docs**: `factions/colonists/structures.md > Robo Portal`, `> Bio Portal > Exit Behavior`; `core/entities.md > Structure Type > Exit Doorway`, `> Structure Type` (lettering, 0079); `factions/colonists/overview.md > Beacon Capacity`.

#### Question
1. **When is Beacon Capacity checked?** "The player cannot build units if doing so would exceed the total available Beacon Capacity." **Proposal:** at queue time, with the cost reserved while queued, as GDO Unit Control does (0082).
2. **Capacity falls below what is used** (buildings lost). **Proposal:** units stay, and new units are refused. Units already queued keep their reservation and complete.
3. **The Robo Portal's exit side.** "A warped-in Drone may emerge from any of the 4 sides", but which? **Proposal:** the Bio Portal's rule over all four sides: the side closest to the RallyPoint; with none, or with that side blocked, a random unblocked side.
4. **"Closest to the RallyPoint".** **Proposal:** straight-line distance from the middle of each side to the rally point (an object's centre). Ties go top, right, bottom, left.
5. **The Bio Portal's AABB.** Lettering runs clockwise from the top (0079), so both A sides are adjacent: top and right in the default orientation. Is that intended? ABAB would put them on opposite sides. **Proposal:** build AABB as written.
6. **"Materializes".** **Proposal:** a Drone appears half inside at the doorway, by the Exit Doorway rule (0080), like any produced unit.

#### Context
Production reuses M2's queue, rally and doorway. These are the Colonists' differences.

#### Options
- **Point 1:** A, at queue time, as proposed. B, at production start, as the Syndicate does.
- **Point 3:** A, as proposed. B, a fixed side. C, always random.
- **Point 5:** A, AABB as written. B, ABAB.
- **Other points:** the proposal, or a rule you give.

### Q0042: Shields, melee range and Weapon Conversion
- **Date**: 2026-11-24
- **Blocking**: no. A and B build the proposals.
- **Docs**: `systems/combat.md > Shields`, `> DamageModifierOrder`, `> Range Measurement`, `> AttackPhase`; `factions/colonists/units.md > Weapon Conversion`, `> Sabre Legionnaire`; `systems/units.md > HoverVehicle`; `core/game_rules.md > On Defeat`.

#### Question
1. **Shields of neutral objects** (a defeated player's units, 0101). **Proposal:** they keep regenerating.
2. **Regeneration while transforming.** **Proposal:** up to the current interpolated maximum (Q0039 point 2).
3. **Which modifiers touch Shields.** "At full value": **proposal:** no pre-armor multiplier and no DirectionalArmor applies to Shields. The Drone's DirectionalArmor (HoverVehicle) applies to the overflow only.
4. **Shields display.** **Proposal:** a bar above the HP bar, and "Shields a/b" in the InfoPanel.
5. **Melee range.** "Adjacent contact", "a gap of about 0". **Proposal:** 8 su (the Proximity gap), as a placeholder. Exactly 0 is fragile with collision margins.
6. **Convert while the unit has orders.** **Proposal:** the orders stay. The unit holds still for 4 frames, then carries on with the other weapon.
7. **Shift with Convert.** **Proposal:** it means nothing; Convert isn't queued.
8. **The weapon after conversion.** **Proposal:** the new weapon starts ready. A reload in progress isn't carried over.
9. **A mixed group in which some units are Firing or in Cooldown.** **Proposal:** those units are left out and the others convert.
10. **HoverVehicle `HasTurret - true`, but the Drone's `TurretAttributes - None`.** **Proposal:** read HasTurret as "the base can carry a turret".

#### Context
The shield formulas are complete. These points are where Shields meet other rules, and the parts of conversion the docs leave open.

#### Options
- **Point 6:** A, keep the orders, as proposed. B, Convert replaces the orders, and the unit idles afterwards.
- **Point 8:** A, ready, as proposed. B, carry the reload over.
- **Other points:** the proposal, or a rule you give.

### Q0043: M3 scope: the Colonists content left TBD (FYI)
- **Date**: 2026-11-24
- **Blocking**: no for M3. Yes for the deferred content, when it is scheduled.
- **Docs**: `factions/colonists/economy.md > Constructs List`, `> Refining`; `structures.md` (tech buildings, `> Portal Upgrade Line`, `> Open Questions`); `units.md > Scout`, `> Lancer`; `core/map_editor.md > Starting Layouts > Layout validation`.

#### Question
1. **What M3 builds.** M3 builds the statted opening only.
   - It defers refining (Alloy Furnace, Essence Refinery, Flux Reactor), the six tech buildings, the Skylance and Plasma Turret, the Scout, the Lancer and the psionic unit, the Heavy and Star Portals, and research.
   - Their costs, times and stats are TBD.
   - The HUD shows Alloys, Essence and Flux at 0.

   Tell us if any of these should be in M3; they would need stats.
2. **The Bio Portal's cost** (already an open question in `structures.md`). At 150 it is the Colonists' first army building, so it affects how even the early game is against the GDO.
3. **Colonists Starting Layouts in the editor.** Our reading of "the faction's normal in-game placement rules, minus visibility": Buildable, flat, no overlap, not on resources, and no build area. Starting Drones follow the unit-start rule.
4. **Notifications.** A Deploy that fails on arrival, or a waiting Drone, uses M2's feedback line until notifications are designed.

#### Context
About 50 TBD notes in the Colonists docs are concentrated in the tech tier. M3 is planned around the part that is statted.

#### Options
- **Point 1:** A, as proposed. B, add an item; it needs stats first.

---

## 6. Risks

| # | Risk | Mitigation |
|---|---|---|
| 1 | **Gathering crowds.** Under hard collision, Drones that are mining, waiting or travelling on a Gather job never yield. About 24 of them in the strip between the patches and the node cause jams, deadlocks and saturation below the targets. | Q0038 (blocking); H on Fable with a soak of 24 Drones on 8 patches; a pass-through for gathering Drones ready to add; split H if needed. |
| 2 | **Momentum movement (G).** A coasting unit strains I3, I5 and I7; overshoot at arrival and in doorways; lane seeking assumes speed changes instantly. | G touches Drag units only; the stand-in ships first; invariants amended explicitly; `movement_perf` A/B; Fable. |
| 3 | **Oracle fidelity.** The design's table comes from an idealised model. The stand-in runs about 30% fast per Drone (T25); collision and paths slow things down. | K1 after G; assert the targets' shape (1–4) and report the table; deviations go to the Designer, not into the code (Q0038 point 3). |
| 4 | **Determinism.** Rng draws for exits, kind changes mid-tick, the order of claims, f32 Shields. | Draws only with two or more candidates; one `change_kind`; claims in id order; replays and goldens. `state_hash` destructures every type, so an unhashed field doesn't compile. |
| 5 | **Transformations cut across M2 assumptions:** a footprint fixed per object, unit and structure categories fixed per object, defeat counts, records and path knowledge with a footprint that grows (a remembered 2x2 Beacon against a 4x4 portal). | T15: kinds change only before 5b. Plan on knowledge, collide on the truth (M2-E). A test with a Warp In seen, then fogged. |
| 6 | **GDO-only code in M2:** the single exit side, SC-only `Recipe::cost`, Unit Control in `plan_training`, `build_order`, `constructor_for` keyed on `constructs`, the HoverVehicle flags and turret test, client exhaustive matches and templates, `Placing` for constructors only, `ResourceFigure::of`. | Each is assigned to a slice (A, C, D, E, J1, K1); A greps the exhaustive matches. |
| 7 | **Mandatory Colonists layouts.** Starting Conditions make them mandatory on Skirmish Default maps (T11), so a map without them stops validating. | C updates `dev_skirmish` in the same slice; `scenarios::empty` is Custom; the editor flags the rest. |
| 8 | **Golden churn.** B, C, E, F and H each change hashed state. | Re-bless once per slice after merging master, with explanations; the `colonists_*` goldens only in K1. |
| 9 | **Perf with many Drones:** path requests, crowd collision, the per-tick claims. | `drones_perf` (K2), A/B on a quiet machine; no per-tick searches (T17). |
| 10 | **Machine load.** Two Fable tracks plus Opus slices push the load past 20 and make timing flaky. | At most three builds at once; G when a slot frees; tests independent of machine speed. |
| 11 | **Design churn.** About 50 TBDs, and answers arriving in the middle of a slice. | Defaults as named placeholders citing their question; fold answers in with SendMessage (insights). |

---

## 7. Director to-dos implied by this plan

- **Questions.** File Q0037–Q0043 from section 5. Q0038 is blocking: ask for it first.
- **G.** Ask the user whether G (DragMovement) belongs in M3 (T25), since it pulls the hover parts of 0035 and 0039 forward from the vehicles milestone. Update that `plan.md` entry either way.
- **Plan.** Add the slices M3-A to M3-K2 to `plan.md` and decide T14–T28.
- **Before W1.** Confirm the user's go-ahead after the M2 playtest. Check `maps/` for other Skirmish Default maps (today only `dev_skirmish.json`).
- **Briefs.**
  - Give each slice the ownership table from 3.2 as its "don't touch" list.
  - Use Fable for G and H.
  - Tell H that Q0038's answer may add a collision exemption in `movement/collide*`, and agree it with G.

## 8. Design answers received after planning (they override the defaults above)

| Entry | Answers | Slices | What it settles |
|---|---|---|---|
| 0106 | Q0038 point 1 | H2, G, K1 | Gathering Drones of any player pass through each other at every step of the job. They still collide with everything else. General rule: two ground units found overlapping ignore each other until separated, with no push-apart. |
| 0107 | Q0038 point 2 | H, K1 | The reference geometry: 8 patches in a row, the 2x2 node centred on the long side, a 2-GU gap edge to edge. |
| 0108 | Q0038 point 3 | K1 | Targets 1–4 must hold, and each income figure within ±15% of the table. A larger gap goes back as a question with the measured table. The reference layout is only a benchmark: no gameplay rule may assume it. |
| 0109 | Q0039 point 1 | E, F | The object is the target structure from the first frame, with its footprint claimed at once. A cancel reverts it (Drone at the footprint centre; Beacon on its original footprint). |
| 0110 | Q0039 point 2 | E, F | Each frame, `current += (newMax − oldMax)`: damage stays taken, including damage from before the transformation, and is never rescaled. Shields regenerate up to the current maximum. |
| 0111 | Q0039 point 3 | E, F | On cancel, the original form returns with its own maxima and the same HP and Shields fraction: `max(1, original_max × current ÷ max)`. |
| 0112 | Q0039 point 4 (and Q0041 point 2 in effect) | C, E, F | A deploying Drone's 1 capacity is freed at deploy start, and the Beacon's 5 arrives at completion. A building being warped in keeps providing 5. A cancel takes the Drone's 1 back even over capacity. Over capacity is allowed and shown red: units stay, and none are built until there's room. |
| 0113 | Q0039 point 5 | E, F | Transforming structures count for defeat. Cancelling a deployment that was the player's only building defeats them in that tick; the cancel is never refused. |
| 0114 | Q0039 point 6 | F | A loaded Drone deploys without a detour. The load is kept during deployment, lost on completion, and returned on cancel. |
| 0115 | Q0039 point 7 | E | A building destroyed mid-Warp-In refunds nothing: both the Beacon and the cost are lost. A cancel keeps its 75%. M2's 0084 refund must not fire for a portal still warping in. |
| 0116 | Q0039 point 8 | E, J1 | A transforming object shows only Cancel, with no rally. Production, drop-off, research and abilities start at completion; only the 5 Beacon Capacity is active during a Warp In. Drones can't deliver to an Egress Node under Warp In. |
| 0117 | Q0040 point 1 | F | The Drone moves to the footprint centre and deploys on arrival. If it is blocked while overlapping the footprint, it deploys there; if it stops without overlapping, the order fails and it idles. The Beacon goes on the ghost's footprint. |
| 0118 | Q0040 point 2 | E, F | "Unoccupied" includes ground units of any player for every placement method. The building worker itself doesn't count, and air and underground units never block. |
| 0119 | Q0040 point 3 | E, J1 | **Changed from the proposal:** every Warp In tile must be Visible, as in Direct Placement. A non-Visible tile is red whatever stands there. |
| 0120 | Q0040 point 4 | F, J1 | The Worker-Built ghost follows the player's knowledge: remembered terrain and structures count, hidden units are ignored, and never-explored tiles pass. The order is always allowed; the truth is checked on arrival. |
| 0121 | Q0040 point 5 | E, J1 | **Replaces cursor snapping.** An odd-size footprint has its extras at the bottom-right by default; turns and flips transform it about the Beacon centre, and the exit sides rotate with it. **Director:** the sim derives the footprint from (Beacon, kind, orientation) in one function shared by the gate and the ghost, and `WarpIn` carries no `origin` (amends T22). |
| 0122 | Q0040 point 6 | E, J1 | Beacons are Ungroupable: each is its own SelectionGroup, and Warp In applies to the Tabbed-to Beacon. |
| 0123 | Q0040 point 7; Q0037 point 12, and point 5 in part | F | **Replaces the after-the-next-drop-off proposal.** A Gather ends on its own only when the Drone's patch is gone and no patch at all is within its search radius. The Drone then delivers any load and starts the next queued order (Deploy included) or idles. A Drone waiting at an occupied patch is still gathering. The radius is centred on the Drone, not the patch: a per-unit Drone stat, 6 GU placeholder. Today it is `orders/gather.rs::REDIRECT_RADIUS_GU`, centred on `around`. |
| 0124 | Q0040 point 8 | D, J1, J2 | **Amends J1's single menu.** The Beacon has Q Warp In (Basic) and W Warp In (Advanced):
- Basic: Q Egress Node, W Robo Portal, E Bio Portal, A Alloy Furnace, S Essence Refinery, D Flux Reactor, Z Back.
- Advanced: eight buildings, Q to F, then Z.

Buildings and units whose costs are TBD show in their slot, unavailable. A transforming object shows only X Cancel. The other grids:
- Robo Portal: Q Drone, W Scout, X, C.
- Bio Portal: Q Sabre, W Lancer, E psionic unit, X, C.
- Drone: Q Move, A Deploy, S Set Drop-off, X Stop.
- Egress Node: F Select Returning Drones. |
| 0125 | Q0040 point 9 | F, J1 | **Changes J1's Deploy dispatch.**
- Drones become Groupable.
- A Deploy given to a group goes to one Drone: the first in selection order (a plain click), or the first with no Deploy queued (a Shift-click).
- Shift-placement chains through the group and ends once every Drone has a Deploy. A single Drone ends after one Shift-click.
- Every ordered worker build leaves a lingering ghost, owner-only, until construction starts, is cancelled or replaced, or fails. It reserves its site against the owner's further placements but doesn't block units.

**Director:** the sim derives the ghosts from pending Deploy orders, current and queued, and the gate refuses an owner's Deploy whose footprint overlaps one of them, so validity is the same for every client. The client draws them. Agents and Recruits follow when those factions are built. |
| 0126 | Q0045 point 1 | G | An idle unit overlapping another moves off on its own until clear, and two idle units both move. Units with an active command (Hold Position and mining included) are never moved. Amends 0106's "nothing is pushed apart". |
| 0127 | Q0045 point 2; Q0037 point 1 | H3 | **Replaces the return-to-patch proposal.** An explicit Drop Off is only a delivery. Then the order completes: the next queued order starts, or the Drone idles at the drop-off. A Drone on Drop Off passes through gathering Drones (already built). |
| 0128 | Q0037 point 2 | H3 | **Replaces deliver-first.** A loaded Drone sent to Gather goes to the new patch or refinery and skips mining there, without taking occupancy. It then delivers as if it had just mined the load, and continues the loop. |
| 0129 | Q0037 point 3 | H3 | The load is cleared only by a delivery or by deploy completion (0114), never by a change of order. |
| 0130 | Q0037 point 4 | H3 | Mining interrupted before 32 frames gives no partial load, and the patch frees at once. |
| 0131 | Q0037 point 5 | H3 | Within the Drone's search radius (centred on the Drone, 0123), only patches the player knows, visible or remembered, and not under a structure qualify. M3-H already filters this way, but around the patch; H3 re-centres it and tests that a never-seen patch is never chosen. |
| 0132 | Q0037 point 6 in part | H3 | With no free patch in range, a Drone waits **at the patch it tried** (within MiningReach) and mines it when it frees. It re-checks every 16 frames (placeholder) for a freed patch nearby and goes there; occupied again on arrival, it starts over from there. After each delivery it returns to the patch where it **last mined**. |
| 0133 | Q0037 point 6 | H3 | Which waiting Drone gets a freed patch is the implementation's choice, kept deterministic. **Director:** in id order, the lowest-id waiting Drone whose re-check sees the patch free claims it. |
| 0134 | Q0037 point 7; Q0044 point 5 | H3 | **Replaces the immediate redirect.** A depleted patch counts as occupied. Waiting Drones search at once. Drones walking to it go on to its spot and search on arrival, because the radius is centred on the Drone. Finding nothing ends the Gather (0123): deliver, then idle or start the next order. |
| 0135 | Q0037 point 8 | H3 | The proposal: when the return trip starts, a missing AssignedDropOff is replaced by the nearest own Egress Node. With none, the Drone idles holding its load. |
| 0136 | Q0037 points 9–10 | R, J2 | A rally always gives the new unit its own default right-click command against the target, using RightClickResolution with 0062's fallback. There are no per-building rally lists: a Drone rallied on a patch Gathers, and on an enemy unit Follows it. 0088 still applies to units. **Director:** one pure sim function, from unit and target to the default order, serves both the rally and the client's right-click. |
| 0137 | Q0044 point 2; Q0037 point 11 | H3 | As built. A mining Drone doesn't block a plate. A plated patch is gone for Drones, and a Drone mid-mining collects nothing. Units on the footprint still block. |
| 0138 | Q0037 point 13 | H3 | As built: the full 32 frames, taking the remaining crystals (fewer than 8); the patch depletes. |
| 0139 | Q0017 point 2; Q0044 points 1 and 6 | R, J2 | **Replaces "object targets only when visible".** Remembered structures and objects can be targeted by right-click, Attack, Gather, Move to object and rally, but are still not selectable.
- The order goes to the last-known position. On sight, it carries on if the object is still there, and ends as for a vanished target if not.
- There are no leaks: everything treats the remembered object as present until it is seen otherwise.
- A rally on an enemy *structure* stays on it when it leaves sight. 0088's ground rally is for units only. |
| 0140 | Q0044 point 3 | H3, J2 | There is no empty Drop Off order: an empty Drone right-clicking its own node gets a Move there. **Director:** the gate drops empty Drones from a Drop Off command. |
| 0141 | Q0044 point 4 | H3 | An unreachable patch is handled like an occupied one. The search skips it; with no reachable free patch, the Drone waits as close as it can and re-checks every 16 frames. It never gives up. |
| 0142 | Q0041 point 1 | (none) | Superseded by 0143. |
| 0143 | Q0041 points 1–2 | P, F, J2 | **Replaces 0082 and 0142, for every faction.**
- Queuing needs only the cost and a queue slot, with no capacity check.
- At production start, the unit needs free capacity ≥ its cost, and it then counts as used. HUD "used" = alive + in production.
- Without room, the producer waits at 0% with the unit at the front, starts automatically when room appears, and raises a notification (system TBD). A cancel gives a full refund.
- A unit in production always finishes.
- A destroyed producer releases only its in-production unit.
- The Barracks' Q and W no longer depend on Unit Control, and 0083 ignores it.

**Director:** a new slice, M3-P, after F. |
| 0144 | Q0041 points 3–4 | D | As built: the Bio Portal's rule on all four sides of the Robo Portal, closest by side midpoint, ties top, right, bottom, left. |
| 0145 | Q0041 points 5–6 | D2 | **Replaces 0144** with an energy doorway at each portal's centre.
1. Units materialize at the centre and move straight out through their own portal's footprint until clear.
2. They may leave in any direction the structure doesn't impede: 360° for the Robo Portal; for the Bio Portal (AABB, top and right by default), a half-circle from the top-left corner direction to the bottom-right one.
3. They head straight for the RallyPoint if that direction is open. If it is closed or blocked, they go round by the nearest open direction that gets out. With no rally, they take a random open, unblocked direction, deterministically.
4. With no way out, a unit stays half out at the footprint edge as a normal unit.
5. The next unit reaches 100% and waits fully inside until the previous units have left (Exit Doorway rule 3). It is refunded if the portal is destroyed.

The Barracks keeps its side doorway (0080). |
| 0146 | Q0042 points 1–2 | B | Shields regenerate with no ownership condition (neutral objects too), and up to the current maximum while transforming. |
| 0147 | Q0042 point 3 | B | No modifier applies to Shields; all of them apply to the overflow only. |
| 0148 | Q0042 point 4 | B | As proposed: a bar above HP, and "Shields a / b" in the InfoPanel. Styling waits for the HUD pass. |
| 0149 | Q0042 point 5 | B, F | Melee reach is 8 su edge to edge (placeholder), already built. The doc citations still name Q0042 (F fixes them). |
| 0150 | Q0042 point 6 | B, R | Convert and the Riot Shield are mode switches, so orders stay and the queue is never cleared. A shielded Peacekeeper's existing Attack runs as a Follow and resumes when the shield is lowered. AttackMove and Patrol only move while shielded. A *new* Attack still leaves shielded units out (0075). |
| 0151 | Q0042 point 7 | B, R | Mode switches happen at once: Shift has no effect, and nothing is queued. |
| 0152 | Q0042 point 8 | B | The weapon starts ready after conversion. |
| 0153 | Q0042 point 9 | R | **Replaces M3-B's leave-out.** A unit in Firing or Cooldown converts right after its Cooldown ends, instead of entering Reloading. This is a per-unit pending flag, for single units too. |
| 0154 | Q0042 point 10 | R | Rename the flag to CanHaveTurret. A unit has a turret only if TurretAttributes are set, and the attack source is the turret if present, otherwise the base. The Drone is unarmed. |
| 0155 | Q0034 | J2 | Only the plate's owner sees its patch's remaining crystals. Others get the reduced structure panel: name, owner, HP (0105). Uncovered patches still show their count to everyone. |
| 0156 | Q0033 point 1 | (none) | As built by M2-E: a destination resolved at order time is final. |

## 9. Notes from merged slices for later slices

| From | For | Note |
|---|---|---|
| M3-A | E, F, B | `change_kind` keeps orders, mover and attack target, refunds a production queue or construction, and only clamps HP. Callers decide what a transformation clears (unit to structure in F), and interpolate HP and Shields (Q0039). |
| M3-A | G | The Drone's (Ground, 1) grid doubles every grid set: nav rebuild 1.0 → 1.4–1.7 ms. G's GroundRugged hover class should remove it. Verify the grid-set count returns to M2's, and re-measure `nav_perf`. |
| M3-A | D | Portals exit through their first A side, clockwise from north, until D's exit rules. |
| Director | B, J1 | E builds the generic client response to `KindChanged` (visuals rebuilt, selection kept). B reuses it for Convert; J1 adds selection regrouping. |
| Director | all of wave 2 | E splits `sim/gate.rs` by command family. C and H port their gate edits when merging master. |
| M3-H | H2, G | A unit blocked by one "on its way" inherits that unit's progress clock (`movement::stuck`). Restarted legs can then keep the units behind from re-routing. H worked around it with `LEG_RETRY_TICKS` = 32. Legs are `MoveGoal::Point` to the nearest open face, because an object goal always planned to the north side. |
| M3-H | C, E, K1 | `gathering::step` credits banks with `credit_crystals`. Removing the last Egress Node defeats a player who owns no other structure. On the stand-in mover, one Drone at the reference gap delivers every 60 ticks (2.13/s). |
| M3-C | F, D, K1 | `army_cap` counts own Colonists structures by kind, so a Beacon under deployment counts as 5 until F excludes it (0112). `ScenarioBuilder::faction` builds Colonists tests. The M2 refusal names (`UnitControlCap`) serve every faction. |
| M3-H2 | G, every slice | Pass-through lives in `movement::space::blocks`, and I1's exceptions in `invariants::may_overlap`: any new way for boxes to overlap must extend it, with a negative test. A Drop Off order counts as a gathering job (Q0045). `stuck::leads` changed crowd paths: the battle perf scenario takes 1070 ticks instead of 880. |
| M3-H2 | G, K1 | Stand-in curve (income per second): 4 → 8.3, 8 → 15.1, 12 → 22.0, 16 → 26.8, 20 → 32.0, 24 → 32.0. That is ahead of the design below saturation (+17% at 16 Drones, +35% at 4); G re-measures with momentum. |
| M3-H2 | G | Steering: a non-yielding unit parked dead-centre on a gatherer's line, with the target close behind it, flips the rule-4 lean on every sidestep, so the Drone oscillates. A "never step back along the face" fix broke `a_line_clicked_on_itself_closes_up` and 2 goldens, so it was reverted. |
| M3-D | J2, K1, every producer | `ExitRule`, `Doorway`, `door_at` and `exit::choose` live in `production/exit.rs`. `Rng::below(n)` is drawn only when n ≥ 2 (ARCHITECTURE > Production, Determinism). `Door::room_before` was fixed: units behind a door no longer hold releases. When every free door is obstructed (not occupied), the unit is still released. The panel is built from `ProductionDef`; 0124's unavailable W/E slots are J2's. |
| M3-E | F, B, J1 | `Transform` is general: from a Drone, `from_placement` is None. `follow_max` and `keep_fraction` (1-HP floor, HP only) serve B's Shields. A deploying Beacon must stay out of `provides_capacity` (F). While transforming, production is None (`NoProducers`), and drop-off and the nearest-node search skip the object. `Sim::warp_in_check` and `transform::warp_in_footprint` are for J1's ghost. The client response to `KindChanged` is generic: `MeshKey`, then `ControlState::set_kind`. |
| M3-G | H3, K1 | With DragMovement (before the 0035 correction), income per second at 4, 8, 12, 16, 20 and 24 Drones is 6.34, 11.65, 17.54, 23.10, 25.44 and 27.10. The design's table gives 6.1 (4 Drones), 23 (16) and 32 (24). 24 Drones are 15.3% under, and one Drone went 578 ticks without delivering because the nearest-unoccupied redirect bounced it along the row. 0132's wait-at-the-patch rule (H3) should fix that; K1 judges against 0108. |
| M3-G | all | One nav grid per set again (`GroundRugged, 1`), so nav rebuilds are back to about 1.1 ms. **Accepted deviation from T28:** `Mover::momentum` is hashed and serialized only while it is non-zero, so the goldens are stable. Sidestep, doorway exit and 0126 separation stay kinematic. Braking uses omni thrust at 15/16. Neutral units never separate, because `can_yield` needs an owner. |
| M3-B | F, R, K2 | `damage::resolve` takes Shields first. `Phase::Shields` sits in the pipeline. `Ability::Convert` lives in `conversion.rs`, and the Modes step in `sim/modes.rs`. The gate's Convert rule lives in the gate's abilities family file. The SelectionPanel shows no Shields, and memory records keep none. A second Convert during a conversion is refused. |
| M3-F | J1, P, K1 | `Sim::deploy_check` serves the ghost (`Mode::Known`). `Sim::pending_deploys` gives the lingering ghosts. `TileProblem::Reserved` is the reservation problem. `SimEvent::DeployFailed` carries the feedback text. A deploying Beacon is out of `provides_capacity` until completion. A gate-refused Deploy emits no event, so the client's ghost must show the problem first. |
