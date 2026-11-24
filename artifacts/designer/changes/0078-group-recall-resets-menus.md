# 0078 — Any control group recall resets menus

- **Date**: 2026-11-24T01:45:00-05:00
- **Answers**: Q0021 (point 3; Q0021 is now fully answered)
- **Docs**:
  - `systems/controls/selection_and_hud.md > ObjectInterfaceState` (reset rule)
  - `systems/controls/selection_and_hud.md > AwaitingTarget[CommandType] > Selection changes while targeting`

## What changed and why
The user picked **B**, and widened it. Every control group recall resets the ObjectInterfaceState to the default state, **even if the recalled group is exactly the current selection**. The user's view: a recall refreshes the selection, and players shouldn't press group keys in the middle of a multi-step command.

The reset covers every non-default state, not only targeting:
- AwaitingTarget;
- AwaitingPlacement;
- the BuildMenu and other sub-menus.

## Implementation impact
- This replaces the current behaviour, where a recall that gives an identical selection leaves targeting active.
- The other player-initiated changes (Tab, portrait clicks, clicks and boxes) are unchanged: they reset when they change the Selection or ActiveGroup.
- Automatic removals (deaths, entering a building) still don't reset, unless the ActiveGroup empties (0069).

## Verification
1. Recall group 1, press A, then press 1 again. Targeting is cancelled and the panel shows the default commands.
2. Select a Deployment Center via its control group, open the BuildMenu, then press the same group key. The panel returns to the default state.
3. Press A, then a unit dies from the ActiveGroup (others remain). Targeting stays active.
