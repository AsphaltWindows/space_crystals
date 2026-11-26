# 0376 — When a turret's lock ends: as built, and Stop clears it

- **Date**: 2026-11-27T09:45:00-05:00
- **Answers**: Q0137
- **Docs**: `systems/controls/commands.md > TurretCommandState > When a lock ends`

## What changed and why
User: **(a)**, and "you should press stop if you want it to stop targeting the current target".

- A lock lasts while the base engages the target, or while the turret can fire at it (in range, in its arc, seen by its owner). Otherwise the turret drops it and scans in the same tick.
- A Move (or any command that sets no target) **keeps** the lock, including a forced Attack on a friendly unit.
- **Stop clears the turret's lock**; the turret then scans.

## Implementation impact
- As built in M5-U1. Remove the placeholder mark.
- **New:** Stop also clears TurretCommandState.LockedTarget, if it doesn't already.

## Verification
- Force-Attack a friendly unit with a Raptor, then Move nearby: the turret keeps firing at it.
- Press Stop: the turret drops it and scans for enemies.
