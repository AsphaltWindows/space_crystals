# Controls: Selection, Interface and HUD

## ControlState
The client-side state of a player's control interface. Contains the player's current Selection and the ObjectInterfaceState for the active selection. ControlState is not part of the game simulation — it lives entirely on the client. Each tick, ControlState is validated against the current game state: selected objects that no longer exist are removed from the Selection, and if the ObjectInterfaceState becomes invalid as a result, it resets to the default state for the current selection.

### Selection - Selection
### ObjectInterfaceState[ActiveGroup] - ObjectInterfaceState[ObjectEnum]

## Selection
The set of ObjectInstances currently selected by the player, grouped by object type. Each group contains references to all selected instances of that type. The ActiveGroup determines which group's commands are displayed in the command panel and which ObjectInterfaceState is active.

### Groups - array of SelectionGroup
### ActiveGroup - SelectionGroup | None (None when selection is empty). Usually identified by its object type; for Ungroupable objects each instance is its own SelectionGroup, so ActiveGroup points at one specific group, and GroupCycling (Tab) steps through them one by one.

### Constraints:
- If any selected object is not owned by the player, the selection must contain exactly one object and one group
- If all selected objects are owned by the player, no limit on count or mixing of types
- Ungroupable objects (Groupable = false) always occupy their own SelectionGroup, even when selected alongside other instances of the same type
- ActiveGroup must be a type that exists in the current Groups

## SelectionGroup
A group of selected ObjectInstances that share the same object type. For Groupable objects, all selected instances of that type are combined into one group. For Ungroupable objects, each instance is its own group.

### Type - ObjectEnum
### Instances - array of ObjectInstance references

### No smart-casting
A command issued from the command panel (hotkey or button) goes to **every instance it is issued to** (the whole Selection for a common command, the whole ActiveGroup for a group command; see CommandPanel). The game **never hands a group's command to just one member** on the player's behalf ("smart-casting"), for any command (user, 2026-11-23).

Rationale (user): smart-casting makes mass use of abilities effortless, which constrains how powerful unit abilities can realistically be. Workers follow the same rule.

