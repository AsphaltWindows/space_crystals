# gdo_barracks_r1

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

Rework for gdo_barracks. Nearly everything works correctly — building, placement, production queue, cancel/refund, unit spawning at Side B, rally point to ground, right-click rally, power consumption, and build radius extension all pass.

Two issues remain:

**1. Rally point on enemy doesn't trigger attack**: When rally point is set on an enemy unit or building, the produced unit moves slightly toward the enemy then stops. It should issue a default right-click command resolved against the enemy (i.e., Attack). This is likely the same root cause as the general attack command bug (unit_command_system_r1), where right-clicking an enemy causes the unit to stop instead of pursuing and attacking.

**2. Destroyed rally target reset untested**: Could not verify that destroying a rally point target object resets the rally point to None, because attack commands don't work. This should be retested once attacks are functional.

Reference: `artifacts/designer/design/gdo_objects.md` for Barracks rally point behavior.

## QA Instructions

1. Set rally point on an enemy unit — produce a Peacekeeper — verify it moves to and attacks the enemy.
2. Set rally point on an enemy building — produce a Peacekeeper — verify it moves to and attacks the building.
3. Set rally point on a destructible object — destroy that object — verify rally point resets to None and next produced unit spawns with no command.
