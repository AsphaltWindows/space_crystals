# Subterranean Transport
(user, 2026-11-25; name tentative, the user picks) Syndicate **infantry transport** that travels **underground**. It carries **infantry only**.

## Entity Type - Unit
## Faction - TheSyndicate
## UnitBase - DrillUnit (confirmed, Q0071; exits only through Tier 2+ Tunnels like other vehicles)
## Produced at - Vehicle Assembly
## TechPrerequisite - player owns at least one **Forge** (user, 2026-11-25)
## AttackAttributes - none (confirmed, Q0071)

## Above-ground mode
- When built, it **drives out of the Tunnel in its above-ground mode**, behaving as a tracked vehicle.
- In this mode, **infantry can be loaded** into it, and it can **unload** them.

## Burrow (ability)
Makes it **go underground**. Underground it can travel **under buildings and obstacles such as cliffs**: across **all drillable terrain**.

## Underground: hidden unless detected
(user, 2026-11-25; the rule for every DrillUnit, systems/units.md > DrillUnit) While underground it is **hidden** from other players (its owner sees it). It becomes visible, and **targetable**, only while **revealed by a detector**. Undetected, it can't be **targeted**, but **area (AoE) damage still affects it** (user, 2026-11-25; confirmed Q0071): a blast landing over it damages it like any unit in the area, e.g. a Grenadier's Attack Ground on a spot where it is suspected. Detection itself is not designed yet (detectors: which units/structures, range; to be designed, likely together with stealth such as the Colonists Scout's).

## Underground movement
(user, 2026-11-25; Q0071) A burrowed transport **passes through every underground structure** (expansions of any player, a retracted pop-up Laser Turret, GDO mines), as underground units already pass through each other (systems/units.md > Underground Collision). Nothing hidden can block it, so moving never reveals anything.

## Surface (ability)
Brings it back to its above-ground mode. **It can't surface on top of a building, or on ground that tracked vehicles can't traverse** (user, 2026-11-25). The check is the tile itself (it must be passable for a tracked vehicle and free of buildings), **not** whether a tracked vehicle could drive there: so it **can** surface on a plateau or behind cliffs that ground vehicles can't reach. It **can surface above an expansion** (units stand over expansions anyway). The general surfacing rules also apply (systems/units.md > Underground Collision > Surfacing, > Reserved while surfacing): it can't surface where it would overlap an above-ground unit, and the spot is reserved once surfacing starts (user, 2026-11-25; Q0071). Once surfaced it can unload and load again.

## Burrowing and surfacing work like vehicle deploy
(user, 2026-11-25) Burrow and Surface follow the **vehicle deploy model** (factions/gdo/units/artillery_rig.md > Modes): each takes time, is a **queueable order**, **can't be cancelled** once started, and the unit changes mode only when the process completes. The **last order given meanwhile is held** and tried first once it's done; orders the unit can't carry out in its new mode are dropped.

## Entering the Tunnel Network loaded
(user, 2026-11-25) It **can drive into a Tunnel with infantry inside**; they travel along inside it and stay aboard. Inside the network the passengers are **still locked inside the transport**: they only come out when the transport unloads (on the surface), and can't be ejected from a Tunnel on their own. **They always keep counting against the Tunnel Space cap** (user, 2026-11-25; Q0071: Tunnel Space is an army limit, factions/syndicate/tunnel_area_and_space.md > Tunnel Space > Production and Tunnel Space), aboard or not, inside the network or on the surface.

## Destroyed with passengers
(user, 2026-11-25) If it is destroyed, **all infantry inside die** with it (above ground or underground).

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 72x48 space units; MaxHP 300; PointArmor 3, FullArmor 3; SightRange 5 grid units; TunnelSpaceCost 3.
- Above ground as a TrackedVehicle: MaxSpeed 4.5 su/frame; Acceleration 0.4; Deceleration 1; TrackWidth 36 su; ReverseMaxSpeed 2.25 (designer placeholders, 2026-11-26; Q0153: the Laser Battery's handling; spins about 14.3° a frame); underground MaxSpeed 3.5.
- **Burrow and Surface: 48 frames** each.
- **Capacity 4** infantry (user, 2026-11-27: **4 heavy infantry, e.g. 4 Guards**; Syndicate infantry are Heavy Infantry).
- **Loading**: right-click the transport with infantry selected (Enter); it must be above ground. **Unloading**: **F: Unload All** (above ground only; they come out around it).
- Panel: Q Move, X Stop, **R: Burrow / Surface**, **F: Unload All**.
- Cost: Vehicle Assembly, 200 Space Crystals, 400 frames.
