# Bunker
GDO field defensive structure built by a Field Engineer. Infantry garrison inside and fire out.

## Entity Type - Structure Type
## Faction - GlobalDefenseOrganization
## Constructed by - Field Engineer (Build Bunker ability)
## BuildRadiusExtension - 0 (does not extend the GDO build area, so Field Engineers cannot use Bunkers to expand)

## Placement
Can be placed on **any Buildable terrain** — not limited to the GDO build area. Follows the Worker-Built Structures rules (core/entities.md > Placement Validation > Worker-Built Structures).

## Garrison
Works like the Cults Garrison (factions/cults/structures/garrison.md), except as noted:
- **Who can enter**: all GDO infantry, up to **4 units**, using the Enter command.
- Units inside **fire their own weapons** from the Bunker. (Unlike the Cults Garrison, the Bunker gives no units a substitute weapon — an unarmed unit stays unarmed.)
- **No bonus range** for units inside (the Cults Garrison gives Militants and Gunners +1).
- **Protection**: units inside cannot be targeted and take no damage while the Bunker stands.
- **Evacuation on destruction**: when the Bunker is destroyed, everyone inside comes out alive around its footprint.
- Units inside are not consumed and still count toward Unit Control.
- **No abilities from inside**: units inside the Bunker cannot use their abilities (e.g. a Drone Operator cannot launch drones, a Rocketman cannot use Target Lock, a Peacekeeper cannot raise its Riot Shield). They only fire their normal weapons.

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Size 2x2; SymmetryType AAAB with **one door on side B** (as the Cults Garrison); MaxHP 500; PointArmor 2, FullArmor 8; SightRange 6 grid units; Groupable true; Power 0.
- Cost: Field Engineer, 100 Space Crystals, 320 frames (20 seconds).
- Units inside see with the Bunker's SightRange (as the Garrison; to confirm). Panel: **A: Eject All** (everyone out at once through the door); **X: Cancel construction** while being built.
