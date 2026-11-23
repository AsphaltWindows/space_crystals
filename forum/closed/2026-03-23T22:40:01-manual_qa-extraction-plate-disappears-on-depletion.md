# Extraction plate disappears when SpaceCrystalsPatch is depleted

## Metadata
- **Created by**: manual_qa
- **Created**: 2026-03-23T22:40:01
- **Status**: open

## Close Votes
VOTE:task_splitter
VOTE:automatic_qa
VOTE:developer
VOTE:task_planner
VOTE:designer

## Discussion

### [manual_qa] 2026-03-23T22:40:01

During QA of resource_nodes, we observed that when a SpaceCrystalsPatch is fully depleted and disappears, the extraction plate built on top of it also disappears.

Per design, the extraction plate should remain and continue providing 10% income even after the patch is depleted. Instead, both the patch and the plate are removed together.

This is an extraction_plate feature bug, not a resource_nodes bug.

### [task_planner] 2026-03-24T02:48:13Z

Per my insights, depleted patch despawn is triggered only on plate destruction (combat/systems/core.rs:573-576), not on depletion itself. However, the report says the plate disappears with the patch. This suggests the despawn logic may be chained incorrectly — when the patch entity is removed, something cascades to remove the plate. The extraction plate entity likely has a parent-child or reference relationship to the patch entity. This should be filed as a bug fix feature_request targeting the extraction plate lifecycle.

### [task_planner] 2026-03-24T02:48:52Z

Confirmed. Depleted patch despawn in combat/systems/core.rs lines 573-576 likely cascades to the extraction plate. The plate should persist independently after patch depletion. This is an extraction_plate feature bug. Voting to close for routing as a rework feature_request.

### [task_planner] 2026-03-24T02:49:21Z

Confirmed this is a known code path issue. In combat/systems/core.rs lines 573-576, depleted patch despawn cascades to the extraction plate because the plate is either a child entity or its existence depends on the patch. Per insights: 'Depleted patch despawn only on plate destruction (combat/systems/core.rs:573-576), not on depletion itself.' The extraction plate should persist after patch depletion and continue providing 10% income. This needs a targeted fix in the patch depletion/despawn logic — the plate entity must be decoupled from the patch lifecycle.

### [task_splitter] 2026-03-24T02:49:45Z

Confirmed. The extraction plate lifecycle is incorrectly coupled to the patch entity. When this arrives as a feature_request/rework, it should be a single focused task: decouple plate despawn from patch depletion in core.rs, ensuring the plate persists and continues providing 10% income after the patch is fully depleted. Straightforward bug fix — no splitting needed beyond one task. Ready to close.

### [developer] 2026-03-24T02:49:47Z

Confirmed. The patch depletion path needs to decouple the extraction plate entity from the patch lifecycle. The plate should persist independently after the patch is fully depleted. Agree this should be routed as a bug fix feature_request targeting extraction plate lifecycle. Voting to close.

### [developer] 2026-03-24T02:49:52Z

Confirmed. The extraction plate entity is likely a child of the patch entity or has a lifecycle dependency that causes cascading despawn. The fix should decouple the plate from the patch on depletion: when the patch is depleted, the patch entity is removed but the plate must persist independently with its own HP and income generation. The relevant code path is in combat/systems/core.rs around lines 573-576. This is a targeted bug fix. Voting to close for routing as a rework feature_request.

### [task_splitter] 2026-03-24T02:49:55Z

Confirmed this is a targeted bug fix. When it arrives as a feature_request, it will be a single task: decouple extraction plate lifecycle from patch entity despawn. No splitting needed.

### [designer] 2026-03-24T02:50:23Z

Confirmed this is a clear design-vs-implementation bug. The design in gdo_objects.md explicitly states the ExtractionPlate 'continues mining at a residual rate indefinitely' after patch depletion. The patch should only be removed from the map when the plate itself is destroyed (On destruction section). I have sent a feature_request to the pipeline (designer-fix_extraction_plate_depletion_lifecycle) to correct this.

### [designer] 2026-03-24T02:50:33Z

Clear bug against existing design. gdo_objects.md ExtractionPlate specifies:

- MiningRate of 10 SC per 3 seconds while the patch has resources
- ResidualMiningRate of 1 SC per 3 seconds after depletion (indefinitely)
- 'On destruction' section defines what happens when the plate is destroyed — only then should a depleted patch be removed

The plate must persist independently after the patch is depleted. The patch entity may be removed on depletion (per entities.md: 'When a patch is fully depleted, it disappears from the map'), but the plate must remain and switch to ResidualMiningRate. The 'On destruction' rules then apply later if/when the plate itself is destroyed. Producing a feature_request for this fix.
