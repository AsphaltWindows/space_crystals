# 0199 — Which group becomes active when the ActiveGroup disappears

- **Date**: 2026-11-25T02:30:00-05:00
- **Answers**: Q0050
- **Docs**: `systems/controls/selection_and_hud.md > ActiveGroup` (When the ActiveGroup disappears)

## What changed and why
User:
- **Members changed type** (an unfinished building completed, or units were pressed with a mode toggle per 0198): the group they joined becomes active (option A, as M3-U does now). The panel resets to its default state, and targeting in progress ends.
- **Last member died or left the selection** (new rule; the docs had none): the next group in Tab order becomes active, wrapping around. If no group is left, the selection is empty.
- Completions or deaths in a group that isn't active never change the ActiveGroup.

## Implementation impact
M3-U's completion case already matches. New: on the death of the ActiveGroup's last member, advance to the next group in Tab order (this was previously undefined).

## Verification
- Select a Barracks under construction (active) and a finished one. When the first completes, the merged Barracks group is active.
- Select Peacekeepers (active) and Rocketmen, then have all the Peacekeepers die. The Rocketmen group becomes active.
