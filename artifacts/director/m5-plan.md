# M5 plan: the Cults, and the first playtest (planning spike M5-P)

> **Status (2026-11-25): draft 1, for the Director to adopt as `artifacts/director/m5-plan.md`.**
> - **What M5 is.** Three things:
>   - the Cults, as far as the design gives their numbers;
>   - playtest readiness: every faction against every other from the start options;
>   - the data-files step ("Modules now, data files at playtest", the user through 0282).
>
>   M5 ends at the user's first playtest.
> - **Where it starts.** The plan is drawn on master `2d93bafb`, where M4-V and M4-S2 (the pop-up turret) have merged,
>   and assumes M4-M (QA, next) merges first. The user's order: M5 starts after M4's QA.
> - **Marks.**
>   - **Placeholder (QD-slug)** marks a rule a slice builds as a proposal until question draft D-n (§5) is answered.
>     The Director swaps the slug for the Q number when filing.
>   - **Silent** means the design says nothing about the point. The plan proposes a default and drafts a question.
> - **Question drafts.** 17 drafts, D1–D17, all non-blocking (§5), in the Director's format. Number them from **Q0092**:
>   Q0091 (the pop-up turret's details, M4-S2) is the last one filed.
> - **Hashing.** M4's rule still holds: new state is hashed only while it is set. No existing golden moves until a game
>   has a Cults object in it.

- **Date**: 2026-11-25. Read-only: no repository file was changed, no git state was changed, and cargo was not run.
- **Code read at**: `abb1a902` (M4-S1 merged), re-checked at `dd8b57ae` (M4-V merged: the client's event consumers
  and `SimEvent::Died`) and at `2d93bafb` (M4-S2 merged). What S2 changed that this plan uses:
  - `StructureDef::popup`, a new field on every structure def (`None` for the Cults: A);
  - `movement/yielding.rs::make_way`, the push of a footprint's own idle units off it (0296 point 3), which E extends
    to a new site's own builders;
  - `placement.rs`: pending worker builds reserve their sites in expansion placement too (0125, `reserved_sites`);
  - the armour test's building list (14 today) and its `EXCEPTIONS` list (empty today).

  Line counts are from `2d93bafb`.
- **Design read at**: `b8fd394b` (0303). The Cults docs are unchanged since `a4afd2da` (0302, Soldier → Militant);
  0303 touched only the Syndicate's Agent and Tunnel docs.
- **Sources:**
  - **Design:** every file in `factions/cults/` (overview, economy, augmentation, `structures/` and `units/`), plus
    the sections they rely on:
    - `core/entities.md`: Groupable, Building armor rule, Side lettering, Exit Doorway, ConstructionHP Rule, Placement
      Validation (Worker-Built Structures, Lingering build ghosts);
    - `core/tech_and_research.md` (Counting Rule, Unit Prerequisites, the Cults research exceptions);
    - `core/map.md` (Recruitable, DefaultTilePresets);
    - `core/camera.md > Starting Position`;
    - `core/game_rules.md`;
    - `core/map_editor.md > Starting Layouts`;
    - `systems/controls/selection_and_hud.md` (No smart-casting, Groupable, unfinished buildings, Shift while
      placing, PointerDisplayType, BasicCombatUnitInterfaceState);
    - `systems/controls/commands.md > Enter`;
    - `factions/colonists/units/drone.md > Gathering Drones pass through each other`;
    - `factions/gdo/structures/bunker.md`.
  - **Change entries** that name the Cults or the Recruit: 0002, 0008, 0012, 0071, 0072, 0079, 0080, 0081, 0083,
    0106, 0118, 0143, 0149, 0185, 0282, 0302. The Syndicate precedents cited below: 0229, 0233, 0250, 0252, 0255,
    0259, 0289, 0291, 0296, 0297, 0298, 0299, 0300 and 0303.
  - **Director artifacts:** `m4-plan.md` (the shape, T29–T54), `m3-plan.md`, `plan.md` (the M5 entry, rows 0071,
    0072 and 0302, the Built entries, the Technical follow-ups, the Questions table), `insights.md`, and
    `.claude/agents/director.md > Outbound: design questions`.
  - **Code:**
    - `ARCHITECTURE.md` ("Adding a faction", "Adding a unit or structure", Tunables);
    - the sim's `defs/` (the registry, `structures.rs`, `units.rs`, `syndicate.rs`, `leak.rs`, `test_defs.rs`,
      `tests/serde.rs`), `tuning/`, `construction/site.rs`, `orders/build.rs`, `orders.rs`, `sim/builds.rs`,
      `sim/sites.rs`, `network.rs`, `sim/network.rs`, `production.rs`, `economy/army.rs`, `scenario/starting.rs`,
      `scenario/mapdef.rs`, `map.rs` and `scenarios.rs`;
    - the tests that use the Cults as the unimplemented faction;
    - the client's `kinds/`, `command_ui/interface/templates/`, `placing.rs`, `cli/usage.rs` and
      `combat_effects.rs`;
    - `maps/dev_skirmish.json`.

> **Adopted by the Director (2026-11-25)**, from planning spike M5-P's draft 1. The draft follows unchanged, except
> that its question slugs (QD-…) now read as the filed numbers.
> - **Questions filed:** D1–D17 are **Q0092–Q0108**, in order (D1 = Q0092 … D17 = Q0108). None blocks.
> - **T66 accepted:**
>   - JSON merge patches over the built-in Rust rules;
>   - the client loads them and sends a recorded `SetRules`, and the sim swaps rulesets at a tick boundary;
>   - live reload in dev builds;
>   - goldens use only the built-in rules.
> - **X1 stays last** (W6, alone), as drafted. The Cults defs follow the existing pattern, so they convert
>   mechanically, and a solo wave before the first playable would delay the Cults for little gain.
> - **0304 folded in (the Gunner can miss), as a new slice, O3: TailDisjointed attacks** (Opus, W4, beside J1 and L;
>   after A).
>   - **The design:** `factions/cults/units/gunner.md > AttackAttributes`, `systems/combat.md > AttackType`. "The target's
>     position locks at the end of Aiming, and the shot lands there at the end of Firing"; "a target that moves in
>     that window is missed, and whatever unit is at the spot (any owner) is hit instead".
>   - **The tasks:**
>     - build the TailDisjointed attack (today it hits as FullyConnected does: the TODO in `combat/phases.rs`);
>     - give the Gunner Attack Ground, on its panel (every attack that can miss has it);
>     - take the Technical follow-up **"Target too close" for a location** (`systems/controls/behaviors.md > Target
>       inside MinRange`), since the Gunner has MinRange 2. That supersedes §1.2's last-but-one row.
>   - It comes before G, which makes Gunners trainable.
> - **The user decided (2026-11-25):**
>   - the playtest is **hot-seat**: one person plays both sides, switching with F2;
>   - it starts from the **command line**.
>
>   No AI and no start screen in M5. The Cults' scope for the playtest is Q0092, which the Designer takes to the user.
> - **Since the plan:**
>   - 0305 renamed the Acolyte the **Disciple**;
>   - 0306–0312 sketch its cloud spell, named **Shroud** (0312). They are a concept only, "not for the current playtest": gated N4;
>   - 0313 sketches the Cults' light vehicle, a buggy from the Chop Shop: gated N6, with the vehicles;
>   - 0314–0318 name it the Jackal and add the Sparrow glider. The Chop Shop and the Airstrip work like the Armory;
>   - **0319 gives placeholder numbers to every designed Cults building and unit** (it answers Q0092–Q0094 and Q0108 point 1), **so the gated slices and G are in M5.** Planning spike M5-P2 drafts their slices and the waves from W5, as an addendum to this plan;
>   - **T56 amended (the Director, 2026-11-25):** `Object::lineage` is one entry per Recruit a unit embodies (`Vec<ObjectId>`), not an `Option`. The Jackal consumes 2 Recruits, and "each point of cost [is] attributed to the originating hall of the respective Recruit" (`factions/cults/economy.md > Unit Control Tracking`). C builds it, and F2 concatenates the consumed Recruits' entries.
> - **M5 starts** after M4-M merges.

---

## 0. Summary

- **Scope.** M5 builds the designed core of the Cults, and makes the game ready for the user's first playtest:
  - **The Recruit:** the Cults' worker and the base of every Cults unit.
    - It gathers crystals into a Storage: 2 SC a trip, any number to a patch, and normal collision.
    - Recruits build every Cults structure, as a group, from outside the site.
    - It enters the Armory, and fights with a melee attack.
  - **Recruitment Halls:**
    - they make Recruits automatically and for free, from the Recruitable tiles they claim in a 10x10 area;
    - effectiveness scales both capacity (20) and production time (12 s);
    - Unit Control is tracked per hall, by lineage, and the HUD shows it as used / available.
  - **Cults construction:**
    - one building, any number of Recruits, and a speed in proportion to the number of builders;
    - progress stalls when nobody builds;
    - paid when the order is given; refunded in full before the start, and 75% on Cancel;
    - Assist, and Shift chains.
  - **The Storage, and the Armory:**
    - the Armory holds a pool of 10, trains serially (paid and a Recruit reserved at queue), releases out of side C,
      and has Eject All;
    - everyone inside comes out alive when it is destroyed.
  - **Militant and Gunner.** The Gunner, with MinRange 2, is trainable only behind a Gunworks (G, if D1 and D2 put
    one in).
  - **The rest of the faction:** the start (a Hall, a Storage, six Recruits, 0 SC), layouts on every Skirmish Default
    map, the panels and pointers, an economy oracle and goldens.
  - **Playtest readiness:**
    - every pairing of the four factions starts from `--factions` on every map, and plays;
    - the controls and the known placeholders are documented;
    - the user has a way to report findings.
  - **The data-files step (T66):**
    - one ruleset in the sim, swapped at tick boundaries;
    - JSON files that override it, reloaded while the game runs in dev builds.
- **Deferred:**
  - the Altar and augmentation, the Garrison, the Temple (research), the Cathedral and Acolyte, and the Shrine: their
    numbers are TBD (gated N1–N5);
  - the Chop Shop, Machine Shop, Airstrip and Scrap Hangar: vehicles and aircraft aren't designed (N6);
  - the Gunworks, unless D1 and D2 give it placeholders;
  - AI opponents, a start screen, art and audio (§1.2).
