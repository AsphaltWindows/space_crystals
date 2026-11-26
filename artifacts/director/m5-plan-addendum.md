> **Adopted by the Director (2026-11-25)** as M5's plan for 0319's content. The draft follows unchanged, with these
> amendments:
> - **Questions filed:** D18–D31 are **Q0112–Q0125**, in order (D18 = Q0112 … D31 = Q0125). None blocks.
>   - Q0124 (D30) drops its point on the Agent's key: 0326 puts Repair on D.
>   - 0325 settled 0322's open point: a right-click on a damaged building the selection can enter is Enter.
> - **T69–T83 accepted** as written.
> - **0323–0326 arrived after the draft:**
>   - **0323 and 0324** (Q0111's step 1, the 90° body-width contact step): slice **M4-W3** (Fable) runs now, outside
>     the waves, in the slot C left. It splits `movement/collide.rs` first, so V1 builds on that split. The goldens
>     re-bless with it.
>   - **0325:** N8 keeps Enter for a right-click on a damaged building the selection can enter, and adds the Agent's
>     right-click Repair on a damaged Laser Turret.
>   - **0326:** the Agent's panel is Q Move, A Construct (Q Tunnel, W Laser Turret, Z Back), S Drop Off, D Repair,
>     X Stop. **J1** builds the Agent's Construct menu beside the Recruit's, with D `Planned`. **N8** makes the Agent's
>     **D** live, not F as §B's N8 says.
> - **0327 and 0328 arrived next:**
>   - **0327** (answers Q0096): the Recruit has two build menus, **A** Basic and **W** Advanced, with fixed slots by
>     branch. **J1** builds both, with every building not yet built greyed. This replaces the single
>     `CONSTRUCT_MENU` of the J1 note in §C.
>   - **0328** (answers Q0123, Q0125, and points of Q0112, Q0120 and Q0124):
>     - **New slice N9: the Field Engineer and the Career Campus** (Opus, about 2 h; after N8, before X1). It covers
>       their defs, the Barracks' E recipe, the Deployment Center's Construct entry, the Field Engineer's panel
>       (A Construct with Bunker and First Aid Tent greyed; D Repair), and its `repairs` data on N8's mechanism.
>     - Shroud is on **R** (N4a and N4b).
>     - The Sparrow's SeparationRadius of 32 su is now the design's (H1).
>     - **N8:** a Tunnel repairs at its 480-frame rate at every Tier, and a vehicle's repair price is crystals only.
> - **0329 and 0330** (answer Q0124, and points of Q0114 and Q0121):
>   - **N8:** a repair ends at full HP, or when the bank is empty (replacing T83's wait at the step's top). Gliders
>     aren't repaired, and Agents don't repair Tunnel expansions. Instead **N8 builds the expansions' regeneration:**
>     1% of MaxHP a second while their Tunnel lives, in combat too.
>   - **N1a (0330, refined by 0331):** when MaxHP goes up, HP goes up by the same amount, so the damage is kept; when it
>     goes down, HP is capped at the new maximum, with no floor. This replaces T73's floor at 1 HP.
>   - **N6 and N7:** products come out at full HP, as T71 planned.
> - **0332–0334:**
>   - **0332:** every pool building reserves in order of entry, and trained infantry keep the Recruit's damage (T71's
>     placeholders confirmed, for F2, N1b, N4a, N6 and N7).
>   - **0333:** the Gunner's FiringDuration is 4 and its CooldownDuration 6. F2 makes the change.
>   - **0334, T77 amended for N4b:**
>     - each connected cloud has its own timer, releasing 8 frames after the first hit it absorbs, replacing the
>       global clock;
>     - clouds that join release their buffers at once and start a fresh joint buffer, replacing the summed buffers;
>     - releases ignore directional armour;
>     - underground objects are affected;
>     - the Disciple's regeneration continues while channelling;
>     - every player sees the cloud over their Visible tiles;
>     - an object is under a cloud when its centre is.
>
>     **N4a:** the Disciple is Ungroupable.
> - **0335** (answers Q0115–Q0118):
>   - **N2, T72 amended.** The Garrison has one door, on side B (AAAB). Eject All lets everyone out at once through
>     that door, passing through each other until clear, which replaces `ExitRule::AnySide` with T62's spots.
>     Evacuation is unchanged. Each occupant picks its own target, Range is measured from the Garrison's edge, there
>     are no spells from inside, and augmentation doesn't change the borrowed Militant attack.
>   - **G2:** the Gunworks and Machine Shop become ABAC (in at B, out at C). They still can't be entered until their
>     research exists (0328). The tech gate follows 0335's general rule: losing the Gunworks never cancels Gunners
>     already queued or in training, and W greys again for new orders. Test it.
>   - **N3:** the Temple is ABAC (in at B, out at C), and staff leave one at a time. T72's nearest-side entrance stays
>     only for the Shrine, until its entrance is decided. Level 2 waits for Level 1 to complete, and the Cathedral is
>     checked at queue only (T74, confirmed).
>   - **N5:** a unit is in the circle by its centre; a recast restarts the duration, with no stacking; Shrines are
>     Ungroupable.
>   - **S1:** sweep the factions already built for 0335's general rule. Nothing already started or finished may be
>     cancelled or disabled when a prerequisite is lost. Report what happens to an order given but not yet started,
>     such as a builder still walking to its site.
> - **0336** (answers Q0127):
>   - **Augment research is retroactive, and Level Locking is removed.** AugmentationState becomes two booleans
>     (T73: `Augmentation { constitution: bool, metabolism: bool }`), and the amounts are read from the owner's current
>     Temple levels (T74's techs).
>   - When a level completes, every unit with that augmentation changes at once, with HP by 0331: an increase keeps
>     the damage, and a decrease caps HP.
>   - N1a builds the accessors on the owner's levels (none until N3, so L0 amounts), and N3 applies the HP rule as a
>     level completes.
>   - **N5:** the Shrine is ABAC, in at B and out at C, one at a time. T72's nearest-side entrance is no longer used.
> - **0337** (kinds and Release; `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`):
>   - **F2 builds** the kinds (Plain, Constitution, Metabolism, Both), the two-step panel and the `Release` order. T70's
>     `QueueEntry` enum (`Unit`, `Release`) moves into F2, so N1b adds only `Augment`, and N3 `Research`. T73's
>     `Object::augmentation` (two booleans) also moves into F2, without effects, since the kinds need it; N1a adds the
>     accessors.
>   - **Reservations** take the first in within the order's kind. Eject All skips reserved units.
>   - **N4a, N6 and N7** reuse the kind pick: the Chop Shop and Airstrip pick a kind too, and a Jackal takes 2 Recruits
>     of it. **N1b:** Release in every Altar menu, and the "Fully augmented" tile is enabled with Release only.
>     **N3, G2 and N5:** A/S/D/F each send out one Recruit of a kind at once, with no queue, and V is Eject All.
> - **0338:** every Cults stat upgrade applies to existing units. T67's "locked levels" go, so G2, N3 and the Machine
>   Shop's research keep no "trained after" bookkeeping. The DrillUnit steps sideways (M4-W3 built it).
> - **0339 and 0340** answer Q0097 and Q0098 and are folded into E, which is running. **N8's Repair** reuses E's
>   Recruit-frame count and its closest-part spots.
> - **0341** answers Q0099. F2 takes points 2–5 (capacity rounded down; same-tick ties by the nearer Hall centre).
>   **J1** takes point 1: a Hall whose area would leave the map can't be placed, in the sim, on the ghost, and in the
>   editor's layout check.
> - **0342** answers Q0100: the Hall is AAAB with its exit on B, has no X, counts the Recruit in production toward
>   local usage, and restarts production from 0. F2 makes the change. **J1** rotates the Hall's ghost at placement.
> - **0343** answers Q0129 with (a), as built. **0344** (Q0101): the global figure is display only (F2 drops the
>   marks).
> - **0345 and 0346, for L:**
>   - **0345** (Q0102): the Storage rule is as D built it. L drops the marks and tests the Verification.
>   - **0346** (Q0103): L's oracle compares edge to edge, at ±15%, with one Storage per face for the ideal.
>   - **0346's second scenario** is the user's build line: one Storage; save for a second Hall with no extra Storages,
>     sending the new Recruits to farther patches; build the Hall with every non-optimal Recruit; then a Storage per
>     face. It is measured against the GDO and the Colonists at 1:30 and 2:48.
>   - L reports the second scenario's numbers without a target, and chooses the second Hall's place and the far
>     Recruits' patches.
> - **0347–0349, entering, leaving and evacuation (Q0104–Q0106):**
>   - **F2 takes 0347 point 5:** the capacity counts reserved units, so a pool building's queue never exceeds its
>     capacity. The Armory's separate 20 goes, and T70's per-def limits for the pool buildings with it. F2 also takes
>     the load lost on training (point 3) and 0348 (the Armory Ungroupable; the rally and ready order confirmed).
>   - **New slice F3, holding (Opus, after F2, before N2):** it builds 0347 points 1, 2 and 4 and 0349 once, for every
>     building that holds units:
>     - a dedicated entrance doorway at the middle of the entrance side, replacing F1's whole-side entry;
>     - a full building ends the Enter, and the unit idles beside it;
>     - sites can't be entered;
>     - **evacuation inside the footprint:** everyone appears at once on the former footprint and the overlap rule
>       separates them (`systems/units.md`, units that already overlap); a finished unit comes out trained, with no
>       refund; they idle. This replaces F1's ring search (T62) for evacuation, though the spots stay for
>       everything else.
>   - **N2's Garrison** reuses F3's evacuation. **N1b, N4a, N6 and N7** set Groupable false (0348).
> - **0350–0352 (Q0107), for J1:** the Hall's placement preview shows the area, the claimable, own-claimed and
>   non-Recruitable tiles, and the Effectiveness %. The claims of enemy Halls that are visible or remembered (the
>   player's memory, never the true state) count as unclaimable. Own Halls under construction and placed ghosts are
>   claimants in placement order (display only). A selected own Hall shows its claims.
> - **0353 (Q0119), for V2 and N6:** an idle vehicle doesn't yield; own idle infantry yield to vehicles; vehicles reverse
>   only on W; a point inside the turning circle is reached by a forward loop.
> - **0354 and 0355 (Q0120), for H1 and N7:**
>   - **H1:** no cap on overshooting the map edge; strafing flies on out of range, then loops back at full speed;
>     Stop circles in place, and Hold circles without chasing; the idle circle is a fixed 1 GU at IdleSpeed²/64.
>   - **N7:** takeoff at IdleSpeed, speeding up along the runway; with no rally, the Sparrow circles 2 GU past the
>     C end.
>   - **G2 (or H1):** the Sparrow's MaxCentripetalAcceleration is 0.5 su/frame².
> - **0356–0360, two new slices outside the Cults:**
>   - **M5-K, GDO anti-air (Opus, after H1; Q0122 answered by 0356):**
>     - **The Arsenal** (2x2, HP 350, 1/4, Power −20, 150 SC, 320 f, from the Deployment Center; no panel yet).
>     - **The Rocketman** (Barracks W; LightInfantry; HP 45; armour 1/1; sight 6; speed 3). Its missile is a
>       DoublyDisjointed projectile (M4-O): Damage 20, Range 6, MinRange 2 (0203), 16 su/f, every 32 f, hitting air
>       and ground.
>     - **Target Lock** (R): LockOn 48 f and LockRange 7. A locked missile can't be dodged, its fire ignores
>       MinRange (0204), and it may target structures.
>     - 0356's Verification needs H1's Sparrow: Rocketmen shoot one down; a circling Sparrow sometimes dodges the
>       normal missile, and a locked one never does.
>   - **M5-Y, Syndicate turrets (Opus, after E, which owns `orders/build.rs` and `sim/sites.rs`):**
>     - **0359:** remove the Agent's turret (its order, site, refunds, exit and client), so the Agent's panel is Q Move,
>       A Build Tunnel, S Drop Off, D Repair (Planned until N8), X Stop. The ExpandMenu's turret and the Agent's
>       repair of turrets stay.
>     - **0360:** remove the upright form and conversion. On Tunnel loss a turret joins the expansions' path:
>       neutral, visible to all, high armour, a raised turret retracts at once, it never fires, it decays 1% MaxHP/s,
>       and a new Tunnel covering it reclaims it. Defeat no longer counts turrets.
>     - **0358:** every expansion of a destroyed Tunnel becomes visible at once, and the Q0091 marks go.
>     - Goldens may move if one has an Agent-built or upright turret.
>   - **J1** builds no Agent Construct menu (0326 is superseded). **S1** shows every player the defeat notice
>     (0357), and the playtest notes name the Colonists' anti-air gap (0356).
> - **J1 is split:** the Recruitment Area display (0350–0352, and Q0107's "a selected own Hall shows its claims")
>   becomes **J2** (Opus, after J1, the same placement files). J1 keeps the panel, the menus, placement, the
>   right-click and the pointers, plus 0341 point 1.
> - **The waves after these additions** (a sketch; the binding rule is each slice's "after"):
>   - W4: J1, Y and F3;
>   - W5: L, U1 and G2;
>   - W6: V1, N8 and N1a;
>   - W7: N2, V2 and N1b;
>   - W8: N4a, H1 and N3;
>   - W9: N4b, N6 and N5;
>   - W10: N7, K, N9 and S1;
>   - W11: X1; W12: X2 and S2; W13: M, then the playtest.
>
>   The playtest moves about one wave later, to the end of W13. An infantry-core playtest would come at the end of
>   W10.
> - **The waves (§C) stand,** but the playtest's timing is the user's call: the full Cults at the end of W12, or a
>   first playtest of the infantry core at the end of W9 (§F), put to the user on 2026-11-25.

# M5 plan addendum: every designed Cults building and unit (planning spike M5-P2)

**For the Director.** Design change 0319 gave placeholder numbers to every designed Cults building and unit: "Q0092
scope: every designed Cults building and unit can now be implemented on these placeholders. The order of slices is the
Director's call." This addendum puts all of it into M5. It extends `artifacts/director/m5-plan.md`:
- **§A and §B** replace §3.3's G and the gated N1–N6 with 14 slices;
- **§C** replaces §3.2's waves from W5 on. W1–W4 and the first playable (end of W4) are unchanged;
- **§D** continues §4 from T69;
- **§E** continues §5 from D18;
- **§F** adds to §6.

**What I read:**
- design commit `a7ad7733`: 0319, and five later entries:
  - **0320**, which answers Q0110 with (d): "No movement change for Recruits";
  - **0321**, Repair for Recruits, Agents and the Field Engineer;
  - **0322**, which answers Q0095 and Q0108 point 2: the Recruit's panel, and the Assist and Repair pointers;
  - **0323**, the 90° body-width sidestep, with wheeled vehicles turning and grazing instead;
  - **0324**, which answers Q0111 with a plan: 0323's sidestep first, measured;
- the M5 plan, and `plan.md` at `adb91011`;
- F1's worktree (`HoldsDef`, `holding.rs`), read only.

Cargo was not run.

**The state I assumed (2026-11-25):**
- A, R, D and M4-W2 (`b81b2089`) are merged.
- C, F1 and O3 are running. O3 was pulled forward from W4, so W4 has a free slot.
- W3 (B | E | F2) is as planned. W4 is J1 | L, plus that slot.
- Q0111 (the Agents' jam after 0315) has 0324's plan. As the Director asked, this addendum plans no slice for it;
  §C notes where 0323's sidestep collides with V1.

---

## 0. Summary

- **Scope: 15 new slices.**
  - **Content slices:**
    - **G2:** the 13 remaining Cults kinds; the Gunworks, Machine Shop and Scrap Hangar made complete; the Gunner unlocked;
    - **N1a, N1b:** per-unit stats with augmentation, then the Altar;
    - **N2:** the Garrison;
    - **N3:** the Temple and research;
    - **N4a, N4b:** the Cathedral, the Disciple and energy; then Shroud;
    - **N5:** the Shrine;
    - **N6:** the Chop Shop and the Jackal;
    - **N7:** the Airstrip and the Sparrow;
    - **N8:** Repair (0321), built once for Recruits and Agents. The Field Engineer gets it as data, but the unit
      itself isn't built.
  - **Engine slices** (the content can't do without them):
    - **V1:** oriented silhouettes;
    - **V2:** wheeled movement and Reverse;
    - **U1:** unit turrets;
    - **H1:** gliders and air.
  - G2 supersedes G. These slices replace the gated N1–N6.
- **Answered meanwhile:**
  - **Q0110,** by 0320 (the player's job), so no slice fixes it. L's layouts stay within 4–5 Recruits per face.
  - **Q0095 and Q0108 point 2,** by 0322. J1 builds the Recruit's panel (Q A S D X) and the Assist pointer, and N8
    builds the Repair pointer.
  - **Q0111,** by 0324's plan, whose first step is 0323's 90° sidestep. No slice for it here, as asked, but it
    collides with V1's file and would take W4's free slot (§C). 0323 also settles how the Jackal gets round blockers.
- **Waves:**
  - W4: J1 | L | U1, with U1 in the slot O3 left;
  - W5–W9: content, three builds each, with one Fable build in each of W5–W7;
  - W10: X1 alone; W11: X2 | S2; W12: M, then the playtest.
  - The playtest comes **four waves later** than planned. The first playable is unchanged.
- **Technical decisions:** T69–T83.
- **Question drafts:** 14 (D18–D31), none blocking.
- **Size:**
  - M5 now has **31 slices** and needs about **79 h** of agent work, of which about **64 h** is still ahead.
  - The new content is about 38 h, with three Fable slices.
- **An earlier playtest (§F):**
  - Playtest 1, at the end of W9: the Cults' infantry core with the Gunworks, the Garrison, the Altar and Repair. It
    takes G2, N1a, N8, N2 and N1b, then X1, X2 and M.
  - The rest follows as M5b, with a second playtest at the end of W14.

---

## §A Scope

**How it is cut.** Each building gets one vertical slice: the building, the units it makes, its panel and its tests.
There are two exceptions:
- **Engine slices.** Machinery two buildings share, or that Fable should build alone on test defs, gets a horizontal
  slice (V1, V2, U1, H1).
- **Kinds.** The registry's order is hashed, and the tech tree's `requires` names kinds across slices: the Machine
  Shop requires a Chop Shop, and the Scrap Hangar an Airstrip. So G2 registers every remaining kind at once (T69).

| 0319 content | Slice | Notes |
|---|---|---|
| Gunworks, Machine Shop, Scrap Hangar | **G2** | "no commands and can't be entered"; nothing to unlock but the Gunner |
| The Gunner, unlocked by the Gunworks | **G2** | the Armory's W made live |
| The Storage's armour, 1/6 → 1/4 (Q0094 a) | **G2** (taken from S1) | G2 owns the armour test's Cults rows |
| The base defs of the other 10 kinds: Garrison, Chop Shop, Airstrip, Altar, Temple, Cathedral, Shrine, Disciple, Jackal, Sparrow | **G2** | numbers only. Each building stays out of the Construct menu until its slice |
| The Altar and augmentation, with the Metabolism increment | **N1a** (per-unit stats, T67 built), **N1b** (the Altar) | |
| The Garrison | **N2** | F1's holding, plus occupants that fire (T72) |
| The Temple and research | **N3** | its technologies are the augmentation lines, so after N1b |
| The Cathedral, the Disciple, and energy | **N4a** | F2's pool production |
| Shroud | **N4b** | T76, T77 |
| The Shrine and its two abilities | **N5** | N4a's energy |
| The Chop Shop and the Jackal | **N6** | on V1, V2, U1 and F2 |
| The Airstrip and the Sparrow | **N7** | on H1, U1 and F2 |
| Enter's TargetObject, "the structure being entered" (Q0108.1) | **J1** (pointer and feedback text), **S1** (code docs) | |
| *Not 0319:* 0320 (Q0110 d, the player's job) | **no slice** | L's brief: reference layouts within 4–5 Recruits per face |
| *Not 0319:* 0321, Repair: Recruits (own buildings and vehicles, for crystals), Agents (Tunnels free, Laser Turrets for crystals), and the Field Engineer's rules | **N8** | one mechanism for every repairer, on E's builder machinery. The Field Engineer isn't built (D31) |
| *Not 0319:* 0322, the Recruit's panel (Q A S D X), the Assist and Repair pointers, and right-click Repair | **J1** (the panel with D `Planned`, and the Assist pointer); **N8** (D made live, the Repair pointer, and right-click Repair) | |
| *Not 0319:* 0323 and 0324, Q0111's first step: the 90° body-width sidestep | **no slice here**, as the Director asked | it collides with V1's file (§C); its line on wheeled vehicles settles D25's first point |

**Not in M5** (not designed):
- the Gunworks' and Machine Shop's research;
- the Scrap Hangar's aircraft;
- a Disciple casting from inside a Garrison (the doc says "TBD");
- the Field Engineer, and with it the GDO's repair. Its stats are "TBD", and so are its prerequisite's, the Career
  Campus's (D31).

---

## §B Slices

Every slice follows §3.2's Definition-of-done additions. 0319's numbers are the design's own placeholders: the code
cites the doc's "Placeholder stats" and 0319 next to each number. A `Placeholder (QD-slug)` mark is only for a gap
that §E drafts a question about. **After** lists only what isn't merged by W4's end. W3 and W4 merge B, E, F2, J1, L
and O3, which every slice below may assume.

#### G2: The remaining Cults kinds, the simple buildings, and the Gunner (Opus; supersedes G)

**Goal.**
- Every Cults building and unit of 0319 exists in the model with its numbers.
- The Gunworks, Machine Shop and Scrap Hangar can be built, and the Gunworks unlocks the Gunner.
- G2 is M5's last slice that adds a kind (T69).

**Design**
- **0319's table.** "Gunworks | 150 / 480 | 2x2 | 300 | 1/4"; "Machine Shop | 150 / 480 | 2x2 | 300 | 1/4"; "Scrap
  Hangar | 200 / 640 | 3x3 | 450 | 1/9"; and the other rows and units (§A).
- **`factions/cults/structures/gunworks.md`:**
  - "Unlocks — **Gunner** training at the Armory (Gunner's TechPrerequisite)";
  - "Size 2x2; SymmetryType AAAA; MaxHP 300; PointArmor 1, FullArmor 4; SightRange 4 grid units; Groupable true";
  - "**Until its research is designed it has no panel commands and Recruits can't enter it** (a right-click on it
    with Recruits is a Move; Q0093)".
- **`factions/cults/structures/machine_shop.md`:** "TechPrerequisite - player owns at least one Chop Shop"; "Until
  its research is designed it has no panel commands and Recruits can't enter it."
- **`factions/cults/structures/scrap_hangar.md`:** "TechPrerequisite - player owns at least one Airstrip"; "Size 3x3;
  SymmetryType AAAA; MaxHP 450; PointArmor 1, FullArmor 9"; "No panel commands (it only unlocks)."
- **`factions/cults/units/recruit.md > Constructs`:**
  - "Gunworks: 150 Space Crystals, 480 frames (30 seconds) BaseConstructionTime (placeholder). Requires: Armory.";
  - "Machine Shop: 150 ... 480 frames ... Requires: Chop Shop.";
  - "Scrap Hangar: 200 ... 640 frames ... Requires: Airstrip.";
  - and the seven other new lines.
- **`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`:** "**W: Train Gunner** ... and the player
  owns at least one Gunworks (TechPrerequisite)". `factions/cults/units/gunner.md`: 40 SC, 32 frames.
- **`core/tech_and_research.md > Unit Prerequisites`:** "Check Timing: checked only when the unit is queued"; "units
  already queued or in production continue and complete normally".
- **`factions/cults/economy.md > Storage`** (FullArmor 4, Q0094 a); **`core/entities.md > Building armor rule`**.
- **The other kinds' "Placeholder stats":**
  - `garrison.md`, `chop_shop.md`, `airstrip.md`, `temple.md`, `cathedral.md` and `shrine.md`;
  - `factions/cults/augmentation.md > Altar`;
  - `factions/cults/units/{disciple,jackal,sparrow}.md`.

**Placeholders**
- 0319, as cited.
- D29 point 1: recruit.md still lists "Enter Gunworks / Enter Machine Shop". G2 builds 0319's Move.

**Code** (T69, on A's pattern)
- **The sim:**
  - 13 def modules: 10 in `defs/cults/structures/` and 3 in `defs/cults/units/`.
    - They hold base fields only, plus each structure's `site` (E's field) and `requires` (M4-K).
    - Registry lines go at the end, in the tech tree's order: Gunworks, Garrison, Chop Shop, Machine Shop, Airstrip,
      Scrap Hangar, Altar, Temple, Cathedral, Shrine, Disciple, Jackal, Sparrow.
  - **The units, as given:**
    - the Jackal's `MovementAttrs::FixedTurnRadius {..}` and the Sparrow's `Glider {..}`;
    - their `TurretAttrs` and attacks (FullyConnected, Universal);
    - the Disciple's `attack: None`.

    Nothing reads their turns or turrets until V2, H1 and U1. No slice produces these units before N4a, N6 and N7.
  - **The Recruit's `builds`** (E's field) gains the three simple buildings. Each other building joins with its slice.
  - **The Armory's Gunner recipe,** `requires: [Gunworks]`, through `tech::check_requires` (`Reject::Requires`).
  - **The Storage's armour 1/4.** The armour test's Storage exception goes, and it gains rows for the ten buildings.
- **The client:**
  - 13 `kinds/cults/*.rs`. Labels follow `kinds::short_label`'s rule, and the label test keeps them unique.
  - The structures get A's look. The Jackal and the Sparrow get the plain unit box until V1, N6 and N7.
  - `CONSTRUCT_MENU` (J1's): the three entries go from `Planned` to `Kind`. The other seven stay greyed (0291).
  - W goes live in `templates/cults/armory.rs`.
  - A right-click with Recruits on any of the three is a Move (J1's rule for an own building that holds nothing).
- **Tests that walk every kind** take the new kinds: the defs' consistency, the labels, the editor's palette. A kind
  whose behaviour isn't built is never produced or offered.

**Acceptance**
- **Construction.** Four Recruits build each simple building from the Construct menu, at 0319's cost and time,
  scaled by E's rule.
- **Prerequisites.**
  - The Machine Shop is refused without a complete Chop Shop, and the Scrap Hangar without an Airstrip. The test
    spawns a complete Chop Shop and Airstrip: their kinds exist.
  - W stays greyed, "Requires a Gunworks", until one is complete. Then a Gunner costs 40 SC and 32 frames.
  - When the last Gunworks is lost, queued Gunners complete and no new one can be queued.
- **No entering.** Recruits right-clicked onto a Gunworks walk to it and don't go in.
- **Armour.** Every Cults building follows the rule (n = 2 for the Garrison) with no exception left.
- **QA screenshot:** the Construct menu, with three new live entries and seven greyed.
- **The golden.** L's `cults_vs_syndicate` gains a Gunworks and Gunners, re-blessed once. R's pairing test stays
  green.

**Owns:** see §C. **After:** none beyond W4. **Model:** Opus, about 3 h. If it runs long, each unit kind moves to its own
slice (N4a, N6, N7), one per wave.

#### V1: Oriented silhouettes (Fable)

**Goal.** A non-square Silhouette rotates with its unit's facing for collision, range and proximity. A turn blocked
by an obstacle is cut as a blocked move is (0035).

**Design**
- **`systems/units.md > UnitCollision > Silhouette Rotation`:** "A unit's Silhouette **rotates with its facing** ...
  Collision, crushing, range measurement and proximity all use the rotated shape. A long unit fits through a narrow
  gap only when pointed through it."
- **`> Collision Response (Un-sticking)`:** "Overlap caused by the rotation itself (a corner sweeping into a wall)
  follows the same principle: the blocked part is discarded and the unit never ends up overlapping. The exact
  geometric method is a technical choice."
- **`systems/combat.md > AttackAttributes > Range Measurement`:** "Range and MinRange are measured **edge to edge**
  ... Shapes are a unit's Silhouette and a structure's footprint."
- **`factions/cults/units/jackal.md`:** "Silhouette 56x40 space units (length x width)". It is the game's first
  non-square unit.
- `plan.md`'s queue: "0033 ... **rotation queued for the vehicles milestone**"; `docs/movement.md` G6.

**Placeholders**
- No new gap. T78's approximation (square silhouettes stay axis-aligned) is technical. It is recorded in
  `docs/movement.md` G6 and in ARCHITECTURE.

**Code** (T78)
- **`shape.rs`:**
  - `Shape` gains an angle, set only for non-square silhouettes;
  - overlap by the separating-axis test;
  - the gap as the least vertex-to-edge distance;
  - its bounding box for tile work (vision, grids).

  Its module docs promise "oriented boxes change this module only", for measurement.
- **Collision** (`movement/collide.rs`, 477 lines, split first):
  - A rotating unit's step (move and turn) is tried whole. If it is blocked, it is halved up to five times; the free
    part is kept and `Mover::momentum` is zeroed (0035).
  - A square box against an oriented one uses the same test.
- **Nav:** the size class still comes from the longer side (56 su: class 1).
- **Test defs:** a 56x40 test unit on TurnRate movement, so V1 needs nothing from V2.
- **Client:** an oriented shape draws a rotated box (`object_render/units.rs`).

**Acceptance**
- **Unit tests** for overlap and gap on rotated pairs: touching, corner to edge, edge to edge.
- **Gaps.** The 56x40 test unit passes a 48-su gap pointed through it, and is blocked across it.
- **Turning at a wall.** It turns in place beside a wall and stops at contact. The soak's no-overlap invariant
  covers oriented shapes.
- **Range.** It is measured to a rotated target's edge: a Militant fires at 4 GU from the long side.
- **No golden changes:** no square shape changes.
- **Perf:**
  - `movement_perf`: interleaved A/B within noise;
  - a new `vehicles_perf` (30 oriented units in a crowd), with its mean and p99 recorded.

**Owns:** see §C. **After:** none. **Model:** **Fable**, about 3 h.

#### U1: Unit turrets (Opus)

**Goal.** A unit's turret aims on its own, within its arc and at its turn rate, while the base moves. With no locked
target, it scans for one, during a Move too.

**Design**
- **`systems/units.md > TurretAttributes`:** "The turret is centered on the unit's facing direction, with TurnAngle
  split equally clockwise and counter-clockwise from center."
- **`systems/combat.md`:**
  - `> TurretSource`: "The turret rotates independently to aim while the unit base is free to move and turn during
    all attack phases.";
  - `> AttackPhase > Aiming`: "The unit or turret rotates to face the target."
- **`systems/controls/behaviors.md`:**
  - `> TurretAutonomousScanning`: "selects the best available target from ValidTarget enemies within weapon range
    and turret arc". The priority is threatening first, then the least rotation "in **45° steps**", then the closest;
  - `> BaseAutoTargeting > Not active during`: "Move — pure movement, no base engagement. Turret units'
    TurretAutonomousScanning still operates independently";
  - `> AttackingObject`: "set TurretCommandState.LockedTarget = TargetObject (turret units)";
  - `> TurretTurning`: "applies the turret's turn rate each tick, constrained by TurnAngle".
- **0319:** the Jackal's "turret 360° at 12°/f"; the Sparrow's "turret 45° at 8°/f".

**Placeholders**
- 0319 only. When fire waits for alignment is technical (T80).

**Code** (T80)
- **State:** `AttackState::turret: Option<TurretState { facing, locked }>`, relative to the base. It is hashed only
  while set (T65).
- **Turning:** in 5e, before the phases. At most TurnRate a tick, clamped to ±TurnAngle/2 of the base's facing. The
  base's own turn carries the turret.
- **Scanning.** It goes in `orders/world.rs` (474 lines, split first), using its existing threat ranking and
  rotation helper, measured from the turret's facing.
  - It runs in every behaviour when nothing is locked, Move included.
  - The behaviours set `locked` wherever the design sets LockedTarget.
- **`combat.rs`** (456 lines): the turret code goes into a new `combat/turret.rs`, and `combat.rs` is split first if
  it would pass about 480 lines.
- **Test defs:** a turret unit on TurnRate movement (a 90° arc at 10° a tick).
- **Client:** the Laser Turret's `TurretBarrel` (`object_render/turret.rs`) on units, its facing interpolated.

**Acceptance**
- **Firing on the move.** A unit driving east with a 360° turret fires at an enemy to its north and never stops.
- **Arc.** A 45° cone doesn't fire at a target 30° off its facing until the base turns.
- **Turn rate.** 90° at 12° a tick takes 8 ticks before the first shot.
- **Scanning.** A table test of the scan order, from `behaviors.md`'s example ("an enemy at max range straight ahead
  and an adjacent enemy 5° to the side are in the same step").
- **Goldens:** unchanged.
- **QA screenshot:** a barrel tracking a target while its unit moves.

**Owns:** see §C. **After:** O3, which is running and also changes the attack phases. **Model:** Opus, about 2.5 h.
It takes the W4 slot that O3 left.

#### V2: Wheeled movement and Reverse (Fable)

**Goal.** A WheeledVehicle drives as designed:
- it can't turn in place, and turns on a fixed radius;
- it accelerates and brakes;
- it backs up on the Reverse order;
- it creeps out of contact.

**Design**
- **`systems/units.md`:**
  - `> FixedTurnRadiusMovement`: "Unit cannot turn in place. Turns at a fixed minimum radius regardless of speed. Can
    stop and reverse.";
  - `> WheeledVehicle`: RuggedTerrain false, CanTurnInPlace false, CanReverse true, DirectionalArmor true.
- **`systems/combat.md > LocomotionOrientationConstraints[FixedTurnRadiusMovement]`:**
  - "Stationary + Turning: invalid";
  - "Moving + Turning: maxTurnRate = currentSpeed / MinimumTurnRadius";
  - "Reversing + Turning: maxTurnRate = currentSpeed / MinimumTurnRadius";
  - "Stopping + Turning: invalid".
- **`systems/units.md > Collision Response (Un-sticking)`:** "it moves at **single-frame-acceleration speed**. It
  creeps, but it keeps making its turn ... This matters most for units that can't turn in place (WheeledVehicle:
  turning needs forward motion)".
- **Reversing:**
  - `systems/controls/commands.md > Reverse`;
  - `systems/controls/behaviors.md > ReversingToLocation`: "Identical to MovingToLocation except the plan uses
    Reversing locomotion instead of Moving.";
  - `systems/controls/selection_and_hud.md > BasicCombatUnitInterfaceState`: "[W] Reverse*"; "W (Reverse) only
    available if UnitBase has CanReverse = true."
- **`systems/units.md > Ground Collision`:** "Vehicles' version (turning to graze around obstacles) is designed with
  vehicle movement." **0323** settles it: "**WheeledVehicles** turn and graze around the obstacle instead" of
  stepping sideways. So V2 builds the grazing as a rule, not a placeholder.
- **0319, the Jackal:** "MinimumTurnRadius 64 su (1 grid unit); ForwardMaxSpeed 8 su/frame; ForwardAcceleration 1
  su/frame²; ReverseMaxSpeed 3 su/frame; ReverseAcceleration 0.5 su/frame²; Deceleration 1 su/frame²."

**Placeholders**
- **D25:** yielding, reversing on its own, and a goal inside the turning circle.

**Code** (T79)
- **`movement/wheeled.rs`,** beside `drag.rs`:
  - a signed speed in `Mover::momentum`;
  - acceleration, braking and the braking distance;
  - a turn rate of |v| / r, and no turn while Stationary or Stopping;
  - 0035's zeroing on contact, through V1's cut.
- **`Mover::turn_rate`** for the speed-dependent models gets filled in (its `TODO(movement models)`).
  `movement.rs` (447 lines) is split first.
- **Steering:** toward `paths::serve`'s steering target. A goal inside the turning circle is reached by a forward
  loop (D25), and the vehicle brakes in time to stop at it.
- **Reverse:**
  - `OrderKind::Reverse(Vec2)` and the ReversingToLocation behaviour;
  - its gate arm;
  - the client's `PanelCommand::Reverse` goes live (it "has no order yet").
- **Yielding** (`movement/yielding.rs`), with D25's placeholders: a vehicle doesn't sidestep, and an idle vehicle
  doesn't yield.
- **Test defs:** the 56x40 test vehicle, or G2's Jackal def.

**Acceptance**
- **Speed.** From a standstill, full speed after 8 frames; a stop takes 8 frames and 32 su.
- **Turning.** A 90° turn at full speed follows a 64-su arc, within 2 su.
- **Behind it.** Ordered to a point behind it, it loops forward and never turns in place.
- **Reverse.** W drives it backwards at up to 3 su/frame, turning at |v| / r.
- **Walls.** Pressed against a wall at an angle, it creeps along and scrapes free (0035's verification).
- **Terrain.** Rugged tiles are impassable (`NavClass::Ground`).
- **Crowds.** Eight test vehicles cross a field of idle own infantry with no overlap, within a time bound.
- **Perf:** `vehicles_perf` with 30 vehicles crossing each other, mean and p99; `movement_perf` A/B.
- **Goldens:** unchanged.

**Owns:** see §C. **After:** V1. **Model:** **Fable**, about 3 h.

#### H1: Gliders and air (Fable)

**Goal.** A Glider flies as designed:
- it never stops, and circles when idle;
- it turns within its centripetal limit;
- it strafes what it attacks;
- it may overshoot the map edge.

Air units keep apart softly.

**Design**
- **`systems/units.md > GliderMovement`:** "Unit must always maintain movement to stay airborne. When idle, the unit
  circles at a low speed in tight loops. When given orders, the unit accelerates to max speed. Turn radius is
  governed by centripetal acceleration (r = v²/a)".
- **`systems/units.md > Air Collision`:**
  - "**Gliders may unavoidably travel a little outside the map edge** while turning or circling";
  - "Orders can only target points inside the map";
  - "A Glider outside the map edge **stays visible and attackable as if it were at the nearest edge tile**";
  - "TBD: maximum overshoot";
  - "Air units use soft separation with other air units — a gentle repulsion force that prevents stacking but does
    not hard-block";
  - "SeparationRadius — per unit type, circular ... Must be larger than the unit's Silhouette".
- **`systems/combat.md > LocomotionOrientationConstraints[GliderMovement]`:** "Moving + Turning: maxTurnRate =
  f(currentSpeed, MaxCentripetalAcceleration)".
- **`systems/controls/behaviors.md`'s Glider exceptions:**
  - MovingToLocation: "Final step is Locomotion = Moving(idle circle over TargetLocation) ... Behavior does not
    complete";
  - AttackingObject: "1. Fly toward TargetObject 2. When in range and turret arc: fire while passing through 3.
    Continue past the target 4. Loop around for another pass 5. Repeat until target is destroyed or a new command is
    issued";
  - AttackingLocation: the same;
  - AttackMovingToLocation: "performing strafing runs on enemies within SightRange ... On arrival at TargetLocation,
    transitions to idle circling";
  - ReversingToLocation: "not applicable".
- **`core/vision.md > Air exempt`** (already built: `elevation.rs`).
- **0319, the Sparrow:** "IdleSpeed 4 su/frame; MaxSpeed 10 su/frame; Acceleration 0.5 su/frame²; Deceleration 0.5
  su/frame²; MaxCentripetalAcceleration 1 su/frame² (turn radius about 100 su at full speed)".

**Placeholders**
- **D26:** the SeparationRadius, the overshoot, the idle radius, strafing, and Stop and Hold.

**Code** (T81)
- **`movement/glider.rs`:**
  - the speed stays between IdleSpeed and MaxSpeed;
  - the turn rate is a / v;
  - it is never Stationary.
- **The behaviours' glider arms:**
  - idle circling;
  - strafing passes (through the target, on until out of range, then a loop back);
  - AttackMove strafing what it sees;
  - Stop and Hold circle where the Glider is.
- **Air separation** (`movement/air.rs`): after the moves, for each pair within `UnitDef::separation_radius` (a new
  field), a capped push, in id order. It never blocks.
- **The map edge:**
  - a position may leave the map;
  - `Object::map_pos()` clamps it to the nearest edge tile, for vision, targeting, grids and selection;
  - the gate clamps orders' points into the map.
  - `vision.rs` (471 lines) is split first if this adds to it.
- **Test defs:** a glider with a 45° turret. With U1 merged, it fires while passing.
- **Client:** air units are drawn raised, with a ground shadow, at their true position, even past the edge.

**Acceptance**
- **Idle.** It circles its last point at IdleSpeed, on a radius of IdleSpeed²/a (16 su), and never stops.
- **Speed and turns.** It accelerates at 0.5 su/frame² to 10. A 180° turn at full speed has a radius of 100 su.
- **Strafing.** Attack makes repeated passes over a stationary target. It fires only while the target is within its
  45° cone and 3 GU.
- **The edge.**
  - Circling near the edge takes it off the map. It stays visible and attackable at the nearest edge tile.
  - A click past the edge moves it to the edge.
- **Separation.** Six gliders sent to one point circle without stacking: the closest pair stays at least the
  SeparationRadius apart after 10 s.
- **Existing cases.** `orders/tests/air.rs`'s HoverCraft cases still pass.
- **Perf:** a new `air_perf` with 40 gliders, recorded.
- **Goldens:** unchanged.

**Owns:** see §C. **After:** U1, for strafing fire. **Model:** **Fable**, about 3 h.

#### N1a: Per-unit stats and AugmentationState (Opus)

**Goal.**
- A unit's MaxHP, speed and attack are read through accessors that apply its own augmentation (T67, now built).
- Units carry an AugmentationState.

**Design**
- **`factions/cults/augmentation.md > Increments`:**
  - "**Constitution increment**: +10 MaxHP; +1 melee Damage (unarmed units only)";
  - "**Metabolism increment**: **+0.5 space units/frame MaxSpeed** (placeholder); for unarmed units also **−1 frame
    of FiringDuration** per increment (placeholder; FiringDuration never below 2 frames; a negative increment adds 1
    frame)";
  - "Increments can be negative.";
  - "**Unarmed units** are infantry whose attack is an unarmed melee attack — currently only the Recruit. Armed
    infantry (Militant, Gunner) receive only the MaxHP part of Constitution increments and only the movement-speed
    part of Metabolism increments."
- **`> Augmentation Effects`** (the table), and "A unit with both sums their effects."
- **`> AugmentationState (per unit instance)`; `> Eligibility`.**
- **`> Persistence`:**
  - "**Training** ... a trained unit inherits its Recruit's AugmentationState unchanged";
  - "**Vehicles**: if augmented Recruits are consumed to produce a vehicle, their Augmentations are lost";
  - "**Death**: Augmentations are lost with the unit."
- **`factions/cults/units/disciple.md > AttackAttributes`:** "None (placeholder: no weapon; for Augmentation it is
  **armed** ... To confirm with the user.)"
- **0319:** the Metabolism increment.

**Placeholders**
- **D20, point 1:**
  - when MaxHP changes, the damage is kept, but HP never goes below 1;
  - Improved Metabolism at L2 gives a Recruit MaxHP 10 and melee Damage 0, as written.

**Code** (T73)
- **The state:** `Object::augmentation: Option<Augmentation>`, hashed only while set.
  - `change_kind` keeps it.
  - It is cleared when the new kind isn't augmentable (a vehicle).
- **The def:** `UnitDef::augmentable: Option<Augmentable { unarmed }>`, set on the Recruit (unarmed), the Militant,
  the Gunner and the Disciple.
- **The data:** the increments and the effects table, in `defs/cults/augmentation.rs`.
- **The accessors:**
  - `Object::max_hp()` adds 10 per Constitution increment (c);
  - a new `Object::max_speed()` adds 0.5 su a tick per Metabolism increment (m). It replaces the `max_speed()` reads
    in `movement.rs`, `movement/{doorway,yielding,follow}.rs`, `orders/{engage,attack_ground}.rs`, `riot_shield.rs`
    and `testing/invariants.rs`;
  - `combat::weapon(o)` returns `Option<AttackAttrs>`, owned. An unarmed unit gets Damage + c and FiringDuration − m,
    at least 2. It has 17 callers; `Shooter` keeps a copy.
- **HP** follows the maximum by `transform`'s rule (damage kept), floored at 1 (D20).
- **`DebugCommand::SetAugmentation`,** for QA and tests. The gate refuses it unless debug commands are on.

**Acceptance**
- **Table tests:**
  - a Militant with Enhanced Constitution at L2 has MaxHP 100;
  - a Recruit with both at no research: MaxHP 50, Damage 4, FiringDuration 7, speed 4.5;
  - a Recruit with Improved Metabolism at L2: MaxHP 10, Damage 0 (each hit then does the 0.5 minimum), FiringDuration
    2, speed 7.
- **Kind changes.** `change_kind` to a Militant keeps the augmentation; to a test vehicle, it clears it.
- **HP.** A damaged unit keeps its damage when its MaxHP rises, and keeps 1 HP when it falls.
- **Goldens:** byte-identical, since none sets an augmentation.
- **Perf:** `movement_perf` and `recruits_perf` A/B within noise.

**Owns:** see §C. **After:** G2 (the Disciple's def). **Model:** Opus, about 2 h.

#### N1b: The Altar (Opus)

**Goal.**
- Cults infantry enter the Altar and are augmented there.
- Its queue is serial and paid at queue. Each order reserves a unit of the chosen type and state.
- Its panel has three stages.

**Design**
- **`factions/cults/augmentation.md > Altar`:**
  - "Any Cults infantry can enter the Altar ... The Altar works like the Armory: it holds an internal pool of units,
    processes a serial queue of orders, and sends finished units out to its RallyPoint.";
  - 2x3, ABAC, "EntranceSide / ExitSide - B / C", "InternalUnitCapacity - 10".
- **`> Augmentation Orders`:**
  - "An order can only target a unit that is missing every Augmentation the order applies.";
  - "Queuing an order immediately deducts its Space Crystals cost and reserves one unit of the selected type and
    AugmentationState from StoredUnits ... When an order finishes, the unit's AugmentationState is updated using the
    research level in effect at that moment ... and the unit **automatically exits** from the exit side and moves to
    the RallyPoint.";
  - "25 Space Crystals, 160 frames (10 s) each; **Both**: the sum, 50 Space Crystals, 320 frames" (0319).
- **`> AltarInstanceState`:** "AugmentationQueue ... (max 20, placeholder)".
- **`> Eject All`:** "Units reserved by a queued order are not ejected. This is the only way for fully augmented
  units to leave."
- **`> ObjectInterfaceState[Altar]`:**
  - the type tiles Q Recruit, W Militant, E Gunner, R Disciple;
  - the state tiles Q Unaugmented, W Enhanced Constitution only, E Improved Metabolism only, R Fully augmented
    ("always disabled");
  - the orders: Q Enhanced Constitution, W Improved Metabolism, E Both;
  - Z goes back; A Eject All, X Cancel Production, C Set Rally Point; right-click sets the rally.
- **`> Level Locking`.**
- **Entering:**
  - `systems/controls/commands.md > Enter`: "**Cults — Altar**: any Cults infantry (Recruit, Militant, Gunner,
    Disciple)";
  - `systems/controls/selection_and_hud.md > BaseCommandState[UnitBase] > RightClickResolution`: "Cursor over own
    Altar (Cults infantry only — Militant, Gunner, Disciple): issues Enter";
  - `factions/cults/units/recruit.md`: "Own Altar → Enter Altar".
- **`factions/cults/economy.md > Evacuation on Destruction`:** "a unit mid-augmentation does not receive the
  Augmentation".

**Placeholders**
- **D20, point 2:** the unit of that type and state that has been inside the longest is reserved.
- **Q0105's doorway placeholder,** as at the Armory.
- **Levels:** NoResearch until N3.

**Code** (T70, T71)
- **The Altar's `holds`:**
  - entrance B, exit `ExitRule::Side(C)`;
  - it accepts the four infantry kinds;
  - capacity 10.
- **Its `production`:** the entry `QueueEntry::Augment(AugmentOrder { order, kind, state })`, which reserves one
  matching unit; `queue_limit` 20.
- **Completion:**
  - it stamps the levels through `Sim::augment_level(player, line)`, which returns NoResearch until N3 fills it in;
  - then it releases the unit through F2's path from the held store, to the rally.
- **`ProductionCommand::Augment { kind, state, order }`,** and the refusals: no matching unit, a full queue, too few
  SC, and an augmentation the unit already has.
- **The client:**
  - `templates/cults/altar.rs`: the three stages on J1's menu pattern, with each tile's count; the queue in the
    InfoPanel;
  - right-click Enter Altar for the four kinds;
  - the Construct menu entry goes live, and the Altar joins the Recruit's `builds`.
- **The client's `command_ui/interface/commands.rs`** (467 lines) is split first.

**Acceptance**
- **One augmentation.** With three Militants inside, W, Q, Q queues Enhanced Constitution for one: 25 SC and 160
  frames. The Militant leaves through C with MaxHP 60.
- **Both, on a Recruit:** 50 SC and 320 frames, giving MaxHP 50, Damage 4, speed 4.5 and FiringDuration 7.
- **Fully augmented units.** One can enter, its R tile is always disabled, and Eject All brings it out.
- **Cancel** refunds the order and returns its unit.
- **Destruction** evacuates everyone unaugmented and refunds every order.
- **Training after augmenting.** An augmented Recruit trained at the Armory keeps its augmentation.
- **QA screenshots:** each stage of the AugmentMenu, and the InfoPanel's queue.
- **A new golden,** `cults_altar`: augmented Militants against Agents.

**Owns:** see §C. **After:** N1a and G2. **Model:** Opus, about 2.5 h.

#### N2: The Garrison (Opus)

**Goal.**
- Infantry enter the Garrison and fire from it, and it protects them.
- Eject All puts them out on any side.

**Design**
- **`factions/cults/structures/garrison.md`:**
  - "Recruits and Disciples fire **as if they were Militants** (using the Militant's attack ...), while Militants and
    Gunners fire their own weapons. Without units inside, the Garrison does not attack.";
  - "**Range bonus**: **Militants and Gunners** inside get **+1 grid unit of Range** on their attack. Recruits and
    Disciples (firing with the Militant's attack) do not get the bonus";
  - "**Protection**: units inside cannot be targeted and take no damage while the Garrison stands.";
  - "Units inside see with the Garrison's SightRange and fire at enemies the player sees within their (bonus)
    range.";
  - "Units **leave through any side**, at the first clear spot next to the footprint (like Evacuation on
    Destruction).";
  - "Panel: **A: Eject All** (everyone inside leaves); **X: Cancel construction** while being built. Right-click
    resolution: none (it has no rally point).";
  - "(Whether an Disciple can also cast spells from inside: TBD.)"
- **`systems/controls/commands.md > Enter`:** "**Cults — Garrison**: Recruits, Militants, Gunners, and Disciples (up
  to 4 ...)". `selection_and_hud.md > RightClickResolution`: "Cursor over own Garrison (Cults infantry only —
  Militant, Gunner, Disciple): issues Enter".
- **0319's verification:** "A Garrison with 4 Militants fires at range 5."
- **The weapons:**
  - `factions/cults/units/militant.md > AttackAttributes`: Damage 4, Range 4 GU, Universal;
  - `factions/cults/units/gunner.md`: TailDisjointed, Range 5, MinRange 2 (O3's attack).
- **`factions/syndicate/structures/laser_turret.md > Targeting`:** the rule a structure's own pick follows (T49).

**Placeholders**
- **D21:** targets, range, Eject All, casting from inside, and whose units may enter.
- **D22:** the side units enter.

**Code** (T72)
- **`holds`:**
  - it accepts the Recruit, Militant, Gunner and Disciple;
  - capacity 4;
  - entrance by D22: the side nearest the unit;
  - exit `ExitRule::AnySide`, which is new: everyone at once at T62's spots, idle, as in an evacuation.
- **`StructureDef::occupants_fire: Option<OccupantFire>`:** the stand-in attacks (Recruit → Militant, Disciple →
  Militant), and +1 GU for the Militant and the Gunner.
- **Phase 5e** (a new `combat/occupants.rs`), right after the structures' own attacks:
  - each occupant runs its own `AttackState` from the footprint;
  - it uses T49's `pick`, the Garrison's vision, and its stand-in or own weapon with the bonus.
- **The client:**
  - `templates/cults/garrison.rs`: A, X while a site, and the occupants in the InfoPanel;
  - right-click Enter for the four kinds, with the Enter pointer;
  - shots start from the footprint's edge nearest the target;
  - the menu entry goes live, and `builds` gains the Garrison.

**Acceptance**
- **0319's verification:** with four Militants inside, a target 4.9 GU from the footprint is hit and one at 5.1 GU
  isn't.
- **Stand-in weapons.** A Recruit inside fires the Militant's attack at 4 GU.
- **Gunners** fire TailDisjointed shots from the footprint, and their MinRange of 2 counts from it.
- **Empty:** no attack.
- **Protection.**
  - Occupants can't be targeted, even by area damage over the Garrison.
  - Destroying it evacuates them alive (F1).
- **Eject All** puts all four at the first clear spots, idle. Unit Control doesn't change while they are inside.
- **Perf:** `structures_perf` with ten full Garrisons, A/B.
- **A new golden,** `cults_garrison`: Militants and a Gunner inside, against Agents and Grenadiers.
- **QA screenshot:** a Garrison firing, with its InfoPanel.

**Owns:** see §C. **After:** G2. **Model:** Opus, about 2.5 h.

#### N3: The Temple and research (Opus)

**Goal.**
- The game's first research. The Temple researches the two augmentation lines, staffed by Recruits.
- The level is locked onto each unit when its augmentation finishes.

**Design**
- **`core/tech_and_research.md > Research > Shared Rules`:**
  - "1. **Once per player, applies everywhere**";
  - "2. **Cost and time** ... The cost is deducted when the technology is queued.";
  - "3. **Serial queue**";
  - "4. **No duplicates**: a technology cannot be queued or in progress at more than one structure at the same time.
    Once completed, it no longer appears in any research menu.";
  - "5. **Levels**: a multi-level research line is a chain of separate technologies, each requiring the previous
    level.";
  - "6. **Cancel** ... refunds its full cost ... removes the last entry";
  - "7. **Destruction** ... cancelled, with a full refund of each";
  - "8. **Permanence**".
- **`> Faction Exceptions > Cults — stat upgrades are not retroactive`:** "**Augmentation improvements** (Temple):
  the moment the unit's augmentation finishes at the Altar".
- **`> Cults — research is staffed by Recruits`:**
  - "**BaseResearchTime** ... the time with exactly 1 Recruit inside";
  - "**Linear scaling** ... 2 Recruits halve it";
  - "**Pauses when empty** ... (no decay) ... Queued research waits likewise."
- **`factions/cults/structures/temple.md`:**
  - the table of the four technologies;
  - "Level 2 of both lines ... also requires the player to own a Cathedral";
  - "Recruit staffing capacity 6; research queue 5";
  - "**100 SC, 960 frames (60 s)** each; L2 of each: **150 SC, 1440 frames (90 s)**";
  - "Panel: **Q** Enhanced Constitution (next level), **W** Improved Metabolism (next level), **A: Eject All** (staff
    leave), **X: Cancel** (last research). Recruits enter by right-click."
- **`factions/cults/augmentation.md > Level Locking`:** "A unit receives the research level that is in effect **at
  the moment its augmentation finishes**".
- **`factions/cults/economy.md > Evacuation on Destruction`:** "Recruits staffing research" come out, and research is
  refunded.

**Placeholders**
- **D22:** the side.
- **D23:** the next level, and the Cathedral check.

**Code** (T70, T74)
- **Data:** `defs/cults/research.rs` holds the `Tech` list and each `TechDef`.
- **Per player:** `techs: BTreeSet<Tech>`, hashed only while it isn't empty.
- **The Temple:**
  - `holds`: Recruits, capacity 6, the nearest side;
  - `StructureDef::research`: the techs it offers;
  - the queue entries are `QueueEntry::Research(tech)`, with `queue_limit` 5;
  - progress is one `RATIO_ONE` a frame per Recruit inside.
- **`ProductionCommand::Research(Tech)`.** The gate checks, in order:
  - no duplicates (a scan of the owner's research queues);
  - the previous level complete;
  - the structure requirement (M4-K's `check_requires`);
  - the cost.
- **Completion:** the tech joins `techs`, and N1b's `Sim::augment_level` reads it. Research completes before
  augmentations in the same frame (T74).
- **Cancel and destruction** refund through Production's paths.
- **The client:**
  - `templates/cults/temple.rs`: Q and W show the next level, then A and X;
  - the InfoPanel shows the queue, the progress and the staff count;
  - right-click Enter Temple for Recruits;
  - the menu entry goes live, and `builds` gains the Temple.

**Acceptance**
- **Staffing.** With one Recruit inside, Enhanced Constitution L1 takes 960 frames; with three, 320. With none it
  makes no progress, and it resumes when a Recruit returns.
- **Cost.** 100 SC at queue, refunded in full by X (the last entry) and by the Temple's destruction, which evacuates
  its staff.
- **Level 2** needs L1 complete and a complete Cathedral (`Reject::Requires`). Losing the Cathedral after queuing
  cancels nothing.
- **No duplicates.** A second Temple can't queue what the first has queued.
- **Level locking.**
  - An augmentation finishing after L1 completes gives +4/−2.
  - One that finished earlier keeps +2/−1.
  - Both on the same frame give L1.
- **QA screenshots:** the panel with a queue, and the InfoPanel's progress and staff.

**Owns:** see §C. **After:** N1b, and N2 (the nearest-side entrance, D22). **Model:** Opus, about 2.5 h.

#### N4a: The Cathedral, the Disciple, and energy (Opus)

**Goal.**
- The Cathedral trains Disciples from its pool.
- Units and structures can have energy, which regenerates.

**Design**
- **`factions/cults/structures/cathedral.md`:**
  - "Works like the Armory's Training: Recruits enter the Cathedral and join an internal pool; training orders are
    queueable and serial; queuing an order deducts its Space Crystals cost and reserves one Recruit; when the order
    finishes, the Recruit is consumed and the trained unit exits to the RallyPoint.";
  - "Size 2x3; SymmetryType ABAC; EntranceSide B, ExitSide C (like the Armory)";
  - "InternalRecruitCapacity 10; training queue 20";
  - "**Disciple**: 1 Recruit, 75 Space Crystals, 320 frames";
  - "Panel like the Armory's: **Q: Train Disciple**, **A: Eject All**, **C: Set Rally Point**, **X: Cancel
    Production**".
- **`factions/cults/units/disciple.md`:**
  - `> Energy`: "The Disciple has its own energy pool, which it spends on its spells. It regenerates naturally over
    time";
  - the Placeholder stats: "LightInfantry", "24x24", "MaxHP - 40", "SightRange - 6 grid units", "Energy - MaxEnergy
    200; starts at 50; regenerates 0.5 per second", "AttackAttributes - None (placeholder: no weapon ... To confirm
    with the user.)", "ObjectInterfaceState: BasicCombatUnitInterfaceState, plus **Q: Shroud**".
- **`systems/controls/commands.md > Enter`:** "**Cults — Armory, Cathedral**: Recruits only (to be trained)".

**Placeholders**
- **D18:** Shroud's letter and grouping. N4a shows the button greyed until N4b.
- **D27:** the trained Disciple keeps its Recruit's damage, which is Q0105's placeholder.
- **The Disciple has no weapon** (0319: "to confirm with the user"). The Director raises it with the user; it isn't a
  QD.

**Code** (T71, T75)
- **The Cathedral:** `holds` (B, C, Recruits, 10), and `production` with the Disciple's recipe and `queue_limit` 20.
  It is F2's pool path, as data.
- **Energy:**
  - `ObjectDef::energy: Option<EnergyDef { max, start, regen }>`, which the Shrine reuses (N5);
  - `Object::energy`, an integer count of `RATIO_ONE`s, hashed only while set;
  - `energy::step`, right after `shields::step`.
- **The client:**
  - an energy bar under the HP bar;
  - the Disciple's panel: BasicCombat, its unavailable attack commands greyed, and Shroud's slot;
  - `templates/cults/cathedral.rs`;
  - right-click Enter Cathedral for Recruits;
  - the menu entry goes live, and `builds` gains the Cathedral.

**Acceptance**
- **Training.** Q with a Recruit inside: 75 SC and 320 frames. A Disciple leaves through C with 50 energy, and reaches
  200 after 300 s.
- **Shared paths.** Group Q, X and evacuation work as at the Armory (F2's tests, re-run on the Cathedral).
- **No weapon.** A Disciple never auto-targets, and a right-click on an enemy moves it, as for any unit without an
  attack.
- **Augmentation** (with N1a) gives a Disciple only its MaxHP and speed parts.
- **QA screenshots:** the Cathedral's panel, and a Disciple's energy bar.

**Owns:** see §C. **After:** G2. **Model:** Opus, about 2 h.

#### N4b: Shroud (Opus)

**Goal.** A channelling Disciple casts a cloud. Anti-ground damage aimed at ground objects under the connected cloud
goes into a shared buffer, which is released every 8 frames as one area hit over the whole cloud.

**Design** (`factions/cults/units/disciple.md > Spells > Shroud`)
- "A **channeling** spell: while the Disciple channels it, its energy **drains gradually**, and it creates a
  **cloud**."
- "every attack against a unit standing **under the cloud** has its damage **converted to area (AoE) damage**,
  spread across the **whole area of all clouds contiguously connected** to the cloud the attacked unit stands
  under."
- "**The cloud is centered on the Disciple** ... The Disciple **can't move while channeling**."
- "**It affects every ground object under it** ... **ground units and buildings**, whoever owns them, **not air
  units**. **All anti-ground damage** landing on objects under the cloud, single-target and area alike, is
  **absorbed and redistributed**".
- "each receiver's share from a release is treated like any damage it receives: its own pre-armor reductions, armor,
  directional armor and minimum damage apply as always".
- "if the channel ends or the connected cloud splits, the buffer is **released immediately** over the cloud as it
  was."
- "only the Disciple's **death**, a **cancel** command (Stop or any other order), or running **out of energy**.
  Taking damage does **not** end it."
- "**all contiguously connected clouds share one damage buffer**".
- "an attack fired by a unit standing **under the cloud** is not absorbed; it hits normally. Designer's reading: under
  the same connected cloud as its target".
- "Every **0.5 s (8 frames), placeholder**, the buffer is released as **one area hit over the whole connected
  cloud** and empties. Each ground object under it takes its share by **overlap area** ... the share falling on
  empty cloud area is **lost**."
- The numbers: "cloud radius **3 grid units** around the Disciple; drains **5 energy per second** (40 s from full);
  needs at least **10 energy** to start".
- `systems/combat.md > AoEDamage` (shares, effective armour, the minimum 0.5 × the overlap fraction).

**Placeholders**
- **D18:** the letter and grouping.
- **D19:**
  - what is under the cloud: the centre inside a circle;
  - connection: when circles touch;
  - the release's direction: from the nearest member;
  - underground objects: not affected;
  - the clock: global;
  - who sees the cloud: whoever sees the Disciple;
  - no regeneration while channelling ("40 s from full").

**Code** (T76, T77)
- **The order:** `OrderKind::Shroud`.
  - Its behaviour holds the Disciple Stationary and drains energy each tick.
  - It ends when the energy can't pay a tick.
  - It is refused below 10 energy (`Reject::NotEnoughEnergy`).
  - Any new order ends it, by the order rules.
- **The clouds:**
  - `Sim::clouds: Vec<Cloud { members, buffer }>`, hashed only while not empty;
  - recomputed after 5a, with union-find over the channelling Disciples in id order;
  - matched to the last tick's clouds for merges, splits and ends (T77).
- **Absorption.** A hook in the hit application (`combat/damage.rs`) handles single hits and area shares alike.
- **Releases** run in 5e, on the frames where the tick is a multiple of 8. Their areas are lattice counts (T77).
- **`SimEvent`s:** ShroudStarted, ShroudEnded and ShroudReleased, with their `viewer_events` rows.
- **The client:**
  - clouds drawn as translucent discs, merged where they overlap, for every viewer who sees the Disciple;
  - the button live, by D18's placeholder letter;
  - a release flash.

**Acceptance**
- **A lone unit.** A lone Militant under one cloud, shot once for 4 from outside, takes 0.5 at the next release, the
  area minimum, instead of 4. The rest is lost.
- **A building.** A 3x3 building wholly under a cloud takes about 32% of the absorbed damage, before armour.
- **Inside the cloud.** Units fighting inside the same cloud deal full damage. Air units are never absorbed or hit.
- **Connection.** Two Disciples 5 GU apart share one buffer; a third 8 GU away doesn't.
- **Splitting.** Stop on one of a joined pair releases the shared buffer at once over both circles.
- **Ending.**
  - Taking damage doesn't end the channel.
  - Moving does, and so does running dry: about 40 s from full.
  - It can't start below 10 energy.
- **Determinism.** A replay with clouds splitting and merging reproduces the hash.
- **Perf:** `shroud_perf` (8 Disciples, 60 units under fire), mean and p99.
- **A new golden,** `cults_shroud`.
- **QA screenshots:** a single cloud, and joined clouds.

**Owns:** see §C. **After:** N4a. **Model:** Opus, about 3 h. The method is fixed in T77, so it isn't research.

#### N5: The Shrine (Opus)

**Goal.**
- Recruits inside charge the Shrine's energy.
- It spends the energy on two area buffs, cast at a point in vision.

**Design** (`factions/cults/structures/shrine.md`)
- "Recruits enter the Shrine to take part in the ritual ... Only Recruits can participate. Participating Recruits are
  not consumed. Evacuation on Destruction applies."
- "**Energy regeneration**: proportional to the number of Recruits inside ... With no Recruits inside, the Shrine does
  not regenerate energy."
- "Both abilities are cast on an area anywhere the player currently has vision, and spend Shrine energy."
- "The effect is a **buff applied to units at the moment of casting**: qualifying units inside the area when the
  ability is cast receive the buff for its duration and keep it even if they leave the area. Units that enter the
  area afterward do not receive it. Only the caster's own units are affected."
- The table's "Affects": "Own Cults infantry"; "Own Disciples".
- The Placeholder stats:
  - "Size 2x2; SymmetryType AAAA; MaxHP 300";
  - "Recruit capacity 8. MaxEnergy 200; starts at 0; regeneration **0.25 energy per second per Recruit inside**";
  - "**Health regeneration** (hotkey Q): 75 energy; circle of radius 4 grid units; **+2 HP per second for 15
    seconds**";
  - "**Energy regeneration** (hotkey W): 50 energy; circle of radius 4 grid units; affected Disciples' energy
    regeneration is **+1 per second for 20 seconds**";
  - "No cooldowns. Targeting: AwaitingTarget, left-click a point in vision. **A: Eject All**".
- **`factions/cults/structures/README.md > Cults Infantry`:** "Cults infantry have no natural health regeneration.
  Their only source of healing is the Shrine's health regeneration ability."

**Placeholders**
- **D22:** the side.
- **D24:**
  - who is "inside": a unit whose centre is in the circle;
  - a recast refreshes the duration, and the rates don't add;
  - one press casts once, from the selected Shrine with the most energy.

**Code** (T75, T76)
- **The Shrine:**
  - `holds`: Recruits, capacity 8, the nearest side;
  - `ObjectDef::energy` (N4a's field), with a regeneration per Recruit inside;
  - `StructureDef::abilities`: the two casts' costs, radii, effects and durations.
- **The casts:**
  - `Command::Cast { casters, ability, at }`;
  - the gate checks that `at` is in the owner's vision (as the last tick left it), that the Shrine is operational,
    and its energy;
  - one Shrine casts (D24).
- **Buffs:** `Object::buffs`, hashed only while not empty. `buffs::step` runs beside `shields::step`:
  - HP regeneration of 0.125 a tick up to `max_hp()`;
  - the Disciples' bonus regeneration, read by `energy::step`.
- **`SimEvent::Cast`,** with its `viewer_events` row.
- **The client:**
  - `templates/cults/shrine.rs`: Q, W and A, with the energy in the InfoPanel;
  - the AwaitingTarget cursor, with a 4 GU ring; a click outside vision gives "invalid target" feedback;
  - a buff marker on affected units;
  - right-click Enter Shrine for Recruits;
  - the menu entry goes live, and `builds` gains the Shrine.

**Acceptance**
- **Energy.** With 8 Recruits inside, 2 energy a second: the first Q is possible after 37.5 s, and 200 is reached
  after 100 s. With none inside, it doesn't regenerate.
- **Health regeneration.**
  - Q heals each own Cults infantry unit inside the circle by 30 HP over 15 s, capped at MaxHP.
  - It doesn't heal enemies, other factions' units, or units that arrive after the cast.
  - A unit that leaves the circle keeps the buff.
- **Energy regeneration.** W adds 1 energy a second to Disciples for 20 s.
- **Refusals.** A point outside vision is refused, and too little energy greys the command.
- **Destruction** evacuates the Recruits, and the energy is lost.
- **QA screenshots:** the cast cursor with its ring, the InfoPanel's energy, and a buffed unit.

**Owns:** see §C. **After:** N4a, and N2 (the nearest-side entrance, D22). **Model:** Opus, about 2.5 h.

#### N6: The Chop Shop and the Jackal (Opus)

**Goal.**
- The Chop Shop turns two pooled Recruits into a Jackal.
- The Jackal drives on V2's movement, fires its turret on the move (U1), and takes directional damage.

**Design**
- **`factions/cults/structures/chop_shop.md`:**
  - "**Production works like the Armory** ... queuing an order deducts its Space Crystals cost and **reserves the
    vehicle's number of Recruits** from the pool (an order can only be queued if enough unreserved Recruits are
    inside); when it finishes, they are consumed and the vehicle exits to the RallyPoint. Eject All and Cancel work as
    at the Armory (cancel returns the reserved Recruits to the pool).";
  - "Size 3x3; SymmetryType ABAC; EntranceSide B, ExitSide C (like the Armory)"; "InternalRecruitCapacity 10;
    production queue 10 orders";
  - "Panel like the Armory's: **Q: Build Jackal**, **A: Eject All**, **C: Set Rally Point**, **X: Cancel
    Production**".
- **`factions/cults/units/jackal.md`:**
  - "a wheeled ATV/buggy ... with a **machine-gun turret** that fires **low damage at a high rate**, **cannot miss**,
    and **rotates 360°**";
  - "TargetDomain: **Universal** (ground and air)";
  - "consumes **2 Recruits**; Unit Control cost **2**";
  - "Attack: Damage 3; Range 4 grid units; MinRange 0; Aim 1, Firing 1, Cooldown 1, Reload 1 frames";
  - "ObjectInterfaceState: BasicCombatUnitInterfaceState".
- **`factions/cults/economy.md > Unit Control Tracking`:** "A trained unit's Unit Control cost equals the number of
  Recruits consumed to produce it, with each point of cost attributed to the originating hall of the respective
  Recruit."
- **`factions/cults/augmentation.md > Persistence > Vehicles`** (quoted in N1a).
- **`systems/combat.md > DirectionalArmor`.** It is built (`combat/damage.rs::directional_multiplier`), and the
  WheeledVehicle base has it.
- **0319's verification:** "The Chop Shop needs 2 free Recruits inside to queue a Jackal."

**Placeholders**
- **D25:** vehicle yielding (V2's).
- **D27:**
  - the two Recruits inside the longest are reserved;
  - a Jackal comes out at full HP.

**Code** (T71)
- **The Chop Shop:**
  - `holds` (B, C, Recruits, 10);
  - `production`: the Jackal's recipe, whose reservation size is the unit's `unit_control_cost` (2), with
    `queue_limit` 10.
- **Completion:**
  - `change_kind` on the first reserved Recruit to a Jackal;
  - the second is removed, and its lineage entries are appended;
  - the augmentation is cleared, and HP set to full.
- **The doorway.** The Jackal appears half inside side C, facing out, and drives straight out (V2).
- **The client:**
  - `templates/cults/chop_shop.rs`;
  - the Jackal's oriented body with its barrel (V1, U1), and its panel with W Reverse (V2);
  - right-click Enter Chop Shop for Recruits;
  - the menu entry goes live, and `builds` gains the Chop Shop.

**Acceptance**
- **0319's verification:** with one free Recruit, Q is refused (`Refusal::NoRecruit`); with two, it is accepted.
- **Production.** 75 SC and two Recruits make a Jackal, which leaves through C after 320 frames.
- **Unit Control.** It counts 1 against each Recruit's Hall. Losing one Hall makes half of it an orphan.
- **Cancel** returns both Recruits and the 75 SC.
- **Augmentations** on the Recruits are lost.
- **Combat.**
  - A Jackal kills a Militant while driving past.
  - It hits a test glider (Universal).
  - Rear hits do more than frontal ones.
- **Terrain.** It can't enter rugged tiles.
- **Repair (N8),** now with the Jackal's recipe: one Recruit repairs a Jackal from 60 to 120 HP in 80 frames, and two
  in 40. Either way it costs 19 SC, one per 3.2 HP (half its 75 SC over its 120 HP, in whole crystals).
- **A new golden,** `cults_vehicles`: Jackals against Guards and Militants.
- **QA screenshots:** the Chop Shop's panel, and a Jackal turning with its turret on a target.

**Owns:** see §C. **After:** V2, U1 and G2. **Model:** Opus, about 2 h.

#### N7: The Airstrip and the Sparrow (Opus)

**Goal.**
- The Airstrip turns one pooled Recruit into a Sparrow, which takes off along the runway and never lands.
- The Sparrow strafes ground and air.

**Design**
- **`factions/cults/structures/airstrip.md`:**
  - "A finished aircraft **takes off from the Airstrip** to its RallyPoint; Cults aircraft (so far) never land
    back.";
  - "Size 2x4 (a runway); SymmetryType ABAC with the short sides B and C; EntranceSide B (Recruits walk in); aircraft
    take off along the runway toward C";
  - "InternalRecruitCapacity 10; production queue 10 orders"; "**Sparrow**: 1 Recruit, 75 Space Crystals, 320 frames";
  - "Panel like the Armory's: **Q: Build Sparrow**, **A: Eject All**, **C: Set Rally Point**, **X: Cancel
    Production**".
- **`factions/cults/units/sparrow.md`:**
  - "UnitBase - Glider (must keep moving; circles when idle ...)";
  - "On completion it **takes off from the Airstrip** and **never lands** again";
  - "TurnAngle: **45° total cone** (22.5° to each side of its facing)";
  - "**Unlimited ammo** ... it never returns to re-arm";
  - "Attack: Damage 3; Range 3 grid units; MinRange 0; Aim 1, Firing 1, Cooldown 1, Reload 1 frames".
- **`core/entities.md > Exit Doorway`:** "Aircraft and units released underground ... are not covered by this rule."
  And "Size (Height x Width) is also given for the default orientation": 2 tall, 4 wide, with B on the right and C on
  the left.

**Placeholders**
- **D26:** takeoff, a Sparrow with no rally, and its flight.
- **D27:** the Sparrow at full HP, with augmentations lost (it isn't infantry).
- **D28:** the Sparrow in the playtest.

**Code** (T71, T81)
- **The Airstrip:**
  - `holds` (entrance B, Recruits, 10);
  - `production`: the Sparrow's recipe, `queue_limit` 10;
  - the release is `ExitRule::Runway { from: B, toward: C }`, which is new. The Sparrow appears over the B end,
    facing C, at IdleSpeed, and then heads for the rally or circles. It never pauses.
- **The Sparrow** runs on H1's glider and U1's turret.
- **The client:**
  - `templates/cults/airstrip.rs`;
  - the runway's look;
  - the Sparrow's raised look with its shadow;
  - right-click Enter Airstrip for Recruits;
  - the menu entry goes live, and `builds` gains the Airstrip.

**Acceptance**
- **Takeoff.** A Sparrow appears over the B end, flies along the runway toward C, then goes to its rally. With no
  rally, it circles past C (D26).
- **It never lands,** and it strafes a group of Agents.
- **Who can hit it.** Militants, Guards, the Laser Turret and Jackals hit it. Peacekeepers, Legionnaires and Agents
  can't (their TargetDomain is Ground).
- **Destruction** of the Airstrip evacuates its pool and leaves airborne Sparrows alone.
- **A new golden,** `cults_air`: Sparrows against Militants and Guards.
- **QA screenshots:** a takeoff, and a strafing pass.

**Owns:** see §C. **After:** H1, U1 and G2. **Model:** Opus, about 2 h.

#### N8: Repair, for every repairer (Opus)

**Goal.** Damaged own buildings and vehicles get repaired, by one mechanism for every repairer:
- Recruits repair own buildings and vehicles, for Space Crystals;
- Agents repair own Tunnels for free, and own Laser Turrets for Space Crystals;
- several repairers on one target repair faster, linearly, as Cults builders build;
- the Field Engineer gets the same rules as data once it is built. **Nothing of it is in the code:** there is no
  Field Engineer kind and no repair (`grep -ri repair crates/` finds nothing; the GDO's only unit is the
  Peacekeeper). Its doc still says "TBD: stats, cost, build time, hotkeys, ObjectInterfaceState" (D31).

**Design**
- **0321,** and the Repair sections it adds:
  - `factions/cults/units/recruit.md > Repair`: "Recruits can **repair** own **buildings and vehicles**. Repair
    **costs Space Crystals**.";
  - `factions/syndicate/units/agent.md > Repair`: "Agents can **repair** own **Tunnels** and **Laser Turrets**.
    **Laser Turrets cost Space Crystals to repair; Tunnels are repaired for free**"; "How an Agent reaches a pop-up
    turret or a Tunnel (side, reach) follows the Recruit/Field Engineer repair flow; details: Director's discretion"
    (T83);
  - `factions/gdo/units/field_engineer.md > Abilities`: "**Repair**: repairs buildings and vehicles. Repair costs
    Space Crystals (not free).";
  - all three: "several repairers on one target repair **faster, scaling linearly** (2 repairers = twice the rate),
    as construction does. **Placeholder rate and cost** (designer, 2026-11-26): repairing from 0 to full HP takes
    **half the target's build time** with one repairer, and costs **half its price** in the resource it was bought
    with, paid gradually as HP is restored".
- **0322,** and `factions/cults/units/recruit.md > Recruit ObjectInterfaceState`:
  - "**D: Repair** — targeted command (StateOnlyTransition). Left-click a damaged own building or vehicle to repair
    it";
  - `> Right-Click Resolution`: "Damaged own building or vehicle (completed) → Repair";
  - 0322's open point: "right-clicking a damaged building that can also be entered (Armory, Tunnel, …) is
    unresolved ... Until then, keep **Enter** for enterable buildings".
- **`systems/controls/selection_and_hud.md > PointerDisplayType`:**
  - "Repair — Unit will repair this damaged own building or vehicle (Recruits, Agents, Field Engineers ...).
    Placeholder look.";
  - DefaultState: "Cursor over damaged own building or vehicle that the selection can repair: **Repair**";
  - "AwaitingTarget[Repair] over a damaged own building or vehicle the selection can repair: **Repair**; elsewhere:
    **Inactive**".
- **"As construction does":** `factions/cults/economy.md > BaseConstructionTime Scaling`, E's rule: "2 Recruits halve
  it, 3 Recruits reduce it to a third".

**What 0321's placeholders give** (one repairer, from 0 to full):

| Target | MaxHP | Build time | Price | Repair time | Repair cost |
|---|---|---|---|---|---|
| Storage | 250 | 320 f | 100 SC | 160 f | 50 SC |
| Armory | 300 | 480 f | 200 SC | 240 f | 100 SC |
| Recruitment Hall | 1000 | 1440 f | 300 SC | 720 f | 150 SC |
| Jackal | 120 | 320 f | 75 SC and 2 Recruits | 160 f | 37.5 SC, paid as 38 (D30) |
| Laser Turret | 300 | 320 f | 100 SC | 160 f | 50 SC |
| Tunnel, Tiers 1–3 | 600–1000 | 480 f | Supplies | 240 f at every tier (D30) | free |

G2's buildings follow from 0319's numbers in the same way.

**Placeholders**
- 0321's rate and cost are the design's own placeholders, cited in the code.
- **D30:**
  - the order ends at full HP;
  - with an empty bank, the repairers wait, and go on when crystals come in;
  - the Agent's key is F, since its D is Build Laser Turret;
  - aircraft (the Sparrow) aren't repaired;
  - nor are the Tunnel's expansions (Hideout, Chemical Plant, Optics Lab);
  - a Tunnel's build time is its construction's 480 frames, at every tier;
  - the Jackal's price is its crystals.
- **0322's open point:** a right-click on a damaged building the unit can enter stays Enter, and the panel command
  repairs it. It is being put to the user, so it has no draft.
- **D31:** no Field Engineer in M5.

**Code** (T83)
- **Data:**
  - `WorkerDef::repairs: Option<RepairDef>`, at the end of `WorkerDef`:
    - the Recruit: `OwnBuildingsAndVehicles`;
    - the Agent: `Kinds(&[(TUNNEL, Free), (LASER_TURRET, Paid)])`;
    - the Drone: `None`.
  - A target's terms are derived from its build data, halved by two tunables in a new `tuning/repair.rs`: a
    structure's `SiteDef` (`frames`, `SitePrice::Crystals`), or the `Recipe` that produces a unit. A kind with neither
    can't be repaired.
- **The order:** `OrderKind::Repair(ObjectId)`, in a new `orders/repair.rs`.
  - **The gate:** an own, completed, damaged target that the repairer's `repairs` covers, with known terms. Otherwise
    `Reject::CantRepair`.
  - **A building:** the repairer claims its own spot within reach of the footprint's edge, as E's Assist does (T62's
    first ring). That is any side, for a Tunnel and a lowered turret alike.
  - **A vehicle:** the repairer closes to reach as a melee attacker does, and follows it.
  - **The end:** at full HP, or when the target dies or changes hands. Shift-queued orders go on.
- **The step** (a new `repair.rs`, after construction's `step_sites`):
  - the repairers are derived each tick, as E derives builders: the units on `Repair(target)` within reach;
  - HP rises by n × MaxHP / repair_frames, capped at `max_hp()`;
  - **payment** is in whole crystals, with no stored state. Each crystal buys k = MaxHP / repair_price HP. HP rises
    free to the top of its current k-step, and each further step costs a crystal. With an empty bank, HP stops at
    the step's top;
  - targets are taken in id order, so a short bank serves the lowest id first.
- **`Sim::can_repair(unit, target)`** serves the gate, the client's pointer and the right-click.
- **The client:**
  - the Recruit's D, J1's `Planned` slot, goes live, with AwaitingTarget[Repair];
  - the Agent gets F: Repair (D30);
  - `PointerDisplay::Repair`, in DefaultState and in AwaitingTarget[Repair];
  - the right-click resolution: Repair on a damaged, completed own building or vehicle the unit can repair, unless
    the unit can enter it (0322). A mixed selection gets the Mixed pointer, as now.
- **Test defs:** a test vehicle with a test recipe. The Jackal joins once N6 adds its recipe, and N6's acceptance
  checks it.

**Acceptance**
- **0321's verifications:**
  - two Recruits repair an Armory from 150 to 300 HP in 60 frames, and one Recruit in 120;
  - the crystals drop as the HP rises: one per 3 HP, 50 in all;
  - an Agent repairing a Tunnel spends nothing.
- **Linear stacking:** 1, 2, 3 and 4 Recruits repair at 1, 2, 3 and 4 times one Recruit's rate.
- **The Laser Turret:** an Agent repairs one from 150 to 300 HP in 80 frames for 25 SC, raised or lowered, and the
  turret doesn't rise for it.
- **A vehicle:** two Recruits repair the test vehicle, and follow it when it drives off.
- **An empty bank:** the repair stops at its step's top and goes on when crystals come in. The bank never goes below
  0.
- **Refused:** a site under construction, a target at full HP, an enemy's target, an aircraft, and a Hideout for an
  Agent.
- **The end:** at full HP, the repairers take their Shift-queued orders.
- **The client:**
  - the Recruit's panel shows Q, A, S, D and X, with D live;
  - the Repair pointer shows over a damaged Storage, and AwaitingTarget[Repair] over a full one is Inactive;
  - a right-click on a damaged Storage repairs it, and on a damaged Armory enters it (0322).
- **Goldens:** unchanged. The only new hashed state is the order tag, while a unit carries it.
- **Perf:** `structures_perf` A/B, with 40 repairers on 10 targets.
- **QA screenshots:** the Recruit's panel with D, and the Repair pointer.

**Owns:** see §C. **After:** E (Assist's spots and builder count, which N8 reuses for the linear stacking), and J1
(the Recruit's panel and right-click resolution). **Model:** Opus, about 2.5 h.

---

## §C Waves

```
W1  A | R                                             (merged)
W2  C | D (Fable, merged) | F1; O3 pulled forward     (C, F1 and O3 running)
W3  B | E | F2
W4  J1 | L | U1 (after O3, in its old slot)           -> first playable after J1 (unchanged)
W5  G2 (after E, F2, J1) | V1 (Fable) | N8 (after E, J1)
W6  N1a (after G2) | N2 (after G2) | V2 (Fable, after V1)
W7  N1b (after N1a) | N4a (after G2) | H1 (Fable, after U1)
W8  N3 (after N1b, N2) | N4b (after N4a) | N6 (after V2, U1)
W9  N5 (after N4a, N2) | N7 (after H1) | S1 (answers)
W10 X1 (alone; after every content slice)             -> the ruleset; goldens byte-identical
W11 X2 (after X1) | S2 (late answers)
W12 M (after everything)                              -> the user's first playtest
```

- **Repair's wave (N8, 0321): W5.**
  - **It needs E first,** for its linear stacking. It reuses E's spot claim and E's per-tick count of the units on
    an order within reach, which make "2 repairers = twice the rate" exact. It also needs J1, whose Recruit panel
    holds the D slot and whose right-click resolution it extends. Both have merged by W5.
  - **W5 has room** because U1, which needs nothing new, moves into W4's free slot.
  - **Its registries are free in W5:** no other slice adds an `OrderKind`, and its field is on `WorkerDef`, which
    G2's three new unit literals don't carry.
  - **It touches all three implemented factions** (the Recruit, the Agent, and the Field Engineer as data), but only
    through data, one order and one step. No faction's own systems change.
- **0323 and 0324 (Q0111's first step).** As the Director asked, there's no slice for them here, but they collide
  with this plan:
  - **The file.** 0323's 90° sidestep replaces the 45° one in `movement/collide.rs`'s `sidestep` (477 lines), which
    V1 splits and changes in W5. The two mustn't share a wave. Whichever comes first splits the file, and V1 is
    better second.
  - **The goldens.** It changes every infantry unit's contact moves, so the goldens re-bless with it, L's too if L
    has merged.
  - **The slot.** It is Fable work. If it takes W4's free slot, U1 moves to W5, N8 to W9 in S1's place, and S1 merges
    into S2 (W11). The registries still fit, and the playtest stays at the end of W12.
  - **For V2,** 0323 settles D25's first point: "WheeledVehicles turn and graze around the obstacle instead".
- **The critical path:**
  - **Capacity.** It is the binding constraint: the 15 new slices and S1 fill W4's free slot and W5–W9, at three
    builds a wave.
  - **The chains:**
    - J1 → G2 → N1a → N1b → N3, and G2 → N4a → {N4b, N5};
    - V1 → V2 → N6, and U1 → H1 → N7.
    - They end in W8 or W9.
  - **Then:** X1 → X2 → M.
  - **Slack.** N6 and N8 have no successor, so either can trade waves with a slice whose registries differ (check
    the table below). H1 could run in W5 or W6, but for the one-Fable rule.
- **The first playable** stays at the end of W4. Each later wave adds playable content:
  - W5: the Gunworks and the Gunner, and Repair;
  - W6: the Garrison;
  - W7: the Altar, the Cathedral and the Disciple;
  - W8: the Temple, Shroud and the Jackal;
  - W9: the Shrine and the Sparrow.
- **The playtest moves from the end of W8 to the end of W12.** §F gives an earlier option.
- **Machine load:**
  - three builds in every wave from W4 to W9;
  - one Fable build in each of W5, W6 and W7 (V1, V2, H1), and none in W4, W8 or W9.
  - Q0110's answer (0320) removed the fix that would have needed one.
- **Answers.** One arriving before a slice launches goes into its brief. Later ones go to S1 (W9) or S2 (W11).
- **Shared registries.** These amend §3.2. Edits stay additive, and no two slices of a wave touch the same item:

  | Registry | W4 | W5 | W6 | W7 | W8 | W9 |
  |---|---|---|---|---|---|---|
  | Kind registry (`defs.rs`, `kinds::of`) | — | G2 (13 kinds; then closed for M5) | — | — | — | — |
  | `OrderKind` | — | N8 `Repair` | V2 `Reverse` | — | N4b `Shroud` | — |
  | Commands | — | — | N1a `DebugCommand::SetAugmentation` | N1b `ProductionCommand::Augment` | N3 `ProductionCommand::Research` | N5 `Command::Cast` |
  | `Object` / `AttackState` fields | U1 `AttackState::turret` | — | N1a `augmentation` | N4a `energy` | — | N5 `buffs` |
  | `UnitDef` field | — | — | N1a `augmentable` | H1 `separation_radius` | N4b `shroud` | — |
  | `WorkerDef` field | — | N8 `repairs` | — | — | — | — |
  | `StructureDef` / `ProductionDef` field | — | — | N2 `occupants_fire` | N1b `ProductionDef::augment` | N3 `research` | N5 `abilities` |
  | `ObjectDef` field | — | — | — | N4a `energy` | — | — |
  | `ExitRule` | — | — | N2 `AnySide` | — | — | N7 `Runway` |
  | `QueueEntry` (`production.rs`) | — | — | — | N1b (new) | N3 `Research` | — |
  | Client `PointerDisplay` | J1 `Assist` | N8 `Repair` | — | — | — | — |

  - **Def fields.** A new def field goes at the end of its struct's literal, in every kind module and test def. The
    ends of `UnitDef`, `StructureDef` and `ObjectDef` are at least two unchanged lines apart (`}),` and the
    `structure`/`default_settings` lines), so W7's N4a and H1 merge cleanly, as E and F2 do in W3.
  - **W5's shared file,** the Recruit's def: G2 changes only the `builds` list in its `WorkerDef`, and N8 adds
    `repairs` at the end, after E's `shares_builds`.
  - **Open to every slice** (m5-common), with the slice that merges second resolving the one-line conflict:
    - `SimEvent` variants and their `viewer_events` rows;
    - the client's `PanelCommand` variants: J1's Assist, N8's Repair, then N2's, N1b's, N3's, N4b's and N5's.
  - **The tuning modules:**
    - U1: `tuning/combat.rs` (the scan's 45° step);
    - V1 and V2: `tuning/movement.rs` (bisection steps; vehicle steering), in different waves;
    - N8: a new `tuning/repair.rs` (the two halves);
    - H1: a new `tuning/air.rs`;
    - N4a: a new `tuning/energy.rs`;
    - N4b: a new `tuning/spells.rs` (the release period and the lattice step).
- **Split first** (T28). This continues §3.2's list:
  - `orders/world.rs` (474 lines): **U1** (W4);
  - `combat.rs` (456): **U1**, if it adds more than about 25 lines there;
  - `movement/collide.rs` (477): **V1** (W5);
  - `movement.rs` (447): **V2** (W6);
  - `vision.rs` (471): **H1** (W7), if it adds there;
  - the client's `command_ui/interface/commands.rs` (467): **N1b** (W7), which adds the most there. J1, N8 and N2
    each add a variant to it before that, unsplit, which keeps it under 490 lines.

**What each new slice owns.** The shared registries above are open on their terms.

| Slice | Owns | Read-only |
|---|---|---|
| U1 | new `combat/turret.rs`; `AttackState::turret`; the turret scan in `orders/world/` (split first); the LockedTarget arms in `orders/engage.rs` and the behaviours; `tuning/combat.rs` additions; client `object_render/turret.rs`; new `tests/turrets/` | `combat/structures.rs`, O3's phases |
| G2 | new `defs/cults/structures/{gunworks,garrison,chop_shop,machine_shop,airstrip,scrap_hangar,altar,temple,cathedral,shrine}.rs`, `defs/cults/units/{disciple,jackal,sparrow}.rs` (base fields, `site`, `requires`) and their 13 registry lines; their 13 `kinds/cults/*.rs` and `kinds::of` lines; the Recruit's `builds` entries for the three simple buildings; the Armory's Gunner recipe; `CONSTRUCT_MENU`'s three entries; W in `templates/cults/armory.rs`; `tests/building_armor.rs`'s Cults rows; the Storage's armour | `tech.rs`; every system |
| V1 | `shape.rs`; `movement/collide.rs` (split into `movement/collide/`); the oriented arms of `movement/space.rs`; `docs/movement.md` G6 and §3; client `object_render/units.rs` (the rotated box); new `tests/oriented/`, `tests/vehicles_perf.rs` | `movement/drag.rs`, `movement/yielding.rs` |
| N8 | new `repair.rs`, `orders/repair.rs` and `tuning/repair.rs`; `OrderKind::Repair` and its gate arm; `WorkerDef::repairs` and its lines in the Recruit's, Agent's and Drone's defs; `Sim::can_repair`; E's builder count and spot claim made generic over the order, if E left them on Assist; client `PanelCommand::Repair`, `PointerDisplay::Repair`, the Repair arms of `command_ui/resolution.rs`, the Recruit's D (`templates/cults/recruit.rs`) and the Agent's F (`templates/syndicate/agent.rs`); new `tests/repair/` | `construction/site.rs` (E's), the Tunnel's and Laser Turret's defs |
| V2 | new `movement/wheeled.rs`; `movement.rs` (split first) and `Mover::turn_rate`; the vehicle arms of `movement/{yielding,stuck,follow}.rs`; `OrderKind::Reverse`, new `orders/reverse.rs` and its gate arm; the client's Reverse made live (`command_ui/interface/panel.rs`'s `offers`, `command_ui/resolution.rs`); new `tests/vehicles/` | `shape.rs`, `movement/collide/` (V1's) |
| N1a | new `augmentation.rs`, `defs/cults/augmentation.rs`; `UnitDef::augmentable`; `Object::augmentation` and its hash arm; `Object::max_speed` and its readers; `combat::weapon`'s signature and its callers; `DebugCommand::SetAugmentation`; new `tests/augmentation/` | `transform.rs` |
| N2 | the Garrison's `holds`; `StructureDef::occupants_fire`; new `combat/occupants.rs` and its call in `combat::step`; `ExitRule::AnySide` and its arm in `sim/holding.rs`; the AAAA entrance rule (D22) in `holding.rs`; client new `templates/cults/garrison.rs` and its panel command; the Garrison's menu entry and `builds` line; new `tests/garrison/` | `combat/structures.rs` |
| H1 | new `movement/glider.rs`, `movement/air.rs`, `tuning/air.rs`; the glider arms in `orders/`; `Object::map_pos` and the edge clamp (`vision.rs` split first if touched); `UnitDef::separation_radius`; client air look; new `tests/air/`, `tests/air_perf.rs` | `combat/turret.rs` (U1's) |
| N1b | the Altar's `holds` and `production`; `ProductionDef::augment`; `QueueEntry` in `production.rs`; the Augment arms in `sim/producers/`; `ProductionCommand::Augment` and its gate arm; `Sim::augment_level` (a stub); client `command_ui/interface/commands.rs` (split first), new `templates/cults/altar.rs`; the Altar's menu entry and `builds` line; new `tests/altar/` | `holding.rs`, `production/exit/` |
| N4a | the Cathedral's `holds` and `production`; `ObjectDef::energy`; `Object::energy`; new `energy.rs` and `tuning/energy.rs`; the Disciple's energy; client energy bar, new `templates/cults/{cathedral,disciple}.rs`; the Cathedral's menu entry and `builds` line; new `tests/cathedral/`, `tests/energy/` | `production.rs` |
| N3 | new `defs/cults/research.rs`; the players' `techs`; `StructureDef::research`; the Temple's `holds` and `production`; `QueueEntry::Research`; `ProductionCommand::Research`; `Sim::augment_level` (filled); client new `templates/cults/temple.rs`; the Temple's menu entry and `builds` line; new `tests/research/` | `tech.rs` |
| N4b | `OrderKind::Shroud`, new `orders/shroud.rs` and its gate arm; new `shroud.rs` (clouds, matching, releases) and `Sim::clouds`; `UnitDef::shroud`; the absorption hook in `combat/damage.rs`; new `tuning/spells.rs`; the Shroud events; client cloud rendering; new `tests/shroud/`, `tests/shroud_perf.rs` | `combat/area.rs` |
| N6 | the Chop Shop's `holds` and `production`; any vehicle arm in F2's completion; client new `templates/cults/chop_shop.rs`, the Jackal's look; the menu entry and `builds` line; new `tests/chop_shop/` | `movement/` (V1's and V2's), `combat/turret.rs`, `repair.rs` (N8's) |
| N5 | the Shrine's `holds` and `energy`; `StructureDef::abilities`; `Command::Cast` and its gate arm; new `buffs.rs` and `Object::buffs`; `SimEvent::Cast`; client new `templates/cults/shrine.rs`, the cast cursor; the menu entry and `builds` line; new `tests/shrine/` | `energy.rs` (N4a's) |
| N7 | the Airstrip's `holds` and `production`; `ExitRule::Runway` and its release arm in `production/exit/`; client new `templates/cults/airstrip.rs`, the runway and Sparrow looks; the menu entry and `builds` line; new `tests/airstrip/` | `movement/glider.rs`, `movement/air.rs` (H1's) |

**Briefs not yet written that 0319–0322 change:**
- **B (W3):** unchanged. The editor's palette lists G2's kinds by itself, since it is data-driven.
- **E (W3):**
  - Build `site` and the Recruit's `builds` as data that G2 and the N-slices extend. E covers A's six kinds.
  - The builders' spots (T62) must work for every footprint, including the 2x3 and 2x4 oblong ones (ABAC) rotated.
  - **For N8 (0321):** write the per-tick count of the units within reach, and the spot claim, over an order and its
    target, not for Assist alone. N8's Repair reuses both for its linear stacking.
- **F2 (W3):**
  - **Reservations per queue entry:** `reserved: Vec<Vec<ObjectId>>`, sized by the produced unit's
    `unit_control_cost` (1 now; 2 for the Jackal) (T70).
  - **`ProductionDef::queue_limit` per def,** with `QUEUE_LIMIT` (20) as the default.
  - **Completion consumes n held units:** `change_kind` on the first; the others are removed and their lineage
    entries appended (T56 as amended).
  - **The pool path is generic** over `holds` and `production`, not the Armory's own arms. The Cathedral, Chop Shop,
    Airstrip and Altar then reuse it as data (T71).
  - **`Refusal::NoRecruit`** means "fewer free Recruits than the order needs".
- **J1 (W4):**
  - **The Recruit's panel (0322):** Q Move, A Construct, S Assist Construction (no longer the off-grid B), D Repair,
    X Stop. D is `Planned`, greyed "Not in the build yet", until N8 makes it live in W5.
  - **The pointers (0322):** the Assist pointer is J1's: "Cursor over own building under construction (selection can
    assist construction): **Assist**", and in AwaitingTarget[AssistConstruction], "**Assist**; elsewhere:
    **Inactive**". The Repair pointer and right-click Repair are N8's.
  - **`CONSTRUCT_MENU` holds all 13 buildings** in fixed slots (Q0096). The ten new ones are `Planned`, greyed "Not in
    the build yet", until G2 and the N-slices make them live.
  - **Right-click on an own Gunworks, Machine Shop or Scrap Hangar is a Move** (0319, Q0093 a), not Enter.
    recruit.md's line is stale (D29).
  - **Enter's resolution and pointer are generic:** Enter for "an own building that holds units, is operational and
    admits the selected kind" (F1's `admits`). The N-slices then add no resolution code, only their kinds in
    `accepts`.
  - **Enter's feedback** names "the structure being entered" (Q0108.1).
- **L (W4):**
  - **The trip.** The oracle expects M5-D's 24-frame Recruit trip, not 22: 1.33, 0.57 and 0.36 SC/s at gaps of 1, 2
    and 3 (−8%, −4% and −2% from the design's figures, within ±15%).
  - **Layouts (0320).** "The reference layouts and the Cults economy check should stay within the design's 4–5
    Recruits per face." M5-D's two ignored jam tests stay ignored, as designed behaviour ("Overloading a face can
    block Recruits head-on for a long time").
  - **Goldens.** L's goldens stay with the infantry core. G2 adds Gunners to `cults_vs_syndicate`, and each N-slice
    adds a golden of its own instead of growing L's.
- **S1 (now W9):** the Storage's armour moves to G2. S1 keeps Q0108.1's wording in code docs and the answers for
  merged slices, and changes the citation of the two ignored tests in `tests/recruit_gathering/jam.rs` to 0320.
- **X1 (now W10):**
  - **More tuning modules:** holding, energy, buffs, spells, air, vehicles, turrets, research and repair.
  - **The faction rules** gain the augmentation increments and effects table, and the techs' table.
  - **`Ruleset::structural_diff`** also covers:
    - `holds` (sides, `accepts`, exit rules) and `occupants_fire`'s kinds;
    - queue-entry kinds and techs;
    - the presence of `energy`, `abilities`, `shroud` and `repairs`;
    - non-square silhouettes and movement-model variants.
  - **A/B perf** adds `vehicles_perf`, `air_perf` and `shroud_perf`.
  - If X1 passes about 3.5 h, it splits into X1a (the tuning modules) and X1b (defs and faction rules), each alone,
    back to back.
- **X2 (now W11):** the export covers the new def fields and tuning modules. The client's new feel tunables (clouds,
  energy bars, the air shadow's offset) go in `client/*.json`.
- **M (now W12):**
  - **The walkthrough** goes up both branches of the tech tree, with a Jackal, a Sparrow, Shroud, both Shrine
    abilities, and Recruits and an Agent repairing.
  - **Perf** adds `vehicles_perf`, `air_perf` and `shroud_perf`.
  - **New soak invariants:**
    - reserved units are held in their building;
    - energy stays within [0, max];
    - every cloud member is channelling;
    - air units stay within the overshoot bound;
    - only augmentable kinds carry an augmentation;
    - techs come only from completed research;
    - repair never lifts HP above its maximum, nor takes a bank below 0.
  - **`docs/playtest/m5.md`** lists the new placeholders, the anti-air gap (D28) and the GDO's missing repair (D31).
- **m5-common:** the shared-registries table and split list above. The kind registry closes after G2.
- **C (running, for a SendMessage):** 0320 says "This includes the Hall rally-on-patch case". C's rally-gather test
  should use a layout within 4–5 Recruits per face, and must not assert jam-free gathering beyond a face's lanes.
- **O3 (running):** nothing. N2 fires the Gunner's TailDisjointed shot from the Garrison's footprint, which only needs
  the launch to come from the shooter's shape.

---

## §D Technical decisions, from T69

**Fable pieces:**
- **V1, V2 and H1:** vehicle turning, oriented contact, gliders and air crowds.
- **Q0110's crowd fix** is no longer needed (0320).
- **N4b** stays on Opus, because T77 fixes its method.
- Fable costs about twice as much (insights), so it gets one slot a wave.

### T69. The remaining Cults kinds, registered at once (G2)
- **Recommend:**
  - **What G2 adds.** All 13 kinds 0319 gives numbers to, with base fields, `site` and `requires`, as A did for six.
    No later M5 slice adds a kind.
  - **Made live later.** A kind joins the Recruit's `builds` and the Construct menu only in the slice that makes it
    work. Until then it is never produced or offered: its unit has no recipe, its building no `builds` entry.
  - **Why:**
    - the registry's order is hashed ("New kinds go last: a kind's place in this list is part of the state hash");
    - `requires` names kinds across slices: the Machine Shop needs a Chop Shop, the Scrap Hangar an Airstrip, the
      Airstrip a Chop Shop, the Temple an Altar, and Level 2 research a Cathedral.

    Registering them in one place closes the registry before the parallel waves.
- **Alternatives:**
  - **Each N-slice adds its own kinds.** More vertical, but:
    - most waves would have two slices adding kinds, and the second to merge resolves the registry conflict and
      re-blesses its goldens;
    - G2 would still need the Chop Shop and Airstrip kinds for its `requires`.
  - **G2 adds the structures only,** and N4a, N6 and N7 add their units, one a wave (W7–W9). This is the fallback if
    G2 runs long.

### T70. One queue engine: training, augmentation and research (F2, N1b, N3)
- **Recommend:**
  - **Entries.** `Production::queue` holds `QueueEntry::{Unit(ObjectKind), Augment(AugmentOrder), Research(Tech)}`. A
    `Unit` entry serializes and hashes exactly as a bare kind does today, so no golden moves.
  - **What is shared:**
    - progress in `RATIO_ONE`s;
    - paid at queue;
    - X removes the last entry with a full refund, and destruction refunds every entry;
    - 0083's group order;
    - the panel's queue display.
  - **Reservations:** `reserved: Vec<Vec<ObjectId>>`, parallel to the queue, and empty for a producer without a pool.
    - A `Unit` entry reserves as many units as its `unit_control_cost` ("equals the number of Recruits consumed",
      T55).
    - An `Augment` entry reserves one unit of its type and state.
  - **The queue limit** is per def (`ProductionDef::queue_limit`):
    - 20 for the Armory, Cathedral and Altar;
    - 10 for the Chop Shop and Airstrip;
    - 5 for the Temple.
    - `QUEUE_LIMIT` is the default.
  - **Progress per frame:**
    - one `RATIO_ONE`, or the power ratio for the GDO, as today;
    - for research, one per staffing Recruit, and none without staff. That is "Linear scaling ... Same rule as Cults
      BaseConstructionTime Scaling", exact in integers.
  - **The Altar's orders:** `ProductionDef::augment: Option<&'static [AugmentRecipe]>` (the order, its cost and
    frames).
  - **Who builds what:**
    - F2: the reservation lists and the per-def limit;
    - N1b: `QueueEntry` and `Augment`;
    - N3: `Research`.
- **Alternatives:**
  - **A queue per capability** (an `AugmentQueue`, and a `ResearchQueue` on the Tunnel upgrade's model,
    `sim/upgrades.rs`). Each would duplicate the queue display, the refunds, cancel and destruction, and 0083's group
    order.
  - **Generalizing `Recipe::kind` into a work item now.** It touches every producer's recipes for no gain before N1b.

### T71. Pool producers: several units consumed, aircraft released (F2, N4a, N6, N7)
- **Recommend:**
  - **One path** for every building that trains from its pool: F2's, written over `holds` and `production`. The
    Cathedral, Chop Shop, Airstrip and Altar are data plus an entry type.
  - **Completion:**
    - `change_kind` on the first reserved unit, which keeps its id, owner, lineage and augmentation;
    - each further reserved unit is removed as consumed, with no `Died` event, and its lineage entries are appended
      (T56 as amended);
    - a kind that isn't augmentable clears the augmentation ("vehicles are not infantry");
    - HP: Q0105's placeholder for infantry, and full HP for a vehicle or an aircraft (D27).
  - **Release:**
    - ground units: through the exit doorway (`ExitRule::Side`, M2's rule);
    - aircraft: `ExitRule::Runway { from, toward }`, which is new. The aircraft appears over the middle of the `from`
      end, facing `toward`, at IdleSpeed, and is never paused ("Aircraft ... are not covered by this rule", and air
      units block nothing).
- **Alternatives:**
  - **Spawning the product as a new object.** T61 rejected it: the id changes, and the lineage and damage must be
    copied by hand. With two Recruits, which id survives is arbitrary anyway, so `change_kind` on the first keeps one
    id at no cost.

### T72. Holding variations, and occupants that fire (N2, N3, N5)
- **Recommend:**
  - **Holding** (F1's `HoldsDef`) gains:
    - `ExitRule::AnySide`, for the Garrison: everyone comes out at once at T62's spots (`Sim::spot_around`), idle, as
      in an evacuation;
    - for an AAAA type, the entrance is the side nearest the unit when the Enter starts (D22), replacing F1's "first
      side so lettered clockwise from the top";
    - `accepts` per type.
  - **Occupants that fire:** `StructureDef::occupants_fire: Option<OccupantFire { stand_in, range_bonus_gu,
    bonus_for }>`.
    - In 5e, right after the structures' own attacks (T49), each operational Garrison is taken in id order and its
      occupants in pool order.
    - Each occupant runs its own `AttackState`. A held object keeps it, and leaving clears it.
    - Its `Shooter` is built from the Garrison: its footprint shape, elevation, owner and vision.
    - Its attack is the stand-in's for a Recruit or Disciple (the Militant's), and its own otherwise (`combat::weapon`,
      T73). The range bonus is added to that copy.
    - Each occupant picks its own target with T49's `pick`: "keeps its target"; threatening first; then the closest.
  - **Vision:** the Garrison's own SightRange. Occupants add none ("Units inside see with the Garrison's
    SightRange").
  - **Projectiles** (the Gunner's TailDisjointed shot, O3) leave from the footprint's edge nearest the target, as the
    Laser Turret's do.
  - **Reused:**
    - F1's held store and protection (held units are off the map, so they can't be targeted or hit);
    - M4-S1's `pick` and phases;
    - M4-V's events.
- **Alternatives:**
  - **One combined attack for the structure,** with the damage summed. It loses each occupant's timing and range, and
    the Gunner's projectile.
  - **Occupants on the map inside the footprint.** That breaks "units inside cannot be targeted".

### T73. Per-unit stats and AugmentationState (N1a; T67 built)
- **Recommend:**
  - **State:** `Object::augmentation: Option<Augmentation { constitution: Option<Level>, metabolism: Option<Level>
    }>`, where `Level` is NoResearch, L1 or L2. It is hashed only while set.
  - **Data:**
    - `UnitDef::augmentable: Option<Augmentable { unarmed }>`;
    - the increments and the effects table in `defs/cults/augmentation.rs`, faction rules for X1.
    - The net increments (c, m) are derived, never stored.
  - **The accessors** are the only readers:
    - `Object::max_hp()`: +10c;
    - `Object::max_speed()`: +0.5m su a tick, before `speed_factor`;
    - `combat::weapon(o) -> Option<AttackAttrs>`, owned. An unarmed unit gets Damage + c and FiringDuration − m,
      floored at 2 frames; a negative m adds frames.
    - Range bonuses don't belong here: the Garrison adds its own at the shooter (T72).
  - **HP on a MaxHP change:** `transform`'s rule (the damage is kept, 0110), floored at 1 HP (D20).
  - **`change_kind`** keeps the augmentation when the new kind is augmentable, and clears it otherwise.
  - **Perf:** `weapon()` returns a small Copy struct, and the speed accessor is one addition per unit per tick.
- **Alternatives:**
  - **A kind per combination of augmentations.** T67 rejected it: the combinations multiply.
  - **Modified stats stored on the object.** They go stale when a def or X2's data changes.

### T74. Research and level locking (N3)
- **Recommend:**
  - **Data:** `Tech` (four values) and `TechDef { line, level, cost, frames, requires: Option<Tech>,
    requires_structures }`.
  - **State:** per player, `techs: BTreeSet<Tech>`, hashed only while not empty. Queued and in-progress research live
    in the structures' queues (T70), so "no duplicates" is a scan of the owner's few research queues.
  - **The gate checks, in order:**
    - no duplicate;
    - the previous level complete (shared rule 5, "each requiring the previous level"; D23);
    - the structure requirement, at queue only (M4-K's `check_requires`);
    - the cost.
  - **Timing.** In the producers' step, research completes before augmentations in the same tick. So "the research
    level in effect at that moment" includes research finishing that frame, whatever the ids.
  - **Level locking holds by construction:** the Altar stamps the level on the unit when the augmentation finishes
    (T73), and nothing reads `techs` for that unit afterwards.
  - **Reused:**
    - Production's queue and refunds (T70);
    - M4-K's `tech.rs`;
    - F1's holding, for the staff;
    - M4-G's precedent for refunds on destruction.
- **Alternatives:**
  - **Research as one operation per structure,** as the Tunnel upgrade does. There would be no queue of 5.

### T75. Energy and timed buffs (N4a, N5)
- **Recommend:**
  - **The def:** `ObjectDef::energy: Option<EnergyDef { max, start, regen }>`, with `regen` either per second (the
    Disciple) or per held Recruit (the Shrine). It is one field for units and structures, as `max_shields` is.
  - **The state:** `Object::energy: Option<u32>`, counted in `RATIO_ONE`s per point. This follows the economy's rule
    (ARCHITECTURE > Determinism: "Money and progress are integers").
    - Rates convert once: per second × `RATIO_ONE` / 16.
    - Every 0319 rate is exact: 0.25/s → 1024 a tick, 0.5/s → 2048, 1/s → 4096, 5/s → 20480.
    - The thresholds (10, 50, 75) compare exactly.
  - **`energy::step`,** right after `shields::step`: regeneration plus any buff, capped at the maximum. A channel's
    drain belongs to its order (T76).
  - **Buffs:** `Object::buffs: Vec<Buff { kind, until }>`, hashed only while not empty.
    - `buffs::step` runs beside the shields' step. HP regeneration adds 0.125 a tick (an f32, as Shields' 0.25 is),
      up to `max_hp()`.
    - Expired buffs are dropped, and a recast refreshes `until` (D24).
- **Alternatives:**
  - **f32 energy, as for Shields.** It is exact for 0319's rates too, but a tuned rate (X2) such as 0.3/s would drift
    against the 10- and 75-point thresholds.

### T76. Spells: Shroud is an order, the Shrine's casts are commands (N4b, N5)
- **Recommend:**
  - **Shroud is `OrderKind::Shroud`.** As an order, it gets the design's endings for free: "a cancel command (Stop or
    any other order)" replaces it, and Shift-queued orders wait behind it.
    - Its behaviour holds Locomotion Stationary and drains energy each tick. It completes when the energy can't pay
      a tick's drain.
    - It is refused below 10 energy (`Reject::NotEnoughEnergy`).
    - "Taking damage does not end it" needs no hook.
    - A channelling unit counts as busy with a stationary task, so it doesn't yield.
  - **The Shrine's casts:** `Command::Cast { casters, ability, at }`. A command, not an order: a structure has no order
    queue, and the Production and Tunnel commands are the precedent.
    - The gate checks:
      - that `at` is in the owner's vision as the last tick left it (as a Place is judged by knowledge);
      - that the Shrine is operational;
      - its energy.
    - One Shrine casts per command (D24).
  - **When buffs apply.** They are applied in the commands phase (step 4), so the units "inside the area when the
    ability is cast" are where the last tick left them.
  - **Not `Command::Ability`.** Abilities are "not an order", and are unit toggles. Shroud must end on any order.
- **Alternatives:**
  - **Shroud as an `Ability`,** with a cancel hook in every order: it would duplicate the order rules.

### T77. Shroud's cloud: components, buffers and releases (N4b)
- **Recommend:**
  - **Components.** The clouds are derived each tick after 5a, from the channelling Disciples in id order.
    - Each is a circle of radius 3 GU.
    - Union-find joins two circles when their centres are at most 2r apart (D19: touching joins).
  - **Buffers:** `Sim::clouds: Vec<Cloud { members, buffer }>`, kept sorted. Each cloud is matched to last tick's:
    - a cloud whose members all still channel, in one new component, carries its buffer into it;
    - a new component that contains several whole old clouds sums their buffers;
    - an old cloud that lost a member or split releases its buffer at once over its old circles ("released
      immediately over the cloud as it was"), and the new parts start empty.
  - **Absorption** is a hook where hits are applied (`combat/damage.rs`), for single hits and area shares alike. A
    hit's raw damage (before armour) goes into the cloud's buffer instead when:
    - the attack is anti-ground (a Ground or Universal attack);
    - the target is a ground object whose centre is under a cloud;
    - the attacker isn't under that same cloud. For a projectile, that is judged by its position at launch.
  - **The release** runs in 5e on ticks where `tick % 8 == 0`, a global clock (D19). It is one area hit over the
    union:
    - each ground object takes `buffer × A(object ∩ union) / A(union)`, under the AoE rules: effective armour by the
      overlap fraction, and the minimum of 0.5 × the overlap fraction;
    - its direction is from the nearest member's centre;
    - areas are integer counts on a 4-su lattice (a 3-GU circle is about 7,200 points): deterministic, and within
      about 1% of exact.
  - **Perf.** Clouds are few. There is one lattice pass per cloud every 8 ticks, and the objects come from the
    existing spatial index.
- **Alternatives:**
  - **Exact areas** of a rectangle against a union of circles (inclusion–exclusion, or Green's theorem over arcs).
    Exact, but intricate code for a placeholder rule.
  - **A timer per cloud** from its first absorbed hit. Merges and splits make a cloud's identity, and so its timer,
    ambiguous.

### T78. Oriented silhouettes for non-square units (V1)
- **Recommend:**
  - **Scope.** Only a non-square Silhouette rotates with the facing; squares stay axis-aligned, as built. A 24x24
    square at 45° differs by at most 5 su in range or clearance, while rotating every unit would re-bless every golden
    and re-tune M5-D's measured crowds. `docs/movement.md` G6 records the approximation.
  - **`Shape` gains an angle:**
    - overlap by the separating-axis test (four axes);
    - the gap as the least vertex-to-edge distance, for edge-to-edge Range and proximity;
    - its bounding box for tile work.
  - **Collision for a rotating unit.** The whole step, move and turn, is tried first. If blocked, it is halved up to
    five times, the free part is kept, and the speed is zeroed (0035: "the blocked part is discarded").
  - **Determinism.** Only + − × ÷ and `sqrt`, with `sc_sim::math` for the facing's sine and cosine.
  - **Nav** is unchanged: the size class comes from the longer side, so a 56x40 vehicle plans as class 1.
- **Alternatives:**
  - **Rotate every unit.** Exact to the design, but every golden and every crowd number moves.
  - **A swept time-of-impact solver** for oriented boxes. Exact contact, but more code; bisection is enough at 8 su a
    tick.

### T79. Wheeled movement (V2)
- **Recommend:**
  - **`movement/wheeled.rs`,** beside `drag.rs` (M3-G's model for momentum):
    - a signed speed on `Mover::momentum`;
    - ForwardAcceleration, ReverseAcceleration, Deceleration and the two maxima;
    - a turn rate of |v| / MinimumTurnRadius while Moving or Reversing, and none while Stationary or Stopping.
  - **Steering** follows the farthest visible waypoint, as now.
    - The radius doesn't depend on speed, so the vehicle doesn't slow for corners.
    - A goal inside its turning circle is reached by a forward loop (D25).
    - It brakes at v² / (2 × Deceleration) before the goal.
    - "Predictable, not optimal": no Reeds–Shepp or hybrid-A* plans.
  - **Reverse:** `OrderKind::Reverse(Vec2)` with ReversingToLocation: the same plan, with Reversing (`behaviors.md`).
  - **Contact.** V1's cut zeroes the speed, so a blocked vehicle restarts at one frame's acceleration each tick and
    creeps while turning (0035).
  - **Yielding** (D25's placeholders):
    - vehicles don't sidestep;
    - an idle vehicle doesn't yield;
    - own idle infantry yields to vehicles as to any unit.
- **Alternatives:**
  - **Kinematic car planning** (optimal paths, three-point turns). Unpredictable to the player, and costly.

### T80. Unit turrets (U1)
- **Recommend:**
  - **State:** `AttackState::turret: Option<TurretState { facing: f32, locked: Option<ObjectId> }>`, hashed only
    while set. The facing is relative to the base, so the base's turn carries the turret.
  - **Turning,** in 5e before the phases: toward the locked or scanned target, by at most TurnRate, clamped to
    ±TurnAngle/2.
  - **Aiming:**
    - AimDuration counts from when the target is in range and arc;
    - Firing starts when the duration is over and the turret faces the target (within the tick's remaining turn);
    - a target leaving the arc or range cancels the aim (the existing interruptible rule).
  - **Scanning** uses the existing threat ranking and rotation step (`orders/world.rs`), measured from the turret's
    facing.
    - It runs whenever nothing is locked, Move included.
    - The behaviours set `locked` wherever the design sets LockedTarget.
  - **Reused:**
    - `Shooter`'s `turret` flag ("the turret aims, not the base");
    - `may_move` (a turret unit's base is always free);
    - the Laser Turret's barrel in the client.
- **Alternatives:**
  - **The turret as an object of its own:** two objects per unit, with ids and hashing for nothing.

### T81. Gliders, air separation, and the map edge (H1)
- **Recommend:**
  - **`movement/glider.rs`:**
    - the speed stays between IdleSpeed and MaxSpeed, with Acceleration and Deceleration;
    - the turn rate is MaxCentripetalAcceleration / v;
    - it is always Moving.
  - **Paths** stay straight ("Air units fly straight").
    - Idle and on arrival, it circles the point at IdleSpeed, at a radius of IdleSpeed²/a.
    - Strafing: it flies at the target, keeps going until out of range, then loops back at full speed.
  - **Separation** (`movement/air.rs`) runs after the moves.
    - Each pair of air units within the larger SeparationRadius is pushed apart, by at most 2 su a tick each, in id
      order.
    - The radius is `UnitDef::separation_radius`, a new field (D26's placeholder: 32 su).
    - It never blocks, and the pairs come from the spatial index.
  - **The edge.**
    - A position may leave the map.
    - `Object::map_pos()` clamps it to the nearest edge tile, for vision, targeting, grids and selection.
    - The gate clamps orders' points into the map.
  - **Takeoff:** T71's runway release.
- **Alternatives:**
  - **Boids-style steering** (cohesion and alignment): unpredictable, and more to tune.

### T82. Def fields, registries, tunables and X1 (every new slice)
- **Recommend:**
  - **Def fields:** one new field per def struct per wave (§C's table), each at the end of its struct's literal.
  - **Hashing.** New state is hashed only while set (T65): the turret, augmentation, energy, buffs, clouds, techs and
    reservations.
  - **Tunables** go at the end of their system's module. The new modules are `tuning/air.rs`, `tuning/energy.rs` and
    `tuning/spells.rs`. Numbers stay in the defs.
  - **The kind registry** closes after G2 (T69).
  - **X1** stays alone and last among the content (W10), and converts all of it once. Its `structural_diff` grows by
    the new structural fields (§C).
- **Alternatives:**
  - **X1 before the new content,** as in the earlier-playtest option (§F). The content slices after it then write
    their tunables in the ruleset's form from the start.

### T83. Repair: one order and one step for every repairer (N8)
- **Recommend:**
  - **Who repairs what is worker data:** `WorkerDef::repairs: Option<RepairDef { targets }>`.
    - The Recruit: `OwnBuildingsAndVehicles`, paid. The Field Engineer takes the same when it is built.
    - The Agent: `Kinds(&[(TUNNEL, Free), (LASER_TURRET, Paid)])`. "Tunnels are repaired for free" is the Agent's
      doc's rule, so it lives in the Agent's list, not on the Tunnel.
    - "Vehicles" are units whose base is neither infantry nor a Glider (D30).
  - **The target's terms are derived** from what built it: a structure's `SiteDef` (`frames`,
    `SitePrice::Crystals`), or the `Recipe` that produces a unit. Two tunables halve them. A kind with neither isn't
    repairable, so the Jackal becomes repairable when N6 adds its recipe.
  - **The order:** `OrderKind::Repair(ObjectId)`, hashed only while carried.
    - **At a building,** the repairer claims its own spot within reach of the edge (T62's first ring), as E's Assist
      does. That settles the Agent's "side, reach" (Director's discretion): any side, from outside. A lowered
      Laser Turret is repaired as it is, and repair doesn't raise it.
    - **At a vehicle,** it closes to reach as a melee attacker does, and follows it.
    - **It ends** at full HP, or when the target dies or isn't its owner's any more (D30).
  - **The step** runs after construction's. The repairers are derived each tick as E derives builders (the units on
    `Repair(target)` within reach), so n repairers give exactly n times the rate.
    - HP rises by n × MaxHP / repair_frames, capped at `max_hp()`: f32, as shields are.
    - **Payment in whole crystals, with no stored state.** The bank holds whole crystals (`Player::crystals: u32`).
      Each crystal buys k = MaxHP / repair_price HP. HP rises free to the top of its current k-step, and each further
      step costs one crystal. With an empty bank, HP stops at the step's top.
    - From 0 to full, that is ceil(price / 2) crystals: 38 for the Jackal's 37.5. A repair that starts mid-step gets
      that step's remainder free, which is less than one crystal's worth.
    - Targets are taken in id order, so a short bank serves the lowest id first.
  - **Reused:**
    - E's Assist: the spot claim, and the per-tick count within reach;
    - T62;
    - the gate's patterns, and the bank's checked spend (T7).
- **Alternatives:**
  - **A per-target credit in `RATIO_ONE`s,** paid each frame. Exact to the frame, but it is new hashed state that
    must be cleaned up on death, for a difference under one crystal.
  - **A stored list of repairers per target.** It must follow every re-order and death. Derived counts are E's rule.
  - **Paying the half price up front.** That contradicts "paid gradually as HP is restored".

---

## §E Question drafts

**14 new drafts, D18–D31.**
- **Format.** The Director's format, as in §5: a `# QXXXX — Title` heading, the Date left as `DATE`, and options with
  their consequences.
- **None blocks.** Each slice builds the proposal, marked `Placeholder (QD-slug)`, until the answer comes.
- **Numbers.** File them from **Q0112**, since Q0111 is the Agents' hatch question. Answers before a slice launches
  go into its brief; later ones go to S1 or S2.
- **Not a draft:** 0319 gives the Disciple "no weapon (placeholder, to confirm with the user)". The Director confirms
  it with the user directly. N4a builds no attack meanwhile, and an attack later is a small change (T69, N4a).
- **Answered meanwhile:**
  - **Q0110,** by 0320 (option d). It needs no slice. L's brief takes the layout rule, and C gets a note (§C).
  - **Q0095 and Q0108 point 2,** by 0322. That goes into J1's brief (§C).
- **Being asked already, so no draft:** 0322's open point, a right-click on a damaged building that can also be
  entered. N8 keeps Enter meanwhile, as 0322 says.

| Draft | Title | Blocking | Slices | Built meanwhile |
|---|---|---|---|---|
| D18 | The Disciple's panel: Shroud's key, and casting with a group | no | N4a, N4b | R: Shroud; Groupable, and every selected Disciple that can channels |
| D19 | Shroud: what is under the cloud, and the release | no | N4b | the centre inside; touching circles join; the direction from the nearest Disciple; underground objects unaffected; no regeneration while channelling; seen by all who see the Disciple; a global 8-frame clock |
| D20 | Augmentation: HP on a MaxHP change, the reserved unit, the weakest Recruit | no | N1a, N1b | damage kept, never below 1 HP; the unit inside the longest; the numbers as written |
| D21 | The Garrison: targets, range, Eject All, casting, augmented occupants | no | N2 | each occupant picks its own target; range from the footprint; all out at once; no casting; the Militant's attack unchanged |
| D22 | Entering a building whose sides are all alike | no | N2, N3, N5 | the side nearest the unit |
| D23 | Temple research: the next level, and the Cathedral check | no | N3 | Level 2 only after Level 1 completes; the Cathedral checked at queue |
| D24 | The Shrine's abilities: who is inside, recasting, several Shrines | no | N5 | the centre inside; a recast refreshes; one cast, from the fullest Shrine |
| D25 | Vehicles among other units: yielding and reversing (0323 settled the grazing) | no | V2, N6 | an idle vehicle doesn't yield; reversing only on W; a forward loop to a point behind |
| D26 | The Sparrow's flight: separation, edge, takeoff, idling, strafing | no | H1, N7 | 32 su; no overshoot cap; takeoff from the B end at IdleSpeed; circling 2 GU past C; the 16-su idle circle; on until out of range; Stop and Hold circle |
| D27 | Pool production beyond the Armory: which Recruits, and the product's HP | no | N4a, N6, N7 | the Recruits inside the longest; full HP for vehicles and aircraft; the Disciple as Q0105 |
| D28 | The Sparrow at the playtest: no anti-air for the GDO or the Colonists | no (the playtest's content) | N7, M | kept in, with the gap named in `docs/playtest/m5.md` |
| D29 | Stale lines after 0319 | no (doc cleanup) | J1, G2 (read only) | 0319's readings |
| D30 | Repair: the order's end, an empty bank, the Agent's key, aircraft, Tunnel expansions and tiers, the Jackal's price | no | N8 | ends at full HP; waits for crystals; F for the Agent; no aircraft; no expansions; 480 frames at every tier; crystals only |
| D31 | The GDO's repair: the Field Engineer has no numbers | no (M5's scope) | N8 (as data only), M | no Field Engineer in M5, and the gap named in `docs/playtest/m5.md` |

**The three most important:**
- **D28:** whether two of the three pairings against the Cults are playable in the playtest. Neither the GDO nor the
  Colonists can hit air in the build.
- **D26:** the Sparrow's flight. H1 builds the game's first Glider on Fable, and seven answers may rework it. (0323
  settled D25's main point, the vehicles' grazing, so D25 drops out of the top three.)
- **D19:** Shroud's open points decide whose damage is absorbed and how it comes back. They are the subtlest rules in
  0319's content.

#### D18

````markdown
# QXXXX — The Disciple's panel: Shroud's key, and casting with a group

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a) and (c): R casts Shroud, Q stays Move; the Disciple stays Groupable and every selected Disciple with 10 energy channels.
- **Docs**: `factions/cults/units/disciple.md > ObjectInterfaceState`, `> Placeholder stats`; `systems/controls/selection_and_hud.md > BasicCombatUnitInterfaceState`, `> SelectionGroup`; `core/entities.md > Groupable`; `factions/gdo/units/peacekeeper.md` (Hotkey: R); `factions/colonists/units/sabre_legionnaire.md` (R: Convert)

## Question
Two points of the Disciple's panel:
1. 0319 gives Shroud "hotkey **Q**", and the panel is "BasicCombatUnitInterfaceState, plus **Q: Shroud**". That template already has "[Q] Move". Which key should Shroud use?
2. The Disciple is "Groupable - true", but caster-type units are Ungroupable ("one cast should spend one unit's energy, not the whole group's"). Which applies to the Disciple?

## Context
- The units that add an ability to the combat template use R: the Peacekeeper's shield ("Hotkey: R ... placeholder") and the Legionnaires' "R: Convert". R is empty in BasicCombatUnitInterfaceState.
- Shroud has no target: it is a channel centred on the Disciple. With Groupable Disciples, one press starts every selected Disciple's cloud, and clouds that touch join ("all contiguously connected clouds share one damage buffer"). To keep some Disciples back, the player selects fewer.

## Options
Key:
- **(a) R: Shroud**, and Q stays Move, as for the Peacekeeper and the Legionnaires.
- **(b) Q: Shroud**, replacing Move on the Disciple's panel. Move is then by right-click only.

Grouping:
- **(c) Groupable, and all channel.** One press starts every selected Disciple that has 10 energy. A large cloud is quick to make.
- **(d) Ungroupable,** by the caster rule. One press, one Disciple, then Tab to the next. Precise, but a large cloud is slower to make.

Proposed: (a) and (c).
````

#### D19

````markdown
# QXXXX — Shroud: what is under the cloud, and the release

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: the proposal below.
- **Docs**: `factions/cults/units/disciple.md > Spells > Shroud`, `> Placeholder stats`; `systems/combat.md > AoEDamage`, `> DirectionalArmor`; `systems/units.md > Underground Collision`

## Question
Shroud is built from the doc's rules. Some points are still open:
1. **Under the cloud.** Is an object under the cloud when its centre is inside a cloud's circle, or when any part of it overlaps one? This decides whose damage is absorbed, and which attackers are "under the cloud" and so do full damage.
2. **Connected.** Do two clouds connect when their circles overlap, or also when they only touch?
3. **The release's direction.** For directional armour, an area hit's zone is "the angle from the unit's center to the AoE center", but a connected cloud has several centres. Which one counts for a unit?
4. **Underground objects** (DrillUnits, Syndicate expansions, a lowered pop-up Laser Turret). "It affects every ground object". Are underground objects under the cloud protected? Do they take a share of a release, as ground area damage hits them (Q0071)?
5. **Regeneration while channelling.** "drains **5 energy per second** (40 s from full)": 40 s means the Disciple's own 0.5/s regeneration stops while it channels. Is that right?
6. **Who sees the cloud:** every player who sees the Disciple, or only its owner?
7. **When releases happen:** every 8 frames on the game's clock, for all clouds together, or 8 frames after a cloud first absorbs damage?

## Context
- "standing **under the cloud**" suggests the centre. The shares already use the overlap area.
- **The numbers:**
  - A cloud's area is about 200 times a Militant's.
  - A lone Militant under a lone cloud, shot for 4, takes 0.5 at the release: the area minimum ("0.5 × overlap fraction").
  - A 3x3 building wholly under one cloud takes about a third of what was absorbed.
- **For 7:** either way, damage comes back at most 0.5 s late. A fixed clock is the simplest to read.

## Options
1. **(a)** the centre; **(b)** any overlap.
2. **(a)** touching joins; **(b)** overlapping only.
3. **(a)** from the nearest Disciple of the connected cloud; **(b)** no directional armour on releases.
4. **(a)** neither protected nor hit by releases, since they aren't ground objects; **(b)** protected, and they share releases as ground area damage does.
5. **(a)** no regeneration while channelling: 40 s from full; **(b)** it regenerates: about 44 s from full.
6. **(a)** every player who sees the Disciple; **(b)** its owner only.
7. **(a)** the game's clock; **(b)** from the first absorbed hit.

Proposed: (a) on all seven.
````

#### D20

````markdown
# QXXXX — Augmentation: HP when MaxHP changes, the reserved unit, and the weakest Recruit

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: the proposal below.
- **Docs**: `factions/cults/augmentation.md > Increments`, `> Augmentation Effects`, `> Augmentation Orders`; `systems/combat.md > MinimumDamage`

## Question
1. **HP when MaxHP changes.** An augmentation changes MaxHP by 10 per Constitution increment, up or down. What happens to a damaged unit's HP? Can a negative increment kill a badly damaged unit?
2. **Which unit is reserved.** An order "reserves one unit of the selected type and AugmentationState". If several match, which one?
3. **The weakest Recruit.** Improved Metabolism at Level 2 (+6 Metabolism / −3 Constitution) gives a Recruit:
   - MaxHP 10 (40 − 30);
   - melee Damage 0 (3 − 3), so each hit does the 0.5 minimum;
   - FiringDuration 2 and speed 7.

   Is that intended?

## Context
- Elsewhere a change of maximum keeps the damage: a Warp In's HP follows its maximum (0110), and the planned live tuning works the same way.
- The Armory reserves "the Recruit inside the longest" (Q0105's placeholder).

## Options
1. **(a)** the damage is kept, but HP never drops below 1, so an augmentation never kills; **(b)** HP is kept, capped at the new maximum; **(c)** HP scales with the maximum.
2. **(a)** the matching unit inside the longest; **(b)** the most damaged; **(c)** the least damaged.
3. **(a)** as written, for the numbers pass; **(b)** a floor (e.g. MaxHP at least half the base, and Damage at least 1).

Proposed: 1a, 2a, 3a.
````

#### D21

````markdown
# QXXXX — The Garrison: targets, range, Eject All, casting, and augmented occupants

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: the proposal below.
- **Docs**: `factions/cults/structures/garrison.md`; `factions/syndicate/structures/laser_turret.md > Targeting`; `systems/combat.md > AttackAttributes > Range Measurement`; `factions/cults/units/gunner.md`

## Question
1. **Targets.** Does each unit inside pick its own target, as the Laser Turret does ("keeps its target"; threatening first, then the closest)? Or do they all fire at one?
2. **Range.** Is range measured from the Garrison's footprint, edge to edge? Then a Gunner's MinRange of 2 also counts from the footprint.
3. **Eject All.** Do the units come out all at once around the footprint, as in an evacuation, or one after another?
4. **Casting inside.** The doc leaves this open ("Whether an Disciple can also cast spells from inside: TBD"). Until it is decided, a Disciple inside can't cast.
5. **Augmented occupants.** A Recruit inside fires the Militant's attack. Do its augmentation's attack parts (+melee Damage, −FiringDuration) change that attack?

## Context
- Since every occupant uses the same rule, they mostly pick the same target anyway.
- 0319's check "A Garrison with 4 Militants fires at range 5" reads naturally with 2(a).

## Options
1. **(a)** each picks its own, by the Laser Turret's rule; **(b)** all fire at one, the closest threatening target.
2. **(a)** from the footprint's edge; **(b)** from its centre.
3. **(a)** all at once, like an evacuation; **(b)** one after another, a rapid stream, like the Armory's.
4. **(a)** no casting inside for now; **(b)** casting allowed, centred on the Garrison.
5. **(a)** no: the Militant's attack, unchanged; **(b)** yes.

Proposed: (a) on all five.
````

#### D22

````markdown
# QXXXX — Entering a building whose sides are all alike (Garrison, Temple, Shrine)

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a), the side nearest the unit.
- **Docs**: `systems/controls/commands.md > Enter`; `factions/cults/structures/garrison.md`, `temple.md`, `shrine.md`; `core/entities.md > Side lettering convention`

## Question
The Garrison, Temple and Shrine are AAAA, and none names an entrance side. Enter says "The unit walks to the structure's entrance side and goes inside". Which side do units go in by?

## Context
- The Armory, Altar, Cathedral, Chop Shop and Airstrip name side B.
- The Garrison's units "leave through any side". For a defensive building, a long walk round to one fixed side under fire matters.
- The Armory's slice, now being built, takes the top side for an AAAA building, for now.

## Options
- **(a) Any side:** the side nearest the unit when it is ordered in.
- **(b) One fixed side:** the top in the default orientation, turning with the building.
- **(c) Per building:** (a) for the Garrison, and (b) for the Temple and the Shrine.

Proposed: (a).
````

#### D23

````markdown
# QXXXX — Temple research: queuing the next level, and the Cathedral check

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: 1(a) and 2(a).
- **Docs**: `factions/cults/structures/temple.md`; `core/tech_and_research.md > Research > Shared Rules`; `> Tech Prerequisites > Check Timing`; `> Unit Prerequisites`

## Question
1. The Temple's Q and W research "the next level". While Level 1 is queued or in progress, can Level 2 be queued behind it?
2. Level 2 also "requires the player to own a Cathedral". Is that checked only when Level 2 is queued, as a unit's TechPrerequisite is, so that losing the Cathedral later doesn't cancel it?

## Context
- Shared rule 5: "each requiring the previous level". Rule 4: "a technology cannot be queued or in progress at more than one structure at the same time".
- Units: "Check Timing: checked only when the unit is queued", and "units already queued or in production continue and complete normally".

## Options
1. **(a)** No. Q is unavailable ("In progress") until Level 1 completes.
   **(b)** Yes, behind Level 1 in the same Temple. Cancelling Level 1 then cancels Level 2 too.
2. **(a)** checked at queue only; **(b)** checked again when it starts.

Proposed: 1(a), 2(a).
````

#### D24

````markdown
# QXXXX — The Shrine's abilities: who is inside, recasting, and several Shrines

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a) on all three.
- **Docs**: `factions/cults/structures/shrine.md > Abilities`, `> Placeholder stats`; `systems/controls/selection_and_hud.md > SelectionGroup`

## Question
1. **Inside the area.** Is a unit inside when its centre is in the circle, or when any part of it overlaps?
2. **Recasting** on units that still have the buff. Does the duration restart, or do the effects add up (two health regenerations giving +4 HP/s)?
3. **Several Shrines selected.** Shrines are Groupable. Does one press cast once, or once per selected Shrine?

## Context
- "No cooldowns." With effects that add up, two Shrines double a heal.
- The caster rule: "one cast should spend one unit's energy, not the whole group's".

## Options
1. **(a)** the centre; **(b)** any overlap.
2. **(a)** the duration restarts, and the rates don't add; **(b)** the rates add.
3. **(a)** one cast, from the selected Shrine with the most energy; **(b)** every selected Shrine casts at the point.

Proposed: (a) on all three.
````

#### D25

````markdown
# QXXXX — Vehicles among other units: yielding and reversing

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a) on all three.
- **Docs**: `systems/units.md > Ground Collision`, `> Collision Response (Un-sticking)`, `> FixedTurnRadiusMovement`; `systems/controls/behaviors.md > MovingToLocation`, `> ReversingToLocation`; entry 0323

## Question
The Jackal is the game's first vehicle. 0323 settles how a moving one gets round a blocker: "**WheeledVehicles** turn and graze around the obstacle instead" of stepping sideways. Three points remain:
1. **Yielding.** An idle vehicle can't take "one short sidestep". Does it yield at all, for example by driving forward a little? Or does it stand like a unit on Hold Position? And does own idle infantry yield to a moving vehicle, as to any unit?
2. **Reversing by itself.** Does a vehicle ever back up on its own, to turn round in a tight spot or back off a wall? Or only on the Reverse order (W)?
3. **A point behind it.** Ordered to a point inside its turning circle, such as just behind it, it drives a forward loop. Is that right?

## Context
- The Jackal turns on a 1 GU radius at 8 su/frame, so it fits most corridors.
- 0035's creep is designed to free a vehicle pressed against a wall without reversing.

## Options
1. **(a)** an idle vehicle doesn't yield, and infantry yields to vehicles; **(b)** an idle vehicle drives forward up to one body length to clear the way.
2. **(a)** only on W; **(b)** automatic three-point turns.
3. **(a)** a forward loop; **(b)** reversing when the point is behind it and close.

Proposed: (a) on all three.
````

#### D26

````markdown
# QXXXX — The Sparrow's flight: separation, the map edge, takeoff, idling and strafing

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a) on all seven.
- **Docs**: `systems/units.md > Air Collision`, `> GliderMovement`; `systems/controls/behaviors.md` (the GliderMovement exceptions); `factions/cults/structures/airstrip.md`; `factions/cults/units/sparrow.md`

## Question
The Sparrow is the game's first Glider. Open points:
1. **SeparationRadius** ("per unit type ... Must be larger than the unit's Silhouette") has no value. Is 32 su right? That is just over its half-diagonal of 28 su.
2. **Maximum overshoot** past the map edge ("TBD"). Is there no limit beyond what its turn needs (up to about 3 GU at full speed)?
3. **Takeoff.** A new Sparrow appears over the runway's B end, facing C, at IdleSpeed, then speeds up along the runway. Is that right?
4. **No rally point.** Where does it circle? 2 GU past the runway's C end?
5. **Idle circling.** At IdleSpeed with its full centripetal limit, the circle's radius is 16 su, a quarter of a tile: nearly a spin on the spot. Is that intended, or should idle circles have a set radius (e.g. 1 GU)?
6. **Strafing.** After a pass, how far does it fly on before looping back? Until the target is out of range, then turning at full speed?
7. **Stop and Hold Position.** It can't stop. Does Stop make it circle where it is? Does Hold make it circle there and fire only at what comes into range, without chasing?

## Context
- At 0319's numbers the full-speed turn radius is 100 su, so a pass and its loop take about 3 s.
- A set idle radius (5b) would be one more number per type.

## Options
1. **(a)** 32 su; **(b)** another value.
2. **(a)** no limit; **(b)** a cap (e.g. 2 GU), steering back earlier.
3. **(a)** as described; **(b)** it appears already at MaxSpeed.
4. **(a)** 2 GU past the C end; **(b)** over the Airstrip.
5. **(a)** the tightest circle (16 su); **(b)** a set idle radius of 1 GU.
6. **(a)** until out of range; **(b)** a fixed distance past the target (e.g. 2 GU).
7. **(a)** Stop and Hold both circle in place, and Hold doesn't chase; **(b)** no Hold for Gliders.

Proposed: (a) on all seven.
````

#### D27

````markdown
# QXXXX — Pool production beyond the Armory: which Recruits, and the product's HP

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a) on all three.
- **Docs**: `factions/cults/structures/chop_shop.md`, `airstrip.md`, `cathedral.md`; `factions/cults/structures/armory.md > Training`; `factions/cults/augmentation.md > Persistence`

## Question
Q0105 asks which Recruit the Armory reserves, and whether the trained unit keeps its damage. The Chop Shop, Airstrip and Cathedral raise the same points:
1. **Which Recruits.** A Jackal reserves two. Are they the two that have been inside the longest?
2. **The HP of a vehicle or aircraft.** A Jackal (MaxHP 120) is made from two Recruits (40 each), a Sparrow (MaxHP 50) from one. If the Recruits were damaged, does it come out at full HP, or carry their damage?
3. **The Disciple.** Does it follow the Armory's answer (Q0105) for its Recruit's damage?

## Context
- A vehicle loses the Recruits' augmentations ("vehicles are not infantry"). Carrying their damage but not their augmentations would be inconsistent.
- Q0105's placeholder keeps the Recruit's damage for a Militant.

## Options
1. **(a)** the longest inside; **(b)** the least damaged.
2. **(a)** full HP for vehicles and aircraft; **(b)** the Recruits' damage, summed; **(c)** their HP fraction, kept.
3. **(a)** as at the Armory; **(b)** full HP.

Proposed: (a) on all three.
````

#### D28

````markdown
# QXXXX — The Sparrow at the playtest: the GDO and the Colonists can't hit air yet

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a).
- **Docs**: `factions/cults/units/sparrow.md`; `factions/gdo/units/peacekeeper.md`; `factions/colonists/units/sabre_legionnaire.md`, `rifle_legionnaire.md`; `systems/combat.md > ValidTarget`

## Question
The Sparrow attacks ground and air, never lands, and has unlimited ammo. In the playtest build, no GDO or Colonists unit or structure can attack air: the Peacekeeper and both Legionnaires are Ground-only. Their designed answers (the GDO's Rocketman, Missile Turret and aircraft, and the Colonists' Lancer and Skylance) aren't built, because their numbers are still TBD. With Sparrows, a Cults player can beat them without loss. What should the playtest do about it?

## Context
- The Syndicate can answer it (the Guard and the Laser Turret are Universal), and so can the Cults (the Militant and the Jackal).
- The playtest is hot-seat, so whoever plays the Cults can simply not build Sparrows.

## Options
- **(a)** Keep the Sparrow in, and name the gap in `docs/playtest/m5.md`.
- **(b)** Build and test the Sparrow, but leave the Airstrip out of the playtest's Construct menu (greyed, "Not in this build").
- **(c)** Give the GDO and the Colonists an anti-air answer before the playtest. That means design work and one or two more slices.

Proposed: (a).
````

#### D29

````markdown
# QXXXX — Stale lines after 0319

- **Date**: DATE
- **Blocking**: no — doc cleanup; the build follows 0319.
- **Docs**: `factions/cults/units/recruit.md > Unit Commands`, `> Right-Click Resolution`; `systems/controls/commands.md > Enter`; `factions/cults/structures/chop_shop.md`, `airstrip.md`, `shrine.md`, `garrison.md`, `README.md > Cults Vehicles`

## Question
Can these lines be brought up to date?
1. **Gunworks and Machine Shop.** 0319 says they have "no commands and can't be entered (a right-click on it with Recruits is a Move)". Three places still say otherwise:
   - `recruit.md > Unit Commands`: "**Enter Gunworks** / **Enter Machine Shop** (staff research)";
   - `recruit.md > Right-Click Resolution`: "Own Gunworks / Machine Shop → Enter (staff research)";
   - `commands.md > Enter`: "**Cults — Temple, Gunworks, Machine Shop**: Recruits only".
2. **The Garrison.** `recruit.md > Right-Click Resolution` has no line for it, though `garrison.md` lets Recruits enter it by right-click.
3. **Old TBD lines** that the same files' Placeholder stats now settle:
   - `chop_shop.md > Production`: "Internal capacity, queue size, sides: TBD";
   - `airstrip.md > Production`: "(number per aircraft TBD)" and "Capacity, queue size, sides: TBD";
   - `shrine.md > Abilities`: the TBD cells of the table;
   - `structures/README.md > Cults Vehicles`: "Vehicles (not yet designed)", "Aircraft (not yet designed)".
4. **A typo** in `garrison.md`: "Whether an Disciple".

## Context
Found while planning the rest of the Cults for M5. 0322 settled the Recruit's panel letters, but not these lines.

## Options
- **(a)** Update them as listed: until their research is designed, a right-click on the Gunworks or Machine Shop is a Move, and "Own Garrison → Enter Garrison" is added.
- **(b)** Leave them.

Proposed: (a).
````

#### D30

````markdown
# QXXXX — Repair: when it ends, an empty bank, the Agent's key, and which targets

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a) on all seven.
- **Docs**: `factions/cults/units/recruit.md > Repair`; `factions/syndicate/units/agent.md > Repair`, `> Agent ObjectInterfaceState`; `factions/syndicate/tunnels.md`; `factions/cults/units/jackal.md`, `sparrow.md`; entry 0321

## Question
Repair is being built from 0321. Some points are open:
1. **When the order ends.** Do repairers stop once the target is at full HP, or stay on it and repair it again when it takes more damage?
2. **An empty bank.** Repair is "paid gradually as HP is restored". When the player runs out of Space Crystals, do the repairers wait and go on when crystals come in, or does the order end?
3. **The Agent's key.** The Agent's panel has Q Move, A Build Tunnel, S Drop Off Resources, D Build Laser Turret and X Stop, so Repair has no letter. Which key? Without one, an Agent can't repair a Tunnel at all while a right-click on a Tunnel means Enter (0322's open point).
4. **Aircraft.** The Sparrow "never lands". Can it be repaired?
5. **Tunnel expansions.** Can Agents repair a Hideout, Chemical Plant or Optics Lab? If so, for free like the Tunnel, or for their price?
6. **A Tunnel's build time at Tiers 2 and 3** (MaxHP 800 and 1000): is it its construction's 480 frames, or construction plus its upgrades (960 and 1440)? It only sets the rate, since Tunnel repair is free.
7. **The Jackal's price** is "2 Recruits, 75 Space Crystals". Is half of it 37.5 crystals, with no Recruits?

## Context
- **The numbers:**
  - one Recruit repairs an Armory (300 HP, 480 frames, 200 SC) from 0 to full in 240 frames, for 100 SC;
  - an Agent repairs a Laser Turret in 160 frames for 50 SC, and a Tier 1 Tunnel in 240 frames for nothing.
- **The bank holds whole crystals,** so repair pays one crystal per equal share of the HP: a Jackal costs 38 crystals from 0 to full.
- **For 3:** F, W, E and R are free on the Agent's grid, and F sits next to D, the Recruit's Repair key.

## Options
1. **(a)** stop at full HP; **(b)** stay on the target until given another order.
2. **(a)** wait, and go on when crystals come in; **(b)** the order ends.
3. **(a)** F: Repair; **(b)** W: Repair; **(c)** no key: right-click only.
4. **(a)** no, aircraft aren't repaired; **(b)** yes, by a repairer under it.
5. **(a)** no: the doc names only Tunnels and Laser Turrets; **(b)** yes, for free; **(c)** yes, for half their price.
6. **(a)** 480 frames at every tier; **(b)** construction plus the upgrades.
7. **(a)** yes, crystals only; **(b)** something else.

Proposed: (a) on all seven.
````

#### D31

````markdown
# QXXXX — The GDO's repair in M5: the Field Engineer has no numbers

- **Date**: DATE
- **Blocking**: no — proceeding on assumption: (a), no Field Engineer in M5.
- **Docs**: `factions/gdo/units/field_engineer.md`; `factions/gdo/structures/career_campus.md`; entry 0321

## Question
0321 gives the Field Engineer's repair the shared rules. But the unit itself is still "TBD: stats, cost, build time, hotkeys, ObjectInterfaceState", and it needs a Career Campus, which has "Full stats TBD". Neither is in the game, so the GDO will be the one faction with no way to repair in the playtest. Should M5 build them?

## Context
- Repair is built once, as worker data. With numbers, the Field Engineer needs only its stats, its Barracks recipe and its panel, and the Career Campus its stats: one more slice of about 2 h before the playtest.
- The Field Engineer's other abilities (Bunker, First Aid Tent) are TBD too, and would stay greyed.
- The Cults and the Syndicate repair (0321). The Colonists' shields regenerate at 4 a second, though their HP doesn't.
- No GDO vehicle is built, so the Field Engineer would repair GDO buildings only.

## Options
- **(a)** No Field Engineer in M5. The playtest notes name the GDO's lack of repair.
- **(b)** Placeholder numbers for the Field Engineer (with Repair as its only working ability) and for the Career Campus, and one more slice.

Proposed: (a).
````

---

## §F Size and risks

### Size

| Part | Slices | Hours (rough) |
|---|---|---|
| Merged: A, R, D | 3 | 7.5 |
| Running: C, F1, O3 | 3 | 7 |
| W3–W4 as planned: B, E, F2, J1, L (with 0320–0322's additions) | 5 | 13 |
| New: G2, V1, U1, V2, H1, N1a, N1b, N2, N3, N4a, N4b, N5, N6, N7, N8 | 15 | 38, of which Fable (V1, V2, H1) about 9 |
| Tail: S1, X1, X2, S2, M | 5 | 13.5 |
| **M5** | **31** | **about 79; about 64 ahead** |

- **12 waves instead of 8.** The playtest moves from the end of W8 to the end of W12.
- **"Ahead"** counts what isn't merged or running.
- **X1** grows with what it converts: about 3.5 h, or X1a and X1b if it passes that.

### What is likely to change
- **Answers that rework a Fable slice:** D25 (vehicles among units) after V2, and D26 (the Sparrow's flight) after H1.
  Each answer lands in one placeholder module, so the rework stays local.
- **D19** (Shroud) may change N4b's cloud rules. T77 keeps each rule in one place.
- **The Disciple's weapon:** if the user gives it one, N4a gains an attack (data, and a test).
- **0322's open point,** a right-click on a damaged building that can be entered, changes one arm of N8's resolution.
- **D31** may add a Field Engineer slice (about 2 h) before X1.
- **Q0111** (the Agents' jam) must be fixed before the playtest (`plan.md`). 0324's first step is 0323's sidestep,
  a Fable slice that isn't planned here. Its likely slot is W4's, which moves U1, N8 and S1 (§C). Steps 2 and 3, a
  2-square hatch and then the backstop, may follow if the measurements need them.
- **More Cults design** (the Gunworks' and Machine Shop's research, the Scrap Hangar's aircraft, casting from a
  Garrison) would add slices after X1, written in the ruleset's form.
- **The numbers pass.** 0319's placeholders will change, and X2's data files absorb that without code.
- **Running long:** G2 (13 kinds) or X1 (everything). G2 has a fallback (T69: its units move to N4a, N6 and N7).

### Risks

| Risk | Mitigation |
|---|---|
| V1's oriented collision moves M5-D's crowd numbers | Only non-square shapes rotate (T78). The goldens and `recruits_perf` must not change |
| Gliders interact badly with ground systems (vision, targeting, the map edge) | `map_pos()` is the one clamp (T81), and H1's tests cover the edge |
| Shroud's absorption hook sits on every hit | One hook in `combat/damage.rs`, a no-op without clouds; `shroud_perf` |
| `weapon()` (17 callers) and `max_speed()` change (N1a) | One slice changes both, with nothing changed for unaugmented units and the goldens unchanged |
| Parallel slices collide in shared registries | §C's table gives each item one owner per wave. The kind registry closes after G2 |
| The Sparrow is unanswerable for the GDO and the Colonists | D28: named in the playtest notes, or the Airstrip held back |
| The IM L2 Recruit (MaxHP 10, Damage 0) is degenerate | D20. It is the design's number, and X2 makes it data |
| The playtest is four waves later | The earlier-playtest option below |
| X1's breadth | X1a and X1b, each alone |
| Fable load and cost | One Fable build a wave (W5–W7) |
| Kinds registered before their behaviour (G2) | They stay out of `builds`, menus and recipes until their slice (T69) |
| Golden churn | New goldens per slice (L's stay infantry), and new state hashed only while set |
| Repair's whole-crystal payment drifts from "half its price" | It stays within one crystal (T83), and N8 tests it |
| The GDO can't repair in the playtest | D31 |
| 0323's sidestep (Q0111's step 1) needs a slot, touches V1's file and re-blesses goldens | W4's slot, before V1 (§C); later steps of 0324 take the next free slot, and M waits for them |

### An earlier playtest (for the Director to put to the user)

**Playtest 1: the Cults' infantry core,** with the Gunworks and the Gunner, the Garrison, the Altar, and Repair.

```
W4  J1 | L | U1                     (as in §C)
W5  G2 | N1a | N8                   (N1a leaves the Disciple to N4a)
W6  N2 | N1b | S1
W7  X1
W8  X2 | S2
W9  M                               -> playtest 1, three waves before §C's playtest
```

- **Fair enough for a first game:**
  - every faction has its infantry and at most one static defence (the Syndicate's Laser Turret, the Cults'
    Garrison);
  - nobody has vehicles or aircraft, so D28 doesn't arise;
  - Repair helps the Cults and the Syndicate. The GDO has none (D31), and the Colonists' shields regenerate.
- **The shortcut's cost:** N1a runs beside G2, against §C's rule. It skips the Disciple, and whichever of the two
  merges second adds `augmentable: None` to the other's new unit literals: a three-line fix.
- **Variant P+:** add N3 (the Temple and research) in W7. Then X1 is in W8, X2 in W9, and M in W10.
- **Then M5b,** the other nine slices, written in the ruleset's form after X1:

  ```
  W10 V1 (Fable) | N4a | N3
  W11 H1 (Fable) | N4b | N5
  W12 V2 (Fable) | N7
  W13 N6 | S3
  W14 M2                            -> playtest 2
  ```

  If two Fable builds may share a wave (V1 and H1 in W10), M5b ends a wave sooner, with playtest 2 at the end of W13.
- **The trade-off:** the user plays three waves sooner, and the core gets feedback before the rest is built. The full
  set comes one or two waves later than in §C, and there's a second QA pass.
