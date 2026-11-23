# cults_armory_stats_and_building_cost_restructure

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Filled in the Armory's building stats, and restructured how Cults building construction costs/times are represented, aligning them with the pattern GDO already uses (cost/time lives with the builder, not the building). Changed `artifacts/designer/design/cults_objects.md`.

## Armory building stats filled in
The Armory's previously-TBD stats now match the GDO Barracks (same role: a production/training structure of the same 3x2 footprint):
- MaxHP: 300, PointArmor: 1, FullArmor: 6, SightRange: 4 grid units

## New: Space Crystals cost + construction time moved off individual buildings, onto the Recruit's Constructs list
Previously each Cults building (RecruitmentCenter, Storage, Armory) had its own `SpaceCrystalsCost` field (mostly TBD). This is now removed from the building blocks themselves. Instead, a new **Constructs:** list was added under the Recruit's "Building" section (`artifacts/designer/design/cults_objects.md`, Recruit unit definition), listing every buildable Cults structure with its cost and construction time — directly mirroring how GDO's DeploymentCenter has a `Constructs:` list for Power Plant/Barracks/etc. rather than each GDO building carrying its own cost field.

New Recruit > Constructs list:
- **RecruitmentCenter**: TBD Space Crystals, TBD BaseConstructionTime — still open
- **Storage**: TBD Space Crystals, TBD BaseConstructionTime — still open
- **Armory**: 200 Space Crystals, 480 frames (30 seconds) BaseConstructionTime (with 1 Recruit assigned) — resolved

## New: BaseConstructionTime Scaling rule (Cults Building Mechanics section)
Because Cults buildings can have any number of Recruits assigned to speed up construction (unlike GDO's fixed build time at the DeploymentCenter), each buildable structure's construction time is now defined as a **BaseConstructionTime**: the time to build with exactly 1 Recruit. Effective construction time scales down linearly with more Recruits assigned — 2 Recruits halve it, 3 Recruits reduce it to a third, no cap other than the practical limit of Recruits that can be assigned to a site. This formalizes the existing "construction speed scales proportionally with the number of Recruits currently assigned" rule with a concrete baseline number to scale from.

## RecruitmentCenter is buildable
Clarified that the RecruitmentCenter is not exclusively a one-time starting structure — Recruits can build additional ones like any other Cults structure (cost/time TBD, see Constructs list above). This was implicit in the existing TileClaiming rules (which already described competing/multiple Recruitment Centers) but is now explicit.

## Still open (not in this change)
- RecruitmentCenter and Storage building stats (MaxHP/PointArmor/FullArmor/SightRange) are still TBD.
- RecruitmentCenter and Storage costs/construction times in the new Constructs list are still TBD.

## QA Instructions

1. Select an Armory in the InfoPanel/inspector (or wherever building stats surface in-game) and confirm it shows MaxHP 300, PointArmor 1, FullArmor 6, SightRange 4 grid units.
2. Select a Recruit and open its Construct menu (option A). Confirm the Armory entry shows a cost of 200 Space Crystals and highlight/reject the option if the player has fewer than 200 Space Crystals.
3. Order a single Recruit to build an Armory alone. Confirm it takes 480 frames (30 seconds) to complete from placement to full HP (per the ConstructionHP Rule: 10% HP at start, ramping linearly to 100%).
4. Order a second Recruit to Assist Construction partway through an Armory build with only 1 Recruit assigned. Confirm the remaining time drops roughly in half from that point once 2 Recruits are working (linear scaling — verify a 3rd Recruit reduces it to roughly a third of the 1-Recruit baseline for the remaining work).
5. Confirm the Armory building itself no longer displays/uses a standalone "SpaceCrystalsCost" stat separate from the Recruit's Constructs list — cost lives only in the Construct menu now, consistent with how GDO's Barracks has no cost field of its own (only the DeploymentCenter's Constructs list shows Barracks cost).
6. Confirm a Recruit's Construct menu lists RecruitmentCenter and Storage as buildable options (costs currently unset/placeholder — flag if the game requires a concrete number to be selectable, since design still has these as TBD).
