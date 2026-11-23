# cults_recruitmentcenter_and_storage_stats

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Filled in the remaining TBD building stats and construction costs for the Cults' RecruitmentCenter and Storage, completing the full stat set for all three core Cults structures (RecruitmentCenter, Storage, Armory). Changed `artifacts/designer/design/cults_objects.md`.

## RecruitmentCenter (4x4 footprint)
- MaxHP: 1000, PointArmor: 1, FullArmor: 16, SightRange: 6 grid units — matches the GDO DeploymentCenter's role as the central hub structure of its faction.
- Constructs list entry (on the Recruit, per the previous `cults_armory_stats_and_building_cost_restructure` change): 300 Space Crystals, 1440 frames (90 seconds) BaseConstructionTime with 1 Recruit assigned.

## Storage (3x2 footprint)
- MaxHP: 250, PointArmor: 1, FullArmor: 6, SightRange: 3 grid units. Note: Storage is the Cults' *only* Space Crystals drop-off point (there is no alternate location Recruits can deliver to), so while its footprint is small, losing it halts all Space Crystals income — its stats reflect that criticality (comparable to GDO's PowerPlant, another small, non-combat, economically-critical structure).
- Constructs list entry: 100 Space Crystals, 320 frames (20 seconds) BaseConstructionTime with 1 Recruit assigned.

Note on FullArmor: both values follow the informal pattern already present across every existing building in the docs (FullArmor = PointArmor × footprint area in grid units — e.g. Barracks 3x2=6 units × PointArmor 1 = FullArmor 6). This pattern is being applied consistently here but per the user's explicit direction it is NOT being written up as a formal rule in `entities.md` — treat it as coincidental consistency, not a documented constraint to enforce elsewhere.

This completes all previously-open TBD values for RecruitmentCenter, Storage, and Armory — no more TBD fields remain on any of the three Cults core structures.

## QA Instructions

1. Select a RecruitmentCenter in the InfoPanel/inspector. Confirm it shows MaxHP 1000, PointArmor 1, FullArmor 16, SightRange 6 grid units.
2. Select a Storage. Confirm it shows MaxHP 250, PointArmor 1, FullArmor 6, SightRange 3 grid units.
3. Select a Recruit and open its Construct menu. Confirm RecruitmentCenter costs 300 Space Crystals and Storage costs 100 Space Crystals; both options are greyed out/rejected if the player has insufficient Space Crystals.
4. Order a single Recruit to build a RecruitmentCenter alone. Confirm it takes 1440 frames (90 seconds) to reach full HP from placement (10% starting HP per the ConstructionHP Rule, ramping linearly).
5. Order a single Recruit to build a Storage alone. Confirm it takes 320 frames (20 seconds) to reach full HP from placement.
6. Confirm both structures scale construction time down linearly with additional Recruits assigned (same BaseConstructionTime scaling rule verified for the Armory in the prior feature_request — e.g. 2 Recruits should roughly halve the remaining time).
7. Destroy a player's only Storage mid-game. Confirm Recruits carrying Space Crystals have no valid drop-off location afterward (can't deliver crystals) until a new Storage is built.
