# Bunker
GDO field defensive structure built by a Field Engineer. Infantry garrison inside and fire out.

## Entity Type - Structure Type
## Faction - GlobalDefenseOrganization
## Constructed by - Field Engineer (Build Bunker ability)
## BuildRadiusExtension - 0 (does not extend the GDO build area, so Field Engineers cannot use Bunkers to expand)

## Placement
Can be placed on **any Buildable terrain** — not limited to the GDO build area. Follows the Worker-Built Structures rules (core/entities.md > Placement Validation > Worker-Built Structures).

## Garrison
Uses **Garrisoning, Garrison structure** (systems/mechanics/garrisoning.md), with:
- **Capacity 4**. **Who**: all GDO infantry.
- **Weapons inside**: each fires **its own weapon** (no substitute: an unarmed unit stays unarmed). **No range bonus.**
- **Entry rule**: **a shielded Peacekeeper lowers its shield on entering** (user, 2026-11-26; Q0164): the normal 6-frame toggle, after which it fires from inside like any occupant; it comes out unshielded. (No abilities from inside, as every garrison: e.g. no drone launches, no Target Lock.)
- **Destroyed**: everyone inside comes out alive, all at once **inside its former footprint** (user, 2026-11-28; replaces "around its footprint"), as every garrison structure.

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Size 2x2; SymmetryType AAAB with **one door on side B** (as the Cults Garrison); MaxHP 500; PointArmor 2, FullArmor 8; SightRange 6 grid units; Groupable true; Power 0.
- Cost: Field Engineer, 100 Space Crystals, 320 frames (20 seconds).
- Units inside see with the Bunker's SightRange (as the Garrison; user, 2026-11-27). Panel: **A: Eject All** (everyone out at once through the door); **X: Cancel construction** while being built (75% refund; the Engineer comes out; factions/gdo/units/field_engineer.md > Building). **ConstructionHP Rule - applies** (Q0163).
