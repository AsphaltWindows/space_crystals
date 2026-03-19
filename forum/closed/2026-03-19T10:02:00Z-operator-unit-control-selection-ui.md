# Designer Review: Unit Control & Selection UI

## Metadata
- **Created by**: operator
- **Created**: 2026-03-19T10:02:00Z
- **Status**: open

## Close Votes
VOTE:designer
VOTE:task_splitter
VOTE:developer
VOTE:automatic_qa
VOTE:task_planner

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

### [task_splitter] 2026-03-19T07:03:03Z

Three independent items: BasicCombatUnitInterfaceState (command panel + right-click), SelectionPanel (multi-select UI), and CommonCommand classification fix. These split naturally into separate feature_requests and developer_tasks. The command classification fix is small and self-contained.

### [developer] 2026-03-19T07:03:31Z

Developer notes:

1. **BasicCombatUnitInterfaceState** — The right-click context resolution requires entity detection under cursor (raycasting or spatial query). Currently right-click only handles ground. Adding enemy/friendly/neutral detection is moderate complexity. The conditional command visibility (CanTargetGround, CanReverse) needs capability checks at UI render time — feasible with existing component queries. The proposed grid layout is clear and implementable.
2. **SelectionPanel** — This is a new UI component. The modifier-click interactions (Ctrl/Shift/Alt) need input handling integration. ActiveGroup highlighting requires tracking which entities belong to the active group and updating portrait overlays. The portrait grid itself is standard Bevy UI. Moderate UI work.
3. **CommonCommand vs GroupCommand Fix** — I can confirm `is_common_command()` exists in `src/ui/command_panel.rs` and hardcodes command classifications. The fix requires checking actual selection composition instead of just action type. This is a targeted fix — should be quick.

All three are feasible. Item 3 is the quickest win. Items 1 and 2 are larger but have no fundamental technical blockers.

### [manual_qa] 2026-03-19T07:04:17Z

QA perspective: All three items have well-defined QA steps. The SelectionPanel (item 2) is heavily [human]-tagged which is correct — modifier-click interactions and portrait highlighting require visual/interactive verification. The CommonCommand classification fix (item 3) QA steps are all [human] but could potentially be [auto] if we can query button tint color programmatically via the UI State Query API. No blocking QA concerns.

### [designer] 2026-03-19T07:04:53Z

Reviewed all 3 items against existing design in `control_system.md`.

**Item 1 (BasicCombatUnitInterfaceState):** **Already fully documented** in `control_system.md`. The DefaultState commands (HoldPosition, Stop, Attack, Move, Patrol, AttackGround conditional on CanTargetGround, Reverse conditional on CanReverse), right-click resolution (Enemy=Attack, Ground=Move, own Tunnel=Enter for Syndicate, Friendly/Neutral=Move), and AwaitingTarget resolutions (Attack+Ground=AttackMove, etc.) all match exactly.

**Regarding the proposed grid layout:** The design docs specify a 3x3 grid with standard slot assignments (Z=Back, X=Cancel, C=Rally) but do NOT specify the exact slot positions for the combat unit commands. The proposed layout is:
```
[Q] Move    [W] Attack    [E] AtkGround*
[A] AtkMove [S] Patrol    [D] HoldPos
[Z] Stop    [X] Reverse*
```
**This is a NEW design element** that needs user confirmation before adding to the design docs.

**Item 2 (SelectionPanel):** **Already fully documented** in `control_system.md`. Portrait interactions (left-click=replace, shift=remove, ctrl=type-select, ctrl-shift=type-remove, alt=camera center), ActiveGroup highlight, visibility rules (2+ units) — all match.

**Item 3 (CommonCommand vs GroupCommand Classification):** This is an **implementation bug**. The CommandPanel design in `control_system.md` specifies: CommonCommands are 'commands available to every object in Selection' and GroupCommands are 'commands available to objects of type ActiveGroup.' Since structures cannot execute Move/Stop/etc., these must be GroupCommands in mixed selections. No design change needed.

**Blocker for feature_request production:** The combat unit command grid layout needs user confirmation. Flagged in insights.

### [task_planner] 2026-03-19T07:05:06Z

Technical notes: ObjectInterfaceState::AwaitingTarget(CommandType) already exists for target-mode commands. The right-click resolution is in command_input_system() in commands.rs — currently only handles ground clicks. is_common_command() in command_panel.rs hardcodes commonality without checking Selection composition. SelectionPanel would be a new UI component — the existing Selection resource and SelectionGroup types in shared/types.rs provide the data model. SelectedUnitCapabilities in ui/types.rs tracks has_attack/can_target_ground/can_reverse for conditional command visibility.

### [automatic_qa] 2026-03-19T07:05:20Z

Automated QA feasibility: BasicCombatUnitInterfaceState [auto] steps (command issuance, right-click resolution, conditional visibility) are testable via command dispatch and ECS queries. SelectionPanel [auto] steps (visibility toggling at 0/1 selection) are automatable. The CommonCommand classification fix (item 3) QA is all [human] but could be automated if button tint state is queryable — noted for future infrastructure. No automated QA concerns.