**Individual use is done through Groupable** (user, 2026-11-23). Groupable is fixed per object type, chosen by the design:
- **Groupable = false** for units whose commands are used **one unit at a time**: **workers that are used up by a build** (Syndicate Agent, Colonists Drone: each build or deploy order needs one specific worker) and **caster-type units** (one cast should spend one unit's energy, not the whole group's). Each instance is its own SelectionGroup, so the player casts with one unit, presses **Tab** to the next, and casts again, with no mouse reselection. This is the intended middle ground: easier than picking units out with the mouse, harder than smart-casting.
- **Groupable = true** for units whose abilities are normally used **all together** (e.g. the Peacekeeper's Riot Shield), including workers that **share** a build: Cults Recruits all build the same building together, so Construct goes to every selected Recruit (user, 2026-11-24).

To use a Groupable unit's ability on only one instance, the player selects that instance alone (e.g. left-click its portrait in the SelectionPanel).

**Group production with limited resources** (user, 2026-11-24): when a production command (e.g. Q: Peacekeeper) goes to several selected production buildings and the player can't pay for all of them (crystals, GDO Unit Control or any other cost), **as many as possible** queue it. The buildings are served in order of **least work queued first**: the least total remaining production time in their queue (including the unit in production), ties broken in a fixed, deterministic order. The rest don't queue (insufficient-resources feedback). This spreads production across idle buildings first, without picking a single building for the player.

Considered and **parked** (user, 2026-11-23): a player-controlled toggle of a type's grouping (e.g. from the SelectionPanel portraits). Start with the fixed per-type flag; revisit if needed.

## BoxSelection
When the player drags a selection box, the objects captured are filtered by a 5-tier priority system. The highest-priority tier containing any objects wins; all lower tiers are excluded.

### Priority tiers (highest to lowest):
1. Own units — all own units in the box are selected (multi-select)
2. Own buildings — single-select: the own building closest to the box center
3. Enemy units — single-select: the enemy unit closest to the box center
4. Enemy buildings — single-select: the enemy building closest to the box center
5. Neutral objects — single-select: the neutral object closest to the box center

**Allied objects** (once teams exist) are other players' objects and are never multi-selected (user, 2026-11-23). For the priority tiers they are treated like enemy objects (placeholder until teams are designed).

Only tier 1 (own units) produces a multi-selection. All other tiers select exactly one object — the one closest to the center of the box.

### Shift + box (add)
Holding **Shift** while dragging a box **adds** the box's result (after the priority tiers) to the current selection. The current selection **always stays selected**: Shift + box never removes anything, unlike Shift-click, which toggles (user, 2026-11-23). Objects that can't be added under Selection > Constraints (e.g. an enemy, neutral object or other player's object when own objects are selected) are not added, and the selection stays unchanged.

**Adding to a non-owned selection** (user, 2026-11-23): when the current selection is a single enemy, neutral or other player's object, any add input (Shift + box, Shift-click, Ctrl+Shift-click on own units) does **nothing**: the inspected object stays selected. Use a plain click or box to select own units instead.

### Empty clicks and boxes (user, 2026-11-23)
- A plain click on empty ground, or a plain box that captures nothing, **clears** the selection.
- Shift-click or Ctrl-click on empty ground does **nothing** (selection kept).
- Ctrl+Shift-click on empty ground also does nothing. (Ctrl+Shift-click on an own unit: see MapClickSelection.)

## MapClickSelection
Selection by clicking objects on the map (as opposed to portraits in the SelectionPanel). Default inputs (user, 2026-11-23):

| Input | Effect |
|-------|--------|
| Left-click object | Replace selection with that object |
| Shift-click own unit | Toggle that unit in the selection: add it if not selected, remove it if selected |
| Ctrl-click own unit | Select all own units of that type currently on screen |
| Double-click own unit | Same as Ctrl-click |
| Ctrl+Shift-click own unit | **Add** all own units of that type currently on screen to the current selection (never removes; user, 2026-11-23) |

The Selection Constraints above still apply (e.g. an enemy or neutral object is always selected alone).

## ControlGroups
A set of 10 saved selections (indexed 0-9) stored per player as part of ControlState. Each control group holds an array of ObjectInstance references. An entity can belong to multiple control groups simultaneously. Control groups persist for the duration of the game. When a control group is recalled, destroyed entities are silently removed from the group.

### Groups - array of 10 ObjectInstance reference arrays

**Own objects only** (user, 2026-11-23): only the player's own objects can be put into a control group. Assign or Add with a non-owned object selected does nothing.

### Operations:
- Assign: replace a control group's contents with the current Selection
- Add: merge the current Selection into an existing control group (no duplicates)
- Recall: replace the current Selection with the control group's contents
- Recall and Center: replace the current Selection with the control group's contents and center the camera on the group's centroid

### Default keys (user, 2026-11-23):
- Assign: **Ctrl + 0–9**
- Add: **Shift + 0–9**
- Recall: **0–9**
- Recall and Center: **double-tap 0–9** (press the same number twice quickly)

## Key Rebinding
All default inputs in the controls docs (map clicks, control-group keys, command panel hotkeys, camera panning, command queueing) are **defaults**. Players will be able to **rebind** them; when rebinding is added is up to implementation scheduling (user, 2026-11-23).

## ObjectInterfaceState[ObjectEnum]
The current state of the command flow for the active selection group. Parameterized by ObjectEnum — each object type defines its own concrete set of interface states and transitions. Reset to the default state when the player changes the Selection or ActiveGroup, **and on every control group recall even if it gives the same selection** (user, 2026-11-24: a recall refreshes the selection, and all menus reset; players shouldn't be pressing group keys in the middle of a multi-step command). This applies to every non-default state: AwaitingTarget, AwaitingPlacement, BuildMenu and other sub-menus (automatic removals such as deaths don't reset it unless the ActiveGroup becomes empty; see AwaitingTarget > Selection changes while targeting). The available InterfaceTransitions from any given ObjectInterfaceState depend on the game state of the instances in the active SelectionGroup. Each tick, the ObjectInterfaceState is validated against the active SelectionGroup's game state — if the current state is no longer valid (e.g., a structure's construction completed, changing what commands are available), it resets to the default state. For Ungroupable objects, the SelectionGroup always contains exactly one instance, so the ObjectInterfaceState can reliably read that instance's specific game state.

## InterfaceTransition
A transition from one ControlState to another, triggered by player input. Each InterfaceTransition either only modifies the ControlState or also issues a Command to the selected objects.

### Value - InterfaceTransitionEnum

## StateOnlyTransition
An InterfaceTransition that only modifies ControlState. No Command is issued to any game object. Examples: pressing an attack hotkey to enter target selection, opening a build menu, cycling the active group.

## CommandIssuingTransition
An InterfaceTransition that modifies ControlState and also issues a Command to one or more selected objects. Examples: left-clicking a target after pressing attack, confirming a building placement.

## GroupCycling
A StateOnlyTransition that changes the ActiveGroup to an adjacent SelectionGroup in the Selection.

### Forward (Tab): advance ActiveGroup to the next SelectionGroup. Wraps around to the first group after the last.
### Backward (Shift-Tab): advance ActiveGroup to the previous SelectionGroup. Wraps around to the last group before the first.

## HUD Bar
The bottom section of the screen, containing all selection-related panels. The HUD Bar is a horizontal strip divided into five fixed-position sections, arranged left to right. All sections share the same vertical height. Each section occupies a locked horizontal slot — panels appear and disappear in place but never shift position based on selection state. Empty slots remain visually present (their space is reserved) even when their panel is not active.

### Sections (left to right):

| Section | Visibility Rule |
|---------|----------------|
| Minimap | Always visible |
| Portrait | Visible when at least 1 unit or object is selected |
| InfoPanel | Visible when at least 1 object is selected |
| SelectionPanel | Visible when at least 2 objects are selected |
| CommandPanel | Visible when at least 1 player-owned object is selected |

### InfoPanel with 2+ objects selected
Placeholder (bookmarked by the user, 2026-11-23, to be designed with the HUD): shows the ActiveGroup's type, how many objects are in that group, and the total number selected.

### Layout Rules:
- All five sections have identical vertical height
- Horizontal positions are fixed — sections never shift when adjacent panels appear or disappear
- When a section's visibility condition is not met, the slot remains reserved (empty space, no content)

## SelectionPanel
Displays a grid of portraits for the current selection. Only visible when 2 or more objects are selected. A selection of 2+ objects always consists of the player's own objects (units and/or structures, e.g. from a control group), since other players' and neutral objects are always selected alone (Selection > Constraints). "Unit" below means any selected object. Each cell shows the portrait of one selected unit instance. Portraits belonging to the ActiveGroup are shown with a sheer highlight.

**Bookmarked for the HUD pass** (user, 2026-11-23): how the SelectionPanel presents the selection's grouping (portrait arrangement by SelectionGroup, the order of groups, which also sets the GroupCycling order, and how the ActiveGroup is shown). Until then: portraits are arranged group by group, in a stable order of the implementation's choosing, with the ActiveGroup highlighted as above.

### Visibility:
- Visible when Selection contains 2+ objects
- Hidden when Selection contains 0 or 1 objects (single-object display is handled by the InfoPanel)

### Interactions:

| Input | Effect |
|-------|--------|
| Left-click portrait | Replace selection with only that unit |
| Shift-click portrait | Remove that unit from selection |
| Ctrl-click portrait | Replace selection with all units of that type in the current selection |
| Ctrl-Shift-click portrait | Remove all units of that type from selection |
| Alt-click portrait | Center camera on that unit (no selection change) |

### Edge cases:
- If an action reduces the selection to 1 unit, the SelectionPanel hides and the InfoPanel displays that unit
- If an action removes all remaining units, the selection becomes empty

## CommandPanel
The display of available commands for the current selection, derived from ControlState and game state each tick. The CommandPanel is only displayed when all objects in the Selection are owned by the player. When selecting enemy or neutral objects, only the InfoPanel is shown — no commands are available, no right-click resolution is performed, and no InterfaceTransitions can be initiated. Commands can only be issued to objects owned by the player.

When visible, commands available to all objects in the Selection are visually distinguished from commands available only to the ActiveGroup. Clicking a common command issues it to all selected objects. Clicking a group-specific command issues it only to objects in the ActiveGroup.

### CommonCommands - commands available to every object in Selection
### GroupCommands - commands available to objects of type ActiveGroup

### Grid Layout
The CommandPanel is a 4x3 grid of command slots (4 columns, 3 rows). Each slot has a default hotkey:

```
| Q | W | E | R |
| A | S | D | F |
| Z | X | C | V |
```

### Standard Slot Assignments
Certain slots have standardized functions across all object types that use them. Slots are identified by their **hotkey letter** in the 4x3 grid above (Z, X, C = the first three slots of the bottom row):

- **Z (bottom row, 1st slot / bottom-left)**: Back / Cancel menu. In any multi-stage menu (BuildMenu, ExpandMenu, EjectMenu, AwaitingTarget, AwaitingPlacement), Z returns to the previous state (StateOnlyTransition). Equivalent to Escape or right-click cancel.
- **X (bottom row, 2nd slot)**: Cancel Production / Cancel Upgrade. In production buildings, cancels the last queued item and refunds cost. In structures with upgrades, cancels the in-progress upgrade and refunds cost.
- **C (bottom row, 3rd slot)**: Set Rally Point. In unit-producing structures, enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState). Clicking the **producing building itself** clears the rally point (RallyPoint = None; user, 2026-11-24).
- **V (bottom row, 4th slot / bottom-right)**: no standard function; free for object-specific commands.

