# Feature Update: Entity System - ConstructionHP Rule

**Feature file**: `features/entity_system.md`
**Design sources**: `design/entities.md`

## Modifications

### ConstructionHP Rule Added to Structure Type
- New opt-in rule for structures built on-site
- HP during construction = MaxHP x (10% + 90% x construction_progress)
- Structures start at 10% MaxHP, gain HP linearly during construction
- Partially-built structures can be attacked and destroyed
- Currently referenced by Syndicate Tunnel (Agent construction flow)
