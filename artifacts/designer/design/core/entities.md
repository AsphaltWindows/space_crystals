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
### Groupable - boolean (if false, each instance is always in its own SelectionGroup even when selected with other instances of the same type). Caster-type units and workers that are used up by a build or build alone (Agent, Drone, Field Engineer: Q0163) are Ungroupable; Recruits, which share a build, are Groupable; see systems/controls/selection.md > SelectionGroup > No smart-casting

## Structure Type
An Object Type of a building, an instance of which may appear on the game's map. During placement, the player can rotate the building in 90-degree increments and flip it across the horizontal or vertical axis. This gives up to 8 possible orientations for fully asymmetric buildings (ABCD), fewer for more symmetric types.

It has one of 7 symmetry types from the perspective of how relevant the building's orientation is. AAAA is the building is fully symmetrical on all 4 sides (requires for Height and Width to be equal) making the orientation irrelevant. Height and Width can be non-equal for symmetry types ABAB, ABAC and ABCD.

### SymmetryType - AAAA | AAAB | AABB | ABAB | AABC | ABAC | ABCD

### Building armor rule
(user, 2026-11-25) A building's armor should generally be **PointArmor n, FullArmor n x (its size in grid squares)**, for some n chosen per building (e.g. a 2x2 building with n = 1: 1 / 4; a 4x4 Tunnel: 1 / 16; underground expansions use n = 6, so a 2x2 one is 6 / 24). There may be exceptions. Since AoE effective armor = FullArmor x the fraction of the building covered (systems/combat.md > AoEDamage), this gives every covered grid square the same n of armor against a blast.

**Side lettering convention** (user, 2026-11-24): the letters name the building's four sides **clockwise starting from the top edge**, in the building's **default orientation** (rotation 0, unflipped): 1st letter = top, 2nd = right, 3rd = bottom, 4th = left. Size (Height x Width) is also given for the default orientation. Example: the GDO Barracks is 2x3 (2 tall, 3 wide) with ABAC: top A, right B (short side), bottom A, left C (short side). Rotating or flipping the building carries the sides with it.

### Entrance and exit (production and research buildings)
(user, 2026-11-26; Q0116) **Every production and research building has an entrance side and an exit side.** **For training, augmentation and unit-producing buildings the exit is the side opposite the entrance** (user, 2026-11-26; Q0104), except the **Airstrip**, which places a finished aircraft directly on its runway, from which it takes off. **The entrance is a dedicated doorway** at the **middle of the entrance side** (like the Exit Doorway): units walk to it and go in there (Q0104). Units go in by the entrance and come out of the exit **one at a time** (Exit Doorway below, a little like leaving a Tunnel), whether trained, ejected or leaving research. The **Garrison** is the exception: it has **one door** (entrance and exit), and on Eject All everyone inside comes out **at once** through it, ignoring collision with each other until they are out (factions/cults/structures/garrison.md).

**Arrows on the ghost** (user, 2026-11-27): while placing a building (AwaitingPlacement), its ghost shows an **arrow at each doorway**: an arrow **pointing in** at an entrance, an arrow **pointing out** at an exit, and a **two-way arrow** at a single door used both ways (e.g. the Cults Garrison, the GDO Bunker). The arrows sit at the doorway's position on its side and **turn and flip with the ghost**, so the player can see where units will go in and come out before placing. **The same arrows show on a placed building while it is selected** (user, 2026-11-27). This applies to every faction's buildings with entrances or exits (including those that release units through an exit side, e.g. the GDO Barracks).

### Exit Doorway
Applies to structures that release units through an **exit side** (e.g. GDO Barracks side B, Cults Armory exit side C). Decided by the user, 2026-11-24:
1. **Spawning in the doorway**: a produced or released unit appears **half inside the building**, at the middle of the exit side. Only the releasing building lets a unit overlap its footprint this way; nothing else can overlap a building. Since the spot is partly inside the footprint, no other unit can stand on it, so **the spawn spot can't be blocked from outside**.
2. **Walking out**: the unit then walks out toward its RallyPoint (or straight out of the exit side if there is none). If the way is blocked, it **stops in the doorway**. It is a normal unit there: selectable, attackable, and able to attack (so a lone enemy blocker gets shot by the unit in the doorway).
3. **Pausing**: the next unit can only appear once the doorway is free. While a unit still stands in it, the building **pauses**: the finished unit waits inside and the rest of the queue waits behind it. Production resumes as soon as the doorway is free. The waiting unit is **fully inside** (its production shows 100%) and doesn't exist on the map yet.
4. Own idle units outside step aside for the unit walking out (systems/units.md > Ground Collision, friendly yielding), so they don't cause stalls.
5. **Exits may be blocked by buildings and terrain** (allowed for now, user): placement does not keep exits clear. A building or cliff right against the exit side leaves the first unit stuck in the doorway and the building effectively stalled; that is the player's own mistake.

