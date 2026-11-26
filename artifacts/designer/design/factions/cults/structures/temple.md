# Temple

Tier 2 of the Mystical/Human Enhancement branch. Researches technologies for the Cults. Research follows the shared rules in core/tech_and_research.md > Research, including the Cults exception: research only progresses while Recruits are staffing the building (speed scales with the number of Recruits inside).

## Faction - TheCults
## Constructed by - Recruit
## TechPrerequisite - player owns at least one Altar

Size, armor and other stats: see Placeholder stats below. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

## Research (first set)

Each Augmentation has its own two-level improvement line. Each level raises the effect of that Augmentation for **every** unit that has it, including units already augmented (retroactive; see Augmentation > Research applies to everyone).

| Technology | Requires | Effect | Cost / Time |
|---|---|---|---|
| Enhanced Constitution Level 1 | — | Enhanced Constitution becomes +4 Constitution / −2 Metabolism | 100 SC / 960 frames (placeholder) |
| Enhanced Constitution Level 2 | Enhanced Constitution Level 1; player owns at least one Cathedral | Enhanced Constitution becomes +6 Constitution / −3 Metabolism | 150 SC / 1440 frames (placeholder) |
| Improved Metabolism Level 1 | — | Improved Metabolism becomes +4 Metabolism / −2 Constitution | 100 SC / 960 frames (placeholder) |
| Improved Metabolism Level 2 | Improved Metabolism Level 1; player owns at least one Cathedral | Improved Metabolism becomes +6 Metabolism / −3 Constitution | 150 SC / 1440 frames (placeholder) |

Level 2 of both lines is researched at the Temple, but also requires the player to own a Cathedral (counting rule as in core/tech_and_research.md > Tech Prerequisites).

Further Temple research sets: TBD.

## Recruit Staffing
Recruits enter the Temple to staff its research (right-click own Temple with Recruits selected → Enter Temple). Only Recruits can staff research. See core/tech_and_research.md > Research > Faction Exceptions > Cults for the staffing rules (not consumed, linear speed scaling, pauses when empty).

**Placeholder stats** (designer, 2026-11-26, at the user's request: provisional numbers so the Director can implement; tune in the numbers pass):
- Size 3x3; SymmetryType ABAC, **EntranceSide B, ExitSide C** (like the Armory; user rule 2026-11-26, Q0116: research buildings have an entrance and an exit, and staff leave one at a time); MaxHP 450; PointArmor 1, FullArmor 9; SightRange 4 grid units; Groupable true.
- Cost and time: Recruit > Constructs (200 SC, 640 frames).
- Recruit staffing capacity 6; research queue 5.
- **Levels** (user, 2026-11-26; Q0117): a Level 2 can't be queued until its Level 1 has **completed** (its key shows "In progress" meanwhile). Level 2's Cathedral prerequisite is checked only when it is queued; losing the Cathedral later doesn't cancel it.
- Research (BaseResearchTime = with 1 Recruit): Enhanced Constitution L1 and Improved Metabolism L1: **100 SC, 960 frames (60 s)** each; L2 of each: **150 SC, 1440 frames (90 s)**.
- Panel: **Q** Enhanced Constitution (next level), **W** Improved Metabolism (next level), **A/S/D/F** release one Recruit of a kind (README > Choosing Recruits, Release and Eject All), **V: Eject All** (staff leave), **X: Cancel** (last research). Recruits enter by right-click.
