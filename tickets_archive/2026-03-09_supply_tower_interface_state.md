# Ticket: Supply Tower ObjectInterfaceState Implementation

## Current State
The Supply Tower structure has its stats, production chain, and attach/delivery mechanics defined, but its full ObjectInterfaceState (hotkey assignments and interaction flows) has not been implemented. Rally point setting (both right-click and hotkey) was not previously specified for the Supply Tower.

## Desired State
Implement `ObjectInterfaceState[SupplyTower]` with the following commands mapped to the 3x3 command panel grid:

- **Right-click Ground/Object**: SetRallyPoint (new — Supply Tower now supports rally points via right-click)
- **Q: Build Supply Chopper** (CommandIssuingTransition): deducts 100 SC, adds to BuildQueue. Requires queue < 5, sufficient SC.
- **X: Cancel Production** (CommandIssuingTransition): removes last BuildQueue entry, full refund. Only if queue non-empty.
- **C: Set Rally Point** (StateOnlyTransition → AwaitingTarget[SetRallyPoint]): left-click ground/object sets rally (CommandIssuingTransition, returns to DefaultState).
- **S: Schedule Deliveries** (StateOnlyTransition → AwaitingTarget[ScheduleDeliveries]): only available if tower has an attached chopper. Left-click an SDS to set scheduled deliveries (CommandIssuingTransition, returns to DefaultState).

Key new behavior: both right-click SetRallyPoint and C: Set Rally Point are new additions. The S: Schedule Deliveries command provides keyboard-driven access to the delivery scheduling mechanic.

## Justification
`features/gdo_objects.md` — Supply Tower section, ObjectInterfaceState[SupplyTower]. The feature spec now defines the full interface state with explicit hotkey assignments. Adding rally point support (C and right-click) brings the Supply Tower in line with other production structures. The S hotkey for Schedule Deliveries surfaces an important mechanic that would otherwise require a more complex UI flow.

## QA Steps
1. [human] Select a Supply Tower — verify the command panel shows Q (Build Supply Chopper), X (Cancel Production), C (Set Rally Point), and S (Schedule Deliveries) in their correct grid positions
2. [human] Press Q with sufficient SC and queue space — verify a Supply Chopper is added to the build queue and 100 SC is deducted
3. [human] Press Q when queue is full (5 entries) — verify the command is rejected
4. [human] Press X with a non-empty queue — verify the last entry is removed and fully refunded
5. [human] Press X with an empty queue — verify nothing happens
6. [human] Press C — verify the cursor/state changes to AwaitingTarget[SetRallyPoint]
7. [human] While in AwaitingTarget[SetRallyPoint], left-click a ground location — verify rally point is set and state returns to DefaultState
8. [human] Right-click a ground location while Supply Tower is selected — verify rally point is set directly
9. [human] Press S with an attached chopper — verify state changes to AwaitingTarget[ScheduleDeliveries]
10. [human] While in AwaitingTarget[ScheduleDeliveries], left-click an SDS — verify scheduled deliveries are set and state returns to DefaultState
11. [human] Press S without an attached chopper — verify the command is unavailable or rejected
12. [human] Produce a Supply Chopper with a rally point set — verify it moves to the rally point after spawning

## Expected Experience
Selecting a Supply Tower shows a command card with four active slots: Q, X, C, and S. The Q/X production commands work identically to the Barracks pattern. The C rally point flow mirrors Barracks — press C, click a location, see a rally marker. The S command is contextual: it only activates when the tower has an attached chopper, and targets SDS objects specifically. Right-clicking ground also sets rally points directly. The interface feels consistent with other GDO production structures while surfacing the Supply Tower's unique delivery scheduling mechanic.