### Production Building Default Right-Click
For all unit-producing structures, right-click from DefaultState sets the rally point:
- Right-click Ground: sets rally point to that location
- Right-click Object: sets rally point to that object
- Right-click the building itself: **clears** the rally point (RallyPoint = None; user, 2026-11-24)

### Rally points on enemy objects
(user, 2026-11-24) A rally point on an enemy object only follows that object **while the owner can see it**. As soon as the enemy object leaves the owner's sight, the rally point **turns into a ground rally point at its last-seen position**. So:
- A unit produced while the enemy is visible attacks it; one produced after the enemy was lost from sight moves to the last-seen position (matching chasing a target into the fog: systems/controls/behaviors.md > AttackingObject).
- An enemy dying unseen in the fog doesn't change the rally point, so the rally marker never reveals hidden information. (The "reset to None when the object no longer exists" rule applies to own and neutral objects, and to enemies only while visible.)
- Rallying on a remembered (fogged) enemy structure: TBD with Director question Q0017 point 2.

## DefaultState
The default ObjectInterfaceState for a selection. The command panel displays available commands. Right-click performs a context-sensitive command based on what is under the cursor. Command hotkeys or button clicks initiate transitions to AwaitingTarget or issue immediate commands.

## RightClickResolution
Context-sensitive command issued by right-clicking from DefaultState. The command is determined by what is under the cursor and what the selected objects can do. **Each selected unit resolves the right-click for itself**, against its own capabilities, not the ActiveGroup's (user, 2026-11-23). The result doesn't depend on which group is active.

