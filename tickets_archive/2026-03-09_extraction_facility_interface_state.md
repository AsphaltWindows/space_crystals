# Ticket: Extraction Facility ObjectInterfaceState Implementation

## Current State
The Extraction Facility structure exists but its full ObjectInterfaceState is not implemented with explicit hotkey-to-slot assignments. The DefaultState has three sub-conditions (idle, constructing, ready to place) that need concrete command panel grid bindings, plus an AwaitingPlacement state for plate placement on Space Crystal Patches.

## Desired State
Implement `ObjectInterfaceState[ExtractionFacility]` with the following states and hotkey assignments:

### DefaultState (idle — no active construction)
- **Q: Build Extraction Plate** (CommandIssuingTransition): deducts 75 SC, starts plate construction. Requires sufficient SC; slot shown inactive otherwise.

### DefaultState (constructing)
- **X: Cancel Construction** (CommandIssuingTransition): full refund, clears CurrentConstruction.
- Q slot hidden/inactive during construction.

### DefaultState (ready to place)
- **Q: Place Plate** (StateOnlyTransition → AwaitingPlacement): enters placement mode.
- **X: Cancel Ready Plate** (CommandIssuingTransition): 75% refund (rounded down), clears ReadyToPlace.

### AwaitingPlacement
- Ghost preview on valid Space Crystal Patches within GDO build area that have no existing plate (green tint). Invalid patches show red tint. Build area overlay displayed.
- Left-click valid patch: places plate (CommandIssuingTransition, returns to DefaultState).
- Escape/right-click: returns to DefaultState (StateOnlyTransition).

## Justification
`features/gdo_objects.md` — Extraction Facility Interface section. `features/control_system.md` — Standard Slot Assignments (Q=primary action, X=cancel). The ExFac uses Q for its primary action (build/place) and X for cancellation, following the standard grid conventions. Unlike the Deployment Center which uses a BuildMenu sub-state, the ExFac stays in DefaultState with context-dependent command availability, making the Q/X pattern particularly important for clarity.

## QA Steps
1. [human] Select an idle Extraction Facility — verify Q (Build Extraction Plate) appears in the CommandPanel with cost indicator (75 SC)
2. [human] Press Q with sufficient SC — verify 75 SC is deducted and construction begins
3. [human] Press Q with insufficient SC — verify the command is blocked (slot inactive or error feedback)
4. [human] During construction — verify X (Cancel Construction) appears and Q is hidden/inactive
5. [human] Press X during construction — verify full refund is received and the facility returns to idle state
6. [human] Let construction complete — verify Q changes to "Place Plate" and X shows "Cancel Ready Plate"
7. [human] Press X when plate is ready — verify 75% refund (rounded down) is received and facility returns to idle
8. [human] Press Q when plate is ready — verify AwaitingPlacement state is entered with ghost preview
9. [human] In AwaitingPlacement, hover over Space Crystal Patches — verify valid patches (in build area, no existing plate) show green, invalid show red
10. [human] Left-click a valid patch — verify plate is placed and interface returns to DefaultState
11. [human] In AwaitingPlacement, press Escape — verify it returns to DefaultState
12. [human] In AwaitingPlacement, right-click — verify it returns to DefaultState

## Expected Experience
The Extraction Facility interface is simpler than the Deployment Center — it only builds one thing (Extraction Plates). Q always does the "next action" (start building when idle, enter placement when ready), and X always cancels. The player quickly learns the Q/X rhythm: Q to build, wait, Q to place, click a crystal patch. During AwaitingPlacement, valid Space Crystal Patches light up clearly, making it obvious where plates can go. The green/red feedback is immediate on hover, and placement happens instantly on click.
