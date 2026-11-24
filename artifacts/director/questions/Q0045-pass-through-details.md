# Q0045: Pass-through details: idle units left stacked, and the Drop Off order

- **Date**: 2026-11-24T10:39:00-05:00
- **Blocking**: no. M3-H2 is built with the current behaviour listed under each point.
- **Docs**: `systems/units.md > UnitCollision` (Units that already overlap); `factions/colonists/units.md > Gathering Drones pass through each other`; change 0106.

## Question
1. **Two units left overlapping and idle.** For example, a gathering Drone is told to Stop while it passes through another, or an order ends while it overlaps a unit.
   - **Current behaviour:** they stay stacked until one of them is ordered to move. That's 0106's "nothing is pushed apart", read literally. On screen, two units sit on top of each other for good, and a click picks only the top one.
2. **Is an explicit Drop Off order part of the gathering job?**
   - **Current behaviour:** yes. A Drone sent to deliver with Drop Off passes through other gathering Drones, as on its normal trip back.

## Context
Both came up while building 0106 (M3-H2). The rule as written covers Drones on the Gather job, and units that already overlap until they separate. It doesn't say what happens when overlapping units are idle, or whether Drop Off counts.

## Options
- **Point 1:**
  - A, as now: they stay stacked until ordered.
  - B, idle overlapping units step apart on their own: a short automatic move the player didn't order.
  - C, only Drones whose gathering job ends while overlapping move off.
- **Point 2:**
  - A, as now: Drop Off counts as part of the job.
  - B, only Gather counts: a Drone on a Drop Off order collides with gathering Drones.
