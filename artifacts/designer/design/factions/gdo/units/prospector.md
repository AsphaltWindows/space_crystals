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

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 64x48 space units; MaxHP 300; PointArmor 2, FullArmor 2; SightRange 6 grid units; Groupable true; UnitControlCost 2.
- SpeedTurnRadiusMovement: MaxSpeed 4 su/frame; Acceleration 0.4; Deceleration 1; SpeedToTurnRadiusRatio 8.
- **No weapon** (as assumed). **Deploy 64 frames (4 s)**; it follows the vehicle deploy model: an order that **can't be cancelled** once started (the Artillery Rig's rules; user, 2026-11-27).
- Cost: Factory, 300 Space Crystals, 480 frames.
- Panel: Q Move, X Stop, **R: Deploy into Outpost** (targeted: a location; it drives there and deploys).
