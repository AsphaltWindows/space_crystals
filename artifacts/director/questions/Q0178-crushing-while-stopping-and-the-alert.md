# Q0178 — Crushing: does a braking crusher crush, and does a crush ping the minimap?

- **Date**: 2026-11-27T02:51:47-05:00
- **Blocking**: no. M5-V4 built both literally: a Stopping crusher doesn't crush, and a crush pings nothing. It stays so until the answer, and the playtest notes say so.
- **Docs**:
  - `systems/units.md > UnitCollision > Crushing` (Trigger; Stationary crushers do not overlap);
  - `systems/combat.md > LocomotionOrientationConstraints[SpeedTurnRadiusMovement]` (the Locomotion states: Stationary, Moving, Stopping, Reversing);
  - `systems/controls/hud.md > Alerts`;
  - design changes 0418 and 0443.

## Question
1. **Does a crusher crush while it is Stopping,** braking or coasting with speed left? The Trigger names only "Locomotion = Moving or Reversing".
2. **Does a crush ping the minimap,** when the crushed unit is off screen? The alert's rule is a unit that "takes damage while it is off screen", and a crush is "instant death of the crushed unit".

## Context
M5-V4 built crushing for the Hand of Heaven, the first crusher. Two edges are unclear.

**1. Stopping.** The Locomotion states are Stationary, Moving, Stopping and Reversing.
- The Trigger lists Moving and Reversing. "Stationary crushers do not overlap" covers a crusher that is "not moving".
- A Stopping crusher still has speed, so neither rule clearly covers it.
- **As built (literal):** a Stopping crusher doesn't crush, so enemy Light Infantry are hard obstacles to it.
- **What the player sees:** a Hand of Heaven braking for its goal, or coasting after Stop, halts at contact with an enemy Peacekeeper in its braking path, instead of rolling over it. A tank's braking takes a few ticks, so this is a narrow case, but it is the case where a tank is sent onto a clump of infantry.

**2. The alert.**
- **As built,** the alert fires on damage only. A crush isn't damage in the sim: the unit is removed at once, with its own event.
- **What the player sees:** infantry crushed off screen gives no ping, while infantry shot off screen does.

## Options
**1. Stopping:**
- **(a) Literal** (as built): only Moving and Reversing crush. A braking crusher is stopped by enemy infantry.
- **(b) Any speed crushes:** a crusher crushes while it has any speed (Moving, Reversing or Stopping), and becomes a hard obstacle only once Stationary. A spin at rest crushes as 0418 says, unchanged.

**2. The alert:**
- **(a) No ping** (as built).
- **(b) A crush pings like damage,** at the crushed unit's spot, when it is off screen. This is a small client change.
