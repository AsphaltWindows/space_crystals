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

## Building
(user, 2026-11-26; Q0163) The Field Engineer builds **like an Agent builds a Tunnel** (factions/syndicate/units/agent.md > Building): it **goes inside** the site and disappears until the building is complete. **One Engineer per site; no help.**
1. The Engineer receives the order and walks to the site. **Payment**: the cost is deducted when the order is given (the order needs enough Space Crystals then) and refunded in full if the order ends before construction starts (another order, the Engineer killed, placement failing on arrival), as for Recruits (factions/cults/economy.md > Build Command).
2. **Construction begins**: the site appears with the Engineer inside it. The Engineer is **untargetable** and unseen meanwhile. The site follows the **ConstructionHP Rule** (starts at 10% HP; damage taken stays, as the Agent's Tunnel).
3. **Completed**: the Engineer **comes back out** (user: a; it doesn't garrison the Bunker by itself): through the **Bunker's door** (side B), or at the **edge of the Tent** (designer's placeholder: on the side it walked in from). It then idles.
4. **Site destroyed**: the Engineer survives and comes out at the **centre of the footprint**; the cost is lost.
5. **Cancelled** (X: Cancel construction, on the Bunker and the Tent): the Engineer comes out at the centre of the footprint; **75% of the cost is refunded, rounded down** (user: a; as Cults buildings): 75 of the Bunker's 100, 56 of the Tent's 75.
- While under construction the site does nothing but Cancel. It counts as a structure for the Defeat Condition.

The Field Engineer carries **no weapon**.

**Placeholder stats** (designer, 2026-11-26, user's standing permission; tune in the numbers pass):
- UnitBase LightInfantry; Silhouette 24x24 space units; MaxHP 50; PointArmor 0, FullArmor 0; SightRange 5 grid units; **Groupable false** (user, 2026-11-26; Q0163: only one Engineer can build a site, so, as the Agent, it is Ungroupable: Construct with one, Tab to the next); UnitControlCost 1.
- Movement as the Peacekeeper (MaxSpeed 4 space units/frame).
- Cost: Barracks, 75 Space Crystals, 160 frames (10 seconds).
- Panel (rearranged 2026-11-27 (user, 2026-11-27: A is Attack on every unit, systems/controls/interface.md > CommandPanel > Standard Slot Assignments)): **Q** Move; **W** Construct (a menu: **Q** Bunker, **W** First Aid Tent; **Z** back); **D** Repair (targeted: a damaged own building or vehicle); **X** Stop. Right-click a damaged own building or vehicle: Repair (Enter wins on a building it can enter, as for Recruits). Repair works through the same shared rules as the Recruit's.
