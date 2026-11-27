# 0453 — A braking crusher still crushes; a crush gives no minimap alert

- **Date**: 2026-11-27T12:55:00-05:00
- **Answers**: Q0178
- **Docs**:
  - `systems/units.md > UnitCollision > Crushing` (Trigger; Stationary crushers do not overlap)
  - `systems/controls/hud.md > Minimap > Alerts` (When)

## What changed and why
1. **Option (b).** A crusher crushes while it has **any speed**: Moving, Reversing or **Stopping**. A tank braking for its goal, or coasting after Stop, rolls over enemy infantry in its path instead of halting against them. It becomes a hard obstacle only once it is Stationary and not turning. Turning in place still crushes (0418).
2. **Option (a).** A crush gives **no** minimap alert, even off screen. A crush isn't damage, and the alert fires on damage only.

## Implementation impact
- **Changed:** the Stopping state now triggers crushing (M5-V4 built "doesn't crush").
- **Unchanged:** the alert stays damage-only, as built.

## Verification
1. Order a Hand of Heaven to stop on a spot covered by enemy Peacekeepers, or press Stop just before it reaches them. While braking it crushes the Peacekeepers it overlaps.
2. Once at rest, it blocks them like any obstacle.
3. Crush an enemy-owned infantry unit while its owner's camera is elsewhere. The owner gets no ping.
