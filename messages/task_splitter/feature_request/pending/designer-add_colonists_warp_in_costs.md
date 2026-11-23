# add_colonists_warp_in_costs

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Filled in the previously-missing Space Crystals cost and WarpInTime for the two Colonists buildings currently defined in `artifacts/designer/design/colonists_objects.md`. The Warp In mechanic (Beacon transforms in place into a target structure) already existed with full mechanics (footprint validation, HP/Shields interpolation, cancellation refund, etc.), but the actual per-building cost/time numbers on the Beacon's "Constructs List" had never been assigned — both buildings just said "Built via Warp In" with no numbers.

Added a Constructs List table under the `## Warp In` > `### Constructs List` section (same pattern as GDO's DeploymentCenter/ExtractionFacility `### Constructs:` lists):

| Building | Cost | WarpInTime |
|----------|------|------------|
| Egress Node | 75 Space Crystals | 320 frames (20 seconds) |
| Roboport | 400 Space Crystals | 640 frames (40 seconds) |

These are new resource costs and durations to be consumed by the Warp In transformation logic already implemented (or being implemented) for Colonists buildings — this is additive data, not a mechanic change.

Note: both the Egress Node and Roboport are also part of the Colonists' Starting Conditions (see `## Starting Conditions` in the same file) — each player starts with one pre-built copy of each. The Warp In cost/time above only applies when building additional copies during play; the starting pair should NOT be constructed via the Warp In flow or charged this cost at game start.

## QA Instructions

1. Start a Colonists game/scenario and confirm the pre-built Egress Node and Roboport are present at game start with no resource cost deducted (Starting Conditions, unaffected by this change).
2. Get an established Beacon (via Drone Deploy) and select it.
3. Trigger Warp In and select Egress Node from the Constructs list — confirm it costs 75 Space Crystals and the transformation takes 320 frames (20 seconds) to complete, during which HP/Shields interpolate per the existing Warp In interpolation formula.
4. Repeat for Roboport — confirm it costs 400 Space Crystals and takes 640 frames (40 seconds).
5. Confirm cancelling a Warp In part-way through refunds 75% of the respective cost (56 for Egress Node, 300 for Roboport, per the existing Cancel Warp In rule) and reverts the structure back to a normal Beacon.
