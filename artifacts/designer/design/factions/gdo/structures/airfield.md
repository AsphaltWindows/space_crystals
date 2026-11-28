# Airfield
GDO aircraft production structure. Constructs aircraft and provides a landing/reloading zone for them.

## Faction - GlobalDefenseOrganization
## Constructed by - DeploymentCenter
## TechPrerequisite - player owns at least one Advanced Mechanics

## Produces (so far):
- Hornet: 150 Space Crystals, 320 frames (20 seconds) (placeholder).
- Dragonfly: 175 Space Crystals, 320 frames (20 seconds) (placeholder). Requires: player owns at least one Propulsion Labs.

## Aircraft Support
GDO aircraft use **Aircraft Landing and Rearming** (systems/mechanics/aircraft_landing.md): **Vertical** landing on the Airfield's slots and the **Assigned base** rearming archetype, with the Airfield as the base. The Airfield's parameters:
- **Limited ammo**: GDO aircraft carry limited ammo and rearm at their assigned Airfield (ammo and RearmTimePerAmmo per aircraft: factions/gdo/units/README.md).
- **Capacity: each Airfield supports at most 4 aircraft** (user, 2026-11-23): 4 assignments and **4 landing slots**, so all of its aircraft can be landed at once. The production limit follows (assignments plus queued aircraft at most 4; e.g. 3 assigned + 1 queued = full).
- **Slot layout** (designer placeholder, 2026-11-28; Q0189): a 2x2 grid, one slot near each corner of the footprint.
- **Return to Base**: hotkey **F** (placeholder); right-clicking the own assigned Airfield does the same; right-clicking another own Airfield reassigns.
- **Landing and take-off: 16 frames (1 s)** each (placeholder). The Dragonfly (a Glider) lands vertically on its slot (aircraft_landing.md > Vertical).
- **Airfield destroyed**: its aircraft become unassigned, and aircraft landed on it are destroyed with it (aircraft_landing.md > Landed aircraft).

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Size 3x3; SymmetryType AAAA; MaxHP 500; PointArmor 1, FullArmor 9; SightRange 5 grid units; Groupable true; Power -30; BuildRadiusExtension 2 grid units.
- Panel: **Q** Hornet, **W** Dragonfly (greyed without Propulsion Labs), **X** Cancel Production, **C** Set Rally Point.
