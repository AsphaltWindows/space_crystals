# M4 plan: the Syndicate (planning spike M4-P)

> **Status (2026-11-24):** draft for the Director. Nothing filed or accepted yet.
> - Technical decisions are numbered T29–T44, after M3's T14–T28.
> - The questions are Q0059–Q0071. Each is its own file in `scratchpad/m4/questions/` (`Dn-slug.md`), in the Director's format, for the Director to number from Q0059 (Q0058 is the last filed).
> - Q0059 (Optics Lab stats) blocks slice K, and Q0060 (upgrade duration) blocks slice G. Q0070 blocks only the pass/fail of the oracle (L). The others are not blocking.
> - **Revised:** design changes 0210–0222 are folded in. All of them are deferred content (gated slice N3):
>   - the Laser Battery's Optics Lab gate and deploy order;
>   - the vehicle deploy order, which the GDO's Artillery Rig shares (T44);
>   - the Subterranean Transport;
>   - every DrillUnit hidden underground (0219).
>
>   They change no scheduled slice's scope. Slices A, C and E, and B's Tunnel Space, are shaped so that the Transport fits later (T30, T32, T33).
> - **Revised again** for 0223–0228:
>   - **0223** ("predictable, not optimal") made three defaults simpler:
>     - an Agent keeps the Tunnel it took for its Gather, instead of re-choosing the nearest each trip (Q0066 point 2);
>     - an Agent that can't reach Side B gives up by the Failure rule, instead of retrying (Q0066 point 3);
>     - an ejected unit appears at one fixed spot, with no search along Side A (Q0064 point 1).
>   - **0224 and 0225** split Side A into two single-file paths with fixed spots, which changes slice C: units enter one at a time, and Q0063 and Q0064 ask how long a unit holds a path. Sides B and C are hatches. Slice A draws the look by tier.
>   - **0226–0228**: the Phantom is deferred with aircraft and the Tier 3 air doors (gated slice N6).

- **Date**: 2026-11-24. Read-only spike: no repository file was changed, and cargo was not run.
- **Code read at**: `3a1b296` (master). Master has since moved to `bfc61f3`. The commits in between are:
  - design commits (0207–0222);
  - Director notes;
  - the M3-G2 merge (`d98ae13`), which touched movement, the mover and `sim/removal.rs`'s `forget`.

  This plan builds on none of them. The paths cited in T44 were checked at `bfc61f3`.
- **Design read at**: `c6f5c60` (`git log -1 --format=%h -- artifacts/designer/design`).
  - The Syndicate docs were first read at `138121a`.
  - They were read again at `36a8cd8`. In between, 0207–0209 added the Laser Battery and red Syndicate lasers.
  - The diff from `36a8cd8` to `dc6f662` (0210–0222) was then read in full: `factions/syndicate/units.md` (Laser Battery, Subterranean Transport), `structures.md` (Optics Lab, Vehicle Assembly, Forge), `factions/gdo/units.md > Artillery Rig > Modes` and `systems/units.md > DrillUnit`.
  - Then the diff from `dc6f662` to `c6f5c60` (0223–0228) was read: `systems/controls/behaviors.md > Core principle: predictable, not optimal` and `> AttackingObject > Blocked by its own side`; `factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD` (paths, hatches, look by tier); `units.md > Phantom`; `structures.md > Optics Lab` and `> Aerospace Assembly`.
- **Sources**:
  - `artifacts/director/plan.md`:
    - the M4 entry ("tunnels, Hideout, Agent, Guard economy, 0186's Optics Lab gate and Agent pistol, the Grenadier (0187) and the Assault Bike (0188), once their numbers exist. Underground expansions are exempt from the flatness rule (0005)");
    - the change rows naming M4: 0005, 0186, 0187, 0188 and 0189–0196;
    - the Built entries for M2 and M3;
    - the questions table: Q0001–Q0056 at the first read. Only Q0055 was open, and the last triaged entry was 0206. Since then the Director has filed Q0057 and Q0058 and triaged up to 0228:
      - 0207–0222 are queued for M4's N3 (the deploy order "with the first deploying vehicle");
      - 0223 went to M3-G3;
      - 0224 and 0225 are queued for M4 (A, C, D and I; "air doors deferred with air units");
      - 0226–0228 are queued for M4, deferred.
  - `artifacts/director/insights.md`, and `artifacts/director/m3-plan.md` (the template; T14–T28; §8 and §9).
  - Change entries up to 0228:
    - the Syndicate: 0186–0196, then 0207–0209 (published while this spike ran), then 0210–0228 (folded in after the first draft; the Artillery Rig's part of 0210–0214 too, and 0223's principle);
    - underground: 0005, 0043, 0044 and 0046;
    - rules that carry over: 0106 (pass-through is "Drones only"), 0108 (oracle tolerance), 0125 (lingering ghosts), 0136 (a rally gives the default right-click), 0143 (capacity checked at production start) and 0166 (the Designer supplies placeholder numbers).
  - `ARCHITECTURE.md`, above all "Adding a faction" (items 1–15) and "Working as an implementing agent".
  - The design docs below. Paths are relative to `artifacts/designer/design/`, written as `file.md > Section`:
    - all of `factions/syndicate/` (overview, tunnels, structures, units);
    - `core/entities.md` (Exit Doorway, ConstructionHP Rule, Placement Validation), `core/tech_and_research.md`, `core/game_rules.md`, `core/map.md` (Elevation, SupplyDeliveryStation), `core/vision.md`, `core/camera.md`, `core/map_editor.md > Starting Layouts`;
    - `systems/units.md` (UnitCollision, Underground Collision, the unit bases), `systems/combat.md` (ValidTarget, AttackType, AoEDamage), `systems/controls/commands.md > Enter`, `systems/controls/selection_and_hud.md`, `systems/controls/behaviors.md`;
    - `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`;
    - `factions/colonists/units.md > Drone > Gathering`, for the rules the Agent may share;
    - `factions/cults/economy.md > Cults Building Mechanics`, only to check that the M4 decisions also fit M5.
  - The code:
    - sim: `defs*`, `object.rs`, `hash.rs`, `sim.rs`, `sim/removal.rs`, `sim/deploy.rs`, `deploy.rs`, `transform.rs`, `construction.rs`, `production/` and `sim/producers*`, `economy/army.rs`, `economy/gathering.rs`, `orders/gather.rs`, `orders/resolution.rs`, `placement/check.rs`, `nav.rs`, `paths.rs`, `vision/player.rs`, `scenario/starting.rs`, `player.rs`, `maps/dev_skirmish.json`;
    - client: `command_ui/interface/` (templates, placing), `hud/resources.rs`, `fog.rs`.
- **"Silent"** means the design says nothing about the point. The plan then proposes a default and drafts a question (section 5).

---

## 0. Summary

- **Scope.** M4 builds the Syndicate as far as the design gives its numbers:
  - **the Agent**: gathering crystals and Supplies with one drop-off per Tunnel side at a time; its pistol; building Tunnels;
  - **the Tunnel**: tiers 1–3, its four sides, the Tunnel Area, Tunnel Space, cost scaling in Supplies and upgrades;
  - **the Tunnel Network**: Enter, Eject, transit between Tunnels, production into the network and the rally auto-eject;
  - **underground expansions**: hidden from enemies, walkable, placed in the Tunnel Area, one operation per Tunnel at a time;
  - **the Hideout**, and the **Optics Lab** with placeholder stats, which gives the Guard (built in M3-L2) its gate;
  - **tech and unit prerequisites**, generic for every faction;
  - **Losing Tunnels**: neutral expansions, decay, reclaiming and stranded units;
  - Starting Conditions, layouts, the HUD and panels, and red Syndicate lasers;
  - an economy oracle against the Agent's saturation table.

  **Deferred**, because they are TBD in the design:
  - the Grenadier, Assault Bike, Shadebringer, Laser Battery, Subterranean Transport, Phantom and Laser Turret;
  - aircraft, and the Tier 3 Tunnel's air doors (0224);
  - the vehicle deploy order (0211–0214), which the Laser Battery, the Transport and the GDO's Artillery Rig share (T44);
  - every expansion other than the Hideout and the Optics Lab;
  - research, detection and notifications.
- **Slices.** 14 (A–M), in 6 waves, with C and D on Fable:
  1. **A**
  2. **B** ∥ **C** (Fable) ∥ **D** (Fable)
  3. **E** ∥ **I** ∥ **J2**
  4. **F** ∥ **H** ∥ **J1**: gives the first playable
  5. **G** (needs Q0060) ∥ **K** (needs Q0059) ∥ **L**
  6. **M** (QA)

  The content gated on numbers (section 3.3, end) is not scheduled.
- **First playable** after A, B, C, D, E, H, J1 and J2: eight slices, at the end of wave 4 (M3 needed fifteen). On `dev_skirmish` with `--factions syndicate,gdo`, you:
  - start with a Tunnel, a Hideout and one Agent;
  - gather by right-click, and train Agents into the network;
  - eject them, or rally them out;
  - enter the Tunnel, and build a second Tunnel with the starting Supply;
  - fight the GDO through F2 with pistols;
  - lose your Tunnels and be defeated.

  Guards arrive with F and K, once the Optics Lab has placeholder stats (Q0059).
- **Key decisions.**
  - **T30:** units in the Tunnel Network, and an Agent inside its site, are held off the map in one hashed store outside `objects`, keeping their ids. Every system that walks the map skips them without a filter, so a forgotten case fails closed. This is the hard piece, on Fable (C).
  - **T31:** one on-site `Construction` state, with ConstructionHP and its builder, serves Agent-built Tunnels and Tunnel-built expansions. It is shaped so that the Cults' Recruit construction can use it in M5.
  - **T32:** "underground" is a structure flag. An underground structure blocks nothing, is hidden from enemies and leaves them no records, and needs no flat ground. Its parent Tunnel is derived from the geometry.
  - **T33:** Tunnel Space is a third derived army cap, beside Unit Control and Beacon Capacity. The check at production start (0143) is already shared.
  - **T34:** the Agent reuses the Drone's Gather job, with patch sharing, a drop-off side taking one Agent at a time, two resources and no pass-through (0106: "Drones only"). On Fable (D).
  - **T37:** tech and unit prerequisites are one generic check, for every faction, M5's Gunner included.
- **Readiness: mostly ready.**
  - The opening is fully statted: the Agent, the Tunnel, the Hideout, Tunnel Space, cost scaling, the Starting Conditions and the saturation table.
  - Two gaps block one slice each:
    - the Optics Lab has no stats (Q0059), which blocks K, so there are no Guards until it is answered;
    - the upgrade has no duration (Q0060), which blocks G.

    The decay rate of neutral expansions is TBD as well; H uses a named placeholder for it.
  - About 60 rule gaps can take documented defaults (Q0061–Q0070), for example:
    - when Supplies are paid;
    - how long a unit holds each Side A path (0225);
    - what an expansion's placement checks;
    - the line at Side B;
    - the panel letters that fall outside the 4x3 grid.
  - The six new units are not ready (concept only), so they are deferred: the Grenadier, Assault Bike, Shadebringer, Laser Battery, Subterranean Transport and Phantom. Nor is the Laser Turret. Q0071 asks which of them will get numbers in time for M4.

---

## 1. Scope

### 1.1 In M4

| Area | Content | Design |
|---|---|---|
| Agent | Unit, HeavyInfantry, 36x36, HP 75, armour 1/1, sight 5, TunnelSpaceCost 1, Groupable false, 6 su per frame, turn 180° per frame. Pistol: FullyConnected, Ground, 5 damage, range 2, Aim 3, Fire 1, Cooldown 2, Reload 14. | `factions/syndicate/units.md > Agent` |
| Agent gathering | Crystals: carry 50, mining 80 frames, drop-off 64 frames at Side B, reaches 8 su, any number of Agents per patch. One drop-off per Tunnel side at a time. Supplies: carry 1, pick-up 48 frames, drop-off 48 frames at Side C. | `units.md > Agent > Gathering`; `tunnels.md > Tunnel > SymmetryType - ABCD` (sides) |
| Tunnel | Structure, 4x4, ABCD (A exit, B crystals, C Supplies, D back wall), Groupable false, sight 5. Tiers: HP 600/800/1000, armour 1/16, Area radius 2/3/4. An Agent builds it in 480 frames, for as many Supplies as the player owns Tunnels. | `tunnels.md > Tunnel` |
| Tunnel Network | Enter and exit at Side A, by two paths that each take one unit at a time at a fixed spot (0224, 0225), allowed by tier and unit base. Eject, at least 8 frames apart. Transit between Tunnels. Production into the network; auto-eject for a rally on the surface. | `tunnels.md > Tunnel > SymmetryType - ABCD`, `> Tunnel Network`, `> EjectMenu`; `structures.md > Rally Point Behavior`; `systems/controls/commands.md > Enter` |
| Tunnel Area and Tunnel Space | Areas of 8x8, 10x10 and 12x12 that may not overlap at their current tier (which also blocks an upgrade). Tunnel Space = ⌊unused tunnel-region / 4⌋, at most 200; production waits for it; usage is shown over capacity. | `tunnels.md > Tunnel Area`, `> Tunnel Area Non-Overlap Rule`, `> Tunnel Space` |
| Agent-built Tunnels | Worker-Built placement and a lingering ghost. The site starts at 10% HP (ConstructionHP), with the Agent inside and untargetable. Cancel refunds 100%; a destroyed site loses the Supplies. One Agent per Tunnel. | `units.md > Agent > Building`, `> Tunnel Construction Flow`, `> Cancel Construction`; `core/entities.md > Placement Validation > Worker-Built Structures`, `> Lingering build ghosts`, `> ConstructionHP Rule` |
| Expansions | Underground, hidden from enemies, walkable. Placed within the Tunnel Area, exempt from the flatness rule. A Tier Requirement against the building Tunnel, and a TechPrerequisite anywhere in the network. One operation per Tunnel at a time. | `structures.md > Tunnel Expansions`; `tunnels.md > ExpandMenu`, `> AwaitingPlacement (Expansion)`, `> Construction and Upgrade Rules`; `core/map.md > Elevation` (0005) |
| Hideout | Underground, 2x2, Tier 1, 200 SC, 400 frames, HP 400, armour 1/4. Queue of 20. Agent 100 SC and 240 frames; Guard 125 SC and 120 frames, needing an Optics Lab. Q, W, X and C; right-click sets the rally. | `structures.md > Hideout` |
| The Guard's gate and the Optics Lab | The Guard's Unit Prerequisite (the Guard itself was built in M3-L2); the Optics Lab on placeholder stats (Q0059) | `units.md > Guard > TechPrerequisite`; `structures.md > Optics Lab`; `core/tech_and_research.md` |
| Tunnel upgrades | Tier 2 costs 2 + 2 × (T2+ owned), Tier 3 costs 3 + 3 × (T3 owned). Cancel Upgrade refunds in full. Blocked by area overlap; one operation at a time. Duration on a placeholder (Q0060). | `tunnels.md > Tiers`, `> Upgrade Costs`, `> Construction and Upgrade Rules` |
| Losing Tunnels | Expansions turn neutral, become visible and decay (rate on a placeholder, Q0069). Work in progress is lost, queued work refunded. Stranded units are destroyed. Reclaiming. Defeat by the shared rule. With no Tunnels, the next one costs 0. | `tunnels.md > Losing Tunnels`; `core/game_rules.md > Defeat Condition` |
| Start | A Tier 1 Tunnel with a Hideout, one Agent, 100 SC, 1 Supply; the camera on the Tunnel | `tunnels.md > Tunnel > Starting Conditions`; `core/camera.md > Starting Position` |
| Layouts | Syndicate Starting Layouts on `dev_skirmish` | `core/map_editor.md > Starting Layouts` |
| HUD and panels | SC, Supplies, and Tunnel Space used / available. Panels for the Agent, the Tunnel, the site and the Hideout; the ExpandMenu and EjectMenu; the Enter, GatherResources and ReturnResources pointers; the Tunnel Area overlay. | `overview.md > DisplayHud`; the ObjectInterfaceStates above; `systems/controls/selection_and_hud.md > PointerDisplayType` |
| Lasers | Syndicate lasers are red | `units.md` header (0207) |
| QA | A Syndicate oracle against the Agent table and the opening; goldens, perf and a soak | `units.md > Agent > Gathering` (table); `tunnels.md > Starting Conditions`; `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)` |

### 1.2 Deferred

