# Designer Session Log

## 2026-03-20T10:00:00Z — Interactive Session: Pointer Display Types

**Trigger**: User session, picked up loose end on pointer display types.

**Work done**:
- Resumed A/B/C pointer approach question from previous session. User chose Communicative (B) — pointer always shows what action will happen.
- User defined 6 pointer types: Move, Attack, AttackGround, Patrol, GatherResources, ReturnResources.
- Added 2 more through discussion: Inactive (muted, for no-action states) and Enter (Syndicate tunnel entry).
- Key decisions: Attack on ground = AttackMove (not AttackGround); AwaitingPlacement uses building ghost as cursor (no pointer type); ScheduleDeliveries reuses GatherResources pointer; Inactive is visually muted.
- Added PointerDisplayType section to control_system.md with full resolution rules for DefaultState, AwaitingTarget, and AwaitingPlacement.
- Sent feature_request `pointer_display_types` to task_splitter.
- Updated insights: resolved pointer display types loose end, cleared both urgent forum questions (Supply Tower invalid target now covered by Inactive pointer).

## 2026-03-19T10:30:00Z — Non-Interactive Forum Pass

**Trigger**: Scheduler launch — 6 open forum topics needing close-vote.

**Actions taken**:

1. **`dc-ef-construction-submenu-rework`** — Commented. Identified 3 items matching existing design (auto-enter bug, EF flat interface, progress bar) and 1 new design element (DC DefaultState Cancel at slot (2,1)). Flagged DC Cancel as urgent forum question.

2. **`syndicate-agent-core-gameplay`** (6 items) — Commented. All 6 items match existing design docs exactly: Agent interface, spawn bugs, groupable, resource gathering, tunnel building, arrival validation. Flagged as ready for feature_requests pending user confirmation.

3. **`syndicate-tunnels-underground`** (5 items) — Commented. All 5 items match existing design: Tunnel interface, underground walkability bug, HQ production bug, Enter command, rally point behavior. Flagged as ready for feature_requests pending user confirmation.

4. **`unit-control-selection-ui`** (3 items) — Commented. BasicCombatUnitInterfaceState and SelectionPanel match existing design. Command classification is a bug fix. Identified new design element: specific grid slot layout for combat commands. Flagged grid layout as urgent forum question.

5. **`gdo-structures-guard-unit`** (3 items) — Commented. Supply Tower and Guard match existing design. DC Cancel is same new design element from topic 1. Flagged.

6. **`visual-bugs-qa-infrastructure`** (2 items) — Voted to close. Both items are purely technical (visual debugging, QA tag maintenance) with no design input needed.

**Insights updated**: Created full TOC, 3 urgent forum questions, 16 pending feature_requests, 3 loose ends.

**Feature requests produced**: 0 (non-interactive mode — all pending user confirmation in next interactive session).

## 2026-03-19T11:00:00Z — Non-Interactive Forum Pass

**Trigger**: Scheduler launch — 6 open forum topics needing designer close-vote.

**Actions taken**: Voted to close all 6 topics. Designer had already commented substantively on all topics in the previous session. All analysis is captured in insights (3 urgent forum questions, 16 pending feature_requests, 3 loose ends). No new design elements or forum questions identified since last pass — the topics had no new comments requiring designer response.

**Topics closed** (designer was last needed vote on all 6):
1. `dc-ef-construction-submenu-rework`
2. `syndicate-agent-core-gameplay`
3. `syndicate-tunnels-underground`
4. `unit-control-selection-ui`
5. `gdo-structures-guard-unit`
6. `visual-bugs-qa-infrastructure`

**Feature requests produced**: 0 (blocked on user confirmation in next interactive session).

## 2026-03-19T12:00:00Z — Non-Interactive Forum Pass

**Trigger**: Scheduler launch — 1 open forum topic needing close-vote.

**Actions taken**: Voted to close `2026-03-19T00-00-00Z-operator-telegram-integration-successful.md` — informational announcement about Telegram integration, no design input needed.

**Feature requests produced**: 0.

## 2026-03-19T13:00:00Z — Non-Interactive Forum Pass

**Trigger**: Scheduler launch — 1 open forum topic needing close-vote.

**Actions taken**: Voted to close `2026-03-19T00-00-00Z-operator-telegram-integration-successful.md` — informational Telegram announcement, no design input needed. Developer had already voted; designer vote should close the topic.

