# Robo Portal

The Colonists' Drone production structure. Produces Drones from a build queue, same standard production pattern as GDO's Barracks or Cults' Armory — a finished Drone "warps in" (materializes) at the Robo Portal rather than walking out from a fixed exit side, consistent with the Colonists' overall warping theme. Built via Warp In (see Warp In above). Standalone: not part of the portal upgrade line, and cannot be upgraded (see Portals).

## Entity Type - Structure Type
## Faction - Colonists
## Size - 4x4 grid units
## MaxHP - 600
## MaxShields - 20
## PointArmor - 1
## FullArmor - 16
## SightRange - 4 grid units
## Destructible - true
## Groupable - true
## SymmetryType - AAAA

Since the Robo Portal is fully symmetric (AAAA), units may leave in any direction (Portals > Portal Exits).

## RoboPortalInstanceState:
- RallyPoint: Coordinates | ObjectInstance | None
- BuildQueue: array of ObjectEnum (max 20, including the unit currently in production; user, 2026-11-24)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

## RallyPoint behavior:
When a unit (Drone or Scout) finishes production and spawns, if RallyPoint is set, it is issued its own default right-click command resolved against the RallyPoint target (systems/controls/selection_and_hud.md > What a rally point makes a new unit do). E.g. a Drone rallied on a patch Gathers it. If RallyPoint is None, the unit spawns with no command. If the RallyPoint references an ObjectInstance that no longer exists, the RallyPoint is reset to None. Rally points on enemy objects turn into ground rally points when the enemy leaves sight (systems/controls/selection_and_hud.md > Rally points on enemy objects). (Same pattern as GDO's Barracks.)

## Produces:
- Drone: 50 Space Crystals, 160 frames (10 seconds)
- Scout: cost/time TBD. Unit Prerequisite: player owns at least one Relay Array.

## Robo Portal ObjectInterfaceState
(user, 2026-11-24) Standard production panel, same conventions as GDO's Barracks:
```
[Q] Drone     [W] Scout   [E] —                  [R] —
[A] —         [S] —       [D] —                  [F] —
[Z] —         [X] Cancel Production   [C] Set Rally Point   [V] —
```
