# Controls: Selection

See also: `systems/controls/selection.md`, `systems/controls/interface.md`, `systems/controls/hud.md` (split from `selection_and_hud.md`, 2026-11-27).

## ControlState
The client-side state of a player's control interface. Contains the player's current Selection and the ObjectInterfaceState for the active selection. ControlState is not part of the game simulation — it lives entirely on the client. Each tick, ControlState is validated against the current game state: selected objects that no longer exist are removed from the Selection, and if the ObjectInterfaceState becomes invalid as a result, it resets to the default state for the current selection.

### Selection - Selection
### ObjectInterfaceState[ActiveGroup] - ObjectInterfaceState[ObjectEnum]

## Selection
The set of ObjectInstances currently selected by the player, grouped by object type. Each group contains references to all selected instances of that type. The ActiveGroup determines which group's commands are displayed in the command panel and which ObjectInterfaceState is active.

### Groups - array of SelectionGroup
### ActiveGroup - SelectionGroup | None (None when selection is empty). Usually identified by its object type; for Ungroupable objects each instance is its own SelectionGroup, so ActiveGroup points at one specific group, and GroupCycling (Tab) steps through them one by one.

**When the ActiveGroup disappears** (user, 2026-11-25; Q0050):
- **Its members changed type and joined another group** (an unfinished building completes and joins its finished group, 0185; units pressed with a mode toggle, factions/colonists/units/weapon_conversion.md > A toggle sets a target form): **the group they joined becomes active**. The command panel resets to that group's default state, which ends any targeting in progress.
- **Its last member died** (or otherwise left the selection, e.g. by entering a building, transport or tunnel): **the next group in Tab order becomes active** (the group after it in the SelectionPanel, wrapping around to the first). If no group is left, the selection is empty.
- Any other group change, e.g. a completion in a group that isn't active, **never changes the ActiveGroup**.

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
A command issued from the command panel (hotkey or button) goes to **every instance it is issued to** (the whole Selection for a common command, the whole ActiveGroup for a group command; see systems/controls/interface.md > CommandPanel). The game **never hands a group's command to just one member** on the player's behalf ("smart-casting"), for any command (user, 2026-11-23).

Rationale (user): smart-casting makes mass use of abilities effortless, which constrains how powerful unit abilities can realistically be. Workers follow the same rule.

