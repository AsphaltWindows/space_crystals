# test_unit_spawner_all_bases

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-unit_bases_movement_collision_r1.md

## Task

Expand `spawn_test_units` in `game/units/systems/core.rs` to spawn one properly-configured representative unit for each of the 9 UnitBaseEnum variants, so QA can test all movement models and associated mechanics.

Currently spawns: 4 Peacekeepers (LightInfantry) + 3 placeholder enemy units (WheeledVehicle, TrackedVehicle, DrillUnit) with wrong ObjectEnum and placeholder stats.

Required changes:
1. For Player 0 (GDO), ensure at least a Peacekeeper (LightInfantry) is spawned — already done.
2. For Player 1 (Syndicate), ensure Agent (HeavyInfantry) and Guard (HeavyInfantry) are spawned — use `spawn_syndicate_agent` and `spawn_syndicate_guard` from `game/utils.rs`.
3. Replace the 3 placeholder enemy units with properly-configured test units covering the missing base types. For each, attach the correct movement param component:
   - **WheeledVehicle**: `FixedTurnRadiusMovementParams` component (use values from `UnitBaseEnum::WheeledVehicle.data()` movement model)
   - **TrackedVehicle**: `SpeedTurnRadiusMovementParams` component
   - **HoverVehicle**: `DragMovementParams` component (ground domain)
   - **Mech**: `TurnRateMovementParams` component + turret components (`TurretCommandState`, `TurretBehaviorState`, `TurretOrientationChannel`, `TurretAttackChannel`, `Turret`) + directional armor
   - **Glider**: `GliderMovementParams` component (air domain)
   - **DrillUnit**: keep, ensure underground domain
4. Each test unit must have: correct `UnitBaseEnum`, `DomainEnum`, `Armor` (with `directional_armor` from base data), `Silhouette`, movement params, `Velocity`, `CommandQueue`, `BaseCommandState`, `BaseBehaviorState`, channel components, `UnitControlCost`.
5. For air units (HoverCraft already exists via SupplyChopper, Glider), ensure `SeparationRadius` component is attached.
6. Spawn units at distinct grid positions so they don't overlap.

Note: HoverCraft (air drag movement) is already testable via SupplyChopper — no need to add another.
