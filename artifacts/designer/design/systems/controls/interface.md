# Controls: Interface States and Command Panel

See also: `systems/controls/selection.md`, `systems/controls/interface.md`, `systems/controls/hud.md` (split from `selection_and_hud.md`, 2026-11-27).

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

## CommandPanel
The display of available commands for the current selection, derived from ControlState and game state each tick. The CommandPanel is only displayed when all objects in the Selection are owned by the player. When selecting enemy or neutral objects, only the InfoPanel is shown — no commands are available, no right-click resolution is performed, and no InterfaceTransitions can be initiated. Commands can only be issued to objects owned by the player.

When visible, commands available to all objects in the Selection are visually distinguished from commands available only to the ActiveGroup. Clicking a common command issues it to all selected objects. Clicking a group-specific command issues it only to objects in the ActiveGroup. **Common means the same command**, wherever its button sits (designer's clarification, 2026-11-27).

**Attack goes to everyone** (user, 2026-11-27): an Attack (A) is issued to **every selected unit**, even when some can't attack:
- Units that can attack carry it out: Attack on a target, or Attack Move on ground.
- Units that can't attack (e.g. a Prayer-mobile, Shadebringer, Disciple or Drone) treat an Attack or an Attack Move as a **Move** (user, 2026-11-27): on ground they move to the spot (with the group, as for Attack Move); on a target they move to it (as a Move on that object).
- So an attack-move with casters in the army brings the casters along.
- **Units that can't attack this target** (user, 2026-11-27; Q0186): a unit whose weapon can't hit the clicked target's domain (e.g. an anti-air Twister on a ground unit or structure, Peacekeepers on a Sparrow) is treated like a unit that can't attack: it **moves to the target (follows it) with the group**. If **no** selected unit can attack the target, the click is an **invalid target** (AwaitingTarget > Invalid target clicks): ignored, with feedback, and the Attack still awaits a target.

**Pressing an unavailable command** (user, 2026-11-24): a greyed-out command (e.g. Train with too few Space Crystals, or a full queue) can still be pressed or its hotkey used; nothing is issued, and the player gets the **same feedback as when an order is refused**, naming the reason (e.g. "Not enough Space Crystals", "Queue is full"). Same channel as invalid target clicks (AwaitingTarget > Invalid target clicks; exact form TBD with the notification system).

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

- **Z (bottom row, 1st slot / bottom-left)**: Back / Cancel menu. In any multi-stage menu (BuildMenu, ExpandMenu, EjectMenu, AwaitingTarget, AwaitingPlacement), Z returns to the previous state (StateOnlyTransition). Equivalent to Escape.
- **Right-click in multi-stage menus** (user, 2026-11-27): right-click **doesn't reset a menu**. In a menu where the next step is choosing an option (BuildMenu, ExpandMenu, EjectMenu, KindMenu, AugmentMenu and the like), a right-click does its **normal right-click** for the selection (e.g. moves the selected workers, or sets a building's rally point), and the menu **stays open**. Only when the next step is a **left-click on a location or target** (AwaitingTarget, AwaitingPlacement) does a right-click cancel, and it goes **straight back to DefaultState** (not to the previous menu). Escape and Z still go back one step.
- **X (bottom row, 2nd slot)**: Cancel Production / Cancel Upgrade. In production buildings, cancels the last queued item and refunds cost. In structures with upgrades, cancels the in-progress upgrade and refunds cost.
- **C (bottom row, 3rd slot)**: Set Rally Point. In unit-producing structures, enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState). Clicking the **producing building itself** clears the rally point (RallyPoint = None; user, 2026-11-24).
- **V (bottom row, 4th slot / bottom-right)**: no standard function; free for object-specific commands.
- **A: Attack, on every unit** (user, 2026-11-27): Attack is always in slot **A**, whichever group is in focus, so the key never changes. No unit puts another command on A; workers build from **W** (Recruit, Agent, Field Engineer, Drone; 2026-11-27). **A is shown whenever at least one selected group can attack** (user, 2026-11-27), whichever group is in focus, and issues the Attack to the whole selection. Only when **no** selected unit can attack is A not shown.

