# Q0181 — SeparationRadius for the five air units whose docs give none

- **Date**: 2026-11-27T06:36:21-05:00
- **Blocking**: no. M6-GK and M6-SK register the radii of option (a) below, marked `Placeholder (Q0181)`.
- **Docs**:
  - `systems/units.md > Air units`: "SeparationRadius — per unit type, circular, the distance at which air-to-air repulsion activates. Must be larger than the unit's Silhouette";
  - `factions/gdo/units/hornet.md`, `factions/gdo/units/supply_chopper.md`, `factions/gdo/units/drone_operator.md` (the drone);
  - `factions/syndicate/units/phantom.md`, `factions/syndicate/units/prism.md`.

## Question
What SeparationRadius do the Hornet, the Supply Chopper, the Drone Operator's drone, the Phantom and the Prism have?

## Context
Every air unit type has a SeparationRadius, larger than its Silhouette. Three docs give one, and each is the Silhouette's side plus 8 space units:
- the Sparrow (40x40) and the Dragonfly (40x40): 48;
- the Specter (56x56): 64.

The other five air units' docs give none. All five are HoverCraft, whose repulsion is three times a Glider's (Q0152). The three given radii are all Gliders'.

## Options
- **(a) The Silhouette's side + 8 su,** as the three given radii (the assumption):

  | Unit | Silhouette | SeparationRadius |
  |---|---|---|
  | Hornet | 48x48 | 56 |
  | Supply Chopper | 60x60 | 68 |
  | Drone (the operator's) | 24x24 | 32 |
  | Phantom | 44x44 | 52 |
  | Prism | 40x40 | 48 |

- **(b) Other numbers,** per unit. With stronger repulsion than a Glider's, HoverCraft might spread enough with a smaller margin, or the user might want them farther apart.
