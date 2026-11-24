# 0070 — Shift keeps targeting active

- **Date**: 2026-11-23T23:59:00-05:00
- **Answers**: Q0016 (point 4; Q0016 is now fully answered)
- **Docs**: `systems/controls/selection_and_hud.md > AwaitingTarget[CommandType]` (new "Shift while targeting" block; the "returns to DefaultState" sentence now notes the Shift exception)

## What changed and why
The user picked option B. A Shift-click on a valid target queues the command, and targeting **stays active while Shift is held**. The player can place several targets or waypoints in a row with one command keypress (A, then Shift-click ×3; patrol routes; GDO air attack runs). Targeting ends and returns to DefaultState when Shift is **released** after at least one queued click. A plain click (no Shift) issues the command and ends targeting, as before.

## Implementation impact
- This replaces the current behaviour, where a Shift-click queues and then leaves targeting.
- Each Shift-click appends one command to each unit's queue, the same way a single Shift-click does today. For group moves, the magic box for each later click is built from the planned positions (`systems/controls/commands.md > Group Destinations (Magic Box)`).
- Invalid targets while Shift is held are ignored with feedback, as in 0065. Targeting stays active.
- The cancel rules are unchanged: Esc, right-click, Z and player selection changes (0069).
- **Still TBD (being asked now):**
  - Releasing Shift before any target was clicked. The proposed default is that targeting simply continues.
  - Whether Shift-placing structures (AwaitingPlacement) works the same way. Until decided, placement keeps its current behaviour.

## Verification
1. Select Peacekeepers, press A, hold Shift and click three enemy buildings, then release Shift. Targeting ends on release. The units attack the three buildings in order.
2. Pick the Move command, hold Shift and click 3 ground points. The units walk the waypoints in order.
3. Press A and click an enemy without Shift. The order is issued and targeting ends immediately.
4. While holding Shift in targeting, click a target the pending command can't use (Inactive pointer). The click is ignored with feedback, and targeting stays.
