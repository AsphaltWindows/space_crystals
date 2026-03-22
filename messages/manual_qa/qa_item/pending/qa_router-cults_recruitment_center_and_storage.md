# cults_recruitment_center_and_storage

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

## Content

# cults_recruitment_center_and_storage

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Created `artifacts/designer/design/cults_objects.md` with the initial Cults faction objects and building mechanics.

**Cults Building Mechanics**: Recruits build structures and are consumed on completion. Cancelled buildings return all Recruits. Multiple Recruits speed up construction proportionally. Assist Construction command allows adding Recruits to in-progress buildings. Cannot selectively remove Recruits — only full cancellation.

**Recruitment Center** (4x4, AAAA symmetry): Starting structure. Has a 10x10 Recruitment Area. Claims Recruitable tiles on a first-built basis (no overlap). Effectiveness = claimed Recruitable tiles / total tiles in area. Base stats: 20 Unit Control capacity, 12 seconds per Recruit — both scale linearly with effectiveness. Auto-produces free Recruits until local capacity is reached. Unit Control tracking traces each Recruit back to its originating center, persisting through training into other units. Trained units cost equals number of Recruits consumed, split across originating centers. HUD displays aggregated usage/capacity but production is entirely local.

**Storage** (3x2, ABAB symmetry): Drop-off point for Space Crystals collected by Recruits. Multiple Recruits can drop off simultaneously.

## QA Instructions

1. Place a Recruitment Center on terrain with 100% Recruitable tiles in its area. Verify it produces one Recruit every 12 seconds up to 20 Unit Control capacity, then stops.
2. Place a Recruitment Center where ~50% of tiles in its area are Recruitable (rest water or already claimed). Verify capacity is ~10 and production takes ~24 seconds per Recruit.
3. Build two Recruitment Centers with overlapping areas. Verify tiles are claimed by the first-built center only and the second center has reduced effectiveness.
4. Destroy the first center. Verify the second center reclaims the freed tiles and its effectiveness increases.
5. Train a Recruit into another unit. Verify the Unit Control usage tracks back to the originating center.
6. Train a unit requiring multiple Recruits from different centers. Verify each center's usage reflects the Recruits it contributed.
7. Destroy a trained unit. Verify usage decreases at the correct originating center(s).
8. Verify the HUD shows (total used) / (sum of capacities) across all centers.
9. Select Recruits and build a Storage. Verify Recruits walk to the location, enter the building, and are consumed on completion.
10. Cancel a building in progress. Verify all assigned Recruits are returned.
11. Use Assist Construction to add Recruits to an in-progress building. Verify construction speeds up.
12. Order Recruits to gather Space Crystals and drop off at Storage. Verify multiple Recruits can drop off simultaneously.
