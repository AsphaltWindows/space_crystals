> **Adopted by the Director on 2026-11-26** from planning spike M6-P's draft (below, unchanged), which read master at
> `4d44137a` and the design at `acf7ef6f`. Where this note and the draft differ, this note wins.
> - **V3 and V4 move into M5.** N10's Hand of Heaven is a tracked crusher (`hand_of_heaven.md`, `systems/units.md >
>   Crushing`), and no M5 slice built either. In M5: V2 → V3 → N10 (after N6 and V3), and V4 after V3, beside N10.
>   The draft had V4 in W4; W4's freed slot takes SC or SP early.
> - **GS is not gated.** 0077 deferred the GDO's supply loop "to the air milestone", and M6 brings the GDO's aircraft.
>   GS runs in W10 whatever Q0169 says Supplies buy.
> - **The questions are filed.** D42 became Q0153 first, since V3 and V4 need it. Check slice M6-Q then checked every
>   other draft against the docs as they stood on 2026-11-26: every quote word for word, the arithmetic, and whether the docs already
>   answer a point. The filed questions supersede §E's drafts:
>
>   | Draft | Question | Draft | Question | Draft | Question | Draft | Question |
>   |---|---|---|---|---|---|---|---|
>   | D32 | Q0154 | D33 | Q0155 | D39 | Q0156 | D40 | Q0157 |
>   | D48 | Q0158 | D44 | Q0159 | D35 | Q0160 | D36 | Q0161 |
>   | D34 | Q0162 | D37 | Q0163 | D38 | Q0164 | D46 | Q0165 |
>   | D45 | Q0166 | D43 | Q0167 | D47 | Q0168 | D41 | Q0169 |
>   | D42 | Q0153 | | | | | | |
>
>   Points the docs already answer were dropped, and their slices build the docs' answer:
>   - D36: the footprint and Unit Control;
>   - D39: hidden objects (detection comes later, 0267) and high ground;
>   - D40: construction and production;
>   - D41: whether to build the Supply Tower now (0077);
>   - D44: reversing;
>   - D45: allied boarding (no teams yet);
>   - D46: the Specter's landing;
>   - D48: three points.
>
>   D43's "inside" point moved into D44 (Q0159). A slice marks a placeholder `Placeholder (Qnnnn point N)`.
> - **For the briefs,** from the M6-Q check:
>   - **GK and GH1:** the HoverCraft base "Relies on high drag to decelerate passively", but `movement/drag.rs` brakes by
>     thrusting against the motion, which the design gives to ground hover units. The air HoverCraft need drag-only
>     braking, as Q0154's placeholder (a) reads.
>   - **GM:** at the placeholder numbers, "a Grenadier's Attack Ground destroys a mine" takes about 8 grenades. The
>     test must allow for that.
>   - **Q0164 point 2** (whom the First Aid Tent heals) could also change M5's Shrine casts, which skip units inside
>     buildings.
> - **X1 and X2 stay in M5**, and M6 is written in X1's form (T101), as recommended. M6 starts at X1's merge.
> - **Answered since adoption** (each answer is a row of `plan.md`'s change table, and the slice's brief carries it):
>   - Q0153 by 0417 (V3's tread law), 0418 (a spin crushes the enemies it sweeps) and 0419 (the surfaced Transport
>     crushes): V3 and V4;
>   - Q0154 by 0420: option (c), DragMovement numbers of each unit's own. The air HoverCraft still stop by drag alone,
>     so the note for GK and GH1 above stands;
>   - Q0155 by 0421: R at the Barracks, no rebuy while the drone is out, the leash edge to edge;
>   - Q0156 by 0422: the Radar Station is Groupable false;
>   - Q0157 by 0423: low Power also slows cooldowns, turret fire and rearming;
>   - Q0158 by 0424 and Q0167 by 0433: as the placeholders;
>   - Q0159 by 0425: a stationary Shadebringer lays its trail; the centre rule; the toggles go off in the network;
>   - Q0160 by 0426: mines are deployed units, selectable; hover units pass over; no blast when destroyed;
>   - Q0161 by 0427: as the placeholder;
>   - Q0162 by 0428: the Airfield's landing rules; new aircraft appear landed;
>   - Q0163 by 0429: the Field Engineer builds inside, alone, like an Agent;
>   - Q0164 by 0430: the shield drops in a Bunker; the Tent heals only units on the map, once complete;
>   - Q0165 by 0431: the Phantom takes 16 frames at the air doors;
>   - Q0166 by 0432: Enter surfaces a burrowed Transport first;
>   - Q0168 by 0434: Thrust is targeted and Shift-queues.

# M6 plan: the GDO's and the Syndicate's remaining content (planning spike M6-P)

**For the Director.** Design changes 0366 and 0367 gave placeholder numbers to every designed GDO and Syndicate
building and unit that isn't built: "Everything listed can now be built on placeholders" (0366). 0368 and 0369 turned
the Designer's readings into rules. On 2026-11-26 the user said "just continue with the milestones work", which the
Director reads as M6 = this content, built after M5's remaining slices. This plan uses the M5 addendum's format:
- **§A and §B:** 29 slices, V3 included (M5 needs V3 first, for N10);
- **§C:** waves W1–W11, from X1's merge;
- **§D:** technical decisions T84–T101;
- **§E:** question drafts D32–D48;
- **§F:** size, risks, the X1/X2 recommendation, and a first subset for a partway playtest.

**What I read:**
- **Commits.** Master is `4d44137a`. The design was read at `acf7ef6f`, the last commit under `artifacts/designer/`.
  The permission system refused `git -C /home/iv/dev/space_crystals log -1 -- artifacts/designer` ("isolated in the
  worktree"). The same command run in the worktree gave `acf7ef6f`.
- **Entries 0408–0412 arrived after that commit** (seen in the main checkout's `changes/`, not read in full). Only
  0412 touches M6: gliders push apart at a third of the HoverCraft strength, not a quarter. GL and SA cite it. 0408's
  strafing rule reaches the Dragonfly and the Specter through H2's code.
- **The change entries:**
  - 0366–0369, and every design section they list;
  - the older entries: 0014–0020, 0039, 0077, 0188–0196, 0210–0228, 0262–0273, 0353, 0392 and 0405.
- **The design:**
  - `factions/gdo/{structures,units}/*`, `factions/gdo/{economy,overview}.md`;
  - `factions/syndicate/{structures,units}/*`, `factions/syndicate/{tunnels,tunnel_menus}.md`;
  - `systems/units.md` (UnitBase, collision, Crushing, the movement models), `systems/combat.md`, `core/vision.md`;
  - `core/entities.md > Placement Validation`, `core/game_rules.md > Defeat Condition`.
- **The Director's files:**
  - `plan.md`: In progress, the queue, the gated rows, and the open questions (Q0149, Q0151, Q0152);
  - the M5 plan's §4 (T55–T68);
  - the addendum: its format, T69–T83 and the adoption note;
  - the briefs m5-common, m5k, m5n7 and m5v1.
- **The code at `4d44137a`:**
  - the sim: the defs and `defs/units/bases.rs`, `movement.rs`, `vision.rs` and `vision/hidden.rs`, `combat.rs` and
    `combat/{targeting,area,projectiles}.rs`, `network.rs`, `tuning/tunnel.rs`, `production.rs`, `construction.rs`,
    `deploy.rs`, `transform.rs`, `buffs.rs`, `toggle.rs`, `command.rs`, `economy/stations.rs`, and the menus in
    `defs/syndicate.rs`;
  - the client's GDO and Syndicate templates;
  - `crates/ai/src/factions/gdo.rs`.
- **Cargo was not run.**

**The state I assumed (2026-11-26):**
- **Running:** A2a–c, the computer players for the Cults, the Syndicate and the Colonists, in `crates/ai`.
- **M5's remaining order:** N7 ∥ K ∥ A2d; then V1 → V2 → N6 → N10, and N11; last X1, X2 and M.
- **What lands before M6:**
  - **K:** a general HeadDisjointed attack (the Rocketman's Target Lock), the Arsenal and the Rocketman. K also decides
    what a homing missile does when its target dies in flight.
  - **N7:** `ExitRule::Runway`. Its Sparrow never lands.
  - **V1:** oriented silhouettes.
  - **V2:** FixedTurnRadius movement, Reverse, and Rugged impassable for vehicles (T79).
- **A gap in M5's plan.** No slice builds SpeedTurnRadius (tracked) movement or crushing:
  - `movement::turn_rate` leaves that model "without limit ... (TODO(movement models): not built)";
  - the Hand of Heaven's def says "slice V2 turns vehicles", but T79 is wheeled only.

  N10's Hand of Heaven is tracked, so **V3 (§B) has to run in M5, after V2 and before N10.** N10's "After" then reads
  "N6 and V3".

---

## §0 Summary

- **Scope: 29 slices, about 78 h of agent work.**
  - **Engine (2):**
    - **V3:** tracked movement. It runs in M5, before N10.
    - **V4:** crushing.
  - **Kinds (2):** GK registers the 23 new GDO kinds and SK the 15 Syndicate kinds. Then the registry closes (T84).
  - **GDO (11):** GR, GD, GF, GA, GM, GP, GB, GH1, GH2, GL, and GS (gated on D41).
  - **Syndicate (11):** SV, SN, SB, SL, SD, ST1, SC, SP, SA, ST2, SR.
  - **Tail (3, plus an optional one):** S6a and S6b (answer sweeps) and MQ (M6's QA); MQa only if the first-subset
    playtest is taken.
- **Not in M6** (not designed): the Supply Chopper's transport; Syndicate research, which waits for the techs pass;
  detection; what the GDO spends Supplies on (§A).
- **Waves (§C):**
  - W1 starts once X1 has merged, beside X2; M joins W2.
  - W1–W10 run three builds a wave, with at most one Fable build in each. W11 is MQ alone.
  - The critical path is capacity. The longest chains, GK → GH1 → GH2 → GL and SK → SV → SD → {SP → SR, ST1 → ST2},
    fit with a wave or two of slack.
- **Technical decisions:** T84–T101 (§D).
- **Question drafts:** 17 (D32–D48), none blocking. The three most important:
  - **D34,** how GDO aircraft use the Airfield, the Dragonfly's landing in particular;
  - **D32,** the HoverCraft movement numbers;
  - **D43,** the smoke's sight blocking.
- **X1/X2:** they stay in M5, before its playtest. M6 is written in X1's form from its first slice (T101, §F).
- **First subset (§F):** the ground war: both factions' ground vehicles and static defences, the Bunker, the mines,
  the Null Chamber and the Phantom. It is ready after W6, with the playtest at the end of W7. The air line and the
  Transport follow.
- **Computer players:** no M6 slice changes their games, which use only M5 units. A3 would add M6 units (§B notes what
  each slice offers them).

---

## §A Scope

**How it is cut.** As in M5, each building gets one vertical slice with the units it makes, its panel and its tests.
The exceptions:
- **Engine slices:** V3 and V4. Machinery that four units (and N10's Hand of Heaven) share is built once, on test
  defs.
- **Kinds slices:** GK and SK, as T69. The registry's order is hashed, and `requires` and the menus name kinds across
  slices: the Defense System needs the Arsenal (K's) and a Radar Station, and the Aerospace Assembly a Forge.
- **Three splits,** where one building's content passes 3 h:
  - the Airfield: GH1, then GH2;
  - the Shadebringer: SB, then SC;
  - the Transport: ST1, then ST2.

| 0366–0369 content | Slice | Notes |
|---|---|---|
| Every new kind, base defs only | **GK** (GDO, 23), **SK** (Syndicate, 15) | nothing produced or offered until its slice (T84) |
| SpeedTurnRadius movement: the Prospector, Artillery Rig, Laser Battery and surfaced Transport (and N10's Hand of Heaven) | **V3** | runs in M5 |
| Crushing (`systems/units.md > UnitCollision > Crushing`) | **V4** | the TBD on turning in place: D42 |
| Radar Station and Scan; Defense System (0392's one-slot flow); Missile Turret | **GR** | Scan: T92 |
| Drone Operator and its drone | **GD** | builds the unit stock (T89) |
| Factory; Advanced Mechanics; Industrial Powerplant; Raptor; the Twister and its machine gun | **GF** | the Factory's E and R stay greyed until GP and GA |
| Artillery Rig, both kinds, and the vehicle deploy model | **GA** | T44, built (T87) |
| The Twister's mines | **GM** | T89, T90 |
| Prospector and Outpost | **GP** | T87 from a site |
| Bunker, First Aid Tent, and the Field Engineer's Construct menu | **GB** | how it builds: D37 |
| Airfield, Hornet, ammo, assignment, the production limit | **GH1** | the Airfield stays `Planned` in the menu until GH2 |
| Landing slots, landing and take-off, rearming, Return to Base | **GH2** | T88 |
| Propulsion Labs and the Dragonfly | **GL** | a glider's landing: D34 |
| Supply Tower and the Supply Chopper's supplies | **GS** | gated on D41; the transport is out |
| Vehicle Assembly and Assault Bike | **SV** | |
| Null Chamber | **SN** | T98 |
| Vault, the Shadebringer's tanks, and Sludge | **SB** | Cover stays greyed until SC |
| The Shadebringer's Cover (smoke) | **SC** | the game's first sight blocking (T93) |
| Laser Battery, both kinds, its battery, and Amplify | **SL** | |
| The Tier 3 air doors, Forge, Aerospace Assembly, Phantom | **SD** | T96 |
| The Subterranean Transport above ground: loading, unloading, Tunnels | **ST1** | T94 |
| Burrow and Surface, underground movement, hiding | **ST2** | T95 |
| Orbital Command and the Prism | **SP** | its panel: D47 |
| The Specter | **SA** | T88's landing, at the air door |
| The Laser Battery's Relay | **SR** | T97 |
| 0368's and 0369's readings, now rules | the slices named in §B | quoted in their Design |

**Not in M6** (not designed, so named and left out):
- **The Supply Chopper's transport.** `factions/gdo/units/supply_chopper.md > Cargo`: "Unit transport mechanics
  (Load / Unload commands, landing to load ...) are TBD — deferred until unit design resumes." GS builds its
  Supplies only.
- **What the GDO spends Supplies on** (0077's first missing piece). GS would gather Supplies that buy nothing (D41).
- **Syndicate research,** and the Chemical Plant's, Optics Lab's, Forge's and Orbital Command's commands: 0367 says
  they "wait for the techs pass". These buildings are prerequisites only.
- **Detection** (`core/vision.md`: stealth and detection are TBD; 0267: after every faction's units are done). Until
  it exists, nothing reveals mines, burrowed Transports or underground structures, and only area damage hurts them
  (T32). The Null Chamber's buff has no detection.
- **Computer players** for M6 units: they wait for A3.

---

## §B Slices

**Every slice:**
- **Checks** (m5-common): fmt, clippy `-D warnings`, `cargo test --workspace`, `design_citations` and the seed sweep
  all pass.
- **The computer players' games are unchanged:** `cargo test --release -p sc_ai --test ai games::ai_games --
  --ignored --nocapture` gives the same rows as master. Their games use only M5 units, so a moved row is a bug in the
  slice.
- **Goldens:**
  - no existing golden moves (the 11, K's `gdo_anti_air` and N7's), unless the slice says so;
  - each content slice adds one of its own;
  - new state is hashed and saved only while set (T65).
- **Numbers.** 0366's and 0367's numbers are the docs' own "Placeholder stats": the code cites the doc and the entry
  next to each. `Placeholder (QDnn-slug point N)` marks a gap that §E drafts a question about; it becomes
  `Placeholder (Qxxxx point N)` once filed. `Placeholder (not in the design: why)` marks the rest.
- **The ruleset's form (T101):** tunables go in X1's tuning modules, new structural fields join
  `Ruleset::structural_diff`, and new def fields round-trip through X2's export.
- **After** lists only what hasn't merged by W1. K, N7, V1, V2, V3, N6, N10, N11 and X1 have all merged by then.

**Computer players.** Each slice says what it offers A3. None plans AI work.

### Engine

#### V3: Tracked movement (Opus; Fable if the corner choice resists; runs in M5)

**Goal.**
- A TrackedVehicle, and a DrillUnit above ground, drives by SpeedTurnRadiusMovement:
  - it turns in place when stopped;
  - its turn radius grows with its speed;
  - it slows for a sharp corner, or stops and spins.
- Rugged ground is impassable to it (0039), as V2 made it for wheeled vehicles. It reverses on W, by V2's Reverse.
- N10's Hand of Heaven and M6's four tracked units drive on it.

**Design**
- **`systems/units.md > SpeedTurnRadiusMovement`:** "Unit can rotate in place by spinning treads in opposite
  directions. Turn radius increases with speed ... Small directional adjustments at speed cost little momentum, but
  sharp turns require significant slowing or a complete stop."
  - Its pathfinding note: "the implementation may use **pre-computed values** (e.g. tables of maneuvers: time and
    space needed to turn by a given angle from a given speed)".
- **`systems/units.md > UnitBase`:** TrackedVehicle has CanTurnInPlace, CanReverse and DirectionalArmor, and
  RuggedTerrain false. DrillUnit has the same movement model.
- **The units:**
  - `factions/cults/units/hand_of_heaven.md`;
  - `factions/gdo/units/{prospector,artillery_rig}.md`;
  - `factions/syndicate/units/{laser_battery,subterranean_transport}.md`.

  For example: "SpeedTurnRadiusMovement: MaxSpeed 4 su/frame; Acceleration 0.4; Deceleration 1;
  SpeedToTurnRadiusRatio 8", and the Hand of Heaven's "(turns in place when stopped)".
- **Built already:** 0323 and 0338, the 90° contact step for TrackedVehicle and DrillUnit. 0353's yielding rules are
  V2's.

**Placeholders**
- **The spin rate isn't given:** 1/ratio radians a frame, which is the rate at any speed under the linear law (T85).
  `Placeholder (QD42-tracked point 1)`.
- **The Transport's Acceleration, Deceleration and ratio aren't given:** ST1 takes the Laser Battery's (D42).

**Code** (T85)
- **New `movement/tracked.rs`,** beside V2's `wheeled.rs`:
  - signed speed, with Acceleration, Deceleration and MaxSpeed;
  - a radius of |v| × ratio, so a constant angular rate of 1/ratio while moving;
  - a spin at that rate when stopped.
- **Steering** is V2's follower (the farthest visible waypoint, braking at v²/2d). At each corner it chooses the arc at
  speed or a stop and spin, analytically (T85).
- **`movement::turn_rate`** gets its SpeedTurnRadius arm, and the TODO goes.
- **A spin blocked by a wall** is tried whole, then halved, by V1's rule.
- **Nav:** the tracked bases use V2's vehicle NavClass.
- **Test units:** the Hand of Heaven's def, and a 64x48 test def.

**Acceptance**
- **Spinning.** A tracked unit standing still facing east, ordered to a point due west, spins in place without
  moving, then drives off.
- **A sharp corner.** At full speed into a 90° bend of a corridor two tiles wide, it stops and spins, and never
  touches the walls. A gentle curve keeps it within 10% of MaxSpeed.
- **Reverse:** W reverses it, at its speed.
- **Rugged** is impassable to it: its path goes round.
- **At a wall,** a blocked spin stops at contact (V1).
- **Perf:** `vehicles_perf` gains 30 tracked units; record their mean and p99 beside V2's.
- **Goldens:** none moves, since no tracked unit plays in one.
- **QA screenshots:** a Hand of Heaven spinning in place, and one taking a corner.

**Computer players:** none has a tracked unit. A3 would get them working out of the box.

**Owns:**
- new `movement/tracked.rs`, and its arms in `movement.rs` (`turn_rate`, `Mover`) and in V2's follower;
- `docs/movement.md`'s tracked section;
- new `tests/tracked/`.

Read-only: `movement/wheeled.rs`, `movement/collide/` and `shape.rs`.

**After:** V2, in M5, before N10. **Model:** Opus, about 3 h. Fable is the fallback if the corner choice won't settle.

#### V4: Crushing (Opus; Fable flag)

**Goal.**
- A moving tracked vehicle kills the enemy Light Infantry it overlaps, and drives through them.
- Standing still, it is a hard obstacle.
- Its own side's and allies' infantry are obstacles to it.

**Design**
- **`systems/units.md > UnitCollision > Crushing`:**
  - "Units whose UnitBase can crush (TrackedVehicle, Mech) crush enemy units whose UnitBase has Crushable = true
    (Light Infantry)";
  - "**Effect**: instant death of the crushed unit";
  - "**Trigger**: the crusher's Silhouette overlaps a crushable enemy **while the crusher is moving**";
  - "**Pathing exception to hard collision**: crushers may path and drive through crushable enemies";
  - "**Stationary crushers do not overlap**"; "**Crushable units avoid crushers**"; "**No friendly crushing**";
  - "TBD: whether a crusher turning in place (Stationary + Turning) can crush, or is blocked by, an adjacent
    crushable enemy".
- **Rotated shapes** count (`> Silhouette Rotation`: "Collision, crushing ... use the rotated shape").

**Superseded by 0418 and 0419** (2026-11-26, answering Q0153 parts 2 and 4). A spin in place crushes the crushable
enemies the turning Silhouette sweeps, and own and allied crushables still block it. A surfaced Transport crushes like a
TrackedVehicle, the spin rule included, and never underground. The two placeholders below are replaced.

**Placeholders** (D42)
- A spin in place doesn't crush, and an adjacent crushable enemy blocks it.
- A surfaced Transport (a DrillUnit) doesn't crush. The doc names TrackedVehicle and Mech only.

**Code** (T86)
- **`UnitBaseDef::crushes`:** true for TrackedVehicle and Mech.
- **`movement::space::blocks`** gets the crushing arm its docs already name.
- **The kill** comes in the movement phase, after collision, through the death path, with `SimEvent::Crushed`.
- **Avoidance:** a crusher's follower doesn't treat enemy crushables as obstacles.

**Acceptance**
- **Through a line.** A moving test tracked unit drives through 5 enemy Peacekeepers. All die, and its path stays
  straight.
- **No friendly crushing.** Own and allied Light Infantry stop it, and it goes round them.
- **Standing still,** it lets no enemy infantry walk into it, and crushes nothing.
- **A spin beside an enemy Peacekeeper** crushes nothing, and stops at contact (D42).
- **Heavy Infantry** (Guards) are never crushed.
- **The soak's no-overlap invariant** allows one exception: a moving crusher with an enemy crushable, in the tick of
  contact, after which it is dead.
- **Perf:** `vehicles_perf` within noise. **Goldens:** none moves.

**Computer players:** their Peacekeepers and Recruits are crushable, but no crusher plays in their games. A3's
armies would have to give way to tracked units.

**Owns:**
- `UnitBaseDef::crushes`, and new `movement/crush.rs`;
- `space::blocks`'s arm, and `SimEvent::Crushed` with its viewer row;
- new `tests/crushing/`.

**After:** V3. **Model:** Opus, about 2 h. Fable is the fallback.

### Kinds

#### GK: The GDO kinds (Opus)

**Goal.**
- Every GDO kind of 0366 exists with its numbers: 12 buildings, 10 units, and the mine.
- The Deployment Center's two menus hold every building in its slot. The new ones are `Planned`, greyed "Not in the
  build yet", until their slice.
- GK and SK are M6's only slices that add kinds. The registry closes after both (T84).

**Design**
- **0366's tables,** and each doc's "Placeholder stats":
  - `factions/gdo/structures/{factory,airfield,advanced_mechanics,propulsion_labs,industrial_powerplant,radar_station,defense_system,missile_turret,outpost,bunker,first_aid_tent}.md`;
  - `factions/gdo/economy.md > SupplyTower`;
  - `factions/gdo/units/{raptor,twister,prospector,artillery_rig,hornet,dragonfly,drone_operator,supply_chopper}.md`;
  - `factions/gdo/units/twister.md > Mines` (the mine, "a 16x16 su object, hidden underground").
- **`factions/gdo/structures/deployment_center.md`:**
  - the requirements: "Supply Tower: a Radar Station"; "Factory: a Barracks"; "Radar Station: a Generator";
    "Advanced Mechanics: a Factory"; "Airfield: Advanced Mechanics"; "Propulsion Labs: an Airfield"; "Industrial
    Powerplant: Advanced Mechanics"; "Defense System: a Radar Station and an Arsenal";
  - the Basic menu: Q Generator, W Barracks, E Extraction Facility, R Supply Tower / A Radar Station, S Arsenal,
    D Career Campus, F Defense System;
  - the Advanced menu: Q Factory, W Advanced Mechanics, E Airfield, R Propulsion Labs / A Industrial Powerplant.
- **`core/entities.md > Building armor rule`,** for the armour test's rows.

**Placeholders**
- 0366, as cited.
- **The drone** (D33): a HoverCraft, Unit Control 0, armour 0/0, Groupable.
- **The Hornet's, the drone's and the Supply Chopper's movement** (D32): Drag. The Chopper's numbers are its doc's.

**Code** (T84)
- **23 def modules** (`defs/gdo/structures/`, `defs/gdo/units/`, and `defs/gdo/mine.rs`), holding:
  - their base fields;
  - each Construct entry's recipe and `requires`, kept out of `constructs`, `production` and `builds` until its slice
    (T69's rule).
- **Their registry lines** go at the end, in 0366's order:
  - Factory, Airfield, Advanced Mechanics, Propulsion Labs, Industrial Powerplant, Radar Station, Defense System,
    Missile Turret, Outpost, Bunker, First Aid Tent, Supply Tower;
  - Raptor, Twister, Prospector, Artillery Rig, Artillery Rig (Deployed), Hornet, Dragonfly, Drone Operator,
    `OperatorDrone` (shown as "Drone"; the Colonists' `Drone` exists), Supply Chopper;
  - `Mine`.
- **`Category::Mine`,** with its arms (T90). No mine is spawned before GM.
- **Split `defs/units.rs` first** (454 lines). Then add the struct-update defaults, if X1 left the defs as static
  literals (T84).
- **The client:**
  - 23 `kinds/gdo/*.rs`, whose labels stay unique;
  - the Deployment Center's menus, with `Planned` entries for the new buildings.
- **Tests that walk every kind** take the new ones: the defs' consistency, the labels, the editor's palette, and the
  building armour rule.

**Acceptance**
- **Each new kind's def** matches its doc, with one test per kind, as G2 did.
- **No player can produce, construct or spawn** a new kind. The editor's debug spawn can.
- **The armour rule** holds for the 12 GDO buildings.
- **QA screenshot:** the Deployment Center's two menus, with the new entries greyed.
- **No golden moves.** If appending kinds moves any hash (a per-kind array in hashed state), GK and SK go back to
  back instead of side by side (T84).

**Computer players:** nothing. They construct by kind.

**Owns:**
- the 23 def modules and their registry lines, and `Category::Mine`;
- the split of `defs/units.rs`, and the struct-update defaults;
- 23 `kinds/gdo/*.rs`, and `Planned` entries in `templates/gdo/deployment_center.rs`;
- the GDO rows of `tests/building_armor.rs`.

Read-only: every system.

**After:** X1. **Model:** Opus, about 3 h. If it runs long, the unit modules go to a GK2 straight after it, still
before any content slice.

#### SK: The Syndicate kinds (Opus)

**Goal.**
- Every Syndicate kind of 0367 exists with its numbers: 6 expansions and 9 units, the two-kind pairs counted twice.
- The ExpandMenu's six `Planned` entries stay `Planned` until their slices.
- The EjectMenu's order lists the new units.

**Design**
- **0367's tables,** and each doc's "Placeholder stats":
  - `factions/syndicate/structures/{vault,vehicle_assembly,forge,null_chamber,aerospace_assembly,orbital_command}.md`;
  - `factions/syndicate/units/{assault_bike,shadebringer,laser_battery,subterranean_transport,phantom,specter,prism}.md`.
- **`factions/syndicate/structures/README.md > Tech Tree Overview`:**
  - Vault: Tier 1, a Hideout;
  - Vehicle Assembly: Tier 2, a Chemical Plant;
  - Forge: Tier 2, a Vehicle Assembly;
  - Null Chamber: Tier 2, a Hideout;
  - Aerospace Assembly: Tier 3, a Forge;
  - Orbital Command: Tier 3, an Aerospace Assembly.

  Expansions follow n = 6: 6/24 at 2x2 and 6/54 at 3x3.
- **`factions/syndicate/tunnel_menus.md > EjectMenu`:** tiles in README order: Agent, Guard, Grenadier, Assault Bike,
  Shadebringer, Laser Battery, Transport, Phantom, Specter, Prism.
- **Unit prerequisites:**
  - the Shadebringer needs a Vault;
  - the Laser Battery and the Phantom need an Optics Lab;
  - the Transport needs a Forge;
  - the Specter needs a Vault and a Chemical Plant;
  - the Prism needs an Orbital Command.

**Placeholders**
- 0367, as cited.
- **The Phantom's and the Prism's movement:** Drag, by D32's proposal.
- **The Transport's tracked numbers:** the Laser Battery's (D42).

**Code** (T84)
- **15 def modules.** Their registry lines go after GK's block:
  - Vault, Vehicle Assembly, Forge, Null Chamber, Aerospace Assembly, Orbital Command;
  - Assault Bike, Shadebringer, Laser Battery, Laser Battery (Deployed), Subterranean Transport, Subterranean
    Transport (Burrowed), Phantom, Specter, Prism.
- **`EJECT_MENU_ORDER`** gains the seven base kinds. The deployed and burrowed kinds never enter the network, so the
  test that "it lists every Syndicate unit once" exempts them by name.
- **The client:** 15 `kinds/syndicate/*.rs`.
- **Tests that walk every kind,** as in GK, and the expansions' armour rows.

**Acceptance**
- **Each def** matches its doc.
- **The ExpandMenu** is unchanged: six greyed `Planned` entries.
- **The EjectMenu** shows the new tiles greyed with a count of 0.
- **No golden moves.**

**Computer players:** nothing. A2b expands by kind.

**Owns:** the 15 def modules and their registry lines, `EJECT_MENU_ORDER`, 15 `kinds/syndicate/*.rs`, and the
Syndicate rows of `tests/building_armor.rs`.

**After:** X1. It runs beside GK, and whichever merges second rebases its registry block. **Model:** Opus, about
2.5 h.

### GDO

#### GR: The Radar Station and Scan, the Defense System, and the Missile Turret (Opus)

**Goal.**
- The Deployment Center builds the Radar Station (with a Generator) and the Defense System (with a Radar Station and
  an Arsenal).
- **Scan** (Q) reveals a 6 GU circle anywhere for 128 frames. Each station has its own 960-frame cooldown.
- **The Defense System** builds Missile Turrets one at a time, by the Extraction Facility's flow, for placing in the
  GDO build area.
- **The Missile Turret** fires a homing missile at air and ground.

**Design**
- **`factions/gdo/structures/radar_station.md`:**
  - "**Scan** (hotkey **Q**, targeted: left-click any point on the map): reveals a circle of radius **6 grid units**
    for **8 seconds** (128 frames); **cooldown 60 seconds** (960 frames); free. Each Radar Station has its own
    cooldown";
  - 2x2, HP 300, sight 8, Power −20; 150 SC and 320 frames.
- **`factions/gdo/structures/defense_system.md`** (0392):
  - "**Works like the Extraction Facility** ... one slot, Q, no build menu";
  - "**Idle**: **Q: Build Missile Turret** (100 Space Crystals, starts construction)";
  - "**Constructing**: **Q: Place Missile Turret**, greyed out, with a **running progress bar** on the button; **X:
    Cancel Construction** (full refund)";
  - "**Ready**: **Q: Place Missile Turret** enters AwaitingPlacement (ghost within the GDO build area ...); **X: Cancel
    Ready Turret** (75% refund, rounded down)".
- **`factions/gdo/structures/missile_turret.md`,** and 0368 ("homing"):
  - HeadDisjointed, Universal; Damage 18, Range 6, 16 su/f; Aim 4, Fire 2, Cooldown 4, Reload 14;
  - 1x1, HP 250, armour 2/2, sight 7, Power −10; it targets as the Laser Turret does.
- **0392's Verification:** "the same for a Missile Turret".

**Placeholders**
- 0366.
- **D39 (Scan):**
  - it shows hidden objects no more than other sight does;
  - it goes to every station it is issued to, by `systems/controls/selection.md > No smart-casting`, since the doc
    says "Groupable true": two grouped stations both scan the point;
  - high ground doesn't shrink it (no ElevationModifier, as for air units).
- **D40 (Power):** low Power slows the Defense System's turret construction (construction by a consumer), but not
  Scan's cooldown or the turret's fire.

**Code** (T92; T49 with K's HeadDisjointed)
- **Scan:**
  - Scan joins the ability enum and the Radar Station's `abilities`, with `Object::cooldown_until`;
  - `Command::Cast`'s gate takes a cooldown ability with no vision check;
  - new `vision/scans.rs`, after splitting `vision.rs` (471 lines).
- **The Defense System:** `constructs: [MissileTurret]`, on M2-D1's constructor as data. Its client template follows
  the Extraction Facility's.
- **The Missile Turret:** `StructureDef::attack` with K's HeadDisjointed, and the Laser Turret's picker.
- **The Deployment Center:** its `constructs` gains both buildings, and A and F go live in its Basic menu.

**Acceptance**
- **Requirements.** The Radar Station is refused without a Generator, and the Defense System without a Radar Station
  or without an Arsenal.
- **Scan:**
  - At an unexplored point, its tiles are Visible for exactly 128 ticks, then fall to Explored after the fog delay.
  - The same station's next Scan is refused for 960 ticks.
  - With two grouped stations selected, Q scans from both, and both cool down (No smart-casting; D39).
  - An enemy's underground expansion inside the circle stays hidden (D39).
- **Power.** At negative Power the Defense System's construction slows by the ratio. Scan's cooldown and the turret's
  fire don't (D40).
- **The Defense System's flow:** build, cancel (full refund), ready, cancel ready (75%, rounded down), and placing
  inside the build area only.
- **The Missile Turret** hits a circling Sparrow and a running Peacekeeper and never misses (K's rule). It fires from
  its first tick (GDO buildings are placed complete).
- **Perf:** `vision_perf` with four scans active, within noise.
- **Golden** `gdo_defense`: a Radar Station, a Defense System and two Missile Turrets against a Sparrow raid.
- **QA screenshots:** the Scan cursor and circle; the Defense System's button with its progress bar; a Missile Turret
  firing.

**Computer players:** none. A3 could scan and build turrets.

**Owns:**
- the three buildings' live fields: `abilities`, `constructs` and `attack`;
- Scan's arm in the ability enum, `Object::cooldown_until`, and the cooldown arm of `sim/casts.rs`;
- new `vision/scans.rs`, and the split of `vision.rs`;
- the Deployment Center's two recipes and menu entries;
- client: `templates/gdo/{radar_station,defense_system}.rs`, the Scan cursor, the Missile Turret's look;
- new `tests/radar/` and `tests/defense_system/`.

Read-only: `combat/structures.rs`, K's HeadDisjointed.

**After:** GK. **Model:** Opus, about 3 h.

#### GD: The Drone Operator (Opus)

**Goal.**
- The Barracks trains the Drone Operator, given a Career Campus.
- The operator carries one drone. It buys a new one with F, and launches it with R.
- The player flies the drone into ground enemies. Its one-use explosion deals area damage.
- The drone falls:
  - when it leaves its leash, and explodes;
  - when its battery runs out, or its operator dies, without exploding.
- The operator can't move while its drone is out.

**Design**
- **`factions/gdo/units/drone_operator.md`:**
  - **`> Drone Stock`:** "Starts with **1** drone when trained; holds a **maximum of 1**";
  - **`> Launching and piloting`:**
    - "The drone is a separate flying unit **controlled directly by the player**";
    - "The Drone Operator **cannot move** while it has a drone active";
    - "It takes normal Attack orders and attacks like any melee unit; using its attack destroys it";
    - "**Explosion targets ground units only**";
  - **`> Drone limits — how a drone ends`:** the table;
  - **`> Placeholder stats`:**
    - Build Drone (F): 50 SC, 160 frames; Launch Drone (R);
    - the drone: leash 8 GU, battery 320 frames; 150 over 48 su.
- **0368:** "the drone's explosion is **AoE**".
- **`factions/gdo/structures/barracks.md > Produces`:** "Drone Operator: 100 Space Crystals, 160 frames ... Requires:
  player owns at least one Career Campus".

**Placeholders**
- 0366.
- **D33:**
  - the Barracks' key is R;
  - the drone is a HoverCraft (D32's numbers at MaxSpeed 10), Unit Control 0, armour 0/0, Groupable;
  - it appears over the operator, and launching doesn't change the selection;
  - F can buy the next drone while one is out;
  - the leash is measured centre to centre, and the drone falls in the tick its end comes.

**Code** (T89, T91)
- **The stock:** `UnitDef::stock` and `Object::stock`, bought by `Command::Ability(Ability::Buy)`.
- **The launch:** `OrderKind::Launch`, untargeted and queueable.
- **The tether:**
  - `Object::tether` on the drone, and `Object::drone` on the operator;
  - `may_move` is false while the drone is out;
  - the tether step runs after movement.
- **The explosion:** the drone's melee attack is an area hit that removes it.
- **The Barracks** gets its recipe and R.
- **The client:**
  - the operator's panel: R Launch, and F Build Drone with a progress bar;
  - the drone's panel: Move, Attack, Stop;
  - the leash ring.

**Acceptance**
- **Training.** It needs a Career Campus. It starts with one drone, F is refused while the stock is full, and after a
  launch F buys one for 50 SC in 160 frames. The stock never passes 1.
- **Launch.** The drone appears over the operator. The operator refuses Move, with feedback, until the drone is gone.
- **Its attack** on a Peacekeeper group deals 150 over 48 su by the area rule. Then the drone is gone and the operator
  is free.
- **Domains.** The drone can't attack air units. Rocketmen can shoot it, and Peacekeepers can't.
- **The leash.** Ordered 9 GU out, it falls and explodes at the leash's edge, hurting ground units there, own ones
  included.
- **The battery.** It falls without exploding 320 ticks after the launch.
- **The operator's death.** The drone falls without exploding.
- **Golden** `gdo_drones`. **QA screenshot:** a drone at its leash ring.

**Computer players:** the Barracks gains R. They train by kind, so their games don't change. A3 could use operators.

**Owns:**
- `UnitDef::stock` and `tether`; `Object::stock`, `tether` and `drone`;
- new `stock.rs`, `tether.rs` and `orders/launch.rs`, and `Ability::Buy`;
- the Barracks' recipe;
- client: `templates/gdo/{drone_operator,operator_drone}.rs`, and the Barracks' R;
- new `tests/drone_operator/`.

Read-only: `combat/area.rs`.

**After:** GK. **Model:** Opus, about 3 h.

#### GF: The Factory, Advanced Mechanics, the Industrial Powerplant, the Raptor and the Twister (Opus)

**Goal.**
- The Deployment Center builds the Factory (with a Barracks), Advanced Mechanics (with a Factory) and the Industrial
  Powerplant (with Advanced Mechanics; +60 Power).
- The Factory trains the Raptor (Q) and the Twister (W). E (Prospector) and R (Artillery Rig) stay greyed "Not in the
  build yet" until GP and GA.
- The Raptor fires a homing missile from its 60° turret.
- The Twister's 360° machine gun hits air only. Its mines come with GM, so its R and F stay greyed.

**Design**
- **`factions/gdo/structures/factory.md`:**
  - "Size 3x3; SymmetryType ABAC (exit side B, like the Barracks)"; HP 500, armour 1/9, sight 4, Power −30, BRE 2;
  - "Production as the Barracks (queue 20, refunds, rally). Panel: **Q** Raptor, **W** Twister, **E** Prospector,
    **R** Artillery Rig (greyed without Advanced Mechanics), **X** Cancel Production, **C** Set Rally Point".
- **`advanced_mechanics.md`:** 2x2, HP 350, armour 1/4, Power −20, no panel. **`industrial_powerplant.md`:** 3x3,
  HP 600, armour 1/9, Power +60.
- **`factions/gdo/units/raptor.md`:**
  - WheeledVehicle, 48x32; turn radius 80, top speed 10;
  - its turret: 60° at 10°/f;
  - HeadDisjointed Universal: Damage 14, Range 6, 14 su/f, one missile every 32 frames.
- **`factions/gdo/units/twister.md`:** 52x36; top speed 8; turret 360° at 12°/f; an Air-only FullyConnected machine
  gun, Damage 4, Range 6, timings 1/1/1/1.

**Placeholders:** 0366.

**Code**
- **The Factory's `production`** is the Barracks'. Its exit is `ExitRule::Side` on B, with the exit spot fitting the
  vehicle's oriented box, facing out (T100).
- **Advanced Mechanics and the Industrial Powerplant:** plain structures with `requires` and Power.
- **The Raptor and the Twister go live:** V2's FixedTurnRadius, U1's turret, K's HeadDisjointed, and FullyConnected
  Air.
- **The Deployment Center's Advanced menu:** Q, W and A go live.
- **The client:** `templates/gdo/factory.rs`, and the two vehicles' looks.

**Acceptance**
- **Requirements.** The Factory is refused without a Barracks, Advanced Mechanics without a Factory, and the
  Industrial Powerplant without Advanced Mechanics. The Powerplant adds +60 to the Power summary.
- **Production.** Both vehicles at 0366's cost and time, Unit Control 2 each, reserved at start. Five Raptors queued
  leave by B, facing out, with no overlap.
- **The Raptor** hits a running Peacekeeper at Range 6 every 32 frames, its turret tracking while the hull drives.
- **The Twister** hits a Sparrow. An Attack on a Peacekeeper is refused, with feedback.
- **Golden** `gdo_vehicles`: Raptors and Twisters against Recruits and Sparrows.
- **Perf:** `vehicles_perf` with Raptors, within V2's numbers.
- **QA screenshots:** the Factory's panel with E and R greyed; a Raptor's turret.

**Computer players:** none builds a Factory. A3 would.

**Owns:**
- the five kinds' live fields;
- the Deployment Center's three recipes and menu entries;
- the vehicle exit test at B;
- client: `templates/gdo/factory.rs`, the Raptor's and Twister's looks;
- new `tests/factory/`.

Read-only: `movement/wheeled.rs`, `combat/turret.rs`, K's files.

**After:** GK. **Model:** Opus, about 3 h.

#### GA: The Artillery Rig and the vehicle deploy model (Opus)

**Goal.**
- The Factory trains the Artillery Rig (R; with Advanced Mechanics).
- R deploys it in 48 frames into the Artillery Rig (Deployed): a fixed obstacle that fires a TailDisjointed area shell
  at ground targets or the ground. The shell deals 300 over 1 GU, at Range 12 with MinRange 3, with a 16-frame dodge
  window.
- R undeploys it in 48 frames.
- Deploying follows the vehicle deploy model:
  - it is queueable, and can't be cancelled;
  - the last order given meanwhile is held, and checked at completion.

**Design**
- **`factions/gdo/units/artillery_rig.md > Modes`:**
  - "**Two unit types**: **Artillery Rig** and **Artillery Rig (Deployed)**, with separate SelectionGroups";
  - "**Deploy is an order, not a mode switch** ... the deploying process is what actually changes the unit";
  - "**Not cancellable** ... Stop doesn't interrupt it. Undeploying likewise always completes";
  - "**Orders given while deploying**: the **last order given** (without Shift ...) is **held** ... one it can't carry
    out (e.g. **Move** ...) is **dropped** with the usual unavailable-command feedback".
- **`> AttackAttributes`:**
  - "TargetType: AoE — **friendly fire**";
  - "AttackType: **TailDisjointed** — the target location is locked at the end of Aiming; the blast lands there at the
    end of Firing ... Can also be fired at a ground location".
- **`> Placeholder stats`:**
  - Damage 300 over 64 su; Range 12, MinRange 3;
  - Aim 16, Firing 16, Cooldown 8, Reload 64;
  - its turret: 360° at 3°/f; deploy and undeploy 48 frames each.
- **`systems/units.md > UnitCollision`:** units that can't move never yield. A deployed Rig is "a fixed obstacle that
  never yields".
- **0263.**

**Placeholders:** 0366.

**Code** (T87, which builds T44)
- **The model:** `UnitDef::vehicle_deploy`, `OrderKind::VehicleDeploy`, `vehicle_deploy.rs` and
  `orders/vehicle_deploy.rs`, run after `step_modes`.
- **Completion** goes through `Sim::change_kind`.
- **Orders:** the held order and `can_take`. A dropped order emits T44's feedback event.
- **The Deployed kind** is immobile by its def.
- **The shell:** O3's TailDisjointed with T46's area damage.
- **The client:** R Deploy and Undeploy, the group switch (the Riot Shield's path), and the deploy's progress.

**Acceptance**
- **Deploying.** It takes 48 ticks, during which the Rig neither moves nor attacks, and Stop doesn't interrupt it. It
  stays an Artillery Rig until the last tick, then becomes the Deployed kind and joins its group.
- **Orders while deploying:**
  - a Move given meanwhile is held, then dropped at completion with the feedback event;
  - an Attack given meanwhile runs once it is deployed;
  - Shift-queued orders line up after it.
- **Undeploying** is symmetric. Deploy is unavailable until it completes.
- **Deployed,** it never yields: a friendly unit goes round it, and nothing pushes it.
- **The shell:**
  - an enemy at 2.5 GU can't be targeted (MinRange), and one at 12 GU can;
  - a target that moves 1.5 GU during the 16-frame Firing is missed;
  - own units in the blast are hurt;
  - Attack Ground works.
- **Golden** `gdo_artillery`. **QA screenshots:** the deploy, step by step.

**Computer players:** none.

**Owns:**
- T44's files, `UnitDef::vehicle_deploy` and `OrderKind::VehicleDeploy`;
- the Rig's two defs' live fields, and the Factory's R recipe;
- client: `templates/gdo/artillery_rig.rs`;
- new `tests/vehicle_deploy/` and `tests/artillery_rig/`.

Read-only: the combat phases, `toggle.rs`.

**After:** GF (V3 merged). **Model:** Opus, about 3 h.

#### GM: The Twister's mines (Opus)

**Goal.**
- A Twister starts with 3 mines.
- **Buying.** It buys more one at a time (F: 25 SC, 80 frames), up to 3.
- **Laying.** It lays one anywhere (R): it drives there and stops for 16 frames.
- **A mine** is hidden underground. An enemy infantry unit or vehicle within 32 su sets it off, and 8 frames later
  it explodes for 200 over 64 su. The blast hurts own units too, but never air.
- **Destroying them.** Area damage can destroy a mine, and so will attacks once detection exists.

**Design**
- **`factions/gdo/units/twister.md > Mines`:**
  - "**Stock:** the Twister carries up to **3 mines** and **starts with 3** ... **purchased one at a time**";
  - "**Lay Mine (targeted ability):** the player picks a location — **anywhere** ... The Twister drives there,
    **stops temporarily** to lay the mine";
  - "**Trigger:** ... When an enemy unit — **infantry or vehicle** — enters it, the mine **explodes after a short
    delay**";
  - "**Friendly fire:** the explosion **damages friendly units** ... but friendly units **do not trigger** the mine";
  - "**Air units** neither trigger mines nor take damage from them";
  - "**Destroyable:** enemies can attack and kill mines (once they can see them — detection not yet designed)".
- **`> Placeholder stats`:**
  - Buy Mine: 25 SC, 80 frames; laying: 16 frames;
  - activation radius 32 su, delay 8 frames;
  - Damage 200 over 64 su; HP 40; R Lay Mine, F Buy Mine.
- **`systems/combat.md > AoEDamage`:** area damage hits underground objects, "GDO mines included".
- **0267:** detection comes later.

**Placeholders**
- 0366.
- **D35:**
  - a mine isn't a building for defeat, and isn't selectable;
  - it is laid where the Twister stands at the end of the 16 frames;
  - every enemy ground-domain unit triggers it;
  - a mine destroyed by area damage, a neighbour's blast included, is removed without a blast of its own;
  - F, pressed while a purchase runs, is refused.

**Code** (T89, T90)
- **The stock:** the Twister's `stock`, on GD's mechanism.
- **The order:** `OrderKind::LayMine { at }`.
- **The mine:**
  - `ObjectDef::mine`, read by the trigger and fuse step in phase 5e;
  - its arm in `Hiding::of`;
  - its blast through `combat/area.rs`.
- **The client:** R and F live; the owner's look of a mine.

**Acceptance**
- **The stock.** 3 at production. F is refused at 3. After a lay, F buys one in 80 ticks for 25 SC.
- **Laying.** The Twister drives to the point and stops for 16 ticks. The mine appears there, and the Twister's
  queue goes on.
- **Hidden.** The enemy's vision never shows or records it. Its owner sees it.
- **The trigger:**
  - an enemy Peacekeeper walking within 32 su sets it off, and the blast comes 8 ticks later;
  - the owner's Peacekeeper crossing it doesn't;
  - a Sparrow overhead neither sets it off nor takes damage;
  - the blast hurts the owner's units in range.
- **Area damage.** A Grenadier's Attack Ground destroys a mine, with no blast. So does a neighbour's blast (D35).
- **Perf:** a new `mines_perf`, with 100 mines under a 200-unit crowd. Record its mean and p99 beside
  `movement_perf`.
- **Golden** `gdo_mines`. **QA screenshots:** a minefield as its owner sees it, and the same ground as the enemy sees
  it.

**Computer players:** their units set off mines as any enemy does. No game of theirs has a Twister.

**Owns:**
- `OrderKind::LayMine` and new `orders/lay_mine.rs`;
- `ObjectDef::mine` and new `combat/mines.rs`;
- the mine's arm in `vision/hidden.rs`, and the Twister's `stock`;
- client: the Twister's R and F, and the mine's look;
- new `tests/mines/` and `tests/mines_perf.rs`.

Read-only: `stock.rs` (GD's), `combat/area.rs`.

**After:** GF, GD. **Model:** Opus, about 2.5 h.

#### GP: The Prospector and the Outpost (Opus)

**Goal.**
- The Factory trains the Prospector (E; with Advanced Mechanics).
- **R Deploy into Outpost** targets a site on any Buildable ground. The Prospector drives there and, after 64 frames
  that can't be cancelled, becomes an Outpost.
- **The Outpost** is a 3x3 building that seeds the GDO build area by 6 GU.

**Design**
- **`factions/gdo/units/prospector.md`:**
  - "No weapon ... **Deploy 64 frames (4 s)**; it follows the vehicle deploy model: an order that **can't be
    cancelled** once started";
  - "Panel: Q Move, X Stop, **R: Deploy into Outpost** (targeted: a location; it drives there and deploys)";
  - TrackedVehicle, 64x48, HP 300.
- **0368:** "the Prospector follows the **vehicle deploy model** (can't cancel)".
- **`factions/gdo/structures/outpost.md`:** "Size 3x3 (the Prospector deploys in place); SymmetryType AAAA; MaxHP 600
  ... **BuildRadiusExtension 6 grid units**; Power −10". It is permanent, and it seeds the build area.
- **`core/entities.md > Placement Validation > Worker-Built Structures`** and **`> Lingering build ghosts`.**

**Placeholders**
- 0366.
- **D36:**
  - the footprint is centred on the target point;
  - the damage the Prospector had taken carries into the Outpost unchanged, as 0110 has it for the Beacon and Warp
    In;
  - its Unit Control (2) is freed at completion.
- **Held orders** follow the deploy model: an Outpost can carry out none, so each is dropped with the feedback.

**Code** (T87, deploying from a site)
- **The order's first half** is `deploy.rs`'s Drone pipeline:
  - the gate judges by what the player knows, and the lingering ghost reserves the site;
  - the walk;
  - the arrival check judges by the truth, with the deployer excluded (`Method::WorkerBuilt`).
- **The second half** is GA's 64-frame deploy in place.
- **Completion:** `change_kind` from unit to structure, the Drone's path to a Beacon without its cancel.
- **The Outpost:** `seeds_build_area: true` with BRE 6, and its Power counted from completion.

**Acceptance**
- **The ghost** is green or red by what the player knows, and a lingering ghost reserves the site.
- **The arrival check.** A Peacekeeper on the site makes the order fail, and the Prospector stops and idles. The
  Prospector itself never counts.
- **The deploy** takes 64 ticks, and Stop doesn't interrupt it. A damaged Prospector's damage carries into the
  Outpost (D36).
- **The build area grows:** a Generator can be placed 6 GU from the Outpost's edge, and not 7.
- **Unit Control.** The Prospector's 2 are freed when the Outpost completes (D36).
- **Defeat.** The Outpost counts as a building from its first tick.
- **Golden** `gdo_outpost`. **QA screenshots:** the ghost, the deploy, and the build-area overlay.

**Computer players:** none.

**Owns:**
- the Prospector's and the Outpost's live fields, and the Factory's E recipe;
- the site arm of `vehicle_deploy.rs`;
- client: `templates/gdo/prospector.rs`;
- new `tests/outpost/`.

Read-only: `deploy.rs`, `transform.rs`, `placement/build_area.rs`.

**After:** GA. **Model:** Opus, about 2.5 h.

#### GB: The Bunker and the First Aid Tent (Opus)

**Goal.**
- The Field Engineer builds a Bunker or a First Aid Tent on any Buildable ground (W Construct, then Q or W).
- **The Bunker:**
  - up to 4 GDO infantry enter it and fire their own weapons from it;
  - A ejects all of them;
  - they come out if it is destroyed.
- **The First Aid Tent** heals own and allied infantry within 3 GU by 2 HP/s. Tents don't stack.

**Design**
- **`factions/gdo/structures/bunker.md`:**
  - `> Garrison`: it holds up to 4 GDO infantry, who fire their own weapons with no bonus range and no abilities.
    They are protected, evacuated when it is destroyed, and keep counting toward Unit Control;
  - `> Placement`: "**any Buildable terrain** — not limited to the GDO build area. Follows the Worker-Built Structures
    rules";
  - `> Placeholder stats`:
    - 2x2, one door on B, HP 500, armour 2/8, sight 6; 100 SC and 320 frames;
    - "Units inside see with the Bunker's SightRange (as the Garrison)";
    - "Panel: **A: Eject All** (everyone out at once through the door); **X: Cancel construction** while being built".
- **`factions/gdo/structures/first_aid_tent.md > Healing`:** it heals infantry whose centre is within 3 GU by +2 HP/s,
  and "the highest applies". 2x2, HP 200; 75 SC and 240 frames.
- **0368:** it "heals own and allied infantry". Bunker occupants see with the Bunker's sight.
- **`factions/gdo/units/field_engineer.md > Placeholder stats`:** "**W** Construct (a menu: **Q** Bunker, **W** First
  Aid Tent ...; **Z** back)".

**Placeholders**
- 0366.
- **D37:** the Field Engineer builds as a Recruit does: from outside the footprint, shared, so n Engineers build n
  times as fast.
- **D38:**
  - a Peacekeeper that enters with its Riot Shield raised stays shielded inside, so it doesn't fire;
  - the Tent heals only units on the map, not those inside a Bunker or a Garrison, and only once it is complete.

**Code** (T99)
- **The Field Engineer's `builds`.** It is a worker that gathers nothing; if `WorkerDef::crystals` can't be empty, GB
  makes it optional (T99). Its builds are shared (E's `shares_builds`).
- **The sites:** `site` for both buildings.
- **The Bunker:**
  - F3's `holds`, with the door on B, admitting GDO infantry;
  - N2's `occupants_fire`, with no stand-in and no range bonus.
- **The Tent:** `StructureDef::heals`, and its step after the buffs.
- **The client:** the Construct menu goes live, with `templates/gdo/{bunker,first_aid_tent}.rs`, A Eject All and
  X Cancel.

**Acceptance**
- **Building.** Two Engineers build a Bunker in half the time one takes (D37). A Peacekeeper standing on the site
  holds the start back.
- **Entering.** Four Peacekeepers right-clicked onto it go in through the B door. A fifth is refused, and so is a
  Syndicate Guard.
- **Firing.** The occupants fire their own weapons, measured from the Bunker's footprint, with no bonus. A
  Rocketman's Target Lock isn't offered inside.
- **Protection.** No enemy can target an occupant. Killing the Bunker evacuates them, and they keep counting toward
  Unit Control throughout.
- **Eject All:** everyone comes out at once, through the door.
- **The Tent:**
  - an injured Peacekeeper at 2.9 GU heals 2 HP/s, and one at 3.1 GU doesn't;
  - two Tents still give 2 HP/s;
  - an allied infantry unit heals, and an enemy doesn't;
  - a Peacekeeper inside a Bunker doesn't.
- **Golden** `gdo_bunker`. **QA screenshots:** the Construct menu; a Bunker firing; the Tent's radius.

**Computer players:** none builds these. Their Peacekeepers are unaffected.

**Owns:**
- the Field Engineer's `builds`, and the `WorkerDef::crystals` change if needed;
- the Bunker's `holds` and `occupants_fire`, and `StructureDef::heals` with its new `heal.rs`;
- client: the Construct menu, and `templates/gdo/{bunker,first_aid_tent}.rs`;
- new `tests/bunker/` and `tests/first_aid_tent/`.

Read-only: `holding.rs`, `combat/occupants.rs`, `construction/site.rs`.

**After:** GK. **Model:** Opus, about 3 h.

#### GH1: The Airfield, the Hornet, ammo and assignment (Opus)

**Goal.**
- The Airfield (with Advanced Mechanics) trains Hornets (Q).
- **Assignment:**
  - each aircraft is assigned to the Airfield that built it;
  - an Airfield holds at most 4 assignments, its queue counting;
  - a right-click on another Airfield with room reassigns the aircraft;
  - when an Airfield is destroyed, its aircraft become unassigned.
- **The Hornet** fires 5 dodgeable missiles, then flies home by itself. Any order interrupts the return.
- **Landing and rearming are GH2's.** In GH1 a Hornet that comes home hovers over its Airfield, and the Airfield
  stays `Planned` in the Deployment Center's menu until GH2.

**Design**
- **`factions/gdo/structures/airfield.md`:**
  - `> Assignment`: "An aircraft is assigned to the Airfield that built it"; "An Airfield holds at most **4
    assignments**"; "**Reassigning** ... its assignments plus the aircraft in its production queue must be **fewer
    than 4** ... Input: select the aircraft and **right-click the other Airfield**"; "**No manual unassigning**";
    "**Airfield destroyed**: its aircraft become **unassigned** ... **There is no automatic reassignment**";
  - `> Production limit`: "An Airfield **can't queue another aircraft** if its assignments plus the aircraft already
    in its production queue would exceed 4";
  - `> Returning and rearming`: "**Automatic return**: when an aircraft runs out of ammo, it automatically flies back
    to its assigned Airfield"; "The automatic return **can be interrupted** by giving the aircraft any other order";
    "An unassigned aircraft has no base";
  - `> Placeholder stats`: 3x3, HP 500, armour 1/9, sight 5, Power −30; "Panel: **Q** Hornet, **W** Dragonfly (greyed
    without Propulsion Labs), **X** Cancel Production, **C** Set Rally Point".
- **`factions/gdo/units/hornet.md`:**
  - HoverCraft 48x48; "No turret (fires forward; it turns in place to aim)";
  - "**DoublyDisjointed** (not homing ... it can Attack Ground)"; Damage 30, Range 5;
  - "**RearmTimePerAmmo 32 frames** (5 missiles in 10 s)"; "F: Return to Base".
- **0368** (the Hornet is DoublyDisjointed); **0014–0019,** now in `airfield.md`.

**Placeholders**
- 0366.
- **D32:** Drag movement with the Hornet's numbers mapped.
- **D34:** in GH1, a new Hornet appears over its Airfield. GH2 makes it appear landed.

**Code** (T88)
- **Ammo:** `UnitDef::ammo` and `Object::ammo`.
- **Assignment:**
  - `Object::base`, with the Airfield's counts derived, not stored;
  - the production gate's limit (`Reject::AirfieldFull`);
  - `OrderKind::Rebase(airfield)`;
  - unassignment on the removal path.
- **The return:** `OrderKind::ReturnToBase`, pushed when ammo reaches 0.
- **The client:** `templates/gdo/airfield.rs` (Q, W `Planned`, X, C), the Hornet's ammo pips, and the right-click
  Rebase pointer.

**Acceptance**
- **The limit.** With 3 assigned and 1 queued, Q is refused ("Airfield full"). Once one of the 3 dies, Q works.
- **Reassigning.** A right-click on Airfield B assigns the Hornet to B if B's assigned and queued aircraft are fewer
  than 4, and is refused otherwise.
- **A destroyed Airfield.** Its Hornets become unassigned, F is unavailable, and none is reassigned automatically.
- **Ammo.** Five shots of 30, each dodgeable. Then the attack is unavailable and the Hornet flies home. A Move
  interrupts that, and it stays at 0.
- **Aiming.** The Hornet hovers in place and turns in place to aim. It fires only when facing its target.
- **No golden yet:** GH2 adds `gdo_air`.

**Computer players:** none.

**Owns:**
- `UnitDef::ammo`, `Object::ammo` and `Object::base`;
- `OrderKind::Rebase` and `ReturnToBase`, with new `orders/air_base.rs`;
- the Airfield's production limit, and the Hornet's and the Airfield's live fields;
- client: `templates/gdo/airfield.rs`, the ammo pips, and the Rebase pointer;
- new `tests/airfield/`.

Read-only: `movement/air.rs`, `production/queue.rs`.

**After:** GK. **Model:** Opus, about 2.5 h.

#### GH2: Landing and rearming (Opus)

**Goal.**
- **Landing.** Aircraft land on their Airfield's 4 pads, in 16 frames.
- **Rearming.** It takes RearmTimePerAmmo × the missing ammo, and the aircraft stays landed after. Any order lifts
  it off, in 16 frames.
- **Landed aircraft are ground targets.**
- **Return to Base:** F, or a right-click on its own Airfield.
- **The Airfield goes live** in the Deployment Center's Advanced menu (E).

**Design**
- **`factions/gdo/structures/airfield.md`:**
  - `> Landing slots`: "An Airfield has **4 landing slots**, one per possible assignment";
  - `> Returning and rearming`: "**Return to Base** command ... (hotkey **F**, placeholder). Right-clicking its own
    Airfield does the same"; "After rearming, the aircraft **stays landed**"; "Rearming an aircraft takes
    RearmTimePerAmmo × missing ammo"; "**Landed aircraft are ground targets** ... Air-only weapons ... can't hit it.
    Once it takes off it is an air target again";
  - "Landing and take-off: **16 frames (1 s)** each".

**Placeholders**
- 0366.
- **D34:**
  - during the 16 frames of landing and of take-off, the aircraft is an air target;
  - a new aircraft appears landed on a free pad, and takes off for its rally point; with no rally point it stays
    landed;
  - when its Airfield is destroyed, a landed aircraft stays landed, and is unassigned;
  - an order given while it rearms lifts it off with the rounds rearmed so far.
- **D40:** low Power doesn't slow rearming.

**Code** (T88)
- **The landing:** the `Object::landing` phases, and pads derived each tick.
- **The domain:** a landed aircraft's is Ground.
- **The rearm timer.**
- **Return to Base:** F, and its arm in the right-click resolution.
- **The exit:** `ExitRule::Pad`.
- **The client:** the landed look (no air shadow), the pads, and F.

**Acceptance**
- **Four Hornets** out of ammo land on the 4 pads.
- **Domains.** During the 16-tick landing a Twister can hit a Hornet. Once it has landed, the Twister can't, and a
  Peacekeeper can.
- **Rearming.** A Hornet with 2 of 5 missiles rearms in 96 ticks and stays landed. A Move lifts it off in 16 ticks,
  and then it flies.
- **F** from anywhere: it flies home, lands and rearms. A right-click on its own Airfield does the same.
- **A new Hornet** appears landed, then takes off for its rally point. With none, it stays landed (D34).
- **Golden** `gdo_air`: Hornets raid, return, rearm and raid again.
- **QA screenshots:** four Hornets on their pads, and one landing.

**Computer players:** none.

**Owns:**
- `Object::landing` and new `landing.rs`, and `ExitRule::Pad`;
- the landed domain arm in `combat/targeting.rs`, and the Deployment Center's E recipe;
- client: the landed look, the pads, and F;
- new `tests/landing/`.

Read-only: `orders/air_base.rs` (GH1's), `movement/air.rs`.

**After:** GH1. **Model:** Opus, about 3 h.

#### GL: Propulsion Labs and the Dragonfly (Opus; Fable flag)

**Goal.**
- Propulsion Labs (with an Airfield) unlocks the Dragonfly at the Airfield (W).
- **The Dragonfly** is a glider with one 90-damage homing anti-air missile, fired forward.
- When empty it flies home, lands and rearms in 64 frames, as the Hornet does.

**Design**
- **`factions/gdo/structures/propulsion_labs.md`:** 2x2, HP 350, Power −20. It needs an Airfield.
- **`factions/gdo/units/dragonfly.md`:**
  - "UnitBase - Glider (must keep moving; circles when idle; wider turns at higher speed)";
  - its Glider numbers: IdleSpeed 6, MaxSpeed 14, Acceleration 0.6, Deceleration 0.6, centripetal 0.6;
  - it fires forward: HeadDisjointed Air, Damage 90, Range 6, 20 su/f, Aim 8, Fire 2, Cooldown 4, Reload 0;
  - Ammo 1, RearmTimePerAmmo 64; F Return to Base.
- **0368:** homing. **0405:** "the Dragonfly's SeparationRadius is 48".
- **`factions/gdo/structures/airfield.md`:** its landing rules, as GH2.

**Placeholders**
- 0366.
- **D34 (the Dragonfly):**
  - it approaches its slot straight at IdleSpeed, then lands vertically over the 16 frames, stopping over the slot,
    as the Specter lands at a door;
  - it takes off vertically, and leaves at IdleSpeed.
- **Its forward shot** follows the Sparrow's rule (H2's).

**Code**
- **The approach:** H1's glider with T88's approach, a glide path that ends over the pad.
- **The client:** the Deployment Center's R goes live, and so does the Airfield's W. The Dragonfly gets its look.

**Acceptance**
- **W** is greyed without Propulsion Labs. Then a Dragonfly costs 175 SC and takes 320 frames.
- **One missile** shoots down a Sparrow. Then the Dragonfly flies home, lands in 16 ticks and rearms in 64.
- **It never stops in the air.** It lands from any heading.
- **Separation:** two Dragonflies end 48 su apart, pushed at the gliders' third of the HoverCraft strength (0412).
- **Golden** `gdo_dragonflies`. **QA screenshot:** a Dragonfly on its landing approach.

**Computer players:** none.

**Owns:** the glider arm of `landing.rs`; Propulsion Labs' and the Dragonfly's live fields; the Deployment Center's R
recipe and the Airfield's W; client: the Dragonfly's look; new `tests/dragonfly/`.

Read-only: `movement/glider.rs`.

**After:** GH2. **Model:** Opus, about 2.5 h. Fable is the fallback if the landing approach won't settle.

#### GS: The Supply Tower and the Supply Chopper (Opus; gated on D41)

**Goal.**
- The Deployment Center builds the Supply Tower (with a Radar Station).
- **The Tower** comes with one free, attached Supply Chopper, and trains more (100 SC, 160 frames).
- **Choppers** fly to a Supply Delivery Station, pick up all its Supplies, and drop them at a Tower.
- **Scheduled deliveries.** An attached Chopper leaves in time to reach its station as the next delivery arrives.
- **No units.** The Chopper carries none: its transport is TBD.

**Design**
- **`factions/gdo/economy.md > SupplyTower`:**
  - `> On Placement`: "One Supply Chopper is spawned on the tower at no additional cost";
  - `> Landing and Attachment`: "drops off all carried supplies and is gradually repaired at no cost"; "immediately
    evicts any existing landed Supply Chopper that is not the attached chopper"; "Any command given to an attached
    Supply Chopper breaks its attachment";
  - `> Scheduled Deliveries`: "departs at a time calculated so it arrives at the SDS simultaneously with the next
    expected supply delivery".
- **`factions/gdo/units/supply_chopper.md`:** "Right-click SupplyDeliveryStation: issues PickUpSupplies command (fly
  to SDS, land, pick up all available supplies)", and the other commands. HoverCraft, 60x60, HP 150.
- **`core/map.md > SupplyDeliveryStation`.** **0077:** "deferred to the air milestone ... what GDO spends Supplies
  on".

**Placeholders**
- **D41**, and the repair rate (`Placeholder (not in the design: the Tower's repair rate)`). The Chopper's Drag
  numbers are its doc's.

**Code**
- **The pick-up** reuses M4-I's stations (`economy/stations.rs`'s `take` and its claims) for a whole station.
- **The orders:** PickUpSupplies, DropOffSupplies and AttachToTower, and the Tower's schedule.
- **Landing** is GH2's, on the Tower.
- **The bank:** the GDO's Supplies, which the HUD already shows at 0.

**Acceptance**
- **Placing a Tower** brings its free, attached Chopper.
- **A scheduled run** reaches the station within a tick of its delivery, picks up everything, and lands back at the
  Tower. The GDO's Supplies rise by the delivery.
- **The landing rules:** eviction; a command breaks attachment; repair while landed.
- **Sharing.** Agents and a Chopper share a station by M4-I's claims.
- **Golden** `gdo_supplies`. **QA screenshot:** the Tower's panel with S Schedule.

**Computer players:** a GDO Chopper could empty the stations their Syndicate Agents use, but none plays in their
games.

**Owns:** the Supply Tower's and the Chopper's live fields; the three orders and new `orders/supply_run.rs`; the
Tower's schedule; client: `templates/gdo/{supply_tower,supply_chopper}.rs`; new `tests/supply_tower/`.

Read-only: `economy/stations.rs`, `landing.rs`.

**After:** GR, GH2. **Model:** Opus, about 3 h.

### Syndicate

#### SV: The Vehicle Assembly and the Assault Bike (Opus)

**Goal.**
- A Tier 2 Tunnel builds the Vehicle Assembly (with a Chemical Plant).
- **It produces vehicles into the network,** and they leave only through Tier 2+ Tunnels.
- **The Assault Bike** (Q) is a fast wheeled bike with a dodgeable area mortar.
- **The other slots.** W (Shadebringer), E (Laser Battery) and R (Transport) stay greyed until SB, SL and ST1.

**Design**
- **`factions/syndicate/structures/vehicle_assembly.md`:**
  - 3x3, HP 600, 250 SC and 480 frames;
  - its panel is the Hideout's: Q Bike, W Shadebringer, E Laser Battery, R Transport.
- **`factions/syndicate/structures/README.md`:**
  - the tech tree;
  - n = 6 armour, and 1%/s regeneration;
  - `> Rally Point Behavior`: on the surface the unit is auto-ejected; none, or on a Tunnel: it stays in the network;
    on an expansion: it goes to the spot above.
- **`factions/syndicate/tunnels.md > Tunnel Network`:** "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle,
  Mech)"; `> Look by tier`: "Tier 2: the paths get **wider and well paved**, supporting vehicles".
- **`factions/syndicate/units/assault_bike.md`:**
  - Wheeled, 56x32; turn radius 128, top speed 12; turret 40° at 8°/f;
  - DoublyDisjointed Ground area: 120 over 48 su, Range 5, 8 su/f, Aim 8, Fire 2, Cooldown 4, Reload 34;
  - Tunnel Space 2.

**Placeholders:** 0367.

**Code**
- **The expansion** is M4-F's, with `requires` and its tier.
- **Its `production`** is the Hideout's (`ExitRule::Network`).
- **Ejecting** through Side A's exit path must fit a 56x32 box, and a 72x48 one for ST1 (T100).
- **The Bike** uses V2's wheeled movement, U1's turret, and M4-O's area projectile.
- **`EXPAND_MENU`'s Vehicle Assembly** becomes a `Kind`.
- **The client:** `templates/syndicate/vehicle_assembly.rs`, after the Hideout's, and the Bike's EjectMenu tile.

**Acceptance**
- **Expand** is refused at Tier 1 ("Tier 2"), and without a Chemical Plant. At Tier 2 it builds at 250/480, with
  armour 6/54, regenerating 1%/s.
- **The Bike.**
  - It is produced into the network, reserving Tunnel Space 2 at the start.
  - A rally point on the surface auto-ejects it.
  - Ejecting it from a Tier 1 Tunnel is refused.
  - From a Tier 2 Tunnel, five queued Bikes come out of the exit path without overlapping.
- **The mortar** hits a group by the area rule, can be dodged, and can Attack Ground.
- **Golden** `syndicate_vehicles`. **QA screenshots:** the Vehicle Assembly's panel; the EjectMenu with the Bike.

**Computer players:** A2b's Syndicate expands by kind, so its games don't change. A3 would add Bikes.

**Owns:**
- the Vehicle Assembly's and the Bike's live fields, and `EXPAND_MENU`'s entry;
- the vehicle exit test on Side A;
- client: `templates/syndicate/vehicle_assembly.rs` and the Bike's look;
- new `tests/vehicle_assembly/`.

Read-only: `sim/expansions.rs`, `movement/wheeled.rs`.

**After:** SK. **Model:** Opus, about 2.5 h.

#### SN: The Null Chamber (Opus)

**Goal.**
- A Tier 2 Tunnel builds the Null Chamber (with a Hideout).
- **Its queue.** While the player owns one, each manned unit in the network takes its turn in a chamber: 4 at a time
  per chamber, 2 s each, first come first served.
- **The buff.** A unit whose turn is done leaves the network with a 20-second buff: +2 GU of sight, and 20% less
  damage taken, applied before armour. Leaving again restarts it.

**Design**
- **`factions/syndicate/structures/null_chamber.md > Null Chamber Exit Buff`:**
  - "every **manned** Syndicate unit that exits the Tunnel Network through any Tunnel receives a temporary buff";
  - "**Manned units**: all Syndicate units except satellites";
  - "**Percentage damage-taken reduction**, applied **before armor**: damage taken = Attack Damage × (1 − reduction)
    − armor";
  - "**Refresh**: exiting the Tunnel Network again instantly resets the buff to full duration".
- **`> Placeholder stats`:** "duration **20 s**; **+2 grid units SightRange**; **20%** ... ends abruptly; no research
  needed. Throughput: each Null Chamber buffs **4 units at a time**, each buff taking **2 s** inside before the unit
  can leave buffed; a unit that leaves before its turn leaves unbuffed".
- **0369** confirms these. **`systems/combat.md > DamageModifierOrder`,** step 1.

**Placeholders**
- 0367.
- **D48:**
  - the queue runs in the order units entered the network, a produced unit from its production;
  - a unit whose turn is done keeps it while it stays inside.
- **Detection,** one of the buff's effects, isn't built (not designed).

**Code** (T98)
- **The queue:** `Held::service` and the network services step. Split `network.rs` first (477 lines).
- **The buff:** `BuffKind::NullChamber`.
- **Its effects:** the eye's sight reads the buff, and `combat::pre_armor_multiplier` the reduction.
- **The client:** `EXPAND_MENU`'s entry, and the buff mark (N5's).

**Acceptance**
- **The effects.** A Guard that spent 2 s in the network with a chamber leaves buffed: its sight is 6 + 2, and a hit
  of 10 deals 10 × 0.8 − armour.
- **The queue.** Six Guards enter together, with one chamber. After 2 s four are done, and after 4 s all six. Ejected
  at 3 s, four leave buffed and two unbuffed.
- **The end.** The buff ends at once, 20 s after the unit leaves.
- **Without a chamber,** no buff.
- **Golden** `syndicate_null_chamber`.

**Computer players:** A2b doesn't build it, so its games don't change.

**Owns:**
- the split of `network.rs`, `Held::service`, and new `network/services.rs`;
- `BuffKind` (widening `Buff::ability`);
- the buff's sight arm in `vision/sight.rs`, and its arm in `pre_armor_multiplier`;
- client: the Chamber's menu entry;
- new `tests/null_chamber/`.

Read-only: `buffs.rs` (beyond `BuffKind`), `vision.rs`.

**After:** SK. **Model:** Opus, about 2 h.

#### SB: The Vault and the Shadebringer's Sludge (Opus)

**Goal.**
- **The Vault.** A Tunnel builds it (Tier 1, with a Hideout). It serves units anywhere in the network, 2 at a time.
- **The Shadebringer.** The Vehicle Assembly trains it (W, with a Vault): a wheeled support vehicle with two tanks.
- **Sludge.** R toggles it: a trail that slows ground units by 40%.
- **Cover** (F) stays greyed until SC.

**Design**
- **`factions/syndicate/structures/vault.md`:**
  - "a Vault re-arms and recharges units **anywhere in the Tunnel Network**";
  - "each Vault re-arms or recharges **2 units at a time**; others wait their turn (network-wide, first come first
    served) ... Shadebringer tanks refill at 5 per second each";
  - 2x2, HP 400; 150 SC and 400 frames.
- **`factions/syndicate/units/shadebringer.md`:**
  - `> Sludge (toggle)`:
    - "Infantry and non-hover vehicles that step into sludge are slowed";
    - "**Mechs** are slowed like vehicles";
    - "**Drill Units** are slowed while on the surface";
    - "Not affected: hover units (HoverVehicle, HoverCraft) and air units";
    - "**Affects everyone**";
    - "the sludge is spread **behind** it ... if it drives back over sludge ... it is slowed like any other
      vehicle";
  - `> Charges and recharging`: "**Each charge is a tank** that **drains while its toggle is on** ... When a tank
    runs **empty**, that ability switches off";
  - `> Placeholder stats`:
    - each patch lasts 10 s; units are slowed by 40% for 2 s after leaving it, and the slow "**refreshes while they
      stay in it**"; the trail is 48 su wide;
    - tanks of 100, draining 10 per second, refilling at 5 per second in the network with a Vault;
    - "a toggle can be switched on with **any charge**"; "Toggles follow the mode-switch rule"; R Sludge, F Cover.
- **0369** confirms the toggles and the refresh.

**Placeholders**
- 0367.
- **D44:**
  - the trail is laid only while it moves, behind its motion, so at its front when it reverses; a tank drains while
    its toggle is on, moving or not;
  - a unit is in sludge while its centre is, as for Rugged terrain;
  - entering the network switches both toggles off.
- **D48:** the Vault's queue order.

**Code** (T93, T98)
- **The tanks:** `UnitDef::tanks` and `Object::tanks`; the toggles through `toggle.rs`.
- **The trail:** `Sim::patches`, with a coarse grid.
- **The slow,** in `Object::max_speed`.
- **The Vault's service:** T98's step, refilling tanks.
- **The menus:** `EXPAND_MENU`'s Vault becomes a `Kind`, and the Vehicle Assembly's W goes live.
- **The client:** the trail, and the tank bars.

**Acceptance**
- **The toggle** switches on with any charge, drains 10/s, and switches off when empty. It is refused at 0.
- **The trail:** 48 su wide, behind the Shadebringer, each patch lasting 10 s.
- **The slow:**
  - a Peacekeeper crossing the trail moves at 0.6 × its speed while in it and for 2 s after;
  - the Shadebringer is slowed driving back over its own trail;
  - a HoverVehicle (the Colonists' Drone) and air units aren't.
- **The Vault:** in the network with a Vault, tanks refill 5/s. With three Shadebringers and one Vault, two refill at
  a time.
- **Perf:** `movement_perf` with four trails active, within noise.
- **Golden** `syndicate_sludge`. **QA screenshot:** a trail, with units in it.

**Computer players:** their infantry is slowed by sludge as anyone's is. No game of theirs has a Shadebringer.

**Owns:**
- `UnitDef::tanks`, `Object::tanks`, `Sim::patches`, and new `patches.rs`;
- the Shadebringer's toggles, its arm in `max_speed`, and the Vault's arm in `network/services.rs`;
- `EXPAND_MENU`'s Vault, and the Vehicle Assembly's W;
- client: the trail and the tank bars, and `templates/syndicate/shadebringer.rs`;
- new `tests/sludge/`.

Read-only: `toggle.rs`, `network/services.rs` (SN's).

**After:** SV. **Model:** Opus, about 3 h.

#### SC: The Shadebringer's Cover (Opus)

**Goal.**
- F toggles Cover: a 64 su wall of smoke behind the Shadebringer, each piece lasting 12 s.
- **What it blocks.** The smoke blocks sight into it and past it, for every unit and building but satellites.
- **Units inside** see nothing.

**Design**
- **`factions/syndicate/units/shadebringer.md > Cover (toggle)`:**
  - "The smoke **blocks line of sight into it and behind it** for every unit except **satellites**: it's not possible
    to see into the smoke, or past it, from the other side";
  - "This is the **first sight-blocking mechanic** in the game";
  - "**Units inside the smoke can't see anything**";
  - "**Structures too**".
- **`> Placeholder stats`:** "each piece of smoke lasts **12 s**; the wall is **64 su (1 grid unit)** wide and blocks
  sight at any height except satellites'".
- **`factions/syndicate/units/prism.md > Sight`:** "It sees through Shadebringer smoke".
- **`core/vision.md`:** vision is otherwise a plain radius.

**Placeholders**
- 0367.
- **D43:**
  - an attacker revealed by damage (`core/vision.md > Attacks from outside vision`) is shown even in or behind smoke;
  - smoke blocks a Scan as it blocks any sight but satellites';
  - a unit is inside smoke while its centre is, as for Rugged terrain.

**Code** (T93)
- **The smoke:** patches, as SB's.
- **The occlusion,** in `vision/sight.rs`. Each eye carries the revision of the smoke near it.
- **Satellites:** a flag the Prism's def sets (`UnitDef::satellite`, which SP fills).

**Acceptance**
- **Behind a wall.** An enemy behind a wall is unseen, and the tiles past it aren't Visible. From the side, both show.
- **Inside.** A unit inside the smoke stamps no tiles.
- **Structures.** A structure's sight is blocked. A satellite's isn't, through a test def until SP.
- **Clearing.** After 12 s each piece clears, and the view comes back.
- **Perf:** a new `smoke_perf`, with four Shadebringers' walls and 200 sight sources: within 20% of `vision_perf`'s
  mean, with its p99 recorded.
- **Golden** `syndicate_smoke`. **QA screenshot:** the fog with a smoke shadow.

**Computer players:** smoke blocks their units' sight as anyone's. No game of theirs has a Shadebringer.

**Owns:**
- the smoke's arm of `patches.rs`;
- the occlusion in `vision/sight.rs`, and the eye's smoke revision;
- `UnitDef::satellite`;
- client: the smoke's look;
- new `tests/smoke/` and `tests/smoke_perf.rs`.

Read-only: `vision/scans.rs` (GR's), `vision/reveal.rs`.

**After:** SB. **Model:** Opus, about 3 h.

#### SL: The Laser Battery (Opus)

**Goal.**
- The Vehicle Assembly trains the Laser Battery (E, with an Optics Lab). It is tracked.
- **The battery** holds 100 charge and charges 5/s. The laser surges every 16 frames for 4 × charge^(2/3), using 10,
  or whatever has charged.
- **Mobile,** it fires at ground targets, with a 30° turret.
- **Deployed** (R, 48 frames), it becomes immobile, with a 360° turret and air targets only.
- **Amplify** (F) marks the next surge to use the whole battery.

**Design**
- **`factions/syndicate/units/laser_battery.md`:**
  - `> Battery`:
    - "**always charging** at its charge rate, attacking or not";
    - "**A normal surge draws twice what charges in one surge interval**";
    - "**Empty battery**: it keeps firing **just as often**, but each surge uses only what charged since the last
      one";
  - `> Attack`: "**Damage per surge** is proportional to the **charge used^(2/3)**"; FullyConnected; Ground while
    mobile, Air while deployed;
  - `> Deploy`: "**Switching modes works like the GDO Artillery Rig**"; two unit types;
  - `> Placeholder stats`:
    - capacity 100; 5 per second; "surge interval **16 frames**"; a normal surge uses 10;
    - "**Damage per surge = 4 × charge^(2/3)**: a normal surge deals about 18.6; an empty-battery surge (5) about
      11.7; Amplify from full (100) about 86";
    - Range 8, Aim 8; turret 30° mobile, 360° deployed, at 6°/f; deploy and undeploy 48 frames;
    - "**Amplify** (hotkey **F**): pressing it **marks the next attack** ... Usable mobile and deployed; not
      queueable".
- **0369:** "Press Amplify on a battery that is between surges: the next surge empties it in one big hit".

**Placeholders**
- 0367.
- **D48:** the battery charges inside the network too, as "always charging" reads.

**Code** (T97, T87)
- **The battery:** `UnitDef::battery` and `Object::battery`, which `change_kind` keeps.
- **The surge's damage,** in the FullyConnected phase.
- **Amplify:** `Ability::Amplify`.
- **The pair** is data on GA's `vehicle_deploy`.
- **The Vehicle Assembly's E** goes live.
- **The client:** the battery bar, and the beam and its surges.

**Acceptance**
- **The surges.** With a full battery, each deals 18.6 (to f32 precision), and the charge falls by 5 a second. After
  about 20 s it is empty, and each surge deals 11.7.
- **Amplify,** pressed between surges: the next surge deals 4 × c^(2/3) for the whole charge, 86 from full, and
  leaves it empty (0369's Verification).
- **Domains.** Mobile, it can't target a Sparrow. Deployed, it targets only air.
- **Deploying** follows T87: GA's tests, repeated on this pair, held orders included. The battery charges while it
  deploys.
- **Golden** `syndicate_lasers`. **QA screenshot:** a deployed Battery's beam surging.

**Computer players:** none.

**Owns:**
- `UnitDef::battery`, `Object::battery`, and new `battery.rs`;
- the surge damage arm in `combat/`, and `Ability::Amplify`;
- the Battery's two defs' live fields, and the Vehicle Assembly's E;
- client: the battery bar and the beam;
- new `tests/laser_battery/`.

Read-only: `vehicle_deploy.rs` (GA's).

**After:** SV, GA. **Model:** Opus, about 3 h.

#### SD: The air doors, the Forge, the Aerospace Assembly and the Phantom (Opus)

**Goal.**
- **The air doors.** Tier 3 Tunnels get their two roof doors. Air units go in by the entry door and come out by the
  exit door, one every 8 frames, beside the ground lane.
- **The Forge.** A Tier 2 Tunnel builds it (with a Vehicle Assembly). It is a prerequisite only.
- **The Aerospace Assembly.** A Tier 3 Tunnel builds it (with a Forge). It trains the Phantom (Q, with an Optics Lab).
- **The Phantom** is a HoverCraft gunship with a 75° turret, hitting air and ground.
- **The other slots.** W (Specter) and E (Prism) stay greyed until SA and SP.

**Design**
- **`factions/syndicate/tunnels.md > Look by tier`:** "**Tier 3**: adds **two horizontal circular sliding doors**,
  **one for flying units entering and one for flying units exiting**. Aircraft use these doors, not the Side A
  paths".
- **`factions/syndicate/tunnel_menus.md > EjectMenu`:** air units leave by the Tier 3 air exit door, in parallel with
  ground units, "one aircraft per 8 frames (0.5 s), placeholder (the same rate for the air entry door)"; "The **air
  doors are on the Tunnel's roof**: the entry door over the entry path's half of the Tunnel, the exit door over the
  exit path's half".
- **`factions/syndicate/tunnels.md > Transit delay`:** "after a unit **enters** the network, there is a **small delay
  before it can exit**", for air units as for any.
- **`tunnels.md > Transit`:** "Tier 3+: Air units (Hover Craft, Glider)". **0250:** the air doors waited for this.
- **The buildings:**
  - `factions/syndicate/structures/forge.md`: 2x2, HP 400; 200 SC and 480 frames;
  - `aerospace_assembly.md`: 3x3, HP 600; 300 SC and 640 frames; Q Phantom, W Specter, E Prism.
- **`factions/syndicate/units/phantom.md`:**
  - "HoverCraft (enters and exits only through Tier 3 Tunnels' air doors)";
  - FullyConnected Universal: Damage 12, Range 5, Aim 2, Fire 1, Cooldown 3, Reload 10;
  - turret 75° at 8°/f; 44x44, HP 70; Tunnel Space 2.

**Placeholders**
- 0367.
- **D32:** the Phantom's Drag numbers.
- **D46:**
  - an air unit goes in once its centre is over the entry door;
  - a Phantom appears over the exit door at once, at flying height.
- **The doors' exact points** within their halves of the roof (`Placeholder (not in the design: the points)`).

**Code** (T96)
- **`AIR_DOORS = true`.**
- **The doors:** their spots in `Paths`, and the air lane in the eject queue.
- **Enter** for air units.
- **The Aerospace Assembly's production** is the Hideout's.
- **The Phantom** goes live.
- **The menus:** `EXPAND_MENU`'s Forge and Aerospace Assembly become `Kind`s.
- **The client:** the doors' look, and the Aerospace Assembly's panel.

**Acceptance**
- **Out.** A produced Phantom goes into the network and auto-ejects by the air exit door to its rally point. Ejected
  from a Tier 2 Tunnel, it is refused ("Tier 3").
- **Two lanes.** Five Phantoms come out one every 8 ticks, while Guards walk out of Side A at the same time.
- **In.** A right-click on an own Tier 3 Tunnel takes a Phantom to the entry door, and it is held.
- **Mid-upgrade.** A Tunnel being upgraded to Tier 3 admits no air unit (0249: the tier it had before).
- **Stranded.** Losing the last Tier 3 Tunnel destroys the aircraft inside.
- **The Phantom** hits a Sparrow and a Peacekeeper.
- **Golden** `syndicate_air`. **QA screenshots:** the doors, and a Phantom coming out.

**Computer players:** the change to `admits` touches air units only, and theirs are infantry, so their games don't
change.

**Owns:**
- `AIR_DOORS`, the door spots, the air lane, and the air arm of `orders/enter.rs`;
- the Forge's, the Aerospace Assembly's and the Phantom's live fields, and their `EXPAND_MENU` entries;
- client: the doors, and `templates/syndicate/aerospace_assembly.rs`;
- new `tests/air_doors/`.

Read-only: `network/services.rs`, `movement/air.rs`.

**After:** SK, SV (the Forge needs a Vehicle Assembly). **Model:** Opus, about 3 h.

#### ST1: The Subterranean Transport, above ground (Opus)

**Goal.**
- The Vehicle Assembly trains the Transport (R, with a Forge): a DrillUnit that drives as a tracked vehicle above
  ground, and carries 4 infantry.
- **A ground unit above ground.** Its base's domain is Underground, so its surfaced kind needs its own.
- **Loading.** Infantry board it by right-click.
- **Unloading.** F unloads them all around it.
- **Into a Tunnel.** It can drive into a Tier 2+ Tunnel loaded, and its passengers stay aboard.
- **Its death.** Its passengers die with it.

**Design**
- **`factions/syndicate/units/subterranean_transport.md`:**
  - "It carries **infantry only**"; "UnitBase - DrillUnit (confirmed, Q0071 ...)";
  - `> Above-ground mode`: "behaving as a tracked vehicle. In this mode, **infantry can be loaded** into it, and it can
    **unload** them";
  - `> Entering the Tunnel Network loaded`: "they travel along inside it and stay aboard ... can't be ejected from a
    Tunnel on their own. **They always keep counting against the Tunnel Space cap**";
  - `> Destroyed with passengers`: "**all infantry inside die** with it";
  - `> Placeholder stats`: 72x48, HP 300, armour 3/3, sight 5, Tunnel Space 3; 4.5 su/f above ground; "**Capacity 4**
    infantry (... **4 heavy infantry, e.g. 4 Guards** ...)".
- **`systems/units.md > DrillUnit`:** "Domain - Underground"; "Has an above-ground mode in which it is stationary or
  behaves as tracked".
- **`factions/syndicate/tunnels.md > Regeneration inside`:** "Units carried inside a Subterranean Transport don't count
  as being in the network themselves (assumed; the transport itself regenerates)".
- **0369:** "The Transport holds 4 infantry"; it "loads by right-click and F unloads all". **0265.**

**Placeholders**
- 0367.
- **D42:** the Laser Battery's tracked numbers.
- **D45:**
  - only the owner's infantry board, not allies';
  - they board from any side;
  - they don't fire out;
  - aboard in the network, they get no regeneration and no Null Chamber turn (the doc's assumption, to confirm).

**Code** (T94)
- **The surfaced kind's domain:** `UnitDef::domain: Option<Domain>` overrides the base's, read through one accessor
  wherever the base's domain is read (targeting's `object_domain`, the collision layers, hiding, the mines' trigger).
  The surfaced kind sets Ground. The burrowed kind keeps its base's Underground.
- **The carrier:** `UnitDef::carries`, and `Place::Carrier`.
- **Boarding** goes through Enter on a carrier, with F1's `admits`.
- **Unload All:** F3's evacuation spots, around an oriented shape.
- **Death:** passengers die on the removal path.
- **The network:** a carrier is held with its passengers.
- **The menus:** the Vehicle Assembly's R goes live.
- **The client:** the passenger pips, and F.

**Acceptance**
- **Boarding.** Four Guards board by right-click. A fifth is refused.
- **Unloading.** F brings them all out around it, in a ring, without overlapping.
- **Its death,** with 3 aboard: all 3 die, and their Tunnel Space is freed.
- **The network.** It enters a Tier 2 Tunnel loaded. The EjectMenu lists the Transport and not its passengers.
  Ejected, it still carries them.
- **Tunnel Space** counts the passengers throughout.
- **In the network,** the Transport regenerates and its passengers don't (the doc's assumption).
- **A ground unit.** A Peacekeeper can target the surfaced Transport, a Twister can't, and it collides with ground
  units.
- **Golden** `syndicate_transport`. **QA screenshot:** a loaded Transport with its pips.

**Computer players:** none.

**Owns:**
- `UnitDef::carries` and `Place::Carrier`, with new `carry.rs`;
- `UnitDef::domain` and its accessor, and the call sites that read a base's domain;
- the carrier arms of `orders/enter.rs`, `sim/holding.rs` and the removal path;
- the Transport's live fields, and the Vehicle Assembly's R;
- client: `templates/syndicate/transport.rs`;
- new `tests/transport/`.

Read-only: `holding.rs`, `network/services.rs`.

**After:** SV, SD (the Forge). **Model:** Opus, about 2.5 h.

#### SP: The Orbital Command and the Prism (Opus)

**Goal.**
- A Tier 3 Tunnel builds the Orbital Command (with an Aerospace Assembly).
- **The Prism.** The Aerospace Assembly's E trains it (with an Orbital Command): an unmanned satellite.
- **The launch.** It goes straight up out of the air exit door (32 frames) and unfolds (32 frames). It is vulnerable
  throughout.
- **Moving.** It drifts at 1 su/f, and has three thrusts of 3 GU over 1 s each.
- **One way.** It can never go back into the network.
- **Sight.** It sees through smoke.

**Design**
- **`factions/syndicate/units/prism.md`:**
  - `> Launch`: "**launched like a rocket straight up**, and **transforms in the air into its orbital form** ... It
    **can be shot down while launching and unfolding**";
  - `> One way only`: "the **only unit that can't get back into the Tunnel Network**";
  - `> Movement`: "**Directional thrust**: a few **charges**, each giving a quick burst of displacement in a chosen
    direction"; "**Charges never come back**";
  - `> Sight`: "It sees through Shadebringer smoke";
  - `> Placeholder stats`:
    - 40x40, HP 100, armour 0/0, sight 6, Tunnel Space 2; MaxSpeed 1 su/f;
    - "**Launch** 32 frames (straight up) and **transform** 32 frames (vulnerable during both)";
    - "**Thrust charges: 3**; each moves it **3 grid units** over 1 s in a chosen direction".
- **`factions/syndicate/structures/orbital_command.md`:** 2x2, HP 400; 250 SC and 640 frames.

**Placeholders**
- 0367.
- **D47:**
  - the panel is Q Move, X Stop and R Thrust, which is targeted (a point gives the direction) and not queueable;
  - orders given while it launches and unfolds are held until it has unfolded.
- **D32:** Drag numbers at MaxSpeed 1.

**Code**
- **The launch phases,** on T96's exit lane.
- **The thrusts:** `UnitDef::thrust` and `Object::thrust`.
- **One way:** `admits` refuses it (`NoTransit::OneWay`).
- **Satellites:** `UnitDef::satellite` set, read by smoke (SC) and by the Null Chamber's "manned" (SN).
- **The menus:** `EXPAND_MENU`'s Orbital Command, and the Aerospace Assembly's E.

**Acceptance**
- **The launch:** 32 + 32 ticks, during which it can be hit as an air unit. Then it drifts.
- **Thrusts.** Three, each moving it 3 GU in 16 ticks in the direction clicked. A fourth is refused.
- **One way.** A right-click on an own Tunnel is refused ("can't re-enter").
- **Smoke.** It sees through smoke (SC's test, now on the real kind).
- **The Null Chamber** doesn't buff it.
- **No golden of its own:** SR's covers it.

**Computer players:** none.

**Owns:**
- `UnitDef::thrust`, `Object::thrust`, and new `thrust.rs`;
- the launch arm of the air lane, and `NoTransit::OneWay`;
- the Orbital Command's and the Prism's live fields, and their menu entries;
- client: `templates/syndicate/prism.rs`;
- new `tests/prism/`.

Read-only: the air lane (SD's), `patches.rs`.

**After:** SD. **Model:** Opus, about 2 h.

#### SA: The Specter (Opus; Fable flag)

**Goal.**
- The Aerospace Assembly's W trains the Specter (with a Vault and a Chemical Plant).
- **Flying.** It is a glider bomber. It lifts off and lands through a Tier 3 Tunnel's air doors, 24 frames each, and
  can be hit as an air unit meanwhile.
- **Its bomb.** It carries one: 400 over 80 su, with a fixed 24-frame fall.
- **Re-arming.** Empty, it flies to the nearest Tier 3 Tunnel and re-arms inside in 160 frames, given a Vault (2 at a
  time).
- **Orders.** 0272's attack-order rules apply.

**Design**
- **`factions/syndicate/units/specter.md`:**
  - "flies as a **Glider** (must keep moving), but **lifts off and lands vertically** through a Tier 3 Tunnel's air
    doors";
  - "**Can miss**: the bomb falls to a locked location ... (DoublyDisjointed, assumed)"; it "can **Attack Ground**";
  - "**Out of ammo** ... **fly back to the nearest own Tier 3 Tunnel** ... **even if the player owns no Vault** ...
    If there is **no Tier 3 Tunnel**, it **circles**";
  - "**Attack orders and ammo**":
    - "**Ordered to attack while unarmed** ... It **flies to the nearest own Tier 3 Tunnel** to re-arm";
    - "**On entering the Tunnel its orders are wiped**";
    - "**After it bombs**: if the target is **destroyed** ... it goes on to its next queued order ... If the target
      **survives** ... it **ignores the orders queued after the Attack** and flies home";
    - "With **no Tier 3 Tunnel**, an unarmed Specter **ignores Attack orders**";
  - `> Placeholder stats`:
    - "**Lift-off and landing**: vertical, **24 frames** each at the air door (it can be hit during both, as an air
      unit)";
    - the bomb: "Damage **400** total over the circle; AoERadius 80 su; Range 1 grid unit; a fixed **24-frame fall**";
      turret 20°;
    - "**Re-arming**: 160 frames inside the network with a Vault";
    - 56x56, HP 140; Glider numbers IdleSpeed 6, MaxSpeed 11, Acceleration 0.5, Deceleration 0.5, centripetal 0.5.
- **0369:** "the Specter can be hit during lift-off and landing". **0405:** SeparationRadius 64, pushed at the
  gliders' third of the HoverCraft strength (0412). **0272.**

**Placeholders**
- 0367.
- **D46 (the approach):** it flies over the entry door at IdleSpeed, stops over it, and descends for the 24 frames.
  It lifts off for 24 frames and leaves at IdleSpeed.
- **D48:** the Vault's queue order.

**Code** (T88, T96, T98)
- **Ammo 1,** and the return to the nearest Tier 3 Tunnel.
- **Landing:** into the network by the entry door, over 24 frames.
- **Re-arming:** the Vault's service.
- **The bomb:** DoublyDisjointed with a fixed fall (T100).
- **0272's rules,** in its order handling.
- **The Aerospace Assembly's W** goes live.

**Acceptance**
- **The bomb.** It lands 24 ticks after release. A target that leaves the 80 su circle in time is missed. Attack
  Ground works.
- **After a bomb:**
  - target destroyed: it goes on to the next queued order;
  - target alive: it flies home, ignoring the queue, and entering the Tunnel wipes its orders.
- **Unarmed:**
  - ordered to attack, it goes to re-arm;
  - with no Tier 3 Tunnel, the Attack is dropped and it circles.
- **Re-arming.** 160 ticks inside with a Vault. Without one it waits. It shares the two Vault slots with
  Shadebringers.
- **Lift-off and landing:** 24 ticks each, during which a Phantom can shoot it.
- **Golden** `syndicate_specter`. **QA screenshot:** a Specter landing at a door.

**Computer players:** none.

**Owns:**
- the Specter's order rules, in new `orders/specter.rs`;
- the door-landing arm of `landing.rs`, and the fixed-fall arm of `combat/projectiles.rs`;
- the Specter's live fields, and the Aerospace Assembly's W;
- client: the Specter's look;
- new `tests/specter/`.

Read-only: `landing.rs` (GH2's), the air lane (SD's), `network/services.rs`.

**After:** SD, SB (the Vault's service). **Model:** Opus, about 3 h. Fable is the fallback if the door approach won't
settle.

#### ST2: Burrowing (Opus; Fable flag)

**Goal.**
- **Burrow** (R, 48 frames) takes the Transport underground into its Burrowed kind:
  - hidden from other players;
  - crossing any drillable tile at 3.5 su/f, under buildings and cliffs;
  - passing through every underground thing: expansions, lowered turrets, mines;
  - still hit by area damage.
- **Surface** (R, 48 frames) brings it up where a tracked vehicle could stand and no building or unit is. The spot is
  reserved meanwhile.

**Design**
- **`factions/syndicate/units/subterranean_transport.md`:**
  - `> Burrow`: "it can travel **under buildings and obstacles such as cliffs**: across **all drillable terrain**";
  - `> Underground`: "While underground it is **hidden** from other players ... Undetected, it can't be **targeted**,
    but **area (AoE) damage still affects it**";
  - `> Underground movement`: "A burrowed transport **passes through every underground structure** (expansions of any
    player, a retracted pop-up Laser Turret, GDO mines), as underground units already pass through each other";
  - `> Surface`:
    - "**It can't surface on top of a building, or on ground that tracked vehicles can't traverse** ... The check is
      the tile itself";
    - "It **can surface above an expansion**";
    - "it can't surface where it would overlap an above-ground unit, and the spot is reserved once surfacing starts";
  - `> Burrowing and surfacing work like vehicle deploy`.
- **`systems/units.md > Underground Collision`** (`> Surfacing`, `> Reserved while surfacing`), and `> DrillUnit`.
- **0263, 0266.**

**Placeholders**
- 0367.
- **D45:** burrowed, it can't enter a Tunnel: it surfaces and drives in.
- **Held orders** follow the deploy model (`> Burrowing and surfacing work like vehicle deploy`): an Unload given
  while it burrows is dropped, since it can't unload underground; one given while it surfaces runs once it is up.

**Code** (T95, T87)
- **The pair:** GA's `vehicle_deploy`.
- **Nav:** the drillable class grid (Z2's lazy grids).
- **Collision:** `Domain::hard_collision` is false for Underground (gap G14 closes).
- **Hiding:** an underground unit's arm in `Hiding::of`.
- **Surfacing:** its check and its reservation.
- **Targeting:** the TODO in `combat/targeting.rs` is resolved by kind.

**Acceptance**
- **Burrowing:** 48 ticks, not cancellable. Then the enemy never sees or records it, and its owner does.
- **Underground,** its path runs under a cliff and an enemy building, and through an expansion and a mine. The mine
  doesn't go off.
- **Damage.** A Grenadier's Attack Ground over it hurts it. A Peacekeeper can't target it.
- **Surfacing:**
  - refused over a building, or over Rugged or water;
  - allowed over an expansion;
  - refused where a unit stands;
  - once it has started, a unit walking toward the spot is held back by the reservation.
- **Held orders,** by T87: an Unload given while it burrows is dropped with the feedback, and one given while it
  surfaces runs. A right-click on a Tunnel while burrowed is refused (D45).
- **Golden** `syndicate_burrow`. **QA screenshots:** the owner's view, and the enemy's.

**Computer players:** none.

**Owns:**
- the burrowed kind's live fields, the underground arm of `movement/` and the drillable class grid;
- `Domain::hard_collision`'s Underground arm, and the unit arm of `vision/hidden.rs`;
- new `surfacing.rs` and its reservations, and `combat/targeting.rs`'s TODO;
- new `tests/burrow/`.

Read-only: `vehicle_deploy.rs`, `combat/mines.rs`, `nav/`.

**After:** ST1, GA, GM. **Model:** Opus, about 3 h. Fable is the fallback if surfacing and its reservation won't
settle.

#### SR: The Relay (Opus)

**Goal.**
- Deployed Laser Batteries relay their beams through Prisms.
- **Choosing a target.** Relay highlights the Prisms that every selected Battery can reach: 8 GU from Battery to
  Prism, and chains of Prisms 8 GU apart.
- **The armed Prism.** The chosen Prism is armed, and fires at anything the player sees within 6 GU. Each surge is the
  sum of the Batteries' own.
- **The Batteries** are locked in. A broken chain ends the relay.

**Design**
- **`factions/syndicate/units/laser_battery.md > Relay`:**
  - "This **highlights every Prism they can relay to**: each Prism **in range of the Batteries themselves**, plus
    every Prism connected to one of those by a **contiguous chain** of Prisms";
  - "Each selected Battery **locks onto its nearest Prism**";
  - "The target Prism becomes **armed**. It **targets enemies in its own range at will**, ground or air";
  - "Each surge **discharges the Batteries as normal**";
  - "**While relaying** ... **Stop Relaying**; **Relay**, which picks another target Prism; **Amplify**: available
    while relaying **if the target Prism has an active target**";
  - "**Every selected Battery must reach the chain**";
  - "**Sight**: the armed Prism can attack **anything the player sees** within its range";
  - "**Directing the armed Prism**: ... **select the armed Prism and give it a specific target** (Attack)";
  - "**Several Batteries add up**"; "**Chain breaks** ... the Batteries **stop relaying automatically**".
- **`> Placeholder stats`:** "Battery-to-Prism range 8 grid units; Prism-to-Prism link range 8 grid units; an armed
  Prism's attack Range 6 grid units".

**Placeholders**
- 0367.
- **D47:** the armed Prism's panel adds A Attack.

**Code** (T97)
- **The order:** `OrderKind::Relay { prism }`.
- **The chain,** derived each tick.
- **The armed Prism's attack:** T49's picker, firing at the surge cadence.
- **Each surge** discharges every relaying Battery.
- **The client:** the highlights, the beams, and the panels.

**Acceptance**
- **Highlights.** Two Batteries and a chain of three Prisms: the chain is highlighted, and a Prism off it isn't. If
  one Battery has no Prism in range, nothing is.
- **The armed Prism** hits a Peacekeeper 6 GU away that any own unit sees. Each surge is the sum of the Batteries',
  and they discharge.
- **Amplify** while relaying, with an active target: every Battery's whole charge goes into one surge.
- **A broken chain.** Destroying a Prism on it ends the relay, and the Batteries go back to ordinary deployed
  behaviour.
- **Directing.** Selected, the armed Prism takes an Attack on a chosen target.
- **Golden** `syndicate_relay`. **QA screenshots:** the highlights, and the beams.

**Computer players:** none.

**Owns:**
- `OrderKind::Relay`, and new `relay.rs` and `orders/relay.rs`;
- the armed Prism's attack arm;
- client: the relay targeting, the highlights and the beams;
- new `tests/relay/`.

Read-only: `battery.rs` (SL's), `combat/structures.rs`.

**After:** SL, SP. **Model:** Opus, about 3 h.

### Tail

#### MQa: QA of the first subset (Opus; only with the partway playtest, §F)
- **The walkthrough:** both factions' ground lines from §F's subset, hot-seat. The computer players don't use M6
  units.
- **Perf:** `vehicles_perf`, `mines_perf` and `movement_perf` with sludge, with their means and p99 recorded.
- **Soak invariants:**
  - a deploying unit holds no move;
  - stocks stay within [0, max];
  - patches expire;
  - mines never move.
- **`docs/playtest/m6a.md`:** the placeholders, and what isn't in yet (the GDO's aircraft, smoke, the Transport, the
  Specter, the Prism).
- **After:** W6's slices. **Model:** Opus, about 2 h.

#### S6a and S6b: Answer sweeps (Opus)
- They apply the answers to D32–D48 that arrive after their slices have merged. An answer that arrives before a
  slice launches goes into its brief instead.
- **S6a** runs in W9, and **S6b** in W10. **Model:** Opus, about 2 h each.

#### MQ: M6's QA (Opus)
- **The walkthrough:** up both factions' trees, with every M6 unit and building.
- **Perf:**
  - `vehicles_perf`, `mines_perf` and `smoke_perf`;
  - `air_perf` with Hornets landing and Phantoms at the doors.
- **New soak invariants:**
  - ammo stays within [0, max];
  - a landed aircraft is on its own Airfield's pad;
  - every passenger's carrier exists;
  - a burrowed unit is hidden from every other player;
  - the armed Prism has a live chain;
  - battery charges stay within [0, capacity].
- **`docs/playtest/m6.md`:** the placeholders; the Supply Chopper's transport; no detection.
- **After:** everything. **Model:** Opus, about 3 h.

---

## §C Waves

```
M5   ... V1 -> V2 -> V3 (new: before N10) ... X1 (alone)
W1   X2 (M5) | GK | SK
W2   M (M5) | GR | GD                          -> the M5 playtest builds from M's merge
W3   GF | SV | SN
W4   GA | SB | V4 (Fable flag)
W5   GM | GP | SL
W6   GB | GH1 | SD                             -> the first subset is in (§F)
W7   MQa (optional) | GH2 | ST1                -> playtest 6a, the ground war, at W7's end
W8   SC | SP | GL (Fable flag)
W9   SA (Fable flag) | SR | S6a
W10  ST2 (Fable flag) | GS (if D41 says build it) | S6b
W11  MQ                                        -> M6's playtest
```

- **V3 is M5's.** It runs after V2 and before N10. With V2 and N6 in order, it most likely shares N6's wave. M5 grows
  by one slice, and by a wave only if no slot is free.
- **M6 starts at X1's merge** (T101). X2 and M share W1 and W2 with M6's first slices:
  - **GK and SK** add def modules but no def fields, so X2's export just covers them. They don't touch X2's files.
  - **M's walkthrough** is of M5. M6's slices that merge in W2 add only greyed `Planned` entries (GK, SK) or
    self-contained buildings (GR, GD). The playtest build is the commit at M's merge.
- **Dependencies on M5's movement slices:**
  - **V1:** every vehicle; all are non-square. The aircraft are square.
  - **V2:** the wheeled ones (Raptor, Twister, Bike, Shadebringer), and the vehicle NavClass.
  - **V3:** the tracked ones (Prospector, Artillery Rig, Laser Battery, the surfaced Transport). V4 comes after it.
  - **H1:** the gliders (Dragonfly, Specter). **K:** HeadDisjointed (Raptor, Missile Turret, Dragonfly), and the
    Arsenal for the Defense System.

  All of them merge before W1.
- **Dependencies inside M6:**
  - GK → {GR, GD, GF, GB, GH1}, then GF → {GA, GM}, GA → {GP, SL, ST2}, GD → GM, and GH1 → GH2 → {GL, GS};
  - SK → {SV, SN}, then SV → {SB, SL, SD, ST1}, SB → {SC, SA}, SD → {ST1, SP, SA}, {SL, SP} → SR, and ST1 → ST2;
  - GR → GS, and GM → ST2 (the burrowed Transport passes through mines).
- **The critical path:**
  - **Capacity** is the binding constraint: 28 M6 slices after X1 (29 with MQa), beside X2 and M, at three builds a
    wave.
  - **The longest chains:**
    - GK → GH1 → GH2 → GL;
    - SK → SV → SD → SP → SR;
    - SK → SV → SD → ST1 → ST2.

    Each fits the waves with a wave or two of slack.
  - **Slack:** GB, GR, GD, SN and GS have no successors in the content (GS none at all), so any of them can trade
    waves with a slice whose registries differ.
- **Playable content by wave:**
  - W2: Scan, the Defense System and Missile Turrets; the Drone Operator;
  - W3: the Factory, Raptors and Twisters; the Vehicle Assembly and Bikes; the Null Chamber;
  - W4: the Artillery Rig; the Shadebringer's Sludge; crushing;
  - W5: mines; the Prospector and Outposts; the Laser Battery;
  - W6: the Bunker and the Tent; the Phantom and the air doors;
  - W7: Hornets (the Airfield goes live with GH2); the Transport;
  - W8: smoke; the Prism; the Dragonfly;
  - W9: the Specter; the Relay;
  - W10: burrowing; the Supply Tower.
- **Machine load:**
  - three builds in every wave from W1 to W10;
  - at most one Fable build a wave, and only as a fallback: V4 in W4, GL in W8, SA in W9, ST2 in W10. Every slice
    starts on Opus.
- **Answers.** One arriving before a slice launches goes into its brief. Later ones go to S6a (W9) or S6b (W10).
- **The first subset (§F).** W1–W6 hold it. MQa (W7) is its QA, only if the partway playtest is taken. Otherwise
  W7's slot takes SC or SP early.

**Shared registries.** T84's struct-update defaults make def fields open to every slice: each new field is one line
in its struct and in its `NONE`, plus its own kinds' literals. Without them, T82's rule (one new field per def struct
per wave) applies, and W4, W5, W6 and W8 each have two `UnitDef` fields, so the waves stretch by about one. The
other shared items are open to every slice, as in the addendum, with the slice that merges second resolving the
one-line conflict:

| Registry | W1 | W2 | W3 | W4 | W5 | W6 | W7 | W8 | W9 | W10 |
|---|---|---|---|---|---|---|---|---|---|---|
| Kind registry | GK, SK (GDO block first), then closed | — | — | — | — | — | — | — | — | — |
| `Category` | GK `Mine` | — | — | — | — | — | — | — | — | — |
| `UnitBaseDef` field | — | — | — | V4 `crushes` | — | — | — | — | — | — |
| `UnitDef` field | — | GD `stock`, `tether` | — | GA `vehicle_deploy`, SB `tanks` | SL `battery` | GH1 `ammo` | ST1 `carries`, `domain` | SC `satellite`, SP `thrust` | SA `bomb_fall` | — |
| `StructureDef` / `ObjectDef` field | — | GR `abilities` widened | — | — | GM `ObjectDef::mine` | GB `heals`; GH1 `airfield` | — | — | — | GS `supply_tower` |
| `Object` / `Held` fields | — | GR `cooldown_until`; GD `stock`, `tether`, `drone` | SN `Held::service` | GA `vehicle_deploy`; SB `tanks` | SL `battery` | GH1 `aircraft` | GH2 (inside `aircraft`) | SP `thrust` | — | — |
| `OrderKind` | — | GD `Launch` | — | GA `VehicleDeploy` | GM `LayMine` | GH1 `Rebase`, `ReturnToBase` | — | — | SR `Relay` | GS's three |
| `Command::Ability` | — | GD `Buy` | — | SB `Sludge`, `Cover` | SL `Amplify` | — | — | SP `Thrust` | — | — |
| `ExitRule` / `Place` | — | — | — | — | — | — | GH2 `Pad`; ST1 `Place::Carrier` | — | — | — |
| `BuffKind` | — | — | SN (new; `NullChamber`) | — | — | — | — | — | — | — |
| Tuning modules | — | GR `tuning/vision.rs` (scans) | SN `tuning/tunnel.rs` (services) | GA new `tuning/vehicles.rs`; SB new `tuning/patches.rs` | GM new `tuning/mines.rs` | GH1 new `tuning/aircraft.rs`; SD `tuning/tunnel.rs` (doors) | — | SC `tuning/patches.rs` (smoke) | — | — |

- **Two slices in a wave on one tuning module:** none. SN and SD both touch `tuning/tunnel.rs`, in W3 and W6.
- **The EjectMenu and ExpandMenu.** SK writes the order. Each Syndicate slice turns its own `Planned` entry into a
  `Kind`: SV, SN and SB in W3–W4, SD (two entries) in W6, SP in W8.
- **The Deployment Center's menus.** GK writes the `Planned` slots. GR (A, F), GF (Q, W, A), GH2 (E), GL (R) and GS
  (R in Basic) make theirs live, each in a different wave.

**Split first** (T28), continuing the addendum's list:
- `defs/units.rs` (454 lines): **GK** (W1);
- `vision.rs` (471): **GR** (W2);
- `network.rs` (477): **SN** (W3);
- `movement.rs` (488): V2 splits it in M5; if it hasn't, **V3** does;
- `object.rs` (279): no split needed if GH1 keeps the aircraft's state in one `Object::aircraft` struct (T88).

**m6-common**, a brief for every M6 slice, carries:
- the shared-registries table above;
- T101's checklist;
- the AI-games check;
- the golden naming (`gdo_*`, `syndicate_*`), and the scratchpad folder for QA screenshots.

---

## §D Technical decisions, from T84

**Fable pieces:** none is planned on Fable. Every slice starts on Opus, after H1's trial. The flagged ones (V3, V4,
GL, SA, ST2) are movement problems that may need it, so §C keeps them one a wave.

### T84. The M6 kinds, registered in two slices, and struct-update defaults (GK, SK)
- **Recommend:**
  - **Two kinds slices, side by side in W1.** GK registers 23 GDO kinds and SK 15 Syndicate kinds, each as one block
    appended to the registry, GDO first. The one that merges second rebases its block. No new kind is produced, so no
    golden moves.
    - If appending kinds does move a hash (a per-kind array in hashed state), they run back to back instead.
    - The registry then closes for M6.
  - **Names:**
    - the pairs: `ArtilleryRigDeployed`, `LaserBatteryDeployed` and `SubterraneanTransportBurrowed`, as the
      Peacekeeper (Shielded) is;
    - the GDO drone: `OperatorDrone`, shown as "Drone", since the Colonists' `Drone` exists;
    - and `Mine`.
  - **Kept out until their slice** (T69's rule): no `constructs`, `production`, `builds` or menu entry goes live in
    GK or SK.
  - **Struct-update defaults.** If X1 leaves the defs as static literals, GK gives `UnitDef`, `StructureDef` and
    `ObjectDef` a `const NONE` for their optional fields, and every literal ends with `..UnitDef::NONE`. A later field
    is then one line in its struct and its `NONE`, plus its own kinds' literals. That is about 40 files fewer per new
    field, and M6 adds about 16 def fields (11 on `UnitDef`).
- **Reused:** T69's pattern, and G2's tests that walk every kind.
- **Alternatives:**
  - **Kinds with their slices.** Indices would follow merge order, and parallel slices would collide at the
    registry's end. T69 rejected it.
  - **One kinds slice** of 38 kinds: about 5 h, past the slice size.
  - **T82's rule (one field per def struct per wave),** without the defaults: no refactor, but the waves stretch by
    about one (§C).

### T85. Tracked movement: the linear law is analytic (V3)
- **Superseded in part by 0417** (2026-11-26, answering Q0153 parts 1 and 3). The law is now the tread law: SpinRate = 2 × MaxSpeed ÷ TrackWidth, falling linearly to 0 at MaxSpeed. Every bend at full speed now needs slowing, so the corner choice picks a speed rather than comparing an arc with a spin. V3's brief (`scratchpad/briefs/m5v3-tracked-movement.md`) carries it. The rest below stands.
- **Recommend:**
  - **The model.** SpeedTurnRadius gives a radius of r = |v| × ratio, so the angular rate is v / r = 1/ratio a frame
    at any speed. A tracked unit therefore turns at a constant rate, and a turn of θ takes θ × ratio frames, whatever
    its speed. Speed only decides how much room the turn takes: an arc of length v × θ × ratio.
  - **Maneuver choice, analytic.** At each path corner the follower compares an arc at the current speed (if it fits
    the free space, measured on the nav grid) with braking to 0 and spinning. It takes the faster one that fits. No
    precomputed tables are needed, since the law is exact.
  - **Spinning in place** at 0 runs at the same 1/ratio rate (D42's placeholder).
  - **Reverse** is V2's Reversing, with the tracked limits.
  - **Collision:** V1's whole-step try and halving. A spin is a step with no translation.
  - **Nav:** V2's vehicle NavClass (Rugged impassable), for both tracked bases and the surfaced DrillUnit.
- **Reused:** V2's follower and braking, V1's oriented collision, Z2's class grids.
- **New:** `movement/tracked.rs`, and the corner choice.
- **Alternatives:** maneuver tables (the design allows them). They would be needed only if the law weren't linear.

### T86. Crushing (V4)
- **Recommend:**
  - **A base flag,** `UnitBaseDef::crushes` (TrackedVehicle, Mech), beside the existing `crushable`.
  - **In collision:** `movement::space::blocks` lets a crusher whose Locomotion is Moving or Reversing pass through an
    enemy crushable. Every other pair collides as now: a stationary crusher, own or allied crushables, and a crushable
    moving into a crusher.
  - **The kill** comes in the movement phase, after the tick's moves. Each moving crusher's oriented shape is
    overlap-tested against the enemy crushables near it (the movement grid's query), and those it overlaps die
    through the death path (Unit Control freed, lineage, `SimEvent::Crushed`). The kill isn't damage: there is no
    armour, no reveal and no shield.
  - **Avoidance.** A crusher's follower ignores enemy crushables. Crushables keep avoiding crushers.
- **Reused:** the death path, the movement grid, V1's overlap test.
- **New:** the flag, the crush step and the event.

### T87. Deploying vehicles: T44 built, then reused (GA, GP, SL, ST2)
- **Recommend:**
  - **GA builds T44 as decided** (M4 plan):
    - `UnitDef::vehicle_deploy: Option<VehicleDeploy { into, frames }>` on both kinds of a pair, and
      `OrderKind::VehicleDeploy`;
    - `vehicle_deploy.rs` and `orders/vehicle_deploy.rs`, run after `step_modes`;
    - completion through `Sim::change_kind`;
    - held orders: without Shift, a new order replaces only the held slot and the queue;
    - the gate skips capability filters while a unit deploys;
    - at completion, each order is checked by one `can_take(object, order)`, and a dropped one emits the event that
      gives the unavailable-command feedback.
  - **The deployed kind is immobile by its def,** so "can't move" and "never yields" follow from the existing rules.
  - **GP adds a site,** for the Prospector:
    - the order's first half is `deploy.rs`'s Drone pipeline: the gate on what the player knows, the lingering ghost,
      the walk, and the arrival check on the truth with the deployer excluded;
    - its second half is T44's in-place 64 frames;
    - completion is the unit-to-structure `change_kind` that the Drone's Beacon already uses, without the
      transformation's cancel.
  - **SL** is data only. Its battery survives `change_kind`, since the kind change keeps object state.
  - **ST2** adds Surface's check and reservation (T95).
- **Reused:** `change_kind`, `deploy.rs`'s site checks and ghosts, the Riot Shield's client path for the group switch.
- **New:** T44's files, the site arm, and the immobile def.

### T88. Aircraft: ammo, assignment, landing and rearming (GH1, GH2, GL; SA reuses)
- **Recommend:**
  - **Ammo:**
    - `UnitDef::ammo: Option<AmmoDef { max, rearm_ticks_per_ammo }>`;
    - the aircraft's state in one struct, `Object::aircraft: Option<AircraftState { ammo, base, landing }>`, hashed
      while set;
    - a shot spends one round as it launches; at 0 the unit can't attack (`combat::weapon` answers none).
  - **Assignment:**
    - `base` is set at production to the producer;
    - each Airfield's assigned and queued counts are derived, never stored;
    - reassigning is `OrderKind::Rebase(airfield)`, checked at the gate (fewer than 4 assigned and queued), and
      applied when issued: the aircraft is then flying to its new base;
    - the removal path clears `base` when an Airfield goes;
    - the production gate refuses the fifth (`Reject::AirfieldFull`).
  - **Pads:** 4 points in the footprint, a placeholder layout (`Placeholder (not in the design: the pads' layout)`). A
    landing aircraft claims the lowest free pad, derived each tick as E's spot claims are.
  - **The return:** `OrderKind::ReturnToBase` (F, or a right-click on its own Airfield), pushed by itself when ammo
    reaches 0. As an order, any new order replaces it.
  - **Landing phases,** on the order:
    - Approach: a HoverCraft brakes to a stop over its pad; a glider takes GL's straight approach at IdleSpeed;
    - Descend, 16 frames;
    - Landed: rearming one round every `rearm_ticks_per_ammo`, and staying landed after;
    - Ascend, 16 frames, on any order, which then runs.
  - **Domain:** a landed aircraft's domain is Ground. Targeting reads the per-object domain, and the base's
    otherwise. Descend and Ascend count as Air (D34's placeholder, matching the Specter's "as an air unit").
  - **New aircraft:** `ExitRule::Pad`. The aircraft appears landed on a free pad, then lifts off for its rally point.
- **SA reuses** ammo, the return, and the Descend and Ascend phases, with the Tunnel's air entry door as its pad.
  "Landed" is being held in the network, where re-arming is the Vault's service (T98).
- **Reused:** H1's air movement and separation; the production queue and gate; the right-click resolution; the
  removal path.
- **New:** `orders/air_base.rs`, `landing.rs`, `ExitRule::Pad`, and the domain override.
- **Alternatives:** pads as stored reservations. They would have to follow every death and re-order, where derived
  claims don't.

### T89. Unit stocks: drones and mines (GD builds it, GM reuses it)
- **Recommend:**
  - **The def:** `UnitDef::stock: Option<StockDef { max, start, price, frames }>`.
  - **The state:** `Object::stock: Option<Stock { count, buying: Option<u32> }>`, the purchase's progress in frames.
  - **Buying** (F) is `Command::Ability(Ability::Buy)`. It keeps orders, like the Riot Shield and Convert. It pays at
    the press, through the bank's checked spend (T7). It is refused while a purchase runs or the stock is full (D35's
    placeholder). The purchase advances in the economy phase, and isn't slowed by Power, since units don't draw it.
  - **Spending** is an order: GD's `Launch` and GM's `LayMine { at }`. Both are queueable.
- **Reused:** the `Ability` command, T7, the production progress pattern.
- **New:** `stock.rs`.

### T90. Mines (GM)
- **Recommend:**
  - **What a mine is:**
    - an object of a new `Category::Mine`, owned, with `Footprint::Silhouette { 16, 16 }` and an underground domain;
    - it is not a unit (no movement) and not a structure (no tile footprint);
    - not selectable, and not a building for defeat (D35's placeholder).
  - **The def:** `ObjectDef::mine: Option<MineDef { trigger_su: 32, delay: 8, blast }>`, with the blast 200 over
    64 su.
  - **Hidden:** a new arm in `Hiding::of` hides it from every player but its owner, until detection exists (T32).
  - **No collision.** It is in no movement grid's layer, so units pass over it, and a burrowed Transport through it.
  - **The trigger,** in phase 5e before the tick's hits:
    - an armed mine fuses when an enemy unit of the Ground domain, whose shape is within 32 su of the mine's centre
      (shape gap), is found by the movement grid's query;
    - the blast comes 8 ticks later, through `combat/area.rs`: friendly fire, ground and underground, never air;
    - then the mine is removed.
  - **Destroyed by damage** before its blast: removed, with no blast (D35).
  - **Laid** where the Twister stands at the end of its 16 frames (D35).
- **Reused:** T46's area damage (which already hits underground objects), T32's `Hiding`, the movement grid's
  queries, `SimEvent::Blast`.
- **New:** `Category::Mine`, `combat/mines.rs`, and `mines_perf`.
- **Alternatives:**
  - **A mine as a structure:** structures have tile footprints and count for defeat.
  - **A mine as a unit:** every unit has movement and collides.

### T91. The operator's drone (GD)
- **Recommend:**
  - **The drone** is a unit of `OperatorDrone`. The launch spawns it over the operator, not from a building.
  - **The tether:** `Object::tether: Option<Tether { operator, battery_until }>` on the drone, and `Object::drone:
    Option<ObjectId>` on the operator, whose `may_move` is false while it is set.
  - **The step,** after movement:
    - beyond 8 GU, centre to centre, from its operator, the drone falls and explodes where it is (a T46 blast);
    - at its battery's end, or if its operator has gone, it falls without a blast;
    - falling is instant (D33's placeholder), with `SimEvent::DroneFell`.
  - **Its attack** is a melee area hit, centred on the drone, that removes the drone as it lands.
- **Reused:** the melee phases, T46, the removal path, H1's air movement.
- **New:** `tether.rs`.

### T92. Scan: a cooldown cast and a timed sight source (GR)
- **Recommend:**
  - **The command** is T76's `Command::Cast { casters, ability, at }`:
    - the ability enum gains Scan, which the Radar Station's `abilities` lists;
    - a cooldown instead of energy: `Object::cooldown_until: Option<Tick>`, hashed while set;
    - no vision check on `at` ("left-click any point on the map");
    - the cast goes to every station it is issued to, by No smart-casting, and each station whose cooldown is ready
      scans. Radar Stations are Groupable by their doc, so a grouped pair scans twice (D39 asks).
  - **The effect:** `Vision::scans`, a list of (owner, centre, radius, until), each stamped as a sight disc with no
    ElevationModifier, as for air units. Expiry comes from a queue, as the fog delay does. History is hashed and
    saved.
  - **Hidden objects stay hidden** (D39's placeholder): a scan is sight, not detection.
- **Reused:** `vision/sight.rs`'s disc stamps and incremental counts; T76's gate; N5's cast cursor.
- **New:** `vision/scans.rs`, and the cooldown.

### T93. Ground patches: sludge and smoke (SB, SC)
- **Recommend:**
  - **Sim state, not objects,** as projectiles are (T45): `Sim::patches: Vec<Patch { kind, centre, radius, until }>`,
    hashed while not empty, and indexed in a coarse grid (2 GU cells) for lookups.
  - **Laying.** A Shadebringer with a toggle on drops a patch behind its motion (its rear edge, or its front when it
    reverses; D44) each time it has moved 24 su since the last. The trail is a chain of discs: 48 su across for
    sludge, 64 su for smoke.
  - **The tanks:** `Object::tanks: Option<[Tank; 2]>` in fixed point, drained each tick while on, and refilled by the
    Vault's service (T98).
  - **Sludge:**
    - in the movement phase, a unit of an affected base whose centre is in a sludge patch (D44) has its
      `slowed_until` set to now + 32;
    - `Object::max_speed` is × 0.6 while it is slowed.
  - **Smoke (SC):**
    - an eye whose centre is in a smoke patch stamps nothing;
    - otherwise a tile is blocked if it lies in a smoke patch, or if the segment from the eye to its centre crosses
      one;
    - satellites (`UnitDef::satellite`) ignore smoke.
  - **Smoke, incrementally.** The smoke near an eye (the patches within its sight plus the largest radius, from the
    grid) has a revision, which the `Eye` includes. So only the eyes near changing smoke stamp again. An object counts
    as seen as before, by its tiles, so one inside smoke is unseen. Reveals ignore smoke (D43's placeholder).
- **Reused:** the vision stamps and eyes, `Object::max_speed` (N1a), `toggle.rs`.
- **New:** `patches.rs`, the occlusion, and `smoke_perf`.
- **Alternatives:**
  - **Smoke as objects:** heavier, and they would need their own exclusions.
  - **A per-tile smoke count with no shadows:** cheaper, but it can't block sight "behind it".

### T94. Carriers and passengers (ST1)
- **Recommend:**
  - **The def:** `UnitDef::carries: Option<HoldsDef>`, F1's struct, with a capacity of 4 that admits the owner's
    infantry.
  - **The place:** passengers are held at a new `Place::Carrier(id)`, appended so the hash stays put.
  - **Boarding** is Enter on a carrier. F1's `admits` extends to carriers, and a passenger leaves the map on contact.
  - **Unload All** (F) uses F3's evacuation spots, in a ring around the oriented shape, and only above ground.
  - **Death.** Unlike a building's evacuation, the removal path kills the passengers held at a dying carrier (their
    lineage ends, and their Tunnel Space is freed).
  - **The network:** a loaded Transport is held at `Place::Network`, its passengers at `Place::Carrier(id)`. The
    EjectMenu counts only the carrier. Passengers aren't in the network ("Units carried inside a Subterranean
    Transport don't count as being in the network themselves (assumed; the transport itself regenerates)",
    `factions/syndicate/tunnels.md > Regeneration inside`), so they neither regenerate nor take a Null Chamber turn
    (D45 confirms).
  - **The surfaced kind is a ground unit.** Its base, DrillUnit, has Domain Underground, so ST1 adds `UnitDef::domain:
    Option<Domain>`, an override read through one accessor wherever a base's domain is read today. The surfaced kind
    sets Ground, and the burrowed kind keeps Underground. T88's landed aircraft differ by object, not by kind, so they
    stay an arm of `object_domain` in targeting only.
- **Reused:** F1's `HoldsDef` and `admits`, F3's spots, `orders::enter`, the removal path.
- **New:** `Place::Carrier`, and `carry.rs`.

### T95. Underground movement and surfacing (ST2)
- **Recommend:**
  - **Nav:** the Burrowed kind (Domain Underground) paths on a drillable class grid. It is one of Z2's lazy grids,
    whose passability is the tile's `drillable` bit and which ignores structures.
  - **No collision:** `Domain::hard_collision` is false for Underground (gap G14 closes), matching "underground units
    already pass through each other". Underground structures and mines are in no layer it meets.
  - **Hidden:** an underground unit's arm in `Hiding::of`, for every player but its owner.
  - **Targets.** Area damage hits it (T46 already includes underground objects). No attack targets it: `domain_allows`
    stays false for Underground, and the surfaced kind is Ground, which resolves the `combat/targeting.rs` TODO by
    kind.
  - **Surface** is T87's order with a check as it starts:
    - the oriented shape's tiles are passable for a tracked vehicle, and hold no building (expansions are
      underground, so they don't count);
    - no above-ground unit overlaps the shape.

    The spot is then reserved for the 48 frames. `Sim::reservations` is read by `movement::space` as an obstacle,
    and by other surfacing checks.
- **Reused:** T87, Z2's grids, T46, T32.
- **New:** the drillable grid, `surfacing.rs`, and the reservation.

### T96. The Tier 3 air doors (SD; SA and SP reuse them)
- **Recommend:**
  - **`AIR_DOORS = true`.** `network::passes` admits Air at Tier 3, and `admits` refuses the Prism
    (`NoTransit::OneWay`).
  - **The doors.** `Paths` gains `air_entry` and `air_exit`: roof points over the entry path's half and the exit
    path's half, which turn and flip with the Tunnel.
  - **Out.** Eject entries for air kinds release from the air exit door on their own lane, one per 8 frames
    (`tuning::tunnel`), beside the ground lane. The queue keeps its order within each lane.
  - **In.** An air unit's Enter flies it to the entry door, and it is taken in once it is over the door (D46's
    placeholder). The Specter's Descend (24 frames) comes first.
  - **Lifting off.** A Specter rises for 24 frames, and a Prism launches for 32 and unfolds for 32. A Phantom appears
    over the door at once (D46).
  - **Production:** the Aerospace Assembly is the Hideout's `ExitRule::Network`, and its rally auto-ejects through
    the doors.
- **Reused:** the eject queue, the transit delay, Enter, `admits` and `passes`.
- **New:** the door spots, and the air lane.

### T97. The Laser Battery's battery, Amplify and Relay (SL, SR)
- **Recommend:**
  - **The def:** `UnitDef::battery: Option<BatteryDef { capacity, per_second, surge_ticks, surge_charge,
    coefficient }>`.
  - **The state:** `Object::battery: Option<Battery { charge, amplify }>`, the charge in fixed point. It is kept by
    `change_kind`, and charges every tick in every mode, on the map and held in the network (D48's placeholder; the
    services step charges held ones).
  - **The surge:**
    - the FullyConnected cycle is Aim 8, then one surge every 16 frames while engaged (Firing 1, Cooldown 1, Reload
      14: `Placeholder (not in the design: the split)`);
    - its damage is k × c^(2/3), where c is min(charge, surge_charge), or the whole charge when `amplify` is set,
      which clears it.
  - **Amplify** (F): `Command::Ability(Ability::Amplify)` sets the flag. It isn't queueable, and it keeps orders.
  - **Relay (SR):**
    - `OrderKind::Relay { prism }`. The gate checks that every selected Battery has a Prism within 8 GU, and that the
      target can be reached by a breadth-first search over the owner's Prisms with links of at most 8 GU;
    - each tick the chain is derived again: each Battery to its nearest Prism, then the search's path to the target.
      If it breaks, the order ends;
    - the armed Prism (`Object::armed`, derived) attacks with T49's picker, at anything the player sees within 6 GU,
      at the surge cadence. Each surge is the sum of each relaying Battery's own, and discharges each.
- **Reused:** the FullyConnected phases, U1's turrets, T49's picker.
- **New:** `battery.rs`, `relay.rs`.

### T98. Network services and the buff kinds (SN, SB, SA)
- **Recommend:**
  - **One services step** in the network phase, per player:
    - the held units are served first come first served, by the tick they entered the network (produced units from
      their production);
    - each Vault gives 2 slots, each Null Chamber 4;
    - `Held::service: Option<Service { kind, progress }>`, hashed while set.
  - **The Vault** refills tanks at 5/s, and re-arms a Specter in 160 frames.
  - **The Null Chamber** charges a unit in 2 s. `Held::charged` marks it, and it keeps the charge while inside (D48).
  - **The buff:** T75's `Buff::ability` widens to a `BuffKind`, the Shrine's two plus `NullChamber`. It is given on
    exit to a charged, manned unit (not `UnitDef::satellite`), for 20 s, and ends abruptly.
    - Its +2 GU of sight is read by the vision eye.
    - Its 20% reduction goes in `combat::pre_armor_multiplier`, the Riot Shield's hook.
- **Reused:** T75's buffs, the held store, the pre-armour hook, the vision eye.
- **New:** `network/services.rs`, and `BuffKind`.

### T99. The Field Engineer's buildings: worker sites, occupants, and a heal aura (GB)
- **Recommend:**
  - **Building.** The Field Engineer gets a `WorkerDef` with `builds: [Bunker, FirstAidTent]` and `shares_builds:
    true` (E's shared builds; D37's placeholder). It gathers nothing. `WorkerDef::crystals` is mandatory now, so GB
    makes it an `Option`. The Recruit's, Agent's and Drone's defs gain `Some(..)`: a mechanical change in three
    files.
  - **Sites:** `StructureDef::site` (E's) for both buildings, on any Buildable ground, with no build-area rule.
  - **The Bunker:**
    - F3's `holds`, with the door on B, admitting GDO infantry;
    - N2's `occupants_fire`, with no stand-in, no range bonus and no `bonus_for`;
    - its sight is its own, so occupants see as the Garrison's do.
  - **The Tent:** `StructureDef::heals: Option<HealDef { radius_gu, hp_per_second, targets }>`.
    - A step after the buffs gives each own or allied infantry unit on the map, with its centre within 3 GU of a
      complete Tent, the highest rate among those Tents.
    - That is 0.125 HP a tick (exact in an f32), up to `max_hp_with`.
- **Reused:** E's builds and sites, F3, N2, and the buffs' step.
- **New:** `heal.rs`, and the optional `crystals`.
- **Alternatives:** moving `builds` out of `WorkerDef` into its own `UnitDef` field. It is cleaner, but it touches
  every builder and E's code.

### T100. Vehicles and aircraft out of production; a bomb with a fixed fall (GF, GH2, SV, SD, SA)
- **Recommend:**
  - **GDO vehicles** leave by the Barracks' `ExitRule::Side`, their spot fitting the oriented box, facing out. GF
    tests it with the 64x48 Rig's box.
  - **Syndicate vehicles** go into the network (`ExitRule::Network`), and out of a Tier 2+ Tunnel's Side A exit path.
    SV tests a 72x48 box at the exit spot.
  - **Aircraft:** GDO aircraft appear on a pad (`ExitRule::Pad`, T88). Syndicate aircraft go out by the air doors
    (T96).
  - **The Specter's bomb:** `UnitDef::bomb_fall: Option<Tick>`, read by `combat/projectiles.rs` for its shooter's
    projectiles. Its flight is then fixed, whatever the distance. This avoids changing `AttackAttrs::projectile_speed`
    in every attack literal.
- **Alternatives:** `projectile: Flight { Speed, Fixed }` in `AttackAttrs`. It is cleaner, but every attack literal
  changes.

### T101. M6 in the ruleset's form; X1 and X2 stay in M5 (every slice)
- **Recommend:**
  - X1 and X2 stay where they are: last among M5's content, and before its playtest. The user chose data files at
    playtest time.
  - Every M6 slice is written in X1's form from the start:
    - numbers stay in the defs (T82);
    - new tunables go in X1's tuning modules (the ruleset's `tuning`), not in `const`s;
    - new structural fields (pairs, carriers, stocks, ammo, batteries, mines, services, abilities) join
      `Ruleset::structural_diff`;
    - new def fields round-trip through X2's export, and its round-trip test covers them;
    - client feel tunables (trail and smoke looks, the pads' offsets, the beam) go in `client/*.json`.
  - m6-common carries this as a checklist of about 10–15 minutes a slice.
- **Alternatives:** X1 and X2 after M6, §F weighs both.

---

## §E Question drafts (D32–D48)

- **Format.** Each draft is in the Director's question format. File them from Q0153.
- **None is blocking.** Each draft's Blocking line names the slices that build its placeholder meanwhile.
- **Filing order.** File in the order of the table's "First needed" column. D42 goes first, since V3 runs in M5.
- **Placeholder marks.** In the code, a slice marks each point `Placeholder (QDnn-slug point N)`. Once the draft is
  filed, the next brief or sweep turns it into `Placeholder (Qxxxx point N)`.
- **Nothing technical is asked.** Smoke-shadow geometry, patch spacing, the pads' layout and the like stay in §D.

| Draft | Slug | Asks | Slices | First needed |
|---|---|---|---|---|
| D32 | `hovercraft` | the HoverCraft movement model: DragMovement, or the numbers given | GK, GD, GH1, SD, SP | W1 |
| D33 | `drone` | the Drone Operator's Barracks key, and the drone's details | GK, GD | W1 |
| D34 | `airfield` | landing, take-off, new aircraft and rearming; the Dragonfly's landing | GH1, GH2, GL | W6 |
| D35 | `mines` | what a mine counts as, what sets it off, and its blast | GM | W5 |
| D36 | `outpost` | the Outpost's footprint, its HP, and Unit Control | GP | W5 |
| D37 | `engineer` | how the Field Engineer builds | GB | W6 |
| D38 | `bunker-tent` | a shielded Peacekeeper in a Bunker; whom the Tent heals | GB | W6 |
| D39 | `scan` | Scan: hidden objects, grouped stations, high ground | GR | W2 |
| D40 | `power` | what low Power slows in the new buildings | GR, GF, GH2 | W2 |
| D41 | `supplies` | what the GDO spends Supplies on; whether to build the Supply Tower now | GS | W10 |
| D42 | `tracked` | the tracked turn rate; turning in place beside infantry; the Transport | V3 (M5), V4, ST1 | M5 |
| D43 | `smoke` | smoke and revealed attackers, Scan, and "inside" | SC | W8 |
| D44 | `sludge` | trails while standing still or reversing; "in" sludge; toggles in the network | SB, SC | W4 |
| D45 | `transport` | who boards, what passengers do, Tunnels while burrowed | ST1, ST2 | W7 |
| D46 | `air-doors` | air units at the doors; the Specter's vertical landing | SD, SA | W6 |
| D47 | `prism` | the Prism's panel, its thrust input, and orders while it launches | SP, SR | W8 |
| D48 | `network` | the order of service in the network; the battery inside; which unit is ejected | SN, SB, SL, SA | W3 |

**The three most important:**
- **D34.** Landing is the most code built on a placeholder: GH2's `landing.rs`, which GL and SA reuse. A Glider
  landing on a slot has no precedent in the design.
- **D32.** It sets how the game's first five HoverCraft handle, and GK needs it in W1.
- **D43.** Smoke is the game's first sight blocker. Its rules for reveals and Scan fix how vision treats it, and vision
  is the most perf-sensitive system.

D42 is the most urgent, since V3 runs in M5.

### D32 `hovercraft`

```markdown
# QNNNN — HoverCraft movement: DragMovement, or the numbers the units give?

- **Date**: {at filing}
- **Blocking**: no — GK, GD, GH1, SD and SP proceed on (a): DragMovement, with each unit's numbers mapped.
- **Docs**: `systems/units.md > HoverCraft`, `> DragMovement`; `factions/gdo/units/{hornet,drone_operator,supply_chopper}.md > Placeholder stats`; `factions/syndicate/units/{phantom,prism}.md > Placeholder stats`

## Question
Which movement model, with which numbers, do the Hornet, the Drone Operator's drone, the Phantom and the Prism use?

## Context
- The HoverCraft base moves by DragMovement. It "Relies on high drag to decelerate passively", and "MaxSpeed = (OmniDirectionalAcceleration + ForwardAcceleration) / DragRatio".
- The Supply Chopper has DragMovement numbers: ForwardAcceleration 0.9, OmniDirectionalAcceleration 0.1, DragRatio 0.1, TurnRate 10°/frame.
- The others are given TurnRateMovement's numbers instead:
  - the Hornet: "MaxSpeed 9 su/frame; Acceleration 0.6; Deceleration 0.6; TurnRate 8°/frame";
  - the Phantom: "MaxSpeed 9 su/frame; Acceleration 0.8; Deceleration 0.8; TurnRate 8°/frame";
  - the drone: only "MaxSpeed 10 su/frame";
  - the Prism: only "MaxSpeed 1 su/frame".
- These are the game's first HoverCraft, so the answer sets how GDO and Syndicate air units handle.

## Options
- **(a) DragMovement, the numbers mapped** (the placeholder):
  - OmniDirectionalAcceleration = the given Acceleration, ForwardAcceleration 0, DragRatio = Acceleration ÷ MaxSpeed, TurnRate as given;
  - the top speed is then the given one, and drag alone slows the unit at the given Deceleration from top speed, then more gently;
  - from 9 su/frame a Hornet drifts about 2 grid units before it stops;
  - the drone and the Prism take the Hornet's Acceleration and TurnRate.
  Slidey, as DragMovement intends.
- **(b) TurnRateMovement for these units,** with the numbers as given. They start, stop and turn as crisply as infantry do. The HoverCraft base would list TurnRateMovement, or these units would be exceptions to it. The drone and the Prism would still need an Acceleration and a Deceleration.
- **(c) DragMovement numbers for each unit,** given as for the Supply Chopper.

All three are data on code that exists: M3's DragMovement, the infantry's TurnRateMovement, and the air layer.
```

### D33 `drone`

```markdown
# QNNNN — The Drone Operator: its Barracks key, and the drone's details

- **Date**: {at filing}
- **Blocking**: no — GK and GD proceed on the placeholders in Options.
- **Docs**: `factions/gdo/units/drone_operator.md` (`> Drone Stock`, `> Launching and piloting`, `> Drone limits — how a drone ends`, `> Placeholder stats`); `factions/gdo/structures/barracks.md > Commands`

## Question
1. Which Barracks key trains the Drone Operator? Q, W and E are taken (Peacekeeper, Rocketman, Field Engineer).
2. What are the drone's UnitBase, Unit Control cost, armour and Groupable?
3. Can the operator buy its next drone (F) while its drone is out?
4. Does launching select the drone?
5. Is the 8-grid-unit leash measured centre to centre, or edge to edge as ranges are?

## Context
- The drone is "Air unit 24x24; MaxHP 30; MaxSpeed 10 su/frame; SightRange 5; leash range 8 grid units; battery 20 s".
- The stock "Starts with 1 drone when trained; holds a maximum of 1".
- The drone is "a separate flying unit controlled directly by the player". The operator "cannot move while it has a drone active".
- The Barracks' panel lists only Q, W and E.

## Options
1. **R** (the placeholder), or another free key.
2. **A HoverCraft** (the HoverCraft question's numbers at MaxSpeed 10), Unit Control 0, PointArmor 0, FullArmor 0, Groupable (the placeholder: it is spent like ammunition). Or Unit Control 1 while it flies.
3. **Yes** (the placeholder): the stock empties at launch, so the next drone can be ready when this one ends. Or **no**: buying starts only once the drone has ended, which stretches the cadence by its flight.
4. **No** (the placeholder): the selection doesn't change, and the player clicks the drone. Or **yes**: the drone replaces the operator in the selection, ready to fly.
5. **Centre to centre** (the placeholder), or edge to edge.
```

### D34 `airfield`

```markdown
# QNNNN — GDO aircraft at the Airfield: landing, take-off, new aircraft, and the Dragonfly

- **Date**: {at filing}
- **Blocking**: no — GH1, GH2 and GL proceed on the placeholders in Options.
- **Docs**: `factions/gdo/structures/airfield.md` (`> Assignment`, `> Landing slots`, `> Returning and rearming`, `> Placeholder stats`); `factions/gdo/units/{hornet,dragonfly}.md`; `factions/syndicate/units/specter.md`; `systems/units.md > Glider`

## Question
1. During the 16-frame landing and take-off, is the aircraft an air target or a ground target?
2. Where does a newly built aircraft appear: landed on a free slot, or in the air? With no rally point, does it stay landed?
3. When its Airfield is destroyed, what happens to an aircraft landed on it?
4. Given an order while it rearms, does it take off with the rounds rearmed so far, or finish rearming first?
5. How does a Dragonfly, a Glider that "must keep moving", land on a slot and take off?

## Context
- "Landed aircraft are ground targets ... Once it takes off it is an air target again".
- "Landing and take-off: 16 frames (1 s) each"; "After rearming, the aircraft stays landed".
- "Airfield destroyed: its aircraft become unassigned".
- The Specter, the other Glider that lands, does so "vertically", and "can be hit during both, as an air unit".
- Gliders "Cannot turn in place or stop".

## Options
1. **An air target during both** (the placeholder, as the Specter is), or a ground target from the start of the landing to the end of the take-off.
2. **Landed on a free slot, then it takes off for its rally point; with none it stays landed** (the placeholder). Or it appears in the air over the Airfield.
3. **It stays landed where the Airfield was, unassigned** (the placeholder), and is a ground target until ordered off. Or it takes off at once.
4. **It takes off with what it has** (the placeholder). Or it finishes rearming first, and the order waits.
5. **It approaches its slot straight at IdleSpeed, stops over it while landing vertically in the 16 frames, and takes off vertically, leaving at IdleSpeed** (the placeholder, as the Specter at an air door). Or it lands like a plane, rolling to a stop, which needs a runway longer than a slot.
```

### D35 `mines`

```markdown
# QNNNN — Twister mines: what they count as, what sets them off, and their blasts

- **Date**: {at filing}
- **Blocking**: no — GM proceeds on the placeholders in Options.
- **Docs**: `factions/gdo/units/twister.md > Mines`, `> Placeholder stats`; `core/game_rules.md > Defeat Condition`; `systems/units.md > UnitBase`

## Question
1. Is a mine a building for the Defeat Condition, and can its owner select it?
2. Which enemy units set a mine off? Do HoverVehicles (the Colonists' Drone) and Mechs?
3. Does a mine destroyed by damage, a neighbouring mine's blast included, explode?
4. Can Buy Mine be pressed again while a purchase runs, queueing a second?

## Context
- "When an enemy unit — **infantry or vehicle** — enters it, the mine explodes after a short delay".
- "Air units neither trigger mines nor take damage from them".
- A mine is "a 16x16 su object, hidden underground", with "mine HP 40".
- A burrowed Transport passes through mines (`factions/syndicate/units/subterranean_transport.md > Underground movement`).
- A player is defeated when they own no buildings.

## Options
1. **Neither** (the placeholder): a mine is not a building, and isn't selectable. Or it is selectable, to see it or to remove it.
2. **Every enemy ground unit,** HoverVehicles and Mechs included (the placeholder). Or HoverVehicles pass over safely.
3. **No** (the placeholder): it is removed without a blast, so one blast doesn't clear a field by chain reaction. Or it explodes.
4. **Refused** (the placeholder), as "one at a time" reads. Or purchases queue up to the free stock.
```

### D36 `outpost`

```markdown
# QNNNN — The Outpost: where it stands, its HP, and Unit Control

- **Date**: {at filing}
- **Blocking**: no — GP proceeds on the placeholders in Options.
- **Docs**: `factions/gdo/units/prospector.md > Deploy into Outpost (ability)`, `> Placeholder stats`; `factions/gdo/structures/outpost.md`; `core/entities.md > Placement Validation > Worker-Built Structures`; change 0110

## Question
1. Is the Outpost's 3x3 footprint centred on the point the player clicks?
2. Does the damage the Prospector has taken carry into the Outpost?
3. Is the Prospector's Unit Control (2) freed when it becomes the Outpost?

## Context
- The order "can target a location, the Prospector drives there, and placement is validated on arrival".
- The Outpost is "Size 3x3 (the Prospector deploys in place)", with MaxHP 600 against the Prospector's 300.
- For the Beacon and Warp In, 0110 carries damage through unchanged.

## Options
1. **Centred on the click,** snapped to the tile grid (the placeholder). Or centred on wherever the Prospector stops.
2. **Unchanged, as 0110** (the placeholder): a Prospector at 200/300 becomes an Outpost at 500/600. Or the same fraction (400/600), or full HP.
3. **Freed** (the placeholder): structures don't count toward Unit Control. Or kept.
```

### D37 `engineer`

```markdown
# QNNNN — How the Field Engineer builds the Bunker and the First Aid Tent

- **Date**: {at filing}
- **Blocking**: no — GB proceeds on (a).
- **Docs**: `factions/gdo/units/field_engineer.md`; `factions/gdo/structures/{bunker,first_aid_tent}.md`; `factions/cults/units/recruit.md > Building`; `core/entities.md > Placement Validation > Worker-Built Structures`

## Question
How does a Field Engineer construct? Must it keep working on the site, can others assist, and is it used up?

## Context
- "Build Bunker: constructs a Bunker ... Costs Space Crystals".
- The Bunker "Follows the Worker-Built Structures rules", and has "X: Cancel construction while being built".
- The GDO's other buildings are placed complete by the Deployment Center, so these are the GDO's first buildings built on the map.
- Repairers' rates add up "as construction does".

## Options
- **(a) As the Cults' Recruits do** (the placeholder):
  - it builds "from outside the site", and others can assist, so n Engineers build n times as fast;
  - it isn't used up;
  - the site progresses only while someone works on it;
  - the cancel refunds as for the Cults' buildings.
- **(b) One Engineer only,** with no assistance.
- **(c) As the Colonists' Drone deploys:** the Engineer is used up into the building.
```

### D38 `bunker-tent`

```markdown
# QNNNN — A shielded Peacekeeper in a Bunker, and whom the First Aid Tent heals

- **Date**: {at filing}
- **Blocking**: no — GB proceeds on the placeholders in Options.
- **Docs**: `factions/gdo/structures/bunker.md > Garrison`; `factions/gdo/structures/first_aid_tent.md > Healing`; `factions/gdo/units/peacekeeper.md > Riot Shield (toggled ability)`

## Question
1. A Peacekeeper enters a Bunker with its Riot Shield raised. Does it lower the shield, or stay shielded (and so not fire)?
2. Does the First Aid Tent heal infantry inside a Bunker or a Garrison within its 3 grid units?
3. Does a First Aid Tent heal while it is still under construction?

## Context
- "No abilities from inside: ... a Peacekeeper cannot raise its Riot Shield".
- "Units inside fire their own weapons", but a shielded Peacekeeper can't attack.
- The Tent heals infantry "whose centre is within 3 grid units".

## Options
1. **It stays shielded, and doesn't fire** (the placeholder: nothing changes its mode for it), until ejected. Or entering lowers the shield, so it fires.
2. **No** (the placeholder): only units on the map are healed. Or yes.
3. **No: only from completion** (the placeholder). Or yes.
```

### D39 `scan`

```markdown
# QNNNN — Scan: hidden objects, grouped stations, and high ground

- **Date**: {at filing}
- **Blocking**: no — GR proceeds on the placeholders in Options.
- **Docs**: `factions/gdo/structures/radar_station.md`; `systems/controls/selection.md > No smart-casting`, `> SelectionGroup`; `core/vision.md > ElevationModifier`

## Question
1. Does Scan show hidden objects: mines, burrowed units, underground expansions?
2. With several Radar Stations selected, does one press scan from all of them?
3. Does high ground limit a Scan, as the ElevationModifier limits sight?

## Context
- Scan "reveals a circle of radius 6 grid units for 8 seconds ... Each Radar Station has its own cooldown". The Radar Station is "Groupable true".
- "The game never hands a group's command to just one member" (No smart-casting).
- Caster-type units are Ungroupable: "one cast should spend one unit's energy, not the whole group's". The Shrine is Ungroupable (Q0118).
- Detection isn't designed (`core/vision.md`).

## Options
1. **No** (the placeholder): Scan is sight, not detection. Or Scan detects, as the game's first detector.
2. **Yes, all of them,** as the rules stand (the placeholder): two grouped stations scan the same point and both cool down. Or the Radar Station becomes Ungroupable, as the Shrine is, and the player Tabs between stations.
3. **No** (the placeholder): the full 6 grid units everywhere, as air units' sight. Or yes, by the ground at the circle's centre.
```

### D40 `power`

```markdown
# QNNNN — What low Power slows in the new GDO buildings

- **Date**: {at filing}
- **Blocking**: no — GR, GF and GH2 proceed on (a).
- **Docs**: `factions/gdo/overview.md > Power`; `factions/gdo/structures/{radar_station,defense_system,missile_turret,airfield}.md`

## Question
At negative Power, which of the new buildings' activities slow down?
- the Defense System's turret construction;
- the Factory's and the Airfield's production;
- the Radar Station's Scan cooldown;
- the Missile Turret's rate of fire;
- the Airfield's rearming.

## Context
- "If total Power is negative, all power-consuming buildings and Extraction Plates operate slower in proportion ... (affecting construction speed, unit production speed, and mining rate)".
- "Applies to every building with negative Power and to Extraction Plates". "Power producers are never slowed".
- The list was written before any building had a cooldown, a weapon or rearming.

## Options
- **(a) Only the three listed** (the placeholder). The Defense System's construction and the Factory's and Airfield's production slow. Scan's cooldown, the turret's fire and rearming don't.
- **(b) Everything a consuming building does,** cooldowns, rate of fire and rearming included. Low Power then weakens defences too.
- **(c) A list per activity.**
```

### D41 `supplies`

```markdown
# QNNNN — What the GDO spends Supplies on, and whether to build the Supply Tower now

- **Date**: {at filing}
- **Blocking**: no — proceeding on (b): GS waits for this answer, and the Supply Tower stays greyed "Not in the build yet". Nothing else waits.
- **Docs**: `factions/gdo/overview.md > Supplies`; `factions/gdo/economy.md > SupplyTower`; `factions/gdo/units/supply_chopper.md`; change 0077

## Question
1. What does the GDO spend Supplies on?
2. Until that's designed, should the Supply Tower and the Supply Chopper's supply runs be built?
3. If they are, how fast does a Supply Chopper repair while landed on its Tower?

## Context
- Supplies are "Spent on advanced buildings, units, and research", but no GDO cost in 0366 uses them.
- 0077 deferred the GDO's supply loop, with what it spends Supplies on still missing.
- Everything else about the Tower and the Chopper's supply runs is designed and has numbers. The Chopper's transport is TBD, and stays out of this milestone.
- A landed Chopper "is gradually repaired at no cost", with no rate given.

## Options
1. For the design.
2. **(a)** Build them now: Supplies are gathered and shown, but buy nothing yet, and the flow can be playtested. **(b)** Wait until GDO Supply costs are designed (the placeholder): GS moves to that milestone. **(c)** Build them now, and give some of 0366's buildings or units a Supply cost.
3. With (a) or (c): **the repairers' rate, free** (the placeholder), so 0 to full HP in half its build time. Or another rate.
```

### D42 `tracked`

```markdown
# QNNNN — Tracked vehicles: the turn rate, turning in place beside infantry, and the Transport

- **Date**: {at filing}
- **Blocking**: no — V3 (in M5), V4 and ST1 proceed on the placeholders in Options.
- **Docs**: `systems/units.md > SpeedTurnRadiusMovement`, `> UnitCollision > Crushing`, `> DrillUnit`; `factions/syndicate/units/{subterranean_transport,laser_battery}.md > Placeholder stats`

## Question
1. Is a tracked unit's turning rate meant to be the same at every speed?
2. Can a crusher turning in place crush an adjacent crushable enemy, or is it blocked by it? (The doc's TBD.)
3. What are the Subterranean Transport's above-ground Acceleration, Deceleration and SpeedToTurnRadiusRatio?
4. Does the surfaced Transport crush?

## Context
- SpeedTurnRadiusMovement: "Turn radius increases with speed ... sharp turns require significant slowing or a complete stop".
- With turn radius = speed × SpeedToTurnRadiusRatio, the turning rate is speed ÷ radius = 1 ÷ ratio, whatever the speed. At ratio 8 that is about 7.2° a frame, at full speed and spinning in place alike.
- So slowing makes a turn tighter, not quicker. A Laser Battery at 4.5 su/frame turns 90° in about 13 frames, on a 36 su radius.
- Crushing: "TBD: whether a crusher turning in place (Stationary + Turning) can crush, or is blocked by, an adjacent crushable enemy".
- The Transport is given only "MaxSpeed 4.5 su/frame" above ground. Its UnitBase is DrillUnit, which Crushing doesn't list ("TrackedVehicle, Mech"), though above ground it "behaves as tracked".

## Options
1. **(a) Yes, as the law gives** (the placeholder). **(b)** A separate, slower turning rate when spinning in place. **(c)** A turning rate that falls with speed, so slowing does turn faster.
2. **It neither crushes nor pushes: the spin stops at contact** (the placeholder), as a stationary crusher is a hard obstacle. Or it crushes what its turning shape sweeps.
3. **The Laser Battery's: 0.4, 1 and 8** (the placeholder), or numbers of its own.
4. **No** (the placeholder), by the base list. Or yes, as a tracked vehicle above ground.
```

### D43 `smoke`

```markdown
# QNNNN — Smoke: revealed attackers, Scan, and when a unit is inside

- **Date**: {at filing}
- **Blocking**: no — SC proceeds on the placeholders in Options.
- **Docs**: `factions/syndicate/units/shadebringer.md > Cover (toggle)`, `> Placeholder stats`; `core/vision.md > Attacks from outside vision`; `factions/gdo/structures/radar_station.md`; `systems/units.md > UnitBase` ("Which tile counts")

## Question
1. An attacker in or behind smoke damages your unit. Is it revealed to you, as an attacker in the fog is?
2. Does smoke block a Radar Station's Scan?
3. When is a unit inside the smoke, and so blind and unseen: when its centre is in it, or any part of it?

## Context
- The smoke "blocks line of sight into it and behind it for every unit except satellites".
- "Units inside the smoke can't see anything"; "Structures too".
- An attacker outside your vision "becomes visible to you for 1.5 seconds ... after each shot".
- Scan comes from neither a unit nor a structure.
- A unit is "on" Rugged terrain when the tile under its centre is.

## Options
1. **Yes** (the placeholder): the reveal isn't sight, so smoke doesn't stop return fire. Or no: smoke hides attackers, and the reveal waits until they leave it.
2. **Yes** (the placeholder): only satellites see through. Or no: a Scan sees from above, as a satellite does.
3. **Its centre** (the placeholder, as for Rugged terrain), or any part of it.
```

### D44 `sludge`

```markdown
# QNNNN — Shadebringer trails: standing still, reversing, being "in" sludge, and the network

- **Date**: {at filing}
- **Blocking**: no — SB and SC proceed on the placeholders in Options.
- **Docs**: `factions/syndicate/units/shadebringer.md > Sludge (toggle)`, `> Charges and recharging`, `> Placeholder stats`

## Question
1. Does a Shadebringer standing still with a toggle on lay sludge or smoke where it stands?
2. When it reverses, is its trail laid behind its motion (at its front), or still at its rear?
3. When is a unit "in" sludge: when its centre is, or any part of it?
4. Does entering the Tunnel Network switch its toggles off?

## Context
- It "leaves a trail of sludge on the ground in its wake"; "the sludge is spread behind it, so laying the trail doesn't slow it".
- Each tank "drains while its toggle is on". "Toggles follow the mode-switch rule (keep orders, not queueable)".
- Tanks refill only inside the network, and only with a Vault.

## Options
1. **No: it lays only while moving, though the tank still drains** (the placeholder). Or it lays a patch where it stands: a static puddle or screen.
2. **Behind its motion** (the placeholder), so laying never slows it. Or always at its rear, so reversing drives it into its own trail.
3. **Its centre** (the placeholder, as for Rugged terrain), or any part of it.
4. **Yes, both** (the placeholder), so it comes out with both off. Or they stay on, and drain again as it comes out.
```

### D45 `transport`

```markdown
# QNNNN — The Subterranean Transport: who boards, what passengers do, and Tunnels

- **Date**: {at filing}
- **Blocking**: no — ST1 and ST2 proceed on the placeholders in Options.
- **Docs**: `factions/syndicate/units/subterranean_transport.md`; `factions/syndicate/tunnels.md > Regeneration inside`

## Question
1. Can allied infantry board, or only the owner's?
2. Do passengers fire out of it?
3. While a loaded Transport is in the network, do its passengers count as outside it, as the doc assumes, for regeneration and for the Null Chamber?
4. Can a burrowed Transport enter a Tunnel from underground?

## Context
- "It carries infantry only".
- "Loading: right-click the transport with infantry selected (Enter); it must be above ground".
- "Inside the network the passengers are still locked inside the transport".
- "Units carried inside a Subterranean Transport don't count as being in the network themselves (assumed; the transport itself regenerates)".
- Units enter the network through a Tunnel's Side A, on the surface.

## Options
1. **The owner's only** (the placeholder), or allies' too.
2. **No** (the placeholder), or yes, as from a Bunker.
3. **Yes, outside** (the placeholder): no regeneration, no Null Chamber turn, and no buff when the Transport leaves. Or they count as inside, and come out buffed.
4. **No: it surfaces, and drives in** (the placeholder). Or it enters from below, staying hidden throughout.
```

### D46 `air-doors`

```markdown
# QNNNN — Air units at a Tier 3 Tunnel's doors, and the Specter's vertical landing

- **Date**: {at filing}
- **Blocking**: no — SD and SA proceed on the placeholders in Options.
- **Docs**: `factions/syndicate/tunnels.md > Look by tier`; `factions/syndicate/tunnel_menus.md > EjectMenu`; `factions/syndicate/units/{phantom,specter}.md`

## Question
1. When is an air unit taken in at the entry door: once it is over the door?
2. Does a Phantom coming out appear over the exit door at once, or rise out of it over some frames (and can it be hit meanwhile)?
3. The Specter is a Glider that "lifts off and lands vertically". Does it stop over the door to land? At what speed does it leave after lifting off?

## Context
- The doors are on the roof, "one for flying units entering and one for flying units exiting".
- Their rate is "one aircraft per 8 frames (0.5 s), placeholder (the same rate for the air entry door)".
- The Specter's "Lift-off and landing: vertical, 24 frames each at the air door (it can be hit during both, as an air unit)".
- The Prism has its own 32-frame launch. The Phantom has no such numbers.

## Options
1. **Once its centre is over the door** (the placeholder), or after a short descent like the Specter's.
2. **At once, at flying height** (the placeholder), or rising for some frames, open to fire.
3. **It flies over the door at IdleSpeed, stops over it, and descends for the 24 frames. It lifts off for 24 frames, and leaves at IdleSpeed** (the placeholder). Or it dives in without stopping.
```

### D47 `prism`

```markdown
# QNNNN — The Prism: its panel, aiming a thrust, and orders while it launches

- **Date**: {at filing}
- **Blocking**: no — SP and SR proceed on the placeholders in Options.
- **Docs**: `factions/syndicate/units/prism.md`; `factions/syndicate/units/laser_battery.md > Relay`

## Question
1. What is the Prism's panel, armed and not?
2. How is a thrust aimed, and can thrusts be queued?
3. Are orders given while it launches and unfolds held until it has unfolded?

## Context
- "Directional thrust: a few charges, each giving a quick burst of displacement in a chosen direction".
- "Thrust charges: 3; each moves it 3 grid units over 1 s in a chosen direction".
- "Launch 32 frames (straight up) and transform 32 frames (vulnerable during both)".
- The armed Prism can be given "a specific target (Attack)".
- The doc gives no panel.

## Options
1. **Q Move, X Stop, R Thrust; and A Attack while armed** (the placeholder).
2. **Targeted** (the placeholder): the player clicks a point, and the thrust goes toward it, 3 grid units whatever the distance. Not queueable. Or queueable, so thrusts chain.
3. **Held,** as in the vehicle deploy model (the placeholder). Or refused until it has unfolded.
```

### D48 `network`

```markdown
# QNNNN — Inside the network: the order of service, the Laser Battery, and which unit is ejected

- **Date**: {at filing}
- **Blocking**: no — SN, SB, SL and SA proceed on the placeholders in Options.
- **Docs**: `factions/syndicate/structures/{vault,null_chamber}.md`; `factions/syndicate/units/laser_battery.md > Battery`; `factions/syndicate/tunnel_menus.md > EjectMenu`

## Question
1. Does "first come first served" count from when the unit entered the network, or from when it began to need the service?
2. Does a unit that has finished its Null Chamber turn keep it, however long it stays inside?
3. Does a Laser Battery's battery charge inside the network?
4. Can one unit hold a Vault slot and a Null Chamber turn at the same time?
5. Should the EjectMenu prefer units whose service is done?

## Context
- "each Vault re-arms or recharges 2 units at a time; others wait their turn (network-wide, first come first served)".
- "each Null Chamber buffs 4 units at a time, each buff taking 2 s inside before the unit can leave buffed".
- The battery is "always charging at its charge rate, attacking or not".
- The EjectMenu ejects "the one that has been inside the network longest. (Not "the healthiest": other per-unit factors such as re-arming at a Vault or the Null Chamber's re-buff may matter too.)"

## Options
1. **From entering the network** (the placeholder), a produced unit from its production. Or from when it began waiting.
2. **Yes** (the placeholder). Or it must leave within some time.
3. **Yes,** as "always charging" reads (the placeholder). Or only on the map. Or only with a Vault, taking one of its slots.
4. **Yes** (the placeholder): they are separate queues. Or one at a time.
5. **No: the one inside longest, as now** (the placeholder). Or units whose service is done come first.
```

---

## §F Size and risks

**Size**

| Group | Slices | Agent hours |
|---|---|---|
| Engine | V3 (in M5), V4 | 5 |
| Kinds | GK, SK | 5.5 |
| GDO | GR, GD, GF, GA, GM, GP, GB, GH1, GH2, GL, GS | 31 |
| Syndicate | SV, SN, SB, SL, SD, ST1, SC, SP, SA, ST2, SR | 30 |
| Tail | S6a, S6b, MQ | 7 |
| **Total** | **29** | **78.5** |
| Optional | MQa (with the partway playtest) | +2 |

- **M6 proper,** without V3: 28 slices and 75.5 h (77.5 h with MQa). M5 grows by V3's 3 h.
- **Wall-clock time:** 11 waves after X1, three builds a wave to W10. At about 3.5 h a wave with review, that is
  roughly 40 h.
- **The Director:** 28 reviews and merges, about two waves of answers to fold in, and two playtest write-ups (three
  with MQa).

**What is likely to change**
- **The numbers.** Every 0366 and 0367 number says "tune in the numbers pass". After X1 and X2 they are data, so a
  change is a data edit, not a slice.
- **The answers.** Most drafts swap one placeholder rule for another inside one slice, which S6a and S6b absorb. Four
  could move more work:
  - D34 (landing) reshapes `landing.rs`, which GH2, GL and SA share;
  - D32's option (b) changes the movement model of five units (data, plus the follower's model choice);
  - D41's option (b) takes GS out of M6;
  - D43 changes SC's occlusion rules.
- **New design during M6,** none of it in M6's scope:
  - detection (0267: after every faction's units are done), which would reach mines, burrowed units, underground
    expansions, the Null Chamber's buff and Scan;
  - the Supply Chopper's transport;
  - the techs pass (Syndicate research);
  - the GDO's Supply costs.

  Each would be a slice after M6, or a fold-in during a sweep.

**Risks**

| Risk | Slices | Mitigation |
|---|---|---|
| The tracked corner choice won't settle, holding up N10 | V3 (M5) | the analytic law (T85); the Fable fallback; N10 is the only slice waiting |
| Crushing breaks collision invariants | V4 | one bounded exception in the no-overlap invariant; the Fable fallback |
| Glider landings at a slot or a door won't settle | GL, SA | vertical landings (D34's and D46's placeholders) keep gliders out of tight approaches; the Fable fallback |
| Surfacing and its reservation | ST2 | the reservation is an obstacle in `movement::space`; the Fable fallback |
| Smoke slows vision | SC | per-eye smoke revisions (T93); `smoke_perf` within 20% of `vision_perf` |
| Def literals churn across about 40 files | all | T84's struct-update defaults; failing those, T82's rule, about one wave longer |
| Appending kinds moves a hash | GK, SK | append only; if a hash moves, GK and SK run back to back |
| X1 runs long | W1 | nothing in M6 starts before X1 merges; the waves shift, unchanged |
| A slice changes the computer players' games | all | the AI-games gate in every slice; their games use M5 units only |
| `Object` and `UnitDef` keep growing | GD, GA, SB, SL, GH1, ST1, SP, SA | one state struct per mechanism (`Object::aircraft`, `Object::stock`), hashed only while set (T65) |
| Answers arrive after their slices | S6a, S6b | two sweeps; later answers go to MQ's notes |
| The partway playtest lacks GDO aircraft | MQa | named in `docs/playtest/m6a.md`; the GDO has the Twister, the Missile Turret and K's Rocketman against air |

**X1 and X2: keep them in M5, or move them after M6?**

The rule: "X1 stays last among content, alone: it converts every tunable once."

- **(a) Keep them in M5, and write M6 in X1's form (T101):**
  - the M5 playtest gets the data files the user chose for playtests;
  - X1 converts only M5's tunables, once, so it is smaller;
  - M6's slices write each new tunable once, in its final form, for about 10–15 minutes a slice (T101's checklist);
  - the cost is that M6 can't start until X1 has merged, and X1 runs alone. M5's plan already has that wave.
- **(b) Move them after M6:**
  - M6 starts about one wave sooner, right after N10 and N11;
  - but the M5 playtest has no data files, against the user's choice;
  - X1 then converts M6's 38 kinds as well, in one slice that runs alone for longer at M6's end;
  - and every M6 tunable is written twice: as a `const`, then converted.
- **Recommendation: (a).** Its only cost is a wave's wait, and it buys data files at both playtests and a single
  conversion. It also keeps "converts every tunable once" true, since nothing written after X1 needs converting.

**A partway playtest (the first useful subset)**
- **What it holds:** the ground war, playable after W6.
  - **GDO:** Scan, the Defense System and Missile Turrets, the Drone Operator, the Factory with Raptors and Twisters
    (mines included), the Artillery Rig, the Prospector and Outposts, the Bunker and the Tent.
  - **Syndicate:** the Vehicle Assembly with Bikes, the Null Chamber, the Vault and the Shadebringer's Sludge, the
    Laser Battery without its Relay, the Forge, and the Aerospace Assembly with the Phantom.
  - **Engine:** crushing.
- **Not in it:** the GDO's aircraft and Supply Tower; smoke; the Transport; the Specter, the Prism and the Relay.
- **Cost:** MQa (2 h, in W7) and the user's playtest time. M6 still ends at W11.
- **Benefit:** the ground units' placeholder numbers get played four waves before M6 ends, while S6a and S6b can
  still absorb what the playtest finds.
- **Without it,** W7's third slot takes SP early, which only adds slack.