**Units that can't carry out the resolved command** (user, 2026-11-23): every selected unit acts on a right-click. Units that have the resolved command carry it out. Units that can't, e.g. an unarmed Supply Chopper, or ground-only Peacekeepers when the target is an aircraft, **move to or follow the target instead**: Move to the location, or Follow a unit (systems/controls/behaviors.md > MovingToObject). The group stays together.

## AwaitingTarget[CommandType]
An ObjectInterfaceState entered after the player selects a command that requires a target. The command panel indicates which command is pending. Left-clicking a valid target issues the command via a CommandIssuingTransition and returns to DefaultState (except while Shift is held; see Shift while targeting). The target may resolve to a different command than the one that initiated the state, depending on what is under the cursor. Escape or right-click cancels via a StateOnlyTransition back to DefaultState.

**Other commands while targeting** (user, 2026-11-23): while a target is awaited, the command panel shows the pending command and Back/Cancel (Z). The other commands are **unavailable**: pressing their hotkeys or buttons does nothing. To change commands, the player cancels first (Escape, right-click or Z), then picks the new command.

**Dragging while targeting** (user, 2026-11-23): a left-button drag while a target is awaited does **nothing**. While targeting, a press and release only counts as a **drag** if the mouse moved more than a **larger click allowance** between press and release: **16 pixels** (placeholder, half a tile; user, 2026-11-24), instead of the normal drag threshold. Within the allowance it is a targeting click, so a slightly sloppy click on a moving enemy isn't lost; a deliberate drag away before release is a way to abort a press on the wrong target. On a drag: no selection box is drawn, nothing is selected, and targeting stays active. A targeting click counts at the **press point** (where the button went down), even if the mouse moves before release (user, 2026-11-23). The command matches what was under the cursor when the player committed (e.g. pressing on an enemy and releasing on the ground still issues Attack on that enemy).

