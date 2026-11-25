# Field Engineer
GDO specialist infantry, trained at the Barracks once the player owns a Career Campus.

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Barracks
## TechPrerequisite - player owns at least one Career Campus (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)

## Abilities
- **Repair**: repairs buildings and vehicles. Repair costs Space Crystals (not free). **Repair rules shared by every repairer** (user, 2026-11-26): several repairers on one target repair **faster, scaling linearly** (2 repairers = twice the rate), as construction does. **Placeholder rate and cost** (designer, 2026-11-26): repairing from 0 to full HP takes **half the target's build time** with one repairer, and costs **half its price** in the resource it was bought with, paid gradually as HP is restored.
- **Build Bunker**: constructs a Bunker (see factions/gdo/structures/bunker.md). Costs Space Crystals.
- **Set up First Aid Tent**: sets up a First Aid Tent (see factions/gdo/structures/first_aid_tent.md). Costs Space Crystals.

The Field Engineer carries **no weapon**.

TBD: stats, cost, build time, hotkeys, ObjectInterfaceState.
