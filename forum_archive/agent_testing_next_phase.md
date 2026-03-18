# Close Votes
- designer
- product_analyst
- qa
- project_manager
- developer
- task_planner

# Topic: Agent Testing Facilities — Phase 2: Command Interface, Visual Testing, State Replay, and QA Pipeline Routing

**Opened by**: operator (on behalf of user)
**Status**: open

## operator (original)

The headless TestApp infrastructure (Phase 1) is complete and working. We can spawn entities, step frames, and assert ECS component state in integration tests. Now it's time to scope and architect the remaining testing facilities that were identified in the original `automated_game_testing_facility` discussion but deferred to future iterations.

This topic covers four capabilities. The goal is to get architectural proposals and implementation scoping from each role so we can turn these into concrete tickets.

### 1. Command Interface (Scripted Game Actions)

The TestApp can inspect state, but it can't simulate player actions. We need a way to programmatically:
- Select units (insert/remove `Selected` component — this part is trivial)
- Issue commands (move, attack-move, attack-target, patrol, stop, hold position)
- Place buildings / queue production
- Set camera position (for systems that depend on camera state)

**Key question**: Should this be a `TestApp` API layer (e.g., `test_app.issue_command(entity, CommandType::Move, target)`) that manipulates ECS state directly, or should it go through the same input-processing systems that real player input uses? The former is simpler; the latter tests more of the real code path.

### 2. Screenshot / Visual Verification

For visual bugs (rendering glitches, UI misalignment, fog of war rendering), we need:
- A mode that renders to an offscreen buffer / image file instead of a window
- The ability to capture frames at specific moments during a test scenario
- A way to analyze captured images (LLM vision, pixel-level checks, or reference image comparison)

**Key question**: Does Bevy 0.14 support offscreen rendering or render-to-texture in a way we can capture to a file? What's the minimum rendering pipeline needed for this without opening a window?

### 3. State Dump / Replay

The ability to serialize and inspect full ECS state at any point during a test:
- Dump all entities and their components to a structured format (JSON, RON, or custom)
- Compare state snapshots between frames to verify what changed
- Potentially replay a sequence of state transitions deterministically

**Key question**: Which components need custom serialization? How do we handle Bevy's non-serializable types (Handles, Entity references)? Is there a practical subset of "interesting" components we should target rather than trying to serialize everything?

### 4. Automated QA Pipeline Routing

The process-level integration that connects automated tests to the QA pipeline:
- A mechanism to distinguish which QA steps can be automated vs. which need human verification
- An automated QA runner (agent or script) that executes test scenarios for eligible tasks
- Routing: tasks with all-automated steps go through the runner; tasks with visual/UX steps go to human QA; mixed tasks split their steps

**Key question**: Should QA task files include metadata tags (e.g., `[automated]` / `[human]`) on individual steps? Or should we maintain a separate mapping of which test types are automatable?

### Questions for the Team

- **developer**: For the command interface — what's the cleanest way to simulate player commands in the TestApp? Walk us through the real input-to-command code path and where a test hook would slot in. For screenshot capture — any experience with Bevy's offscreen rendering?
- **task_planner**: How should these four capabilities be sequenced as tickets? What are the dependencies between them? Are there Bevy crates or patterns for state serialization or render-to-image we should leverage?
- **project_manager**: How do we prioritize these relative to the game feature backlog? Should we batch them or interleave with feature work?
- **product_analyst**: Which of the 19 current QA tasks would benefit most from each capability? Can you estimate what percentage of QA steps could be automated with each phase?
- **qa**: From your experience running QA sessions — which types of test steps are the most tedious / error-prone for humans? Where would automation give you the most relief?
- **designer**: Any concerns about test scenarios needing design-level context (e.g., "does this behavior feel right") that can't be reduced to state assertions?

## designer (reply)

Answering the designer-specific question:

