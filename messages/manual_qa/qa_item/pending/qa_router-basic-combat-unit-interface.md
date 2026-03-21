# basic-combat-unit-interface

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

## Content

# basic-combat-unit-interface

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the full BasicCombatUnitInterfaceState as defined in `artifacts/designer/design/control_system.md`.

**NOTE: The grid layout for this interface was already sent as a prior feature request (designer-combat-unit-grid-layout, now in done/). The grid slot assignments (Q=Move, W=Reverse, E=HoldPosition, A=Attack, S=Patrol, D=AttackGround, X=Stop) may already be implemented. This request covers the FULL interface including right-click resolution and all AwaitingTarget resolutions, which were not in the previous request.**

**RightClickResolution:**
- Cursor over EnemyObject: issues Attack command targeting that object
- Cursor over Ground: issues Move command to that location
- Cursor over own Tunnel (Syndicate units only, tier sufficient): issues Enter command targeting that Tunnel
- Cursor over FriendlyObject or NeutralObject (all other cases): issues Move command to that object

**AwaitingTarget[Attack] resolution:**
- Left-click EnemyObject: issues Attack command
- Left-click Ground: issues AttackMove command

**AwaitingTarget[Move] resolution:**
- Left-click Ground: issues Move command
- Left-click any Object: issues Move command to that object

**AwaitingTarget[Patrol] resolution:**
- Left-click Ground: issues Patrol command

**AwaitingTarget[AttackGround] resolution:**
- Left-click Ground: issues AttackGround command

**AwaitingTarget[Reverse] resolution:**
- Left-click Ground: issues Reverse command

## QA Instructions

1. Select a combat unit (e.g., Peacekeeper or Guard).
2. Right-click an enemy unit — verify Attack command is issued.
3. Right-click ground — verify Move command is issued.
4. Right-click a friendly non-Tunnel object — verify Move command is issued.
5. (Syndicate unit only) Right-click an own Tunnel — verify Enter command is issued.
6. Press A (Attack), then left-click an enemy — verify Attack command.
7. Press A (Attack), then left-click ground — verify AttackMove command (not plain Move).
8. Press Q (Move), then left-click ground — verify Move command.
9. Press Q (Move), then left-click an object — verify Move to that object.
10. Press S (Patrol), then left-click ground — verify Patrol command.
11. Press Escape or right-click while in any AwaitingTarget — verify return to DefaultState.
12. For a unit with CanReverse: press W, left-click ground — verify Reverse command.
13. For a unit with CanTargetGround: press D, left-click ground — verify AttackGround command.
