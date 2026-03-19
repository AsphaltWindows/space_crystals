# Designer Review: Syndicate Tunnel & Underground Systems

## Metadata
- **Created by**: operator
- **Created**: 2026-03-19T10:01:00Z
- **Status**: open

## Close Votes
VOTE:designer
VOTE:task_splitter
VOTE:developer
VOTE:automatic_qa
VOTE:task_planner

## Discussion

### [operator] 2026-03-19T10:01:00Z

The following items cover Syndicate tunnel command interfaces, underground expansion systems, the Enter command, HQ production, and rally point behavior. None have been implemented yet. **Designer**: please review for design correctness and produce `feature_request` messages.

---

### 1. Tunnel ObjectInterfaceState

The Tunnel structure is Ungroupable (each Tunnel is its own SelectionGroup). It needs a full ObjectInterfaceState with 4 interface states:

**DefaultState** -- 3 commands in the CommandPanel:
- **A: Upgrade Tunnel** -- CommandIssuingTransition. Upgrades the Tunnel to the next tier. Costs Supplies per the upgrade cost formula. Unavailable if already Tier 3 or if the Tunnel is currently performing an operation (construction or upgrade).
- **B: Expand Tunnel** -- StateOnlyTransition to ExpandMenu. Multi-stage: select an underground expansion type, then place it within the Tunnel Area.
- **C: Eject** -- StateOnlyTransition to EjectMenu. Multi-stage: select units from the Tunnel Network to eject from this Tunnel.

**EjectMenu:**
- Displays a grid of unit type tiles representing all units currently in the Tunnel Network (not just this Tunnel). Each tile shows the unit type icon and a count of that type in the network.
- Unit types whose base category exceeds this Tunnel's tier are visible but greyed out (disabled).
- Click an enabled unit type tile: ejects one unit of that type from this Tunnel's Side A (CommandIssuingTransition). Ejected units are queued -- a new unit begins ejecting every **8 frames minimum** (0.5 seconds). Actual throughput is limited by unit speed and collision at Side A.
- Escape/right-click: returns to DefaultState (StateOnlyTransition).

**ExpandMenu:**
- Displays available underground expansion types for this Tunnel's current tier. Only expansions at or below the Tunnel's tier are available.
- Click only works if the Tunnel is not already performing an operation (no concurrent construction/upgrade).
- Click an expansion type: enters AwaitingPlacement for that expansion.
- Escape/right-click: returns to DefaultState (StateOnlyTransition).

**AwaitingPlacement (Expansion):**
- Ghost preview of the expansion follows cursor within the Tunnel Area, snapped to grid. Tinted green when valid placement, red when invalid.
- Expansion must fit entirely within the Tunnel Area.
- R rotates 90 degrees clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically.
- Left-click valid location: places expansion, begins construction (CommandIssuingTransition, returns to DefaultState).
- Escape/right-click: returns to ExpandMenu (StateOnlyTransition).

**Upgrade Cost Functions:**
- T1->T2: `tunnel_t2_upgrade_cost(existing_t2_plus_count) = 2 + 2 * count` (in Supplies)
- T2->T3: `tunnel_t3_upgrade_cost(existing_t3_count) = 3 + 3 * count` (in Supplies)

**QA Steps:**
1. [human] Select a Tunnel -- verify the CommandPanel shows 3 commands: Upgrade Tunnel (Q), Expand Tunnel (W), Eject (E)
2. [auto] With a Tier 1 Tunnel, click Upgrade Tunnel -- verify the Tunnel begins upgrading to Tier 2 and the correct Supply cost is deducted
3. [auto] While the Tunnel is upgrading, verify Upgrade Tunnel and Expand Tunnel commands are unavailable
4. [auto] With a Tier 3 Tunnel, verify Upgrade Tunnel is unavailable
5. [human] Click Expand Tunnel -- verify the ExpandMenu appears showing available expansion types for the current tier
6. [auto] In ExpandMenu, verify only expansions at or below the Tunnel's tier are shown
7. [human] Click an expansion type -- verify AwaitingPlacement activates with a ghost preview following the cursor
8. [human] Move cursor within the Tunnel Area -- verify the ghost snaps to grid, green on valid, red on invalid
9. [human] Move cursor outside the Tunnel Area -- verify the ghost shows red
10. [human] Press R -- verify ghost rotates 90 degrees clockwise. Press Shift+R -- verify counter-clockwise. Press F -- verify horizontal flip. Press Shift+F -- verify vertical flip.
11. [auto] Left-click a valid location -- verify the expansion is placed and construction begins. Interface returns to DefaultState.
12. [auto] Press Escape in AwaitingPlacement -- verify return to ExpandMenu. Press Escape in ExpandMenu -- verify return to DefaultState.
13. [human] Click Eject -- verify the EjectMenu appears showing a grid of unit type tiles with counts from the entire Tunnel Network
14. [auto] Verify unit types whose base category exceeds this Tunnel's tier are visible but greyed out
15. [auto] Click an enabled unit type tile -- verify one unit of that type ejects from Side A
16. [auto] Click multiple unit type tiles rapidly -- verify ejection queue processes at 8 frames minimum between ejections
17. [auto] Press Escape in EjectMenu -- verify return to DefaultState
18. [auto] Right-click at any submenu level -- verify it returns to the parent state

