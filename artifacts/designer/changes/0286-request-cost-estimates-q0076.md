# 0286 — Request for cost estimates for the Agent jam (Q0076): no design change yet

- **Date**: 2026-11-26T09:10:00-05:00
- **Answers**: — (Q0076 is still open; this asks for input before the user decides)
- **Docs**: none changed

## What changed and why
**Nothing is decided yet.** The user is weighing fixes for the head-on jam in Q0076 and asks the Director **how expensive each would be** (implementation effort, simulation/performance cost, and risk to determinism or other behaviors).

What the docs say today, for reference:
- Only **idle** own units yield, with one short sidestep (systems/units.md > Ground Collision).
- Two moving units never yield. A blocked mover becomes "stuck" after 1.5 s, then re-routes after 0.5 / 1 / 2 s (systems/controls/behaviors.md > MovingToLocation > Failure).
- Every head-on meeting therefore costs at least about 2 s, and gathering Agents never give up.

**Options to estimate.** They can be combined.
1. **The mover sidesteps.**
   - Infantry: a moving unit blocked by any unit takes a short sideways step (e.g. always to its right, so two units meeting head-on pass each other), then tries again, falling back to the stuck rule after N tries.
   - Moving vehicles: instead of a step, they turn and try to **graze around** the obstacle.
2. **Idle yielding for infantry only.** The user notes that the current idle sidestep is realistic only for infantry; vehicles, especially wheeled ones, would struggle. What would it cost to restrict idle yielding to infantry, and does that interact with option 1?
3. **Quick re-route when the blocker is a unit**: a few frames instead of 1.5 s, with the stuck rule kept for real dead ends.
4. **A 2-square hatch**: the middle two squares of Side B. Assumption to be confirmed with the user: still one drop-off at a time, so the 12.5 SC/s ceiling is unchanged. Only the delivery area widens.
5. **Workers return to their last successful drop-off position**: each Agent, and possibly every worker, heads back to the spot it last delivered from, instead of the nearest point.
6. For comparison, **gathering Agents pass through each other** (Q0076 option A).

**Also please describe the M4-D test layout:** is the 1 GU strip walled on its sides (a row of patches, or other obstacles), or is it a single 1x1 patch open on both sides?

## Implementation impact
None yet. Please reply with a question file, e.g. a follow-up to Q0076, with rough costs per option and your measurements if cheap. The user will decide after that.

## Verification
—
