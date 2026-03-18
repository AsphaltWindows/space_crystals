# Ticket: BasicCombatUnitInterfaceState

## Current State
The command panel and interface flow framework exists (from the CommandPanel/InterfaceFlow ticket), but no concrete ObjectInterfaceState template is implemented for any object type.

## Desired State
Implement BasicCombatUnitInterfaceState as the first concrete ObjectInterfaceState template for combat units:

**DefaultState immediate commands** (CommandIssuingTransition, no target required):
- `HoldPosition`: issues HoldPosition command
- `Stop`: issues Stop command

**DefaultState target commands** (StateOnlyTransition to AwaitingTarget):
- `Attack`: enters AwaitingTarget[Attack]
- `Move`: enters AwaitingTarget[Move]
- `Patrol`: enters AwaitingTarget[Patrol]
- `AttackGround`: enters AwaitingTarget[AttackGround] (only if unit's AttackType has CanTargetGround=true)
- `Reverse`: enters AwaitingTarget[Reverse] (only if unit's UnitBase has CanReverse=true)

**RightClickResolution** (from DefaultState):
- Cursor over EnemyObject: issues Attack command targeting that object
- Cursor over Ground: issues Move command to that location
- Cursor over own Tunnel (Syndicate units only, target Tunnel tier sufficient for unit's base category): issues Enter command targeting that Tunnel
- Cursor over FriendlyObject or NeutralObject: issues Move command to that object

**AwaitingTarget[Attack] resolution**:
- Left-click EnemyObject: issues Attack command targeting that object
- Left-click Ground: issues AttackMove command to that location

**AwaitingTarget[Move] resolution**:
- Left-click Ground: issues Move command to that location
- Left-click any Object: issues Move command to that object

**AwaitingTarget[Patrol] resolution**:
- Left-click Ground: issues Patrol command to that location

**AwaitingTarget[AttackGround] resolution**:
- Left-click Ground: issues AttackGround command to that location

**AwaitingTarget[Reverse] resolution**:
- Left-click Ground: issues Reverse command to that location

**Conditional command availability**:
- AttackGround only appears if the unit's AttackType.CanTargetGround = true
- Reverse only appears if the unit's UnitBase.CanReverse = true

## Justification
Required by `features/control_system.md`. This is the concrete command interface for all standard combat units. Without it, players cannot issue commands to combat units. Depends on the CommandPanel/InterfaceFlow ticket and on `features/unit_system.md` (UnitBase.CanReverse) and `features/combat_system.md` (AttackType.CanTargetGround).

## QA Steps
1. Select a combat unit. Verify the command panel shows HoldPosition, Stop, Attack, Move, Patrol. Verify AttackGround appears only if the unit's attack type has CanTargetGround=true. Verify Reverse appears only if the unit's base has CanReverse=true.
2. Click HoldPosition. Verify HoldPosition command is issued immediately (no target selection).
3. Click Stop. Verify Stop command is issued immediately.
4. Click Attack. Verify AwaitingTarget[Attack] is entered. Left-click an enemy. Verify Attack command targeting that enemy is issued.
5. In AwaitingTarget[Attack], left-click ground. Verify AttackMove command to that location is issued.
6. Click Move. Left-click ground. Verify Move command to that location. Click Move again, left-click a friendly object. Verify Move command to that object.
7. Click Patrol. Left-click ground. Verify Patrol command to that location.
8. For a unit with CanTargetGround: click AttackGround. Left-click ground. Verify AttackGround command to that location.
9. For a unit with CanReverse: click Reverse. Left-click ground. Verify Reverse command to that location.
10. Right-click an enemy unit. Verify Attack command is issued targeting that enemy.
11. Right-click ground. Verify Move command to that location.
12. Right-click a friendly unit. Verify Move command to that object.
13. Right-click a neutral object. Verify Move command to that object.
14. With a Syndicate unit selected, right-click an own Tunnel whose tier is sufficient for the unit's base category. Verify Enter command is issued targeting that Tunnel.
15. With a Syndicate unit selected, right-click an own Tunnel whose tier is NOT sufficient for the unit's base category. Verify Move command is issued (falls through to Friendly resolution), not Enter.
16. With a non-Syndicate unit selected, right-click a Tunnel. Verify Move command is issued (no Enter resolution for non-Syndicate units).
17. For a unit without CanTargetGround: verify AttackGround does not appear in the command panel.
18. For a unit without CanReverse: verify Reverse does not appear in the command panel.

## Expected Experience
- Combat unit selection shows a clean command panel with all applicable commands.
- Immediate commands (HoldPosition, Stop) execute instantly on click.
- Target commands show visual feedback during target selection.
- Right-clicking feels natural: enemies get attacked, own Tunnels (Syndicate, tier sufficient) get entered, everything else gets moved to.
- AwaitingTarget[Attack] on ground smartly resolves to AttackMove instead of failing.
- Conditional commands only appear when the unit supports them.