- **Slices.** 14 (G depends on D1 and D2), plus answer sweeps S1 and S2 as needed, in 8 waves. D is on Fable; the
  rest are on Opus.
  1. **A** (Cults foundation) ∥ **R** (playtest pairings)
  2. **C** (Recruitment Halls) ∥ **D** (Recruit gathering, Fable) ∥ **F1** (buildings that hold units)
  3. **B** (Cults start) ∥ **E** (Cults construction) ∥ **F2** (Armory training)
  4. **J1** (Recruit UI) ∥ **L** (oracle and goldens) → **first playable**
  5. **G** (Gunworks, if D1 and D2) ∥ **S1** (answers sweep) ∥ one gated N-slice whose numbers have arrived
  6. **X1** (the sim's ruleset), alone
  7. **X2** (data files and live reload) ∥ **S2** (late answers)
  8. **M** (M5 QA) → **the user's playtest**
- **First playable.** After A, B, C, D, E, F1, F2 and J1, at the end of W4.
  - `cargo run -- --factions cults,gdo` starts on a Hall, a Storage and six Recruits.
  - The Hall makes a Recruit every 12 s, up to 20. Recruits mine at about 1.45 SC/s each.
  - Four Recruits build a Storage in about 5 s, paid once; a group builds an Armory; the Armory trains Militants out
    of side C.
  - F2 switches to the GDO, to fight (§2).
- **Key decisions.**
  - **Reuse:** M4's held store, Enter, exit doorway, Gather job, `Construction`, Build order, `requires`,
    `change_kind`, `Production` and rallies.
  - **New:**
    - Unit Control by lineage, derived from the objects (T56);
    - Hall claims derived, with production counted in exact integers (T57);
    - one site with many builders and a derived group ticket (T59);
    - one "holds units" capability, with evacuation (T60);
    - training from a pool with reservations (T61);
    - one predictable search for spots around a footprint (T62);
    - no unimplemented faction left (T63).
  - **Decided now, built later:** per-unit stats for augmentation and the Garrison (T67).
- **Data files (T66), in one line.** JSON files, read by the client as merge patches over the built-in Rust rules and
  installed in the sim as a new ruleset generation at a tick boundary, through a recorded debug command. They reload
  live in dev builds. Goldens run on the built-in rules only. Placed in W6 and W7, after all the content.
- **Readiness: ready to start after M4-M.**
  - Nothing blocks.
  - D1 and D2 decide whether G runs.
  - D6–D9 shape C and E, which build the proposals meanwhile.

---

## 1. Scope

### 1.1 In M5

| Area | Content | Design |
|---|---|---|
| Recruit | Unit, LightInfantry, 24x24, HP 40, armour 0/0, sight 5, Groupable (0072), 4 su per frame, 180° per frame, Rugged bonus 50%. Melee: FullyConnected, Ground, 3 damage, Aim 2 / Fire 8 / Cooldown 1 / Reload 1, contact reach 8 su (0149). Unit Control cost 1. | `factions/cults/units/recruit.md` |
| Recruit gathering | Carry 2; mining 8 frames; drop-off 2 frames; both reaches 8 su. Any number of Recruits to a patch. Normal collision while mining (0106). About 1.45 SC/s at a 1-tile gap. The Storage: **Placeholder (Q0102)**. | `recruit.md > Gathering`; `factions/colonists/units/drone.md > Gathering Drones pass through each other` |
| Recruitment Hall | 4x4, AAAA, HP 1000, 1/16, sight 6, Groupable false; 300 SC and 1440 frames to build. A 10x10 area; first-built tile claims; effectiveness; capacity 20 and 192 frames per Recruit, both scaled. Free, automatic production; lineage; the HUD aggregation; what its destruction does. Panel: X Cancel Production, C rally, right-click rally. **Placeholders (Q0099, Q0100, Q0101).** | `factions/cults/economy.md > RecruitmentHall`; `recruit.md > Constructs` |
| Storage | 2x2, AAAA, HP 250, **1/6** (**Placeholder (Q0094)**), sight 3, Groupable; 100 SC and 320 frames. Many Recruits drop off at once. Info only. | `economy.md > Storage` |
| Cults construction | Groups build from outside, at a speed in proportion to the builders; a stalled site keeps its progress. Paid at the order, once for the group (0072). A full refund before the start; X refunds 75%. Assist; Shift chains (0071, 0072); lingering ghosts; ConstructionHP. **Placeholders (Q0097, Q0098).** | `economy.md > Cults Building Mechanics`; `core/entities.md > ConstructionHP Rule`, `> Worker-Built Structures`, `> Lingering build ghosts`; `systems/controls/selection_and_hud.md > Shift while placing` |
| Armory | 2x3, ABAC, HP 300, 1/6, sight 4, Groupable; needs a Storage; 200 SC and 480 frames. Entrance B, exit C, pool 10. Serial training, paid and a Recruit reserved at queue; a queue of 20. Eject All; X; C; Q Train Militant; W Train Gunner. **Placeholders (Q0104, Q0105).** | `factions/cults/structures/armory.md`; `core/entities.md > Exit Doorway` |
| Evacuation | Everyone inside a destroyed Armory comes out alive, unchanged, around its footprint, and its orders are refunded in full. Built once, for every future building that holds units. **Placeholder (Q0106).** | `economy.md > Evacuation on Destruction` |
| Militant | As the Recruit's body. FullyConnected, Universal, 4 damage, range 4, Aim 2 / Fire 2 / Cooldown 1 / Reload 3. 20 SC and 32 frames at the Armory. BasicCombatUnit panel. | `factions/cults/units/militant.md` |
| Gunner | As the Recruit's body. FullyConnected, Ground, 16 damage, range 5, **MinRange 2**, Aim 4 / Fire 2 / Cooldown 8 / Reload 8. 40 SC and 32 frames, behind a Gunworks. | `factions/cults/units/gunner.md` |
| Gunworks (G, if D1 and D2) | On placeholder stats, with no research: it unlocks the Gunner. | `factions/cults/structures/gunworks.md` |
| Prerequisites | The Armory needs a Storage, and the Gunner a Gunworks. The Cults counting rule: "the Recruits must have finished constructing the building". | `core/tech_and_research.md` |
| Unit Control | Per hall, by lineage. A trained unit costs one per Recruit consumed. Orphans count. The HUD shows (Σ usage + orphans) / Σ capacity. | `economy.md > Unit Control Tracking`, `> Cults Unit Control HUD Display`; `overview.md` |
| Start | A Hall, a Storage and six Recruits belonging to the Hall; 0 SC. The camera starts on the Hall. | `economy.md > Starting Conditions`; `core/camera.md > Starting Position` |
| Layouts | Cults layouts on every Skirmish Default map, and no faction left "not implemented" (T63) | `core/map_editor.md > Starting Layouts` |
| HUD and panels | Space Crystals; Unit Control used / available. The Recruit's panel (**Placeholder (Q0095)**) and the Construct menu (**Placeholder (Q0096)**). The Hall's and the Armory's panels; the Storage's InfoPanel; the site's Cancel. The Enter, GatherResources and ReturnResources pointers. The Recruitment Area display (**Placeholder (Q0107)**). | `overview.md > DisplayHud`; the panels in each doc; `selection_and_hud.md > PointerDisplayType` |
| Building armour | The Hall (1/16) and the Armory (1/6) follow the rule; the Storage is listed as the exception. | `core/entities.md > Building armor rule` |
| QA | A Cults oracle (**Placeholder (Q0103)**), goldens, perf, and a soak over every pairing | `economy.md > RecruitmentHall` (rationale), `> Storage` (rationale); `recruit.md > Gathering` |
| Playtest readiness | Every pairing from `--factions` on every map; the controls; the placeholders; a way to report findings | the user's goal (the brief) |
| Data files | One ruleset; JSON overrides; live reload in dev builds | 0282; T54; T66 |

### 1.2 Deferred

| Item | Why |
|---|---|
| Altar and augmentation | Numbers TBD: the Altar's size, sides, HP, armour, sight, capacity, order costs and times, and its queue limit. The Metabolism increment's speed and attack speed are TBD, and so is its mapping onto attack phases. It also needs per-unit stats (T67). Gated N1. |
| Garrison | Size, symmetry, HP, armour, sight, Groupable and the panel (with its eject commands) are TBD. So are whether an Acolyte casts from inside and whether sight changes. Gated N2. It will reuse F1 (holding, protection, evacuation), M4-S1 (structures that attack) and T67 (+1 range). |
| Temple, and Cults research | No technology is designed (its list, costs and times), and the staffing capacity is TBD. The rules are written (`core/tech_and_research.md > Cults — research is staffed by Recruits`). No research system exists yet in any faction. Gated N3. |
| Cathedral and Acolyte | Stats, spells, energy and the panel are TBD. Gated N4. |
| Shrine | Rituals and abilities are TBD. Gated N5. |
| Gunworks | Every stat is TBD, and so is its research. **G** builds it on placeholders if D1 and D2 say so; otherwise the Armory's W stays greyed ("Requires a Gunworks"), and Gunners exist only in tests and debug spawns. |
| Chop Shop, Machine Shop, Airstrip, Scrap Hangar | Vehicles and aircraft are "not yet designed". Gated N6, with the vehicle and air milestones. |
| AI opponents | Undecided (`plan.md > Later`). The playtest is hot-seat with F2 unless the user decides otherwise (§8). |
| A start screen | None is designed. The playtest starts from the command line (`--factions`, `--scenario`, `--map`) and from the editor's play-tests (§7, §8). |
| Art and audio | The user: art comes after the gameplay design is mostly final. Shapes, two-letter labels and the current HUD stay. Audio needs `alsa-lib-devel`. |
| "Target too close for a location" (a Technical follow-up) | The Gunner is FullyConnected, so CanTargetGround is false (`systems/combat.md`) and it can't Attack Ground. The follow-up stays with the first unit that can (the Artillery Rig). |
| Notifications, detection | Not designed ("after all faction units are done", 0267) |

### 1.3 What already exists (so the slices don't redo it)

**For the Cults' data:**
- **`Faction::Cults`** exists (`player.rs`, `Faction::ALL = [Gdo, Syndicate, Cults, Colonists]`), and so do the maps'
  Cults layouts (`scenario/mapdef.rs`, a `cults: Vec<LayoutObject>`, empty on every map).
- **Recruitable** is a tile-preset flag in `map.rs`, read from map files and hashed. Every default preset is Recruitable
  except Water, as `core/map.md` says. No map change is needed.
- **The "not implemented" faction:** `StartingConditions::of(Cults)` and `ArmyResource::of(Cults)` are `None`. The
  tests that use the Cults that way (line numbers at `abb1a902`):
  - sim: `tests/scenario_problems.rs:72`, `tests/scenario_files.rs:85,125,136`, `tests/scenarios.rs:281–306`,
    `tests/game_start/{start.rs:33–70, layout_rules.rs:64–68, skirmish_default.rs:11,106–170, main.rs:6}`,
    `src/tests/hash.rs:84`, `player.rs:225`, `economy/army.rs:243`, `scenario/starting.rs:288`;
  - client: `editor/layouts.rs:27`, `editor/layouts/tests.rs:63,137–140`, `hud/resources.rs:28,112–119,422–427`
    (`the_cults_show_space_crystals_only`), `cli/usage.rs:19`, `cli/tests.rs:98`.
  - **Note:** the `--factions` usage line still says "syndicate and cults start with nothing until implemented",
    which has been stale for the Syndicate since M4-B.
- **Cults notes in tests:**
  - `tests/gathering_pass_through.rs:5` names "Recruits mining a patch still bump into each other", which has no test
    yet (D's);
  - `tests/movement_collision/passing.rs:7` cites the Recruit's two-lane note.

**Machinery to reuse** (plan.md's Built entries):
- **Defs and organization (M4-R):**
  - `defs/<faction>/{units,structures}/<object>.rs`, a faction rules module `defs/<faction>.rs`, and one
    `kind_registry!` line per kind, in an order that feeds the hash and the serialized names;
  - serde-ready defs (`defs/leak.rs`; `defs/tests/serde.rs` round-trips every def);
  - `tuning/<system>.rs`: 62 constants in ten modules, used in 69 files;
  - client `kinds/<faction>/<object>.rs` behind the exhaustive `kinds::of`, templates per object, and the citation
    test.
- **Units off the map (M4-C):**
  - `Sim::held` with `Held { object, place, since, entered }` and `Place::{Network, Site}` ("Match it exhaustively");
  - `OrderKind::Enter`, and `step_entries`, which takes a unit in "in the tick it gets there" along an entry path;
  - Eject queues, and `take_off_map` / `put_on_map`;
  - M4-H's stranding: the opposite rule to evacuation.
- **Production (M2, M3):**
  - `Production { queue, progress, rally, blocked }` and `ProductionCommand::{Train, CancelProduction, SetRally}`;
  - `QUEUE_LIMIT` 20, the current unit included (0081);
  - group training by least work (0083), with full refunds;
  - the exit doorway: `ExitRule::{Side, Centre, Network}`, `Doorway`, waiting while the doorway is occupied;
  - 0143's start check (`Production::blocked`);
  - M3-R's rule that a rally gives the unit its right-click order.
- **Gathering (M3, M4-D):** one Gather job for every worker:
  - `WorkerDef { crystals: LoadDef, supplies, shares_patches, keeps_drop_off, search_radius, pass_through, builds }`;
  - `DropOff::{Anywhere, Hatches}`;
  - arrival within a reach of an object's edge;
  - the Drones' pass-through, which the Recruits don't get.
- **Construction (M4-E, F, S1, S2):**
  - the kind-generic `OrderKind::Build` (T51); `Sim::pending_builds`, derived from the orders (the lingering ghosts);
  - `SiteDef { price, frames, start_hp, builder_goes }`, `SitePrice::Crystals`, and `BuildPrice::cancelled` (75% for
    crystals);
  - `Construction { progress, builder }` with `Builder::{Held, Tunnel}`; `rule_max_hp`; `transform::follow_max`;
  - `step_sites` (one frame a tick) and `cancel_site`;
  - the unfinished-building group (0185) and `SITE_GRID`;
  - `spot_beside` / `beside_order`: eight spots in a fixed order from the side a unit came from (0298 point 7);
  - M4-S2's footprint push (`movement/yielding.rs::make_way`): a footprint asks its owner's idle units over it to
    step off, through the idle push (0296 point 3);
  - the reservation of a player's pending builds' sites (0125, `Sim::reserved_sites`);
  - `check_build`, which today requires exactly one builder (`Reject::NoWorkers`, `Reject::SeveralConstructors`;
    a Shift-queued second Build gives `Reject::ConstructorBusy`).
- **Prerequisites (M4-K):** `tech.rs`, `requires` on structures and recipes, `Reject::Requires`,
  `Conditions::planned`, and `tests/building_armor.rs` (the 14 buildings of today, and an `EXCEPTIONS` list, empty
  today, each entry with its reason).
- **Menus (M4-F, J2):** `defs::EXPAND_MENU` with `ExpandEntry::{Kind, Planned}` in fixed slots (0291); 11 tile
  slots per menu, Z being Back; `templates/planned.rs`.
- **Kind change (M3):** `Sim::change_kind`, used by transforms and deploys.
- **Combat (M3-L2, M4-O, M4-S1):** MinRange (`SimEvent::TargetTooClose`, test defs); melee contact at 8 su
  (`MELEE_CONTACT_GU`); structures that attack.
- **Client (M4-V):** one per-viewer event filter (`viewer_events.rs`). Its table test fails for a `SimEvent` variant
  without a row.
- **QA (M2–M4):**
  - `testing::{build_order, colonists, syndicate}`, the oracles and `docs/economy_oracle.md`;
  - goldens through the `Script` hook;
  - `skirmish_soak` and `soak_sweep`; the perf tests (`structures_perf`, `drones_perf`, `nav_perf`, and others);
  - `--commands`, `--input-script` and `--screenshot`;
  - `docs/playtest/m2.md` and `m3.md` (M4-M writes `m4.md`).

**Files near 500 lines** (split before adding to them), at `2d93bafb`:
- sim:
  - `sim/network.rs` 493 (F1), `orders/world.rs` 474 (E, if it adds there), `economy/gathering.rs` 452 (D);
  - `movement.rs` 447 (D, if a crowd fix goes there), `sim.rs` 440, `orders/travel.rs` 421;
  - `production.rs` 416 and `sim/producers.rs` 416 (C, then F2), `sim/gate.rs` 402.
- client: `command_ui/interface/commands.rs` 467, `interface/tests.rs` 450, `command_ui/resolution.rs` 450 (J1),
  `hud/resources.rs` 432 (C), `conditions.rs` 431.

---

## 2. First playable

After A, B, C, D, E, F1, F2 and J1, at the end of W4.

1. Run `cargo run -- --factions cults,gdo`. Player 1 is the Cults, in slot 1:
   - a Recruitment Hall ("RH"), its 10x10 area wholly on the map and Recruitable (100%);
   - a Storage ("SE") a 1-tile gap from the crystal patches, the design's best layout;
   - six Recruits ("RT");
   - 0 SC, and Unit Control 6/20;
   - the camera on the Hall.
2. The Hall makes a Recruit every 12 s (192 frames), out of the side facing its rally point (**Placeholder
   (Q0100)**).
   - With the Hall selected, right-click a patch: new Recruits then gather by themselves (M3-R's default order).
   - The Hall reaches 20/20 at 2:48 and stops.
3. Select the Recruits (one group: they are Groupable) and right-click a patch.
   - Each mines 2 SC in 0.5 s, walks to the Storage and drops them off.
   - About 1.45 SC/s per Recruit at a 1-tile gap. They bump into each other (0106).
4. Select four Recruits, press **A** (Construct), choose the Storage, and place it.
   - 100 SC is deducted once.
   - All four walk there. The site appears at 10% HP when the first arrives, and is done about 80 frames after all
     four are there (0072's verification).
   - Select the site and press **X**: 75 SC comes back.
   - Shift-click two sites: the group builds one, then walks to the next.
5. Build an Armory (200 SC). It is greyed, "Requires a Storage", until a Storage is complete.
   - Right-click it with Recruits: they walk to side B and go in.
   - Select the Armory and press **Q**: 20 SC, and a pooled Recruit is reserved. A Militant ("MT") comes out of side
     C 2 s later and walks to the rally.
   - **W** is greyed: "Requires a Gunworks".
   - **A** ejects the pool.
6. Training doesn't change Unit Control: the Militant counts 1 against its Recruit's Hall.
   - Build a second Hall (300 SC; 90 s with one Recruit, about 22 s with four). Unit Control's available rises by its
     claimed tiles, and it produces as well.
7. Press **F2** to play the GDO: build Peacekeepers and attack.
   - Militants shoot, and Recruits fight back in melee.
   - Destroy the Armory: the Recruits inside come out alive around it, and its queued training is refunded.
   - Destroy every Cults building: the Cults are defeated, and their units turn neutral.

**Not needed for it:**
- G, L, R's extension to the Cults, X1, X2 and M;
- the Gunworks, and the gated N-slices.

**Earlier checkpoint (after W2, scripted).** A Custom QA map (`qa/maps/cults_economy.json`), since the Cults have no
Starting Conditions before B. It holds a Hall, a Storage and six Recruits as plain spawns. A `--commands` file:
- the Hall produces, and its rally sends Recruits to the patches;
- ten Recruits enter an Armory spawned on the map, and come out with Eject All.

Screenshots show the bank rising, Unit Control 6/20 → 12/20, and the pool count.

**Later checkpoints.**
- **After W3:** `--factions cults,<any>` starts from the layouts, and Cults construction works from scripts.
- **After W5:** Gunners, if G runs.
- **After W7:** the same game with `--tuning <dir>` changes while it runs.

**How it is played without an AI.** As in M2–M4:
- hot-seat with F2;
- scripted QA (`--commands`, `--input-script`, screenshots at fixed ticks);
- goldens through the `Script` hook;
- the Cults oracle (L).

---

## 3. Slices

### 3.1 Overview

| # | Slice | Content | Design | After | Model |
|---|---|---|---|---|---|
| A | Cults foundation | The six designed kinds, each in its own module: Recruit, Militant, Gunner, Recruitment Hall, Storage and Armory. Base stats, attacks, footprints and `requires`; registry lines last. `Object::lineage`. Client kinds (RT, MT, GU, RH, SE, AY). The armour test lists the Storage. | the six docs; `core/entities.md > Building armor rule` | M4-M | Opus |
| R | Playtest pairings | Every pairing of the factions that have Starting Conditions, on every Skirmish Default map: it starts, plays a scripted opening, holds the invariants, and can end. The client starts each from `--factions`. `docs/playtest/README.md`. | the user's playtest goal; `core/game_rules.md` | M4-M | Opus |
| C | Recruitment Halls | Derived claims; effectiveness; capacity; usage by lineage, held units included; orphans. Free automatic production, in exact integers; the release side; rally; X. Cults Unit Control and its HUD figure; the Hall's panel and InfoPanel. | `economy.md > RecruitmentHall`; `overview.md` | A | Opus |
| D | Recruit gathering | The Recruit's worker data and the Storage's drop-off; the nearest Storage; collision kept. Throughput measured against the design's formula; jams; `recruits_perf`. | `recruit.md > Gathering`; `economy.md > Storage` | A | **Fable** |
| F1 | Buildings that hold units | `StructureDef::holds`; `Place::Building`; Enter at the entrance side through M4-C's entry path; capacity; Eject All through the exit doorway; evacuation; spots around a footprint (T62) | `armory.md` (sides, capacity, Eject All); `economy.md > Evacuation on Destruction`; `commands.md > Enter` | A | Opus |
| B | Cults start | Starting Conditions (total), and the Recruits' lineage to the starting Hall. Layouts on every Skirmish Default map; the editor; `--factions cults`; the tests that used the Cults as unimplemented, rewritten (T63). | `economy.md > Starting Conditions`; `core/camera.md`; `core/map_editor.md > Starting Layouts` | A | Opus |
| E | Cults construction | A group Build, paid once, as a derived ticket. The site starts at the first arrival. Builders count at their spots, adding n frames a tick; a site with none stalls. Assist (a new order); Shift chains; refunds; the 75% cancel; the Recruit's `builds`; the sites' defs. | `economy.md > Cults Building Mechanics`; `recruit.md > Constructs`; 0071, 0072 | A, F1 | Opus |
| F2 | Armory training | `Production` with reservations; completion by `change_kind`, keeping the lineage; release from the held store through side C; the rally; X; group training. The Armory's panel and InfoPanel. | `armory.md > Training`, `> ObjectInterfaceState[Armory]` | C, F1 | Opus |
| J1 | Recruit UI | Panel; Construct menu with fixed slots; `Builder::Recruits` with Shift; Assist targeting; right-click; pointers; the Recruitment Area display; group ghosts | `recruit.md > Recruit ObjectInterfaceState`; `selection_and_hud.md` | C, D, E, F1 | Opus |
| L | Cults oracle and goldens | Throughput by gap, Hall fill time, second-Hall timing, the comparison with the GDO and the Colonists; `cults_opening`, `cults_vs_syndicate`; the Cults' scripted opening in R's test | `economy.md` (rationales); `recruit.md > Gathering` | B, C, D, E, F2 | Opus |
| G | Gunworks (if D1 and D2) | Placeholder stats; the Gunner's recipe behind it; the menu entry made live | `structures/gunworks.md`; `armory.md` | F2, J1 | Opus |
| S1, S2 | Answer sweeps | Answers whose slices have merged, each at most about 3 h | the answers | as needed | Opus |
| X1 | The sim's ruleset | One `Ruleset` (defs, bases, faction rules, tuning structs) in generations; `def()` and the tunables read through it; `DebugCommand::SetRules` at tick boundaries; goldens byte-identical | 0282; T54 | every content slice | Opus, alone |
| X2 | Data files and live reload | Export; JSON merge patches over the built-in rules; `--tuning <dir>` polled while running; refusals; a restart key; `docs/tuning.md` | 0282 | X1 | Opus |
| M | M5 QA | Perf, the all-pairings soak, the walkthrough, `docs/playtest/m5.md` (controls, placeholders, reporting), the ignored suite | `ARCHITECTURE.md > Adding a faction` | everything | Opus |

**Gated, not scheduled** (end of §3.3):
- N1 the Altar and augmentation;
- N2 the Garrison;
- N3 the Temple and Cults research;
- N4 the Cathedral and the Acolyte;
- N5 the Shrine;
- N6 the Chop Shop, Machine Shop, Airstrip and Scrap Hangar (with the vehicle and air milestones).

A gated slice moves into W5 when its numbers arrive by the end of W4 (D1).

**Names.** The playtest-pairings slice is **R**, because M5-P is this planning spike. The letters are per milestone:
M5-R is not M4-R.

### 3.2 Waves and parallel tracks

```
W1  A | R                                             (after M4-M merges)
W2  C (after A) | D (Fable, after A) | F1 (after A)
W3  B (after A) | E (after A, F1) | F2 (after C, F1)
W4  J1 (after C, D, E, F1) | L (after B, C, D, E, F2)   -> first playable after J1
W5  G (after F2, J1; if D1/D2) | S1 (answers) | one gated N-slice whose numbers came
W6  X1 (alone; after every content slice)              -> the ruleset; goldens byte-identical
W7  X2 (after X1) | S2 (late answers)
W8  M (after everything)                               -> the user's first playtest
```

- **Critical path:** A → F1 → F2 → L → X1 → X2 → M, and A → F1 → E → J1. The first playable comes after J1, at the end
  of W4.
- **Why F1 runs in W2.** E's builders stand at T62's spots, and F2 trains from F1's pool. Building the holding
  capability first lets both W3 slices start at once.
- **Why B runs in W3.** C, D and F1 test through `ScenarioBuilder` and Custom QA maps, which hold any objects, so
  they don't need the Cults' Starting Conditions. B's test rewrites touch many files, and fit better beside E and F2
  (sim systems) than beside C (`economy/army.rs`) in W2.
- **Why L runs before X1.** L's goldens are what prove X1 byte-identical for the Cults.
- **Why X1 runs alone, and last among the code changes.** It touches every use of a tunable (69 files) and the kind
  registry; that is the M4-R lesson ("a slice touching every file runs alone"). After all content, it converts every
  kind once, the Cults' included, and M's QA checks the shape that ships. §4 T66 gives the alternative: X1 first, in
  W1.
- **W5 is the content buffer.** It holds G, the answers that arrived during W1–W4, and at most one gated slice whose
  numbers came (D1). If none of these is needed, X1 moves up to W5.
- **Machine load.**
  - Three builds in W2, W3 and W5; two in W1, W4 and W7; one in W6 and W8.
  - One Fable build (D, in W2).
- **Shared registries.** These are edited by several slices. Edits are one-line and additive, and the waves are
  arranged so that no two parallel slices append to the same one:
  - **m5-common** (M4's m4-common, renamed): `sim.rs::step` (one call per new step), `command.rs`, `sim/gate.rs`,
    `testing.rs`, `object.rs`, `events.rs`, `state_hash` and `ARCHITECTURE.md`. Small additive edits only.
  - **The kind registry** (`defs.rs` and the client's `kinds::of`): A in W1 and G in W5, the only kind-adding slices.
  - **`OrderKind`:** only E (W3) adds an order, `Assist`. F1 reuses `Enter`.
  - **Commands:** F1 (W2) adds `HoldCommand::EjectAll`, and X1 (W6) adds `DebugCommand::SetRules`. C and F2 reuse
    `ProductionCommand`.
  - **`network::Place`:** F1 alone.
  - **`economy/army.rs`:** C owns it in W2. F2 (W3) changes one function (`taken_by`). B only reads it.
  - **The tuning modules.**
    - W2: D adds to `tuning/gathering.rs`, and F1 adds a new `tuning/holding.rs`.
    - W3: E adds to `tuning/construction.rs`.
    - C's numbers are the Hall's def's (capacity, time, area size), not tunables.
  - **The Cults' def modules: one field per slice.** Each system's slice adds its own field to the def:
    - D: the Recruit's `worker` (crystals) and the Storage's `drop_off`;
    - E: the Recruit's `worker.builds`, and every Cults structure's `site`;
    - C: the Hall's `production`;
    - F1: the Armory's `holds`;
    - F2: the Armory's `production`;
    - G: the Armory's Gunner recipe.

    E and F2 both edit `defs/cults/structures/armory.rs` in W3, in fields eight lines apart, which merges cleanly.
  - **`viewer_events` rows.** Every new `SimEvent` variant needs its row in M4-V's table; the table test fails
    without it. The slice that adds the variant adds the row.
- **Splits first** (T28). Each file near 500 lines is split before anything is added to it:
  - `sim/network.rs` (493): **F1**;
  - `economy/gathering.rs` (452): **D**;
  - `production.rs` and `sim/producers.rs` (416 each): **C**, if it adds more than about 80 lines, before F2;
  - `orders/world.rs` (474): **E**, if it adds there;
  - `movement.rs` (447): **D**, if a crowd fix goes there;
  - client `command_ui/resolution.rs` (450): **J1**; `hud/resources.rs` (432): **C**;
  - `sim.rs` (440): every new step goes to its own `sim/*.rs` file.

**What each slice owns** (for the "don't touch" lists). The shared registries above are open to every slice on the
terms given.

| Slice | Owns | Read-only |
|---|---|---|
| A | new `defs/cults.rs` and `defs/cults/{units,structures}/*.rs` (their base fields), the six registry lines, `Object::lineage` and its hash arm, new `kinds/cults.rs` and `kinds/cults/*.rs`, `tests/building_armor.rs`'s Cults rows | every system |
| R | new `crates/sim/tests/pairings.rs`, new `crates/game/tests/pairings.rs`, new `docs/playtest/README.md` | every sim and client source file |
| C | new `recruitment.rs` and `sim/recruitment.rs`; the Hall's arms in `production.rs` and `sim/producers/` (split first); `economy/army.rs`; the Hall's `production` field; client `hud/resources.rs` (split first), new `templates/cults/recruitment_hall.rs`, `conditions.rs` | `network.rs` (F1's) |
| D | the Gather job's Recruit arms in `orders/gather*`; `economy/gathering.rs` (split first); the Recruit's `worker` (crystals) and the Storage's `drop_off`; `tuning/gathering.rs` additions; new `tests/recruit_gathering/`, new `tests/recruits_perf.rs`; `movement/` only for a crowd fix, with A/B perf | `production.rs`; `network.rs` |
| F1 | new `holding.rs` and `sim/holding.rs`; `network.rs`'s `Place`; `sim/network.rs` (split first; entries generalized); new `placement/spots.rs` (T62, from `sim/sites.rs`'s `spot_beside`); the evacuation hook in `sim/removal.rs`; `HoldCommand`; the Armory's `holds`; new `tuning/holding.rs`; new `tests/holding/` | `production.rs`, `economy/army.rs` (C's) |
| B | `scenario/starting.rs`, `scenario/validate/`, `scenario/problems.rs`, `Scenario::start_objects`'s lineage, `scenarios.rs`, `maps/`; the tests listed in §1.3; new `tests/cults_start.rs`; client `cli.rs`, `cli/usage.rs`, `cli/tests.rs`, `editor/layouts*` | `economy/army.rs`, `hud/resources.rs` (C's) |
| E | `orders/build.rs`, new `orders/assist.rs`, `sim/builds.rs`, `sim/sites.rs`, `construction/site.rs` (`Builder::Recruits`, `BuilderGoes::Outside`), the Build and Assist arms of `sim/gate/`, `tuning/construction.rs` additions, the Recruit's `builds`, the Cults structures' `site`; new `tests/cults_construction/` | `placement/spots.rs` (F1's); `holding.rs` |
| F2 | `production.rs` (reservations), the Armory's arms in `sim/producers/`, `production/exit.rs` (releasing a held unit), `economy/army.rs`'s `taken_by`, the Armory's `production`; client new `templates/cults/armory.rs`; new `tests/armory/` | `holding.rs`, `sim/holding.rs` (F1's) |
| J1 | client new `templates/cults/{recruit,construct_menu}.rs`; `Builder::Recruits` in `placing.rs`; `command_ui/resolution.rs` (split first); `cursor.rs`'s Cults arms (and an Assist pointer); new `placement/recruitment_areas.rs`; `defs/cults.rs`'s `CONSTRUCT_MENU` | the sim, apart from `CONSTRUCT_MENU` |
| L | new `testing/cults.rs`, `tests/economy_oracle/cults.rs` and `tests/golden/cults.rs`; `docs/economy_oracle.md`; the Cults' script hook in R's `tests/pairings.rs` | everything else |
| G | new `defs/cults/structures/gunworks.rs` and its registry line; the Armory's Gunner recipe; `CONSTRUCT_MENU`'s entry; `kinds/cults/gunworks.rs`; W in `templates/cults/armory.rs` | `tech.rs` |
| S1, S2 | the placeholder sites the answers name, after their owners have merged | — |
| X1 | new `ruleset.rs`; every `tuning/<system>.rs` (consts → struct) and their users; `defs.rs`'s `def()` and `UnitBase::def()`; the faction rules constants; `DebugCommand::SetRules` and its gate arm | every hashed state layout (no golden may change) |
| X2 | client new `data/` (export, merge, poll, validate), `cli.rs`'s flags, the HUD's tuning marker, client `tuning/` (consts → structs), new `docs/tuning.md`, `ARCHITECTURE.md`'s data-files section | the sim |
| M | the perf and soak tests, the QA scripts, `docs/playtest/m5.md` and its generated tables, `ARCHITECTURE.md` | — |

**Every slice follows the Definition of done** (`ARCHITECTURE.md > Definition of done`), with these M5 additions:
- **No unimplemented faction (from B on, T63).** A test that needs a faction without something uses `test_defs`, never
  the Cults.
- **Held units count.** Anything that counts a player's units counts the held ones too (`Sim::held`). For the Cults,
  the count is by lineage (T56).
- **One field per slice** in the Cults' def modules (above). A number goes in its def, a tunable at the end of its
  system's module; each cites the design or names the QD it waits on.
- **R's pairing test stays green** in every slice. It is the cross-faction guard.
- **`viewer_events` rows** for new events (above).
- **Goldens.** New state is hashed only while it is set (T65). A slice that changes hashed behaviour re-blesses once,
  after its final `git merge master`, and explains every change. X1 may not re-bless.

### 3.3 Slice details

The quotes below are the design text the briefs should carry word for word (insights: "Quote design text in briefs;
don't paraphrase it"). **Placeholder (QD-slug)** marks a proposal built until the draft is answered.

#### A: Cults foundation (Opus)

**Goal.** The six designed Cults kinds in the game's model, with their numbers and their client kinds, and the
lineage field, so that the later slices add systems only.

**Design**
- `units/recruit.md`:
  - "UnitBase - LightInfantry", "Silhouette - 24x24 space units (square)", "MaxHP - 40", "PointArmor - 0", "FullArmor -
    0", "SightRange - 5 grid units", "Groupable - true";
  - "TurnRate: 180 degrees/frame", "MaxSpeed: 4 space units/frame", "RuggedTerrainDefenseBonus: 50%";
  - its attack: "AttackType: FullyConnected (Melee)", "TargetDomain: Ground", "Damage: 3", "Range: melee (adjacent
    contact)", "AimDuration: 2 frames", "FiringDuration: 8 frames", "CooldownDuration: 1 frame", "ReloadDuration: 1
    frame".
- `units/militant.md`: "(Renamed from "Soldier", user, 2026-11-26.)"; the same body; "AttackType: FullyConnected",
  "TargetDomain: Universal", "Damage: 4", "Range: 4 grid units", "MinRange: 0", Aim 2 / Fire 2 / Cooldown 1 / Reload
  3; "ObjectInterfaceState: BasicCombatUnitInterfaceState".
- `units/gunner.md`: "TargetDomain: Ground", "Damage: 16", "Range: 5 grid units", "MinRange: 2", "AimDuration: 4
  frames", "FiringDuration: 2 frames", "CooldownDuration: 8 frames", "ReloadDuration: 8 frames".
- `economy.md > RecruitmentHall`: "Size - 4x4 grid units", "SymmetryType - AAAA", "MaxHP - 1000", "PointArmor - 1",
  "FullArmor - 16", "SightRange - 6 grid units", "Groupable - false".
- `economy.md > Storage`: "Size - 2x2 grid units", "SymmetryType - AAAA", "MaxHP - 250", "PointArmor - 1", "FullArmor
  - 6", "SightRange - 3 grid units", "Groupable - true".
- `structures/armory.md`: "Size - 2x3 grid units (2 tall, 3 wide in the default orientation)", "SymmetryType - ABAC
  (top A, right B, bottom A, left C ...)", "MaxHP - 300", "PointArmor - 1", "FullArmor - 6", "SightRange - 4 grid
  units", "Groupable - true", "TechPrerequisite - player owns at least one Storage".
- `overview.md`: "Each Cult unit has a Unit Control cost". `economy.md > Unit Control Tracking`: "A trained unit's
  Unit Control cost equals the number of Recruits consumed to produce it".
- `core/entities.md > Building armor rule`: "PointArmor n, FullArmor n x (its size in grid squares) ... There may be
  exceptions."

**Placeholders**
- The Storage is built at 1/6, as written, and listed as an exception in the armour test, citing the draft
  (**Placeholder (Q0094)**). The rule gives 1/4.

**Code**
- **The defs.** `defs/cults.rs` (the faction's rules: docs only for now; J1 adds `CONSTRUCT_MENU`), and one module
  per object: `defs/cults/units/{recruit,militant,gunner}.rs` and
  `defs/cults/structures/{recruitment_hall,storage,armory}.rs`.
  - Each cites its design file and tests its numbers, as `the_agent_follows_the_design` does.
  - The registry lines go at the end, in the order Recruit, Militant, Gunner, RecruitmentHall, Storage, Armory.
- **Unit Control.** `unit_control_cost: Some(1)` on the three units (T55). `ArmyResource::of(Cults)` stays `None`
  until C.
- **Data only.** The Armory's `requires: [Storage]`, checked by M4-K's gate once E lets Recruits build.
- **System fields stay `None`:** `worker` (D, E), `drop_off` (D), `site` (E), `production` (C, F2) and `holds` (F1).
  `attack` and `popup` (M4-S1's and M4-S2's) are `None` on every Cults structure for good.
- **The lineage field.** `Object::lineage: Option<ObjectId>`: a serde default, skipped while `None`, and hashed only
  while set (T56). Nothing sets it yet.
- **The armour test.** Its building list grows from 14 to 17. The Hall and the Armory follow the rule; the Storage
  goes in `EXCEPTIONS`, with Q0094 as its reason.
- **The Gunner** is the first design kind with a MinRange. A test shows that it doesn't fire at a target inside 2 GU,
  and that it reports `TargetTooClose`, through M3-L2's code with no test def.
- **Client** (`kinds/cults.rs`, `kinds/cults/<object>.rs`):
  - labels RT, MT, GU, RH, SE, AY, by `Planned::label`'s rules (first and last letters; initials for two words). The
    Gunner's rule label, "GR", is the Grenadier's, so it is GU;
  - the round unit look the Peacekeeper (LightInfantry) has;
  - `ShotStyle::Tracer`;
  - templates: `BasicCombatUnit` for the Militant and the Gunner; `NONE` for the others until C, F2 and J1.

**Acceptance**
- Each kind's design test passes, and the serde round trip covers the six.
- The labels are unique (`kinds::tests::every_label_is_its_own`).
- The armour test passes.
- A debug-spawned Militant fights a Peacekeeper to a result.
- Every golden is unchanged.

**Owns**: see §3.2. **After**: M4-M. **Model**: Opus.

#### R: Playtest pairings (Opus)

**Goal.** From M5's first wave on, one test checks that every pairing of the factions starts and plays on every map,
so that each later slice keeps all of them green. Also the playtest docs' skeleton.

**Design**
- The user (brief): the playtest is "once all the 4 factions are complete", "with every faction against every other
  from the game's start options".
- `core/game_rules.md > Defeat Condition`: "a player is **defeated when they own no buildings**"; `> End of Game`:
  "When only **one player or team** is left undefeated, it **wins**."

**Code**
- **`crates/sim/tests/pairings.rs`.** For every Skirmish Default map (the built-in scenarios and `maps/*.json`), and
  for every ordered pair of factions with Starting Conditions:
  - It starts the game and runs a scripted opening per faction, through a per-faction script hook: the existing
    `testing::{build_order, colonists, syndicate}` helpers now, and L adds the Cults'.
  - It checks the invariants every tick.
  - It then removes player 2's buildings with a debug command and checks that player 1 wins.
  - There are 9 pairings now. After B there are 16, with no edit, since the table reads
    `StartingConditions::of`.
  - The default run is one map and 30 s of game time; an ignored variant runs every map for 3 minutes.
- **`crates/game/tests/pairings.rs`.** `--factions a,b --headless --exit-at-tick 64` for every pair: the start
  options' own path.
- **`docs/playtest/README.md`:**
  - how to start each pairing (`cargo run -- --factions syndicate,cults`, `--scenario`, `--map`);
  - hot-seat with F2, F3 for the fog, and replays saved to `replays/`;
  - how to report a finding (§7).

**Acceptance**
- 9 pairings pass on every map, and the test names each pairing it ran.
- The default run takes under a minute in a debug build.
- The headless client test passes for every pair.

**Owns**: see §3.2. **After**: M4-M. **Model**: Opus.

#### C: Recruitment Halls (Opus)

**Goal.** Halls that claim tiles, make Recruits for free up to their capacity, and track Unit Control by lineage; the
Cults' Unit Control in the HUD; the Hall's panel.

**Design** (`economy.md > RecruitmentHall`)
- "Automatically produces Recruits from the surrounding population of Recruitable terrain. Each Recruitment Hall
  operates independently with its own local capacity and production rate."
- "RecruitmentArea: A 10x10 grid unit area centered on the 4x4 building."
- "TileClaiming: A Recruitment Hall claims Recruitable tiles within its Recruitment Area that are not already claimed
  by another active Recruitment Hall. Tiles are claimed on a first-built basis — the hall that was constructed first
  has priority. This priority also applies when tiles are freed by the destruction of a competing hall; if multiple
  halls contest a newly freed tile, the one that was built first claims it."
- "Effectiveness = (number of claimed Recruitable tiles in Recruitment Area) / (total number of tiles in Recruitment
  Area)."
- "Buildings do not affect recruitability. ... A hall on fully Recruitable terrain with no competing halls therefore
  reaches 100% effectiveness."
- "Effectiveness scales both the hall's maximum Unit Control capacity and its Recruit production speed linearly."
  "BaseUnitControlCapacity - 20"; "BaseRecruitProductionTime - 12 seconds (192 frames) per Recruit"; "Example: at
  50% effectiveness, capacity = 10, production time = 24 seconds per Recruit."
- The instance state: "LocalUnitControlCapacity: number (BaseUnitControlCapacity * Effectiveness)",
  "CurrentProductionTime: number (BaseRecruitProductionTime / Effectiveness)", "ProductionProgress: number (frames
  elapsed) | None", "RallyPoint: Coordinates | ObjectInstance | None".
- "**Production Behavior**: The Recruitment Hall automatically and continuously produces Recruits at no resource cost
  until LocalUnitControlUsage reaches LocalUnitControlCapacity. Produced Recruits spawn at the hall and move to its
  rally point. Production logic is entirely local — global Unit Control state does not affect production decisions."
- "**Unit Control Tracking**: Each Recruit tracks which Recruitment Hall produced it. This lineage persists when a
  Recruit is trained into another unit. ... When a unit is destroyed, LocalUnitControlUsage decreases at the
  originating hall(s)."
- "**Cults Unit Control HUD Display**: Displays: (sum of all LocalUnitControlUsage across all active Recruitment Halls
  and orphaned units) / (sum of all LocalUnitControlCapacity across all active Recruitment Halls)."
- "**On Destruction**: LocalUnitControlCapacity is removed from the global sum; Surviving units produced by this hall
  remain on the field and continue to count toward the global usage sum; Claimed tiles are freed and may be
  reclaimed by other active Recruitment Halls based on build-order priority."
- Its panel: "Right-click Ground: issues SetRallyPoint command to that location"; "**X: Cancel Production**: cancels
  the current Recruit in production, resetting ProductionProgress. Only available if ProductionProgress is not
  None."; "**C: Set Rally Point**".
- `overview.md`: "Unit Control: current used / current available".

**Placeholders**
- **Q0099:**
  - off-map tiles count among the area's 100, and can't be claimed;
  - "constructed first" means completed first, and ties go by object id;
  - a Hall under construction claims and produces nothing;
  - capacity is compared exactly, and shown rounded down;
  - claims compete across players, by the letter.
- **Q0100:**
  - Recruits come out of the side facing the rally point, the bottom side when there is none, and the next free side
    clockwise when that doorway is occupied;
  - X sets the progress to 0, and production restarts at once;
  - the Recruit in production doesn't count toward usage until it exists;
  - at capacity, no Recruit is in production.
- **Q0101:**
  - no global check in M5;
  - Recruits placed by a Custom map or a debug command belong to no Hall: they are orphans.

**Code**
- **`recruitment.rs`** (new; T57):
  - `claims` is derived from the completed Halls in (completion tick, id) order, over each 10x10 area's Recruitable
    tiles;
  - `effectiveness`; capacity as an exact ratio;
  - `usage(hall)`: the units, on the map and held, whose lineage is the Hall;
  - `orphans`: the units whose lineage is a Hall that's gone, or none.
- **The Hall's production** reuses `Production` (progress, rally, the doorway, `CancelProduction` as X) in an
  automatic mode (T57):
  - the progress gains the Hall's claimed tile count each frame, and the Recruit is done at 192 × 100;
  - production starts while 100 × usage < 20 × claimed;
  - the new Recruit's lineage is the Hall.

  The completion tick goes in the Hall's state, which pre-built Halls have at 0, hashed only while set.
- **`economy/army.rs`:** `ArmyResource::CultsUnitControl`, named "Unit Control", and `of(Cults)`.
  - `available` is the sum of the completed Halls' capacities, rounded down.
  - `used` counts every unit with a Cults cost, held units included, orphans too.
- **Client:**
  - `hud/resources.rs` (split first): Unit Control as used / available, red when over;
  - `templates/cults/recruitment_hall.rs`: X, greyed without progress, and C; right-click sets the rally;
  - the InfoPanel: effectiveness in %, the local usage / capacity, the progress.

**Acceptance**
- **At 100%:** a Recruit every 192 ticks, 6 → 20 in 2688 ticks (168 s, the design's "2:48"), then none.
- **At 50 claimed tiles:** capacity 10 and 384 ticks per Recruit, the design's example.
- **Overlapping areas:** the first Hall completed keeps the shared tiles. When it is destroyed, the second takes them
  in the same tick, and its capacity rises.
- **A destroyed Hall:** its units still count in the HUD's used, and its capacity leaves the available.
- **Terrain:** Water tiles in the area lower effectiveness; buildings don't.
- **The rally:** a rally on a patch makes new Recruits gather (with D). X resets the progress.
- **Unchanged:** every golden, and the GDO's, the Colonists' and the Syndicate's production tests.

**Owns**: see §3.2. **After**: A. **Model**: Opus.

#### D: Recruit gathering (Fable)

**Goal.** Recruits gather crystals into Storages with the design's numbers, under normal collision, and their
throughput is measured against the design's figures.

**Design**
- `recruit.md > Gathering`:
  - "CarryCapacity: 2 Space Crystals per load", "MiningDuration: 8 frames", "DropOffDuration: 2 frames", "MiningReach: 8
    space units — a Recruit can mine once its Silhouette is within 8 su of the patch's edge", "DropOffReach: 8 space
    units — a Recruit can drop off once its Silhouette is within 8 su of the Storage's edge";
  - "Expected throughput: ... the best layout leaves a 1-tile (64 su) gap. With a 24 su Silhouette and 8 su reach on
    each side, a Recruit walks 24 su (6 frames) each way, so a round trip is 22 frames (6 + 8 mining + 6 + 2
    drop-off) ≈ 1.45 SC/s per Recruit. A 64 su gap fits two Recruits side by side (one lane in, one lane out).";
  - "Patch sharing: any number of Recruits may mine the same Space Crystals patch at the same time".
- `economy.md > Storage`:
  - "Multiple Recruits can drop off Space Crystals simultaneously.";
  - "each 2-tile face supports roughly 4–5 Recruits at full efficiency. Moving a Storage one tile closer (2-tile gap →
    1-tile gap) gains ~0.86 SC/s per Recruit".
- `factions/colonists/units/drone.md > Gathering Drones pass through each other`: "**Drones only.** Other factions'
  workers (e.g. Cults Recruits) keep normal collision while mining." 0106: "Recruits mining a patch still bump into
  each other."
- `recruit.md > Right-Click Resolution`: "Space Crystal patch → Gather", "Own Storage (carrying Space Crystals) → Drop
  off resources".

**Placeholders**
- **Q0102:** the nearest own completed Storage, edge to edge from the patch, chosen at each return. With
  none, the Recruit idles with its load.
- **The search radius for a new patch** when one runs out: the shared placeholder ("Placeholder (not in the design)").

**Code**
- **The data.** The Recruit's `WorkerDef`: crystals `LoadDef { 2, 8, 2, 8 su, 8 su }`, `shares_patches: true`,
  `pass_through: false`, no supplies. The Storage's `DropOff::Anywhere`.
- **The drop-off choice**, by Q0102, in the one Gather job. `economy/gathering.rs` is split first.
- **Measurements** (`tests/recruit_gathering/`):
  - one Recruit at gaps of 1, 2 and 3 tiles. By the design's formula (walk = gap − 24 − 16 su, at 4 su per frame)
    that gives 1.45, 0.59 and 0.37 SC/s;
  - 4 and 5 Recruits on one 2-tile face;
  - 10 Recruits on an 8-patch field with three Storages;
  - a jam check: a Recruit that delivers nothing for 10 s fails the test.
- **Perf.** `tests/recruits_perf.rs` (ignored): 60 Recruits on a field with four Storages, and three Halls producing.
- **If crowds jam:**
  - the fix goes in the Gather job, or in `movement/`;
  - A/B perf against master;
  - the Colonists' and the Syndicate's oracles and goldens must stay unchanged.

**Acceptance**
- The single-Recruit rates are within ±15% of the formula. L does the oracle's final check.
- A 5-minute soak has no jam.
- The other factions' oracles and goldens are unchanged; `drones_perf` holds by interleaved A/B.

**Owns**: see §3.2. **After**: A. **Model**: **Fable** (crowds under hard collision: the Q0076 precedent).

#### F1: Buildings that hold units (Opus)

**Goal.** One capability for Cults buildings that hold units. It is built for the Armory, and reused later by the
Altar, Temple, Cathedral, Shrine, Gunworks, Machine Shop and Garrison (and the GDO's Bunker, which "Works like the
Cults Garrison"):
- entering at the entrance side;
- a capacity;
- Eject All through the exit doorway;
- evacuation alive on destruction.

**Design**
- `systems/controls/commands.md > Enter`: "Order the unit to enter an own structure that can hold units. The unit
  walks to the structure's entrance side and goes inside." "**Cults — Armory, Cathedral**: Recruits only (to be
  trained), up to the building's internal capacity."
- `structures/armory.md`:
  - "EntranceSide - one short side (B)", "ExitSide - opposite short side (C)";
  - "Recruits enter through the entrance side. Trained units and ejected Recruits exit through the exit side, one at a
    time through the exit doorway (core/entities.md > Structure Type > Exit Doorway).";
  - "InternalRecruitCapacity - 10: The Armory can hold up to 10 Recruits in its internal pool at a time.";
  - "**Eject All**: Immediate command that ejects all stored Recruits from the exit side in a rapid stream, one after
    another. Ejected Recruits are unchanged — they return to the field as normal Recruits. This only affects
    StoredRecruits (the unreserved pool)".
- `economy.md > Evacuation on Destruction`:
  - "Applies to every Cults building that can hold units inside (Armory, Altar, Temple, Cathedral, Shrine, Gunworks,
    Machine Shop, Garrison, and any future such building ...)";
  - "**Everyone inside is evacuated alive**: units in the internal pool, units reserved by queued orders, the unit
    currently being trained or augmented, ...";
  - "**Units come out unchanged**: a Recruit mid-training comes out as a Recruit";
  - "**Placement**: evacuated units appear around the building's footprint wherever there is open space, unharmed.";
  - "**Orders are cancelled with a full refund**".
- `recruit.md > Right-Click Resolution`: "Own Armory → Enter Armory". `selection_and_hud.md > PointerDisplayType`:
  "Cursor over own enterable Cults building that accepts the selected unit (see Enter): **Enter**".
- `core/entities.md > Exit Doorway`, points 1–5.

**Placeholders**
- **Q0104:**
  - the entry path is at the middle of side B, as the Tunnel's is at Side A;
  - a full pool ends the Enter (the normal Failure rule);
  - a load is kept inside;
  - an unfinished building can't be entered;
  - the capacity counts the pool only.
- **Q0106:**
  - spots in rings around the footprint, clockwise from the exit side's middle;
  - the order: the unit in training, then the reserved ones in queue order, then the pool by time inside;
  - the units idle after.

**Code**
- **The def.** `StructureDef::holds: Option<HoldsDef { entrance: SideLetter, capacity: u32, accepts: &'static
  [ObjectKind] }>`, serde-ready. The Armory's is `{ B, 10, [Recruit] }` (T60).
- **The store.** `network::Place::Building(ObjectId)`, a new arm in the hash, which only Cults games use, and
  `Sim::held_in(building)`. The pool is the building's held units not reserved, ordered by `Held::since`.
- **Enter.** `OrderKind::Enter` is reused. `network::admits` is generalized to a holding building: own, operational,
  accepting the kind, with room in the pool. M4-C's entry path is set at the entrance side's middle (`Paths`
  generalized). `sim/network.rs` is split first.
- **Eject All.** `Command::Holding(HoldCommand::EjectAll)` queues the pool into the building's exit doorway (M2's
  `Doorway` on side C, one at a time, waiting while the doorway is occupied). The Recruits walk straight out until F2
  gives the Armory its rally.
- **Evacuation.** In the removal path (`sim/removal.rs`), every unit held in a removed building comes out at T62's
  spots, before defeat is judged, with a `SimEvent::Evacuated` (and its `viewer_events` row).
- **T62's spots** (`placement/spots.rs`, generalizing `sim/sites.rs`'s `spot_beside`):
  - rings of unit-sized spots around a footprint, each ring in a fixed order from a start side;
  - each spot checked by `check_unit_fits`, against the units placed before it.