### Production Building Default Right-Click
For all unit-producing structures, right-click from DefaultState sets the rally point:
- Right-click Ground: sets rally point to that location
- Right-click Object: sets rally point to that object
- Right-click the building itself: **clears** the rally point (RallyPoint = None; user, 2026-11-24)
- **Several producers selected** (user, 2026-11-24): each applies these rules for itself. Right-clicking one of them clears that building's rally point, and the others rally to the clicked building (Move to object).

### Shift and rally points
(user, 2026-11-24) A producer has a single rally point, so **Shift does nothing special** for rallies: a Shift-click (after C) or Shift+right-click sets the rally point and ends targeting, exactly like a plain click (an exception to AwaitingTarget > Shift while targeting). Shift is left free for possible multi-waypoint rallies later (not designed).

### What a rally point makes a new unit do
(user, 2026-11-24) A unit that spawns with a rally point set **always** gets its **own default right-click command** resolved against the rally target (RightClickResolution, including the rule for units that can't carry out the resolved command). No per-building lists override this. Examples: a Drone rallied on a crystal patch **Gathers** it; a Drone rallied on an enemy unit **follows** it (unarmed, so it can't attack); a Peacekeeper rallied on an enemy attacks it.

### Rally points on enemy objects
(user, 2026-11-24) A rally point on an enemy object only follows that object **while the owner can see it**. As soon as the enemy object leaves the owner's sight, the rally point **turns into a ground rally point at its last-seen position**. So:
- A unit produced while the enemy is visible attacks it; one produced after the enemy was lost from sight moves to the last-seen position. (This is a rally rule only: a unit already attacking a target that goes into the fog simply stops, systems/controls/command_behaviors.md > AttackingObject > Algorithm step 6.)
- An enemy dying unseen in the fog doesn't change the rally point, so the rally marker never reveals hidden information. (The "reset to None when the object no longer exists" rule applies to own and neutral objects, and to enemies only while visible.)
- **Remembered enemy structures** (user, 2026-11-24): a rally point can be set on a remembered (fogged) enemy structure, since remembered objects can be targeted (core/vision.md > Explored > Targeting remembered objects). Structures don't move, so a rally on an enemy structure stays on it when it leaves sight (as a remembered target) instead of becoming a ground rally. New units get their default command against it (e.g. attack it once they see it; if it turns out to be gone, they stop there).

## DefaultState
The default ObjectInterfaceState for a selection. The command panel displays available commands. Right-click performs a context-sensitive command based on what is under the cursor. Command hotkeys or button clicks initiate transitions to AwaitingTarget or issue immediate commands.

## RightClickResolution
Context-sensitive command issued by right-clicking from DefaultState. The command is determined by what is under the cursor and what the selected objects can do. **Each selected unit resolves the right-click for itself**, against its own capabilities, not the ActiveGroup's (user, 2026-11-23). The result doesn't depend on which group is active.

**Units that can't carry out the resolved command** (user, 2026-11-23): every selected unit acts on a right-click. Units that have the resolved command carry it out. Units that can't, e.g. an unarmed Supply Chopper, or ground-only Peacekeepers when the target is an aircraft, **move to or follow the target instead**: Move to the location, or Follow a unit (systems/controls/command_behaviors.md > MovingToObject). The group stays together.

## AwaitingTarget[CommandType]
An ObjectInterfaceState entered after the player selects a command that requires a target. The command panel indicates which command is pending. Left-clicking a valid target issues the command via a CommandIssuingTransition and returns to DefaultState (except while Shift is held; see Shift while targeting). The target may resolve to a different command than the one that initiated the state, depending on what is under the cursor. Escape or right-click cancels via a StateOnlyTransition back to DefaultState (right-click always to DefaultState: Standard Slot Assignments > Right-click in multi-stage menus).

**Other commands while targeting** (user, 2026-11-23): while a target is awaited, the command panel shows the pending command and Back/Cancel (Z). The other commands are **unavailable**: pressing their hotkeys or buttons issues nothing, and the player gets the unavailable-command feedback **"Cancel the current command first"** (CommandPanel > Pressing an unavailable command; user, 2026-11-24); targeting or placement continues. To change commands, the player cancels first (Escape, right-click or Z), then picks the new command.

