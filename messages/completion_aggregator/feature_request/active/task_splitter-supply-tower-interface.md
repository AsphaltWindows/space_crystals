# supply-tower-interface

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

Implement the Supply Tower's ObjectInterfaceState as defined in `artifacts/designer/design/gdo_objects.md` under 'ObjectInterfaceState[SupplyTower]'.

**DefaultState commands:**

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Immediate commands (CommandIssuingTransition):
- **Q: Build Supply Chopper** — deducts 100 Space Crystals, adds SupplyChopper to BuildQueue. Available if queue < 5 and sufficient crystals.
- **X: Cancel Production** — removes last BuildQueue entry, refunds full cost. Available if queue not empty.

Target commands (StateOnlyTransition):
- **C: Set Rally Point** — enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets rally point.
- **S: Schedule Deliveries** — enters AwaitingTarget[ScheduleDeliveries]. Only available if tower has an attached chopper.

**AwaitingTarget[ScheduleDeliveries] resolution:**
- Left-click SupplyDeliveryStation: sets ScheduledSDS, attached chopper begins automated delivery loop (CommandIssuingTransition, returns to DefaultState).
- Left-click anything else: no action.
- Escape/right-click: returns to DefaultState (StateOnlyTransition).

**Supply Tower placement:** On placement, one free Supply Chopper spawns and auto-attaches.

## QA Instructions

1. Place a Supply Tower via DeploymentCenter. Verify a free Supply Chopper spawns on it.
2. Select the Supply Tower. Verify Q (Build Chopper), X (Cancel), C (Rally Point), S (Schedule Deliveries) are shown.
3. Verify S is available (tower has attached chopper from placement).
4. Press Q — verify chopper is queued and 100 crystals deducted.
5. Press X — verify last queue entry removed and cost refunded.
6. Press C, left-click ground — verify rally point is set.
7. Right-click ground — verify rally point is set via right-click.
8. Press S — verify AwaitingTarget[ScheduleDeliveries] state is entered.
9. Left-click a SupplyDeliveryStation — verify ScheduledSDS is set and chopper begins delivery loop.
10. Left-click something other than an SDS — verify no action occurs.
11. Press Escape — verify return to DefaultState.
12. Detach the chopper (give it a command). Verify S becomes unavailable.
