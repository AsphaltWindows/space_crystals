# unit-command-system

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the unit command system as defined in `artifacts/designer/design/control_system.md` under Unit Command, and all individual command types (Move, Attack, AttackGround, AttackMove, Patrol, HoldPosition, Stop, Reverse, Enter).

**Unit Command:**
An order issued to a unit via a CommandIssuingTransition. Units maintain a command queue; current command is dequeued and sets BaseCommandState. Commands can be queued via shift-click. Commands only mutate BaseCommandState — TurretCommandState is managed by base behavior.

**BaseCommandState[UnitBase]:**
Set by the currently executing command. Contains CommandType, TargetLocation, TargetObject.

**Commands and their BaseCommandState:**

| Command | CommandType | TargetLocation | TargetObject |
|---------|------------|----------------|--------------|
| Move | Move | location OR None | ObjectInstance OR None (one must be set) |
| Attack | Attack | None | Destructible ObjectInstance |
| AttackGround | AttackGround | location | None |
| AttackMove | AttackMove | location | None |
| Patrol | Patrol | location | None |
| HoldPosition | HoldPosition | None | None |
| Stop | Stop | None | None |
| Reverse | Reverse | location | None |
| Enter | Enter | None | Tunnel ObjectInstance |

**Special rules:**
- Move can target a location OR an object (one required)
- Attack targets a specific destructible object
- AttackGround only available if AttackType has CanTargetGround=true
- Reverse only available if UnitBase has CanReverse=true
- Enter only available to Syndicate units when target Tunnel's tier is sufficient for unit's base category
- Commands are queued via shift-click (appended to queue)
- When current command's behavior completes, next command dequeued

## QA Instructions

1. Issue a Move command to ground — verify unit moves to location.
2. Issue a Move command to an object — verify unit moves toward the object.
3. Issue an Attack command on an enemy — verify unit pursues and attacks.
4. Shift-click multiple locations — verify commands queue and execute sequentially.
5. Issue HoldPosition — verify unit stops and doesn't move (even when enemies nearby).
6. Issue Stop — verify unit ceases all activity immediately.
7. Issue Patrol between two points — verify unit walks back and forth.
8. For a CanReverse unit: issue Reverse — verify unit drives backward to location.
9. For a Syndicate unit: issue Enter on an own Tunnel — verify unit walks to Side A and enters the network.
10. Issue AttackGround (if unit supports it) — verify unit attacks a ground location.
11. Issue AttackMove — verify unit moves toward location but engages enemies along the way.
