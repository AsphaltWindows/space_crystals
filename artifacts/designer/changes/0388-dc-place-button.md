# 0388 — Deployment Center: a Place button with a progress bar in DefaultState

- **Date**: 2026-11-27T12:00:00-05:00
- **Answers**: —
- **Docs**: `factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter] > DefaultState commands`

## What changed and why
User: when the Deployment Center is building anything, there should be a **Place** button. It is greyed out but shows a running progress bar, so the player doesn't need to go into the build menu to place the building.
- **E: Place** (the letter is a placeholder) is in the DC's **DefaultState**, showing the building's icon.
- **While constructing**, it is greyed out, with a progress bar filling as construction progresses.
- **Once ready**, it is enabled. Pressing it enters AwaitingPlacement, the same as "Select completed building" in the build menu. Escape or right-click returns to DefaultState.
- It is hidden while the DC is idle.
- The build menus' existing placement slot stays.

## Implementation impact
- A new DefaultState button with a progress-bar overlay. The client reads CurrentConstruction progress.
- Whether the same shortcut applies to the Defense System (which constructs the same way) is being asked.

## Verification
- Order a Generator and stay in DefaultState: E shows a greyed Generator with a filling bar.
- When it completes, E lights up. Press E, then click to place it, without opening Basic Construction.
