# Feature Update: Unit Commands and Behaviors - Agent Commands

**Feature file**: `features/unit_commands_and_behaviors.md`
**Design sources**: `design/syndicate_objects.md`
**Design update**: `design_updates/2026-03-06_agent_control_panel.md`

## Modifications

### New commands added to command table
- **Gather**: CommandType=Gather, TargetObject=Resource source, Agent only
- **DropOffResources**: CommandType=DropOffResources, TargetObject=Own Tunnel, Agent when carrying resources
- **BuildTunnel**: CommandType=BuildTunnel, TargetLocation=location, Agent via AwaitingPlacement

### New behaviors added
- **GatheringResource**: Move to resource source, mine/pickup, auto-deliver to nearest Tunnel (correct side based on resource type), drop off
- **DroppingOffResources**: Move to target Tunnel's appropriate side (B for crystals, C for supplies), perform drop-off
- **BuildingTunnel**: Move to build location, embed in partially-built Tunnel, construct for 480 frames. Single-Agent only. See syndicate_objects for full flow.

Total commands: 12 (was 9). Total behaviors: 13 (was 10).
