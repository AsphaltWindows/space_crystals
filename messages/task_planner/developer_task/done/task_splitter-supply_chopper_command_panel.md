# supply-chopper-command-panel

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-supply_chopper_commands.md

## Task

Implement the SupplyChopper command panel grid with target commands, AwaitingTarget resolutions, and availability gating.

### Changes needed:

**1. Add CommandButtonAction variants** (`ui/command_panel.rs`):
- Add `ChopperPickUpSupplies` — target command, enters AwaitingTarget[PickUpSupplies]
- Add `ChopperAttachToTower` — target command, enters AwaitingTarget[AttachToTower]  
- Add `ChopperDropOffSupplies` — target command, enters AwaitingTarget[DropOffSupplies]
- Follow the pattern of existing target commands (AgentGather, AgentDropOff, UnitEnter)

**2. Add SupplyChopper command panel grid**:
The SupplyChopper currently uses BasicCombatUnitInterfaceState which doesn't have chopper-specific commands. Add a SupplyChopper-specific grid to the Default state command panel:
- Grid layout (row, col):
  - (0, 0) = Move (Q) — target command, enters AwaitingTarget[Move]
  - (0, 1) = Pick Up Supplies (W) — target command, availability: NOT carrying units
  - (0, 2) = Attach to Tower (E) — target command, availability: NOT carrying units
  - (1, 0) = Drop Off Supplies (A) — target command, availability: carrying supplies > 0
  - (1, 2) = Hold Position (D) — immediate command
  - (2, 1) = Stop (X) — immediate command

Detect SupplyChopper via `ObjectEnum::SupplyChopper` in the object type of the active selection group. Override the default combat unit grid for this type.

**3. Add `object_type_supports_action` entries**:
- SupplyChopper supports: UnitMove, UnitHoldPosition, UnitStop, ChopperPickUpSupplies, ChopperAttachToTower, ChopperDropOffSupplies
- SupplyChopper does NOT support: UnitAttack, UnitAttackMove, UnitAttackGround, UnitPatrol, UnitReverse, UnitEnter, AgentGather, AgentDropOff

**4. Implement AwaitingTarget resolutions** (in `execute_command_action` or the left-click/entity-click handlers):
- **AwaitingTarget[PickUpSupplies]**: left-click SDS → issue PickUpSupplies command, return to Default. Left-click anything else → no action. Escape/right-click → return to Default.
- **AwaitingTarget[AttachToTower]**: left-click own SupplyTower with no attached chopper → issue AttachToTower command, return to Default. Left-click anything else → no action. Escape/right-click → return to Default.
- **AwaitingTarget[DropOffSupplies]**: left-click own SupplyTower with no attached chopper → issue DropOffSupplies command, return to Default. Left-click anything else → no action. Escape/right-click → return to Default.
- **AwaitingTarget[Move]**: left-click ground/object → issue Move command. Escape/right-click → return to Default.

**5. Add button availability checks**:
- ChopperPickUpSupplies: available only if selected chopper is NOT carrying units (need to add a `chopper_carrying_units` field to SelectedUnitCapabilities or check SupplyChopperState directly)
- ChopperAttachToTower: available only if selected chopper is NOT carrying units
- ChopperDropOffSupplies: available only if selected chopper has `carried_supplies > 0`

**6. Add button labels**:
- ChopperPickUpSupplies → "[W] Pick Up\nSupplies"
- ChopperAttachToTower → "[E] Attach\nTower"
- ChopperDropOffSupplies → "[A] Drop Off\nSupplies"

**7. Add CommandType variants if needed** for PickUpSupplies, AttachToTower, DropOffSupplies (check `CommandType` enum in commands.rs).

### Reference:
- Design: `artifacts/designer/design/gdo_objects.md` lines 329-368
- Pattern: AgentGather/AgentDropOff in command_panel.rs for target command flow
- SelectedUnitCapabilities: used for button availability gating
- AwaitingTarget pattern: see existing Enter/Gather/DropOff handling in core.rs entity-click section
