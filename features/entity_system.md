# Feature: Entity System

## Overview
The core entity hierarchy defining all game object types, their properties, and their instances on the map.

## Design Sources
- `design/entities.md`

## Specifications

### Entity Hierarchy
- **Entity**: Base type. Has a `Visible` boolean.
  - **Invisible Entity** (Visible=false): No visual representation. Abstract types.
    - **Faction**: Has FactionEnum, Name, DisplayHud.
    - **Player**: Has Name, Faction, PlayerNumber, DisplayHudInfo.
  - **Visible Entity** (Visible=true): Visual representation on screen. Has `Selectable` boolean.
    - **Tile** (Selectable=false): Map terrain. Properties: Buildable, Traversible, Rugged, Drillable, Recruitable.
    - **Object Type** (Selectable=true): Instances exist at coordinates on the map.

### Object Type Properties
- Value (ObjectEnum), Name, Size (HxW)
- InfoPanel: function of (owner, selector) -> Display
- Destructible: boolean
- SightRange: number (provides vision to owning player)
- Groupable: boolean (if false, each instance is always in its own SelectionGroup)

### Structure Type (extends Object Type)
- SymmetryType: AAAA | AAAB | AABB | ABAB | AABC | ABAC | ABCD
- During placement, player can rotate in 90-degree increments and flip across horizontal or vertical axis
- Up to 8 possible orientations for fully asymmetric buildings (ABCD), fewer for more symmetric types
- Height and Width can be non-equal for ABAB, ABAC, ABCD symmetry types

#### ConstructionHP Rule (opt-in)
Some structures are built on-site and use a progressive HP rule during construction. This rule applies only to structures that explicitly reference it.
- HP during construction = `MaxHP x (10% + 90% x construction_progress)`
- Structure starts at 10% of MaxHP when construction begins
- HP increases linearly as construction progresses, reaching full MaxHP at completion
- Partially-built structures can be attacked and destroyed before completion
- Currently referenced by: Syndicate Tunnel (Agent construction flow)

### Object Instance
- Type (ObjectEnum), Location (Coordinates), Owner (PlayerNumber | None)
- HP (Destructible types only)

### Structure Instance (extends Object Instance)
- Rotation: 0 | 90 | 180 | 270
- FlipHorizontal: boolean
- FlipVertical: boolean
- StructureState: struct (to be defined per structure)

### Placement Validation
Structure placement uses different validation depending on the placement method:

**Direct Placement** (GDO buildings, Tunnel underground expansions):
- Building placed immediately from an interface menu
- All footprint tiles must pass validation at confirmation
- Surface buildings: all footprint tiles must be in **Visible** state for the placing player. Red ghost shown if any tile is not Visible; placement cannot be confirmed
- Underground expansions: validated against underground spatial rules (Tunnel Area bounds, no overlap). Surface visibility not relevant
- Standard spatial checks also apply: tiles must be Buildable, no existing structure overlap, faction-specific constraints (e.g., GDO Build Area)

**Worker-Built Structures** (e.g., Agent building a Tunnel):
- Player queues a build command targeting a map location
- Command accepted regardless of current visibility (no visibility check at command time)
- Worker pathfinds to location and validates on arrival: tiles must be Buildable, unoccupied, and meet faction-specific constraints
- If validation fails on arrival: command cancelled, worker stops and idles
- No visibility requirement on arrival (worker is physically present)

### Resource Types
- **SpaceCrystalsPatch**: 1x1, indestructible, unowned, no vision. Has RemainingAmount. Disappears when depleted. InfoPanel shows RemainingAmount when visible.
- **SupplyDeliveryStation**: 2x2, indestructible, unowned, no vision. Has DeliverySize, DeliveryInterval, CurrentSupplies. Delivery countdown begins when CurrentSupplies reaches 0. InfoPanel always shows DeliverySize/DeliveryInterval; shows CurrentSupplies when visible.

## Dependencies
- `simulation_core` (GridUnit for structure sizes, SpaceUnit for unit silhouettes)
