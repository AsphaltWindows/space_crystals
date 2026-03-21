# dc-defaultstate-cancel

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

## Content

Added Cancel command to DeploymentCenter DefaultState, accessible via X hotkey.

Previously, players had to enter the BuildMenu to cancel an in-progress construction or ready-to-place building. Now the Cancel command is surfaced directly in DefaultState:
- **X: Cancel Construction** — visible when CurrentConstruction is set. Full refund, clears CurrentConstruction.
- **X: Cancel Ready Building** — visible when ReadyToPlace is set. 75% refund (rounded down), clears ReadyToPlace.

The Build command was also assigned to the **Q** slot for consistency with the grid layout.

The BuildMenu still retains its own Cancel commands (unchanged).

Modified file: `artifacts/designer/design/gdo_objects.md` — ObjectInterfaceState[DeploymentCenter] DefaultState section.

## QA Instructions

1. Select a Deployment Center.
2. Start constructing a building (enter BuildMenu, pick one).
3. Verify the DefaultState now shows a Cancel (X) button.
4. Press X — verify construction is cancelled and full cost is refunded.
5. Start another construction and let it complete to ready-to-place.
6. Verify DefaultState shows Cancel (X) for the ready building.
7. Press X — verify the building is cancelled and 75% cost (rounded down) is refunded.
8. Verify that entering the BuildMenu still shows Cancel as before (no regression).
9. Verify that when idle (no construction, no ready building), the X slot is empty/hidden in DefaultState.
