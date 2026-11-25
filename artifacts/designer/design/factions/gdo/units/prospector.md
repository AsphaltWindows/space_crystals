# Prospector
GDO expansion vehicle produced at the Factory once the player owns Advanced Mechanics. A tracked vehicle that deploys into an **Outpost** (see factions/gdo/structures/outpost.md), which extends the GDO build area so buildings can be placed far from the main base.

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Factory
## TechPrerequisite - player owns at least one Advanced Mechanics (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)
## UnitBase - TrackedVehicle

## Deploy into Outpost (ability)
- Can deploy on **any Buildable terrain** (not limited to the existing GDO build area). Placement follows the Worker-Built Structures rules (core/entities.md > Placement Validation > Worker-Built Structures): the order can target a location, the Prospector drives there, and placement is validated on arrival.
- Deploying is **permanent** — an Outpost cannot pack back up into a Prospector.

TBD: weapon (assumed none), deploy time, whether deployment can be cancelled mid-way, stats, cost, build time, Unit Control cost, hotkeys, ObjectInterfaceState.
