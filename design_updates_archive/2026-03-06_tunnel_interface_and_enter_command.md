# Design Update: Tunnel Interface, Enter Command & Construction Mechanics

**Date**: 2026-03-06
**Files modified**: `design/syndicate_objects.md`, `design/control_system.md`, `design/entities.md`

## Changes

### Tunnel Interface (syndicate_objects.md)

- Tunnel changed to **Ungroupable** (was Groupable)
- Full ObjectInterfaceState added with three DefaultState options:
  - **A: Upgrade Tunnel** — upgrade to next tier
  - **B: Expand Tunnel** — multi-stage: select expansion type, then place within Tunnel Area
  - **C: Eject** — multi-stage: unit type grid showing all network units with counts, click to eject one from Side A
- Eject queue: new unit begins ejecting every 8 frames minimum, actual throughput limited by unit speed and collision
- Unit types incompatible with Tunnel tier are visible but greyed out in Eject menu

### Enter Command (control_system.md)

- New unit command: **Enter** — unit walks to Tunnel Side A and enters the Tunnel Network
- Only available when target Tunnel's tier is sufficient for the unit's base category
- Right-click resolution updated: own Tunnel with sufficient tier resolves to Enter instead of Move

### Agent Construction Flow (syndicate_objects.md)

- Agent embeds inside the Tunnel during construction (untargetable)
- Tunnel starts at 10% HP (ConstructionHP Rule), gains HP linearly
- On completion: Agent is in the Tunnel Network
- On destruction: Agent survives and emerges at location

### ConstructionHP Rule (entities.md)

- New opt-in rule for structures built on-site
- HP during construction = MaxHP x (10% + 90% x construction_progress)
- Partially-built structures can be attacked and destroyed
