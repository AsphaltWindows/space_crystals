# gdo-power-plant-verification

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-gdo-power-plant.md

## Task

Verify that the PowerPlant structure implementation matches the design spec. The PowerPlant appears to be fully implemented already. Verify the following are correct and complete:

1. **ObjectEnum::PowerPlant** exists with correct ObjectType (name="Power Plant", size=(2,2), destructible=true, sight_range=3, groupable=true)
2. **StructureType** has correct armor (point_armor=1, full_armor=4) and symmetry (AAAA)
3. **Constants** are correct: PP_MAX_HP=350, PP_POINT_ARMOR=1, PP_FULL_ARMOR=4, PP_BUILD_RADIUS=1, PP_POWER=20
4. **spawn_power_plant()** spawns with all required components: ObjectInstance, StructureInstance, PowerValue(20), BuildRadiusExtension(1), SightRange(3), Owner, Selectable, GridPosition
5. **DeploymentCenter construction**: cost=150 SC, build_frames=160, cancellation refunds correct
6. **ConstructionHP rule**: starts at 10% HP, scales linearly, ConstructionHP component removed on completion
7. **Power grid integration**: compute_power_grid system tracks PowerValue, has_power_plant flag updated
8. **No ObjectInterfaceState**: command panel shows no commands for PowerPlant (info display only)
9. **Build area extension**: BuildRadiusExtension(1) expands GDO build area
10. **Tests pass**: run existing tests to confirm everything works

If everything checks out, no code changes are needed — just confirm the implementation is complete and correct. If any gaps are found, fix them.