| Item | Why |
|---|---|
| Grenadier (0187) | Numbers TBD ("All numbers ... TBD"). It needs the Chemical Plant (stats TBD) and combat machinery that doesn't exist: DoublyDisjointed projectiles, AoE damage and Attack Ground. |
| Assault Bike (0188) | Numbers TBD, and the Vehicle Assembly is TBD. The WheeledVehicle's FixedTurnRadiusMovement and its narrow turret arc belong with the vehicles milestone (0033, 0035, 0039). |
| Shadebringer (0189–0196) | Concept only. 0189: "don't build it until the rules are settled". Smoke that blocks line of sight is new to the vision model ("the first sight-blocking mechanic in the game"). The Vault is TBD. |
| Laser Battery (0207–0214) | Concept only: "all numbers and several rules are TBD", and "TBD: deploy/undeploy time". The beam with surges, and damage that grows with charge^(2/3), are new to combat. It needs an Optics Lab (0210), so Q0059 gates it too. Mobile and deployed are two unit types (0210). A deploy order switches between them: queueable, uncancellable, and holding the orders given meanwhile (0211, 0213, 0214; T44). The Vehicle Assembly is TBD. |
| Subterranean Transport (0215–0222) | "Concept only; times and capacity TBD" (0220). An infantry transport from the Vehicle Assembly, behind a Forge (0221). Its base is DrillUnit ("assumed"), which no kind uses yet. Burrow and Surface are T44's deploy order (0220). Underground it is hidden unless detected, yet area damage still hits it (0216, 0218). It surfaces on tiles that a tracked vehicle can cross and that have no building (0217). Its passengers die with it (0220). In the network they stay aboard, and only its own Tunnel Space counts (0221, 0222). Loading and unloading are a new mechanic, and their commands are TBD. T30, T32 and T33 leave room for it. |
| Phantom (0226–0228) | Concept only: "TBD: all numbers (HP, armor, sight, speed and handling, TunnelSpaceCost, damage, range, timings, cost, build time); hotkeys". A HoverCraft from the Aerospace Assembly, behind an Optics Lab, so Q0059 gates it too. It rises out of a Tier 3 Tunnel's air door. It has a Universal red laser on a turret with a 60–90° total cone (0228). Its role is the Guard's, "somewhat squishier for its cost, and faster". No air unit is built for any faction yet ("Later: air"). |
| Vehicle deploy order (0211, 0213, 0214) | Shared by three units: the Laser Battery's Deploy and Undeploy, the Transport's Burrow and Surface (0220), and the GDO's Artillery Rig, which isn't built. It comes with the first of them (T44), and all their times are TBD. It is not the Drone's Beacon Deployment, which "becomes a Beacon immediately" and can be cancelled (0214). |
| Laser Turret (pop-up and upright) | "(Full stats TBD — cost, build time, HP, raised/underground armor, range, damage, rise/retract timing.)" Its hotkey is TBD, and so is whether it takes the Tunnel's operation slot. The pop-up form is hidden underground "like an expansion", so it can be built before detection exists. |
| Chemical Plant, Vault, Vehicle Assembly, Forge, Null Chamber, Aerospace Assembly, Orbital Command | Each is "(Full stats TBD)". The Null Chamber's exit buff is TBD (duration, sight, reduction). The Forge now gates the Transport (0221), and the Aerospace Assembly produces the Phantom (0226). |
| Research, and Supplies spent on it | No Syndicate research is designed (the Optics Lab: "Whether it also hosts laser research is TBD") |
| Detection | "stealth/detection mechanic not yet designed" (`structures.md > Null Chamber`). Since 0219 it also decides when an enemy sees any DrillUnit underground ("hidden from other players unless revealed by a detector"). The GDO's mines and the Colonists Scout wait for it too. In M4 an owned expansion is never visible to an enemy, and no unit kind has the DrillUnit base. |
| A notification for a blocked queue | "notification system not yet designed" (`tunnels.md > Production and Tunnel Space`). M4 shows the waiting state on the producer's panel. |
| Transit at Tier 2 in play | M4 has no Syndicate vehicle. The Transport, like the other vehicles, "exits only through Tier 2+ Tunnels". The tier rule is built, and tested with test units. |
| Aircraft and the Tier 3 air doors (0224–0226) | M4 has no Syndicate aircraft: the Phantom is concept only, and the Aerospace Assembly is TBD. "Aircraft use these doors, not the Side A paths" (0224). Where the doors are, and whether they take one aircraft at a time, are TBD (Q0064 point 11). M4 builds no doors, so no air unit can enter or leave the network; in M4 Tier 3 gives only its HP and area. |
| Depends on M5 | Nothing. Two decisions (T30, units held off the map; T31, sites) are shaped so that the Cults' buildings that hold units, and their Recruit construction, can reuse them. T30 also leaves room for the Transport's passengers (N3). |
| AI opponent | Undecided (insights) |

### 1.3 What already exists (so the slices don't redo it)

**Sim**
- **Kinds and defs.**
  - `Faction::Syndicate` exists.
  - `defs/syndicate.rs` holds only the Guard: HeavyInfantry, `tunnel_space_cost: Some(1)`, an 18-damage Universal laser, Aim 4 / Fire 1 / Cooldown 3 / Reload 16 (M3-L2).
  - `UnitDef` has `{ unit_control_cost, tunnel_space_cost, beacon_capacity_cost, riot_shield, worker }`.
  - `StructureDef` has `{ symmetry, build_radius_extension, power, production, mining, constructs, warp_in, drop_off }`.
  - `ObjectKind` is appended to only, and ends at `PeacekeeperShielded`.
- **Changing kinds.** `Sim::change_kind`. `Object::transform` provides `Object::max_hp()`, the current maximum interpolated over a transformation with damage kept. `step_transforms` runs the progress.
- **Placement.**
  - `placement::check` has `Mode::{Command, Known, Arrival, Layout}` and `Method::{Direct, WarpIn, WorkerBuilt}`.
  - It applies "no buildings on resources" and the flatness rule (`needs_flat_ground`).
  - Lingering ghosts come from `Sim::pending_deploys` and give `TileProblem::Reserved`.
- **Worker orders.**
  - **Deploy:** walk, validate on arrival, and `step_deployments` right after 5a and before 5b, with `SimEvent::DeployFailed` on failure.
  - **Gather and Drop Off** (`orders/gather.rs`): phases, claims, waiting, search and legs, with `RECHECK_TICKS` 16 and `LEG_RETRY_TICKS` 32. The timed work runs in the economy phase (`economy/gathering.rs`, `ResourcesDelivered`).
  - **The Worker capability:** `WorkerDef { carry_capacity, mining_ticks, drop_off_ticks, mining_reach, drop_off_reach, search_radius, pass_through }`, plus `Sim::reassign_drop_offs`.
  - **Collision while gathering:** Drones on a Gather job pass through each other (0106), and the general overlap rule applies.
- **Production.**
  - A queue of 20, with `blocked` and `in_production`.
  - The army cap is checked when production starts, for every faction (0143). That is exactly the Syndicate's "Starting production" rule.
  - `ExitRule::{Side, Centre}` doorways and `release_unit`.
  - Rally points give the unit's default right-click (0136, `orders::resolution::resolve`).
  - Refunds on destruction.
- **Army caps.** `economy/army.rs`: `ArmyResource` (Unit Control, Beacon Capacity) and `ArmyCap::fits` (`cost == 0 || used + cost <= available`). `ArmyResource::of` returns `None` for the Syndicate.
- **Supplies.** `Player::supplies` (a counter). The Supply Delivery Station is `ResourceState::SupplyStation`; `update_supply_stations` delivers 4 every 960 frames once a station is empty, with `SuppliesDelivered`.
- **Underground hooks.** `Domain::Underground` (the DrillUnit base) and `NavClass::Underground`. `BlockerKind::UndergroundExpansion` already blocks nothing, but nothing uses it. Targeting treats Underground as submerged.
- **Vision.** `PlayerVision::object_box_state` is the one visibility query, for both the sim and the client (`Sight`). There are memory records of structures and resources. Path knowledge records structures and resources only (`paths.rs`).
- **Removal and defeat.**
  - `Sim::remove_objects` clears every reference to the objects it removes.
  - A player is defeated when their owned-structure count drops to 0. A player who never owned a structure is never defeated.
- **Construction.** The GDO's `Constructor` state machine. `construction.rs` records "Not built: tech prerequisites … ConstructionHP".
- **Start.**
  - `StartingConditions { objects, crystals, supplies, primary }`, whose `of()` returns `None` for the Syndicate.
  - `MapDef` already has a Syndicate layout list, empty in `dev_skirmish.json`.
  - Layout rules are checked for implemented factions only.
- **Tests and QA.**
  - `ScenarioBuilder::faction`, `testing::build_order` and `testing::colonists` (with `skirmish`), and the invariants.
  - Nine goldens, and the GDO and Colonists oracles (`tests/economy_oracle/`).
  - `drones_perf`, `skirmish_soak` and `gathering_soak`.
  - `Option` fields hash a tag even for `None`, so a new field on `Object` changes every golden.

**Client**
- **Templates.**
  - `InterfaceTemplate::{BasicCombatUnit, Production, DeploymentCenter, ExtractionFacility, Beacon, Drone, EgressNode}`. The Production template is built from a `ProductionDef`, so the Hideout reuses it.
  - The Deployment Center's build menus are the pattern for the ExpandMenu and the EjectMenu.
  - `PanelCommand::Planned`. No smart-casting (`is_common`, `recipients`, `instance_only`).
- **Placement.** `Builder::{Constructor, Drones { queued }, Beacon}` in `placing.rs`, with R and F turns, the knowledge-mode Deploy ghost and the lingering ghosts.
- **Right-click.** The client uses the sim's rule (`command_ui::resolution::unit_right_click`, M3-R). The GatherResources and ReturnResources pointers exist.
- **HUD and CLI.** `ResourceFigure::of(faction)` shows SC only for the Syndicate. `--factions`, `--input-script`, `--screenshot` and F2.
- **Editor.** Layouts and palettes per faction; units are drawn by their silhouette.
- **Fog.** `Sight::sees` and `Sight::object`, through `PlayerVision::object_state`, and the records.

**Files near or over 500 lines** (split before adding to them):
- sim: `sim.rs` 596, `scenario/validate.rs` 502, `orders/gather.rs` 490, `movement/collide.rs` 480, `combat/phases.rs` 478, `orders.rs` 477, `placement/check.rs` 474, `testing/invariants.rs` 465, `vision.rs` 454, `paths.rs` 452;
- client: `editor/objects.rs` 613, `fog.rs` 564, `camera.rs` 499, `editor/actions.rs` 498, `map_render.rs` 452, `object_render.rs` 440, `command_ui/indicators.rs` 440, `command_ui/interface/templates.rs` 431, `command_ui/cursor.rs` 422, `cli.rs` 419, `hud/selection_panel.rs` 419, `command_ui/resolution.rs` 413.

---

## 2. First playable

After A, B, C, D, E, H, J1 and J2 (end of wave 4):

1. Run `cargo run -- --factions syndicate,gdo` (B adds the Syndicate to `--factions`). Player 1 starts as the Syndicate in slot 1:
   - a Tier 1 Tunnel (labelled "T1"), with Side B facing the crystal field and a Hideout under it, drawn as an outline that only its owner sees;
   - one Agent; 100 SC and 1 Supply;
   - Tunnel Space 1/11: the Tier 1 tunnel-region is 48 cells, the Hideout takes 4, and 44 / 4 = 11;
   - the camera on the Tunnel.
