# First Aid Tent
GDO support structure set up by a Field Engineer. Heals nearby infantry.

## Entity Type - Structure Type
## Faction - GlobalDefenseOrganization
## Constructed by - Field Engineer (Set up First Aid Tent ability)
## BuildRadiusExtension - 0 (does not extend the GDO build area)

## Healing
Heals **nearby infantry** (no need to enter). **Permanent** — it stays until destroyed.

## Placement
Can be placed on **any Buildable terrain**, like the Bunker (Worker-Built Structures rules).

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Size 2x2; SymmetryType AAAA; MaxHP 200; PointArmor 1, FullArmor 4; SightRange 4 grid units; Groupable true; Power 0.
- Cost: Field Engineer, 75 Space Crystals, 240 frames (15 seconds).
- Heals **own GDO infantry** (to confirm) whose centre is within **3 grid units**: **+2 HP per second**, continuously; several tents don't stack (the highest applies).
