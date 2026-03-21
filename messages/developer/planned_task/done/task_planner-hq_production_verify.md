# hq-production-verify

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-hq-production-interface.md

## Task

**Verification task — HQ production interface is already implemented.**

The Headquarters production interface is fully implemented. This task requires verification that all pieces work correctly together:

### Already implemented (do NOT re-implement):
1. **HeadquartersState** (structures.rs): BuildQueue (max 5), CurrentBuild, CurrentBuildProgress, RallyPoint, production_cost() for Agent (100 SC) and Guard (125 SC), try_queue(), cancel_last(), has_cancellable()
2. **HeadquartersMenu** (command_panel.rs): Grid slots Q(0,0)=BuildAgent, W(0,1)=BuildGuard, X(2,1)=CancelProduction (conditional on queue), C(2,2)=SetRallyPoint
3. **HqTrain/HqCancel** execution: Crystal deduction, queue management, cost refund on cancel
4. **headquarters_production_tick_system** (faction.rs): Processes build queue, spawns Agent/Guard at parent Tunnel Side A, rally point eject logic
5. **production_rally_point_system** (faction.rs): Right-click sets rally point on selected HQ
6. **set_rally_point_click_system** (core.rs): AwaitingTarget(SetRallyPoint) left-click flow
7. **Object routing**: HQ routes to StructureMenu(HeadquartersMenu), not unit commands
8. **Availability checks**: Queue full and insufficient crystals checks in HqTrain execution

### Verification steps:
- Confirm `cargo test` passes (all existing HQ tests in command_panel.rs, structures.rs, faction.rs)
- Confirm no unit commands (Move, Attack, Stop, etc.) appear when HQ is selected
- If any test failures or minor wiring issues are found, fix them
- If everything passes, report task complete with no changes needed

## Technical Context

### Files to verify (no changes expected):

1. **`artifacts/developer/src/game/types/structures.rs`** — HeadquartersState struct with build_queue, current_build, current_build_progress, rally_point fields. Contains production_cost(), try_queue(), cancel_last(), has_cancellable() methods. ~20 HQ unit tests starting at line 1665.

2. **`artifacts/developer/src/ui/command_panel.rs`** — HeadquartersMenu grid layout at line 105: (0,0)=HqTrain(SyndicateAgent), (0,1)=HqTrain(SyndicateGuard), (2,1)=HqCancel (conditional on queue), (2,2)=SetRallyPoint. HqTrain execution at line 1147 deducts crystals and calls try_queue(). HqCancel at line 1174 calls cancel_last() and refunds crystals. ~20 HQ menu tests starting at line 3401.

3. **`artifacts/developer/src/game/world/faction.rs`** — headquarters_production_tick_system at line 372 processes build queue and spawns units. production_rally_point_system at line 550 handles right-click rally point. ~10 HQ tests starting at line 1970.

4. **`artifacts/developer/src/game/units/systems/core.rs`** — set_rally_point_click_system at line 623 handles AwaitingTarget(SetRallyPoint) left-click to place rally point marker.

5. **`artifacts/developer/src/ui/types.rs`** — StructureMenuState::HeadquartersMenu variant at line 215. CommandButtonAction::HqTrain and HqCancel at lines 294-296.

### System registration (already wired):
- `artifacts/developer/src/game/world/mod.rs` line 90: production_rally_point_system registered
- `artifacts/developer/src/game/world/mod.rs` line 95: headquarters_production_tick_system registered
- `artifacts/developer/src/game/units/mod.rs` line 45: set_rally_point_click_system registered

### Running tests:
```bash
cd artifacts/developer && cargo test
```
All HQ-related tests are spread across structures.rs (~20 tests), command_panel.rs (~20 tests), and faction.rs (~10 tests). Look for test names containing 'headquarters' or 'hq' or 'rally_point'.

### What to check for unit command isolation:
In command_panel.rs, the HeadquartersMenu match arm (line 105) only maps HqTrain/HqCancel/SetRallyPoint — no Move/Attack/Stop/Enter etc. The update_command_panel_state function (line ~377) forces StructureMenu(HeadquartersMenu) when HQ is selected, preventing unit commands from appearing.

## Dependencies

None — this is a standalone verification task. All HQ production code is already implemented and registered.
