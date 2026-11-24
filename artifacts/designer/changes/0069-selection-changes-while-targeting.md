# 0069 — Selection changes while targeting: player changes cancel, deaths don't

- **Date**: 2026-11-23T20:49:36-05:00
- **Answers**: Q0016 (point 3 complete; point 4 still open)
- **Docs**: `systems/controls/selection_and_hud.md > ObjectInterfaceState` (reset rule refined), `> AwaitingTarget[CommandType]` (Selection changes while targeting)

## What changed and why
- Player-initiated selection or ActiveGroup changes (control group recall, Tab, portrait clicks) **cancel** targeting. This was already in the docs and matches the current behaviour.
- **Refinement:** automatic removals (a selected unit dies or enters a building, transport or tunnel) **don't** cancel targeting, or reset the ObjectInterfaceState in general, unless the **whole ActiveGroup** is gone. Without this, any death in a fight would silently cancel the player's aim.

## Implementation impact
Distinguish player-driven selection changes from validation-driven removals when deciding whether to reset ObjectInterfaceState.

## Verification
- Press Attack with 10 Peacekeepers selected. One dies before you click: still targeting, and the click issues the attack.
- Press Attack, then press 2 (recall a control group): targeting is cancelled and group 2 is selected.
- Press Attack with a single unit selected; it dies: targeting is cancelled.
