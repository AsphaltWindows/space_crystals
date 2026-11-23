# unit_command_system_r1

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

## Content

Rework for unit_command_system. Move commands (right-click ground, right-click object) work correctly. The following parts failed:

**1. Attack command (right-click enemy)**: When a unit is selected and the player right-clicks on an enemy unit or enemy building, the unit stops in place instead of pursuing and attacking the target. The unit should move toward the enemy and engage in combat.

**2. Command queuing (shift-click)**: When shift-right-clicking multiple locations, the unit only moves to the first location and ignores the rest. Expected behavior: each shift-clicked location should be appended to the command queue, and the unit should visit them sequentially.

**3. Hotkeys not functional**: None of the command hotkeys (Q=Move, W=Reverse, E=HoldPosition, etc. as defined in control_system.md) are responding. This blocks testing of all commands that require hotkeys: HoldPosition, Stop, Patrol, Reverse, Enter, AttackGround, AttackMove.

Note: The command panel is also not visible for selected units, but that is a separate feature (command_panel_framework).

Reference: `artifacts/designer/design/control_system.md` for hotkey layout and command definitions.

## QA Instructions

1. Select a unit and right-click on an enemy unit — verify the unit moves toward the enemy and attacks it.
2. Select a unit and right-click on an enemy building — verify the unit moves toward the building and attacks it.
3. Select a unit, shift-right-click three different ground locations — verify the unit visits all three in order (command queue).
4. Select a unit and press E — verify the unit enters HoldPosition (stops moving, stays stationary).
5. Select a unit and verify other hotkeys respond: Q (Move), W (Reverse), A (Attack), S (Patrol), D (AttackGround).
6. Issue a Patrol command (S hotkey, click two points) — verify unit walks back and forth.
7. Issue AttackMove (A hotkey, click ground) — verify unit moves toward location but engages enemies along the way.
8. Issue Stop command — verify unit ceases all activity immediately.
