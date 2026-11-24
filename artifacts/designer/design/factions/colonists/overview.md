# Colonists: Overview

## Colonists
The Colonists are a psionic faction with a deep refining economy. Buildings are warped in from abroad onto Beacons placed by Drones. Army size is limited by Beacon Capacity, which must be built up through Beacon placement. The Colonists have no building tech tree — any building can be Warped In at any time (the one exception is the portal upgrade line: the Heavy Portal and Star Portal are upgrade stages of a Bio Portal); progression is gated by the cost of advanced buildings in refined resources, which Drones must physically carry through a chain of refineries. Resources only enter the player's pool when delivered to an Egress Node.

### Value - Colonists
### Name - "Colonists"

### DisplayHud:
- Space Crystals: current amount
- Alloys: current amount
- Essence: current amount
- Flux: current amount
- Beacon Capacity: current used / current available (max 200)

## ColonistResources
The resources available to a Colonists player.

### Space Crystals
Core resource. Gathered from Space Crystals Patches by Drones. Spent on refining and basic construction.

### Alloys
Refined from Space Crystals at the Alloy Furnace (1 Space Crystal → 1 Alloy). A more advanced material necessary for construction of buildings and vehicles.

### Essence
Refined from Space Crystals at the Essence Refinery (1 Space Crystal → 1 Essence). Necessary for research and more advanced psychic abilities.

### Flux
Refined from Alloys and Essence at the Flux Reactor (1 Alloy + 1 Essence → 2 Flux). Necessary for advanced research and psionic weaponry.

### Beacon Capacity
Unit control resource. Each Beacon provides a certain amount of Beacon Capacity. Total Beacon Capacity caps at 200. Each Colonist unit has a Beacon Capacity cost. The player cannot build units if doing so would exceed the total available Beacon Capacity.

#### Reserved when production starts
(user, 2026-11-24; replaces "reserved when queued") Queuing a unit needs only its cost in resources (paid when queued) and a free queue slot; it does **not** check or reserve Beacon Capacity. When the unit reaches the front of the queue and production is about to start, it needs free Beacon Capacity (available minus used) at least equal to its BeaconCapacityCost; from then on it counts as used, through production and for as long as the unit lives. If there isn't enough, the building **waits** with the unit at the front (0% progress) and starts automatically once there is room; the player can still cancel it for a full refund. A blocked queue raises a notification (not yet designed). A unit already in production always finishes, even if capacity drops below used in the meantime. Several waiting buildings are served in a fixed internal order, and a unit with a capacity cost of 0 always passes, even over capacity (factions/gdo/overview.md > Unit Control). Same model as GDO Unit Control and Syndicate Tunnel Space.

#### Over capacity
(user, 2026-11-24) Used Beacon Capacity can end up above available, e.g. when a Colonists building is destroyed, or when a Beacon deployment is cancelled after its freed slot was spent (factions/colonists/economy.md > Beacon Deployment > Beacon Capacity during deployment). This is allowed: the counter shows in red, and no new unit starts production until there is room again (Reserved when production starts); units already in production finish. Nothing is killed or removed.
