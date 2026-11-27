# Q0155 — The Drone Operator: its Barracks key, and the drone's details

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — GK, which registers the GDO's new kinds, registers the drone on point 2's placeholder. GD, the Drone Operator and its drone, builds the rest on the placeholders in Options.
- **Docs**: `factions/gdo/units/drone_operator.md` (`> Drone Stock`, `> Launching and piloting`, `> Drone limits — how a drone ends`, `> Placeholder stats`); `factions/gdo/structures/barracks.md > Produces`, `> ObjectInterfaceState[Barracks]`; `factions/gdo/overview.md > GDOResources > Unit Control`; `systems/combat.md > Range Measurement`; change 0366

## Question
1. Which Barracks key trains the Drone Operator?
2. What are the drone's UnitBase, Unit Control cost, armour and Groupable?
3. Can the operator buy its next drone while its drone is out?
4. Does launching a drone select it?
5. Is the 8-grid-unit leash measured centre to centre, or edge to edge as attack ranges are?

## Context
- **The Barracks' panel:** "Q: Build Peacekeeper", "W: Build Rocketman (hotkey tentative)", "E: Build Field Engineer (placeholder letter)", X Cancel Production and C Set Rally Point. The Drone Operator is listed under Produces, with no key.
  - 0366's hotkey placeholders: "R for deploy and launch abilities", "F for Return to Base and the buy or build abilities", and "the Factory panel: Q, W, E, R". None places the Drone Operator.
- **The drone:** "Air unit 24x24; MaxHP 30; MaxSpeed 10 su/frame; SightRange 5; leash range 8 grid units; battery 20 s (320 frames)". It is "a separate flying unit controlled directly by the player". No UnitBase, Unit Control cost, armour or Groupable is given.
  - "Each GDO unit has a Unit Control cost." The operator's is 1.
- **The stock:** "Starts with 1 drone when trained; holds a maximum of 1". Build Drone: "once its drone has been used, it purchases/builds a replacement", for 50 Space Crystals in 160 frames (10 s).
  - "No recall: once launched, a drone is spent." "Only one drone can be active at a time per operator."
- **Ranges** are measured edge to edge: "the shortest distance between the attacker's shape and the target's shape". The operator and the drone are both 24x24.

## Options
1. **(a) R** (the placeholder), the next key in the Barracks' top row. **(b)** Another free key.
2. **(a) A HoverCraft (with the HoverCraft question's numbers at MaxSpeed 10), Unit Control 0, PointArmor 0, FullArmor 0, Groupable** (the placeholder): it is spent like ammunition, and several drones can be flown as one group. **(b)** The same, but Unit Control 1 while it flies, so an operator with its drone out uses 2.
3. **(a) Yes** (the placeholder): the stock empties at launch, so a drone bought at once is ready after 10 s, before the 20 s battery runs out. **(b) No:** buying starts only once the drone has ended, so an operator gets at most one drone per flight plus 10 s.
4. **(a) No** (the placeholder): the selection doesn't change, and the player clicks the drone to fly it. **(b) Yes:** the drone replaces the operator in the selection, ready to fly.
5. **(a) Centre to centre** (the placeholder). **(b) Edge to edge,** as ranges are: the drone gets about 24 su (0.4 GU) farther along an axis, a little more on a diagonal.
