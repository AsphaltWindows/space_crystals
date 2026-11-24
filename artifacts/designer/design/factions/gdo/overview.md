# GDO: Overview

## GlobalDefenseOrganization
The Global Defense Organization (GDO) is a military faction that builds and expands through a centralized Deployment Center system. GDO maintains a global power grid that affects building performance and fields armies within a fixed unit control cap.

### Value - GlobalDefenseOrganization
### Name - "Global Defense Organization" (working name — may change later)

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

**Reserved when queued** (user, 2026-11-24): a unit takes its Unit Control the moment it is **queued** (as its Space Crystals are paid then), not when production starts. Queuing is refused if used + the unit's cost would exceed 200. The HUD's "used" count includes queued and in-production units. Cancelling a queued unit (or its refund on the producer's destruction) gives its Unit Control back. So a GDO queue is never blocked by the cap, and reinforcements can't be pre-queued beyond 200. (This differs deliberately from the Syndicate, whose Tunnel Space is checked when production starts: factions/syndicate/tunnels.md > Production and Tunnel Space.)
