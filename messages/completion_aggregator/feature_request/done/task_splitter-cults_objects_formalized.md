# cults_objects_formalized

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

# cults_objects_formalized

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Created `artifacts/designer/design/cults_objects.md` — formal design document for the initial Cults faction objects and building mechanics.

**Cults Building Mechanics**: Recruits build structures and are consumed on completion. Cancelled buildings return all Recruits. Multiple Recruits speed up construction proportionally. Assist Construction command allows adding Recruits to in-progress buildings. Cannot selectively remove Recruits — only full cancellation.

**RecruitmentCenter** (4x4, AAAA): Starting structure for Cults. Has a 10x10 Recruitment Area. Claims Recruitable tiles first-built-first-served (no overlap). Effectiveness = claimed Recruitable tiles / total tiles in area. Base stats: 20 Unit Control capacity, 192 frames per Recruit — both scale linearly with effectiveness. Auto-produces free Recruits until local capacity reached. Unit Control tracks per-Recruit back to originating center, persisting through training. Includes RecruitmentCenterInstanceState, ObjectInterfaceState with rally point and cancel production commands, and destruction/reclaim rules.

**Storage** (3x2, ABAB): Drop-off point for Space Crystals collected by Recruits. Multiple simultaneous drop-offs allowed. Info display only, no commands.

Both structures use ConstructionHP Rule. Numeric stats (HP, armor, sight range, costs) are marked TBD.

## QA Instructions

1. Place a Recruitment Center on terrain with 100% Recruitable tiles in its area. Verify it produces one Recruit every 12 seconds up to 20 Unit Control capacity, then stops.
2. Place a Recruitment Center where ~50% of tiles in its area are Recruitable. Verify capacity is ~10 and production takes ~24 seconds per Recruit.
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
13. Verify Recruitment Center starts with ConstructionHP Rule (10% HP, gains HP during construction).
14. Verify Storage starts with ConstructionHP Rule.