- **Counting held units.** C's lineage usage already reads `Sim::held()`, so nothing changes there.

**Acceptance**
- Ten Recruits right-clicking an Armory go in one after another, with no jam (timed).
- The 11th Recruit's Enter fails, with feedback.
- A carrying Recruit keeps its load inside, and out again.
- Eject All streams the pool out of side C, one at a time.
- Destroying the Armory with ten inside puts all ten around it alive, in the fixed order, none overlapping anything.
- A building under construction refuses Enter.
- The Tunnel's Enter tests and every golden are unchanged.

**Owns**: see §3.2. **After**: A. **Model**: Opus. The entry line reuses M4-C's single-file path. If ten Recruits jam
at the entrance anyway, relaunch the slice on Fable (§6).

#### B: Cults start (Opus)

**Goal.** A Cults player can start a game: the Starting Conditions, layouts on every Skirmish Default map, the
editor, and `--factions cults`. After B, no faction is "not implemented".

**Design**
- `economy.md > Starting Conditions`: "Where each starting object is placed is defined by the map's Starting Layout for
  this faction"; "One pre-built **Recruitment Hall**"; "One pre-built **Storage**"; "Six **Recruits**, all belonging
  to the starting Recruitment Hall (they count toward its LocalUnitControlUsage ...)"; "**0 Space Crystals**".
