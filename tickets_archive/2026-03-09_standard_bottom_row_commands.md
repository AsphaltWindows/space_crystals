# Ticket: Standard Bottom-Row Slot Assignments and Production Right-Click

## Current State
The CommandPanel has no standardized slot assignments. Each object type's command placement is ad-hoc. Production buildings have no default right-click behavior for rally points from the control system interface layer.

## Desired State
Implement three standardized bottom-row commands and production building right-click:

### Standard Slot Z (Bottom-Left): Back / Cancel Menu
- In any multi-stage menu state (BuildMenu, ExpandMenu, EjectMenu, AwaitingTarget, AwaitingPlacement): pressing Z performs a StateOnlyTransition back to the previous state
- Equivalent to Escape or right-click cancel
- Must appear in slot Z whenever the current ObjectInterfaceState is a sub-menu or awaiting state (not DefaultState)

### Standard Slot X (Bottom-Center): Cancel Production / Cancel Upgrade
- In production buildings: cancels the last queued production item and refunds cost (CommandIssuingTransition)
- In structures with upgrades: cancels the in-progress upgrade and refunds cost (CommandIssuingTransition)
- Only appears when there is an active production queue or in-progress upgrade

### Standard Slot C (Bottom-Right): Set Rally Point
- In unit-producing structures from DefaultState: pressing C enters AwaitingTarget[SetRallyPoint] (StateOnlyTransition)
- Left-click ground: sets rally point to that location (CommandIssuingTransition, returns to DefaultState)
- Left-click object: sets rally point to that object (CommandIssuingTransition, returns to DefaultState)

### Production Building Default Right-Click
- For all unit-producing structures, right-click from DefaultState sets the rally point:
  - Right-click ground → set rally point to that location
  - Right-click object → set rally point to that object
- This is a convenience shortcut — equivalent to pressing C then left-clicking

## Justification
`features/control_system.md` — Standard Slot Assignments and Production Building Default Right-Click sections. Standardized bottom-row commands create muscle memory across all object types (Z always goes back, X always cancels production, C always sets rally). The right-click shortcut for rally points matches RTS genre conventions. Note: the existing `2026-03-09_rally_point_behavior.md` ticket covers rally point *behavior* (what happens when units are produced with a rally point set). This ticket covers the *interface layer* — how the player issues the rally point command.

## QA Steps
1. [human] Select a unit-producing structure and press C — verify the interface enters AwaitingTarget[SetRallyPoint] mode (cursor changes or visual indicator shown)
2. [human] In AwaitingTarget[SetRallyPoint], left-click a ground location — verify the rally point is set and the interface returns to DefaultState
3. [human] Select a unit-producing structure and right-click ground — verify the rally point is set directly (shortcut, no intermediate state)
4. [human] Select a unit-producing structure and right-click an object — verify the rally point is set to that object
5. [human] Enter a multi-stage menu (e.g., BuildMenu) and press Z — verify the interface returns to the previous state
6. [human] Press Escape in the same multi-stage menu — verify it behaves identically to pressing Z
7. [human] Queue a production item, then press X — verify the last queued item is cancelled and cost is refunded
8. [human] With no production queue active, verify the X slot is not shown or is inactive
9. [human] In DefaultState (no sub-menu), verify the Z slot is not shown or is inactive

## Expected Experience
Bottom-row commands feel consistent across all object types. Z always means "go back" — the player builds this muscle memory quickly. X is a safe cancel that refunds resources, visible only when relevant. C and right-click both set rally points, matching the convention from StarCraft/Warcraft where right-click on a production building sets rally. The interface transitions are instant and clear — entering AwaitingTarget mode should show a distinct cursor or overlay so the player knows the game is waiting for a click.
