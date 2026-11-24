# Entities

## Entity
A type or class of objects in the game

### Visible - boolean

## Invisible Entity
An Entity which has no direct visual representation on the map. An Abstract type class or category.

### Visible - false

## Faction
The Invisible Entity of in-game faction type has a faction-specific hud with displaying relevant information on the screen

### Value - FactionEnum
### Name - string
### DisplayHud[Value] - Display

## Player
The Invisible Entity of an in-game player. Each player has a number from 1 to n where n is the number of players in the game. Player has info and statistics relevant to their Faction, which are displayed in the Faction-specific DisplayHud.

### Name - string
### Faction - FactionEnum
### PlayerNumber - number
### DisplayHudInfo[Faction] - struct

## Visible Entity
An Entity which has an immediate visible representation on the screen.

### Visible - true

### Selectable - boolean

## Object Type
Selectable visible entity, instances of which exist in the game at some coordinates on the map. When selected by a player the InfoPanel is displayed for the object on that player's screen, depending on the player selecting and the player owning (if applicable) the object instance.

### Selectable - true

### Value - ObjectEnum
### Name - string
### Size - Height x Width
### InfoPanel - (owner: (PlayerNumber | None), seelctor: Player) -> Display

**Default InfoPanel for structures** (user, 2026-11-24; unless a structure's own section specifies more):
- **Own structure**: name, HP, and what it is doing: construction or production with progress, queue length, a building ready to place, and the rally point.
- **Another player's structure**: name, owner and HP only (no production or queue information).
### Destructible - boolean
### SightRange - number (vision is provided to the owning player when the instance has an Owner)
### Groupable - boolean (if false, each instance is always in its own SelectionGroup even when selected with other instances of the same type). Caster-type units and workers that are used up by a build (Agent, Drone) are Ungroupable; Recruits, which share a build, are Groupable; see systems/controls/selection_and_hud.md > SelectionGroup > No smart-casting

## Structure Type
An Object Type of a building, an instance of which may appear on the game's map. During placement, the player can rotate the building in 90-degree increments and flip it across the horizontal or vertical axis. This gives up to 8 possible orientations for fully asymmetric buildings (ABCD), fewer for more symmetric types.

It has one of 7 symmetry types from the perspective of how relevant the building's orientation is. AAAA is the building is fully symmetrical on all 4 sides (requires for Height and Width to be equal) making the orientation irrelevant. Height and Width can be non-equal for symmetry types ABAB, ABAC and ABCD.

### SymmetryType - AAAA | AAAB | AABB | ABAB | AABC | ABAC | ABCD

**Side lettering convention** (user, 2026-11-24): the letters name the building's four sides **clockwise starting from the top edge**, in the building's **default orientation** (rotation 0, unflipped): 1st letter = top, 2nd = right, 3rd = bottom, 4th = left. Size (Height x Width) is also given for the default orientation. Example: the GDO Barracks is 2x3 (2 tall, 3 wide) with ABAC: top A, right B (short side), bottom A, left C (short side). Rotating or flipping the building carries the sides with it.

### Exit Doorway
Applies to structures that release units through an **exit side** (e.g. GDO Barracks side B, Cults Armory exit side C). Decided by the user, 2026-11-24:
1. **Spawning in the doorway**: a produced or released unit appears **half inside the building**, at the middle of the exit side. Only the releasing building lets a unit overlap its footprint this way; nothing else can overlap a building. Since the spot is partly inside the footprint, no other unit can stand on it, so **the spawn spot can't be blocked from outside**.
2. **Walking out**: the unit then walks out toward its RallyPoint (or straight out of the exit side if there is none). If the way is blocked, it **stops in the doorway**. It is a normal unit there: selectable, attackable, and able to attack (so a lone enemy blocker gets shot by the unit in the doorway).
3. **Pausing**: the next unit can only appear once the doorway is free. While a unit still stands in it, the building **pauses**: the finished unit waits inside and the rest of the queue waits behind it. Production resumes as soon as the doorway is free. The waiting unit is **fully inside** (its production shows 100%) and doesn't exist on the map yet.
4. Own idle units outside step aside for the unit walking out (systems/units.md > Ground Collision, friendly yielding), so they don't cause stalls.
5. **Exits may be blocked by buildings and terrain** (allowed for now, user): placement does not keep exits clear. A building or cliff right against the exit side leaves the first unit stuck in the doorway and the building effectively stalled; that is the player's own mistake.

Buildings with **several exit-eligible sides** (e.g. Colonists Bio Portal, Robo Portal) choose the side by their own rules; a side whose doorway is occupied counts as blocked, and the building pauses only when every eligible doorway is occupied.

Aircraft and units released underground (e.g. Syndicate Tunnels) are not covered by this rule.

### ConstructionHP Rule
Some structures are built on-site and start at 10% of their maximum HP, gaining HP linearly as construction progresses, reaching full HP when construction completes. HP during construction = MaxHP x (10% + 90% x construction_progress). A partially-built structure can be attacked and destroyed before completion. This rule applies only to structures that explicitly reference it.

## Placement Validation

Structure placement is validated differently depending on the placement method:

### Direct Placement (e.g., GDO buildings, Tunnel underground expansions)
- The building is placed immediately from an interface menu
- All tiles under the footprint must pass validation at the moment the player confirms placement
- For surface buildings: all footprint tiles must be in the **Visible** state for the placing player. This includes GDO Extraction Plates (no exemption; user, 2026-11-24). If any tile is not Visible, the placement ghost is shown in red and placement cannot be confirmed
- For underground expansions: validated against the relevant underground spatial rules (e.g., Tunnel Area bounds, no overlap). Surface visibility is not relevant. Surface flatness is not required either (core/map.md > Elevation > Structures only on flat terrain)
- **No ground units on the footprint** (user, 2026-11-24): a ground unit of any player (own, allied or enemy) standing on any footprint tile makes those tiles invalid (red on the ghost), and the building can't be placed until they move. Own units are not pushed aside (considered: idle own units stepping aside; judged unnecessary). Air and underground units never block placement.
- Standard spatial checks also apply (tiles must be Buildable, no existing structure overlap, faction-specific constraints like GDO Build Area). Surface structures also need every footprint tile to be **flat** (core/map.md > Elevation > Structures only on flat terrain)

### All placement methods: no buildings on resources
(user, 2026-11-24) No structure may cover any tile of a Space Crystals Patch or a Supply Delivery Station (core/map.md > Resource), whatever the placement method. The only exception is a GDO Extraction Plate, which is placed onto a patch.

### Worker-Built Structures (e.g., Agent building a Tunnel, Drone deploying a Beacon)
- The player queues a build command targeting a map location
- The command is accepted regardless of current visibility — no visibility check at command time
- The worker pathfinds to the location and validates on arrival: tiles must be Buildable, flat (core/map.md > Elevation), unoccupied, and meet faction-specific constraints
- **Units block** (user, 2026-11-24): "unoccupied" includes units, as in Direct Placement: a ground unit of any player on the footprint makes the site invalid; the building worker itself (e.g. the deploying Drone) doesn't count. Air and underground units never block.
- If validation fails on arrival, the command is cancelled and the worker stops and idles
- No visibility requirement on arrival (the worker is physically present)

### Warp In (Colonists Beacon transformation, see factions/colonists/economy.md > Warp In)
- The player selects an established Beacon, selects Warp In, and selects a target building
- The building's footprint ghost is centered on the Beacon's location; the player may rotate/flip it before confirming
- All footprint tiles must be Buildable, flat (core/map.md > Elevation) and unoccupied, except for the Beacon's own footprint, which will be consumed by the transformation
- **Units block** (user, 2026-11-24): "unoccupied" includes units, as in Direct Placement: a ground unit of any player on the footprint makes those tiles invalid. Air and underground units never block.
- **Visible tiles** (user, 2026-11-24): as in Direct Placement, every footprint tile must be **Visible** to the player; a tile that isn't is red on the ghost. This rarely matters (the Beacon's sight covers its surroundings) but keeps the ghost from revealing hidden units in fog.
- If any tile outside the Beacon's own footprint is blocked, the ghost is shown in red and placement cannot be confirmed
- Once confirmed, the Beacon begins transforming in place into the selected building (transformation mechanics, HP/Shields interpolation, and cancellation defined in factions/colonists/economy.md > Warp In)

## Object Instance
An instance of an Object Type existing on the map.

### Type - ObjectEnum
### Location - Coordinates
### Owner - PlayerNumber | None
### HP - number (Destructible types only)

## Structure Instance
An Object Instance of a Structure Type.

### Rotation - 0 | 90 | 180 | 270
### FlipHorizontal - boolean
### FlipVertical - boolean
### StructureState - struct (to be defined with structure capabilities)
