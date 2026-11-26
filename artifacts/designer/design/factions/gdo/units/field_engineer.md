# Field Engineer
GDO specialist infantry, trained at the Barracks once the player owns a Career Campus.

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Barracks
## TechPrerequisite - player owns at least one Career Campus (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)

## Abilities
- **Repair**: repairs buildings and vehicles. Repair costs Space Crystals (not free). **Repair rules shared by every repairer** (user, 2026-11-26): several repairers on one target repair **faster, scaling linearly** (2 repairers = twice the rate), as construction does. **Placeholder rate and cost** (designer, 2026-11-26): repairing from 0 to full HP takes **half the target's build time** with one repairer, and costs **half its price** in the resource it was bought with, paid gradually as HP is restored. **When repair ends** (user, 2026-11-26; Q0124): repairers **stop once the target is at full HP**; if the player **runs out of Space Crystals** mid-repair, the order **ends** (they stop; a free Tunnel repair never runs out). **Gliders can't be repaired** (they can't stop; user, 2026-11-26).
- **Build Bunker**: constructs a Bunker (see factions/gdo/structures/bunker.md). Costs Space Crystals.
- **Set up First Aid Tent**: sets up a First Aid Tent (see factions/gdo/structures/first_aid_tent.md). Costs Space Crystals.

The Field Engineer carries **no weapon**.

**Placeholder stats** (designer, 2026-11-26, user's standing permission; tune in the numbers pass):
- UnitBase LightInfantry; Silhouette 24x24 space units; MaxHP 50; PointArmor 0, FullArmor 0; SightRange 5 grid units; Groupable true; UnitControlCost 1.
- Movement as the Peacekeeper (MaxSpeed 4 space units/frame).
- Cost: Barracks, 75 Space Crystals, 160 frames (10 seconds).
- Panel (rearranged 2026-11-27 (user, 2026-11-27: A is Attack on every unit, systems/controls/interface.md > CommandPanel > Standard Slot Assignments)): **Q** Move; **W** Construct (a menu: **Q** Bunker, **W** First Aid Tent, both greyed until designed; **Z** back); **D** Repair (targeted: a damaged own building or vehicle); **X** Stop. Right-click a damaged own building or vehicle: Repair (Enter wins on a building it can enter, as for Recruits). Repair works through the same shared rules as the Recruit's.