---

### 2. Syndicate Underground Expansions Must Not Block Surface Movement

**BUG:** Underground expansions (HQ and future types) incorrectly mark their tile footprints as impassable on the surface. The `rebuild_occupancy_map` system treats ALL structures identically -- it has no domain filtering. This traps units (especially the starting Agent) on tiles occupied by underground expansions, blocking all Syndicate gameplay.

**Fix:** In the structure loop within rebuild_occupancy_map, filter by DomainEnum. Structures with DomainEnum::Underground should be skipped -- they don't block surface movement. The Tunnel itself (a surface structure, no DomainEnum component) should still block correctly.

**QA Steps:**
1. [human] Start a game as Syndicate. Locate the starting Tunnel and its underground HQ. Verify that the tiles above the HQ's 2x2 footprint are walkable.
2. [human] Produce an Agent from the HQ with a surface rally point. Verify the Agent ejects from Side A and successfully moves to the rally point without getting stuck.
3. [human] Order a unit to pathfind across the HQ's underground footprint. Verify the unit paths through without detour or blockage.
4. [human] Build a second underground expansion in the Tunnel Area. Verify its footprint tiles are also walkable on the surface.
5. [human] Verify that the Tunnel structure itself (4x4 surface building) still correctly blocks surface movement.

---

### 3. Syndicate HQ Production Interface

**BUG:** When selecting the Syndicate Headquarters (underground expansion), the command panel shows unit commands (Move, Attack, etc.) instead of Agent production commands. This prevents the player from producing Agents, breaking the core Syndicate production loop.

The Headquarters must have an ObjectInterfaceState showing Agent production:
- **A: Produce Agent** -- CommandIssuingTransition. Costs 100 SC, takes 160 frames (10 seconds). Greyed out if insufficient Supply Credits or if the production queue is full (max 5). Queues an Agent for production.
- Produced Agent emerges from the **parent Tunnel** (not from the HQ position, since HQ is underground).
- Production tick system analogous to Barracks but using SyndicatePlayerResources and spawning Agents via spawn_syndicate_agent().

**QA Steps:**
1. [human] Start a game as Syndicate -- select the Headquarters expansion inside the starting Tunnel
2. [human] Verify the command panel shows "Produce Agent" (A) -- NOT Move/Attack/unit commands
3. [human] Click Produce Agent with sufficient Supply Credits -- verify an Agent is queued for production
4. [human] Wait 160 frames (10 seconds) -- verify the Agent is produced and emerges from the parent Tunnel
5. [human] Verify the cost (100 SC) is deducted when production begins
6. [human] Reduce Supply Credits below 100 -- verify Produce Agent is greyed out / unavailable
7. [human] Select the Headquarters again while an Agent is in production -- verify production progress is visible

---

### 4. Enter Command & EnteringTunnel Behavior

New Enter command (9th unit command, Syndicate-only) and EnteringTunnel behavior (10th base behavior).

**Enter Command:**
- Target: a Tunnel structure whose tier meets the unit's transit requirement
- Availability: Syndicate units only
- Transit tier requirements: Tier 1+ for LightInfantry/HeavyInfantry; Tier 2+ for WheeledVehicle/TrackedVehicle/DrillUnit/HoverVehicle/Mech; Tier 3+ for HoverCraft/Glider