- `core/camera.md > Starting Position`: "Cults: starting Recruitment Hall".
- `core/map_editor.md > Starting Layouts > Skirmish Default restriction`, `> Outdated layouts`. 0012's verification:
  "Leave the Cults layout incomplete: the editor lists the missing Recruits".
- `economy.md > Effectiveness > Map design guideline`: "balancing starts from the assumption that the whole map is
  Recruitable".
- `economy.md > Storage` (rationale): "The best layout leaves a 1-tile gap between patch and Storage".

**Code**
- **The Starting Conditions.** `StartingConditions::of` becomes total (T63): CULTS = [(RecruitmentHall, 1), (Storage,
  1), (Recruit, 6)], 0 SC, 0 Supplies, primary `RecruitmentHall`.
- **The lineage.** `Scenario::start_objects` gives the six starting Recruits their layout's Hall as lineage.
- **Layouts.**
  - On `maps/dev_skirmish.json`, mirrored by its Rotate180 symmetry: the Hall with its area on the map and
    uncontested, the Storage a 1-tile gap from patches, and the Recruits beside it.
  - On the built-in Skirmish Default scenarios, too (`the_maps_in_the_repository_are_playable`).
- **The tests** that used the Cults as the unimplemented faction (§1.3) are rewritten to test the Cults' layouts.
- **Client:**
  - `--factions cults`, and the usage line, which now names all four factions;
  - the editor's Cults layout, with its missing objects listed;
  - `cli/tests.rs`.

**Acceptance**
- `cargo run -- --factions cults,gdo` starts with the objects and 0 SC, the camera on the Hall.
- An incomplete Cults layout lists what it lacks.
- Every Skirmish Default map validates.
- R's pairing test runs 16 pairings.
- Every golden is unchanged: no golden has a Cults player.

**Owns**: see §3.2. **After**: A. **Model**: Opus.

#### E: Cults construction (Opus)

**Goal.** The Cults way of building:
- a group of Recruits builds one building together, from outside, faster with more builders;
- paid once, at the order;
- Assist, Shift chains and refunds.

**Design**
- `economy.md > Cults Building Mechanics`: "Cults buildings are constructed on-site by Recruits working from outside
  the structure. Assigned Recruits are not consumed — they remain on the field near the site for the duration of
  construction and are vulnerable to attack ... Construction speed scales proportionally with the number of Recruits
  currently assigned. Recruits can be individually reassigned to other orders at any time; removing a Recruit
  immediately reduces the assigned count and slows progress. If all Recruits are pulled off a site, progress stalls at
  its current value (no decay) until Recruits resume work or the building is cancelled. On completion, all Recruits
  that participated remain on the field as ordinary Recruits, free to be issued new orders."
- `> BaseConstructionTime Scaling`: "the time to complete construction with exactly 1 Recruit assigned. Effective
  construction time scales down linearly with the number of Recruits currently assigned — 2 Recruits halve it, 3
  Recruits reduce it to a third, and so on (no cap on this scaling other than the practical limit of how many Recruits
  can be assigned to a site)."
- `> Build Command`:
  - "All selected Recruits walk to the location and begin construction from outside the site";
  - "**Payment**: the building's Space Crystals cost is deducted as soon as the build order is given (the order
    requires enough Space Crystals at that moment), not when the Recruits arrive.";
  - "**Order cancelled before construction starts**: if the build order is cancelled before the building is placed
    (the Recruits are given another order before arriving, or placement fails validation on arrival), the full Space
    Crystals cost is refunded. Once construction has started, the order can no longer be cancelled through the Recruits
    — re-ordering them just removes them from the site (progress stalls, see above), and the building can only be
    cancelled through its own Cancel Construction command (75% refund ...)."
- `> Assist Construction Command`: "Left Click on an in-progress building"; "Once the selected Recruits reach the
  building they join construction from outside, increasing build speed".
- `> Cancel Construction`: "refunds 75% of the building's Space Crystals cost (rounded down), and destroys the
  partially-built structure ... Any Recruits assigned to the site are unaffected by the cancellation — they remain on
  the field and go idle".
- `recruit.md > Constructs`: "RecruitmentHall: 300 Space Crystals, 1440 frames (90 seconds) BaseConstructionTime (with
  1 Recruit)"; "Storage: 100 Space Crystals, 320 frames (20 seconds)"; "Armory: 200 Space Crystals, 480 frames (30
  seconds) ... Requires: Storage." Each structure: "ConstructionHP Rule - applies".
- 0072: "The cost is paid once per building, when the order is given." `selection_and_hud.md > Shift while placing`:
  "Cults Recruits (the whole selected group) build one and walk together to the next (each order's cost is paid when
  it is given ...)".
- `core/entities.md > Worker-Built Structures`: "a ground unit of any player on the footprint makes the site invalid;
  the building worker itself (e.g. the deploying Drone) doesn't count". `> Lingering build ghosts`: "Every **ordered
  worker build** (Drone Deploy, Agent build, Cults Recruits' Construct), including **every Shift-queued one**, leaves a
  **ghost of the building on its site** until construction actually starts."
- `core/tech_and_research.md > Counting Rule`: "**Cults**: the Recruits must have finished constructing the
  building."

**Placeholders**
- **Q0097:**
  - each builder stands at its own spot within 8 su of the footprint's edge (T62's first ring);
  - the site starts at the first arrival;
  - the order's own Recruits standing on the footprint step off it;
  - Recruits still walking don't count as builders;
  - builders keep building when attacked, as gatherers keep gathering;
  - a failed check at the first arrival fails the whole order;
  - Assist goes only to a site that has started.
- **Q0098:**
  - a full refund when the order's last holder drops it (or dies) before the start;
  - each Shift-queued building is paid and refunded on its own;
  - after a cancel, the builders go on to their queued orders.

**Code**
- **The data.**
  - The Recruit's `WorkerDef::builds` = [RecruitmentHall, Storage, Armory], with a new `shares_builds: true`: workers
    that "share a build" (T59).
  - Each Cults structure's `SiteDef`: `SitePrice::Crystals`, its frames, start HP 10%, and a new
    `BuilderGoes::Outside` (the builders never leave the map).
- **The gate.** A Build to several Recruits is one order, paid once. `Reject::SeveralConstructors` stays for Agents
  and Drones.
- **The ticket** (T59) is derived: the Build orders with the same (owner, kind, origin, orientation).
  `pending_builds` gives one ghost per ticket.
- **The start.**
  - The first arrival validates by the truth. Units block, except the order's own Recruits.
  - The site appears at 10% HP.
  - Every Recruit's current or queued Build of that ticket becomes `Assist(site)`.
  - The order's Recruits on the footprint step off it by M4-S2's push (`yielding::make_way`), extended from the
    owner's idle units to the site's own builders, then walk to their spots.
