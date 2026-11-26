# 0400 — Right-click doesn't reset multi-stage menus; in targeting it goes straight to DefaultState

- **Date**: 2026-11-27T13:35:00-05:00
- **Answers**: —
- **Docs**:
  - `systems/controls/interface.md > CommandPanel > Standard Slot Assignments` (Z; Right-click in multi-stage menus);
  - `systems/controls/interface.md > AwaitingTarget[CommandType]`;
  - `factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]` (BuildMenu, AwaitingPlacement).

## What changed and why
User: "when using multi-stage menus, right click shouldn't reset the state of the menu. Only when the next step in action selection is the need to left click to select the location should right click reset the menu state to default."

- **In menus where the next step is choosing an option**, right-click no longer closes or backs out of the menu:
  - menus: BuildMenu, ExpandMenu, EjectMenu, the Cults KindMenu and AugmentMenu, etc.;
  - the right-click does its **normal right-click** for the selection (moving workers, setting a rally point), and the menu stays open. This is the designer's reading of "shouldn't reset".
- **In AwaitingTarget and AwaitingPlacement**, right-click cancels **straight to DefaultState**:
  - e.g. the Deployment Center's placement used to return to the BuildMenu;
  - a ready building stays ready.
- **Escape and Z** still go back one step.

## Implementation impact
- Remove right-click as "back" from every menu state.
- In menu states, right-click runs the normal RightClickResolution without changing the interface state.
- Right-click in targeting and placement returns to DefaultState.

## Verification
- Select Recruits, open Basic Construct and right-click ground: the Recruits move and the build menu stays open.
- Pick a building and right-click during placement: the panel is back at DefaultState.
- Escape during placement goes back to the build menu.
