# colonists_drone_assigned_egress_node_and_select_returning

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Added the AssignedEgressNode mechanic for Drones plus two new commands: Drone's "Set Drop-off Location" and Egress Node's "Select Returning Drones". Changed `artifacts/designer/design/colonists_objects.md`.

## New: AssignedEgressNode (Drone)
Each Drone now tracks an AssignedEgressNode — the specific Egress Node it delivers its current and future loads to (previously the design implied delivery to "an Egress Node" without specifying which one).

- **Auto-assignment**: the first time a Drone with no AssignedEgressNode is ordered to Gather, it automatically assigns the nearest Egress Node as its AssignedEgressNode.
- **Manual (re)assignment**: the player can explicitly set/change a Drone's AssignedEgressNode via the new Set Drop-off Location command (panel button **S**, AwaitingTarget[SetDropOffLocation] flow — left-click an Egress Node to confirm, standard CommandIssuingTransition/StateOnlyTransition pattern already used elsewhere for target commands). This does NOT interrupt whatever the Drone is currently doing (mining, walking, carrying) — it only updates AssignedEgressNode, which is read to determine the destination of the return trip. If a Drone is already walking back with cargo when reassigned, its return trip redirects toward the new node.
- Added "Set Drop-off Location" to the Drone's Unit Commands list and DefaultState commands (panel button S) in its ObjectInterfaceState.

## New: Egress Node "Select Returning Drones" command (panel button F)
An Immediate command (StateOnlyTransition — no Command issued to any game object, only modifies the ControlState's Selection, same category as other selection-only interface transitions). Replaces the current Selection with every Drone whose AssignedEgressNode is this Egress Node and which is currently executing a Gather command — this includes Drones that are mining (not yet carrying), walking to/from the patch, carrying resources back, or actively dropping off. Only available if at least one such Drone exists. This is the Egress Node's full ObjectInterfaceState (previously TBD).

## Still open (not in this change)
- Beacon's own ObjectInterfaceState (should expose Warp In as a command).
- Colonists combat units — none defined yet.
- Shield regen upgrades — still a placeholder for future discussion.

## QA Instructions

1. Produce/deploy a fresh Drone with no prior gather orders. Order it to Gather from a Space Crystal patch. Confirm that once it fills its CarryCapacity (8), it automatically walks to the nearest Egress Node to drop off, even though no drop-off location was ever explicitly set.
2. While that Drone is still mining (before it starts carrying), select it and use Set Drop-off Location (panel button S) to target a DIFFERENT Egress Node than the nearest one. Confirm the mining action is NOT interrupted — the Drone keeps mining uninterrupted — but once it finishes mining and starts its return trip, it walks to the newly-assigned Egress Node instead of the original nearest one.
3. Reassign a Drone's drop-off location WHILE it is already walking back carrying a full load. Confirm it immediately redirects toward the newly-assigned Egress Node instead of continuing to the old one.
4. Select an Egress Node and confirm the CommandPanel shows a "Select Returning Drones" command bound to panel button F.
5. With several Drones assigned to a given Egress Node in various states (some mining, some walking back with cargo, some actively dropping off, and at least one Drone assigned to a DIFFERENT Egress Node or idle/unassigned), select the Egress Node and trigger Select Returning Drones. Confirm the Selection updates to include exactly the Drones assigned to this Egress Node that are actively gathering (mining/walking/carrying/dropping off) — and excludes the Drone(s) assigned elsewhere or not currently gathering.
6. If no Drones are currently assigned/gathering toward a given Egress Node, confirm the Select Returning Drones command is unavailable/disabled (per the "only available if at least one such Drone exists" rule).
