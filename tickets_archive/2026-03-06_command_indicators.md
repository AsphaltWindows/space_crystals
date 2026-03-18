# Ticket: Command Indicators for Selected Units

## Current State
When a player selects units and issues commands (Move, Attack, Patrol, etc.), there is no visual feedback at the command's target location or target object. The player has no way to see where their selected units are headed or what they are targeting.

## Desired State
Visual markers (CommandIndicators) are displayed at command targets for all selected units. Two indicator types exist:

- **Location Indicator**: A marker on the ground at target coordinates (used by Move, AttackMove, AttackGround, Patrol, Reverse).
- **Object Indicator**: A marker surrounding the target object's perimeter (used by Attack, Enter).

Indicators are color-coded by intent:
| Command | Indicator Type | Color |
|---------|---------------|-------|
| Move | Location | Green |
| Attack | Object | Red |
| AttackMove | Location | Orange |
| AttackGround | Location | Red |
| Patrol (origin) | Location | Orange |
| Patrol (destination) | Location | Orange |
| Reverse | Location | Green |
| Enter | Object | Green |

Color language: Green = peaceful movement, Red = hostile target, Orange = aggressive movement.

Indicators are only visible when a unit with that active command is part of the current Selection. All active command indicators for selected units are shown simultaneously. Indicators are removed when the unit is deselected or the command completes.

## Justification
Defined in `features/control_system.md` (CommandIndicators section), sourced from `design/control_system.md`. Command indicators are essential RTS feedback — without them, players cannot visually confirm where their units are going or what they are targeting, especially when managing multiple units with different commands.

## QA Steps
1. Start the game and spawn a combat unit (e.g., Peacekeeper).
2. Select the unit.
3. Right-click on open ground to issue a Move command. Verify a **green Location indicator** appears at the target position on the ground.
4. While the unit is moving (still selected), confirm the green indicator remains visible at the destination.
5. Deselect the unit (click empty space). Verify the indicator disappears.
6. Re-select the unit. If the Move command is still active, verify the indicator reappears.
7. Wait for the unit to arrive at the destination. Verify the indicator disappears when the command completes.
8. Select the unit and issue an Attack command on an enemy unit. Verify a **red Object indicator** appears around the enemy unit's perimeter.
9. Select the unit and issue an AttackMove command to a ground location. Verify an **orange Location indicator** appears at the target.
10. Select the unit and issue an AttackGround command (if the unit has CanTargetGround). Verify a **red Location indicator** appears at the target.
11. Select the unit and issue a Patrol command. Verify **two orange Location indicators** appear — one at the patrol origin and one at the patrol destination.
12. If a Reverse-capable unit is available, issue a Reverse command and verify a **green Location indicator** appears at the target.
13. If an Enter-capable unit and valid Tunnel are available, issue an Enter command and verify a **green Object indicator** appears around the Tunnel's perimeter.
14. Select multiple units with different active commands. Verify all active command indicators for all selected units are displayed simultaneously.

## Expected Experience
- When a Move command is issued, a green marker appears on the ground at the clicked location, clearly showing the unit's destination.
- When an Attack command is issued on an enemy, a red marker outlines the targeted enemy, making it obvious which object is being attacked.
- AttackMove and Patrol destinations show orange markers, visually distinguishing aggressive movement from peaceful movement.
- AttackGround shows a red marker on the ground at the targeted location.
- Indicators appear instantly when a unit with an active command is selected and vanish instantly on deselect.
- When a command completes (unit arrives, target dies), the corresponding indicator disappears.
- With multiple units selected, the screen shows all their command indicators at once, giving the player a clear tactical overview of all active orders.
