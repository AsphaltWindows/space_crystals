# GDO: Overview

## GlobalDefenseOrganization
The Global Defense Organization (GDO) is a military faction that builds and expands through a centralized Deployment Center system. GDO maintains a global power grid that affects building performance and fields armies within a fixed unit control cap.

### Value - GlobalDefenseOrganization
### Name - "Global Defense Organization" (working name — may change later)

### DisplayHud:
- Space Crystals: current amount
- Supplies: current amount
- Power: used / available (used = total consumed, available = total generated; e.g. 21/30 at the start, 51/30 in the all-in Barracks case). Shown in red when used exceeds available (negative power). Same "used / available" format as the other capacity counters (Unit Control, Syndicate Tunnel Space, Colonists Beacon Capacity) (user, 2026-11-24)
- Unit Control: current used / 200

## GDOResources
The resources available to a GDO player.

### Space Crystals
Core resource. Gathered from Space Crystals Patches via Extraction Facilities and Extraction Plates. Spent on buildings, units, and upgrades.

### Supplies
Tactical and tech resource. Gathered from Supply Delivery Stations via Supply Towers and Supply Choppers. Spent on advanced buildings, units, and research.

### Power
Flat capacity system. Each GDO building has a static Power value — positive for generators, negative for consumers. The player's total Power is the sum across all owned buildings. If total Power is negative, all power-consuming buildings and Extraction Plates operate slower in proportion to total available power versus total required power (affecting construction speed, unit production speed, and mining rate).

- **Speed ratio**: generated power ÷ consumed power, capped at 100% (e.g. 30 generated, 51 consumed → ≈59%, the opening baseline's all-in case). Applies to every building with negative Power and to Extraction Plates.
- **Power producers are never slowed** (user, 2026-11-24): the Deployment Center (+30) and Generators are not consumers, so the DC builds at full speed even at negative power; the player can always build their way out with Generators. Slowed: Extraction Facility (plate construction), Barracks and other consuming producers (production), plates (mining).
- **When a building's Power counts**: from the tick it is placed until the tick it is destroyed (GDO buildings are placed complete).

### Unit Control
Hard cap of 200, always fully available with no buildings required to unlock it. Each GDO unit has a Unit Control cost. The player cannot build units if doing so would exceed the cap.

**Reserved when production starts** (user, 2026-11-24; replaces "reserved when queued"): queuing a unit needs only its Space Crystals (paid when queued) and a free queue slot; it does **not** check or reserve Unit Control. When the unit reaches the front of the queue and production is about to start, it needs free Unit Control (200 minus used) at least equal to its cost; its Unit Control then counts as used from that moment, through production and for as long as the unit lives. If there isn't enough, the building **waits** with the unit at the front (0% progress) and starts automatically once there is room; the player can still cancel it for a full refund. A blocked queue raises a notification (notification system not yet designed). A unit already in production always finishes. **Several buildings waiting** (user, 2026-11-24): when room opens for fewer units than are waiting, buildings are served in a **fixed internal order** (e.g. lowest id, effectively oldest building first); the player has no control over it. Applies to every faction's capacity. **No power, no start** (follows from "reserved when the building starts making progress"; 2026-11-24): a building that can't make progress (e.g. GDO at 0 power generated) doesn't take the room; another building may use it. **Zero-cost units always pass** (user, 2026-11-24): a unit whose capacity cost is 0 never waits for capacity, even when used is over available. The HUD's "used" count includes in-production units, not merely queued ones. Same model for every faction's capacity (Syndicate Tunnel Space: factions/syndicate/tunnels.md > Production and Tunnel Space; Colonists Beacon Capacity: factions/colonists/overview.md > Beacon Capacity).