2. Select the Hideout and press Q to train an Agent (100 SC, 15 s). With no rally, it goes into the network.
   - Select the Tunnel, open Eject (its letter is Q0068's), and click the Agent tile ("Agent 1"). The Agent comes out at Side A.
   - With the Hideout selected, right-click a patch. From then on, new Agents come out at Side A by themselves and gather (0136).
3. Select Agents and right-click a patch.
   - Each mines for 5 s, walks to Side B and drops off 50 in 4 s, one Agent at a time; the others wait at the side.
   - Three Agents close to Side B saturate the Tunnel at 12.5 SC per second, around 0:40.
4. Select an Agent and press A. Tab steps through the Agents, which are ungroupable.
   - The Tunnel ghost shows its 8x8 area, and turns red where that area would overlap your Tunnel's. R and F turn it.
   - A click sends the Agent there. The Tunnel appears at 10% HP with the Agent inside, and the Supply is paid. X on the site cancels with a full refund.
   - 30 s later the Tunnel is operational, available Tunnel Space rises from 11 to 23 ((44 + 48) / 4), and the Agent is in the network: eject it from either Tunnel.
5. Right-click a Tunnel with Agents that carry nothing: they walk to Side A and go in. Agents that carry a load drop it off instead.
6. Press F2 to play the GDO, build Peacekeepers and attack. The Agents fight back with their pistols. The GDO sees the Tunnels but never the Hideout.
7. Destroy a Tunnel.
   - Its Hideout turns neutral, now visible to everyone, and decays.
   - Units in the network survive while another Tunnel stands.
   - Destroy every Tunnel: the units in the network are destroyed, and the Syndicate is defeated, having no structure left. Its Agents on the map become neutral.

**Not needed for it:**
- F (expansions after the start), G (upgrades), K (Guards), L and M.
- I (Supplies): the starting Supply pays for the second Tunnel. I lands in wave 3 anyway.

**Earlier checkpoint (scripted, after A, B, C and D, end of wave 2).**
- There are no Syndicate panels yet, so a `--commands` QA file trains the Agents, ejects them and gives the Gather orders.
- Screenshots show Agents at the patches and in line at Side B, the bank rising, and Tunnel Space at 3/11.
- The same as a test: from a Syndicate start, 3 Agents gathering 1 GU from Side B deliver about 12.5 SC per second after 0:40. D measures and records the exact figure.

**How it is played and tested without an AI**
- **Hot-seat.** The user, or an agent, plays both sides with F2.
- **Scripted QA.** `--commands` and `--input-script` files, with screenshots at fixed ticks.
- **Goldens** through the `Script` hook: `syndicate_opening`, and `syndicate_vs_gdo` with both players scripted (L).
- **The Syndicate oracle** (L) plays a build line through the gate and checks the design's figures.

---

## 3. Slices

### 3.1 Overview

| # | Slice | Content | Design | After | Model |
|---|---|---|---|---|---|
| A | Syndicate foundation | The Agent, Tunnel and Hideout kinds with their stat blocks; the Tunnel's tier state and its max HP by tier; the underground layer (blocks nothing, hidden from enemies, no records, no flatness); the Agent's pistol; placeholder visuals, the Tunnel's look by tier (0224) and owner-only outlines for expansions; red Syndicate lasers; debug spawns; the editor palette | `units.md > Agent`; `tunnels.md > Tunnel` (Size, SymmetryType, Tiers); `structures.md > Hideout`, `> Tunnel Expansions`; `units.md` header (0207) | M3 done; user go-ahead | Opus |
| B | Syndicate start | Tunnel Space as a derived cap, shown over capacity; the Syndicate's banks and Starting Conditions; `dev_skirmish` layouts and their validation; `--factions syndicate`; the HUD | `overview.md > DisplayHud`, `> SyndicateResources`; `tunnels.md > Starting Conditions`, `> Tunnel Space` | A | Opus |
| C | Tunnel Network | The held store; the Enter order; Side A's two single-file paths (0225); the eject queue and emergence on the exit path; transit tiers; Hideout production into the network; the rally auto-eject; the Eject command | `tunnels.md > Tunnel > SymmetryType - ABCD` (the paths), `> Tunnel Network`, `> Transit Tier Requirements`, `> EjectMenu`; `structures.md > Rally Point Behavior`; `commands.md > Enter` | A | **Fable** |
| D | Agent crystal gathering | The Agent as a worker: Gather, and Drop Off at a Tunnel's Side B, one drop-off at a time; shared patches; the Tunnel kept for each Gather; the line at the side, with no crowd logic (0223). `orders/gather.rs` is split first. | `units.md > Agent > Gathering > Space Crystals`; `tunnels.md > Tunnel > SymmetryType - ABCD` (sides), `> Tunnel Area Non-Overlap Rule` ("Crystal intake and stacked Tunnels") | A | **Fable** |
| E | Agent-built Tunnels | The Build Tunnel order (the Deploy pattern); the check on arrival; area non-overlap; Supplies cost scaling; the site, with ConstructionHP and the Agent inside; cancel; destruction; completion into the network | `units.md > Agent > Building`, `> Tunnel Construction Flow`, `> Cancel Construction`; `tunnels.md > Cost Scaling`, `> Tunnel Area Non-Overlap Rule`; `core/entities.md > ConstructionHP Rule`, `> Worker-Built Structures`, `> Lingering build ghosts` | C | Opus |
| F | Expansions | The Expand command; expansion placement (`Method::Expansion`); the Tunnel's one operation; construction by the Tunnel; cancel; Tunnel Space taken at placement. The ExpandMenu, and the ghost within the area. | `structures.md > Tunnel Expansions`, `> Expansion Requirements`; `tunnels.md > ExpandMenu`, `> AwaitingPlacement (Expansion)`, `> Construction and Upgrade Rules`; `core/entities.md > Direct Placement` | B, E, J2 | Opus |
| G | Tunnel upgrades | Upgrade and Cancel Upgrade; the cost formulas; the overlap block; the change of tier (HP, area, Tunnel Space, transit); reclaiming on completion; the panel entries | `tunnels.md > Tiers`, `> Upgrade Costs`, `> Construction and Upgrade Rules`, `> Tunnel Area Non-Overlap Rule` | F, H; Q0060 | Opus |
| H | Losing Tunnels | Neutral expansions, visible and decaying; work in progress lost and queued work refunded; stranded units; reclaiming on completion; defeat; the free rebuild | `tunnels.md > Losing Tunnels`; `core/game_rules.md > Defeat Condition` | C, E | Opus |
| I | Supplies | The Agent's Supplies job: pick-up at a station, drop-off at Side C one at a time, switching between resources; the right-click on stations | `units.md > Agent > Gathering > Supplies`; `core/map.md > SupplyDeliveryStation`; `overview.md > Supplies` | D | Opus |
| J1 | Agent UI | The Agent panel; the Build Tunnel ghost (knowledge mode, area outline, lingering ghosts, Tab); the site panel; the Enter pointer; the carried load | `units.md > Agent ObjectInterfaceState`, `> Right-Click Resolution`; `selection_and_hud.md > Shift while placing`, `> PointerDisplayType` | C, D, E | Opus |
| J2 | Tunnel and Hideout UI | The Tunnel panel; the EjectMenu (counts, greyed by tier); the Tunnel Area overlay; the network's contents; the Hideout's rally (in the network or out); units leaving the selection on Enter | `tunnels.md > Tunnel ObjectInterfaceState`, `> EjectMenu`; `structures.md > ObjectInterfaceState[Hideout]`, `> Rally Point Behavior` | B, C | Opus |
| K | Prerequisites and the Guard | Generic tech and unit prerequisites; the Optics Lab on placeholders; the Guard at the Hideout behind it; greyed entries | `core/tech_and_research.md > Tech Prerequisites`; `structures.md > Optics Lab`, `> Hideout > Produces`; `units.md > Guard > TechPrerequisite` | F; Q0059 | Opus |
| L | Syndicate oracle and goldens | The `testing::syndicate` build line; the reference map; the oracle against the Agent table and the opening; two goldens | `units.md > Agent > Gathering` (table); `tunnels.md > Starting Conditions`, `> Tunnel Area Non-Overlap Rule` ("Crystal intake and stacked Tunnels"); `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)` | B, C, D, E, H, I | Opus |
| M | M4 QA | `agents_perf`; an ignored soak; a scripted walkthrough with screenshots; `docs/playtest/m4.md`; the ARCHITECTURE review; the ignored tests | `ARCHITECTURE.md > Adding a faction` item 15 | everything | Opus |

**Gated, not scheduled.** Each waits for numbers. They are listed at the end of 3.3, and Q0071 asks which will come in time:
- N1 Grenadier;
- N2 Laser Turret;
- N3 the Vehicle Assembly, with the Assault Bike, the Laser Battery, the Subterranean Transport and the vehicle deploy order (T44);
- N4 Shadebringer;
- N5 the remaining expansions (Null Chamber, Vault, Forge and the others);
- N6 the Phantom, with aircraft and the Tier 3 air doors.

### 3.2 Waves and parallel tracks

```
W0  M3 done (M3-T, L3, G2, K3, J3); user go-ahead for M4
W1  A
W2  B (after A) | C (Fable, after A) | D (Fable, after A)
W3  E (after C) | J2 (after B, C) | I (after D)
W4  J1 (after C, D, E) | H (after C, E) | F (after B, E, J2)      -> first playable (after J1 and H)
W5  K (after F; Q0059 answered) | G (after F, H; Q0060 answered) | L (after B, C, D, E, H, I)
W6  M (after everything)                                           -> agent QA; the user playtests after M5
```

- **Critical path**: A → C → E → J1 and H, which gives the first playable; then F → K and G → M.
- **Machine load.** At most three builds at once, in W2 to W5. W2 runs both Fable slices. If one of them stalls, only its own followers wait: E needs only C, J2 needs B and C, and I needs only D.
- **Shared files.** These slices touch shared files, so expect small merge conflicts, as in M3's waves 2 and 3:
  - in W2:
    - `sim.rs` (C's steps);
    - `sim/gate.rs` and `command.rs` (C's Enter and Eject, D's Drop Off on Tunnels);
    - `orders/resolution.rs` (C's Enter rule, D's Tunnel drop-off);
    - `hash.rs` and `events.rs`;
  - in W3–W4:
    - `placement/` (E, then F);
    - `construction/` (E adds `Construction`, F extends it);
    - client `templates/syndicate.rs` (J2 creates the Tunnel template; F adds Expand to it, which is why F waits for J2).

  If the Director wants no conflicts in W2, run D after C, at the cost of one wave.
- **Splits first.** These files are near or over 500 lines, so they are split before they grow:
  - `sim.rs` (596): the new steps go to `sim/network.rs` and `sim/sites.rs` (C, E);
  - `orders/gather.rs` (490), before D adds to it;
  - `placement/check.rs` (474), before E;
  - client `command_ui/interface/templates.rs` (431): the Syndicate templates go to `templates/syndicate.rs` (J2, J1).

**What each slice owns** (for the "don't touch" lists):

| Slice | Owns | Read-only |
|---|---|---|
| A | `defs/syndicate.rs`; the Syndicate entries in `defs.rs`, `defs/structures.rs` (`underground`, `TunnelDef`) and `defs/units.rs`; `Object::tunnel` (the tier only); the hidden rule in `vision/player.rs` and the records; the underground blocker in `nav.rs` and `paths.rs`; placeholder visuals; red lasers; editor palette arms | — |
| B | `economy/army.rs` (Tunnel Space), `player.rs`, `scenario/starting.rs`, the layout rules in `scenario/` (validate, problems), `scenarios.rs`, `maps/`; client `cli.rs`, `hud/resources.rs`, `editor/layouts.rs` | `defs/` |
| C | `network.rs` (new: the held store), `sim/network.rs` (new: the entry and ejection steps), `orders/enter.rs` (new), the Enter rule in `orders/resolution.rs`, `ExitRule::Network` and release in `production/` and `sim/producers*`, the reference cleanup split out of `sim/removal.rs`, `Command::Tunnel` (Eject), the eject queue in `TunnelState`, the held invariants in `testing/invariants.rs` | `economy/army.rs` (B's) |
| D | `orders/gather*` (split first), `economy/gathering.rs`, `sim/workers.rs`, the `WorkerDef` generalisation in `defs/units.rs`, side drop-offs | `movement/collide*` (raise a question rather than add an exemption) |
| E | `orders/build.rs` (new: Build Tunnel), `construction/site.rs` (new: `Construction`), `sim/sites.rs` (new), the generalised ghosts from `sim/deploy.rs`, the Tunnel's constraint in `placement/`, `TunnelCommand::CancelConstruction`, Supplies prices | `network.rs` (C's) |
| F | `Method::Expansion` in `placement/`, `TunnelCommand::Expand`, the operation slot, the Tunnel as a builder in `construction/site.rs`; client ExpandMenu, `Builder::Tunnel`, the clamped ghost | `templates/syndicate.rs` outside the Expand entries |
| G | `TunnelCommand::{Upgrade, CancelUpgrade}`, the upgrade operation, the tier change; the client's Upgrade entries | `sim/tunnels.rs` (H's; G calls its `reclaim`) |
| H | `sim/tunnels.rs` (new: neutralise, decay, strand, reclaim), its hook in `sim/removal.rs`, the defeat tests; the client's look for neutral expansions | `network.rs` |
| I | the station leg and pick-up in `orders/gather*` and `economy/gathering.rs`, the station decrement, Side C | D's side rule |
| J1 | client `templates/syndicate.rs` (Agent, site), `Builder::Agent` in `placing.rs`, the Enter pointer in `command_ui/cursor.rs`, the lingering Tunnel ghosts in `command_ui/indicators.rs`, the carried load | the sim |
| J2 | client `templates/syndicate.rs` (Tunnel, EjectMenu), the Tunnel Area overlay (new `tunnel_areas.rs`), the network's contents in `hud/selection_panel.rs`, selection and control groups for held units | the sim |
| K | `tech.rs` (new), `requires` in `defs/structures.rs`, `OPTICS_LAB` and the Guard recipe in `defs/syndicate.rs`, the gate checks, the client's `conditions.rs` | — |
| L | `testing/syndicate.rs` (new), `tests/economy_oracle/syndicate.rs`, `tests/golden/syndicate.rs`, `docs/economy_oracle.md` | everything else |
| M | perf and soak tests, the QA scripts, `ARCHITECTURE.md`, `docs/playtest/m4.md` | — |

**Every slice follows the Definition of done** (`ARCHITECTURE.md > Definition of done`):
- clippy, fmt and tests are clean;
- doc comments cite the design, and placeholders are named constants that cite their question (D-number until the Director files it);
- `ARCHITECTURE.md` is updated for new conventions:
  - Object model: held units, `Construction`, `TunnelState`;
  - Tick pipeline: entries after 5a, ejections in the production step, sites, decay;
  - Economy: Tunnel Space, two-resource workers, side drop-offs;
  - Placement: `Method::Expansion`, area non-overlap, the two layers;
  - Vision: hidden underground structures;
  - "Adding a faction": each item checked against what M4 needed (M reviews);
- goldens are re-blessed once after merging master, with each change explained. A new field on `Object` or `Sim` changes every hash (a `None` still hashes a tag), so A, C and E each re-bless every golden once, for that reason alone.

### 3.3 Slice details

The quotes below are the design text the briefs should carry word for word (insights: "Quote design text in briefs; don't paraphrase it").

#### A: Syndicate foundation (Opus)

**Goal.** The Agent, the Tunnel and the Hideout exist with their stat blocks. They can be created by debug spawn, in scenarios and in layouts.
- The Tunnel blocks ground units, gives sight 5, is remembered and can be destroyed. It carries its tier (Tier 1), and its HP follows the tier table.
- The Hideout is underground: ground units walk over it, and enemies never see, remember or target it. It needs no flat ground.
- The Agent fights with its pistol.

No network, production, gathering or construction yet.

**Design**
- `factions/syndicate/units.md > Agent`:
  - "Silhouette - 36x36", "MaxHP - 75", "PointArmor - 1", "FullArmor - 1", "SightRange - 5", "TunnelSpaceCost - 1", "Groupable - false";
  - "MaxSpeed: 6 space units/frame", "TurnRate: 180 degrees/frame";
  - the pistol: "AttackType: FullyConnected (Ranged; instant hit)", "TargetDomain: Ground", "Damage: 5", "Range: 2 grid units", Aim 3, Fire 1, Cooldown 2, Reload 14: "One shot every 20 frames (1.25 s) = 4 damage per second".
- `tunnels.md > Tunnel`: "Size - 4x4", "SymmetryType - ABCD", "Destructible - true", "Groupable - false", "SightRange - 5". `> Tiers`: HP 600 / 800 / 1000, PointArmor 1, FullArmor 16, Tunnel Area radius "2 (8x8)", "3 (10x10)", "4 (12x12)".
- `structures.md > Hideout`: "Structure Type (Underground)", "Size - 2x2", "Tier Requirement - 1", "HP - 400", "PointArmor - 1", "FullArmor - 4". Its SymmetryType, SightRange and Groupable are silent (T29, Q0065).
- `structures.md > Tunnel Expansions`: "Expansions are invisible to enemies without detection and can be walked over by surface units."
- `tunnels.md > Tunnel > SymmetryType - ABCD` (0224): Side A "has **two paths: one for entering, one for exiting**"; Sides B and C are each "a **hatch**"; "Look by tier":
  - "**Tier 1**: the Side A paths are **narrow and shaggy-looking**, supporting only infantry";
  - "**Tier 2**: the paths get **wider and well paved**, supporting vehicles";
  - "**Tier 3**: adds **two horizontal circular sliding doors**, **one for flying units entering and one for flying units exiting**".
- `core/map.md > Elevation` (0005): "Syndicate underground expansions (inside a Tunnel Area) may be placed on sloped tiles; the flatness rule doesn't apply to them".
- `units.md` header (0207): "every Syndicate laser-shooting unit and structure (Guard, Laser Battery, Laser Turret) fires a **red** laser."

**Code**
- **Kinds.** `defs/syndicate.rs` gets AGENT, TUNNEL and HIDEOUT beside GUARD, each value with its design line. Append `Agent`, `Tunnel` and `Hideout` to `ObjectKind` and `ALL`.
- **New def fields** (T29, T32, T39):
  - `StructureDef::underground: bool`;
  - `StructureDef::tunnel: Option<&'static TunnelDef>`, the tier table: `TierDef { max_hp, area_radius, transit }`;
  - the Hideout's sides are AAAA, since none has a function.
- **Tunnel state** (T39). `Object::tunnel: Option<TunnelState { tier }>`, hashed, with `serde(default)`, created for every kind with a `TunnelDef`. `Object::max_hp()` reads the tier; it already interpolates transformations.
- **The underground layer** (T32):
  - an underground footprint uses `BlockerKind::UndergroundExpansion`, which blocks nothing, and gets no path-knowledge record; movement ignores it;
  - `PlayerVision::object_box_state` never reports an owned underground structure as Visible to another player; a neutral one shows as usual; no record is taken of a hidden one;
  - that rule is one predicate over objects, `Sim::hidden_from(object, player)` (T32), not a structure test inside `object_box_state`:
    - 0219 hides every DrillUnit underground the same way ("hidden from other players unless revealed by a detector"). The Transport (N3) adds a unit arm to the predicate, and detection later adds a "revealed" term;
    - A adds no unit arm, since no kind has the DrillUnit base;
    - targeting uses the predicate, but damage resolution must not: area damage still hits a hidden DrillUnit (0218);
  - its elevation, for the ElevationModifier, is the terrain under its centre;
  - `needs_flat_ground` is false for the Hideout. Until F, underground kinds are placed only by debug spawn, scenarios and layouts (`Mode::Layout`).
- **Client.**
  - Placeholder meshes with two-letter labels: AG for the Agent, T1, T2 or T3 for the Tunnel by tier, HI for the Hideout.
  - The Tunnel's sides follow 0224's look:
    - two path strips on Side A, narrow at Tier 1 and wider at Tiers 2 and 3;
    - a hatch mark on Sides B and C;
    - at Tier 3, two circles on the roof for the air doors. They are only drawn: they have no function in M4.
  - Own underground structures are drawn as translucent outlines below the units.
  - Selection panel labels, the editor palette, and every exhaustive match over kinds.
  - Every Syndicate laser is drawn red, the Guard's included.
  - `InterfaceTemplate::of` gives the new kinds none until J1 and J2.

**Acceptance**
- **Defs.** One test per kind checks the values against the design. The consistency test is extended:
  - underground implies a structure, with `needs_flat_ground` false;
  - only the Tunnel has a `TunnelDef`, with the tier table's values.
- **Spawns.** Each kind spawns for two owners.
  - The Tunnel blocks the ground grids and gives sight 5.
  - Units walk across a Hideout, which adds no blocker to the nav grids.
  - Seen by an enemy, the Hideout is never Visible, even with the enemy's units standing on it, and leaves no record. A neutral Hideout is visible.
- **Combat.** An Agent shoots an enemy Peacekeeper every 20 ticks, at up to 2 GU. It can't target air.
- **Tiers.** A Tier 1 Tunnel has max HP 600. Setting tier 2 in a test gives 800 (state only; G builds upgrades).
- **QA screenshots.** From a `--commands` file: one of each kind for two owners, then the same scene seen by the enemy, without the Hideout.
- **Goldens.** `Object::tunnel` hashes `None` for every object, so every golden changes once. Re-bless, explaining that this is the only cause.

**Owns**: `defs/syndicate.rs`; the Syndicate entries in `defs.rs`, `defs/structures.rs` and `defs/units.rs`; `Object::tunnel` (the tier); the hidden rule in `vision/player.rs` and the records; the underground blocker in `nav.rs` and `paths.rs`; placeholder visuals, red lasers and the editor palette. **Read-only**: none.

**After**: M3 done and the user's go-ahead. **Model**: Opus. The work is wide but shallow, mostly data.

#### B: Syndicate start (Opus)

**Goal.** A Syndicate player starts by the Starting Conditions on `dev_skirmish`, with Tunnel Space derived and shown in the HUD.

**Design**
- `tunnels.md > Tunnel > Starting Conditions`:
  - "One pre-built **Tier 1 Tunnel** containing one pre-built **Hideout**";
  - "One **Agent**";
  - "**1 Supply** (enough for the 2nd Tunnel, which costs 1 Supply)";
  - "Starting Space Crystals: **100** (same as GDO and the Colonists)".
- `core/camera.md > Starting Position`: "Syndicate: starting Tunnel".
- `tunnels.md > Tunnel Space`:
  - "Tunnel-region: a Tunnel's Tunnel Area, excluding the 4x4 grid units occupied by the Tunnel's own footprint";
  - "Tunnel Space = floor((sum of unused tunnel-region grid units across all owned Tunnels) / 4) — rounded down to a whole number — capped at a hard maximum of 200";
  - "usage is simply displayed over capacity (e.g. 12/10) and no further units can be produced until usage drops back within capacity".
- `> Production and Tunnel Space`: "the unit's Tunnel Space counts as used from that moment — through production and for as long as the unit lives"; "a unit with TunnelSpaceCost 0 always passes".
- `overview.md > DisplayHud`: "Space Crystals: current amount", "Supplies: current amount", "Tunnel Space: current used / current available (max 200)".
- `core/map_editor.md > Starting Layouts`: the map places each faction's starting objects.

**Code**
- **Tunnel Space** (T33). `ArmyResource::TunnelSpace` in `economy/army.rs`, with `of(Syndicate)`, `name`, `cost_of` (the unit's `tunnel_space_cost`) and `cap`:
  - available comes from the owner's operational Tunnels: the tier's area, less the Tunnel's footprint and the owner's underground structures inside the area, found as in T32;
  - used counts the owner's units on the map and in production. C adds the held units through a hook B leaves for them.
- **Starting Conditions.** `StartingConditions::of(Syndicate)`: Tunnel 1, Hideout 1, Agent 1; 100 crystals; 1 Supply; primary the Tunnel.
- **Layouts** (T40):
  - both slots of `maps/dev_skirmish.json` and `scenarios::dev_skirmish_map`:
    - the Tunnel with Side B facing the patches about 1 GU away, and Side A toward the open map;
    - the Hideout inside the Tunnel Area, off the footprint and off resources;
    - the Agent near Side A;
  - a new validation rule: a Syndicate layout's expansions lie inside a layout Tunnel's Tier 1 area, by `Method::Expansion` in `Mode::Layout`. Until F adds that method, B checks the area's bounds directly, and F switches the rule over;
  - check with `MapDef::validate` and the editor tests.
- **Client.** `--factions syndicate` and the editor's faction lists. `ResourceFigure::TunnelSpace`, and the Syndicate's DisplayHud: SC, Supplies, Tunnel Space, shown over capacity as "12/10".

**Acceptance**
- **Start.** A `syndicate_start` test, for both slots: the objects, 100 SC, 1 Supply, Tunnel Space 1/11, the camera on the Tunnel.
- **Tunnel Space.** Unit tests:
  - the tunnel-region of each tier: 48, 84 and 128 cells (8×8, 10×10 and 12×12, less 16);
  - with expansions, the rounding down, and the cap of 200;
  - over capacity: a Hideout spawned into a full area;
  - two Tunnels.
- **Layouts.** `dev_skirmish` validates with the Syndicate layouts, and the editor tests pass. A layout with the Hideout outside the area is a validation problem.
- **HUD screenshot**: SC 100, Supplies 1, Tunnel Space 1/11.
- **Goldens.** No change is expected: only a slot's chosen faction's layout spawns.

**Owns**: `economy/army.rs` (Tunnel Space), `player.rs`, `scenario/starting.rs`, the layout rules in `scenario/`, `scenarios.rs`, `maps/`; client `cli.rs`, `hud/resources.rs`, `editor/layouts.rs`. **Read-only**: `defs/`.

**After**: A. **Model**: Opus.

#### C: Tunnel Network (Fable)

**Goal.**
- Syndicate units enter a Tunnel at Side A and leave the map into their owner's network.
- The player ejects them, by type, from any Tunnel whose tier allows them.
- The Hideout produces into the network, or out through its Tunnel's Side A towards a rally on the surface.

**Design**
- `tunnels.md > Tunnel Network`: "Units inside the Tunnel Network can travel between Tunnels freely, but can only enter or exit the network through a Tunnel whose tier is sufficient for their unit base category."
- `> Transit Tier Requirements`:
  - "Tier 1+: Infantry (Light Infantry, Heavy Infantry)";
  - "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)";
  - "Tier 3+: Air units (Hover Craft, Glider)".
- `tunnels.md > Tunnel > SymmetryType - ABCD` (0224, 0225): Side A "has **two paths: one for entering, one for exiting** ... **Each path takes one unit at a time** and fixes where units go in and come out: entering and exiting units never block each other, but an army enters or leaves single file." At Tier 3: "Aircraft use these doors, not the Side A paths."
- `systems/controls/behaviors.md > Core principle: predictable, not optimal` (0223): "Avoiding bottlenecks and getting the best out of units is **the player's job**."
- `systems/controls/commands.md > Enter`: "**Syndicate — Tunnel**: any Syndicate unit, when the target Tunnel's tier is sufficient for the unit's base category. The unit walks to Side A and enters the Tunnel Network."
- `tunnels.md > EjectMenu`: "ejects one unit of that type from this Tunnel's Side A (CommandIssuingTransition). Ejected units are queued — a new unit begins ejecting every 8 frames minimum, but actual throughput is limited by unit speed and collision at Side A. Standard movement and collision mechanics apply as units emerge."
- `structures.md > Rally Point Behavior`:
  - "**Rally point set on the surface**: Unit auto-ejects from the parent Tunnel (Side A) and moves to the rally point."
  - "**No rally point, or rally point set on the parent Tunnel**: Unit stays in the Tunnel Network, available for ejection from any sufficiently-tiered Tunnel."
- `selection_and_hud.md > What a rally point makes a new unit do` (0136): "A unit that spawns with a rally point set **always** gets its **own default right-click command** resolved against the rally target".
- `tunnels.md > Losing Tunnels`: "A Tunnel that is still under construction does not count as an exit, and a Tunnel that is mid-upgrade counts at its current (pre-upgrade) tier."
- `core/entities.md > Exit Doorway`: "Aircraft and units released underground (e.g. Syndicate Tunnels) are not covered by this rule."
- `systems/units.md > Underground Collision > Surfacing`: "not allowed to surface where its Silhouette would overlap an above-ground unit ... nor a structure or terrain impassable for its surface form". This is the closest rule for the exit spot's test (Q0064).
- `structures.md > Hideout > Produces`: "Agent: 100 Space Crystals, 240 frames (15 seconds)". `> HideoutInstanceState`: "BuildQueue: array of ObjectEnum (max 20, including the unit currently in production ...)".
- Q0063 and Q0064 for the gaps.

**Code**
- **The held store** (T30). `Sim::held`, hashed and serialized, with `Place::Network`; E adds `Place::Site(id)`.
  - `Place` is matched exhaustively, and nothing assumes that a held unit's place is on the map. The case of a unit held inside a held unit is not built: the Transport's passengers in the network (0221, 0222). It must stay possible, though (T30).
  - `Sim::take_off_map(id, place)` clears every reference to the unit as a removal does, but without the effects of a death: orders, attacks and follows on it end, and rallies on it reset. It also clears the unit's own orders, mover and attack state.
  - `Sim::put_on_map(id, spot)` puts it back, with its id.
- **Enter** (T35).
  - `OrderKind::Enter(tunnel)`: a leg to Side A.
  - `step_entries` runs right after 5a, beside `step_deployments`. Each Tunnel's entry path takes one unit at a time (0225). A unit goes into the network when three things hold:
    - it is within reach of the entry spot (Q0063 point 2);
    - the Tunnel is its owner's, operational, and of a tier that allows its base;
    - the path is free.

    If several units are within reach, the first to arrive goes first. The path is then busy for 8 ticks (Q0063 point 7).
  - A unit waiting its turn stands where it stopped, with no formation and no other path or Tunnel. One that can't get within reach follows the normal Failure rule (0223).
  - Air units are refused Enter, since M4 builds no air doors (0224).
  - The gate checks the same conditions. `resolve` gives Enter on an own Tunnel to a Syndicate unit when the tier is sufficient; D adds Drop Off for an Agent that carries a load.
- **Eject** (T35, T36).
  - `Command::Tunnel { tunnel, command: TunnelCommand::Eject { kind } }`.
  - The eject queue lives in `TunnelState` and is hashed. The command reserves a specific unit (Q0064 point 3).
  - Units emerge one at a time on the exit path's fixed spot (Q0064 point 1). A unit appears only when its silhouette there overlaps no unit and no structure, and at least 8 ticks after the last (Q0064 point 10); otherwise the queue waits. Nothing searches for another spot (0223). A unit with no order then walks 1 GU straight out.
  - Emergence happens in the production step, through the release point that produced units use (T35).
- **Production** (T35).
  - The Hideout's `ProductionDef`, with `ExitRule::Network` and the Agent's recipe; K adds the Guard.
  - Release goes into the network, or, for a rally on the surface, onto the parent Tunnel's eject queue with the rally's default order (0136). The parent is derived (T32).
- **Tunnel Space.** Held units count as used, through B's hook.
- **Client.** The minimum to make the network playable from QA scripts: a unit taken into the network leaves the selection ("leaves the selection by entering a building, transport or tunnel") and stays in its control groups (Q0063 point 4). J2 builds the EjectMenu; until then QA uses `--commands`.
- **Invariants.**
  - A held unit is never in `objects`, and nothing on the map names it.
  - Every reserved unit in an eject queue is held in its Tunnel owner's network.

**Acceptance**
- **Enter.** A Guard ordered to Enter walks to the entry spot and leaves the map into the network.
  - Tunnel Space is unchanged.
  - It leaves the selection.
  - An enemy unit that was attacking it stops, as for a death.
- **Tiers.** A test unit with a vehicle base (a Drone owned by the Syndicate test player) is refused Enter and Eject at a Tier 1 Tunnel, and allowed at Tier 2 (the tier is set in the test). An air test unit is refused at every tier, since M4 has no air doors.
- **Eject.**
  - Five ejects at one Tunnel come out at least 8 ticks apart.
  - With a unit parked on the exit spot, the queue waits until it moves (Q0064 point 1). A unit parked elsewhere along Side A doesn't stop it.
- **Single file** (0225's verification). 10 Guards ordered to Enter go in one after another through the entry path, at least 8 ticks apart. Units ejected meanwhile come out of the exit path, unhindered by those entering.
  - The same units can come out of a second Tunnel in the same tick (instant transit, Q0063).
- **Production.**
  - With no rally, a trained Agent appears in the network.
  - With a rally on the ground, it emerges at Side A and moves there.
  - With a rally on the parent Tunnel, it stays in the network.
  - A rally on a patch gives Gather once D is merged; whichever slice merges second adds that test.
- **Tunnel Space.** A unit in production and a unit in the network both count. Production waits when Tunnel Space is full, and starts when a unit dies.
- **Crowds.** 40 Guards enter one Tunnel and are then ejected.
  - Each Guard enters, or gives up by the Failure rule and idles (0223), within a bounded number of ticks. None waits indefinitely.
  - The test records how many give up. If many do while the line still moves, that goes to the Designer as a question, not into crowd logic.
  - `movement_perf` A/B against master is unchanged.
- **QA screenshots**: Guards entering single file by the entry path; units leaving by the exit path.
- **Goldens.** `Sim::held` (empty) changes every hash once. Re-bless, explaining that.

**Owns**: `network.rs`, `sim/network.rs` and `orders/enter.rs` (new); the Enter rule in `orders/resolution.rs`; `ExitRule::Network` and the release in `production/` and `sim/producers*`; the reference cleanup split out of `sim/removal.rs`; `Command::Tunnel` (Eject); the eject queue; the held invariants. **Read-only**: `economy/army.rs`.

**After**: A. **Model**: Fable. Units leave and re-enter the map, emerge into collision, and crowd at Side A. M2-Q0060's doorway, the nearest precedent, was on Fable.

#### D: Agent crystal gathering (Fable)

**Goal.** Agents gather Space Crystals.
- They mine any patch, shared with any number of Agents.
- They carry each load to Side B, the crystal hatch, of the Tunnel they took when the Gather began (Q0066 point 2). There one Agent drops off at a time, and they repeat.
- Drop Off on a Tunnel delivers a load.

**Design**
- `units.md > Agent > Gathering > Space Crystals`:
  - "CarryCapacity: 50 Space Crystals per load";
  - "MiningDuration: 80 frames (5 seconds)";
  - "DropOffDuration: 64 frames (4 seconds)";
  - "Drop-off side: Tunnel Side B";
  - "MiningReach: 8 space units — an Agent can mine once its Silhouette is within 8 su of the patch's edge (placeholder)";
  - "DropOffReach: 8 space units — within 8 su of Side B (placeholder)";
  - "Patch sharing: any number of Agents may mine the same Space Crystals patch at the same time (unlike Drones, see Colonists > Drone > Patch Occupancy). The only queue in an Agent's trip is at the Tunnel's Side B."
- `tunnels.md > Tunnel > SymmetryType - ABCD`: "Only one Agent may drop off resources at a side at a time. Because crystal and supply drop-offs are on separate sides, one crystal delivery and one supply delivery can occur simultaneously." 0224 adds: "Each resource must be delivered to **its own hatch**: crystals only at Side B, Supplies only at Side C".
- `systems/controls/behaviors.md > Core principle: predictable, not optimal` (0223): "prefer the **simple, explainable rule** over a cleverer one that handles more cases, as long as the simple rule fails in ways the player can see and fix".
- `tunnels.md > Tunnel Area Non-Overlap Rule` ("Crystal intake and stacked Tunnels"): "one Tunnel takes in at most 12.5 Space Crystals per second however many patches surround it"; "Two Tier 1 Tunnels may share a rich crystal field (doubling its intake)".
- `units.md > Agent > Right-Click Resolution`: "Space Crystals patch → Gather crystals"; "Own Tunnel (carrying resources) → Drop off resources (auto-routes to correct side)"; "Own Tunnel (not carrying resources) → Enter".
- `factions/colonists/units.md > Gathering Drones pass through each other`: "**Drones only.** Other factions' workers (e.g. Cults Recruits) keep normal collision while mining." So Agents collide as usual.
- The Drone's job rules, reused by Q0066's proposal (0123, 0127–0141, 0170).

**Code**
- **Split first.** `orders/gather.rs` (490 lines): phases, occupancy and legs into their own files.
- **`WorkerDef`, generalised** (T34):
  - timings and reaches per resource;
  - `shares_patches`;
  - a drop-off rule from the drop-off's def: anywhere, for the Egress Node, or a side taking one Agent at a time, for the Tunnel (Side B for crystals; I adds Side C for Supplies);
  - `pass_through` stays the Drone's alone.
- **Side occupancy** is derived from the workers' states as 5a begins, the way `World::patch_mined` derives a patch's. A side is busy while an Agent drops off there. Agents waiting within DropOffReach of the side are served in the order they arrived, ties by id (T34). An Agent that can't get within reach follows the normal Failure rule, and idles holding its load (Q0066 point 3, 0223). The Drone's `LEG_RETRY_TICKS` retry isn't used for Agents.
- **Where to deliver.**
  - The Tunnel taken as a Gather's first return starts: the nearest own operational one, by straight line to the middle of Side B. It is kept for the rest of that Gather (Q0066 point 2), through the Drone's AssignedDropOff with no Set Drop-off command. A new Gather order takes the nearest again.
  - Drop Off uses the Tunnel clicked.
  - The removal path chooses again when a Tunnel goes (0170).
- **`resolve`.** For Agents: Gather on patches, Drop Off on own Tunnels while carrying, and Enter when empty (C's rule).
- **Timed work** in the economy step, mining and drop-off, with `ResourcesDelivered`.

**Acceptance**
- **One Agent** at 0–6 GU from Side B: trip times measured and recorded against the table. L's oracle asserts them.
- **Two, three and four Agents** at 1 GU: income measured.
  - It never passes 12.5 SC per second per Tunnel.
  - Never two Agents drop off at one side at once.
  - No Agent waits for ever: one that can't reach Side B gives up after about 7 s and idles with its load (Q0066 point 3). An ignored 10-minute soak runs six Agents on two stacked Tier 1 Tunnels sharing one field, and records how many give up.
  - Each Agent keeps its Tunnel for the whole Gather, even when it ends up nearer the other one (Q0066 point 2).
- **Patch sharing.** Four Agents mine one patch together.
- **The Drone's rules** (Q0066), with tests on the Agent: the patch runs out and the Agent looks round; a load kept through another order; Gather while carrying.
- **Drop Off.** A right-click on an own Tunnel while carrying delivers at Side B, then the order completes (0127).
- **No change for the Colonists.** The Colonists goldens and the Colonists oracle stay as they are. This guards the shared job. `drones_perf` A/B is unchanged.
- **QA screenshot**: three Agents, one dropping off at Side B, one waiting, one at the patch.

**Owns**: `orders/gather*`, `economy/gathering.rs`, `sim/workers.rs`, `WorkerDef` in `defs/units.rs`, side drop-offs. **Read-only**: `movement/collide*`.

**After**: A. **Model**: Fable: a crowd at the side under hard collision, and a job shared with the Drone. M3-H, the nearest precedent, was on Fable.

#### E: Agent-built Tunnels (Opus)

**Goal.** An Agent builds a Tunnel.
- The order walks it to the site.
- On arrival the site appears with the Agent inside, at 10% HP, and grows to full over 480 ticks. Then the Agent is in the network.
- Cancelling refunds the Supplies; destruction loses them.
- The cost grows with the Tunnels owned, and Tunnel Areas never overlap.

**Design**
- `units.md > Agent > Building > Tunnel Construction Flow`:
  1. "Agent receives build command and walks to the target location"
  2. "Construction begins — the partially-built Tunnel appears at the location with the Agent inside it. The Tunnel starts at 10% HP (ConstructionHP Rule). The Agent is untargetable for the duration of construction."
  3. "**If construction completes**: The Tunnel becomes operational and the Agent is inside the Tunnel Network, available for redeployment from any Tunnel."
  4. "**If the partially-built Tunnel is destroyed**: The Agent survives and emerges at the location. The Tunnel is lost and any Supplies spent are lost."
  5. "**If construction is cancelled by the player**: The Agent survives and emerges at the location, same as destruction. A rubble animation plays (cosmetic only, no gameplay effect). 100% of the Supplies cost is refunded."
- `> Building`: "Only one Agent may construct a given Tunnel — multiple Agents cannot speed up construction."
- `> Cancel Construction`: "**X: Cancel Construction**: cancels the in-progress initial build (CommandIssuingTransition). The Agent survives and emerges at the location. Full refund of Supplies cost. Only available while the Tunnel is under initial construction".
- `tunnels.md > Cost Scaling > Construction Cost`: "Cost = current Tunnels owned (all tiers count), in Supplies." `> Construction Time`: "480 frames (30 seconds). Agent must be present for the duration."
- `tunnels.md > Tunnel Area Non-Overlap Rule`: "Tunnel Areas cannot overlap with each other. The non-overlap check uses the **current tier's area**, not the maximum potential area."
- `core/entities.md > ConstructionHP Rule`: "HP during construction = MaxHP x (10% + 90% x construction_progress). A partially-built structure can be attacked and destroyed before completion."
- `core/entities.md > Worker-Built Structures`:
  - "The command is accepted regardless of current visibility";
  - "**The ghost judges by what the player knows**";
  - "validates on arrival: tiles must be Buildable, flat (core/map.md > Elevation), unoccupied, and meet faction-specific constraints";
  - "If validation fails on arrival, the command is cancelled and the worker stops and idles".
- `> Lingering build ghosts`: "Every **ordered worker build** (Drone Deploy, Agent build, Cults Recruits' Construct), including **every Shift-queued one**, leaves a **ghost of the building on its site** until construction actually starts."
- `tunnels.md > Losing Tunnels > Defeat`: "with no Tunnels owned, a new Tunnel costs 0 Supplies (see Cost Scaling)".
- Q0061 and Q0062 for the gaps.

**Code**
- **Split first.** `placement/check.rs` (474 lines).
- **The order.** `OrderKind::BuildTunnel { origin, orientation }`, in the Deploy pattern:
  - the gate checks in knowledge mode;
  - the arrival step, `step_deployments`, generalises to every worker build;
  - failure emits `SimEvent::BuildFailed`;
  - `Sim::pending_deploys` generalises to every worker build's lingering ghost.
- **Placement.** `Method::WorkerBuilt` for the Tunnel, plus the faction constraint: its Tier 1 area overlaps no current area of the owner's Tunnels (Q0062, own Tunnels only). The ghost judges it in `Known` mode, and the arrival in `Arrival` mode, together with the reservations of other pending builds.
- **Supplies** (Q0061).
  - The price is the number of Tunnels owned, those under construction included, when construction starts.
  - The gate checks the current price when the order is given.
  - On arrival, a price the player can't pay fails the order, as an invalid site does.
- **`Construction`** (T31). `Object::construction` holds:
  - the progress, 2^16 fixed point, over 480 frames;
  - the builder, held at `Place::Site`;
  - the price paid.

  `Object::max_hp()` gives ConstructionHP's maximum, with damage kept (Q0061 point 4). `step_sites` runs in the economy phase, beside `step_transforms`.
- **Completion.** The Tunnel becomes operational: an exit, a source of Tunnel Space, a drop-off, an Enter and Eject target. The Agent moves from `Place::Site` to `Place::Network`.
- **Cancel and destruction.**
  - `TunnelCommand::CancelConstruction` refunds 100%, removes the site, and puts the Agent back on the map at the footprint's centre.
  - Destruction, in the removal path, also puts the Agent back on the map; the Supplies are lost. This is written as the removal path's rule for the units held at a removed object, with one arm per `Place`. A transport's passengers, who die with it (0220), are then one more arm later (T30).
- **While under construction**, the site offers only Cancel: no exit, no drop-off and no Tunnel Space (Q0061 point 3).
- **Defeat.** A site counts as a structure, so cancelling the player's only one defeats them in that tick (Q0061 point 7).

**Acceptance**
- **The flow.** Order, walk, a site at 60 HP (10% of 600), 600 HP after 480 ticks with damage kept, the Agent in the network, then ejected.
- **Cost.**
  - The first Tunnel built, with one owned, costs 1.
  - A second, while the first is still under construction, costs 2.
  - With no Tunnels, 0.
- **Overlap.** An overlapping site is refused at the gate (the player knows its own areas) and on arrival.
- **Failure.** A unit standing on the site at arrival fails the order: the ghost goes, and the Agent idles.
- **Cancel and destruction.**
  - The Agent appears at the footprint's centre.
  - Cancelling refunds the price; destruction loses it.
  - The Agent's Tunnel Space stays used throughout.
- **Untargetable.** Enemies can't target the Agent during construction, since it is off the map. Their attacks hit the site.
- **The lingering ghost** is shown to its owner only, and blocks the owner's other placements.
- **Defeat.** Cancelling the player's only structure defeats them in that tick.
- **QA screenshots**: the site at 10%, and the finished Tunnel.
- **Goldens.** `Object::construction` changes every hash once. Re-bless, explaining that.

**Owns**: `orders/build.rs`, `construction/site.rs` and `sim/sites.rs` (new); the generalised ghosts from `sim/deploy.rs`; the Tunnel's constraint in `placement/`; `TunnelCommand::CancelConstruction`; Supplies prices. **Read-only**: `network.rs`.

**After**: C (the held builder). **Model**: Opus.

#### F: Expansions (Opus)

**Goal.** A Tunnel builds expansions in its Tunnel Area.
- The ExpandMenu leads to a ghost within the area.
- Placing pays, and starts the construction, which takes the Tunnel's one operation.
- On completion, the expansion works.
- Tunnel Space falls by the expansion's cells divided by 4, rounded as the formula says.

**Design**
- `tunnels.md > ExpandMenu`:
  - "An expansion is available only if (1) its Tier Requirement is at or below **this** Tunnel's tier, and (2) its TechPrerequisite (if any) is satisfied anywhere in the Tunnel Network";
  - "Click an expansion type: begins construction if the Tunnel is not already performing an operation (enters AwaitingPlacement for that expansion)".
- `> AwaitingPlacement (Expansion)`:
  - "A ghost preview of the expansion follows the cursor within the Tunnel Area, snapped to the grid. Tinted green when valid, red when invalid. The expansion must fit entirely within the Tunnel Area."
  - "R rotates the ghost 90 degrees clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically."
  - "Left-click valid location: places expansion, begins construction (CommandIssuingTransition, returns to DefaultState)".
  - "**Z**: returns to ExpandMenu".
- `> Tunnel Area`: "Underground expansions are placed spatially within the Tunnel Area on the grid, occupying cells just like surface buildings."
- `> Construction and Upgrade Rules`: "A Tunnel can only perform one operation at a time: Constructing an underground expansion, OR Upgrading to the next tier".
- `structures.md > Expansion Requirements`: the Tier Requirement is "checked against the building Tunnel"; the TechPrerequisite is "checked across the whole Tunnel Network".
- `core/entities.md > Direct Placement`: "For underground expansions: validated against the relevant underground spatial rules (e.g., Tunnel Area bounds, no overlap). Surface visibility is not relevant. Surface flatness is not required either". `> All placement methods: no buildings on resources`: "No structure may cover any tile of a Space Crystals Patch or a Supply Delivery Station ... whatever the placement method."
- `tunnels.md > Tunnel Space`: "Building an expansion permanently converts part of a Tunnel's unused tunnel-region into occupied space, directly reducing total Tunnel Space."
- `structures.md > Hideout`: "Cost - 200 Space Crystals", "Build Time - 400 frames (25 seconds)".
- Q0062 and Q0065 for the gaps.

**Code**
- **`Method::Expansion { tunnel }`** (T32). The expansion must lie inside the Tunnel's current area, off its footprint, overlap no underground structure, and stay off resources. The tile rule is Q0062's. No flatness, no units and no visibility are checked.
- **`TunnelCommand::Expand { kind, origin, orientation }`.**
  - The gate checks:
    - the Tunnel is operational, with no operation running;
    - the tier;
    - the prerequisites (K's check; a stub that passes until K);
    - the SC.
  - It pays at placement (Q0065 point 1).
  - It creates the expansion with a `Construction` whose builder is the Tunnel (T31). That site is the Tunnel's operation.
- **ConstructionHP and cancel** by Q0065: the proposal is that ConstructionHP applies, and that X on the site, or on the Tunnel, refunds in full.
- **Tunnel Space** counts the expansion from placement.
- **Production.** A finished Hideout produces through C's release.
- **Client.**
  - The Expand entry on J2's Tunnel template, and the ExpandMenu: the Hideout; K adds the Optics Lab.
  - A `Builder::Tunnel(id)` ghost, held within the area, with the area overlay while placing.
  - A progress bar on the site.

**Acceptance**
- **Placement**:
  - inside and outside the area;
  - on the Tunnel's footprint, over another expansion, on a patch;
  - on a slope (allowed), and over a ground unit (allowed);
  - refused while the Tunnel is busy.
- **Tunnel Space.** From the start's 1/11, a second Hideout gives 1/10.
- **Construction.** 400 ticks, then the second Hideout trains Agents into the network.
- **Cancel** refunds, and frees the operation.
- **Hidden.** The enemy never sees the new Hideout or its ghost; ghosts are for their owner only.
- **Tier Requirement.** A test expansion kind with Tier Requirement 2 is refused at a Tier 1 Tunnel.
- **QA screenshots**: the ExpandMenu; the ghost red and green; the area overlay.
- **Goldens.** No change expected beyond A's and E's fields.

**Owns**: `Method::Expansion`, `TunnelCommand::Expand`, the operation slot, the Tunnel as a builder; client ExpandMenu and `Builder::Tunnel`. **Read-only**: the rest of `templates/syndicate.rs`.

**After**: B, E and J2. **Model**: Opus.

#### G: Tunnel upgrades (Opus)

**Goal.** A Tunnel upgrades to Tier 2, then Tier 3, for Supplies. The upgrade is its one operation. On completion, the Tunnel's HP, area, Tunnel Space and transit grow.

**Design**
- `tunnels.md > Tunnel ObjectInterfaceState`:
  - "**A: Upgrade Tunnel** — upgrades to next tier (CommandIssuingTransition). Costs Supplies per upgrade cost formula. Unavailable if already Tier 3 or if currently performing an operation."
  - "**X: Cancel Upgrade** — cancels the in-progress Tunnel upgrade (CommandIssuingTransition). Full refund of Supplies cost. Only available while an upgrade is in progress."
- `> Upgrade Costs`:
  - "Higher-tier Tunnels count toward lower-tier cost calculations (a T3 Tunnel counts for both T2 and T1 cost scaling)";
  - "**Upgrade to Tier 2**: 2 + 2 x (number of T2+ Tunnels owned), in Supplies";
  - "**Upgrade to Tier 3**: 3 + 3 x (number of T3 Tunnels owned), in Supplies".
- `> Tunnel Area Non-Overlap Rule`: "upgrading a Tunnel is blocked if the radius increase would cause its Tunnel Area to overlap with another Tunnel's current area."
- `> Construction and Upgrade Rules`: "Upgrading temporarily locks the Tunnel out of expansion construction for the upgrade duration."
- `> Losing Tunnels`:
  - "a Tunnel that is mid-upgrade counts at its current (pre-upgrade) tier";
  - reclaiming waits "until that Tunnel is upgraded to a tier where its Tunnel Area fully covers the expansion and meets its Tier Requirement".
- Q0060 for the duration, the HP and the payment.

**Code**
- `TunnelCommand::{Upgrade, CancelUpgrade}`, and the operation `Upgrading { progress }`.
- The duration is a named placeholder citing Q0060.
- At completion, the tier changes: max HP (by Q0060), the area, and transit. H's `reclaim` runs.
- The client enables the panel entries, which J2 shows greyed until G.

**Acceptance**
- **Costs.** The first Tier 2 upgrade costs 2 and the second 4. Tier 3 costs 3, then 6. A Tier 3 Tunnel counts in the Tier 2 price.
- **Blocked.** Refused when a neighbour lies within the Tier 2 area, when the Tunnel is busy, and at Tier 3.
- **During the upgrade.**
  - The Tunnel is still Tier 1 for transit.
  - Cancelling refunds the price.
  - A Tunnel destroyed mid-upgrade loses the Supplies (Q0060).
- **Completion.**
  - Max HP 800; the current HP by Q0060.
  - Tunnel Space +9: (84 − 48) cells / 4.
  - A neutral expansion now fully covered is reclaimed.
- **QA screenshot**: a Tier 2 Tunnel and its area.

**Owns**: `TunnelCommand::{Upgrade, CancelUpgrade}`, the upgrade operation, the tier change, the client's Upgrade entries. **Read-only**: `sim/tunnels.rs`.

**After**: F (the operation) and H (`reclaim`); Q0060 answered. **Model**: Opus.

#### H: Losing Tunnels (Opus)

**Goal.** When a Tunnel is destroyed:
- its expansions turn neutral, visible to everyone, and decay;
- their work in progress is lost, and their queued work is refunded;
- network units that can no longer get out are destroyed.

A new own Tunnel reclaims the neutral expansions it fully covers, and defeat follows the shared rule.

**Design**
- `tunnels.md > Losing Tunnels`:
  - "**Expansions become neutral**: when a Tunnel is destroyed, every expansion in its Tunnel Area becomes neutral — no longer owned by the Syndicate player."
  - "Neutral expansions are **no longer hidden** (visible to everyone, no detection needed)."
  - "They **slowly lose HP** until destroyed. Decay rate: TBD."
  - "**Reclaiming**: if a Tunnel is built whose Tunnel Area fully covers a neutral expansion, and that Tunnel's tier meets the expansion's Tier Requirement, the expansion becomes owned by that Tunnel's owner — the original Syndicate player or an enemy Syndicate player — and stops decaying. It occupies that Tunnel's tunnel-region like any expansion (reducing Tunnel Space). An expansion only partly covered by the new Tunnel Area, or whose Tier Requirement is higher than the new Tunnel's tier, stays neutral (and keeps decaying)".
  - "**In-progress work is destroyed**: any production or research in progress in the expansion when its Tunnel is destroyed is lost."
  - "**Queued work is refunded**: units or research queued in the expansion but not yet started are removed and their full cost is refunded to the player."
  - "**Stranded units**: whenever a Tunnel is destroyed, any unit inside the Tunnel Network that can no longer exit through any remaining Tunnel (no remaining Tunnel's tier is sufficient for its unit base category) is **destroyed**."
  - "**Defeat**: the shared rule applies ... Losing the last Tunnel therefore defeats the Syndicate only if they own no Laser Turrets."
- `core/game_rules.md > Defeat Condition`: "**The Syndicate**: when a Tunnel is destroyed, all of that Tunnel's expansions become neutral". `> On Defeat`: "**Remaining units become neutral.**"
- Q0069 for the gaps.

**Code**
- **In the removal path** (T38): for each removed Tunnel, in id order:
  - its own expansions, those fully inside its area, become ownerless;
  - their production: the work in progress is lost and the queue refunded. This is a separate path from a destroyed producer's, which refunds everything;
  - their rallies are cleared;
  - expansions under construction follow Q0065 point 7.
- **Decay.** `step_decay` in the economy phase: neutral expansions lose HP at the placeholder rate (Q0069 point 1) until destroyed.
- **Stranded units.** After the removals, every held unit that no remaining operational Tunnel of its owner lets out is removed, with `SimEvent::Removed` and its cause.
- **Reclaim.** `reclaim(tunnel)` runs when a Tunnel completes (E's site) or finishes an upgrade (G). Every neutral expansion fully inside the area, with a Tier Requirement at or below the tier, goes to the Tunnel's owner and is hidden again.
- **Vision.** A neutral expansion is visible to everyone and remembered. Once reclaimed, it is hidden, and an enemy's record of it drops when the enemy sees its tiles again.
- **Client.** Neutral expansions drawn for everyone in the neutral colour, with an HP bar that falls.

**Acceptance**
- **The only Tunnel destroyed**, with its Hideout:
  - the Hideout turns neutral, becomes visible to the enemy, and decays;
  - with no other structure (no Laser Turret exists yet), the Syndicate is defeated;
  - its Agents on the map become neutral, and the units in the network are destroyed.
- **One of two Tunnels destroyed**:
  - its Hideout turns neutral;
  - infantry in the network survive, since a Tier 1 Tunnel remains;
  - the Hideout's unit in production is lost, and its queued units are refunded.
- **Reclaim.**
  - A new Tunnel whose area fully covers the neutral Hideout takes it back, hidden again.
  - One that covers it partly leaves it neutral (Q0069 point 4).
- **The free rebuild.** With no Tunnels, the next Tunnel costs 0. Tested on a player who also owns another structure (a debug spawn), since no Laser Turret exists yet.
- **Ignored tests.** This slice changes the defeat rules, so run the ignored suite afterwards (insights).

**Owns**: `sim/tunnels.rs` (new), its hook in `sim/removal.rs`, the defeat tests, the client's look for neutral expansions. **Read-only**: `network.rs`.

**After**: C and E. **Model**: Opus.

#### I: Supplies (Opus)

**Goal.** Agents gather Supplies.
- A right-click on a Supply Delivery Station starts the job.
- The Agent picks up 1 Supply in 48 ticks and carries it to Side C, the Supply hatch, of the Tunnel it took for this Gather (Q0066 point 2).
- It drops off in 48 ticks, one Agent at a time at Side C, and goes back.

**Design**
- `units.md > Agent > Gathering > Supplies`: "CarryCapacity: 1 Supply per trip", "PickUpDuration: 48 frames (3 seconds)", "DropOffDuration: 48 frames (3 seconds)", "Drop-off side: Tunnel Side C".
- `units.md > Agent > Right-Click Resolution`: "Supply Delivery Station → Gather supplies".
- `core/map.md > SupplyDeliveryStation`: "The delivery countdown begins only once the station's current supply content reaches 0. Factions collect supplies from the station using faction-specific collection mechanics."
- `overview.md > SyndicateResources > Supplies`: "Gathered from Supply Delivery Stations by Agents. Spent on Tunnel construction, Tunnel upgrades, and research."
- Q0067 for the gaps.

**Code**
- `WorkerDef`'s second resource (T34). `OrderKind::Gather(station)`.
- The pick-up as timed work. The station's `current_supplies` falls by Q0067 point 2.
- Side C uses D's side rule. Deliveries go to the bank's `supplies`, with `ResourcesDelivered { resource }`.
- `resolve`: a station gives Gather, for Agents.
- Switching between resources by Q0067 point 4.

**Acceptance**
- **One Agent** at a station holding 4: four trips empty it, the 960-tick countdown runs, and 4 Supplies arrive again. The bank gains 4.
- **Two Agents** at one station at once (Q0067 point 1).
- **Both sides at once**: one crystal delivery at Side B and one Supply delivery at Side C.
- **An empty station**: the Agent waits there (Q0067 point 3).
- **QA screenshot**: an Agent carrying a Supply to Side C.

**Owns**: the station leg and pick-up in `orders/gather*` and `economy/gathering.rs`, the station decrement, Side C. **Read-only**: D's side rule.

**After**: D. **Model**: Opus.

#### J1: Agent UI (Opus)

**Goal.** The Agent is played from the UI: its panel, the Build Tunnel ghost, the site's Cancel, and the right-click pointers (Gather, Drop Off, Enter).

**Design**
- `units.md > Agent ObjectInterfaceState > DefaultState commands`:
  - "**A: Build Tunnel** — enters AwaitingPlacement for a Tunnel (StateOnlyTransition). Ghost preview follows cursor, snapped to grid. Tinted green when valid, red when invalid. R rotates 90° clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically. Left-click valid location confirms placement and dispatches the Agent (CommandIssuingTransition, returns to DefaultState). Escape/right-click cancels back to DefaultState."
  - "**B: Drop Off Resources** — targeted command (CommandIssuingTransition). Requires clicking an own Tunnel. Agent walks to the appropriate side automatically (Side B for crystals, Side C for supplies). Always visible, greyed out when Agent is not carrying resources."
  - "**Build Laser Turret** (hotkey TBD) ... Visible but greyed out until the player owns an Optics Lab."
- `> Right-Click Resolution`: the list of six, and "Right-click commands are issued to all selected Agents even though the Agent is ungroupable (no shared control panel — the panel displays one Agent's interface)".
- `selection_and_hud.md > Shift while placing`:
  - "an Agent goes inside the structure it builds ..., so each Agent takes exactly one placement";
  - "Placement **never moves on to another selected worker** automatically ...: to place with the next Agent, the player presses Tab and issues the next order."
- `> PointerDisplayType`: "Cursor over own Tunnel (Syndicate unit, tier sufficient): **Enter**"; GatherResources; ReturnResources.
- `core/entities.md > Lingering build ghosts`: "Visible to its owner."
- Q0068 for the letters.

**Code**
- **`templates/syndicate.rs`:**
  - the Agent: Q Move, A Build Tunnel, S Drop Off (targeted, greyed when the Agent carries nothing), X Stop. Build Laser Turret shows greyed with the reason "not available yet" (Q0068 point 1);
  - the site: X Cancel Construction only.
- **`Builder::Agent(id)`** in `placing.rs`:
  - the knowledge-mode ghost (the Deploy ghost's), with R and F;
  - an 8x8 area outline, and the owner's areas;
  - a Shift-click sends the order and ends placement.
- The lingering Tunnel ghosts (indicators).
- The Enter pointer; GatherResources over stations; ReturnResources over own Tunnels while carrying.
- The carried load, crystals or a Supply, drawn on the Agent.

**Acceptance.** Client tests in `tests/commands/syndicate*.rs`:
- A, then the ghost, then a click gives the order;
- the ghost is red over the owner's own area;
- Shift and a click ends placement, and Tab moves to the next Agent;
- S is greyed when the Agent carries nothing;
- each right-click gives its outcome, with its pointer.

Plus the screenshot set.

**Owns**: client `templates/syndicate.rs` (Agent, site), `Builder::Agent` in `placing.rs`, the Enter pointer, the lingering Tunnel ghosts, the carried load. **Read-only**: the sim.

**After**: C, D and E (and I, for the Supply icon, if merged). **Model**: Opus.

#### J2: Tunnel and Hideout UI (Opus)

**Goal.**
- The Tunnel's panel ejects units through the EjectMenu.
- The selected Tunnel shows its area and the network's contents.
- The Hideout trains and rallies from the UI.

**Design**
- `tunnels.md > Tunnel ObjectInterfaceState > DefaultState commands`: A Upgrade, B Expand, C Eject, X Cancel Upgrade (the letters are Q0068's question).
- `> EjectMenu`:
  - "Displays a grid of unit type tiles representing all units currently in the Tunnel Network. Each tile shows the unit type icon and a count of how many of that type are in the network. Unit types whose base category exceeds this Tunnel's tier are visible but greyed out (disabled)."
  - "**Z**: returns to DefaultState".
- `structures.md > ObjectInterfaceState[Hideout]`:
  - "Right-click Ground: issues SetRallyPoint command to that location";
  - "**Q: Build Agent**"; "**W: Build Guard**"; "**E: Build Grenadier** (hotkey tentative)"; "**X: Cancel Production**"; "**C: Set Rally Point**".
- `selection_and_hud.md > Standard Slot Assignments`: Z goes back, X cancels, C sets the rally. "Clicking the **producing building itself** clears the rally point".
- `> Selection changes while targeting`: "**Automatic removals** (a selected unit dies, or leaves the selection by entering a building, transport or tunnel) do **not** cancel targeting."
- `> Rally points`: "a **line** from the building to the rally point".

**Code**
- **`templates/syndicate.rs`:**
  - the Tunnel, with Q0068's letters. Upgrade and Expand show greyed ("not yet") until G and F;
  - the EjectMenu: one tile per kind in the network, with its count, greyed by tier;
  - the Hideout, through the Production template: Q Agent; K adds W Guard.
- **The Tunnel Area overlay** for a selected own Tunnel. J1 and F also use it while placing.
- **The selection panel**: the Tunnel's tier, the progress of its operation, and the network's contents.
- **Selection.** Held units leave the selection and stay in their control groups (Q0063 point 4). A recall selects only the members on the map.
- **The rally display.** With a rally on the parent Tunnel, or none, the panel says the units stay in the network.

**Acceptance.** Client tests:
- the Eject letter opens the menu, and a tile sends an Eject command;
- the counts update;
- a test vehicle's tile is greyed at a Tier 1 Tunnel;
- the overlay is drawn;
- Q on the Hideout trains;
- with the Hideout selected, a right-click on the Tunnel sets the rally there, and the units stay in the network.

Plus screenshots.

**Owns**: client `templates/syndicate.rs` (Tunnel, EjectMenu), `tunnel_areas.rs` (new), the network's contents in `hud/selection_panel.rs`, selection and control groups for held units. **Read-only**: the sim.

**After**: B and C. **Model**: Opus.

#### K: Prerequisites and the Guard (Opus)

**Goal.** Tech and unit prerequisites work for every faction. The Syndicate's Optics Lab, on Q0059's placeholder stats, unlocks the Guard at the Hideout.

**Design**
- `core/tech_and_research.md > Tech Prerequisites`:
  - Counting Rule: "**Syndicate**: the underground expansion must have finished construction"; "Structures under construction (or awaiting placement) never satisfy a TechPrerequisite."
  - Check Timing: "checked only at the moment construction is **started**".
  - Unit Prerequisites: "checked only when the unit is queued"; "units already queued or in production continue and complete normally".
- `structures.md > Hideout > Produces`: "Guard: 125 Space Crystals, 120 frames (7.5 seconds). Unit Prerequisite: player owns at least one Optics Lab (2026-11-24)." "**W: Build Guard**: ... Only available if the player owns at least one Optics Lab".
- `structures.md > Optics Lab`: "Tier Requirement - 1", "TechPrerequisite - None", "(Full stats TBD)".
- `units.md > Guard > TechPrerequisite`: "So the Optics Lab is the Syndicate's gate to an army; before it, only Agents (pistol) can fight."

**Code**
- **`tech.rs`** (new; T37):
  - `requires: &'static [ObjectKind]` on recipes and on structure defs;
  - `Sim::owns_completed(player, kind)`: owned, neither under construction nor transforming, and not neutral;
  - gate checks when a unit is queued and when construction starts;
  - panel entries greyed, with the reason (`conditions.rs`).
- **The Optics Lab.** `OPTICS_LAB`, on Q0059's placeholders as named constants citing Q0059, with its ExpandMenu entry.
- **The Guard.** Its recipe at the Hideout.
- **Other factions.** No existing kind has a prerequisite, so nothing else changes.

**Acceptance**
- The Guard is refused without an Optics Lab, allowed with one, and refused while the Lab is under construction.
- Queued Guards continue after the Optics Lab dies.
- A test expansion with a prerequisite is satisfied by a structure in another Tunnel's area.
- The panel greys the entry and gives the reason.
- QA screenshot.

**Owns**: `tech.rs` (new), `requires` in `defs/structures.rs`, `OPTICS_LAB` and the Guard recipe in `defs/syndicate.rs`, the gate checks, the client's `conditions.rs`. **Read-only**: none.

**After**: F; Q0059 answered. **Model**: Opus.

#### L: Syndicate oracle and goldens (Opus)

**Goal.** The Syndicate's economy is checked against the design's figures, and two goldens pin Syndicate games.

**Design**
- `units.md > Agent > Gathering`: the saturation table (0–6 GU: trip, SC per second per Agent, payback, Agents to saturate, income with 2, 3 and 4 Agents) and "a Tunnel takes in at most 50 Space Crystals per 4 seconds = **12.5 Space Crystals per second**".
- `tunnels.md > Starting Conditions`: "With 100, the second Agent can start immediately and a starting Tunnel is saturated (3 Agents, 12.5 Space Crystals per second) around 0:40."
- `tunnels.md > Tunnel Area Non-Overlap Rule` ("Crystal intake and stacked Tunnels"): "Two Tier 1 Tunnels may share a rich crystal field (doubling its intake)".
- `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`: "the Syndicate should have the lowest one-base saturation income and expand fast, gated by Supplies".
- Q0070 for the reference layout and the tolerance.

**Code**
- **`testing::syndicate`**: a build line through the gate (Train, Eject, Gather, Gather Supplies, Build Tunnel, Expand), with `Books`.
- **The reference map**, by Q0070: Tunnels with Side B at 0–6 GU from a patch.
- **`tests/economy_oracle/syndicate.rs`**: trips by distance, income per Agent, income with 2, 3 and 4 Agents, the ceiling, the opening ramp, stacked Tunnels. The table goes into `docs/economy_oracle.md`.
- **Goldens**, through `testing::syndicate::skirmish`:
  - `syndicate_opening`, one player;
  - `syndicate_vs_gdo`, both players scripted, with a Tunnel lost and a defeat.

**Acceptance**
- The oracle passes within Q0070's tolerance. Otherwise the measured table goes to the Designer as a question, by 0108's rule, and the figures are pinned.
- The goldens are blessed.
- The GDO and Colonists oracles are unchanged.

**Owns**: `testing/syndicate.rs`, `tests/economy_oracle/syndicate.rs` and `tests/golden/syndicate.rs` (new), `docs/economy_oracle.md`. **Read-only**: everything else.

**After**: B, C, D, E, H and I. **Model**: Opus.

#### M: M4 QA (Opus)

**Goal.** Perf, a soak, a scripted walkthrough, the docs and the ignored suite.

**Tasks**
- **`agents_perf`** (ignored): 4 Tunnels with 4 Agents each, plus 60 Guards cycling through Enter and Eject. Record the mean and p99 tick, A/B against master.
- **`syndicate_soak`** (ignored, 10 minutes of the golden game):
  - the invariants, the network's included;
  - the books;
  - no stuck order;
  - the hash stable over two runs.
- **The walkthrough.** `--factions syndicate,gdo --input-script` through section 2, plus F, G and K, with screenshots at fixed ticks.
- **`docs/playtest/m4.md`**, in `m3.md`'s sections.
- **`ARCHITECTURE.md`.** Review "Adding a faction" against M4, and add the new items: held units, sites, the underground layer, prerequisites.
- **The ignored tests.** `cargo test --release --workspace -- --ignored --test-threads=1` on a quiet machine.

**Owns**: perf and soak tests, the QA scripts, `ARCHITECTURE.md`, `docs/playtest/m4.md`. **Read-only**: none.

**After**: everything. **Model**: Opus.

#### Gated slices (not scheduled; each waits for its numbers)

- **N1 Grenadier.**
  - Needs its numbers and the Chemical Plant's stats.
  - First a combat spike: DoublyDisjointed projectiles, AoE damage with overlap shares (`systems/combat.md > AoEDamage`), Attack Ground and `CanTargetGround`.
  - Then the unit, behind the Chemical Plant (K's prerequisites). The Hideout's E is "(hotkey tentative)".
- **N2 Laser Turret.** Needs its numbers. Then:
  - the upright and pop-up forms;
  - conversion when Tunnel Areas change;
  - a 1x1 cell of tunnel-region;
  - the Agent's build flow, emerging "adjacent to the finished structure";
  - the Tunnel's ExpandMenu entry;
  - the operation-slot question.
- **N3 Vehicle Assembly, Assault Bike, Laser Battery, Subterranean Transport.**
  - Needs the numbers (the Vehicle Assembly's too), and the vehicles milestone: FixedTurnRadiusMovement, turret arcs, 0033 and 0035.
  - Transit through Tier 2 Tunnels is already built (C).
  - **The vehicle deploy order** (T44; 0211, 0213, 0214) comes with the first vehicle that deploys: the Laser Battery's Deploy and Undeploy, or the Transport's Burrow and Surface (0220). The GDO's Artillery Rig, not built yet, reuses it.
  - **The Laser Battery** is behind the Optics Lab (0210; K's prerequisites, Q0059's stats). Its beam and surges are new combat work.
  - **The Subterranean Transport** (0215–0222) is behind a Forge (0221; N5's stats). Its pieces:
    - the DrillUnit base. It exists in the defs (`UnitBase::DrillUnit`, with `NavClass::Underground` over Drillable tiles), but no kind uses it yet;
    - no collision underground (`systems/units.md > Underground Collision`). `Domain::hard_collision` still makes underground units collide. That was M1's assumption for gap G14 (`docs/movement.md`), and the design has since settled it the other way;
    - the surface check (0217: the tiles' tracked passability, and no building), and the footprint reserved while surfacing;
    - hidden underground (0219, through T32's predicate), with area damage still hitting it (0218; after N1);
    - targeting surfaced and submerged units: the TODO at `combat/targeting.rs:41`;
    - loading and unloading, a new mechanic;
    - passengers held in the carrier (T30), who die with it (0220) and stay aboard in the network (0222).

    It also needs Q0071's answers.
- **N4 Shadebringer.** Smoke that blocks sight is a new kind of vision. Also slow fields, and tanks refilled in the network.
- **N5 Null Chamber, Vault, Chemical Plant, Forge, Aerospace Assembly, Orbital Command.** These are data on F's and K's machinery once their stats exist. The Null Chamber's buff also needs detection. The Forge also gates the Transport (0221), and the Aerospace Assembly produces the Phantom (N6).
- **N6 Phantom** (0226–0228), with aircraft and the Tier 3 air doors.
  - Needs its numbers, the Aerospace Assembly's stats (Tier 3, behind a Forge), and Q0059's Optics Lab.
  - No air unit exists for any faction yet: air movement and air collision are plan.md's "Later: air". `UnitBase::HoverCraft` exists in the defs, unused.
  - The air doors: where they are, and whether they take one aircraft at a time (Q0064 point 11).
  - Its turret's 60–90° cone shares the turret-arc work with the vehicles milestone.

---

## 4. Technical decisions for the Director

### T29. Syndicate kinds in the M2 and M3 model (A)
- **Recommend:** one static per kind in `defs/syndicate.rs`, in the pattern of `defs/gdo.rs` and `defs/colonists.rs`.
  - Append `Agent`, `Tunnel` and `Hideout` to `ObjectKind`; K appends `OpticsLab`.
  - **Structure fields:** `StructureDef::underground: bool` (T32) and `StructureDef::tunnel: Option<&'static TunnelDef>`, the tier table (T39).
  - **The Hideout's SymmetryType** is silent. AAAA, since none of its sides has a function.
  - **Worker and drop-off.** The Agent's `worker` is filled in by D and I. The Tunnel's `drop_off` is true, with the side rule in its def (T34).
  - **The army cost** stays `UnitDef::tunnel_space_cost`, next to `unit_control_cost` and `beacon_capacity_cost`.
- **Alternatives:** a generic `ArmyCost` on `UnitDef`, tidier with three caps, but it churns M2 and M3 code now. M5 can decide it when the Cults' Unit Control arrives.

### T30. Units off the map: one held store (C, E). The hard piece, on Fable.
- **Recommend:** `Sim::held: BTreeMap<ObjectId, Held { object: Object, place: Place }>`, with `Place::Network` and `Place::Site(ObjectId)`.
  - **Hashed and serialized**, through the destructured `Sim`. Ids are kept.
  - **The network's contents** are derived from `held` by owner, for the EjectMenu and for Tunnel Space.
  - **Leaving the map** reuses `remove_objects`' reference cleanup without its effects of a death. Split `forget` out of `sim/removal.rs` for this.
    - Orders, attacks and follows naming the unit end.
    - Rallies on it reset.
    - Its own orders, mover state and attack state are cleared.
  - **Fails closed.** Every loop over `objects` skips held units by construction: movement, collision, vision, targeting, auto-targeting, picking and records. A forgotten case can't show, move or hit a unit that isn't there.
  - **What must count them** says so explicitly: Tunnel Space, the stranded-unit rule, defeat-time cleanup, the client's control groups, the invariants and the hash.
  - **M5 fit.** `Place::Building(id)` would hold the Cults' internal pools (Altar, Armory, Temple, Gunworks, Shrine). There, "Evacuation on Destruction" (`factions/cults/economy.md`: "Everyone inside is evacuated alive") is the opposite rule to stranded units, which suits a per-place rule. The GDO Bunker's occupants fire from inside, so they would stay on the map.
  - **Transport fit (0215, 0220–0222; deferred, N3).** M4 needs no case of a unit held inside a held unit, since the Transport is deferred. But the store must not rule one out.
    - A passenger would be held at `Place::Carrier(transport)`. The place names the carrier, not where the carrier is, so passengers follow it through Enter and Eject without being moved (0221: "They stay aboard while it travels through the network").
    - The per-place rules decide the rest:
      - passengers die with the carrier (0220). This is the opposite of a site's Agent, who "survives and emerges", and of the Cults' evacuation;
      - they can't be ejected on their own (0222). The EjectMenu lists `Place::Network` only;
      - whether they count for Tunnel Space is T33's.
    - C and E must therefore avoid two things: matching `Place` with a wildcard, and assuming that a held unit's place is on the map.
- **Alternatives:**
  - keep held units in `objects` with an "in the network" flag. Every system then needs a filter, so it fails open: one missed filter shows, moves or hits a unit that isn't there;
  - remove and re-create with a new id. Selection, control groups, replays and kill credit lose track, and the design speaks of the same unit coming out.

### T31. On-site construction: one `Construction` state (E, F)
- **Recommend:** `Object::construction: Option<Construction { progress, frames, builder: Builder, paid: Cost }>`, with `Builder::Held(ObjectId)` (the Agent in its site) and `Builder::Tunnel(ObjectId)` (for an expansion).
  - **The object is its kind from the first frame** (T15's rule). It blocks, or not, as its layer says; it is visible, or hidden; it counts for defeat and records.
  - **Its functions wait for completion**: exit, drop-off, Tunnel Space, production.
  - **HP.** `Object::max_hp()` gives MaxHP × (10% + 90% × progress), and damage stays taken (the Warp In rule, Q0061 point 4).
  - **Progress** in the economy phase (`step_sites`, beside `step_transforms`), 2^16 fixed point.
  - **M5 fit.** Cults construction scales with the builders: "Construction speed scales proportionally with the number of Recruits currently assigned", stalls at zero, and cancels at 75%. A `rate` from the builder count, and `Builder::Recruits(...)`, extend this without a second mechanism.
- **Alternatives:**
  - reuse `Transform`: its revert semantics (Drone ↔ Beacon) don't fit. A site has nothing to revert to, and a cancel removes it;
  - GDO-style constructors: they build off the map and then place, which is not on-site construction.

### T32. The underground layer: a structure flag (A, F, H)
- **Recommend:** `StructureDef::underground`. An underground structure:
  - **blocks nothing**: no nav blocker (`BlockerKind::UndergroundExpansion`, which already blocks nothing), no collision, no path-knowledge record;
  - **lives in its own layer** for placement. `Method::Expansion` checks underground structures; the surface methods ignore them, except as Q0062 says;
  - **is hidden from other players while owned**, through one predicate, `Sim::hidden_from(object, player)`. `object_box_state`, records and targeting use it. It is ready for detection and for the pop-up Laser Turret later.
    - It takes an object, not a structure. 0219 hides every DrillUnit underground in the same way ("hidden from other players unless revealed by a detector"), and the Transport (N3) adds that arm.
    - Damage resolution must not use it. Area damage still hits a hidden DrillUnit (0218: "AoE resolution includes hidden underground units in the area");
  - **is visible when neutral**;
  - **takes its elevation from the terrain at its centre**, since slopes are allowed.
- **Its parent Tunnel is derived**: the owner's Tunnel whose current area fully contains it. That Tunnel is unique, because an owner's areas never overlap, which is an invariant (T42). The rally auto-eject, losing Tunnels and Tunnel Space use it.
- **Alternatives:**
  - `Domain::Underground` on structures. Domain belongs to unit bases, and the targeting rule for underground units ("Ground attacks can target Ground and surfaced Underground units") isn't the expansions' rule;
  - a stored parent id, which would have to be kept right through destruction and reclaiming.

### T33. Tunnel Space: a derived army cap (B)
- **Recommend:** `ArmyResource::TunnelSpace`, beside Unit Control and Beacon Capacity.
  - **Available** = min(200, ⌊unused / 4⌋). Unused is summed over the owner's operational Tunnels: the area's cells on the map (Q0062 point 2), less the 16 of the footprint, less the owner's underground cells inside.
  - **Used** = the sum of TunnelSpaceCost over the owner's units on the map, held, and in production. The check at production start is 0143's, already shared.
  - **Recomputed when asked.** The few Tunnels make it cheap, and nothing is stored.
  - **One function for Used.** 0222 says that inside the network "only the transport's own Tunnel space counts". That may exempt its passengers (Q0071 item 6). If so, the rule lands in this one function.
- **Alternatives:** counters updated by events: fast, but prone to drift, and M2 chose "derived, not stored" for power and Unit Control.

### T34. The Agent as a worker: one Gather job (D, I). On Fable for D.
- **Recommend:** one Gather job for both factions.
  - **`WorkerDef`** gets one spec per resource: crystals, and Supplies (optional). It also gets `shares_patches`, true for Agents.
  - **The drop-off rule comes from the target's def:** `DropOff::Anywhere` (the Egress Node), or `DropOff::Side { crystals: B, supplies: C }`, one Agent at a time (the Tunnel).
  - **Side occupancy is derived from worker states as 5a begins,** as patch occupancy is. Waiting Agents are served by arrival tick, then id. Derived occupancy can't outlive the order that holds it (the M3-H lesson).
  - **The drop-off target** is the Drone's AssignedDropOff, auto-assigned, with no Set Drop-off command.
    - As a Gather's first return starts, the Agent takes the nearest own operational Tunnel, by straight line to the side's middle.
    - It keeps that Tunnel for the rest of the Gather. It takes the nearest again only when the Tunnel is lost (0170) or a new Gather is given (Q0066 point 2).
    - Re-choosing at every return would make Agents between two stacked Tunnels switch between them on small differences in where they stand (0223).
  - **The line** (Q0066 point 3). Waiting Agents are served in arrival order, and one that can't reach the side follows the normal Failure rule. The Drone's `LEG_RETRY_TICKS` isn't used for Agents: there, a leg back that ends short is tried again after 2 s, a placeholder with no design rule. Whether the Drones keep it under 0223 is M3's business.
  - **Pass-through stays the Drone's** (0106: "Drones only").
  - **The Drone's behaviour must not change.** The Colonists goldens and oracle are the guard.
- **Alternatives:**
  - a separate Agent job, which duplicates about 500 lines of edge cases already settled for the Drone (0123 to 0141);
  - a queue stored on the Tunnel, which needs cleaning up on every order change.

### T35. Entering and ejecting at Side A (C)
- **Recommend:**
  - **Enter** is an order with a leg to the entry path's spot, and an arrival step right after 5a. This is the Deploy pattern: it runs before 5b, so the grids never include a unit that has gone in.
    - The entry path takes one unit at a time (0225). The first to arrive within reach goes in when the path is free, and the path then stays busy for 8 ticks (Q0063 points 2 and 7).
    - The tick it last took a unit lives in `TunnelState`, hashed.
  - **Ejection** runs in the production step, through the release point that produced units use (`release_unit`).
    - It uses the exit path's one fixed spot (0225; Q0064 point 1). A unit emerges there only if its silhouette overlaps no unit and no structure, in the spirit of `systems/units.md > Underground Collision > Surfacing`; otherwise the queue waits. Nothing searches along the side (0223).
    - At most one unit per 8 ticks per Tunnel, and none while the last is still on the spot (Q0064 point 10).
    - The queue reserves specific units, and is hashed in `TunnelState`.
  - **Old ids come back.** A unit that re-enters the map keeps its old id, so the steps after production that use `first_new` must treat it as arrived this tick. Audit `update_supply_stations`, vision, `remember` and `follow_targets`.
  - **Air units** can't enter or leave in M4. Aircraft use the Tier 3 air doors (0224), which come with the first aircraft (N6).
- **Alternatives:**
  - the clear spot along Side A nearest its middle, the first draft's default. It is a search that 0223 rules out ("re-checking for spots opening along a row"), and 0225 replaces it with fixed spots;
  - the doorway rule, half inside the footprint. The design excludes it: "Aircraft and units released underground (e.g. Syndicate Tunnels) are not covered by this rule";
  - ejecting in 5a, where units would join this tick's collision before the grids know them.

### T36. Commands (C, E, F, G, I)
- **Recommend:** one command enum per capability, M2's convention:
  - **the Tunnel:** `Command::Tunnel { tunnel, command }`, with `TunnelCommand::{Eject { kind }, Expand { kind, origin, orientation }, Upgrade, CancelUpgrade, CancelConstruction}`. CancelConstruction also serves an expansion's own site; its target is the site;
  - **the Hideout:** the existing `Command::Production`;
  - **the Agent's orders:** `OrderKind::{Gather, DropOff, Enter, BuildTunnel}`. Gather and Drop Off are generalised to Tunnels and stations.

  Each slice extends the `every_command` tests.
- **Alternatives:** `Command::Ability` for Eject. Abilities are unit modes, and Eject is a structure's command.

### T37. Tech and unit prerequisites: one generic check (K)
- **Recommend:**
  - **Data:** `requires: &'static [ObjectKind]` on recipes and on structure defs, each meaning "owns at least one".
  - **One check:** `Sim::owns_completed`. The gate checks when a unit is queued and when construction starts, and the client greys the entry with the reason.
  - **Generic:** M5's Gunner (needs a Gunworks), the Cults' aircraft (need a Scrap Hangar), the Colonists' Skylance and Plasma Turret, and GDO buildings later.
- **Alternatives:** a check per faction: three copies by M5.

### T38. Losing Tunnels in the removal path (H)
- **Recommend:** `remove_objects` runs the Tunnel rule for every removed Tunnel, all in id order in the same tick:
  1. neutralise its expansions, found by T32 before the Tunnel leaves;
  2. refund their queued work and drop their work in progress. This is a separate path from "producer destroyed", which refunds everything;
  3. then strand: remove the held units that no remaining operational Tunnel lets out.

  Decay runs in the economy phase. Reclaiming runs at completion: E's site and G's upgrade. Defeat is unchanged: the owned-structure count.
- **Alternatives:** a separate pass at the end of the tick. Defeat and vision would then see, for one tick, expansions still owned under a Tunnel that is gone.

### T39. Tunnel tiers: one kind with a tier (A, G)
- **Recommend:**
  - one kind, with hashed `TunnelState { tier, operation, eject }`;
  - the tier table in `TunnelDef`, and `Object::max_hp()` by tier;
  - the cost formulas count Tunnels by tier from the objects.
- **Alternatives:** three kinds, Tunnel1–3, with `change_kind` on upgrade (the pattern of the Colonists' portal line). The stat blocks come for free, but the design has one Tunnel with tiers, and counts, selection and records would see a kind change.

### T40. Start, layouts and validation (B)
- **Recommend:** the T11 and T24 pattern. Starting Conditions and the `dev_skirmish` layouts go in the same slice, because an implemented faction's layouts become mandatory on Skirmish Default maps.
  - Validation adds the rule that an expansion lies in its Tunnel's area, through `Method::Expansion` in `Mode::Layout`. The editor, validation and the game then agree.
  - B checks the area's bounds directly until F adds the method.

### T41. Client (A, F, J1, J2)
- **Recommend:**
  - `templates/syndicate.rs`: the Agent, the Tunnel, the site, and the ExpandMenu and EjectMenu as panel menus, like the Deployment Center's build menus;
  - `Builder::Agent(id)` and `Builder::Tunnel(id)` in `placing.rs`;
  - a Tunnel Area overlay;
  - underground structures drawn as outlines for their owner, and neutral ones for everyone;
  - `ResourceFigure::TunnelSpace`; red lasers by faction; the Enter pointer.

  The client learns what is hidden only through `Sight` (T32), so it can't show what the sim hides.

### T42. Tests, hashing and splits (all slices)
- **Recommend:**
  - **`testing::syndicate`**, a build line through the gate.
  - **Invariants:**
    - held units are off the map, and nothing names them;
    - an owner's Tunnel Areas are disjoint;
    - each owned expansion lies inside exactly one own Tunnel's area;
    - Tunnel Space matches a recount;
    - eject queues name held units of the Tunnel's owner;
    - a site's builder is held at that site.
  - **Hashing.** `Sim::held`, `Object::tunnel` and `Object::construction` are destructured, so a field that isn't hashed doesn't compile. A `None` hashes a tag, so A, C and E each re-bless every golden once.
  - **Splits before adding:** `sim.rs`, `orders/gather.rs`, `placement/check.rs`, and the client's `templates.rs`.
- **Alternatives:** hash new optional fields only when they are `Some`. Old hashes would survive, at the cost of two encodings of "absent".

### T43. Work gated on numbers (not in M4)
- **Projectiles and AoE.** HeadDisjointed and DoublyDisjointed projectiles, AoE damage and Attack Ground, for the Grenadier and the Assault Bike. A combat spike when their numbers arrive (N1); Opus. Its AoE must reach hidden DrillUnits underground (0218, 0219), which `systems/combat.md > AoEDamage` still excludes (Q0071 item 3).
- **Vehicles.** FixedTurnRadiusMovement and turret arcs, for the Assault Bike, the Laser Battery and the Subterranean Transport: the vehicles milestone. Fable, and the user's call, as with M3-G.
- **The vehicle deploy order** (T44), with the first vehicle that deploys.
- **The Laser Battery's beam,** with surges whose damage follows charge^(2/3). Its Deploy and Undeploy are T44's order.
- **The Subterranean Transport**: the DrillUnit's underground movement and its surface check; loading and unloading; passengers in the held store (T30); hiding underground (T32). Fable for the movement.
- **Aircraft** (N6): air movement and collision for the Phantom, and the Tier 3 air doors. This is plan.md's "Later: air".
- **Smoke that blocks sight** (the Shadebringer): a change to the vision model.
- **Detection**: the Null Chamber, the pop-up Laser Turret seen while underground, and every DrillUnit underground (0219). The GDO's mines wait for it too. It adds a "revealed" term to T32's predicate.

### T44. The vehicle deploy order (0211, 0213, 0214, 0220; not in M4)
One mechanism serves four pairs of commands, on three units:
- the Laser Battery's Deploy and Undeploy (Syndicate, N3);
- the GDO's Artillery Rig's Deploy and Undeploy (not built yet);
- the Subterranean Transport's Burrow and Surface (N3). 0220: they "follow the vehicle deploy model".

Build it with the first of them; the others add a def row.

- **The rule.**
  - It is a queued order, not a mode. The design: "Deploy is an order, not a mode switch", and "the deploying process is what actually changes the unit: it stays the mobile type until the process completes".
  - Once started it can't be cancelled (0213), and meanwhile the unit neither moves nor attacks.
  - At completion the kind changes, and the held orders are checked against the new kind (0214).
- **Not the existing machinery.** Each is the wrong fit:
  - **The Modes step** (`sim/modes.rs`, `riot_shield.rs`, `conversion.rs`) runs abilities. "A mode isn't an order" (`ARCHITECTURE.md > Adding a new system`), and M3-T's toggle (0198) switches the type at the press.
  - **`transform.rs`** is cancellable with a refund, and there "the object is the target kind from the first frame". That is the opposite of 0211 and 0213.
  - **The Drone's `OrderKind::Deploy { origin }`** (`deploy.rs`, `sim/deploy.rs`) "becomes a Beacon immediately" and can be cancelled. 0214 calls it "a different model".
- **Recommend:**
  - **Def.** `UnitDef::vehicle_deploy: Option<VehicleDeploy { into: ObjectKind, frames: u32 }>`, one row on each kind of a pair. The values are TBD for all three units.
  - **The order.**
    - `OrderKind::VehicleDeploy`, with no payload: `into` comes from the def. It is a working name, kept distinct from the Drone's `Deploy`.
    - The panel button depends on the kind: Deploy, Undeploy, Burrow or Surface. The other kind of the pair carries the reverse button, so "Undeploy is only available once deployment is complete" follows from the kind, with no flag.
  - **Where.**
    - The rules go in `crates/sim/src/vehicle_deploy.rs`, beside `conversion.rs`.
    - The behaviour goes in `orders/vehicle_deploy.rs`, beside `orders/deploy.rs`. It holds the unit still, with no auto-targeting.
    - Progress and completion run in a Sim step right after `step_modes`, before 5a, as a Weapon Conversion's do in `step_modes`. At the last frame the step calls `Sim::change_kind`, ends the current order and re-checks the queue. The unit then acts as its new kind from the first phase of the tick, its held order first.
    - The progress is the order's behaviour state, hashed with the orders. So there is no new `Object` field, and no golden changes until a kind uses it.
  - **Held orders (0214).**
    - `Orders::give` keeps a started vehicle deploy as `current`. Without Shift, a new order replaces only the queue ("the last order given ... is held"); with Shift, it appends. Stop is held like any other order (0213: "Stop doesn't interrupt it").
    - The gate skips its per-unit capability filters for a deploying unit, since orders are "stored, not executed or rejected"; its target checks still apply. The other direction needs this as well: a Move given while a unit undeploys is valid only for the kind it becomes.
    - At completion, each queued order is checked against the new kind. The per-unit filters in `sim/gate/orders.rs` (those like `riot_shield::allows_attack`) become one `can_take(object, order)`, which the gate and this check share.
    - A dropped order emits a new `SimEvent`. The client turns it into the unavailable-command feedback (0162, `command_ui/feedback.rs`).
  - **Client.** The existing `KindChanged` event moves the unit to its new SelectionGroup at completion. The ActiveGroup rule for a group that disappears (0199) applies unchanged.
  - **The Transport's extras**, behind the same order:
    - Surface checks the footprint when the order starts: tiles a tracked vehicle can cross, with no building (0217), and no above-ground unit (`systems/units.md > Underground Collision > Surfacing`);
    - it reserves the footprint until surfacing completes (`> Reserved while surfacing`);
    - a failed check drops the order: "It stays underground until the player surfaces it somewhere clear".
- **Alternatives:**
  - extend `transform.rs` with a flag for "no cancel, switch at the end". This would bend a mechanism whose two users, Warp In and Beacon Deployment, both switch at the start;
  - a per-unit ability in the Modes step. Abilities aren't queued, and the design makes Deploy queueable.

---

## 5. Design questions

Each question is its own file, in the Director's format, in `scratchpad/m4/questions/`. Each has a `# QXXXX — Title` heading and the Date left as `DATE`, for the Director to number and date when filing. Cross-references between them are written Dn (for example "Q0065, point 5"). Replace them with the Q numbers when filing.

**Blocking status**
- **Q0059 is blocking for K.** Until the Optics Lab has stats, the Guard can't be produced, so the Syndicate fights with Agents only.
- **Q0060 is blocking for G**, for the upgrade's duration.
- **Q0070 blocks only L's pass or fail.** L builds and measures anyway.
- **The others are not blocking.** The slices build the proposals as the current behaviour, and cite the question.
- **Q0071 asks which deferred items will get numbers or designs in time for M4.** It lists what each one needs, with the rule gaps found in 0207–0222. It blocks nothing in M4; each gap blocks only its own deferred item.
- Answers that arrive before a slice launches go into its brief (insights: the Designer answers within the hour).

| Q | File | Title | Blocking | Slices |
|---|---|---|---|---|
| Q0059 | `Q0059-optics-lab-stats.md` | Optics Lab: placeholder stats | yes | K |
| Q0060 | `Q0060-tunnel-upgrade.md` | Tunnel upgrade: duration, HP and payment | yes | G |
| Q0061 | `Q0061-agent-built-tunnels.md` | Agent-built Tunnels: payment, ownership and the site | no | E |
| Q0062 | `Q0062-tunnel-areas-and-layers.md` | Tunnel Areas and the two layers | no | E, F |
| Q0063 | `Q0063-tunnel-network.md` | The Tunnel Network: transit, entering single file, and units inside | no | C |
| Q0064 | `Q0064-ejection.md` | Ejection and production out of the network | no | C |
| Q0065 | `Q0065-expansions-under-construction.md` | Expansions under construction | no | F |
| Q0066 | `Q0066-agent-gathering.md` | Agent gathering: the Drone's rules, the Tunnel it uses, and the line at Side B | no | D, I |
| Q0067 | `Q0067-supplies-gathering.md` | Supplies gathering | no | I |
| Q0068 | `Q0068-panel-letters.md` | Panel letters outside the 4x3 grid | no | J1, J2 |
| Q0069 | `Q0069-losing-tunnels.md` | Losing Tunnels: the open cases | no | H |
| Q0070 | `Q0070-syndicate-economy-check.md` | The Syndicate economy check | only L's pass or fail | L |
| Q0071 | `Q0071-deferred-content-in-m4.md` | Deferred Syndicate content: which items will get their numbers or design in time for M4? | no | N1–N6 |

**Q0059–Q0070 re-checked against 0207–0228.** None is answered by them, but several change:
- **Q0059** gains context. The Optics Lab also gates the Laser Battery (0210) and the Phantom (0226). The Forge, which Q0059 lists as not needed in M4, now gates the Transport (0221).
- **Q0062** stands. Whether an underground unit blocks expansions or passes through them is Q0071 item 6. M4 has no underground unit.
- **Q0063 and Q0064** change for 0225: units enter and exit single file, each path at its fixed spot. New points ask:
  - how long a unit holds each path (Q0063 point 7, Q0064 point 10);
  - whether the Tier 3 air doors take one aircraft at a time (Q0064 point 11).

  0221 and 0222 concern the Transport's passengers, which are deferred with it (Q0071 item 6).
- **Q0066 and Q0067** change for 0223:
  - an Agent keeps its Tunnel for a whole Gather (Q0066 point 2);
  - one that can't reach Side B gives up by the Failure rule (Q0066 point 3);
  - the options that sent Agents to another side, Tunnel or field by themselves are gone.

  A new point asks whether a hatch is the whole side (Q0066 point 6, 0224).
- **The others** stand.

---

## 6. Risks

| # | Risk | Mitigation |
|---|---|---|
| 1 | **Units off the map (C).** Systems that must count held units can miss them: Tunnel Space, stranded units, the hash, replays, the client's control groups. | T30: one store, so every map system skips held units by construction. Invariants and the hash cover the store. Tests for each kind of reference: attack, follow, rally, gather, selection. Fable. |
| 2 | **Crowds at Side A.** Each path takes one unit at a time (0225), so an army queues to go in, and units waiting their turn may hit the Failure rule (0223) while the line still moves. Also: ejecting into a crowd, a blocked exit spot, units appearing under enemy fire. | Fixed spots and fixed holding times, with no crowd logic (0223; Q0063, Q0064). C's test of 40 units each way counts those that give up. Many would mean a question to the Designer, not cleverer code. `movement_perf` A/B. Fable. |
| 3 | **The shared Gather job (D).** Generalising it for the Agent changes the Drone. | `orders/gather.rs` is split first. The Colonists goldens and oracle must not move. Fable. |
| 4 | **The line at Side B under hard collision.** Waiting Agents block the one whose turn it is, or those walking off; jams with stacked Tunnels. | An Agent drops off where it stands, within reach of the whole side. One that can't reach gives up visibly (Q0066 point 3, 0223). A soak of six Agents on two stacked Tunnels counts those that give up. If jams remain, a question to the Designer like Q0038 (pass-through), not a collision exemption or crowd logic added on the code side. |
| 5 | **Hidden information leaks.** Placement refusals, ghost colours, paths bending round, records, pointers, or later AoE, could reveal hidden expansions. Later, area damage still hits a hidden DrillUnit (0218, 0219). The unit must take that damage without the hit, its death or a refused surface revealing it. | One predicate over objects (T32); layers private per player (Q0062); no nav blocker and no records. Tests from the enemy's view. |
| 6 | **Oracle fidelity.** The table ignores silhouettes, reaches, turning and the line at Side B. | Q0070 settles the reference geometry. The measured table goes to the Designer (0108's rule); no game number is changed to fit. |
| 7 | **Design churn.** Syndicate entries keep arriving: 0207–0209 came while this spike ran, and 0210–0228 straight after (the deploy order, the Subterranean Transport, hidden DrillUnits, Side A's single-file paths, the Phantom). | (a) Numbers are data in `defs/syndicate.rs`: a change of one line, plus a re-bless. (b) New units are separate, gated slices (N1–N6). (c) The generic pieces (held store, prerequisites, tiers, `Construction`, `Method::Expansion`) take new expansions and units as data. (d) The Director triages each entry as data, a new slice, or later. (e) Answers go into briefs that haven't launched, and through SendMessage to running slices. (f) Defaults are named placeholders that cite their question. (g) Shared machinery for deferred content is shaped now but not built: T30 (a carrier place), T32 (a predicate over objects), T33 (one function for Used) and T44 (the deploy order). 0210–0222 changed no scheduled slice's scope. 0225 changed C's entry and exit rules, and 0223 simplified three defaults. |
| 8 | **Golden churn.** A, C, E, F and H each add or change hashed state. | Re-bless once per slice, after merging master, with explanations. The Syndicate goldens come only with L. |
| 9 | **Mandatory layouts.** Implementing the Syndicate's Starting Conditions makes its layouts mandatory on Skirmish Default maps. | B updates `dev_skirmish` in the same slice; the editor flags any other map. Only `dev_skirmish.json` exists today. |
| 10 | **Old ids back on the map.** Ejected units break assumptions built on `first_new`. | T35's audit. A test ejects a unit in the tick a station delivers, and in the tick a unit is produced. |
| 11 | **Defeat edge cases.** A site counting as a structure; expansions turned neutral in the same tick as the last Tunnel's death; stranded units. | H's tests. The ignored suite after H, since it changes a game rule (insights). |
| 12 | **Machine load.** Two Fable slices and one Opus slice in W2. | At most three builds at once. Tests that don't depend on machine speed. Perf judged by A/B runs. |
| 13 | **Files over 500 lines**: `sim.rs` 596, `orders/gather.rs` 490, `placement/check.rs` 474. | Split first (T42), in the slice that adds to the file. |

---

## 7. QA

- **The Syndicate oracle (L).** The design has Syndicate economy targets, so M4 gets an oracle, in `tests/economy_oracle/syndicate.rs`, with its table in `docs/economy_oracle.md`. It checks, on Q0070's reference layout:
  1. **The opening.** The second Agent starts at 0:00, and the Tunnel is saturated by 3 Agents at about 12.5 SC per second around 0:40.
  2. **The Agent table.** For each distance from 0 to 6 GU: the trip time, SC per second per Agent, and the income with 2, 3 and 4 Agents, each within ±15% (Q0070).
  3. **The ceiling.** Exactly 12.5 SC per second per Tunnel, however many Agents.
  4. **Stacked Tunnels.** Two Tier 1 Tunnels on one field double its intake.
  5. **The ordering.** The Syndicate's one-base saturation income is the lowest of the three factions' measured figures.

  Run it with `cargo test -p sc_sim --test economy_oracle -- --nocapture`, and the long variant with `--release ... --ignored`.
- **Goldens (L).** Through the `Script` hook and `testing::syndicate::skirmish`:
  - `syndicate_opening`: one player scripted. It trains Agents, rallies them, gathers, gathers Supplies, builds a second Tunnel and expands;
  - `syndicate_vs_gdo`: both players scripted. Agents and Guards against Peacekeepers, a Tunnel lost, neutral expansions, stranded units, a defeat.

  Every slice after L re-blesses them once, explained.
- **Soak (M).** `syndicate_soak`, ignored: 10 minutes of the golden game, with the invariants (the network's included), the books, no stuck order, and the hash stable over two runs. Like `skirmish_soak`.
- **Perf (M).** `agents_perf`, ignored: 4 Tunnels with 4 Agents each on shared fields, and 60 Guards cycling through Enter and Eject at one Tunnel. Judged by interleaved A/B runs against master; absolute numbers only on a quiet machine. D also runs an ignored Agent gathering soak.
- **Scripted walkthrough (M).** `cargo run -- --factions syndicate,gdo --input-script <file> --screenshot ...`, with screenshots at fixed ticks:
  1. the start;
  2. an ejected Agent;
  3. gathering, and the line at Side B;
  4. the Tunnel ghost and its area overlay;
  5. a site at 10%;
  6. the second Tunnel;
  7. the Expand ghost;
  8. a Hideout under construction;
  9. the EjectMenu with greyed tiles;
  10. neutral expansions after a Tunnel dies;
  11. the Guard unlocked by the Optics Lab;
  12. a Tier 2 Tunnel.

  The earlier checkpoint (end of W2) uses `--commands` only.
- **`docs/playtest/m4.md` (M)**, in `m3.md`'s sections:
  - Starting;
  - Controls (Enter, Eject, the rally into and out of the network, Build Tunnel, Expand, Upgrade, Supplies);
  - Things to try;
  - Known placeholders: the Optics Lab's stats, the upgrade's duration, the decay rate, how long a unit holds each Side A path, the entry reach;
  - Known issues;
  - Design questions.
- **Ignored tests.** `cargo test --release --workspace -- --ignored --test-threads=1`, on a quiet machine, after H (it changes defeat) and at the end of M4 (insights: "Ignored tests rot silently").
- **New invariants** (T42), checked by every soak and golden run: held units, disjoint areas, expansions in exactly one area, a Tunnel Space recount, eject queues, and site builders.

---

## 8. Director to-dos implied by this plan

- **Questions.** File Q0059–Q0071 from `scratchpad/m4/questions/`, numbered from Q0059 (Q0058 is taken). Replace the Dn cross-references with the new numbers. File Q0059 and Q0060 first: they block K and G.
- **`plan.md`.**
  - Replace the M4 entry with the slices M4-A to M4-M, and the gated N1–N6.
  - Update the change rows:
    - 0186: the Optics Lab gate goes to M4-K, the Hideout and Agent to A–E;
    - 0187, 0188, 0189–0196 and 0207–0222: gated, numbers TBD. 0210–0222 are already queued for N3. Point the deploy-order row (0211, 0213, 0214) at T44;
    - 0224 and 0225: A (the look) and C (the paths), with the air doors in N6; 0226–0228: N6;
    - 0223: the briefs for C, D and I ask for no crowd or flow logic (insights);
    - 0207's red lasers: M4-A.
- **Decide** T29–T44.
- **Before W1.**
  - M3 done: M3-T, L3 and G2 are running; K3 and J3 are queued.
  - The user's go-ahead.
  - Check `maps/` for other Skirmish Default maps; only `dev_skirmish.json` exists today.
- **Briefs.**
  - Quote the design text in 3.3 word for word.
  - Give each slice the ownership table in 3.2 as its "don't touch" list.
  - Run C and D on Fable. Tell them that `orders/resolution.rs` and `sim/gate.rs` are shared in W2.
  - Tell D that the Colonists goldens and oracle must not change.
- **Folding answers in.** The Designer answers within the hour, so fold each answer into the briefs that haven't launched. Rewrite a placeholder only when its slice is open.

## 9. Design answers since the plan was adopted
Quote these in the slice briefs.
- **0229** (Q0059, and Q0060 points 1–2):
  - The Optics Lab's placeholders are as proposed: 2x2, 150 SC with no Supplies, 400 frames, HP 400, armour 1/4, Groupable, and no commands. **This unblocks K.**
  - A Tunnel upgrade takes 480 frames per tier, with the Supplies paid when it is ordered. **This unblocks G.**
- **0231 and 0232** (Q0060 points 3–6, for G):
  - An upgrade in progress counts toward the next upgrade's cost.
  - Cancel Upgrade refunds the current top price, not counting itself (0232 corrects 0231's "exactly what was paid").
  - On completion, current HP rises with max HP.
  - Supplies are lost if the Tunnel is destroyed mid-upgrade.
  - The larger area is reserved from the start of the upgrade.
- **0233** (Q0061 points 1–2, for E):
  - A Tunnel build order pays its Supplies when the ghost is placed.
  - Lingering ghosts and sites count as owned Tunnels for the price.
  - An order that can't be paid for is refused, with feedback.
  - Cancelled, replaced or failed orders, and cancelled constructions, refund the current top price.
- **0234** (Q0061 points 3–5 and 7–9, for E): accepted as proposed.
  - An unfinished Tunnel offers only Cancel.
  - The ConstructionHP formula gives the maximum, and damage stays.
  - The Agent comes out at the footprint's centre and idles.
  - The site counts for defeat.
  - The Agent keeps its load.
  - Enemies see the site, not the Agent.
- **0237** (Q0062 point 1, for E and G): the non-overlap rule applies between all players' Tunnels, which replaces the own-Tunnels-only proposal. Crowding or blocking an enemy's Tunnels is a valid tactic.
  - The build ghost judges known enemy Tunnels at their remembered position and last-seen tier.
  - The check on arrival uses the truth, so it can fail on an upgrade the player didn't see.
  - Upgrade checks use the truth, and any Tunnel's area blocks them.
- **0238** (Q0062 point 2, for B and F): Tunnel Space counts the whole area square, cells past the map edge included. Expansions can be placed on map cells only.
- **0239** (Q0062 points 3a, 3b and 3e, for F): every tile under an expansion must be Drillable. Plane, Rugged and Cliff are; Mountain and Water aren't. Own and neutral underground structures, and the Tunnel's own footprint, block it, as proposed.
- **0240** (Q0062 points 3c and 4, for E and F): surface buildings and underground expansions never share a tile, whatever the owner, neutral included. Units still walk over expansions. This replaces T32's "the surface methods ignore them".
  - **E: surface placement reads the underground truth.**
    - A direct placement (GDO) over a hidden expansion is red at once. The leak is accepted.
    - A worker-built ghost (the Deploy pattern, Agent-built Tunnels) judges by what the player knows, so it stays green, and the order fails on the worker's arrival.
  - **F:** no expansion under a surface building.