**Dragging while targeting** (user, 2026-11-23): a left-button drag while a target is awaited does **nothing**. While targeting, a press and release only counts as a **drag** if the mouse moved more than a **larger click allowance** between press and release: **16 pixels** (placeholder, half a tile; user, 2026-11-24), instead of the normal drag threshold. Within the allowance it is a targeting click, so a slightly sloppy click on a moving enemy isn't lost; a deliberate drag away before release is a way to abort a press on the wrong target. On a drag: no selection box is drawn, nothing is selected, and targeting stays active. A targeting click counts at the **press point** (where the button went down), even if the mouse moves before release (user, 2026-11-23). The command matches what was under the cursor when the player committed (e.g. pressing on an enemy and releasing on the ground still issues Attack on that enemy).

**Selection changes while targeting** (user, 2026-11-23):
- **Player-initiated** selection or ActiveGroup changes (control group recall, Tab / Shift-Tab, portrait clicks) **cancel** targeting, per ObjectInterfaceState's reset rule. A control group recall cancels it **even if it recalls exactly the current selection** (user, 2026-11-24).
- **Automatic removals** (a selected unit dies, or leaves the selection by entering a building, transport or tunnel) do **not** cancel targeting. Targeting is only cancelled if the **whole ActiveGroup** is gone, since nothing would be left to carry out the command.

**Shift while targeting** (user, 2026-11-23): a Shift-click on a valid target **queues** the command (systems/controls/commands.md > Unit Command) and targeting **stays active while Shift is held**, so the player can place several targets or waypoints in a row with one command keypress (e.g. A, then Shift-click, Shift-click, Shift-click). Each Shift-click queues one more command. Targeting ends (back to DefaultState) when Shift is **released** after at least one queued click. A click without Shift issues the command and ends targeting as usual. TBD: releasing Shift before any target was clicked (presumably targeting simply continues, since nothing was issued).

**Shift while placing** (AwaitingPlacement; user, 2026-11-23): the same rule applies. A Shift-click on a valid spot queues the build order and placement **stays active while Shift is held**, so one builder can be given several buildings in a row; releasing Shift ends placement. This only applies when the builder **can carry out more queued buildings**: e.g. Cults Recruits (the whole selected group) build one and walk together to the next (each order's cost is paid when it is given: factions/cults/economy.md > Build Command). When the builder **can't**, a Shift-click sends it and ends placement like a plain click:
- an Agent goes inside the structure it builds (factions/syndicate/units/agent.md > Building > Tunnel Construction Flow), so each Agent takes exactly one placement;
- a GDO Deployment Center or Extraction Facility only ever has one building ready to place.

Placement **never moves on to another selected worker** automatically (systems/controls/selection.md > SelectionGroup > No smart-casting): to place with the next Agent, the player presses Tab and issues the next order.

**Exception, Drones** (user, 2026-11-24; replaces 0071 for Drones): a Drone becomes the Beacon, so one Drone can't take a second Deploy. With **several Drones in the group**, Shift-placement **stays active and each Shift-click goes to the next Drone** in selection order that has no Deploy yet (systems/controls/selection.md > SelectionGroup > Groupable workers used up by a build). Example: 4 Drones selected, hold Shift, click 4 sites → 4 lingering ghosts, one Drone each. Once every Drone in the group has a Deploy, placement ends. With a single Drone, a Shift-click sends it and ends placement as before.

**Invalid target clicks** (user, 2026-11-23): a left-click on a target the pending command can't use (shown by the Inactive pointer) is **ignored**. Targeting stays active and waits for a valid target, and the player gets short **feedback** (an "invalid target" message and/or sound; exact form TBD, ties into the future notification system).

### PendingCommand - CommandType
### Resolution - (CursorTarget) -> Command

## CursorTarget
What is under the player's cursor at the time of a click. Used by AwaitingTarget and RightClickResolution to determine which command to issue.

### Value - CursorTargetEnum (Ground | EnemyObject | FriendlyObject | NeutralObject)
### Location - Coordinates
### Object - ObjectInstance | None

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
- Cursor over own Altar (Cults infantry only — Militant, Gunner, Disciple): issues Enter command targeting that Altar
- Cursor over own Garrison (Cults infantry only — Militant, Gunner, Disciple): issues Enter command targeting that Garrison
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
