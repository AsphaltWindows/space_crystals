# agent-resource-gathering-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-agent-resource-gathering.md

## Task

Verify that Agent resource gathering and drop-off behaviors are fully implemented and working correctly. The implementation already exists — this is a verification-only task.

**What should already exist (verify all are present and correct):**

1. **GatheringResourceBehavior** component (behavior.rs) with GatherPhase enum: MovingToResource, Extracting, MovingToTunnel, DroppingOff
2. **DroppingOffResourcesBehavior** component (behavior.rs) with DropOffPhase enum: MovingToTunnel, DroppingOff
3. **AgentCarryState** component (types.rs) tracking crystals and supplies carried
4. **Constants** in unit_data.rs: AGENT_MINING_DURATION=48, AGENT_PICKUP_DURATION=48, AGENT_DROPOFF_DURATION=48, AGENT_CRYSTAL_CARRY=50, AGENT_SUPPLY_CARRY=1
5. **gathering_resource_behavior_system** (behaviors.rs) — full gather-deliver cycle with side occupancy checking
6. **dropping_off_resources_behavior_system** (behaviors.rs) — standalone drop-off with side occupancy checking
7. **Right-click integration** (core.rs) — Crystal patch and SDS right-click resolves to Gather command for Agents
8. **Side logic** — drop_off_side_for_carry: crystals→Side B, supplies→Side C
9. **Occupancy enforcement** — only one Agent per side at a time, crystal and supply deliveries can happen simultaneously
10. **Resource transfer** — resources added to SyndicatePlayerResources on drop-off completion

Run `cargo test` and verify all existing tests pass. If any piece is missing or broken, implement the fix. If everything passes, the task is complete.