**Feature requests produced**: 0.

## 2026-03-19T14:00:00Z — Interactive Session

**Trigger**: User session — answering urgent forum questions.

**Actions taken**:

1. **DC DefaultState Cancel (resolved)**: User confirmed Cancel (X) should appear in both DefaultState and BuildMenu. Updated `gdo_objects.md` — added X: Cancel Construction and X: Cancel Ready Building to DC DefaultState. Sent feature_request `dc-defaultstate-cancel`.

2. **BasicCombatUnit Grid Layout (resolved)**: User provided revised layout. Key decisions: AttackMove is not a separate command (it's Attack targeting ground), Stop on X, Reverse on W, HoldPosition on E, Attack on A, Patrol on S, AttackGround on D. Updated `control_system.md` — added grid layout and hotkey assignments to BasicCombatUnitInterfaceState. Sent feature_request `combat-unit-grid-layout`.

3. **Supply Tower Error Feedback (pinned)**: User wants invalid pointer display type for invalid targets, but pointer display types haven't been designed yet. Added to loose ends.

**Insights updated**: Removed resolved urgent questions (#1, #2). Updated #3 with user's answer. Added pointer display types to loose ends. Unblocked pending feature_requests #11 and #14.

**Feature requests produced**: 2 (`dc-defaultstate-cancel`, `combat-unit-grid-layout`).

## 2026-03-19T15:00:00Z — Interactive Session

**Trigger**: User session — sending pending feature_requests into the pipeline.

**Actions taken**:

User confirmed sending all 16 pending feature_requests from insights into the pipeline. Noted overlap with 2 already-completed feature_requests (`designer-combat-unit-grid-layout`, `designer-dc-defaultstate-cancel`) in the relevant messages so downstream agents can avoid duplication.

**Feature requests produced** (16 total):
1. `agent-interface` — Agent ObjectInterfaceState (Build Tunnel, Drop Off, right-click resolution)
2. `agent-groupable-construction` — Agent Groupable=false, single-agent construction enforcement
3. `agent-resource-gathering` — Agent crystal/supply gathering and drop-off behaviors
4. `agent-tunnel-building` — Agent Tunnel construction flow, ConstructionHP, 480 frames
5. `worker-built-validation` — Two-phase placement validation for worker-built structures
6. `tunnel-interface` — Tunnel 4-state ObjectInterfaceState (Upgrade/Expand/Eject/AwaitingPlacement)
7. `underground-surface-walkability` — Fix: underground expansions don't block surface movement
8. `hq-production-interface` — Fix: HQ shows production commands, not unit commands
9. `enter-command-tunnel` — Enter command and EnteringTunnel behavior
10. `syndicate-rally-point` — Rally point behavior for Syndicate production expansions
11. `basic-combat-unit-interface` — Full BasicCombatUnitInterfaceState (right-click + AwaitingTarget resolutions, notes prior grid layout overlap)
12. `selection-panel` — SelectionPanel portrait grid with click interactions
13. `common-vs-group-commands` — Fix: CommonCommand vs GroupCommand classification
14. `dc-ef-construction-rework` — DC/EF construction flow rework (notes prior DC Cancel overlap)
15. `supply-tower-interface` — Supply Tower ObjectInterfaceState (Q/S/X/C, Schedule Deliveries)
16. `guard-unit` — Guard unit full implementation (stats, production, BasicCombatUnitInterfaceState)

**Insights updated**: Cleared pending design review section. Kept loose ends and urgent forum questions unchanged.

## 2026-03-20 — Bulk Feature Request Generation

**Context:** User requested checking which design docs hadn't been turned into feature requests and sending them all.

**Work Done:**
- Audited all existing feature requests (9 done, 9 pending) against all 12 design doc files
- Identified 26 design areas not yet covered by feature requests
- Sent 26 new feature requests to task_splitter:

1. `scale-camera-system` — SimulationFrame (16 FPS), GridUnit, SpaceUnit, fixed 28 GridUnit camera, HUD layout
2. `tile-terrain-system` — Tiles, TilePresets, 5 DefaultTilePresets, TilePlacement with elevation
3. `fog-of-war-elevation` — 3-state vision system, ElevationModifier (+/-1 range)
4. `resource-nodes` — SpaceCrystalsPatch (1x1, depletable), SupplyDeliveryStation (2x2, periodic)
5. `factions-resources` — All 4 factions with resource systems and HUD displays
6. `unit-bases-movement-collision` — 9 UnitBase types, 5 MovementModels, TurretAttributes, collision
7. `combat-attack-system` — AttackAttributes, 4 phases, 4 attack types, damage calculation, ValidTarget
8. `locomotion-orientation-constraints` — All 5 movement model constraint tables
9. `control-state-selection` — ControlState, Selection, BoxSelection (5-tier), ControlGroups, GroupCycling
10. `unit-command-system` — 9 commands with BaseCommandState, command queue
11. `base-behaviors` — All 9 base behaviors (movement, attack, patrol, hold, stop)
12. `turret-behavior-system` — TurretCommandState, TurretBehavior, TurretAutonomousScanning
13. `base-auto-targeting` — Idle/HoldPosition auto-engagement, 4 grid unit leash
14. `action-channels` — Base (Locomotion, Orientation, BaseAttack) + Turret (TurretOrientation, TurretAttack)
15. `command-indicators` — Visual markers with color coding
16. `peacekeeper-unit` — GDO LightInfantry, full stats
17. `gdo-power-plant` — 2x2, +20 power, ConstructionHP rule
18. `gdo-barracks` — 3x2, infantry production, rally points
19. `gdo-build-area` — Build radius expansion system
20. `extraction-plate` — 1x1 mining structure, mining/residual rates
21. `supply-chopper` — Unarmed HoverCraft, supply transport
22. `tunnel-network-mechanics` — Tiers, transit rules, area, non-overlap, cost scaling
23. `gdo-deployment-center` — 4x4, construction catalog, prerequisites
24. `gdo-extraction-facility` — 3x3, structure stats
25. `syndicate-headquarters-structure` — 2x2 T1 expansion, production catalog
26. `command-panel-framework` — 3x3 grid, standard slots, Common vs Group commands

**Noted:** gdo_objects.md is truncated at line 348, SupplyChopper AwaitingTarget section incomplete.

## 2026-03-20 — Non-Interactive (Scheduler)

- Loaded insights. No urgent forum questions requiring user input.
- Found 1 open forum topic: `2026-03-20T00-00-00Z-operator-avoid-cargo-clean.md` — operator directive to avoid `cargo clean`. Not relevant to designer (we don't build). Voted to close.
- No other work found. Exiting.

## 2026-03-20 — Non-Interactive (Scheduler)

- Loaded insights. No urgent forum questions requiring user input.
- Found 1 open forum topic: `2026-03-20T00-00-00Z-operator-avoid-cargo-clean.md` — still open, already had designer close-vote from previous session. Re-confirmed vote. No new topics or work. Exiting.

## 2026-03-20 — Interactive Session

**Trigger**: User session — completing truncated Supply Chopper design.

**Actions taken**:

1. Reviewed truncated `gdo_objects.md` — file cut off at line 349 mid-sentence in AwaitingTarget[PickUpSupplies].
2. Discussed and resolved Supply Chopper command design with user:
   - PickUpSupplies: no action on invalid target, unavailable when carrying units
   - AttachToTower: only valid on own towers with no attached chopper, unavailable when carrying units
   - New DropOffSupplies command: available when carrying supplies, valid on own towers with no attached chopper
   - Right-click resolution is state-dependent (carrying supplies vs not)
   - Drop-off at non-attached tower is touch-and-go (immediate lift off)
   - Automated scheduled departures don't break attachment; player commands do
3. Updated `gdo_objects.md` with complete Supply Chopper ObjectInterfaceState.
4. Sent feature_request `supply_chopper_commands`.

**Feature requests produced**: 1 (`supply_chopper_commands`).

## 2026-03-20 — Interactive Session

**Trigger**: User asked about outstanding work areas.

**Actions taken**:

1. Reviewed insights and design docs to compile a prioritized list of outstanding work areas for the user.
2. Presented 10 items across 4 priority tiers: blocking (pointer display types), open loose ends (alt-click camera, Agent UnitBase, tunnel transit tiers), incomplete factions (Cults, Colonists), and undesigned systems (maps, win conditions, tech trees, fog of war details).
3. User chose to work on pointer display types.
4. Framed the design space: identified 4 main contexts where pointer feedback is needed (DefaultState right-click preview, AwaitingTarget valid/invalid, AwaitingPlacement valid/invalid, non-interactive). Asked opening question about cursor approach (distinct icons vs color tinting vs both).
5. User requested recording and exit before answering the first question.

**Insights updated**: Expanded pointer display types loose end with session context and the specific open question to resume with.

**Feature requests produced**: 0 (no design decisions reached).

## 2026-03-20 — Interactive Session

**Trigger**: User asked to continue pointer display types discussion.

**Actions taken**:

1. Loaded insights, checked forum (no open topics).
2. Restated the pointer display types open question from previous session: 4 contexts identified (DefaultState right-click preview, AwaitingTarget valid/invalid, AwaitingPlacement valid/invalid, non-interactive), first decision point is distinct icons vs color tinting vs both.
3. User requested recording and exit before answering the question.

**Insights updated**: No changes — loose end remains identical.

**Feature requests produced**: 0 (no design decisions reached).

## 2026-03-20 — Interactive Session

**Trigger**: User asked what to work on next; chose pointer display types.

**Actions taken**:

1. Loaded insights, checked forum (no open topics).
2. Resumed pointer display types discussion. Refined the visual approach question into two concrete options:
   - **Option A (Minimal)**: Pointer signals valid/invalid only; player infers action from context.
   - **Option B (Communicative)**: Pointer previews the specific action (attack crosshair over enemy, move arrow over ground, enter icon over tunnel, etc.).
3. Presented both options to user. User requested recording and exit before choosing.

**Insights updated**: Updated pointer display types loose end with the A/B framing for next session.

**Feature requests produced**: 0 (no design decisions reached).

## 2026-03-20 — Interactive Session

**Trigger**: User asked what to work on next; chose pointer display types.

**Actions taken**:

1. Loaded insights, checked forum (no open topics).
2. Resumed pointer display types discussion. Presented the A/B choice from previous session, added a third option:
   - **Option A (Minimal)**: Pointer signals valid/invalid only.
   - **Option B (Communicative)**: Pointer previews specific action.
   - **Option C (Hybrid)**: Minimal for AwaitingTarget (player already chose command), Communicative for DefaultState right-click (action varies by cursor target).
3. Surfaced that Option B has significant design surface area (many distinct pointer icons needed).
4. User did not choose; requested recording and exit.

**Insights updated**: Added Option C (Hybrid) to the pointer display types loose end.

**Feature requests produced**: 0 (no design decisions reached).

## 2026-03-20 — Interactive Session: Pointer Display Follow-up

**Trigger**: User session. User reported a previous pointer display session didn't save to logs/insights.

**Actions**:
1. Loaded insights, checked forum (no open topics).
2. Investigated current state of pointer display design in `control_system.md` — found the PointerDisplayType section is fully populated (8 types, resolution rules for DefaultState, AwaitingTarget, AwaitingPlacement). Insights already mark pointer displays as RESOLVED.
3. Asked user what they thought was missing. User did not elaborate and requested recording and exit.
4. Confirmed no design doc changes needed — pointer display content appears intact.

**Insights updated**: No changes (content already captured).

**Feature requests produced**: 0 (no design changes made).

## 2026-03-20 — Interactive Session: Loose Ends Cleanup + Cults Kickoff

**Trigger**: User session.

**Actions taken**:

1. **Agent UnitBase (resolved)**: User confirmed HeavyInfantry is intentional. No doc change needed — already correct.
2. **Tunnel transit tiers (resolved)**: User confirmed Tier 1 Infantry includes both Light Infantry and Heavy Infantry. Updated `syndicate_objects.md` line 8. Sent feature_request `tunnel-transit-light-infantry`.
3. **Alt-click camera (resolved)**: User specified standard pan, currently snap (instant). Added Panning section to `camera.md`. Sent feature_request `camera-pan-snap`.
4. **Cults faction**: User wanted to start on Cults but suggested doing a lore session first. Opening question asked ("What's the setting? What are Space Crystals, and why are factions fighting?") but no answer before session end.

**Insights updated**: Cleared all three resolved loose ends. Added lore session and Cults design as new loose ends.

**Feature requests produced**: 2 (`tunnel-transit-light-infantry`, `camera-pan-snap`).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — 1 open forum topic needing close-vote.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T12-00-00Z-operator-auto-qa-capability-expansion.md` — file exists but is empty (0 bytes), malformed topic. No content to engage with as designer. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Original empty topic (`operator-auto-qa-capability-expansion`) had been commented on by automatic_qa (clearing designer's prior close-vote) and then moved to closed.
- Created informational forum topic `2026-03-21T14-00-00Z-designer-empty-forum-topic.md` documenting the malformed file. Voted to close it.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — 1 open forum topic needing close-vote.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T120000-operator-expand-automatic-qa-capabilities.md` — operator request to expand automatic QA capabilities (analyze manual QA patterns, write tests, update auto_capabilities.txt). Purely architect/developer/automatic_qa concern, no design input needed. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T120000-operator-expand-automatic-qa-capabilities.md` — same topic, now with substantive comments from task_splitter and task_planner. Still purely architect/developer/automatic_qa domain (test infrastructure, QA routing config). No design input needed. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T120000-operator-expand-automatic-qa-capabilities.md` — same QA expansion topic, now with additional comments from developer and automatic_qa. Still purely developer/architect/automatic_qa domain (test infrastructure, compilation fixes, auto_capabilities.txt). No design input needed. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T140000Z-operator-fix-broken-tests.md` — operator directive to developer to fix 37 test compilation errors. Purely developer/task_splitter/task_planner domain. No design input needed. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T140000Z-operator-fix-broken-tests.md` — same broken tests topic, now with comments from automatic_qa and task_splitter. Still purely developer domain. Re-confirmed close vote.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T140000Z-operator-fix-broken-tests.md` — same broken tests topic, now with task_planner's detailed root cause analysis. Still purely developer domain. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T140000Z-operator-fix-broken-tests.md` — broken tests topic now resolved (automatic_qa confirmed compilation fixed, 293 tests pass). Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T140000Z-operator-fix-broken-tests.md` — fully resolved (developer confirmed all 37 errors fixed, automatic_qa verified). Developer already voted to close. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Interactive Session: Lore

**Trigger**: User session — asked what to work on next.

**Actions taken**:

1. Checked forum — 1 open topic (`fix-broken-tests`) already has designer close-vote and is resolved. No action needed.
2. Presented loose ends: lore session and Cults faction design.
3. User provided background lore covering: Space Crystal origin (meteorite, self-replicating), dual properties (scientific/manufacturing + psychoactive with ego imprinting), neurotoxicity, and all 4 faction backstories.
4. Confirmed lore is background context only — psychoactive mechanics, neurotoxicity, and crystal replication are not gameplay-relevant.
5. Created `artifacts/designer/design/lore.md` with full lore writeup.
6. Updated insights: added lore.md to TOC, updated Cults loose end (no longer blocked on lore).

**Feature requests produced**: 0 (lore is narrative context, not implementable design).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T122200Z-manual_qa-build-qa-artifact-missing-diagnostics-feature.md` — manual_qa reporting missing `diagnostics` feature in Cargo.toml causing build script failure. Purely developer/build tooling issue. No design input needed. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T122200Z-manual_qa-build-qa-artifact-missing-diagnostics-feature.md` — diagnostics feature missing from Cargo.toml. Already resolved by developer, had 3 close votes. Voted to close (topic closed immediately).
- No other open forum topics. No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T122500Z-manual_qa-cannot-build-extraction-facility.md` — manual_qa reporting inability to build Extraction Facility during QA. This is an implementation bug, not a design issue. The Extraction Facility design is complete in `gdo_objects.md`. No design input needed. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics needing designer close-vote:
  1. `2026-03-21T122500Z-manual_qa-cannot-build-extraction-facility.md` — inability to build Extraction Facility. Implementation bug, design is complete in `gdo_objects.md`. Already had automatic_qa close-vote. Voted to close.
  2. `2026-03-21T122700Z-manual_qa-syndicate-camera-not-centered-on-starting-tunnel.md` — camera not centering on starting Tunnel for Syndicate. Implementation bug, camera design in `camera.md` covers panning/centering. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21T17:00:00Z — Non-Interactive: Forum Pass

**Trigger**: Scheduler launch, 2 open forum topics.

**Actions**:
- `syndicate-camera-not-centered`: Already had my close vote. No action needed.
- `cannot-build-extraction-facility`: Confirmed design gap — EF has no acquisition path in gdo_objects.md. Added comment acknowledging the gap. Flagged as urgent forum question in insights for next interactive session (requires user decision on EF cost/build_frames and whether to add to DC Constructs or spawn at start).

**Produced**: No feature_requests (design decision pending user input).

## 2026-03-21T18:00:00Z — Non-Interactive: Forum Pass

**Trigger**: Scheduler launch, 2 open forum topics.

**Actions**:
- `cannot-build-extraction-facility`: Already commented and flagged as urgent forum question. No new comments since last pass. No action needed (avoiding comment that would clear close votes).
- `syndicate-camera-not-centered-on-starting-tunnel`: Added comment identifying design gap — camera.md doesn't specify starting position. Flagged as urgent forum question in insights (need user to confirm camera centers on starting structure at game start).

**Insights updated**: Added camera starting position as second urgent forum question.

**Produced**: No feature_requests (both topics need user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both already have designer comments and are flagged as urgent forum questions in insights:
  1. `cannot-build-extraction-facility` — awaiting user decision on EF acquisition path (add to DC Constructs with cost/build_frames, or spawn at start).
  2. `syndicate-camera-not-centered-on-starting-tunnel` — awaiting user confirmation that camera should center on starting structure at game start.
- No new comments on either topic since last pass. No action taken (cannot resolve without user input, avoiding comments that would clear close votes).
- No pending messages for designer.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both already have designer comments and are flagged as urgent forum questions in insights:
  1. `cannot-build-extraction-facility` — awaiting user decision on EF acquisition path.
  2. `syndicate-camera-not-centered-on-starting-tunnel` — awaiting user confirmation on camera starting position.
- No new comments on either topic since last pass. No action taken.
- No pending messages for designer.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both unchanged since last pass. Designer comments already present, both flagged as urgent forum questions in insights:
  1. `cannot-build-extraction-facility` — awaiting user decision on EF acquisition path.
  2. `syndicate-camera-not-centered-on-starting-tunnel` — awaiting user confirmation on camera starting position.
- No pending messages for designer. No action taken.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both unchanged since last pass. Designer comments already present, both flagged as urgent forum questions in insights:
  1. `cannot-build-extraction-facility` — awaiting user decision on EF acquisition path.
  2. `syndicate-camera-not-centered-on-starting-tunnel` — awaiting user confirmation on camera starting position.
- No pending messages for designer. No action taken.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both unchanged. Designer comments already present, both flagged as urgent forum questions in insights:
  1. `cannot-build-extraction-facility` — awaiting user decision on EF acquisition path.
  2. `syndicate-camera-not-centered-on-starting-tunnel` — awaiting user confirmation on camera starting position.
- No pending messages for designer. No action taken.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both unchanged. Designer comments already present, both flagged as urgent forum questions in insights:
  1. `cannot-build-extraction-facility` — awaiting user decision on EF acquisition path.
  2. `syndicate-camera-not-centered-on-starting-tunnel` — awaiting user confirmation on camera starting position.
- No pending messages for designer. No action taken.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both unchanged. Designer comments already present, both flagged as urgent forum questions in insights:
  1. `cannot-build-extraction-facility` — awaiting user decision on EF acquisition path.
  2. `syndicate-camera-not-centered-on-starting-tunnel` — awaiting user confirmation on camera starting position.
- No pending messages for designer. No action taken.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both unchanged since previous pass. Designer comments already present, both flagged as urgent forum questions in insights. No new comments from other agents.
- No pending messages for designer. No action taken.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both with designer comments but missing designer close-vote (previous comments had cleared votes).
- Voted to close both — designer work is complete (comments added, urgent forum questions flagged in insights). Design decisions require user input in next interactive session.
  1. `cannot-build-extraction-facility` — closed (all agents voted).
  2. `syndicate-camera-not-centered-on-starting-tunnel` — closed (all agents voted).
- Urgent forum questions remain in insights for next interactive session.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T130000Z-manual_qa-enemies-dont-attack-by-default.md` — enemy units don't auto-attack nearby player units. Commented citing existing design: `control_system.md` BaseAutoTargeting (idle and HoldPosition auto-engagement), TurretAutonomousScanning, and `combat.md` ValidTarget criteria. This is an implementation gap, not a design gap. Voted to close.
- 2 urgent forum questions remain in insights (EF acquisition path, camera starting position) — both require user input in next interactive session.

**Feature requests produced**: 0 (no design changes needed).

## 2026-03-21T17:10:00Z — Non-interactive (scheduler)

- Loaded insights, checked `forum/open/` — found 2 topics.
- **enemies-dont-attack-by-default**: Already commented previously. Well-resolved as implementation gap with tasks in pipeline. Voted to close.
- **can-control-enemy-units-and-buildings**: New topic. Commented citing `control_system.md` ownership-aware selection rules and RightClickResolution. Identified a design gap: no explicit rule that CommandPanel is empty/hidden for non-owned selections. Added to Pending Design Review in insights for user confirmation.

## 2026-03-21T17:15:00Z — Non-interactive (scheduler)

- Loaded insights, checked `forum/open/` — found 2 topics.
- **enemies-dont-attack-by-default**: Already has designer close-vote. No action needed.
- **can-control-enemy-units-and-buildings**: Already commented in prior session. All agents weighed in and agree. Voted to close. Design doc update (explicit ownership guard in `control_system.md`) remains in Pending Design Review for user confirmation.
- No pending messages for designer.

**Feature requests produced**: 0 (design decision pending user input).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T130200Z-manual_qa-can-control-enemy-units-and-buildings.md` — enemy control bug. All agents have commented substantively. Designer already commented identifying the design gap (no explicit ownership guard rule for CommandPanel). Topic is well-resolved. Voted to close.
- CommandPanel ownership guard remains in Pending Design Review in insights for user confirmation in next interactive session.
- No pending messages for designer.

**Feature requests produced**: 0 (design doc update pending user confirmation).

## 2026-03-21 — Interactive Session

**Trigger**: User session — asked what to work on.

**Actions taken**:

1. Checked forum — no open topics. Presented 2 urgent forum questions and 1 pending design review from insights.

2. **Extraction Facility in DC Constructs (resolved)**: User confirmed EF should be buildable from DC (was always intended per `to_be_converted.md` line 435, missed during formalization). Added to DC Constructs: 200 SC, 320 frames (20s), no prerequisite. Also added to DC BuildMenu idle state. Sent feature_request `dc_builds_extraction_facility`.

3. **Extraction Plate power cost (resolved)**: User added Power -3 to Extraction Plates. Updated `gdo_objects.md`.

4. **Extraction Plate power penalty (resolved)**: User specified that mining rate slows under power deficit, same as buildings. Updated `factions.md` Power description to include Extraction Plates and list affected operations (construction speed, unit production speed, mining rate). Added `PowerPenalty` line to Extraction Plate in `gdo_objects.md`. Sent feature_request `extraction_plate_power_penalty`.

5. **CommandPanel ownership guard (resolved)**: User confirmed CommandPanel should only show for own units, and commands can only be issued to own objects. Updated `control_system.md` CommandPanel section with explicit ownership guard rule (no CommandPanel, no right-click resolution, no InterfaceTransitions for non-owned selections). Sent feature_request `command_panel_ownership_guard`.

**Insights updated**: Resolved EF acquisition path urgent question, resolved CommandPanel ownership guard pending review, cleaned up strikethrough entries, added new insight about cross-checking `to_be_converted.md` for missed formalizations.

**Feature requests produced**: 3 (`dc_builds_extraction_facility`, `extraction_plate_power_penalty`, `command_panel_ownership_guard`).

## 2026-03-22 — Interactive Session: Cults Armory

**Trigger**: User session — asked where we left off.

**Actions taken**:

1. Checked forum — no open topics.
2. Presented loose end: Cults faction design. User chose to add one more Cults building.
3. User specified an Armory — training building where Recruits enter and are held as an internal pool, then trained into combat units (Soldier, Gunner).
4. Collaborative design established:
   - 3x2 building, ABCB symmetry (entrance on one short side, exit on opposite, matching long sides)
   - Internal Recruit capacity: 10
   - Training consumes one internal Recruit + Space Crystals
   - Two unit types: Soldier and Gunner
   - Eject All command sends stored Recruits out the exit side in a rapid stream
   - Full ObjectInterfaceState: Rally Point (C), Eject All (E), Train Soldier (Q), Train Gunner (W)
5. Updated `cults_objects.md` with Armory section including open questions.
6. Sent feature_request `add_cults_armory`.

**Open questions deferred to next session**: Training queue (one at a time vs queueable?), cancel mid-training (Recruit back? Crystals refunded? Both?), parallel vs serial training. Soldier and Gunner unit definitions not started.

**Insights updated**: TOC updated for Armory, loose ends updated with Armory open questions and unit definition needs.

**Feature requests produced**: 1 (`add_cults_armory`).
