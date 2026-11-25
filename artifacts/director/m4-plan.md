# M4 plan: the Syndicate (planning spike M4-P), revised for the design answers 0229–0282

> **Status (2026-11-25): revision 2, for the Director.** It replaces the adopted plan.
> - **The answers are folded in.** Every M4 question (Q0059–Q0071) is answered by 0229–0271, and they are folded in.
>   0268–0281 (the Specter, the satellite "Prism", Relay) are gated content, and are noted only.
> - **0282 moved the design docs** to one file per unit and per structure, with no design change. This plan cites the
>   new paths. Through 0282 the user asked for the code to be "organized and modular" too, and then decided the
>   shape: "Modules now, data files at playtest". That is the new **M4-R**, in two parallel slices, **R1** (the sim)
>   and **R2** (the client, docs and a citation test), in a wave of its own after B, C and I land, before E and J2.
> - **Status of the slices.**
>   - **M4-A** is merged (`fcd06dc`), and so is **M4-D** (`edc30e11`).
>   - **M4-B** is running on this revision's B spec (`scratchpad/m4/rev/B.md`, which §3.3 repeats).
>   - **M4-C** and **M4-I** are running on their briefs (`scratchpad/briefs/m4c-tunnel-network.md`,
>     `m4i-supplies.md`), which fold in 0245–0253, 0248 and 0256. C and I are **not revised** here. Their §3.3 text is
>     the adopted plan's, and their briefs override it.
>   - **Q0076 is open** (Agents jam head-on near the hatch at 0–1 GU). It partly blocks L, and the first playable's
>     layout.
>   - **M3-L5** is merged (`a777ae3`), so nothing in M4 waits on M3 any more.
> - **0260 brought four new M4 items**, planned as new slices:
>   - **O**, the combat spike: projectiles, area damage and Attack Ground;
>   - **U**, the Grenadier (named U, since R is the code organization's);
>   - **S1**, the Laser Turret, upright and Agent-built;
>   - **S2**, the pop-up turret, and the turret built by a Tunnel.
>
>   The Chemical Plant goes into **K**. The gated list keeps N3–N6 and adds N7, the Specter.
> - **Marks.**
>   - **Changed (NNNN)** marks each place where a design answer changed the adopted plan.
>   - **Placeholder (QD-slug)** marks a rule a slice builds as a proposal until the question draft
>     `scratchpad/m4/rev/questions/QD-slug.md` is answered.
>   - Technical decisions T29–T44 are updated where answers change them. T45–T54 are new; T54 is the code
>     organization.
> - **Question drafts.** Eight, all non-blocking (§5), in the Director's format, for the Director to number from Q0077
>   (Q0076, Agents jamming near the hatch, is the last filed).
> - **Hashing.** m4-common's rule ("hash a new field only while it is set") overrides the adopted plan's "re-bless every
>   golden once" for A, C and E. A merged with the goldens unchanged.

- **Date**: 2026-11-25. A read-only revision: no repository file was changed, and cargo was not run.
- **Code read at**: `172d496` (master; no code change since `6460e2a`), which includes M4-A (`fcd06dc`) and M3-L5
  (`a777ae3`). Re-checked at `d399c44`, after M4-D's merge. Build times come from `target/` timestamps only; nothing
  was built.
- **Design read at**: `70d1bbc` (0282). The design diff `0a661e5..2e5124e` (0277–0281) touches only the gated Relay
  and Prism content, and 0282 only moves text.
- **Sources** (beyond the adopted plan's, which still apply):
  - **Change entries** 0229–0281, and the doc sections each lists:
    - `factions/syndicate/tunnels.md` (all of it);
    - `factions/syndicate/structures/`: `README.md` (Tunnel Expansions, High armor underground, Expansion
      Requirements, Rally Point Behavior), `hideout.md`, `optics_lab.md`, `chemical_plant.md`, `laser_turret.md`;
    - `factions/syndicate/units/`: `README.md` (red lasers), `agent.md`, `grenadier.md`, `guard.md`;
    - `systems/combat.md` (Range Measurement, AttackTarget, ValidTarget, AttackType, AoEDamage,
      DamageModifierOrder);
    - `core/entities.md` (Building armor rule, Placement Validation);
    - `core/map.md` (Tile, DefaultTilePresets);
    - `core/vision.md` (Attacks from outside vision);
    - `core/map_editor.md > Starting Layouts`;
    - `systems/controls/commands.md` (AttackGround);
    - `systems/controls/behaviors.md` (Core principle, BaseAutoTargeting, AttackingObject, AttackingLocation);
    - `systems/controls/selection_and_hud.md` (BasicCombatUnitInterfaceState, PointerDisplayType,
      CommandIndicators).
  - **`artifacts/director/plan.md`:** the M4-A entry, and the status rows 0229–0281.
  - **The running briefs:** `m4-common.md`, `m4c-tunnel-network.md` and `m4d-agent-gathering.md`.
  - **Code:**
    - sim: `combat.rs` and `combat/` (phases, damage, targeting), `defs/units.rs` (`AttackAttrs`, `TargetType`),
      `defs/syndicate.rs`, `tunnel.rs`, `vision/hidden.rs`, `placement/`, `scenario/validate*`,
      `scenario/starting.rs`, `economy/army.rs`, `orders.rs` (`OrderKind`), `command.rs`, `map.rs` (presets),
      `maps/dev_skirmish.json`;
    - client: `combat_effects.rs` (`ShotStyle`, `visible_spans`), `object_render/tunnel.rs`,
      `command_ui/` (AttackGround's slot, pointer and indicator), `hud/resources.rs`, `cli/values.rs`,
      `editor/layouts/`.
- **"Silent"** means the design says nothing about the point. The plan then proposes a default and drafts a question
  (§5).

---

## 0. Summary

- **Scope.** M4 builds the Syndicate as far as the design gives its numbers:
  - **The Agent:** gathering crystals and Supplies, at 1-square hatches (B crystals, D Supplies), from the nearest
    Tunnel; its pistol; building Tunnels, and the Laser Turret.
  - **The Tunnel:**
    - tiers 1–3;
    - Tunnel Areas, which must be wholly on the map and Drillable, and may not overlap any player's;
    - Tunnel Space, cost scaling in Supplies paid at the ghost with top-price refunds, and upgrades.
  - **The Tunnel Network:** walking in single file, the transit delay, regeneration, Eject, and production into the
    network. These are C's, per its brief.
  - **Underground expansions:**
    - hidden from enemies and walkable;
    - Drillable tiles, never sharing a tile with a surface building;
    - ConstructionHP, a 75% refund on cancel, and one operation per Tunnel.
  - **The buildings:** the Hideout, and the **Optics Lab** and **Chemical Plant** on placeholders. They give the Guard
    and the Grenadier their gates.
  - **Tech and unit prerequisites**, generic for every faction.
  - **Losing Tunnels:** decay at 1% per second, explicit attacks only, reclaiming at completion, stranded units, and
    the defeated player's network units destroyed.
  - **New (0260):**
    - **the combat spike:** DoublyDisjointed projectiles at 8 su per frame; area damage by overlap, which reaches
      underground objects; Attack Ground;
    - **the Grenadier**, at the Hideout;
    - **the Laser Turret**, upright, or a hidden pop-up inside an own Tunnel Area, built by an Agent or by a Tunnel.
  - **Also:** the Starting Conditions, layouts, the HUD and panels with 0257's letters, red lasers, and an economy
    oracle (0259).

  - **Code organization (0282):** one module per unit and per structure, defs as plain data ready for serde,
    tunables in their own modules, and shared mechanics built once (R1, R2).

  **Deferred, because they are TBD in the design or wait on other milestones:**
  - the Assault Bike, Shadebringer, Laser Battery, Subterranean Transport, Phantom and Specter, and the satellite
    (Prism) with Relay;
  - aircraft and the Tier 3 air doors;
  - the vehicle deploy order (T44);
  - the Vault, Vehicle Assembly, Forge, Null Chamber, Aerospace Assembly and Orbital Command;
  - research, **detection** (0267: "after all faction units are done") and notifications;
  - **data files and hot reload** for the numbers: playtest prep, after all four factions (the user's decision).
- **Slices.** 20 (A–M, R1, R2, O, U, S1, S2), in 9 waves. C and D are on Fable; the rest on Opus.
  1. **A** (merged)
  2. **B** (running) ∥ **C** (Fable, running) ∥ **D** (Fable, merged), then **I** (running since D landed)
  3. **R1** (code organization: the sim) ∥ **R2** (the client, docs and the citation test), after B, C and I
  4. **E** ∥ **J2**
  5. **J1** ∥ **H** ∥ **F**: gives the first playable
  6. **K** ∥ **G** ∥ **O** (the combat spike)
  7. **L** ∥ **S1** (Laser Turret)
  8. **U** (Grenadier) ∥ **S2** (pop-up turret)
  9. **M** (QA)

  No two slices of a wave share a file outside the shared registries (§3.2). U and S1 both add a kind, which is why
  they don't run together.
- **First playable.** After A, B, C, D, R1, R2, E, H, J1 and J2, at the end of wave 5: M4-R moves it one wave later.
  Its layout waits on Q0076 (with the patch 1 GU from the hatch, three Agents jam).
  Guards come with K (W6), Grenadiers with U (W8), and turrets with S1 and S2 (W7, W8).
- **Key decisions.** T29–T44 as adopted, with these changes:
  - **T32 (Changed, 0240).** "The surface methods ignore them" is replaced: surface and underground placements never
    share a tile, judged by the truth, except that worker-built ghosts judge by knowledge.
  - **T33 (Changed, 0242, 0258, 0265).** Tunnel Space has no off-map or non-Drillable case. Neutral expansions count
    as occupied, and every unit counts for life.
  - **T42 (Changed, m4-common).** New state is hashed only while set.
  - **T44 (Changed, 0282).** The deploy model is built once, for every deploying kind, on the shared progress type
    and kind change.
  - **New:** T45 projectiles; T46 area damage, built once for every AoE attack; T47 Attack Ground; T48 the
    DoublyDisjointed sequence, its events and reveals; T49 structures that attack; T50 the pop-up form; T51 the
    kind-generic Build order; T52 drawing projectiles and blasts; T53 the placement problem sets; **T54 the code
    organization** (per-object modules, serde-ready defs, tuning modules, shared mechanics, the citation test).
- **Readiness: ready.**
  - Nothing is blocking. Every question Q0059–Q0071 is answered.
  - Eight new question drafts cover the gaps and contradictions the answers left (§5). Each slice builds the
    proposal, marked as a placeholder.
  - The least settled slice is S2, the pop-up turret: several of its rules are silent (Q0084).

---

## 1. Scope

### 1.1 In M4

| Area | Content | Design |
|---|---|---|
| Agent | Unit, HeavyInfantry, 36x36, HP 75, armour 1/1, sight 5, TunnelSpaceCost 1, Groupable false, 6 su per frame, turn 180° per frame. Pistol: FullyConnected, Ground, 5 damage, range 2, Aim 3, Fire 1, Cooldown 2, Reload 14. (Built in A.) | `factions/syndicate/units/agent.md` |
| Agent gathering | **Built in D (`edc30e11`).** **Changed (0255, 0248).** The Drone's job rules, except that any number of Agents share a patch and there is no Set Drop-off. The nearest own operational Tunnel, chosen at the Gather's first return and kept. A 1-square hatch near the middle of Side B (crystals) and of Side D (Supplies). Agents wait at the hatch for ever. Crystals: carry 50, mining 80 frames, drop-off 64 frames, reaches 8 su. Supplies (**Changed, 0256**): carry 1, pick-up 48 frames, drop-off 48 frames, at Side D; shared stations; waiting at an empty station; switching resources loses the load (I, running). **Open: Q0076**, Agents jam head-on at 0–1 GU. | `units/agent.md > Gathering` (rules, Space Crystals, Supplies); `tunnels.md > Tunnel > SymmetryType - ABCD` |
| Tunnel | Structure, 4x4, ABCD: A entrance and exit, B crystal hatch, C back wall, D Supplies hatch (**Changed, 0248**). Groupable false, sight 5. Tiers: HP 600/800/1000, armour 1/16, Area radius 2/3/4. Enemies see the tier (0258). (Built in A.) | `tunnels.md > Tunnel` |
| Tunnel Network | C's brief: walking in single file at normal speed (0246), the normal Failure rule (0247), a 32-frame transit delay (0245), orders cleared and loads kept inside, 1% MaxHP per second regeneration (0249), walking out single file on a clear exit spot, the longest-inside unit first, a reserved eject queue (0250, 0251), rallies (0252, 0253). No air doors in M4. | `tunnels.md > Tunnel Network`, `> EjectMenu`; `structures/README.md > Rally Point Behavior` |
| Tunnel Area and Tunnel Space | **Changed (0237, 0242, 0258).** Areas of 8x8, 10x10 and 12x12. They may not overlap **any player's** at the current tier, which also blocks an upgrade. A Tunnel's whole area must be on the map and Drillable, for placement and upgrades. Tunnel Space = ⌊unused region / 4⌋, at most 200, with no off-map or non-Drillable case; neutral expansions count as occupied. | `tunnels.md > Tunnel Area Non-Overlap Rule` (Drillable area, Every player's Tunnels count), `> Tunnel Space` |
| Agent-built Tunnels | **Changed (0233, 0234).** Worker-Built placement and a lingering ghost. **Paid when the ghost is placed**, and ghosts and sites count toward the price. Refunds at the current top price. The site does nothing but Cancel, starts at 10% (ConstructionHP, as a maximum, damage kept), with the Agent inside and untargetable. A destroyed site loses the Supplies. One Agent per Tunnel. | `units/agent.md > Building`, `> While the Tunnel is under construction`; `tunnels.md > Cost Scaling`; `core/entities.md > Placement Validation` |
| Expansions | **Changed (0239, 0240, 0241, 0254).** Underground, hidden from enemies, walkable, in the Tunnel Area. On Drillable tiles, never under a surface building of any player, never under resources. Paid at the ghost; Tunnel Space used from construction start. ConstructionHP; X cancels for **75%**. No sight. One operation per Tunnel, which also covers a turret (0267). | `structures/README.md > Tunnel Expansions`; `tunnels.md > ExpandMenu`, `> AwaitingPlacement (Expansion)`, `> Expansions under construction` |
| Hideout | Underground, 2x2, Tier 1, 200 SC, 400 frames, HP 400, **armour 6/24** (**Changed, 0262, 0264**), sight 0, Groupable (0254). Queue of 20. Q Agent (100 SC, 240 frames), W Guard (125 SC, 120 frames, needs an Optics Lab), **E Grenadier (150 SC, 160 frames, needs a Chemical Plant; 0260)**, X Cancel, C rally. | `structures/hideout.md`, `> High armor underground` |
| Prerequisites, the Optics Lab and the Chemical Plant | **Changed (0229, 0260).** The Optics Lab and the Chemical Plant on placeholders: 2x2, 150 SC, 400 frames, HP 400, armour 6/24 by the expansion rule, Groupable, no commands. They gate the Guard, the Laser Turret and the Grenadier. Generic Tech and Unit Prerequisites. | `structures/optics_lab.md`, `> Chemical Plant`; `core/tech_and_research.md` |
| Tunnel upgrades | **Changed (0229, 0231, 0232, 0242).** 480 frames per tier, paid when ordered. Tier 2 costs 2 + 2 × (T2+ owned, upgrades in progress included), Tier 3 3 + 3 × (T3 owned). Cancel refunds the current top price. HP rises with max HP, and Supplies are lost if the Tunnel is destroyed mid-upgrade. The larger area is reserved from the start, and is blocked by any Tunnel's area and by a non-Drillable or off-map cell. | `tunnels.md > Tiers`, `> Upgrade Costs`, `> Construction and Upgrade Rules`, `> Tunnel Area Non-Overlap Rule` |
| Losing Tunnels | **Changed (0254, 0258).** Expansions turn neutral, become visible, and decay at **1% of MaxHP per second** (placeholder). Only an explicit Attack hits them. Reclaiming at the covering Tunnel's or upgrade's completion. A partly covered neutral expansion blocks its cells and counts as occupied. Work in progress is lost and queued work refunded. Unfinished expansions are destroyed with no refund. Stranded units are destroyed, and so are a defeated player's network units. Defeat by the shared rule; Laser Turrets stay owned. With no Tunnels, the next one costs 0. | `tunnels.md > Losing Tunnels`; `core/game_rules.md > Defeat Condition` |
| Combat spike (new, 0260, 0262, 0264) | DoublyDisjointed projectiles with travel time. Area damage by overlap, which hits underground objects even unseen and never asks the hidden predicate. Attack Ground for attacks that can miss. | `systems/combat.md > AttackType`, `> DoublyDisjointed`, `> AoEDamage`, `> Range Measurement`; `systems/controls/commands.md > AttackGround`; `behaviors.md > AttackingLocation` |
| Grenadier (new, 0260) | HeavyInfantry 36x36, HP 70, armour 1/1, sight 5, speed 5, TunnelSpaceCost 1. Grenade: DoublyDisjointed, Ground, AoE 100 over radius 40 su, range 4, 8 su per frame, Aim 6 / Fire 2 / Cooldown 4 / Reload 28. 150 SC and 160 frames at the Hideout (E), behind a Chemical Plant. | `units/grenadier.md` |
| Laser Turret (new, 0260, 0264, 0267) | 1x1, 100 SC, 320 frames, HP 300, sight 6, behind an Optics Lab. Red laser: Universal, 24 damage, range 5, Aim 4 / Fire 1 / Cooldown 3 / Reload 16. Upright outside own Tunnel Areas (1/1, always visible, no Tunnel Space). A hidden pop-up inside one (6/6 underground, 1 region cell): it rises in 8 frames and retracts after 32 frames with no target, taking 8. Built by an Agent (D), or by a Tunnel with its operation slot. | `structures/laser_turret.md` |
| Start | A Tier 1 Tunnel with a Hideout, one Agent, 100 SC, 1 Supply; the camera on the Tunnel | `tunnels.md > Tunnel > Starting Conditions`; `core/camera.md > Starting Position` |
| Layouts | Syndicate Starting Layouts on `dev_skirmish`, validated by 0242 (**Changed**) | `core/map_editor.md > Starting Layouts`; `tunnels.md > Tunnel Area Non-Overlap Rule > Drillable area` |
| Building armour | **New (0264).** PointArmor n, FullArmor n × size in grid squares, with listed exceptions, for every faction's buildings: a defs test | `core/entities.md > Structure Type > Building armor rule` |
| HUD and panels | SC, Supplies, and Tunnel Space used / available. **Changed (0257):** the Agent's Q Move, A Build Tunnel, S Drop Off, D Build Laser Turret, X Stop; the Tunnel's Q Upgrade, W Expand, E Eject, X Cancel; menus in grid order with Z back. The Enter, GatherResources, ReturnResources and **AttackGround** pointers; the Tunnel Area overlay. | `overview.md > DisplayHud`; `units/agent.md > Agent ObjectInterfaceState`; `tunnels.md > Tunnel ObjectInterfaceState`; `selection_and_hud.md` |
| Lasers | Syndicate lasers are red: the Guard's and the Laser Turret's | `units/README.md` header (0207) |
| QA | **Changed (0259).** A Syndicate oracle measured edge to edge to the hatch square: ±15%, and exactly 12.5 SC per second per Tunnel. Goldens, perf and soaks. | `units/agent.md > Gathering > Space Crystals` (Syndicate economy check) |

### 1.2 Deferred

| Item | Why |
|---|---|
| Assault Bike (0188) | Numbers TBD, and the Vehicle Assembly is TBD. The WheeledVehicle's FixedTurnRadiusMovement and its turret arc belong with the vehicles milestone. Its mortar will reuse O's projectiles and area damage. |
| Shadebringer (0189–0196) | Concept only. Smoke that blocks sight is new to the vision model. The Vault is TBD. |
| Laser Battery (0207–0214), with Relay through Prisms (0276–0281) | Concept only: the beam with surges, the deploy order (T44), and Relay (concept, 0277–0280). It needs an Optics Lab (K). |
| Subterranean Transport (0215–0222, 0263, 0265, 0266) | Concept only; times and capacity TBD. DrillUnit base with no attack (0263). Passengers count for Tunnel Space, aboard or not (0265). It passes through underground structures and may surface above expansions (0266). Hidden underground until detection, which comes "after all faction units are done" (0267), so only area damage can hurt it meanwhile. Gated N3. |
| Phantom (0226–0228, 0263) | Concept only. A Groupable HoverCraft with a FullyConnected SingleTarget laser (0263). Aircraft, and the Tier 3 air doors, wait for air units. Gated N6. |
| Specter (0268–0273) | **New.** Concept only: a single-payload Glider bomber from the Aerospace Assembly, behind a Vault and a Chemical Plant. Ground only; DoublyDisjointed, assumed, so it can Attack Ground. It lands and re-arms in Tier 3 Tunnels, flies home when unarmed, and has ammo-dependent order rules. All numbers are TBD. Gated N7. |
| Prism, the satellite (0274, 0275, 0280, 0281) | Concept only: an Air HoverCraft from the Aerospace Assembly, behind an Orbital Command. Launched one-way through an air exit. Relay only. |
| Vehicle deploy order (0211, 0213, 0214, 0263) | Shared by the Laser Battery, the Transport and the GDO's Artillery Rig (T44). 0263 confirms that undeploy always completes, and that held orders the new kind can't carry out are dropped with feedback. |
| Vault, Vehicle Assembly, Forge, Null Chamber, Aerospace Assembly, Orbital Command | Each is "(Full stats TBD)". The Vault works network-wide (0269). |
| Research, and Supplies spent on it | No Syndicate research is designed |
| Detection | **Changed (0267).** "Detection will be designed after all faction units are done. Until then hidden things (expansions, pop-up turrets, burrowed DrillUnits, mines) can only be hurt by area damage over them." In M4 an owned expansion or a lowered pop-up turret is never visible to an enemy. |
| Notifications | "notification system not yet designed": for a blocked queue (`tunnels.md > Production and Tunnel Space`) and an empty station (0256). M4 shows placeholders (I, J2). |
| Transit at Tier 2 in play | M4 has no Syndicate vehicle. The tier rule is built and tested with test units (C). |
| Aircraft and the Tier 3 air doors | No Syndicate aircraft in M4. C refuses air units at every tier, and cites 0250 and 0251 as the rule to come. |
| Data files and hot reload | **New (the user, after 0282: "Modules now, data files at playtest").** Playtest prep, after all four factions: the defs and tuning modules load from data files that can be reloaded live. M4-R leaves the defs as plain serde-ready data and the tunables grouped by system, so this step is a loader, not a refactor (T54). |
| Depends on M5 | Nothing. |
| AI opponent | Undecided |

### 1.3 What already exists (so the slices don't redo it)

The adopted plan's list, at `3a1b296`, still holds. Since then:

**M4-A** (`fcd06dc`; plan.md "M4-A Syndicate foundation"):
- **Kinds.** `defs/syndicate.rs` (232 lines, tests in `defs/syndicate/tests.rs`) holds GUARD, AGENT, TUNNEL_TIERS, TUNNEL and HIDEOUT. `ObjectKind` ends `PeacekeeperShielded, Agent, Tunnel, Hideout`.
  - The Tunnel is 4x4, 1/16, `needs_flat_ground`, ABCD, `tunnel: Some`.
  - The Hideout is 2x2, `underground`, Groupable, sight 0, HP 400, **6/24**, AAAA, `production: None`. C adds its production.
- **Tunnel state.** `Object::tunnel: Option<TunnelState { tier }>`, hashed only while set. `tunnel::{BASE_TIER, def_of, fresh, tier_def, set_tier}`; `set_tier` keeps the damage (`follow_max`). Maps can't set a per-object tier: tests use `SetTier`.
- **The hidden predicate.** `vision/hidden.rs`: `Hiding { owner }`, `Hiding::of` (an owned underground structure), and `hidden_from(object, player)` / `Sim::hidden_from`.
  - Targeting, `World::sees`, the gate, the records and the client's `Sight` use it. **Damage never asks it.**
  - The DrillUnit arm and a "revealed" term come later.
- **Combat.** The Agent's pistol, and an AoE test def (a test Guard), whose test `a_hidden_underground_structure_in_an_area_takes_its_damage` pins "damage never asks the predicate". AoE resolution itself is not built (O).
- **Client.**
  - Labels AG, T1–T3 and HI.
  - Owner-only outlines for underground structures.
  - `object_render/tunnel.rs`: the Side A paths by tier, hatch marks on B and D, and the Tier 3 roof circles. Its docs say "slice M4-D picks the square".
  - `combat_effects.rs::ShotStyle::of`: the Guard is a `RedLaser`, everything else a `Tracer`.
  - The no-fog debug views draw hidden Hideouts. That is accepted: they are debug views.
- **Deviations to remember.** `sim.rs` is at 602 lines and wasn't split. Its new steps go to their own files
  (`sim/network.rs`, `sim/sites.rs`, ...; §3.2, Splits first), and `Sim::step` takes one call per step.

**M4-D** (`edc30e11`): one Gather job for the Drone and the Agent; `WorkerDef::LoadDef` per resource;
`StructureDef::drop_off` (`Anywhere`, or `Hatches { crystals: B, supplies: D }`); `tunnel::hatch_square`;
`GatherPhase::AtHatch`, the one-at-a-time line; the nearest Tunnel by its hatch centre. §3.3 D has the details.

**Combat today** (for O): `combat.rs` (364 lines) and `combat/{phases.rs 479, damage.rs 416, targeting.rs}`.
- Phases: Aiming → Firing → Cooldown → Reloading.
- `fire()` pushes a `Hit` at the end of Firing, with `SimEvent::Attacked { attacker, target, from, to }`. The tick's list is applied once by `damage::apply`, which emits `Hit` and `Died`. `vision::reveal` reveals the attacker to the owner of what it hit.
- `AttackState { target: Option<ObjectId>, phase, elapsed }` is hashed.
- Only units attack: `Shooter::new` takes a `&UnitDef`.
- `domain_allows`: Ground → Ground; the surfaced Underground target is a TODO. `reaches_point` exists, for location targets.
- The module docs: "Not built yet: turrets, projectiles and location targets (the disjointed attack types), and AoE damage."
- The defs already have:
  - `AttackType::{FullyConnected(Ranged | Melee), HeadDisjointed, TailDisjointed, DoublyDisjointed}`;
  - `TargetType::{SingleTarget, AoE { radius }}`;
  - `AttackAttrs { ..., projectile_speed: Option<f32> /* su per tick */, aim_ticks, firing_ticks, cooldown_ticks, reload_ticks }`.
- The client:
  - `combat_effects.rs` (443 lines) draws `Tracer`s from `Attacked`, clipped to Visible tiles by `visible_spans`. A revealed attacker's shot at the viewer's own object is drawn whole (0244, `shot_spans`).
  - The AttackGround panel slot (D) exists, conditional on CanTargetGround (`templates.rs:425`). Its left-click gives no order ("no sim order yet", `resolution/tests.rs:150`). Its pointer and colour exist (`cursor.rs`).
  - `OrderKind` has no AttackGround; orders go through `Command::Order { units, kind, queue }`.

**Placement and validation** (for B, E, F):
- `placement/check.rs` (474 lines):
  - `TileProblem::{OffMap, NotBuildable, NotFlat, NotVisible, Structure, Resource, Unit, Reserved}`. It is a u8 bitset, and it is **full**.
  - `WholeProblem::{NotAStructure, OutsideBuildArea, NotOnPatch}`.
- **Any structure overlaps any structure**, underground included. A GDO direct placement over a hidden Hideout is therefore already red (0240's accepted leak).
- `Mode::Known`'s occupants are own, seen and remembered objects, so a worker-built ghost over a hidden expansion stays green. Arrival uses the truth.
- Map spawns and debug spawns follow only the shared-tiles and Onto rules.
- `scenario/validate.rs` is 502 lines, but layouts live in `scenario/validate/layouts.rs` (143 lines).
- `restriction()` treats a faction without Starting Conditions as allowing nothing, so Syndicate layouts are empty everywhere (B's findings).
- The map's presets carry `drillable` (`map.rs`): Plane, Rugged and Cliff true; Mountain and Water false.

**Files near or over 500 lines** (split before adding to them), at `6460e2a`:
- sim: `sim.rs` 602 (C splits it), `scenario/validate.rs` 502, `orders.rs` 495, `paths.rs` 486, `movement/collide.rs` 480, `combat/phases.rs` 479, `placement/check.rs` 474, `testing/invariants.rs` 465, `vision.rs` 462, `production.rs` 416, `combat/damage.rs` 416;
- client: `editor/objects.rs` 613, `fog.rs` 564, `editor/actions.rs` 498, `object_render.rs` 493, `hud/selection_panel.rs` 476, `command_ui/indicators.rs` 461, `map_render.rs` 452, `combat_effects.rs` 443, `command_ui/interface/templates.rs` 435, `command_ui/cursor.rs` 419, `cli.rs` 419, `command_ui/resolution.rs` 418.

---

## 2. First playable

After A, B, C, D, R1, R2, E, H, J1 and J2 (end of wave 5; M4-R adds a wave before it). It is unchanged except where
marked.

1. Run `cargo run -- --factions syndicate,gdo`. Player 1 starts as the Syndicate in slot 1:
   - a Tier 1 Tunnel ("T1"), with Side B's hatch 1 GU from a patch and Side D toward the map's centre;
   - a Hideout inside its area, drawn as an outline that only its owner sees;
   - one Agent; 100 SC and 1 Supply;
   - Tunnel Space 1/11: the Tier 1 region is 48 cells, the Hideout takes 4, and 44 / 4 = 11;
   - the camera on the Tunnel.
2. Select the Hideout and press Q to train an Agent (100 SC, 15 s). With no rally, it goes into the network.
   - **Changed (0257):** select the Tunnel, press **E** (Eject), then **Q** for the Agent tile. The Agent walks out of
     Side A's exit half, the one nearer B (0248, 0250).
   - With the Hideout selected, right-click a patch. From then on, new Agents come out by themselves and gather (0136,
     0253).
3. Select Agents and right-click a patch.
   - Each mines for 5 s, walks to the crystal hatch and drops off 50 in 4 s, one Agent at a time.
   - **Changed (0255):** the others wait at the hatch for as long as it takes.
   - Three Agents saturate the Tunnel at 12.5 SC per second, between 0:34 and 0:46 (0259). **Open (Q0076):** with the
     patch 1 GU from the hatch, as 0259 and B's layout put it, three Agents jam head-on today (5.33 SC per second).
     The first playable needs Q0076's answer, or a layout with the patch 3 GU or more away.
4. Select an Agent and press A. Tab steps through the Agents, which are ungroupable.
   - The ghost shows the 8x8 area. **Changed (0237, 0242):** it turns red where the area would overlap any known
     Tunnel's, or where it would reach Water, Mountain or the map edge. R and F turn it.
   - **Changed (0233):** the click pays the Supply at once and sends the Agent there.
   - The Tunnel appears at 10% HP with the Agent inside. **X** on the site cancels it, refunding the current top price:
     1 Supply here.
   - 30 s later the Tunnel is operational, available Tunnel Space rises from 11 to 23 ((44 + 48) / 4), and the Agent
     is in the network: eject it from either Tunnel.
5. Right-click a Tunnel with Agents that carry nothing: they walk in single file and go in (0246). Agents that carry a
   load drop it off instead.
6. Press F2 to play the GDO, build Peacekeepers and attack. The Agents fight back with their pistols. The GDO sees the
   Tunnels and their tier, but never the Hideout.
7. Destroy a Tunnel.
   - Its Hideout turns neutral, now visible to everyone, and decays at 1% per second (100 s; **Changed, 0258**).
   - Units in the network survive while another Tunnel stands.
   - Destroy every Tunnel: the units in the network are destroyed, and the Syndicate is defeated, having no structure
     left. Its Agents on the map become neutral.

**Not needed for it:**
- F (expansions after the start), G (upgrades), I (Supplies), K (Guards), L and M;
- the new slices: O, U, S1 and S2. (R1 and R2 change no behaviour.)

**Earlier checkpoint (scripted, after A, B, C and D, end of wave 2).** As adopted: a `--commands` QA file trains,
ejects and gathers. Screenshots show Agents at the patches and at the hatch, the bank rising, and Tunnel Space 3/11.

**Later checkpoints.**
- **After W3 (M4-R):** nothing visible changes. Every test and golden passes unchanged, and a number lives in one
  place.
- **After W6:** Guards behind an Optics Lab, Tier 2 Tunnels, and O's Attack Ground and area damage, with test units.
- **After W8:** Grenadiers behind a Chemical Plant, upright Laser Turrets, and pop-up turrets that rise at an enemy.

**How it is played and tested without an AI.** As adopted: hot-seat with F2, scripted QA (`--commands`,
`--input-script`, screenshots at fixed ticks), goldens through the `Script` hook, and the Syndicate oracle (L).

---

## 3. Slices

### 3.1 Overview

| # | Slice | Content | Design | After | Model |
|---|---|---|---|---|---|
| A | Syndicate foundation | **Merged** (`fcd06dc`) | — | — | Opus |
| B | Syndicate start | Tunnel Space by 0242 and 0258; the Starting Conditions; `dev_skirmish` layouts; **0242's area rule** for layouts and the editor; **0264's armour test** for every faction; `--factions syndicate`; the HUD | `tunnels.md > Starting Conditions`, `> Tunnel Space`, `> Drillable area`; `core/entities.md > Building armor rule` | A | Opus |
| C | Tunnel Network | **Running**, on its brief (0245–0253) | `tunnels.md > Tunnel Network`, `> EjectMenu` | A | **Fable** |
| D | Agent crystal gathering | **Merged** (`edc30e11`), on its brief (0255, 0248) | `units/agent.md > Gathering` | A | **Fable** |
| **R1** | **Code organization: the sim (new, 0282)** | One module per unit and per structure, `defs/<faction>/{units,structures}/<object>.rs`, as plain serde-ready data; each object's numbers in its module; tunables in `tuning/<system>.rs`; shared mechanics (one toggle rule, one progress type); the `orders.rs` and `combat/phases.rs` splits; the sim's citations moved to 0282's paths | 0282 | B, C, I | Opus |
| **R2** | **Code organization: the client and docs (new, 0282)** | The citation test; the client's, `docs/`' and `ARCHITECTURE.md`'s citations; one client module per kind (label, panel template, shot style); the client splits later slices need; the client's feel tunables | 0282 | B, C, I | Opus |
| E | Agent-built Tunnels | The kind-generic Build order (T51); paid at the ghost, top-price refunds (0233); the site (0234); placement against **every player's** areas (0237), drillable and on the map (0242), and over hidden expansions (0240) | `units/agent.md > Building`; `tunnels.md > Cost Scaling`, `> Tunnel Area Non-Overlap Rule`; `core/entities.md > Placement Validation` | C, R1 | Opus |
| F | Expansions | `Method::Expansion`: Drillable tiles, both layers, resources (0239–0241); the operation slot; paid at the ghost; ConstructionHP; the 75% refund; the Tunnel's X (0254); the ExpandMenu in grid order (0257) | `structures/README.md > Tunnel Expansions`; `tunnels.md > ExpandMenu`, `> AwaitingPlacement (Expansion)`, `> Expansions under construction` | B, E, J2 | Opus |
| G | Tunnel upgrades | 480 frames, paid at order, counting in-progress upgrades, top-price refunds, HP with max HP, the reserved area, blocked by any Tunnel or undrillable cells (0229, 0231, 0232, 0242); Q and X (0257) | `tunnels.md > Upgrade Costs`, `> Tunnel Area Non-Overlap Rule` | F, H | Opus |
| H | Losing Tunnels | Decay at 1% MaxHP per second, explicit Attack only, reclaiming at completion, blocked cells, unfinished expansions lost, the defeated player's network units destroyed (0254, 0258) | `tunnels.md > Losing Tunnels`; `core/game_rules.md` | C, E | Opus |
| I | Supplies | **Running**, on its brief (0248, 0256): Supplies at Side D's hatch; shared stations, waiting at an empty one, switching loses the load | `units/agent.md > Gathering > Supplies` | D | Opus |
| J1 | Agent UI | Q Move, A Build Tunnel, S Drop Off, D Build Laser Turret (greyed), X Stop (0257); the ghost with every known area and the drillable rule; the site's X; pointers | `units/agent.md > Agent ObjectInterfaceState`; `selection_and_hud.md` | C, D, E, R2 | Opus |
| J2 | Tunnel and Hideout UI | Q Upgrade, W Expand, E Eject, X Cancel (0257); the EjectMenu in grid order; the overlay; the network's contents; the Hideout's Q W E X C; rallies (0252) | `tunnels.md > Tunnel ObjectInterfaceState`, `> EjectMenu`; `structures/hideout.md` | B, C, R2 | Opus |
| K | Prerequisites, the Optics Lab and the Chemical Plant | Generic prerequisites; the Optics Lab (0229) and **the Chemical Plant** (0260), each in its own module, at 6/24; the Guard's gate; the Grenadier's and the turret's gates as data | `core/tech_and_research.md`; `structures/optics_lab.md`, `structures/chemical_plant.md` | F | Opus |
| L | Syndicate oracle and goldens | Measured to the hatch square, ±15%, the exact 12.5 ceiling, the 0:34–0:46 opening, stacked Tunnels on opposite sides (0259); **Q0076 open** | `units/agent.md > Gathering > Space Crystals` | B, C, D, E, H, I | Opus |
| M | M4 QA | Perf, soaks (with grenades and turrets), the walkthrough, docs, the ignored suite | `ARCHITECTURE.md > Adding a faction` | everything | Opus |
| **O** | **Combat spike (new)** | DoublyDisjointed projectiles with travel time; area damage by overlap, underground objects included (0218, 0262, 0264), built once for every AoE attack; Attack Ground (order, behaviour, gate, panel, pointer, indicator); drawing projectiles and blasts, clipped by vision | `systems/combat.md`; `commands.md > AttackGround`; `behaviors.md > AttackingLocation`; `core/vision.md > Attacks from outside vision` | R1, R2 | Opus |
| **U** | **Grenadier (new)** | The kind, in `defs/syndicate/units/grenadier.rs`; its recipe at the Hideout (E); its Chemical Plant gate; 0260's verification; its client module | `units/grenadier.md`; `structures/hideout.md` | O, K | Opus |
| **S1** | **Laser Turret: upright, Agent-built (new)** | The kind, in `defs/syndicate/structures/laser_turret.rs`; structures that attack and auto-target; the Agent's D and the Build order's turret arm; the Agent emerging adjacent; red laser; counts for defeat | `structures/laser_turret.md`; `structures/README.md > Surface Defensive Structures`; `units/agent.md > Building` | E, J1, K, O | Opus |
| **S2** | **Laser Turret: pop-up, Tunnel-built (new)** | The pop-up form: hidden, 6/6, 1 region cell, rise and retract; conversion when areas change; the ExpandMenu entry and the operation slot (0267) | `structures/laser_turret.md` (Upright vs Pop-up, Conversion, Construction) | S1, F, G, H | Opus |

The Grenadier's slice is **U**, because M4-R is the code organization's.

**Gated, not scheduled** (end of 3.3):
- N3 the vehicles: the Vehicle Assembly, Assault Bike, Laser Battery (with Relay), the Subterranean Transport, and the
  deploy order;
- N4 Shadebringer;
- N5 the remaining expansions;
- N6 the Phantom, with aircraft and the air doors;
- **N7 the Specter (new)**.

N1 (the Grenadier) and N2 (the Laser Turret) are now scheduled, as O, U, S1 and S2.

### 3.2 Waves and parallel tracks

```
W1  A                                                              merged (fcd06dc)
W2  B (running) | C (Fable, running) | D (Fable, merged edc30e11) -> I (after D; running)
W3  R1 (after B, C, I) | R2 (after B, C, I; merges after R1)        code organization (0282)
W4  E (after C, R1) | J2 (after B, C, R2)
W5  J1 (after E, R2) | H (after C, E) | F (after B, E, J2)          -> first playable (after J1 and H)
W6  K (after F) | G (after F, H) | O (after R1, R2)
W7  L (after B, C, D, E, H, I) | S1 (after E, J1, K, O)
W8  U (after K, O) | S2 (after S1, F, G, H)
W9  M (after everything)                                            -> agent QA; the user playtests after M5
```

- **Critical path**: A → C → R1 → E → F → K → S1 → S2 → M. M4-R adds one wave. The first playable comes after J1 and H,
  at the end of W5.
- **I ran early.** It launched as soon as D landed, in the slot D freed, so W4 keeps two slices.
- **Why R1 and R2 run as a wave of their own** (the Director: after B, C and I land, before E and J2):
  - Both touch most files of their crate, so anything running beside them would conflict.
  - Every later slice then adds its kinds, numbers and files in the new layout, and cites 0282's paths.
  - They split by crate, so they share no file. R1 updates the client's `use` lines for the sim constants it moves
    (about 16 lines); R2 touches only comments, its own new modules and the client splits. R2 merges second, so that
    its citation test also checks R1's files.
- **Why O is in W6.** O depends only on A and M4-R. It could join W4, which has two slices, but it would append to
  `OrderKind` beside E. If the Director accepts that one-line conflict, O moves to W4, which de-risks the spike
  early, and nothing else changes. Nothing needs it before W7 (S1) and W8 (U).
- **Why U and S1 don't run together.** Each adds a kind. With one module per kind they no longer share a defs file,
  but both append to `ObjectKind`'s list and to the client's kind registry. U in W8 costs no time, since S2, on the
  critical path, is in W8 anyway.
- **Machine load.** Three builds at most, in W2, W5 and W6; two in W3, W4, W7 and W8. W2 ran both Fable slices.
- **Shared registries.** These are edited by several slices. Edits are one-line and additive, and **the waves are
  arranged so that no two parallel slices append to the same one**, except for the m4-common files:
  - **m4-common's shared files:** `sim.rs::step`, `command.rs`, `sim/gate.rs`, `testing.rs`, `object.rs`, `events.rs`,
    `state_hash` and `ARCHITECTURE.md`. Small additive edits only, as m4-common says.
  - **`OrderKind`** (in `orders.rs` after R1's split), with its `destination`/`target` arms and the dispatch. Only one
    order-adding slice per wave: C in W2, E in W4, O in W6. S1 adds no order: E's Build order is kind-generic (T51).
  - **The kind registry**: R1's single list of kinds in `defs.rs` (T54), and R2's client kind registry. Only one
    kind-adding slice per wave: K in W6, S1 in W7, U in W8.
  - **The tuning modules** (`tuning/<system>.rs`, R1). A slice adds its constants at the end of its system's module.
    Two slices in one wave never add to the same module: E (W4) adds to `construction`; F (W5) to `construction`,
    H (W5) to `tunnel`; G (W6) to `tunnel`, O (W6) to `combat`; S1 (W7) to `combat`. R1 moves the constants of B, C
    and I, which land before it.
  - **`economy/army.rs`'s occupancy predicate** (B's). S2 adds the pop-up turret's arm in W8; U, beside it, doesn't
    touch it.
  - **`vision/hidden.rs`'s arms** (A's). S2 adds the lowered turret's arm in W8, alone.
- **Splits first** (T28). Each file near 500 lines is split before anything is added to it:
  - `placement/check.rs` (474): **B**;
  - `orders/gather.rs` (490): D (done);
  - `orders.rs` (495) and `combat/phases.rs` (479): **R1**;
  - the defs: **R1**, into one module per object (T54);
  - `command_ui/interface/templates.rs` (435), `hud/selection_panel.rs` (476), `command_ui/indicators.rs` (461),
    `object_render.rs` (493) and `combat_effects.rs` (443): **R2**;
  - `sim.rs` (602): every new step goes to its own `sim/*.rs` file.

**What each slice owns** (for the "don't touch" lists). The shared registries above are open to every slice on the
terms given.

| Slice | Owns | Read-only |
|---|---|---|
| A | merged | — |
| B | `economy/army.rs` (Tunnel Space); `player.rs`; new `tunnel/area.rs` and its one `mod` line in `tunnel.rs`; the `placement/check.rs` split and the area and layout-expansion rules; `scenario/starting.rs`, `scenario/validate/`, `scenario/problems.rs`; `scenarios.rs`, `maps/`; new `tests/syndicate_start.rs`, `tests/building_armor.rs`; client `cli.rs`, `hud/resources.rs`, `editor/layouts*` | `defs/`; C's and D's files |
| C | as adopted, and its brief | `economy/army.rs` (B's) |
| D | merged | — |
| **R1** | everything under `crates/sim/` for the moves T54 lists: `defs/**` and the kind registry; new `tuning/**`, `progress.rs`, `toggle.rs`; the `orders.rs` and `combat/phases.rs` splits; constants and citations in every sim file; new `defs/tests/serde.rs`; the client's `use` lines for moved sim constants | every hashed state layout (no golden may change) |
| **R2** | the citations in `crates/game/**`, `docs/**` and `ARCHITECTURE.md`; new `crates/sim/tests/design_citations.rs` (its one sim file); new `crates/game/src/kinds/**` (one module per kind); per-object template files under `command_ui/interface/templates/`; the five client splits; new `crates/game/src/tuning/**` | the sim; `use` lines of sim constants (R1's) |
| E | `orders/build.rs` (new: the Build order); `construction/site.rs` (new: `Construction`); `sim/sites.rs` (new); the generalised ghosts from `sim/deploy.rs`; the Tunnel's Worker-Built constraint in `placement/`; the Tunnel's price in `defs/syndicate/structures/tunnel.rs`; the site's Cancel | `network.rs` (C's); `tunnel/area.rs` (B's) |
| F | `Method::Expansion` in `placement/` (with `TileProblem` widened, T53); the operation slot (`TunnelState::operation`); `TunnelCommand::{Expand, Cancel}`; the Tunnel as a builder in `construction/site.rs`; the 75% refund; the ExpandMenu's ordered list in `defs/syndicate.rs`; client `templates/syndicate/expand_menu.rs` (new), `Builder::Tunnel`, the clamped ghost | `templates/syndicate/tunnel.rs` outside the W entry |
| G | `TunnelCommand::Upgrade` and Cancel's upgrade arm; the upgrade operation; the reserved area; the tier change; the client's Q entry in `templates/syndicate/tunnel.rs` | `sim/tunnels.rs` (H's) |
| H | `sim/tunnels.rs` (new: neutralise, decay, strand, reclaim, unfinished expansions); its hook in `sim/removal.rs`; the defeat tests; the client's look for neutral expansions | `network.rs` |
| I | as its brief: the station leg and pick-up in `orders/gather*` and `economy/gathering.rs`; the station decrement; the Supplies hatch; the empty-station placeholder in a new small client module | D's side rule; B's and C's files |
| J1 | client `templates/syndicate/agent.rs` and the site's template; `Builder::Agent` in `command_ui/interface/placing.rs`; the Enter pointer in `command_ui/cursor.rs`; the lingering Tunnel ghosts; the carried load | the sim |
| J2 | client `templates/syndicate/{tunnel,hideout}.rs` and the EjectMenu; `tunnel_areas.rs` (new); the network's contents; selection and control groups for held units | the sim |
| K | `tech.rs` (new); `requires` in the structure defs; new `defs/syndicate/structures/{optics_lab,chemical_plant}.rs`; the Guard recipe's gate; the production and construction gate checks; the client's `conditions.rs`; `kinds/syndicate/{optics_lab,chemical_plant}.rs` | — |
| L | `testing/syndicate.rs`, `tests/economy_oracle/syndicate.rs`, `tests/golden/syndicate.rs` (new); `docs/economy_oracle.md` | everything else |
| M | perf and soak tests, the QA scripts, `ARCHITECTURE.md`, `docs/playtest/m4.md` | — |
| **O** | `combat/` (new `combat/projectiles.rs` and `combat/area.rs`; AttackState's location); `orders/attack_ground.rs` (new); the AttackGround checks in `sim/gate/orders.rs`; `tuning/combat.rs`'s additions; client: `combat_effects/{projectiles,blasts}.rs` (new), AttackGround in `command_ui/resolution.rs`, its marker in `command_ui/indicators/` | `defs/` (test kinds through `test_defs`) |
| **S1** | new `defs/syndicate/structures/laser_turret.rs`; `StructureDef::attack`; structure shooters and auto-targeting in `combat/`; the Build order's turret arm in `orders/build.rs` and `construction/site.rs`; client: the D entry in `templates/syndicate/agent.rs`, the turret ghost in `placing.rs`, `kinds/syndicate/laser_turret.rs`, `object_render/turret.rs` (new) | `tech.rs` (K's); `network.rs` |
| **U** | new `defs/syndicate/units/grenadier.rs`; its recipe in `defs/syndicate/structures/hideout.rs`; client `kinds/syndicate/grenadier.rs` and the E entry in `templates/syndicate/hideout.rs` | `combat/` (O's) |
| **S2** | `popup.rs` (new: the pop-up form); the lowered arm in `vision/hidden.rs`; armour by form; the occupancy arm in `economy/army.rs`; the turret arms of `Method::Expansion` and of the operation slot; the ExpandMenu entry (data); client: the pop-up's look in `object_render/turret.rs` | `sim/tunnels.rs` (H's); `defs/syndicate/units/` and `structures/hideout.rs` (U's) |

**Every slice follows the Definition of done** (`ARCHITECTURE.md > Definition of done`), as adopted, with these
changes:
- **Changed (m4-common).** **Goldens.** New state is hashed only while it is set, so goldens change only when behaviour
  does. A slice that changes hashed behaviour re-blesses once, after its final `git merge master`, and explains every
  change. This replaces the adopted "A, C and E each re-bless every golden once".
- **Changed (0282).** **Where things go** (T54), from W4 on:
  - a new kind gets its own def module, `defs/<faction>/{units,structures}/<object>.rs`, and its own client module,
    `kinds/<faction>/<object>.rs`, each citing its design file;
  - a number that belongs to one object goes in its def; a system's tunable goes in `tuning/<system>.rs`, with its
    design citation or "Placeholder (not in the design)", or the QD it waits on;
  - new code cites 0282's paths (`factions/<faction>/units/<unit>.md`, `.../structures/<structure>.md`), and R2's
    citation test fails on a path that doesn't exist.
- The `ARCHITECTURE.md` items to add, as adopted, plus:
  - Code organization: per-object modules, tuning modules, the citation test (R1, R2);
  - Combat: projectiles, area damage, location targets, structures that attack (O, S1);
  - Object model: the pop-up form (S2).

### 3.3 Slice details

The quotes below are the design text the briefs should carry word for word (insights: "Quote design text in briefs;
don't paraphrase it"). **Changed (NNNN)** marks where an answer changed the adopted plan, and **Placeholder
(QD-slug)** marks a proposal built until a question draft is answered.

#### A: Syndicate foundation (Opus): merged

Merged in `fcd06dc`, with 1412 tests and the goldens unchanged. §1.3 lists what it built.
- **Folded in while it ran:**
  - the hatches on B and D (0248);
  - the Hideout's sight 0 and Groupable (0254);
  - the tier seen and remembered by enemies (0258);
  - the Hideout's 6/24 armour, and "damage never asks the predicate" (0262, 0264).
- **What later slices take from A:**
  - `tunnel::{tier_def, set_tier}`: G's tier change goes through `set_tier`, which keeps the damage (`follow_max`), as
    0231 asks ("A Tunnel at 500/600 HP upgraded to Tier 2 ends at 700/800").
  - `hidden_from`: S2 adds the lowered turret's arm. The DrillUnit and detection arms stay gated.
  - `object_render/tunnel.rs`'s side data: D derives the hatch square from it, and B, I and L use D's function.
  - `ShotStyle::of`: S1 adds `LaserTurret => RedLaser`.

#### B: Syndicate start (Opus)

> The revised spec, written first for B's launch. It is also `scratchpad/m4/rev/B.md`.

**Goal.** A Syndicate player starts on `dev_skirmish` by the Starting Conditions, and the HUD shows Tunnel Space,
derived from the owner's Tunnels. Syndicate Starting Layouts are validated, including 0242's area rule, in the sim and
so in the editor. A defs test holds every faction's buildings to 0264's armor rule.

**Design**
- `tunnels.md > Tunnel > Starting Conditions`: "One pre-built **Tier 1 Tunnel** containing one pre-built **Hideout**";
  "One **Agent**"; "**1 Supply** (enough for the 2nd Tunnel, which costs 1 Supply)"; "Starting Space Crystals:
  **100** (same as GDO and the Colonists)". "Where each starting object is placed is defined by the map's Starting
  Layout for this faction".
- `core/camera.md > Starting Position`: "Syndicate: starting Tunnel".
- `tunnels.md > Tunnel Space` (as 0242 left it):
  - "**Tunnel-region**: a Tunnel's Tunnel Area, excluding the 4x4 grid units occupied by the Tunnel's own footprint";
  - "**Unused tunnel-region**: the portion of a Tunnel's tunnel-region not currently occupied by a placed expansion";
  - "**The whole area is always drillable** (... replaces the earlier "off-map cells count" and "non-Drillable cells
    don't count"): a Tunnel can only be placed, and only be upgraded, if its whole Tunnel Area (at the new tier, for an
    upgrade) is on the map and Drillable ... So every tunnel-region cell is usable";
  - "Tunnel Space = floor((sum of unused tunnel-region grid units across all owned Tunnels) / 4) — rounded down to a
    whole number — capped at a hard maximum of 200";
  - "Because of the rounding, structures smaller than 2x2 (e.g. a 1x1 pop-up Laser Turret, ¼ of a Tunnel Space) only
    lower Tunnel Space when they push the unused total below the next multiple of 4";
  - "usage is simply displayed over capacity (e.g. 12/10) and no further units can be produced until usage drops back
    within capacity".
- `> Production and Tunnel Space`: "the unit's Tunnel Space counts as used from that moment — through production and
  for as long as the unit lives"; "a unit with TunnelSpaceCost 0 always passes".
- `tunnels.md > Tunnel Area Non-Overlap Rule > Drillable area`: "a Tunnel can only be **placed** if every cell of its
  Tunnel Area is **on the map and Drillable** (no Water or Mountain; core/map.md > DefaultTilePresets). So **no Tunnel
  near the map edge** (closer than its area reaches)."
- 0242, Implementation impact: "Starting Layouts must respect this (the map editor's starting Tunnel placement); flag
  if any existing layout violates it."
- `core/map.md > Tile`: "Any combination of properties is valid": Drillable is the preset's flag (Plane, Rugged, Cliff
  true; Mountain, Water false), never its look.
- `core/map_editor.md > Starting Layouts > Layout validation`: "Each faction's layout is checked with that faction's
  **normal in-game placement rules, minus visibility**".
- 0258 point 4: "A neutral expansion partly inside an own Tunnel Area **still blocks its cells and counts as occupied
  for Tunnel Space**."
- 0254 point 4: "Tunnel Space is used once construction starts. Today ghost placement starts construction at once";
  0255: "the expansion's cells are reserved from construction start".
- `units/agent.md > While the Tunnel is under construction`: "It **does nothing but Cancel**: no Tunnel Space, no
  drop-offs, no Enter, no Expand, no Eject".
- `structures/laser_turret.md > Upright vs Pop-up`: the pop-up "Occupies 1 grid unit of that Tunnel's
  tunnel-region, like an expansion"; the upright one "Costs no Tunnel Space".
- 0265: "No special case: units count from production for life."
- `overview.md > DisplayHud`: "Space Crystals: current amount", "Supplies: current amount", "Tunnel Space: current used
  / current available (max 200)".
- `core/entities.md > Structure Type > Building armor rule` (0264): "A building's armor should generally be
  **PointArmor n, FullArmor n x (its size in grid squares)**, for some n chosen per building ... There may be
  exceptions."
- `structures/README.md > Tunnel Expansions > High armor underground` (0262, 0264): "**Placeholder for every expansion: n =
  6, i.e. PointArmor 6, FullArmor 6 x size in grid squares** (a 2x2 expansion: 6 / 24) ... this replaces the 1/4
  listed in individual stat blocks."
- 0248, the sides: "**A**: entrance/exit; **B**: crystal hatch; **C**: back wall (no function); **D**: Supplies
  hatch". 0255 point 6: "**The hatch is 1 grid square, near the middle of the side.**" 0259: "**Distance** is measured
  edge to edge, patch to the crystal **hatch**", and the opening target has the "patch 1 GU from the hatch".

**What changed from the plan**
1. **Changed (0242; 0238 void; 0241 in part).** Tunnel Space has no off-map or non-Drillable case: every region cell
   counts. The function tests neither bounds nor drillability.
2. **Changed (0258 pt 4).** The occupied cells are those under *any* own or neutral underground structure, counted per
   cell. A neutral expansion partly inside the area counts only its inside cells.
3. **New (0242).** A layout Tunnel's whole Tier 1 area must be on the map and Drillable. The rule goes into the
   placement check, so layouts, the editor and later E's ghost and arrival check share it.
4. **New (0264).** A defs test across all factions' buildings.
5. **Changed (0248, 0255, 0259).** The layout geometry. Side B's hatch is 1 GU edge to edge from a patch, and Side A
   faces the open map. Side D (Supplies) faces the map's centre, toward the station.
6. **Unchanged.** The site filter (E), the current tier during an upgrade (G) and the held units (C) are left to those
   slices as hooks. Goldens: no change.

**Code**
- **The Tunnel Area geometry.** New `crates/sim/src/tunnel/area.rs`, declared by one line in `tunnel.rs` (C edits
  `TunnelState` there, so keep it to that line).
  - `area(placement, radius)`: the footprint grown by `radius` on all four sides. It is an `i32` rectangle that may
    reach off the map.
  - The region cells: the area less the footprint.
  - `breaches(map, area)`: the area's off-map cells and its on-map cells whose preset isn't `drillable`.
  - E, F, G, H, J1 and J2 all use this module, so it is the one place that computes a Tunnel Area.
- **Tunnel Space** (T33 as revised), in `economy/army.rs`.
  - `ArmyResource::TunnelSpace`: `of(Syndicate)`, `name` "Tunnel Space", and `cost_of` returning the unit's
    `tunnel_space_cost`.
  - **Available** = min(`TUNNEL_SPACE_CAP` 200, ⌊Σ unused / 4⌋), summed over the owner's Tunnels.
    - Each Tunnel's unused cells are its region cells at its current tier (`tunnel::tier_def`), less the region cells
      under a structure that **occupies tunnel-region**.
    - That is one predicate: the owner's or a neutral `underground` structure. S2 adds the pop-up Laser Turret's arm.
  - **Which Tunnels count.** One function, `gives_tunnel_space(o)`, true for every Tunnel in B.
    - E adds "not a site" (`units/agent.md > While the Tunnel is under construction`: "no Tunnel Space").
    - G needs nothing: the tier changes only at the upgrade's completion.
  - **Used** is the generic `taken_by` (units, plus their producers' units in production), over an iterator.
    `Sim::army_cap` passes the map's objects. **C chains its held store into that iterator** (units in the network,
    and an Agent inside its site, are alive and count).
  - Another player's expansion is never counted. It can't lie inside your area, since areas never overlap (0237) and
    expansions lie inside their own Tunnel's area. Counting it would also leak a hidden structure through the HUD.
- **The area rule (0242) in placement.**
  - Split `placement/check.rs` (474 lines) before adding to it. That split moves from E to B. C and D don't touch
    `placement/`.
  - For a kind with a `TunnelDef`, at `BASE_TIER`'s radius, two new `WholeProblem`s:
    - `TunnelAreaOffMap`: a cell past the map edge;
    - `TunnelAreaNotDrillable`: a cell on Water, Mountain or a custom non-Drillable preset.
  - In every mode, so E inherits it:
    - `Mode::Layout`, `Arrival` and `Command` use the truth;
    - `Mode::Known` counts a non-Drillable cell only on a tile the player has explored. Off-map cells always count,
      since 0242 says "terrain is always known to the player as far as explored; never-explored tiles pass on the
      ghost".
  - Map spawns and debug spawns keep only the shared-tiles and Onto rules, so they stay exempt.
- **The expansion rule for layouts.** A layout's `underground` structure lies wholly inside the Tier 1 area of a
  Tunnel in the same layout, the way GDO plates must lie in the same layout's Deployment Center build area.
  - Overlap with the Tunnel's footprint is already a `Structure` tile problem.
  - Report it as `WholeProblem::OutsideTunnelArea` through `check_layout`.
  - Until F's `Method::Expansion`, the Hideout is still checked as `Direct`, so it needs Buildable tiles, which is
    stricter than 0239's Drillable. Note it in a doc comment; F switches the rule over.
- **Starting Conditions.** `StartingConditions::of(Syndicate)`: Tunnel 1, Hideout 1, Agent 1; 100 crystals; 1 Supply;
  primary the Tunnel.
- **Layouts**, on both slots of `maps/dev_skirmish.json` and in `scenarios::dev_skirmish_map()`. Slot 2 is
  regenerated by the map's Rotate180 symmetry.
  - **Slot 1**, as a proposal. B may shift it, within these constraints.
    - **Tunnel** at tile (4,4), `R180`: A faces south, B west, C north, D east.
    - Its Tier 1 area, x 2–9 and y 2–9, is all flat Plane, and on the map.
    - Side B's middle squares (4,5) and (4,6) face the patches (2,5) and (2,6) across one empty column, x = 3. That is
      1 GU edge to edge whichever middle square D picks as the hatch (0255: "near the middle of the side"; the A
      client says "slice M4-D picks the square").
    - **Hideout** at tile (8,8), covering 8–9 × 8–9: inside the area, off the footprint and off the patches.
    - **Agent** about 1.5 GU south of Side A's middle, for example at (384, 608) su. That keeps it clear of the entry
      and exit spots C puts at Side A.
  - **Slot 2**, the image: the Tunnel at (56,40) with `R0`, the Hideout at (54,38), the Agent at (3712, 2464) su.
  - **Constraint for any other map:** a Tunnel origin must lie in [2, W−6] × [2, H−6] for its area to be on the map.
- **Client.**
  - `ResourceFigure::TunnelSpace`. `ResourceFigure::of(Syndicate)` = [Space Crystals, Supplies, Tunnel Space].
  - Used over available, in red when over capacity, as Beacon Capacity is.
  - `--factions syndicate,gdo` starts the Syndicate's layout.
  - The editor's Syndicate palette offers the Tunnel, the Hideout and the Agent.
  - Text for the three new `WholeProblem`s, in `rule_text` and the problem lines.
  - A starting Tunnel whose area breaches shows a red ghost with that text. `marked_tiles` already turns a whole
    problem red.
- **Tests that used the Syndicate as the not-implemented faction move to the Cults** (they would now hit
  `LayoutExtra` and `LayoutMissing`):
  - `sim/tests/scenarios.rs:282`;
  - `game_start/layout_rules.rs:64`;
  - `game_start/start.rs:34`;
  - `game_start/skirmish_default.rs:91, 155`;
  - `game/src/editor/layouts/tests.rs:63, 114`;
  - `game/tests/editor/layouts/tool.rs:38–45`;
  - and any other that assumes an empty Syndicate palette.

  `game/tests/editor/symmetry.rs:254` only mirrors a patch and may stay.

**Acceptance**
- **Start.** A `syndicate_start` test, for both slots:
  - the objects: the Tier 1 Tunnel with the Hideout inside its area, and the Agent;
  - 100 SC and 1 Supply;
  - Tunnel Space **1/11**: 48 region cells less the Hideout's 4 is 44, and 44 / 4 = 11;
  - the camera on the Tunnel.
- **Validation.**
  - `dev_skirmish` validates with the GDO, the Colonists and the Syndicate in all nine slot combinations, and
    `Sim::try_new` succeeds for each.
  - `outdated_layouts()` is empty, and the file equals the built-in map.
  - An incomplete Syndicate layout gives `LayoutMissing` for both slots, and no game starts. This is the M3 Colonists
    test (`colonists_start.rs:187`) for the Syndicate.
- **0242, as unit and layout tests:**
  - a layout Tunnel whose 8x8 area touches Water or Mountain → `TunnelAreaNotDrillable`;
  - one whose area touches Rugged or Cliff is valid;
  - a custom preset with `drillable: false` → `TunnelAreaNotDrillable`;
  - a Tunnel 1 tile from the map edge → `TunnelAreaOffMap`;
  - **a Tunnel 2 tiles from the edge is valid**: its area just reaches the edge. 0242's Verification says "A Tunnel 2
    tiles from the map edge is red", which is one tile off from its own rule. Build the rule as written, and cite
    Q0078.
  - in `Mode::Known`, Water on an unexplored tile passes and an off-map cell fails.
- **Expansions in layouts.**
  - A layout Hideout outside the Tunnel's area → `OutsideTunnelArea`.
  - A Hideout half inside the area → `OutsideTunnelArea`.
  - A Hideout on the footprint → `Structure`.
  - A Hideout on a patch → `Resource`.
- **Tunnel Space**, as unit tests:
  - the region of each tier: 48, 84 and 128 cells;
  - an own Hideout: −4 cells;
  - **a neutral expansion with 2 of its 4 cells inside the area: −2 cells** (0258 pt 4);
  - another player's underground structure is not counted;
  - rounding down: 47 unused cells give 11;
  - the cap: enough Tunnels for more than 800 unused cells give 200;
  - two Tunnels add up;
  - over capacity: a Hideout spawned into a full area leaves used above available, and nothing is removed;
  - **a map-spawned Tunnel by Water or the edge still counts all 48 cells** (0242: "no longer needs an off-map or
    non-Drillable case"; only spawns can make one);
  - used counts the units on the map and their producers' units in production;
  - cost 0 always fits.
- **0264, the defs test**, in new `crates/sim/tests/building_armor.rs`. It reads only `ObjectKind::ALL` and `def()`,
  so it shares no file with C or D.
  - For every kind of `Category::Structure` with `Footprint::Tiles { w, h }`: `armor.full == armor.point × w × h`.
  - `EXCEPTIONS: &[(ObjectKind, &str)]` holds each exception with its reason. It is **empty today**. The test fails if
    a listed kind actually follows the rule (a stale exception).
  - Every `underground` structure has PointArmor 6 ("Placeholder for every expansion: n = 6").
  - All 11 buildings pass today:
    - GDO: Deployment Center 1/16, Generator 1/4, Barracks 1/6, Extraction Facility 1/9, Extraction Plate 2/2;
    - Colonists: Beacon 0/0, Egress Node 0/0, Robo Portal 1/16, Bio Portal 1/9;
    - Syndicate: Tunnel 1/16, Hideout 6/24.

    The resources (patch, station) are `Category::Resource` and are not tested.
  - Later kinds go through this test unchanged:
    - K's Optics Lab and Chemical Plant at 6/24;
    - the Laser Turret's 1/1 and 6/6, both of which fit with n = 1 and n = 6. S2 adds its underground armor to the
      test.
- **Client.**
  - HUD screenshot: "Space Crystals 100", "Supplies 1", "Tunnel Space 1/11"; and an over-capacity "12/10" in red.
  - An editor test: a starting Tunnel dragged against Water, or to the edge, is refused with the rule's text.
  - The Syndicate palette holds the three kinds.
- **Goldens.** No change: Tunnel Space is derived, not stored, and only a slot's chosen faction's layout spawns. If a
  golden changes, explain it.

**Existing maps and layouts (checked for 0242): no breach.**
- `maps/dev_skirmish.json` is the only map file. Its Syndicate layouts are empty in both slots.
- The built-in maps (`scenarios::NAMES`: `dev_skirmish`, `dev_structures`, `empty`) have no Syndicate layout objects.
- The test maps that fill a Syndicate layout hold GDO objects, units or a patch, never a Tunnel. They are listed above,
  since they move to the Cults.
- Tunnels placed by map or debug spawns in tests are exempt (spawn rules), so they stay valid:
  - `sim/tests/syndicate.rs` at (1,0), (4,3), (4,4), (9,8), (11,3) and (14,10);
  - `game/tests/fog_memory/records.rs` at (8,4);
  - `vision/memory/tests.rs`.

  Some of these would breach 0242 if the rule covered spawns, for example the one at (1,0) on the edge.

**Owns.**
- Sim:
  - `economy/army.rs` (Tunnel Space);
  - `player.rs`;
  - new `tunnel/area.rs`, and its one `mod` line in `tunnel.rs`;
  - the `placement/check.rs` split, and the area and layout-expansion rules in `placement/`;
  - `scenario/starting.rs`, `scenario/validate/` and `scenario/problems.rs`;
  - `scenarios.rs` and `maps/`;
  - new `tests/syndicate_start.rs` and `tests/building_armor.rs`;
  - the moved tests.
- Client: `cli.rs`, `hud/resources.rs` and `editor/layouts*`.

**Read-only:**
- `defs/`. The armor test only reads it: report a violation, don't fix it.
- C's held store, `orders/resolution.rs` and `production/`.
- D's `orders/gather*` and `WorkerDef`.

**After**: A (merged). Runs beside C and D.

**Model**: Opus. The work is wide but shallow: one derived function, two validation rules, data and tests. Nothing in
it touches movement or collision.

**What C and D need from B, or must know**
- **C:**
  - Once B lands, `ArmyResource::of(Syndicate)` is `Some`. Syndicate production then waits on Tunnel Space (0143's
    shared check), so C's tests that train many units need the room: 11 at the start.
  - C chains the held units, network and site, into `used`, through the iterator B leaves.
- **D:** expose the hatch square as a sim function on the sides data. B's layout assumes either middle square of Side
  B, and L's oracle measures "to the hatch square" (0259).
#### C: Tunnel Network (Fable)

> **Running** on `scratchpad/briefs/m4c-tunnel-network.md`, which folds in 0245–0253 and overrides the text below where they differ (walking in with no fixed hold, the 32-frame transit delay, regeneration, the exit half nearer B, the longest-inside unit first, rallies, goldens hashed only while set). Not revised here. What later slices need from C is in "What later slices need from C and D" after D.

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
- `structures/README.md > Rally Point Behavior`:
  - "**Rally point set on the surface**: Unit auto-ejects from the parent Tunnel (Side A) and moves to the rally point."
  - "**No rally point, or rally point set on the parent Tunnel**: Unit stays in the Tunnel Network, available for ejection from any sufficiently-tiered Tunnel."
- `selection_and_hud.md > What a rally point makes a new unit do` (0136): "A unit that spawns with a rally point set **always** gets its **own default right-click command** resolved against the rally target".
- `tunnels.md > Losing Tunnels`: "A Tunnel that is still under construction does not count as an exit, and a Tunnel that is mid-upgrade counts at its current (pre-upgrade) tier."
- `core/entities.md > Exit Doorway`: "Aircraft and units released underground (e.g. Syndicate Tunnels) are not covered by this rule."
- `systems/units.md > Underground Collision > Surfacing`: "not allowed to surface where its Silhouette would overlap an above-ground unit ... nor a structure or terrain impassable for its surface form". This is the closest rule for the exit spot's test (Q0064).
- `structures/hideout.md > Produces`: "Agent: 100 Space Crystals, 240 frames (15 seconds)". `> HideoutInstanceState`: "BuildQueue: array of ObjectEnum (max 20, including the unit currently in production ...)".
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

#### D: Agent crystal gathering (Fable): merged

Merged in `edc30e11` (1439 tests; goldens unchanged), on its brief, which folded in 0255 and 0248. What it built
(`artifacts/director/plan.md`, "M4-D"):
- **One Gather job** for the Drone and the Agent (T34). `orders/gather.rs` is split (`claims.rs`, `patch.rs`,
  `deliver.rs`, and others).
  - `WorkerDef` has a `LoadDef` per resource. The Agent's Supplies are `None` until I.
  - `WorkerDef` has `shares_patches` and `keeps_drop_off`.
  - `StructureDef::drop_off` is `Anywhere`, or `Hatches { crystals: B, supplies: D }`.
- **The hatch.** `economy/drop_off.rs::hatch_square(object, resource)`, re-exported as `tunnel::hatch_square`, reads
  the square off `sides()`. It is the middle square farther from Side A (a placeholder, Q0076).
- **The line.**
  - `GatherPhase::AtHatch`: one Agent unloads at a time, by arrival tick, ties by id.
  - Waiting Agents stand side by side along the wall.
  - An Agent out of reach retries every 32 ticks for ever (0255).
- **The Tunnel.**
  - It is the nearest by its hatch centre, taken when the first return starts, and kept for the Gather.
  - Set Drop-off is refused for Agents.
  - "Operational" is `!is_transforming()` for now: **E must exclude sites**.
- **Measured.**
  - Trip times from 0 to 6 GU are within 15% of the table, and the 12.5 SC/s ceiling is exact.
  - Incomes are within 15% from 3 GU on.
  - **At 0–1 GU, three or more Agents jam head-on** (5.33 SC/s), and at 2 GU they reach 9.8–10.0. This is
    **Q0076, open**. It partly blocks L, and the first playable's layout.
- **Citations.** Its new code already cites 0282's paths.

#### What later slices need from C, D and I

These are for the Director to pass on, or to check at review. Neither running brief contradicts a later answer.
- **From C:**
  - **Held units count for Tunnel Space** through the iterator B leaves (`used`). Once B lands, Syndicate production
    waits on Tunnel Space (0143's shared check), so C's tests that train many units need room: 11 at the start.
  - **`Sim::take_off_map` and its counterpart for removal**, so that H can destroy held units for two rules:
    - stranded units;
    - 0258 point 5: "A defeated Syndicate player's units in the network are destroyed".
  - **Eject must not depend on a Tunnel's operation.** 0254: "Eject still works (it isn't an operation)". F and G add
    the operation slot after C.
  - **The "current tier" for entry and exit** must be one function. G keeps it at the pre-upgrade tier until
    completion (0249).
  - **`Place` matched without wildcards** (T30). E adds `Place::Site(id)`, which S1's Agent-built turret also uses.
  - **New code cites 0282's paths**, and new constants carry a citation or a placeholder note, so that R1 moves rather
    than rewrites them.
- **From D (merged):**
  - B's layout, I's Side D and L's oracle use `tunnel::hatch_square`.
  - E replaces "operational = `!is_transforming()`" with a check that excludes sites (E's spec).
  - R1 moves D's tunables (`RECHECK_TICKS`, `LEG_RETRY_TICKS`, in `orders/gather.rs`) to `tuning/gathering.rs`. D's
    per-object numbers are already def data (`LoadDef`, `drop_off`).
- **From I (running):**
  - The empty-station event and its client placeholder, which M's playtest notes list as a known placeholder.
  - Whatever I settles as a placeholder (the station's reach, when the old load is lost) goes to the Designer as
    Q0082.

#### R1 and R2: Code organization (Opus), new (0282)

**Goal.** The code is laid out the way the design now is, before any more Syndicate content lands:
- every unit and structure has its own def module with its design citation;
- its numbers sit in that one place;
- the tunables of each system sit in named tuning modules;
- each mechanic used twice is built once.

Nothing a player can see changes. Every test passes and **no golden changes**.

**Design and the user's decision**
- 0282, "From the user, for the code side": "keep the code **organized and modular** as well. Once testing starts we
  want to experiment and make changes rapidly and accurately, so it's worth investing time now to keep things in
  order. Examples: per-unit and per-structure definitions living in their own modules, data and tunables kept apart
  from logic, and shared mechanics (deploy model, toggles, AoE) implemented once and reused."
- 0282's impact: "Code comments and docs that cite design paths are now stale. I count about 320 citations across about
  150 files. Please update them using the table below. The section part after the path carries over unchanged, except
  that the entity name (and, for Syndicate structures, the grouping) is now the file." Its reference convention: "a
  path-less reference starting with an object name means that object's file".
- The user's decision, through the Director: "Modules now, data files at playtest".
  - **Now, in M4:** one Rust module per unit and per structure, with tunables in their own modules, kept apart from
    logic.
  - **At playtest**, after all four factions: hot-reloadable data files. Plan for it so that the later step is cheap:
    - defs are plain data structs, ready to derive serde, with no logic and no function pointers inside the data;
    - each object's numbers sit in one place;
    - tunables are named constants grouped by system, each citing the design or marked as a placeholder.
  - **Out of scope for M4:** the data-file and hot-reload layer itself. It is listed in §1.2 as a playtest-prep item.

**Survey** (read-only, at `172d496`, re-checked at `d399c44` after D's merge; B, C and I will add to it before M4-R
starts)
- **Stale citations.**
  - 352 lines in 151 files cite a faction's removed `units.md` or `structures.md` (216 in the sim, 112 in the client,
    the rest in `docs/` and `ARCHITECTURE.md`). At `172d496` there were 355:
    - `factions/colonists/units.md` 137, `factions/gdo/structures.md` 97, `factions/gdo/units.md` 60,
      `factions/colonists/structures.md` 27;
    - the Syndicate's two files 25, `factions/cults/units.md` 2, and 10 without a path.
  - `systems/units.md` (98 more) was not split and stays.
  - 1922 lines cite a design `path.md > Section` in all.
  - No test checks a citation today.
- **Defs.**
  - The defs are per faction:
    - `defs/gdo.rs`, 305 lines, 7 kinds;
    - `defs/colonists.rs`, 360 lines, 7 kinds;
    - `defs/syndicate.rs`, 232 lines, 4 kinds;
    - `defs.rs`, 278 lines: the registry, the Crystal Patch and the Supply Delivery Station.
  - Each kind is one `static ObjectDef`, with a nested `UnitDef` or `StructureDef`.
  - **No function pointers.** The def types carry accessors only (`TunnelDef::tier`, `TierDef::admits`,
    `ProductionDef::recipe`, `MovementAttrs::max_speed`). One table lives in code: `Transit::of(base)`.
  - **Not serde-ready.** Borrowed statics (`&'static str` names, `&'static [Recipe]`, `&'static [TierDef]`,
    `Option<&'static TunnelDef>`, `&'static [Transit]`, `&'static [SideLetter]`) can derive `Serialize`, but not
    `Deserialize`. Only `ObjectKind` derives serde today.
  - **`ObjectKind`, `ObjectKind::ALL` and `def()`** are three hand-kept lists, which a test keeps in step.
- **One object's numbers outside its def.**
  - `deploy.rs`: `DEPLOYMENT_FRAMES` (48, the Drone's) and `BEACON_DEPLOYMENT`.
  - `conversion.rs`: `CONVERT_TICKS` (4, the Legionnaires').
  - `economy/army.rs`: `BEACON_CAPACITY_CAP`, `BEACON_CAPACITY_PER_BUILDING`, and the GDO's `UNIT_CONTROL_CAP`. These
    are faction rules.
  - The Peacekeeper's `RIOT_SHIELD_TOGGLE_TICKS` is in `defs/gdo.rs`, but outside its def; 6 client lines use it.
  - Kind matches in logic: `tunnel.rs`, `scenario/settings.rs`, `combat/damage.rs`.
- **Tunables.** The sim's logic files hold 158 named constants, in 62 files.
  - **About 100 are behaviour numbers:**
    - movement and paths: `movement/collide/lanes.rs` 7, `mover/schedule.rs` 6, `nav.rs` 6, `mover.rs` 5,
      `mover/sidestep.rs` 4, `paths/checks.rs` 4, and others;
    - orders: `orders.rs` 7, and `orders/reach.rs` 7, `REJUDGE_MOVE_SU` among them; `orders/gather.rs`'s
      `LEG_RETRY_TICKS`;
    - combat: `MINIMUM_DAMAGE`, `DIRECTIONAL_*`, `FACING_TOLERANCE`;
    - production: `QUEUE_LIMIT`;
    - vision: `FOG_DELAY_TICKS`, `REVEAL_TICKS`;
    - shields: `SHIELD_REGEN_PER_TICK`;
    - transforms: `MIN_HP_AFTER_CANCEL`.
  - **The rest are technical:** units of measure (`TICKS_PER_SECOND`, `SU_PER_GU`, `RATIO_ONE`), hashing, the RNG,
    map encoding and limits, symmetry tables, file formats, and test fixtures.
  - The client has 365 constants, 105 of them colours. It uses about 16 of the sim's constants.
- **Mechanics built more than once.**
  - **The toggle, twice.** `riot_shield.rs` (272 lines; `RiotShield { up }`) and `conversion.rs` (237;
    `Conversion { form, left }`) both implement 0198's "a toggle sets a target form". Both run in `sim/modes.rs`, and
    both change kind through `Sim::change_kind`.
  - **Kind changes over time.** `transform.rs` (Warp In and Beacon Deployment: cancellable, refunded, the target kind
    from the first frame), the Drone's Deploy order, the two toggles, and T44's deploy to come. Their rules differ
    (T44's text). They share `Sim::change_kind`, and should share how they time.
  - **Timed work, in four shapes:** `Transform { progress, total }`, `Production { progress }`, `Conversion { left }`
    and `RiotShield { up }`, plus the worker's job timers. M4 adds six more: C's transit, E's site, F's and G's
    operation, H's decay, I's pick-up, and S2's rise and retract.
  - **Worker-built placement.** The Drone's deploy ghost (M3-F) becomes E's kind-generic Build order (T51), not a
    second one.
  - **AoE.** Not built yet. O builds it once (T46).
- **Rebuild time today.** Read from `target/debug/build/*/fingerprint/invoked.timestamp` against each output's mtime.
  Nothing was built for this.
  - **The last incremental build after a sim change** (2026-11-23 13:32):
    - `sc_sim`'s lib took 1 s;
    - the client lib (`space_crystals`) took 43 s;
    - linking the game took 5 s;
    - the test binaries took 6 s (the sim's), 13 s (the client lib's) and 8–9 s (the client's integration tests).
  - **So a number change in the sim costs:**
    - about **50 s before `cargo run` starts**;
    - about **65 s for `cargo test`**;
    - about **10 s for `cargo test -p sc_sim`**.
  - **A client-only change** (12:24) took about 8 s.
  - **Caveats.** These are single samples at 1 s resolution, and what changed in those builds is unknown.
  - **What changes this.** Per-object modules won't: any sim change recompiles the client crate. The data files at
    playtest remove the rebuild for numbers. Until then, tune with `cargo test -p sc_sim`.

**What R1 does** (Opus; everything under `crates/sim/`)
1. **Defs per object** (T54).
   - One module per unit and per structure, mirroring the design's files and named like them:
     - `defs/gdo/units/peacekeeper.rs`, which holds both of the Peacekeeper's types;
     - `defs/gdo/structures/{deployment_center, generator, barracks, extraction_facility, extraction_plate}.rs`;
     - `defs/colonists/units/{drone, sabre_legionnaire, rifle_legionnaire}.rs`;
     - `defs/colonists/structures/{beacon, egress_node, robo_portal, bio_portal}.rs`;
     - `defs/syndicate/units/{agent, guard}.rs`;
     - `defs/syndicate/structures/{tunnel, hideout}.rs`;
     - `defs/map/{crystal_patch, supply_delivery_station}.rs`.
   - Each module's doc comment cites its design file, for example `factions/syndicate/units/agent.md`. The economy
     buildings cite `factions/gdo/economy.md` and `factions/colonists/economy.md`.
   - Each faction's root module, `defs/<faction>.rs`, holds that faction's own rules, each cited:
     - the GDO's Unit Control cap;
     - the Colonists' Beacon capacity;
     - the Syndicate's Tunnel Space cap and divisor (B's), and the ExpandMenu's ordered list (F's).
   - An object's tests move into its module. Tests across objects stay in `defs/tests.rs`.
2. **One object's numbers in one place.**
   - Move `DEPLOYMENT_FRAMES`, `BEACON_DEPLOYMENT`, `CONVERT_TICKS` and `RIOT_SHIELD_TOGGLE_TICKS` into their objects'
     defs, as fields.
   - Move C's and I's per-object numbers there as well: the transit delay and regeneration go in the Tunnel's def,
     and the Supplies numbers in the Agent's `LoadDef`. D's are def data already (`LoadDef`, `drop_off`); its
     `RECHECK_TICKS` and `LEG_RETRY_TICKS` go to `tuning/gathering.rs`.
   - Replace the kind matches in logic with def fields, where a field says it better.
3. **Defs as plain data.**
   - Borrowed statics become `Cow<'static, …>`: `Cow::Borrowed` works in a `static`.
   - `Option<&'static TunnelDef>` becomes an inline value.
   - `Transit::of(base)` becomes data.
   - Every def type derives `Serialize` and `Deserialize`.
   - A new test, `defs/tests/serde.rs`, round-trips every kind's def through JSON and compares. That proves the data
     can be loaded later without a refactor.
   - Accessor methods may stay; logic may not move into the defs.
4. **The kind registry.**
   - One `macro_rules!` list in `defs.rs` generates `ObjectKind`, `ALL` and `def()`, so a new kind adds one line there.
   - The serialized names and the order stay, so maps and replays are unchanged. New kinds go last, as m4-common says.
5. **Tunables.**
   - The behaviour constants move to `tuning/<system>.rs`: `movement`, `paths`, `orders`, `gathering`,
     `construction`, `combat`, `production`, `vision`, `shields`, `tunnel`.
   - Each constant keeps its name and its value, and gets a doc comment: a design citation, or "Placeholder (not in the
     design)" with why it has this value.
   - Technical constants stay where they are.
   - The constants grouped by system map one-to-one onto a future loaded struct per system.
6. **Shared mechanics**, only where the hashed state stays byte-identical:
   - a `Progress` value type (`progress.rs`): ticks done of a total, a 2^16 fixed-point fraction, `step` and `done`.
     The existing timers use it where their hashed bytes don't change. C's and the new slices' timers use it.
   - one toggle rule (`toggle.rs`), "a toggle sets a target form" (0198), which `riot_shield.rs` and `conversion.rs`
     both call, each keeping its own `Object` field and its hash.

   If a mechanic can't be shared without changing a hash, R1 stops there and reports it. It is then shared at the next
   re-bless that touches it (T54). Area damage (O) and the deploy model (T44) are built shared from the start.
7. **Splits.** `orders.rs` (495 lines) and `combat/phases.rs` (479), with no behaviour change.
8. **Citations in the sim**, rewritten with 0282's table, by a script R1 writes and runs once. The 10 path-less ones
   are done by hand.

**What R2 does** (Opus; `crates/game/**`, `docs/**`, `ARCHITECTURE.md`)
1. **The citation test**, a new `crates/sim/tests/design_citations.rs`: in the sim crate, so that it runs in the
   fast `cargo test -p sc_sim` loop, and needs no Bevy. It is R2's one file under `crates/sim/`.
   - It scans `crates/**/*.rs`, `docs/**/*.md` and `ARCHITECTURE.md` for backticked design paths:
     `core/`, `systems/`, `factions/`, `overview.md` and the rest of the design's top level.
   - It fails, naming file and line, if `artifacts/designer/design/<path>` doesn't exist. `docs/` paths are checked
     against the repository's `docs/`.
   - A second, ignored test reports cited sections that aren't headings of the file. It is a report, not a gate: the
     design names many sections by bold paragraphs.
2. **Citations** in the client, `docs/` and `ARCHITECTURE.md`, by the same table. R2 merges after R1, and its test then
   passes on the whole repository.
3. **One client module per kind**, `kinds/<faction>/<object>.rs`, behind one registry.
   - Each holds what the client knows of that kind: its label and abbreviation, its `ShotStyle`, its render style, and
     its panel template, moved to `command_ui/interface/templates/<faction>/<object>.rs`.
   - A new kind then adds one registry line and its own files.
   - Drawing code stays in `object_render/`.
4. **Splits**: `command_ui/interface/templates.rs`, `hud/selection_panel.rs`, `command_ui/indicators.rs`,
   `object_render.rs` and `combat_effects.rs`.
5. **The client's feel tunables**, in `crates/game/src/tuning/`: camera, input and feedback timings, cited or marked
   as placeholders. Colours and drawing sizes stay with the drawing code.
6. **`ARCHITECTURE.md`**: "Adding a unit or structure" (its def module, its client module, the registry lines, where
   numbers go), and the tuning modules.

**Acceptance (both)**
- `cargo test`, clippy and the ignored suite pass, with **every golden byte-identical** and no hash change. A golden that
  moves means behaviour moved: fix it, don't re-bless.
- The citation test passes, and fails on a planted bad path.
- The serde round-trip passes for every kind.
- The 0282 survey's stale citations count is 0.
- No logic file keeps a behaviour constant without a citation or a placeholder note.
- `ObjectKind`'s serialized names and order are unchanged.
- Perf: `nav_perf` and `drones_perf` by interleaved A/B runs, unchanged within noise.
- The diff report lists:
  - every constant moved, with its old file and new module;
  - every per-object number moved into a def;
  - every mechanic shared or deferred, with the reason.

**Size and risk**
- **Size.** R1 is M4's widest slice: about 100 sim files, most of them for comments and constant moves. R2 touches
  about 60 client and doc files. Both are mechanical. R1's riskiest parts are the `Cow` change and the shared toggle.
- **Conflicts.**
  - With B, C, D and I: none, since M4-R starts after they merge.
  - With later slices: none, since nothing runs beside M4-R. Briefs written before M4-R lands must use the new paths
    (§8).
  - Between R1 and R2: only the ~16 client `use` lines, which are R1's.
- **Model: Opus** for both. This is no movement or collision work. The movement constants move unchanged, and the
  goldens prove it.
- **Alternatives** (T54):
  - one slice instead of two: no merge order to manage, but about twice as long on the critical path;
  - three slices (defs; tunables and mechanics; client): faster, but the first two both edit the sim files whose
    per-object constants move into defs;
  - data files now: rejected by the user.

**Owns**: see §3.2. **After**: B, C and I (D has landed). **Model**: Opus.

#### E: Agent-built Tunnels (Opus)

**Goal.** An Agent builds a Tunnel.
- **Changed (0233).** The order pays the Supplies at once and leaves a lingering ghost. The Agent walks to the site.
- On arrival the site appears with the Agent inside, at 10% HP, and grows to full over 480 ticks. Then the Agent is in
  the network.
- **Changed (0233).** These refund the current top price: cancelling the site, and an order that is cancelled,
  replaced or fails before construction starts. Destruction loses the Supplies.
- **Changed (0237, 0242).** The price counts ghosts and sites. The area overlaps no player's Tunnel Area, and lies
  wholly on the map, on Drillable cells.

**Design**
- `units/agent.md > Building > Tunnel Construction Flow`:
  1. "Agent receives build command and walks to the target location";
  2. "Construction begins — the partially-built Tunnel appears at the location with the Agent inside it. The Tunnel
     starts at 10% HP (ConstructionHP Rule). The Agent is untargetable for the duration of construction.";
  3. "**If construction completes**: The Tunnel becomes operational and the Agent is inside the Tunnel Network,
     available for redeployment from any Tunnel.";
  4. "**If the partially-built Tunnel is destroyed**: The Agent survives and emerges at the location. The Tunnel is
     lost and any Supplies spent are lost."

  Step 5's "100% of the Supplies cost is refunded", and `> Cancel Construction`'s "Full refund of Supplies cost", are
  stale against the top-price rule below (Q0085). Build the top price.
- `> Building`: "Only one Agent may construct a given Tunnel — multiple Agents cannot speed up construction."
- `> While the Tunnel is under construction` (0234):
  - "It **does nothing but Cancel**: no Tunnel Space, no drop-offs, no Enter, no Expand, no Eject; the only command
    is X Cancel Construction";
  - "**HP**: the ConstructionHP formula gives the **maximum** HP; damage taken stays (current HP rises by the same
    amount each frame, like the Colonists' Warp In)";
  - "**Where the Agent comes out** on cancel or destruction: at the **centre of the footprint**, which is now free; it
    then idles";
  - "**Defeat**: the site counts as a structure ... Cancelling it when it is the player's only structure defeats them
    in that tick";
  - "**A carried load**: an Agent ordered to build while carrying crystals or Supplies **keeps its load**, in the site
    and in the network";
  - "**What enemies see**: the site, like any surface structure, but not the Agent inside it."
- `tunnels.md > Cost Scaling > Construction Cost` (0233):
  - "Cost = current Tunnels owned (all tiers count), in Supplies";
  - "**Paid when the ghost is placed** ...: an Agent's build order pays the Supplies **when it is given** ... not on
    arrival";
  - "**Lingering Tunnel ghosts count as Tunnels owned** for the price of further orders, so two orders given in a row
    cost 1 and 2 (likewise a Tunnel under construction counts)";
  - "The order is refused, with feedback, if the player can't pay. If the order is cancelled, replaced, or fails
    before construction starts, it is refunded at the current top price".
- `> Upgrade Costs`: "**Tunnel construction refunds the same way** (user, 2026-11-25): the current construction price,
  not counting itself." And: "The Supplies paid for what the player still has always equal the formula's total, so the
  order of starting and cancelling can't gain or lose Supplies."
- `> Construction Time`: "480 frames (30 seconds). Agent must be present for the duration."
- `tunnels.md > Tunnel Area Non-Overlap Rule`:
  - "The non-overlap check uses the **current tier's area**";
  - **Every player's Tunnels count** (0237): "**The ghost** judges enemy Tunnels by what the player knows: their
    remembered position and their tier as last seen ... The real check on arrival uses the truth, so a build can fail
    on arrival because an enemy Tunnel was upgraded unseen";
  - **Drillable area** (0242): "a Tunnel can only be **placed** if every cell of its Tunnel Area is **on the map and
    Drillable** ... The ghost shows this like any other invalid placement (terrain is always known to the player as
    far as explored; never-explored tiles pass on the ghost, and the real check happens on arrival)".
- `structures/README.md > Tunnel Expansions` (0240): "**Surface buildings over a hidden expansion**: an enemy's placement is
  refused. A **direct placement** (e.g. a GDO building) shows its ghost **red at once** ...; this is accepted ... A
  **worker-built** building's ghost judges by what the player knows (so it stays green), and the order **fails when the
  worker arrives**".
- `core/entities.md > ConstructionHP Rule`, `> Worker-Built Structures` and `> Lingering build ghosts`, as adopted.
- `tunnels.md > Losing Tunnels > Defeat`: "with no Tunnels owned, a new Tunnel costs 0 Supplies".

**What changed from the plan**
1. **Changed (0233).**
   - Payment happens when the order is accepted, not on arrival.
   - The price counts the owner's Tunnels, sites and lingering Tunnel ghosts.
   - An order that can't be paid for is refused with feedback.
   - Cancel, replace, failure before the start, and Cancel Construction all refund the **current top price**: the
     count of the others.
   - The adopted "on arrival, a price the player can't pay fails the order" is gone.
2. **Changed (0237).** Non-overlap is against **every player's** Tunnels. The ghost judges by knowledge (remembered
   position, last-seen tier), and arrival by the truth.
3. **Changed (0242).** The area's drillability and bounds: B's rule in `placement::check`. In `Mode::Known` only
   explored cells count; on arrival, the truth.
4. **Changed (0240).** A Tunnel's footprint over any underground structure fails on arrival, by the truth. Its ghost
   judges by knowledge, so it stays green over a hidden one. Surface placement already counts underground structures
   as occupants (§1.3), so for the GDO and the Colonists 0240 needs only its tests.
5. **Changed (0234).** The site's rules are as proposed. ConstructionHP is a maximum, and damage is kept.
6. **Changed (T51).** The order is kind-generic, `OrderKind::Build { kind, origin, orientation }`. The Tunnel is its
   only kind in E; S1 adds the Laser Turret.
7. **Placeholder (Q0079).**
   - An Agent killed before construction starts refunds the top price, as a failed order.
   - A lingering Tunnel ghost reserves its whole Tier 1 area against its owner's further Tunnel placements, not just
     its footprint.
   - A site's area counts for non-overlap like any Tunnel's.

**Code**
- **Where it goes** (T54). R1 has split `orders.rs`, so the order is a new `orders/build.rs`. The Tunnel's numbers
  (the price rule, the 480 frames, ConstructionHP's 10%) go in `defs/syndicate/structures/tunnel.rs`, each citing its
  design line.
- **The order** (T51). `OrderKind::Build { kind, origin, orientation }`, in the Deploy pattern:
  - the gate checks in knowledge mode, with the price;
  - the command's application pays the price;
  - the arrival step, `step_deployments`, generalises to every worker build;
  - a failure emits `SimEvent::BuildFailed` and refunds;
  - `Sim::pending_deploys` generalises to every worker build's lingering ghost.
- **The price.** `tunnel_price(player)` = the owner's Tunnels, sites included, plus their lingering Tunnel ghosts. A
  refund is that count less one, which is "not counting itself". One function serves the gate, the client's tooltip and
  every refund.
- **Placement.** `Method::WorkerBuilt` for the Tunnel, plus the faction rules:
  - its Tier 1 area overlaps no current area of any player's Tunnel, sites included: `WholeProblem::TunnelAreaOverlap`.
    The ghost uses remembered Tunnels at their last-seen tier, and arrival the truth;
  - its area overlaps no area of the owner's pending Tunnel ghosts (placeholder);
  - B's area rule (`TunnelAreaOffMap`, `TunnelAreaNotDrillable`), inherited from `check`.

  No new `TileProblem` is needed.
- **`Construction`** (T31). `Object::construction: Option<Construction { progress, builder }>`, hashed only while set:
  - the progress, 2^16 fixed point, over 480 frames;
  - `Builder::Held(agent)`: the Agent, held at `Place::Site(site)`.

  `Object::max_hp()` gives ConstructionHP's maximum, with damage kept. `step_sites` runs in the economy phase, beside
  `step_transforms`.
- **Completion.**
  - The Tunnel becomes operational: an exit, a drop-off, an Enter and Eject target. **D's "operational" is
    `!is_transforming()` today, so E makes it exclude sites**, for drop-offs (D's nearest Tunnel), exits and Enter.
  - E adds the "not a site" term to B's `gives_tunnel_space`.
  - The Agent moves from `Place::Site` to `Place::Network`, keeping its load.
- **Cancel and destruction.**
  - `CancelConstruction` on the site refunds the top price, removes the site, and puts the Agent back on the map at
    the footprint's centre, idle, with its load.
  - On destruction the Agent comes out the same way and the Supplies are lost. This is the removal path's rule for
    the units held at a removed object, with one arm per `Place` (T30).
- **Defeat.** A site counts as a structure.

**Acceptance**
- **The flow.**
  - The order is given and 1 Supply is paid at once; the ghost appears.
  - On arrival the site has a maximum of 60 HP (10% of 600). After 480 ticks it is at 600, with damage kept.
  - The Agent is then in the network, and is ejected.
- **0233's verification**: "With 3 Supplies and 1 Tunnel, order two Tunnels with Shift. Both ghosts appear and 1 + 2
  Supplies are deducted at once. Cancel one order: 2 come back." With two Agents: "each Agent takes exactly one
  placement".
- **Refusal.** With 0 Supplies and 1 Tunnel, the order is refused with feedback, and nothing changes.
- **Refunds.**
  - Cancelling a site, replacing an order with a Move, and an order failing on arrival each refund the top price.
  - A destroyed site refunds nothing.
  - **The books:** after any sequence of orders and cancels with no destruction, Supplies spent less refunded equals the
    formula's total.
- **Cost.** With no Tunnels, the next costs 0.
- **Overlap (0237).**
  - An area overlapping an own Tunnel's, or a known enemy Tunnel's at its last-seen tier, is red on the ghost and
    refused on arrival.
  - An enemy Tunnel upgraded unseen fails the build on arrival.
- **Drillable (0242's verification).**
  - "A Tunnel ghost whose 8x8 area touches Water is red."
  - A Tunnel 1 tile from the map edge is red. One 2 tiles away is valid (Q0078).
  - Water on an unexplored tile passes on the ghost, and fails on arrival.
- **0240's verification:**
  - "A GDO player's ghost over a hidden Syndicate Hideout is red";
  - "A Colonists Drone ordered to deploy there fails on arrival";
  - an Agent's Tunnel ghost over an enemy's hidden Hideout is green, and fails on arrival.
- **Failure.** A unit standing on the site at arrival fails the order: the ghost goes, the Agent idles, and the price
  is refunded.
- **The site.**
  - It offers only Cancel.
  - The Agent can't be targeted, since it is off the map; enemies hit the site.
  - The Agent keeps its load.
  - Enemies see the site.
- **The lingering ghost** is shown to its owner only, and reserves the site and, as a placeholder, its area.
- **Defeat.** Cancelling the player's only structure defeats them in that tick.
- **QA screenshots**: the site at 10%, and the finished Tunnel.
- **Goldens.** `Object::construction` is hashed only while set, so no golden changes.

**Owns**: see §3.2. **After**: C (the held builder) and R1. **Model**: Opus.

#### F: Expansions (Opus)

**Goal.** A Tunnel builds expansions in its Tunnel Area.
- **W** opens the ExpandMenu (**Changed, 0257**), and an entry leads to a ghost within the area.
- Placing pays, starts the construction and uses the Tunnel Space at once (**Changed, 0254**). The construction takes
  the Tunnel's one operation.
- On completion, the expansion works.

**Design**
- `tunnels.md > ExpandMenu`:
  - "An expansion is available only if (1) its Tier Requirement is at or below **this** Tunnel's tier, and (2) its
    TechPrerequisite (if any) is satisfied anywhere in the Tunnel Network";
  - "Click an expansion type: begins construction if the Tunnel is not already performing an operation (enters
    AwaitingPlacement for that expansion)";
  - "The ExpandMenu also offers the **Laser Turret** (once the player owns an Optics Lab), placed within this Tunnel's
    Tunnel Area like an expansion" (S2).
- `> Tunnel ObjectInterfaceState > Panel letters` (0257): "**W** Expand Tunnel ... In the **ExpandMenu** and
  **EjectMenu**, entries take the grid slots in order (Q, W, E, R, A, S, …): expansions in the Tech Tree table's order
  (Hideout, Optics Lab, Chemical Plant, Vault, …) ...; **Z** goes back; unavailable entries are greyed."
- `> AwaitingPlacement (Expansion)` (0239, 0240):
  - "The expansion must fit entirely within the Tunnel Area";
  - "**Every tile under it must be Drillable** (... Plane, Rugged and Cliff yes; Mountain and Water no ...)";
  - "Own and neutral underground structures block it; so does the Tunnel's own footprint";
  - "**Surface structures block it too** ...: no expansion can be placed under any surface building, any player's
    (judged by the truth; the placement is refused)";
  - R, Shift+R, F and Shift+F turn it, and a left-click places it.
- `structures/README.md > Tunnel Expansions` (0241): "**No expansions under resources** ...: a Space Crystals patch or Supply
  Delivery Station above blocks an expansion".
- `tunnels.md > Losing Tunnels` (0258): a neutral expansion partly inside "still **blocks its cells**: nothing can be
  built there".
- `> Expansions under construction` (0254):
  - "**Paid when the ghost is placed** ... **Tunnel Space is used once construction has started.** Today placing the
    ghost starts construction at once";
  - "**HP**: the ConstructionHP Rule applies to every expansion (starts at 10%, grows with progress); damage taken
    stays";
  - "**Cancelling**: while building, the expansion offers only **X Cancel Construction**, refunding **75%** of its
    cost (rounded down). The Tunnel's X cancels whatever operation it is running (an upgrade or an expansion)";
  - "**Sight**: none (underground). **Groupable**: true";
  - "**Its Tunnel destroyed mid-build**: the unfinished expansion is destroyed, no refund" (H's rule);
  - "**The Tunnel meanwhile** shows the progress; Eject still works (it isn't an operation)";
  - "**Z**: returns to ExpandMenu".
- `> Construction and Upgrade Rules`: "A Tunnel can only perform one operation at a time". `structures/laser_turret.md > Construction` (0267): a turret built by a Tunnel "**uses the Tunnel's single operation slot**".
- `structures/README.md > Expansion Requirements`: the Tier Requirement is checked against the building Tunnel, the
  TechPrerequisite across the network.
- `core/entities.md > Direct Placement` "(e.g., GDO buildings, Tunnel underground expansions)": "**No ground units on
  the footprint** ...: a ground unit of any player ... standing on any footprint tile makes those tiles invalid ... Air
  and underground units never block placement."
- `structures/hideout.md`: "Cost - 200 Space Crystals", "Build Time - 400 frames (25 seconds)".

**What changed from the plan**
1. **Changed (0239).** The tiles must be Drillable, not Buildable.
2. **Changed (0240).** Any player's surface structures block the expansion, judged by the truth.
3. **Changed (0241).** Resources block it.
4. **Changed (0254).**
   - ConstructionHP applies to every expansion, with damage kept.
   - X refunds **75%**, rounded down, which replaces the proposed full refund.
   - The Tunnel's X cancels its current operation.
   - No sight, and Groupable. Eject works meanwhile.
   - The unfinished expansion dies with its Tunnel (H).
5. **Changed (0257).** W opens the menu, its entries take grid order, Z goes back, and unavailable entries are greyed.
6. **Changed (0267).** The operation slot covers expansions, upgrades (G) and turrets (S2).
7. **Placeholder (Q0080).** 0240 replaced the adopted proposal 3c ("surface structures, units and
   lingering ghosts don't block"). Until it is answered, follow `core/entities.md > Direct Placement` literally:
   - **ground units on the footprint block** an expansion, which replaces the adopted acceptance "over a ground unit
     (allowed)";
   - so do the owner's lingering ghosts.
8. **Placeholder (Q0081).** Every expansion of the Tech Tree table holds its slot, in table order, greyed while
   unavailable (tier, prerequisite, or not in this build). The Laser Turret takes the slot after them (S2).

**Code**
- **`Method::Expansion { tunnel }`** (T32, T53):
  - inside the Tunnel's current area (B's `tunnel/area.rs`), and off its footprint;
  - **Drillable tiles**: a new `TileProblem::NotDrillable`, so widen the bitset to u16 first (T53);
  - any structure in either layer, by the truth → `Structure`;
  - resources → `Resource`;
  - ground units → `Unit` (placeholder);
  - the owner's lingering ghosts → `Reserved` (placeholder);
  - no flatness and no visibility.

  B's layout rule switches over to this method.
- **Numbers** (T54). The 75% refund is `tuning/construction.rs`'s `EXPANSION_CANCEL_REFUND`, citing 0254. Each
  expansion's cost and time stay in its own def module.
- **The operation slot.** `TunnelState::operation: Option<Operation>`, hashed only while set, with its progress in
  R1's shared progress type. `Operation::Building
  (site)` here; G adds `Upgrading`; S2 builds a turret through `Building(site)`.
- **`TunnelCommand::Expand { kind, origin, orientation }`.**
  - The gate checks: the Tunnel is operational with no operation; the tier; the prerequisites (K's check, a stub that
    passes until K); the SC.
  - It pays at placement.
  - It creates the expansion with a `Construction` whose builder is `Builder::Tunnel(id)`.
- **Cancelling.**
  - `TunnelCommand::Cancel` cancels the current operation: a site refunds 75% rounded down; G adds the upgrade arm.
  - The expansion's own `CancelConstruction` (E's command) refunds 75% too.
- **Tunnel Space** counts the expansion from its creation. B's occupancy already counts underground structures, so
  nothing is added here.
- **Production.** A finished Hideout produces through C's release.
- **The menu's data.** The ExpandMenu's entries come from an ordered list in the sim's defs (the Tech Tree's order),
  so that K and S2 add entries without client edits.
- **Client.**
  - The W entry on J2's Tunnel template.
  - The ExpandMenu, in grid order, greyed with reasons, Z back.
  - A `Builder::Tunnel(id)` ghost held within the area, with the area overlay, R and F.
  - A progress bar on the site; the Tunnel's panel shows the operation's progress.

**Acceptance**
- **Placement:**
  - inside the area, and outside it;
  - on the Tunnel's footprint; over an own or a neutral expansion; on a patch or a station;
  - on a slope (allowed);
  - **on Water inside the area: red** (0239's verification);
  - **under an enemy's surface building: refused** (0240's verification: "A Syndicate player can't place an expansion
    under an enemy's building");
  - under a ground unit: refused (placeholder);
  - refused while the Tunnel builds or upgrades.
- **Tunnel Space.** From the start's 1/11, a second Hideout gives 1/10 at the click.
- **Construction.** 400 ticks, HP rising from a maximum of 40 to 400, with damage kept. Then the second Hideout trains
  Agents into the network.
- **Cancel** (0254's verification): "Place a Hideout (200 SC), then cancel: 150 SC come back."
  - The same through the Tunnel's X.
  - A test kind costing 150 refunds 112.
  - The operation is freed.
- **Eject during construction** works.
- **Hidden.** The enemy never sees the new Hideout or its ghost.
- **Tier Requirement.** A test kind with Tier Requirement 2 is refused at a Tier 1 Tunnel, and greyed in the menu.
- **The menu.** Letters and order as 0257 says.
- **QA screenshots**: the ExpandMenu; the ghost red and green; the area overlay.
- **Goldens.** No change.

**Owns**: see §3.2. **After**: B, E and J2. **Model**: Opus.

#### G: Tunnel upgrades (Opus)

**Goal.** A Tunnel upgrades to Tier 2, then Tier 3, for Supplies paid when ordered.
- The upgrade takes 480 frames (**Changed, 0229**) and is the Tunnel's one operation.
- On completion its HP, area, Tunnel Space and transit grow, and current HP rises with max HP.

**Design**
- `tunnels.md > Tunnel ObjectInterfaceState > Panel letters` (0257): "**Q** Upgrade Tunnel ... **X** Cancel
  (upgrade, construction or the current operation)". The older "A: Upgrade Tunnel — ... Unavailable if already Tier 3
  or if currently performing an operation" still gives the availability.
- `> Upgrade Costs`:
  - "Higher-tier Tunnels count toward lower-tier cost calculations";
  - "**Upgrade duration** (placeholder ...): **480 frames (30 seconds)** to Tier 2 and to Tier 3";
  - "**Paid when the upgrade is ordered**";
  - "**Upgrade to Tier 2**: 2 + 2 x (number of T2+ Tunnels owned)"; "**Upgrade to Tier 3**: 3 + 3 x (number of T3
    Tunnels owned)";
  - "**Upgrades in progress count** ...: a Tunnel upgrading to Tier 2 already counts as a T2+ Tunnel (and one
    upgrading to Tier 3 as a T3 Tunnel) for the cost of further upgrades. Two upgrades started together cost 2 and 4,
    not 2 each";
  - "**Refunds are the current top price** ...: Cancel Upgrade refunds **the price that upgrade would cost if ordered
    right now** (not counting itself) ... Example: A and B upgrade to Tier 2 together (paying 2 and 4). Cancelling
    either one first refunds **4**; cancelling the other afterwards refunds **2**";
  - "**HP on completion** ...: max HP rises (600 → 800 → 1000) and current HP rises by the same amount";
  - "**Destroyed mid-upgrade** ...: the Supplies paid for the upgrade are lost";
  - "**The future area is reserved** ...: from the start of the upgrade, the larger Tunnel Area of the target tier
    counts against other placements".
- `> Tunnel Area Non-Overlap Rule`:
  - "upgrading a Tunnel is blocked if the radius increase would cause its Tunnel Area to overlap with another Tunnel's
    current area";
  - (0237) "**Upgrades** are blocked by any Tunnel's area, own or enemy (the upgrade check uses the truth; the
    upgrading Tunnel's owner learns that something blocks it)";
  - (0242) "An **upgrade** is likewise blocked if the larger area would include a non-Drillable or off-map cell, so a
    Tier 1 Tunnel by a lake or a mountain may never be upgradable".
- `tunnels.md > Tunnel Network` (0249): "until the upgrade has fully finished, a Tunnel only accepts, and only lets
  out, units supported by the tier it had **before** the upgrade began".
- `> Losing Tunnels` (0258): "**Reclaiming happens** the moment the covering Tunnel, or its upgrade, **finishes**".
- `structures/laser_turret.md > Conversion`: "A new Tunnel, or a Tunnel upgrade, whose Tunnel Area now covers an
  upright turret converts it into a pop-up turret" (S2).

**What changed from the plan** (Q0060 is answered; G is unblocked)
- **0229:** 480 frames, paid when ordered.
- **0231:** in-progress upgrades count; HP rises with max HP; Supplies lost mid-upgrade; the reserved area.
- **0232:** Cancel refunds the current top price, replacing "Full refund".
- **0237:** any Tunnel's area blocks, by the truth.
- **0242:** non-Drillable and off-map cells block.
- **0257, 0254:** Q, and the shared X.

**Code**
- `TunnelCommand::Upgrade`, and the upgrade arm of F's `TunnelCommand::Cancel`.
- `Operation::Upgrading { progress, to }`. The 480 frames (a placeholder citing 0229) and the two price rules go in
  the Tunnel's tier data in `defs/syndicate/structures/tunnel.rs`, next to each tier's HP and area.
- **The price.** One function: 2 + 2 × (the owner's other Tunnels at T2+ or upgrading to T2 or T3), or 3 + 3 × (the
  other Tunnels at T3 or upgrading to it). The order and every refund use it, "not counting itself".
- **The checks, by the truth:**
  - tier below 3, no operation, the Supplies;
  - the target tier's area overlaps no area of any player's Tunnel, including other Tunnels' reserved upgrade areas;
  - B's area rule at the target tier.

  A refusal gives feedback ("something blocks it").
- **The reserved area.** While upgrading, other placements and upgrades check against the target tier's area. Transit,
  Tunnel Space, exits and reclaiming keep the current tier until completion (0249).
- **Completion.** `tunnel::set_tier` (A's, damage kept), then H's `reclaim`. S2's conversion is derived and needs no
  call.
- **Client.** Q enabled (J2 showed it greyed), the progress on the panel, X cancelling.

**Acceptance**
- **Costs.**
  - The first Tier 2 costs 2, the second 4; Tier 3 costs 3, then 6.
  - A Tier 3 Tunnel counts in the Tier 2 price.
  - Two upgrades started together cost 2 and 4.
- **Refunds** (0232's verification): "A and B upgrade together. Cancel B: refund 4. Then cancel A: refund 2", and in
  the other order. The books hold in any order of starts and cancels.
- **Blocked:**
  - by an own neighbour within the Tier 2 area;
  - by an enemy's (0237's verification: "place a Tier 1 Tunnel 1 tile from where an enemy Tier 1 Tunnel's Tier 2
    area would reach. The enemy can't upgrade that Tunnel");
  - by Mountain in the Tier 2 area (0242's verification: "unavailable, with feedback");
  - by the map edge;
  - while busy, and at Tier 3.
- **The reserved area.** During an upgrade, a Tunnel ghost overlapping the Tier 2 area is red, and refused.
- **During the upgrade:**
  - transit is at the old tier (0249's verification, with a test vehicle);
  - cancelling refunds;
  - destruction loses the Supplies.
- **Completion:**
  - 480 ticks (0229's "A Tunnel upgrade takes 30 s");
  - 500/600 becomes 700/800 (0231's verification);
  - Tunnel Space +9;
  - a neutral expansion now fully covered is reclaimed.
- **QA screenshot**: a Tier 2 Tunnel and its area.

**Owns**: see §3.2. **After**: F (the operation) and H (`reclaim`). **Model**: Opus.

#### H: Losing Tunnels (Opus)

**Goal.** When a Tunnel is destroyed:
- its expansions turn neutral, visible to everyone, and decay;
- their work in progress is lost and their queued work refunded;
- its unfinished expansions are destroyed (**Changed, 0254**);
- network units that can no longer get out are destroyed.

A Tunnel or upgrade that finishes reclaims the neutral expansions it fully covers. Defeat follows the shared rule, and
a defeated player's network units are destroyed (**Changed, 0258**).

**Design**
- `tunnels.md > Losing Tunnels`, as 0258 left it:
  - "**Expansions become neutral**: when a Tunnel is destroyed, every expansion in its Tunnel Area becomes neutral";
  - "Neutral expansions are **no longer hidden** (visible to everyone, no detection needed)";
  - "They **slowly lose HP** until destroyed. **Decay is a percentage of MaxHP per second** ...: **1% per second,
    placeholder** (a Hideout lasts 100 s)";
  - "**Attacking them** ...: only with an **explicit Attack** order; right-click never attacks a neutral object and
    auto-targeting ignores them";
  - "**Reclaiming happens** the moment the covering Tunnel, or its upgrade, **finishes**; a Tunnel under construction
    reclaims nothing";
  - "**Partly inside an own Tunnel Area** ...: until reclaimed, a neutral expansion still **blocks its cells** ... and
    they **count as occupied** for that Tunnel's Tunnel Space" (B and F);
  - "**Reclaiming**: if a Tunnel is built whose Tunnel Area fully covers a neutral expansion, and that Tunnel's tier
    meets the expansion's Tier Requirement, the expansion becomes owned by that Tunnel's owner ... and stops decaying";
  - "**In-progress work is destroyed**"; "**Queued work is refunded**";
  - "**Laser Turrets stay owned**: a destroyed Tunnel's pop-up Laser Turrets become upright turrets" (S2: derived);
  - "**Defeat** ...: Losing the last Tunnel therefore defeats the Syndicate only if they own no Laser Turrets";
  - "**Stranded units**: ... any unit inside the Tunnel Network that can no longer exit through any remaining Tunnel
    ... is **destroyed**";
  - "A Tunnel that is still under construction does not count as an exit, and a Tunnel that is mid-upgrade counts at
    its current (pre-upgrade) tier".
- `> Expansions under construction` (0254): "**Its Tunnel destroyed mid-build**: the unfinished expansion is destroyed,
  no refund."
- 0258 point 5 (`core/game_rules.md`): "A defeated Syndicate player's units in the network are destroyed."

**What changed from the plan** (Q0069 is answered)
- **0258:**
  - decay at 1% of MaxHP per second;
  - explicit Attack only;
  - reclaiming at completion, never by a site;
  - partly covered expansions block their cells and count as occupied;
  - a defeated player's network units destroyed.
- **0254:** unfinished expansions destroyed with no refund.

**Code**
- **In the removal path** (T38). For each removed Tunnel, in id order:
  - its completed expansions, which all lie inside its area, become ownerless: in-progress work lost, the queue
    refunded, rallies cleared;
  - its unfinished expansions (sites built by it) are destroyed, with no refund;
  - after the removals, the held units that no remaining operational Tunnel of their owner lets out are removed
    (cause: stranded).
- **Decay.** `step_decay` in the economy phase: 1% of MaxHP per second, a placeholder citing 0258, in
  `tuning/tunnel.rs`. Accumulate it
  exactly per tick in fixed point, so that a Hideout lasts 100 s, within a tick.
- **Defeat.** When a Syndicate player is defeated, their held units are removed (0258 point 5).
- **Reclaim.** `reclaim(tunnel)` runs at E's completion and at G's upgrade completion. Every neutral expansion fully
  inside the area, with a Tier Requirement at or below the tier, goes to the Tunnel's owner, is hidden again, and stops
  decaying.
- **Explicit Attack only.** Auto-targeting and right-click already never pick neutral objects (`commands.md >
  Attack`). H adds the tests.
- **Vision.** A neutral expansion is visible to everyone and remembered. A reclaimed one is hidden, and an enemy's
  record of it drops when the enemy sees its tiles again.
- **Client.** Neutral expansions drawn for everyone in the neutral colour, with an HP bar that falls.

**Acceptance**
- **The only Tunnel destroyed**, with its Hideout:
  - the Hideout turns neutral, becomes visible to the enemy, and loses 4 HP per second, dying after 100 s;
  - with no other structure, the Syndicate is defeated;
  - its Agents on the map become neutral, and the units in the network are destroyed.
- **One of two Tunnels destroyed:**
  - its Hideout turns neutral;
  - infantry in the network survive;
  - the Hideout's unit in production is lost, and its queued units are refunded.
- **A Hideout under construction** when its Tunnel falls is destroyed, with no refund.
- **Explicit Attack.** A Peacekeeper's right-click on a neutral Hideout moves to it, auto-targeting ignores it, and an
  explicit Attack hits it.
- **Reclaim.**
  - A new Tunnel fully covering a neutral Hideout takes it back at completion, never while a site.
  - One covering it partly leaves it neutral. It then blocks its cells (F's placement) and counts as occupied (B's
    Tunnel Space).
- **The free rebuild.** With no Tunnels, the next Tunnel costs 0. Test it with a debug-spawned structure; S1 adds the
  case with a Laser Turret.
- **Ignored tests.** This slice changes the defeat rules, so run the ignored suite afterwards.

**Owns**: see §3.2. **After**: C and E. **Model**: Opus.

#### I: Supplies (Opus)

> **Running** on `scratchpad/briefs/m4i-supplies.md`, launched when D landed. Its brief folds in 0248 (the Supplies hatch on Side D) and 0256 (shared stations, waiting at an empty one with a notification placeholder, switching resources loses the load), reuses D's Gather job, `LoadDef` and `hatch_square`, and overrides the text below where they differ. Not revised here. Q0082 asks the Designer the points the brief had to settle as placeholders.

**Goal.** Agents gather Supplies.
- A right-click on a Supply Delivery Station starts the job.
- The Agent picks up 1 Supply in 48 ticks and carries it to Side C, the Supply hatch, of the Tunnel it took for this Gather (Q0066 point 2).
- It drops off in 48 ticks, one Agent at a time at Side C, and goes back.

**Design**
- `units/agent.md > Gathering > Supplies`: "CarryCapacity: 1 Supply per trip", "PickUpDuration: 48 frames (3 seconds)", "DropOffDuration: 48 frames (3 seconds)", "Drop-off side: Tunnel Side C".
- `units/agent.md > Right-Click Resolution`: "Supply Delivery Station → Gather supplies".
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

**Goal.** The Agent's panel, ghosts and pointers.
- **Changed (0257).** Q Move, A Build Tunnel, S Drop Off, D Build Laser Turret (greyed until S1), X Stop.
- The Tunnel ghost, with its area, judged as E judges it (0237, 0242).
- The order refused with feedback when the Supplies are short (0233).
- The site's panel shows only X.
- The lingering ghosts, which only their owner sees, and the carried load.

**Design**
- `units/agent.md > Agent ObjectInterfaceState > Panel letters` (0257): "**Q** Move, **A** Build Tunnel, **S** Drop Off
  Resources, **D** Build Laser Turret (greyed until available), **X** Stop. No Attack, Hold Position, Patrol or
  AttackMove buttons; the Agent attacks by right-click. These letters replace the older ones listed below where they
  differ."
- `> DefaultState commands`: "**A: Build Tunnel** — enters AwaitingPlacement for a Tunnel ... Ghost preview follows
  cursor, snapped to grid. Tinted green when valid, red when invalid. R rotates 90° clockwise, Shift+R
  counter-clockwise. F flips horizontally, Shift+F flips vertically. Left-click valid location confirms placement and
  dispatches the Agent".
- `units/agent.md > Right-Click Resolution`, as adopted. A carrying Agent's right-click on an own Tunnel drops off;
  an empty one's enters (C).
- `tunnels.md > Cost Scaling > Construction Cost` (0233): "The order is refused, with feedback, if the player can't
  pay."
- `tunnels.md > Tunnel Area Non-Overlap Rule` (0237, 0242), for the ghost: other players' Tunnels by "their remembered
  position and their tier as last seen"; "never-explored tiles pass on the ghost".
- `units/agent.md > While the Tunnel is under construction` (0234): "the only command is X Cancel Construction".
- `systems/controls/selection_and_hud.md > Lingering build ghosts`, and `> Shift while placing`, as adopted.

**What changed from the plan**
- **0257:** the letters. The doc's older "B: Drop Off Resources" and "Build Laser Turret (hotkey TBD)" are stale
  (Q0085).
- **0233:** the price shows on the A tile's tooltip. A click the player can't pay for gives the unavailable-command
  feedback (0162), and no ghost.
- **0237, 0242:** the ghost is red over any known Tunnel's area, at its last-seen tier, and over a known
  non-Drillable or off-map cell.
- **0234:** the site's panel holds X only.
- **0260:** the D tile, greyed with the reason "Requires an Optics Lab". S1 enables it.

**Code**
- R2 split the client files. J1's own files: `templates/syndicate/agent.rs`, the site's template, `Builder::Agent` in
  `command_ui/interface/placing.rs`, and the Enter pointer in `command_ui/cursor.rs`.
- **The ghost.** Uses E's gate check in knowledge mode, with E's problem kinds turned into the red tiles and the reason
  line. The area overlay comes from J2's `tunnel_areas.rs` when J2 has landed, otherwise from B's `tunnel::area`.
- **Lingering Tunnel ghosts** from E's pending builds, drawn for their owner only.
- **The carried load**, drawn on the Agent as D left it.

**Acceptance**
- The panel and its letters, with D greyed and its reason.
- **The ghost** is red:
  - over an own Tunnel's area;
  - over a remembered enemy Tunnel's area at its last-seen tier;
  - over known Water and over the map edge;
  - and green over unexplored Water.
- **Refusal.** With 0 Supplies, A then a click gives the feedback, and no order.
- **Shift** places several ghosts, one Agent each.
- **The site** offers X only.
- **The owner sees** the lingering ghost; the enemy doesn't.
- **QA screenshots**: the panel, the red and green ghost.

**Owns**: see §3.2. **After**: C, D, E and R2. **Model**: Opus.

#### J2: Tunnel and Hideout UI (Opus)

**Goal.** The Tunnel's and the Hideout's panels, the EjectMenu, the network's contents, the Tunnel Area overlay and
rallies.
- **Changed (0257).** Q Upgrade, W Expand, E Eject, X Cancel. Menus are in grid order, with Z back.
- **Changed (0260).** The Hideout gains E Grenadier.

**Design**
- `tunnels.md > Tunnel ObjectInterfaceState > Panel letters` (0257): "**Q** Upgrade Tunnel, **W** Expand Tunnel, **E**
  Eject, **X** Cancel (upgrade, construction or the current operation). In the **ExpandMenu** and **EjectMenu**,
  entries take the grid slots in order (Q, W, E, R, A, S, …): expansions in the Tech Tree table's order (Hideout,
  Optics Lab, Chemical Plant, Vault, …), and units in the order (Agent, Guard, Grenadier, …); **Z** goes back;
  unavailable entries are greyed. These letters replace the older ones listed below".
- `tunnels.md > EjectMenu`, as adopted, and C's sim.
- `structures/hideout.md > ObjectInterfaceState[Hideout]`:
  - "**Q: Build Agent** ... Only available if BuildQueue has fewer than 20 entries and player has sufficient Space
    Crystals";
  - "**W: Build Guard** ... Only available if the player owns at least one Optics Lab";
  - "**E: Build Grenadier**: deducts 150 Space Crystals (placeholder), adds Grenadier to BuildQueue. Only available if
    the player owns at least one Chemical Plant";
  - "**X: Cancel Production**: removes last entry from BuildQueue, refunds full cost";
  - "**C: Set Rally Point**".
- `structures/README.md > Rally Point Behavior` (0252, 0253), as C built it. 0252's verification: "Rally a Hideout on
  another Hideout. The produced Guard comes out of the parent Tunnel and stands over the other Hideout"; "Rally on a
  Tunnel: the Guard stays inside."

**What changed from the plan**
- **0257:**
  - the letters;
  - the EjectMenu in grid order, by unit kind (Agent, Guard, Grenadier, …);
  - Z back;
  - greyed entries;
  - Q (Upgrade) greyed until G, W (Expand) until F.
- **0260:** the Hideout's E (Grenadier), greyed until U and a Chemical Plant.
- **0252, 0253:** the rally pointer and marker for a rally on an own Tunnel or Hideout.

**Code**
- R2 split the client files. J2's own files: `templates/syndicate/{tunnel,hideout}.rs`, the EjectMenu, the new
  `tunnel_areas.rs` (the overlay: own Tunnels' current areas, and known enemy ones at their last-seen tier), the
  network's contents on the Tunnel's panel, and the selection and control groups for held units.
- **The menus' order comes from data.** The EjectMenu's unit order and the ExpandMenu's expansion order come from the
  sim's ordered lists in `defs/syndicate.rs`, so U and S2 add entries without editing J2's files.

**Acceptance**
- **The letters**, with the right entries greyed and their reasons.
- **The EjectMenu:**
  - in grid order, with counts;
  - Q ejects the oldest Agent;
  - Z goes back.
- **The Hideout:** Q, W (greyed "Requires an Optics Lab"), E (greyed "Requires a Chemical Plant"), X and C.
- **0252's verification**, through the client: the rally marker over the other Hideout.
- **The overlay**, for own and known enemy areas.
- **QA screenshots**: the EjectMenu, the Hideout panel, the overlay.

**Owns**: see §3.2. **After**: B, C and R2. **Model**: Opus.

#### K: Prerequisites, the Optics Lab and the Chemical Plant (Opus)

**Goal.** Generic Tech and Unit Prerequisites, for every faction.
- **The Optics Lab (Changed, 0229)** and **the Chemical Plant (new, 0260)**, each on placeholders and in its own
  module.
- The Guard needs an Optics Lab. The Grenadier's and the Laser Turret's gates are data, used by U and S1.

**Design**
- `core/tech_and_research.md`:
  - "The TechPrerequisite is checked only at the moment construction is **started** (queued at the constructor, Warp
    In begun, expansion construction begun, etc.). Options whose TechPrerequisite is not met are unavailable in the
    build menu";
  - "If the player loses their last qualifying prerequisite structure: Already-completed structures that required it
    keep functioning normally. Constructions already started ... continue ... New constructions that require it cannot
    be started until the player again owns a qualifying (fully constructed) prerequisite structure";
  - for units: "**Check Timing**: checked only when the unit is queued", and "**Losing a Prerequisite**: units already
    queued or in production continue and complete normally".
- `structures/README.md > Tunnel Expansions > Expansion Requirements`: the TechPrerequisite is "satisfied by a fully
  constructed structure anywhere in the Tunnel Network".
- `structures/optics_lab.md` (0229): 2x2, 150 SC (no Supplies), 400 frames, HP 400, Groupable, no commands. Its stat
  block's armour 1/4 is replaced by "**High armor underground**" (`structures/README.md > Tunnel Expansions`, 0264):
  "replaces the 1/4 listed in individual stat blocks", giving 6/24.
- `structures/chemical_plant.md` (0260): the same placeholders; it gates "the Grenadier and the Vehicle Assembly".
- `units/guard.md`: produced at the Hideout, requiring an Optics Lab. `units/grenadier.md`: "TechPrerequisite - player
  owns at least one **Chemical Plant** (explosives)". `structures/laser_turret.md`: "TechPrerequisite - player owns at
  least one Optics Lab".
- 0229's verification: "Building an Optics Lab takes 25 s and then enables Guards at the Hideout."

**What changed from the plan**
- **0229:** the Optics Lab's numbers (Q0059 answered; K is unblocked).
- **0260:** the Chemical Plant, new.
- **0264:** both at 6/24, by the expansion rule. B's armour test covers them.
- **0282 and T54:**
  - the new kinds land in `defs/syndicate/structures/optics_lab.rs` and `chemical_plant.rs`, and in
    `kinds/syndicate/`;
  - no `defs/syndicate.rs` split is left to do: R1 did it;
  - the ExpandMenu's list (F's) gains the two entries.

**Code**
- **`tech.rs`.** `requirement_met(player, kind)`: the player owns a completed structure of that kind. For the
  Syndicate, "anywhere in the Tunnel Network" means owned and complete; neutral expansions don't count (H).
- **`requires`**, a list of kinds (T37: each owned at least once; a `Cow<'static, [ObjectKind]>`, T54), on recipes and
  on structure defs, as data:
  - the Guard's recipe requires the Optics Lab;
  - the Grenadier's (U) requires the Chemical Plant;
  - the Laser Turret (S1) requires the Optics Lab.
- **The checks.** Production checks at enqueue, and construction and expansion orders at start (F's stub becomes
  real). Queued and started work continue when the prerequisite is lost.
- **Client.** `conditions.rs` greys entries with the reason "Requires …".

**Acceptance**
- **0229's verification**: an Optics Lab ordered at a Tunnel completes in 400 ticks, and the Hideout's W enables.
- **Before it**, W is greyed and the gate refuses a Guard.
- **Losing it:** a queued Guard completes, and no new Guard can be queued.
- **The Chemical Plant:** same, for a Grenadier test recipe.
- **Both are hidden expansions:** Tunnel Space falls by 1 each, and the armour is 6/24.
- **Generic:** a GDO test recipe with a requirement behaves the same way.

**Owns**: see §3.2. **After**: F. **Model**: Opus.

#### L: Syndicate oracle and goldens (Opus)

**Goal.** As adopted: the Syndicate's economy oracle and its goldens, now with 0259's settled method, and with Q0076
open.

**Design**
- 0259 (Q0070 answered): "**Distance** is measured edge to edge, patch to the crystal **hatch** (the 1-square hatch
  from 0255)"; "**Tolerance:** ±15%; the 12.5 SC/s ceiling must be exact"; "**Targets:** the opening at 0:34–0:46
  (patch 1 GU from the hatch); the table rows; stacked Tunnels double intake; the Syndicate has the lowest one-base
  saturation"; "**Layout:** one Tunnel facing a single patch per distance. **Stacked check:** the two Tunnels stand on
  **opposite sides of the field**, so each Agent's nearest Tunnel is its own".
- `units/agent.md > Gathering > Space Crystals` (the Syndicate economy check), as adopted.

**What changed from the plan**
- **0259:** the reference geometry, the tolerance and the stacked layout. "Only L's pass or fail" waited on Q0070; that
  is answered.
- **Open (Q0076, filed by the Director):** "With the patch 0–1 GU from the crystal hatch, Agents coming in meet Agents
  going out head-on, in a strip too narrow to pass". D measured 5.33 SC/s at 1 GU with 3 or 4 Agents, and 9.8–10.0 at
  2 GU.
  - **0259's opening target**, with the patch 1 GU from the hatch, can't pass until Q0076 is answered.
  - **L builds everything anyway.** It reports the opening and the 0–2 GU rows as "blocked by Q0076". If its option A
    is chosen ("one flag on the Agent's worker definition", after R1 in the Agent's def), L checks them after it.
  - **The rest passes or fails now:** the rows from 3 GU, the ceiling and the stacked check (D's soak already holds
    25.00 SC/s at 2 GU).

**Code and acceptance.** As adopted:
- `testing/syndicate.rs`, `tests/economy_oracle/syndicate.rs`, `tests/golden/syndicate.rs`;
- `docs/economy_oracle.md`, measured with D's `hatch_square`.

The goldens `syndicate_opening` and `syndicate_vs_gdo` use a layout with the patch 3 GU from the hatch until Q0076 is
answered.

**Owns**: see §3.2. **After**: B, C, D, E, H and I. **Model**: Opus.

#### M: M4 QA (Opus)

**Goal.** As adopted: perf, soaks, the scripted walkthrough, `docs/playtest/m4.md` and the ignored suite, extended for
the new slices.

**What changed from the plan**
- **The walkthrough** (§7) adds:
  - an Attack Ground blast over a hidden Hideout;
  - a Grenadier's grenade in flight;
  - an upright turret firing;
  - a pop-up turret risen at an enemy, and lowered.
- **Perf:**
  - `agents_perf` adds 40 Grenadiers shelling a Tunnel field, and 12 pop-up turrets rising and retracting;
  - a new `projectiles_perf` (O's) is re-run.
- **The soak** adds grenades and turrets. The invariants add "every projectile lands within its flight time".
- **Known placeholders** for the playtest notes:
  - the Optics Lab's and the Chemical Plant's stats, the upgrade's 480 frames, the decay rate;
  - the Grenadier's cost;
  - every Placeholder (QD-…) still open;
  - Q0076.
- **Code organization (0282):**
  - M checks that every kind added in M4 has its def module, its client module and its citations;
  - the citation test passes;
  - `ARCHITECTURE.md`'s "Adding a unit or structure" matches what the slices did.

**Owns**: see §3.2. **After**: everything. **Model**: Opus.

#### O: Combat spike: projectiles, area damage, Attack Ground (Opus), new (0260)

**Goal.** A DoublyDisjointed attack locks its target's position at the end of Aiming. At the end of Firing it launches
a projectile, which lands after its flight time. Its area damage then hits whatever stands there, underground objects
included, even unseen.
- Attack Ground orders such an attack at a point.
- The client draws projectiles and blasts only where the viewer has vision.

O builds this once, for every AoE attack (T46, and the user's "shared mechanics (... AoE) implemented once"). It tests
with test kinds; U and S1 bring the real ones.

**Design**
- `systems/combat.md > DoublyDisjointed`: "The Firing phase is short and spawns a projectile which travels to the
  location the target was at at the end of the Aiming phase. The unit is free after the projectile spawns. The effect
  applies to whatever units are at the target location when the projectile arrives. Targets can dodge by moving out of
  the locked location." Also "RequiresProjectileSpeed - true", and lines 91–93: "CanMiss - derived (... true for
  TailDisjointed/DoublyDisjointed)", "CanTargetGround - derived (... true for TailDisjointed/DoublyDisjointed)",
  "ValidAttackTarget - derived (... UnitTarget | LocationTarget for TailDisjointed/DoublyDisjointed)".
- `systems/combat.md > AoEDamage`:
  - "Damage is uniform across the AoE circle. AoE only affects units whose domain matches the attack's TargetDomain:
    Ground AoE hits Ground and surfaced Underground units, Air AoE hits Air units, Universal AoE hits all units
    regardless of domain";
  - "**Underground objects are hit by area damage too** (user, 2026-11-25, Q0071; Ground and Universal AoE):
    underground DrillUnits, Syndicate expansions, a retracted pop-up Laser Turret and GDO mines take their share of any
    blast over them, even unseen";
  - "A unit's damage share = Attack Damage × (unit overlap area / AoE area). Effective armor = FullArmor × (unit
    overlap area / unit total area). Damage taken = max((damage share × pre-armor multipliers - effective armor) ×
    directional multiplier, 0.5 × overlap fraction) ... The directional zone is determined by the angle from the unit's
    center to the AoE center."
- `> DamageModifierOrder`, and Shields: "For AoE, the full value is the unit's damage share."
- `systems/controls/commands.md > AttackGround`: "Order the unit to attack a specific ground location. Only available
  to units whose AttackType has CanTargetGround = true." It sets "TargetLocation = location, TargetObject = None".
- `systems/controls/behaviors.md > AttackingLocation`: "Same as AttackingObject, except: Target position is
  BaseCommandState.TargetLocation (static); Uses MovingToLocation sub-behavior ...; Range/arc check is against the
  target location; TurretCommandState.LockedTarget is not set ...; Behavior complete after attack effect is applied".
- `core/vision.md > Attacks from outside vision`:
  - "a shot, tracer or projectile is visible to a player only along the part of its path that crosses tiles
    **Visible** to that player";
  - "a unit or structure that attacks one of your units or structures while outside your vision becomes **visible to
    you** for **1.5 seconds (24 frames), placeholder**, after each shot".
- 0262 and 0264 (Q0071): expansions take area damage with high armour underground. 0264's verification: Attack Ground
  over a hidden Hideout does "about 93". 0218: Ground AoE reaches underground DrillUnits (the Transport, gated).

**What changed from the plan.** New: the adopted plan had all of this gated under N1 (the Grenadier). 0260 scheduled
the Grenadier and its 8-su-per-frame grenade, so the mechanics come first, as their own slice.
- **Placeholder (Q0083):**
  - friendly fire: an AoE hits everything whose domain matches, any owner, the attacker's own units and the attacker
    included (the literal rule);
  - the reveal: the attacker, if still alive at the blast, is revealed to the owner of each object the blast damaged;
  - an Attack Ground order fires one shot and completes at its blast;
  - a revealed attacker's projectile at your object is drawn whole (as 0244 does for shots).

**Code** (T45–T48, T52)
- **Splits.** R1 split `combat/phases.rs`, and R2 split `combat_effects.rs` and `command_ui/indicators.rs`.
- **`combat/projectiles.rs`** (T45). `Sim::projectiles`, in launch order, hashed only while non-empty. Each holds:
  - the shooter, its owner and its attack's parameters, copied at launch;
  - the start, the locked point, and the arrival tick from `projectile_speed`.

  In 5e, due projectiles land in launch order, before this tick's new hits are applied. A projectile lands even if its
  shooter has died.
- **`combat/area.rs`** (T46). The only AoE path:
  - the exact overlap of the circle with each candidate's footprint or silhouette rectangle, through `sc_sim::math`;
  - candidates by layer and domain;
  - the share, the effective armour, the directional zone and the minimum, as quoted;
  - Shields first, at full value.

  It never asks `hidden_from`. Single-target damage keeps its path in `combat/damage.rs`.
- **The attack sequence** (T48). The location is locked at the end of Aiming, for a unit or a point target. The
  projectile spawns at the end of Firing, and the unit goes on into Cooldown and Reload.
  - Events: `ProjectileLaunched { shooter, from, to, lands_at }` and `Blast { at, radius }`, and the existing `Hit` and
    `Died` per victim.
  - Reveals as in the placeholder above.
- **Attack Ground** (T47):
  - `OrderKind::AttackGround(Vec2)`, in `orders/attack_ground.rs`;
  - `AttackState::location: Option<Vec2>`, hashed only while set;
  - the gate, which requires CanTargetGround from the def;
  - range from the attacker's shape to the point (`reaches_point`);
  - MovingToLocation to get in range;
  - the order completes at the blast (placeholder).
- **Not in O:** HeadDisjointed and TailDisjointed, which no M4 kind uses. The projectile store keeps a slot for a unit
  target, so that they come cheaply.
- **Where numbers go** (T54). Every number is in `AttackAttrs` (data). Nothing new goes in `tuning/combat.rs` except
  the client-side drawing times, which go in `crates/game/src/tuning/`.
- **Client** (T52):
  - **`combat_effects/projectiles.rs`.** A projectile is drawn from `ProjectileLaunched`, interpolated along its path
    with a cosmetic arc, and only over tiles Visible to the viewer (`visible_spans`).
  - **`combat_effects/blasts.rs`.** A ring at the blast, if it is Visible.
  - Nothing is drawn for a hidden victim.
  - AttackGround's left-click in `command_ui/resolution.rs` gives the order: the test at `resolution/tests.rs:150`
    flips.
  - A ground marker in `command_ui/indicators/`.

**Acceptance** (with a `test_defs` DoublyDisjointed Ground AoE unit: damage 100, radius 40 su, 8 su per frame, range 4)
- **Flight.** A shot at 4 GU lands 32 ticks after launch (256 su at 8 su per tick: 0260's "about 2 s over 4 GU").
- **Dodging.** A target walking away at 5 su per tick is clear of the 40 su circle when it lands, and takes nothing
  (0260: "a walker escapes").
- **The numbers:**
  - a 36x36 infantry fully inside takes 100 × 1296 / 5026.5 ≈ 25.8 before armour, and 24.8 after FullArmor 1;
  - a blast wholly over a hidden 2x2 Hideout (6/24) takes 100 − 24 × 5026.5 / 16384 ≈ 92.6 (0264's "about 93");
  - Shields take the share at full value;
  - the directional zone is taken from the blast's centre;
  - the minimum is 0.5 × the overlap fraction.
- **Domains.** Ground AoE hits a hidden expansion and a surfaced test DrillUnit, and not an air unit. Universal AoE hits
  all three.
- **Attack Ground.**
  - The order walks into range, fires once, and completes at the blast.
  - It is refused for a FullyConnected unit.
  - The panel's D slot gives the order.
- **Hidden information.**
  - The shooter's owner sees the blast where they have vision, and nothing about the hidden Hideout it damaged: no hit
    marker, no death marker, no HP bar.
  - The Hideout's owner sees the blast, and the revealed attacker.
- **Determinism.** A golden with test shooters is stable across runs, and the hash is unchanged while no projectile
  exists.
- **Perf.** A new ignored `projectiles_perf`: 200 projectiles in flight, by interleaved A/B runs.

**Owns**: see §3.2. **After**: R1 and R2 (from A). **Model**: Opus. The work is combat and geometry, not movement or
collision.

#### U: Grenadier (Opus), new (0260)

**Goal.** The Hideout trains Grenadiers behind a Chemical Plant. The Grenadier throws O's grenades, and can Attack
Ground.

**Design**
- `units/grenadier.md`:
  - "UnitBase - HeavyInfantry (assumed, like all Syndicate infantry)"; "Produced at - Hideout"; "TechPrerequisite -
    player owns at least one **Chemical Plant** (explosives)"; "Groupable - true (assumed)";
  - "Silhouette - 36x36", "MaxHP - 70", "PointArmor - 1", "FullArmor - 1", "SightRange - 5", "TunnelSpaceCost - 1";
  - AttackAttributes:
    - "AttackType: DoublyDisjointed (slow projectile to the target's position at the end of Aiming; can be dodged)";
    - "TargetDomain: Ground", "TargetType: AoE";
    - "Damage: 100 (total over the circle ...)", "AoE radius: 40 space units (0.625 grid units)";
    - "Range: 4 grid units", "MinRange: 0";
    - "ProjectileSpeed: 8 space units/frame (a 4 GU throw takes 2 s, so moving units can step out of the blast)";
    - "AimDuration: 6 frames; FiringDuration: 2 frames; CooldownDuration: 4 frames; ReloadDuration: 28 frames";
  - "Cost - 150 Space Crystals; Build Time - 160 frames (10 seconds)";
  - "ObjectInterfaceState: BasicCombatUnitInterfaceState".
- `structures/hideout.md`: "**E: Build Grenadier**: deducts 150 Space Crystals (placeholder) ... Only available if the
  player owns at least one Chemical Plant".
- `units/README.md`: Syndicate lasers are red. The grenade is not a laser.

**What changed from the plan.** New: N1 is scheduled (0260). The Grenadier is in the EjectMenu's order "(Agent, Guard,
Grenadier, …)" (0257).

**Code**
- **The def.** `defs/syndicate/units/grenadier.rs` holds GRENADIER, citing its design file. `TargetType::AoE { radius }`
  is in grid units: 40 / 64 = 0.625. `projectile_speed: Some(8.0)` is in su per tick.
- **The registries.** `ObjectKind::Grenadier` goes last, with its line in R1's kind registry.
- **The recipe.** On the Hideout (`defs/syndicate/structures/hideout.rs`), requiring the Chemical Plant (K's
  `requires`).
- **The EjectMenu's order** is a list in `defs/syndicate.rs`.
- **Client:**
  - `kinds/syndicate/grenadier.rs`: its label "GR" and a grenade projectile style;
  - the E entry in `templates/syndicate/hideout.rs`;
  - the generic combat template already gives it Attack Ground.

**Acceptance**
- **Training.**
  - Without a Chemical Plant, E is greyed and the gate refuses it.
  - With one, a Grenadier costs 150 SC, takes 160 ticks, and goes into the network.
- **0260's verification** with the real kind: "the grenade takes about 2 s over 4 GU and a walker escapes".
- **0264's verification**: a Grenadier's Attack Ground over a hidden enemy Hideout does about 93, and the attacker's
  owner learns nothing of the Hideout.
- **Firing rate.** One grenade per 40 ticks against a stationary target.
- **The Tunnel.** It walks into a Tier 1 Tunnel and is ejected, as HeavyInfantry.
- **Goldens.** No change: the kind is new, and no golden trains it.

**Owns**: see §3.2. **After**: O and K. **Model**: Opus.

#### S1: Laser Turret, upright and Agent-built (Opus), new (0260)

**Goal.** An Agent builds a Laser Turret by the Tunnel flow, once an Optics Lab is owned. Outside every own Tunnel
Area, it stands as an upright surface structure that fires a red laser at enemies in range.

**Design**
- `structures/laser_turret.md`:
  - "**Upright turret** (outside every own Tunnel Area): a normal surface structure, always visible. Costs no Tunnel
    Space";
  - "**By an Agent**: built using the same flow as Tunnel construction (see Agent > Building > Tunnel Construction
    Flow): the Agent enters the partially-built structure and is untargetable for the duration; if the structure is
    destroyed or cancelled mid-construction the Agent survives and emerges at the location. On successful completion,
    the Agent emerges on the surface adjacent to the finished structure ... If built inside a Tunnel Area, it is a
    pop-up";
  - the placeholder stats:
    - "Cost: 100 Space Crystals (no Supplies). Build time: 320 frames (20 s)";
    - "HP: 300. SightRange: 6 grid units";
    - "Armor (building armor rule, 1x1): raised PointArmor 1, FullArmor 1";
    - "AttackAttributes: FullyConnected red laser (instant, cannot miss); TargetDomain Universal; SingleTarget; Damage
      24; Range 5 grid units; MinRange 0; Aim 4, Firing 1, Cooldown 3, Reload 16 frames (one shot per 24 frames, 16
      damage per second)";
    - "Hotkey: Agent **D** (Q0068)".
- "TechPrerequisite - player owns at least one Optics Lab (anywhere in the Tunnel Network)".
- `tunnels.md > Losing Tunnels > Defeat`: "Losing the last Tunnel therefore defeats the Syndicate only if they own no
  Laser Turrets".
- `units/README.md`: "**Syndicate lasers are red**".

**What changed from the plan.** New: N2 is scheduled (0260, 0264, 0267). The adopted T-list had no structure that
attacks (T49).
- **Placeholder (Q0084):**
  - it is paid when the ghost is placed, like the Tunnel (0233);
  - cancelling the site, or an order that fails before the start, refunds 100%, the price having no scaling;
  - the site is visible to enemies, like a Tunnel site (0234), and the Agent inside is not;
  - the Agent emerges at the first clear spot next to the footprint, searched in a fixed order from the side facing
    where it came from, ties by direction index;
  - an upright turret blocks movement like any building;
  - it has no commands. It auto-targets like a unit holding position (BaseAutoTargeting within Range, then the closest,
    ties by id);
  - an Agent's turret uses no Tunnel's operation slot.
- **Inside an own Tunnel Area,** S1's turret stands upright until S2 lands in the next wave. S2 then makes it a pop-up.

**Code**
- **The def.** `defs/syndicate/structures/laser_turret.rs`: LASER_TURRET, 1x1, with its raised armour and its attack,
  citing its design file. `ObjectKind::LaserTurret` goes last.
- **Structures that attack** (T49). `StructureDef::attack: Option<AttackAttrs>`, and `Shooter::new` takes the attack
  and the object, not a `&UnitDef`. A structure has no turning, no moving and no Attack orders. In 5e it picks a target
  by the placeholder rule and fires through the same phases.
- **The Build order's turret arm** (T51). E's `OrderKind::Build { kind, origin, orientation }`, with kind LaserTurret:
  - normal surface placement in knowledge mode, and the truth on arrival;
  - K's requirement;
  - the SC price.

  The site's `Construction` holds the Agent at `Place::Site`. At completion, the Agent comes out adjacent (T51), not
  into the network.
- **Defeat** counts the turret as a structure. H's free-rebuild test gains the turret case.
- **Client:**
  - the D entry in `templates/syndicate/agent.rs`, greyed until an Optics Lab;
  - the 1x1 ghost;
  - `kinds/syndicate/laser_turret.rs`: label "LT", `ShotStyle::RedLaser`;
  - `object_render/turret.rs`, with a barrel that faces the target.

**Acceptance**
- **0260's verification**: "a turret kills a Guard in 4 shots" (24 − 1 = 23 per shot).
- **The firing rate.** One shot per 24 ticks, and 16 damage per second against armour 0. Its Universal domain hits a
  test air unit.
- **Building:**
  - greyed and refused without an Optics Lab;
  - 100 SC paid at the order;
  - the site, with the Agent untargetable inside;
  - 320 ticks;
  - the Agent then stands next to it.
- **Cancel and destruction.** Cancel refunds 100 SC and the Agent comes out at the footprint's centre. A destroyed site
  loses the SC, and the Agent survives.
- **Defeat.** The last Tunnel lost while a turret stands: not defeated.
- **QA screenshot**: a turret firing a red laser.

**Owns**: see §3.2. **After**: E, J1, K and O. **Model**: Opus.

#### S2: Laser Turret, pop-up and Tunnel-built (Opus), new (0260, 0264, 0267)

**Goal.** A turret inside an own Tunnel Area is a pop-up:
- it holds 1 cell of the Tunnel's region;
- with no target it stays underground, hidden, at 6/6;
- it rises in 8 frames at an enemy in range and fires;
- it retracts 32 frames after its last target, taking 8 frames.

The form follows the Tunnel Areas as they change. A Tunnel builds turrets from its ExpandMenu, using its operation
slot.

**Design**
- `structures/laser_turret.md > Upright vs Pop-up`: "**Pop-up turret** (inside an own Tunnel Area, whoever built it)":
  - "Occupies 1 grid unit of that Tunnel's tunnel-region, like an expansion, so it reduces Tunnel Space (¼ Tunnel
    Space, subject to the rounding rule in Tunnel Space)";
  - "While it has no target it stays **underground**, hidden like an expansion";
  - "It notices enemies the same way an upright turret does. When an enemy comes into range it rises to the surface
    **already facing the target** and fires; it retracts once it has no target";
  - "It **can be attacked while underground** (by enemies that can see it), but it has **much higher armor while
    underground**".

  Its "Rise and retract timing: TBD" and "Values TBD" are stale against the stats below (Q0085).
- `> Conversion`: "The form updates automatically whenever Tunnel Areas change":
  - "A new Tunnel, or a Tunnel upgrade, whose Tunnel Area now covers an upright turret converts it into a pop-up
    turret";
  - "When a Tunnel is destroyed, the pop-up turrets in its Tunnel Area become upright turrets";
  - "If a conversion pushes Tunnel Space below the player's current usage, the over-capacity rule applies".
- `> Construction`: "**By a Tunnel**: from the Tunnel's ExpandMenu, placed within that Tunnel's Tunnel Area like an
  expansion. It **uses the Tunnel's single operation slot** ...: while a Tunnel builds a turret it can't build an
  expansion or upgrade."
- The stats: "underground (pop-up) PointArmor 6, FullArmor 6"; "Pop-up: rises in 8 frames (already facing its target);
  retracts after 32 frames with no target, taking 8 frames. It can't fire while rising or retracting"; "in the
  Tunnel's ExpandMenu it takes the next slot after the expansions".
- `tunnels.md > ExpandMenu`: "The ExpandMenu also offers the **Laser Turret** (once the player owns an Optics Lab)".
- `systems/combat.md > AoEDamage`: "a retracted pop-up Laser Turret ... take[s] their share of any blast over them, even
  unseen".
- 0267 (detection later): "Until then hidden things (expansions, pop-up turrets, burrowed DrillUnits, mines) can only
  be hurt by area damage over them."

**What changed from the plan.** New (0260, 0264, 0267).
- **Placeholder (Q0084):**
  - hidden and at 6/6 only while fully lowered; rising, raised and retracting it is visible and at 1/1;
  - its sight is 6 in both forms;
  - a pop-up never blocks movement, whatever its state;
  - a Tunnel-built turret's site is hidden like an expansion under construction, and refunds 75% on cancel, like an
    expansion (0254);
  - only an operational Tunnel's area makes a pop-up, as for reclaiming (0258).

  If the Designer rules that a raised pop-up blocks units, S2 becomes navigation work (a footprint appearing under
  units), and Fable becomes arguable.

**Code** (T50)
- **The form is derived, not stored.** A turret is a pop-up if and only if its tile is inside an own operational
  Tunnel's current area. So conversion needs no event: it follows E's, G's and H's changes to the areas. H's "Laser
  Turrets stay owned" needs no code.
- **`popup.rs`.** `TurretState { phase: Lowered | Rising | Raised | Retracting, progress, idle }`, using R1's progress
  type, hashed only while the turret is a pop-up. A rising turret faces its target from the first frame. It can't fire
  while rising or retracting. The 8, 32 and 8 frames are in the turret's def.
- **Hiding, armour and occupancy:**
  - `vision/hidden.rs` gains the arm "a lowered pop-up";
  - `Object::armor()` gives 6/6 while lowered;
  - B's occupancy predicate gains the arm "a pop-up turret: 1 cell of its area's region".
- **Building from a Tunnel:**
  - F's `Method::Expansion` takes a 1x1 kind;
  - the operation slot's `Building(site)` takes a turret;
  - the ExpandMenu's data list gains the turret, after the expansions, behind K's Optics Lab requirement.
- **Client:**
  - its owner sees a lowered pop-up as an outline, like expansions;
  - enemies see it only while it is visible;
  - the rise and retract are drawn as the barrel coming up, in `object_render/turret.rs`.

**Acceptance**
- **0267's verification**: "while the Tunnel builds a turret, Expand and Upgrade are unavailable".
- **Building.** A turret from the ExpandMenu inside the area: 100 SC at placement, 320 ticks, then Tunnel Space falls
  by ¼. Four pop-ups take 1.
- **Hidden.**
  - Lowered, enemies don't see it, can't target it, and their placement over it fails as over an expansion.
  - It rises 8 ticks after an enemy enters range 5, and fires.
  - It retracts 32 ticks after its last target, over 8 ticks, firing at nothing meanwhile.
- **Armour.** A test AoE over a lowered pop-up deals the share less 6 × the overlap fraction. Raised, less 1 × it.
- **Conversion:**
  - an Agent's upright turret becomes a pop-up when a new Tunnel's area covers it at completion;
  - a pop-up becomes upright when its Tunnel dies;
  - over-capacity applies: production waits.
- **B's armour test** gains the turret's two forms.
- **QA screenshots**: a pop-up rising at an enemy, and lowered (owner view).

**Owns**: see §3.2. **After**: S1, F, G and H. **Model**: Opus. If the blocking rule above goes the other way:
Fable.

#### Gated slices (not scheduled; each waits for its numbers)

N1 (the Grenadier) and N2 (the Laser Turret) are scheduled now: O, U, S1 and S2. Each gated slice below lands its kinds
in their own modules (T54), and builds on the shared mechanics: O's area damage, and T44's deploy model.

- **N3 Vehicle Assembly, Assault Bike, Laser Battery, Subterranean Transport.**
  - Needs the numbers (the Vehicle Assembly's too), and the vehicles milestone: FixedTurnRadiusMovement, turret arcs,
    0033 and 0035.
  - Transit through Tier 2 Tunnels is already built (C).
  - **The vehicle deploy order** (T44; 0211, 0213, 0214) comes with the first vehicle that deploys: the Laser Battery's
    Deploy and Undeploy, or the Transport's Burrow and Surface (0220). It is built once, for every deploying kind. The
    GDO's Artillery Rig, not built yet, adds a def row. **Changed (0263):**
    - undeploy always completes;
    - held orders the new kind can't carry out are dropped, with feedback.
  - **The Laser Battery** is behind the Optics Lab (0210; K). Its beam and surges are new combat work. **Relay**
    (0276–0281) needs Prisms (N6) and is concept only.
  - **The Assault Bike's** mortar is a DoublyDisjointed AoE: it uses O's projectiles and area damage, with no new
    mechanics.
  - **The Subterranean Transport** (0215–0222) is behind a Forge (0221; N5's stats). Its pieces:
    - **Changed (0263):** the DrillUnit base, with no attack. It exists in the defs (`UnitBase::DrillUnit`, with
      `NavClass::Underground` over Drillable tiles), but no kind uses it yet;
    - no collision underground (`systems/units.md > Underground Collision`). `Domain::hard_collision` still makes
      underground units collide. That was M1's assumption for gap G14 (`docs/movement.md`), and the design has since
      settled it the other way;
    - **Changed (0266):** it passes through underground structures, and may surface above expansions;
    - the surface check (0217: the tiles' tracked passability, and no building), and the footprint reserved while
      surfacing;
    - hidden underground (0219, through A's predicate). Detection comes "after all faction units are done" (0267), so
      until then only area damage hurts it (0218, through O);
    - targeting surfaced and submerged units: the TODO at `combat/targeting.rs:41`;
    - loading and unloading, a new mechanic;
    - passengers held in the carrier (T30), who die with it (0220) and stay aboard in the network (0222). **Changed
      (0265):** passengers count for Tunnel Space, aboard or not, through B's `used` iterator.
- **N4 Shadebringer.** Smoke that blocks sight is a new kind of vision. Also slow fields, and tanks refilled in the
  network.
- **N5 Null Chamber, Vault, Forge, Aerospace Assembly, Orbital Command.**
  - These are data on F's and K's machinery once their stats exist, each in its own module.
  - The Chemical Plant left this list for K (0260).
  - The Null Chamber's buff also needs detection.
  - The Forge also gates the Transport (0221).
  - The Aerospace Assembly produces the Phantom, the Specter and Prisms (N6, N7).
  - The Vault works network-wide (0269).
  - The Orbital Command gates Prisms (0274).
- **N6 Phantom** (0226–0228), with aircraft and the Tier 3 air doors. **Prisms** (0274, 0275, 0280, 0281) come with
  it.
  - **Changed (0263):** a Groupable HoverCraft, with a FullyConnected SingleTarget laser.
  - Needs its numbers, the Aerospace Assembly's stats (Tier 3, behind a Forge), and the Optics Lab (K).
  - No air unit exists for any faction yet: air movement and air collision are plan.md's "Later: air".
    `UnitBase::HoverCraft` exists in the defs, unused.
  - The air doors: where they are, and whether they take one aircraft at a time (Q0064 point 11).
  - Its turret's 60–90° cone shares the turret-arc work with the vehicles milestone.
  - A Prism is launched one-way through an air exit, and only Relays.
- **N7 Specter (new; 0268–0273; concept only).**
  - A single-payload Glider bomber from the Aerospace Assembly, behind a Vault and a Chemical Plant.
  - It attacks Ground only. Its bomb is DoublyDisjointed (assumed), so it can Attack Ground, and it uses O's area
    damage and the GliderMovement exception (`behaviors.md > AttackingLocation`: "Same strafing run pattern as
    AttackingObject, targeting the ground location").
  - It lands and re-arms in Tier 3 Tunnels, flies home when unarmed, and its order rules depend on its ammunition.
  - All numbers are TBD. It needs N6's air movement and air doors.

---

## 4. Technical decisions for the Director

T29–T44 are the adopted decisions, updated where answers changed them (**Changed (NNNN)**). T45–T54 are new: each has a
recommendation and its alternatives.

### T29. Syndicate kinds in the M2 and M3 model (A; done)
- **Done in A** as recommended: one static per kind, `ObjectKind` appended, `StructureDef::underground`,
  `StructureDef::tunnel`, the Hideout AAAA, the Agent's worker (D), `UnitDef::tunnel_space_cost`.
- **Changed (0282, T54).** R1 moves each kind into its own module (`defs/syndicate/{units,structures}/<object>.rs`), and
  `StructureDef::tunnel` becomes an inline value. K, S1 and U add their kinds the same way, each in its own module.
- **Alternatives:** as adopted. A generic `ArmyCost` is still M5's call.
### T30. Units off the map: one held store (C, E). The hard piece, on Fable.
- **Status:** C is building it on its brief, which supersedes this text where they differ. E adds `Place::Site`, and S1 uses it.
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

### T31. On-site construction: one `Construction` state (E, F, S1, S2)
- **Changed (0234, 0254, 0260):** ConstructionHP gives the maximum, with damage kept, for the Tunnel site, every expansion and the turret. The progress uses R1's progress type (T54).
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
- **Built in A** as recommended: `StructureDef::underground`, `Hiding::of`, `hidden_from`, visible when neutral, the
  terrain's elevation. The parent Tunnel is derived. Damage never asks the predicate.
- **Changed (0240).** "The surface methods ignore them" is replaced. 0240 (`structures/README.md > Tunnel
  Expansions`): "a surface structure and an underground expansion (any player's, including neutral ones) can never
  share a tile".
  - **Each layer consults the other's truth.**
    - The surface methods already count underground structures as occupants (`ObjectKind::may_overlap`), so a direct
      placement over a hidden Hideout is red at once. 0240 accepts that leak: "this is accepted".
    - `Mode::Known` excludes hidden objects, so a worker-built ghost stays green and fails on arrival, as 0240 says.
    - Warp In is judged like direct placement (Q0080).
  - **F's `Method::Expansion`** refuses any surface structure of any player, by the truth (0240), and resources (0241).
- **Changed (0260, 0264).** S2 adds the lowered pop-up turret to the predicate and to the layer. Neither the pop-up nor
  the expansions block movement.
- **Alternatives:** as adopted.

### T33. Tunnel Space: a derived army cap (B)
- **Recommend** (as adopted, changed by B's spec): `ArmyResource::TunnelSpace`. Available = min(200, ⌊unused / 4⌋),
  and Used is one iterator.
- **Changed (0242).** A Tunnel's whole area is on the map and Drillable, by placement and by upgrade, so Tunnel Space
  has no off-map or non-Drillable case. The adopted "the area's cells on the map (Q0062 point 2)" is gone.
- **Changed (0258).** Unused cells exclude the owner's **and neutral** underground cells inside the area. Another
  player's expansions can't be there, since the areas are disjoint across players (0237).
- **Changed (0265).** Every unit counts for life, wherever it is: on the map, held, in production, or aboard a
  Transport (N3). Used is one iterator that C chains its held units into.
- **Changed (0260).** S2 adds a pop-up turret: 1 cell, so ¼ Tunnel Space.
- **Alternatives:** as adopted.
### T34. The Agent as a worker: one Gather job (D, I). On Fable for D.
- **Done in D** (`edc30e11`) as recommended, with 0255's rules: `WorkerDef::LoadDef` per resource, `StructureDef::drop_off` hatches, `tunnel::hatch_square`, `GatherPhase::AtHatch`. I fills the Supplies arm.
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
- **Status:** C's brief (0245–0253) supersedes this text where they differ: walking in single file with no fixed hold (0246), the exit half nearer B (0248), the longest-inside unit first (0251).
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
- **Recommend:** one command enum per capability, M2's convention.
  - **The Tunnel** (**Changed, 0254, 0257**): `Command::Tunnel { tunnel, command }`, with `TunnelCommand::{Eject { kind },
    Expand { kind, origin, orientation }, Upgrade, Cancel}`.
    - Cancel cancels the Tunnel's current operation: an upgrade, or an expansion or turret under construction (0254:
      "The Tunnel's X cancels whatever operation it is running").
    - A site's own X is `CancelConstruction`, on the site: the Tunnel site (E), an expansion (F), or a turret (S1, S2).
  - **The Hideout:** the existing `Command::Production`.
  - **Orders** (**Changed, T47, T51**):
    - `OrderKind::{Gather, DropOff, Enter}`, with Gather and Drop Off generalised to Tunnels and stations;
    - `OrderKind::Build { kind, origin, orientation }`, which is kind-generic, for the Tunnel (E) and the Laser Turret
      (S1);
    - `OrderKind::AttackGround(Vec2)` (O).

  Each slice extends the `every_command` tests.
- **Alternatives:** as adopted. Also, a separate `CancelUpgrade`. 0257's single X makes it a second path to the same
  thing.

### T37. Tech and unit prerequisites: one generic check (K)
- **Recommend:** as adopted.
  - `requires` is a list of kinds, each owned at least once, on recipes and on structure defs.
  - One check, at queueing and at construction start, as `core/tech_and_research.md` says. Losing a prerequisite stops
    only new work.
  - **Changed (T54):** the list is a `Cow<'static, [ObjectKind]>`, so the defs stay serde-ready.
  - **Changed (0260):** the Grenadier needs a Chemical Plant, and the Specter (N7, concept) a Vault and a Chemical
    Plant.
    So the list form is needed.
- **Alternatives:** as adopted.

### T38. Losing Tunnels in the removal path (H)
- **Recommend:** as adopted: `remove_objects` runs the Tunnel rule for every removed Tunnel, in id order, in the same
  tick.
- **Changed (0254, 0258):**
  1. neutralise its completed expansions;
  2. destroy its unfinished ones, with no refund;
  3. refund queued work and drop work in progress;
  4. strand: remove the held units that no remaining operational Tunnel lets out.

  Decay is 1% of MaxHP per second, in the economy phase. Reclaiming happens at completion only (E's site, G's
  upgrade), never by a site. Defeat removes the defeated player's held units (0258 point 5). Pop-up turrets need no
  step, since their form is derived (T50).
- **Alternatives:** as adopted.

### T39. Tunnel tiers: one kind with a tier (A, F, G)
- **Built in A:** one kind, `TunnelState { tier }`, the tier table in the Tunnel's def, and `set_tier` keeping the
  damage.
- **Changed (0229, 0231, 0254, 0267):** `TunnelState::operation: Option<Operation>`, hashed only while set.
  - `Building(site)` covers an expansion or a turret (F, S2), and `Upgrading { progress, to }` covers G's upgrade.
  - One operation at a time. Eject is not an operation.
  - While upgrading, the target tier's area is reserved for placement and upgrade checks. Transit, exits, Tunnel Space
    and reclaiming keep the current tier until completion (0249, 0258).
- **Changed (T54):** the tiers' numbers (HP, radius, upgrade time and price rule) sit in the Tunnel's def module.
- **Alternatives:** as adopted.

### T40. Start, layouts and validation (B)
- **Changed (0242, 0258, 0264):** as B's spec (§3.3 B).
  - `tunnel/area.rs`.
  - The area rule (`TunnelAreaOffMap`, `TunnelAreaNotDrillable`) in every placement mode. Spawns are exempt.
  - `OutsideTunnelArea` for layout expansions, through `check_layout`, until F's `Method::Expansion` takes over.
  - `StartingConditions::of(Syndicate)`.
  - Layouts on `dev_skirmish`.
  - The armour test.
- **Open (Q0076):** B's layout puts a patch 1 GU from a hatch, where three Agents jam. If Q0076's answer is option B,
  the layouts move the patch to 3 GU or more.
- **Alternatives:** as adopted.

### T41. Client (A, F, J1, J2)
- **Recommend:** as adopted (`Builder::Agent`, `Builder::Tunnel`, the overlay, owner-only outlines,
  `ResourceFigure::TunnelSpace`, red lasers, the Enter pointer), with these changes.
- **Changed (0282, T54):** after R2, templates are per object: `templates/syndicate/{agent,tunnel,hideout,
  expand_menu}.rs`. Each kind has a client module, `kinds/syndicate/<object>.rs`, behind one registry. The menus'
  orders come from the sim's data lists.
- **Changed (0257):** the letters. Menus are in grid order, with Z back.
- **Changed (0260):** the AttackGround order and its indicator (O); projectiles and blasts (T52); the turret's look
  (S1, S2).
- The client still learns what is hidden only through `Sight`.

### T42. Tests, hashing and splits (all slices)
- **Recommend:** as adopted for `testing::syndicate` and the invariants, with these changes.
- **Invariants added:**
  - a site's builder is held at that site (E);
  - at most one operation per Tunnel (F);
  - every projectile lands by its arrival tick (O);
  - a pop-up's form matches the areas (S2).
- **Changed (m4-common): hashing.** A new field is hashed only while it is set. That is the adopted "Alternatives"
  line, which m4-common chose. A merged with the goldens unchanged, and so did D. A slice re-blesses only when
  behaviour changes. R1 and R2 change no golden.
- **Changed (0282): splits.**
  - `placement/check.rs`: B.
  - `orders/gather.rs`: D, done.
  - `orders.rs`, `combat/phases.rs` and the defs: R1.
  - `templates.rs`, `hud/selection_panel.rs`, `command_ui/indicators.rs`, `object_render.rs` and `combat_effects.rs`:
    R2.
  - `sim.rs`: one new file per new step.
- **Alternatives:** as adopted.

### T43. Work gated on numbers (not in M4)
- **Changed (0260):** projectiles, AoE and Attack Ground are scheduled (O), and so are the Grenadier and the Laser
  Turret (U, S1, S2). Still gated:
  - HeadDisjointed and TailDisjointed, which no M4 kind uses (O leaves a slot);
  - **vehicles:** FixedTurnRadiusMovement and turret arcs (the vehicles milestone; Fable, and the user's call);
  - **the vehicle deploy order** (T44), with the first vehicle that deploys;
  - **the Laser Battery's beam**, and Relay (concept);
  - **the Subterranean Transport** (N3; Fable for the movement);
  - **aircraft** (N6, N7);
  - **smoke that blocks sight** (N4).
- **Changed (0267): detection** comes "after all faction units are done". It adds a "revealed" term to A's predicate,
  for expansions, pop-up turrets, DrillUnits and mines. Until then, only area damage reaches them.
### T44. The vehicle deploy order (0211, 0213, 0214, 0220, 0263; not in M4)
- **Changed (0282: "shared mechanics (deploy model, toggles, AoE) implemented once and reused").** The model below is
  built once, with the first vehicle that deploys, and every other deploying kind adds a def row, never code. What it
  shares with the other kind changes, and what it doesn't:
  - **Shared:** `Sim::change_kind`; R1's progress type (`progress.rs`) for its timer; the one `can_take(object,
    order)` filter below, which the gate also uses; the unavailable-command feedback.
  - **Kept apart, on purpose:** its order semantics. They differ from the toggle (R1's `toggle.rs`: an ability, the
    kind switching at the press) and from `transform.rs` (cancellable, the target kind from the first frame), as
    argued below.
  - **The pop-up turret (S2)** is not a deploy: it changes no kind, takes no order, and its form follows the areas.
    It shares only the progress type.
- **Changed (0263):** undeploy always completes, and held orders the new kind can't carry out are dropped, with
  feedback. This confirms the recommendation below.

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
  - **Def.** `UnitDef::vehicle_deploy: Option<VehicleDeploy { into: ObjectKind, frames: u32 }>`, one row on each kind of a pair, in each kind's own module (T54). The values are TBD for all three units.
  - **The order.**
    - `OrderKind::VehicleDeploy`, with no payload: `into` comes from the def. It is a working name, kept distinct from the Drone's `Deploy`.
    - The panel button depends on the kind: Deploy, Undeploy, Burrow or Surface. The other kind of the pair carries the reverse button, so "Undeploy is only available once deployment is complete" follows from the kind, with no flag.
  - **Where.**
    - The rules go in `crates/sim/src/vehicle_deploy.rs`, beside `conversion.rs` and R1's `toggle.rs`.
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

### T45. Projectiles: launches stored in the sim (O), new
- **Recommend:**
  - `Sim::projectiles`, in launch order, hashed only while non-empty. Each projectile holds:
    - its shooter, owner, attack parameters, start, locked point and arrival tick;
    - the parameters copied at launch, so that a dead shooter's projectile still lands with its numbers.
  - The flight time is the straight distance divided by `projectile_speed`, rounded up to whole ticks. There is no
    path and no collision in flight. The design has none: the effect "applies to whatever units are at the target
    location when the projectile arrives".
  - Due projectiles land in 5e, in launch order, then removals run as today. A projectile is not an object: it can't
    be seen, selected or targeted.
- **Alternatives:**
  - projectiles as `Object`s: free vision and records, but they would need ids, hashing of every field, and a skip in
    every object system;
  - resolving at launch, with a delayed effect only for the client: wrong, since targets dodge by moving.

### T46. Area damage: one module, exact overlap (O), new
- **Recommend:** `combat/area.rs`, the only AoE path, for every AoE attack in the game: grenades, and later the mortar,
  bombs, shells and mines.
  - The exact area of the circle inside each candidate's axis-aligned rectangle (a footprint, or a unit's silhouette),
    in f32, with the arcsine and square root from `sc_sim::math` (libm). This is deterministic, as everywhere else.
  - **Candidates:** objects whose layer and domain the attack's TargetDomain allows, underground structures included,
    any owner (Q0083). Destructible map objects are candidates too, if they have HP.
  - **The formula** is as quoted (share, effective armour, directional zone from the AoE centre, minimum 0.5 × the
    overlap fraction), in `DamageModifierOrder`'s steps. Shields go first, at full value.
  - It never asks `hidden_from`.
- **Alternatives:**
  - sampling the circle on a grid: simpler, but approximate, and its error shows in the 0.5 minimum and in small
    overlaps;
  - fixed point: exact across platforms, but the rest of combat is f32 under the determinism rules, which already
    cover libm.

### T47. Attack Ground: an order with a location target (O), new
- **Recommend:**
  - `OrderKind::AttackGround(Vec2)`.
  - `AttackState::location: Option<Vec2>`, hashed only while set, next to `target: Option<ObjectId>`. At most one is
    set; an enum would rewrite the hashed shape of every attacking unit.
  - The gate requires CanTargetGround, derived from the def's attack type.
  - Range is measured from the attacker's shape to the point.
  - The order completes at its blast (placeholder, Q0083). The point is the clicked point, with no snapping.
- **Alternatives:** a `Target::Point` arm in the existing Attack order. That would touch every Attack path, and the
  design gives AttackGround its own command and behaviour.

### T48. The DoublyDisjointed sequence, its events and reveals (O), new
- **Recommend:**
  - The target's position is locked at the end of Aiming.
  - The projectile spawns at the end of Firing. The shooter goes on into Cooldown and Reload, free to move, as the
    design says.
  - **Events:** `ProjectileLaunched { shooter, from, to, lands_at }` and `Blast { at, radius }`, then `Hit` and `Died`
    per victim. The client filters them by vision (T52).
  - **The reveal:** at the blast, the shooter, if still alive, is revealed for 24 frames to the owner of each object
    the blast damaged (Q0083). A blast that damages nothing reveals nothing.
- **Alternatives:** revealing at launch, to the owners of objects at the locked point. That is wrong for dodged shots,
  and it leaks hidden objects at the point.

### T49. Structures that attack (S1), new
- **Recommend:**
  - `StructureDef::attack: Option<AttackAttrs>`.
  - The combat code's shooter takes the attack and the object, so units and structures share the phases and damage.
  - A structure has no turn rate (it fires in any direction), no orders, and its own target pick (Q0084's
    placeholder).
  - Its domain: a structure is Ground for Ground attacks. A lowered pop-up is underground (S2).
- **Alternatives:** model the turret as a unit with zero speed. That gives the wrong placement, nav, Tunnel Space and
  defeat rules, and a building that "moves".

### T50. The pop-up form: derived from the areas (S2), new
- **Recommend:**
  - The form is not stored. A turret is a pop-up if and only if its tile is in an own operational Tunnel's current
    area.
  - Only the pop-up's rise and retract state is stored (`TurretState`, hashed only while it is a pop-up).
  - Conversion then follows any change to the areas (a completed Tunnel, an upgrade, a Tunnel lost) with no event
    plumbing.
  - Armour, hiding and occupancy read the form and the state.
- **Alternatives:** a stored form, updated by E, G and H's events. Three writers, and a desync risk the derived form
  doesn't have.

### T51. One Build order for worker-built structures (E, S1), new
- **Recommend:** `OrderKind::Build { kind, origin, orientation }`.
  - It generalises M3-F's lingering ghosts and the Drone's arrival step to any worker-built kind. The kind's def
    gives the method, price and site rule.
  - The Tunnel (E) and the Laser Turret (S1) use it. The Drone's Deploy stays as it is: it is a transform, and the Drone
    is used up.
  - At completion, the def says where the builder goes: into the network (the Tunnel), or out next to the structure
    (the turret).
- **Alternatives:** `OrderKind::BuildTunnel` and `BuildTurret`: two copies of the same flow, against the user's "shared
  mechanics ... implemented once".

### T52. Drawing projectiles and blasts (O), new
- **Recommend:**
  - A projectile is drawn from `ProjectileLaunched`, interpolated by tick along its line, with a cosmetic arc by
    attack type.
  - It is clipped by `visible_spans` to the viewer's Visible tiles (`core/vision.md > Attacks from outside vision`),
    except a revealed attacker's projectile at the viewer's own object, which is drawn whole, as `shot_spans` does
    for shots (0244).
  - A blast is a ring, drawn only if Visible.
  - Nothing is drawn for a hidden victim.
  - The styles come from each kind's client module (T54).
- **Alternatives:** projectile positions in the sim snapshot. That costs state for a cosmetic.

### T53. The placement problem sets (B, E, F), new
- **Recommend:**
  - Widen `TileProblem` from a full u8 to u16 when F adds `NotDrillable`. It is a bitset, so the change is local.
  - `WholeProblem` grows by B's three (`TunnelAreaOffMap`, `TunnelAreaNotDrillable`, `OutsideTunnelArea`) and E's
    `TunnelAreaOverlap`.
  - The client's problem text gets one line each.
- **Alternatives:** reuse `NotBuildable` for undrillable tiles. That is wrong: Rugged is Drillable but not Buildable,
  and the reason line would mislead.

### T54. Code organization: modules now, data files at playtest (R1, R2, every later slice), new
- **Recommend** (the user's decision, "Modules now, data files at playtest"):
  - **One module per object.** `defs/<faction>/{units,structures}/<object>.rs`, named like the design's files, and
    `defs/map/` for map objects. Each cites its design file. A faction's own rules sit in `defs/<faction>.rs`.
  - **Defs as plain data, ready for serde:**
    - `Cow<'static, …>` in place of borrowed statics;
    - no function pointers, no logic, accessors only;
    - `Serialize` and `Deserialize` derived;
    - a round-trip test per kind.
  - **Each object's numbers in its def**, including the ones now in logic files, and the Tunnel's network, tier and
    cost numbers.
  - **One kind registry:** a `macro_rules!` list in `defs.rs` generating `ObjectKind`, `ALL` and `def()`, in the same
    order and with the same serialized names.
  - **Tunables** in `tuning/<system>.rs`: named constants grouped by system, each with a design citation or
    "Placeholder (not in the design)". Technical constants stay where they are.
  - **Shared mechanics, built once:**
    - the progress type;
    - the toggle rule (Riot Shield and Weapon Conversion);
    - area damage (T46);
    - the Build order (T51);
    - the deploy model (T44).
    They are shared only where hashes stay byte-identical; the rest is listed, and shared at its next re-bless.
  - **Client:** a module per kind (`kinds/<faction>/<object>.rs`) and templates per object, behind one registry.
  - **The citation test** fails on a cited design path that doesn't exist.
  - **Later, at playtest prep (not M4):** a loader reads the defs and the tuning structs from data files and swaps them
    live, in dev builds. The defs would keep a `&'static` registry, filled by leaking the loaded data, as `test_defs`
    already does.
- **Alternatives:**
  - data files now: rejected by the user;
  - per-faction def files with sections: fewer files, but not "per-unit and per-structure ... in their own modules";
  - tunables as fields of a `Tuning` struct now: closer to the data-file step, but every use site would change twice;
  - keeping `&'static [T]` and leaking at load: less churn now, but `Deserialize` can't be derived, so "ready to
    derive serde" fails.

---

## 5. Design questions

**Every M4 question is answered.** Q0059–Q0071 are answered by 0229–0271, and their blocking status is gone: K and G
are unblocked (0229), and L's pass or fail is settled (0259).

**Open: Q0076** (filed by the Director after D): Agents jam head-on near the hatch at 0–1 GU.
- It partly blocks L (0259's opening target) and the first playable's layout (B's patch 1 GU from the hatch).
- Everything else proceeds.

**Eight new question drafts**, in `scratchpad/m4/rev/questions/`. Each is in the Director's format: a `# QXXXX —
Title` heading, the Date left as `DATE`, and options with their consequences.
- **None is blocking.** Each slice builds the proposal, marked "Placeholder (QD-slug)", and cites the draft until the
  answer comes.
- **Number them from Q0077** (Q0076 is taken). Answers that arrive before a slice launches go into its brief.

| Draft | Title | Blocking | Slices |
|---|---|---|---|
| `Q0078.md` | 0242's verification: is 2 tiles from the edge valid? | no | B, E, G |
| `Q0079.md` | Agent build orders: an Agent killed before the start, and the area of a pending ghost | no | E |
| `Q0080.md` | Expansion placement: ground units, lingering ghosts, Warp In over a hidden expansion | no | F |
| `Q0081.md` | The ExpandMenu's slots: entries not yet in the game, and the turret's slot | no | F, J2, S2 |
| `Q0082.md` | Supplies gathering: the reaches, the nearest Tunnel, when the old load goes | no | I (running) |
| `Q0083.md` | Area damage: friendly fire, the reveal, Attack Ground's end, drawing | no | O, U |
| `Q0084.md` | Laser Turret: hidden when, sight, blocking, payment and refunds, targeting, the Agent's exit | no | S1, S2 |
| `Q0085.md` | Stale lines the answers left in the docs | no | — (doc cleanup) |

---

## 6. Risks

| # | Risk | Mitigation |
|---|---|---|
| 1 | **Units off the map (C).** Systems that count held units can miss them. | As adopted (T30). B's `used` iterator, and C chaining into it. |
| 2 | **Crowds at Side A.** | As adopted. 0246 dropped the fixed hold per unit, which eases it. |
| 3 | **The shared Gather job.** | Done in D: the Colonists' goldens and oracle didn't move. |
| 4 | **The line at the hatch under hard collision.** | **Realised (Q0076):** three Agents jam at 0–1 GU. L reports the affected rows as blocked, and layouts keep the patch 3 GU or more away until the answer. |
| 5 | **Hidden information leaks.** Placement refusals, ghost colours, paths, records and pointers; **and now blasts, hit and death markers, HP bars, reveals** (O), and pop-ups rising (S2). | 0240 accepts the direct-placement leak. O tests that the shooter's owner learns nothing about a hidden victim. Reveals go only to the owners of damaged objects (T48). |
| 6 | **Oracle fidelity.** | 0259 settled the geometry. Q0076 remains. |
| 7 | **Design churn.** 0249–0282 arrived during this revision, 0282 among them, which moved every doc. | Quote 0282's paths. The citation test (R2) catches a moved doc. The Designer answers within the hour, so fold answers into briefs not yet launched. |
| 8 | **Golden churn.** | Hash new state only while set (m4-common). A and D changed no golden; R1 and R2 must not. |
| 9 | **Mandatory layouts.** | B's spec. |
| 10 | **Old ids back on the map.** | As adopted (T35's audit). |
| 11 | **Defeat edge cases**, now with turrets keeping a Syndicate alive. | H's and S1's tests; the ignored suite after H. |
| 12 | **Machine load.** | At most three builds per wave (§3.2). |
| 13 | **Files over 500 lines.** | Split first: B, D (done), R1 and R2 (§3.2). |
| 14 | **M4-R's breadth.** Two slices touch most files. A behaviour change hidden in a refactor would pass review. | Goldens byte-identical, with no re-bless allowed. A/B perf. R2 merges after R1. Nothing runs beside them. |
| 15 | **Combat determinism.** f32 geometry for overlaps and flight times. | `sc_sim::math` (libm) only. Ticks rounded once at launch. Goldens with test shooters. |
| 16 | **The citation test couples `cargo test` to the Designer's tree.** A doc moved or renamed by the Designer fails the build until the citations follow. | That is the point of the test. The Director updates citations when it processes a change entry that moves docs. The test names file and line. |
| 17 | **The pop-up turret's silent rules** (S2): hidden when, blocking, refunds. | Q0084's proposals, marked as placeholders. S2 is last on the critical path, so answers have time to arrive. |
| 18 | **Briefs written before M4-R** (B, C, I) add files in the old layout. | R1 moves them. Their new citations use 0282's paths (D's already do). |

---

## 7. QA

As adopted, with these changes.
- **The Syndicate oracle (L).**
  - 0259 settles the geometry: distances edge to edge to the hatch square; ±15%; exactly 12.5 SC/s per Tunnel; the
    0:34–0:46 opening; stacked Tunnels on opposite sides of a field.
  - **Q0076 is open:** L reports the opening and the 0–2 GU rows as blocked until it is answered.
- **Goldens (L).** `syndicate_opening` and `syndicate_vs_gdo`, with the patch 3 GU from the hatch until Q0076 is
  answered. `syndicate_vs_gdo` adds Grenadiers and turrets once U and S1 land, each re-blessed once, with the reason.
- **Soak (M).** As adopted, plus grenades and turrets, and the new invariants (T42).
- **Perf (M).** `agents_perf` as adopted, plus 40 Grenadiers shelling and 12 pop-up turrets; O's `projectiles_perf`.
  All by interleaved A/B runs.
- **R1 and R2.** No golden moves. `nav_perf` and `drones_perf` are unchanged within noise.
- **Scripted walkthrough (M).** The adopted 12 screenshots, plus:
  13. an Attack Ground blast over a hidden Hideout, from both players' views;
  14. a grenade in flight, clipped at the fog's edge;
  15. a Laser Turret firing;
  16. a pop-up turret risen at an enemy;
  17. the same turret lowered, in its owner's view.
- **`docs/playtest/m4.md` (M).**
  - Its Known placeholders:
    - the Optics Lab's and the Chemical Plant's stats;
    - the upgrade's 480 frames;
    - the decay rate;
    - the Grenadier's cost;
    - the turret's timings;
    - every open QD placeholder;
    - Q0076.
  - Its Controls: 0257's letters.
- **Ignored tests.** After H (defeat), after R1 (a refactor of everything), and at the end of M4.

---

## 8. Director to-dos implied by this plan

- **Questions.**
  - File the eight drafts in `scratchpad/m4/rev/questions/` as Q0077 onward, and replace the QD-slugs in this plan
    and in briefs with the numbers.
  - None is blocking.
- **`plan.md`.**
  - Replace the M4 entry with A–M, R1, R2, O, U, S1, S2, and the gated N3–N7.
  - Mark N1 and N2 as scheduled.
  - Add the data-file and hot-reload layer to the playtest-prep list.
  - Update the change rows:
    - 0249–0259 to their slices (§9);
    - 0260 to O, U, K, S1 and S2;
    - 0261 as void (replaced by 0262);
    - 0262–0267 to O, B, S2, T44 and N3;
    - 0268–0281 as gated (N3, N6, N7);
    - 0282 to R1 and R2.
- **Tell the running slices:**
  - **C:** what §3.3's "What later slices need from C, D and I" lists:
    - held units into B's `used`;
    - a removal counterpart to `take_off_map`, for H;
    - Eject independent of operations;
    - one "current tier" function;
    - new citations with 0282's paths.
  - **I:** cite 0282's paths. Name new constants with their design citation or a placeholder note, so that R1 moves
    rather than rewrites them. Q0082 asks the Designer about its placeholders.
  - **B:** `REV/B.md`'s citations now use 0282's paths. Q0078 is drafted. Q0076's answer may move
    its layout's patch to 3 GU.
- **M4-R.**
  - Brief R1 and R2 from §3.3 and T54, with 0282's table and the survey.
  - Launch both when B, C and I have merged. Merge R1 first, then R2.
  - Run the ignored suite after R1.
- **Every brief from W4 on.**
  - Quote the design with 0282's paths.
  - Give the slice T54's "where things go", and its ownership row in §3.2 as its "don't touch" list.
- **Decide** T44's change, and T45–T54.
- **Folding answers in.** As adopted: fold each answer into the briefs that haven't launched, and rewrite a
  placeholder only when its slice is open.

---
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
- **0238** (Q0062 point 2): superseded by 0242.
- **0239** (Q0062 points 3a, 3b and 3e, for F): every tile under an expansion must be Drillable. Plane, Rugged and Cliff are; Mountain and Water aren't. Own and neutral underground structures, and the Tunnel's own footprint, block it, as proposed.
- **0240** (Q0062 points 3c and 4, for E and F): surface buildings and underground expansions never share a tile, whatever the owner, neutral included. Units still walk over expansions. This replaces T32's "the surface methods ignore them".
  - **E: surface placement reads the underground truth.**
    - A direct placement (GDO) over a hidden expansion is red at once. The leak is accepted.
    - A worker-built ghost (the Deploy pattern, Agent-built Tunnels) judges by what the player knows, so it stays green, and the order fails on the worker's arrival.
  - **F:** no expansion under a surface building.
- **0241** (Q0062 points 3d and 5, for F): patches and Supply Delivery Stations above block expansions. Its "non-Drillable cells give no Tunnel Space" is superseded by 0242.
- **0242** (completes Q0062, for B, E and G): a Tunnel can be placed only if its whole Tunnel Area is on the map and Drillable.
  - An upgrade is blocked if the larger area would include a non-Drillable or off-map cell.
  - Every tunnel-region cell is usable, so Tunnel Space needs no special cases.
  - **E:** the ghost judges by what the player knows, and the final check on arrival uses the truth.
  - **G:** the upgrade check.
  - **B:** starting layouts and the editor's starting Tunnels must respect it. B's layout validation reports a breach.
- **0245** (Q0063 point 1, for C): after entering, a unit can exit only once a flat delay of 32 frames (placeholder) has passed. Each unit inside stores when it entered. Produced units have no delay (0246).
- **0246** (Q0063 point 7 and part of 2, for C):
  - Entering units walk into the entry path at their normal speed, single file, with normal spacing, like any one-unit-wide passage. There is no fixed hold per unit, which replaces the 8-frame proposal. 0225's "one unit at a time" means "one unit wide".
  - Units leave the map once they have walked into the Tunnel, so throughput follows unit speed and spacing.
  - Exits wait on Q0064.
- **0247** (Q0063 point 2, for C): entering uses the normal Failure rule, with no special waiting rule.
- **0248** (Q0063 point 2's "which half of Side A"; corrects 0224): the sides, clockwise from the top in the default orientation, are:
  - **A:** entrance and exit;
  - **B:** crystal hatch;
  - **C:** back wall, with no function;
  - **D:** Supplies hatch.

  The hatches sit next to the entrance. Side A's exit path is the half nearer B, and its entry path the half nearer D; both turn and flip with the Tunnel. This affects:
  - **A** (folded in): the hatch marks are on B and D;
  - **C:** the entry and exit spots;
  - **I:** Supplies are delivered at Side D, not C;
  - **D:** crystals at Side B, unchanged.

**Since the first revision's cut-off (0249–0282):**
- **0249** (C, G): inside the network, orders are cleared, groups and loads kept, and HP regenerates at 1% of MaxHP per
  second. An upgrading Tunnel admits and lets out by its old tier until the upgrade completes.
- **0250, 0251, 0253** (C): ground units walk out single file; the longest-inside unit first; the queue rules;
  auto-ejected units share the exit queue. Air doors are on the roof with their own rate (N6).
- **0252** (C, J2): a rally on a Tunnel stays inside; a rally on an expansion means the spot above it.
- **0254** (F, H, J2): expansions under construction: paid at the ghost; ConstructionHP; X refunds 75%; the Tunnel's
  X cancels its operation; Eject still works; destroyed with no refund if the Tunnel falls.
- **0255** (D, done): the Drone's rules, the nearest Tunnel, the 1-square hatch, waiting for ever.
- **0256** (I, running): shared stations; waiting at an empty station with a notification; switching loses the load.
- **0257** (J1, J2, F, G): the panel letters on the 4x3 grid; menus in grid order; Z back.
- **0258** (H, B, G, A): decay at 1% per second; explicit Attack only; reclaiming at completion; blocked cells;
  defeat removes network units; enemies see the tier.
- **0259** (L): the economy check's geometry and tolerance. Q0076 followed from D's measurements.
- **0260** (O, U, K, S1, S2): placeholders for the Laser Turret, the Grenadier and the Chemical Plant, for M4.
- **0261**: void, replaced by 0262.
- **0262, 0264** (O, B, A): area damage hits underground objects; high armour underground; the building armour rule
  (PointArmor n, FullArmor n × size).
- **0263** (T44, N3, N6): undeploy and held orders confirmed; the Transport is a DrillUnit with no attack; the
  Phantom's attack.
- **0265, 0266** (N3, T33): Transport passengers always count for Tunnel Space; a burrowed Transport passes through
  underground structures.
- **0267** (S2, T43): a Tunnel-built turret uses the operation slot; detection comes after the faction rosters.
- **0268–0273** (N7): the Specter, a single-payload bomber behind a Vault and a Chemical Plant, re-arming in the
  network. The Vault works network-wide (0269).
- **0274–0281** (N3, N6): the satellite, named Prism, behind the Orbital Command; Relay through Prisms for Laser
  Batteries.
- **0282** (R1, R2, every brief): the design docs split per object, with no design change; the user's request for
  organized, modular code, and the decision "Modules now, data files at playtest".

