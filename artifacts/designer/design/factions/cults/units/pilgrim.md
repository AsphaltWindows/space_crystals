# Pilgrim

(Named by the user, 2026-11-27; "Ark" was the other candidate, kept in mind for the airship.)

(user, 2026-11-27) The Cults' **paratrooper plane**: a big, unarmed **glider** built at the Airstrip once the player owns a Scrap Hangar. It **loads up to 16 infantry while landed on an Airstrip** and **drops them by parachute** anywhere on the map.

## Entity Type - Unit
## Faction - TheCults
## UnitBase - Glider (GliderMovement, as the Sparrow)
## Produced at - Airstrip (consumes **2 Recruits** (user, 2026-11-27); Unit Control cost 2, one per Recruit)
## TechPrerequisite - player owns at least one Scrap Hangar (factions/cults/structures/scrap_hangar.md)
## AttackAttributes - None (unarmed)

## On the runway
Uses **Aircraft Landing and Rearming, Runway** archetype (systems/mechanics/aircraft_landing.md) on an own Airstrip (no rearming: it is unarmed). Its parameters and exceptions:
- **A new Pilgrim appears landed on the runway** (user, 2026-11-27) and **stays there until it is given an order**. Unlike the Sparrow, it doesn't take off by itself. While it is landed, the Airstrip's production is blocked (below).
- **Takeoff**: **any order** (a Move, a Drop, etc.) makes it **take off along the runway** toward its C end, as the Sparrow does, and then carry the order out. It is an air target during the run (as the Sparrow).
- **Landing** (user, 2026-11-27): **right-click an own Airstrip** with the Pilgrim selected: it flies to **that** Airstrip and lands on its runway (lining up and touching down along it, toward C; about **2 s**, 32 frames, placeholder; an air target meanwhile). There is no general Land command: the player picks the Airstrip, since with several Airstrips it matters where it will be loaded and repaired. It **never lands anywhere else**. An Airstrip holds **one landed Pilgrim** at a time; a Pilgrim sent to an occupied runway circles over it until the runway is free (designer's reading).
- **Landing while the Airstrip is producing** (user, 2026-11-27): it **lands at once**. Production carries on, but a finished aircraft **waits at 100%** until the Pilgrim leaves the runway, as when a building's exit is blocked (core/entities.md > Exit Doorway).
- **Landed**: it is a **ground target** (as landed GDO aircraft and the landed Supply Chopper). **Recruits can repair it** while it is landed (user, 2026-11-27; factions/cults/units/recruit.md > Repair).
- **After a drop it doesn't come back by itself** (user, 2026-11-27): it circles where it is (as a glider's Stop) until given an order; the player may not want it to land while that Airstrip is busy producing.

## Carrying
Uses **Carrying, Docked carrier + Paradrop** (systems/mechanics/carrying.md), with:
- **Capacity: 16 infantry** (user, 2026-11-27). **Who**: every Cults infantry type (Recruits, Militants, Gunners, Disciples) and **Disciples' illusions** (factions/cults/units/disciple.md > Mirage). Each takes one place.
- **Docked on**: an own **Airstrip** (On the runway, above). Input: **Enter** (right-click the landed Pilgrim with infantry selected; they walk over to it).
- **Unload All** (while landed; user, 2026-11-27): along the **side of the runway** (placeholder: the long side to the left of the takeoff direction).
- **Drop** (user, 2026-11-27): targeted, anywhere on the map; one passenger every **2 frames** (placeholder; 16 in about 2 s); each falls for **3 s** (48 frames, placeholder) as an **air target** (user, 2026-11-27: anti-air can shoot them).
- **Destroyed**: everyone aboard dies (user, 2026-11-27), as every carrier.

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette **96x96 su** (fits the 128 su wide runway); MaxHP **600**; PointArmor 2, FullArmor 2; SightRange **8 grid units**; Groupable true; SeparationRadius **104 su** (Silhouette side + 8).
- GliderMovement: IdleSpeed 3 su/frame; MaxSpeed **7 su/frame** (Sparrow: 10); Acceleration 0.3; Deceleration 0.3; MaxCentripetalAcceleration 0.25 su/frame² (tightest turn about 196 su at full speed: wider-turning than the Sparrow relative to its speed). Idle circle radius 2 grid units.
- Cost: Airstrip, **2 Recruits** (user), **300 Space Crystals, 960 frames (60 s)**; hotkey **W** at the Airstrip (Q is the Sparrow).
- Drop: one passenger per 2 frames; fall 48 frames. Landing 32 frames.
- Panel: Q Move, X Stop, E HoldPosition (circles), **R: Drop** (targeted), **F: Unload All** (only while landed). Right-click: an own Airstrip → land on it; elsewhere → Move.