**Individual use is done through Groupable** (user, 2026-11-23). Groupable is fixed per object type, chosen by the design:
- **Groupable = false** for units whose commands are used **one unit at a time**: **workers that are used up by a build** (Syndicate Agent: each build order needs one specific worker; the Colonists Drone is the exception, see Groupable workers used up by a build below), and likewise **structures used up by a build** (Colonists Beacon, consumed by its Warp In; user, 2026-11-24) and **caster-type units** (one cast should spend one unit's energy, not the whole group's). Each instance is its own SelectionGroup, so the player casts with one unit, presses **Tab** to the next, and casts again, with no mouse reselection. This is the intended middle ground: easier than picking units out with the mouse, harder than smart-casting.
- **Groupable = true** for units whose abilities are normally used **all together** (e.g. the Peacekeeper's Riot Shield), including workers that **share** a build: Cults Recruits all build the same building together, so Construct goes to every selected Recruit (user, 2026-11-24).

To use a Groupable unit's ability on only one instance, the player selects that instance alone (e.g. left-click its portrait in the SelectionPanel).

**Unfinished buildings form their own group** (user, 2026-11-24): a building still under construction or transformation (e.g. a portal being warped in, a Cults building under construction) forms a **separate SelectionGroup** from finished buildings of the same type, as if it were another type, with its own panel (e.g. Cancel only: factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel). **Rally points on unfinished buildings** (user, 2026-11-27): a building that will produce or release units can have its **RallyPoint set while still under construction or transformation**: its panel adds **C: Set Rally Point**, and right-click ground or an object sets it, as on the finished building. The rally point is kept when it completes. When it completes, it joins the finished group. Example: 2 finished Bio Portals + 1 warping in → two groups; Tab between them.

**Groupable workers used up by a build** (user, 2026-11-24): the **Colonists Drone is Groupable** (so bulk commands such as Set Drop-off reach every selected Drone), even though each Deploy uses up one Drone. A Deploy order given to a group of Drones goes to **one Drone only: the first in selection order** (the SelectionGroup's portrait order in the SelectionPanel) that can still take it:
- a plain click goes to the first Drone in the group (replacing its orders, like any plain order);
- a Shift-click goes to the first Drone that has **no Deploy in its queue yet**, since a Drone that will already have become a Beacon can't deploy again (user: "the game should understand that after the first drone it won't exist"). See AwaitingTarget > Shift while placing.

The other Drones are untouched. This is a deliberate **exception to No smart-casting**: the rule exists to stop effortless mass use of abilities (power), which a worker's deploy doesn't have, and one Beacon can only ever take one Drone. Every ordered build is shown as a lingering ghost (core/entities.md > Placement Validation > Lingering build ghosts).

**Group production with limited resources** (user, 2026-11-24): when a production command (e.g. Q: Peacekeeper) goes to several selected production buildings and the player can't pay for all of them (crystals or any other resource paid at queue time; unit capacity is not checked when queuing), **as many as possible** queue it. The buildings are served in order of **least work queued first**: the least total remaining production time in their queue (including the unit in production), ties broken in a fixed, deterministic order. The rest don't queue (insufficient-resources feedback). This spreads production across idle buildings first, without picking a single building for the player.

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

Only tier 1 (own units) produces a multi-selection. All other tiers select exactly one object — the one closest to the center of the box. This holds for **groupable buildings** too (user, 2026-11-26; Q0151): a box over several Recruitment Halls selects one; select several with Shift-click, Ctrl-click or double-click (MapClickSelection), or Shift + box.

### Shift + box (add)
Holding **Shift** while dragging a box **adds** the box's result (after the priority tiers) to the current selection. The current selection **always stays selected**: Shift + box never removes anything, unlike Shift-click, which toggles (user, 2026-11-23). Objects that can't be added under Selection > Constraints (e.g. an enemy, neutral object or other player's object when own objects are selected) are not added, and the selection stays unchanged.

**Adding to a non-owned selection** (user, 2026-11-23): when the current selection is a single enemy, neutral or other player's object, any add input (Shift + box, Shift-click, Ctrl+Shift-click on own objects) does **nothing**: the inspected object stays selected. Use a plain click or box to select own objects instead.

### Empty clicks and boxes (user, 2026-11-23)
- A plain click on empty ground, or a plain box that captures nothing, **clears** the selection.
- Shift-click or Ctrl-click on empty ground does **nothing** (selection kept).
- Ctrl+Shift-click on empty ground also does nothing. (Ctrl+Shift-click on an own object: see MapClickSelection.)

## MapClickSelection
Selection by clicking objects on the map (as opposed to portraits in the SelectionPanel). Default inputs (user, 2026-11-23):

| Input | Effect |
|-------|--------|
| Left-click object | Replace selection with that object |
| Shift-click own object | Toggle that object in the selection: add it if not selected, remove it if selected |
| Ctrl-click own object | Select all own objects of that type currently on screen |
| Double-click own object | Same as Ctrl-click |
| Ctrl+Shift-click own object | **Add** all own objects of that type currently on screen to the current selection (never removes; user, 2026-11-23) |

**Own buildings too** (user, 2026-11-26; Q0151): the modifier clicks above work on **own buildings** as on own units (they were units only). Shift-click adds or removes any own building; Ctrl-click, double-click and Ctrl+Shift-click take every own building of that type on screen. Ungroupable buildings selected this way each form their own SelectionGroup (Constraints above). "Of that type" follows the SelectionGroup rule: an **unfinished** building counts as another type from finished ones (SelectionGroup > No smart-casting, "Unfinished buildings form their own group"), so Ctrl-click on a finished Hall takes the finished Halls on screen.

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

## GroupCycling
A StateOnlyTransition that changes the ActiveGroup to an adjacent SelectionGroup in the Selection.

### Forward (Tab): advance ActiveGroup to the next SelectionGroup. Wraps around to the first group after the last.
### Backward (Shift-Tab): advance ActiveGroup to the previous SelectionGroup. Wraps around to the last group before the first.
