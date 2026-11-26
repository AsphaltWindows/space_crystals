# Drone Operator
GDO specialist infantry, trained at the Barracks once the player owns a Career Campus. Carries explosive suicide drones, launches them one at a time and pilots them into enemies. (User reference: like the Reaver and its Scarabs.)

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Barracks
## TechPrerequisite - player owns at least one Career Campus (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)

## Drone Stock
- Starts with **1** drone when trained; holds a **maximum of 1** (changed from 3).
- **Build Drone** is one of the Drone Operator's abilities: once its drone has been used, it purchases/builds a replacement (placeholders below).

## Launching and piloting
- The Drone Operator launches a drone from its stock. The drone is a separate flying unit **controlled directly by the player**.
- Only **one drone can be active at a time** per operator.
- The Drone Operator **cannot move** while it has a drone active.
- The drone **can be shot down** — but only by attacks that can hit air (Air or Universal TargetDomain). It is an Air-domain unit.
- **Detonation:** the drone behaves like a combat unit with a **melee-range attack that it can use only once** — the attack is its explosion. It takes normal Attack orders and attacks like any melee unit; using its attack destroys it.
- **Explosion targets ground units only** (TargetDomain: Ground) — the drone cannot attack air units.
- **No recall:** once launched, a drone is spent. It cannot return to the operator's stock.
- The Drone Operator has **no weapon** of its own; its only offense is its drones.

## Drone limits — how a drone ends
| Event | Drone falls | Explodes |
|---|---|---|
| Detonates on enemies (its purpose) | — | Yes |
| Leaves the **leash range** around its operator | Yes | **Yes** (falls and explodes) |
| Runs out of **battery** (time limit) | Yes | No |
| Its **operator dies** | Yes | No |

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Operator: LightInfantry 24x24; MaxHP 45; PointArmor 1, FullArmor 1; SightRange 6 grid units; MaxSpeed 4 su/frame; Groupable true; UnitControlCost 1. Cost: Barracks, 100 Space Crystals, 160 frames.
- **Build Drone** (hotkey **F**): 50 Space Crystals, 160 frames (10 s).
- **Launch Drone** (hotkey **R**).
- Drone: Air unit 24x24; MaxHP 30; MaxSpeed 10 su/frame; SightRange 5; **leash range 8 grid units**; **battery 20 s** (320 frames).
- Explosion: **AoE** (designer's choice, to confirm), Damage **150** total, AoERadius **48 su**, melee range.
