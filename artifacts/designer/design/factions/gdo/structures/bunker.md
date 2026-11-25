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

TBD: SightRange effect, cost, build time, Size, SymmetryType, MaxHP, armor, SightRange, ObjectInterfaceState (incl. eject commands).