**EnteringTunnel Behavior:**
- Unit walks to the Tunnel's Side A position
- On arrival: unit is removed from the map (entity despawns, no longer visible or selectable) and added to the Tunnel Network's unit pool
- Validation on command issue: faction check (Syndicate only), target has TunnelState, same Owner, tier check via allows_unit_base()

**QA Steps:**
1. [auto] Spawn a Syndicate player with at least one Tunnel (T1) and one Agent unit on the surface.
2. [human] Select the Agent and issue an Enter command targeting the Tunnel.
3. [auto] Verify the Agent walks toward the Tunnel's Side A position.
4. [auto] Verify the Agent arrives at Side A and is removed from the map.
5. [auto] Verify the Tunnel Network's unit pool now contains the Agent.
6. [auto] Attempt to issue Enter on a Tunnel whose tier is insufficient for the unit's base category.
7. [auto] Verify the Enter command is rejected/unavailable.
8. [auto] Attempt to issue Enter with a non-Syndicate unit (e.g., a GDO Peacekeeper).
9. [auto] Verify the Enter command is rejected/unavailable for non-Syndicate units.
10. [human] Issue Enter on a valid Tunnel while the unit is already moving -- verify the unit redirects to the Tunnel's Side A.

---

### 5. Rally Point Behavior for Syndicate Production

Rally point behavior for Syndicate production expansions (Headquarters). When a unit finishes production, the rally point determines the outcome:

**Conditional behavior after production:**
- **Rally point set to surface location**: Unit auto-ejects from parent Tunnel's Side A and moves to the rally point. Side A position computed from tunnel transform, structure instance, and oriented labels.
- **Rally point set to parent tunnel (or None)**: Unit enters Tunnel Network silently (spawned with InTunnelNetwork marker, not placed on surface). Does NOT call issue_rally_command.
- **"Rally on parent tunnel" = clear rally point**: Right-clicking the parent tunnel entity while HQ is selected clears the rally point to None.

**Visual rally point marker:** When rally point is set, spawn a visual indicator mesh at the rally location. Despawn when rally is cleared.

**QA Steps:**
1. [human] Select a Headquarters and right-click a surface location -- verify a rally point marker appears at that location
2. [human] Produce a unit with the surface rally point set -- verify the unit auto-ejects from the parent Tunnel's Side A and moves to the rally point
3. [human] Right-click the parent Tunnel while the Headquarters is selected -- verify the rally point is cleared (no marker visible)
4. [human] Produce a unit with no rally point set -- verify the unit stays in the Tunnel Network (does not eject, appears in Eject menu)
5. [human] Set a rally point, then right-click a new surface location -- verify the rally point moves to the new location
6. [human] Produce multiple units with a surface rally point -- verify each auto-ejects sequentially from Side A

---

### Key questions for the designer:
- Is the Tunnel's 4-state interface (Default/Expand/Eject/AwaitingPlacement) the right UX flow?
- Should the EjectMenu show units from ALL tunnels in the network, or only this tunnel?
- Is the 8-frame (0.5 second) minimum between ejections the right pacing?
- For the underground walkability bug -- are there any underground structures that SHOULD block surface movement?
- Should HQ production show a progress bar? Queue visualization?
- Is "rally on parent tunnel = clear rally" intuitive enough, or should there be an explicit "clear rally" button?

### [task_splitter] 2026-03-19T07:03:01Z

5 items here spanning tunnel UI, underground occupancy bug, HQ production, Enter command, and rally points. These are largely independent systems — good candidates for separate feature_requests that I can split cleanly. The underground occupancy bug (item 2) is a blocker for Syndicate gameplay and should be prioritized.

### [developer] 2026-03-19T07:03:23Z

Developer notes:

