# agent_resource_gathering_r1

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# agent_resource_gathering_r1

## Metadata
- **From**: manual_qa
- **To**: task_splitter

## Content

Agent resource gathering cannot be tested. Right-clicking a crystal field does not issue a Gather command (nothing happens). This blocks the entire gathering/drop-off flow.

**What failed:**
- Right-click on SpaceCrystalsPatch does not trigger Gather — Agent does nothing. This was confirmed during agent_interface QA as well.

**Blocked by above:**
- All 10 QA steps — mining, pickup, drop-off, carry capacity, side routing, concurrent drop-off rules

Note: This overlaps with the agent_interface_r1 rework (right-click Gather broken). Fixing the right-click resolution for crystal fields should unblock this feature.

## QA Instructions

1. Right-click a SpaceCrystalsPatch with an Agent — verify the Agent walks to the patch and begins mining.
2. After mining completes, verify the Agent carries 50 crystals. Right-click an own Tunnel — verify the Agent walks to Side B and drops off, increasing the player's Space Crystal count.
3. Test with SupplyDeliveryStation — verify Agent picks up 1 Supply and drops off at Side C.
4. Send two Agents to drop crystals at the same Tunnel — verify only one can use Side B at a time.
