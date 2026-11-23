# cults_recruit_carry_capacity_reduction

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Reduced the Cults Recruit's Space Crystals carry capacity. Changed `artifacts/designer/design/cults_objects.md`.

## Change
- **Recruit CarryCapacity**: 4 -> 2 Space Crystals per load. MiningDuration (8 frames) and DropOffDuration (2 frames) are unchanged.

This halves the Space Crystals delivered per mining trip. Since MiningDuration is unchanged, a Recruit now needs twice as many mining/travel/drop-off cycles to deliver the same total amount of Space Crystals compared to before.

## QA Instructions

1. Order a Recruit to gather from a Space Crystal Patch. Confirm it mines for 8 frames then returns to Storage carrying exactly 2 Space Crystals (not 4).
2. Confirm drop-off at Storage still takes 2 frames and multiple Recruits can drop off simultaneously (unchanged).
3. Compare total Space Crystals/minute throughput for a single Recruit before and after this change — it should be roughly half, since the only change is per-load capacity, not mining/travel speed.
