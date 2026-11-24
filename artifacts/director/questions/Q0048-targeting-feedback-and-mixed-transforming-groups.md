# Q0048: Pressing other commands while targeting; a group of portals only partly transforming

- **Date**: 2026-11-24T15:01:49-05:00
- **Blocking**: no. M3-J1 is built with the current behaviour listed under each point.
- **Docs**: `systems/controls/selection_and_hud.md > AwaitingTarget[CommandType]` (Other commands while targeting), `> CommandPanel` (Pressing an unavailable command); `factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel`; changes 0066, 0116 and 0162.

## Question
1. **Another command's hotkey pressed while a target or a placement is awaited.**
   - "Other commands while targeting" (0066) says those commands are unavailable and pressing them **does nothing**.
   - "Pressing an unavailable command" (0162) says an unavailable command can still be pressed, and the player gets the refusal feedback naming the reason.
   - **Current behaviour:** the feedback band says "Cancel the current command first", and targeting or placement continues.
2. **A SelectionGroup of portals where some are still warping in and some are finished** (same type, so one group).
   - "While transforming: only Cancel" covers a building on its own, not a group that mixes the two.
   - **Current behaviour:** the group shows the normal portal panel. The warping-in portals can't use its commands, and the group has no Cancel.

## Context
Both came up while building the Colonists Warp In and Deploy UI (M3-J1).

## Options
- **Point 1:**
  - A, as now: feedback that names the reason.
  - B, silent, as 0066 says.
- **Point 2:**
  - A, as now: the portal panel.
  - B, the Cancel-only panel while any portal in the group is warping in.
  - C, warping-in portals form their own SelectionGroup, with the Cancel-only panel, as if they were another type.