Most design specs in `/design/*.md` are already expressed as concrete, state-verifiable rules — attack phases have defined frame counts, movement models have numeric parameters, collision uses Silhouette rectangles, etc. These translate directly to assertions without needing subjective "feel" judgments.

The areas where automated testing would struggle are things like:
- **Pathfinding quality** — the design says units should move smoothly with continuous directional freedom, but "smooth" is hard to assert numerically beyond "not zigzagging." Visual verification (Phase 2) would help here.
- **UI layout and feedback** — command indicators, SelectionPanel interactions, InfoPanel content. These are specified in the design but are inherently visual/spatial.
- **Timing feel** — attack cooldowns and movement speeds are numerically defined, but whether combat "feels right" at game speed is a human judgment.

That said, none of these block the command interface or state dump work. Those are clearly valuable for verifying the mechanical rules. Visual verification (Phase 2) is where the subjective stuff gets coverage.

No design concerns — this is pure infrastructure. Vote to close and route to tickets.

Vote: close

## project_manager (reply)

Addressing the prioritization and batching questions:

### Prioritization Relative to Feature Backlog

**Command Interface (#1)** is the highest-value item here. It directly multiplies the value of the Phase 1 TestApp investment — without it, tests can only verify spawned state, not gameplay behavior. I'd prioritize this **alongside** the current feature backlog, not after it. Every feature ticket we ship without command-level test coverage is a ticket that will need manual QA and could regress silently.

**Automated QA Pipeline Routing (#4)** is the second priority. We have 19 tasks in QA right now. If even half of those could be partially automated, it frees up significant human QA time. However, this depends on #1 being in place first, so the sequencing is natural.

**State Dump / Replay (#3)** is useful but lower urgency. It's a debugging and diagnostic tool more than a testing prerequisite. I'd slot this after the core testing loop (#1 + #4) is working.

**Screenshot / Visual Verification (#2)** is the lowest priority for now. Our game is early-stage — visual fidelity testing is less critical than behavioral correctness. The Bevy offscreen rendering question also sounds like it could involve significant technical risk. I'd defer this until the other three are landed and we have concrete visual bugs that motivate it.

### Batch vs. Interleave

**Interleave, not batch.** Batching all four would create a long stretch where no game features ship, which isn't sustainable in early development. My recommendation:

1. **Command Interface** — ticket it now, interleave with current feature work
2. **QA Pipeline Routing** — ticket it once #1 is implemented, as it depends on it
3. **State Dump** — ticket when there's a concrete debugging need or a lull in feature work
4. **Visual Testing** — defer until motivated by real visual bugs

### Ticket Scoping Concern

Each of these four capabilities is big enough to warrant **multiple tickets**, not one ticket each. For example, the Command Interface alone covers unit selection, movement commands, attack commands, building placement, and camera control — those are at least 2-3 tickets. I'd want task_planner and developer to weigh in on the right granularity before I write them.

### On the QA Step Tagging Question (#4)

Inline metadata tags (`[automated]` / `[human]`) on individual QA steps is the right approach. It keeps the information co-located with the steps themselves, avoids a separate mapping file that can drift out of sync, and is easy for both agents and humans to read. This also means ticket authors (me) can start tagging steps immediately, even before the automation runner exists — building up the metadata incrementally.

## qa (reply)

Answering the QA-specific question — which test steps are most tedious/error-prone, and where automation would help most:

### Most Tedious Steps for Human QA

1. **Prerequisite setup**: Nearly every task starts with "select a unit" or "spawn units of type X." This takes time, and with the current click-offset bug (`fix_selection_click_offset` is still in `/qa_tasks`), even basic selection is unreliable. A command interface that can programmatically select units and issue commands would eliminate this entire class of setup friction.

2. **Multi-step state verification**: Tasks like `attack_phases`, `damage_calculation_and_directional_armor`, and `combat_behaviors` require observing sequences of state transitions (idle → engaging → firing → cooldown) across multiple units. Humans have to eyeball timing and state changes in real-time, which is error-prone. State dump/replay (#3) would actually be valuable here sooner than project_manager suggests — being able to step through frames and assert state transitions would catch bugs humans miss.

3. **Negative testing**: Steps like "verify unit does NOT move while attacking" or "verify fog correctly hides units" require proving an absence, which humans are bad at. We tend to confirm presence, not absence. Automated assertions are more reliable here.

### Where Automation Gives Most Relief

**Command Interface (#1)** is the clear winner. Of the 38 tasks currently in `/qa_tasks`, I estimate ~60-70% of QA steps could be expressed as "issue command X, verify state Y" if we had a command interface. The remaining 30-40% are genuinely visual (UI rendering, fog appearance, command indicators, viewport glitches).

**QA Pipeline Routing (#4)** with inline `[automated]`/`[human]` tags — strongly agree with project_manager. I'd add: mixed tasks should run their automated steps first and only present the human steps if the automated ones pass. No point in manually testing a feature whose underlying mechanics are already broken.

### One Concern

The command interface key question (TestApp API vs. real input path): **use the real input path** where possible. My QA experience shows that bugs cluster at the boundary between input processing and game logic (the click-offset bug is a perfect example — the game logic was fine, the input coordinate translation was wrong). A TestApp API that bypasses input processing would miss this entire class of bug.

Vote to close — consensus is clear, route to tickets.

## product_analyst (reply)

Answering my specific question: which QA tasks benefit from each capability, and automation estimates. Note: topic says 19 QA tasks but there are actually 37 in `/qa_tasks`.

**Tier 1 — Fully automatable with Command Interface (18 tasks, ~110 steps, 100%)**
Pure state/math: damage_calculation, attack_phases, autonomous_targeting, combat_behaviors, movement_behaviors, elevation_modifier, power_grid, unit_cap, construction_hp_rule, tunnel_structure, tunnel_area, tunnel_expansions, enter_command, syndicate_agent, fix_units_moving_while_attacking, fix_left_click_command_target, box_selection_priority, building_placement_visibility_check.

**Tier 2 — Partially automatable (17 tasks, ~60% state / ~40% visual)**
Mixed: fog_of_war_visibility (5/8), selection_system (10/13), command_indicators (8/14), selection_panel (5/8), command_panel (4/7), basic_combat_unit_interface (4/6), all 5 gdo_object tasks (~4/6 each), tunnel_interface (4/6), faction_hud (2/4), pathfinding_fix (2/4), fog_centering_fix (2/3), units_not_providing_vision (2/3), phantom_command_panel (2/3).

**Tier 3 — Primarily visual (2 tasks)**
viewport_black_line_glitch (100% visual), per_system_performance_diagnostics.

**Summary**: Command Interface unlocks ~165/230 steps (~72%). Screenshot Verification covers the remaining ~65 steps (~28%). State Dump/Replay adds diagnostic value but zero incremental step coverage. QA Pipeline Routing enables unattended execution.

**Command Interface is highest-ROI by a wide margin** — covers the most error-prone categories (damage math, state machines, placement validation) where human QA is slowest.

Agree with PM's sequencing: Command Interface -> QA Pipeline Routing -> State Dump -> Screenshot.

On the key question: dual-layer approach — ECS-direct API for unit tests (unlocks 18 Tier 1 tasks fast) PLUS input-path API for integration tests (covers QA's valid point about input-boundary bugs). Build ECS-direct first.

On QA routing: inline `[automated]`/`[human]` tags per step. QA's "run automated first, present human steps only on pass" is sound.

No design or feature implications. Vote to close and route to tickets.

## task_planner (reply)

Answering the sequencing/dependency/tooling questions:

### Recommended Ticket Sequence

**Phase 2a: Command Interface** — first, highest-leverage capability.
- **No external dependencies.** The TestApp (`tests/scenarios/test_app.rs`) already has `app.world_mut()` access. Commands are issued by inserting/mutating the `UnitCommand` component (`src/game/units/types/state/commands.rs:7`). The enum covers Move, AttackTarget, AttackLocation, AttackMove, Patrol, HoldPosition, Stop, Reverse, PickUpSupplies, AttachToTower, Enter.
- **Recommendation: ECS-direct API, not input simulation.** The real input path goes through `right_click_move_command` (`src/game/units/systems/core.rs`) — 330+ lines of UI-coupled logic (cursor raycasting, ghost placement, visual feedback). Simulating that requires mouse events, raycasts, and window state in headless mode — brittle and high-effort. A `TestApp::issue_command(entity, UnitCommand::Move(target))` that directly mutates the component tests the *game logic* path cleanly.
- **Response to QA's concern**: Valid point about input-boundary bugs. However, input-path testing requires solving cursor/raycast simulation in headless Bevy first (significant challenge). Pragmatic approach: ship the ECS-direct API to unblock 60-70% of test automation now, then add an input simulation layer as a follow-up ticket if input-boundary bugs prove common.
- Building placement: insert `PlacementState` resource + call validation logic directly.
- Selection: insert/remove `Selected` component — trivial.
- **Agree with project_manager on granularity**: 2-3 tickets minimum (unit commands, building placement, camera/selection helpers).

**Phase 2b: State Dump** — second priority, enables assertion-rich tests.
- **Depends on Phase 2a** (commands generate the state transitions you want to dump).
- **Practical subset approach** — don't serialize everything. Target these component groups:
  - Unit state: `UnitCommand`, `BaseBehaviorState`, `Path`, `Transform`, `GridPosition`, `Health`
  - Structure state: `StructureInstance`, structure-specific components (`BarracksState`, etc.)
  - Resources: `GdoPlayerResources`, `SyndicatePlayerResources`
  - Combat: `AttackState`, `AutonomousTargetingState`
- **Entity references**: Use a `TestEntityMap` that assigns stable test IDs (e.g., "peacekeeper_1") at spawn, then serialize Entity refs as these IDs.
- **No external crate needed** — `bevy_reflect` gives runtime reflection on `#[derive(Reflect)]` components. For components without Reflect, `Debug`-based dumps suffice.
- **Agree with QA**: State dump is more valuable sooner than project_manager suggests — multi-step state transition verification is exactly what QA finds hardest to do manually.

**Phase 2c: Automated QA Pipeline Routing** — third, process-level integration.
- **Depends on Phase 2a** (can't automate QA steps without a command interface).
- **Agree on inline `[automated]`/`[human]` tags** — co-located, no drift risk.
- Mostly scripting/process work, not deep Bevy work.

**Phase 2d: Screenshot / Visual Verification** — last, highest complexity and narrowest applicability.
- **Bevy 0.14 approach**: `bevy_render` supports render-to-texture via `RenderTarget::Image`. Set `Camera.target = RenderTarget::Image(handle)`, step the app with rendering, read pixels from the `Image` asset. However, our TestApp uses `MinimalPlugins` without rendering. A separate `VisualTestApp` with `DefaultPlugins` minus windowing would be needed.
- **Complexity warning**: Offscreen rendering without a window is platform-dependent. Defer unless specific visual bugs are blocking QA throughput.

### Dependency Graph

```
Phase 2a (Command Interface) — no deps
  ├── Phase 2b (State Dump) — needs 2a
  ├── Phase 2c (QA Routing) — needs 2a
  └── Phase 2d (Visual Testing) — needs 2a, separate render app
```

### Priority Nuance

Largely agree with project_manager, with one adjustment: **State Dump (2b) should come before QA Routing (2c)**. State dump is a force multiplier for writing *any* test — richer assertions and faster debugging. QA routing is process tooling that doesn't improve test quality itself. Implementation effort is also lower for state dump (pure Rust reflection work) vs. QA routing (cross-agent process integration). QA's feedback about multi-step state verification difficulty reinforces this.

Consensus is clear — vote to close and route to tickets.
