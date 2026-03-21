# supply-chopper-behaviors

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-supply-chopper.md

## Task

Implement behavior systems for SupplyChopper PickUpSupplies and AttachToTower commands, including supply transfer, attachment management, and repair.

### What already exists
- UnitCommand::PickUpSupplies(Entity) and UnitCommand::AttachToTower(Entity) are issued by right-click and AwaitingTarget resolution (core.rs)
- SupplyChopperState { carried_supplies, attached_tower } in game/types/structures.rs
- SupplyTowerState { attached_chopper, ... } in game/types/structures.rs
- SupplyDeliveryStation { current_supplies, ... } in game/world/types.rs
- MovingToObjectBehavior pattern exists for reference (units/systems/behaviors.rs)
- Free chopper spawns on ST placement (faction.rs ~line 1420) but attachment is NOT linked (supply-tower-placement-attach-chopper is a separate task)

### What needs to be implemented

1. **PickUpSupplies behavior system** (`supply_chopper_pickup_system`):
   - When a chopper has UnitCommand::PickUpSupplies(target), move toward the target SDS (use existing movement infrastructure — set MoveTarget to SDS position)
   - On arrival (within threshold distance), transfer all `current_supplies` from the SDS to the chopper's `carried_supplies` (set SDS current_supplies to 0)
   - After pickup, if the chopper has an attached_tower, automatically issue a move command back to the tower location

2. **AttachToTower behavior system** (`supply_chopper_attach_system`):
   - When a chopper has UnitCommand::AttachToTower(target), move toward the target SupplyTower
   - On arrival, set `chopper_state.attached_tower = Some(tower_entity)` and `tower_state.attached_chopper = Some(chopper_entity)`
   - If the chopper was already attached to a different tower, clear the old tower's attached_chopper first

3. **Supply dropoff on tower landing**: When an attached chopper arrives at its attached tower with carried_supplies > 0, transfer carried_supplies to the player's GdoPlayerResources (add to space_crystals), then set carried_supplies = 0.

4. **Detachment on command**: When any command is issued to an attached chopper (any UnitCommand insert), break the attachment: clear chopper_state.attached_tower and the tower's attached_chopper. This can be done in the command_to_state_system or as a separate small system that watches for command changes on choppers with attached_tower.

5. **Repair while landed/attached**: When a chopper is attached to its tower and idle (no active command/movement), gradually restore HP at no cost. A simple tick system: if chopper has attached_tower, is at the tower location, and HP < max, increment HP by a small amount per frame (e.g., 1 HP/frame or similar reasonable rate).

6. **Register systems**: Register all new systems in the appropriate plugin (game/units/mod.rs or game/world/mod.rs).

7. **Tests**: Test supply pickup (SDS current_supplies transferred to carried_supplies), supply dropoff (carried_supplies → player resources), attachment/detachment logic, and repair ticking.
