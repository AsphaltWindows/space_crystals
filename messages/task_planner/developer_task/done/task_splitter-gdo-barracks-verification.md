# gdo-barracks-verification

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-gdo-barracks.md

## Task

Verify the existing Barracks implementation matches the design spec. All components appear to be fully implemented:

- **BarracksState** component (structures.rs): rally_point (RallyTarget), build_queue (Vec, max 5), current_build, current_build_progress
- **spawn_barracks()** (utils.rs): 3x2 grid, ABAC symmetry, BK_MAX_HP=300, PowerValue(BK_POWER=-30), BuildRadiusExtension(BK_BUILD_RADIUS=2), ConstructionHP
- **barracks_production_tick_system** (faction.rs): ticks build progress with power_ratio, spawns Peacekeeper at B-side exit, calls issue_rally_command()
- **issue_rally_command()** (faction.rs): ground→Move, enemy→Attack, friendly/neutral→Move resolution
- **Command panel** (command_panel.rs): BarracksMenu grid — Q=BkTrain(Peacekeeper), X=BkCancel, C=SetRallyPoint
- **production_rally_point_system** (faction.rs): right-click ground/object sets rally point from BarracksMenu state
- **DC construction**: 200 SC, 160 frames, ObjectEnum::Barracks in DC build menu at W slot
- **Peacekeeper production cost**: 50 SC, 80 frames

Verification checklist:
1. Confirm BK_MAX_HP=300, BK_POINT_ARMOR=1, BK_FULL_ARMOR=6 constants match spec
2. Confirm SightRange=4 on spawned entity
3. Confirm Groupable=true (ObjectEnum::Barracks.object_type().groupable)
4. Confirm queue max size is 5 (BarracksState::MAX_QUEUE_SIZE)
5. Confirm rally point destroyed-object cleanup (check if rally_targets.get() failure causes idle spawn — it does via the match arm returning early)
6. Run existing tests (barracks_train_peacekeeper_at_q, bk_production_cost_peacekeeper, dc_construction_cost_barracks, etc.)

If any values don't match the spec, fix them. If all match, this task is complete with no code changes needed.
