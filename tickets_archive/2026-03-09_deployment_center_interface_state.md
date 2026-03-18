# Ticket: Deployment Center ObjectInterfaceState Implementation

## Current State
The Deployment Center structure exists but its full ObjectInterfaceState is not implemented with explicit hotkey-to-slot assignments. The BuildMenu sub-states (idle, constructing, ready to place) need concrete command panel grid bindings.

## Desired State
Implement `ObjectInterfaceState[DeploymentCenter]` with the following states and hotkey assignments:

### DefaultState
- Build command enters BuildMenu (StateOnlyTransition)

### BuildMenu (idle — no active construction)
- Slot-mapped build options: Power Plant, Barracks, Supply Tower (CommandIssuingTransition, returns to DefaultState)
- Supply Tower option requires player owns >= 1 Power Plant; slot shown inactive/hidden otherwise
- **Z**: returns to DefaultState (StateOnlyTransition)

### BuildMenu (constructing)
- **X: Cancel Construction** (CommandIssuingTransition): full refund, clears CurrentConstruction
- **Z**: returns to DefaultState (StateOnlyTransition)
- Build options hidden/inactive during construction

### BuildMenu (ready to place)
- Select building enters AwaitingPlacement
- **X: Cancel Ready Building** (CommandIssuingTransition): 75% refund (rounded down), clears ReadyToPlace
- **Z**: returns to DefaultState (StateOnlyTransition)

### AwaitingPlacement
- Ghost preview with green/red tint, build area overlay
- R/Shift+R rotation, F/Shift+F flipping (horizontal/vertical)
- Side labels (A/B/C/D per SymmetryType) displayed on ghost, updating with rotation/flipping
- Left-click valid location: place building (CommandIssuingTransition, returns to DefaultState)
- Escape/right-click: returns to BuildMenu (StateOnlyTransition)

## Justification
`features/gdo_objects.md` — Deployment Center Interface section. `features/control_system.md` — Standard Slot Assignments (Z=back, X=cancel). The DC is the primary GDO structure-building interface. Explicit hotkey assignments ensure consistency with the control system's standard bottom-row conventions and give players reliable muscle memory (X always cancels, Z always goes back).

## QA Steps
1. [human] Select the Deployment Center — verify the CommandPanel shows a "Build" option to enter BuildMenu
2. [human] Enter BuildMenu when idle — verify Power Plant, Barracks, and Supply Tower options appear with hotkey indicators, and Z (back) is available
3. [human] Press Z in BuildMenu — verify it returns to DefaultState
4. [human] Start constructing a building — verify BuildMenu shows X (Cancel Construction) and Z (back), with build options hidden/inactive
5. [human] Press X during construction — verify the construction is cancelled, full refund is received, and BuildMenu returns to idle state
6. [human] Let construction complete to ready — verify BuildMenu shows the ready building option, X (Cancel Ready Building), and Z (back)
7. [human] Press X when building is ready — verify 75% refund (rounded down) is received and the ready building is cleared
8. [human] Enter AwaitingPlacement — verify ghost preview appears with green/red tint on valid/invalid locations
9. [human] Press Escape or right-click in AwaitingPlacement — verify it returns to BuildMenu
10. [human] Verify Supply Tower option is inactive/hidden when player owns 0 Power Plants, and becomes available after placing one

## Expected Experience
The Deployment Center interface feels structured and predictable. Entering the BuildMenu shows available buildings when idle, a cancel option during construction, and placement options when ready. Z consistently takes the player "back" one level without any resource consequence. X is the resource-affecting cancel — the player sees the refund amount (full during construction, 75% when ready) reflected immediately in their SC count. AwaitingPlacement shows a clear ghost preview that snaps to the grid, with obvious green/red feedback for valid/invalid positions.
