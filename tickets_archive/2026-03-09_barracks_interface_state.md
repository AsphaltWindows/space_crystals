# Ticket: Barracks ObjectInterfaceState Implementation

## Current State
The Barracks structure has its stats and production chain defined, but its full ObjectInterfaceState (hotkey assignments and interaction flows) has not been implemented. Rally point setting was previously only available via right-click; there was no dedicated hotkey for it.

## Desired State
Implement `ObjectInterfaceState[Barracks]` with the following commands mapped to the 3x3 command panel grid:

- **Right-click Ground/Object**: SetRallyPoint
- **Q: Build Peacekeeper** (CommandIssuingTransition): deducts 50 SC, adds to BuildQueue. Requires queue < 5, sufficient SC and Unit Control.
- **X: Cancel Production** (CommandIssuingTransition): removes last BuildQueue entry, full refund. Only if queue non-empty.
- **C: Set Rally Point** (StateOnlyTransition → AwaitingTarget[SetRallyPoint]): left-click ground/object sets rally (CommandIssuingTransition, returns to DefaultState).

Key new behavior: the C hotkey provides a dedicated Set Rally Point command in addition to the existing right-click method. In the C flow, the player enters AwaitingTarget state, then left-clicks to confirm the rally point location.

## Justification
`features/gdo_objects.md` — Barracks section, ObjectInterfaceState[Barracks]. The feature spec now defines the full interface state with explicit hotkey assignments. The C: Set Rally Point hotkey is a new addition that gives players a keyboard-driven alternative to right-clicking, consistent with standard RTS command card design.

## QA Steps
1. [human] Select a Barracks — verify the command panel shows Q (Build Peacekeeper), X (Cancel Production), and C (Set Rally Point) in their correct grid positions
2. [human] Press Q with sufficient SC and queue space — verify a Peacekeeper is added to the build queue and 50 SC is deducted
3. [human] Press Q when queue is full (5 entries) — verify the command is rejected (no SC deducted, no queue addition)
4. [human] Press X with a non-empty queue — verify the last entry is removed and its full cost is refunded
5. [human] Press X with an empty queue — verify nothing happens
6. [human] Press C — verify the cursor/state changes to AwaitingTarget[SetRallyPoint]
7. [human] While in AwaitingTarget, left-click a ground location — verify the rally point is set and state returns to DefaultState
8. [human] Right-click a ground location while Barracks is selected (in DefaultState) — verify the rally point is set directly without entering AwaitingTarget
9. [human] Produce a Peacekeeper with a rally point set — verify it exits from the B side and moves to the rally point

## Expected Experience
Selecting a Barracks shows a clean command card with three active slots. Pressing Q queues a Peacekeeper with immediate visual and resource feedback. Pressing C switches to a targeting cursor — left-clicking confirms the rally point, which appears as a visual marker. Right-clicking also sets rally points directly, giving two ergonomic options. The X key provides quick cancel access. All hotkeys feel responsive and consistent with the 3x3 grid layout.
