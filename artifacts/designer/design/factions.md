# Factions

## GlobalDefenseOrdinance
The Global Defense Ordinance (GDO) is a military faction that builds and expands through a centralized Deployment Center system. GDO maintains a global power grid that affects building performance and fields armies within a fixed unit control cap.

### Value - GlobalDefenseOrdinance
### Name - "Global Defense Ordinance"

### DisplayHud:
- Space Crystals: current amount
- Supplies: current amount
- Power: current / total (current = total generated - total consumed)
- Unit Control: current used / 200

## GDOResources
The resources available to a GDO player.

### Space Crystals
Core resource. Gathered from Space Crystals Patches via Extraction Facilities and Extraction Plates. Spent on buildings, units, and upgrades.

### Supplies
Tactical and tech resource. Gathered from Supply Delivery Stations via Supply Towers and Supply Choppers. Spent on advanced buildings, units, and research.

### Power
Flat capacity system. Each GDO building has a static Power value — positive for generators, negative for consumers. The player's total Power is the sum across all owned buildings. If total Power is negative, all power-consuming buildings and Extraction Plates operate slower in proportion to total available power versus total required power (affecting construction speed, unit production speed, and mining rate).

### Unit Control
Hard cap of 200, always fully available with no buildings required to unlock it. Each GDO unit has a Unit Control cost. The player cannot build units if doing so would exceed the cap.

## TheSyndicate
The Syndicate is a covert faction that builds underground Tunnel networks and uses Agents as its worker units. Army size is limited by Tunnel Space, which must be built up through Tunnel construction and upgrades.

### Value - TheSyndicate
### Name - "The Syndicate"

### DisplayHud:
- Space Crystals: current amount
- Supplies: current amount
- Tunnel Space: current used / current available (max 200)

## SyndicateResources
The resources available to a Syndicate player.

### Space Crystals
Core resource. Gathered from Space Crystals Patches by Agents. Spent on buildings, units, and upgrades.

### Supplies
Resource necessary for expanding and teching. Gathered from Supply Delivery Stations by Agents. Spent on Tunnel construction, Tunnel upgrades, and research.

### Tunnel Space
Unit control resource. Each Tunnel provides Tunnel Space based on its upgrade level. Total Tunnel Space caps at 200. Each Syndicate unit has a Tunnel Space cost. The player cannot build units if doing so would exceed the total available Tunnel Space.

## TheCults
The Cults are a faction that spreads across the map through Recruitment Centers, drawing free Recruits from Recruitable terrain. Recruits serve as both workers and the base unit that is trained into all other Cult units. Army size scales with territorial control of Recruitable tiles, with no hard cap.

### Value - TheCults
### Name - "The Cults"

### DisplayHud:
- Space Crystals: current amount
- Unit Control: current used / current available

## CultsResources
The resources available to a Cults player.

### Space Crystals
Core resource. Gathered from Space Crystals Patches by Recruits. Spent on buildings, units, and upgrades.

### Unit Control
Provided by Recruitment Centers in proportion to the number of Recruitable tiles each center is actively recruiting from. No hard cap — army size is bounded only by territorial control of Recruitable land. Each Cult unit has a Unit Control cost. The player cannot build units if doing so would exceed the total available Unit Control.

## Colonists
The Colonists are a psionic faction with a deep refining economy. Buildings are warped in from abroad onto Beacons placed by Prospectors. Army size is limited by Beacon Capacity, which must be built up through Beacon placement.

### Value - Colonists
### Name - "Colonists"

### DisplayHud:
- Space Crystals: current amount
- Alloys: current amount
- Essence: current amount
- Conduits: current amount
- Beacon Capacity: current used / current available (max 200)

## ColonistResources
The resources available to a Colonists player.

### Space Crystals
Core resource. Gathered from Space Crystals Patches by Prospectors. Spent on refining and basic construction.

### Alloys
Refined from Space Crystals. A more advanced material necessary for construction of buildings and vehicles.

### Essence
Refined from Space Crystals. Necessary for research and more advanced psychic abilities.

### Conduits
Refined from Alloys and Essence. Necessary for advanced research and psionic weaponry.

### Beacon Capacity
Unit control resource. Each Beacon provides a certain amount of Beacon Capacity. Total Beacon Capacity caps at 200. Each Colonist unit has a Beacon Capacity cost. The player cannot build units if doing so would exceed the total available Beacon Capacity.
