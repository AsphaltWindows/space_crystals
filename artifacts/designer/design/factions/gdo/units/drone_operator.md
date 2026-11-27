# Drone Operator
GDO specialist infantry, trained at the Barracks once the player owns a Career Campus. Carries explosive suicide drones, launches them one at a time and pilots them into enemies. (User reference: like the Reaver and its Scarabs.)

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Barracks
## TechPrerequisite - player owns at least one Career Campus (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)

## Drone Stock
- Starts with **1** drone when trained; holds a **maximum of 1** (changed from 3).
- **Build Drone** is one of the Drone Operator's abilities: once its drone has been used, it purchases/builds a replacement (placeholders below).
- **Not while a drone is out** (user, 2026-11-26; Q0155): Build Drone can start only once the operator's drone has **ended** (detonated, fallen or lost with the leash, battery). Each flight is followed by at least the build time with no drone, so drones stay bursts to time, not a constant presence.

## Launching and piloting
- The Drone Operator launches a drone from its stock. The drone is a separate flying unit **controlled directly by the player**.
- **Launching doesn't change the selection** (user, 2026-11-26; Q0155): the operator stays selected; the player clicks the drone (or uses its portrait) to fly it.
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
- Operator: trained at the Barracks on **R** (Q0155); LightInfantry 24x24; MaxHP 45; PointArmor 1, FullArmor 1; SightRange 6 grid units; MaxSpeed 4 su/frame; Groupable true; UnitControlCost 1. Cost: Barracks, 100 Space Crystals **+ 2 Supplies** (user, 2026-11-26; Q0169), 160 frames.
- **Build Drone** (hotkey **F**): 50 Space Crystals, 160 frames (10 s).
- **Launch Drone** (hotkey **R**).
- Drone (designer placeholders, 2026-11-26; Q0155): UnitBase **HoverCraft**; **Unit Control 0** (it is spent like ammunition); PointArmor 0, FullArmor 0; **Groupable true** (several drones can be flown as one group). Air unit 24x24; MaxHP 30; top speed 10 su/frame (DragMovement ForwardAcceleration 0.9, OmniDirectionalAcceleration 0.1, DragRatio 0.1, TurnRate 10°/frame, as the Supply Chopper; Q0154); SightRange 5; **leash range 8 grid units**, measured **edge to edge** like ranges (systems/combat.md > Range Measurement; Q0155); **battery 20 s** (320 frames).
- Explosion: **AoE** (user, 2026-11-27), Damage **150** total, AoERadius **48 su**, melee range.
