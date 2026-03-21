# supply-chopper-interface

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-supply-chopper.md

## Task

Add the SupplyChopper command panel interface and AwaitingTarget modes.

### What already exists
- ObjectEnum::SupplyChopper with all stats (60x60, HP=150, armor 1/1, sight=5, groupable, unarmed)
- SupplyChopperState component (carried_supplies, attached_tower) in game/types/structures.rs
- spawn_supply_chopper() in game/utils.rs with DragMovementParams
- UnitCommand::PickUpSupplies(Entity) and UnitCommand::AttachToTower(Entity) in units/types/state/commands.rs
- Right-click resolution in core.rs (~line 349): SDS→PickUpSupplies, own ST→AttachToTower
- command_to_state mapping maps both to CommandType::Default (commands.rs ~line 215)
- object_type_supports_action already excludes attack actions for SupplyChopper (command_panel.rs ~line 2176)

### What needs to be implemented

1. **CommandType variants**: Add `CommandType::PickUpSupplies` and `CommandType::AttachToTower` to the CommandType enum in units/types/state/commands.rs.

2. **Command-to-state mapping**: Update commands.rs line 215-216 so UnitCommand::PickUpSupplies maps to CommandType::PickUpSupplies (not Default), and UnitCommand::AttachToTower maps to CommandType::AttachToTower (not Default).

3. **SupplyChopper command panel grid**: When a SupplyChopper is the active selection group, show the following 3x3 grid in command_panel.rs:
   - (0,0) Q = Move → enters AwaitingTarget[Move]
   - (0,1) W = Pick Up Supplies → enters AwaitingTarget[PickUpSupplies]
   - (0,2) E = Attach to Tower → enters AwaitingTarget[AttachToTower]
   - (1,0) A = Stop → issues Stop immediately
   - (1,1) S = HoldPosition → issues HoldPosition immediately
   - No attack commands (unarmed unit)
   Add CommandButtonAction variants (e.g., UnitPickUpSupplies, UnitAttachToTower) and wire them into execute_command_action, button_label, button_availability, and object_type_supports_action.

4. **AwaitingTarget resolution**: In right_click_move_command (core.rs), add left-click entity handling for CommandType::PickUpSupplies (left-click SDS → issue PickUpSupplies command, reset state) and CommandType::AttachToTower (left-click own SupplyTower → issue AttachToTower command, reset state). Follow the same pattern as the existing Enter/Gather/DropOff AwaitingTarget handlers (~lines 274-346).

5. **Tests**: Verify SupplyChopper grid layout, button availability (no attack buttons), AwaitingTarget mode transitions, and object_type_supports_action for the new action variants.