- **`OrderKind::Assist(ObjectId)`** (a new tag, hashed):
  - the Recruit walks to its own spot within reach of the site's edge (T62), then builds;
  - the order ends when the site completes or goes;
  - the gate: an own Cults site under construction.
- **The frames.** `Builder::Recruits` in `construction/site.rs`. `step_sites` adds one frame per Recruit on
  `Assist(site)` within reach that tick, exactly: none means a stall, with no decay. ConstructionHP comes from
  `rule_max_hp`.
- **Completion.** The Assist orders complete, and queued orders (the Shift chains) go on.
- **Refunds:**
  - in full, when a ticket's last holder drops it before the start (a new order, death, entering a building, a
    failed check);
  - none after the start;
  - X on the site gives 75%, rounded down (`BuildPrice::cancelled`);
  - a destroyed site gives nothing.

**Acceptance**
- **0072's verifications:**
  - four Recruits build a Storage in about 80 frames once all four are there, and the cost is deducted once;
  - Shift with two buildings: the group builds one, then the other;
  - a single Recruit chosen by its portrait builds alone.
- **0071's verification:** Shift-click 3 spots, and each click deducts the cost.
- **Builders:**
  - with every builder pulled off, the progress and the HP stay where they are; with one pulled off, it slows;
  - X refunds 75 of 100.
- **Refunds before the start:** re-ordering every Recruit before arrival refunds 100, and so does killing them all;
  after the start, nothing comes back.
- **The Armory's prerequisite:** refused (`Reject::Requires`) until a Storage is complete, not while one is under
  construction.
- **Unchanged:** a Drone's Deploy and an Agent's Build, with their tests and goldens.

**Owns**: see §3.2. **After**: A, F1. **Model**: Opus. If builders crowd at T62's spots and stall arrivals, switch to
Fable (§6).

#### F2: Armory training (Opus)

**Goal.** The Armory trains pooled Recruits into Militants (and Gunners, after G), paid and reserved at queue, one at
a time, out of side C; its panel.

**Design**
- `structures/armory.md > Training`: "Training is queueable and serial — orders are processed one at a time, in the
  order queued. Queuing an order immediately deducts its Space Crystals cost from the player and reserves one Recruit
  from StoredRecruits (removing it from the internal pool for the duration it sits in the queue). Only the front of
  the queue actively trains ... When an order finishes training, its reserved Recruit is consumed and the trained
  unit exits from the exit side. The trained unit inherits the Recruit's AugmentationState unchanged".
- "**Militant** (SpaceCrystalsCost: 20, TrainingTime: 32 frames)"; "**Gunner** (SpaceCrystalsCost: 40, TrainingTime:
  32 frames). TechPrerequisite: player owns at least one Gunworks."
- `> ArmoryInstanceState`: "TrainingQueue: array of {UnitType: Militant | Gunner, ReservedRecruit: Recruit reference}
  (max 20, including the order currently training ...)".
- `> Eject All`: "Recruits already reserved by a queued training order are not ejected and remain committed to that
  order."
- `> ObjectInterfaceState[Armory]`:
  - "**A: Eject All** ... Only available if StoredRecruits is not empty.";
  - "**X: Cancel Production**: removes the last entry from TrainingQueue, refunds its full Space Crystals cost to the
    player, and returns its reserved Recruit to StoredRecruits.";
  - "**C: Set Rally Point**";
  - "**Q: Train Militant** ... Only available if StoredRecruits is not empty, TrainingQueue has fewer than 20 entries,
    and player has sufficient Space Crystals.";
  - "**W: Train Gunner** ... and the player owns at least one Gunworks (TechPrerequisite)."
- `economy.md > Unit Control Tracking`: "This lineage persists when a Recruit is trained into another unit."
  `> Evacuation on Destruction`: "a Recruit mid-training comes out as a Recruit".
- 0083: with several producers, "those with the least work queued pay first, as many as the player can afford".

**Placeholders**
- **Q0105:**
  - the Recruit inside the longest is reserved;
  - the trained unit keeps the Recruit's damage;
  - ejected Recruits go to the rally;
  - trained units and ejected Recruits share the one doorway, in the order they are ready;
  - with several Armories, 0083's order, among those that have a free Recruit.

**Code**
- **The def.** The Armory's `ProductionDef`: `ExitRule::Side(C)`, and the recipe [Militant, 20 SC, 32 frames]. The
  Gunner's recipe waits for G. Until then the client's W is a `Planned` entry, greyed: "Requires a Gunworks".
- **Reservations** (T61). `Production::reserved: Vec<ObjectId>`, parallel to the queue: a serde default, hashed only
  while not empty.
  - Train reserves a Recruit; CancelProduction returns it.
  - A new refusal, `Refusal::NoRecruit`.
- **Completion.**
  - `Sim::change_kind(reserved, Militant)` keeps the id, the lineage, the damage (QD) and the owner.
  - The unit is released through the doorway from the held store, not spawned.
  - `taken_by` adds nothing for the unit in production of a producer that consumes a held unit: it already counts.
- **The rally.** `Production::rally`. F1's ejected Recruits go to it too.
- **Client** (`templates/cults/armory.rs`):
  - A Eject All, greyed with an empty pool; X; C; Q Train Militant; W, the planned Gunner;
  - the InfoPanel: the pool's count, the queue and the progress;
  - right-click sets the rally.

**Acceptance**
- With three Recruits pooled, Q pays 20 SC and reserves one; a Militant comes out of side C 32 ticks later.
- The queue stops at 20.
- X gives back the Recruit and the 20 SC.
- Eject All leaves the reserved Recruits inside.
- Unit Control doesn't change through training: the Militant counts against its Recruit's Hall. A Militant trained
  from a destroyed Hall's Recruit is an orphan.
- Destroying the Armory mid-training brings a Recruit out, and every queued entry is refunded in full.
- Group Q works on three Armories.

**Owns**: see §3.2. **After**: C, F1. **Model**: Opus.

#### J1: Recruit UI (Opus)

**Goal.** The player commands Recruits from the panel and the mouse: Construct (with Shift), Assist, Stop and Move;
the right-click as the design gives it; and the Recruitment Areas shown.

**Design**
- `recruit.md > Recruit ObjectInterfaceState`:
  - "**A: Construct** — enters BuildMenu (StateOnlyTransition). Multi-stage: select a building type, then place it on
    the ground (ghost preview, same placement flow as the Build Command).";
  - "**B: Assist Construction** — targeted command (StateOnlyTransition). Left-click an in-progress own building to
    join construction.";
  - "**Stop** — panel button".
- Its Right-Click Resolution: "Space Crystal patch → Gather", "Own Storage (carrying Space Crystals) → Drop off
  resources", "Own Armory → Enter Armory", "In-progress own building → Assist Construction", "Enemy unit/building →
  Attack (melee)", "Ground → Move".
- "Recruits are Groupable: all selected Recruits form one SelectionGroup, and both right-click and panel commands
  (Construct, Assist Construction) are issued to all of them. ... To build with a single Recruit, select it alone
  first (e.g. click its portrait)."
- `selection_and_hud.md`:
  - `> Shift while placing` (quoted in E);
  - `> PointerDisplayType`: Enter, GatherResources, ReturnResources;
  - "**Unfinished buildings form their own group**".
- 0291 (the precedent for fixed slots): "Unavailable ones are greyed, including ones not in the build yet."

**Placeholders**
- **Q0095:** Q Move, A Construct, S Assist Construction, X Stop; a new Assist pointer over an own site.
- **Q0096:** the slots.
- **Q0107:** placing a Hall shows its area and the tiles it would claim, judged by own Halls; a selected own
  Hall shows its claimed tiles; enemy areas are never shown.

**Code**
- **The menu.** `defs/cults.rs`'s `CONSTRUCT_MENU` is a data list with `Planned` entries, in the ExpandMenu's pattern.
  The client's menus go in `templates/cults/construct_menu.rs`.
- **The panel.** `templates/cults/recruit.rs`: its grid, and Assist's AwaitingTarget (left-click an own building
  under construction).
- **`Builder::Recruits`** in `placing.rs`:
  - the ghost, judged by knowledge (`Sim::build_check`);
  - a click sends the Build to the whole ActiveGroup;
  - Shift keeps placement while it is held.
- **Right-click.** `command_ui/resolution.rs` (split first): patch, Storage while carrying, Armory, site, enemy,
  ground.
- **Pointers** in `cursor.rs`: Enter over an Armory that accepts the selection, GatherResources, ReturnResources, and
  the placeholder Assist pointer.
