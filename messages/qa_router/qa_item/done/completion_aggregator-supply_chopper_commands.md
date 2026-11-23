# supply_chopper_commands

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# supply_chopper_commands

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Completed the truncated Supply Chopper design in `artifacts/designer/design/gdo_objects.md`. The file was previously cut off mid-sentence at the AwaitingTarget[PickUpSupplies] section.

## Changes to `artifacts/designer/design/gdo_objects.md`:

### Updated right-click resolution (state-dependent behavior):
- Right-click SupplyDeliveryStation now gated: only works if chopper is not carrying units
- Right-click own SupplyTower is now state-dependent:
  - When carrying supplies: issues DropOffSupplies command (fly to tower, land, drop off supplies, immediately lift off — unless it's the chopper's attached tower)
  - When not carrying supplies: issues AttachToTower command (only if not carrying units)

### Updated target command availability:
- Pick Up Supplies: only available if chopper is not carrying units
- Attach to Tower: only available if chopper is not carrying units
- New command — Drop Off Supplies: only available if chopper is carrying supplies

### New AwaitingTarget resolutions added:
- **AwaitingTarget[PickUpSupplies]**: left-click SDS issues command, anything else no action, escape cancels
- **AwaitingTarget[AttachToTower]**: left-click own SupplyTower with no attached chopper issues command, anything else no action, escape cancels
- **AwaitingTarget[DropOffSupplies]**: left-click own SupplyTower with no attached chopper issues command, anything else no action, escape cancels
- **AwaitingTarget[Move]**: left-click ground or object issues Move command, escape cancels

### Key design decisions:
- Drop-off at a non-attached tower is touch-and-go (land, drop supplies, immediately lift off). Only attached towers provide persistent landing and repair.
- Any player-issued command to an attached chopper breaks its attachment (already documented). Automated scheduled delivery departures do NOT break attachment.
- A chopper cannot drop off supplies at a tower that already has an attached chopper.

## QA Instructions

1. Select a Supply Chopper. Verify the command panel shows Move, Pick Up Supplies, Attach to Tower commands. Drop Off Supplies should NOT appear (chopper has no supplies).
2. If the chopper is carrying units, verify Pick Up Supplies and Attach to Tower commands are unavailable/hidden.
3. Click Pick Up Supplies, then left-click a Supply Delivery Station — chopper should fly to SDS, land, and pick up supplies. Verify it returns to DefaultState.
4. Click Pick Up Supplies, then left-click empty ground or a non-SDS object — nothing should happen, should remain in AwaitingTarget mode.
5. Press Escape while in any AwaitingTarget mode — should return to DefaultState.
6. Once chopper is carrying supplies, verify Drop Off Supplies command appears in the command panel.
7. Right-click an own Supply Tower while carrying supplies — chopper should fly to tower, drop off supplies, and immediately lift off (if not attached to that tower).
8. Right-click an own Supply Tower while carrying supplies and attached to that tower — chopper should fly to tower, drop off supplies, and remain landed for repair.
9. Use Drop Off Supplies target command, left-click a tower that already has an attached chopper — nothing should happen.
10. Issue any player command (Move, Stop, etc.) to an attached chopper — verify it breaks attachment to its tower.
11. Verify that automated scheduled delivery departures do NOT break the chopper's attachment to its tower.
