# First Aid Tent
GDO support structure set up by a Field Engineer. Heals nearby infantry.

## Entity Type - Structure Type
## Faction - GlobalDefenseOrganization
## Constructed by - Field Engineer (Set up First Aid Tent ability)
## BuildRadiusExtension - 0 (does not extend the GDO build area)

## Healing
Heals **nearby infantry** (no need to enter). **Permanent** — it stays until destroyed. Uses **Restoring HP, Healing area** (systems/mechanics/hp_restoration.md); parameters in Placeholder stats.

## Placement
Can be placed on **any Buildable terrain**, like the Bunker (Worker-Built Structures rules).

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Size 2x2; SymmetryType AAAA; MaxHP 200; PointArmor 1, FullArmor 4; SightRange 4 grid units; Groupable true; Power 0. Built by a Field Engineer inside the site (factions/gdo/units/field_engineer.md > Building; Q0163); **ConstructionHP Rule - applies**; while being built its panel has only **X: Cancel construction** (75% refund).
- Cost: Field Engineer, 75 Space Crystals, 240 frames (15 seconds).
- Heals **own and allied infantry** (user, 2026-11-27) whose centre is within **3 grid units**: **+2 HP per second**, continuously; several tents don't stack (the highest applies). **Only units on the map** (user, 2026-11-26; Q0164): infantry inside a building (e.g. a Bunker) are **not** healed; wounded infantry step out to heal. **Only once complete** (user, 2026-11-26; Q0164): a Tent under construction heals nobody, as every building works only once complete.