- **The areas.** `placement/recruitment_areas.rs`, after `tunnel_areas.rs`.
- **Ghosts.** One lingering ghost per ticket (E's `pending_builds`). The carried load shows in the InfoPanel, as the
  Agent's does.

**Acceptance**
- **QA screenshots:**
  - the panel;
  - the menu, with its greyed planned entries;
  - a Shift chain of two ghosts;
  - the area overlay;
  - the Enter pointer over an Armory.
- **`--input-script`:** four Recruits build a Storage from the panel.
- **Tab** stops once on the Recruits (0072's verification 4).
- **Interface tests** for every letter; the labels stay unique.

**Owns**: see §3.2. **After**: C, D, E, F1. **Model**: Opus.

#### L: Cults oracle and goldens (Opus)

**Goal.** The Cults economy, checked against the design's figures; Cults behaviour, frozen in goldens.

**Design**
- `economy.md > RecruitmentHall` (rationale):
  - "At 12 s a starting hall (6 Recruits → 20) fills in 14 × 12 s = 168 s (2:48).";
  - "At 12 s the Cults stay roughly level with them until ~1:30 and pull slightly ahead by 2:48 (best case, all
    Recruits mining with ideal Storage placement)";
  - "the earliest second hall is ~0:55–1:15 at either 9 s or 12 s, because the 6 starting Recruits and saving the
    first 300 SC dominate".
- `recruit.md > Gathering`: "≈ 1.45 SC/s per Recruit". `economy.md > Storage`: "each 2-tile face supports roughly 4–5
  Recruits at full efficiency"; "~0.86 SC/s per Recruit" from a 2-tile to a 1-tile gap.
- The method's precedents:
  - 0259: "**Tolerance:** ±15%";
  - 0303: "The table is an ideal: with collision, real income varies a little below it ... 'It's better with some
    variation.'"

**Placeholders**
- **Q0103:**
  - the geometry: the gap from patch to Storage, edge to edge;
  - ±15%;
  - the targets: the rows above;
  - the opening's build line.

**Code**
- **`testing/cults.rs`:** a Cults opening:
  - every Recruit to the nearest patches;
  - save 300 SC, then four Recruits build a second Hall;
  - an Armory, and Militants.
- **The oracle.** `tests/economy_oracle/cults.rs`, and a Cults section in `docs/economy_oracle.md`.
- **Goldens:**
  - `cults_opening`: the Cults alone (production, gathering, a Storage, a second Hall, an Armory, Militants);
  - `cults_vs_syndicate`: Militants and Recruits against Agents, Guards, Grenadiers and a Laser Turret.
- **R's pairing test** gets the Cults' scripted opening.

**Acceptance**
- Each row passes within ±15%, or is reported with its measured value (the Q0089 precedent).
- The comparison with the GDO and the Colonists at 1:30 and 2:48, on an 8-patch field, is reported.
- The goldens are blessed once, with the explanation.

**Owns**: see §3.2. **After**: B, C, D, E, F2. **Model**: Opus.

#### G: Gunworks (Opus, if D1 and D2 put it in)

**Goal.** The Gunworks on placeholder stats, with no research, so that the Gunner can be trained.

**Design**
- `structures/gunworks.md`: "TechPrerequisite - player owns at least one Armory"; "Unlocks — **Gunner** training at
  the Armory"; "Size, SymmetryType, MaxHP, armor, SightRange, Groupable, Recruit staffing capacity, research queue
  size, and ObjectInterfaceState: TBD".
- `recruit.md > Constructs`: "Gunworks: TBD Space Crystals, TBD BaseConstructionTime. Requires: Armory."
- `armory.md`: W (quoted in F2).
- `core/tech_and_research.md > Unit Prerequisites`.

**Placeholders**
- **Q0093:** its numbers.
- No research and no staffing. A right-click on it with Recruits does what it does on any own building that holds
  nothing: Move.

**Code**
- `defs/cults/structures/gunworks.rs`, with its registry line, its `site` and `requires: [Armory]`.
- The Armory's Gunner recipe, with `requires: [Gunworks]`.
- `CONSTRUCT_MENU`'s entry goes from `Planned` to `Kind`.
- `kinds/cults/gunworks.rs` (GS); the Armory's W made live.
- The armour test's row; Gunners in `cults_vs_syndicate`, re-blessed once.

**Acceptance**
- W stays greyed, "Requires a Gunworks", until one is complete. Then a Gunner costs 40 SC and 32 frames.
- When the last Gunworks is lost, queued Gunners complete and no new one can be queued (M4-K's rule).

**Owns**: see §3.2. **After**: F2, J1. **Model**: Opus.

#### S1 and S2: Answer sweeps (Opus, as needed)

- **What they do.** Each applies the design answers whose slices have already merged. An answer replaces its
  Placeholder (QD-slug) marks, tests and docs.
- **Size.** At most about 3 h each. A bigger answer becomes its own slice.
- **Answers that arrive before a slice launches** go into its brief instead (insights).
- **When.** S1 in W5, for the answers of W1–W4; S2 in W7, for the later ones.

#### X1: The sim's ruleset (Opus, alone)

**Goal.** Every number the sim reads comes from one ruleset value: the defs, the unit bases, the faction rules and
the tuning structs.
- It is built in by default, and can be replaced at a tick boundary.
- Behaviour doesn't change.

**Design**
- 0282, from the user: "Once testing starts we want to experiment and make changes rapidly and accurately, so it's
  worth investing time now to keep things in order."
- The user's decision, "Modules now, data files at playtest"; `plan.md > Later`: "the defs and the `tuning/` modules
  load from data files, reloaded while the game runs ... a loading layer, not a refactor".

**Code** (T66)
- **`ruleset.rs`** (new): `Ruleset { defs, bases, factions, tuning }`.
  - `tuning` is a `Tuning` holding one struct per system: combat, construction, gathering, holding, movement, orders,
    paths, production, shields, tunnel and vision.
  - `Ruleset::builtin()` is generation 0, made from today's statics and constants.
- **Generations.** `static GENERATIONS: [OnceLock<&'static Ruleset>; 64]`, plus `static CURRENT: AtomicUsize`, behind
  `rules()`. There is no `unsafe` and no lock on reads. `ObjectKind::def()` and `UnitBase::def()` read it, test defs
  first.
- **The tuning modules.** Each `tuning/<system>.rs` turns its constants into one struct, with fields named as the
  constants were and their docs and citations kept, and a `const BUILTIN`.
  - Use sites read `rules().tuning.<system>.<field>`, once per step in hot loops.
  - Constants used in const contexts (array sizes) stay technical constants.
- **The faction rules.** Numbers such as `UNIT_CONTROL_CAP`, `BEACON_CAPACITY_*`, `TUNNEL_SPACE_CAP` and
  `CELLS_PER_TUNNEL_SPACE` move into `FactionRules`. The menu lists (`EXPAND_MENU`, `EJECT_MENU_ORDER`,
  `CONSTRUCT_MENU`) are structure, so they stay in code.
- **`DebugCommand::SetRules(Box<Ruleset>)`.**
  - The gate refuses it unless debug commands are on, and unless the ruleset is structurally the current one
    (`Ruleset::structural_diff`: the kinds, footprints, symmetry, sides, silhouettes, the bases' domains, the
    recipes' kinds, `holds`).
  - It is applied at the start of the next tick, with `SimEvent::RulesChanged`.
  - Objects keep their state. HP follows a changed MaxHP by `transform::follow_max`: the damage stays.
- **Replays** record the command like any other.

**Acceptance**
- Every golden is byte-identical; no re-bless is allowed.
- Interleaved A/B perf against master on `movement_perf`, `nav_perf`, `drones_perf`, `recruits_perf` and
  `structures_perf`: within noise.
- A test installs, through SetRules, a ruleset in which the Peacekeeper does other damage, and a duel changes from
  that tick on.
- A structural change is refused (`Reject::RulesStructural`).
- The ignored suite passes.

**Owns**: see §3.2. **After**: every content slice (W5). **Model**: Opus. It runs alone.

#### X2: Data files and live reload (Opus)

**Goal.** The user and the agents tune numbers in JSON files while the game runs.

**Code** (client; T66)
- **`crates/game/src/data/`** (new):
  - **Export.** The built-in ruleset as JSON files that mirror the modules: `defs/<faction>/{units,structures}/
    <object>.json`, `defs/<faction>.json`, `defs/bases/<base>.json`, `tuning/<system>.json` and
    `client/{camera,input,feedback}.json`. They are pretty-printed, in field order, so diffs stay small.
  - **Load.** Each `.json` under the tuning directory is a JSON merge patch (RFC 7386, applied on
    `serde_json::Value`) over its built-in counterpart; a full exported file is a valid patch. An unknown file or
    field is refused, with its path.
  - **Validation:** `Ruleset::structural_diff`.
- **Flags:**
  - `--export-data <dir>`;
  - `--tuning <dir>`, in dev builds with debug commands on. It is loaded before the first tick, as a SetRules at tick
    0. The directory is polled every 0.5 s for modification times (no new dependency), and a change sends SetRules.
- **Feedback:**
  - refusals in the feedback line ("tuning/combat.json: minimum_damage: expected a number");
  - "Tuned: N files" in a HUD corner while a patch applies;
  - F8 restarts the scenario with the current data, for structural changes.
- **The client's feel tunables** (`crates/game/src/tuning/`) become structs, read live from `client/*.json`.
- **`docs/tuning.md`:**
  - how to export, edit and reload;
  - how a tuned value reaches the design: the user tells the Designer, and the Director bakes the value into the Rust
    default with the change entry, re-blessing goldens with the data diff as the reason.

**Acceptance**
- **The round trip changes nothing:** export, then load the export unchanged, and the state hash matches a run without
  `--tuning`.
- **Live changes:** a patch mid-game changes behaviour from its tick on.
- **Structural changes:** a structural patch is refused with a message, and the game goes on.
- **Replays:** a replay of a tuned game reproduces it, with the same final hash.
- **Goldens** never read `--tuning`.
- **A QA run** with `--tuning qa/tuning/peacekeeper_damage/` changes a fight's outcome; the state dumps differ only
  from the patch's tick on.

**Owns**: see §3.2. **After**: X1. **Model**: Opus.

#### M: M5 QA (Opus)

**Goal.** The QA of M5, and the playtest kit (§7):
- perf, with interleaved A/B;
- the all-pairings soak;
- the scripted walkthrough;
- `docs/playtest/m5.md`, with its generated controls and placeholders tables;
- the ignored suite;
- `ARCHITECTURE.md` checked against what the slices did ("Adding a faction" now has no unimplemented faction; the
  ruleset; the tuning workflow).

**Owns**: see §3.2. **After**: everything. **Model**: Opus.

#### Gated slices (not scheduled; each waits for its numbers)

- **N1 Altar and augmentation.**
  - **Waits for:** the Altar's stats, sides, capacity, order costs and times, and queue limit; the Metabolism
    increment and how attack speed maps onto the attack phases.
  - **Reuses:** F1 (holding), F2 (reservations, with an augmentation entry), and the three-stage AugmentMenu on J1's
    menu pattern.
  - **Adds:** per-unit stats (T67) and `Object::augmentation` (hashed while set).
- **N2 Garrison.**
  - **Waits for:** its stats and panel.
  - **Reuses:** F1 (holding, protection, evacuation), and M4-S1 (a structure that attacks, here with its occupants'
    attacks).
  - **Adds:** T67's +1 range; the Recruits fire with the Militant's attack.
- **N3 Temple and Cults research.**
  - **Waits for:** the list of technologies, their costs and times, and the staffing capacity.
  - **Adds:** the game's first research system. Staffing is holding (F1), with E's linear scaling by the number
    inside.
- **N4 Cathedral and Acolyte:** spells and energy, which are new machinery.
- **N5 Shrine:** rituals, area abilities, and health regeneration.
- **N6 Chop Shop, Machine Shop, Airstrip, Scrap Hangar:**
  - with the vehicle and air milestones;
  - F2's reservation grows to several Recruits per unit;
  - the Unit Control cost follows from "one per Recruit consumed".

---

## 4. Technical decisions for the Director

T29–T54 (M4) still hold. New ones from T55 on.

### T55. Cults kinds in the M2–M4 model (A)
- **Recommend:**
  - **Modules and registry:** one module per object, each citing its file, and the registry lines last;
  - **Unit Control cost:** the Cults' is the Recruits consumed, kept in the existing `unit_control_cost` (1 for each
    infantry unit), with a test that each Cults unit's cost equals the Recruits its recipe consumes;
  - **System fields:** each one comes with the slice that implements its system (§3.2, one field per slice);
  - **MinRange:** the Gunner's uses M3-L2's code as it is.
- **Alternatives:**
  - a new `recruits_consumed` field, beside the cost: two numbers that must always be equal;
  - A filling every system field from the design: it would guess field shapes that C, D, E, F1 and F2 define.

### T56. Cults Unit Control: lineage on the object, counts derived (A, B, C, F2)
- **Recommend:**
  - **The field:** `Object::lineage: Option<ObjectId>`, the Hall that produced the unit's Recruit, hashed while set;
  - **Who sets it:** the Hall, at spawn; the layout, at game start (B). `change_kind` keeps it (F2).
  - **Usage:** derived each time it is read, never stored, as power and the other caps are (T7), from the map's
    objects and `Sim::held()`;
  - **Orphans:** units whose Hall is gone, or that never had one (map and debug spawns; **Placeholder
    (Q0101)**);
  - **The resource:** `ArmyResource::CultsUnitControl`, named "Unit Control". `available` is the sum of the completed
    Halls' capacities, `used` counts every Cults unit;
  - **No global check in M5.** Training is neutral, and Halls check locally. `fits` is never called for the Cults;
  - **No double count:** `taken_by` doesn't count the unit in production of a producer that consumes a held unit
    (F2).
- **Alternatives:**
  - stored per-Hall counters, updated on spawn, death and training: faster to read, but every death and every kind
    change must update them, so they can drift from the truth;
  - reusing `ArmyResource::UnitControl`: its `available` is the GDO's fixed 200, so every use would branch on the
    faction.

### T57. Recruitment Halls: claims derived, production in exact integers (C)
- **Recommend:**
  - **Claims:** a pure function of the completed Halls, sorted by (completion tick, id), and the map. Each Recruitable
    tile in a Hall's 10x10 area goes to the first such Hall. Off-map tiles count in the 100 but can't be claimed
    (**Placeholder (Q0099)**). The Hall stores only its completion tick (0 for pre-built ones).
  - **Cost of deriving:** about Halls × 100 cells per tick, which is trivial; a cache can be added if profiling asks.
  - **Production:** `Production` in an automatic mode. The progress gains the Hall's claimed tile count each frame,
    and the Recruit is done at `BaseRecruitProductionTime × 100`, so 24 s at 50 tiles comes out exactly, with no
    rounding. It starts while `100 × usage < BaseUnitControlCapacity × claimed`, also exact.
  - **Reuse:** the exit doorway, the rally (and M3-R's default order), `CancelProduction` as X, and the panel's
    progress.
  - **The release side:** the one facing the rally (**Placeholder (Q0100)**). It is a new arm of the
    exit rule for an AAAA building's "several exit-eligible sides" (`core/entities.md > Exit Doorway`).
- **Alternatives:**
  - claims stored as a tile → Hall map: it must be updated on every completion and loss, and gives the same answers;
  - effectiveness as a fixed-point ratio of `RATIO_ONE`: 655.36 per tile doesn't divide evenly, and the design's
    example would come out a frame off;
  - a production state of its own: it duplicates the rally, the doorway and the panel.

### T58. Recruit gathering on the one Gather job (D). On Fable.
- **Recommend:**
  - **The job:** the Recruit is data on the existing job (M4-D: `WorkerDef`, `DropOff::Anywhere`, `shares_patches`,
    no pass-through);
  - **The drop-off choice:** by Q0102, in the job's one place for it;
  - **Measurements** against the design's own formula (walk = gap − silhouette − 2 × reach, at 4 su per frame), at 1,
    2 and 3 tiles;
  - **Crowd fixes:** only if the measurements show a jam, with A/B perf and the other oracles unchanged;
  - **Why Fable:** 4–5 Recruits per face in a 64 su gap, under hard collision, is the Q0076 situation.
- **Alternatives:** pass-through for gathering Recruits, as for Drones. The design rules it out ("**Drones only.**").

### T59. Cults construction: one site, many builders, a derived ticket (E)
- **Recommend:**
  - **The ticket** of a group's order is its Build orders' shared (owner, kind, origin, orientation). Nothing new is
    stored.
    - Two tickets can't share a site, because the second ghost overlaps the first's reservation and is refused.
    - The ghosts, the refund "when the last holder drops it" and the start all read the orders.
  - **Paid once:** a Build command to several Recruits is one order. A new `WorkerDef::shares_builds` says which
    workers share a build: today the Recruit.
  - **At the start,** every Build of the ticket becomes `Assist(site)`. A Recruit still walking can then never start
    a second site after the first is lost.
  - **Builders are derived each tick:** the Recruits on `Assist(site)` within reach of the footprint's edge, counted
    as patch occupancy is. Progress gains one frame per builder, in `Progress`'s units, so "2 Recruits halve it"
    holds exactly.
  - **The builder kinds:** `Builder::Recruits` (no stored builder), and `BuilderGoes::Outside`.
  - **Spots:** each builder walks to its own spot from T62's first ring, so a group doesn't converge on one point.
  - **Assist** is a new `OrderKind` tag, hashed only while a unit carries it.
- **Alternatives:**
  - a stored ticket (`Sim::group_builds`) with members and the price paid: explicit, but it must follow every
    re-order, death and Shift queue;
  - builders as a stored list in `Construction`: it must be updated on every order change;
  - one site per Recruit, merged: not what the design says.

### T60. Buildings that hold units: one capability (F1)
- **Recommend:**
  - **The def:** `StructureDef::holds { entrance, capacity, accepts }`;
  - **The store:** `network::Place::Building(id)` in M4's held store;
  - **Entering:** Enter through M4-C's entry path at the entrance side, the unit going in "in the tick it gets there";
  - **The pool:** the building's held units not reserved, ordered by `since` (the longest inside first, as 0250
    ejects);
  - **Eject All:** through the M2 exit doorway;
  - **Evacuation:** in the removal path, before defeat is judged: the units are placed at T62's spots, unchanged,
    with their orders cleared. It is the opposite of M4-H's stranding.
  - **Held units** are counted (T56), protected and unseen, as the network's are.
  - **Later users:** the Altar, Temple, Cathedral, Shrine, Gunworks, Machine Shop and Garrison; the GDO's Bunker.
- **Alternatives:**
  - a pool list stored on the Armory: a second home for off-map units beside `Sim::held`, which every system that
    counts units would have to learn;
  - per-building code: three similar copies by N3.

### T61. Training from a pool: `Production` with reservations (F2)
- **Recommend:**
  - **Reuse `Production`:** its queue, progress, rally, refunds, the 0081 limit and 0083's group training;
  - **The reservations:** `reserved: Vec<ObjectId>`, parallel to the queue;
  - **Completion** by `change_kind` on the held unit, then a release through the doorway. The unit keeps its id,
    lineage and damage;
  - **Later:** the Altar's augmentation queue reuses the reservation, with its own entry type (N1).
- **Alternatives:**
  - a separate `PoolQueue` type: cleaner for the Altar, but it duplicates the panel, the refunds and the group rules
    now;
  - training by spawning a new Militant and removing the Recruit: the id changes, and the lineage and damage must be
    copied by hand.

### T62. Spots around a footprint: one predictable search (F1; E and evacuation reuse it)
- **Recommend:**
  - **The search:** M4-S1's `spot_beside` (eight spots, ordered from a side), generalized to rings of unit-sized spots
    around a footprint, each ring in a fixed order from a start side, each spot checked by `check_unit_fits` against
    the units placed before;
  - **One function, two uses:** evacuation, and the builders' spots. Builders on a new site's footprint step off by
    M4-S2's push (`yielding::make_way`), which is a movement request, not a placement;
  - **"Predictable, not optimal"** (0223).
- **Alternatives:** a nearest-free-cell flood fill. It is as good, but its order is harder to state in a design
  answer.

### T63. No unimplemented faction (B)
- **Recommend:**
  - **Total functions:** `StartingConditions::of` and `ArmyResource::of` return values, not `Option`s, and the "not
    implemented" paths go;
  - **Tests:** every test that used the Cults as unimplemented (§1.3) now tests the Cults' real layouts;
  - **A faction without something:** the tests that need one use `test_defs`;
  - **Maps:** every Skirmish Default map needs Cults layouts.
- **Alternatives:** keeping the `Option`s. Dead code that the next reader has to understand.

### T64. Client (A, C, F2, J1)
- **Recommend:**
  - **Kinds and templates:** a module per kind and a template per object, under `kinds/cults/` and
    `templates/cults/`;
  - **The Construct menu:** `defs::CONSTRUCT_MENU` as sim data with fixed `Planned` slots (the 0291 pattern);
  - **`Builder::Recruits`,** with Shift held;
  - **The Recruitment Area overlay** on the `tunnel_areas.rs` model;
  - **Unit Control** as its own `ResourceFigure`;
  - **Events:** every new `SimEvent` gets its `viewer_events` row.
- **Alternatives:** none worth listing.

### T65. Tests, hashing and splits (all slices)
- **Recommend:**
  - **Hashing:** new state is hashed only while set (`lineage`, the Hall's completion tick, `reserved`,
    `Place::Building`, Assist's order and behaviour tags);
  - **Splits first,** as listed (§3.2);
  - **R's pairing harness** from W1;
  - **The soak's new invariants:**
    - the sum of the Cults usages plus the orphans equals the Cults units alive;
    - held units are only in buildings that exist;
    - no unit overlaps a building;
    - a site's progress is never above its total;
    - reserved Recruits are held in their Armory.
- **Alternatives:** re-blessing every golden once per slice. Rejected in M4.

### T66. The data-files step (X1, X2), new
- **Recommend:**
  - **Format: JSON.**
    - `serde_json` is approved for the client, and the sim needs no new dependency, since the client parses.
    - One file per def module, per faction rules module, per unit base and per tuning module, in a tree that mirrors
      the code; pretty-printed in field order.
  - **Where the numbers live.** The Rust defaults stay the committed rules, with their design citations and design
    tests.
    - The data files are a tuning layer over them: each file a JSON merge patch, a full export included.
    - The user tunes without breaking `cargo test`, and the Director bakes the tuned values into the defaults once the
      Designer has recorded them (a change entry). This is the "loading layer, not a refactor" that `plan.md`
      promises.
  - **Loading.**
    - The client reads the tuning directory, applies the patches to the built-in ruleset's JSON, deserializes the
      result, and sends `DebugCommand::SetRules`.
    - The sim installs the new generation at the next tick boundary.
    - The sim never does I/O.
  - **Hot reload: yes, in dev builds with debug commands on.**
    - The directory is polled every 0.5 s by modification time.
    - Changes to structural fields are refused live, and need F8, the restart.
    - Numbers apply at once. Objects keep their state, and HP follows MaxHP with the damage kept.
    - A changed cost refunds at the new price (documented).
  - **Determinism.**
    - The swap happens only between ticks, in command order.
    - The ruleset travels inside the command, so replays reproduce tuned games.
    - The sim reads one generation for a whole tick.
    - Readers on other threads see an old or a new generation, both leaked and valid forever.
  - **Goldens stay meaningful.** They run on the built-in rules only and never read a tuning directory, and the state
    hash doesn't include the rules. A golden therefore fails only when behaviour changes. Baking a tuned value is an
    ordinary rule change: it re-blesses with the data diff as the explanation.
  - **The registry.** Process-wide generations: a fixed array of `OnceLock<&'static Ruleset>` and an `AtomicUsize`
    index, with no `unsafe` and no lock on the hot path. `test_defs` keeps its thread-local overrides. Tests never
    install a generation, except one dedicated test binary.
  - **Placement: X1 and X2 last, before M** (W6 and W7).
    - The user's words: "data files at playtest".
    - X1 touches every tunable's users and must run alone (M4-R's lesson). At the end it converts once, with every
      kind present, the Cults' included.
    - M's QA covers the shape that ships.
  - **Fable:** none. It's a refactor with byte-identical goldens and A/B perf, the M4-R pattern on Opus.
- **Alternatives:**
  - **Data files as the source of truth,** embedded with `include_str!`, with Rust keeping only the schema. One home
    for each number, but:
    - the citations move out of the code;
    - every design test fails while the user tunes in the checkout;
    - every def module changes.
    It may be worth it after the playtest, once the numbers settle.
  - **RON or TOML:** comments in the files, but a new dependency.
  - **A ruleset owned by the `Sim`,** passed down: the cleanest for determinism, but every def reader changes
    signature, which is a far bigger refactor.
  - **A thread-local ruleset:** breaks under Bevy's multithreaded executor.
  - **Reload only on restart:** simpler, but tuning a fight needs a new game each time.
  - **X1 first, in W1,** before A: no Cults tunable is written twice, and M5's own measurements could tune live. But
    it puts a solo wave before the first playable, and the user asked for it at playtest. Take this if the Director
    wants live tuning for D's and L's measurements.

### T67. Per-unit stats, for augmentation and the Garrison (N1, N2; decided now, built later)
- **Recommend:**
  - **Accessors:** when N1 lands, MaxHP, melee damage, speed, attack timing and range are read through accessors that
    apply the object's modifiers to its def's (the augmentation increments; the Garrison's +1 range and the Militant's
    attack for a Recruit inside);
  - **Locked levels:** the Cults' "not retroactive" research stamps its level on the unit at the moment that counts,
    as augmentation locks its level;
  - **X1's fit:** X1's ruleset holds the base numbers only, so modifiers stay instance state.
  - **Not built in M5.**
- **Alternatives:** a kind per combination of augmentations: the combinations multiply.

### T68. Follow-ups and non-goals
- **Recommend:**
  - **"Target too close for a location"** stays a follow-up, for the first ground-targeting unit with a MinRange: the
    Gunner can't Attack Ground;
  - **No Cults research in M5:** no technology is designed (N3);
  - **No AI:** the playtest is hot-seat, unless the user decides otherwise (§8).

**Fable pieces:** **D** (T58). Conditionally **F1** (the entrance line, if M4-C's path can't serve it) and **E** (the
builders' spots, if crowds stall arrivals). Fable costs about twice as much (insights).

---

## 5. Design questions

**Open before M5:** Q0090 (what a player perceives of events they don't see; M4-V) and Q0091 (the pop-up turret's
details; M4-S2), neither blocking. Neither touches M5.

**17 new question drafts**, below.
- **Format.** Each is in the Director's format: a `# QXXXX — Title` heading, the Date left as `DATE`, and options
  with their consequences.
- **Blocking.** None blocks. Each slice builds the proposal, marked "Placeholder (QD-slug)", and cites the draft
  until the answer comes.
- **Numbers.** File them from **Q0092**. Answers that arrive before a slice launches go into its brief; answers after
  it go to S1 or S2.

| Draft | Slug | Title | Blocking | Slices |
|---|---|---|---|---|
| D1 | Q0092 | Which TBD Cults buildings get placeholder numbers for the first playtest? | no (it decides whether G and N-slices run) | G, N1–N5 |
| D2 | Q0093 | Placeholder stats for the Gunworks | no | G |
| D3 | Q0094 | The Storage's armour, 1/6 against the rule's 1/4 | no | A |
| D4 | Q0095 | The Recruit's panel letters, and its pointer over a site | no | J1 |
| D5 | Q0096 | The Construct menu: 13 buildings, 11 slots | no | J1, G |
| D6 | Q0097 | Group construction: where Recruits build, when the site starts, who counts | no | E, J1 |
| D7 | Q0098 | Group construction: payment and refunds | no | E |
| D8 | Q0099 | Recruitment Area: off-map tiles, rounding, priority, Halls under construction | no | C, J1 |
| D9 | Q0100 | Hall production: the exit side, X, and counting the Recruit in production | no | C |
| D10 | Q0101 | Unit Control: the global rule, and Recruits that no Hall produced | no | C |
| D11 | Q0102 | Which Storage a Recruit drops off at | no | D |
| D12 | Q0103 | The Cults economy check: geometry, tolerance, targets, build line | no (L's pass or fail) | L |
| D13 | Q0104 | Entering the Armory: the spot, a full pool, loads, capacity | no | F1 |
| D14 | Q0105 | Armory training: which Recruit, HP, ejected Recruits, the doorway | no | F2 |
| D15 | Q0106 | Evacuation: where and in what order units come out | no | F1 |
| D16 | Q0107 | Showing the Recruitment Area | no | J1 |
| D17 | Q0108 | Stale lines in the Cults docs and the Enter command | no | — (doc cleanup) |

**The three most important:**
- **D1 (with D2):** what the Cults are at the playtest. The Gunner exists only with a Gunworks.
- **D6 (with D7):** the Cults' core mechanic, construction, has several silent points that E must build somehow.
- **D9 (with D8):** the economy engine: the Hall's exit, X, and effectiveness.

#### D1 — Q0092

````markdown
# QXXXX — Which TBD Cults buildings get placeholder numbers for the first playtest?

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (b), the Gunworks on D2's placeholder stats, and nothing else.
- **Docs**: `factions/cults/structures/README.md > Tech Tree Overview`; `factions/cults/units/recruit.md > Constructs`; `factions/cults/structures/gunworks.md`; `factions/cults/structures/garrison.md`; `factions/cults/augmentation.md > Altar`

## Question
The first playtest comes "once all the 4 factions are complete". Of the Cults, the Recruitment Hall, Storage, Armory, Recruit, Militant and Gunner have their numbers. Which of the TBD buildings should the playtest build have, on placeholder numbers, so that the user can try them?

## Context
The Recruit's Constructs list gives costs and times for three buildings. Every other entry is "TBD Space Crystals, TBD BaseConstructionTime". The Gunner needs a Gunworks ("TechPrerequisite: player owns at least one Gunworks"), whose stats are all TBD, so without one no Gunner can be trained. The Syndicate's Optics Lab and Chemical Plant went in on placeholders the same way (0229, 0260).

## Options
- **(a) Only the designed content.** The Cults play with Recruits and Militants, and the Armory's W stays greyed. No extra work.
- **(b) Add the Gunworks** on placeholder stats (next question), with no research. Gunners are in the playtest. One small slice (about an hour).
- **(c) Also the Garrison,** on placeholder size, HP, armour, sight and panel. Recruits and Militants fire from inside it. One more slice. It reuses the holding capability built for the Armory, and M4's structures that attack.
- **(d) Also the Altar and augmentation,** on placeholder size, sides, capacity, order costs and times, and Metabolism increments. One or two slices. Units would get stats of their own (every reader of MaxHP, damage and speed changes). The Cults' signature mechanic gets tried.
- **(e) Also the Temple, Cathedral, Acolyte and Shrine.** Their mechanics (research, spells, energy, rituals) must be designed first. Several slices; the playtest moves well past M5.

Proposed: (b). Add (c) and (d) if wanted, provided their numbers come before the content wave ends.
````

#### D2 — Q0093

````markdown
# QXXXX — Placeholder stats for the Gunworks

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: the stats in option (a), with no research and no staffing.
- **Docs**: `factions/cults/structures/gunworks.md`; `factions/cults/units/recruit.md > Constructs`; `core/entities.md > Building armor rule`

## Question
If the Gunworks is in the playtest (previous question), what are its placeholder stats? And does it hold Recruits in this build, which has no research?

## Context
"Size, SymmetryType, MaxHP, armor, SightRange, Groupable, Recruit staffing capacity, research queue size, and ObjectInterfaceState: TBD." Its cost and time are TBD in the Recruit's Constructs list. Its research ("Weapon upgrades (list, costs, BaseResearchTimes TBD)") isn't designed, so staffing would do nothing. The nearest designed buildings are the Armory (2x3, HP 300, 1/6, sight 4, 200 SC, 480 frames) and the Storage (2x2, HP 250, 100 SC, 320 frames).

## Options
- **(a)** 2x2, AAAA, HP 300, armour 1/4 (the building rule with n = 1), sight 4, Groupable; 150 SC and 480 frames with one Recruit; no panel commands; no staffing until research is designed. A right-click on it with Recruits moves them there.
- **(b)** The Syndicate's Optics Lab's numbers (2x2, HP 400, 150 SC, 400 frames), with a surface armour of 1/4.
- **(c)** Option (a) with staffing: Recruits can enter, as the Recruit's right-click list says ("Own Gunworks / Machine Shop → Enter (staff research)"), but nothing happens inside. That is confusing without research.

Proposed: (a).
````

#### D3 — Q0094

````markdown
# QXXXX — The Storage's armour: 1/6, or the rule's 1/4?

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: 1/6 as written, listed as an exception in the armour test.
- **Docs**: `factions/cults/economy.md > Storage`; `core/entities.md > Building armor rule`

## Question
The Storage is 2x2 with PointArmor 1 and FullArmor 6. The building armour rule gives 1/4 for a 2x2 building with n = 1. Is 1/6 an intended exception, or a leftover from the Storage's earlier size?

## Context
The rule: "A building's armor should generally be **PointArmor n, FullArmor n x (its size in grid squares)** ... There may be exceptions." The Storage's rationale says it went "2x2 (down from 3x2)". A 3x2 Storage with n = 1 is exactly 1/6, so the armour may simply not have followed the size. The Hall (4x4, 1/16) and the Armory (2x3, 1/6) both follow the rule.

## Options
- **(a) 1/4:** it follows the rule, and needs no exception.
- **(b) 1/6:** an intended exception (tougher against area damage). The armour test lists it, with this answer as the reason.

Proposed: (a). The build uses (b) until the answer comes, since that is what the doc says.
````

#### D4 — Q0095

````markdown
# QXXXX — The Recruit's panel letters, and its pointer over a building under construction

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: Q Move, A Construct, S Assist Construction, X Stop; a new Assist pointer.
- **Docs**: `factions/cults/units/recruit.md > Recruit ObjectInterfaceState`; `systems/controls/selection_and_hud.md > CommandPanel` (the grid), `> PointerDisplayType`; `factions/syndicate/units/agent.md > Agent ObjectInterfaceState` (0257)

## Question
1. The Recruit's panel gives "**B: Assist Construction**", but B isn't a slot of the 12-key grid (Q W E R / A S D F / Z X C V). Which letter does Assist take?
2. "Stop — panel button" has no letter, and Move has no panel button. Do they take X and Q, as on the Agent?
3. Right-clicking an own building under construction Assists, but no pointer type says so. Which pointer is shown?

## Context
The Agent's panel (0257): "**Q** Move, **A** Build Tunnel, **S** Drop Off Resources, ... **X** Stop", with Q Move and X Stop "as in BasicCombatUnitInterfaceState". The pointer types are Inactive, Move, Attack, AttackGround, Patrol, GatherResources, ReturnResources, Enter and Mixed.

## Options
1. Assist on **S** (the slot after Construct), **D**, or **F**.
2. **Q** Move and **X** Stop, as on the Agent; or no Move button (right-click only) and **X** Stop.
3. A new **Assist** pointer (placeholder look); or the **Move** pointer; or the **Enter** pointer.

Proposed: Q Move, A Construct, S Assist Construction, X Stop; a new Assist pointer. Each is one line in the client.
````

#### D5 — Q0096

````markdown
# QXXXX — The Recruit's Construct menu: 13 buildings, 11 slots

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (b), the base buildings in Q and W, and one sub-menu per branch.
- **Docs**: `factions/cults/units/recruit.md > Constructs`, `> Recruit ObjectInterfaceState` (A: Construct); `factions/cults/structures/README.md > Tech Tree Overview`; `factions/gdo/structures/deployment_center.md` (BuildMenu[Basic] and [Advanced]); `factions/syndicate/tunnels.md > ExpandMenu` (0291)

## Question
"A: Construct — enters BuildMenu". The Recruit constructs 13 buildings, but a menu has 11 slots (Z is Back). Which slots do the buildings take?

## Context
The Constructs list: Recruitment Hall, Storage, Armory, Altar, Temple, Cathedral, Shrine, Garrison, Gunworks, Chop Shop, Machine Shop, Airstrip and Scrap Hangar. The GDO's Deployment Center has two menus (Q Basic, W Advanced). The Syndicate's ExpandMenu lists every expansion in fixed slots, greyed when not available, "including ones not in the build yet", so "no hotkey ever moves" (0291).

## Options
- **(a) Two buttons, as the GDO has:** A Construct (Basic: Hall, Storage, Armory, Altar, Garrison, Gunworks) and a second panel button (Advanced: the rest). The Recruit's panel gains a letter.
- **(b) One menu with branch sub-menus:** Q Recruitment Hall, W Storage, E Tech/Weaponry ▸ (Armory, Gunworks, Garrison, Chop Shop, Machine Shop, Airstrip, Scrap Hangar), R Mystical ▸ (Altar, Temple, Cathedral, Shrine). Each sub-menu is in the Tech Tree table's order, in fixed slots. The Armory takes one more key press; there is room for 11 per branch.
- **(c) One flat menu** for the 11 earliest buildings, with the Airstrip and the Scrap Hangar added with aircraft. Slots would then move, against 0291's principle.

Proposed: (b). The unavailable entries are greyed in their slots, as in 0291.
````

#### D6 — Q0097

````markdown
# QXXXX — Group construction: where Recruits build, when the site starts, and who counts

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: the proposal under each point.
- **Docs**: `factions/cults/economy.md > Cults Building Mechanics`, `> Build Command`, `> Assist Construction Command`; `core/entities.md > Worker-Built Structures`, `> Lingering build ghosts`

## Question
Several rules of building in a group are silent:
1. **Where do builders stand?** "from outside the site", with no reach given.
2. **When does the site start:** at the first Recruit's arrival, or when all have arrived?
3. **Recruits of the order on the footprint.** "the building worker itself ... doesn't count" as a blocker. What happens to Recruits of the order who stand on the footprint when it starts?
4. **A failed check.** If the first arrival finds the site blocked, does the whole order fail, or does each later Recruit try again?
5. **Who counts as "assigned"?** Only the Recruits at the site, or also those still walking to it?
6. **Under attack.** Do builders keep building, or fight back?
7. **Assist targets.** Can Assist target a placed ghost that hasn't started yet, or only a building under construction?
8. **Limits.** Is there a limit to the builders, beyond room around the site?

## Context
The doc: "All selected Recruits walk to the location and begin construction from outside the site"; "Construction speed scales proportionally with the number of Recruits currently assigned"; Assist: "Once the selected Recruits reach the building they join construction"; "no cap on this scaling other than the practical limit of how many Recruits can be assigned to a site". Mining and drop-off use an 8 su reach of the object's edge. An Agent goes inside its site; a Drone becomes the Beacon. Recruits do neither. The nearest precedent for point 3 is the pop-up turret's rise: "The **owner's own idle units** over it are pushed off by the idle push ... Any other unit over it (an enemy, or an own unit with an order) keeps it **underground** until the spot is clear" (`factions/syndicate/structures/laser_turret.md > Pop-up rules`, 0296).

## Options
1. (a) Each builder at its own spot within 8 su of the footprint's edge, the spots searched in a fixed order around it. (b) Anywhere within 8 su; the group may crowd one side.
2. (a) At the first arrival, as the Agent's site does. (b) When all have arrived: slower, and one straggler holds up the group.
3. (a) They are pushed off, as the owner's idle units are pushed off a rising pop-up turret, although they have an order, and then walk to their spots. (b) They block, as any unit does, so the order fails.
4. (a) The whole order fails, and is refunded once. (b) Each arrival tries again until one succeeds.
5. (a) Only those at the site, as "Once the selected Recruits reach the building" suggests. (b) All holders of the order.
6. (a) Keep building, as gatherers keep gathering. (b) Fight back, which stops the building.
7. (a) Only a building under construction. (b) A pending ghost too: the Recruit joins the order.
8. (a) Only those within reach count; others wait further out and don't count. (b) A fixed cap.

Proposed: (a) for each.
````

#### D7 — Q0098

````markdown
# QXXXX — Group construction: payment and refunds

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: the proposal under each point.
- **Docs**: `factions/cults/economy.md > Build Command`, `> Cancel Construction`; `systems/controls/selection_and_hud.md > Shift while placing`; 0072; 0289 point 1 (the Syndicate's precedent)

## Question
1. **Partly re-ordered.** "the full Space Crystals cost is refunded" if the order is cancelled before the start because "the Recruits are given another order before arriving". If only some of the group are re-ordered, is anything refunded?
2. **Killed builders.** If every Recruit of the order is killed before the start, is the cost refunded?
3. **Shift chains.** A Shift chain pays each building as it is placed. If the group abandons the chain before reaching the second site, is that site's cost refunded in full?
4. **After a cancel.** A cancelled site's Recruits "remain on the field and go idle". If they have more buildings queued (Shift), do they go idle, or go on to the next?

## Context
0072: "The cost is paid once per building, when the order is given." The Syndicate: "If an Agent is killed before construction starts, its order is refunded" (0289). In the rest of the game, when an order ends, the unit's queued orders go on.

## Options
1. (a) A refund only when no Recruit of the group still holds the order. (b) A share per Recruit: complex, and nothing else in the game refunds that way.
2. (a) Refunded in full, as the Agent's order is (0289). (b) Lost.
3. (a) Each queued building is refunded in full when its last holder drops it before its start. (b) Only the current one is refunded.
4. (a) They go on to their queued orders, and idle if they have none. (b) They go idle, and their queued buildings are refunded.

Proposed: (a) for each.
````

#### D8 — Q0099

````markdown
# QXXXX — Recruitment Area: off-map tiles, rounding, priority, Halls under construction

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: the proposal under each point.
- **Docs**: `factions/cults/economy.md > RecruitmentHall` (RecruitmentArea, TileClaiming, Effectiveness, RecruitmentHallInstanceState, Cults Unit Control HUD Display)

## Question
1. **Off-map tiles.** Near the map's edge, part of the 10x10 area is off the map. Do those tiles count in "the total number of tiles in Recruitment Area"? If they do, a Hall at the edge is weaker.
2. **Rounding.** At 33 claimed tiles, the capacity is 6.6. Does production stop at 6 or at 7, and what does the HUD show?
3. **"Constructed first."** Does it mean completed first? How are ties broken, for two Halls completed in the same frame, or two starting Halls?
4. **Halls under construction.** Does a Hall under construction claim tiles, provide capacity or produce?
5. **Claims across players.** Do claims compete across players? The letter ("another active Recruitment Hall") says yes.

## Context
"Effectiveness = (number of claimed Recruitable tiles in Recruitment Area) / (total number of tiles in Recruitment Area)"; "LocalUnitControlCapacity: number (BaseUnitControlCapacity * Effectiveness)"; production runs "until LocalUnitControlUsage reaches LocalUnitControlCapacity". The Syndicate's areas must lie wholly on the map (0242); nothing says so for Halls.

## Options
1. (a) Off-map tiles count, and can't be claimed: edge Halls are weaker. (b) Only on-map tiles count: a Hall at the edge can reach 100%. (c) Halls must keep their area on the map, as Tunnels must.
2. (a) Production continues while usage is below capacity, so 7 at 6.6, and the HUD shows the capacity rounded down (6). (b) The capacity is rounded down everywhere (6). (c) Rounded to the nearest everywhere (7).
3. (a) Completed first, with ties by the order the Halls were placed (object id). (b) Ordered first.
4. (a) No: it claims and produces from completion, like every other function of a building. (b) It claims from placement, reserving its tiles.
5. (a) Yes, by the letter. (b) Only own Halls compete.

Proposed: (a) for each. 2(a) keeps the exact rule: production runs while 100 × usage < 20 × claimed tiles.
````

#### D9 — Q0100

````markdown
# QXXXX — Hall production: the exit side, X, and counting the Recruit in production

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: the proposal under each point.
- **Docs**: `factions/cults/economy.md > RecruitmentHall > Production Behavior`, `> ObjectInterfaceState[RecruitmentHall]`; `core/entities.md > Exit Doorway`; `factions/gdo/overview.md > Unit Control` (0143, "Reserved when production starts")

## Question
1. **The exit side.** "Produced Recruits spawn at the hall and move to its rally point." The Hall is AAAA, so every side is eligible. Which side do Recruits come out of?
2. **What X does.** "X: Cancel Production: cancels the current Recruit in production, resetting ProductionProgress." Production is automatic: does the next Recruit start at once, or does the Hall stop?
3. **Counting.** Does the Recruit in production count toward LocalUnitControlUsage, as other factions' units in production count toward used (0143)?
4. **At capacity.** When the Hall is at capacity and a unit dies, does production start from zero?

## Context
The exit doorway rule: "Buildings with **several exit-eligible sides** choose the side by their own rules; a side whose doorway is occupied counts as blocked". The Colonists' portals use an energy doorway at the centre instead.

## Options
1. (a) The side facing the rally point (the bottom when there is none), then the next free side clockwise. (b) Always the bottom side, so a blocked doorway pauses the Hall. (c) The centre, like a portal's energy doorway.
2. (a) Production restarts at once: X only delays it. (b) The Hall stops until some later event: none is designed, so it needs a resume command.
3. (a) It counts from when it appears. The HUD shows 19/20 while the 20th is in production. (b) It counts from the start of its production, as in 0143.
4. (a) Yes: nothing is in production while the Hall is at capacity. (b) The progress is kept, paused.

Proposed: (a) for each.
````

#### D10 — Q0101

````markdown
# QXXXX — Cults Unit Control: the global rule, and Recruits that no Hall produced

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a) for each point.
- **Docs**: `factions/cults/overview.md > CultsResources > Unit Control`; `factions/cults/economy.md > Unit Control Tracking`, `> Cults Unit Control HUD Display`, `> Starting Conditions`

## Question
1. **The global rule.** "The player cannot build units if doing so would exceed the total available Unit Control." Halls produce by their local count, and training costs one per Recruit consumed, which already counted. So nothing the design has would ever be stopped by the global total. Is the global figure for display only?
2. **Recruits no Hall produced.** Recruits placed directly by a custom map, or spawned by a debug command, weren't produced by a Hall. Do they count as orphans (global usage only), or against some Hall?

## Context
"Production logic is entirely local — global Unit Control state does not affect production decisions." "A trained unit's Unit Control cost equals the number of Recruits consumed to produce it." The starting Recruits are "all belonging to the starting Recruitment Hall".

## Options
1. (a) Display only, for every Cults unit so far. Vehicles too, since they cost one per Recruit consumed. (b) A real check on something not yet designed. Say what.
2. (a) Orphans. (b) The owner's first Hall in the map's layout. (c) The nearest own Hall.

Proposed: (a) and (a).
````

#### D11 — Q0102

````markdown
# QXXXX — Which Storage does a Recruit drop off at?

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a).
- **Docs**: `factions/cults/units/recruit.md > Gathering`; `factions/cults/economy.md > Storage`; `factions/syndicate/units/agent.md > Gathering` (0255, the Agent's rule)

## Question
With several Storages, which one does a gathering Recruit take its load to? And what does it do when it owns no completed Storage?

## Context
The Recruit has no Set Drop-off. The Agent uses "the nearest own operational Tunnel, chosen at the Gather's first return and kept" (0255). The Storage rationale assumes Recruits use the Storage next to their patch ("each 2-tile face supports roughly 4–5 Recruits").

## Options
- **(a)** The nearest own completed Storage, edge to edge from the patch, chosen at each return. With none, the Recruit idles holding its load.
- **(b)** As the Agent does: chosen at the first return and kept while it stands.
- **(c)** As (a), but with none, the Recruit waits at its patch until a Storage completes.

Proposed: (a). It is the same as (b) while the Storages don't change, and it follows a new Storage at once.
````

#### D12 — Q0103

````markdown
# QXXXX — The Cults economy check: geometry, tolerance, targets and build line

- **Date**: DATE
- **Blocking**: no — the oracle is built anyway; only its pass or fail waits.
- **Docs**: `factions/cults/economy.md > RecruitmentHall` (rationale), `> Storage` (rationale); `factions/cults/units/recruit.md > Gathering`; 0259 and 0303 (the Syndicate's precedent)

## Question
Which Cults figures does the economy check hold the build to, measured how, and within what tolerance?

## Context
The figures in the docs:
- "≈ 1.45 SC/s per Recruit" at a 1-tile gap, which by the doc's own formula gives about 0.59 at 2 tiles and 0.37 at 3;
- "each 2-tile face supports roughly 4–5 Recruits at full efficiency";
- "a starting hall (6 Recruits → 20) fills in ... 168 s (2:48)";
- "the earliest second hall is ~0:55–1:15";
- "At 12 s the Cults stay roughly level with them [the GDO and the Colonists] until ~1:30 and pull slightly ahead by 2:48 (best case, all Recruits mining with ideal Storage placement)".

For the Syndicate, the user chose distances edge to edge, ±15%, and "The table is an ideal ... It's better with some variation" (0259, 0303).

## Options
- **Geometry:** (a) the gap from the patch's edge to the Storage's edge, in whole tiles. (b) Centre to centre.
- **Tolerance:** (a) ±15%, as for the Syndicate. (b) Something else.
- **Targets:** (a) all of the figures above. The comparison with the GDO and the Colonists is on an 8-patch field with one Storage per face, measured at 1:30 and 2:48. (b) Only the per-Recruit rates and the Hall's fill time.
- **The opening's build line:** (a) every Recruit mines; the first 300 SC go to a second Hall, built by four Recruits next to the field. (b) A line the user gives.

Proposed: (a) for each.
````

#### D13 — Q0104

````markdown
# QXXXX — Entering the Armory: the spot, a full pool, loads, capacity

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a) for each point.
- **Docs**: `factions/cults/structures/armory.md` (EntranceSide, InternalRecruitCapacity, Training, ArmoryInstanceState); `systems/controls/commands.md > Enter`; `factions/syndicate/tunnels.md > Tunnel Network` (the precedent)

## Question
1. **The spot.** Where exactly do Recruits go in?
2. **A full pool.** What happens to a Recruit that arrives when the pool holds 10?
3. **Loads.** A Recruit carrying crystals enters. What happens to its load?
4. **Under construction.** Can an Armory under construction be entered?
5. **Capacity.** Does InternalRecruitCapacity count the reserved Recruits? Reserved ones are removed "from the internal pool". By the letter the Armory can hold 10 in its pool plus up to 20 reserved.

## Context
"Recruits enter through the entrance side." The Tunnel: units go in at Side A's entry path; an Enter whose walk ends short "ends and the unit idles"; loads are kept inside.

## Options
1. (a) The middle of side B, as the Tunnel's Side A. (b) Anywhere along side B.
2. (a) The Enter ends and it idles beside the Armory (the normal Failure rule). (b) It waits at the entrance until there is room.
3. (a) It keeps it inside and comes out with it; a Recruit trained into a Militant loses it. (b) It drops it off first. (c) It loses it on entering.
4. (a) No. A right-click on one Assists. (b) Yes.
5. (a) The pool only: up to 30 inside. (b) Everyone inside: 10 in all, so the queue can never pass 10.

Proposed: (a) for each.
````

#### D14 — Q0105

````markdown
# QXXXX — Armory training: which Recruit, its HP, ejected Recruits, the doorway

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a) for each point.
- **Docs**: `factions/cults/structures/armory.md > Training`, `> Eject All`, `> ObjectInterfaceState[Armory]`; `factions/cults/structures/README.md > Cults Infantry` ("no natural health regeneration"); `factions/syndicate/structures/README.md > Rally Point Behavior` (0252)

## Question
1. **Which Recruit.** Queuing "reserves one Recruit from StoredRecruits". Which one? Lineage decides which Hall the trained unit counts against.
2. **HP.** A damaged Recruit is trained into a Militant of the same MaxHP. Does the Militant keep the damage?
3. **Ejected Recruits.** Do they walk to the rally point, or straight out?
4. **One doorway.** Trained units and ejected Recruits share the exit doorway. In what order do they come out?
5. **Several Armories.** One press of Q goes to several Armories. Which ones train?

## Context
"The trained unit inherits the Recruit's AugmentationState unchanged." "Cults infantry have no natural health regeneration. Their only source of healing is the Shrine's health regeneration ability." Ejected Syndicate units go to the rally (0252). Group production: "those with the least work queued pay first, as many as the player can afford" (0083).

## Options
1. (a) The one inside the longest (as the Tunnel ejects, 0250). (b) The healthiest. (c) The one that entered last.
2. (a) It keeps the damage. (b) It comes out at full HP, which makes training a heal.
3. (a) To the rally point, as trained units do. (b) Straight out of side C.
4. (a) In the order they are ready to leave. (b) Trained units first.
5. (a) 0083's order, among the Armories that have an unreserved Recruit. (b) Only the first selected.

Proposed: (a) for each.
````

#### D15 — Q0106

````markdown
# QXXXX — Evacuation: where and in what order units come out

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a) for each point.
- **Docs**: `factions/cults/economy.md > Evacuation on Destruction`; `factions/syndicate/structures/laser_turret.md > Construction` (0298 point 7, the precedent)

## Question
1. **Where.** "evacuated units appear around the building's footprint wherever there is open space". In what order are the spots searched, and how far out?
2. **Order.** In what order are the units placed?
3. **The doorway.** A unit that finished training and waits for the doorway: does it come out as the new unit, and is its order refunded?
4. **Afterwards.** Do evacuated units idle, or go to the rally point?

## Context
"Units come out unchanged: a Recruit mid-training comes out as a Recruit"; "Orders are cancelled with a full refund". The Syndicate precedent: "at the **first clear spot next to the turret**, searched in a fixed order starting from the side it came from" (0298 point 7). An Armory can hold up to 30 Recruits (see the Armory entering question).

## Options
1. (a) Rings around the footprint, nearest first, each ring clockwise from the middle of the exit side, as far out as needed. (b) Only next to the footprint; units that don't fit die.
2. (a) The unit in training, then the reserved ones in queue order, then the pool, longest inside first. (b) By id.
3. (a) It comes out as the trained unit, and nothing is refunded: its training was complete. (b) It comes out as a Recruit, refunded.
4. (a) They idle. (b) They go to the rally point.

Proposed: (a) for each.
````

#### D16 — Q0107

````markdown
# QXXXX — Showing the Recruitment Area

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a).
- **Docs**: `factions/cults/economy.md > RecruitmentHall > RecruitmentArea`, `> TileClaiming`; `factions/syndicate/tunnels.md > Tunnel Area display` (0299, 0300, the precedent)

## Question
Where shown, and how, is a Recruitment Area and the tiles a Hall claims?

## Context
Effectiveness depends on which Recruitable tiles a Hall claims, so Hall placement is a decision the player needs to see. For Tunnels: selecting your own shows its area, placing shows the areas, and "**enemy areas are never shown**" (0299). Claims compete with every player's Halls (see the effectiveness question), including enemy Halls the player may not see.

## Options
- **(a) As the Tunnels' display.** Placing a Hall shows its 10x10 area: the tiles it would claim, those taken by your own Halls, and the non-Recruitable ones, with the effectiveness in %. A selected own Hall shows its claimed tiles. Enemy Halls' areas and claims are never shown, so the preview can be too optimistic.
- **(b)** As (a), and seen enemy Halls' areas count in the preview.
- **(c)** Only the area's outline, with no claims.

Proposed: (a).
````

#### D17 — Q0108

````markdown
# QXXXX — Stale lines in the Cults docs and the Enter command

- **Date**: DATE
- **Blocking**: no — doc cleanup only; the build follows the readings below.
- **Docs**: `systems/controls/commands.md > Enter > Sets BaseCommandState`; `factions/cults/units/recruit.md > Recruit ObjectInterfaceState`, `> Unit Commands`

## Question
Can these lines be brought up to date?
1. `commands.md > Enter > Sets BaseCommandState` says "TargetObject = Tunnel (ObjectInstance)". Enter now targets every structure that holds units (the Armory, Altar, Garrison, the GDO's Bunker).
2. `recruit.md > Unit Commands` says "Assist Construction — via panel button B", as its panel does. That letter waits for the Recruit's panel question.

## Context
Found while planning M5. The rest of the Cults docs were consistent with one another.

## Options
- **(a)** "TargetObject = the structure (ObjectInstance)", and the Recruit's letter as answered.
- **(b)** Leave them.

Proposed: (a).
````

---

## 6. Risks

| # | Risk | Mitigation |
|---|---|---|
| 1 | **Recruit crowds under hard collision** (0106): 4–5 per Storage face in a 64 su gap, many at one patch. The Agents jammed this way (Q0076). | D on Fable in W2, so it is measured early. L reports measured values (0303: variation is fine). Layouts keep the design's 1-tile gap. |
| 2 | **Group construction churn.** D6 and D7 have eight silent points. | The ticket is derived from the orders (T59), so each rule lives in one place. E's tests name each rule and its QD. S1 applies the answers. |
| 3 | **Unit Control drifting from the truth:** held units, `change_kind`, evacuation, neutral units. | One derived count over the objects and `Sim::held` (T56). A soak invariant: the usages plus the orphans equal the Cults units alive. |
| 4 | **Hall claims and determinism** | Integer tile counts, `BTreeMap` order, completion ticks and ids; exact production (T57). No floats. |
| 5 | **Held-unit edge cases:** destroyed while a unit waits in the doorway; the last building destroyed with units inside. | F1's tests. Evacuation runs before defeat in the removal path. Q0106 point 3. |
| 6 | **The "unimplemented faction" rewrite** weakens tests | B keeps each rewritten test's intent, with Cults layouts. A faction without something comes from `test_defs` (T63). |
| 7 | **Cults design arriving mid-milestone** (the user keeps feeding the Designer) | Answers go into briefs not yet launched; S1 and S2 handle the rest. W5 is the content buffer. Gated slices run only when their numbers arrive (D1). |
| 8 | **Scope creep from D1** (more placeholder buildings) | Each is one slice in W5 at most. X1 waits for content; the playtest date moves visibly, by a wave per extra slice. |
| 9 | **X1's breadth:** the tunables' 69 files, and the registry | It runs alone. Goldens byte-identical, with no re-bless allowed. Interleaved A/B perf against master. The ignored suite runs after it. |
| 10 | **Hot paths reading a global ruleset** | Read it once per step, never per unit. A/B on `movement_perf` and `nav_perf`. If needed, a step-local copy of the system's struct. |
| 11 | **Live-reload semantics** (HP, refunds, structural fields) | Structural changes are refused live (F8 restarts). HP follows MaxHP with the damage kept. Refunds at the new price, documented. |
| 12 | **Tuned values drifting from the design** | Overrides live outside the committed rules. The HUD shows "Tuned". Replays carry the rules. `docs/tuning.md` gives the bake path, through the Designer. |
| 13 | **Machine load** | At most three builds a wave; one Fable build (§3.2). |
| 14 | **Files over 500 lines** | Split first (§3.2). |
| 15 | **No AI opponent for the playtest** | Hot-seat with F2. The Director raises it with the user before W6 (§8). |
| 16 | **The Cults' income figures may miss their tolerance** (the Q0089 precedent) | L reports measured against expected. The Director asks with the numbers. |
| 17 | **The Armory holding up to 30** (pool and reserved) | Built by the letter; D13 asks. Evacuation searches as far out as needed. |
| 18 | **Defeat while cancelling the last site** | It is consistent with "Structures under transformation count" (a cancel that leaves no building defeats in that tick). E and R test it. |

---

## 7. QA and the playtest

- **Cults oracle (L).** The design gives targets (D12):
  - 1.45, 0.59 and 0.37 SC/s per Recruit at gaps of 1, 2 and 3 tiles;
  - 4–5 Recruits per 2-tile face;
  - a Hall going from 6 to 20 in 168 s;
  - the second Hall at 0:55–1:15;
  - level with the GDO and the Colonists until about 1:30, and slightly ahead by 2:48, on an 8-patch field.

  Within ±15% (0259), reported against expected (0303).
- **Goldens.**
  - `cults_opening` and `cults_vs_syndicate` (L), and Gunners in the second if G runs (re-blessed once, with the
    reason).
  - Every existing golden stays unchanged through M5 unless a slice explains why. X1 keeps them byte-identical.
- **Soak (M).** `soak_sweep` over all 16 pairings and several seeds (ignored).
  - Scripted Cults orders: gathering, group building, training, evacuation by destruction.
  - T65's invariants.
- **Perf (ignored; interleaved A/B against master).**
  - `recruits_perf` (D);
  - `cults_perf` (M): three Halls, 80 Recruits, four sites, two Armories training, one evacuation;
  - X1's A/B over the existing perf suite.
- **The ignored suite** runs after C (production), after F1 (the removal path), after X1, and at M.
- **Scripted walkthrough (M).** Screenshots at fixed ticks, with `--input-script`:
  1. the start, with Unit Control 6/20 in the HUD;
  2. a Recruit coming out of the Hall;
  3. Recruits at the Storage;
  4. a group site at 10%;
  5. four builders around it;
  6. a Shift chain of ghosts;
  7. X on a site, with the refund in the bank;
  8. Recruits entering the Armory;
  9. a Militant at side C;
  10. W greyed;
  11. Eject All's stream;
  12. evacuation after a destruction;
  13. 20/20, and the Hall idle;
  14. a second Hall's claims overlay;
  15. Cults defeat;
  16. a live tuning change (X2).
- **`docs/playtest/m5.md` (M), the playtest kit:**
  - **Every pairing.** The 16 `--factions` commands, on `dev_skirmish` and on every other map. The editor's
    play-tests (F5) as the second way in.
  - **Controls** for all four factions: a table generated from the panel templates, which a test keeps current. Also
    F2 (switch player), F3 (fog), Tab, Shift, the rally, and the debug keys.
  - **The known placeholders:** a list generated from the code's `Placeholder (` marks, by faction, with their Q
    numbers. Also the deferred list (§1.2).
  - **Tuning (X2):** `--export-data`, then `--tuning <dir>` while playing.
  - **How to report a finding:**
    - one finding per item: what you did, what you expected and what happened;
    - the replay file (recorded to `replays/` by default) and the tick;
    - the tuning directory, if one was used;
    - design matters go to the Designer session, and bugs to the Director session.
    A template in `docs/playtest/README.md` (R).
- **Playtest readiness for all four factions**, checked by M:
  - every pairing starts from the start options (R and B);
  - every Skirmish Default map has all four layouts (B);
  - the controls table and the placeholders list are current (M);
  - tuning works live (X2);
  - the walkthrough's screenshots are reviewed.

---

## 8. Director to-dos implied by this plan

- **Questions.**
  - File D1–D17 as Q0092 onward, and replace the QD slugs in this plan and in briefs with the numbers.
  - None is blocking. Send D1 and D2 first: they decide G.
- **Ask the user** (product decisions, not design):
  - **The opponent.** Is the first playtest hot-seat (F2), or does it wait for an AI? An AI is not planned.
  - **The start.** Is starting from the command line enough, or does the user want a start screen to choose factions
    and maps? None is designed.
  - **The scope.** D1's answer may add slices to W5, and so moves the playtest.
- **`plan.md`.**
  - Replace the M5 entry with these slices, and the gated N1–N6.
  - The change rows: 0071 and 0072 to E and J1; 0302 to A.
  - The follow-up "Target too close for a location" stays open: not the Gunner (§1.2).
  - Add T66's tuning workflow to the playtest-prep entry.
- **Decide T55–T68.**
  - T66 needs no new dependency. Confirm the merge-patch layer over the Rust defaults, rather than data files as the
    source.
  - X1's placement: last, as planned, or first (the T66 alternative).
- **Launch order.**
  - Brief A and R once M4-M merges, quoting §3.3's design text with 0282's paths.
  - Give each slice its ownership row (§3.2) as its "don't touch" list, and the m5-common rules.
  - After B merges, tell the running slices that no faction is unimplemented any more.
- **Folding answers in.** As in M4: fold each answer into the briefs that haven't launched. Rewrite a placeholder
  only when its slice is open, or in S1 and S2.
- **The ignored suite:** after C, F1 and X1, and at M (§7).
