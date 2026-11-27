# Controls: HUD, Pointers and Command Indicators

See also: `systems/controls/selection.md`, `systems/controls/interface.md`, `systems/controls/hud.md` (split from `selection_and_hud.md`, 2026-11-27).

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

## Minimap
(user, 2026-11-26; the user asked for it to be prioritised) The leftmost HUD Bar section: a small view of the **whole map**.

### What it shows
- **Terrain** in simplified colours.
- **Objects** as marks in their **player colour**: buildings as larger squares (their footprint at minimap scale), units as small dots. **Space Crystal patches** and other neutral objects in a **neutral colour**.
- **Fog of war, as on the main map** (core/vision.md > VisibilityState): Unexplored is black; Explored is dimmed and shows only **remembered** objects (buildings, patches) in their last-known state; only **Visible** areas show live objects, units included. Hidden objects (e.g. underground, cloaked) don't appear for players who can't see them.
- **The camera's view** as a rectangle outline.

### Using it
- **Left-click or drag** on the minimap: the camera centres on that point (follows the drag).
- **Right-click** on the minimap: the selection's **right-click on that ground point**, as a right-click on ground in the main view (systems/controls/interface.md > RightClickResolution; usually a Move). Designer's placeholder: the minimap targets only ground, never an object.
- **While a target command waits** (AwaitingTarget, e.g. Attack, Move): a **left-click** on the minimap targets that ground point (e.g. Attack Move there), as a left-click on ground would (designer's placeholder). Building placement (AwaitingPlacement) is not possible from the minimap.

### Alerts
(user, 2026-11-26: minimal alerts)
- **When**: one of the player's units or buildings **takes damage while it is off screen** (outside the camera's view). Fights already on screen give no alert.
- **What**: a **red ping on the minimap** at that spot: a ring that shrinks and fades over about **2 s** (placeholder). No text and no sound.
- **How often**: at most **one ping per area every 10 s** (placeholder; an area: within about 8 grid units of an earlier ping still in its 10 s), so a long fight doesn't keep flashing.
- **Jump to the latest alert: Space** (user, 2026-11-26; default key): centres the camera on the **latest** alert's spot (snap, as Centering, core/camera.md). Later notifications of other kinds (none yet) will use the same key.

## SelectionPanel
Displays a grid of portraits for the current selection. Only visible when 2 or more objects are selected. A selection of 2+ objects always consists of the player's own objects (units and/or structures, e.g. from a control group), since other players' and neutral objects are always selected alone (systems/controls/selection.md > Selection > Constraints). "Unit" below means any selected object. Each cell shows the portrait of one selected unit instance. Portraits belonging to the ActiveGroup are shown with a sheer highlight.

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
| Assist | — | Unit will help construct this own building that is under construction (Cults Recruits; user, 2026-11-26; Q0095). Placeholder look. |
| Repair | — | Unit will repair this damaged own building or vehicle (Recruits, Agents, Field Engineers; user, 2026-11-26). Placeholder look. |
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
- While targeting **Set Drop-off** (AwaitingTarget[SetDropOffLocation]), cursor over a valid drop-off point: **ReturnResources** (user, 2026-11-25; Q0052)
- Cursor over own building under construction (selection can assist construction): **Assist**
- Cursor over damaged own building or vehicle that the selection can repair: **Repair**, unless the selection can enter it: then **Enter** (user, 2026-11-26)
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
- AwaitingTarget[AssistConstruction] over own building under construction: **Assist**; elsewhere: **Inactive**
- AwaitingTarget[Repair] over a damaged own building or vehicle the selection can repair: **Repair**; elsewhere: **Inactive**

#### AwaitingPlacement:
No pointer display type — the building ghost preview serves as the cursor. The ghost shows **entrance and exit arrows** at its doorways (core/entities.md > Structure Type > Entrance and exit, Arrows on the ghost).

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
| Gather (sets the source: crystal patch, refinery's output) | Object | Gather marker (distinct look, TBD) |
| Drop Off (return resources to an Egress Node or refinery) | Object | Return Resources marker (distinct look, TBD) |
| Set Drop-off | Object | Set Drop-off marker (distinct look, TBD) |
| Rally point (ground) | Location | Green |
| Rally point (own or neutral object) | Object | Green |
| Rally point (enemy object) | Object | Red |

**Rally points** (user, 2026-11-24): shown while the producing building is selected, with a **line** from the building to the rally point so distant or off-screen rally points are easy to follow. A cleared rally point (None) shows nothing.

**Gathering markers** (user, 2026-11-25; Q0052): Gather, Drop Off and Set Drop-off each have their **own distinct marker** on the target object, not the Move marker. Gather targets are crystal patches and refineries (Alloy Furnace, Essence Refinery, Flux Reactor) as sources; Drop Off and Set Drop-off targets are Egress Nodes and refineries as drop-offs. Like all indicators, they are shown only to the owner while the unit is selected.

### Color language:
- Green = peaceful movement
- Red = hostile target
- Orange = aggressive movement