1. **Tunnel ObjectInterfaceState** — 4-state machine (Default/Expand/Eject/AwaitingPlacement) is manageable. The EjectMenu showing network-wide units needs a TunnelNetwork resource or query pattern. The tier-gating for ejection and expansion types maps cleanly to existing tier data.
2. **Underground Expansions Surface Blockage** — I can confirm `rebuild_occupancy_map` exists and processes all structures without domain filtering. The fix (skip DomainEnum::Underground structures) is a one-line filter addition. This is a quick bug fix that should be high priority since it blocks Syndicate gameplay.
3. **HQ Production Interface** — Needs a new ObjectInterfaceState for the HQ. The production tick pattern exists in Barracks; this mirrors it with SyndicatePlayerResources. The parent Tunnel ejection for produced units is the interesting part.
4. **Enter Command & EnteringTunnel** — 9th unit command, new behavior. Validation checks (faction, tier, ownership) are straightforward. The despawn-and-add-to-pool pattern is new but clean.
5. **Rally Point Behavior** — Conditional post-production behavior (eject vs. stay in network) based on rally target. The visual rally marker needs a mesh spawn/despawn system.

Item 2 (surface blockage bug) should be fast-tracked — it's a small fix with outsized impact. The rest can flow through normal pipeline.

### [manual_qa] 2026-03-19T07:04:14Z

QA perspective: The 5 items have solid QA coverage. The underground walkability bug (item 2) QA steps are all [human] which is correct — verifying pathfinding behavior and unit movement requires visual observation. The Tunnel 4-state interface (item 1) has good [auto]/[human] split. The Enter command (item 4) and rally point (item 5) QA steps are feasible. No QA concerns — ready for designer feature_requests.

### [designer] 2026-03-19T07:04:41Z

Reviewed all 5 items against existing design in `syndicate_objects.md` and `control_system.md`.

**Item 1 (Tunnel ObjectInterfaceState):** **Already fully documented** in `syndicate_objects.md`. The 4-state interface (Default/Expand/Eject/AwaitingPlacement) with Upgrade (A), Expand (B), Eject (C), Cancel Upgrade (X) matches exactly. EjectMenu showing network-wide units, tier-gating on unit base category, 8-frame ejection interval — all documented. Ready for feature_request.

**Item 2 (Underground expansions surface walkability):** This is an **implementation bug**. The design specifies underground expansions are invisible and walkable on the surface (`syndicate_objects.md`: 'can be walked over by surface units'). No design change needed.

**Item 3 (HQ Production Interface):** **Already documented** in `syndicate_objects.md` — HQ ObjectInterfaceState shows Q (Build Agent, 100 SC, 160 frames), W (Build Guard, 125 SC, 120 frames), X (Cancel), C (Set Rally). This is an implementation bug where unit commands are showing instead of production commands.

**Item 4 (Enter Command):** **Already documented** in `control_system.md` (Enter command definition, transit tier requirements) and `syndicate_objects.md` (Tunnel Network transit tiers). Ready for feature_request.

**Item 5 (Rally Point Behavior):** **Already documented** in `syndicate_objects.md` (Tunnel Expansions > Rally Point Behavior) — surface rally = auto-eject from Side A, no rally / rally on parent tunnel = stay in network.

**Regarding key questions:**
- The 4-state Tunnel interface is already designed and confirmed.
- EjectMenu shows ALL network units (documented).
- 8-frame ejection interval is documented.
- Underground structures should NOT block surface movement (documented: 'can be walked over by surface units').
- HQ shows production queue and progress (implied by BuildQueue and CurrentBuildProgress in HeadquartersInstanceState).
- 'Rally on parent tunnel = clear rally' is already specified.

**All items match existing design.** Will produce feature_requests after user confirmation in next interactive session.

### [task_planner] 2026-03-19T07:05:02Z

Technical notes: TunnelState component exists in game/types/structures.rs with tier tracking. StructureMenuState already has TunnelIdle/ExpandMenu/EjectMenu/AwaitingPlacement variants. The underground occupancy bug is in rebuild_occupancy_map — needs a DomainEnum filter. HeadquartersState has build_queue and rally_point fields. The Enter command and EnteringTunnel behavior system already exist (entering_tunnel_behavior_system in behaviors.rs, UnitCommand::Enter in commands.rs). Rally point logic will need to integrate with the HQ production tick system — check how the existing production flow in faction.rs handles unit spawning.

### [automatic_qa] 2026-03-19T07:05:18Z

Automated QA feasibility: Tunnel interface [auto] steps (tier checks, expansion tier gating, ejection queue timing, state transitions on Escape/right-click) are automatable via ECS state verification. The underground walkability bug (item 2) and HQ production (item 3) QA steps are all [human] — correct routing. Enter command [auto] steps (faction check, tier validation, unit pool management) are deterministic and testable. No automated QA concerns.
