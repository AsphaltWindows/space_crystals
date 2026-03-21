# hq-production-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

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
8. **Availability checks**: Queue full and insufficient crystals checks in is_structure_action_available()

### Verification steps:
- Confirm `cargo test` passes (all existing HQ tests in command_panel.rs, structures.rs, faction.rs)
- Confirm no unit commands (Move, Attack, Stop, etc.) appear when HQ is selected
- If any test failures or minor wiring issues are found, fix them
- If everything passes, report task complete with no changes needed
