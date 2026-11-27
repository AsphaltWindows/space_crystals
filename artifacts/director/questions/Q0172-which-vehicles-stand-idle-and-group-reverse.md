# Q0172 — Which vehicles stand when idle, and where a group Reverse sends each unit

- **Date**: 2026-11-26T22:00:18-05:00
- **Blocking**: no. Proceeding on these assumptions:
  1. an idle TrackedVehicle, and a DrillUnit above ground, don't yield, as the WheeledVehicle doesn't. The HoverVehicle still yields.
  2. a Reverse order given to several units places them by the magic box, as a Move does.
- **Docs**:
  - `systems/units.md > UnitCollision > Ground Collision` ("Vehicles", "The contact step", "Idle units push apart");
  - `systems/controls/commands.md > Group Destinations (Magic Box)` and `> Reverse`;
  - `systems/controls/command_behaviors.md > ReversingToLocation`.

## Question
1. **Idle vehicles.** "An **idle vehicle doesn't yield** (it can't take a sidestep; it stands like a unit on Hold Position)." Which bases does "vehicle" cover? The candidates are the TrackedVehicle, a DrillUnit in its above-ground mode, and the HoverVehicle.
2. **A group Reverse.** When one Reverse order goes to several units at once, does each unit get its destination from the magic box, as with Move, Attack Move and Patrol? Or does every unit reverse to the clicked point?

## Context
- **Point 1** comes from the yielding rule's answer to Q0119 (0353).
  - Q0119 asked about the Jackal, now the Seeker. Slice M5-V2 has just built wheeled movement: an idle WheeledVehicle doesn't yield, and so doesn't take the nudge of "Idle units push apart" either ("the same units that yield").
  - The next slice, M5-V3, builds tracked movement, and needs the rule for the Hand of Heaven.
  - The reason in brackets fits a tank: treads can't move it sideways.
  - But the contact step gives TrackedVehicles, DrillUnits and HoverVehicles "a **90°** step to its heading (straight sideways)" when *they* are blocked while moving. So the sim can move them sideways, and the bracket's reason doesn't settle it.
  - The HoverVehicle (the Colonists' Drone) yields today, and the Drones' gathering and the computer players' games depend on that.
- **Point 2:**
  - The magic box names "Move, Attack Move, Patrol".
  - ReversingToLocation is "Identical to MovingToLocation except the plan uses Reversing locomotion".
  - M5-V2 placed a group Reverse by the magic box, as a Move.

## Options
**Point 1:**
- **(a) Every tracked base stands** (the assumption): the TrackedVehicle, and the DrillUnit above ground, don't yield and don't take the idle nudge. The HoverVehicle yields, since it can step any way. Players move an idle tank out of a lane themselves.
- **(b) Only the WheeledVehicle stands.** Tracked and hover units yield by a short sideways step, as infantry do. An idle tank in a base gets out of its own units' way.
- **(c) Every vehicle base stands, the HoverVehicle included.** This would change how idle Drones behave and the computer players' games. It contradicts nothing in the docs, but no hover unit can be moved aside any more.

**Point 2:**
- **(a) The magic box** (the assumption): a group backs up keeping its shape, or converges on a point inside the box.
- **(b) Every unit to the clicked point.** Vehicles reversing together would crowd onto one point.
