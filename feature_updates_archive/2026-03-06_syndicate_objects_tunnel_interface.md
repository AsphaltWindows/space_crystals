# Feature Update: Syndicate Objects - Tunnel Interface & Construction Flow

**Feature file**: `features/syndicate_objects.md`
**Design sources**: `design/syndicate_objects.md`, `design/entities.md`

## Modifications

### Tunnel Groupable Changed
- Tunnel changed from **Groupable: true** to **Groupable: false** (Ungroupable)
- Each Tunnel instance is always its own SelectionGroup, enabling per-Tunnel ObjectInterfaceState

### Tunnel ObjectInterfaceState Added
Full command interface for selected Tunnel:
- **DefaultState**: 3 commands (A: Upgrade Tunnel, B: Expand Tunnel, C: Eject)
- **EjectMenu**: Grid of unit type tiles showing all Tunnel Network units with counts. Tier-incompatible types greyed out. Eject queues units at 8 frames minimum between ejections.
- **ExpandMenu**: Shows available expansion types for Tunnel's tier. Blocked if Tunnel is already performing an operation.
- **AwaitingPlacement (Expansion)**: Ghost preview within Tunnel Area with rotation/flip controls. Same placement mechanics as surface buildings.

### Agent Construction Flow Added
Detailed construction sequence for Tunnel building:
- Agent embeds inside Tunnel during construction (untargetable)
- Tunnel starts at 10% HP (ConstructionHP Rule), gains HP linearly
- On completion: Agent enters Tunnel Network
- On destruction: Agent survives and emerges at location