**Selection changes while targeting** (user, 2026-11-23):
- **Player-initiated** selection or ActiveGroup changes (control group recall, Tab / Shift-Tab, portrait clicks) **cancel** targeting, per ObjectInterfaceState's reset rule. A control group recall cancels it **even if it recalls exactly the current selection** (user, 2026-11-24).
- **Automatic removals** (a selected unit dies, or leaves the selection by entering a building, transport or tunnel) do **not** cancel targeting. Targeting is only cancelled if the **whole ActiveGroup** is gone, since nothing would be left to carry out the command.

**Shift while targeting** (user, 2026-11-23): a Shift-click on a valid target **queues** the command (systems/controls/commands.md > Unit Command) and targeting **stays active while Shift is held**, so the player can place several targets or waypoints in a row with one command keypress (e.g. A, then Shift-click, Shift-click, Shift-click). Each Shift-click queues one more command. Targeting ends (back to DefaultState) when Shift is **released** after at least one queued click. A click without Shift issues the command and ends targeting as usual. TBD: releasing Shift before any target was clicked (presumably targeting simply continues, since nothing was issued).

**Shift while placing** (AwaitingPlacement; user, 2026-11-23): the same rule applies. A Shift-click on a valid spot queues the build order and placement **stays active while Shift is held**, so one builder can be given several buildings in a row; releasing Shift ends placement. This only applies when the builder **can carry out more queued buildings**: e.g. Cults Recruits (the whole selected group) build one and walk together to the next (each order's cost is paid when it is given: factions/cults/economy.md > Build Command). When the builder **can't**, a Shift-click sends it and ends placement like a plain click:
- an Agent goes inside the structure it builds (factions/syndicate/units.md > Tunnel Construction Flow), and a Drone becomes the Beacon (factions/colonists/units.md), so each takes exactly one placement;
- a GDO Deployment Center or Extraction Facility only ever has one building ready to place.

Placement **never moves on to another selected worker** automatically (No smart-casting, above): to place with the next Agent or Drone, the player presses Tab and issues the next order.

**Invalid target clicks** (user, 2026-11-23): a left-click on a target the pending command can't use (shown by the Inactive pointer) is **ignored**. Targeting stays active and waits for a valid target, and the player gets short **feedback** (an "invalid target" message and/or sound; exact form TBD, ties into the future notification system).

### PendingCommand - CommandType
### Resolution - (CursorTarget) -> Command

## CursorTarget
What is under the player's cursor at the time of a click. Used by AwaitingTarget and RightClickResolution to determine which command to issue.

### Value - CursorTargetEnum (Ground | EnemyObject | FriendlyObject | NeutralObject)
### Location - Coordinates
### Object - ObjectInstance | None

## PointerDisplayType
The visual appearance of the player's cursor, communicating what action will occur on click. Determined each tick by the current ControlState, the selected units' capabilities, and the CursorTarget. Only one PointerDisplayType is active at a time.

### Types:

| Type | Visual Feel | Description |
|------|------------|-------------|
| Inactive | Muted, subdued | Nothing will happen on click. Shown when nothing is selected, or when hovering over an invalid target for the current context. |
| Move | — | Unit will move to this location or object. |
| Attack | — | Unit will attack this target. |
| AttackGround | — | Unit will attack this ground location. |
| Patrol | — | Unit will patrol to this location. |
| GatherResources | — | Unit will gather from this resource. |
| ReturnResources | — | Unit will return carried resources to this drop-off point. |
| Enter | — | Unit will enter this object (e.g., Syndicate unit entering a Tunnel, Cults infantry entering an Altar). |
| Mixed | Generic hybrid | Selected units will do different things on right-click (e.g. some attack, some follow). Visual TBD. |

### Mixed Right-Click Preview
When a right-click would make different selected units do different things (e.g. Peacekeepers would attack and a Supply Chopper would follow), the pointer shows a **generic hybrid cursor** that signals mixed actions (user, 2026-11-23). When all selected units would do the same thing, the normal cursor for that action is shown. Visual design of the hybrid cursor: TBD (art).

### Resolution Rules

PointerDisplayType is resolved based on the current ObjectInterfaceState:

#### DefaultState (right-click preview):
The pointer previews the action that right-click would perform, resolved per unit (RightClickResolution). If all selected units would do the same thing, that action's pointer is shown, per the list below. If they would do different things, the **Mixed** pointer is shown (Mixed Right-Click Preview):

- Nothing selected: **Inactive**
- Cursor over enemy object (and selection can attack): **Attack**
- Cursor over ground (and selection can move): **Move**
- Cursor over friendly/neutral object (and selection can move): **Move**
- Cursor over own Tunnel (Syndicate unit, tier sufficient): **Enter**
- Cursor over own enterable Cults building that accepts the selected unit (see Enter): **Enter**
- Cursor over resource (resource gatherer selected): **GatherResources**
- Cursor over drop-off point (resource gatherer carrying resources): **ReturnResources**
- Selection is a production building: **Move** (rally point)
- No valid action for current cursor target: **Inactive**

#### AwaitingTarget[CommandType]:
The pointer reflects the pending command and whether the current CursorTarget is valid:

- AwaitingTarget[Attack] over enemy object: **Attack**
- AwaitingTarget[Attack] over ground: **Attack** (will issue AttackMove)
- AwaitingTarget[Attack] over own or neutral **destructible** object: **Attack** (force-attack, systems/controls/commands.md > Attack)
- AwaitingTarget[Attack] over an indestructible object (e.g. crystal patch): **Inactive**
- AwaitingTarget[Move] over ground or any object: **Move**
- AwaitingTarget[Patrol] over ground: **Patrol**
- AwaitingTarget[Patrol] over non-ground: **Inactive**
- AwaitingTarget[AttackGround] over ground: **AttackGround**
- AwaitingTarget[AttackGround] over non-ground: **Inactive**
- AwaitingTarget[Reverse] over ground: **Move**
- AwaitingTarget[Reverse] over non-ground: **Inactive**
- AwaitingTarget[ScheduleDeliveries] over valid target: **GatherResources**
- AwaitingTarget[ScheduleDeliveries] over invalid target: **Inactive**
- AwaitingTarget[SetRallyPoint] over ground or object: **Move**

#### AwaitingPlacement:
No pointer display type — the building ghost preview serves as the cursor.

## BasicCombatUnitInterfaceState
The ObjectInterfaceState template used by combat units. Defines the available commands, right-click resolution, and awaiting-target resolution rules common to most combat units.

### DefaultState commands:

Grid layout:
```
[Q] Move      [W] Reverse*    [E] HoldPosition  [R] —
[A] Attack    [S] Patrol      [D] AttackGround*  [F] —
[Z] —         [X] Stop        [C] —              [V] —
```
*Conditional: W (Reverse) only available if UnitBase has CanReverse = true. D (AttackGround) only available if AttackType has CanTargetGround = true.*

Immediate commands (CommandIssuingTransition, no target required):
- **E: HoldPosition**: issues HoldPosition command
- **X: Stop**: issues Stop command

Target commands (StateOnlyTransition to AwaitingTarget):
- **A: Attack**: enters AwaitingTarget[Attack]
- **Q: Move**: enters AwaitingTarget[Move]
- **S: Patrol**: enters AwaitingTarget[Patrol]
- **D: AttackGround**: enters AwaitingTarget[AttackGround] (only available if unit's AttackType has CanTargetGround = true)
- **W: Reverse**: enters AwaitingTarget[Reverse] (only available if unit's UnitBase has CanReverse = true)

### RightClickResolution:
- Cursor over EnemyObject: issues Attack command targeting that object
- Cursor over Ground: issues Move command to that location
- Cursor over own Tunnel (Syndicate units only, tier sufficient): issues Enter command targeting that Tunnel
- Cursor over own Altar (Cults infantry only — Soldier, Gunner, Acolyte): issues Enter command targeting that Altar
- Cursor over own Garrison (Cults infantry only — Soldier, Gunner, Acolyte): issues Enter command targeting that Garrison
- Cursor over own Bunker (GDO infantry): issues Enter command targeting that Bunker
- Cursor over FriendlyObject or NeutralObject (all other cases): issues Move command to that object

### AwaitingTarget[Attack] resolution:
- Left-click EnemyObject: issues Attack command targeting that object
- Left-click own or neutral destructible object: issues Attack command targeting that object (force-attack, systems/controls/commands.md > Attack)
- Left-click Ground: issues AttackMove command to that location

### AwaitingTarget[Move] resolution:
- Left-click Ground: issues Move command to that location
- Left-click any Object: issues Move command to that object

### AwaitingTarget[Patrol] resolution:
- Left-click Ground: issues Patrol command to that location

### AwaitingTarget[AttackGround] resolution:
- Left-click Ground: issues AttackGround command to that location

### AwaitingTarget[Reverse] resolution:
- Left-click Ground: issues Reverse command to that location

## CommandIndicators
Visual markers displayed at command targets for selected units. Indicators are only visible when a unit or building with that active command is part of the current Selection. All active command indicators in the selection are shown simultaneously. Indicators are removed when the unit is deselected or the command completes.

### Location Indicator
A marker on the ground at the target coordinates.

### Object Indicator
A marker surrounding the target object's perimeter.

### Indicator assignments:

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
| Rally point (ground) | Location | Green |
| Rally point (own or neutral object) | Object | Green |
| Rally point (enemy object) | Object | Red |

**Rally points** (user, 2026-11-24): shown while the producing building is selected, with a **line** from the building to the rally point so distant or off-screen rally points are easy to follow. A cleared rally point (None) shows nothing.

### Color language:
- Green = peaceful movement
- Red = hostile target
- Orange = aggressive movement
