# supply-chopper-dropoff-command

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-supply_chopper_commands.md

## Task

Add the `UnitCommand::DropOffSupplies(Entity)` variant and update the Supply Chopper right-click resolution to be state-dependent with carrying-units gating.

### Changes needed:

**1. Add DropOffSupplies command variant** (`game/units/types/state/commands.rs`):
- Add `DropOffSupplies(Entity)` to `UnitCommand` enum
- Mark it as requiring target (`is_entity_targeted() => true`)
- Add tests matching existing PickUpSupplies/AttachToTower pattern

**2. Update right-click resolution** (`game/units/systems/core.rs`, ~line 350-391):
The current SupplyChopper right-click handler issues PickUpSupplies for SDS and AttachToTower for own SupplyTower unconditionally. Update:

- **Right-click SDS**: Only issue PickUpSupplies if chopper is NOT carrying units. Check `chopper_state.carried_units` or equivalent field. (Currently no carrying-units check.)
- **Right-click own SupplyTower**: Make state-dependent:
  - If chopper `carried_supplies > 0`: issue `DropOffSupplies(tower_entity)` instead of AttachToTower
  - If chopper `carried_supplies == 0` AND not carrying units: issue `AttachToTower(tower_entity)`
  - If carrying units: do nothing (fall through to default move)

**3. Break attachment on player commands** (`game/units/systems/commands.rs` or core.rs):
- When any player-issued command is given to a SupplyChopper that has `attached_tower.is_some()`, clear the attachment (set `attached_tower = None`, also clear the tower's `attached_chopper` if it references this chopper)
- Automated scheduled delivery departures must NOT break attachment (only player-issued commands do)

### Reference:
- Design: `artifacts/designer/design/gdo_objects.md` lines 329-368
- SupplyChopperState fields: `carried_supplies`, `attached_tower`
- Existing right-click handler: `core.rs` ~line 350-391
