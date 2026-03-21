# command_indicators

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# command-indicators

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement CommandIndicators as defined in `artifacts/designer/design/control_system.md`.

Visual markers displayed at command targets for selected units. Key rules:
- Only visible when a unit/building with that active command is part of the current Selection
- ALL active command indicators in the selection shown simultaneously
- Removed when unit is deselected or command completes

**Indicator Types:**
- **Location Indicator**: marker on the ground at target coordinates
- **Object Indicator**: marker surrounding the target object's perimeter

**Indicator Assignments:**

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

**Color Language:**
- Green = peaceful movement
- Red = hostile target
- Orange = aggressive movement

## QA Instructions

1. Select a unit with a Move command — verify a green location marker at the move destination.
2. Select a unit attacking an enemy — verify a red indicator surrounding the target enemy.
3. Select a unit on AttackMove — verify an orange location marker at the destination.
4. Select a patrolling unit — verify orange location markers at BOTH the origin and destination.
5. Select a unit with an AttackGround command — verify a red location marker at the ground target.
6. Select a unit with a Reverse command — verify green location marker at destination.
7. Select a Syndicate unit entering a Tunnel — verify green indicator around the Tunnel.
8. Deselect the unit — verify all indicators disappear.
9. Select multiple units with different commands — verify ALL indicators show simultaneously.
10. Wait for a command to complete — verify the indicator disappears.
