# Designer Review: Unit Control & Selection UI

## Metadata
- **Created by**: operator
- **Created**: 2026-03-19T10:02:00Z
- **Status**: open

## Close Votes

## Discussion

### [operator] 2026-03-19T10:02:00Z

The following items address the core unit command interface, the selection panel for multi-unit selections, and a command classification bug. None have been implemented yet. **Designer**: please review and produce `feature_request` messages.

---

### 1. BasicCombatUnitInterfaceState

The command panel and unit command infrastructure lack three critical behaviors:
1. **Conditional command visibility**: All 7 unit commands are always shown regardless of unit capabilities
2. **Right-click context resolution**: Right-click in Default mode only issues Move-to-ground; it doesn't detect enemy/friendly entities
3. **Attack mode ground click**: Left-clicking ground in Attack command mode silently cancels instead of issuing AttackMove

**DefaultState immediate commands (no target required):**
- HoldPosition: issues HoldPosition command
- Stop: issues Stop command

**DefaultState target commands (enters command mode):**
- Attack: enters Attack mode
- Move: enters Move mode
- Patrol: enters Patrol mode
- AttackGround: enters AttackGround mode (only if unit's AttackType has CanTargetGround=true)
- Reverse: enters Reverse mode (only if unit's UnitBase has CanReverse=true)

**Right-click resolution (from DefaultState):**
- Cursor over EnemyObject: issues Attack command targeting that object
- Cursor over Ground: issues Move command to that location
- Cursor over FriendlyObject or NeutralObject: issues Move command to that object

**AwaitingTarget resolutions:**
- Attack + left-click EnemyObject: issues Attack command
- Attack + left-click Ground: issues AttackMove command (NOT silent cancel)
- Move + left-click Ground/Object: issues Move command
- Patrol + left-click Ground: issues Patrol command
- AttackGround + left-click Ground: issues AttackGround command
- Reverse + left-click Ground: issues Reverse command

**Proposed grid layout:**
```
[Q] Move    [W] Attack    [E] AtkGround*
[A] AtkMove [S] Patrol    [D] HoldPos
[Z] Stop    [X] Reverse*
```
(* = conditional on unit capabilities)

**QA Steps:**
1. [human] Select a combat unit. Verify the command panel shows HoldPosition, Stop, Attack, Move, Patrol. Verify AttackGround appears only if CanTargetGround=true. Verify Reverse appears only if CanReverse=true.
2. [auto] Click HoldPosition. Verify HoldPosition command is issued immediately.
3. [auto] Click Stop. Verify Stop command is issued immediately.
4. [human] Click Attack. Verify AwaitingTarget[Attack] is entered. Left-click an enemy. Verify Attack command targeting that enemy is issued.
5. [auto] In AwaitingTarget[Attack], left-click ground. Verify AttackMove command is issued.
6. [auto] Click Move. Left-click ground. Verify Move command. Click Move again, left-click a friendly object. Verify Move command to that object.
7. [auto] Click Patrol. Left-click ground. Verify Patrol command.
8. [auto] For a unit with CanTargetGround: click AttackGround. Left-click ground. Verify AttackGround command.
9. [auto] For a unit with CanReverse: click Reverse. Left-click ground. Verify Reverse command.
10. [auto] Right-click an enemy unit. Verify Attack command is issued.
11. [auto] Right-click ground. Verify Move command.
12. [auto] Right-click a friendly unit. Verify Move command.
13. [auto] Right-click a neutral object. Verify Move command.
14. [auto] For a unit without CanTargetGround: verify AttackGround does not appear.
15. [auto] For a unit without CanReverse: verify Reverse does not appear.

---

### 2. SelectionPanel

No selection panel currently exists. When multiple units are selected, a simple multi-select card grid renders with no click interactions and no ActiveGroup highlight. The SelectionPanel should be a grid of unit portraits with click interactions and ActiveGroup highlighting, visible when Selection contains 2+ entities.

**ActiveGroup highlight:** Portraits of units in the ActiveGroup get a semi-transparent overlay (e.g., white at 15% opacity). Highlight updates immediately when active group changes (e.g., Tab press).

**Portrait click interactions:**
- **Left-click (no modifier)**: Clear all Selected, insert Selected on portrait's entity only
- **Shift-click**: Remove that unit from selection
- **Ctrl-click**: Select all entities of same ObjectEnum type from current selection
- **Ctrl-Shift-click**: Remove all entities of same ObjectEnum type from selection
- **Alt-click**: Center camera on that unit's position (no selection change)

**QA Steps:**
1. [human] Select 2+ owned units. Verify the SelectionPanel appears as a grid of portraits.
2. [human] Verify portraits of units in the ActiveGroup have a sheer highlight. Verify others do not.
3. [human] Press Tab to change ActiveGroup. Verify the highlight moves to the new ActiveGroup's portraits.
4. [auto] Select exactly 1 unit. Verify the SelectionPanel is hidden.
5. [auto] Select 0 units (click empty ground). Verify the SelectionPanel is hidden.
6. [human] With 3+ units selected, left-click a portrait. Verify the selection is replaced with only that unit and the SelectionPanel hides.
7. [human] With 3+ units selected, shift-click a portrait. Verify that unit is removed from the selection.
8. [human] With 3+ units selected (including 2+ of the same type), ctrl-click a portrait. Verify the selection is replaced with all units of that type from the previous selection.
9. [human] With a mixed-type selection, ctrl-shift-click a portrait. Verify all units of that type are removed from the selection.
10. [human] With 2+ units selected, alt-click a portrait. Verify the camera centers on that unit. Verify the selection does not change.
11. [human] With exactly 2 units selected, shift-click one portrait. Verify the selection reduces to 1 unit and the SelectionPanel hides.
12. [human] With 2 units of the same type selected, ctrl-shift-click a portrait. Verify all units of that type are removed, emptying the selection entirely.

---

### 3. Fix CommonCommand vs GroupCommand Classification

**BUG:** When a unit and a structure are selected together (with the unit as the ActiveGroup), the commands Move, Stop, HoldPosition, and Patrol appear as CommonCommands (green tint, indicating shared by all selected entities). However, structures do not support any of these commands -- they should appear as GroupCommands (yellow tint, specific to the ActiveGroup).

**Root cause:** `is_common_command()` hardcodes certain CommandButtonAction variants as "common" based solely on action type, without considering the actual selection composition. It does not check whether all groups in the Selection can actually execute the command.

**Correct behavior:** A command should only be classified as Common if **every object in the Selection** can execute it. Since structures cannot execute Move, Stop, HoldPosition, or Patrol, these must appear as GroupCommands in any mixed unit+structure selection. When all groups are unit types, unit commands ARE common across them.

This affects both visual rendering (button green vs yellow tint) and command dispatch (which entities receive the command).

**QA Steps:**
1. [human] Select a unit and a structure simultaneously (box-select or shift-click).
2. [human] Confirm the unit is the ActiveGroup.
3. [human] Observe the CommandPanel -- verify that **all** unit commands appear as **group-specific** commands (yellow tint, not green).
4. [human] Select only a single unit (no structure). Verify all unit commands appear normally.
5. [human] Repeat steps 1-3 with a Syndicate Agent and Syndicate Tunnel to confirm cross-faction correctness.

---

### Key questions for the designer:
- Is the proposed grid layout for combat unit commands correct? Should the slot assignments be different?
- Are the conditional commands (AttackGround, Reverse) the only capability-gated commands, or should others be conditional too?
- For the SelectionPanel, is the modifier-click behavior (Ctrl/Shift/Alt) the standard RTS convention you want?
- Should Alt-click center the camera smoothly (pan animation) or snap instantly?
- For the command classification fix -- should mixed unit+structure selections ever show CommonCommands, or is "all GroupCommands" always correct for mixed selections?
