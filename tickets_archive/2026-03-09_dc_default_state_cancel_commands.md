# Ticket: Add Cancel commands to DC DefaultState

## Current State
The Deployment Center's DefaultState (DcIdle) only has the Build command (Q) to enter BuildMenu. Cancel Construction and Cancel Ready Building are only accessible inside the BuildMenu's Constructing and Ready sub-states. The player must press Q to enter BuildMenu first, then X to cancel — two keypresses for a basic cancel operation.

## Desired State
DC DefaultState (DcIdle) shows conditional Cancel commands:
- **X: Cancel Construction** (CommandIssuingTransition): visible when `current_construction` is active. Full refund, clears CurrentConstruction.
- **X: Cancel Ready Building** (CommandIssuingTransition): visible when `ready_to_place` is active. 75% refund (rounded down), clears ReadyToPlace.

These are additive — the BuildMenu retains its own Cancel commands. The player can cancel from either location.

This aligns DC with EF's existing pattern (EF already has Cancel in its DefaultState) and follows standard RTS convention that cancel should always be one keypress away.

## Justification
- Feature spec update in `features/gdo_objects.md` — DC DefaultState now explicitly includes conditional Cancel commands.
- Forum topic `dc_ef_construction_ux.md` — QA identified the UX gap, product_analyst confirmed the spec asymmetry (EF already had Cancel in DefaultState, DC did not), and updated the spec.
- Feature update `2026-03-09_gdo_objects.md` (DC DefaultState Cancel Commands).

## QA Steps
1. [human] Select a DC that is not constructing — verify only Build (Q) appears in the command panel, no Cancel (X).
2. [human] Start a construction (e.g., Power Plant) and return to DefaultState — verify Cancel Construction (X) now appears alongside Build (Q).
3. [human] Press X during construction from DefaultState — verify full refund is received (150 SC for Power Plant) and the DC returns to idle.
4. [human] Start another construction, let it complete to ready-to-place, return to DefaultState — verify Cancel Ready Building (X) appears.
5. [human] Press X from DefaultState when ready to place — verify 75% refund (rounded down, e.g., 112 SC for Power Plant) is received and the DC returns to idle.
6. [human] Enter BuildMenu (Q) while constructing — verify Cancel (X) is still available inside BuildMenu as well (both paths work).
7. [human] Verify the X slot position is consistent: Cancel should appear at (2,1) in both DefaultState and BuildMenu contexts.

## Expected Experience
The player always has Cancel within one keypress when a DC is constructing or has a building ready to place. There's no need to remember to enter the BuildMenu first — X from the default view handles it. This matches how EF already works and feels natural for any RTS player. The refund amounts match the spec (full during construction, 75% when ready to place).
