# agent-resource-gathering

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

# agent-resource-gathering

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement Agent resource gathering and drop-off behaviors as defined in `artifacts/designer/design/syndicate_objects.md` under 'Agent > Gathering'.

**Space Crystals:**
- CarryCapacity: 50 Space Crystals per load
- MiningDuration: 48 frames (3 seconds)
- DropOffDuration: 48 frames (3 seconds)
- Drop-off side: Tunnel Side B

**Supplies:**
- CarryCapacity: 1 Supply per trip
- PickUpDuration: 48 frames (3 seconds)
- DropOffDuration: 48 frames (3 seconds)
- Drop-off side: Tunnel Side C

**Behaviors:**
- When gathering crystals: Agent walks to SpaceCrystalsPatch, performs mining for 48 frames, picks up 50 crystals (or remaining amount), then needs to return to a Tunnel.
- When gathering supplies: Agent walks to SupplyDeliveryStation, performs pickup for 48 frames, picks up 1 Supply, then needs to return to a Tunnel.
- When dropping off: Agent walks to the appropriate Tunnel side (B for crystals, C for supplies), performs drop-off for 48 frames, resources are added to the player's stockpile.
- Only one Agent may drop off at a given side at a time. Crystal (Side B) and Supply (Side C) drop-offs are on separate sides, so one crystal delivery and one supply delivery can occur simultaneously.

## QA Instructions

1. Place an Agent near a SpaceCrystalsPatch. Right-click the patch.
2. Verify the Agent walks to the patch and begins mining (visible animation/state for 48 frames / 3 seconds).
3. Verify the Agent picks up 50 crystals (or the remaining amount if the patch has fewer).
4. Order the Agent to drop off at an own Tunnel (right-click Tunnel while carrying).
5. Verify the Agent walks to Side B of the Tunnel and performs drop-off (48 frames / 3 seconds).
6. Verify the player's Space Crystal count increases by the carried amount.
7. Repeat with a SupplyDeliveryStation — verify Agent picks up 1 Supply, walks to Side C, drops off.
8. Send two Agents to drop off crystals at the same Tunnel simultaneously.
9. Verify only one Agent can be at Side B at a time — the second should wait.
10. Send one Agent to drop crystals (Side B) and another to drop supplies (Side C) simultaneously — verify both can proceed concurrently.
