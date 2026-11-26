# Bio Portal

The Colonists' basic military production structure. Produces basic combat units from a build queue, same standard production pattern as GDO's Barracks, Cults' Armory, or the Colonists' own Robo Portal. Built via Warp In (see Warp In above). First stage of the portal upgrade line: can be upgraded in place into a Heavy Portal, after which it no longer produces infantry (see Portals).

## Entity Type - Structure Type
## Faction - Colonists
## Size - 3x3 grid units
## MaxHP - 250
## MaxShields - 15
## PointArmor - 1
## FullArmor - 9
## SightRange - 3 grid units
## Destructible - true
## Groupable - true
## SymmetryType - AABB

## Exit Behavior
The Bio Portal's two open sides are its "A" sides (AABB: **neighbouring** sides, top and right in the default orientation; user, 2026-11-24). Units leave in any direction through them, toward the RallyPoint when possible (Portals > Portal Exits).

## BioPortalInstanceState:
- RallyPoint: Coordinates | ObjectInstance | None
- BuildQueue: array of ObjectEnum (max 20, including the unit currently in production; user, 2026-11-24)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

## RallyPoint behavior:
When a unit finishes production and spawns, if RallyPoint is set, it is issued the default right-click command resolved against the RallyPoint target (same resolution rules as the produced unit's own Right-Click Resolution). If RallyPoint is None, the unit spawns with no command. If the RallyPoint references an ObjectInstance that no longer exists, the RallyPoint is reset to None. Rally points on enemy objects turn into ground rally points when the enemy leaves sight (systems/controls/interface.md > CommandPanel > Rally points on enemy objects). (Same pattern as GDO's Barracks and the Colonists' own Robo Portal.)

## Produces:
- Sabre Legionnaire: 100 Space Crystals, 160 frames (10 seconds)
- Lancer: cost/time TBD. Unit Prerequisite: player owns at least one Exophysics Engine.
- Psionic unit (unnamed, not yet designed): cost/time TBD. Unit Prerequisite: player owns at least one Psionic Amplifier.

Rifle Legionnaire is not directly produced — it is reached only by converting an existing Sabre Legionnaire (see Weapon Conversion below).

## Bio Portal ObjectInterfaceState
(user, 2026-11-24) Standard production panel, same conventions as GDO's Barracks:
```
[Q] Sabre Legionnaire   [W] Lancer              [E] Psionic unit (TBD)   [R] —
[A] —                   [S] —                   [D] —                    [F] —
[Z] —                   [X] Cancel Production   [C] Set Rally Point      [V] —
```