6. **Ground the unit can't stand on** (user, 2026-11-27; Q0134): a leaving unit may overlap its building, but is never released onto ground under it that it couldn't stand on (e.g. a custom tile that is Buildable but not Traversible). Such a building **holds its finished unit** indefinitely, as an occupied doorway. Placement doesn't prevent it; map makers are advised not to make such tiles (core/map.md > Tile).

Buildings with **several exit-eligible sides** choose the side by their own rules; a side whose doorway is occupied counts as blocked, and the building pauses only when every eligible doorway is occupied. **Colonists portals** don't use exit sides: units emerge from an energy doorway at the centre (factions/colonists/structures/README.md > Portals > Portal Exits).

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
- **No ground units on the footprint** (user, 2026-11-24): a ground unit of any player (own, allied or enemy) standing on any footprint tile makes those tiles invalid (red on the ghost), and the building can't be placed until they move. Own units are not pushed aside (considered: idle own units stepping aside; judged unnecessary). Air and underground units never block placement. **Exception, underground expansions** (user, 2026-11-26; Q0080): ground units **don't** block a Syndicate expansion's placement, since it is underground and units walk over expansions anyway.
- Standard spatial checks also apply (tiles must be Buildable, no existing structure overlap, faction-specific constraints like GDO Build Area). Surface structures also need every footprint tile to be **flat** (core/map.md > Elevation > Structures only on flat terrain)

### All placement methods: no buildings on resources
(user, 2026-11-24) No structure may cover any tile of a Space Crystals Patch or a Supply Delivery Station (core/map.md > Resource), whatever the placement method. The only exception is a GDO Extraction Plate, which is placed onto a patch.

### Worker-Built Structures (e.g., Agent building a Tunnel, Drone deploying a Beacon)
- The player queues a build command targeting a map location
- The command is accepted regardless of current visibility — no visibility check at command time
- **The ghost judges by what the player knows** (user, 2026-11-24): it is green if, as far as the player knows, nothing blocks the footprint. On Explored tiles it uses remembered terrain and remembered structures; units hidden in fog are ignored. Never-explored tiles pass. The order can be given even where the area isn't visible; the real check happens on arrival.
- The worker pathfinds to the location and validates on arrival: tiles must be Buildable, flat (core/map.md > Elevation), unoccupied, and meet faction-specific constraints
- **Units block** (user, 2026-11-24): "unoccupied" includes units, as in Direct Placement: a ground unit of any player on the footprint makes the site invalid; the building worker itself (e.g. the deploying Drone) doesn't count. Air and underground units never block.
- If validation fails on arrival, the command is cancelled and the worker stops and idles

### Lingering build ghosts
(user, 2026-11-24) Every **ordered worker build** (Drone Deploy, Agent build, Cults Recruits' Construct), including **every Shift-queued one**, leaves a **ghost of the building on its site** until construction actually starts.
- **Visible to its owner.** Enemies never see it. Whether allies see it: TBD.
- **Reserves the site against the owner's further placements**: a new ghost overlapping it is invalid (red). This is how a Drone group's Deploy reaches only one Drone (systems/controls/selection.md > SelectionGroup > Groupable workers used up by a build).
- **Doesn't block units**: anyone can walk over it; units standing on the site still make the build fail on arrival (Worker-Built Structures above).
- **Disappears** when construction starts (the real structure replaces it) or when the order is cancelled, replaced by a new order, or fails.
- Reservation against allies' placements: TBD (with ally visibility).
- No visibility requirement on arrival (the worker is physically present)

### Warp In (Colonists Beacon transformation, see factions/colonists/economy.md > Warp In)
- The player selects an established Beacon, selects Warp In, and selects a target building
- The building's footprint ghost is centered on the Beacon's location; the player may rotate/flip it before confirming. Odd-sized footprints can't be exactly centred: they start offset to the bottom-right and rotate/flip around the Beacon's centre (factions/colonists/economy.md > Warp In > Warp In Command)
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
